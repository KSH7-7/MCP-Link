<script lang="ts">
  import { Star, Github, Trash2, Skull, AlertTriangle, AlertCircle, ShieldCheck, HelpCircle } from "lucide-svelte"
  import { invoke } from "@tauri-apps/api/core"
  import { goto } from "$app/navigation"
  import { createEventDispatcher } from "svelte"
  import ConfirmModal from "./confirm-modal.svelte"
  import { showSuccess, showError } from "$lib/stores/toast"

  // Create event dispatcher
  const dispatch = createEventDispatcher()
  
  // Modal state
  let showDeleteModal = false

  // Define props needed for the card component
  export let id: number
  export let title: string
  export let description: string
  export let url: string = "" // GitHub URL
  export let stars: number = 0 // GitHub Stars count
  export let installed = false // Prop to indicate if the MCP server is installed
  export let securityRank: "CRITICAL" | "HIGH" | "MODERATE" | "LOW" | "UNRATE" = "UNRATE" // Security rank status
  export let onClick: (() => void) | undefined = undefined // Click handler (optional)
  export let className: string = "" // Additional CSS class (optional)
  export let variant: "default" | "primary" | "accent" = "default" // Card style variant
  export let maxDescLength: number = 150 // Maximum description length
  export let mode = "" // 'installed' or other. Indicates current page context.

  // Format star count (display K if 1000 or more)
  function formatStars(count: number): string {
    if (count >= 1000) {
      return `${Math.round(count / 100) / 10}k`
    }
    return count.toString()
  }

  // Truncate description text
  function truncateDescription(text: string, maxLength: number): string {
    if (text.length <= maxLength) return text
    return text.substring(0, maxLength) + "..."
  }

  // Final description to display
  const displayDescription = truncateDescription(description, maxDescLength)
  
  // use securityRank value provided by backend
  // no longer use random ID-based value
  const assignedSecurityRank = securityRank

  // Function to open GitHub link
  async function openGitHub(urlToOpen: string) {

    if (!urlToOpen) return

    // Add https:// if protocol is missing
    const finalUrl = urlToOpen.startsWith("http") ? urlToOpen : `https://${urlToOpen}`


    try {
      // Use browser's default window.open method
      window.open(finalUrl, "_blank")
    } catch (error) {
      // Notify user on failure
      alert(`Could not open the link. Please navigate manually: ${finalUrl}`)
    }
  }

  // Function to navigate to detail page
  function goToDetail() {
    const params = new URLSearchParams({
      id: id.toString(),
      title,
      description,
      url: url || "",
      stars: stars.toString(),
      mode: mode === "installed" ? "edit" : "install",
      referrer: window.location.pathname, // Save current path as referrer
    })

    // Ensure we're using a clean navigation method that doesn't cause problems
    // with the tab highlighting system
    window.location.href = `/detail?${params.toString()}`
  }

  interface MCPServerConfigTypeScript {
    command: string
    args?: string[] | null
    env?: Record<string, any> | null // Corresponds to serde_json::Map<String, Value>
  }

  // Show delete modal
  function openDeleteModal() {
    showDeleteModal = true;
  }
  
  // Handle delete confirmation
  async function handleDeleteConfirm() {
    // User clicked "Yes" - proceed with deletion
    let errorMessage = ""
    try {
      // 1. Remove config from backend
      await invoke("remove_mcp_server_config", {
        serverName: title,
      })

      // 2. Dispatch 'deleted' event (request GUI update)
      dispatch("deleted", { id: id })

      // 3. Restart Claude Desktop (after event dispatch)
      await invoke("restart_claude_desktop")
      
      // Show success toast notification
      showSuccess("MCP successfully deleted", 3000, "bottom-center")
    } catch (err: any) {
      // Specify err type as any or Error
      errorMessage = `error occurred: ${err.message || err}`

      showError(`Error: ${errorMessage}`, 5000, "bottom-center")
    }
  }
  
  // Use original handler as a trigger for the modal
  function handleComplete() {
    openDeleteModal();
  }
</script>

<!-- Reusable MCP card component -->
<div class="card card-border w-full shadow-sm {variant === 'default' ? 'bg-base-100' : variant === 'primary' ? 'bg-primary text-primary-content' : 'bg-accent text-accent-content'} {className} {onClick ? 'cursor-pointer' : ''}" on:click={onClick}>
  <div class="card-body h-[140px] p-4">
    <!-- split main card content into left and right -->
    <div class="flex h-full gap-2">
      <!-- left content: title and description -->
      <div class="w-[85%] flex flex-col">
        <!-- title area -->
        <div class="flex items-center gap-2 mb-1">
          <h2 class="card-title text-lg truncate">{title}</h2>
          <!-- Security rank status icon -->
          {#if assignedSecurityRank === "CRITICAL"}
            <span class="tooltip" data-tip="CRITICAL">
              <Skull class="text-error" size={16} />
            </span>
          {:else if assignedSecurityRank === "HIGH"}
            <span class="tooltip" data-tip="HIGH">
              <AlertTriangle class="text-warning" size={16} />
            </span>
          {:else if assignedSecurityRank === "MODERATE"}
            <span class="tooltip" data-tip="MODERATE">
              <AlertCircle style="color: oklch(0.7952 0.1617 86.05)" size={16} />
            </span>
          {:else if assignedSecurityRank === "LOW"}
            <span class="tooltip" data-tip="LOW">
              <ShieldCheck class="text-success" size={16} />
            </span>
          {:else}
            <span class="tooltip" data-tip="UNRATE">
              <HelpCircle class="text-neutral" size={16} />
            </span>
          {/if}
        </div>
        
        <!-- description area - fixed height and line clamp -->
        <div class="flex-grow overflow-hidden">
          <p class="text-sm line-clamp-3">{displayDescription}</p>
        </div>
      </div>
      
      <!-- right content: star/github icon and buttons -->
      <div class="w-[15%] flex flex-col justify-between">
        <!-- top: star count and github link (horizontal alignment) -->
        <div class="flex justify-end items-center gap-2">
          <!-- Star icon and count -->
          {#if stars > 0}
            <div class="flex items-center gap-1 tooltip tooltip-left" data-tip="{stars} stars">
              <Star class="text-yellow-400" size={16} />
              <span class="text-sm">{formatStars(stars)}</span>
            </div>
          {/if}

          <!-- GitHub link -->
          {#if url}
            <!-- If URL exists: clickable link -->
            <button on:click|stopPropagation={() => openGitHub(url)} class="text-gray-600 hover:text-black focus:outline-none flex items-center">
              <span class="tooltip" data-tip="Visit GitHub">
                <Github size={16} />
              </span>
            </button>
          {:else}
            <!-- If URL does not exist: display as disabled -->
            <span class="text-gray-400 opacity-40 flex items-center">
              <Github size={16} />
            </span>
          {/if}
        </div>
        
        <!-- bottom: button area - always fixed at bottom (horizontal alignment) -->
        <div class="flex justify-end mt-auto">
          {#if mode === "installed"}
            <div class="flex gap-1">
              <button on:click|stopPropagation={goToDetail} class="btn btn-xs btn-secondary px-3">Edit</button>
              <button on:click|stopPropagation={handleComplete} class="btn btn-xs btn-natural btn-square">
                <span class="tooltip tooltip-top" data-tip="Delete">
                  <Trash2 size={16} class="text-error" />
                </span>
              </button>
            </div>
          {:else if installed}
            <button on:click|stopPropagation={goToDetail} class="btn btn-xs btn-secondary px-3">Installed</button>
          {:else}
            <button on:click|stopPropagation={goToDetail} class="btn btn-xs btn-secondary px-3">Install</button>
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>

<!-- Delete confirmation modal -->
<ConfirmModal
  isOpen={showDeleteModal}
  title="Delete MCP Server"
  message={`For quick application,
  Claude will restart with the deletion
Would you like to delete?`}
  type="warning"
  okLabel="Delete"
  cancelLabel="Cancel"
  on:confirm={handleDeleteConfirm}
  on:cancel={() => showDeleteModal = false}
/>

<style>
  /* Remove bold from tooltips */
  :global(.tooltip::before) {
    font-weight: normal !important;
  }
</style>