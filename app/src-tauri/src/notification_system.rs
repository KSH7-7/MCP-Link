//! 크로스 플랫폼 알림 시스템 구현
//! Windows에서는 winrt-notification, 다른 플랫폼에서는 notify-rust 사용

use std::error::Error;
use tauri::{AppHandle, Emitter, Manager, Runtime};

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
        use std::io::Write;
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
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] Using URI: {}",
            chrono::Local::now().format("%H:%M:%S"),
            uri
        );
    }

    // 알림이 표시될 때 바로 키워드 파일 생성 (알림 클릭 감시 스레드에서 읽을 수 있게 함)
    // 원래 알림 클릭 시 URI 스킴으로 처리했어야 하지만 Windows에서 제대로 동작하지 않는 경우를 위한 대책
    if let Some(ref kw) = keyword {
        // 알림 클릭 로그 파일 경로
        let click_log_path = std::env::temp_dir().join("mcplink_notification_click.log");

        // 로그 파일에 기록
        if let Ok(mut click_file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&click_log_path)
        {
            use std::io::Write;
            let _ = writeln!(
                click_file,
                "[{}] 알림과 함께 키워드 설정됨: {}",
                chrono::Local::now().format("%H:%M:%S"),
                kw
            );
        }

        // 키워드 파일 생성
        let keyword_path = std::env::temp_dir().join("mcplink_last_keyword.txt");
        if let Ok(mut file) = std::fs::File::create(&keyword_path) {
            use std::io::Write;
            let _ = write!(file, "{}", kw);

            // 로그 파일에 기록
            if let Ok(mut click_file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&click_log_path)
            {
                use std::io::Write;
                let _ = writeln!(
                    click_file,
                    "[{}] 알림 표시와 함께 키워드 파일 생성됨: {}",
                    chrono::Local::now().format("%H:%M:%S"),
                    kw
                );
            }
        }
    }

    // 알림 생성 - 추가 핸들러 구성
    let toast = Toast::new(Toast::POWERSHELL_APP_ID)
        .title(title)
        .text1(body)
        .sound(Some(Sound::Default))
        .duration(Duration::Short);

    // 로그에 알림 설정 기록
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] Toast notification configured with activation handler",
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
        use std::io::Write;
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
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] Show notification called: title='{}', body='{}', keyword={:?}",
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
            use std::io::Write;
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
            use std::io::Write;
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
        use std::io::Write;
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
        use std::io::Write;
        let _ = writeln!(
            file,
            "=== [{}] 알림 시스템 초기화됨 ===",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        );
    }

    // 알림 클릭 감지를 위한 키워드 파일 감시 스레드 시작
    let app_handle = app.handle().clone();

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
            "[{}] 알림 시스템 초기화 - 키워드 파일 감시 시작",
            chrono::Local::now().format("%H:%M:%S")
        );
    }

    // 이전에 생성된 키워드 파일이 있으면 먼저 체크
    let keyword_path = std::env::temp_dir().join("mcplink_last_keyword.txt");
    if keyword_path.exists() {
        if let Ok(keyword) = std::fs::read_to_string(&keyword_path) {
            if !keyword.is_empty() {
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
                        "[{}] 초기화 중 이전 키워드 파일 발견: {}",
                        chrono::Local::now().format("%H:%M:%S"),
                        keyword
                    );
                }

                // 키워드 파일 삭제
                let _ = std::fs::remove_file(&keyword_path);
            }
        }
    }

    // 알림 클릭 감시 스레드 시작 - 키워드 파일 모니터링
    std::thread::spawn(move || {
        // 1초 대기
        std::thread::sleep(std::time::Duration::from_secs(2));

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
                "[{}] 감시 스레드 시작: 키워드 파일 모니터링",
                chrono::Local::now().format("%H:%M:%S")
            );
        }

        // 무한 루프로 0.5초마다 키워드 파일 확인
        loop {
            // 키워드 파일 경로
            let keyword_path = std::env::temp_dir().join("mcplink_last_keyword.txt");

            // 파일이 존재하면 읽기 시도
            if keyword_path.exists() {
                if let Ok(keyword) = std::fs::read_to_string(&keyword_path) {
                    if !keyword.is_empty() {
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
                                "[{}] 감시 스레드: 키워드 파일 감지됨: {}",
                                chrono::Local::now().format("%H:%M:%S"),
                                keyword
                            );
                        }

                        // 알림 클릭 시 동작 구현 - 앱이 먼저 활성화된 경우 검색 실행
                        // 1. 먼저 키워드 파일을 삭제하여 중복 처리 방지
                        let _ = std::fs::remove_file(&keyword_path);

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
                                "[{}] 감시 스레드: 키워드 파일 삭제됨 (처리 전)",
                                chrono::Local::now().format("%H:%M:%S")
                            );
                        }

                        // 2. 앱 강제 활성화 시도 - 다중 시도 구현
                        let mut activation_success = false;
                        for attempt in 1..=5 { // 시도 횟수 증가
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
                                    "[{}] 감시 스레드: 앱 활성화 시도 #{}",
                                    chrono::Local::now().format("%H:%M:%S"),
                                    attempt
                                );
                            }

                            // 활성화 시도
                            match crate::force_activate::force_app_to_foreground() {
                                Err(e) => {
                                    // 오류 로깅
                                    if let Ok(mut file) = std::fs::OpenOptions::new()
                                        .create(true)
                                        .write(true)
                                        .append(true)
                                        .open(&log_path)
                                    {
                                        use std::io::Write;
                                        let _ = writeln!(
                                            file,
                                            "[{}] 감시 스레드: 앱 활성화 오류 #{}: {}",
                                            chrono::Local::now().format("%H:%M:%S"),
                                            attempt,
                                            e
                                        );
                                    }

                                    // 잠시 대기 후 재시도 (시도마다 대기시간 증가)
                                    if attempt < 5 {
                                        std::thread::sleep(std::time::Duration::from_millis(
                                            300 * attempt as u64,
                                        ));
                                    }
                                }
                                Ok(_) => {
                                    // 성공 로깅
                                    if let Ok(mut file) = std::fs::OpenOptions::new()
                                        .create(true)
                                        .write(true)
                                        .append(true)
                                        .open(&log_path)
                                    {
                                        use std::io::Write;
                                        let _ = writeln!(
                                            file,
                                            "[{}] 감시 스레드: 앱 활성화 성공 #{}",
                                            chrono::Local::now().format("%H:%M:%S"),
                                            attempt
                                        );
                                    }
                                    activation_success = true;
                                    break;
                                }
                            }
                        }

                        // 3. 충분한 시간 대기 (앱 초기화 및 활성화 완료를 기다림)
                        std::thread::sleep(std::time::Duration::from_millis(1500));

                        // 4. 창 조작 시도
                        let mut window_found = false;
                        for window_attempt in 1..=3 {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                window_found = true;
                                
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
                                        "[{}] 감시 스레드: 창 상태 변경 시도 #{}",
                                        chrono::Local::now().format("%H:%M:%S"),
                                        window_attempt
                                    );
                                }

                                // 창이 보이도록 설정
                                let _ = window.unminimize();
                                let _ = window.show();
                                let _ = window.set_focus();

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
                                        "[{}] 감시 스레드: 창 상태 변경 완료",
                                        chrono::Local::now().format("%H:%M:%S")
                                    );
                                }
                                break;
                            } else {
                                // 창을 찾지 못한 경우
                                if let Ok(mut file) = std::fs::OpenOptions::new()
                                    .create(true)
                                    .write(true)
                                    .append(true)
                                    .open(&log_path)
                                {
                                    use std::io::Write;
                                    let _ = writeln!(
                                        file,
                                        "[{}] 감시 스레드: 창을 찾을 수 없음 (시도 #{})",
                                        chrono::Local::now().format("%H:%M:%S"),
                                        window_attempt
                                    );
                                }

                                // 다음 시도 전 잠시 대기 
                                std::thread::sleep(std::time::Duration::from_millis(500));
                            }
                        }

                        // 추가 대기 시간 (앱이 완전히 UI 렌더링을 마칠 때까지)
                        std::thread::sleep(std::time::Duration::from_millis(700));

                        // 5. 키워드 검색 이벤트 발생
                        if window_found {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                // 키워드 이벤트 발생 
                                let keyword_clone = keyword.clone();
                                let emit_result = window.emit("search-keyword", keyword_clone);

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
                                        "[{}] 감시 스레드: search-keyword 이벤트 발생: {} (결과: {})",
                                        chrono::Local::now().format("%H:%M:%S"),
                                        keyword,
                                        if emit_result.is_ok() {
                                            "성공"
                                        } else {
                                            "실패"
                                        }
                                    );
                                }
                            }
                        } else {
                            // 창을 찾지 못한 경우
                            if let Ok(mut file) = std::fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .append(true)
                                .open(&log_path)
                            {
                                use std::io::Write;
                                let _ = writeln!(
                                    file,
                                    "[{}] 감시 스레드: 창을 찾을 수 없어 키워드 이벤트 발생 실패",
                                    chrono::Local::now().format("%H:%M:%S")
                                );
                            }
                        }
                    }
                }
            }

            // 0.5초 대기 후 다시 시도 (더 빠른 응답을 위해)
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });

    Ok(())
}
