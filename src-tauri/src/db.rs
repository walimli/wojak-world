use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, params};

use crate::models::{ImageDetail, ImageSummary, LibraryState, default_roots};
use crate::tokens::{self, Derived};

const MIGRATION_1: &str = r#"
CREATE TABLE images (
  path TEXT PRIMARY KEY NOT NULL,
  basename TEXT NOT NULL,
  folder TEXT NOT NULL,
  extension TEXT NOT NULL,
  size INTEGER NOT NULL,
  mtime INTEGER NOT NULL,
  favorite INTEGER NOT NULL DEFAULT 0 CHECK (favorite IN (0, 1)),
  display_name TEXT,
  missing INTEGER NOT NULL DEFAULT 0 CHECK (missing IN (0, 1)),
  indexed_at INTEGER NOT NULL
);

CREATE TABLE tags (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL COLLATE NOCASE UNIQUE
);

CREATE TABLE image_tags (
  path TEXT NOT NULL,
  tag_id INTEGER NOT NULL,
  PRIMARY KEY (path, tag_id),
  FOREIGN KEY (path) REFERENCES images(path) ON DELETE CASCADE,
  FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX image_tags_tag ON image_tags(tag_id);

CREATE TABLE image_tokens (
  path TEXT NOT NULL,
  token TEXT NOT NULL,
  PRIMARY KEY (path, token),
  FOREIGN KEY (path) REFERENCES images(path) ON DELETE CASCADE
);

CREATE TABLE image_categories (
  path TEXT NOT NULL,
  category TEXT NOT NULL,
  PRIMARY KEY (path, category),
  FOREIGN KEY (path) REFERENCES images(path) ON DELETE CASCADE
);

CREATE TABLE image_search (
  path TEXT PRIMARY KEY,
  basename TEXT NOT NULL,
  tokens TEXT NOT NULL,
  tags TEXT NOT NULL,
  display_name TEXT NOT NULL DEFAULT '',
  FOREIGN KEY (path) REFERENCES images(path) ON DELETE CASCADE
);

CREATE VIRTUAL TABLE image_fts USING fts5(
  basename,
  tokens,
  tags,
  display_name,
  content='image_search',
  content_rowid='rowid',
  tokenize='unicode61 remove_diacritics 2'
);

CREATE TRIGGER image_search_ai AFTER INSERT ON image_search BEGIN
  INSERT INTO image_fts(rowid, basename, tokens, tags, display_name)
  VALUES (new.rowid, new.basename, new.tokens, new.tags, new.display_name);
END;

CREATE TRIGGER image_search_ad AFTER DELETE ON image_search BEGIN
  INSERT INTO image_fts(image_fts, rowid, basename, tokens, tags, display_name)
  VALUES ('delete', old.rowid, old.basename, old.tokens, old.tags, old.display_name);
END;

CREATE TRIGGER image_search_au AFTER UPDATE ON image_search BEGIN
  INSERT INTO image_fts(image_fts, rowid, basename, tokens, tags, display_name)
  VALUES ('delete', old.rowid, old.basename, old.tokens, old.tags, old.display_name);
  INSERT INTO image_fts(rowid, basename, tokens, tags, display_name)
  VALUES (new.rowid, new.basename, new.tokens, new.tags, new.display_name);
END;

CREATE TABLE thumbnails (
  path TEXT PRIMARY KEY,
  file_name TEXT NOT NULL,
  width INTEGER NOT NULL,
  height INTEGER NOT NULL,
  source_mtime INTEGER NOT NULL,
  source_size INTEGER NOT NULL,
  generated_at INTEGER NOT NULL,
  FOREIGN KEY (path) REFERENCES images(path) ON DELETE CASCADE
);

CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE INDEX images_favorite ON images(favorite);
CREATE INDEX images_missing ON images(missing);
CREATE INDEX images_basename ON images(basename);
"#;

pub struct FileMeta {
    pub path: String,
    pub basename: String,
    pub folder: String,
    pub extension: String,
    pub size: i64,
    pub mtime: i64,
}

pub enum Upsert {
    Added,
    Updated,
    Unchanged,
}

#[derive(Debug)]
pub struct IndexedFile {
    pub mtime: i64,
    pub size: i64,
    pub missing: bool,
}

#[derive(Debug)]
pub struct ThumbRow {
    pub file_name: String,
    pub source_mtime: i64,
    pub source_size: i64,
}

pub fn migrate(conn: &Connection) -> Result<(), rusqlite::Error> {
    let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version < 1 {
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(MIGRATION_1)?;
        tx.pragma_update(None, "user_version", 1)?;
        tx.commit()?;
    }
    Ok(())
}

pub fn library_state(conn: &Connection) -> Result<LibraryState, String> {
    let setup_complete = setting(conn, "setup_complete")?.as_deref() == Some("1");
    let roots = match setting(conn, "roots")? {
        Some(json) => serde_json::from_str(&json).unwrap_or_else(|_| default_roots()),
        None => default_roots(),
    };
    Ok(LibraryState {
        setup_complete,
        roots,
        default_roots: default_roots(),
    })
}

pub fn setup_complete(conn: &Connection) -> Result<bool, String> {
    Ok(setting(conn, "setup_complete")?.as_deref() == Some("1"))
}

pub fn mark_setup_complete(conn: &Connection) -> Result<(), String> {
    set_setting(conn, "setup_complete", "1")
}

pub fn stored_roots(conn: &Connection) -> Result<Vec<String>, String> {
    match setting(conn, "roots")? {
        Some(json) => serde_json::from_str(&json).map_err(|err| err.to_string()),
        None => Ok(default_roots()),
    }
}

pub fn save_roots(conn: &Connection, roots: &[String]) -> Result<(), String> {
    let json = serde_json::to_string(roots).map_err(|err| err.to_string())?;
    set_setting(conn, "roots", &json)
}

pub fn normalize_roots(roots: Vec<String>) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut seen = Vec::new();
    for root in roots {
        let trimmed = trim_root(&root);
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.len() > 1024 {
            return Err("A folder path is too long.".into());
        }
        let key = trimmed.to_lowercase();
        if seen.iter().any(|existing: &String| existing == &key) {
            continue;
        }
        seen.push(key);
        out.push(trimmed);
    }
    if out.is_empty() {
        return Err("Add at least one folder.".into());
    }
    if out.len() > 32 {
        return Err("Use 32 folders or fewer.".into());
    }
    Ok(out)
}

pub fn upsert_file(
    conn: &Connection,
    meta: &FileMeta,
    derived: &Derived,
) -> Result<Upsert, String> {
    let existing: Option<(i64, i64, i64, String)> = conn
        .query_row(
            "SELECT size, mtime, missing, basename FROM images WHERE path = ?",
            [&meta.path],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(err)?;

    match existing {
        None => {
            insert_new(conn, meta, derived)?;
            Ok(Upsert::Added)
        }
        Some((size, mtime, missing, basename))
            if size == meta.size
                && mtime == meta.mtime
                && missing == 0
                && basename == meta.basename =>
        {
            Ok(Upsert::Unchanged)
        }
        Some((_, _, _, basename)) => {
            conn.execute(
                "UPDATE images
                 SET basename = ?, folder = ?, extension = ?, size = ?, mtime = ?, missing = 0
                 WHERE path = ?",
                params![
                    meta.basename,
                    meta.folder,
                    meta.extension,
                    meta.size,
                    meta.mtime,
                    meta.path
                ],
            )
            .map_err(err)?;
            if basename != meta.basename {
                replace_derived(conn, &meta.path, derived)?;
                let display = display_name_text(conn, &meta.path)?;
                let tags = tags_text(conn, &meta.path)?;
                write_search(conn, &meta.path, derived, &tags, &display)?;
            }
            Ok(Upsert::Updated)
        }
    }
}

/// Paths the walk did not see are stat'd. A missing file keeps its row.
/// A file that is still on disk stays indexed, including folders no longer scanned.
pub fn reconcile_unseen(conn: &Connection, seen: &HashSet<String>) -> Result<u64, String> {
    let mut stmt = conn.prepare("SELECT path FROM images").map_err(err)?;
    let paths = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;
    drop(stmt);

    let mut marked = 0u64;
    for path in paths {
        if seen.contains(&path) {
            continue;
        }
        match std::fs::metadata(&path) {
            Ok(metadata) => {
                let size = i64::try_from(metadata.len()).unwrap_or(i64::MAX);
                let mtime = metadata
                    .modified()
                    .ok()
                    .and_then(|time| time.duration_since(SystemTime::UNIX_EPOCH).ok())
                    .map(|duration| duration.as_secs() as i64)
                    .unwrap_or(0);
                conn.execute(
                    "UPDATE images SET missing = 0, size = ?1, mtime = ?2 WHERE path = ?3",
                    params![size, mtime, path],
                )
                .map_err(err)?;
            }
            Err(_) => {
                let changed = conn
                    .execute(
                        "UPDATE images SET missing = 1 WHERE path = ?1 AND missing = 0",
                        [&path],
                    )
                    .map_err(err)?;
                marked += changed as u64;
            }
        }
    }
    Ok(marked)
}

pub fn counts(conn: &Connection) -> Result<(u64, u64), String> {
    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM images", [], |row| row.get(0))
        .map_err(err)?;
    let favorites: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM images WHERE favorite = 1",
            [],
            |row| row.get(0),
        )
        .map_err(err)?;
    Ok((total as u64, favorites as u64))
}

pub fn search_filtered(
    conn: &Connection,
    order_sql: &str,
    match_query: Option<&str>,
    favorites_only: bool,
    hide_missing: bool,
) -> Result<Vec<ImageSummary>, String> {
    let sql = if match_query.is_some() {
        format!(
            "SELECT i.path, i.basename, i.folder, i.extension, i.size, i.mtime,
                    i.favorite, i.display_name, i.missing
             FROM image_fts
             JOIN image_search s ON s.rowid = image_fts.rowid
             JOIN images i ON i.path = s.path
             WHERE image_fts MATCH ?1
               AND (?2 = 0 OR i.favorite = 1)
               AND (?3 = 0 OR i.missing = 0)
             {order_sql}"
        )
    } else {
        format!(
            "SELECT path, basename, folder, extension, size, mtime,
                    favorite, display_name, missing
             FROM images
             WHERE (?1 = 0 OR favorite = 1)
               AND (?2 = 0 OR missing = 0)
             {order_sql}"
        )
    };

    let mut stmt = conn.prepare(&sql).map_err(err)?;
    let favorites = if favorites_only { 1 } else { 0 };
    let hide = if hide_missing { 1 } else { 0 };
    let rows = if let Some(query) = match_query {
        stmt.query_map(params![query, favorites, hide], map_summary)
            .map_err(err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(err)?
    } else {
        stmt.query_map(params![favorites, hide], map_summary)
            .map_err(err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(err)?
    };
    Ok(rows)
}

pub fn image_detail(conn: &Connection, path: &str) -> Result<ImageDetail, String> {
    let image =
        load_summary(conn, path)?.ok_or_else(|| "Image is not in the index.".to_string())?;
    Ok(ImageDetail {
        image,
        tags: tag_names(conn, path)?,
        categories: category_names(conn, path)?,
    })
}

pub fn indexed_file(conn: &Connection, path: &str) -> Result<Option<IndexedFile>, String> {
    conn.query_row(
        "SELECT mtime, size, missing FROM images WHERE path = ?",
        [path],
        |row| {
            Ok(IndexedFile {
                mtime: row.get(0)?,
                size: row.get(1)?,
                missing: row.get::<_, i64>(2)? != 0,
            })
        },
    )
    .optional()
    .map_err(err)
}

pub fn thumb_row(conn: &Connection, path: &str) -> Result<Option<ThumbRow>, String> {
    conn.query_row(
        "SELECT file_name, source_mtime, source_size FROM thumbnails WHERE path = ?",
        [path],
        |row| {
            Ok(ThumbRow {
                file_name: row.get(0)?,
                source_mtime: row.get(1)?,
                source_size: row.get(2)?,
            })
        },
    )
    .optional()
    .map_err(err)
}

pub fn save_thumb(
    conn: &Connection,
    path: &str,
    file_name: &str,
    width: u32,
    height: u32,
    source_mtime: i64,
    source_size: i64,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO thumbnails(path, file_name, width, height, source_mtime, source_size, generated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(path) DO UPDATE SET
           file_name = excluded.file_name,
           width = excluded.width,
           height = excluded.height,
           source_mtime = excluded.source_mtime,
           source_size = excluded.source_size,
           generated_at = excluded.generated_at",
        params![
            path,
            file_name,
            width,
            height,
            source_mtime,
            source_size,
            unix_now()
        ],
    )
    .map_err(err)?;
    Ok(())
}

pub fn set_favorite(conn: &Connection, path: &str, favorite: bool) -> Result<bool, String> {
    require_image(conn, path)?;
    conn.execute(
        "UPDATE images SET favorite = ? WHERE path = ?",
        params![if favorite { 1 } else { 0 }, path],
    )
    .map_err(err)?;
    Ok(favorite)
}

pub fn set_display_name(
    conn: &Connection,
    path: &str,
    display_name: Option<String>,
) -> Result<Option<String>, String> {
    require_image(conn, path)?;
    let cleaned = tokens::clean_display_name(display_name);
    let tx = conn.unchecked_transaction().map_err(err)?;
    tx.execute(
        "UPDATE images SET display_name = ? WHERE path = ?",
        params![cleaned, path],
    )
    .map_err(err)?;
    tx.execute(
        "UPDATE image_search SET display_name = ? WHERE path = ?",
        params![cleaned.clone().unwrap_or_default(), path],
    )
    .map_err(err)?;
    tx.commit().map_err(err)?;
    Ok(cleaned)
}

pub fn set_tags(conn: &Connection, path: &str, tags: &[String]) -> Result<Vec<String>, String> {
    require_image(conn, path)?;
    let cleaned = tokens::normalize_tag_list(tags);
    let tx = conn.unchecked_transaction().map_err(err)?;
    tx.execute("DELETE FROM image_tags WHERE path = ?", [path])
        .map_err(err)?;
    for name in &cleaned {
        tx.execute(
            "INSERT INTO tags(name) VALUES (?1) ON CONFLICT(name) DO NOTHING",
            [name],
        )
        .map_err(err)?;
        let id: i64 = tx
            .query_row(
                "SELECT id FROM tags WHERE name = ?1 COLLATE NOCASE",
                [name],
                |row| row.get(0),
            )
            .map_err(err)?;
        tx.execute(
            "INSERT OR IGNORE INTO image_tags(path, tag_id) VALUES (?1, ?2)",
            params![path, id],
        )
        .map_err(err)?;
    }
    tx.execute(
        "DELETE FROM tags WHERE id NOT IN (SELECT tag_id FROM image_tags)",
        [],
    )
    .map_err(err)?;
    let stored = tag_names(&tx, path)?;
    let blob = stored.join(" ");
    tx.execute(
        "UPDATE image_search SET tags = ? WHERE path = ?",
        params![blob, path],
    )
    .map_err(err)?;
    tx.commit().map_err(err)?;
    Ok(stored)
}

pub fn trim_root(root: &str) -> String {
    let trimmed = root.trim();
    let mut chars = trimmed.chars();
    let first = chars.next();
    let second = chars.next();
    let drive = matches!(
        (first, second),
        (Some(letter), Some(':')) if letter.is_ascii_alphabetic()
    );
    if drive && trimmed.chars().count() <= 3 {
        let letter = first.unwrap().to_ascii_uppercase();
        return format!("{letter}:\\");
    }
    trimmed.trim_end_matches(['\\', '/']).to_string()
}

fn insert_new(conn: &Connection, meta: &FileMeta, derived: &Derived) -> Result<(), String> {
    conn.execute(
        "INSERT INTO images(
            path, basename, folder, extension, size, mtime, favorite, display_name, missing, indexed_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, NULL, 0, ?7)",
        params![
            meta.path,
            meta.basename,
            meta.folder,
            meta.extension,
            meta.size,
            meta.mtime,
            unix_now()
        ],
    )
    .map_err(err)?;
    replace_derived(conn, &meta.path, derived)?;
    write_search(conn, &meta.path, derived, "", "")
}

fn replace_derived(conn: &Connection, path: &str, derived: &Derived) -> Result<(), String> {
    conn.execute("DELETE FROM image_tokens WHERE path = ?", [path])
        .map_err(err)?;
    conn.execute("DELETE FROM image_categories WHERE path = ?", [path])
        .map_err(err)?;
    for token in &derived.tokens {
        conn.execute(
            "INSERT INTO image_tokens(path, token) VALUES (?1, ?2)",
            params![path, token],
        )
        .map_err(err)?;
    }
    for category in &derived.categories {
        conn.execute(
            "INSERT OR IGNORE INTO image_categories(path, category) VALUES (?1, ?2)",
            params![path, category],
        )
        .map_err(err)?;
    }
    Ok(())
}

fn write_search(
    conn: &Connection,
    path: &str,
    derived: &Derived,
    tags: &str,
    display_name: &str,
) -> Result<(), String> {
    let exists: bool = conn
        .query_row("SELECT 1 FROM image_search WHERE path = ?", [path], |_| {
            Ok(true)
        })
        .optional()
        .map_err(err)?
        .unwrap_or(false);
    if exists {
        conn.execute(
            "UPDATE image_search
             SET basename = ?1, tokens = ?2, tags = ?3, display_name = ?4
             WHERE path = ?5",
            params![derived.stem, derived.token_blob, tags, display_name, path],
        )
        .map_err(err)?;
    } else {
        conn.execute(
            "INSERT INTO image_search(path, basename, tokens, tags, display_name)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![path, derived.stem, derived.token_blob, tags, display_name],
        )
        .map_err(err)?;
    }
    Ok(())
}

fn display_name_text(conn: &Connection, path: &str) -> Result<String, String> {
    conn.query_row(
        "SELECT COALESCE(display_name, '') FROM images WHERE path = ?",
        [path],
        |row| row.get(0),
    )
    .map_err(err)
}

fn tags_text(conn: &Connection, path: &str) -> Result<String, String> {
    Ok(tag_names(conn, path)?.join(" "))
}

fn tag_names(conn: &Connection, path: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT t.name
             FROM image_tags it
             JOIN tags t ON t.id = it.tag_id
             WHERE it.path = ?1
             ORDER BY t.name COLLATE NOCASE",
        )
        .map_err(err)?;
    stmt.query_map([path], |row| row.get(0))
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)
}

fn category_names(conn: &Connection, path: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT category FROM image_categories WHERE path = ?1 ORDER BY category COLLATE NOCASE",
        )
        .map_err(err)?;
    stmt.query_map([path], |row| row.get(0))
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)
}

fn load_summary(conn: &Connection, path: &str) -> Result<Option<ImageSummary>, String> {
    conn.query_row(
        "SELECT path, basename, folder, extension, size, mtime, favorite, display_name, missing
         FROM images WHERE path = ?",
        [path],
        map_summary,
    )
    .optional()
    .map_err(err)
}

fn require_image(conn: &Connection, path: &str) -> Result<(), String> {
    let found: Option<i64> = conn
        .query_row("SELECT 1 FROM images WHERE path = ?", [path], |row| {
            row.get(0)
        })
        .optional()
        .map_err(err)?;
    if found.is_some() {
        Ok(())
    } else {
        Err("Image is not in the index.".into())
    }
}

fn map_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<ImageSummary> {
    let size: i64 = row.get(4)?;
    let favorite: i64 = row.get(6)?;
    let missing: i64 = row.get(8)?;
    Ok(ImageSummary {
        path: row.get(0)?,
        basename: row.get(1)?,
        folder: row.get(2)?,
        extension: row.get(3)?,
        size: size.max(0) as u64,
        mtime: row.get(5)?,
        favorite: favorite != 0,
        display_name: row.get(7)?,
        missing: missing != 0,
    })
}

fn setting(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    conn.query_row("SELECT value FROM settings WHERE key = ?", [key], |row| {
        row.get(0)
    })
    .optional()
    .map_err(err)
}

fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO settings(key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .map_err(err)?;
    Ok(())
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn err(error: impl ToString) -> String {
    error.to_string()
}
