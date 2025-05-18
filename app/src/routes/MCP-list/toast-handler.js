// MCP 목록 페이지에서 알림 처리
import { showNotification, setupNotificationListeners } from '../../lib/toast-system.js';
import { onMount, onDestroy } from 'svelte';
import { goto } from '$app/navigation';
import { page } from '$app/stores';
import { get } from 'svelte/store';

// 알림 핸들러 훅
export function useNotifications() {
  let cleanup = null;

  onMount(async () => {
    // 알림 이벤트 리스너 설정
    cleanup = await setupNotificationListeners();

    // URL에서 키워드 파라미터 추출
    const currentPage = get(page);
    if (currentPage.url.searchParams.has('keyword')) {
      const keyword = currentPage.url.searchParams.get('keyword');
      // 키워드로 검색 기능 트리거
      triggerSearch(keyword);
    }
  });

  onDestroy(() => {
    // 컴포넌트 언마운트 시 리스너 정리
    if (cleanup) {
      cleanup();
    }
  });

  // 검색 트리거 함수 (컴포넌트에서 구현해야 함)
  function triggerSearch(keyword) {
    // 이 함수는 MCP-list 페이지에서 오버라이드해야 함
    console.log('검색 트리거:', keyword);
    // 기본 구현은 콘솔 로그만 표시
  }

  // 알림 표시 함수 노출
  return {
    showNotification,
    triggerSearch
  };
}