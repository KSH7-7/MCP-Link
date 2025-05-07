<script lang="ts">
  import "../app.css";
  import { onMount } from 'svelte';
  import type { Window } from '@tauri-apps/api/window';
  import { Presentation, Cog, Minus, X, Square, Settings } from 'lucide-svelte';
  
  // Tauri window object to save
  let tauriWindow: Window | null = null;
  // OS platform detection
  let platform: string = 'unknown';
  
  onMount(async () => {
    // Check if Tauri API exists
    if (typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window)) {
      try {
        // Get current window - Tauri v2 API
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        tauriWindow = getCurrentWindow();
        
        // OS type detection
        const { platform: getPlatform } = await import('@tauri-apps/plugin-os');
        platform = await getPlatform();
      } catch (error) {
        console.error('error: get current window or platform', error);
      }
    } else {
      // If running in the browser (use Navigator API)
      if (typeof navigator !== 'undefined') {
        if (navigator.userAgent.indexOf('Win') !== -1) platform = 'win32';
        else if (navigator.userAgent.indexOf('Mac') !== -1) platform = 'darwin';
        else if (navigator.userAgent.indexOf('Linux') !== -1) platform = 'linux';
      }
    }
  });
  
  // Minimize button handler
  async function minimizeWindow() {
    if (tauriWindow) {
      try {
        await tauriWindow.minimize();
      } catch (error) {
        console.error('error: minimize window', error);
      }
    }
  }
  
  // Maximize button handler
  async function maximizeWindow() {
    if (tauriWindow) {
      try {
        const isMaximized = await tauriWindow.isMaximized();
        if (isMaximized) {
          await tauriWindow.unmaximize();
        } else {
          await tauriWindow.maximize();
        }
      } catch (error) {
        console.error('error: maximize window', error);
      }
    }
  }
  
  // Hide to tray button handler
  async function hideToTray() {
    if (tauriWindow) {
      try {
        await tauriWindow.hide();
      } catch (error) {
        console.error('error: hide to tray', error);
      }
    }
  }

  // Current active tab
  let activeTab = "/";

  // Tab information and colors
  const tabs = [
    { path: "/Installed-MCP", name: "Installed MCP", icon: Presentation, color: "#eef2ff", hoverColor: "#eef2ff", bgColor: "#eef2ff" },
    { path: "/MCP-list", name: "MCP List", icon: Cog, color: "#ecfdf5", hoverColor: "#ecfdf5", bgColor: "#ecfdf5" },
  ];
  
  // Settings tab (placed on the right)
  const settingsTab = { path: "/settings", name: "Settings", icon: Settings, color: "#f3f4f6", hoverColor: "#f3f4f6", bgColor: "#f3f4f6" };

  // Background color of the current active tab
  $: activeTabBgColor = settingsTab.path === activeTab 
    ? settingsTab.bgColor 
    : tabs.find(tab => tab.path === activeTab)?.bgColor || "#f8fafc";
</script>

<div class="flex flex-col h-screen overflow-hidden">
  <div class="flex flex-col overflow-hidden">
    <!-- Top bar - draggable and transparent background -->
    <div class="bg-transparent p-2 flex justify-between border-b border-transparent titlebar">
      <div class="flex-1 drag-region"></div>
      
      {#if platform === 'darwin'}
        <!-- macOS style window controls (left side) -->
        <div class="flex ml-2 order-first">
          <button
            on:click|preventDefault|stopPropagation={hideToTray}
            title="Close"
            aria-label="window close"
            class="mr-2 text-red-500 bg-red-500 rounded-full w-3 h-3 flex items-center justify-center hover:text-red-700"
          >
          </button>
          
          <button
            on:click|preventDefault|stopPropagation={minimizeWindow}
            title="Minimize"
            aria-label="window minimize"
            class="mr-2 text-yellow-500 bg-yellow-500 rounded-full w-3 h-3 flex items-center justify-center hover:text-yellow-700"
          >
          </button>
          
          <button
            on:click|preventDefault|stopPropagation={maximizeWindow}
            title="Maximize"
            aria-label="window maximize"
            class="text-green-500 bg-green-500 rounded-full w-3 h-3 flex items-center justify-center hover:text-green-700"
          >
          </button>
        </div>
      {:else}
        <!-- Windows/Linux style window controls (right side) -->
        <div class="flex">
          <button
            on:click|preventDefault|stopPropagation={minimizeWindow}
            title="Minimize"
            aria-label="window minimize"
            class="mr-2 text-base-content opacity-70 hover:opacity-100"
          >
          <Minus class="w-5 h-5" />
          </button>
          
          <button
            on:click|preventDefault|stopPropagation={maximizeWindow}
            title="Maximize"
            aria-label="window maximize"
            class="mr-2 text-base-content opacity-70 hover:opacity-100"
          >
          <Square class="w-4 h-4" />
          </button>
          
          <button
            on:click|preventDefault|stopPropagation={hideToTray}
            title="Close"
            aria-label="close window"
            class="text-base-content opacity-70 hover:opacity-100 hover:text-red-500"
          >
          <X class="w-5 h-5" />
          </button>
        </div>
      {/if}
    </div>

    <!-- Add tab navigation -->
    <div class="tab-bar px-2 bg-transparent flex w-full">
      <div class="flex gap-2">
        {#each tabs as tab}
          <a 
            href={tab.path}
            class="tab {activeTab === tab.path ? 'active' : ''}" 
            on:click={() => activeTab = tab.path}
            style="--tab-color: {tab.color}; --tab-hover-color: {tab.hoverColor};"
          >
            <svelte:component this={tab.icon} class="w-4 h-4 mr-2" />
            <span>{tab.name}</span>
          </a>
        {/each}
      </div>
      
      <!-- Settings tab (placed on the right) -->
      <div class="ml-auto">
        <a 
          href={settingsTab.path}
          class="tab {activeTab === settingsTab.path ? 'active' : ''}" 
          on:click={() => activeTab = settingsTab.path}
          style="--tab-color: {settingsTab.color}; --tab-hover-color: {settingsTab.hoverColor};"
        >
          <svelte:component this={settingsTab.icon} class="w-4 h-4 mr-2" />
          <span>{settingsTab.name}</span>
        </a>
      </div>
    </div>
    
    <main class="flex-1 overflow-auto p-4" style="background-color: {activeTabBgColor}; margin-top: -1px;">
      <slot />
    </main>
  </div>
</div>

<style>
  /* Set the draggable area of the window */
  .titlebar {
    height: 40px;
  }
  
  .drag-region {
    -webkit-app-region: drag;
    app-region: drag;
    height: 100%;
  }
  
  /* Apply no-drag to clickable elements within the drag region */
  button, a {
    -webkit-app-region: no-drag;
    app-region: no-drag;
  }

  /* Symmetrical trapezoid tab styling */
  .tab-bar {
    padding-bottom: 0;
    position: relative;
    z-index: 10;
  }

  .tab {
    position: relative;
    padding: 0.7em 1.5em;
    color: #4b5563;
    text-decoration: none;
    font-weight: bold;
    z-index: 1;
    display: inline-flex;
    align-items: center;
    transform: perspective(10px) rotateX(1deg);
    transform-origin: bottom;
  }

  .tab::before {
    content: '';
    position: absolute;
    left: 0; top: 0; right: 0; bottom: 0;
    z-index: -1;
    background: rgba(243, 244, 246, 0.7);
    border: 1.5px solid #e5e7eb;
    border-bottom: none;
    border-radius: 8px 8px 0 0;
    transition: all 0.2s ease;
  }

  .tab.active::before {
    background-color: var(--tab-color, #eef2ff); 
    border-color: transparent;
    border-bottom: none;
  }

  .tab.active {
    color: #4b5563;
  }

  .tab:not(.active):hover::before {
    background-color: var(--tab-hover-color, #eef2ff);
    border-color: var(--tab-hover-color, #eef2ff);
    opacity: 0.8;
  }

  .tab:not(.active):hover {
    color: #4b5563;
  }
</style>