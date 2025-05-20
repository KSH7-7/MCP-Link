<script lang="ts">
  import { onMount } from "svelte"
  import { invoke } from "@tauri-apps/api/core"
  import MCPCardComponent from "$lib/components/mcp-card.svelte"
  import type { MCPCard, PageInfo } from "$lib/data/mcp-api"
  import { writable } from "svelte/store"

  // State variables
  let installedServers = writable<MCPCard[]>([])
  let originalServers = writable<MCPCard[]>([]) // Store original data
  let pageInfo = writable<PageInfo | null>(null)
  let isLoading = writable(true)
  let errorMessage = writable<string | null>(null)

  // Search related state
  let searchTerm = ""
  let isSearching = writable(false)

  // Define local MCP server interface
  interface LocalMCPServer {
    id: number
    name: string
    command: string
    args?: string[]
  }

  // Function to load installed MCP data
  async function loadInstalledMCPs() {
    isLoading.set(true)
    errorMessage.set(null)

    try {
      // Get the config file content directly from the backend
      const configContent = await invoke<string>("read_mcplink_config_content")

      let serverIds: number[] = []

      try {
        // Parse the content string
        const configData = JSON.parse(configContent)
        serverIds = Object.keys(configData)
          .map(Number)
          .filter((id) => !isNaN(id) && id !== -1) // -1 is a fallback server, so exclude it
      } catch (parseError: any) {
        // Handle parsing error, e.g., if the content is not valid JSON (though Rust returns "{}" if not found)
        errorMessage.set("Failed to parse local MCP configuration.")
        installedServers.set([])
        originalServers.set([])
        isLoading.set(false)
        return
      }

      // If no server IDs found (empty config file or only invalid keys)
      if (serverIds.length === 0) {
        installedServers.set([])
        originalServers.set([])
        pageInfo.set({
          has_next_page: false,
          end_cursor: null,
          total_items: 0,
        })
        isLoading.set(false)
        return
      }

      // Invoke the Rust command to get data for the installed servers
      const response = await invoke<{ cards: MCPCard[]; page_info: PageInfo }>("get_installed_mcp_data", {
        serverIds: serverIds,
        cursorId: null,
        searchTerm: searchTerm.trim() || null, // Pass search term if it exists
      })

      installedServers.set(response.cards)
      originalServers.set(response.cards) // Store original data

      // Update pageInfo with the actual number of installed servers, excluding the fallback server
      const actualTotalItems = response.page_info.total_items
      pageInfo.set({
        ...response.page_info,
        total_items: actualTotalItems,
      })
    } catch (error: any) {
      errorMessage.set(`Failed to load installed MCPs: ${error.message || error}`)
      installedServers.set([])
      originalServers.set([])
      pageInfo.set(null)
    } finally {
      isLoading.set(false)
    }
  }

  // ✅ Added delete event handler
  function handleCardDeleted(event: CustomEvent<{ id: number }>) {
    const deletedId = event.detail.id
    // Update installedServers store
    installedServers.update((servers) => servers.filter((server) => server.id !== deletedId))
    // Also update originalServers
    originalServers.update((servers) => servers.filter((server) => server.id !== deletedId))
    // Also update page information (optional)
    pageInfo.update((info) => {
      if (!info) return null
      return { ...info, total_items: Math.max(0, info.total_items - 1) }
    })
  }

  // Simple debounce utility function implemented directly
  function createDebounce(func: Function, wait: number) {
    let timeout: ReturnType<typeof setTimeout> | null = null

    return function (...args: any[]) {
      if (timeout) clearTimeout(timeout)

      timeout = setTimeout(() => {
        func(...args)
        timeout = null
      }, wait)
    }
  }

  // Implement search function
  async function performSearch() {
    const term = searchTerm.trim()

    if (term === "") {
      // If search term is empty, display original data
      installedServers.set($originalServers)
      isSearching.set(false)
      return
    }

    isSearching.set(true)

    try {
      // Request search directly from Rust backend
      const results = await invoke<LocalMCPServer[]>("search_local_mcp_servers", { searchTerm: term })

      // Extract server IDs
      const matchingIds = results.map((server) => server.id)

      // Filter items from original data that have matching IDs
      if (matchingIds.length > 0) {
        installedServers.set($originalServers.filter((server) => matchingIds.includes(server.id)))
      } else {
        // If no search results, set an empty array
        installedServers.set([])
      }
    } catch (error) {
      console.error("[Installed-MCP] Search error:", error)
      // Maintain original data on error
      installedServers.set($originalServers)
    } finally {
      isSearching.set(false)
    }
  }

  // Create debounced search function (300ms)
  const handleSearch = createDebounce(performSearch, 300)

  // Call search function when search term changes
  $: {
    if (searchTerm !== undefined) {
      handleSearch()
    }
  }

  // Data loading - execute immediately and also in onMount
  // This way, data starts loading immediately when the page is entered
  ;(async () => {
    await loadInstalledMCPs()
  })()

  // Execute again in onMount (after the component is fully mounted)
  onMount(() => {
    return loadInstalledMCPs()
  })
</script>

<div class="container mx-auto pb-4">
  <!-- Top header area (not fixed) - background color same as page background -->
  <div class="pt-1 pb-2 border-b border-primary-content/10">
    <div class="flex flex-col sm:flex-row justify-between items-center w-full px-4">
      <h1 class="text-2xl font-bold text-center sm:text-left sm:mr-auto">Installed MCP Servers ({searchTerm.trim() ? $installedServers.length : $pageInfo ? $pageInfo.total_items : 0})</h1>

      <!-- Search UI -->
      <div class="relative w-full max-w-xs mx-auto sm:mx-0 sm:w-64 mt-2 sm:mt-0 sm:ml-auto">
        <input type="text" bind:value={searchTerm} placeholder="Search servers..." class="input input-bordered w-full pr-10" />
        {#if $isSearching}
          <span class="loading loading-spinner loading-xs absolute right-3 top-3"></span>
        {:else}
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="w-5 h-5 absolute right-3 top-3"
          >
            <circle cx="11" cy="11" r="8"></circle>
            <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
          </svg>
        {/if}
      </div>
    </div>
  </div>

  <!-- Add padding to content area -->
  <div class="mt-3 px-4">
    {#if $isLoading}
      <div class="text-center py-10">
        <div class="flex flex-col items-center gap-4">
          <span class="loading loading-spinner loading-lg text-primary"></span>
          <p>Loading installed MCP servers...</p>
        </div>
      </div>
    {:else if $errorMessage}
      <div class="text-center py-10 text-red-600">
        <p>Error: {$errorMessage}</p>
      </div>
    {:else if $installedServers.length > 0}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each $installedServers as server (server.id)}
          <MCPCardComponent id={server.id} title={server.title} description={server.description} url={server.url} stars={server.stars} mode="installed" on:deleted={handleCardDeleted} />
        {/each}
      </div>
    {:else}
      <div class="text-center py-10 text-gray-500">
        {#if searchTerm.trim()}
          <p>No matching MCP servers found for "{searchTerm}".</p>
          <button
            class="btn btn-sm btn-outline mt-3"
            on:click={() => {
              searchTerm = ""
              // Restore original data directly (debounce not applied)
              installedServers.set($originalServers)
              isSearching.set(false)
            }}
          >
            Clear search
          </button>
        {:else}
          <p>No MCP servers are currently installed.</p>
          <p>Go to the 'MCP List' tab to browse and install servers.</p>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  /* Add any specific styles for this page if needed */
  .container {
    max-width: 1200px; /* Optional: Set a max width */
  }

  /* Scrollbar styles */
  :global(::-webkit-scrollbar) {
    width: 8px;
  }

  :global(::-webkit-scrollbar-track) {
    background: transparent;
  }

  :global(::-webkit-scrollbar-thumb) {
    background: #888;
    border-radius: 4px;
  }

  :global(::-webkit-scrollbar-thumb:hover) {
    background: #555;
  }
</style>
