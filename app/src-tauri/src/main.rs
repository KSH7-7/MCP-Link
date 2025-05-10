// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands; // 기존 commands 모듈
use tauri::Emitter; // Emitter 트레잇 추가

// --- 필요한 use 선언들 ---
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post, // POST 요청을 받기 위해 post 사용
    Json, Router,
};
// commands.rs에 정의된 AppState, ApiResponse, DataWrapper, MCPCard 사용
use commands::{AppState, /*ApiResponse, DataWrapper,*/ MCPCard}; // ApiResponse, DataWrapper는 핸들러 내부에서 직접 사용하지 않으면 여기서 제거 가능
use serde::Deserialize; // 요청 본문 파싱용
use std::env; // CRAWLER_API_BASE_URL 환경 변수 읽기용
use std::net::SocketAddr;
use tauri::Manager; // AppHandle을 통해 프론트엔드 이벤트 발생 및 알림을 위해 필요
use urlencoding::encode; // URL 인코딩용
use tauri_plugin_notification::NotificationExt; // Notification 플러그인 사용 (Tauri v2 가정)

// --- Fallback MCP Server로부터 받을 요청 본문 구조체 ---
#[derive(Deserialize, Debug, Clone)]
struct RecommendationRequest {
    keywords: Vec<String>,
}

// --- axum 핸들러: Fallback MCP Server로부터 추천 요청을 받는 엔드포인트 ---
// 경로: /api/v1/recommendation, 메소드: POST
#[axum::debug_handler] // 디버그 핸들러 추가
async fn recommendation_handler(
    State(app_state): State<AppState>, // commands.rs의 AppState 접근
    app_handle: axum::extract::Extension<tauri::AppHandle>, // Tauri AppHandle (Extension으로 주입됨)
    Json(payload): Json<RecommendationRequest>,
) -> impl IntoResponse {
    println!("[GUI Backend] Received POST request on /api/v1/recommendation with payload: {:?}", payload);

    let search_keyword: String;
    let notification_display_keyword: String;

    if payload.keywords.is_empty() {
        println!("[GUI Backend] No keywords received. Performing a general search.");
        search_keyword = "".to_string();
        notification_display_keyword = "전체".to_string();
    } else {
        search_keyword = payload.keywords[0].clone();
        notification_display_keyword = search_keyword.clone();
        println!("[GUI Backend] Using primary keyword for search: {}", search_keyword);
    }

    // --- 1. Tauri Notification으로 팝업 알림 발생 ---
    let notification_title = "MCP 추천 요청".to_string();
    let notification_body = format!(
        "키워드 '{}' 관련 MCP 서버 추천을 확인하세요. 클릭 시 앱에서 검색됩니다.",
        notification_display_keyword
    );
    let notification_event_payload = serde_json::json!({
        "title": notification_title,
        "body": notification_body,
        "keyword_for_search": search_keyword.clone() // 알림 클릭 시 이 키워드로 자동 검색
    });

    // app_handle은 axum::extract::Extension으로 감싸져 있으므로 .0으로 내부 값에 접근
    // emit_all 대신 컴파일러가 제안하는 emit 사용
    if let Err(e) = app_handle.0.emit("show-recommendation-notification", notification_event_payload) {
        eprintln!("[GUI Backend] Failed to emit notification event to frontend: {}", e);
    }

    // --- 2. Crawler Server에 MCP 서버 검색 요청 ---
    let base_url: String = match env::var("CRAWLER_API_BASE_URL") {
        Ok(url_val) => url_val,
        Err(e) => {
            let msg = format!("[GUI Backend] CRAWLER_API_BASE_URL not set: {}", e);
            eprintln!("{}", msg);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": msg}))).into_response();
        }
    };

    let crawler_api_url = if search_keyword.is_empty() {
        base_url
    } else {
        let encoded_term = encode(&search_keyword);
        format!("{}/search?name={}", base_url, encoded_term)
    };

    println!("[GUI Backend] Requesting MCP servers from Crawler: {}", crawler_api_url);

    match app_state.client.get(&crawler_api_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                // commands.rs의 ApiResponse와 DataWrapper를 사용 (이미 pub으로 변경됨)
                match response.json::<commands::ApiResponse>().await { 
                    Ok(api_response) => {
                        println!("[GUI Backend] Successfully parsed ApiResponse from Crawler. Code: {}, Message: {}", api_response.code, api_response.message);
                        
                        match serde_json::from_value::<commands::DataWrapper>(api_response.data.clone()) {
                            Ok(data_wrapper) => {
                                let mcp_cards: Vec<MCPCard> = data_wrapper
                                    .mcpServers // DataWrapper의 필드명 mcpServers
                                    .iter()
                                    .map(|api_card| MCPCard { 
                                        id: api_card.id,
                                        title: api_card.mcpServers.name.clone(), 
                                        description: api_card.mcpServers.description.clone(),
                                        url: api_card.url.clone(),
                                        stars: api_card.stars,
                                    })
                                    .collect();
                                
                                println!("[GUI Backend] Successfully parsed {} MCP cards from DataWrapper.", mcp_cards.len());
                                if !mcp_cards.is_empty() {
                                    // emit_all 대신 컴파일러가 제안하는 emit 사용
                                    if let Err(e) = app_handle.0.emit("mcp-search-results", mcp_cards) {
                                        eprintln!("[GUI Backend] Failed to emit mcp-search-results to frontend: {}", e);
                                    }
                                }
                                (StatusCode::OK, Json(serde_json::json!({"message": "Recommendation request received and processed."}))).into_response()
                            }
                            Err(e) => {
                                eprintln!("[GUI Backend] Failed to parse DataWrapper from ApiResponse.data: {}. Data was: {:?}", e, api_response.data);
                                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Failed to parse crawler data wrapper"}))).into_response()
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[GUI Backend] Failed to parse ApiResponse JSON from Crawler: {}", e);
                        (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Failed to parse crawler response structure"}))).into_response()
                    }
                }
            } else {
                let status = response.status();
                let error_body = response.text().await.unwrap_or_else(|_| "Unknown error from Crawler".to_string());
                eprintln!("[GUI Backend] Crawler server returned error: {} - {}", status, error_body);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Crawler server error", "details": error_body}))).into_response()
            }
        }
        Err(e) => {
            eprintln!("[GUI Backend] Failed to send request to Crawler: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Failed to connect to crawler server"}))).into_response()
        }
    }
}

#[tokio::main] // Tokio 런타임 활성화
async fn main() { // async fn으로 변경
    // .env 파일 로드
    if dotenvy::dotenv().is_err() {
        println!("[GUI Backend] Warning: .env file not found or failed to load. Relying on system environment variables.");
    }

    let client = reqwest::Client::new();
    // AppState를 commands.rs에 정의된 것을 사용
    let app_state = commands::AppState { client };

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init()) // Notification 플러그인 초기화
        .manage(app_state.clone()) // AppState 등록 (axum 핸들러에서 사용)
        .setup(move |app| {
            let app_handle_for_axum = app.handle().clone(); // axum 스레드에서 사용할 AppHandle
            // Tauri에 등록된 AppState를 가져와서 axum 스레드에서 사용할 수 있도록 복사
            let axum_app_state = app.state::<commands::AppState>().inner().clone(); 

            tokio::spawn(async move {
                let app_router = Router::new()
                    // 지정된 경로 /api/v1/recommendation 으로 POST 요청을 받는 라우트
                    .route("/api/v1/recommendation", post(recommendation_handler))
                    .with_state(axum_app_state) // 모든 라우트에 AppState 공유
                    .layer(axum::extract::Extension(app_handle_for_axum.clone())); // AppHandle을 Extension으로 제공

                // GUI_BE_API_BASE_URL 환경변수에서 포트 번호는 8081로 고정되어 있음
                let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
                println!("[GUI Backend - Axum] Listening on http://{}", addr);
                println!("[GUI Backend - Axum] Recommendation endpoint available at POST http://{}/api/v1/recommendation", addr);

                let listener = tokio::net::TcpListener::bind(addr).await.unwrap_or_else(|e| {
                    eprintln!("[GUI Backend - Axum] Failed to bind to address {}: {}", addr, e);
                    // 바인딩 실패 시 앱이 패닉하지 않도록 처리 (예: 로깅 후 종료 또는 다른 포트 시도)
                    // 여기서는 간단히 패닉을 유도하여 개발자가 문제를 인지하도록 함
                    panic!("Failed to bind to address: {}", e);
                });
                axum::serve(listener, app_router).await.unwrap_or_else(|e| {
                     eprintln!("[GUI Backend - Axum] Server error: {}", e);
                     // 서버 실행 중 에러 발생 시 로깅
                });
            });
            Ok(())
        })
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
