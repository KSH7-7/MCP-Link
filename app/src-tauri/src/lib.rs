mod commands;

use commands::AppState;
use reqwest::Client;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

/// Tauri 앱을 실행하는 함수
pub fn run() {
    // AppState 생성 (API 요청용 client 유지)
    let app_state = AppState {
        client: Client::new(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_positioner::init())
        .setup(|app| {
            // Positioner 플러그인 초기화
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_positioner::init());

            // 팝업 윈도우 생성 예시 (필요한 경우)
            let popup = tauri::WebviewWindowBuilder::new(
                app,
                "popup",
                tauri::WebviewUrl::App("popup".into()),
            )
            .title("알림")
            .inner_size(300.0, 200.0)
            .decorations(false) // 창 테두리 제거
            .always_on_top(true) // 항상 위에 표시
            .skip_taskbar(true) // 작업 표시줄에 표시 안 함
            .build()?;

            // 메뉴 아이템 생성
            let open_item = MenuItemBuilder::with_id("open", "open").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "quit").build(app)?;

            // 메뉴 구성
            let menu = MenuBuilder::new(app)
                .items(&[&open_item, &quit_item])
                .build()?;

            // 아이콘 경로 (tauri.conf.json에서 이미 설정함)
            let _icon_path = app
                .path()
                .resolve("icons/32x32.png", tauri::path::BaseDirectory::Resource)
                .expect("error: icon file not found");

            // 트레이 아이콘 생성
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if let Ok(visible) = window.is_visible() {
                                if visible {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                    }
                    TrayIconEvent::Click {
                        button: MouseButton::Right,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {}
                    _ => {}
                })
                .build(app)?;

            // 창 닫기 이벤트 처리 - 소유권 문제 해결을 위해 clone 사용
            let main_window = app.get_webview_window("main").unwrap();
            let window_clone = main_window.clone();
            main_window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    let _ = window_clone.hide();
                    api.prevent_close();
                }
            });

            Ok(())
        })
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::get_mcp_data,
            commands::add_mcp_server_config,
            commands::remove_mcp_server_config,
            commands::restart_claude_desktop
        ])
        .run(tauri::generate_context!())
        .expect("error: Tauri application run failed");
}
