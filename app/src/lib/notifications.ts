// Tauri 2.0에서는 import 경로가 변경됨
import { invoke } from '@tauri-apps/api/core';

/**
 * 네이티브 알림을 표시합니다.
 * 
 * @param title 알림 제목
 * @param body 알림 내용
 * @param keyword 키워드 (선택 사항)
 * @returns 성공 여부
 */
export async function showNotification(
  title: string,
  body: string,
  keyword?: string
): Promise<boolean> {
  try {
    await invoke('show_notification', { title, body, keyword });
    return true;
  } catch (error) {
    console.error('알림 표시 실패:', error);
    return false;
  }
}

/**
 * URI 스킴으로 앱이 활성화될 때 호출됩니다.
 * 
 * @param uri URI 스킴
 * @returns 성공 여부
 */
export async function handleUriScheme(uri: string): Promise<boolean> {
  try {
    await invoke('handle_uri_scheme', { uri });
    return true;
  } catch (error) {
    console.error('URI 스킴 처리 실패:', error);
    return false;
  }
}