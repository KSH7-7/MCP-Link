<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { useNotifications } from './toast-handler.js';
  import { listen } from '@tauri-apps/api/event';
  
  // 알림 핸들러 초기화
  const { showNotification, triggerSearch } = useNotifications();
  
  let unlistenNavigate: (() => void) | null = null;
  let unlistenKeywords: (() => void) | null = null;
  let unlistenBasicNavigate: (() => void) | null = null;
  
  // 알림에서 검색 실행 함수
  function executeSearch(keyword: string) {
    console.log('executeSearch 함수 호출됨:', keyword);
    
    if (!keyword) {
      console.log('키워드가 비어있어 검색을 실행하지 않음');
      return;
    }
    
    console.log(`'${keyword}' 키워드로 검색 실행 중...`);
    
    // URL 업데이트
    try {
      console.log(`페이지 이동: /MCP-list?keyword=${encodeURIComponent(keyword)}`);
      goto(`/MCP-list?keyword=${encodeURIComponent(keyword)}`, { replaceState: false });
    } catch (e) {
      console.error('페이지 이동 오류:', e);
    }
    
    // 토스트 메시지 표시 (사용자 피드백)
    try {
      console.log('토스트 메시지 표시 중...');
      const toastEvent = new CustomEvent('show-toast', {
        detail: {
          message: `'${keyword}' 키워드로 검색합니다`,
          type: 'info',
          duration: 3000
        }
      });
      document.dispatchEvent(toastEvent);
      console.log('토스트 이벤트 발송 완료');
    } catch (e) {
      console.error('Toast event error:', e);
    }
    
    // 키워드 로컬 스토리지에 저장 (앱 다시 시작 시 사용)
    // 중요: 현재 페이지가 이미 MCP-list이고 키워드가 URL에 있다면 저장하지 않음
    // 이미 검색을 실행 중인 경우를 고려
    if (window.location.pathname === '/MCP-list') {
      const urlParams = new URLSearchParams(window.location.search);
      const urlKeyword = urlParams.get('keyword');
      if (urlKeyword === keyword) {
        console.log(`키워드 '${keyword}'는 이미 URL에 있어 저장하지 않음`);
        return;
      }
    }
    
    console.log(`키워드 저장: '${keyword}'를 로컬 스토리지에 저장 (일시적)`);
    // 세션 스토리지에 저장 (현재 세션에서만 유효하도록)
    sessionStorage.setItem('pendingSearchKeyword', keyword);
    
    // 로컬 스토리지에도 백업으로 저장 (타임스탬프도 함께 저장)
    const keywordData = {
      keyword: keyword,
      timestamp: Date.now(),
      used: false
    };
    localStorage.setItem('lastNotificationKeyword', keyword);
    
    console.log('executeSearch 함수 실행 완료');
  }
  
  onMount(async () => {
    // 이벤트 리스너 설정
    try {
      console.log('설정 중: NotificationHandler 이벤트 리스너');
      
      // navigate-to-mcp-list-with-keyword 이벤트 리스너
      unlistenNavigate = await listen('navigate-to-mcp-list-with-keyword', (event) => {
        console.log('수신: navigate-to-mcp-list-with-keyword 이벤트', event);
        const payload = event.payload;
        
        if (typeof payload === 'string' && payload.includes('keyword=')) {
          console.log('키워드 파라미터 포함', payload);
          const urlParams = new URLSearchParams(payload.split('?')[1]);
          const keyword = urlParams.get('keyword');
          if (keyword) {
            console.log('키워드 추출 성공:', keyword);
            executeSearch(keyword);
          }
        } else {
          console.log('키워드 파라미터 없음', payload);
        }
      });
      
      // navigate-to 이벤트 리스너 (기본 탐색용)
      const unlistenBasicNavigate = await listen('navigate-to', (event) => {
        console.log('수신: navigate-to 이벤트', event);
        const payload = event.payload;
        
        if (typeof payload === 'string') {
          console.log('이동 요청:', payload);
          
          if (payload.includes('keyword=')) {
            const urlParams = new URLSearchParams(payload.split('?')[1]);
            const keyword = urlParams.get('keyword');
            if (keyword) {
              console.log('키워드 추출 성공 (기본 탐색):', keyword);
              executeSearch(keyword);
              return;
            }
          }
          
          // 키워드가 없으면 일반 탐색
          goto(payload);
        }
      });
      
      // new-keywords 이벤트 리스너
      unlistenKeywords = await listen('new-keywords', (event) => {
        console.log('수신: new-keywords 이벤트', event);
        const keywords = event.payload;
        
        if (Array.isArray(keywords) && keywords.length > 0) {
          console.log('키워드 배열 수신:', keywords);
          // 첫 번째 키워드 사용
          executeSearch(keywords[0]);
        } else {
          console.log('키워드 배열이 비어있거나 형식이 잘못됨:', keywords);
        }
      });
      
      // URI 스킴 처리 리스너 설정
      window.addEventListener('DOMContentLoaded', () => {
        console.log('DOMContentLoaded 이벤트 발생, URI 스킴 검사');
        const uri = localStorage.getItem('pendingUriScheme');
        
        if (uri) {
          console.log('pendingUriScheme 발견:', uri);
          localStorage.removeItem('pendingUriScheme');
          
          if (uri.includes('keyword=')) {
            console.log('URI에서 키워드 파라미터 발견');
            const parts = uri.split('keyword=');
            if (parts.length > 1) {
              const keyword = parts[1].split(/[?&]/)[0];  // ? 또는 & 이후 부분 제거
              console.log('URI에서 키워드 추출:', keyword);
              executeSearch(keyword);
            }
          }
        } else {
          console.log('pendingUriScheme 없음');
        }
      });
      
      // 이미 DOM이 로드된 경우를 위한 체크
      if (document.readyState === 'complete') {
        console.log('문서가 이미 로드됨, URI 스킴 즉시 검사');
        const uri = localStorage.getItem('pendingUriScheme');
        if (uri) {
          console.log('pendingUriScheme 발견 (즉시 검사):', uri);
          localStorage.removeItem('pendingUriScheme');
          
          if (uri.includes('keyword=')) {
            const parts = uri.split('keyword=');
            if (parts.length > 1) {
              const keyword = parts[1].split(/[?&]/)[0];
              console.log('URI에서 키워드 추출 (즉시 검사):', keyword);
              executeSearch(keyword);
            }
          }
        }
      }
      
      console.log('완료: NotificationHandler 이벤트 리스너 설정');
    } catch (e) {
      console.error('Error setting up notification listeners:', e);
    }
  });
  
  onDestroy(() => {
    // 리스너 정리
    console.log('NotificationHandler 정리 중...');
    if (unlistenNavigate) {
      console.log('navigate-to-mcp-list-with-keyword 리스너 정리');
      unlistenNavigate();
    }
    if (unlistenKeywords) {
      console.log('new-keywords 리스너 정리');
      unlistenKeywords();
    }
    if (typeof unlistenBasicNavigate === 'function') {
      console.log('navigate-to 리스너 정리');
      unlistenBasicNavigate();
    }
    console.log('NotificationHandler 정리 완료');
  });
</script>

<!-- 이 컴포넌트는 UI가 없고 이벤트 처리만 담당합니다 -->