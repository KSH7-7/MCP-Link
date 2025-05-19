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
      console.log("[FRONTEND] handleAppActivated called") // 로그 추가

      // Check for pending keywords from the backend and handle window activation
      console.log("[FRONTEND] Invoking 'check_and_mark_app_activated'") // 로그 추가
      const response = await invoke<any>("check_and_mark_app_activated", {}) // 타입을 any로 변경 또는 구체적인 타입 지정
      console.log("[FRONTEND] 'check_and_mark_app_activated' response:", response) // 로그 추가

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
        console.log(`[FRONTEND] Keyword found: "${keyword}". Preparing to navigate.`) // 로그 추가
        // 키워드가 있으면 검색 실행 함수 호출
        handleKeywordSearch(keyword)
      } else {
        console.log("[FRONTEND] No keyword found in 'check_and_mark_app_activated' response.") // 로그 추가
      }
    } catch (err) {
      console.error("[FRONTEND][Notification] Error in app activation handler:", err)
    }
  }

  // 검색 키워드 처리 함수 분리
  async function handleKeywordSearch(keyword: string) {
    if (!keyword || typeof keyword !== "string" || !keyword.trim()) {
      console.log("[FRONTEND] Invalid keyword:", keyword)
      return
    }

    try {
      console.log(`[FRONTEND] Processing keyword: "${keyword}"`) // 로그 추가

      // 알림 클릭으로 갑자기 활성화될 때는 추가적인 지연 필요
      // 창 시스템이 안정화될 시간을 제공 (Tauri v2.0에서 전체적인 타이밍 조정)
      await new Promise((resolve) => setTimeout(resolve, 800))

      // 이미 해당 URL에 있는지 확인
      const currentUrl = window.location.pathname + window.location.search
      const targetUrl = `/MCP-list?keyword=${encodeURIComponent(keyword)}`
      console.log(`[FRONTEND] Current URL: ${currentUrl}, Target URL: ${targetUrl}`)

      if (currentUrl === targetUrl) {
        console.log(`[FRONTEND] Already at the target URL. Refreshing page instead.`)
        // 이미 같은 URL에 있는 경우, 페이지 새로고침
        window.location.reload()
        return
      }

      // Tauri v2.0에서 WebviewWindow API 사용
      if (tauriWindow) {
        console.log("[FRONTEND] Window object available. Ensuring window visibility")

        // 창이 최소화되어 있으면 복원
        const isMinimized = await tauriWindow.isMinimized()
        if (isMinimized) {
          console.log("[FRONTEND] Window is minimized. Restoring...")
          await tauriWindow.unminimize()
          // 복원 후 약간의 지연 제공
          await new Promise((resolve) => setTimeout(resolve, 200))
        }

        // 창이 보이지 않으면 표시
        const isVisible = await tauriWindow.isVisible()
        if (!isVisible) {
          console.log("[FRONTEND] Window is not visible. Showing...")
          await tauriWindow.show()
          // 표시 후 약간의 지연 제공
          await new Promise((resolve) => setTimeout(resolve, 200))
        }

        // 최종 포커스 설정 (2번의 시도)
        console.log("[FRONTEND] Setting window focus...")
        await tauriWindow.setFocus()

        // 추가 포커스 시도 (약간 지연 후)
        await new Promise((resolve) => setTimeout(resolve, 100))
        await tauriWindow.setFocus()
      }

      // URL encode the keyword to include it as a query parameter
      console.log(`[FRONTEND] Target URL with keyword: ${targetUrl}`) // 로그 추가

      // 1. activeTabPath 설정 (메뉴 하이라이팅용)
      console.log("[FRONTEND] activeTabPath set to /MCP-list")
      activeTabPath = "/MCP-list"

      // 2. 페이지 이동 시도
      try {
        // goto를 사용하는 경우 에러 캐치를 위한 래핑
        console.log("[FRONTEND] Attempting navigation with goto...")
        await goto(targetUrl, { replaceState: false })
      } catch (navError) {
        console.error("[FRONTEND] Navigation failed with goto. Using direct location change.", navError)

        // goto 실패 시 직접 location 변경 (대체 방법)
        window.location.href = targetUrl
      }

      // 키워드 처리 완료 알림
      console.log(`[FRONTEND] Keyword "${keyword}" processing completed`)
    } catch (error) {
      console.error("[FRONTEND] Error in handleKeywordSearch:", error)
    }
  }

  // --- Lifecycle and Subscriptions ---
  onMount(async () => {
    // 개선된 토스트 시스템 초기화
    if (browser) {
      initToastSystem()

      // URI 스킴 프로토콜 핸들러는 deep-link 플러그인으로 대체됨
      // 이벤트 리스너로 처리하는 방식으로 변경
      console.log("[FRONTEND] Setting up 'search-keyword-event' listener.") // 로그 추가
      unlistenSearchKeyword = await listen("search-keyword-event", async (event) => {
        const keyword = event.payload as string
        console.log(`[FRONTEND] Received 'search-keyword-event' with keyword: "${keyword}" (via direct listen)`)
        await handleKeywordSearch(keyword)
      })

      // 'search-keyword' 이벤트 리스너 추가
      console.log("[FRONTEND] Setting up 'search-keyword' listener.") // 로그 추가
      unlistenSearchKeywordEvent = await listen("search-keyword", async (event) => {
        const keyword = event.payload as string
        console.log(`[FRONTEND] Received 'search-keyword' with keyword: "${keyword}"`)
        await handleKeywordSearch(keyword)
      })
      console.log("[FRONTEND] 'search-keyword' listener setup complete.") // 로그 추가

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
        // 간단한 OS 플랫폼 감지
        let osType: string = "unknown"

        // Tauri 플러그인 대신 navigator.platform 사용
        const navPlatform = navigator.platform.toLowerCase()
        if (navPlatform.includes("win")) osType = "windows"
        else if (navPlatform.includes("mac")) osType = "macos"
        else if (navPlatform.includes("linux")) osType = "linux"

        console.log("[Layout] Detected platform:", osType)
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
        console.error("[Layout] OS detection error:", e)
        currentPlatform = "unknown"
      }
    }

    if (typeof window !== "undefined" && "__TAURI__" in window) {
      try {
        tauriWindow = WebviewWindow.getCurrent()
        console.log("[FRONTEND] Tauri window object obtained.", tauriWindow) // 로그 추가

        // Set up event listeners
        unlistenMoveToCenter = await listen("move-main-to-center", async () => {
          // This event should not be triggered anymore
          // We no longer automatically center the window
          console.log("move-main-to-center event received but ignored to prevent auto-centering")
        })

        unlistenNavigateTo = await listen("navigate-to", async (event) => {
          if (event.payload && typeof event.payload === "string") goto(event.payload as string)
        })

        // Focus event listener that completely ignores all focus events
        // This prevents the window from auto-centering when focused
        const focusListener = await tauriWindow.onFocusChanged(async ({ payload: focused }) => {
          if (focused) {
            console.log("[FRONTEND] Window gained focus. Calling handleAppActivated.") // 로그 추가
            await handleAppActivated()
          } else {
            console.log("[FRONTEND] Window lost focus.") // 로그 추가
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
                // 명시적으로 false 비교
                console.log("[Config Watch] Configuration files missing, redirecting to first-install")
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
    console.log("[FRONTEND] Event listeners cleaned up on destroy.") // 로그 추가
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
