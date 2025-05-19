// app/src-tauri/src/lib.rs

use axum::{
    extract::State as AxumState,
    http::{Method, Request, StatusCode}, // Added Method and Request
    middleware::{self, Next},            // Added middleware and Next
    response::Response,                  // Added Response
    routing::post,
    Json,
    Router,
};
use reqwest::Client;
use serde::Deserialize;
use std::{env, net::SocketAddr, sync::Arc};
use dotenvy::dotenv;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Listener, Emitter,
};
use tokio::sync::Mutex;

pub mod commands;
pub mod notification_system;
pub mod force_activate;
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
                    Some(main_keyword.to_string())
                ) {
                    eprintln!("[Recommendation] Failed to show Windows notification: {}", e);
                }
            }
            
            #[cfg(target_os = "macos")]
            {
                if let Err(e) = notification_system::show_macos_notification(
                    title, 
                    &body, 
                    Some(main_keyword.to_string())
                ) {
                    eprintln!("[Recommendation] Failed to show macOS notification: {}", e);
                }
            }
            
            #[cfg(target_os = "linux")]
            {
                if let Err(e) = notification_system::show_linux_notification(
                    title, 
                    &body, 
                    Some(main_keyword.to_string())
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
        use tauri::Emitter;
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
pub async fn start_axum_server(app_state: RecommendationServerState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        Err(e) => {
            Err(Box::new(e))
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
        .open(&activation_log_path) {
        use std::io::Write;
        let _ = writeln!(file, "\n\n=== [{}] MCPLink 앱 시작됨 ===", 
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
        
        // OS 정보 로깅
        let _ = writeln!(file, "OS: {}", std::env::consts::OS);
        let _ = writeln!(file, "ARCH: {}", std::env::consts::ARCH);
    }
    
    // Create AppState (maintains client for API requests)
    let app_state = AppState {
        client: Client::new(),
    };

    // Create AppState for Axum server
    let recommendation_server_state = RecommendationServerState::new();
    let recommendation_server_state_clone = recommendation_server_state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // 새로운 인스턴스가 실행됐을 때, 기존 창에 포커스
            app.get_webview_window("main")
                .expect("메인 창을 찾을 수 없습니다")
                .set_focus()
                .unwrap();

            // 필요시 창 복원 및 전면으로 가져오기
            #[cfg(target_os = "windows")]
            {
                use tauri::Manager;
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }))
        .setup(|app| {
            // 알림 시스템 초기화
            if let Err(e) = init_notification_system(app) {
                eprintln!("Failed to initialize notification system: {}", e);
            }
            
            // Deep Link 리스너 설정
            let app_handle_for_deeplink = app.handle().clone();
            
            // 로그 파일 설정
            let log_path = std::env::temp_dir().join("mcplink_debug.log");
            let log_path_str = log_path.to_string_lossy().to_string();
            eprintln!("DEBUG LOG FILE: {}", log_path_str);
            
            // 디버그용 로그 파일에 시작 메시지 기록
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path) {
                use std::io::Write;
                let _ = writeln!(file, "=== MCPLink Debug Log Started at {} ===", 
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
                let _ = writeln!(file, "Waiting for deep link events...");
            }
            
            // 키워드 파일 읽기 함수 등록
            // 알림에서 저장된 키워드를 읽어 처리
            let app_handle_clone = app.handle().clone();
            
            // Tokio 런타임을 명시적으로 시작하여 비동기 작업 실행
            // 별도의 스레드에서 표준 스레드 API를 사용하여 비동기 작업 실행
            std::thread::spawn(move || {
                // 잠시 대기 후 키워드 파일 확인
                std::thread::sleep(std::time::Duration::from_secs(1));
                
                // 키워드 파일 경로
                let keyword_path = std::env::temp_dir().join("mcplink_last_keyword.txt");
                
                // 파일이 존재하면 읽기 시도
                if keyword_path.exists() {
                    if let Ok(keyword) = std::fs::read_to_string(&keyword_path) {
                        if !keyword.is_empty() {
                            // 로그 파일에 기록
                            let log_path = std::env::temp_dir().join("mcplink_activation.log");
                            if let Ok(mut file) = std::fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .append(true)
                                .open(&log_path) {
                                use std::io::Write;
                                let _ = writeln!(file, "[{}] 임시 파일에서 키워드 읽음: {}", 
                                    chrono::Local::now().format("%H:%M:%S"), keyword);
                            }
                            
                            // 키워드를 세션 스토리지에 저장하도록 이벤트 발생
                            if let Some(window) = app_handle_clone.get_webview_window("main") {
                                let _ = window.emit("search-keyword", &keyword);
                                
                                // 로그 파일에 기록
                                if let Ok(mut file) = std::fs::OpenOptions::new()
                                    .create(true)
                                    .write(true)
                                    .append(true)
                                    .open(&log_path) {
                                    use std::io::Write;
                                    let _ = writeln!(file, "[{}] search-keyword 이벤트 발생: {}", 
                                        chrono::Local::now().format("%H:%M:%S"), keyword);
                                }
                            } else {
                                // 로그 파일에 기록
                                if let Ok(mut file) = std::fs::OpenOptions::new()
                                    .create(true)
                                    .write(true)
                                    .append(true)
                                    .open(&log_path) {
                                    use std::io::Write;
                                    let _ = writeln!(file, "[{}] 창을 찾을 수 없어 이벤트 발생 실패", 
                                        chrono::Local::now().format("%H:%M:%S"));
                                }
                            }
                        }
                    }
                    
                    // 읽은 후 파일 삭제
                    let _ = std::fs::remove_file(&keyword_path);
                }
            });
            
            // deep-link 이벤트 리스닝
            let log_path_clone = log_path.clone();
            let _ = app.listen("deep-link://new-url", move |event| {
                // 이벤트 페이로드를 문자열로 처리
                let url = event.payload().to_string();
                let app_handle = app_handle_for_deeplink.clone();
                eprintln!("Deep Link received: {}", url);
                
                // 디버그 로그 파일에 기록
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&log_path_clone) {
                    use std::io::Write;
                    let _ = writeln!(file, "[{}] Deep Link received: {}", 
                        chrono::Local::now().format("%H:%M:%S"), url);
                }
                
                // URL 파싱
                // URL 파싱 - 더 자세한 디버깅 로그 추가
                eprintln!("Processing URL: {}", url);
                
                // 로그 파일에 기록
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&log_path_clone) {
                    use std::io::Write;
                    let _ = writeln!(file, "[{}] Processing URL: {}", 
                        chrono::Local::now().format("%H:%M:%S"), url);
                }
                
                // mcplink 프로토콜 확인 (URL 형식에 따라 검사 방식 조정)
                if url.contains("mcplink") {
                    eprintln!("mcplink protocol detected");
                    
                    // 로그 파일에 기록
                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(true)
                        .open(&log_path_clone) {
                        use std::io::Write;
                        let _ = writeln!(file, "[{}] mcplink protocol detected", 
                            chrono::Local::now().format("%H:%M:%S"));
                    }
                    
                    // 키워드 추출 - 다양한 URL 형식 처리
                    let keyword = if url.contains("keyword=") {
                        eprintln!("keyword parameter found");
                        
                        // 로그 파일에 기록
                        if let Ok(mut file) = std::fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .append(true)
                            .open(&log_path_clone) {
                            use std::io::Write;
                            let _ = writeln!(file, "[{}] keyword parameter found", 
                                chrono::Local::now().format("%H:%M:%S"));
                        }
                        
                        let parts: Vec<&str> = url.split("keyword=").collect();
                        if parts.len() > 1 {
                            let extracted = parts[1].trim().to_string();
                            // ? 또는 & 이후 부분 제거
                            let clean_keyword = if let Some(pos) = extracted.find(&['?', '&'][..]) {
                                extracted[..pos].to_string()
                            } else {
                                extracted
                            };
                            eprintln!("Extracted keyword: {}", clean_keyword);
                            
                            // 로그 파일에 기록
                            if let Ok(mut file) = std::fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .append(true)
                                .open(&log_path_clone) {
                                use std::io::Write;
                                let _ = writeln!(file, "[{}] Extracted keyword: {}", 
                                    chrono::Local::now().format("%H:%M:%S"), clean_keyword);
                            }
                            
                            Some(clean_keyword)
                        } else {
                            eprintln!("Cannot extract keyword from parts");
                            
                            // 로그 파일에 기록
                            if let Ok(mut file) = std::fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .append(true)
                                .open(&log_path_clone) {
                                use std::io::Write;
                                let _ = writeln!(file, "[{}] Cannot extract keyword from parts", 
                                    chrono::Local::now().format("%H:%M:%S"));
                            }
                            
                            None
                        }
                    } else {
                        eprintln!("No keyword parameter found");
                        
                        // 로그 파일에 기록
                        if let Ok(mut file) = std::fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .append(true)
                            .open(&log_path_clone) {
                            use std::io::Write;
                            let _ = writeln!(file, "[{}] No keyword parameter found", 
                                chrono::Local::now().format("%H:%M:%S"));
                        }
                        
                        None
                    };
                    
                    // 앱 활성화
                    let window_result = app_handle.get_webview_window("main");
                    
                    // 로그 파일에 기록
                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(true)
                        .open(&log_path_clone) {
                        use std::io::Write;
                        let _ = writeln!(file, "[{}] Window lookup result: {}", 
                            chrono::Local::now().format("%H:%M:%S"), 
                            if window_result.is_some() { "success" } else { "failed" });
                    }
                    
                    if let Some(window) = window_result {
                        // 앱 창 활성화 시도
                        let show_result = window.show();
                        let unminimize_result = window.unminimize();
                        let focus_result = window.set_focus();
                        
                        // 로그 파일에 기록
                        if let Ok(mut file) = std::fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .append(true)
                            .open(&log_path_clone) {
                            use std::io::Write;
                            let _ = writeln!(file, "[{}] Window activation: show={:?}, unminimize={:?}, focus={:?}", 
                                chrono::Local::now().format("%H:%M:%S"),
                                show_result.is_ok(),
                                unminimize_result.is_ok(),
                                focus_result.is_ok());
                        }
                        
                        // 키워드가 있으면 검색 이벤트 발생
                        if let Some(kw) = keyword {
                            // 단순화: 키워드만 전달하는 방식으로 변경
                            let emit_result = window.emit("search-keyword", kw.clone());
                            
                            // 로그 파일에 기록
                            if let Ok(mut file) = std::fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .append(true)
                                .open(&log_path_clone) {
                                use std::io::Write;
                                let _ = writeln!(file, "[{}] Emitting search-keyword event with '{}': {:?}", 
                                    chrono::Local::now().format("%H:%M:%S"),
                                    kw,
                                    emit_result.is_ok());
                            }
                        }
                    } else {
                        // 창을 찾을 수 없는 경우 에러 로그
                        eprintln!("Failed to find main window!");
                        
                        // 로그 파일에 기록
                        if let Ok(mut file) = std::fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .append(true)
                            .open(&log_path_clone) {
                            use std::io::Write;
                            let _ = writeln!(file, "[{}] ERROR: Failed to find main window!", 
                                chrono::Local::now().format("%H:%M:%S"));
                            
                            // 대안으로 새 창 생성 시도
                            let _ = writeln!(file, "[{}] Attempting to create a new window...", 
                                chrono::Local::now().format("%H:%M:%S"));
                            
                            // 키워드 정보 로깅
                            if let Some(kw) = keyword.clone() {
                                let _ = writeln!(file, "[{}] Keyword found: {}", 
                                    chrono::Local::now().format("%H:%M:%S"), kw);
                            } else {
                                let _ = writeln!(file, "[{}] No keyword found", 
                                    chrono::Local::now().format("%H:%M:%S"));
                            };
                            
                            // 창 목록 수동 확인
                            let _ = writeln!(file, "[{}] Window lookup failed - manually check for main window", 
                                chrono::Local::now().format("%H:%M:%S"));
                            
                            // 메인 윈도우가 있는지 명시적으로 확인 
                            let main_window = app_handle.get_webview_window("main");
                            let _ = writeln!(file, "[{}] Manual main window check: {}", 
                                chrono::Local::now().format("%H:%M:%S"),
                                if main_window.is_some() { "found" } else { "not found" });
                        }
                    }
                }
            });
            // Create menu items
            let open_item = MenuItemBuilder::with_id("open", "Open").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let hide_item = MenuItemBuilder::with_id("hide", "Hide").build(app)?;
            let show_item = MenuItemBuilder::with_id("show", "Show").build(app)?;

            // Create menu
            let menu = MenuBuilder::new(app)
                .item(&open_item)
                .separator()
                .item(&hide_item)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            // Create tray icon
            let _tray = TrayIconBuilder::new()
                .tooltip("MCP Link")
                .icon(app.default_window_icon().cloned().unwrap())
                .menu(&menu)
                .on_menu_event(move |app_handle, event| match event.id().as_ref() {
                    "quit" => {
                        app_handle.exit(0);
                    }
                    "open" | "show" => {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray_handle, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app_handle = tray_handle.app_handle();
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            
            // --- Start of Axum server startup code addition ---
            let app_handle_for_axum = app.handle().clone();

            // 별도의 스레드에서 Tokio 런타임을 초기화하고 Axum 서버 시작
            std::thread::spawn(move || {
                // 새로운 Tokio 런타임 생성
                let rt = tokio::runtime::Runtime::new().unwrap();
                
                // Tokio 런타임에서 비동기 작업 실행
                rt.block_on(async {
                    // Set AppHandle
                    recommendation_server_state_clone
                        .set_app_handle(app_handle_for_axum)
                        .await;
    
                    // Start Axum server
                    match start_axum_server(recommendation_server_state_clone).await {
                        Ok(_) => {}
                        Err(_e) => {}
                    }
                });
            });

            // --- End of Axum server startup code addition ---

            Ok(())
        })
        .manage(app_state) // Manage AppState with Tauri
        .invoke_handler(tauri::generate_handler![
            commands::get_mcp_data,
            commands::get_mcp_detail_data,
            commands::add_mcp_server_config,
            commands::remove_mcp_server_config,
            commands::restart_claude_desktop,
            commands::get_installed_mcp_data,
            commands::read_mcplink_config_content,
            commands::check_claude_config_exists,
            commands::check_mcplink_config_exists,
            commands::read_mcp_server_config,
            commands::is_mcp_server_installed,
            commands::reset_mcp_settings,
            commands::search_local_mcp_servers,
            commands::ensure_config_files,
            commands::start_config_watch,
            commands::test_force_activate,
            commands::test_search_keyword,
            commands::simulate_notification_click,
            notification_system::show_notification,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}