use tauri::State;

use crate::db;
use crate::models::{ImageDetail, LibraryState, ScanReport, SearchResponse};
use crate::scan;
use crate::search;
use crate::state::AppState;

#[tauri::command]
pub fn library_state(state: State<AppState>) -> Result<LibraryState, String> {
    state.with_db(db::library_state)
}

#[tauri::command]
pub fn set_roots(state: State<AppState>, roots: Vec<String>) -> Result<Vec<String>, String> {
    let roots = db::normalize_roots(roots)?;
    state.with_db(|conn| db::save_roots(conn, &roots))?;
    Ok(roots)
}

#[tauri::command]
pub fn run_setup(state: State<AppState>, roots: Vec<String>) -> Result<ScanReport, String> {
    let roots = db::normalize_roots(roots)?;
    state.with_db(|conn| {
        if db::setup_complete(conn)? {
            return Err("The library is already indexed. Use Refresh.".into());
        }
        db::save_roots(conn, &roots)?;
        let report = scan::refresh(conn, &roots)?;
        db::mark_setup_complete(conn)?;
        Ok(report)
    })
}

#[tauri::command]
pub fn refresh(state: State<AppState>) -> Result<ScanReport, String> {
    state.with_db(|conn| {
        if !db::setup_complete(conn)? {
            return Err("Index the library from the setup screen first.".into());
        }
        let roots = db::stored_roots(conn)?;
        scan::refresh(conn, &roots)
    })
}

#[tauri::command]
pub fn search(
    state: State<AppState>,
    query: String,
    favorites_only: bool,
    hide_missing: bool,
) -> Result<SearchResponse, String> {
    state.with_db(|conn| search::search(conn, &query, favorites_only, hide_missing))
}

#[tauri::command]
pub fn image_detail(state: State<AppState>, path: String) -> Result<ImageDetail, String> {
    state.with_db(|conn| db::image_detail(conn, &path))
}

#[tauri::command]
pub fn set_favorite(state: State<AppState>, path: String, favorite: bool) -> Result<bool, String> {
    state.with_db(|conn| db::set_favorite(conn, &path, favorite))
}

#[tauri::command]
pub fn set_display_name(
    state: State<AppState>,
    path: String,
    display_name: Option<String>,
) -> Result<Option<String>, String> {
    state.with_db(|conn| db::set_display_name(conn, &path, display_name))
}

#[tauri::command]
pub fn set_tags(
    state: State<AppState>,
    path: String,
    tags: Vec<String>,
) -> Result<Vec<String>, String> {
    state.with_db(|conn| db::set_tags(conn, &path, &tags))
}
