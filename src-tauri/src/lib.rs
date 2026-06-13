mod db;
mod models;
mod commands;

use commands::{
    persona::*,
    browser::*,
    leak_check::*,
    audit::*,
    wireguard::*,
    export_import::*,
    container::*,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Persona CRUD
            list_personas,
            create_persona,
            update_persona,
            delete_persona,
            // Browser
            launch_browser,
            get_profile_path,
            detect_browsers,
            // Leak check
            run_leak_check,
            // Audit
            get_audit_log,
            get_data_dir,
            // WireGuard
            wg_up,
            wg_down,
            wg_status,
            // Export / Import
            export_personas,
            import_personas,
            // Container
            launch_container_browser,
            stop_container,
            detect_container_runtimes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
