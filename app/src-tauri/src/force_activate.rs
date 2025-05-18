//! Windows 앱 활성화 및 포커스 강제 지정 모듈
//! 
//! 이 모듈은 Windows에서 알림을 클릭했을 때 
//! 앱을 강제로 활성화하고 전면으로 가져오는 기능을 제공합니다.

#[cfg(target_os = "windows")]
use std::ptr::null_mut;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, SetForegroundWindow, ShowWindow, 
    SW_RESTORE, SW_SHOW, 
    SetWindowPos, HWND_TOPMOST, HWND_NOTOPMOST, 
    SWP_NOMOVE, SWP_NOSIZE, GetWindowThreadProcessId,
    GetForegroundWindow, IsIconic,
};
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

/// 애플리케이션 클래스 이름 (Tauri에서 기본적으로 생성하는 클래스)
static CLASS_NAME: &str = "tauri-runtime-wry";

// Helper function for wide string conversion
#[cfg(target_os = "windows")]
fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
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

        // 윈도우 찾기 시도 (클래스 이름과 윈도우 이름으로)
        let class_wide = to_wide_string(CLASS_NAME);
        let window_wide = to_wide_string(WINDOW_NAME);
        
        let hwnd = FindWindowW(
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
            let _ = writeln!(file, "[{}] FindWindowW 결과: {}", 
                chrono::Local::now().format("%H:%M:%S"),
                if hwnd.0 == 0 { "실패 (핸들 없음)" } else { "성공 (핸들 있음)" });
        }

        // 창을 찾지 못했다면 다른 방법으로 재시도 (클래스 이름만 사용)
        let hwnd = if hwnd.0 == 0 {
            FindWindowW(
                PCWSTR::from_raw(class_wide.as_ptr()),
                PCWSTR(null_mut()),
            )
        } else {
            hwnd
        };
        
        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] 두 번째 FindWindowW 결과: {}", 
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

        // 전면으로 가져오기 (포커스 설정)
        let foreground_result = SetForegroundWindow(hwnd);
        
        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] SetForegroundWindow 결과: {}", 
                chrono::Local::now().format("%H:%M:%S"),
                if foreground_result.as_bool() { "성공" } else { "실패" });
        }

        // 추가적인 창 활성화 시도 (SetForegroundWindow가 실패할 경우)
        if !foreground_result.as_bool() {
            // 현재 포그라운드 윈돀우의 스레드 ID 가져오기
            let foreground_hwnd = GetForegroundWindow();
            let _foreground_thread_id: u32 = 0;
            GetWindowThreadProcessId(foreground_hwnd, Some(std::ptr::null_mut()));
            
            // 대상 윈돀우의 스레드 ID 가져오기
            let _target_thread_id: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(std::ptr::null_mut()));
            
            // 다시 SetForegroundWindow 시도
            let retry_result = SetForegroundWindow(hwnd);
            
            // 로그 파일에 기록
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path) {
                use std::io::Write;
                let _ = writeln!(file, "[{}] 두 번째 SetForegroundWindow 결과: {}", 
                    chrono::Local::now().format("%H:%M:%S"),
                    if retry_result.as_bool() { "성공" } else { "실패" });
            }
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