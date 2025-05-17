// app/src-tauri/src/commands.rs

use dotenvy::dotenv; // Used to load .env at runtime in development mode
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::os::windows::process::CommandExt;
use std::{env, fs, path::PathBuf, process::Command as StdCommand}; // env added for accessing environment variables at runtime
use tauri::{AppHandle, Manager, State, Emitter, WebviewWindowBuilder, WebviewUrl};
use tauri_plugin_notification::NotificationExt;
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

// State struct for saving notification keywords
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotificationState {
    pub pending_keyword: Option<String>,
    pub timestamp: Option<i64>,
    pub needs_activation: bool, // Whether app activation is needed
}

// Create mutex for notification state (global state management)
lazy_static::lazy_static! {
    static ref NOTIFICATION_STATE: std::sync::Mutex<NotificationState> = {

        std::sync::Mutex::new(NotificationState {
            pending_keyword: Some(" ".to_string()),
            timestamp: Some(0),
            needs_activation: false, // Initial state: activation not needed
        })
    };
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
    // Load .env file in development mode (ignored if already loaded)
    #[cfg(debug_assertions)]
    let _ = dotenv();

    // Get environment variables at runtime
    let base_url: String = if cfg!(debug_assertions) {
        // Development mode: get from environment variable
        env::var("CRAWLER_API_BASE_URL").unwrap_or_else(|_| String::new())
    } else {
        // Deployment mode: include the value from .env file at compile time
        env::var("CRAWLER_API_BASE_URL").unwrap_or_else(|_| include_str!("../../.env")
            .lines()
            .find(|line| line.starts_with("CRAWLER_API_BASE_URL="))
            .and_then(|line| line.split('=').nth(1))
            .unwrap_or("")
            .to_string())
    };

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
                format!("{}/search?name={}&size=10&cursorId={}", base_url, encoded_term, cursor)
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
                                            let cards: Vec<MCPCard> = data_wrapper
                                                .mcpServers
                                                .iter()
                                                .map(|api_card| MCPCard {
                                                    id: api_card.id,
                                                    title: api_card.mcpServers.name.clone(),
                                                    description: api_card
                                                        .mcpServers
                                                        .description
                                                        .clone(),
                                                    url: api_card.url.clone(),
                                                    stars: api_card.stars,
                                                    scanned: api_card.scanned,
                                                })
                                                .collect();

                                            // Extract page information
                                            let end_cursor = match data_wrapper.pageInfo.endCursor {
                                                Some(Value::Number(n)) => {
                                                    n.as_i64().map(|x| x as i32)
                                                }
                                                _ => None,
                                            };

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
                                            return Err(format!("[get_mcp_data] Failed to parse data into DataWrapper: {}", e));
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
                                let msg = format!("[get_mcp_data] JSON parsing error for ApiResponse: {}. Body: {:.500}", e, text_body);
                                return Err(msg);
                            }
                        }
                    }
                    Err(e) => {
                        let msg = format!("[get_mcp_data] Failed to read response text: {}", e);
                        return Err(msg);
                    }
                }
            } else {
                let error_body = response
                    .text()
                    .await
                    .unwrap_or_else(|e| format!("Failed to read error body: {}", e));
                let msg = format!(
                    "[get_mcp_data] Server error for {}: {}. Body: {:.500}",
                    request_url, status, error_body
                );
                return Err(msg);
            }
        }
        Err(e) => {
            let msg = format!("[get_mcp_data] Request error for {}: {}", request_url, e);
            return Err(msg);
        }
    }
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
        env::var("CRAWLER_API_BASE_URL").unwrap_or_else(|_| include_str!("../../.env")
            .lines()
            .find(|line| line.starts_with("CRAWLER_API_BASE_URL="))
            .and_then(|line| line.split('=').nth(1))
            .unwrap_or("")
            .to_string())
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
        env::var("CRAWLER_API_BASE_URL").unwrap_or_else(|_| include_str!("../../.env")
            .lines()
            .find(|line| line.starts_with("CRAWLER_API_BASE_URL="))
            .and_then(|line| line.split('=').nth(1))
            .unwrap_or("")
            .to_string())
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
                                                .map(|api_card| MCPCard {
                                                    id: api_card.id,
                                                    title: api_card.mcpServers.name.clone(),
                                                    description: api_card
                                                        .mcpServers
                                                        .description
                                                        .clone(),
                                                    url: api_card.url.clone(),
                                                    stars: api_card.stars,
                                                    scanned: api_card.scanned,
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

#[tauri::command]
pub async fn show_popup(app: AppHandle, tag: String) -> Result<(), String> {
    // Choose between notification modal window and system notification
    let use_modal_window = true; // Change this to false to use system notifications instead
    
    if use_modal_window {
        // Use modal window approach
        show_modal_notification(app, tag).await
    } else {
        // Use system notification approach (original implementation)
        show_system_notification(app, tag).await
    }
}

/// Shows a modal window notification for keyword suggestions
#[tauri::command]
pub async fn show_modal_notification(app: AppHandle, tag: String) -> Result<(), String> {
    // Clear any existing pending keyword first
    if let Err(e) = clear_pending_notification_keyword() {
        eprintln!("[Modal] Failed to clear previous keywords: {}", e);
    }

    // Set the pending keyword in the backend state
    if let Err(e) = set_pending_notification_keyword(tag.clone()) {
        eprintln!("[Modal] Failed to set pending keyword: {}", e);
        return Err(format!("Failed to set pending keyword: {}", e));
    }

    // URL-encode the keyword for passing as a URL parameter
    let encoded_keyword = urlencoding::encode(&tag);
    
    // Create a new modal window for the notification
    let modal_window = tauri::WebviewWindowBuilder::new(
        &app,
        "notification-modal", // window label
        tauri::WebviewUrl::App(format!("/modal-notification?keyword={}", encoded_keyword).into()),
    )
    .title("키워드 추천")
    .center()
    .decorations(false)
    .always_on_top(true)
    .focused(true)
    .inner_size(400.0, 250.0)
    .resizable(false)
    .skip_taskbar(true) // Don't show in taskbar
    .visible(true);
    
    // Create the window
    match modal_window.build() {
        Ok(window) => {
            println!("[Modal] Successfully created notification modal window");
            
            // Set focus to ensure it's visible to the user
            if let Err(e) = window.set_focus() {
                eprintln!("[Modal] Failed to set focus to modal window: {}", e);
            }
        },
        Err(e) => {
            eprintln!("[Modal] Failed to create notification window: {}", e);
            return Err(format!("Failed to create notification window: {}", e));
        }
    }

    Ok(())
}

/// Shows a system notification for keyword suggestions (original implementation)
#[tauri::command]
pub async fn show_system_notification(app: AppHandle, tag: String) -> Result<(), String> {
    // Clear any existing pending keyword first
    if let Err(e) = clear_pending_notification_keyword() {
        eprintln!("[Notification] Failed to clear previous keywords: {}", e);
    }

    // Set the pending keyword in the backend state
    if let Err(e) = set_pending_notification_keyword(tag.clone()) {
        eprintln!("[Notification] Failed to set pending keyword: {}", e);
        return Err(format!("Failed to set pending keyword: {}", e));
    }

    // Create notification body with the tag information
    let notification_body = format!("Selected keyword: {}. Click to confirm.", tag);

    // Create and display notification
    // Add settings for a more prominent notification
    let builder = app
        .notification()
        .builder()
        .title("키워드 추천 확인") // Korean title (more noticeable and clear)
        .body(&notification_body)
        .icon("icons/icon.png");

    // Set notification priority (if supported by the system)
    #[cfg(target_os = "windows")]
    let builder = builder.sound("Default"); // Add sound effect on Windows

    // Display notification
    match builder.show() {
        Ok(_) => println!("[Notification] Enhanced notification sent successfully"),
        Err(e) => {
            eprintln!("[Notification] Notification error: {}", e);
            return Err(format!("Failed to show notification: {}", e));
        }
    }

    // Clone variables for the async task
    let app_clone = app.clone();
    let _tag_clone = tag.clone(); // _tag_clone to avoid warning when variable is not used

    // Start the delayed check task - use a shorter delay (1 second)
    tauri::async_runtime::spawn(async move {
        // Give the user time to click the notification (reduced to 1 second)
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        // Check if the keyword is still pending (if the notification was not clicked)
        let keyword_still_pending = match get_pending_notification_keyword() {
            Ok(keyword) => !keyword.is_empty(),
            Err(_) => false,
        };

        // If the keyword is still pending, force activate the app
        if keyword_still_pending {
            // Window activation (1): Check if the app window is already visible
            if let Some(window) = app_clone.get_webview_window("main") {
                // Check if the window is visible
                let is_visible = window.is_visible().unwrap_or(false);

                // If the window is not visible, show it
                if !is_visible {
                    if let Err(e) = window.show() {
                        eprintln!("[Notification] Failed to show window: {}", e);
                    }
                }

                // If the window is minimized, restore it
                if window.is_minimized().unwrap_or(false) {
                    if let Err(e) = window.unminimize() {
                        eprintln!("[Notification] Failed to unminimize window: {}", e);
                    }
                }

                // Set always on top and activate focus
                if let Err(e) = window.set_always_on_top(true) {
                    eprintln!(
                        "[Notification] Failed to set window always on top: {}",
                        e
                    );
                }

                if let Err(e) = window.set_focus() {
                    eprintln!("[Notification] Failed to set focus: {}", e);
                }
            }

            // Window activation (2): Call the core activation function (this actually handles keyword processing and window activation)
            match check_and_mark_app_activated(app_clone).await {
                Ok(keyword_opt) => match keyword_opt {
                    Some(k) => println!(
                        "[Notification] Successfully activated app with keyword: {}",
                        k
                    ),
                    None => println!("[Notification] App activated but no keyword was found"),
                },
                Err(e) => eprintln!("[Notification] Failed to activate app: {}", e),
            }
        } else {
        }
    });

    Ok(())
}

/// Activates the main window and navigates to MCP list with a keyword
#[tauri::command]
pub async fn activate_main_window_with_keyword(app: AppHandle, keyword: String) -> Result<(), String> {
    // Get the main window
    let main_window = app.get_webview_window("main").ok_or("Main window not found")?;
    
    // 1. Show the window if it's hidden
    if !main_window.is_visible().unwrap_or(false) {
        if let Err(e) = main_window.show() {
            eprintln!("[Activation] Failed to show main window: {}", e);
        }
    }
    
    // 2. Unminimize if minimized
    if main_window.is_minimized().unwrap_or(false) {
        if let Err(e) = main_window.unminimize() {
            eprintln!("[Activation] Failed to unminimize main window: {}", e);
        }
    }
    
    // 3. Set always on top and focus
    if let Err(e) = main_window.set_always_on_top(true) {
        eprintln!("[Activation] Failed to set main window always on top: {}", e);
    }
    
    if let Err(e) = main_window.set_focus() {
        eprintln!("[Activation] Failed to set focus to main window: {}", e);
    }
    
    // 4. Navigate to MCP list with keyword
    // Format the URL to navigate to
    let navigate_url = format!("/MCP-list?keyword={}", urlencoding::encode(&keyword));
    
    // Emit an event to the frontend to navigate to the MCP list with the keyword
    if let Err(e) = main_window.emit("navigate-to-mcp-list-with-keyword", navigate_url) {
        eprintln!("[Activation] Failed to emit navigation event: {}", e);
        return Err(format!("Failed to navigate to MCP list: {}", e));
    }
    
    // 5. Schedule removal of always-on-top after a delay
    let main_window_clone = main_window.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        if let Err(e) = main_window_clone.set_always_on_top(false) {
            eprintln!("[Activation] Failed to remove always-on-top: {}", e);
        }
    });
    
    // 6. Update notification state to mark the keyword as processed
    if let Err(e) = clear_pending_notification_keyword() {
        eprintln!("[Activation] Failed to clear notification keyword: {}", e);
    }
    
    Ok(())
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
        env::var("CRAWLER_API_BASE_URL").unwrap_or_else(|_| include_str!("../../.env")
            .lines()
            .find(|line| line.starts_with("CRAWLER_API_BASE_URL="))
            .and_then(|line| line.split('=').nth(1))
            .unwrap_or("")
            .to_string())
    };
    
    let gui_be_api_base_url = if cfg!(debug_assertions) {
        // Development mode: get from environment variable
        env::var("GUI_BE_API_BASE_URL").unwrap_or_else(|_| String::new())
    } else {
        // Deployment mode: include the value from .env file at compile time
        env::var("GUI_BE_API_BASE_URL").unwrap_or_else(|_| include_str!("../../.env")
            .lines()
            .find(|line| line.starts_with("GUI_BE_API_BASE_URL="))
            .and_then(|line| line.split('=').nth(1))
            .unwrap_or("")
            .to_string())
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
            env_map.insert("CRAWLER_API_BASE_URL".to_string(), Value::String(crawler_api_base_url));
        }
        if !gui_be_api_base_url.is_empty() {
            env_map.insert("GUI_BE_API_BASE_URL".to_string(), Value::String(gui_be_api_base_url));
        }
        env_map.insert("NODE_ENV".to_string(), Value::String("development".to_string()));
        
        let server_config = MCPServerConfig {
            command: "node".to_string(),
            args: Some(vec!["C:\\Users\\SSAFY\\Desktop\\0516\\S12P31A201\\mcp-server\\dist\\main.js".to_string()]),
            cwd: Some("C:\\Users\\SSAFY\\Desktop\\0516\\S12P31A201\\mcp-server".to_string()),
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
                                    let _ = window.emit("config-files-missing", json!({
                                        "claudeConfigExists": claude_exists,
                                        "mcplinkConfigExists": mcplink_exists
                                    }));
                                }
                            }
                        },
                        Err(e) => eprintln!("Error checking mcplink config: {}", e),
                    }
                },
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
        return Err("Claude configuration file does not exist".to_string());
    }

    // Read configuration file
    let config_str = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    // Parse configuration
    let config: ClaudeDesktopConfig = serde_json::from_str(&config_str)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    // Check MCP server configuration
    let servers = config.mcpServers.unwrap_or_default();

    // Get configuration by specific server name
    if let Some(server_config) = servers.get(&server_name) {
        Ok(server_config.clone())
    } else {
        Err(format!(
            "MCP server '{}' not found in configuration",
            server_name
        ))
    }
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

/// Function to return the current time as a Unix timestamp
fn get_current_time() -> Result<i64, String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Failed to get current time: {}", e))
        .map(|d| d.as_secs() as i64)
}

/// Function to set the pending notification keyword
#[tauri::command]
pub fn set_pending_notification_keyword(keyword: String) -> Result<(), String> {
    let mut state = NOTIFICATION_STATE
        .lock()
        .map_err(|e| format!("Failed to lock notification state: {}", e))?;

    // Current time (Unix timestamp)
    let now = get_current_time()?;

    // Update state
    state.pending_keyword = Some(keyword.clone());
    state.timestamp = Some(now);
    state.needs_activation = true; // Setting a notification means app activation is needed

    Ok(())
}

/// Function to get any pending notification keyword
/// Returns the keyword if one exists and is not expired, or an empty string otherwise
#[tauri::command]
pub fn get_pending_notification_keyword() -> Result<String, String> {
    // Acquire the lock on the notification state
    let mut state = NOTIFICATION_STATE
        .lock()
        .map_err(|e| format!("Failed to lock notification state: {}", e))?;

    // Check if we have both a keyword and a timestamp
    if let (Some(keyword), Some(timestamp)) = (&state.pending_keyword, &state.timestamp) {
        // Check if the keyword has expired
        let now = get_current_time().unwrap_or(0);

        // Auto-clear keywords that are more than 30 seconds old
        const KEYWORD_TIMEOUT_SECS: i64 = 30;
        if now - timestamp > KEYWORD_TIMEOUT_SECS {
            // Clear the expired keyword
            state.pending_keyword = None;
            state.timestamp = None;
            state.needs_activation = false;

            return Ok("".to_string());
        }

        // If keyword exists and is not expired, return it
        return Ok(keyword.clone());
    }

    // No keyword found - return empty string
    // Limit logging to every few checks to prevent log spam
    static mut COUNT: u32 = 0;
    unsafe {
        COUNT += 1;
        if COUNT % 10 == 0 {}
    }

    Ok("".to_string())
}

/// Activates the app window and processes any pending notification keywords
/// Returns the keyword that was processed, if any
#[tauri::command]
pub async fn check_and_mark_app_activated(app: AppHandle) -> Result<Option<String>, String> {
    // Extract keyword and reset state in a separate scope to ensure MutexGuard is dropped
    let keyword = {
        // Get the notification state
        let mut state = NOTIFICATION_STATE
            .lock()
            .map_err(|e| format!("Failed to lock notification state: {}", e))?;

        // Extract the keyword if there is one
        let keyword = state.pending_keyword.clone();

        // Only reset state if we actually had a keyword
        if keyword.is_some() {
            // Reset notification state
            state.pending_keyword = None;
            state.timestamp = None;
            state.needs_activation = false;
        }

        // Return the keyword (will be None if no keyword was present)
        keyword

        // MutexGuard is automatically dropped here when this scope ends
    };

    // Process the keyword if we have one
    if let Some(ref _kw) = keyword { // _kw to avoid warning when variable is not used
        // Get the main window and activate it with several different methods
        if let Some(window) = app.get_webview_window("main") {
            // Enhanced window activation sequence

            // 1. Basic show operation
            if let Err(e) = window.show() {
                eprintln!("[NotificationState] Failed to show window: {}", e);
            }

            // 2. Restore if minimized
            if let Err(e) = window.unminimize() {
                eprintln!("[NotificationState] Failed to unminimize window: {}", e);
            }

            // 3. Set window to always on top
            if let Err(e) = window.set_always_on_top(true) {
                eprintln!(
                    "[NotificationState] Failed to set window always on top: {}",
                    e
                );
            }

            // 4. Activate and set focus (try multiple times)
            for _ in 0..3 {
                if let Err(e) = window.set_focus() {
                    eprintln!("[NotificationState] Focus attempt failed: {}", e);
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }

            // 5. Request user attention to force focus (flashes taskbar on Windows)
            if let Err(e) = window.request_user_attention(Some(tauri::UserAttentionType::Critical))
            {
                eprintln!(
                    "[NotificationState] Failed to request user attention: {}",
                    e
                );
            }

            // 6. Try focusing again after a short delay (needed on some OS)
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            if let Err(e) = window.set_focus() {
                eprintln!("[NotificationState] Final focus attempt failed: {}", e);
            }

            // 7. Special measure: Windows-specific focus command - changed to a non-resizing method
            #[cfg(target_os = "windows")]
            {
                // 1. First, use a simple approach to ensure the window is definitely shown
                let _ = window.show();
                let _ = window.unminimize();

                // 2. Request focus multiple times
                for _ in 0..5 {
                    let _ = window.set_focus();
                    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
                }

                // 3. Request user attention (use Informational instead of Critical for less aggression)
                let _ =
                    window.request_user_attention(Some(tauri::UserAttentionType::Informational));

                // 4. Use always-on-top property for topmost display
                let _ = window.set_always_on_top(true);

                // 5. Use Tauri's built-in window centering function
                let _ = window.center();

                // 6. Focus again
                let _ = window.set_focus();
            }

            // 8. Set timer to automatically disable always-on-top state
            // Automatically disable always on top after 5 seconds
            let window_clone = window.clone();
            tauri::async_runtime::spawn(async move {
                // 5-second delay
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                // Disable always on top state
                if let Err(e) = window_clone.set_always_on_top(false) {
                    eprintln!("[NotificationState] Failed to remove always-on-top: {}", e);
                }
            });
        } else {
            eprintln!("[NotificationState] Main window not found");
        }

        return Ok(keyword);
    }

    // No keyword was present
    Ok(None)
}

/// Function to check if the app is active
#[tauri::command]
pub fn is_app_active(app: AppHandle) -> Result<bool, String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Failed to get main window".to_string())?;

    // Check if the window is visible
    let is_visible = window.is_visible().unwrap_or(false);

    Ok(is_visible)
}

/// Function to clear the pending notification keyword
#[tauri::command]
pub fn clear_pending_notification_keyword() -> Result<(), String> {
    let mut state = NOTIFICATION_STATE
        .lock()
        .map_err(|e| format!("Failed to lock notification state: {}", e))?;

    // Initialize state
    state.pending_keyword = None;
    state.timestamp = None;
    state.needs_activation = false;

    Ok(())
}

/// Function to reset MCP settings (excluding fallback server)
#[tauri::command]
pub fn reset_mcp_settings(app: AppHandle) -> Result<(), String> {
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
                let fallback_server = servers_map.remove("McpFallbackServer")
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
