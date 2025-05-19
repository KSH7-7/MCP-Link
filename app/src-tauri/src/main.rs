// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// 단순히 lib.rs의 run() 함수만 호출
fn main() {
    mcp_link::run();
}
