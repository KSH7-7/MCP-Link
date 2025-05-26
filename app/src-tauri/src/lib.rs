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
use dotenvy::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{env, fmt, net::SocketAddr, sync::Arc};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Listener, Manager,
};
use tokio::sync::Mutex;

pub mod commands;
pub mod force_activate;
pub mod notification_system;
use crate::commands::AppState;
use crate::notification_system::{init_notification_system, KeywordState};

/// 애플리케이션의 표준화된 오류 타입
#[derive(Debug, Clone, Serialize)]
pub enum AppError {
    /// API 요청 관련 오류
    ApiError { msg: String, status_code: Option<u16> },
    
    /// 파일 시스템 오류
    FileSystemError { msg: String, path: Option<String> },
    
    /// 구성 파일 오류
    ConfigError { msg: String },
    
    /// JSON 파싱 또는 직렬화 오류
    JsonError { msg: String },
    
    /// OS/환경 관련 오류
    OsError { msg: String },
    
    /// 알림 시스템 오류
    NotificationError { msg: String },
    
    /// 권한 오류
    PermissionError { msg: String },
    
    /// 네트워크 오류
    NetworkError { msg: String },
    
    /// 일반 오류
    GenericError { msg: String },
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ApiError { msg, status_code } => {
                if let Some(code) = status_code {
                    write!(f, "API 오류 ({}): {}", code, msg)
                } else {
                    write!(f, "API 오류: {}", msg)
                }
            }
            AppError::FileSystemError { msg, path } => {
                if let Some(p) = path {
                    write!(f, "파일 시스템 오류 (경로: {}): {}", p, msg)
                } else {
                    write!(f, "파일 시스템 오류: {}", msg)
                }
            }
            AppError::ConfigError { msg } => write!(f, "설정 오류: {}", msg),
            AppError::JsonError { msg } => write!(f, "JSON 오류: {}", msg),
            AppError::OsError { msg } => write!(f, "OS 오류: {}", msg),
            AppError::NotificationError { msg } => write!(f, "알림 시스템 오류: {}", msg),
            AppError::PermissionError { msg } => write!(f, "권한 오류: {}", msg),
            AppError::NetworkError { msg } => write!(f, "네트워크 오류: {}", msg),
            AppError::GenericError { msg } => write!(f, "오류: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

/// 오류 처리 결과 타입 (Result의 에러 타입으로 AppError 사용)
pub type AppResult<T> = Result<T, AppError>;

/// 문자열에서 AppError::GenericError로 변환
impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::GenericError { msg: s }
    }
}

/// &str에서 AppError::GenericError로 변환
impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::GenericError { msg: s.to_string() }
    }
}

/// std::io::Error에서 AppError::FileSystemError로 변환
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::FileSystemError {
            msg: e.to_string(),
            path: None,
        }
    }
}

/// reqwest::Error에서 AppError::NetworkError로 변환
impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::NetworkError { msg: e.to_string() }
    }
}

/// serde_json::Error에서 AppError::JsonError로 변환
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::JsonError { msg: e.to_string() }
    }
}

/// 기존 코드와의 호환성을 위한 헬퍼 함수
/// AppError를 문자열로 변환 (Tauri 명령을 위한 Result<T, String> 호환)
pub fn to_string_error<T, E: Into<AppError>>(result: Result<T, E>) -> Result<T, String> {
    result.map_err(|e| e.into().to_string())
}

/// 로그 레벨 정의
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// 디버깅용 상세 정보
    Debug,
    /// 정보성 메시지
    Info,
    /// 경고 메시지
    Warning,
    /// 오류 메시지
    Error,
}

impl LogLevel {
    /// 로그 레벨을 문자열로 변환
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

/// 로그 카테고리 정의
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogCategory {
    /// 앱 활성화 관련 로그
    Activation,
    /// 알림 관련 로그
    Notification,
    /// 키워드 처리 관련 로그
    Keyword,
    /// 일반 오류 로그
    Error,
    /// 기타 일반 로그
    General,
}

impl LogCategory {
    /// 로그 카테고리를 문자열로 변환
    pub fn as_str(&self) -> &'static str {
        match self {
            LogCategory::Activation => "ACTIVATION",
            LogCategory::Notification => "NOTIFICATION",
            LogCategory::Keyword => "KEYWORD",
            LogCategory::Error => "ERROR",
            LogCategory::General => "GENERAL",
        }
    }
    
    /// 로그 카테고리에 해당하는 파일 이름 가져오기
    pub fn get_file_name(&self) -> &'static str {
        match self {
            LogCategory::Activation => "mcplink_activation.log",
            LogCategory::Notification => "mcplink_notification.log",
            LogCategory::Keyword => "mcplink_keyword.log",
            LogCategory::Error => "mcplink_error.log",
            LogCategory::General => "mcplink_general.log",
        }
    }
}

/// 통합 로그 기록 유틸리티 함수
/// 
/// 모든 로깅을 일관된 포맷으로 처리합니다.
pub fn log_message(
    level: LogLevel,
    category: LogCategory,
    context: &str,
    message: &str,
) {
    // 로그 파일 경로
    let log_path = std::env::temp_dir().join(category.get_file_name());
    
    // 로그 기록 시도
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] [{}] [{}] [{}] {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            level.as_str(),
            category.as_str(),
            context,
            message
        );
    }
    
    // 통합 로그에도 기록 (모든 로그를 한 파일에 모음)
    let all_logs_path = std::env::temp_dir().join("mcplink_all.log");
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&all_logs_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] [{}] [{}] [{}] {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            level.as_str(),
            category.as_str(),
            context,
            message
        );
    }
}

/// 오류 로깅 유틸리티 함수
/// 오류를 파일에 기록하고 AppError로 다시 반환
pub fn log_error<E: Into<AppError>>(error: E, context: &str) -> AppError {
    let app_error = error.into();
    
    // 새로운 로깅 시스템 사용
    log_message(
        LogLevel::Error,
        LogCategory::Error,
        context,
        &format!("{}: {}", app_error.get_type(), app_error)
    );
    
    app_error
}

/// 파일 경로가 포함된 파일 시스템 오류 생성 유틸리티
pub fn fs_error<E: Into<AppError>>(error: E, path: &str) -> AppError {
    let app_error = error.into();
    
    match app_error {
        // 이미 FileSystemError인 경우 path 정보만 업데이트
        AppError::FileSystemError { msg, .. } => AppError::FileSystemError {
            msg,
            path: Some(path.to_string()),
        },
        // 다른 오류 타입은 새 FileSystemError로 변환
        _ => AppError::FileSystemError {
            msg: app_error.to_string(),
            path: Some(path.to_string()),
        },
    }
}

/// API 오류 생성 유틸리티
pub fn api_error<E: Into<AppError>>(error: E, status_code: Option<u16>) -> AppError {
    let app_error = error.into();
    
    match app_error {
        // 이미 ApiError인 경우 status_code 정보만 업데이트
        AppError::ApiError { msg, .. } => AppError::ApiError {
            msg,
            status_code,
        },
        // 다른 오류 타입은 새 ApiError로 변환
        _ => AppError::ApiError {
            msg: app_error.to_string(),
            status_code,
        },
    }
}

impl AppError {
    /// 오류 종류를 문자열로 반환
    pub fn get_type(&self) -> &'static str {
        match self {
            AppError::ApiError { .. } => "API 오류",
            AppError::FileSystemError { .. } => "파일 시스템 오류",
            AppError::ConfigError { .. } => "설정 오류",
            AppError::JsonError { .. } => "JSON 오류",
            AppError::OsError { .. } => "OS 오류",
            AppError::NotificationError { .. } => "알림 오류",
            AppError::PermissionError { .. } => "권한 오류", 
            AppError::NetworkError { .. } => "네트워크 오류",
            AppError::GenericError { .. } => "일반 오류",
        }
    }
    
    /// 오류 메시지 반환
    pub fn message(&self) -> &str {
        match self {
            AppError::ApiError { msg, .. } => msg,
            AppError::FileSystemError { msg, .. } => msg,
            AppError::ConfigError { msg } => msg,
            AppError::JsonError { msg } => msg,
            AppError::OsError { msg } => msg,
            AppError::NotificationError { msg } => msg,
            AppError::PermissionError { msg } => msg,
            AppError::NetworkError { msg } => msg,
            AppError::GenericError { msg } => msg,
        }
    }
}

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
            let title = "MCP keyword recommendation";
            let body = format!("Click for keyword: {}", keywords_str);

            // 키워드 상태에 먼저 저장
            if let Some(keyword_state) = app_handle.try_state::<KeywordState>() {
                keyword_state.set_keyword(main_keyword.clone(), "recommendation", 1);
                
                // 로그 기록
                let log_path = std::env::temp_dir().join("mcplink_notification.log");
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&log_path)
                {
                    use std::io::Write;
                    let _ = writeln!(
                        file,
                        "[{}] 추천 키워드 수신: {}",
                        chrono::Local::now().format("%H:%M:%S"),
                        main_keyword
                    );
                }
            }

            // 네이티브 알림 표시 - app_handle 전달
            #[cfg(target_os = "windows")]
            {
                if let Err(e) = notification_system::show_windows_notification(
                    Some(app_handle),
                    title,
                    &body,
                    Some(main_keyword.to_string()),
                ) {
                    eprintln!(
                        "[Recommendation] Windows 알림 표시 실패: {}",
                        e
                    );
                    
                    // 알림 실패 시 직접 앱 활성화 및 키워드 처리
                    let _ = force_activate::force_app_to_foreground();
                    
                    if let Some(window) = app_handle.get_webview_window("main") {
                        use tauri::Emitter;
                        let _ = window.emit("notification-clicked", main_keyword);
                    }
                }
            }

            #[cfg(target_os = "macos")]
            {
                if let Err(e) = notification_system::show_macos_notification(
                    Some(app_handle),
                    title,
                    &body,
                    Some(main_keyword.to_string()),
                ) {
                    eprintln!("[Recommendation] macOS 알림 표시 실패: {}", e);
                }
            }

            #[cfg(target_os = "linux")]
            {
                if let Err(e) = notification_system::show_linux_notification(
                    Some(app_handle),
                    title,
                    &body,
                    Some(main_keyword.to_string()),
                ) {
                    eprintln!("[Recommendation] Linux 알림 표시 실패: {}", e);
                }
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
            "\n\n=== [{}] MCPLink 앱 시작됨 ===",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        );

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
                .open(&log_path)
            {
                use std::io::Write;
                let _ = writeln!(
                    file,
                    "=== MCPLink Debug Log Started at {} ===",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
                );
                let _ = writeln!(file, "Waiting for deep link events...");
            }

            // 키워드 파일 읽기 함수 등록
            // 알림에서 저장된 키워드를 읽어 처리
            let app_handle_clone = app.handle().clone();

            // Tokio 런타임을 명시적으로 시작하여 비동기 작업 실행
            // 별도의 스레드에서 표준 스레드 API를 사용하여 비동기 작업 실행
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(1));
                
                let keyword_path = std::env::temp_dir().join("mcplink_last_keyword.txt");
                
                if keyword_path.exists() {
                    // 파일의 수정 시간 확인
                    if let Ok(metadata) = std::fs::metadata(&keyword_path) {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(elapsed) = modified.elapsed() {
                                // 10초 이상 된 파일은 무시하고 삭제
                                if elapsed.as_secs() > 10 {
                                    let _ = std::fs::remove_file(&keyword_path);
                                    return;
                                }
                            }
                        }
                    }
                    
                    if let Ok(keyword) = std::fs::read_to_string(&keyword_path) {
                        if !keyword.is_empty() {
                            if let Some(window) = app_handle_clone.get_webview_window("main") {
                                let _ = window.emit("search-keyword", &keyword);
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
                    .open(&log_path_clone)
                {
                    use std::io::Write;
                    let _ = writeln!(
                        file,
                        "[{}] Deep Link received: {}",
                        chrono::Local::now().format("%H:%M:%S"),
                        url
                    );
                }

                // URL 파싱
                // URL 파싱 - 더 자세한 디버깅 로그 추가
                eprintln!("Processing URL: {}", url);

                // 로그 파일에 기록
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&log_path_clone)
                {
                    use std::io::Write;
                    let _ = writeln!(
                        file,
                        "[{}] Processing URL: {}",
                        chrono::Local::now().format("%H:%M:%S"),
                        url
                    );
                }

                // mcplink 프로토콜 확인 (URL 형식에 따라 검사 방식 조정)
                if url.contains("mcplink") {
                    eprintln!("mcplink protocol detected");

                    // 로그 파일에 기록
                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(true)
                        .open(&log_path_clone)
                    {
                        use std::io::Write;
                        let _ = writeln!(
                            file,
                            "[{}] mcplink protocol detected",
                            chrono::Local::now().format("%H:%M:%S")
                        );
                    }

                    // 키워드 추출 - 다양한 URL 형식 처리
                    let keyword = if url.contains("keyword=") {
                        eprintln!("keyword parameter found");

                        // 로그 파일에 기록
                        if let Ok(mut file) = std::fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .append(true)
                            .open(&log_path_clone)
                        {
                            use std::io::Write;
                            let _ = writeln!(
                                file,
                                "[{}] keyword parameter found",
                                chrono::Local::now().format("%H:%M:%S")
                            );
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
                                .open(&log_path_clone)
                            {
                                use std::io::Write;
                                let _ = writeln!(
                                    file,
                                    "[{}] Extracted keyword: {}",
                                    chrono::Local::now().format("%H:%M:%S"),
                                    clean_keyword
                                );
                            }

                            Some(clean_keyword)
                        } else {
                            eprintln!("Cannot extract keyword from parts");

                            // 로그 파일에 기록
                            if let Ok(mut file) = std::fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .append(true)
                                .open(&log_path_clone)
                            {
                                use std::io::Write;
                                let _ = writeln!(
                                    file,
                                    "[{}] Cannot extract keyword from parts",
                                    chrono::Local::now().format("%H:%M:%S")
                                );
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
                            .open(&log_path_clone)
                        {
                            use std::io::Write;
                            let _ = writeln!(
                                file,
                                "[{}] No keyword parameter found",
                                chrono::Local::now().format("%H:%M:%S")
                            );
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
                        .open(&log_path_clone)
                    {
                        use std::io::Write;
                        let _ = writeln!(
                            file,
                            "[{}] Window lookup result: {}",
                            chrono::Local::now().format("%H:%M:%S"),
                            if window_result.is_some() {
                                "success"
                            } else {
                                "failed"
                            }
                        );
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
                            .open(&log_path_clone)
                        {
                            use std::io::Write;
                            let _ = writeln!(
                                file,
                                "[{}] Window activation: show={:?}, unminimize={:?}, focus={:?}",
                                chrono::Local::now().format("%H:%M:%S"),
                                show_result.is_ok(),
                                unminimize_result.is_ok(),
                                focus_result.is_ok()
                            );
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
                                .open(&log_path_clone)
                            {
                                use std::io::Write;
                                let _ = writeln!(
                                    file,
                                    "[{}] Emitting search-keyword event with '{}': {:?}",
                                    chrono::Local::now().format("%H:%M:%S"),
                                    kw,
                                    emit_result.is_ok()
                                );
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
                            .open(&log_path_clone)
                        {
                            use std::io::Write;
                            let _ = writeln!(
                                file,
                                "[{}] ERROR: Failed to find main window!",
                                chrono::Local::now().format("%H:%M:%S")
                            );

                            // 대안으로 새 창 생성 시도
                            let _ = writeln!(
                                file,
                                "[{}] Attempting to create a new window...",
                                chrono::Local::now().format("%H:%M:%S")
                            );

                            // 키워드 정보 로깅
                            if let Some(kw) = keyword.clone() {
                                let _ = writeln!(
                                    file,
                                    "[{}] Keyword found: {}",
                                    chrono::Local::now().format("%H:%M:%S"),
                                    kw
                                );
                            } else {
                                let _ = writeln!(
                                    file,
                                    "[{}] No keyword found",
                                    chrono::Local::now().format("%H:%M:%S")
                                );
                            };

                            // 창 목록 수동 확인
                            let _ = writeln!(
                                file,
                                "[{}] Window lookup failed - manually check for main window",
                                chrono::Local::now().format("%H:%M:%S")
                            );

                            // 메인 윈도우가 있는지 명시적으로 확인
                            let main_window = app_handle.get_webview_window("main");
                            let _ = writeln!(
                                file,
                                "[{}] Manual main window check: {}",
                                chrono::Local::now().format("%H:%M:%S"),
                                if main_window.is_some() {
                                    "found"
                                } else {
                                    "not found"
                                }
                            );
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
            commands::check_and_mark_app_activated,
            commands::get_installed_count,
            commands::get_list_count,
            commands::notify_app_activated,
            commands::clear_keyword_state,
            commands::get_installed_mcp_count,
            notification_system::show_notification,
            notification_system::check_and_get_keyword,
            force_activate::activate_app_window,
            commands::delete_keyword_file,
            commands::check_keyword_file_age,

        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
