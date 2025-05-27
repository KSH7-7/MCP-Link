<script lang="ts">
  import { onMount } from "svelte"
  import { Star, Github, ArrowLeft, Skull, AlertTriangle, AlertCircle, ShieldCheck, HelpCircle } from "lucide-svelte"
  import { fetchMCPCardDetail } from "$lib/data/mcp-api"
  import { invoke } from "@tauri-apps/api/core"
  import { goto } from "$app/navigation"
  import { page } from "$app/stores"
  import ConfirmModal from "$lib/components/confirm-modal.svelte"
  import { showSuccess, showError } from "$lib/stores/toast"
  import { listen } from "@tauri-apps/api/event"

  // Get MCP information from URL
  let id: number = 0
  let title: string = ""
  
  // Modal state
  let showConfirmModal = false
  let description: string = ""
  let url: string = ""
  let stars: number = 0
  let securityRank: "CRITICAL" | "HIGH" | "MODERATE" | "LOW" | "UNRATE" = "UNRATE"
  // Detailed settings binding
  let args: string[] = []
  let env: Record<string, any> = {}
  let command: string = ""
  let loading = true // Initial loading state true
  let error = ""
  let mode: "install" | "edit" = "install" // Add mode variable and set default value
  let referrer: string = "/" // Default referrer value is home

  // Format star count (display K if 1000 or more)
  function formatStars(count: number): string {
    if (count >= 1000) {
      return `${Math.round(count / 100) / 10}k`
    }
    return count.toString()
  }

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

  // Change goBack function to use goto
  function goBack() {
    goto(referrer)
  }

  // Create star rating array
  let starsArray: number[] = []

  // Convert array or object to string (for placeholder)
  function formatValueForPlaceholder(value: any): string {
    if (Array.isArray(value)) {
      return value.join(", ")
    }
    if (typeof value === "object" && value !== null) {
      return Object.entries(value)
        .map(([k, v]) => `${k}: ${v}`)
        .join(", ")
    }
    return String(value || "")
  }

  // Load MCP detail data
  async function loadMCPDetail(mcpId: number) {
    if (!mcpId) {
      error = "ID is missing."
      loading = false
      return
    }

    loading = true
    error = ""

    try {
      // Pass title along to check installation status and fetch from config if installed
      const detail = await fetchMCPCardDetail(mcpId, title)

      // Update basic information (optional, as already fetched from URL)
      title = detail.title || title
      description = detail.description || description
      url = detail.url || url
      stars = detail.stars || stars
      securityRank = detail.securityRank || "UNRATE"

      // Update detailed settings - do not show fields if they are not present in the data received from the crawler server
      if (detail.args) args = detail.args
      // environment variables are only displayed if they exist in the crawler server data
      if (detail.env && Object.keys(detail.env).length > 0) env = detail.env
      else env = {} // if there is no environment variable data, set an empty object to display nothing
      if (detail.command) command = detail.command

      // Update star rating array
      const starCount = Math.min(Math.round(stars / 1000), 5)
      starsArray = Array(5)
        .fill(0)
        .map((_, i) => (i < starCount ? 1 : 0))
      
      // Check if the server is already installed
      try {
        const isInstalled = await invoke<boolean>("is_mcp_server_installed", { serverName: title })
        // Update button text based on installation status
        if (isInstalled && mode === "install") {
          mode = "edit" // Change to edit mode if already installed
        }
      } catch (err) {
      }
    } catch (err: any) {
      error = `Failed to load detail data: ${err.message || err.toString()}`
      // Use example data (optional - for providing alternative UI on error)
      if (mcpId === 16) {
        // Sentry server example
        args = ["mcp-server-sentry", "--auth-token", "YOUR_SENTRY_TOKEN"]
        env = { SENTRY_DSN: "https://example.sentry.io/123456" }
        command = "uvx"
      }
    } finally {
      loading = false
    }
  }


  onMount(async () => {
    // Get parameters from URL
    const params = new URLSearchParams(window.location.search)
    id = parseInt(params.get("id") || "0")

    // Read mode parameter
    const modeParam = params.get("mode")
    if (modeParam === "edit" || modeParam === "install") {
      mode = modeParam
    }

    // Read referrer parameter - save previous page information
    const refParam = params.get("referrer")
    if (refParam) {
      referrer = refParam
    } else {
      // Use document.referrer if available, otherwise set default based on mode
      referrer = mode === "edit" ? "/Installed-MCP" : "/MCP-list"
    }

    // Get basic information from URL parameters
    title = params.get("title") || "Untitled"
    description = params.get("description") || "No description"
    url = params.get("url") || ""
    stars = parseInt(params.get("stars") || "0")
    
    // display notification based on current mode in the detail page
    try {
      if (typeof window !== "undefined") {
        // determine the path of the tab to activate based on the current mode
        const activeTab = mode === "edit" ? "/Installed-MCP" : "/MCP-list";
        
        // display notification (update activeTabPath)
        // slightly delay to ensure all components are mounted
        setTimeout(() => {
          window.dispatchEvent(new CustomEvent('navigate-to-event', { 
            detail: { path: activeTab }
          }));
        }, 10);
      }
    } catch (err) {
    }

    // Create star rating array (max 5)
    const starCount = Math.min(Math.round(stars / 1000), 5)
    starsArray = Array(5)
      .fill(0)
      .map((_, i) => (i < starCount ? 1 : 0))

    // If ID exists, load detail data
    if (id) {
      loadMCPDetail(id)
    } else {
      error = "Invalid access: ID is missing."
      loading = false
    }
    
  })
</script>

<div class="p-8 max-w-5xl mx-auto min-h-screen">
  <div class="bg-white rounded-lg shadow-sm p-6 relative">
    <!-- Back button -->
    <button class="absolute top-4 right-4 btn btn-sm btn-ghost gap-1 flex items-center" on:click={goBack}>
      <ArrowLeft size={20} />
      <span>Back</span>
    </button>

    <!-- Title and version -->
    <div class="mb-8 border-b pb-4">
      <div class="flex items-center gap-3">
        <h1 class="text-2xl font-bold">{title}</h1>
        
        <!-- Security rank status icon -->
        {#if securityRank === "CRITICAL"}
          <span class="tooltip" data-tip="CRITICAL">
            <Skull class="text-error" size={20} />
          </span>
        {:else if securityRank === "HIGH"}
          <span class="tooltip" data-tip="HIGH">
            <AlertTriangle class="text-warning" size={20} />
          </span>
        {:else if securityRank === "MODERATE"}
          <span class="tooltip" data-tip="MODERATE">
            <AlertCircle style="color: oklch(0.7952 0.1617 86.05)" size={20} />
          </span>
        {:else if securityRank === "LOW"}
          <span class="tooltip" data-tip="LOW">
            <ShieldCheck class="text-success" size={20} />
          </span>
        {:else}
          <span class="tooltip" data-tip="UNRATE">
            <HelpCircle class="text-neutral" size={20} />
          </span>
        {/if}
        
        <div class="flex items-center gap-1 tooltip" data-tip="{stars} stars">
          <Star class="text-yellow-400 fill-yellow-400" size={22} />
          <span class="text-gray-600 font-medium">{stars > 0 ? formatStars(stars) : "0"}</span>
        </div>
        {#if url && url.includes("github.com")}
          <button on:click={() => openGitHub(url)} class="text-gray-500 hover:text-gray-700 transition-colors flex items-center">
            <span class="tooltip" data-tip="Visit GitHub">
              <Github size={20} />
            </span>
          </button>
        {/if}
      </div>
    </div>

    {#if loading}
      <div class="flex justify-center p-8">
        <div class="loading loading-spinner loading-lg"></div>
      </div>
    {:else if error}
      <div class="alert alert-error">
        <span>{error}</span>
        {#if id}
          <div class="mt-2">
            <button class="btn btn-sm btn-outline" on:click={() => loadMCPDetail(id)}>Retry</button>
          </div>
        {/if}
      </div>
    {:else}
      <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
        <!-- Left column: MCP basic information -->
        <div>
          <div class="mb-4"></div>

          <div>
            <h2 class="text-xl font-semibold mb-4">Description</h2>
            <div class="prose max-w-none">
              <p>{description}</p>
            </div>
          </div>
        </div>

        <!-- Right column: Required settings -->
        <div>
          <h2 class="text-xl font-semibold mb-4">Required Settings</h2>
          <div class="mb-6">
            <div class="form-control mb-4">
              <div class="form-control">
                <label class="label">
                  <span class="label-text">Command</span>
                </label>
                <input type="text" class="input input-bordered" bind:value={command} />
              </div>
            </div>

            <div class="form-control mb-4">
              <div class="form-control">
                <label class="label">
                  <span class="label-text">Arguments</span>
                </label>
                <!-- First, get the string as a variable and bind it -->
                {#if args}
                  <textarea 
                    class="textarea textarea-bordered resize-none" 
                    rows="2" 
                    value={formatValueForPlaceholder(args)} 
                    on:input={(e) => {
                      // Convert text input value to an array
                      try {
                        args = e.currentTarget.value.split(',').map(s => s.trim()).filter(s => s);
                      } catch (err) {
                      }
                    }}
                  ></textarea>
                {:else}
                  <textarea 
                    class="textarea textarea-bordered resize-none" 
                    rows="2" 
                    on:input={(e) => {
                      try {
                        args = e.currentTarget.value.split(',').map(s => s.trim()).filter(s => s);
                      } catch (err) {
                      }
                    }}
                  ></textarea>
                {/if}
                <label class="label pt-0 mt-0">
                  <span class="label-text-alt text-xs text-gray-500">Comma-separated values</span>
                </label>
              </div>
            </div>

            <!-- Display only if environment variables exist -->
            {#if env && Object.keys(env).length > 0}
              <div class="form-control mb-4">
                <div class="form-control">
                  <label class="label">
                    <span class="label-text">Environment Variables</span>
                  </label>
                  <textarea 
                    class="textarea textarea-bordered resize-none" 
                    rows="2" 
                    value={formatValueForPlaceholder(env)} 
                    on:input={(e) => {
                      // Convert text input value to an environment variable object
                      try {
                        const envObj: Record<string, any> = {};
                        e.currentTarget.value.split(',').forEach(item => {
                          const parts = item.split(':');
                          if (parts.length >= 2) {
                            const key = parts[0].trim();
                            const value = parts.slice(1).join(':').trim(); // Handle values that may contain colons
                            if (key) {
                              envObj[key] = value;
                            }
                          }
                        });
                        env = envObj;
                      } catch (err) {
                      }
                    }}
                  ></textarea>
                  <label class="label pt-0 mt-0">
                    <span class="label-text-alt text-xs text-gray-500">Key:value pairs, separated by commas</span>
                  </label>
                </div>
              </div>
            {/if}

            {#if !command && (!args || args.length === 0) && (!env || Object.keys(env).length === 0)}
              <p class="text-gray-500">No required settings.</p>
            {/if}
          </div>
        </div>
      </div>
    {/if}

    <!-- Bottom button area -->
    <div class="mt-8 flex justify-end gap-2 border-t pt-4">
      <button
        class="btn btn-sm btn-secondary"
        on:click={() => {
          // Open confirmation modal
          showConfirmModal = true;
        }}
        disabled={loading || !!error}
      >
        {mode === "edit" ? "Save Changes" : "Install"}
      </button>
    </div>
  </div>
</div>

<!-- Confirmation modal -->
<ConfirmModal
isOpen={showConfirmModal}
  title={mode === "edit" ? "Update MCP Configuration" : "Install MCP Server"}
  message={mode === "edit" 
    ? `For quick application,
    Claude will restart with the update.
    Would you like to update?`
    : `For quick application,
    Claude will restart with the installation.
    Would you like to install?`
  }
  type="warning"
  okLabel={mode === "edit" ? "Update" : "Install"}
  cancelLabel="Cancel"
  on:confirm={async () => {
    // Configure server settings
    const serverConfig = {
      command: command || "",
      args: args || [],
      env: env || {},
      cwd: null,
    };
    
    try {
      // Installation or save operation
      await invoke("add_mcp_server_config", {
        serverId: id,
        serverName: title,
        serverConfig: serverConfig,
      });
      
      // Restart Claude Desktop
      await invoke("restart_claude_desktop");
      
      if (mode === "edit") {
        showSuccess("MCP configuration has been updated!", 3000, "bottom-center");
      } else {
        showSuccess("MCP has been installed!", 3000, "bottom-center");
      }
      goBack(); // Automatically go back to previous page
    } catch (err) {
      showError(`Error during MCP ${mode === "edit" ? "update" : "installation"}: ${err}`, 5000, "bottom-center");
    }
  }}
  on:cancel={() => showConfirmModal = false}
/>