// 알림 시스템 유틸리티 함수
// Tauri 2.0에서는 import 경로가 변경됨
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { goto } from '$app/navigation';
import toastStore from './stores/toast';

// Tauri의 알림 시스템을 통해 알림 표시
export async function showNotification(title, body, keyword) {
  try {
    await invoke('show_notification', { title, body, keyword });
    return true;
  } catch (error) {
    console.error('알림 표시 실패:', error);
    return false;
  }
}

// 알림 관련 이벤트 리스너 설정
export async function setupNotificationListeners() {
  // 탐색 이벤트 리스닝
  const unlistenNavigate = await listen('navigate-to', (event) => {
    const url = event.payload;
    if (url) {
      goto(url);
    }
  });

  // MCP 목록 페이지 내 키워드로 검색 이벤트 리스닝
  const unlistenKeyword = await listen('navigate-to-mcp-list-with-keyword', (event) => {
    const url = event.payload;
    if (url && url.includes('keyword=')) {
      goto(url);
    }
  });

  // 리스너 정리 함수 반환
  return () => {
    unlistenNavigate();
    unlistenKeyword();
  };
}

// URI 스킴 처리
export async function handleUriScheme(uri) {
  try {
    // Deep Link 플러그인은 자동으로 URL을 처리합니다.
    // 추가 처리가 필요한 경우 여기에 코드를 추가할 수 있습니다.
    return true;
  } catch (error) {
    console.error('URI 스킴 처리 실패:', error);
    return false;
  }
}

// 토스트 시스템 초기화
export function initToastSystem() {
  // 전역 이벤트 리스너 추가 
  document.addEventListener('show-toast', (event) => {
    const { message, type, duration, position } = event.detail;
    showToast(message, { type, duration, position });
  });
  
  console.log('Toast system initialized');
}

// 토스트 메시지 표시
export function showToast(message, options = {}) {
  const { 
    title = '',
    type = 'info', 
    duration = 3000, 
    position = 'bottom-center' 
  } = options;
  
  // Svelte store 업데이트
  toastStore.update(state => ({
    ...state,
    show: true,
    message,
    type,
    duration,
    position
  }));
  
  // 자동으로 숨기기
  setTimeout(() => {
    toastStore.update(state => ({
      ...state,
      show: false
    }));
  }, duration);
  
  return true;
}