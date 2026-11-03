use rusqlite::Connection;

use crate::db;
use crate::models::SearchResponse;
use crate::tokens::{self, FtsQuery};

pub fn search(
    conn: &Connection,
    query: &str,
    favorites_only: bool,
    hide_missing: bool,
    tagged_only: bool,
) -> Result<SearchResponse, String> {
    let (total, favorites) = db::counts(conn)?;
    let items = match tokens::fts_query(query)? {
        FtsQuery::All => db::search_filtered(
            conn,
            "ORDER BY i.basename COLLATE NOCASE, i.path",
            None,
            favorites_only,
            hide_missing,
            tagged_only,
        )?,
        FtsQuery::None => Vec::new(),
        FtsQuery::Match(match_query) => db::search_filtered(
            conn,
            "ORDER BY bm25(image_fts), i.basename COLLATE NOCASE, i.path",
            Some(&match_query),
            favorites_only,
            hide_missing,
            tagged_only,
        )
        .map_err(plain_match_error)?,
    };
    let visible = items.len() as u64;
    Ok(SearchResponse {
        items,
        visible,
        total,
        favorites,
    })
}

fn plain_match_error(error: String) -> String {
    let lower = error.to_ascii_lowercase();
    if lower.contains("fts5") || lower.contains("syntax error") || lower.contains("malformed match")
    {
        "The search could not be read. Check the operators, quotes, and parentheses.".into()
    } else {
        error
    }
}
