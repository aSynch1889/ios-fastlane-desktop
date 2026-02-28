mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::scan_project,
            commands::doctor_check,
            commands::resolve_identity,
            commands::generate_fastlane_files,
            commands::run_lane,
            commands::save_profile,
            commands::load_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
