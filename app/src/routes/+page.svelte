<script lang="ts">
  import { onMount } from 'svelte';
  import Card from '../lib/components/card.svelte';
  import { getCards } from '../lib/data/cards';
  import MCPCard from '../lib/components/mcp-card.svelte';
  import { fetchMCPCards } from '../lib/data/mcp-api';
  import { goto } from '$app/navigation';
  
  // define data type to receive from backend
  import type { Card as CardType } from '../lib/data/cards';
  import type { MCPCard as MCPCardType } from '../lib/data/mcp-api';

  // general card data
  let cards: CardType[] = [];
  
  // MCP card data
  let mcpCards: MCPCardType[] = [];
  
  // data loading state
  let loading = true;
  let mcpLoading = true;
  
  // selected card info
  let selectedCard: CardType | null = null;
  
  // card selection handler
  function handleCardClick(card: CardType) {
    selectedCard = card;
  }
  
  // get data when component is mounted
  onMount(async () => {
    try {
      // Automatically redirect to the Installed-MCP page
      goto('/Installed-MCP');
      
      // get general card data
      loading = true;
      cards = await getCards();
      loading = false;
      
      // get MCP card data (using Tauri backend)
      mcpLoading = true;
      mcpCards = await fetchMCPCards();
      mcpLoading = false;
    } catch (error) {
      console.error('error: get data', error);
      loading = false;
      mcpLoading = false;
    }
  });
</script>

<div class="p-8">
  
  <div class="flex flex-col gap-1.5">
    <!-- top 3 square cards -->
    <div class="p-2 rounded-lg">
      {#if loading}
        <div class="flex justify-center items-center h-64">
          <span class="loading loading-spinner loading-lg text-primary"></span>
        </div>
      {:else}
        <div class="grid grid-cols-1 sm:grid-cols-3 md:grid-cols-3 gap-4">
        </div>
      {/if}
    </div>
    
    <!-- bottom MCP card container -->
    <div class="p-2 rounded-lg">
      <p class="text-1xl font-bold">Recommended MCP</p>
      
      {#if mcpLoading}
        <div class="flex justify-center items-center h-64">
          <span class="loading loading-spinner loading-lg text-primary"></span>
        </div>
      {:else}
        <div class="grid grid-cols-1 sm:grid-cols-1 gap-4">
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
</div>
