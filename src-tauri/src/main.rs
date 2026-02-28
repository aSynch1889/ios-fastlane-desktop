mod commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::scan_project,
            commands::generate_fastlane_files,
            commands::run_lane,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
