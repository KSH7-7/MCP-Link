import { onMount, onDestroy } from 'svelte';
import { listen } from '@tauri-apps/api/event';
import { appWindow } from '@tauri-apps/api/window';
import type { UnlistenFn } from '@tauri-apps/api/event';

/**
 * 알림 관련 이벤트를 처리하는 훅
 * 
 * @param onNavigateToMcpList MCP 목록 페이지로 이동 콜백
 */
export function useNotification(onNavigateToMcpList: (keyword: string) => void) {
  let unlistenFns: UnlistenFn[] = [];

  onMount(async () => {
    // 일반 탐색 이벤트 처리
    const unlisten1 = await listen('navigate-to', (event) => {
      console.log('Navigate to:', event.payload);
      // 페이지 이동 로직을 여기에 추가할 수 있습니다.
    });

    // MCP 목록 페이지 이동 이벤트 처리
    const unlisten2 = await listen('navigate-to-mcp-list-with-keyword', (event) => {
      const url = event.payload as string;
      if (url && url.includes('keyword=')) {
        const keyword = url.split('keyword=')[1];
        if (keyword) {
          onNavigateToMcpList(keyword);
        }
      }
    });

    // 새 키워드 이벤트 처리
    const unlisten3 = await listen('new-keywords', (event) => {
      console.log('New keywords:', event.payload);
      // 새 키워드 처리 로직을 여기에 추가할 수 있습니다.
    });

    unlistenFns = [unlisten1, unlisten2, unlisten3];
  });

  onDestroy(() => {
    // 모든 이벤트 리스너 정리
    unlistenFns.forEach(fn => fn());
  });

  return {
    // 필요한 함수를 여기에 추가할 수 있습니다.
  };
}