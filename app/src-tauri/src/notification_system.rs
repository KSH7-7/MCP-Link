//! 크로스 플랫폼 알림 시스템 구현
//! Windows에서는 winrt-notification, 다른 플랫폼에서는 notify-rust 사용

use tauri::{AppHandle, Manager, Runtime, Emitter};
use std::error::Error;

// 파일 생성을 위한 std::io::Write는 사용하는 위치에서 직접 import

// 알림 클릭 시 사용할 URI 스킴
const URI_SCHEME: &str = "mcplink://notification";

/// 키워드 상태를 저장하는 구조체
pub struct KeywordState {
    keyword: std::sync::Mutex<Option<String>>,
}

impl KeywordState {
    pub fn new() -> Self {
        Self {
            keyword: std::sync::Mutex::new(None),
        }
    }

    /// 키워드를 설정한다
    pub fn set_keyword(&self, keyword: String) {
        let mut guard = self.keyword.lock().unwrap();
        *guard = Some(keyword);
    }

    /// 키워드를 가져오고 상태를 비운다
    pub fn take_keyword(&self) -> Option<String> {
        let mut guard = self.keyword.lock().unwrap();
        guard.take()
    }

    /// 키워드가 있는지 확인한다
    pub fn has_keyword(&self) -> bool {
        let guard = self.keyword.lock().unwrap();
        guard.is_some()
    }
}

/// Windows에서 알림 표시
#[cfg(target_os = "windows")]
pub fn show_windows_notification(
    title: &str, 
    body: &str, 
    keyword: Option<&str>  // 이제 키워드 사용
) -> Result<(), Box<dyn Error>> {
    use winrt_notification::{Duration, Sound, Toast};
    
    // 디버그 로그 파일 경로
    let log_path = std::env::temp_dir().join("mcplink_notification.log");
    
    // 로그 파일에 기록
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path) {
        use std::io::Write;
        let _ = writeln!(file, "[{}] Showing Windows notification: title='{}', body='{}', keyword={:?}", 
            chrono::Local::now().format("%H:%M:%S"),
            title, body, keyword);
    }
    
    // URI 스킴 생성
    let uri = if let Some(kw) = keyword {
        format!("{}?keyword={}", URI_SCHEME, kw)
    } else {
        URI_SCHEME.to_string()
    };
    
    // 로그 파일에 URI 기록
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path) {
        use std::io::Write;
        let _ = writeln!(file, "[{}] Using URI: {}", 
            chrono::Local::now().format("%H:%M:%S"), uri);
    }
    
    // Windows 알림에 프로토콜 핸들러 추가 - 현재 버전에서는 protocol 처리가 제한적
    // 대안으로 별도의 래핑 실행파일 사용
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path) {
        use std::io::Write;
        let _ = writeln!(file, "[{}] Setting up Windows notification with handler", 
            chrono::Local::now().format("%H:%M:%S"));
    }
    
    // 알림 클릭 핸들러 기록 (Windows에서는 현재 자동 실행이 지원되지 않음)
    // 키워드가 있으면 세션 스토리지에 저장 (앱이 활성화되면 사용)
    if let Some(kw) = keyword {
        // 세션 임시 파일에 키워드 저장
        let keyword_path = std::env::temp_dir().join("mcplink_last_keyword.txt");
        if let Ok(mut file) = std::fs::File::create(&keyword_path) {
            use std::io::Write;
            let _ = write!(file, "{}", kw);
        }
        
        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] 키워드 저장됨: {}", 
                chrono::Local::now().format("%H:%M:%S"), kw);
        }
        
        // URI 스킴 생성 (나중에 사용)
        let uri = format!("{}?keyword={}", URI_SCHEME, kw);
        
        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] URI 스킴: {}", 
                chrono::Local::now().format("%H:%M:%S"), uri);
        }
    }
    
    // 알림 생성
    let toast = Toast::new(Toast::POWERSHELL_APP_ID)
        .title(title)
        .text1(body)
        .sound(Some(Sound::Default))
        .duration(Duration::Short);
    
    // 알림 표시
    let result = toast.show();
    
    // 결과 로깅
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path) {
        use std::io::Write;
        match &result {
            Ok(_) => {
                let _ = writeln!(file, "[{}] Successfully showed Windows notification", 
                    chrono::Local::now().format("%H:%M:%S"));
            },
            Err(e) => {
                let _ = writeln!(file, "[{}] Failed to show Windows notification: {}", 
                    chrono::Local::now().format("%H:%M:%S"), e);
            }
        }
    }
    
    // winrt-notification::Error를 Box<dyn Error>로 변환
    match result {
        Ok(()) => Ok(()),
        Err(e) => Err(Box::new(e))
    }
}

/// macOS에서 알림 표시
#[cfg(target_os = "macos")]
pub fn show_macos_notification(
    title: &str, 
    body: &str, 
    keyword: Option<&str>
) -> Result<(), Box<dyn Error>> {
    use notify_rust::{Notification, Hint};
    
    // macOS 알림 표시
    let mut notification = Notification::new();
    
    // 알림 기본 설정
    notification.summary(title)
                .body(body)
                .icon("icons/icon.png")
                .sound_name("default")
                .hint(Hint::CustomInt("sender-pid".to_owned(), std::process::id() as i32));
    
    // URI 스킴 추가
    if let Some(kw) = keyword {
        let uri = format!("{}?keyword={}", URI_SCHEME, kw);
        notification.action("default", "Open");
    }
    
    notification.show()?;
    
    Ok(())
}

/// Linux에서 알림 표시
#[cfg(target_os = "linux")]
pub fn show_linux_notification(
    title: &str, 
    body: &str, 
    keyword: Option<&str>
) -> Result<(), Box<dyn Error>> {
    use notify_rust::Notification;
    
    // Linux 알림 표시
    let mut notification = Notification::new();
    
    // 알림 기본 설정
    notification.summary(title)
                .body(body)
                .icon("icons/icon.png");
    
    // URI 스킴 추가
    if let Some(kw) = keyword {
        let uri = format!("{}?keyword={}", URI_SCHEME, kw);
        notification.action("default", "Open");
    }
    
    notification.show()?;
    
    Ok(())
}

/// 크로스 플랫폼 알림 표시 함수
#[tauri::command]
pub fn show_notification<R: Runtime>(
    app: AppHandle<R>,
    title: String, 
    body: String, 
    keyword: Option<String>
) -> Result<(), String> {
    // 디버그 로그 파일 경로
    let log_path = std::env::temp_dir().join("mcplink_notification.log");
    
    // 로그 파일에 시작 기록
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path) {
        use std::io::Write;
        let _ = writeln!(file, "[{}] Show notification called: title='{}', body='{}', keyword={:?}", 
            chrono::Local::now().format("%H:%M:%S"),
            title, body, keyword);
    }
    
    // 키워드가 있으면 상태에 저장
    if let Some(ref kw) = keyword {
        app.state::<KeywordState>().set_keyword(kw.clone());
        
        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] Keyword saved to state: {}", 
                chrono::Local::now().format("%H:%M:%S"), kw);
        }
        
        // 한 가지 대안: 키워드를 세션 스토리지에 즉시 저장
        // 이는 알림을 수동으로 처리하는 방식으로, 알림 클릭을 통해 앱이 활성화될 때
        // 이벤트 사용자 정의 항목 빌드 후 메인 페이지에서 처리할 수 있음
        let _ = app.emit("store-keyword", kw.clone());
        
        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "[{}] Emitted store-keyword event: {}", 
                chrono::Local::now().format("%H:%M:%S"), kw);
        }
    }
    
    // 플랫폼별 알림 표시
    let result = match () {
        #[cfg(target_os = "windows")]
        () => show_windows_notification(&title, &body, keyword.as_deref())
            .map_err(|e| format!("Windows notification error: {}", e)),
        
        #[cfg(target_os = "macos")]
        () => show_macos_notification(&title, &body, keyword.as_deref())
            .map_err(|e| format!("macOS notification error: {}", e)),
        
        #[cfg(target_os = "linux")]
        () => show_linux_notification(&title, &body, keyword.as_deref())
            .map_err(|e| format!("Linux notification error: {}", e)),
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        () => Err("Unsupported platform".to_string()),
    };
    
    // 결과 로깅
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path) {
        use std::io::Write;
        match &result {
            Ok(_) => {
                let _ = writeln!(file, "[{}] Successfully showed notification", 
                    chrono::Local::now().format("%H:%M:%S"));
            },
            Err(e) => {
                let _ = writeln!(file, "[{}] Failed to show notification: {}", 
                    chrono::Local::now().format("%H:%M:%S"), e);
            }
        }
    }
    
    result
}


/// 알림 시스템 초기화
pub fn init_notification_system<R: Runtime>(app: &mut tauri::App<R>) -> Result<(), Box<dyn Error>> {
    // KeywordState 등록
    app.manage(KeywordState::new());
    
    Ok(())
}