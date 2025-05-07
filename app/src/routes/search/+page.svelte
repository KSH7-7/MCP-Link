<script lang="ts">
  import { onMount } from 'svelte';
  import Search from '../../lib/components/search.svelte';
  import MCPCard from '../../lib/components/mcp-card.svelte';
  import { fetchMCPCards } from '../../lib/data/mcp-api';
  
  // 타입 정의
  import type { MCPCard as MCPCardType } from '../../lib/data/mcp-api';
  
  // MCP 카드 데이터
  let mcpCards: MCPCardType[] = [];
  let filteredCards: MCPCardType[] = [];
  
  // 검색어
  let searchQuery = '';
  
  // 카테고리 필터
  let categoryFilter = 'all'; // 'all', 'project', 'design', 'code', 'test'
  
  // 데이터 로딩 상태
  let loading = true;
  
  // 컴포넌트 마운트 시 데이터 가져오기
  onMount(async () => {
    try {
      // Tauri 백엔드를 통해 MCP 카드 데이터 가져오기
      loading = true;
      mcpCards = await fetchMCPCards();
      applyFilters(); // 초기 필터링 적용
      loading = false;
    } catch (error) {
      console.error('MCP 데이터를 가져오는 중 오류 발생:', error);
      loading = false;
    }
  });
  
  // 필터링 적용 함수
  function applyFilters() {
    // 검색어 필터링
    let results = [...mcpCards];
    
    if (searchQuery.trim()) {
      results = results.filter(card => 
        card.title.toLowerCase().includes(searchQuery.toLowerCase()) || 
        card.description.toLowerCase().includes(searchQuery.toLowerCase())
      );
    }
    
    // 카테고리 필터링 (여기서는 제목에 특정 단어가 포함되어 있는지로 간단히 구현)
    if (categoryFilter !== 'all') {
      const filterTerms = {
        'project': ['프로젝트', '계획'],
        'design': ['디자인', '검토'],
        'code': ['코드', '리팩토링'],
        'test': ['테스트', '자동화']
      };
      
      const terms = filterTerms[categoryFilter as keyof typeof filterTerms];
      results = results.filter(card => 
        terms.some(term => 
          card.title.includes(term) || 
          card.description.includes(term)
        )
      );
    }
    
    filteredCards = results;
  }
  
  // 검색 이벤트 핸들러
  function handleSearchEvent(event: CustomEvent<{ value: string }>) {
    searchQuery = event.detail.value;
    applyFilters();
  }
  
</script>

<div class="p-8">
  <div class="flex flex-col gap-6">
    <!-- 검색 영역 -->

      <h1 class="text-2xl font-bold mb-4">MCP 검색</h1>
      
      <div class="w-full max-w-xl rounded-[10px]">
        <Search on:search={handleSearchEvent} />
      </div>

    
    <!-- 카드 목록 영역 -->

      <p class="text-xl font-semibold mb-2">검색 결과 ({filteredCards.length}개)</p>
      
      {#if loading}
        <div class="flex justify-center items-center h-64">
          <span class="loading loading-spinner loading-lg text-primary"></span>
        </div>
      {:else if filteredCards.length === 0}
        <div class="flex justify-center items-center h-64">
          <p class="text-gray-500">검색 결과가 없습니다.</p>
        </div>
      {:else}
        <div class="grid grid-cols-1 lg:grid-cols-1 gap-2">
          {#each filteredCards as card (card.id)}
            <MCPCard 
              id={card.id}
              title={card.title}
              description={card.description}
            />
          {/each}
        </div>
      {/if}
  </div>
</div> 