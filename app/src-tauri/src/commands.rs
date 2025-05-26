// app/src-tauri/src/commands.rs

use crate::force_activate;
use crate::{api_error, fs_error, log_error, to_string_error, AppError, AppResult};
use dotenvy::dotenv; // Used to load .env at runtime in development mode
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::os::windows::process::CommandExt;
use std::{env, fs, path::PathBuf, process::Command as StdCommand}; // env added for accessing environment variables at runtime
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::time::{sleep, Duration};
use urlencoding::encode;

// --- Existing struct definitions (McpServerInfo, ApiCardData, PageInfo, DataWrapper, ApiResponse) ---
#[derive(Debug, Deserialize)]
struct McpServerInfo {
    name: String,
    description: String,
    args: Option<Vec<String>>,
    env: Option<serde_json::Map<String, Value>>,
    command: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct ApiCardData {
    id: i32,
    #[serde(rename = "type")]
    _type: String,
    url: String,
    stars: i32,
    views: i32,
    scanned: bool,
    #[serde(rename = "mcpServer")]
    mcpServers: McpServerInfo,
    #[serde(rename = "securityRank")]
    security_rank: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct PageInfo {
    startCursor: Option<Value>,
    endCursor: Option<Value>,
    hasNextPage: bool,
    totalItems: i32,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct DataWrapper {
    pageInfo: PageInfo,
    #[serde(rename = "mcpServers")]
    mcpServers: Vec<ApiCardData>,
}

// This ApiResponse struct can be used for both get_mcp_data and get_mcp_detail_data.
#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct ApiResponse {
    // data field is flexible with Value type
    timestamp: String,
    message: String,
    code: String,
    data: Value, // Actual data comes in Value form here
}

// --- MCPCard, MCPCardDetail, MCPServerConfig, ClaudeDesktopConfig, AppState struct definitions ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPCard {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub url: String,
    pub stars: i32,
    pub scanned: bool,
    #[serde(default = "default_security_rank")]
    pub security_rank: String,
}

// 기본 보안 등급 값으로 "UNRATE" 반환하는 함수
fn default_security_rank() -> String {
    "UNRATE".to_string()
}

// Response struct including page information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfoResponse {
    pub has_next_page: bool,
    pub end_cursor: Option<i32>,
    pub total_items: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPCardResponse {
    pub cards: Vec<MCPCard>,
    pub page_info: PageInfoResponse,
}

// DetailApiResponse is now designed to parse the object obtained from `api_response_wrapper.data.get("mcpServer")`
#[derive(Debug, Deserialize)]
struct DetailApiResponse {
    id: i32,
    url: String,
    stars: i32,
    #[serde(rename = "mcpServer")] // This inner mcpServer object holds the actual server details
    mcp_server_info: McpServerInfo,
    scanned: Option<bool>,
    #[serde(rename = "type")]
    _type: Option<String>,
    views: Option<i32>,
    #[serde(rename = "securityRank")]
    security_rank: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPCardDetail {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub url: String,
    pub stars: i32,
    pub scanned: bool,
    pub args: Option<Vec<String>>,
    pub env: Option<Map<String, Value>>,
    pub command: Option<String>,
    #[serde(default = "default_security_rank")]
    pub security_rank: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerConfig {
    pub command: String,
    pub args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaudeDesktopConfig {
    pub mcpServers: Option<HashMap<String, MCPServerConfig>>,
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

pub struct AppState {
    pub client: Client,
}

// --- Tauri Commands ---

#[tauri::command]
pub async fn get_mcp_data(
    state: State<'_, AppState>,
    search_term: Option<String>,
    cursor_id: Option<i32>,
) -> Result<MCPCardResponse, String> {
    // 새로운 내부 함수 구현 (AppResult 사용)
    async fn get_mcp_data_internal(
        client: &Client,
        search_term: Option<String>,
        cursor_id: Option<i32>,
    ) -> AppResult<MCPCardResponse> {
        // Load .env file in development mode (ignored if already loaded)
        #[cfg(debug_assertions)]
        let _ = dotenv();

        // Get environment variables at runtime
        let base_url: String = if cfg!(debug_assertions) {
            // Development mode: get from environment variable
            env::var("CRAWLER_API_BASE_URL")
                .unwrap_or_else(|_| String::new())
        } else {
            // Deployment mode: include the value from .env file at compile time
            env::var("CRAWLER_API_BASE_URL").unwrap_or_else(|_| {
                include_str!("../../.env")
                    .lines()
                    .find(|line| line.starts_with("CRAWLER_API_BASE_URL="))
                    .and_then(|line| line.split('=').nth(1))
                    .unwrap_or("")
                    .to_string()
            })
        };

        if base_url.is_empty() {
            return Err(AppError::ConfigError {
                msg: "API 기본 URL이 설정되지 않았습니다".to_string(),
            });
        }

        // Add cursor ID to URL configuration
        let request_url = if let Some(term) = search_term {
            if term.is_empty() {
                if let Some(cursor) = cursor_id {
                    format!("{}?size=10&cursorId={}", base_url, cursor)
                } else {
                    format!("{}?size=10", base_url)
                }
            } else {
                let encoded_term = encode(&term);
                if let Some(cursor) = cursor_id {
                    // Add cursor for search requests too
                    format!(
                        "{}/search?name={}&size=10&cursorId={}",
                        base_url, encoded_term, cursor
                    )
                } else {
                    format!("{}/search?name={}&size=10", base_url, encoded_term)
                }
            }
        } else {
            if let Some(cursor) = cursor_id {
                format!("{}?size=10&cursorId={}", base_url, cursor)
            } else {
                format!("{}?size=10", base_url)
            }
        };

        // API 요청 수행
        let response = client
            .get(&request_url)
            .send()
            .await
            .map_err(|e| api_error(e, None))?;

        let status = response.status();
        let status_code = status.as_u16();

        if status.is_success() {
            // 응답 성공 시 텍스트 본문 읽기
            let text_body = response.text().await.map_err(|e| {
                api_error(
                    format!("응답 텍스트 읽기 실패: {}", e),
                    Some(status_code),
                )
            })?;

            // ApiResponse 파싱
            let api_response: ApiResponse = serde_json::from_str(&text_body).map_err(|e| {
                log_error(
                    AppError::JsonError {
                        msg: format!("ApiResponse 파싱 오류: {}", e),
                    },
                    "get_mcp_data",
                )
            })?;

            // 데이터 처리
            if let Value::Object(data_obj) = &api_response.data {
                // DataWrapper로 파싱
                let data_wrapper: DataWrapper = serde_json::from_value(Value::Object(data_obj.clone()))
                    .map_err(|e| {
                        log_error(
                            AppError::JsonError {
                                msg: format!("DataWrapper 파싱 오류: {}", e),
                            },
                            "get_mcp_data",
                        )
                    })?;

                // 카드 데이터 매핑
                let cards: Vec<MCPCard> = data_wrapper
                    .mcpServers
                    .iter()
                    .map(|api_card| {
                        // API에서 보안 랭크 값을 가져옴 (없으면 기본값 "UNRATE" 사용)
                        let security_rank = match &api_card.security_rank {
                            Some(rank) => rank.clone(),
                            None => "UNRATE".to_string(),
                        };

                        MCPCard {
                            id: api_card.id,
                            title: api_card.mcpServers.name.clone(),
                            description: api_card.mcpServers.description.clone(),
                            url: api_card.url.clone(),
                            stars: api_card.stars,
                            scanned: api_card.scanned,
                            security_rank,
                        }
                    })
                    .collect();

                // 페이지 정보 추출
                let end_cursor = match data_wrapper.pageInfo.endCursor {
                    Some(Value::Number(n)) => n.as_i64().map(|x| x as i32),
                    _ => None,
                };

                // 응답 생성
                return Ok(MCPCardResponse {
                    cards,
                    page_info: PageInfoResponse {
                        has_next_page: data_wrapper.pageInfo.hasNextPage,
                        end_cursor: end_cursor, 
                        total_items: data_wrapper.pageInfo.totalItems,
                    },
                });
            } else {
                // 데이터가 없거나 객체가 아닌 경우 빈 응답 반환
                return Ok(MCPCardResponse {
                    cards: Vec::new(),
                    page_info: PageInfoResponse {
                        has_next_page: false,
                        end_cursor: None,
                        total_items: 0,
                    },
                });
            }
        } else {
            // 오류 응답 처리
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|e| format!("응답 오류 본문 읽기 실패: {}", e));

            return Err(api_error(
                format!("서버 오류: {} - {}", status, error_body),
                Some(status_code),
            ));
        }
    }

    // 내부 함수 실행 및 오류 변환
    to_string_error(get_mcp_data_internal(&state.client, search_term, cursor_id).await)
}

#[tauri::command]
pub async fn get_mcp_detail_data(
    state: State<'_, AppState>,
    id: i32,
) -> Result<MCPCardDetail, String> {
    // Load .env file in development mode (ignored if already loaded)
    #[cfg(debug_assertions)]
    let _ = dotenv();

    // Get environment variables at runtime
    let base_url: String = if cfg!(debug_assertions) {
        // Development mode: get from environment variable
        env::var("CRAWLER_API_BASE_URL").unwrap_or_else(|_| String::new())
    } else {
        // Deployment mode: include the value from .env file at compile time
        env::var("CRAWLER_API_BASE_URL").unwrap_or_else(|_| {
            include_str!("../../.env")
                .lines()
                .find(|line| line.starts_with("CRAWLER_API_BASE_URL="))
                .and_then(|line| line.split('=').nth(1))
                .unwrap_or("")
                .to_string()
        })
    };
    let request_url = format!("{}/{}", base_url, id);

    match state.client.get(&request_url).send().await {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                match response.json::<ApiResponse>().await {
                    // Outer ApiResponse wrapper
                    Ok(api_response_wrapper) => {
                        if let Value::Object(data_map) = api_response_wrapper.data {
                            if let Some(inner_mcp_server_value) = data_map.get("mcpServer") {
                                match serde_json::from_value::<DetailApiResponse>(
                                    inner_mcp_server_value.clone(),
                                ) {
                                    Ok(detail_data) => {
                                        // API에서 보안 랭크 값을 가져옴 (없으면 기본값 "UNRATE" 사용)
                                        let security_rank = match detail_data.security_rank {
                                            Some(rank) => rank,
                                            None => "UNRATE".to_string(),
                                        };

                                        let card_detail = MCPCardDetail {
                                            id: detail_data.id,
                                            title: detail_data.mcp_server_info.name, // Name from McpServerInfo
                                            description: detail_data.mcp_server_info.description, // Description from McpServerInfo
                                            url: detail_data.url,
                                            stars: detail_data.stars,
                                            scanned: detail_data.scanned.unwrap_or(false),
                                            args: detail_data.mcp_server_info.args,
                                            env: detail_data.mcp_server_info.env,
                                            command: detail_data.mcp_server_info.command,
                                            security_rank,
                                        };
                                        Ok(card_detail)
                                    }
                                    Err(e_inner_struct) => {
                                        let msg = format!("[get_mcp_detail_data] Failed to parse inner_mcp_server_value into DetailApiResponse: {}. Value was: {:?}", e_inner_struct, inner_mcp_server_value);
                                        Err(msg)
                                    }
                                }
                            } else {
                                let msg = format!("[get_mcp_detail_data] Key 'mcpServer' not found inside ApiResponse.data. ApiResponse.data was: {:?}", data_map);
                                Err(msg)
                            }
                        } else {
                            let msg = format!("[get_mcp_detail_data] ApiResponse.data is not an object. It was: {:?}", api_response_wrapper.data);
                            Err(msg)
                        }
                    }
                    Err(e_outer) => {
                        let msg = format!("[get_mcp_detail_data] Failed to parse outer ApiResponse: {}. Check if the overall response matches ApiResponse structure.", e_outer);
                        Err(msg)
                    }
                }
            } else {
                let error_body = response
                    .text()
                    .await
                    .unwrap_or_else(|e| format!("Failed to read error body: {}", e));
                let msg = format!(
                    "[get_mcp_detail_data] Server error {}: {}. Body: {:.500}",
                    request_url, status, error_body
                );
                Err(msg)
            }
        }
        Err(e) => {
            let msg = format!("[get_mcp_detail_data] Request error {}: {}", request_url, e);
            Err(msg)
        }
    }
}

/// Add MCP server configuration to Claude Desktop config file
#[tauri::command]
pub async fn add_mcp_server_config(
    app: AppHandle,
    server_name: String,
    server_config: MCPServerConfig,
    server_id: i64,
) -> Result<(), String> {
    // Generate config file path
    let config_path = match std::env::consts::OS {
        // Explicitly use std::env here for OS const
        "windows" => {
            // For Windows, %APPDATA%\Claude\claude_desktop_config.json
            let appdata = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to get AppData directory: {}", e))?;
            let claude_dir = appdata.parent().unwrap().join("Claude");

            // Create Claude directory if it doesn't exist
            if !claude_dir.exists() {
                fs::create_dir_all(&claude_dir)
                    .map_err(|e| format!("Failed to create Claude directory: {}", e))?;
            }

            claude_dir.join("claude_desktop_config.json")
        }
        _ => {
            return Err("This function is currently only supported on Windows".to_string());
        }
    };

    // Create mcplink file path (same location as claude_desktop_config.json)
    let mcplink_path = config_path
        .parent()
        .unwrap()
        .join("mcplink_desktop_config.json");

    // Read config file (create empty object if file doesn't exist)
    let mut config = if config_path.exists() {
        let config_str = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        match serde_json::from_str::<ClaudeDesktopConfig>(&config_str) {
            Ok(config) => config,
            Err(_) => {
                // If file exists but format is incorrect, create new while preserving other fields
                match serde_json::from_str::<Value>(&config_str) {
                    Ok(value) => {
                        if let Value::Object(map) = value {
                            ClaudeDesktopConfig {
                                mcpServers: None,
                                other: map,
                            }
                        } else {
                            ClaudeDesktopConfig::default()
                        }
                    }
                    Err(_) => ClaudeDesktopConfig::default(),
                }
            }
        }
    } else {
        ClaudeDesktopConfig::default()
    };

    // Get or create MCP server map
    let mut servers = config.mcpServers.unwrap_or_default();

    // Add or update server configuration
    servers.insert(server_name.clone(), server_config);
    config.mcpServers = Some(servers);

    // Write to config file
    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, config_json)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    // Add server_id and server_name to mcplink_desktop_config.json
    let mut mcplink_config = if mcplink_path.exists() {
        let mcplink_str = fs::read_to_string(&mcplink_path)
            .map_err(|e| format!("Failed to read mcplink config file: {}", e))?;

        match serde_json::from_str::<Map<String, Value>>(&mcplink_str) {
            Ok(map) => map,
            Err(_) => Map::new(),
        }
    } else {
        Map::new()
    };

    // Convert server_id to string key and save server_name as value
    mcplink_config.insert(server_id.to_string(), Value::String(server_name));

    // Write to mcplink config file
    let mcplink_json = serde_json::to_string_pretty(&mcplink_config)
        .map_err(|e| format!("Failed to serialize mcplink config: {}", e))?;

    fs::write(&mcplink_path, mcplink_json)
        .map_err(|e| format!("Failed to write mcplink config file: {}", e))?;

    Ok(())
}

/// Remove MCP server configuration from Claude Desktop config file
#[tauri::command]
pub async fn remove_mcp_server_config(app: AppHandle, server_name: String) -> Result<(), String> {
    // Generate config file path
    let config_path = match std::env::consts::OS {
        // Explicitly use std::env here for OS const
        "windows" => {
            let appdata = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to get AppData directory: {}", e))?;
            let claude_dir = appdata.parent().unwrap().join("Claude");
            if !claude_dir.exists() {
                return Err("Claude directory does not exist".to_string());
            }
            claude_dir.join("claude_desktop_config.json")
        }
        _ => {
            return Err("This function is currently only supported on Windows".to_string());
        }
    };

    // Create mcplink file path (same location as claude_desktop_config.json)
    let mcplink_path = config_path
        .parent()
        .unwrap()
        .join("mcplink_desktop_config.json");

    // Check if config file exists
    if !config_path.exists() {
        return Err("Configuration file does not exist".to_string());
    }

    let config_str = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let mut config = match serde_json::from_str::<ClaudeDesktopConfig>(&config_str) {
        Ok(config) => config,
        Err(e) => return Err(format!("Failed to parse config file: {}", e)),
    };

    if config.mcpServers.is_none() {
        return Err("No MCP servers are installed".to_string());
    }

    let mut servers = config.mcpServers.unwrap_or_default();

    if !servers.contains_key(&server_name) {
        return Err(format!("MCP server '{}' not found", server_name));
    }

    servers.remove(&server_name);
    config.mcpServers = Some(servers);

    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, config_json)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    // Remove entries related to server_name from mcplink_desktop_config.json
    if mcplink_path.exists() {
        let mcplink_str = fs::read_to_string(&mcplink_path)
            .map_err(|e| format!("Failed to read mcplink config file: {}", e))?;

        match serde_json::from_str::<Map<String, Value>>(&mcplink_str) {
            Ok(mut mcplink_config) => {
                // Find and delete all entries matching server_name
                let mut keys_to_remove = Vec::new();
                for (key, value) in &mcplink_config {
                    if let Value::String(name) = value {
                        if name == &server_name {
                            keys_to_remove.push(key.clone());
                        }
                    }
                }

                // Delete found keys
                for key in keys_to_remove {
                    mcplink_config.remove(&key);
                }

                // Write updated config to file
                let mcplink_json = serde_json::to_string_pretty(&mcplink_config)
                    .map_err(|e| format!("Failed to serialize mcplink config: {}", e))?;

                fs::write(&mcplink_path, mcplink_json)
                    .map_err(|e| format!("Failed to write mcplink config file: {}", e))?;
            }
            Err(e) => return Err(format!("Failed to parse mcplink config file: {}", e)),
        }
    }

    Ok(())
}

/// Restart Claude Desktop application
#[tauri::command]
pub async fn restart_claude_desktop(_app: AppHandle) -> Result<(), String> {
    // 1) Terminate all claude.exe processes (use precise filter)
    let kill_status = StdCommand::new("taskkill")
        .args([
            "/F", // Force termination
            "/T", // Terminate child processes
            "/FI",
            "IMAGENAME eq claude.exe",
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW 플래그 추가
        .status()
        .map_err(|e| format!("Failed to execute taskkill: {}", e))?;
    match kill_status.code() {
        Some(0) => {}   // Success
        Some(128) => {} // No process found (considered success for this step)
        Some(_c) => {}  // Other exit codes, log if needed (_c to avoid warning)
        None => {}      // Process terminated by signal, log if needed
    }

    // 2) Wait sufficiently (2 seconds)
    sleep(Duration::from_millis(2000)).await;

    // 3) Confirm termination
    let check = StdCommand::new("tasklist")
        .args(["/FI", "IMAGENAME eq claude.exe", "/NH"]) // /NH for no header
        .creation_flags(0x08000000) // Add CREATE_NO_WINDOW flag
        .output()
        .map_err(|e| format!("Failed to execute tasklist: {}", e))?;
    let _running = String::from_utf8_lossy(&check.stdout); // _running to avoid warning

    // 4) Pre-create cache directory (prevent permission issues)
    let cache_dir: PathBuf = {
        let base = std::env::var("LOCALAPPDATA")
            .map_err(|e| format!("Failed to get LOCALAPPDATA: {}", e))?; // std::env for runtime var
        let dir = PathBuf::from(&base).join("AnthropicClaude").join("Cache");
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create cache dir {:?}: {}", dir, e))?;
        dir
    };
    let cache_dir_str = cache_dir.to_string_lossy();

    // 5) Prepare executable path
    let claude_exe: PathBuf = {
        let base = std::env::var("LOCALAPPDATA")
            .map_err(|e| format!("Failed to get LOCALAPPDATA: {}", e))?; // std::env for runtime var
        PathBuf::from(base)
            .join("AnthropicClaude")
            .join("Claude.exe")
    };
    if !claude_exe.exists() {
        return Err(format!("Claude.exe not found at {}", claude_exe.display()));
    }

    // 6) Pass only Electron runtime flags → prevent URL parsing errors
    let _child = StdCommand::new(&claude_exe) // _child to avoid warning
        .args([
            "--user-data-dir",
            &cache_dir_str,
            "--disable-gpu-shader-disk-cache",
            "--disable-gpu",
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW 플래그 추가
        .spawn()
        .map_err(|e| format!("Failed to start Claude Desktop: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn get_installed_mcp_data(
    state: State<'_, AppState>,
    server_ids: Vec<i32>,
    cursor_id: Option<i32>,
    search_term: Option<String>,
) -> Result<MCPCardResponse, String> {
    // Load .env file in development mode (ignored if already loaded)
    #[cfg(debug_assertions)]
    let _ = dotenv();

    // Get environment variables at runtime
    let base_url: String = if cfg!(debug_assertions) {
        // Development mode: get from environment variable
        env::var("CRAWLER_API_BASE_URL").unwrap_or_else(|_| String::new())
    } else {
        // Deployment mode: include the value from .env file at compile time
        env::var("CRAWLER_API_BASE_URL").unwrap_or_else(|_| {
            include_str!("../../.env")
                .lines()
                .find(|line| line.starts_with("CRAWLER_API_BASE_URL="))
                .and_then(|line| line.split('=').nth(1))
                .unwrap_or("")
                .to_string()
        })
    };

    // Use the batch endpoint
    let mut request_url = format!("{}/batch", base_url);

    // Add cursorId query parameter if present
    if let Some(cursor) = cursor_id {
        request_url.push_str(&format!("?size=10&cursorId={}", cursor)); // Assuming default size 10, adjust if needed
    } else {
        request_url.push_str("?size=10"); // Default size if no cursor
    }

    // Create the request body
    let request_body = json!({ "serverIds": server_ids });

    // Send POST request
    match state
        .client
        .post(&request_url)
        .json(&request_body) // Send the body as JSON
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();

            if status.is_success() {
                match response.text().await {
                    // Read as text first for logging
                    Ok(text_body) => {
                        match serde_json::from_str::<ApiResponse>(&text_body) {
                            // Parse outer ApiResponse
                            Ok(api_response) => {
                                if let Value::Object(data_obj) = &api_response.data {
                                    match serde_json::from_value::<DataWrapper>(Value::Object(
                                        data_obj.clone(),
                                    )) {
                                        Ok(data_wrapper) => {
                                            let mut cards: Vec<MCPCard> = data_wrapper
                                                .mcpServers
                                                .iter()
                                                .map(|api_card| {
                                                    // API에서 보안 랭크 값을 가져옴 (없으면 기본값 "UNRATE" 사용)
                                                    let security_rank =
                                                        match &api_card.security_rank {
                                                            Some(rank) => rank.clone(),
                                                            None => "UNRATE".to_string(),
                                                        };

                                                    MCPCard {
                                                        id: api_card.id,
                                                        title: api_card.mcpServers.name.clone(),
                                                        description: api_card
                                                            .mcpServers
                                                            .description
                                                            .clone(),
                                                        url: api_card.url.clone(),
                                                        stars: api_card.stars,
                                                        scanned: api_card.scanned,
                                                        security_rank,
                                                    }
                                                })
                                                .collect();

                                            // If there is a search term, filter locally
                                            if let Some(term) = search_term {
                                                if !term.is_empty() {
                                                    let search_term_lower = term.to_lowercase();
                                                    cards = cards
                                                        .into_iter()
                                                        .filter(|card| {
                                                            // Check if the search term is included in the title or description
                                                            card.title
                                                                .to_lowercase()
                                                                .contains(&search_term_lower)
                                                                || card
                                                                    .description
                                                                    .to_lowercase()
                                                                    .contains(&search_term_lower)
                                                        })
                                                        .collect();
                                                }
                                            }

                                            let end_cursor = match data_wrapper.pageInfo.endCursor {
                                                Some(Value::Number(n)) => {
                                                    n.as_i64().map(|x| x as i32)
                                                }
                                                _ => None,
                                            };

                                            // Update total_items value with the number of filtered items
                                            let response = MCPCardResponse {
                                                cards,
                                                page_info: PageInfoResponse {
                                                    has_next_page: data_wrapper
                                                        .pageInfo
                                                        .hasNextPage,
                                                    end_cursor: end_cursor,
                                                    total_items: data_wrapper.pageInfo.totalItems,
                                                },
                                            };

                                            return Ok(response);
                                        }
                                        Err(e) => {
                                            return Err(format!("[get_installed_mcp_data] Failed to parse data into DataWrapper: {}", e));
                                        }
                                    }
                                } else {
                                    // If data is not an object or missing, return empty response
                                    return Ok(MCPCardResponse {
                                        cards: Vec::new(),
                                        page_info: PageInfoResponse {
                                            has_next_page: false,
                                            end_cursor: None,
                                            total_items: 0,
                                        },
                                    });
                                }
                            }
                            Err(e) => {
                                let msg = format!("[get_installed_mcp_data] JSON parsing error for ApiResponse: {}. Body: {:.500}", e, text_body);
                                return Err(msg);
                            }
                        }
                    }
                    Err(e) => {
                        let msg = format!(
                            "[get_installed_mcp_data] Failed to read response text: {}",
                            e
                        );
                        return Err(msg);
                    }
                }
            } else {
                let error_body = response
                    .text()
                    .await
                    .unwrap_or_else(|e| format!("Failed to read error body: {}", e));
                let msg = format!(
                    "[get_installed_mcp_data] Server error for {}: {}. Body: {:.500}",
                    request_url, status, error_body
                );
                return Err(msg);
            }
        }
        Err(e) => {
            let msg = format!(
                "[get_installed_mcp_data] Request error for {}: {}",
                request_url, e
            );
            return Err(msg);
        }
    }
}

/// Function to directly search for locally installed MCP servers
/// Reads the configuration file directly, filters, and returns the results
#[tauri::command]
pub async fn search_local_mcp_servers(
    app: AppHandle,
    search_term: String,
) -> Result<Vec<LocalMCPServer>, String> {
    // Prepare configuration file path
    let claude_dir = match std::env::consts::OS {
        "windows" => {
            let appdata = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to get AppData directory: {}", e))?;
            appdata.parent().unwrap_or(&appdata).join("Claude")
        }
        _ => {
            return Err("Currently only supported on Windows".to_string());
        }
    };

    // Claude Desktop configuration file path
    let config_path = claude_dir.join("claude_desktop_config.json");
    let mcplink_config_path = claude_dir.join("mcplink_desktop_config.json");

    // Convert search term to lowercase
    let search_term_lower = search_term.to_lowercase();

    // Vector to store results
    let mut results = Vec::new();

    // Check if files exist
    if !config_path.exists() || !mcplink_config_path.exists() {
        return Ok(Vec::new()); // Return empty result
    }

    // 1. Read claude_desktop_config.json
    let config_str = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read claude config file: {}", e))?;

    // 2. Parse configuration
    let claude_config: ClaudeDesktopConfig = serde_json::from_str(&config_str)
        .map_err(|e| format!("Failed to parse claude config file: {}", e))?;

    // 3. Read mcplink_desktop_config.json
    let mcplink_str = fs::read_to_string(&mcplink_config_path)
        .map_err(|e| format!("Failed to read mcplink config file: {}", e))?;

    // 4. Parse mcplink configuration
    let mcplink_config: Map<String, Value> = serde_json::from_str(&mcplink_str)
        .map_err(|e| format!("Failed to parse mcplink config file: {}", e))?;

    // 5. Create ID-name mapping (reverse)
    let mut name_to_id = HashMap::new();
    for (id_str, name_val) in mcplink_config.iter() {
        if let Value::String(name) = name_val {
            // Skip fallback server with ID -1
            if id_str != "-1" {
                if let Ok(id) = id_str.parse::<i32>() {
                    name_to_id.insert(name.clone(), id);
                }
            }
        }
    }

    // 6. Check MCP server configuration
    if let Some(servers) = &claude_config.mcpServers {
        for (name, config) in servers.iter() {
            // Skip fallback server named "MCPlink"
            if name == "MCPlink" {
                continue;
            }

            // If search term is empty, or if name or command contains the search term
            if search_term.is_empty()
                || name.to_lowercase().contains(&search_term_lower)
                || config.command.to_lowercase().contains(&search_term_lower)
            {
                // Get ID
                let server_id = name_to_id.get(name).cloned().unwrap_or(-1);

                // Add to results
                results.push(LocalMCPServer {
                    id: server_id,
                    name: name.clone(),
                    command: config.command.clone(),
                    args: config.args.clone(),
                });
            }
        }
    }

    Ok(results)
}

/// Struct for local MCP server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalMCPServer {
    pub id: i32,
    pub name: String,
    pub command: String,
    pub args: Option<Vec<String>>,
}

/// Reads the content of mcplink_desktop_config.json and returns it as a string.
#[tauri::command]
pub fn read_mcplink_config_content(app: AppHandle) -> Result<String, String> {
    let config_path = match std::env::consts::OS {
        // Explicitly use std::env here for OS const
        "windows" => {
            let appdata = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to get AppData directory: {}", e))?;
            let claude_dir = appdata
                .parent()
                .ok_or("Failed to get parent of AppData")?
                .join("Claude");
            claude_dir.join("mcplink_desktop_config.json")
        }
        _ => return Err("Path resolution currently only supported on Windows".to_string()),
    };

    if !config_path.exists() {
        // If the file doesn't exist, return an empty JSON object string or a specific error/signal
        // For now, returning an empty JSON object string to be parsed by frontend
        // Or, you could return an Err to be handled specifically by the frontend.
        // For example: return Err("mcplink_config_not_found".to_string());
        return Ok("{}".to_string());
    }

    fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read mcplink_desktop_config.json: {}", e))
}

// --- Modified function to check config file existence ---

#[tauri::command]
pub fn check_claude_config_exists(app: AppHandle) -> Result<bool, String> {
    let app_data_dir = match app.path().app_data_dir() {
        Ok(path) => path,
        Err(e) => {
            return Err(format!("Failed to get app data directory: {}", e));
        }
    };

    let claude_dir = app_data_dir
        .parent()
        .unwrap_or(&app_data_dir)
        .join("Claude");
    let claude_config_path = claude_dir.join("claude_desktop_config.json");

    Ok(claude_config_path.exists())
}

#[tauri::command]
pub fn check_mcplink_config_exists(app: AppHandle) -> Result<bool, String> {
    let app_data_dir = match app.path().app_data_dir() {
        Ok(path) => path,
        Err(e) => {
            return Err(format!("Failed to get app data directory: {}", e));
        }
    };

    let claude_dir = app_data_dir
        .parent()
        .unwrap_or(&app_data_dir)
        .join("Claude");
    let mcplink_config_path = claude_dir.join("mcplink_desktop_config.json");

    Ok(mcplink_config_path.exists())
}

/// Ensures config files exist with default values
/// Creates them if they don't exist
#[tauri::command]
pub async fn ensure_config_files(app: AppHandle) -> Result<(), String> {
    // Load .env file in development mode (ignored if already loaded)
    #[cfg(debug_assertions)]
    let _ = dotenv();

    // Get environment variables at runtime
    let crawler_api_base_url = if cfg!(debug_assertions) {
        // Development mode: get from environment variable
        env::var("CRAWLER_API_BASE_URL").unwrap_or_else(|_| String::new())
    } else {
        // Deployment mode: include the value from .env file at compile time
        env::var("CRAWLER_API_BASE_URL").unwrap_or_else(|_| {
            include_str!("../../.env")
                .lines()
                .find(|line| line.starts_with("CRAWLER_API_BASE_URL="))
                .and_then(|line| line.split('=').nth(1))
                .unwrap_or("")
                .to_string()
        })
    };

    let gui_be_api_base_url = if cfg!(debug_assertions) {
        // Development mode: get from environment variable
        env::var("GUI_BE_API_BASE_URL").unwrap_or_else(|_| String::new())
    } else {
        // Deployment mode: include the value from .env file at compile time
        env::var("GUI_BE_API_BASE_URL").unwrap_or_else(|_| {
            include_str!("../../.env")
                .lines()
                .find(|line| line.starts_with("GUI_BE_API_BASE_URL="))
                .and_then(|line| line.split('=').nth(1))
                .unwrap_or("")
                .to_string()
        })
    };

    // Check if config files exist
    let claude_config_exists = check_claude_config_exists(app.clone())?;
    let mcplink_config_exists = check_mcplink_config_exists(app.clone())?;

    // Only create missing files if needed
    if !claude_config_exists || !mcplink_config_exists {
        // Set up fallback server
        let server_name = "McpFallbackServer";

        // Create environment variables map
        let mut env_map = Map::new();

        // Add environment variables only if they exist
        if !crawler_api_base_url.is_empty() {
            env_map.insert(
                "CRAWLER_API_BASE_URL".to_string(),
                Value::String(crawler_api_base_url),
            );
        }
        if !gui_be_api_base_url.is_empty() {
            env_map.insert(
                "GUI_BE_API_BASE_URL".to_string(),
                Value::String(gui_be_api_base_url),
            );
        }
        env_map.insert(
            "NODE_ENV".to_string(),
            Value::String("development".to_string()),
        );

        let server_config = MCPServerConfig {
            command: "node".to_string(),
            args: Some(vec!["C:\\S12P31A201\\mcp-server\\dist\\main.js".to_string()]),
            cwd: Some("C:\\S12P31A201\\mcp-server".to_string()),
            env: Some(env_map),
        };

        // Add the configuration - this will create both config files
        add_mcp_server_config(app, server_name.to_string(), server_config, -1).await?
    }

    Ok(())
}

/// Monitors both configuration files and emits an event if either is missing
/// This function should be called when you want to start watching the config files
#[tauri::command]
pub async fn start_config_watch(app: AppHandle) -> Result<(), String> {
    // We'll use a background task to periodically check the config files
    let app_handle = app.clone();

    // Start a task that runs in the background
    tauri::async_runtime::spawn(async move {
        // Check every 3 seconds for config files
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3));

        loop {
            interval.tick().await; // Wait for the next interval

            // Check if config files exist
            match check_claude_config_exists(app_handle.clone()) {
                Ok(claude_exists) => {
                    match check_mcplink_config_exists(app_handle.clone()) {
                        Ok(mcplink_exists) => {
                            // If either file is missing, emit an event
                            if !claude_exists || !mcplink_exists {
                                // Send an event to the frontend that config is missing
                                use tauri::Manager;
                                if let Some(window) = app_handle.get_webview_window("main") {
                                    // Emit the event with the specific status of each file
                                    // Using Emitter trait which is now in scope
                                    let _ = window.emit(
                                        "config-files-missing",
                                        json!({
                                            "claudeConfigExists": claude_exists,
                                            "mcplinkConfigExists": mcplink_exists
                                        }),
                                    );
                                }
                            }
                        }
                        Err(e) => eprintln!("Error checking mcplink config: {}", e),
                    }
                }
                Err(e) => eprintln!("Error checking claude config: {}", e),
            }
        }
    });

    Ok(())
}

/// Function to read the detailed settings of an installed MCP server
#[tauri::command]
pub fn read_mcp_server_config(
    app: AppHandle,
    server_name: String,
) -> Result<MCPServerConfig, String> {
    // 새로운 내부 함수 구현 (AppResult 사용)
    fn read_mcp_server_config_internal(
        app: &AppHandle,
        server_name: &str,
    ) -> AppResult<MCPServerConfig> {
        // Create Claude directory path
        let claude_dir = match std::env::consts::OS {
            "windows" => {
                let appdata = app
                    .path()
                    .app_data_dir()
                    .map_err(|e| AppError::OsError {
                        msg: format!("AppData 디렉토리 가져오기 실패: {}", e),
                    })?;
                appdata.parent().unwrap_or(&appdata).join("Claude")
            }
            _ => {
                return Err(AppError::OsError {
                    msg: "현재 Windows에서만 지원됩니다".to_string(),
                });
            }
        };

        // Claude Desktop configuration file path
        let config_path = claude_dir.join("claude_desktop_config.json");

        if !config_path.exists() {
            return Err(AppError::ConfigError {
                msg: "Claude 설정 파일이 존재하지 않습니다".to_string(),
            });
        }

        // Read configuration file
        let config_str = fs::read_to_string(&config_path).map_err(|e| {
            fs_error(
                e,
                &config_path.to_string_lossy(),
            )
        })?;

        // Parse configuration
        let config: ClaudeDesktopConfig = serde_json::from_str(&config_str).map_err(|e| {
            log_error(
                AppError::JsonError {
                    msg: format!("설정 파일 파싱 실패: {}", e),
                },
                "read_mcp_server_config",
            )
        })?;

        // Check MCP server configuration
        let servers = config.mcpServers.unwrap_or_default();

        // Get configuration by specific server name
        if let Some(server_config) = servers.get(server_name) {
            Ok(server_config.clone())
        } else {
            Err(AppError::ConfigError {
                msg: format!(
                    "MCP 서버 '{}'가 설정에 없습니다",
                    server_name
                )
            })
        }
    }

    // 내부 함수 실행 및 오류 변환
    to_string_error(read_mcp_server_config_internal(&app, &server_name))
}

/// Function to check if a server name is installed
#[tauri::command]
pub fn is_mcp_server_installed(app: AppHandle, server_name: String) -> Result<bool, String> {
    // Create Claude directory path
    let claude_dir = match std::env::consts::OS {
        "windows" => {
            let appdata = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to get AppData directory: {}", e))?;
            appdata.parent().unwrap_or(&appdata).join("Claude")
        }
        _ => {
            return Err("Currently only supported on Windows".to_string());
        }
    };

    // Claude Desktop configuration file path
    let config_path = claude_dir.join("claude_desktop_config.json");

    if !config_path.exists() {
        return Ok(false);
    }

    // Read configuration file
    let config_str = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    // Parse configuration
    let config: ClaudeDesktopConfig = serde_json::from_str(&config_str)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    // Check MCP server configuration
    let servers = config.mcpServers.unwrap_or_default();

    // Check configuration by specific server name
    Ok(servers.contains_key(&server_name))
}

/// Function to reset MCP settings (excluding fallback server)
#[tauri::command]
pub fn reset_mcp_settings(app: AppHandle) -> Result<(), String> {
    // 함수 내용 유지 (아래는 원래 코드를 다시 넣음)
    // Create Claude directory path
    let claude_dir = match std::env::consts::OS {
        "windows" => {
            let appdata = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to get AppData directory: {}", e))?;
            appdata.parent().unwrap_or(&appdata).join("Claude")
        }
        _ => {
            return Err("Currently only supported on Windows".to_string());
        }
    };

    // Configuration file paths
    let claude_config_path = claude_dir.join("claude_desktop_config.json");
    let mcplink_config_path = claude_dir.join("mcplink_desktop_config.json");

    // 1. Process claude_desktop_config.json
    if claude_config_path.exists() {
        // Read configuration file
        let config_str = fs::read_to_string(&claude_config_path)
            .map_err(|e| format!("Failed to read claude config file: {}", e))?;

        // Parse configuration
        let mut config_value: Value = serde_json::from_str(&config_str)
            .map_err(|e| format!("Failed to parse claude config file: {}", e))?;

        // Use general Value for more precise control
        if let Value::Object(ref mut obj) = config_value {
            // Find and process mcpServers (uppercase version)
            if let Some(Value::Object(ref mut servers_map)) = obj.get_mut("mcpServers") {
                // Find fallback server
                let fallback_server = servers_map
                    .remove("McpFallbackServer")
                    .or_else(|| servers_map.remove("MCPlink"));

                // Initialize map (remove all keys)
                servers_map.clear();

                // If fallback server existed, add it back
                if let Some(fallback) = fallback_server {
                    servers_map.insert(String::from("McpFallbackServer"), fallback);
                }
            }

            // If lowercase mcp_servers value exists, remove it (to prevent creation)
            obj.remove("mcp_servers");
        }

        // Convert value back to ClaudeDesktopConfig
        let config: ClaudeDesktopConfig = serde_json::from_value(config_value)
            .map_err(|e| format!("Failed to convert value back to config: {}", e))?;

        // Write modified configuration to file
        let config_json = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize claude config: {}", e))?;

        fs::write(&claude_config_path, config_json)
            .map_err(|e| format!("Failed to write claude config file: {}", e))?;
    }

    // 2. Process mcplink_desktop_config.json
    if mcplink_config_path.exists() {
        // Read configuration file
        let mcplink_str = fs::read_to_string(&mcplink_config_path)
            .map_err(|e| format!("Failed to read mcplink config file: {}", e))?;

        // Parse configuration
        let mcplink_config: Map<String, Value> = serde_json::from_str(&mcplink_str)
            .map_err(|e| format!("Failed to parse mcplink config file: {}", e))?;

        // Create a new map that only keeps the fallback server with ID -1
        let mut new_mcplink_config = Map::new();
        if let Some(fallback_value) = mcplink_config.get("-1") {
            new_mcplink_config.insert("-1".to_string(), fallback_value.clone());
        }

        // Write modified configuration to file
        let mcplink_json = serde_json::to_string_pretty(&new_mcplink_config)
            .map_err(|e| format!("Failed to serialize mcplink config: {}", e))?;

        fs::write(&mcplink_config_path, mcplink_json)
            .map_err(|e| format!("Failed to write mcplink config file: {}", e))?;
    }

    Ok(())
}

// 테스트를 위한 앱 강제 활성화 명령
#[tauri::command]
pub fn test_force_activate() -> Result<(), String> {
    // 디버그 로그 파일에 기록
    let log_path = std::env::temp_dir().join("mcplink_test.log");
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] test_force_activate 명령 호출됨",
            chrono::Local::now().format("%H:%M:%S")
        );
    }

    // 활성화 로그 파일 경로
    let activation_log_path = std::env::temp_dir().join("mcplink_activation.log");

    // 활성화 로그 초기화
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&activation_log_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] === 테스트 강제 활성화 시작 ===",
            chrono::Local::now().format("%H:%M:%S")
        );
    }

    // 앱 강제 활성화 시도
    if let Err(e) = force_activate::force_app_to_foreground() {
        // 오류 로깅
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&activation_log_path)
        {
            use std::io::Write;
            let _ = writeln!(
                file,
                "[{}] 테스트 활성화 오류: {}",
                chrono::Local::now().format("%H:%M:%S"),
                e
            );
        }
        return Err(e);
    }

    // 성공 로그
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] test_force_activate 명령 성공",
            chrono::Local::now().format("%H:%M:%S")
        );
    }

    // 활성화 로그 완료
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&activation_log_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] === 테스트 강제 활성화 완료 ===",
            chrono::Local::now().format("%H:%M:%S")
        );
    }

    Ok(())
}

// 테스트를 위한 키워드 검색 기능
#[tauri::command]
pub fn test_search_keyword(app: AppHandle, keyword: String) -> Result<(), String> {
    // 디버그 로그 파일에 기록
    let log_path = std::env::temp_dir().join("mcplink_test.log");
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] test_search_keyword 명령 호출됨: {}",
            chrono::Local::now().format("%H:%M:%S"),
            keyword
        );
    }

    // 앱 강제 활성화 시도
    force_activate::force_app_to_foreground()?;

    // Window 찾아서 search-keyword 이벤트 발생
    if let Some(window) = app.get_webview_window("main") {
        use tauri::Emitter;
        let _ = window.emit("search-keyword", keyword.clone());
    }

    // 성공 로그
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] test_search_keyword 명령 성공",
            chrono::Local::now().format("%H:%M:%S")
        );
    }

    Ok(())
}

// 알림 클릭 시뮬레이션 함수 (알림 클릭 테스트용)
#[tauri::command]
pub fn simulate_notification_click(app: AppHandle, keyword: String) -> Result<(), String> {
    // 디버그 로그 파일에 기록
    let log_path = std::env::temp_dir().join("mcplink_test.log");
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] simulate_notification_click 명령 호출됨: {}",
            chrono::Local::now().format("%H:%M:%S"),
            keyword
        );
    }

    // 알림 클릭 로그 파일 경로
    let click_log_path = std::env::temp_dir().join("mcplink_notification_click.log");

    // 알림 클릭 시뮬레이션 로그 기록
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&click_log_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] 알림 클릭 시뮬레이션: {}",
            chrono::Local::now().format("%H:%M:%S"),
            keyword
        );
    }

    // 키워드를 임시 파일에 저장 (감시 스레드에서 처리하도록)
    let keyword_path = std::env::temp_dir().join("mcplink_last_keyword.txt");
    if let Ok(mut file) = std::fs::File::create(&keyword_path) {
        use std::io::Write;
        if let Err(e) = write!(file, "{}", keyword) {
            return Err(format!("키워드 파일 작성 오류: {}", e));
        }

        // 로그 파일에 기록
        if let Ok(mut log_file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&click_log_path)
        {
            use std::io::Write;
            let _ = writeln!(
                log_file,
                "[{}] 알림 클릭 처리: 키워드 파일 생성됨 (작업 ID: {})",
                chrono::Local::now().format("%H:%M:%S"),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            );
        }
    } else {
        return Err("키워드 파일을 생성할 수 없습니다".to_string());
    }

    // 1. 키워드 상태 설정 (KeywordState 상태 객체 사용)
    if let Some(keyword_state) = app.try_state::<crate::notification_system::KeywordState>() {
        keyword_state.set_keyword(keyword.clone(), "notification_click", 1);
        keyword_state.set_pending(true);
        keyword_state.update_click_time();
        
        // 로그 파일에 기록
        if let Ok(mut log_file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&click_log_path)
        {
            use std::io::Write;
            let _ = writeln!(
                log_file,
                "[{}] KeywordState에 키워드 설정됨: {}",
                chrono::Local::now().format("%H:%M:%S"),
                keyword
            );
        }
    }

    // 앱 강제 활성화 시도
    let activation_result = force_activate::force_app_to_foreground();

    // 활성화 시도 로그
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] simulate_notification_click: 앱 활성화 시도 결과: {}",
            chrono::Local::now().format("%H:%M:%S"),
            if activation_result.is_ok() {
                "성공"
            } else {
                "실패 (직접 이벤트 발생 시도)"
            }
        );
    }
    
    // 앱 활성화 이벤트 발생 (개선된 버전)
    let event_result = force_activate::emit_app_activated_event(&app);
    
    // 이벤트 발생 로그
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] 앱 활성화 이벤트 발생 결과: {}",
            chrono::Local::now().format("%H:%M:%S"),
            if event_result.is_ok() { "성공" } else { "실패" }
        );
    }
    
    // 직접 search-keyword 이벤트 발생 (이중 안전장치)
    if let Some(window) = app.get_webview_window("main") {
        use tauri::Emitter;
        let emit_result = window.emit("search-keyword", keyword.clone());
        
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
                "[{}] 키워드 이벤트 직접 발생 결과: {}",
                chrono::Local::now().format("%H:%M:%S"),
                if emit_result.is_ok() { "성공" } else { "실패" }
            );
        }
    }

    // 성공 로그
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] simulate_notification_click 명령 성공",
            chrono::Local::now().format("%H:%M:%S")
        );
    }

    Ok(())
}

/// 앱 활성화 상태를 확인하고 키워드가 있으면 표시하는 함수 - 개선된 버전
#[tauri::command]
pub fn check_and_mark_app_activated(app: AppHandle) -> Result<Option<String>, String> {
    // 디버그 로그 설정
    let log_path = std::env::temp_dir().join("mcplink_activation.log");
    
    // 로그 시작
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] === check_and_mark_app_activated 시작 ===",
            chrono::Local::now().format("%H:%M:%S")
        );
    }

    // 1. 먼저 키워드 파일 확인
    let keyword_path = std::env::temp_dir().join("mcplink_last_keyword.txt");
    let file_keyword = if keyword_path.exists() {
        // 파일의 수정 시간 확인
        if let Ok(metadata) = std::fs::metadata(&keyword_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    // 10초 이상 된 파일은 무시하고 삭제
                    if elapsed.as_secs() > 10 {
                        let _ = std::fs::remove_file(&keyword_path);
                        
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
                                "[{}] 오래된 키워드 파일 삭제됨 ({}초)",
                                chrono::Local::now().format("%H:%M:%S"),
                                elapsed.as_secs()
                            );
                        }
                        None
                    } else {
                        // 파일이 최신이면 읽기
                        match std::fs::read_to_string(&keyword_path) {
                            Ok(content) => {
                                let trimmed = content.trim();
                                if !trimmed.is_empty() {
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
                                            "[{}] 키워드 파일에서 발견: {} ({}초 전)",
                                            chrono::Local::now().format("%H:%M:%S"),
                                            trimmed,
                                            elapsed.as_secs()
                                        );
                                    }
                                    Some(trimmed.to_string())
                                } else {
                                    None
                                }
                            }
                            Err(e) => {
                                // 파일 읽기 오류 로그
                                if let Ok(mut file) = std::fs::OpenOptions::new()
                                    .create(true)
                                    .write(true)
                                    .append(true)
                                    .open(&log_path)
                                {
                                    use std::io::Write;
                                    let _ = writeln!(
                                        file,
                                        "[{}] 키워드 파일 읽기 오류: {}",
                                        chrono::Local::now().format("%H:%M:%S"),
                                        e
                                    );
                                }
                                None
                            }
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        // 파일 없음 로그
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path)
        {
            use std::io::Write;
            let _ = writeln!(
                file,
                "[{}] 키워드 파일이 존재하지 않음",
                chrono::Local::now().format("%H:%M:%S")
            );
        }
        None
    };

    // 2. KeywordState 확인
    let state_keyword = if let Some(keyword_state) = app.try_state::<crate::notification_system::KeywordState>() {
        // 앱 활성화 상태 설정
        keyword_state.set_app_activated(true);
        
        // 대기 중인 키워드 확인
        if keyword_state.is_pending() || keyword_state.has_keyword() {
            let kw = keyword_state.take_keyword();
            
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
                    "[{}] KeywordState에서 키워드: {:?}",
                    chrono::Local::now().format("%H:%M:%S"),
                    kw
                );
            }
            
            kw
        } else {
            None
        }
    } else {
        None
    };

    // 3. 최종 키워드 결정 (파일 우선)
    let final_keyword = file_keyword.or(state_keyword);

    // 4. 키워드가 있으면 이벤트 발생
    if let Some(ref keyword) = final_keyword {
        if let Some(window) = app.get_webview_window("main") {
            use tauri::Emitter;
            let result = window.emit("search-keyword", keyword.clone());
            
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
                    "[{}] search-keyword 이벤트 발생 결과: {:?}",
                    chrono::Local::now().format("%H:%M:%S"),
                    result.is_ok()
                );
            }
        }
        
        // 키워드 처리 후 파일 삭제
        if keyword_path.exists() {
            let _ = std::fs::remove_file(&keyword_path);
        }
    }

    // 로그 종료
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] === check_and_mark_app_activated 종료 (결과: {:?}) ===",
            chrono::Local::now().format("%H:%M:%S"),
            final_keyword
        );
    }

    Ok(final_keyword)
}

/// 앱 활성화를 알리고 앱을 전면으로 가져오는 함수 (개선된 로깅 사용)
#[tauri::command]
pub fn notify_app_activated(app: AppHandle) -> Result<(), String> {
    // 새로운 통합 로깅 시스템 사용
    crate::log_message(
        crate::LogLevel::Info,
        crate::LogCategory::Activation,
        "notify_app_activated",
        "앱 활성화 알림 명령 호출됨"
    );
    
    // 앱 활성화 상태 업데이트 (KeywordState 사용)
    if let Some(keyword_state) = app.try_state::<crate::notification_system::KeywordState>() {
        // 앱 활성화 상태 설정
        keyword_state.set_app_activated(true);
        
        // 앱 활성화 시간과 가장 최근 알림 클릭 시간의 차이 기록
        let time_diff = keyword_state.get_last_click_time().map(|t| {
            let elapsed = t.elapsed();
            format!("{}초 전", elapsed.as_secs())
        }).unwrap_or_else(|| "알 수 없음".to_string());
        
        // 로그 기록
        crate::log_message(
            crate::LogLevel::Debug,
            crate::LogCategory::Activation,
            "notify_app_activated",
            &format!("마지막 알림 클릭 시간: {}", time_diff)
        );
    }
    
    // 앱 활성화 이벤트 발생
    let result = crate::force_activate::emit_app_activated_event(&app);
    
    // 결과 로깅
    if let Err(ref e) = result {
        crate::log_message(
            crate::LogLevel::Error,
            crate::LogCategory::Activation,
            "notify_app_activated",
            &format!("앱 활성화 이벤트 발생 실패: {}", e)
        );
    } else {
        crate::log_message(
            crate::LogLevel::Info,
            crate::LogCategory::Activation,
            "notify_app_activated",
            "앱 활성화 이벤트 발생 성공"
        );
    }
    
    result
}

/// 설치된 MCP 개수를 가져오는 함수
#[tauri::command]
pub fn get_installed_count(app: AppHandle) -> Result<i32, String> {
    // mcplink_desktop_config.json 파일 읽기
    let mcplink_content = read_mcplink_config_content(app)?;

    // 파일 내용 파싱
    let config: Map<String, Value> = serde_json::from_str(&mcplink_content)
        .map_err(|e| format!("Failed to parse mcplink config: {}", e))?;

    // 설치된 MCP 개수 계산 (fallback 서버 ID -1 제외)
    let count = config.keys().filter(|&key| key != "-1").count() as i32;

    Ok(count)
}

/// MCP 리스트 총 개수를 가져오는 함수
#[tauri::command]
pub async fn get_list_count(state: State<'_, AppState>) -> Result<i32, String> {
    // Load .env file in development mode (ignored if already loaded)
    #[cfg(debug_assertions)]
    let _ = dotenv();

    // Get environment variables at runtime
    let base_url: String = if cfg!(debug_assertions) {
        // Development mode: get from environment variable
        env::var("CRAWLER_API_BASE_URL").unwrap_or_else(|_| String::new())
    } else {
        // Deployment mode: include the value from .env file at compile time
        env::var("CRAWLER_API_BASE_URL").unwrap_or_else(|_| {
            include_str!("../../.env")
                .lines()
                .find(|line| line.starts_with("CRAWLER_API_BASE_URL="))
                .and_then(|line| line.split('=').nth(1))
                .unwrap_or("")
                .to_string()
        })
    };

    // 한 개의 항목만 요청해서 전체 개수만 확인
    let request_url = format!("{}?size=1", base_url);

    match state.client.get(&request_url).send().await {
        Ok(response) => {
            let status = response.status();

            if status.is_success() {
                match response.text().await {
                    Ok(text_body) => {
                        match serde_json::from_str::<ApiResponse>(&text_body) {
                            Ok(api_response) => {
                                if let Value::Object(data_obj) = &api_response.data {
                                    match serde_json::from_value::<DataWrapper>(Value::Object(
                                        data_obj.clone(),
                                    )) {
                                        Ok(data_wrapper) => {
                                            return Ok(data_wrapper.pageInfo.totalItems);
                                        }
                                        Err(e) => {
                                            return Err(format!("[get_list_count] Failed to parse data into DataWrapper: {}", e));
                                        }
                                    }
                                } else {
                                    // 데이터가 없거나 접근할 수 없는 경우
                                    return Ok(0);
                                }
                            }
                            Err(e) => {
                                return Err(format!("[get_list_count] JSON parsing error: {}", e));
                            }
                        }
                    }
                    Err(e) => {
                        return Err(format!(
                            "[get_list_count] Failed to read response text: {}",
                            e
                        ));
                    }
                }
            } else {
                return Err(format!("[get_list_count] Server error: {}", status));
            }
        }
        Err(e) => {
            return Err(format!("[get_list_count] Request error: {}", e));
        }
    }
}

/// 키워드 파일 삭제 함수
#[tauri::command]
pub fn delete_keyword_file() -> Result<(), String> {
    let keyword_path = std::env::temp_dir().join("mcplink_last_keyword.txt");
    
    if keyword_path.exists() {
        match fs::remove_file(&keyword_path) {
            Ok(_) => {
                // 로그 기록
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
                        "[{}] 키워드 파일 삭제됨: {:?}",
                        chrono::Local::now().format("%H:%M:%S"),
                        keyword_path
                    );
                }
                Ok(())
            }
            Err(e) => Err(format!("키워드 파일 삭제 실패: {}", e))
        }
    } else {
        Ok(()) // 파일이 없으면 성공으로 처리
    }
}

/// 키워드 파일의 나이를 확인하는 함수
#[derive(Serialize)]
pub struct KeywordFileInfo {
    pub keyword: String,
    pub age_ms: u64,
}

#[tauri::command]
pub fn check_keyword_file_age() -> Result<Option<KeywordFileInfo>, String> {
    let keyword_path = std::env::temp_dir().join("mcplink_last_keyword.txt");
    
    if !keyword_path.exists() {
        return Ok(None);
    }
    
    // 파일 메타데이터 읽기
    match fs::metadata(&keyword_path) {
        Ok(metadata) => {
            // 파일 수정 시간 가져오기
            match metadata.modified() {
                Ok(modified_time) => {
                    // 현재 시간과의 차이 계산
                    match modified_time.elapsed() {
                        Ok(duration) => {
                            let age_ms = duration.as_millis() as u64;
                            
                            // 파일 내용 읽기
                            match fs::read_to_string(&keyword_path) {
                                Ok(keyword) => {
                                    let keyword = keyword.trim().to_string();
                                    if keyword.is_empty() {
                                        return Ok(None);
                                    }
                                    
                                    Ok(Some(KeywordFileInfo {
                                        keyword,
                                        age_ms,
                                    }))
                                }
                                Err(_) => Ok(None)
                            }
                        }
                        Err(_) => Ok(None)
                    }
                }
                Err(_) => Ok(None)
            }
        }
        Err(_) => Ok(None)
    }
}

/// 설치된 MCP 개수를 가져오는 함수
#[tauri::command]
pub fn get_installed_mcp_count(app: AppHandle) -> Result<usize, String> {
    // TODO: 실제 MCP 설치 확인 로직 구현
    // 현재는 더미 데이터 반환
    Ok(3)
}

/// 키워드 상태를 완전히 정리하는 함수
#[tauri::command]
pub fn clear_keyword_state(app: AppHandle) -> Result<(), String> {
    // 디버그 로그 설정
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
            "[{}] === clear_keyword_state 시작 ===",
            chrono::Local::now().format("%H:%M:%S")
        );
    }

    // 1. 키워드 파일들 삭제
    let temp_dir = std::env::temp_dir();
    let files_to_clean = [
        temp_dir.join("mcplink_last_keyword.txt"),
        temp_dir.join("mcplink_keyword_update.txt"),
    ];

    for file_path in files_to_clean.iter() {
        if file_path.exists() {
            if let Err(e) = std::fs::remove_file(file_path) {
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&log_path)
                {
                    use std::io::Write;
                    let _ = writeln!(
                        file,
                        "[{}] 파일 삭제 실패: {:?} - {}",
                        chrono::Local::now().format("%H:%M:%S"),
                        file_path,
                        e
                    );
                }
            } else {
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&log_path)
                {
                    use std::io::Write;
                    let _ = writeln!(
                        file,
                        "[{}] 파일 삭제 성공: {:?}",
                        chrono::Local::now().format("%H:%M:%S"),
                        file_path
                    );
                }
            }
        }
    }

    // 2. KeywordState 정리
    if let Some(keyword_state) = app.try_state::<crate::notification_system::KeywordState>() {
        keyword_state.clear_all();
        
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path)
        {
            use std::io::Write;
            let _ = writeln!(
                file,
                "[{}] KeywordState 정리 완료",
                chrono::Local::now().format("%H:%M:%S")
            );
        }
    }

    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] === clear_keyword_state 완료 ===",
            chrono::Local::now().format("%H:%M:%S")
        );
    }

    Ok(())
}