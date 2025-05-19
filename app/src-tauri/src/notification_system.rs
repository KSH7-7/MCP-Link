//! 크로스 플랫폼 알림 시스템 구현
//! Windows에서는 winrt-notification, 다른 플랫폼에서는 notify-rust 사용

use std::error::Error;
use std::io::Write;
use tauri::{AppHandle, Emitter, Manager, Runtime};

// 알림 클릭 시 사용할 URI 스킴

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
    keyword: Option<String>, // 참조 대신 소유권 있는 String 사용
) -> Result<(), Box<dyn Error>> {
    use winrt_notification::{Duration, Sound, Toast};

    // 디버그 로그 파일 경로
    let log_path = std::env::temp_dir().join("mcplink_notification.log");

    // 로그 파일에 기록
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(
            file,
            "[{}] Showing Windows notification: title='{}', body='{}', keyword={:?}",
            chrono::Local::now().format("%H:%M:%S"),
            title,
            body,
            keyword
        );
    }

    // URI 스킴 생성
    let uri = if let Some(ref kw) = keyword {
        format!("{}?keyword={}", URI_SCHEME, kw)
    } else {
        URI_SCHEME.to_string()
    };

    // 로그 파일에 URI 기록
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(
            file,
            "[{}] Using URI: {}",
            chrono::Local::now().format("%H:%M:%S"),
            uri
        );
    }

    // Tauri v2.0 호환성: 키워드를 즉시 저장 (알림이 클릭되지 않더라도)
    // 이렇게 하면 앱이 활성화될 때 키워드를 찾을 수 있습니다
    if let Some(kw) = &keyword {
        // 키워드 상태 저장
        // 이 키워드는 check_and_mark_app_activated 함수에서 사용됨
        let keyword_pending_path = std::env::temp_dir().join("mcplink_pending_keyword.txt");
        if let Ok(mut file) = std::fs::File::create(&keyword_pending_path) {
            let _ = write!(file, "{}", kw);

            // 로그 기록
            if let Ok(mut log_file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path)
            {
                let _ = writeln!(
                    log_file,
                    "[{}] 키워드 임시 저장됨 (pending): {}",
                    chrono::Local::now().format("%H:%M:%S"),
                    kw
                );
            }
        }
    }

    // 알림 생성 - Toast 객체 설정
    // winrt-notification 라이브러리는 직접적인 딥링크 설정을 지원하지 않음
    // 대신 PowerShell 앱 ID를 사용하여 알림을 표시하고,
    // 사용자가 알림을 클릭하면 앱 자체의 딥링크 핸들러가 처리함
    // Tauri v2.0 호환성: 앱 식별자 사용    let toast = Toast::new("com.ssafy12ksh.app")        .title(title)        .text1(body)        .sound(Some(Sound::Default))        .duration(Duration::Short);

    // 로그에 알림 설정 기록
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(
            file,
            "[{}] Toast notification configured",
            chrono::Local::now().format("%H:%M:%S")
        );
    }

    // 알림 표시
    let result = toast.show();

    // 결과 로깅
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        match &result {
            Ok(_) => {
                let _ = writeln!(
                    file,
                    "[{}] Successfully showed Windows notification",
                    chrono::Local::now().format("%H:%M:%S")
                );
            }
            Err(e) => {
                let _ = writeln!(
                    file,
                    "[{}] Failed to show Windows notification: {}",
                    chrono::Local::now().format("%H:%M:%S"),
                    e
                );
            }
        }
    }

    // 알림이 성공적으로 표시되면 키워드 전달을 위한 임시 파일 생성
    if result.is_ok() && keyword.is_some() {
        let kw = keyword.unwrap();

        // 알림 클릭 로그 파일 경로
        let click_log_path = std::env::temp_dir().join("mcplink_notification_click.log");

        // 로그 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&click_log_path)
        {
            let _ = writeln!(
                file,
                "[{}] 알림 표시 성공, 키워드 전달을 위한 임시 파일 생성: {}",
                chrono::Local::now().format("%H:%M:%S"),
                kw
            );
        }

        // 키워드 상태 저장
        // 이 키워드는 check_and_mark_app_activated 함수에서 사용됨
        // 알림 클릭 시 mcplink:// 프로토콜 처리 이후 앱이 실행되고 이 키워드를 읽음
        let keyword_tmp_path = std::env::temp_dir().join("mcplink_pending_keyword.txt");
        if let Ok(mut file) = std::fs::File::create(&keyword_tmp_path) {
            let _ = write!(file, "{}", kw);
        }
    }

    // winrt-notification::Error를 Box<dyn Error>로 변환
    match result {
        Ok(()) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

/// macOS에서 알림 표시
#[cfg(target_os = "macos")]
pub fn show_macos_notification(
    title: &str,
    body: &str,
    keyword: Option<String>,
) -> Result<(), Box<dyn Error>> {
    use notify_rust::{Hint, Notification};

    // macOS 알림 표시
    let mut notification = Notification::new();

    // 알림 기본 설정
    notification
        .summary(title)
        .body(body)
        .icon("icons/icon.png")
        .sound_name("default")
        .hint(Hint::CustomInt(
            "sender-pid".to_owned(),
            std::process::id() as i32,
        ));

    // URI 스킴 추가
    if let Some(ref kw) = keyword {
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
    keyword: Option<String>,
) -> Result<(), Box<dyn Error>> {
    use notify_rust::Notification;

    // Linux 알림 표시
    let mut notification = Notification::new();

    // 알림 기본 설정
    notification
        .summary(title)
        .body(body)
        .icon("icons/icon.png");

    // URI 스킴 추가
    if let Some(ref kw) = keyword {
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
    keyword: Option<String>,
) -> Result<(), String> {
    // 디버그 로그 파일 경로
    let log_path = std::env::temp_dir().join("mcplink_notification.log");

    // 로그 파일에 시작 기록
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(
            file,
            "[{}] show_notification called: title='{}', body='{}', keyword={:?}",
            chrono::Local::now().format("%H:%M:%S"),
            title,
            body,
            keyword
        );
    }

    // 키워드가 있으면 상태에 저장
    if let Some(ref kw) = keyword {
        app.state::<KeywordState>().set_keyword(kw.clone());

        // 로그 파일에 기록
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path)
        {
            let _ = writeln!(
                file,
                "[{}] Keyword saved to state: {}",
                chrono::Local::now().format("%H:%M:%S"),
                kw
            );
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
            .open(&log_path)
        {
            let _ = writeln!(
                file,
                "[{}] Emitted store-keyword event: {}",
                chrono::Local::now().format("%H:%M:%S"),
                kw
            );
        }
    }

    // 플랫폼별 알림 표시
    let result = match () {
        #[cfg(target_os = "windows")]
        () => show_windows_notification(&title, &body, keyword.clone())
            .map_err(|e| format!("Windows notification error: {}", e)),

        #[cfg(target_os = "macos")]
        () => show_macos_notification(&title, &body, keyword.clone())
            .map_err(|e| format!("macOS notification error: {}", e)),

        #[cfg(target_os = "linux")]
        () => show_linux_notification(&title, &body, keyword.clone())
            .map_err(|e| format!("Linux notification error: {}", e)),

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        () => Err("Unsupported platform".to_string()),
    };

    // 결과 로깅
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        match &result {
            Ok(_) => {
                let _ = writeln!(
                    file,
                    "[{}] Successfully showed notification",
                    chrono::Local::now().format("%H:%M:%S")
                );
            }
            Err(e) => {
                let _ = writeln!(
                    file,
                    "[{}] Failed to show notification: {}",
                    chrono::Local::now().format("%H:%M:%S"),
                    e
                );
            }
        }
    }

    result
}

/// 알림 시스템 초기화
pub fn init_notification_system<R: Runtime>(app: &mut tauri::App<R>) -> Result<(), Box<dyn Error>> {
    // KeywordState 등록
    app.manage(KeywordState::new());

    // 로그 파일 경로
    let log_path = std::env::temp_dir().join("mcplink_activation.log");

    // 로그 파일에 초기화 기록
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(
            file,
            "=== [{}] 알림 시스템 초기화됨 ===",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        );
    }

    // 시작 시 키워드 파일 상태 확인
    check_keyword_files();

    Ok(())
}

/// 키워드 파일들의 상태를 확인하고 정리하는 함수
fn check_keyword_files() {
    // 로그 파일 경로
    let log_path = std::env::temp_dir().join("mcplink_activation.log");

    // 각 키워드 파일 경로
    let keyword_path = std::env::temp_dir().join("mcplink_last_keyword.txt");
    let pending_keyword_path = std::env::temp_dir().join("mcplink_pending_keyword.txt");

    // 파일 상태 로깅
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(
            file,
            "[{}] 키워드 파일 상태 확인: last_keyword 존재={}, pending_keyword 존재={}",
            chrono::Local::now().format("%H:%M:%S"),
            keyword_path.exists(),
            pending_keyword_path.exists()
        );
    }

    // pending 키워드가 있고 last 키워드가 없다면 복사
    if pending_keyword_path.exists() && !keyword_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&pending_keyword_path) {
            if !content.is_empty() {
                // last_keyword 파일 생성
                if let Ok(mut file) = std::fs::File::create(&keyword_path) {
                    let _ = write!(file, "{}", content);

                    // 로그 기록
                    if let Ok(mut log) = std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(true)
                        .open(&log_path)
                    {
                        let _ = writeln!(
                            log,
                            "[{}] Pending 키워드 '{}'를 last_keyword로 복사함",
                            chrono::Local::now().format("%H:%M:%S"),
                            content
                        );
                    }

                    // pending 파일 삭제
                    let _ = std::fs::remove_file(&pending_keyword_path);
                }
            }
        }
    }

    // 만약 어떤 이유로든 둘 다 있다면, pending 파일 삭제
    if pending_keyword_path.exists() && keyword_path.exists() {
        if let Ok(_) = std::fs::remove_file(&pending_keyword_path) {
            // 로그 기록
            if let Ok(mut log_file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path)
            {
                let _ = writeln!(
                    log_file,
                    "[{}] 중복 키워드 파일 감지: pending_keyword 삭제함",
                    chrono::Local::now().format("%H:%M:%S")
                );
            }
        }
    }
}
