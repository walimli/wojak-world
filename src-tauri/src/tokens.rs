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

/// Turn a search box string into an FTS5 prefix query.
/// `All` means the box was empty. `None` means the text survived no tokens.
pub enum FtsQuery {
    All,
    None,
    Match(String),
}

pub fn fts_query(raw: &str) -> FtsQuery {
    if raw.trim().is_empty() {
        return FtsQuery::All;
    }
    let stem = normalize_words(raw);
    let tokens = unique_words(&stem);
    if tokens.is_empty() {
        return FtsQuery::None;
    }
    let parts: Vec<String> = tokens
        .into_iter()
        .map(|token| format!("\"{token}\"*"))
        .collect();
    FtsQuery::Match(parts.join(" AND "))
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
