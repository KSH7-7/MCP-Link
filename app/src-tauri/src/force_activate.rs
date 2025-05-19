//! Windows 앱 활성화 및 포커스 강제 지정 모듈
//! 
//! 이 모듈은 Windows에서 알림을 클릭했을 때 
//! 앱을 강제로 활성화하고 전면으로 가져오는 기능을 제공합니다.

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
use windows::core::PCWSTR;

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
        // Tauri v2.0 호환성: 앱 활성화 개선을 위한 코드 수정
        // 디버그 로그 파일 경로
        let log_path = std::env::temp_dir().join("mcplink_activation.log");

        // 로그 파일에 시작 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "=== [{}] 앱 강제 활성화 시도 시작 ===", 
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
        }

        // *** 중요 수정: Tauri v2.0에서 알려진 버그 처리 ***
        // Tauri v2.0에서 unstable 기능 사용 시 WindowEvent::Focused 이벤트가 제대로 
        // 전달되지 않는 버그가 있음 (GitHub Issue #9755)
        
        // 최대 3번 시도 - Tauri v2.0에서 앱 활성화가 더 어려워진 경우를 대비
        for attempt in 1..=3 {
            // 로그 파일에 기록
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path) {
                use std::io::Write;
                let _ = writeln!(file, "[{}] 앱 활성화 시도 #{} 시작", 
                    chrono::Local::now().format("%H:%M:%S"), attempt);
            }
            
            // AllowSetForegroundWindow 호출 - 모든 프로세스에서 포그라운드 설정 허용
            AllowSetForegroundWindow(ASFW_ANY);
        
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

            // 모든 가능한 방법을 시도하여 앱 창 찾기
            let mut hwnd = HWND(0);
            
            // 1. 새로운 방식: EnumWindows 콜백 사용하여 모든 창 검색
            if let Some(found_hwnd) = find_app_window(&log_path) {
                hwnd = found_hwnd;
                
                // 로그 파일에 기록
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&log_path) {
                    use std::io::Write;
                    let _ = writeln!(file, "[{}] EnumWindows를 통해 창 찾음: {:?}", 
                        chrono::Local::now().format("%H:%M:%S"), hwnd.0);
                }
            }
            
            // 2. 정확한 클래스 이름과 창 이름으로 시도
            if hwnd.0 == 0 {
                let exact_window_wide = to_wide_string("MCP Link");
                let exact_class_wide = to_wide_string("Tauri Window");
                
                hwnd = FindWindowW(
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
                        if hwnd.0 == 0 { "실패" } else { "성공" });
                }
            }
            
            // 3. 다양한 창 이름 조합 시도 (fallback)
            if hwnd.0 == 0 {
                for alt_window_name in ALT_WINDOW_NAMES.iter() {
                    let window_wide = to_wide_string(alt_window_name);
                    let current_hwnd = FindWindowW(
                        PCWSTR::from_raw(std::ptr::null()),  // NULL 클래스 이름 (모든 클래스)
                        PCWSTR::from_raw(window_wide.as_ptr()),
                    );

                    // 로그 파일에 기록
                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(true)
                        .open(&log_path) {
                        use std::io::Write;
                        let _ = writeln!(file, "[{}] 대체 이름 시도: FindWindowW(class=NULL, name='{}') 결과: {}", 
                            chrono::Local::now().format("%H:%M:%S"),
                            alt_window_name,
                            if current_hwnd.0 == 0 { "실패" } else { "성공" });
                    }

                    if current_hwnd.0 != 0 {
                        hwnd = current_hwnd;
                        break;
                    }
                }
            }
            
            // 창을 찾지 못했을 경우 다음 시도로 넘어감
            if hwnd.0 == 0 {
                // 로그 파일에 실패 기록
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&log_path) {
                    use std::io::Write;
                    let _ = writeln!(file, "[{}] 시도 #{}: 앱 창을 찾을 수 없음. 다음 시도로 진행...", 
                        chrono::Local::now().format("%H:%M:%S"), attempt);
                }
                
                // 다음 시도 전에 약간의 대기 시간 추가
                std::thread::sleep(std::time::Duration::from_millis(300));
                continue;
            }

            // *** 중요 수정: 활성화 순서 조정 및 확실한 메시지 전달 ***
            // 아이콘 상태 확인 후 필요하면 복원
            if IsIconic(hwnd).as_bool() {
                ShowWindow(hwnd, SW_RESTORE);
                
                // 로그 파일에 기록
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&log_path) {
                    use std::io::Write;
                    let _ = writeln!(file, "[{}] 최소화된 창 복원 (ShowWindow/SW_RESTORE)", 
                        chrono::Local::now().format("%H:%M:%S"));
                }
                
                // 복원 후 약간의 대기 시간 추가
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            
            // 다양한 방법으로 창 활성화 시도 (순서 중요)
            // 1. 창 표시
            ShowWindow(hwnd, SW_SHOW);
            
            // 2. 일시적으로 최상위로 설정
            SetWindowPos(
                hwnd, 
                HWND_TOPMOST, 
                0, 0, 0, 0, 
                SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW
            );
            
            // 3. 잠시 대기 후 일반 z-order로 복원
            std::thread::sleep(std::time::Duration::from_millis(50));
            SetWindowPos(
                hwnd, 
                HWND_NOTOPMOST, 
                0, 0, 0, 0, 
                SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW
            );
            
            // 4. BringWindowToTop - 이전 포그라운드 창보다 앞으로 가져오기
            BringWindowToTop(hwnd);
            
            // 5. 현재 활성 스레드와 대상 창 스레드 연결 시도
            let fg_window = GetForegroundWindow();
            if fg_window != hwnd {
                let current_thread_id = GetCurrentThreadId();
                
                let mut fg_thread_id: u32 = 0;
                let mut target_thread_id: u32 = 0;
                
                GetWindowThreadProcessId(fg_window, Some(&mut fg_thread_id));
                GetWindowThreadProcessId(hwnd, Some(&mut target_thread_id));
                
                if AttachThreadInput(current_thread_id, fg_thread_id, true).as_bool() {
                    // 필요시 다른 스레드도 연결
                    let mut thread_attached = true;
                    if fg_thread_id != target_thread_id {
                        thread_attached = AttachThreadInput(fg_thread_id, target_thread_id, true).as_bool();
                    }
                    
                    // 스레드 연결 상태에서 SetForegroundWindow 호출
                    SetForegroundWindow(hwnd);
                    
                    // 스레드 연결 해제
                    if fg_thread_id != target_thread_id && thread_attached {
                        AttachThreadInput(fg_thread_id, target_thread_id, false);
                    }
                    AttachThreadInput(current_thread_id, fg_thread_id, false);
                } else {
                    // 직접 SetForegroundWindow 호출
                    SetForegroundWindow(hwnd);
                }
            }
            
            // 6. 마지막으로 정상(Normal) 상태로 표시하고 전경으로 설정
            ShowWindow(hwnd, SW_SHOWNORMAL);
            SetForegroundWindow(hwnd);
            
            // 마무리: 약간의 시간 지연 추가 - 포커스 획득에 필요한 시간 제공
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            // 로그 파일에 성공 기록
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path) {
                use std::io::Write;
                let _ = writeln!(file, "[{}] 시도 #{}: 앱 활성화 과정 완료", 
                    chrono::Local::now().format("%H:%M:%S"), attempt);
            }
            
            // 성공했으므로 전체 루프 종료
            break;
        } // end of for loop
        
        // 최종 성공 로그 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "=== [{}] 앱 강제 활성화 시도 완료 ===", 
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
        }
        
        Ok(())
    }
}

/// 비 Windows 환경에서는 빈 구현만 제공
#[cfg(not(target_os = "windows"))]
pub fn force_app_to_foreground() -> Result<(), String> {
    // 비 Windows 플랫폼에서는 아무 것도 하지 않음
    Ok(())
}