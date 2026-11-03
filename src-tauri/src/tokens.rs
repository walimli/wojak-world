/// Filename tokens and phrases. These feed search. They are not a taxonomy.
#[derive(Debug, Clone)]
pub struct Derived {
    pub stem: String,
    pub tokens: Vec<String>,
    pub categories: Vec<String>,
    pub token_blob: String,
}

pub fn derive(basename: &str, folder: &str) -> Derived {
    let stem_source = strip_image_extension(basename);
    let stem = normalize_words(&split_camel(stem_source));
    let tokens = unique_words(&stem);

    let mut categories = Vec::new();
    if let Some(parent) = std::path::Path::new(folder).file_name() {
        let folder_words = normalize_words(&split_camel(&parent.to_string_lossy()));
        if !folder_words.is_empty() {
            categories.push(folder_words);
        }
    }
    if tokens.len() >= 2 {
        categories.push(stem.clone());
        for pair in tokens.windows(2) {
            categories.push(format!("{} {}", pair[0], pair[1]));
        }
    }
    let categories = dedupe(categories);

    let mut blob_parts = Vec::new();
    if !tokens.is_empty() {
        blob_parts.push(tokens.join(" "));
    }
    blob_parts.extend(categories.iter().cloned());
    let token_blob = blob_parts.join(" ");

    Derived {
        stem,
        tokens,
        categories,
        token_blob,
    }
}

/// Turn a search box string into an FTS5 query.
/// `All` means the box was empty. `None` means no term survived.
/// `Err` is a short message for the UI, never SQL or FTS text.
#[derive(Debug)]
pub enum FtsQuery {
    All,
    None,
    Match(String),
}

pub fn fts_query(raw: &str) -> Result<FtsQuery, String> {
    if raw.trim().is_empty() {
        return Ok(FtsQuery::All);
    }
    let tokens = lex(raw)?;
    if tokens.is_empty() {
        return Ok(FtsQuery::None);
    }
    let mut parser = Parser { tokens, index: 0 };
    let expr = parser.parse_or()?;
    if let Some(message) = parser.leftover_error() {
        return Err(message);
    }
    Ok(FtsQuery::Match(emit(&expr)?))
}

/// Words, operators, quotes, and parentheses. The MATCH string is built only from these.
#[derive(Clone)]
enum Tok {
    Word(String),
    Phrase(Vec<String>),
    And,
    Or,
    Not,
    LParen,
    RParen,
}

enum Expr {
    Word(String),
    Phrase(Vec<String>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>, Box<Expr>),
}

struct Parser {
    tokens: Vec<Tok>,
    index: usize,
}

impl Parser {
    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while self.eat(TokKind::Or) {
            let right = self.parse_and()?;
            left = Expr::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_not()?;
        loop {
            if self.eat(TokKind::And) || self.starts_term() {
                let right = self.parse_not()?;
                left = Expr::And(Box::new(left), Box::new(right));
                continue;
            }
            break;
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_primary()?;
        while self.eat(TokKind::Not) {
            let right = self.parse_primary()?;
            left = Expr::Not(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.peek().cloned() {
            Some(Tok::LParen) => {
                self.index += 1;
                let inner = self.parse_or()?;
                if !matches!(self.peek(), Some(Tok::RParen)) {
                    return Err("The search has an unclosed parenthesis.".into());
                }
                self.index += 1;
                Ok(inner)
            }
            Some(Tok::Word(word)) => {
                self.index += 1;
                Ok(Expr::Word(word))
            }
            Some(Tok::Phrase(words)) => {
                self.index += 1;
                Ok(Expr::Phrase(words))
            }
            Some(Tok::RParen) => Err("The search is missing a word or phrase.".into()),
            Some(Tok::Not) => Err("Put a word or phrase before NOT.".into()),
            Some(Tok::And) | Some(Tok::Or) => {
                Err("The search has an operator in the wrong place.".into())
            }
            None => Err("The search is missing a word or phrase.".into()),
        }
    }

    fn starts_term(&self) -> bool {
        matches!(
            self.peek(),
            Some(Tok::Word(_)) | Some(Tok::Phrase(_)) | Some(Tok::LParen)
        )
    }

    fn eat(&mut self, kind: TokKind) -> bool {
        let matched = match (kind, self.peek()) {
            (TokKind::And, Some(Tok::And)) => true,
            (TokKind::Or, Some(Tok::Or)) => true,
            (TokKind::Not, Some(Tok::Not)) => true,
            _ => false,
        };
        if matched {
            self.index += 1;
        }
        matched
    }

    fn peek(&self) -> Option<&Tok> {
        self.tokens.get(self.index)
    }

    fn leftover_error(&self) -> Option<String> {
        match self.peek() {
            None => None,
            Some(Tok::RParen) => Some("The search has an extra parenthesis.".into()),
            Some(_) => Some("The search could not be read.".into()),
        }
    }
}

#[derive(Clone, Copy)]
enum TokKind {
    And,
    Or,
    Not,
}

fn lex(raw: &str) -> Result<Vec<Tok>, String> {
    let chars: Vec<char> = raw.chars().collect();
    let mut index = 0;
    let mut tokens = Vec::new();
    while index < chars.len() {
        let ch = chars[index];
        if ch.is_whitespace() {
            index += 1;
            continue;
        }
        if ch == '(' {
            tokens.push(Tok::LParen);
            index += 1;
            continue;
        }
        if ch == ')' {
            tokens.push(Tok::RParen);
            index += 1;
            continue;
        }
        if ch == '"' {
            index += 1;
            let start = index;
            while index < chars.len() && chars[index] != '"' {
                index += 1;
            }
            if index >= chars.len() {
                return Err("The search has an unclosed quote.".into());
            }
            let text: String = chars[start..index].iter().collect();
            index += 1;
            let words = phrase_words(&text);
            if words.is_empty() {
                return Err("The search phrase is empty.".into());
            }
            tokens.push(Tok::Phrase(words));
            continue;
        }
        let start = index;
        while index < chars.len() {
            let ch = chars[index];
            if ch.is_whitespace() || ch == '(' || ch == ')' || ch == '"' {
                break;
            }
            index += 1;
        }
        let lexeme: String = chars[start..index].iter().collect();
        if is_operator(&lexeme, "and") {
            tokens.push(Tok::And);
        } else if is_operator(&lexeme, "or") {
            tokens.push(Tok::Or);
        } else if is_operator(&lexeme, "not") {
            tokens.push(Tok::Not);
        } else {
            for word in bare_words(&lexeme) {
                tokens.push(Tok::Word(word));
            }
        }
    }
    Ok(tokens)
}

fn is_operator(lexeme: &str, keyword: &str) -> bool {
    lexeme.eq_ignore_ascii_case(keyword)
}

fn phrase_words(input: &str) -> Vec<String> {
    alnum_words(input)
}

fn bare_words(input: &str) -> Vec<String> {
    alnum_words(input)
        .into_iter()
        .filter(|word| !is_dropped_letter(word))
        .collect()
}

fn alnum_words(input: &str) -> Vec<String> {
    normalize_words(input)
        .split_whitespace()
        .filter(|word| !word.is_empty())
        .map(str::to_string)
        .collect()
}

fn is_dropped_letter(word: &str) -> bool {
    word.chars().count() == 1 && !word.chars().all(|ch| ch.is_ascii_digit())
}

fn emit(expr: &Expr) -> Result<String, String> {
    match expr {
        Expr::Word(word) => Ok(format!("{}*", quote_term(word)?)),
        Expr::Phrase(words) => {
            let mut quoted = Vec::with_capacity(words.len());
            for word in words {
                validate_term(word)?;
                quoted.push(word.as_str());
            }
            Ok(format!("\"{}\"", quoted.join(" ")))
        }
        Expr::And(left, right) => Ok(format!("({} AND {})", emit(left)?, emit(right)?)),
        Expr::Or(left, right) => Ok(format!("({} OR {})", emit(left)?, emit(right)?)),
        Expr::Not(left, right) => Ok(format!("({} NOT {})", emit(left)?, emit(right)?)),
    }
}

fn quote_term(word: &str) -> Result<String, String> {
    validate_term(word)?;
    Ok(format!("\"{word}\""))
}

fn validate_term(word: &str) -> Result<(), String> {
    if word.is_empty() || !word.chars().all(|ch| ch.is_alphanumeric()) {
        return Err("The search could not be read.".into());
    }
    Ok(())
}

pub fn normalize_tag_list(tags: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for raw in tags {
        let collapsed = raw.split_whitespace().collect::<Vec<_>>().join(" ");
        if collapsed.is_empty() {
            continue;
        }
        let collapsed: String = collapsed.chars().take(80).collect();
        let key = collapsed.to_lowercase();
        if out
            .iter()
            .any(|existing: &String| existing.to_lowercase() == key)
        {
            continue;
        }
        out.push(collapsed);
        if out.len() == 64 {
            break;
        }
    }
    out
}

pub fn clean_display_name(value: Option<String>) -> Option<String> {
    let trimmed = value?.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.chars().take(300).collect())
    }
}

fn strip_image_extension(basename: &str) -> &str {
    match basename.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() && is_simple_ext(ext) => stem,
        _ => basename,
    }
}

fn is_simple_ext(ext: &str) -> bool {
    !ext.is_empty() && ext.len() <= 8 && ext.chars().all(|ch| ch.is_ascii_alphanumeric())
}

fn split_camel(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::new();
    for (index, ch) in chars.iter().enumerate() {
        if index > 0 && ch.is_uppercase() {
            let prev = chars[index - 1];
            let next_lower = chars
                .get(index + 1)
                .map(|next| next.is_lowercase())
                .unwrap_or(false);
            if prev.is_lowercase() || (prev.is_uppercase() && next_lower) {
                out.push(' ');
            }
        }
        out.push(*ch);
    }
    out
}

fn normalize_words(input: &str) -> String {
    let mut out = String::new();
    let mut pending_space = false;
    for ch in input.chars() {
        if ch.is_alphanumeric() {
            if pending_space && !out.is_empty() {
                out.push(' ');
            }
            pending_space = false;
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
        } else {
            pending_space = true;
        }
    }
    out
}

fn unique_words(stem: &str) -> Vec<String> {
    let mut out = Vec::new();
    for part in stem.split_whitespace() {
        let single_letter =
            part.chars().count() == 1 && !part.chars().all(|ch| ch.is_ascii_digit());
        if single_letter {
            continue;
        }
        if !out.iter().any(|existing| existing == part) {
            out.push(part.to_string());
        }
    }
    out
}

fn dedupe(items: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for item in items {
        if item.is_empty() {
            continue;
        }
        if out.iter().any(|existing| existing == &item) {
            continue;
        }
        out.push(item);
    }
    out
}
