<script lang="ts">
  import { onMount } from "svelte"
  import { invoke } from "@tauri-apps/api/core"
  // readTextFile is no longer needed from plugin-fs
  import MCPCardComponent from "$lib/components/mcp-card.svelte"
  // Assuming types are defined in mcp-api.ts or exported via an index file
  // Let's assume they are directly in mcp-api.ts for now
  import type { MCPCard, PageInfo } from "$lib/data/mcp-api" 
  import { writable } from 'svelte/store'

  // State variables
  let installedServers = writable<MCPCard[]>([])
  let pageInfo = writable<PageInfo | null>(null)
  let isLoading = writable(true)
  let errorMessage = writable<string | null>(null)

  // Function to load installed MCP data
  async function loadInstalledMCPs() {
    isLoading.set(true)
    errorMessage.set(null)

    try {
      // Get the config file content directly from the backend
      const configContent = await invoke<string>("read_mcplink_config_content")
      console.log("[Installed-MCP] mcplink_desktop_config content from backend:", configContent)

      let serverIds: number[] = []

      try {
        // Parse the content string
        const configData = JSON.parse(configContent)
        serverIds = Object.keys(configData).map(Number).filter(id => !isNaN(id))
        console.log("[Installed-MCP] Found server IDs:", serverIds)
      } catch (parseError: any) {
          // Handle parsing error, e.g., if the content is not valid JSON (though Rust returns "{}" if not found)
          console.error(`[Installed-MCP] Failed to parse config content: ${configContent}. Error:`, parseError)
          // Depending on requirements, could treat as empty or show error
          errorMessage.set("Failed to parse local MCP configuration.")
          installedServers.set([])
          isLoading.set(false)
          return; 
      }

      // If no server IDs found (empty config file or only invalid keys)
      if (serverIds.length === 0) {
        installedServers.set([])
        pageInfo.set({
          hasNextPage: false,
          endCursor: null,
          totalItems: 0,
        })
        isLoading.set(false)
        console.log("[Installed-MCP] No installed servers found in config.")
        return
      }

      // Invoke the Rust command to get data for the installed servers
      const response = await invoke<{ cards: MCPCard[]; pageInfo: PageInfo }>(
        "get_installed_mcp_data",
        { serverIds: serverIds, cursorId: null } 
      )

      console.log("[Installed-MCP] API Response:", response)
      installedServers.set(response.cards)
      pageInfo.set(response.pageInfo)

    } catch (error: any) {
      console.error("[Installed-MCP] Error loading installed MCPs:", error)
      // Check if the error is the specific 'not found' signal from Rust if you implemented it
      // if (error === "mcplink_config_not_found") { ... handle differently ... }
      errorMessage.set(`Failed to load installed MCPs: ${error.message || error}`)
      installedServers.set([])
      pageInfo.set(null)
    } finally {
      isLoading.set(false)
    }
  }

  // Load data on mount
  onMount(loadInstalledMCPs)
</script>

<div class="container mx-auto p-4">
  <h1 class="text-2xl font-bold mb-4">Installed MCP Servers</h1>

  {#if $isLoading}
    <div class="text-center py-10">
      <p>Loading installed servers...</p>
      <!-- Optional: Add a spinner -->
    </div>
  {:else if $errorMessage}
    <div class="text-center py-10 text-red-600">
      <p>Error: {$errorMessage}</p>
    </div>
  {:else if $installedServers.length > 0}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each $installedServers as server (server.id)}
        <MCPCardComponent 
          id={server.id} 
          title={server.title} 
          description={server.description} 
          url={server.url} 
          stars={server.stars}
          mode="installed" 
        />
      {/each}
    </div>
    {#if $pageInfo}
      <p class="text-center mt-4 text-gray-500">Total installed servers: {$pageInfo.totalItems}</p>
      <!-- Note: No pagination/infinite scroll implemented here as we fetch all installed servers at once -->
    {/if}
  {:else}
    <div class="text-center py-10 text-gray-500">
      <p>No MCP servers are currently installed.</p>
      <p>Go to the 'MCP List' tab to browse and install servers.</p>
    </div>
  {/if}
</div>

<style>
 /* Add any specific styles for this page if needed */
 .container {
   max-width: 1200px; /* Optional: Set a max width */
 }
</style> 