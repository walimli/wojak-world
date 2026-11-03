use std::collections::HashSet;
use std::path::Path;

use rusqlite::Connection;
use walkdir::WalkDir;

use crate::db::{self, FileMeta, Upsert};
use crate::models::ScanReport;
use crate::tokens;

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff"];

/// Read the library roots and update the index. Does not copy, move, or write
/// any file under those roots.
pub fn refresh(conn: &Connection, roots: &[String]) -> Result<ScanReport, String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|err| err.to_string())?;
    let mut report = ScanReport::default();
    let mut seen = HashSet::new();

    for root in roots {
        let root = db::trim_root(root);
        if !Path::new(&root).is_dir() {
            push_error(&mut report, format!("Folder not found: {root}"));
            continue;
        }
        let walker = WalkDir::new(&root).follow_links(false);
        for entry in walker {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    push_error(&mut report, err.to_string());
                    continue;
                }
            };
            if !entry.file_type().is_file() {
                continue;
            }
            let basename = entry.file_name().to_string_lossy().into_owned();
            if basename.starts_with('.') {
                continue;
            }
            let Some(extension) = image_extension(&basename) else {
                continue;
            };
            let file_meta = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(err) => {
                    push_error(&mut report, format!("{basename}: {err}"));
                    continue;
                }
            };
            let path = normalize_path(entry.path());
            let folder = entry
                .path()
                .parent()
                .map(normalize_path)
                .unwrap_or_default();
            let size = i64::try_from(file_meta.len()).unwrap_or(i64::MAX);
            let mtime = file_meta
                .modified()
                .ok()
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs() as i64)
                .unwrap_or(0);
            let meta = FileMeta {
                path: path.clone(),
                basename,
                folder,
                extension,
                size,
                mtime,
            };
            let derived = tokens::derive(&meta.basename, &meta.folder);
            match db::upsert_file(&tx, &meta, &derived) {
                Ok(Upsert::Added) => report.added += 1,
                Ok(Upsert::Updated) => report.updated += 1,
                Ok(Upsert::Unchanged) => {}
                Err(err) => push_error(&mut report, err),
            }
            report.seen += 1;
            seen.insert(path);
        }
    }

    report.marked_missing = db::reconcile_unseen(&tx, &seen)?;
    tx.commit().map_err(|err| err.to_string())?;
    Ok(report)
}

fn image_extension(basename: &str) -> Option<String> {
    let (_, extension) = basename.rsplit_once('.')?;
    if extension.is_empty()
        || extension.len() > 8
        || !extension.chars().all(|ch| ch.is_ascii_alphanumeric())
    {
        return None;
    }
    let extension = extension.to_ascii_lowercase();
    if IMAGE_EXTENSIONS.contains(&extension.as_str()) {
        Some(extension)
    } else {
        None
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('/', "\\")
}

fn push_error(report: &mut ScanReport, message: String) {
    if report.errors.len() < 20 {
        report.errors.push(message);
    } else if report.errors.len() == 20 {
        report.errors.push("Further errors were omitted.".into());
    }
}
