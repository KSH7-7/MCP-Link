//! Windows app activation and focus force module

use tauri::Manager;

#[cfg(target_os = "windows")]
use std::ptr::null_mut;

#[cfg(target_os = "windows")]
use windows::core::PCWSTR;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    keybd_event, KEYBD_EVENT_FLAGS, VIRTUAL_KEY, VK_MENU,
};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    AllowSetForegroundWindow, BringWindowToTop, EnumWindows, FindWindowW, GetClassNameW,
    GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindowVisible,
    SetForegroundWindow, SetWindowPos, ShowWindow, ASFW_ANY, HWND_NOTOPMOST, HWND_TOPMOST,
    SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SW_RESTORE, SW_SHOW, SW_SHOWNORMAL,
};

#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;

// application window name
static WINDOW_NAME: &str = "MCP Link";

// application class name
static CLASS_NAME: &str = "Tauri Window";

// possible alternative window names
static ALT_WINDOW_NAMES: [&str; 15] = [
    "MCP Link",
    "MCPLink",
    "MCP-Link",
    "MCPLINK",
    "McpLink",
    "Mcp Link",
    "mcp link",
    "mcplink",
    "tauri app",
    "Tauri App",
    "Tauri",
    "TAURI",
    "Tauri Application",
    "MCP",
    "Link",
];

// Helper function for wide string conversion
#[cfg(target_os = "windows")]
fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

// data structure for EnumWindows callback function
#[cfg(target_os = "windows")]
struct EnumWindowsState {
    log_path: std::path::PathBuf,
    found_hwnd: HWND,
    target_pid: Option<u32>,
}

// EnumWindows callback function
#[cfg(target_os = "windows")]
extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let state = lparam.0 as *mut EnumWindowsState;

        if !IsWindowVisible(hwnd).as_bool() {
            return true.into();
        }

        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        if let Some(target_pid) = (*state).target_pid {
            if process_id == target_pid {
                let mut title_buf = [0u16; 512];
                let title_len = GetWindowTextW(hwnd, title_buf.as_mut_slice());
                let title = if title_len > 0 {
                    let title_slice = &title_buf[0..title_len as usize];
                    String::from_utf16_lossy(title_slice)
                } else {
                    String::new()
                };

                let mut class_buf = [0u16; 256];
                let class_len = GetClassNameW(hwnd, class_buf.as_mut_slice());
                let class_name = if class_len > 0 {
                    let class_slice = &class_buf[0..class_len as usize];
                    String::from_utf16_lossy(class_slice)
                } else {
                    String::new()
                };

                (*state).found_hwnd = hwnd;
                return false.into();
            }
        }

        let mut title_buf = [0u16; 512];
        let title_len = GetWindowTextW(hwnd, title_buf.as_mut_slice());
        let title = if title_len > 0 {
            let title_slice = &title_buf[0..title_len as usize];
            String::from_utf16_lossy(title_slice)
        } else {
            String::new()
        };

        let mut class_buf = [0u16; 256];
        let class_len = GetClassNameW(hwnd, class_buf.as_mut_slice());
        let class_name = if class_len > 0 {
            let class_slice = &class_buf[0..class_len as usize];
            String::from_utf16_lossy(class_slice)
        } else {
            String::new()
        };

        let title_lower = title.to_lowercase();
        let class_lower = class_name.to_lowercase();

        let is_backup_target = (title == "MCP Link" && class_name == "Tauri Window")
            || (title == "MCP Link" && class_lower.contains("tauri"))
            || (title_lower.contains("mcp") && class_lower.contains("tauri"))
            || (title_lower.contains("mcp") && title_lower.contains("link"))
            || (class_lower.contains("tauri"))
            || (class_lower.contains("webview") && title_lower.contains("mcp"));

        if (*state).found_hwnd.0 == 0 && is_backup_target {
            (*state).found_hwnd = hwnd;
            return false.into();
        }

        true.into()
    }
}

// enumerate all windows to find a suitable window
#[cfg(target_os = "windows")]
fn find_app_window() -> Option<HWND> {
    unsafe {
        let current_pid = std::process::id();

        let mut state = EnumWindowsState {
            log_path: std::path::PathBuf::new(),
            found_hwnd: HWND(0),
            target_pid: Some(current_pid),
        };

        EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut state as *mut _ as isize),
        );

        if state.found_hwnd.0 != 0 {
            Some(state.found_hwnd)
        } else {
            None
        }
    }
}

// structure for activation result
#[derive(Debug, Clone, serde::Serialize)]
pub struct ActivationResult {
    pub success: bool,
    pub method_used: String,
    pub elapsed_ms: u64,
    pub error: Option<String>,
    pub attempts: u32,
    pub timestamp: u64,
}

impl Default for ActivationResult {
    fn default() -> Self {
        Self {
            success: false,
            method_used: String::new(),
            elapsed_ms: 0,
            error: None,
            attempts: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

// find and force activate app on Windows
#[cfg(target_os = "windows")]
pub fn force_app_to_foreground() -> Result<(), String> {
    unsafe {
        // SetForegroundWindow permission allow
        AllowSetForegroundWindow(ASFW_ANY);

        // 1. First, find by class name
        let class_name_wide = to_wide_string("Tauri Window");
        let mut hwnd = FindWindowW(
            PCWSTR::from_raw(class_name_wide.as_ptr()),
            PCWSTR::from_raw(null_mut()),
        );

        // 2. If not found by class name, find by window name
        if hwnd.0 == 0 {
            let window_name_wide = to_wide_string("MCP Link");
            hwnd = FindWindowW(
                PCWSTR::from_raw(null_mut()),
                PCWSTR::from_raw(window_name_wide.as_ptr()),
            );
        }

        // 3. If still not found, use EnumWindows
        if hwnd.0 == 0 {
            if let Some(found) = find_app_window() {
                hwnd = found;
            }
        }

        if hwnd.0 == 0 {
            return Err("app window not found".to_string());
        }

        // get current foreground window
        let current_fg = GetForegroundWindow();
        let mut thread_id = 0;
        let mut current_thread_id = 0;

        if current_fg.0 != 0 && current_fg != hwnd {
            thread_id = GetWindowThreadProcessId(hwnd, None);
            current_thread_id = GetWindowThreadProcessId(current_fg, None);

            // thread connection
            if thread_id != current_thread_id {
                AttachThreadInput(current_thread_id, thread_id, true);
            }
        }

        // if minimized, restore
        if IsIconic(hwnd).as_bool() {
            ShowWindow(hwnd, SW_RESTORE);
            std::thread::sleep(std::time::Duration::from_millis(150));
        }

        // 1st step: show window
        ShowWindow(hwnd, SW_SHOW);
        ShowWindow(hwnd, SW_SHOWNORMAL);

        // 2nd step: simulate Alt key (more stable)
        keybd_event(VK_MENU.0 as u8, 0x38, KEYBD_EVENT_FLAGS(0), 0);
        std::thread::sleep(std::time::Duration::from_millis(50));
        keybd_event(VK_MENU.0 as u8, 0x38, KEYBD_EVENT_FLAGS(2), 0);
        std::thread::sleep(std::time::Duration::from_millis(50));

        // 3rd step: try to set foreground
        let result1 = SetForegroundWindow(hwnd);

        // 4th step: BringWindowToTop
        BringWindowToTop(hwnd);

        // 5th step: set topmost and unset
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
        );

        std::thread::sleep(std::time::Duration::from_millis(100));

        SetWindowPos(
            hwnd,
            HWND_NOTOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
        );

        // 6th step: one more SetForegroundWindow
        std::thread::sleep(std::time::Duration::from_millis(50));
        let result2 = SetForegroundWindow(hwnd);

        // 7th step: additional Alt key trick
        keybd_event(VK_MENU.0 as u8, 0x38, KEYBD_EVENT_FLAGS(0), 0);
        keybd_event(VK_MENU.0 as u8, 0x38, KEYBD_EVENT_FLAGS(2), 0);

        // 8th step: last try
        BringWindowToTop(hwnd);
        SetForegroundWindow(hwnd);

        // thread connection release
        if thread_id != 0 && current_thread_id != 0 && thread_id != current_thread_id {
            AttachThreadInput(current_thread_id, thread_id, false);
        }

        // check result
        std::thread::sleep(std::time::Duration::from_millis(100));
        let final_fg = GetForegroundWindow();
        let success = final_fg == hwnd;

        Ok(())
    }
}

// function called when app is activated
#[cfg(target_os = "windows")]
pub fn emit_app_activated_event<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> Result<(), String> {
    emit_app_activated_event_with_source(app_handle, false)
}

// function called when app is activated with source information
#[cfg(target_os = "windows")]
pub fn emit_app_activated_event_with_source<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
    from_notification: bool,
) -> Result<(), String> {
    use tauri::Emitter;
    if let Some(window) = app_handle.get_webview_window("main") {
        // include notification information in payload
        let payload = serde_json::json!({
            "fromNotification": from_notification
        });

        if let Err(e) = window.emit("app-activated", payload) {
            return Err(format!("error triggering app activation event: {}", e));
        }
    } else {
        return Err("failed to trigger app activation event: main window not found".to_string());
    }

    Ok(())
}

// empty implementation for non-Windows environment
#[cfg(not(target_os = "windows"))]
pub fn emit_app_activated_event<R: tauri::Runtime>(
    _app_handle: &tauri::AppHandle<R>,
) -> Result<(), String> {
    emit_app_activated_event_with_source(_app_handle, false)
}

// function called when app is activated with source information
#[cfg(not(target_os = "windows"))]
pub fn emit_app_activated_event_with_source<R: tauri::Runtime>(
    _app_handle: &tauri::AppHandle<R>,
    _from_notification: bool,
) -> Result<(), String> {
    use tauri::Emitter;
    if let Some(window) = _app_handle.get_webview_window("main") {
        let payload = serde_json::json!({
            "fromNotification": _from_notification
        });
        let _ = window.emit("app-activated", payload);
    }
    Ok(())
}

// empty implementation for non-Windows environment
#[cfg(not(target_os = "windows"))]
pub fn force_app_to_foreground() -> Result<(), String> {
    Ok(())
}

// enhanced app window activation function (used as Tauri command)
#[tauri::command]
#[cfg(target_os = "windows")]
pub fn activate_app_window(app_handle: tauri::AppHandle) -> Result<ActivationResult, String> {
    unsafe {
        let start_time = std::time::Instant::now();
        let mut result = ActivationResult::default();

        // method 1: try to activate using Tauri API
        if let Some(window) = app_handle.get_webview_window("main") {
            let show_result = window.show();
            let unminimize_result = window.unminimize();
            let focus_result = window.set_focus();

            let tauri_success = show_result.is_ok() && focus_result.is_ok();

            if tauri_success {
                result.method_used = "tauri_api".to_string();
                result.attempts += 1;
            }
        }

        // method 2: call Win32 API directly
        AllowSetForegroundWindow(ASFW_ANY);

        let mut hwnd = HWND(0);
        let mut win32_method = "unknown";

        // 1. find by class name
        let class_name_wide = to_wide_string("Tauri Window");
        hwnd = FindWindowW(
            PCWSTR::from_raw(class_name_wide.as_ptr()),
            PCWSTR::from_raw(null_mut()),
        );

        if hwnd.0 != 0 {
            win32_method = "class_name";
        }

        // 2. find by window name
        if hwnd.0 == 0 {
            let window_name_wide = to_wide_string("MCP Link");
            hwnd = FindWindowW(
                PCWSTR::from_raw(null_mut()),
                PCWSTR::from_raw(window_name_wide.as_ptr()),
            );

            if hwnd.0 != 0 {
                win32_method = "window_name";
            }
        }

        // 3. find by EnumWindows
        if hwnd.0 == 0 {
            if let Some(found) = find_app_window() {
                hwnd = found;
                win32_method = "enum_windows";
            }
        }

        if hwnd.0 == 0 {
            result.success = false;
            result.method_used = "failed_window_not_found".to_string();
            result.elapsed_ms = start_time.elapsed().as_millis() as u64;
            result.error = Some("app window not found".to_string());
            return Ok(result);
        }

        // activate window
        if IsIconic(hwnd).as_bool() {
            ShowWindow(hwnd, SW_RESTORE);
            result.attempts += 1;
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        ShowWindow(hwnd, SW_SHOW);
        ShowWindow(hwnd, SW_SHOWNORMAL);
        result.attempts += 1;

        // Alt key simulation
        keybd_event(VK_MENU.0 as u8, 0, KEYBD_EVENT_FLAGS(0), 0);
        keybd_event(VK_MENU.0 as u8, 0, KEYBD_EVENT_FLAGS(2), 0);

        SetForegroundWindow(hwnd);
        BringWindowToTop(hwnd);
        result.attempts += 1;

        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
        );

        std::thread::sleep(std::time::Duration::from_millis(50));

        SetWindowPos(
            hwnd,
            HWND_NOTOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
        );

        SetForegroundWindow(hwnd);
        result.attempts += 1;

        let final_fg_window = GetForegroundWindow();
        let activation_success = final_fg_window == hwnd;

        result.success = activation_success;
        result.method_used = format!("win32_{}", win32_method);
        result.elapsed_ms = start_time.elapsed().as_millis() as u64;

        if !activation_success {
            result.error = Some("window found but activation failed".to_string());
        }

        if let Some(window) = app_handle.get_webview_window("main") {
            use tauri::Emitter;
            let _ = window.emit("app-activated", ());
        }

        Ok(result)
    }
}

// implement activate_app_window for other platforms
#[tauri::command]
#[cfg(not(target_os = "windows"))]
pub fn activate_app_window(app_handle: tauri::AppHandle) -> Result<ActivationResult, String> {
    let mut result = ActivationResult::default();
    result.success = true;
    result.method_used = "non_windows_platform".to_string();

    if let Some(window) = app_handle.get_webview_window("main") {
        use tauri::Emitter;
        let _ = window.emit("app-activated", ());
    }

    Ok(result)
}
