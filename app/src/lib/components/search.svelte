<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte"

  // Create event dispatcher
  const dispatch = createEventDispatcher<{
    search: { value: string }
  }>()

  // START: Add initialValue prop
  export let initialValue: string = ""
  export let placeholder: string = "MCP name or keyword search..."
  export let customClass: string = "input input-bordered rounded-[10px]"
  // END: Add initialValue prop

  // Search value
  let searchValue = ""

  // Debounce timer
  let debounceTimer: ReturnType<typeof setTimeout> | null = null

  // Debounce delay (milliseconds)
  const DEBOUNCE_DELAY = 300

  // START: Set searchValue with initialValue in onMount and listen for set-search-term event
  onMount(() => {
    // 초기값 설정
    if (initialValue) {
      searchValue = initialValue
    }
    
    // set-search-term 이벤트 리스너 추가 - 알림에서 검색창에 키워드 설정용
    const handleSetSearchTerm = (event: CustomEvent<string>) => {
      if (event?.detail) {
        searchValue = event.detail;
        
        // 검색창 강조 효과
        setTimeout(() => {
          const inputElement = document.querySelector('input[type="search"]');
          if (inputElement) {
            inputElement.classList.add('highlight-search');
            // 포커스 설정
            inputElement.focus();
            // 1.5초 후 강조 효과 제거
            setTimeout(() => {
              inputElement.classList.remove('highlight-search');
            }, 1500);
          }
        }, 10);
      }
    };
    
    // 이벤트 리스너 등록
    document.addEventListener('set-search-term', handleSetSearchTerm as EventListener);
    
    // 정리 함수
    return () => {
      document.removeEventListener('set-search-term', handleSetSearchTerm as EventListener);
    };
  });
  // END: Set searchValue with initialValue in onMount

  // Debounced search function
  function debouncedSearch() {
    if (debounceTimer) clearTimeout(debounceTimer)
    debounceTimer = setTimeout(() => {
      dispatch("search", { value: searchValue })
    }, DEBOUNCE_DELAY)
  }

  // Search event handler
  function handleSearch() {
    dispatch("search", { value: searchValue })
  }

  // Keyboard event handler (Enter key pressed to search)
  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      if (debounceTimer) clearTimeout(debounceTimer)
      handleSearch()
    }
  }
</script>

<label class="{customClass} flex items-center">
  <input type="search" class="grow" placeholder="{placeholder}" bind:value={searchValue} on:input={debouncedSearch} on:keydown={handleKeyDown} />
</label>

<style>
  label {
    width: 100%; /* Control width from parent element if necessary */
  }
  
  /* 검색창 강조 스타일 */
  :global(.highlight-search) {
    border: 2px solid #ffd700 !important; /* 금색 테두리 */
    box-shadow: 0 0 10px rgba(255, 215, 0, 0.5) !important; /* 금색 그림자 */
    transition: all 0.3s ease-in-out !important;
    animation: pulse 1.5s ease-out !important;
  }
  
  /* 펄스 애니메이션 */
  @keyframes pulse {
    0% { transform: scale(1); }
    25% { transform: scale(1.03); }
    50% { transform: scale(1); }
    75% { transform: scale(1.02); }
    100% { transform: scale(1); }
  }
</style>
