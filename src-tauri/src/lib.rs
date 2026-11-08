mod api;
mod commands;
mod db;
mod model;

use commands::AppState;
use db::Db;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            let db_path = dir.join("nobrake.db");
            let db = Db::open(&db_path).map_err(|e| {
                eprintln!("failed to open db {db_path:?}: {e}");
                e
            })?;
            app.manage(AppState {
                api: api::ApiClient::new(),
                db: Mutex::new(db),
                cancel: std::sync::atomic::AtomicBool::new(false),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::today_holds,
            commands::recent_holds,
            commands::list_holds,
            commands::race_numbers,
            commands::race_detail,
            commands::collect_day,
            commands::collect_recent,
            commands::cancel_collection,
            commands::sync_players,
            commands::search_players,
            commands::player_detail,
            commands::db_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
