<script lang="ts">
  import "../app.css"
  import { onMount, onDestroy, setContext } from "svelte"
  import { showNotification } from "$lib/notifications"
  import { browser } from "$app/environment"
  import type { Window as TauriWindowType } from "@tauri-apps/api/window"
  import { Presentation, Cog, Minus, X, Square, Settings } from "lucide-svelte"
  import { page } from "$app/stores"
  import { goto } from "$app/navigation"
  import { listen, type UnlistenFn } from "@tauri-apps/api/event"
  import { UserAttentionType } from "@tauri-apps/api/window"
  import { platform as getOsPlatform } from "@tauri-apps/plugin-os"
  import { invoke } from "@tauri-apps/api/core"
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow"
  import Toast from "$lib/components/toast.svelte"
  import toastStore from "$lib/stores/toast"
  import { initToastSystem, showToast } from "$lib/toast-system.js"
  import { handleUriScheme } from "$lib/notifications"
  import { scrollableContainerKey } from "$lib/contexts"
  import { sharedDataStore, updateCount, setLoaded } from "$lib/stores/data-store"
  import gsap from "gsap"

  // --- Scrollable Container Context ---
  // export const scrollableContainerKey = Symbol()
  // moved to contexts.ts
  let mainElement: HTMLElement

  // --- Configuration for app appearance ---
  // Window bar (title bar) styling - for unified design, use base-100
  const windowBarBackgroundClass = "bg-base-100"
  const windowBarContentColorClass = "text-base-content"

  // Tab bar styling - for unified design, use base-100
  const tabBarBackgroundClass = "bg-base-100"
  const tabBarContentColorClass = "text-base-content"

  // --- Tab Definitions ---
  // mainClass: CSS class for the main content area background when this tab is active.
  // mainContentClass: CSS class for the text/icon color within the main content area.
  const tabs = [
    { path: "/Installed-MCP", name: "Installed MCP", icon: Presentation, mainClass: "bg-secondary", mainContentClass: "text-secondary-content", tabClass: "hover:bg-secondary/10" },
    { path: "/MCP-list", name: "MCP List", icon: Cog, mainClass: "bg-secondary", mainContentClass: "text-secondary-content", tabClass: "hover:bg-secondary/10" },
  ]
  const settingsTab = { path: "/settings", name: "Settings", icon: Settings, mainClass: "bg-base-300", mainContentClass: "text-base-content", tabClass: "hover:bg-base-content/10" }

  // --- Tauri specific variables ---
  let tauriWindow: TauriWindowType | null = null
  let currentPlatform: string = "unknown"
  let unlistenMoveToCenter: UnlistenFn | undefined
  let unlistenNavigateTo: UnlistenFn | undefined
  let unlistenConfigFiles: UnlistenFn | undefined
  let unlistenFocusChange: UnlistenFn | undefined
  let unlistenSearchKeyword: UnlistenFn | undefined // for new search-keyword-event listener
  let unlistenSearchKeywordEvent: UnlistenFn | undefined // for search-keyword event listener

  // --- Svelte reactive state ---
  let activeTabPath = "/"
  $: isFirstInstallPage = $page?.url?.pathname === "/first-install"
  $: isPopupPage = $page?.url?.pathname === "/popup"

  // reactive block for setting up wheel event listener
  $: if (mainElement && browser) {
    setContext(scrollableContainerKey, mainElement)
    // add wheel event listener
    mainElement.addEventListener("wheel", handleWheel, { passive: true })
    mainElement.addEventListener("touchstart", handleWheel, { passive: true })
  }

  // Notification activation handler - called when the app gains focus
  // Handles notification clicks or automatic activation events
  async function handleAppActivated() {
    try {
      const response = await invoke<any>("check_and_mark_app_activated", {}) // change type to any or specify concrete type

      // Extract keyword from the response
      let keyword = null

      // Handle differently based on data type (to accommodate various ways Rust's Option<String> is converted to JSON)
      if (response && typeof response === "object") {
        if (Object.prototype.hasOwnProperty.call(response, "Some")) {
          // change to safe access
          // Handle Rust's Option<String>::Some
          keyword = (response as { Some: string | null }).Some // add type assertion
        } else if (Object.prototype.hasOwnProperty.call(response, "0")) {
          // change to safe access
          // Handle if converted to an array
          keyword = (response as Array<string | null>)[0] // add type assertion
        }
      } else if (response && typeof response === "string" && response.trim() !== "") {
        // If converted directly to a string
        keyword = response
      }

      if (keyword) {
        // if keyword exists, call search execution function
        handleKeywordSearch(keyword)
      } else {
      }
    } catch (err) {}
  }

  // separate search keyword processing function
  async function handleKeywordSearch(keyword: string) {
    if (!keyword || typeof keyword !== "string" || !keyword.trim()) {
      return
    }

    try {
      // immediately apply animation to scroll position to top
      if (mainElement) {
        smoothScrollToTop()
      }

      // Additional action to ensure the app is actually activated
      if (tauriWindow) {
        // Also attempt to activate the window from the frontend (additional check after backend activation)
        try {
          await tauriWindow.show()
          await tauriWindow.unminimize()
          await tauriWindow.setFocus()

          // Add a short delay to ensure the window is definitely visible
          await new Promise((resolve) => setTimeout(resolve, 100))
        } catch (e) {}
      }

      // URL encode the keyword to include it as a query parameter
      const targetUrl = `/MCP-list?keyword=${encodeURIComponent(keyword)}`

      // Page navigation (goto is client-side routing between pages)
      try {
        // 1. First, switch URL and update state
        activeTabPath = "/MCP-list"

        // 2. Attempt to activate the app even if the window is already visible
        if (tauriWindow) {
          try {
            // Additional attempt to bring window focus
            await tauriWindow.show()
            await tauriWindow.setFocus()

            // Bring the window to the top using always-on-top setting
            await tauriWindow.setAlwaysOnTop(true)

            // Disable always-on-top after 5 seconds (to allow user to use other windows)
            setTimeout(async () => {
              try {
                if (tauriWindow) {
                  // add null check
                  await tauriWindow.setAlwaysOnTop(false)
                }
              } catch (e) {}
            }, 5000)
          } catch (e) {}
        }

        // 3. Handle uniformly whether URL navigation succeeds or fails
        await Promise.race([
          goto(targetUrl, {
            replaceState: true, // Replace the current URL
            invalidateAll: true, // Reload all data
            noScroll: false, // Scroll to the top of the page
          }),
          // 1-second timeout (proceed even if navigation fails)
          new Promise((resolve) => setTimeout(resolve, 1000)),
        ])

        // 4. Attempt to reactivate window regardless of page navigation
        if (tauriWindow) {
          await tauriWindow.setFocus()
        }
      } catch (err) {
        // Attempt to force a path change even if an error occurs
        window.location.href = targetUrl
      }
    } catch (err) {}
  }

  // --- Lifecycle and Subscriptions ---
  onMount(async () => {
    // improved toast system initialization
    if (browser) {
      initToastSystem()

      // simplified event listener - update only active tab path
      window.addEventListener("navigate-to-event", ((event: CustomEvent) => {
        if (event.detail && event.detail.path) {
          // set active tab path received from event
          activeTabPath = event.detail.path
        }
      }) as EventListener)

      // initialize data store
      setLoaded(false)

      // load shared data to be shared between page transitions
      try {
        // load common data needed for the app
        // e.g. data for installed MCP count, MCP list count, etc.

        // load example data (in reality, data should be loaded via invoke, etc.)
        const installedCount = await invoke("get_installed_count").catch(() => 0)
        const listCount = await invoke("get_list_count").catch(() => 0)

        updateCount("installedCount", installedCount as number)
        updateCount("listCount", listCount as number)

        // show data load complete
        setLoaded(true)
      } catch (error) {
        // even if data load fails, the app will continue to operate
        setLoaded(true)
      }

      // URI scheme protocol handler is replaced with deep-link plugin
      // change to handle via event listener
      unlistenSearchKeyword = await listen("search-keyword-event", async (event) => {
        const keyword = event.payload as string
        await handleKeywordSearch(keyword)
      })

      // add 'search-keyword' event listener
      unlistenSearchKeywordEvent = await listen("search-keyword", async (event) => {
        const keyword = event.payload as string
        await handleKeywordSearch(keyword)
      })
    }
    activeTabPath = $page.url.pathname

    if (browser) {
      try {
        const osType: string = await getOsPlatform()
        currentPlatform = osType
        if (osType === "windows") {
          // Check if config files exist on startup
          try {
            // Check claude_desktop_config.json
            const claudeConfigExists = await invoke<boolean>("check_claude_config_exists")

            // Check mcplink_desktop_config.json
            const mcplinkConfigExists = await invoke<boolean>("check_mcplink_config_exists")

            // Get current path
            const currentPath = $page?.url?.pathname

            // If any config file is missing and not already on first-install page
            if ((!claudeConfigExists || !mcplinkConfigExists) && currentPath !== "/first-install") {
              await goto("/first-install", { replaceState: true })
              return
            } else if (claudeConfigExists && mcplinkConfigExists && currentPath === "/first-install") {
              // If all config files exist but we're on first-install page, redirect to main page
              await goto("/Installed-MCP", { replaceState: true })
              return
            }
          } catch (error) {
            // Error checking config files
          }
        }
      } catch (e) {
        currentPlatform = "unknown"
      }
    }

    if (typeof window !== "undefined" && "__TAURI__" in window) {
      try {
        // initialize window handle more stably
        tauriWindow = WebviewWindow.getCurrent()

        // if initialization fails, retry
        if (!tauriWindow) {
          setTimeout(() => {
            try {
              tauriWindow = WebviewWindow.getCurrent()
              if (tauriWindow) {
              }
            } catch (e) {}
          }, 500)
        }

        // Set up event listeners
        unlistenMoveToCenter = await listen("move-main-to-center", async () => {
          // This event should not be triggered anymore
          // We no longer automatically center the window
        })

        unlistenNavigateTo = await listen("navigate-to", async (event) => {
          if (event.payload && typeof event.payload === "string") goto(event.payload as string)
        })

        // Focus event listener that completely ignores all focus events
        // This prevents the window from auto-centering when focused
        const focusListener = await tauriWindow.onFocusChanged(async ({ payload: focused }) => {
          if (focused) {
            await handleAppActivated()
          } else {
          }
        })

        // Store unlisten function for cleanup in a variable for later use
        if (focusListener) {
          unlistenFocusChange = focusListener
        }

        // Start watching for config file changes
        try {
          // Start the config file watcher in the backend
          await invoke("start_config_watch")

          // Listen for config files missing events
          unlistenConfigFiles = await listen("config-files-missing", async (event) => {
            try {
              // Get the current path and ignore if already on first-install page
              const currentPath = $page?.url?.pathname
              if (currentPath === "/first-install") return

              // Extract which files are missing from the event payload
              const payload = event.payload as { claudeConfigExists?: boolean; mcplinkConfigExists?: boolean } // add type assertion
              const { claudeConfigExists, mcplinkConfigExists } = payload

              // If any config file is missing, redirect to first-install page
              if (claudeConfigExists === false || mcplinkConfigExists === false) {
                // explicitly compare false
                await goto("/first-install", { replaceState: true })
              }
            } catch (error) {}
          })
        } catch (error) {}
      } catch (error) {}
    }
  })

  page.subscribe((value) => {
    if (browser) activeTabPath = value.url.pathname
  })

  // Clean up all event listeners on component destruction
  onDestroy(() => {
    // Clean up Tauri event listeners
    if (unlistenMoveToCenter) unlistenMoveToCenter()
    if (unlistenNavigateTo) unlistenNavigateTo()
    if (unlistenConfigFiles) unlistenConfigFiles()
    if (unlistenFocusChange) unlistenFocusChange()
    if (unlistenSearchKeyword) unlistenSearchKeyword() // remove listener
    if (unlistenSearchKeywordEvent) unlistenSearchKeywordEvent() // remove listener

    // if there is a scroll animation in progress, stop it
    if (currentScrollAnimation) {
      currentScrollAnimation.kill()
      currentScrollAnimation = null
    }

    // remove wheel and touch event listeners
    if (browser && mainElement) {
      mainElement.removeEventListener("wheel", handleWheel)
      mainElement.removeEventListener("touchstart", handleWheel)
    }

    // remove custom event listeners
    if (browser) {
      // when removing event listeners, specify null to allow garbage collection
      // event listeners are only cleaned up when the app is closed, so it is not actually necessary to remove them here
    }
  })

  // variable to store reference to ongoing scroll animation
  let currentScrollAnimation: gsap.core.Tween | null = null

  // function to check scroll position and execute scroll animation
  function smoothScrollToTop() {
    if (!mainElement) return

    // apply animation only if current scroll position is 1 pixel or more
    if (mainElement.scrollTop > 1) {
      // if there is already an ongoing animation, stop it
      if (currentScrollAnimation) {
        currentScrollAnimation.kill()
        currentScrollAnimation = null
      }

      // use GSAP to apply smooth scroll animation
      currentScrollAnimation = gsap.to(mainElement, {
        scrollTop: 0,
        duration: 0.6, // animation duration (seconds)
        ease: "power2.out", // acceleration curve (easing)
        onComplete: () => {
          // remove reference when animation is complete
          currentScrollAnimation = null
        },
      })
    }
  }

  // function to stop animation when mouse wheel event occurs
  function handleWheel() {
    if (currentScrollAnimation) {
      currentScrollAnimation.kill()
      currentScrollAnimation = null
    }
  }

  // --- Window control functions (improved version) ---
  async function minimizeWindow() {
    try {
      // if tauriWindow is not available, get current window directly
      const window = tauriWindow || WebviewWindow.getCurrent()
      if (window) {
        await window.minimize()
      } else {
        showToast("cannot minimize window", "error")
      }
    } catch (e) {
      showToast("error minimizing window", "error")
    }
  }

  async function maximizeWindow() {
    try {
      // if tauriWindow is not available, get current window directly
      const window = tauriWindow || WebviewWindow.getCurrent()
      if (window) {
        const isMaximized = await window.isMaximized()
        if (isMaximized) {
          await window.unmaximize()
        } else {
          await window.maximize()
        }
      } else {
        showToast("cannot maximize window", "error")
      }
    } catch (e) {
      showToast("error maximizing window", "error")
    }
  }

  async function hideToTray() {
    try {
      // if tauriWindow is not available, get current window directly
      const window = tauriWindow || WebviewWindow.getCurrent()
      if (window) {
        await window.hide()
      } else {
        showToast("cannot hide window", "error")
      }
    } catch (e) {
      showToast("error hiding window", "error")
    }
  }

  // --- Reactive computations for styling ---
  $: currentActivePageConfig = (() => {
    if (activeTabPath === settingsTab.path) return settingsTab

    // find exact match for tab
    const foundTab = tabs.find((t) => activeTabPath === t.path)
    return foundTab || tabs.find((t) => t.path === "/Installed-MCP") || tabs[0] // default value
  })()

  // calculate background color and text color reactively based on active tab
  $: activeMainAreaBackgroundColor = isFirstInstallPage || isPopupPage ? "var(--color-base-100)" : `var(--${currentActivePageConfig.mainClass.replace("bg-", "color-")})`

  $: activeMainAreaContentColor = isFirstInstallPage || isPopupPage ? "var(--color-base-content)" : `var(--${currentActivePageConfig.mainContentClass.replace("text-", "color-")})`

  // window control button styles are handled in CSS
</script>

<!-- Outermost container. If an overall app background different from main content is needed, apply it here. -->
<!-- For now, it's just a flex container. -->
<div class="flex flex-col h-screen overflow-hidden">
  <!-- Top Area: Title Bar and Tab Bar container -->
  {#if !isPopupPage}
    <!-- Window Title Bar -->
    <div class="{windowBarBackgroundClass} {windowBarContentColorClass}">
      <div class="h-8 flex items-center text-xs select-none" data-tauri-drag-region>
        <!-- left space -->
        <div class="w-[100px]"></div>

        <!-- center title -->
        <div class="absolute left-0 right-0 mx-auto flex justify-center items-center" data-tauri-drag-region>
          <img src="/favicon.png" alt="App Icon" class="w-4 h-4 mr-2" />
          <slot name="title">MCPLINK</slot>
        </div>

        <!-- transparent buttons above icons are placed as decoration -->
        <div class="ml-auto flex">
          <!-- icons are placed inside the button as absolute position for decoration -->
          <div class="window-btn min-btn" on:click={minimizeWindow}>
            <span class="icon-wrapper">
              <Minus size={16} />
            </span>
          </div>

          <div class="window-btn max-btn" on:click={maximizeWindow}>
            <span class="icon-wrapper">
              <Square size={16} />
            </span>
          </div>

          <div class="window-btn close-btn" on:click={hideToTray}>
            <span class="icon-wrapper">
              <X size={16} />
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Tab Bar (separate from title bar) -->
    {#if !isFirstInstallPage}
      <div class="{tabBarBackgroundClass} {tabBarContentColorClass} px-2 flex w-full items-end" style="padding-top: 0.25rem; position: relative;">
        <div class="flex gap-1">
          {#each tabs as tab (tab.path)}
            <a
              href={tab.path}
              class="tab text-sm md:text-base {activeTabPath === tab.path ? 'bg-secondary text-secondary-content' : 'bg-primary text-primary-content'}"
              class:active-tab-styling={activeTabPath === tab.path}
              style="
                opacity: {activeTabPath === tab.path ? '1' : '0.7'};
                font-weight: {activeTabPath === tab.path ? '600' : '500'};
                position: relative;
              "
              on:click|preventDefault={() => {
                // when tab is clicked, initialize the active state of all other tabs and activate only the current tab
                activeTabPath = tab.path
                // apply smooth scroll animation
                smoothScrollToTop()
                goto(tab.path)
              }}
            >
              <div class="flex items-center justify-center">
                <svelte:component this={tab.icon} class="w-4 h-4 mr-1 md:mr-2" />
                <span>{tab.name}</span>
              </div>
            </a>
          {/each}
        </div>
        <div class="ml-auto">
          <!-- Settings Tab -->
          <a
            href={settingsTab.path}
            class="tab text-sm md:text-base bg-base-300 text-base-content"
            class:active-tab-styling={activeTabPath === settingsTab.path}
            style="
              opacity: {activeTabPath === settingsTab.path ? '1' : '0.7'};
              font-weight: {activeTabPath === settingsTab.path ? '600' : '500'};
              position: relative;
            "
            on:click|preventDefault={() => {
              // when settings tab is clicked, initialize all tabs and activate only the settings tab
              activeTabPath = settingsTab.path
              // apply smooth scroll animation
              smoothScrollToTop()
              goto(settingsTab.path)
            }}
          >
            <div class="flex items-center justify-center">
              <svelte:component this={settingsTab.icon} class="w-4 h-4 mr-1 md:mr-2" />
              <span>{settingsTab.name}</span>
            </div>
          </a>
        </div>
      </div>
    {/if}
  {/if}

  <!-- Main Content Area -->
  <!-- This area's background and text color change based on the active tab. -->
  <main
    bind:this={mainElement}
    class="flex-1 overflow-y-auto overflow-x-hidden custom-scrollbar"
    style="
      background-color: {isFirstInstallPage || isPopupPage ? 'var(--color-base-100)' : activeMainAreaBackgroundColor};
      color: {isFirstInstallPage || isPopupPage ? 'var(--color-base-content)' : activeMainAreaContentColor};
    "
  >
    <slot></slot>
  </main>

  <!-- Global Toast Notifications -->
  <Toast bind:show={$toastStore.show} message={$toastStore.message} type={$toastStore.type} duration={$toastStore.duration} position={$toastStore.position} />
</div>

<style>
  /* tab default styles */
  .tab {
    position: relative;
    padding: 10px 15px;
    margin-right: 1px;
    border-top-left-radius: 10px;
    border-top-right-radius: 10px;
    border-bottom-left-radius: 0;
    border-bottom-right-radius: 0;
    font-weight: 500;
    color: inherit;
    box-shadow: none;
    transform: none;
    min-width: 150px; /* slightly wider than "Installed MCP" */
    display: flex;
    align-items: center;
    justify-content: center;
    transition:
      background-color 0.2s,
      color 0.2s,
      opacity 0.2s,
      font-weight 0.2s;
  }

  /* Custom scrollbar styles to make them more contained within the main content */
  .custom-scrollbar::-webkit-scrollbar {
    width: 8px;
    height: 8px;
  }

  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
    /* margin-top: 8px; */ /* Removed to allow scrollbar to reach top of main element */
  }

  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: #888;
    border-radius: 4px;
  }

  .custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: #555;
  }
  /* Add any other global styles or adjustments here */

  /* window control button styles */
  .window-btn {
    width: 46px;
    height: 32px;
    background-color: transparent;
    cursor: default; /* keep default mouse cursor */
    position: relative; /* base point for icons */
    user-select: none;
  }

  .icon-wrapper {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none; /* icons do not handle events */
  }

  .min-btn:hover,
  .max-btn:hover {
    background-color: oklch(var(--color-base-300)) !important; /* base-300 color */
  }

  .close-btn:hover {
    background-color: oklch(var(--color-error)) !important; /* error color */
  }

  /* block pointer events for all SVG icons */
  svg {
    pointer-events: none;
  }
</style>
