use std::sync::Mutex;
use std::time::Duration;

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::db;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub thumbs_dir: std::path::PathBuf,
}

impl AppState {
    pub fn open(app: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let dir = app.path().app_local_data_dir()?;
        std::fs::create_dir_all(&dir)?;
        let thumbs_dir = dir.join("thumbs");
        std::fs::create_dir_all(&thumbs_dir)?;

        let conn = Connection::open(dir.join("library.sqlite"))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.busy_timeout(Duration::from_secs(5))?;
        db::migrate(&conn)?;

        Ok(Self {
            db: Mutex::new(conn),
            thumbs_dir,
        })
    }

    pub fn with_db<T>(
        &self,
        f: impl FnOnce(&Connection) -> Result<T, String>,
    ) -> Result<T, String> {
        let conn = self
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        f(&conn)
    }
}
