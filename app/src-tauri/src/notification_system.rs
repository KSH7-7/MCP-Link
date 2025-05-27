// Windows uses winrt-notification, other platforms use notify-rust

use crate::{to_string_error, AppError, AppResult};
use std::error::Error;
use tauri::{AppHandle, Emitter, Manager, Runtime};

// URI scheme for notification click
const URI_SCHEME: &str = "mcplink://notification";

// keyword status codes
pub const KEYWORD_STATUS_NONE: u8 = 0; // no keyword
pub const KEYWORD_STATUS_PENDING: u8 = 1; // pending
pub const KEYWORD_STATUS_PROCESSING: u8 = 2; // processing
pub const KEYWORD_STATUS_COMPLETED: u8 = 3; // completed
pub const KEYWORD_STATUS_FAILED: u8 = 4; // failed

// central structure to store keyword status
pub struct KeywordState {
    keyword: std::sync::Mutex<Option<String>>,
    status: std::sync::atomic::AtomicU8,
    last_click_time: std::sync::Mutex<Option<std::time::Instant>>,
    source: std::sync::Mutex<String>,
    retry_count: std::sync::atomic::AtomicUsize,
    max_retries: std::sync::atomic::AtomicUsize,
    app_activated: std::sync::atomic::AtomicBool,
    last_updated: std::sync::atomic::AtomicU64,
    queue: std::sync::Mutex<Vec<String>>,
}

impl KeywordState {
    pub fn new() -> Self {
        Self {
            keyword: std::sync::Mutex::new(None),
            status: std::sync::atomic::AtomicU8::new(0),
            last_click_time: std::sync::Mutex::new(None),
            source: std::sync::Mutex::new(String::new()),
            retry_count: std::sync::atomic::AtomicUsize::new(0),
            max_retries: std::sync::atomic::AtomicUsize::new(3),
            app_activated: std::sync::atomic::AtomicBool::new(false),
            last_updated: std::sync::atomic::AtomicU64::new(0),
            queue: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn set_keyword(&self, keyword: String, source: &str, status: u8) {
        let should_queue = {
            let current_status = self.status.load(std::sync::atomic::Ordering::SeqCst);
            current_status == 2
        };

        if should_queue {
            let mut queue = self.queue.lock().unwrap();
            if !queue.contains(&keyword) {
                queue.push(keyword.clone());
                return;
            }
        }

        {
            let mut guard = self.keyword.lock().unwrap();
            *guard = Some(keyword.clone());
        }

        {
            let mut src = self.source.lock().unwrap();
            *src = source.to_string();
        }

        self.status
            .store(status, std::sync::atomic::Ordering::SeqCst);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.last_updated
            .store(now, std::sync::atomic::Ordering::SeqCst);

        let keyword_path = std::env::temp_dir().join("mcplink_last_keyword.txt");
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&keyword_path)
        {
            use std::io::Write;
            let _ = write!(file, "{}", keyword);
        }
    }

    pub fn take_keyword(&self) -> Option<String> {
        let current_status = self.get_status();
        if current_status == 0 {
            return None;
        }

        if current_status == 1 {
            self.set_status(2);
        }

        let keyword_result = {
            let guard = self.keyword.lock().unwrap();
            guard.clone()
        };

        if keyword_result.is_some() {
            return keyword_result;
        }

        let keyword_path = std::env::temp_dir().join("mcplink_last_keyword.txt");
        if keyword_path.exists() {
            if let Ok(metadata) = std::fs::metadata(&keyword_path) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        // ignore files older than 10 seconds and delete
                        if elapsed.as_secs() > 10 {
                            let _ = std::fs::remove_file(&keyword_path);
                            return None;
                        }
                    }
                }
            }

            if let Ok(content) = std::fs::read_to_string(&keyword_path) {
                if !content.is_empty() {
                    {
                        let mut guard = self.keyword.lock().unwrap();
                        *guard = Some(content.clone());
                    }
                    return Some(content);
                }
            }

            let _ = std::fs::remove_file(&keyword_path);
        }

        keyword_result
    }

    pub fn has_keyword(&self) -> bool {
        if self.get_status() == 0 {
            return false;
        }

        {
            let guard = self.keyword.lock().unwrap();
            if guard.is_some() {
                return true;
            }
        }

        let keyword_path = std::env::temp_dir().join("mcplink_last_keyword.txt");
        if keyword_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&keyword_path) {
                return !content.is_empty();
            }
        }

        false
    }

    pub fn set_status(&self, status: u8) {
        let previous = self
            .status
            .swap(status, std::sync::atomic::Ordering::SeqCst);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.last_updated
            .store(now, std::sync::atomic::Ordering::SeqCst);

        if status == 1 {
            self.retry_count
                .store(0, std::sync::atomic::Ordering::SeqCst);
        }

        if (status == 3 || status == 4) && previous == 2 {
            let next_keyword = {
                let mut queue = self.queue.lock().unwrap();
                if !queue.is_empty() {
                    Some(queue.remove(0))
                } else {
                    None
                }
            };

            if let Some(keyword) = next_keyword {
                self.set_keyword(keyword, "queue", 1);
            }
        }
    }

    pub fn get_status(&self) -> u8 {
        self.status.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn is_pending(&self) -> bool {
        self.get_status() == 1
    }

    pub fn set_pending(&self, pending: bool) {
        if pending {
            self.set_status(1);
        } else {
            let current = self.get_status();
            if current == 1 {
                self.set_status(0);
            }
        }
    }

    pub fn update_click_time(&self) {
        let mut guard = self.last_click_time.lock().unwrap();
        *guard = Some(std::time::Instant::now());
    }

    pub fn get_last_click_time(&self) -> Option<std::time::Instant> {
        let guard = self.last_click_time.lock().unwrap();
        *guard
    }

    pub fn increment_retry_count(&self) -> usize {
        let current = self
            .retry_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        current + 1
    }

    pub fn get_retry_count(&self) -> usize {
        self.retry_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn set_app_activated(&self, activated: bool) {
        self.app_activated
            .store(activated, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_app_activated(&self) -> bool {
        self.app_activated
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    // function to clear all states
    pub fn clear_all(&self) {
        // clear keyword
        if let Ok(mut keyword) = self.keyword.lock() {
            *keyword = None;
        }

        // initialize status
        self.status
            .store(KEYWORD_STATUS_NONE, std::sync::atomic::Ordering::Relaxed);

        // clear click time
        if let Ok(mut last_click_time) = self.last_click_time.lock() {
            *last_click_time = None;
        }

        // clear source
        if let Ok(mut source) = self.source.lock() {
            *source = String::new();
        }

        // initialize retry count
        self.retry_count
            .store(0, std::sync::atomic::Ordering::Relaxed);

        // initialize app activation state
        self.app_activated
            .store(false, std::sync::atomic::Ordering::Relaxed);

        // initialize last update time
        self.last_updated
            .store(0, std::sync::atomic::Ordering::Relaxed);

        // clear queue
        if let Ok(mut queue) = self.queue.lock() {
            queue.clear();
        }
    }
}

// show notification on Windows - improved version
#[cfg(target_os = "windows")]
pub fn show_windows_notification<R: Runtime>(
    app_handle: Option<&AppHandle<R>>,
    title: &str,
    body: &str,
    keyword: Option<String>,
) -> Result<(), Box<dyn Error>> {
    use winrt_notification::{Duration, Sound, Toast};

    // if keyword exists, save status
    if let Some(ref kw) = keyword {
        // 1. save to file (backup)
        let keyword_path = std::env::temp_dir().join("mcplink_last_keyword.txt");
        if let Ok(mut file) = std::fs::File::create(&keyword_path) {
            use std::io::Write;
            let _ = write!(file, "{}", kw);
        }

        // 2. save to KeywordState
        if let Some(app) = app_handle {
            if let Some(keyword_state) = app.try_state::<KeywordState>() {
                keyword_state.set_keyword(kw.clone(), "windows_notification", 1);
                keyword_state.set_pending(true);
                keyword_state.update_click_time();
            }
        }
    }

    // create notification
    let toast = Toast::new(Toast::POWERSHELL_APP_ID)
        .title(title)
        .text1(body)
        .sound(Some(Sound::Default))
        .duration(Duration::Short);

    // show notification
    let result = toast.show();

    // improved polling thread for notification click detection
    if result.is_ok() {
        if let Some(app) = app_handle {
            if let Some(kw) = keyword {
                let app_clone = app.clone();
                let kw_clone = kw.clone();

                std::thread::spawn(move || {
                    let mut activated = false;
                    let mut focus_detected = false;

                    // check for up to 15 seconds with 250ms intervals
                    for i in 0..60 {
                        std::thread::sleep(std::time::Duration::from_millis(250));

                        // check if keyword file is deleted (processed)
                        let keyword_path = std::env::temp_dir().join("mcplink_last_keyword.txt");
                        if !keyword_path.exists() && activated {
                            // file is deleted and already activated
                            break;
                        }

                        // check window focus
                        if let Some(window) = app_clone.get_webview_window("main") {
                            if let Ok(is_focused) = window.is_focused() {
                                if is_focused && !focus_detected {
                                    focus_detected = true;

                                    // if focus is detected, force activate immediately
                                    let _activation_result =
                                        crate::force_activate::force_app_to_foreground();

                                    if !activated {
                                        activated = true;
                                        use tauri::Emitter;
                                        let _ = window.emit("notification-clicked", &kw_clone);

                                        // specify that activation is from notification
                                        let _ = crate::force_activate::emit_app_activated_event_with_source(&app_clone, true);
                                    }
                                }
                            }
                        }

                        // if focus is not detected after 3 seconds, force activate
                        if i == 12 && !activated {
                            let _ = crate::force_activate::force_app_to_foreground();

                            if let Some(window) = app_clone.get_webview_window("main") {
                                use tauri::Emitter;
                                let _ = window.emit("notification-clicked", &kw_clone);

                                // specify that activation is from notification
                                let _ = crate::force_activate::emit_app_activated_event_with_source(
                                    &app_clone, true,
                                );
                            }
                            activated = true;
                        }
                    }
                });
            }
        }
    }

    match result {
        Ok(()) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

// show notification on macOS
#[cfg(target_os = "macos")]
pub fn show_macos_notification<R: Runtime>(
    app_handle: Option<&AppHandle<R>>,
    title: &str,
    body: &str,
    keyword: Option<String>,
) -> Result<(), Box<dyn Error>> {
    use notify_rust::{Hint, Notification};

    let mut notification = Notification::new();

    notification
        .summary(title)
        .body(body)
        .icon("icons/icon.png")
        .sound_name("default")
        .hint(Hint::CustomInt(
            "sender-pid".to_owned(),
            std::process::id() as i32,
        ));

    if let Some(ref kw) = keyword {
        notification.action("default", "Open");

        if let Some(app) = app_handle {
            if let Some(keyword_state) = app.try_state::<KeywordState>() {
                keyword_state.set_keyword(kw.clone(), "macos_notification", 1);
                keyword_state.set_pending(true);
                keyword_state.update_click_time();
            }
        }
    }

    notification.show()?;
    Ok(())
}

// show notification on Linux
#[cfg(target_os = "linux")]
pub fn show_linux_notification<R: Runtime>(
    app_handle: Option<&AppHandle<R>>,
    title: &str,
    body: &str,
    keyword: Option<String>,
) -> Result<(), Box<dyn Error>> {
    use notify_rust::Notification;

    let mut notification = Notification::new();

    notification
        .summary(title)
        .body(body)
        .icon("icons/icon.png");

    if let Some(ref kw) = keyword {
        notification.action("default", "Open");

        if let Some(app) = app_handle {
            if let Some(keyword_state) = app.try_state::<KeywordState>() {
                keyword_state.set_keyword(kw.clone(), "linux_notification", 1);
                keyword_state.set_pending(true);
                keyword_state.update_click_time();
            }
        }
    }

    notification.show()?;
    Ok(())
}

// cross-platform notification display function
#[tauri::command]
pub fn show_notification<R: Runtime>(
    app: AppHandle<R>,
    title: String,
    body: String,
    keyword: Option<String>,
) -> Result<(), String> {
    fn show_notification_internal<R: Runtime>(
        app: &AppHandle<R>,
        title: &str,
        body: &str,
        keyword: Option<String>,
    ) -> AppResult<()> {
        match () {
            #[cfg(target_os = "windows")]
            () => show_windows_notification(Some(app), title, body, keyword).map_err(|e| {
                AppError::NotificationError {
                    msg: format!("Windows notification error: {}", e),
                }
            }),

            #[cfg(target_os = "macos")]
            () => show_macos_notification(Some(app), title, body, keyword).map_err(|e| {
                AppError::NotificationError {
                    msg: format!("macOS notification error: {}", e),
                }
            }),

            #[cfg(target_os = "linux")]
            () => show_linux_notification(Some(app), title, body, keyword).map_err(|e| {
                AppError::NotificationError {
                    msg: format!("Linux notification error: {}", e),
                }
            }),

            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            () => Err(AppError::NotificationError {
                msg: "Unsupported platform".to_string(),
            }),
        }
    }

    to_string_error(show_notification_internal(&app, &title, &body, keyword))
}

// initialize notification system
pub fn init_notification_system<R: Runtime>(app: &mut tauri::App<R>) -> Result<(), Box<dyn Error>> {
    fn init_notification_system_internal<R: Runtime>(app: &mut tauri::App<R>) -> AppResult<()> {
        app.manage(KeywordState::new());

        let app_handle = app.handle().clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(2));

            if let Some(window) = app_handle.get_webview_window("main") {
                use tauri::Emitter;
                // initial automatic event is fromNotification=false
                let payload = serde_json::json!({
                    "fromNotification": false
                });
                let _ = window.emit("app-activated", payload);
            }
        });

        // clean temporary files
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

        Ok(())
    }

    match init_notification_system_internal(app) {
        Ok(()) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

// check notification click and get keyword function
#[tauri::command]
pub fn check_and_get_keyword<R: Runtime>(app: AppHandle<R>) -> Result<Option<String>, String> {
    fn check_and_get_keyword_internal<R: Runtime>(app: &AppHandle<R>) -> AppResult<Option<String>> {
        if let Some(keyword_state) = app.try_state::<KeywordState>() {
            if keyword_state.is_pending() {
                keyword_state.set_pending(false);

                if keyword_state.has_keyword() {
                    let keyword = keyword_state.take_keyword();
                    return Ok(keyword);
                }
            }
        }

        Ok(None)
    }

    to_string_error(check_and_get_keyword_internal(&app))
}

/// 알림 클릭 확인 및 키워드 가져오기 함수
#[tauri::command]
pub fn check_and_get_keyword<R: Runtime>(app: AppHandle<R>) -> Result<Option<String>, String> {
    fn check_and_get_keyword_internal<R: Runtime>(app: &AppHandle<R>) -> AppResult<Option<String>> {
        if let Some(keyword_state) = app.try_state::<KeywordState>() {
            if keyword_state.is_pending() {
                keyword_state.set_pending(false);
                
                if keyword_state.has_keyword() {
                    let keyword = keyword_state.take_keyword();
                    return Ok(keyword);
                }
            }
        }
        
        Ok(None)
    }

    to_string_error(check_and_get_keyword_internal(&app))
}
