<script lang="ts">
  import { invoke } from "@tauri-apps/api/core" // Tauri invoke API
  import { goto } from "$app/navigation" // Import the goto function.

  let selectedDirectory = "" // Keep for potential future use, but dialog logic is disabled
  let errorMessage = "" // For displaying errors

  // Interface matching the Rust MCPServerConfig struct (Restored)
  interface MCPServerConfigTypeScript {
    command: string
    args?: string[] | null
    env?: Record<string, any> | null // Corresponds to serde_json::Map<String, Value>
    cwd?: string | null
  }

  async function handleComplete() {
    errorMessage = ""

    try {
      // Use the new ensure_config_files function that creates any missing config files
      await invoke("ensure_config_files")

      // Restart Claude Desktop
      await invoke("restart_claude_desktop")

      // Navigate to the Installed-MCP page
      await goto("/Installed-MCP")
    } catch (err) {
      errorMessage = `Error during setup: ${err}`
    }
  }
</script>

<div class="flex items-center justify-center min-h-screen bg-slate-100">
  <div class="bg-white p-8 rounded-[20px] shadow-md w-full max-w-md">
    <h1 class="text-2xl font-bold text-center mb-6 text-gray-700">Setting Announcement</h1>
    <p class="text-gray-600 mb-6 text-center">Connect the app to the Claude app and restart Claude. If you agree, please press the button to connect and restart.</p>

    {#if errorMessage}
      <p class="text-red-500 text-sm text-center mb-4">{errorMessage}</p>
    {/if}

    <div class="flex justify-center">
      <button
        on:click={handleComplete}
        class="w-1/2 bg-green-500 hover:bg-green-600 text-white font-bold py-3 px-4 rounded-[10px] focus:outline-none focus:shadow-outline transition duration-150 ease-in-out"
      >
        Agree
      </button>
    </div>
  </div>
</div>

<style>
  /* You can add page-specific styles here if needed, though Tailwind handles most of it */
</style>
