// app/src-tauri/src/commands.rs

use crate::force_activate;
use crate::{api_error, fs_error, to_string_error, AppError, AppResult};
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

// function to return default security rank value "UNRATE"
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
    // new internal function implementation (use AppResult)
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

        if base_url.is_empty() {
            return Err(AppError::ConfigError {
                msg: "API base URL is not set".to_string(),
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

        // perform API request
        let response = client
            .get(&request_url)
            .send()
            .await
            .map_err(|e| api_error(e, None))?;

        let status = response.status();
        let status_code = status.as_u16();

        if status.is_success() {
            // read text body when response is successful
            let text_body = response.text().await.map_err(|e| {
                api_error(
                    format!("failed to read response text: {}", e),
                    Some(status_code),
                )
            })?;

            // parse ApiResponse
            let api_response: ApiResponse =
                serde_json::from_str(&text_body).map_err(|e| AppError::JsonError {
                    msg: format!("error parsing ApiResponse: {}", e),
                })?;

            // handle data
            if let Value::Object(data_obj) = &api_response.data {
                // wrap data with DataWrapper
                let data_wrapper: DataWrapper =
                    serde_json::from_value(Value::Object(data_obj.clone())).map_err(|e| {
                        AppError::JsonError {
                            msg: format!("error parsing DataWrapper: {}", e),
                        }
                    })?;

                // map card data
                let cards: Vec<MCPCard> = data_wrapper
                    .mcpServers
                    .iter()
                    .map(|api_card| {
                        // get security rank value from API (default to "UNRATE" if not present)
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

                // extract page information
                let end_cursor = match data_wrapper.pageInfo.endCursor {
                    Some(Value::Number(n)) => n.as_i64().map(|x| x as i32),
                    _ => None,
                };

                // create response
                return Ok(MCPCardResponse {
                    cards,
                    page_info: PageInfoResponse {
                        has_next_page: data_wrapper.pageInfo.hasNextPage,
                        end_cursor: end_cursor,
                        total_items: data_wrapper.pageInfo.totalItems,
                    },
                });
            } else {
                // return empty response if data is missing or not an object
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
            // handle error response
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|e| format!("failed to read error body: {}", e));

            return Err(api_error(
                format!("server error: {} - {}", status, error_body),
                Some(status_code),
            ));
        }
    }

    // execute internal function and convert error
    to_string_error(get_mcp_data_internal(&state.client, search_term, cursor_id).await)
}

#[tauri::command]
pub async fn get_mcp_detail_data(
    state: State<'_, AppState>,
    id: i32,
) -> Result<MCPCardDetail, String> {
    // load
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
                                        // get security rank value from API (default to "UNRATE" if not present)
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

// Add MCP server configuration to Claude Desktop config file
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

// Remove MCP server configuration from Claude Desktop config file
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

// Restart Claude Desktop application
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
        .creation_flags(0x08000000) // Add CREATE_NO_WINDOW flag
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
        .creation_flags(0x08000000) // Add CREATE_NO_WINDOW flag
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
                                                    // get security rank value from API (default to "UNRATE" if not present)
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

// Function to directly search for locally installed MCP servers
// Reads the configuration file directly, filters, and returns the results
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

// Struct for local MCP server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalMCPServer {
    pub id: i32,
    pub name: String,
    pub command: String,
    pub args: Option<Vec<String>>,
}

// Reads the content of mcplink_desktop_config.json and returns it as a string.
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

// Ensures config files exist with default values
// Creates them if they don't exist
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

// Monitors both configuration files and emits an event if either is missing
// This function should be called when you want to start watching the config files
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
                        Err(e) => (),
                    }
                }
                Err(e) => (),
            }
        }
    });

    Ok(())
}

// Function to read the detailed settings of an installed MCP server
#[tauri::command]
pub fn read_mcp_server_config(
    app: AppHandle,
    server_name: String,
) -> Result<MCPServerConfig, String> {
    // implement new internal function (use AppResult)
    fn read_mcp_server_config_internal(
        app: &AppHandle,
        server_name: &str,
    ) -> AppResult<MCPServerConfig> {
        // Create Claude directory path
        let claude_dir = match std::env::consts::OS {
            "windows" => {
                let appdata = app.path().app_data_dir().map_err(|e| AppError::OsError {
                    msg: format!("Failed to get AppData directory: {}", e),
                })?;
                appdata.parent().unwrap_or(&appdata).join("Claude")
            }
            _ => {
                return Err(AppError::OsError {
                    msg: "Currently only supported on Windows".to_string(),
                });
            }
        };

        // Claude Desktop configuration file path
        let config_path = claude_dir.join("claude_desktop_config.json");

        if !config_path.exists() {
            return Err(AppError::ConfigError {
                msg: "Claude config file does not exist".to_string(),
            });
        }

        // Read configuration file
        let config_str = fs::read_to_string(&config_path)
            .map_err(|e| fs_error(e, &config_path.to_string_lossy()))?;

        // Parse configuration
        let config: ClaudeDesktopConfig =
            serde_json::from_str(&config_str).map_err(|e| AppError::JsonError {
                msg: format!("Failed to parse config file: {}", e),
            })?;

        // Check MCP server configuration
        let servers = config.mcpServers.unwrap_or_default();

        // Get configuration by specific server name
        if let Some(server_config) = servers.get(server_name) {
            Ok(server_config.clone())
        } else {
            Err(AppError::ConfigError {
                msg: format!("MCP server '{}' is not in the config", server_name),
            })
        }
    }

    // execute internal function and convert error
    to_string_error(read_mcp_server_config_internal(&app, &server_name))
}

// Function to check if a server name is installed
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

// Function to reset MCP settings (excluding fallback server)
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

// check app activation state and show keyword if exists - improved version
#[tauri::command]
pub fn check_and_mark_app_activated(app: AppHandle) -> Result<Option<String>, String> {
    // 1. first check keyword file
    let keyword_path = std::env::temp_dir().join("mcplink_last_keyword.txt");
    let file_keyword = if keyword_path.exists() {
        // check file modification time
        if let Ok(metadata) = std::fs::metadata(&keyword_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    // ignore files older than 10 seconds and delete
                    if elapsed.as_secs() > 10 {
                        let _ = std::fs::remove_file(&keyword_path);
                        None
                    } else {
                        // read file if it's recent
                        match std::fs::read_to_string(&keyword_path) {
                            Ok(content) => {
                                let trimmed = content.trim();
                                if !trimmed.is_empty() {
                                    Some(trimmed.to_string())
                                } else {
                                    None
                                }
                            }
                            Err(_e) => None,
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
        None
    };

    // 2. check KeywordState
    let state_keyword =
        if let Some(keyword_state) = app.try_state::<crate::notification_system::KeywordState>() {
            // set app activated state
            keyword_state.set_app_activated(true);

            // check pending keyword
            if keyword_state.is_pending() || keyword_state.has_keyword() {
                let kw = keyword_state.take_keyword();
                kw
            } else {
                None
            }
        } else {
            None
        };

    // 3. final keyword decision (file first)
    let final_keyword = file_keyword.or(state_keyword);

    // 4. if keyword exists, emit event
    if let Some(ref keyword) = final_keyword {
        if let Some(window) = app.get_webview_window("main") {
            use tauri::Emitter;
            let _result = window.emit("search-keyword", keyword.clone());
        }

        // after keyword processing, delete file
        if keyword_path.exists() {
            let _ = std::fs::remove_file(&keyword_path);
        }
    }

    Ok(final_keyword)
}

// notify app activation and bring app to front - improved logging
#[tauri::command]
pub fn notify_app_activated(app: AppHandle) -> Result<(), String> {
    // update app activated state (use KeywordState)
    if let Some(keyword_state) = app.try_state::<crate::notification_system::KeywordState>() {
        // set app activated state
        keyword_state.set_app_activated(true);

        // record time difference between app activation and most recent notification click
        let time_diff = keyword_state
            .get_last_click_time()
            .map(|t| {
                let elapsed = t.elapsed();
                format!("{} seconds ago", elapsed.as_secs())
            })
            .unwrap_or_else(|| "unknown".to_string());

        // log message removed for performance
    }

    // emit app activated event
    let result = crate::force_activate::emit_app_activated_event(&app);

    // log messages removed for performance

    result
}

// get installed MCP count
#[tauri::command]
pub fn get_installed_count(app: AppHandle) -> Result<i32, String> {
    // read mcplink_desktop_config.json file
    let mcplink_content = read_mcplink_config_content(app)?;

    // parse file content
    let config: Map<String, Value> = serde_json::from_str(&mcplink_content)
        .map_err(|e| format!("Failed to parse mcplink config: {}", e))?;

    // calculate installed MCP count (exclude fallback server ID -1)
    let count = config.keys().filter(|&key| key != "-1").count() as i32;

    Ok(count)
}

// get total MCP list count
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

    // request only one item to get total count
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
                                    // no data or inaccessible
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

// delete keyword file
#[tauri::command]
pub fn delete_keyword_file() -> Result<(), String> {
    let keyword_path = std::env::temp_dir().join("mcplink_last_keyword.txt");

    if keyword_path.exists() {
        match fs::remove_file(&keyword_path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("failed to delete keyword file: {}", e)),
        }
    } else {
        Ok(()) // if file not found, success
    }
}

// check keyword file age
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

    // read file metadata
    match fs::metadata(&keyword_path) {
        Ok(metadata) => {
            // get file modification time
            match metadata.modified() {
                Ok(modified_time) => {
                    // calculate time difference between current time and file modification time
                    match modified_time.elapsed() {
                        Ok(duration) => {
                            let age_ms = duration.as_millis() as u64;

                            // read file content
                            match fs::read_to_string(&keyword_path) {
                                Ok(keyword) => {
                                    let keyword = keyword.trim().to_string();
                                    if keyword.is_empty() {
                                        return Ok(None);
                                    }

                                    Ok(Some(KeywordFileInfo { keyword, age_ms }))
                                }
                                Err(_) => Ok(None),
                            }
                        }
                        Err(_) => Ok(None),
                    }
                }
                Err(_) => Ok(None),
            }
        }
        Err(_) => Ok(None),
    }
}

// get installed MCP count
#[tauri::command]
pub fn get_installed_mcp_count(app: AppHandle) -> Result<usize, String> {
    // TODO: implement actual MCP installation check logic
    // currently, return dummy data
    Ok(3)
}

// clean up keyword state
#[tauri::command]
pub fn clear_keyword_state(app: AppHandle) -> Result<(), String> {
    // 1. delete keyword files
    let temp_dir = std::env::temp_dir();
    let files_to_clean = [
        temp_dir.join("mcplink_last_keyword.txt"),
        temp_dir.join("mcplink_keyword_update.txt"),
    ];

    for file_path in files_to_clean.iter() {
        if file_path.exists() {
            let _ = std::fs::remove_file(file_path);
        }
    }

    // 2. clean up KeywordState
    if let Some(keyword_state) = app.try_state::<crate::notification_system::KeywordState>() {
        keyword_state.clear_all();
    }

    Ok(())
}
