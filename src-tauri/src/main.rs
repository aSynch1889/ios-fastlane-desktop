mod commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::select_project_path,
            commands::scan_project,
            commands::generate_fastlane_files,
            commands::run_lane,
            commands::save_profile,
            commands::load_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
