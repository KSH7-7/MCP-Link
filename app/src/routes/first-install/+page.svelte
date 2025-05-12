<script lang="ts">
  import { invoke } from "@tauri-apps/api/core" // Tauri invoke API

  let selectedDirectory = "" // Keep for potential future use, but dialog logic is disabled
  let errorMessage = "" // For displaying errors

  // Interface matching the Rust MCPServerConfig struct
  interface MCPServerConfigTypeScript {
    command: string
    args?: string[] | null
    env?: Record<string, any> | null // Corresponds to serde_json::Map<String, Value>
  }

  async function handleComplete() {
    errorMessage = ""

    const serverNameForEntry = "test"
    const configData: MCPServerConfigTypeScript = {
      command: "npx",
      args: ["-y", "@notionhq/notion-mcp-server"],
      env: {
        OPENAPI_MCP_HEADERS: '{"Authorization": "Bearer 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", "Notion-Version": "2022-06-28" }',
      },
    }

    try {
      await invoke("add_mcp_server_config", {
        serverName: serverNameForEntry,
        serverConfig: configData,
      })

      // 2. Restart Claude Desktop
      await invoke("restart_claude_desktop")
    } catch (err) {
      errorMessage = `error occurred: ${err}`
    }
  }
</script>

<div class="flex items-center justify-center min-h-screen bg-slate-100">
  <div class="bg-white p-8 rounded-[20px] shadow-md w-full max-w-md">
    <h1 class="text-2xl font-bold text-center mb-6 text-gray-700">Setting announcement</h1>
    <p class="text-gray-600 mb-6 text-center">Connect the app to the claude app and restart claude. If you agree, please push the button, then connect and restart.</p>

    {#if errorMessage}
      <p class="text-red-500 text-sm text-center mb-4">{errorMessage}</p>
    {/if}

    <button
      on:click={handleComplete}
      class="w-1/2 bg-green-500 hover:bg-green-600 text-white font-bold py-3 px-4 rounded-[10px] focus:outline-none focus:shadow-outline transition duration-150 ease-in-out"
    >
      동의
    </button>
  </div>
</div>

<style>
  /* You can add page-specific styles here if needed, though Tailwind handles most of it */
</style>
