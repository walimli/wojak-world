use tauri::Manager;

mod commands;
mod db;
mod models;
mod protocol;
mod scan;
mod search;
mod state;
mod thumbs;
mod tokens;

pub fn run() {
    tauri::Builder::default()
        .register_asynchronous_uri_scheme_protocol("wojak", protocol::handle)
        .setup(|app| {
            app.manage(state::AppState::open(app.handle())?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::library_state,
            commands::set_roots,
            commands::run_setup,
            commands::refresh,
            commands::search,
            commands::image_detail,
            commands::set_favorite,
            commands::set_display_name,
            commands::set_tags,
        ])
        .run(tauri::generate_context!())
        .expect("error while running wojak-world");
}
