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

// standardized error type for the application
#[derive(Debug, Clone, Serialize)]
pub enum AppError {
    // API request related error
    ApiError {
        msg: String,
        status_code: Option<u16>,
    },

    // file system error
    FileSystemError {
        msg: String,
        path: Option<String>,
    },

    // configuration file error
    ConfigError {
        msg: String,
    },

    // JSON parsing or serialization error
    JsonError {
        msg: String,
    },

    // OS/environment related error
    OsError {
        msg: String,
    },

    // notification system error
    NotificationError {
        msg: String,
    },

    // permission error
    PermissionError {
        msg: String,
    },

    // network error
    NetworkError {
        msg: String,
    },

    // general error
    GenericError {
        msg: String,
    },
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ApiError { msg, status_code } => {
                if let Some(code) = status_code {
                    write!(f, "API error ({}): {}", code, msg)
                } else {
                    write!(f, "API error: {}", msg)
                }
            }
            AppError::FileSystemError { msg, path } => {
                if let Some(p) = path {
                    write!(f, "file system error (path: {}): {}", p, msg)
                } else {
                    write!(f, "file system error: {}", msg)
                }
            }
            AppError::ConfigError { msg } => write!(f, "configuration error: {}", msg),
            AppError::JsonError { msg } => write!(f, "JSON error: {}", msg),
            AppError::OsError { msg } => write!(f, "OS error: {}", msg),
            AppError::NotificationError { msg } => write!(f, "notification system error: {}", msg),
            AppError::PermissionError { msg } => write!(f, "permission error: {}", msg),
            AppError::NetworkError { msg } => write!(f, "network error: {}", msg),
            AppError::GenericError { msg } => write!(f, "error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

// error handling result type (use AppError as the error type for Result)
pub type AppResult<T> = Result<T, AppError>;

// convert String to AppError::GenericError
impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::GenericError { msg: s }
    }
}

// convert &str to AppError::GenericError
impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::GenericError { msg: s.to_string() }
    }
}

// convert std::io::Error to AppError::FileSystemError
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::FileSystemError {
            msg: e.to_string(),
            path: None,
        }
    }
}

// convert reqwest::Error to AppError::NetworkError
impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::NetworkError { msg: e.to_string() }
    }
}

// convert serde_json::Error to AppError::JsonError
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::JsonError { msg: e.to_string() }
    }
}

// helper function for compatibility with existing code
// convert AppError to String (compatible with Result<T, String> for Tauri commands)
pub fn to_string_error<T, E: Into<AppError>>(result: Result<T, E>) -> Result<T, String> {
    result.map_err(|e| e.into().to_string())
}

// file path included file system error creation utility
pub fn fs_error<E: Into<AppError>>(error: E, path: &str) -> AppError {
    let app_error = error.into();

    match app_error {
        // if already FileSystemError, update only path information
        AppError::FileSystemError { msg, .. } => AppError::FileSystemError {
            msg,
            path: Some(path.to_string()),
        },
        // other error types are converted to new FileSystemError
        _ => AppError::FileSystemError {
            msg: app_error.to_string(),
            path: Some(path.to_string()),
        },
    }
}

// API error creation utility
pub fn api_error<E: Into<AppError>>(error: E, status_code: Option<u16>) -> AppError {
    let app_error = error.into();

    match app_error {
        // if already ApiError, update only status_code information
        AppError::ApiError { msg, .. } => AppError::ApiError { msg, status_code },
        // other error types are converted to new ApiError
        _ => AppError::ApiError {
            msg: app_error.to_string(),
            status_code,
        },
    }
}

impl AppError {
    // return error type as String
    pub fn get_type(&self) -> &'static str {
        match self {
            AppError::ApiError { .. } => "API error",
            AppError::FileSystemError { .. } => "file system error",
            AppError::ConfigError { .. } => "configuration error",
            AppError::JsonError { .. } => "JSON error",
            AppError::OsError { .. } => "OS error",
            AppError::NotificationError { .. } => "notification system error",
            AppError::PermissionError { .. } => "permission error",
            AppError::NetworkError { .. } => "network error",
            AppError::GenericError { .. } => "error",
        }
    }

    // return error message
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
    // receive keywords and display notifications and trigger events
    if let Some(app_handle) = &*state.app_handle.lock().await {
        // extract the first keyword (use as main keyword)
        if let Some(main_keyword) = payload.keywords.first() {
            // join all keywords into a string
            let keywords_str = payload.keywords.join(", ");

            // set notification title and content
            let title = "MCP keyword recommendation";
            let body = format!("Click for keyword: {}", keywords_str);

            // save keyword state first
            if let Some(keyword_state) = app_handle.try_state::<KeywordState>() {
                keyword_state.set_keyword(main_keyword.clone(), "recommendation", 1);
            }

            // display native notification - pass app_handle
            #[cfg(target_os = "windows")]
            {
                if let Err(e) = notification_system::show_windows_notification(
                    Some(app_handle),
                    title,
                    &body,
                    Some(main_keyword.to_string()),
                ) {
                    // notification failed, force app to foreground and process keyword
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
                ) {}
            }

            #[cfg(target_os = "linux")]
            {
                if let Err(e) = notification_system::show_linux_notification(
                    Some(app_handle),
                    title,
                    &body,
                    Some(main_keyword.to_string()),
                ) {}
            }
        }

        // trigger keyword event (for UI response)
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
            // when a new instance is running, focus on the existing window
            app.get_webview_window("main")
                .expect("main window not found")
                .set_focus()
                .unwrap();

            // if needed, restore window and bring to front
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
            // initialize notification system
            if let Err(e) = init_notification_system(app) {}

            // Deep Link listener setup
            let app_handle_for_deeplink = app.handle().clone();

            // debug log file removed for performance

            // register keyword file read function
            // read keywords saved in notifications
            let app_handle_clone = app.handle().clone();

            // start Tokio runtime explicitly for asynchronous operations
            // use standard thread API in a separate thread for asynchronous operations
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(1));

                let keyword_path = std::env::temp_dir().join("mcplink_last_keyword.txt");

                if keyword_path.exists() {
                    // check file modification time
                    if let Ok(metadata) = std::fs::metadata(&keyword_path) {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(elapsed) = modified.elapsed() {
                                // ignore files older than 10 seconds and delete
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

                    // delete file after reading
                    let _ = std::fs::remove_file(&keyword_path);
                }
            });

            // deep-link event listener
            let _ = app.listen("deep-link://new-url", move |event| {
                // handle event payload as a string
                let url = event.payload().to_string();
                let app_handle = app_handle_for_deeplink.clone();

                // check mcplink protocol (adjust check method based on URL format)
                if url.contains("mcplink") {
                    // extract keyword - handle various URL formats
                    let keyword = if url.contains("keyword=") {
                        let parts: Vec<&str> = url.split("keyword=").collect();
                        if parts.len() > 1 {
                            let extracted = parts[1].trim().to_string();
                            // remove ? or & after the keyword
                            let clean_keyword = if let Some(pos) = extracted.find(&['?', '&'][..]) {
                                extracted[..pos].to_string()
                            } else {
                                extracted
                            };

                            Some(clean_keyword)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // activate app
                    let window_result = app_handle.get_webview_window("main");

                    if let Some(window) = window_result {
                        // attempt to activate app window
                        let _show_result = window.show();
                        let _unminimize_result = window.unminimize();
                        let _focus_result = window.set_focus();

                        // if keyword exists, trigger search event
                        if let Some(kw) = keyword {
                            let _emit_result = window.emit("search-keyword", kw.clone());
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

            // start Tokio runtime in a separate thread and Axum server
            std::thread::spawn(move || {
                // create new Tokio runtime
                let rt = tokio::runtime::Runtime::new().unwrap();

                // run asynchronous operations in Tokio runtime
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
