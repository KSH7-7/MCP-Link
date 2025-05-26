//! Windows 앱 활성화 및 포커스 강제 지정 모듈

use tauri::Manager;

#[cfg(target_os = "windows")]
use std::ptr::null_mut;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HWND, LPARAM, BOOL};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, SetForegroundWindow, ShowWindow, 
    SW_RESTORE, SW_SHOW, SW_SHOWNORMAL,
    SetWindowPos, HWND_TOPMOST, HWND_NOTOPMOST, 
    SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, 
    GetWindowThreadProcessId, BringWindowToTop,
    GetForegroundWindow, IsIconic, EnumWindows,
    GetWindowTextW, GetClassNameW, IsWindowVisible,
    AllowSetForegroundWindow, ASFW_ANY,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
#[cfg(target_os = "windows")]
use windows::core::{PCWSTR};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    keybd_event, KEYBD_EVENT_FLAGS, VIRTUAL_KEY, VK_MENU
};

#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;

/// 애플리케이션 윈도우 이름
static WINDOW_NAME: &str = "MCP Link";

/// 애플리케이션 클래스 이름
static CLASS_NAME: &str = "Tauri Window";

/// 가능한 대체 창 이름 목록
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
    "Link"
];

// Helper function for wide string conversion
#[cfg(target_os = "windows")]
fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// EnumWindows 콜백 함수에서 사용할 데이터 구조체
#[cfg(target_os = "windows")]
struct EnumWindowsState {
    log_path: std::path::PathBuf,
    found_hwnd: HWND,
    target_pid: Option<u32>,
}

/// EnumWindows 콜백 함수
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
                
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&(*state).log_path) {
                    use std::io::Write;
                    let _ = writeln!(file, "[{}] ✓ 프로세스 ID 일치 창 발견: hwnd={:?}, pid={}, title='{}', class='{}'", 
                        chrono::Local::now().format("%H:%M:%S"),
                        hwnd.0,
                        process_id,
                        title,
                        class_name);
                }
                
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
        
        let is_backup_target = 
            (title == "MCP Link" && class_name == "Tauri Window") ||
            (title == "MCP Link" && class_lower.contains("tauri")) ||
            (title_lower.contains("mcp") && class_lower.contains("tauri")) ||
            (title_lower.contains("mcp") && title_lower.contains("link")) ||
            (class_lower.contains("tauri")) ||
            (class_lower.contains("webview") && title_lower.contains("mcp"));
        
        if (*state).found_hwnd.0 == 0 && is_backup_target {
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&(*state).log_path) {
                use std::io::Write;
                let _ = writeln!(file, "[{}] 백업 방식으로 창 발견: hwnd={:?}, title='{}', class='{}', pid={}", 
                    chrono::Local::now().format("%H:%M:%S"),
                    hwnd.0,
                    title,
                    class_name,
                    process_id);
            }
            
            (*state).found_hwnd = hwnd;
            return false.into();
        }
        
        true.into()
    }
}

/// 모든 창을 열거하여 적합한 창 찾기
#[cfg(target_os = "windows")]
fn find_app_window(log_path: &std::path::Path) -> Option<HWND> {
    unsafe {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] === 모든 창 열거 시작 ===", 
                chrono::Local::now().format("%H:%M:%S"));
        }
        
        let current_pid = std::process::id();
        
        let mut state = EnumWindowsState {
            log_path: log_path.to_path_buf(),
            found_hwnd: HWND(0),
            target_pid: Some(current_pid),
        };
        
        EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut state as *mut _ as isize)
        );
        
        if state.found_hwnd.0 != 0 {
            Some(state.found_hwnd)
        } else {
            None
        }
    }
}

/// 활성화 결과를 나타내는 구조체
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

/// Windows에서 앱을 찾고 강제로 활성화하는 함수 - 강화된 버전
#[cfg(target_os = "windows")]
pub fn force_app_to_foreground() -> Result<(), String> {
    unsafe {
        let log_path = std::env::temp_dir().join("mcplink_activation.log");
        
        // 로그 시작
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "\n[{}] === 앱 활성화 시작 (강화된 버전) ===", 
                chrono::Local::now().format("%H:%M:%S"));
        }
        
        // SetForegroundWindow 권한 허용
        AllowSetForegroundWindow(ASFW_ANY);
        
        // 1. 먼저 클래스 이름으로 찾기
        let class_name_wide = to_wide_string("Tauri Window");
        let mut hwnd = FindWindowW(
            PCWSTR::from_raw(class_name_wide.as_ptr()),
            PCWSTR::from_raw(null_mut()),
        );
        
        // 2. 클래스 이름으로 못 찾으면 창 이름으로 찾기
        if hwnd.0 == 0 {
            let window_name_wide = to_wide_string("MCP Link");
            hwnd = FindWindowW(
                PCWSTR::from_raw(null_mut()),
                PCWSTR::from_raw(window_name_wide.as_ptr()),
            );
        }
        
        // 3. 여전히 못 찾으면 EnumWindows 사용
        if hwnd.0 == 0 {
            if let Some(found) = find_app_window(&log_path) {
                hwnd = found;
            }
        }
        
        if hwnd.0 == 0 {
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path) {
                use std::io::Write;
                let _ = writeln!(file, "[{}] 오류: 앱 창을 찾을 수 없습니다", 
                    chrono::Local::now().format("%H:%M:%S"));
            }
            return Err("앱 창을 찾을 수 없습니다".to_string());
        }
        
        // 로그: 창 찾음
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] 창 찾음: hwnd={:?}", 
                chrono::Local::now().format("%H:%M:%S"), hwnd.0);
        }
        
        // 현재 전경 창 가져오기
        let current_fg = GetForegroundWindow();
        let mut thread_id = 0;
        let mut current_thread_id = 0;
        
        if current_fg.0 != 0 && current_fg != hwnd {
            thread_id = GetWindowThreadProcessId(hwnd, None);
            current_thread_id = GetWindowThreadProcessId(current_fg, None);
            
            // 스레드 연결
            if thread_id != current_thread_id {
                AttachThreadInput(current_thread_id, thread_id, true);
            }
        }
        
        // 최소화된 경우 복원
        if IsIconic(hwnd).as_bool() {
            ShowWindow(hwnd, SW_RESTORE);
            std::thread::sleep(std::time::Duration::from_millis(150));
            
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path) {
                use std::io::Write;
                let _ = writeln!(file, "[{}] 최소화된 창 복원", 
                    chrono::Local::now().format("%H:%M:%S"));
            }
        }
        
        // 1단계: 창 표시
        ShowWindow(hwnd, SW_SHOW);
        ShowWindow(hwnd, SW_SHOWNORMAL);
        
        // 2단계: Alt 키 시뮬레이션 (더 안정적으로)
        keybd_event(VK_MENU.0 as u8, 0x38, KEYBD_EVENT_FLAGS(0), 0);
        std::thread::sleep(std::time::Duration::from_millis(50));
        keybd_event(VK_MENU.0 as u8, 0x38, KEYBD_EVENT_FLAGS(2), 0);
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        // 3단계: 전경 설정 시도
        let result1 = SetForegroundWindow(hwnd);
        
        // 4단계: BringWindowToTop
        BringWindowToTop(hwnd);
        
        // 5단계: 최상위 설정 및 해제
        SetWindowPos(
            hwnd, 
            HWND_TOPMOST, 
            0, 0, 0, 0, 
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW
        );
        
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        SetWindowPos(
            hwnd, 
            HWND_NOTOPMOST, 
            0, 0, 0, 0, 
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW
        );
        
        // 6단계: 한 번 더 SetForegroundWindow
        std::thread::sleep(std::time::Duration::from_millis(50));
        let result2 = SetForegroundWindow(hwnd);
        
        // 7단계: 추가 Alt 키 트릭
        keybd_event(VK_MENU.0 as u8, 0x38, KEYBD_EVENT_FLAGS(0), 0);
        keybd_event(VK_MENU.0 as u8, 0x38, KEYBD_EVENT_FLAGS(2), 0);
        
        // 8단계: 마지막 시도
        BringWindowToTop(hwnd);
        SetForegroundWindow(hwnd);
        
        // 스레드 연결 해제
        if thread_id != 0 && current_thread_id != 0 && thread_id != current_thread_id {
            AttachThreadInput(current_thread_id, thread_id, false);
        }
        
        // 결과 확인
        std::thread::sleep(std::time::Duration::from_millis(100));
        let final_fg = GetForegroundWindow();
        let success = final_fg == hwnd;
        
        // 로그 완료
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] 앱 활성화 완료 - 성공: {}, SetForeground 결과: {:?}, {:?}", 
                chrono::Local::now().format("%H:%M:%S"),
                success,
                result1.as_bool(),
                result2.as_bool());
            let _ = writeln!(file, "[{}] === 앱 활성화 종료 ===\n", 
                chrono::Local::now().format("%H:%M:%S"));
        }
        
        Ok(())
    }
}

/// 앱이 활성화될 때 호출되어 이벤트를 발생시키는 함수
#[cfg(target_os = "windows")]
pub fn emit_app_activated_event<R: tauri::Runtime>(app_handle: &tauri::AppHandle<R>) -> Result<(), String> {
    emit_app_activated_event_with_source(app_handle, false)
}

/// 소스 정보와 함께 앱 활성화 이벤트를 발생시키는 함수
#[cfg(target_os = "windows")]
pub fn emit_app_activated_event_with_source<R: tauri::Runtime>(app_handle: &tauri::AppHandle<R>, from_notification: bool) -> Result<(), String> {
    let log_path = std::env::temp_dir().join("mcplink_activation.log");
    
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path) {
        use std::io::Write;
        let _ = writeln!(file, "[{}] 앱 활성화 이벤트 발생 시도 (fromNotification: {})", 
            chrono::Local::now().format("%H:%M:%S"), from_notification);
    }
    
    use tauri::Emitter;
    if let Some(window) = app_handle.get_webview_window("main") {
        // 페이로드에 알림 여부 포함
        let payload = serde_json::json!({
            "fromNotification": from_notification
        });
        
        if let Err(e) = window.emit("app-activated", payload) {
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path) {
                use std::io::Write;
                let _ = writeln!(file, "[{}] 앱 활성화 이벤트 발생 오류: {}", 
                    chrono::Local::now().format("%H:%M:%S"), e);
            }
            return Err(format!("앱 활성화 이벤트 발생 오류: {}", e));
        }
        
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] 앱 활성화 이벤트 발생 성공 (fromNotification: {})", 
                chrono::Local::now().format("%H:%M:%S"), from_notification);
        }
    } else {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] 앱 활성화 이벤트 발생 실패: 메인 윈도우 찾을 수 없음", 
                chrono::Local::now().format("%H:%M:%S"));
        }
        return Err("앱 활성화 이벤트 발생 실패: 메인 윈도우 찾을 수 없음".to_string());
    }
    
    Ok(())
}

/// 비 Windows 환경을 위한 빈 구현
#[cfg(not(target_os = "windows"))]
pub fn emit_app_activated_event<R: tauri::Runtime>(_app_handle: &tauri::AppHandle<R>) -> Result<(), String> {
    emit_app_activated_event_with_source(_app_handle, false)
}

/// 비 Windows 환경을 위한 소스 정보와 함께 앱 활성화 이벤트 발생
#[cfg(not(target_os = "windows"))]
pub fn emit_app_activated_event_with_source<R: tauri::Runtime>(_app_handle: &tauri::AppHandle<R>, _from_notification: bool) -> Result<(), String> {
    use tauri::Emitter;
    if let Some(window) = _app_handle.get_webview_window("main") {
        let payload = serde_json::json!({
            "fromNotification": _from_notification
        });
        let _ = window.emit("app-activated", payload);
    }
    Ok(())
}

/// 비 Windows 환경에서는 빈 구현만 제공
#[cfg(not(target_os = "windows"))]
pub fn force_app_to_foreground() -> Result<(), String> {
    Ok(())
}

/// 향상된 앱 창 활성화 함수 (Tauri 명령으로 사용)
#[tauri::command]
#[cfg(target_os = "windows")]
pub fn activate_app_window(app_handle: tauri::AppHandle) -> Result<ActivationResult, String> {
    unsafe {
        let start_time = std::time::Instant::now();
        let mut result = ActivationResult::default();
        let log_path = std::env::temp_dir().join("mcplink_activation.log");

        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "=== [{}] activate_app_window 명령 시작 ===", 
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
        }
        
        // 방법 1: Tauri API를 통한 활성화 시도
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
        
        // 방법 2: Win32 API 직접 호출
        AllowSetForegroundWindow(ASFW_ANY);
        
        let mut hwnd = HWND(0);
        let mut win32_method = "unknown";
        
        // 1. 클래스 이름으로 찾기
        let class_name_wide = to_wide_string("Tauri Window");
        hwnd = FindWindowW(
            PCWSTR::from_raw(class_name_wide.as_ptr()),
            PCWSTR::from_raw(null_mut()),
        );
        
        if hwnd.0 != 0 {
            win32_method = "class_name";
        }
        
        // 2. 창 이름으로 찾기
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
        
        // 3. EnumWindows로 찾기
        if hwnd.0 == 0 {
            if let Some(found) = find_app_window(&log_path) {
                hwnd = found;
                win32_method = "enum_windows";
            }
        }
        
        if hwnd.0 == 0 {
            result.success = false;
            result.method_used = "failed_window_not_found".to_string();
            result.elapsed_ms = start_time.elapsed().as_millis() as u64;
            result.error = Some("앱 창을 찾을 수 없습니다".to_string());
            return Ok(result);
        }
        
        // 창 활성화
        if IsIconic(hwnd).as_bool() {
            ShowWindow(hwnd, SW_RESTORE);
            result.attempts += 1;
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        
        ShowWindow(hwnd, SW_SHOW);
        ShowWindow(hwnd, SW_SHOWNORMAL);
        result.attempts += 1;
        
        // Alt 키 시뮬레이션
        keybd_event(VK_MENU.0 as u8, 0, KEYBD_EVENT_FLAGS(0), 0);
        keybd_event(VK_MENU.0 as u8, 0, KEYBD_EVENT_FLAGS(2), 0);
        
        SetForegroundWindow(hwnd);
        BringWindowToTop(hwnd);
        result.attempts += 1;
        
        SetWindowPos(
            hwnd, 
            HWND_TOPMOST, 
            0, 0, 0, 0, 
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW
        );
        
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        SetWindowPos(
            hwnd, 
            HWND_NOTOPMOST, 
            0, 0, 0, 0, 
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW
        );
        
        SetForegroundWindow(hwnd);
        result.attempts += 1;
        
        let final_fg_window = GetForegroundWindow();
        let activation_success = final_fg_window == hwnd;
        
        result.success = activation_success;
        result.method_used = format!("win32_{}", win32_method);
        result.elapsed_ms = start_time.elapsed().as_millis() as u64;
        
        if !activation_success {
            result.error = Some("창은 찾았으나 활성화에 실패했습니다".to_string());
        }
        
        if let Some(window) = app_handle.get_webview_window("main") {
            use tauri::Emitter;
            let _ = window.emit("app-activated", ());
        }
        
        Ok(result)
    }
}

/// 다른 플랫폼용 activate_app_window 구현
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