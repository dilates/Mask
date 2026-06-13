mod db;
mod models;
mod commands;

use commands::{persona::*, browser::*, leak_check::*, audit::*};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_personas,
            create_persona,
            update_persona,
            delete_persona,
            launch_browser,
            get_profile_path,
            run_leak_check,
            get_audit_log,
            get_data_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
