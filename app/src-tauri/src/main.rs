// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    let client = reqwest::Client::new();
    let app_state = commands::AppState { client }; // AppState 생성

    tauri::Builder::default()
        .manage(app_state) // AppState를 tauri에 등록
        .invoke_handler(tauri::generate_handler![
            commands::some_command,
            commands::get_mcp_data,
            commands::get_mcp_detail_data,
            commands::add_mcp_server_config,
            commands::remove_mcp_server_config,
            commands::restart_claude_desktop
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
