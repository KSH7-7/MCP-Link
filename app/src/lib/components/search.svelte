<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  // Create event dispatcher
  const dispatch = createEventDispatcher<{
    search: { value: string }
  }>();
  
  // Search value
  let searchValue = '';
  
  // Search event handler
  function handleSearch() {
    dispatch('search', { value: searchValue });
  }
  
  // Keyboard event handler (Enter key pressed to search)
  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
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
    on:input={handleSearch}
    on:keydown={handleKeyDown}
  />
</label>

  