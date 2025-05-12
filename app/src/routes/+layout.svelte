<script lang="ts">
  import "../app.css"
  import { onMount, onDestroy } from "svelte"
  import { browser } from '$app/environment';
  import type { Window } from "@tauri-apps/api/window"
  import { Presentation, Cog, Minus, X, Square, Settings } from "lucide-svelte"
  import { page } from "$app/stores"
  import { listen } from "@tauri-apps/api/event"
  import type { UnlistenFn } from "@tauri-apps/api/event"
  import { goto } from "$app/navigation"
  import { move_window, Position } from "tauri-plugin-positioner-api"

  // Tauri window object to save
  let tauriWindow: Window | null = null
  // OS platform detection
  let platform: string = "unknown"

  // START: 추가된 변수 (이벤트 리스너 해제용)
  let unlistenMoveToCenter: UnlistenFn | undefined
  let unlistenNavigateTo: UnlistenFn | undefined
  // END: 추가된 변수

  onMount(async () => {
    // --- START: 파일 확인 및 초기 리디렉션 로직 (BaseDirectory 사용으로 복원) ---
    if (browser) {
      const { exists, BaseDirectory } = await import('@tauri-apps/plugin-fs'); // BaseDirectory 다시 임포트
      const CONFIG_FILE_NAME = 'claude_desktop_config.json';
      const FIRST_INSTALL_PATH = '/first-install';
      const DEFAULT_PATH_AFTER_INSTALL = '/Installed-MCP';

      try {
        // BaseDirectory.AppConfig 사용 방식으로 복원
        const configFileExists = await exists(CONFIG_FILE_NAME, { baseDir: BaseDirectory.AppConfig });
        console.log(`[Layout OnMount Check] Config file (${CONFIG_FILE_NAME}) exists in AppConfig: ${configFileExists}`);

        const currentPath = $page.url.pathname;

        if (!configFileExists && currentPath !== FIRST_INSTALL_PATH) {
          console.log(`[Layout OnMount Check] Config file not found. Redirecting to ${FIRST_INSTALL_PATH}`);
          await goto(FIRST_INSTALL_PATH, { replaceState: true });
          return;
        } else if (configFileExists && currentPath === FIRST_INSTALL_PATH) {
          console.log(`[Layout OnMount Check] Config file found. Redirecting from ${FIRST_INSTALL_PATH} to ${DEFAULT_PATH_AFTER_INSTALL}`);
          await goto(DEFAULT_PATH_AFTER_INSTALL, { replaceState: true });
          return;
        }
      } catch(e) {
         console.error("[Layout OnMount Check] Error checking config file:", e);
      }
    }
    // --- END: 파일 확인 및 초기 리디렉션 로직 ---

    console.log("[Layout] onMount 시작 - Tauri 초기화 진행")
    // Check if Tauri API exists
    if (typeof window !== "undefined" && ("__TAURI_INTERNALS__" in window || "__TAURI__" in window)) {
      try {
        // Get current window - Tauri v2 API
        const { getCurrentWindow } = await import("@tauri-apps/api/window")
        tauriWindow = getCurrentWindow()

        // OS type detection
        const { platform: getPlatform } = await import("@tauri-apps/plugin-os")
        platform = await getPlatform()
        console.log("[Layout] Tauri 환경 감지, 플랫폼:", platform)

        // START: 추가된 이벤트 리스너 (메인 윈도우 중앙 이동)
        unlistenMoveToCenter = await listen("move-main-to-center", async () => {
          console.log("[Layout] 'move-main-to-center' 이벤트 수신")
          try {
            console.log("[Layout] move_window(Position.Center) 호출 전")
            await move_window(Position.Center)
            console.log("[Layout] move_window(Position.Center) 호출 후")
            await tauriWindow?.show() // 혹시 숨겨져 있다면 보이도록
            await tauriWindow?.unminimize() // 혹시 최소화 상태라면 해제
            await tauriWindow?.setFocus() // 포커스
            console.log("[Layout] 창 중앙 이동 및 상태 복원 완료")
          } catch (error) {
            console.error("[Layout] Failed to move main window to center:", error)
          }
        })
        console.log("[Layout] 'move-main-to-center' 리스너 등록 완료")

        // START: 추가된 이벤트 리스너 (특정 페이지로 네비게이션)
        // popup 페이지에서 MCP-list로 이동 요청 시 처리
        unlistenNavigateTo = await listen("navigate-to", async (event) => {
          console.log("[Layout] 'navigate-to' 이벤트 수신, payload:", event.payload)
          if (event.payload && typeof event.payload === "string") {
            const targetUrl = event.payload as string
            console.log(`[Layout] goto('${targetUrl}') 호출 전`)
            goto(targetUrl)
            console.log(`[Layout] goto('${targetUrl}') 호출 후`)
          } else {
            console.warn("[Layout] 'navigate-to' 이벤트 페이로드 없음 또는 문자열 아님:", event.payload)
          }
        })
        console.log("[Layout] 'navigate-to' 리스너 등록 완료")
      } catch (error) {
        console.error("[Layout] error: get current window or platform", error)
      }
    } else {
      console.log("[Layout] 브라우저 환경 또는 Tauri API 없음")
      // If running in the browser (use Navigator API)
      if (typeof navigator !== "undefined") {
        if (navigator.userAgent.indexOf("Win") !== -1) platform = "win32"
        else if (navigator.userAgent.indexOf("Mac") !== -1) platform = "darwin"
        else if (navigator.userAgent.indexOf("Linux") !== -1) platform = "linux"
      }
    }
    console.log("[Layout] onMount 종료")
  })

  // START: onDestroy 추가 (이벤트 리스너 해제)
  onDestroy(() => {
    console.log("[Layout] onDestroy 호출됨, 리스너 해제 시도")
    unlistenMoveToCenter?.()
    unlistenNavigateTo?.()
    console.log("[Layout] 리스너 해제 완료")
  })
  // END: onDestroy 추가

  // Minimize button handler
  async function minimizeWindow() {
    if (tauriWindow) {
      try {
        await tauriWindow.minimize()
      } catch (error) {
        console.error("error: minimize window", error)
      }
    }
  }

  // Maximize button handler
  async function maximizeWindow() {
    if (tauriWindow) {
      try {
        const isMaximized = await tauriWindow.isMaximized()
        if (isMaximized) {
          await tauriWindow.unmaximize()
        } else {
          await tauriWindow.maximize()
        }
      } catch (error) {
        console.error("error: maximize window", error)
      }
    }
  }

  // Hide to tray button handler
  async function hideToTray() {
    if (tauriWindow) {
      try {
        await tauriWindow.hide()
      } catch (error) {
        console.error("error: hide to tray", error)
      }
    }
  }

  // Current active tab
  let activeTab = "/"

  // Tab information and colors
  const tabs = [
    { path: "/Installed-MCP", name: "Installed MCP", icon: Presentation, color: "#eef2ff", hoverColor: "#eef2ff", bgColor: "#eef2ff" },
    { path: "/MCP-list", name: "MCP List", icon: Cog, color: "#ecfdf5", hoverColor: "#ecfdf5", bgColor: "#ecfdf5" },
    { path: "/test", name: "Test", icon: Cog, color: "#f0f9ff", hoverColor: "#f0f9ff", bgColor: "#f0f9ff" },
  ]

  // Settings tab (placed on the right)
  const settingsTab = { path: "/settings", name: "Settings", icon: Settings, color: "#f3f4f6", hoverColor: "#f3f4f6", bgColor: "#f3f4f6" }

  // Background color of the current active tab
  $: activeTabBgColor = settingsTab.path === activeTab ? settingsTab.bgColor : tabs.find((tab) => tab.path === activeTab)?.bgColor || "#f8fafc"

  // Determine if the current page is the first-install page
  $: isFirstInstallPage = $page.url.pathname === "/first-install"

  // Determine if the current page is the popup page
  $: isPopupPage = $page.url.pathname === "/popup"
</script>

<div class="flex flex-col h-screen overflow-hidden">
  <div class="flex flex-col overflow-hidden">
    <!-- Top bar - draggable and transparent background -->
    {#if !isPopupPage}
      <div class="bg-transparent p-2 flex justify-between border-b border-transparent titlebar">
        <div class="flex-1 drag-region"></div>

        {#if platform === "darwin"}
          <!-- macOS style window controls (left side) -->
          <div class="flex ml-2 order-first">
            <button
              on:click|preventDefault|stopPropagation={hideToTray}
              title="Close"
              aria-label="window close"
              class="mr-2 text-red-500 bg-red-500 rounded-full w-3 h-3 flex items-center justify-center hover:text-red-700"
            ></button>

            <button
              on:click|preventDefault|stopPropagation={minimizeWindow}
              title="Minimize"
              aria-label="window minimize"
              class="mr-2 text-yellow-500 bg-yellow-500 rounded-full w-3 h-3 flex items-center justify-center hover:text-yellow-700"
            ></button>

            <button
              on:click|preventDefault|stopPropagation={maximizeWindow}
              title="Maximize"
              aria-label="window maximize"
              class="text-green-500 bg-green-500 rounded-full w-3 h-3 flex items-center justify-center hover:text-green-700"
            ></button>
          </div>
        {:else}
          <!-- Windows/Linux style window controls (right side) -->
          <div class="flex">
            <button on:click|preventDefault|stopPropagation={minimizeWindow} title="Minimize" aria-label="window minimize" class="mr-2 text-base-content opacity-70 hover:opacity-100">
              <Minus class="w-5 h-5" />
            </button>

            <button on:click|preventDefault|stopPropagation={maximizeWindow} title="Maximize" aria-label="window maximize" class="mr-2 text-base-content opacity-70 hover:opacity-100">
              <Square class="w-4 h-4" />
            </button>

            <button on:click|preventDefault|stopPropagation={hideToTray} title="Close" aria-label="close window" class="text-base-content opacity-70 hover:opacity-100 hover:text-red-500">
              <X class="w-5 h-5" />
            </button>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Add tab navigation -->
    {#if !isPopupPage && !isFirstInstallPage}
      <div class="tab-bar px-2 bg-transparent flex w-full">
        <div class="flex gap-2">
          {#each tabs as tab}
            <a href={tab.path} class="tab {activeTab === tab.path ? 'active' : ''}" on:click={() => (activeTab = tab.path)} style="--tab-color: {tab.color}; --tab-hover-color: {tab.hoverColor};">
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
            on:click={() => (activeTab = settingsTab.path)}
            style="--tab-color: {settingsTab.color}; --tab-hover-color: {settingsTab.hoverColor};"
          >
            <svelte:component this={settingsTab.icon} class="w-4 h-4 mr-2" />
            <span>{settingsTab.name}</span>
          </a>
        </div>
      </div>
    {/if}

    <main class="flex-1 overflow-auto p-4" style={!isPopupPage ? "background-color:" + activeTabBgColor + "; margin-top: -1px;" : ""}>
      <slot />
    </main>
  </div>
</div>

<style>
  /* Set the draggable area of the window */
  .titlebar {
    height: 3rem; /* Adjust the height as needed */
    user-select: none;
    cursor: grab;
    display: flex;
    justify-content: flex-end; /* Align items to the right for Windows/Linux */
    align-items: center;
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 9999;
    background-color: transparent;
  }

  .titlebar .drag-region {
    flex-grow: 1;
  }

  .titlebar:active {
    cursor: grabbing;
  }

  /* macOS style adjustments */
  @media (prefers-color-scheme: dark) {
    .titlebar.darwin button {
      /* Dark mode specific styles if needed */
    }
  }

  .tab-bar {
    margin-top: 3rem; /* Adjust to match the height of the title bar */
    padding-top: 0.5rem; /* Add some padding if needed */
    border-bottom: 1px solid #e5e7eb; /* Add a bottom border to separate from content */
  }

  .tab {
    padding: 0.5rem 1rem;
    border-radius: 0.375rem 0.375rem 0 0; /* Rounded top corners */
    display: flex;
    align-items: center;
    cursor: pointer;
    transition:
      background-color 0.2s ease-in-out,
      color 0.2s ease-in-out;
    background-color: var(--tab-color);
    color: #374151; /* Default text color */
    font-weight: 500;
  }

  .tab:hover {
    background-color: var(--tab-hover-color);
    color: #1f2937; /* Darker text on hover */
  }

  .tab.active {
    background-color: var(--tab-hover-color); /* Same as hover for active tab */
    color: #111827; /* Even darker text for active tab */
    font-weight: 600;
    position: relative;
  }

  /* Optional: Add a small line below the active tab to make it more prominent */
  .tab.active::after {
    content: "";
    position: absolute;
    bottom: -1px; /* Align with the border of tab-bar */
    left: 0;
    right: 0;
    height: 2px;
  }

  /* Ensure main content area respects the tab bar */
  main {
    padding-top: 1rem; /* Adjust this based on your tab bar's final height */
  }
</style>
