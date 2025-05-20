<script lang="ts">
  import "../app.css"
  import { onMount, onDestroy } from "svelte"
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

  // --- Configuration for app appearance ---
  // Background class for the title bar and the tab bar area.
  const topAreaBackgroundClass = "bg-accent"
  // Text color for the top area (title bar and tabs).
  const topAreaContentColorClass = "text-accent-content"

  // --- Tab Definitions ---
  // mainThemeColorVar: CSS variable for the background of the main content area when this tab is active.
  // mainContentColorVar: CSS variable for the text/icon color within the main content area.
  const tabs = [
    { path: "/Installed-MCP", name: "Installed MCP", icon: Presentation, mainThemeColorVar: "--color-primary", mainContentColorVar: "--color-primary-content" },
    { path: "/MCP-list", name: "MCP List", icon: Cog, mainThemeColorVar: "--color-secondary", mainContentColorVar: "--color-secondary-content" },
  ]
  const settingsTab = { path: "/settings", name: "Settings", icon: Settings, mainThemeColorVar: "--color-base-100", mainContentColorVar: "--color-base-content" }

  // --- Tauri specific variables ---
  let tauriWindow: TauriWindowType | null = null
  let currentPlatform: string = "unknown"
  let unlistenMoveToCenter: UnlistenFn | undefined
  let unlistenNavigateTo: UnlistenFn | undefined
  let unlistenConfigFiles: UnlistenFn | undefined
  let unlistenFocusChange: UnlistenFn | undefined
  let unlistenSearchKeyword: UnlistenFn | undefined // 새로운 search-keyword-event 리스너를 위한 변수
  let unlistenSearchKeywordEvent: UnlistenFn | undefined // search-keyword 이벤트 리스너 추가

  // --- Svelte reactive state ---
  let activeTabPath = "/"
  $: isFirstInstallPage = $page?.url?.pathname === "/first-install"
  $: isPopupPage = $page?.url?.pathname === "/popup"

  // Notification activation handler - called when the app gains focus
  // Handles notification clicks or automatic activation events
  async function handleAppActivated() {
    try {
      const response = await invoke<any>("check_and_mark_app_activated", {}) // 타입을 any로 변경 또는 구체적인 타입 지정

      // Extract keyword from the response
      let keyword = null

      // Handle differently based on data type (to accommodate various ways Rust's Option<String> is converted to JSON)
      if (response && typeof response === "object") {
        if (Object.prototype.hasOwnProperty.call(response, "Some")) {
          // 안전한 접근으로 변경
          // Handle Rust's Option<String>::Some
          keyword = (response as { Some: string | null }).Some // 타입 단언 추가
        } else if (Object.prototype.hasOwnProperty.call(response, "0")) {
          // 안전한 접근으로 변경
          // Handle if converted to an array
          keyword = (response as Array<string | null>)[0] // 타입 단언 추가
        }
      } else if (response && typeof response === "string" && response.trim() !== "") {
        // If converted directly to a string
        keyword = response
      }

      if (keyword) {
        // 키워드가 있으면 검색 실행 함수 호출
        handleKeywordSearch(keyword)
      } else {
      }
    } catch (err) {
      console.error("[FRONTEND][Notification] Error in app activation handler:", err)
    }
  }

  // 검색 키워드 처리 함수 분리
  async function handleKeywordSearch(keyword: string) {
    if (!keyword || typeof keyword !== "string" || !keyword.trim()) {
      return
    }

    try {
      // Additional action to ensure the app is actually activated
      if (tauriWindow) {
        // Also attempt to activate the window from the frontend (additional check after backend activation)
        try {
          await tauriWindow.show()
          await tauriWindow.unminimize()
          await tauriWindow.setFocus()

          // Add a short delay to ensure the window is definitely visible
          await new Promise((resolve) => setTimeout(resolve, 100))
        } catch (e) {
          console.error("[FRONTEND][Notification] Frontend window activation failed:", e)
        }
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
                  // Null 체크 추가
                  await tauriWindow.setAlwaysOnTop(false)
                }
              } catch (e) {
                console.error("[FRONTEND][Notification] Error removing always-on-top:", e)
              }
            }, 5000)
          } catch (e) {
            console.error("[FRONTEND][Notification] Frontend focus error:", e)
          }
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
        window.location.href = targetUrl
      }
    } catch (err) {}
  }

  // --- Lifecycle and Subscriptions ---
  onMount(async () => {
    // 개선된 토스트 시스템 초기화
    if (browser) {
      initToastSystem()

      // URI 스킴 프로토콜 핸들러는 deep-link 플러그인으로 대체됨
      // 이벤트 리스너로 처리하는 방식으로 변경

      unlistenSearchKeyword = await listen("search-keyword-event", async (event) => {
        const keyword = event.payload as string

        await handleKeywordSearch(keyword)
      })

      unlistenSearchKeywordEvent = await listen("search-keyword", async (event) => {
        const keyword = event.payload as string

        await handleKeywordSearch(keyword)
      })

      // 테스트 토스트 알림 (개발 환경에서만)
      if (import.meta.env.DEV) {
        setTimeout(() => {
          showToast("토스트 시스템이 성공적으로 초기화되었습니다.", {
            title: "알림 시스템 초기화",
            type: "success",
          })
        }, 1000)
      }
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
        tauriWindow = WebviewWindow.getCurrent()

        // Set up event listeners
        unlistenMoveToCenter = await listen("move-main-to-center", async () => {})

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
              const payload = event.payload as { claudeConfigExists?: boolean; mcplinkConfigExists?: boolean } // 타입 단언 추가
              const { claudeConfigExists, mcplinkConfigExists } = payload

              // If any config file is missing, redirect to first-install page
              if (claudeConfigExists === false || mcplinkConfigExists === false) {
                await goto("/first-install", { replaceState: true })
              }
            } catch (error) {
              console.error("[Config Watch] Failed to handle config files event:", error)
            }
          })
        } catch (error) {
          console.error("[Config Watch] Failed to start config watch:", error)
        }
      } catch (error) {
        console.error("[Layout] Error during Tauri initialization:", error)
      }
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
    if (unlistenSearchKeyword) unlistenSearchKeyword() // 리스너 해제
    if (unlistenSearchKeywordEvent) unlistenSearchKeywordEvent() // 리스너 해제
  })

  // --- Window control functions ---
  async function minimizeWindow() {
    if (tauriWindow) await tauriWindow.minimize()
  }
  async function maximizeWindow() {
    if (tauriWindow) {
      ;(await tauriWindow.isMaximized()) ? tauriWindow.unmaximize() : tauriWindow.maximize()
    }
  }
  async function hideToTray() {
    if (tauriWindow) await tauriWindow.hide()
  }

  // --- Reactive computations for styling ---
  $: currentActivePageConfig = (() => {
    if (activeTabPath === settingsTab.path) return settingsTab
    const foundTab = tabs.find((t) => activeTabPath.startsWith(t.path))
    return foundTab || tabs.find((t) => t.path === "/Installed-MCP") || tabs[0] // Default
  })()

  // Main content area's background and text color, determined by the active tab.
  $: activeMainAreaBackgroundColor = `var(${currentActivePageConfig.mainThemeColorVar})`
  $: activeMainAreaContentColor = `var(${currentActivePageConfig.mainContentColorVar})`
</script>

<!-- Outermost container. If an overall app background different from main content is needed, apply it here. -->
<!-- For now, it's just a flex container. -->
<div class="flex flex-col h-screen overflow-hidden">
  <!-- Top Area: Title Bar and Tab Bar container, with 'accent' background -->
  {#if !isPopupPage}
    <!-- This is now a non-fixed element that takes up space in the flow -->
    <div class="{topAreaBackgroundClass} {topAreaContentColorClass}">
      <!-- Title Bar -->
      <div class="h-8 flex items-center text-xs select-none" data-tauri-drag-region>
        <div class="p-2">
          <img src="/favicon.png" alt="App Icon" class="w-4 h-4" />
        </div>
        <div class="flex-1" data-tauri-drag-region>
          <slot name="title">MCPLINK</slot>
        </div>
        <div class="flex items-center">
          <button on:click={minimizeWindow} title="Minimize" class="p-2 hover:bg-black/5 rounded-sm"><Minus class="w-5 h-5" /></button>
          <button on:click={maximizeWindow} title="Maximize" class="p-2 hover:bg-black/5 rounded-sm"><Square class="w-4 h-4" /></button>
          <button on:click={hideToTray} title="Close to Tray" class="p-2 hover:bg-black/5 rounded-sm"><X class="w-5 h-5" /></button>
        </div>
      </div>

      <!-- Tab Bar (only if not the first install page) -->
      {#if !isFirstInstallPage}
        <div class="tab-bar px-2 flex w-full items-end" style="--Info">
          <div class="flex gap-1">
            {#each tabs as tab (tab.path)}
              <a
                href={tab.path}
                class="tab text-sm md:text-base px-3 py-2 md:px-4 rounded-t-md transition-colors duration-150 ease-in-out hover:bg-white/10"
                style="
                  border-bottom: 2px solid {activeTabPath.startsWith(tab.path) ? 'currentColor' : 'transparent'};
                  opacity: {activeTabPath.startsWith(tab.path) ? '1' : '0.7'};
                "
                on:click|preventDefault={() => goto(tab.path)}
              >
                <svelte:component this={tab.icon} class="w-4 h-4 mr-1 md:mr-2" />
                <span>{tab.name}</span>
              </a>
            {/each}
          </div>
          <div class="ml-auto">
            <!-- Settings Tab -->
            <a
              href={settingsTab.path}
              class="tab text-sm md:text-base px-3 py-2 md:px-4 rounded-t-md transition-colors duration-150 ease-in-out hover:bg-white/10"
              style="
                border-bottom: 2px solid {activeTabPath === settingsTab.path ? 'currentColor' : 'transparent'};
                opacity: {activeTabPath === settingsTab.path ? '1' : '0.7'};
              "
              on:click|preventDefault={() => goto(settingsTab.path)}
            >
              <svelte:component this={settingsTab.icon} class="w-4 h-4 mr-1 md:mr-2" />
              <span>{settingsTab.name}</span>
            </a>
          </div>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Main Content Area -->
  <!-- This area's background and text color change based on the active tab. -->
  <!-- Now it sits below the tabbar naturally in the document flow -->
  <main
    class="flex-1 overflow-y-auto overflow-x-hidden p-4 custom-scrollbar"
    style="
      background-color: {isFirstInstallPage || isPopupPage ? 'var(--color-base-100)' : activeMainAreaBackgroundColor};
      color: {isFirstInstallPage || isPopupPage ? 'var(--color-base-content)' : activeMainAreaContentColor};
      padding-top: {isPopupPage ? '1rem' : '1rem'}; /* Only add minimal padding since we no longer need to accommodate fixed headers */
    "
  >
    <slot />
  </main>

  <!-- Global Toast Notifications -->
  <Toast bind:show={$toastStore.show} message={$toastStore.message} type={$toastStore.type} duration={$toastStore.duration} position={$toastStore.position} />
</div>

<style>
  .tab {
    font-weight: 500;
    /* Ensure consistent color from parent if not overridden by inline styles */
    color: inherit;
  }
  /* Custom scrollbar styles to make them more contained within the main content */
  .custom-scrollbar::-webkit-scrollbar {
    width: 8px;
    height: 8px;
  }

  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
    margin-top: 8px; /* Add some space at the top to prevent overlap with tabs */
  }

  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: #888;
    border-radius: 4px;
  }

  .custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: #555;
  }
  /* Add any other global styles or adjustments here */
</style>
