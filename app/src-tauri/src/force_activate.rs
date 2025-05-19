//! Windows 앱 활성화 및 포커스 강제 지정 모듈
//! 
//! 이 모듈은 Windows에서 알림을 클릭했을 때 
//! 앱을 강제로 활성화하고 전면으로 가져오는 기능을 제공합니다.

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
use windows::core::{PCWSTR, PWSTR};

#[cfg(target_os = "windows")]
// use std::sync::OnceLock; // 사용하지 않음
#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;

/// 애플리케이션 윈도우 이름 (FindWindowW에서 사용)
static WINDOW_NAME: &str = "MCP Link";

/// 애플리케이션 클래스 이름 (로그에서 확인된 실제 Tauri 창 클래스 이름)
static CLASS_NAME: &str = "Tauri Window";

/// 가능한 대체 창 이름 목록 - Tauri 앱에서 사용할 수 있는 다양한 이름 포함
static ALT_WINDOW_NAMES: [&str; 12] = [
    "MCP Link", 
    "MCPLink",
    "MCP-Link",
    "MCPLINK",
    "tauri app",
    "Tauri App", 
    "Tauri",
    "TAURI",
    "Tauri Application",
    "MCP",  // 더 짧은 이름 추가
    "Link",  // 더 짧은 이름 추가
    ""  // 빈 문자열은 NULL과 같이 모든 창 이름 허용
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
}

/// EnumWindows 콜백 함수
#[cfg(target_os = "windows")]
extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        // 상태 데이터에 대한 포인터를 가져온다
        let state = lparam.0 as *mut EnumWindowsState;
        
        // 창이 보이는지 확인
        if !IsWindowVisible(hwnd).as_bool() {
            return true.into(); // 다음 창으로 계속
        }
        
        // 창 제목 가져오기
        let mut title_buf = [0u16; 512];
        // GetWindowTextW로 창 제목 가져오기 (안전한 방식으로 수정됨)
        let title_len = GetWindowTextW(hwnd, title_buf.as_mut_slice());
        let title = if title_len > 0 {
            let title_slice = &title_buf[0..title_len as usize];
            String::from_utf16_lossy(title_slice)
        } else {
            String::new()
        };
        
        // 클래스 이름 가져오기
        let mut class_buf = [0u16; 256];
        // GetClassNameW로 클래스 이름 가져오기 (안전한 방식으로 수정됨)
        let class_len = GetClassNameW(hwnd, class_buf.as_mut_slice());
        let class_name = if class_len > 0 {
            let class_slice = &class_buf[0..class_len as usize];
            String::from_utf16_lossy(class_slice)
        } else {
            String::new()
        };
        
        // 이 창이 MCP Link와 관련이 있는지 확인
        let is_target = 
            // 정확한 창 이름과 클래스 일치 확인 (최우선)
            (title == "MCP Link" && class_name == "Tauri Window") ||
            // 창 이름 확인
            title.to_lowercase().contains("mcp") ||
            title.to_lowercase().contains("link") ||
            // 클래스 이름에 tauri가 포함되어 있는지 확인
            class_name.to_lowercase().contains("tauri") ||
            class_name.to_lowercase().contains("window");
        
        // 로그 파일에 기록 (특정 조건을 충족하는 창만)
        if !title.is_empty() && is_target {
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&(*state).log_path) {
                use std::io::Write;
                let _ = writeln!(file, "[{}] 발견된 창: hwnd={:?}, title='{}', class='{}'", 
                    chrono::Local::now().format("%H:%M:%S"),
                    hwnd.0,
                    title,
                    class_name);
            }
            
            // 이 창이 타겟 조건과 일치하면 저장
            if is_target {
                (*state).found_hwnd = hwnd;
                return false.into(); // 열거 중지
            }
        }
        
        true.into() // 다음 창으로 계속
    }
}

/// 모든 창을 열거하여 적합한 창 찾기
#[cfg(target_os = "windows")]
fn find_app_window(log_path: &std::path::Path) -> Option<HWND> {
    unsafe {
        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] 모든 창 열거 시작", 
                chrono::Local::now().format("%H:%M:%S"));
        }
        
        // 콜백 데이터 구조체 초기화
        let mut state = EnumWindowsState {
            log_path: log_path.to_path_buf(),
            found_hwnd: HWND(0),
        };
        
        // 모든 창 열거
        EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut state as *mut _ as isize)
        );
        
        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] 모든 창 열거 완료, 결과: {}", 
                chrono::Local::now().format("%H:%M:%S"),
                if state.found_hwnd.0 == 0 { "찾지 못함" } else { "창 찾음" });
        }
        
        // 찾은 창 반환
        if state.found_hwnd.0 != 0 {
            Some(state.found_hwnd)
        } else {
            None
        }
    }
}

/// Windows에서 앱을 찾고 강제로 활성화하는 함수
#[cfg(target_os = "windows")]
pub fn force_app_to_foreground() -> Result<(), String> {
    unsafe {
        // 디버그 로그 파일 경로
        let log_path = std::env::temp_dir().join("mcplink_activation.log");

        // 로그 파일에 시작 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "=== [{}] 앱 강제 활성화 시도 ===", 
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
        }

        // 기존 방식으로 클래스 이름과 창 이름으로 시도
        let class_wide = to_wide_string(CLASS_NAME);
        let mut hwnd = HWND(0);
        
        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] 창 이름 목록으로 검색 시작", 
                chrono::Local::now().format("%H:%M:%S"));
        }
        
        // 정확한 클래스 이름과 창 이름으로 먼저 시도 (로그에서 확인된 정확한 정보)
        let exact_window_wide = to_wide_string("MCP Link");
        let exact_class_wide = to_wide_string("Tauri Window");

        // 정확한 클래스 이름과 창 이름으로 검색
        let current_hwnd = FindWindowW(
            PCWSTR::from_raw(exact_class_wide.as_ptr()),
            PCWSTR::from_raw(exact_window_wide.as_ptr()),
        );

        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] 정확한 조합 시도: FindWindowW(class='Tauri Window', name='MCP Link') 결과: {}", 
                chrono::Local::now().format("%H:%M:%S"),
                if current_hwnd.0 == 0 { "실패" } else { "성공" });
        }

        // 정확한 조합에 성공했다면 바로 사용
        if current_hwnd.0 != 0 {
            hwnd = current_hwnd;
        } else {
            // 각 가능한 창 이름 조합 시도 (fallback)
            for window_name in ALT_WINDOW_NAMES.iter() {
                let window_wide = to_wide_string(window_name);
                
                // 클래스 이름과 창 이름으로 검색
                let current_hwnd = FindWindowW(
                    PCWSTR::from_raw(class_wide.as_ptr()),
                    PCWSTR::from_raw(window_wide.as_ptr()),
                );
            
                // 로그 파일에 기록
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&log_path) {
                    use std::io::Write;
                    let _ = writeln!(file, "[{}] FindWindowW(class='{}', name='{}') 결과: {}", 
                        chrono::Local::now().format("%H:%M:%S"),
                        CLASS_NAME,
                        window_name,
                        if current_hwnd.0 == 0 { "실패" } else { "성공" });
                }
                
                // 창을 찾았으면 루프 종료
                if current_hwnd.0 != 0 {
                    hwnd = current_hwnd;
                    break;
                }
            }
        }
        
        // 클래스 이름만으로도 시도
        if hwnd.0 == 0 {
            // 로그 파일에 기록
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path) {
                use std::io::Write;
                let _ = writeln!(file, "[{}] 클래스 이름만으로 시도 (class='{}')", 
                    chrono::Local::now().format("%H:%M:%S"),
                    CLASS_NAME);
            }

            let current_hwnd = FindWindowW(
                PCWSTR::from_raw(class_wide.as_ptr()),
                PCWSTR(null_mut()),
            );
            
            // 로그 파일에 기록
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path) {
                use std::io::Write;
                let _ = writeln!(file, "[{}] FindWindowW(class='{}', name=NULL) 결과: {}", 
                    chrono::Local::now().format("%H:%M:%S"),
                    CLASS_NAME,
                    if current_hwnd.0 == 0 { "실패" } else { "성공" });
            }
            
            if current_hwnd.0 != 0 {
                hwnd = current_hwnd;
            }
        }
        
        // 일반적인 Tauri 앱 클래스 이름으로 시도
        if hwnd.0 == 0 {
            // 로그 파일에 기록
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path) {
                use std::io::Write;
                let _ = writeln!(file, "[{}] 대체 클래스 이름으로 시도", 
                    chrono::Local::now().format("%H:%M:%S"));
            }

            let alt_class_names = ["Tauri Window", "TAURI WINDOW", "tauri window", "Tauri", "tauri", "Wry", "wry", "MCP", "app", ""];
            
            for alt_class in alt_class_names.iter() {
                if alt_class.is_empty() {
                    continue;
                }
                
                let alt_class_wide = to_wide_string(alt_class);
                let current_hwnd = FindWindowW(
                    PCWSTR::from_raw(alt_class_wide.as_ptr()),
                    PCWSTR(null_mut()),
                );
                
                // 로그 파일에 기록
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&log_path) {
                    use std::io::Write;
                    let _ = writeln!(file, "[{}] FindWindowW(class='{}', name=NULL) 결과: {}", 
                        chrono::Local::now().format("%H:%M:%S"),
                        alt_class,
                        if current_hwnd.0 == 0 { "실패" } else { "성공" });
                }
                
                if current_hwnd.0 != 0 {
                    hwnd = current_hwnd;
                    break;
                }
            }
        }
        
        // 모든 방법이 실패하면 EnumWindows로 모든 창을 검사
        if hwnd.0 == 0 {
            // 로그 파일에 기록
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path) {
                use std::io::Write;
                let _ = writeln!(file, "[{}] 기존 방법 모두 실패. EnumWindows 시도...", 
                    chrono::Local::now().format("%H:%M:%S"));
            }
            
            // EnumWindows를 사용하여 모든 창을 검사
            if let Some(found_hwnd) = find_app_window(&log_path) {
                hwnd = found_hwnd;
                
                // 로그 파일에 기록
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&log_path) {
                    use std::io::Write;
                    let _ = writeln!(file, "[{}] EnumWindows로 창을 찾았습니다: {:?}", 
                        chrono::Local::now().format("%H:%M:%S"), hwnd.0);
                }
            } else {
                // 로그 파일에 기록
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&log_path) {
                    use std::io::Write;
                    let _ = writeln!(file, "[{}] EnumWindows로도 창을 찾지 못했습니다", 
                        chrono::Local::now().format("%H:%M:%S"));
                }
            }
        }
        
        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] 최종 창 검색 결과: {}", 
                chrono::Local::now().format("%H:%M:%S"),
                if hwnd.0 == 0 { "실패 (핸들 없음)" } else { "성공 (핸들 있음)" });
        }

        if hwnd.0 == 0 {
            // 로그 파일에 기록
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path) {
                use std::io::Write;
                let _ = writeln!(file, "[{}] ERROR: 애플리케이션 창을 찾을 수 없음", 
                    chrono::Local::now().format("%H:%M:%S"));
            }
            return Err("Application window not found".to_string());
        }

        // 창이 최소화되어 있는지 확인
        let is_minimized = IsIconic(hwnd).as_bool();
        
        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] 창 상태: {}", 
                chrono::Local::now().format("%H:%M:%S"),
                if is_minimized { "최소화됨" } else { "최소화되지 않음" });
        }

        // 최소화된 창 복원
        if is_minimized {
            ShowWindow(hwnd, SW_RESTORE);
            
            // 로그 파일에 기록
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path) {
                use std::io::Write;
                let _ = writeln!(file, "[{}] ShowWindow(SW_RESTORE) 호출됨", 
                    chrono::Local::now().format("%H:%M:%S"));
            }
        }

        // 창 표시
        ShowWindow(hwnd, SW_SHOW);
        
        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] ShowWindow(SW_SHOW) 호출됨", 
                chrono::Local::now().format("%H:%M:%S"));
        }

        // 창을 최상위로 설정
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE,
        );
        
        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] SetWindowPos(HWND_TOPMOST) 호출됨", 
                chrono::Local::now().format("%H:%M:%S"));
        }

        // 일반 배치로 복원 (다른 창이 위에 올 수 있도록)
        SetWindowPos(
            hwnd,
            HWND_NOTOPMOST,
            0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE,
        );
        
        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] SetWindowPos(HWND_NOTOPMOST) 호출됨", 
                chrono::Local::now().format("%H:%M:%S"));
        }

        // 강화된 창 활성화 단계적 접근법 적용
        
        // 1. AllowSetForegroundWindow 호출 - 이는 다른 프로세스가 SetForegroundWindow를 호출할 수 있도록 함
        let _ = AllowSetForegroundWindow(ASFW_ANY);
        
        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] AllowSetForegroundWindow(ASFW_ANY) 호출됨", 
                chrono::Local::now().format("%H:%M:%S"));
        }

        // 2. 기본 SetForegroundWindow 시도
        let foreground_result = SetForegroundWindow(hwnd);
        
        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] 기본 SetForegroundWindow 결과: {}", 
                chrono::Local::now().format("%H:%M:%S"),
                if foreground_result.as_bool() { "성공" } else { "실패" });
        }

        // 3. AttachThreadInput 방식 시도
        // 현재 포그라운드 윈돀우의 스레드 ID 가져오기
        let foreground_hwnd = GetForegroundWindow();
        let mut foreground_thread_id: u32 = 0;
        GetWindowThreadProcessId(foreground_hwnd, Some(&mut foreground_thread_id));
        
        // 대상 윈돀우의 스레드 ID 가져오기
        let mut target_thread_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut target_thread_id));
        
        // 현재 쓰레드 ID 가져오기
        let current_thread_id = GetCurrentThreadId();
        
        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] 스레드 ID: current={}, foreground={}, target={}", 
                chrono::Local::now().format("%H:%M:%S"),
                current_thread_id,
                foreground_thread_id,
                target_thread_id);
        }
        
        // 첫 번째 방법이 실패하고 현재 포그라운드 윈도우가 다른 윈도우라면
        if !foreground_result.as_bool() && foreground_hwnd != hwnd {
            // 로그 파일에 기록
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path) {
                use std::io::Write;
                let _ = writeln!(file, "[{}] AttachThreadInput 방식 시도", 
                    chrono::Local::now().format("%H:%M:%S"));
            }
            
            // 포그라운드 윈도우의 스레드에 연결
            if foreground_thread_id != 0 && current_thread_id != foreground_thread_id {
                let attach_result = AttachThreadInput(current_thread_id, foreground_thread_id, true);
                
                // 로그 파일에 기록
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&log_path) {
                    use std::io::Write;
                    let _ = writeln!(file, "[{}] AttachThreadInput(current, foreground, true) 결과: {}", 
                        chrono::Local::now().format("%H:%M:%S"),
                        if attach_result.as_bool() { "성공" } else { "실패" });
                }
                
                // 대상 윈도우를 일반 모드로 표시
                ShowWindow(hwnd, SW_SHOWNORMAL);
                
                // BringWindowToTop 호출
                let bring_result = BringWindowToTop(hwnd);
                
                // 로그 파일에 기록
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&log_path) {
                    use std::io::Write;
                    let _ = writeln!(file, "[{}] BringWindowToTop 결과: {}", 
                        chrono::Local::now().format("%H:%M:%S"),
                        if bring_result.as_bool() { "성공" } else { "실패" });
                }
                
                // 포그라운드 윈도우로 설정
                let set_result = SetForegroundWindow(hwnd);
                
                // 로그 파일에 기록
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&log_path) {
                    use std::io::Write;
                    let _ = writeln!(file, "[{}] AttachThreadInput 후 SetForegroundWindow 결과: {}", 
                        chrono::Local::now().format("%H:%M:%S"),
                        if set_result.as_bool() { "성공" } else { "실패" });
                }
                
                // 스레드 연결 해제
                let detach_result = AttachThreadInput(current_thread_id, foreground_thread_id, false);
                
                // 로그 파일에 기록
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&log_path) {
                    use std::io::Write;
                    let _ = writeln!(file, "[{}] AttachThreadInput(current, foreground, false) 결과: {}", 
                        chrono::Local::now().format("%H:%M:%S"),
                        if detach_result.as_bool() { "성공" } else { "실패" });
                }
                
                // 대상 프로세스에도 같은 방식 적용
                if target_thread_id != 0 && current_thread_id != target_thread_id {
                    let attach_result_2 = AttachThreadInput(current_thread_id, target_thread_id, true);
                    
                    // 로그 파일에 기록
                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(true)
                        .open(&log_path) {
                        use std::io::Write;
                        let _ = writeln!(file, "[{}] AttachThreadInput(current, target, true) 결과: {}", 
                            chrono::Local::now().format("%H:%M:%S"),
                            if attach_result_2.as_bool() { "성공" } else { "실패" });
                    }
                    
                    // 다시 활성화 시도
                    let set_result_2 = SetForegroundWindow(hwnd);
                    
                    // 로그 파일에 기록
                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(true)
                        .open(&log_path) {
                        use std::io::Write;
                        let _ = writeln!(file, "[{}] 대상 AttachThreadInput 후 SetForegroundWindow 결과: {}", 
                            chrono::Local::now().format("%H:%M:%S"),
                            if set_result_2.as_bool() { "성공" } else { "실패" });
                    }
                    
                    // 스레드 연결 해제
                    let detach_result_2 = AttachThreadInput(current_thread_id, target_thread_id, false);
                    
                    // 로그 파일에 기록
                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(true)
                        .open(&log_path) {
                        use std::io::Write;
                        let _ = writeln!(file, "[{}] AttachThreadInput(current, target, false) 결과: {}", 
                            chrono::Local::now().format("%H:%M:%S"),
                            if detach_result_2.as_bool() { "성공" } else { "실패" });
                    }
                }
            }
        }
        
        // 4. 최종 대체 시도 - 모든 방법이 실패한 경우
        if !foreground_result.as_bool() {
            // 로그 파일에 기록
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path) {
                use std::io::Write;
                let _ = writeln!(file, "[{}] 최종 대체 활성화 방법 시도", 
                    chrono::Local::now().format("%H:%M:%S"));
            }
            
            // 윈도우 보이기와 최상위로 가져오기 결합
            ShowWindow(hwnd, SW_SHOWNORMAL);
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
            );
            
            // 짧은 대기 후 다시 SetForegroundWindow 시도
            std::thread::sleep(std::time::Duration::from_millis(50));
            BringWindowToTop(hwnd);
            let final_result = SetForegroundWindow(hwnd);
            
            // 로그 파일에 기록
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path) {
                use std::io::Write;
                let _ = writeln!(file, "[{}] 최종 SetForegroundWindow 결과: {}", 
                    chrono::Local::now().format("%H:%M:%S"),
                    if final_result.as_bool() { "성공" } else { "실패" });
            }
            
            // 일반 윈도우 순서로 복원
            SetWindowPos(
                hwnd,
                HWND_NOTOPMOST,
                0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
            );
        }

        // 완료 로그
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] 앱 활성화 프로세스 완료", 
                chrono::Local::now().format("%H:%M:%S"));
        }
    }

    Ok(())
}

/// 비 Windows 환경에서는 빈 구현만 제공
#[cfg(not(target_os = "windows"))]
pub fn force_app_to_foreground() -> Result<(), String> {
    // 비 Windows 플랫폼에서는 아무 것도 하지 않음
    Ok(())
}