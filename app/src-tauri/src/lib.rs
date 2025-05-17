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
    AppHandle, Manager, async_runtime,
};
use tauri_plugin_notification::{NotificationExt, PermissionState};
use tokio::sync::Mutex;

pub mod commands;
use crate::commands::AppState;

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
    // Convert keywords to a comma-separated string
    let keywords_str = payload.keywords.join(", ");

    // Send notification using AppHandle
    if let Some(app_handle) = &*state.app_handle.lock().await {
        
        // Set the first keyword as pending keyword
        if let Some(first_keyword) = payload.keywords.first() {
            
            // First, initialize app state (if previous data exists)
            if let Err(e) = commands::clear_pending_notification_keyword() {
                eprintln!("[Recommendation] Failed to clear previous keywords: {}", e);
            }
            
            // Set pending notification keyword (for notification click handling)
            if let Err(e) = commands::set_pending_notification_keyword(first_keyword.clone()) {
                eprintln!("[Recommendation] Failed to set pending keyword: {}", e);
            } else {
            }
        }
        
        // Use modal notifications instead of system notifications
        if let Some(first_keyword) = payload.keywords.first() {
            // Start async task to show modal notification
            let app_handle_clone = app_handle.clone();
            let first_keyword_clone = first_keyword.clone();
            
            async_runtime::spawn(async move {
                if let Err(e) = commands::show_modal_notification(app_handle_clone, first_keyword_clone).await {
                    eprintln!("[Recommendation] Failed to show modal notification: {}", e);
                }
            });
        } else {
            // Fallback to system notification if no keywords
            let notification_body = format!("Selected keywords: {}. Click to check.", keywords_str);

            let builder = app_handle
                .notification()
                .builder()
                .title("New Recommended Keywords") // Title in English
                .body(&notification_body)
                .icon("icons/icon.png");

            match builder.show() {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("[Recommendation] Failed to send notification: {}", e);
                }
            }
        }

        // Use emit instead of emit_all - Explicitly use Emitter trait
        {
            use tauri::Emitter;
            let _ = app_handle.emit("new-keywords", payload.keywords.clone());
        }
    } else {
        eprintln!("[Recommendation] AppHandle not set, cannot send notification");
    }

    StatusCode::OK
}

// Handler for /api/v1 requests
async fn handle_api_v1_request(
    AxumState(state): AxumState<RecommendationServerState>,
) -> StatusCode {
    // Send notification using AppHandle
    if let Some(app_handle) = &*state.app_handle.lock().await {
        // Set notification body
        let notification_body = String::from("New data received. (From /api/v1)");

        // Create and display notification
        let builder = app_handle
            .notification()
            .builder()
            .title("API Notification") // Title in English
            .body(&notification_body)
            .icon("icons/icon.png");

        // Attempt to display notification and log result
        match builder.show() {
            Ok(_) => {}
            Err(_e) => {}
        }
    } else {
    }

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
        .plugin(tauri_plugin_notification::init()) // Initialize notification plugin
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
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
                .tooltip("Keyword Search")
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

            // --- Start of notification click handler modification ---
            let app_handle = app.handle().clone(); // Clone app_handle here to use in the closure below

            // Check and log notification permission state on app start
            if let Ok(permission_state) = app_handle.notification().permission_state() {
                if permission_state != PermissionState::Granted {
                    // If permission not granted, request it
                    if let Ok(_new_state) = app_handle.notification().request_permission() {
                        // Permission request sent, new state can be handled if needed
                    }
                }
            }

            // Common function to extract tag
            fn extract_tag_from_body(body: &str) -> Option<String> {
                // Extracts TAG from "Selected keywords: TAG. Click to check."
                if let Some(start) = body.find("Selected keywords: ") { // Find the English part for tag extraction
                    let start_idx = start + "Selected keywords: ".len();
                    if let Some(end) = body[start_idx..].find(". ") {
                        let tag = body[start_idx..(start_idx + end)].to_string();
                        return Some(tag);
                    }
                }
                None
            }

            // Notification handler function
            fn handle_notification(body: &str, app_handle: tauri::AppHandle) {
                // Get main window
                if let Some(window) = app_handle.get_webview_window("main") {
                    // Extract tag from notification body
                    let final_tag = extract_tag_from_body(body).unwrap_or_else(|| {
                        // Default tag if extraction fails
                        "GOOGLE".to_string()
                    });

                    // Use our new modal notification approach instead of system notifications
                    // This is handled by the show_modal_notification command
                    let app_handle_clone = app_handle.clone();
                    let _ = async_runtime::spawn(async move {
                        let _ = commands::show_modal_notification(app_handle_clone, final_tag).await;
                    });
                }
            }
            // --- End of notification click handler modification ---

            // --- Start of Axum server startup code addition ---
            let app_handle_for_axum = app.handle().clone();

            // Set AppHandle and start Axum server
            tauri::async_runtime::spawn(async move {
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

            // --- End of Axum server startup code addition ---

            Ok(())
        })
        .manage(app_state) // Manage AppState with Tauri
        .invoke_handler(tauri::generate_handler![
            commands::show_popup,
            commands::show_modal_notification,
            commands::activate_main_window_with_keyword,
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
            commands::set_pending_notification_keyword,
            commands::get_pending_notification_keyword,
            commands::clear_pending_notification_keyword,
            commands::check_and_mark_app_activated,
            commands::is_app_active,
            commands::search_local_mcp_servers,
            commands::ensure_config_files,
            commands::start_config_watch,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}