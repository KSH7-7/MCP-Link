<script lang="ts">
  import { invoke } from "@tauri-apps/api/core"
  import ConfirmModal from "$lib/components/confirm-modal.svelte"
  import { showSuccess, showError } from "$lib/stores/toast"
  import NotificationTest from './notification-test.svelte'
  
  let configPath = ""
  let isResetting = false
  let showResetConfirmModal = false
  let showNotificationTest = false

  // Function to show reset confirmation modal
  function showResetConfirmation() {
    showResetConfirmModal = true
  }

  // Function to reset settings
  async function resetSettings() {
    isResetting = true
    try {
      await invoke("reset_mcp_settings")
      showSuccess("Settings have been reset successfully. The application will now restart.", 3000, "bottom-center")
      // Restart Claude Desktop
      await invoke("restart_claude_desktop")
    } catch (error) {
      console.error("Failed to reset settings:", error)
      showError(`Failed to reset settings: ${error}`, 5000, "bottom-center")
    } finally {
      isResetting = false
    }
  }
</script>

<div class="p-8 max-w-2xl mx-auto">
  <h1 class="text-2xl font-bold mb-6">Settings</h1>
  
  <div class="flex justify-end mb-4">
    <button 
      class="btn btn-primary btn-sm"
      on:click={() => showNotificationTest = !showNotificationTest}
    >
      {showNotificationTest ? 'Hide Notification Test' : 'Show Notification Test'}
    </button>
  </div>
  
  {#if showNotificationTest}
    <NotificationTest />
    <div class="divider my-6"></div>
  {/if}
  
  <div class="bg-white rounded-lg p-6 shadow-sm border border-gray-200 text-center">
    <h2 class="text-lg font-semibold mb-4 text-center">Reset Configurations</h2>
    <p class="text-gray-600 mb-4">
      This function will completely reset the configuration files<br>
      installed on your device.<br> 
      Please use only when absolutely necessary.
    </p>
    <button 
      class="btn btn-error" 
      on:click={showResetConfirmation}
      disabled={isResetting}
    >
      {#if isResetting}
        <span class="loading loading-spinner loading-sm"></span> Resetting...
      {:else}
        Reset Settings
      {/if}
    </button>
  </div>
</div>

<!-- Reset confirmation modal -->
<ConfirmModal
  isOpen={showResetConfirmModal}
  title="Reset Settings"
  message={"Warning: This will reset all MCP server settings\nexcept for the fallback server.\nClaude will restart.\nAre you sure you want to continue?"}
  type="error"
  okLabel="Reset"
  cancelLabel="Cancel"
  on:confirm={resetSettings}
  on:cancel={() => showResetConfirmModal = false}
/>
