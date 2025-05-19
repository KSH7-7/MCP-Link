// app/src-tauri/src/lib.rs

use axum::{
    extract::State as AxumState,
    http::{Method, Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::post,
    Json, Router,
};
use dotenvy::dotenv;
use reqwest::Client;
use serde::Deserialize;
use std::{env, net::SocketAddr, sync::Arc};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Listener, Manager,
};
use tauri_plugin_deep_link::DeepLinkExt;
use tokio::sync::Mutex;

pub mod commands;
pub mod force_activate;
pub mod notification_system;
use crate::commands::AppState;
use crate::notification_system::{init_notification_system, KeywordState};

// POST request logging middleware function
async fn log_post_requests(
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if req.method() == Method::POST {
        let _uri = req.uri().clone(); // _uri to avoid warning, or log it
        let _headers = req.headers().clone(); // _headers to avoid warning, or log them
                                              // Note: Logging the request body requires caution.
                                              // Here, only URI and headers are (potentially) logged.
    }
    // Pass the request to the next handler or middleware
    Ok(next.run(req).await)
}

// Struct for keyword payload
#[derive(Deserialize, Debug)]
pub struct KeywordsPayload {
    keywords: Vec<String>,
}

// Struct for Axum server state
#[derive(Clone)]
pub struct RecommendationServerState {
    app_handle: Arc<Mutex<Option<AppHandle>>>,
}

impl RecommendationServerState {
    pub fn new() -> Self {
        Self {
            app_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn set_app_handle(&self, app_handle: AppHandle) {
        let mut handle = self.app_handle.lock().await;
        *handle = Some(app_handle);
    }
}

// Handler for keyword recommendation requests
async fn handle_recommendations(
    AxumState(state): AxumState<RecommendationServerState>,
    Json(payload): Json<KeywordsPayload>,
) -> StatusCode {
    // 키워드를 받아서 알림 표시 및 이벤트 발생
    if let Some(app_handle) = &*state.app_handle.lock().await {
        // 첫 번째 키워드 추출 (메인 키워드로 사용)
        if let Some(main_keyword) = payload.keywords.first() {
            // 모든 키워드를 문자열로 합치기
            let keywords_str = payload.keywords.join(", ");

            // 알림 제목과 내용 설정
            let title = "MCP 키워드 추천";
            let body = format!("검색어: {}", keywords_str);

            // 네이티브 알림 표시 시도
            #[cfg(target_os = "windows")]
            {
                if let Err(e) = notification_system::show_windows_notification(
                    title,
                    &body,
                    Some(main_keyword.to_string()),
                ) {
                    eprintln!(
                        "[Recommendation] Failed to show Windows notification: {}",
                        e
                    );
                }
            }

            #[cfg(target_os = "macos")]
            {
                if let Err(e) = notification_system::show_macos_notification(
                    title,
                    &body,
                    Some(main_keyword.to_string()),
                ) {
                    eprintln!("[Recommendation] Failed to show macOS notification: {}", e);
                }
            }

            #[cfg(target_os = "linux")]
            {
                if let Err(e) = notification_system::show_linux_notification(
                    title,
                    &body,
                    Some(main_keyword.to_string()),
                ) {
                    eprintln!("[Recommendation] Failed to show Linux notification: {}", e);
                }
            }

            // 키워드 상태에 저장
            if let Some(keyword_state) = app_handle.try_state::<KeywordState>() {
                keyword_state.set_keyword(main_keyword.clone());
            }
        }

        // 키워드 이벤트 발생 (UI 반응용)
        let _ = app_handle.emit("new-keywords", payload.keywords.clone());
    }

    StatusCode::OK
}

// Handler for /api/v1 requests
async fn handle_api_v1_request(
    AxumState(_state): AxumState<RecommendationServerState>,
) -> StatusCode {
    // Just return OK
    StatusCode::OK
}

// Function to start Axum server
pub async fn start_axum_server(
    app_state: RecommendationServerState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load .env file in development mode (ignore if already loaded)
    #[cfg(debug_assertions)]
    let _ = dotenv();

    // Get GUI API URL settings from environment variables
    // Get environment variables at runtime (use default value)
    let gui_api_host = env::var("GUI_API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    // Get environment variables at runtime (use default value)
    let gui_api_port = env::var("GUI_API_PORT").unwrap_or_else(|_| "8082".to_string());

    let addr_str = format!("{}:{}", gui_api_host, gui_api_port);
    let addr: SocketAddr = match addr_str.parse() {
        Ok(addr) => addr,
        Err(e) => {
            return Err(Box::new(e));
        }
    };

    // Configure Axum router
    let app = Router::new()
        .route("/recommendations", post(handle_recommendations))
        .route("/api/v1", post(handle_api_v1_request)) // Add handler for /api/v1 path
        .route("/api/v1/recommendations", post(handle_recommendations)) // Additional path for requests from mcp-server (based on app .env)
        // Removed duplicate path: "/recommendations" path was already added above
        .layer(middleware::from_fn(log_post_requests)) // Apply POST logging middleware
        .with_state(app_state);

    // Attempt to bind TcpListener
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            return Err(Box::new(e));
        }
    };

    // Start Axum server
    match axum::serve(listener, app).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

/// 딥링크 핸들러 설정
fn setup_deep_link_handler(app_handle: &AppHandle) {
    let log_path = std::env::temp_dir().join("mcplink_notification_click.log");

    // 로그 파일에 기록
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] 딥링크 핸들러 설정 중",
            chrono::Local::now().format("%H:%M:%S")
        );
    }

    // 앱 핸들 복제
    let app_handle_clone = app_handle.clone();

    // 1. 프로토콜 등록
    if let Err(err) = app_handle.deep_link().register("mcplink") {
        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path)
        {
            use std::io::Write;
            let _ = writeln!(
                file,
                "[{}] 딥링크 프로토콜 등록 실패: {:?}",
                chrono::Local::now().format("%H:%M:%S"),
                err
            );
        }
    } else {
        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path)
        {
            use std::io::Write;
            let _ = writeln!(
                file,
                "[{}] 딥링크 프로토콜 성공적으로 등록됨",
                chrono::Local::now().format("%H:%M:%S")
            );
        }
    }

    // 2. 딥링크 이벤트 리스너 설정
    app_handle.listen("deep-link://url-received", move |event| {
        // 이벤트 페이로드 가져오기 (이미 &str 타입)
        let url = event.payload().to_string();

        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path)
        {
            use std::io::Write;
            let _ = writeln!(
                file,
                "[{}] 딥링크 URL 수신됨: {}",
                chrono::Local::now().format("%H:%M:%S"),
                url
            );
        }

        // URL 처리 (빈 URL이 아닌 경우에만)
        if !url.is_empty() {
            handle_deeplink_url(&app_handle_clone, &url);
        } else {
            // 로그 파일에 기록
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path)
            {
                use std::io::Write;
                let _ = writeln!(
                    file,
                    "[{}] 딥링크 URL 비어있음",
                    chrono::Local::now().format("%H:%M:%S")
                );
            }
        }
    });
}

/// 딥링크 URL 처리
fn handle_deeplink_url(_app_handle: &AppHandle, url: &str) {
    let log_path = std::env::temp_dir().join("mcplink_notification_click.log");

    // URL에서 키워드 파라미터 추출
    if url.starts_with("mcplink://") || url.starts_with("mcplink:") {
        // 로그 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path)
        {
            use std::io::Write;
            let _ = writeln!(
                file,
                "[{}] 딥링크 URL 처리: {}",
                chrono::Local::now().format("%H:%M:%S"),
                url
            );
        }

        let parts: Vec<&str> = url.split('?').collect();
        if parts.len() > 1 {
            let params: Vec<&str> = parts[1].split('&').collect();
            for param in params {
                if param.starts_with("keyword=") {
                    let keyword = param.replace("keyword=", "");
                    if !keyword.is_empty() {
                        // 로그 기록
                        if let Ok(mut file) = std::fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .append(true)
                            .open(&log_path)
                        {
                            use std::io::Write;
                            let _ = writeln!(
                                file,
                                "[{}] 딥링크에서 키워드 추출: {}",
                                chrono::Local::now().format("%H:%M:%S"),
                                keyword
                            );
                        }

                        // 키워드 파일 생성 (중요: 파일명 변경 -> mcplink_last_keyword.txt)
                        let keyword_path = std::env::temp_dir().join("mcplink_last_keyword.txt");
                        if let Ok(mut file) = std::fs::File::create(&keyword_path) {
                            use std::io::Write;
                            let _ = write!(file, "{}", keyword);

                            // 로그 기록
                            if let Ok(mut log_file) = std::fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .append(true)
                                .open(&log_path)
                            {
                                use std::io::Write;
                                let _ = writeln!(
                                    log_file,
                                    "[{}] 딥링크 처리: 키워드 파일 생성됨",
                                    chrono::Local::now().format("%H:%M:%S")
                                );
                            }
                        }

                        // Tauri v2.0 호환성: 키워드 이벤트 직접 발생
                        let _ = _app_handle.emit("search-keyword", keyword.clone());

                        // 로그 기록
                        if let Ok(mut file) = std::fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .append(true)
                            .open(&log_path)
                        {
                            use std::io::Write;
                            let _ = writeln!(
                                file,
                                "[{}] search-keyword 이벤트 발생함: {}",
                                chrono::Local::now().format("%H:%M:%S"),
                                keyword
                            );
                        }

                        // 앱 활성화 (추가적인 시도)
                        // 1. 메인 윈도우 가져오기
                        if let Some(window) = _app_handle.get_webview_window("main") {
                            // 2. 윈도우 표시
                            let _ = window.show();

                            // 3. 윈도우 포커스 설정
                            let _ = window.set_focus();

                            // 4. 로그 기록
                            if let Ok(mut file) = std::fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .append(true)
                                .open(&log_path)
                            {
                                use std::io::Write;
                                let _ = writeln!(
                                    file,
                                    "[{}] 윈도우 API로 앱 활성화 시도 (show + set_focus)",
                                    chrono::Local::now().format("%H:%M:%S")
                                );
                            }
                        }

                        // 앱 활성화: 백업 방법
                        if let Err(e) = force_activate::force_app_to_foreground() {
                            // 로그 기록
                            if let Ok(mut file) = std::fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .append(true)
                                .open(&log_path)
                            {
                                use std::io::Write;
                                let _ = writeln!(
                                    file,
                                    "[{}] 앱 활성화 오류: {}",
                                    chrono::Local::now().format("%H:%M:%S"),
                                    e
                                );
                            }
                        }

                        // UserAttention API로 사용자 주의 끌기 - Tauri v2에서 추가된 API
                        if let Some(window) = _app_handle.get_webview_window("main") {
                            // 시스템 알림음과 함께 강조 표시 (Critical)
                            let _ = window
                                .request_user_attention(Some(tauri::UserAttentionType::Critical));

                            // 로그 기록
                            if let Ok(mut file) = std::fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .append(true)
                                .open(&log_path)
                            {
                                use std::io::Write;
                                let _ = writeln!(
                                    file,
                                    "[{}] UserAttention API 사용하여 사용자 주의 요청",
                                    chrono::Local::now().format("%H:%M:%S")
                                );
                            }
                        }
                    }
                    break;
                }
            }
        }
    }
}

pub fn run() {
    // Load .env file in development mode (ignore if already loaded)
    #[cfg(debug_assertions)]
    let _ = dotenv();

    // 앱 활성화 로그 초기화
    let activation_log_path = std::env::temp_dir().join("mcplink_activation.log");
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&activation_log_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "=== [{}] 앱 시작됨 (Tauri v2.0) ===",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        );
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            // 임의의 인스턴스 실행시에 주요 인스턴스 표시 및 포커스
            let window = app.get_webview_window("main").unwrap();
            window.show().unwrap();
            window.set_focus().unwrap();
        }))
        .manage(AppState {
            client: Client::new(),
        })
        .setup(move |app| {
            // API 서버 시작
            let server_state = RecommendationServerState::new();
            let app_handle = app.handle().clone();

            // Axum 서버 시작 (백그라운드 스레드)
            {
                let server_state_clone = server_state.clone();
                let app_handle_clone = app_handle.clone();

                // 명시적으로 Tokio 런타임 생성
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("Tokio 런타임 생성 실패");

                    rt.block_on(async {
                        // Axum 서버에 앱 핸들 설정
                        server_state_clone.set_app_handle(app_handle_clone).await;

                        // Axum 서버 시작
                        match start_axum_server(server_state_clone).await {
                            Ok(_) => println!("API 서버 종료됨"),
                            Err(e) => eprintln!("API 서버 오류: {}", e),
                        }
                    });
                });
            }

            // 트레이 아이콘 설정
            // Tauri v2.0 방식으로 수정
            let quit_menu_item = MenuItemBuilder::with_id("quit", "종료").build(app.handle())?;
            let show_menu_item = MenuItemBuilder::with_id("show", "열기").build(app.handle())?;
            let menu = MenuBuilder::new(app.handle())
                .item(&show_menu_item)
                .item(&quit_menu_item)
                .build()?;

            let tray_icon = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("MCP Link")
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "show" => {
                        let window = app.get_webview_window("main").unwrap();
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                    _ => {}
                })
                .build(app.handle())?;

            // Tauri의 window 이벤트 핸들러 설정 (중요: 포커스 이벤트 처리)
            let main_window = app.get_webview_window("main").unwrap();
            let log_path_clone = activation_log_path.clone();

            main_window.on_window_event(move |event| {
                match event {
                    tauri::WindowEvent::Focused(focused) => {
                        // 포커스 획득/상실 이벤트 로깅
                        if let Ok(mut file) = std::fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .append(true)
                            .open(&log_path_clone)
                        {
                            use std::io::Write;
                            let _ = writeln!(
                                file,
                                "[{}] 윈도우 포커스 이벤트: {}",
                                chrono::Local::now().format("%H:%M:%S"),
                                if *focused { "획득" } else { "상실" }
                            );
                        }
                    }
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        // 창이 닫힐 때 처리
                        if let Ok(mut file) = std::fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .append(true)
                            .open(&log_path_clone)
                        {
                            use std::io::Write;
                            let _ = writeln!(
                                file,
                                "[{}] 윈도우 닫기 요청됨",
                                chrono::Local::now().format("%H:%M:%S")
                            );
                        }
                    }
                    _ => {}
                }
            });

            // 알림 시스템 초기화
            if let Err(e) = init_notification_system(app) {
                eprintln!("알림 시스템 초기화 오류: {}", e);
            }

            // 딥링크 핸들러 설정
            setup_deep_link_handler(&app_handle);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::check_and_mark_app_activated,
            commands::simulate_notification_click,
            commands::get_mcp_data,
            commands::get_mcp_detail_data,
            commands::add_mcp_server_config,
            commands::remove_mcp_server_config,
            commands::restart_claude_desktop,
            commands::get_installed_mcp_data,
            commands::search_local_mcp_servers,
            commands::read_mcplink_config_content,
            commands::check_claude_config_exists,
            commands::check_mcplink_config_exists,
            commands::ensure_config_files,
            commands::start_config_watch,
            commands::read_mcp_server_config,
            commands::is_mcp_server_installed,
            commands::reset_mcp_settings,
            commands::test_force_activate,
            commands::test_search_keyword
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Focused(focused) = event {
                if *focused {
                    // 포커스를 얻었을 때 활성화 로그 기록
                    let log_path = std::env::temp_dir().join("mcplink_activation.log");
                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(true)
                        .open(&log_path)
                    {
                        use std::io::Write;
                        let _ = writeln!(
                            file,
                            "[{}] [글로벌 이벤트] 앱이 포커스를 얻음",
                            chrono::Local::now().format("%H:%M:%S")
                        );
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("Tauri 앱 실행 중 오류가 발생했습니다");
}
