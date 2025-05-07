<script lang="ts">
  import { onMount } from 'svelte';
  import Search from '../../lib/components/search.svelte';
  import MCPCard from '../../lib/components/mcp-card.svelte';
  import { fetchMCPCards } from '../../lib/data/mcp-api';
  
  // define data type to receive from backend
  import type { MCPCard as MCPCardType } from '../../lib/data/mcp-api';
  
  // MCP card data
  let mcpCards: MCPCardType[] = [];
  
  // data loading state
  let loading = true;
  
  // get data when component is mounted
  onMount(async () => {
    try {
      // get MCP card data from Tauri backend
      loading = true;
      mcpCards = await fetchMCPCards();
      loading = false;
    } catch (error) {
      console.error('MCP 데이터를 가져오는 중 오류 발생:', error);
      loading = false;
    }
  });
  
  // Search event handler
  async function handleSearchEvent(event: CustomEvent<{ value: string }>) {
    const searchTerm = event.detail.value;
    console.log('검색어:', searchTerm);
    
    if (searchTerm) {
      loading = true;
      try {
        mcpCards = await fetchMCPCards(searchTerm);
      } catch (error) {
        console.error('검색 중 오류 발생:', error);
      } finally {
        loading = false;
      }
    } else {
      // 검색어가 없으면 모든 MCP 가져오기
      loading = true;
      try {
        mcpCards = await fetchMCPCards();
      } catch (error) {
        console.error('MCP 데이터를 가져오는 중 오류 발생:', error);
      } finally {
        loading = false;
      }
    }
  }
  
</script>

<div class="p-8">
  <div class="flex flex-col gap-6">
    <!-- card list area -->

      <div class="flex justify-between items-center mb-2">
        <p class="text-xl font-semibold">Search results ({mcpCards.length} MCPs)</p>
        <div class="ml-auto w-72 rounded-[10px]">
          <Search on:search={handleSearchEvent} />
        </div>
      </div>
      
      {#if loading}
        <div class="flex justify-center items-center h-64">
          <span class="loading loading-spinner loading-lg text-primary"></span>
        </div>
      {:else if mcpCards.length === 0}
        <div class="flex justify-center items-center h-64">
          <p class="text-gray-500">No search results found.</p>
        </div>
      {:else}
        <div class="grid grid-cols-1 lg:grid-cols-1 gap-2">
          {#each mcpCards as card (card.id)}
            <MCPCard 
              id={card.id}
              title={card.title}
              description={card.description}
              url={card.url}
              stars={card.stars}
            />
          {/each}
        </div>
      {/if}
  </div>
</div> 