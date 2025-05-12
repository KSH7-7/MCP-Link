<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  // Create event dispatcher
  const dispatch = createEventDispatcher<{
    search: { value: string }
  }>();
  
  // Search value
  let searchValue = '';
  
  // 디바운스 타이머
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  
  // 디바운스 지연 시간 (밀리초)
  const DEBOUNCE_DELAY = 300;
  
  // 디바운스된 검색 함수
  function debouncedSearch() {
    // 이전 타이머가 있으면 취소
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }
    
    // 새 타이머 설정
    debounceTimer = setTimeout(() => {
      dispatch('search', { value: searchValue });
    }, DEBOUNCE_DELAY);
  }
  
  // Search event handler
  function handleSearch() {
    dispatch('search', { value: searchValue });
  }
  
  // Keyboard event handler (Enter key pressed to search)
  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      // 엔터키 누르면 즉시 검색 (디바운스 취소)
      if (debounceTimer) {
        clearTimeout(debounceTimer);
      }
      handleSearch();
    }
  }
</script>

<label class="input input-bordered rounded-[10px]">
  <input 
    type="search" 
    class="grow" 
    placeholder="MCP name or keyword search..." 
    bind:value={searchValue} 
    on:input={debouncedSearch}
    on:keydown={handleKeyDown}
  />
</label>

  