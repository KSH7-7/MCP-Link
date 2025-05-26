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
  // export const scrollableContainerKey = Symbol() // contexts.ts로 이동
  let mainElement: HTMLElement

  // --- Configuration for app appearance ---
  // Window bar (title bar) styling - 통일된 디자인을 위해 base-100 사용
  const windowBarBackgroundClass = "bg-base-100"
  const windowBarContentColorClass = "text-base-content"

  // Tab bar styling - 상단바와 통일
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
  let unlistenSearchKeyword: UnlistenFn | undefined // 새로운 search-keyword-event 리스너를 위한 변수
  let unlistenSearchKeywordEvent: UnlistenFn | undefined // search-keyword 이벤트 리스너 추가

  // --- Svelte reactive state ---
  let activeTabPath = "/"
  $: isFirstInstallPage = $page?.url?.pathname === "/first-install"
  $: isPopupPage = $page?.url?.pathname === "/popup"

  // wheel 이벤트 리스너 설정을 위한 reactive 블록
  $: if (mainElement && browser) {
    setContext(scrollableContainerKey, mainElement)
    // wheel 이벤트 리스너 추가
    mainElement.addEventListener("wheel", handleWheel, { passive: true })
    mainElement.addEventListener("touchstart", handleWheel, { passive: true })
  }

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
      // 스크롤 위치를 즉시 상단으로 애니메이션 적용
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
        console.error("[FRONTEND][Notification] Navigation error:", err)

        // Attempt to force a path change even if an error occurs
        window.location.href = targetUrl
      }
    } catch (err) {
      console.error("[FRONTEND] Error processing keyword:", err)
    }
  }

  // --- Lifecycle and Subscriptions ---
  onMount(async () => {
    // 개선된 토스트 시스템 초기화
    if (browser) {
      initToastSystem()

      // 단순화된 이벤트 리스너 - 활성 탭 경로만 업데이트
      window.addEventListener("navigate-to-event", ((event: CustomEvent) => {
        if (event.detail && event.detail.path) {
          // 이벤트로 받은 활성 탭 경로 설정
          activeTabPath = event.detail.path
        }
      }) as EventListener)

      // 데이터 스토어 초기화
      setLoaded(false)

      // 페이지 전환 간에 공유될 데이터 로드
      try {
        // 여기서 앱에 필요한 공통 데이터를 로드
        // 예: 설치된 MCP 수, MCP 리스트 수 등의 데이터

        // 예시 데이터 로드 (실제로는 invoke 등을 통해 데이터를 가져와야 함)
        const installedCount = await invoke("get_installed_count").catch(() => 0)
        const listCount = await invoke("get_list_count").catch(() => 0)

        updateCount("installedCount", installedCount as number)
        updateCount("listCount", listCount as number)

        // 데이터 로드 완료 표시
        setLoaded(true)
      } catch (error) {
        console.error("데이터 로드 중 오류:", error)
        // 데이터 로드 실패해도 앱은 계속 동작하도록 로드 완료 표시
        setLoaded(true)
      }

      // URI 스킴 프로토콜 핸들러는 deep-link 플러그인으로 대체됨
      // 이벤트 리스너로 처리하는 방식으로 변경
      unlistenSearchKeyword = await listen("search-keyword-event", async (event) => {
        const keyword = event.payload as string
        await handleKeywordSearch(keyword)
      })

      // 'search-keyword' 이벤트 리스너 추가
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
        console.error("[Layout] OS detection error:", e)
        currentPlatform = "unknown"
      }
    }

    if (typeof window !== "undefined" && "__TAURI__" in window) {
      try {
        // 윈도우 핸들 초기화를 더 안정적으로 처리
        tauriWindow = WebviewWindow.getCurrent()
        
        // 초기화 실패 시 재시도
        if (!tauriWindow) {
          console.warn("초기 윈도우 핸들 가져오기 실패, 재시도 중...")
          setTimeout(() => {
            try {
              tauriWindow = WebviewWindow.getCurrent()
              if (tauriWindow) {
                console.log("윈도우 핸들 재시도 성공")
              }
            } catch (e) {
              console.error("윈도우 핸들 재시도 실패:", e)
            }
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
              const payload = event.payload as { claudeConfigExists?: boolean; mcplinkConfigExists?: boolean } // 타입 단언 추가
              const { claudeConfigExists, mcplinkConfigExists } = payload

              // If any config file is missing, redirect to first-install page
              if (claudeConfigExists === false || mcplinkConfigExists === false) {
                // 명시적으로 false 비교
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

    // 진행 중인 스크롤 애니메이션이 있다면 중단
    if (currentScrollAnimation) {
      currentScrollAnimation.kill()
      currentScrollAnimation = null
    }

    // wheel 및 touch 이벤트 리스너 제거
    if (browser && mainElement) {
      mainElement.removeEventListener("wheel", handleWheel)
      mainElement.removeEventListener("touchstart", handleWheel)
    }

    // 커스텀 이벤트 리스너 제거
    if (browser) {
      // 이벤트 리스너 제거 시 빈 함수로 제거하면 실제로 제거되지 않음
      // 대신 null 참조 지정으로 가비지 컬렉션에 의해 자연스럽게 정리되도록 함
      // 이벤트 리스너는 앱 종료 시에만 정리되므로 실제로 여기서 명시적 제거는 불필요함
    }
  })

  // 진행 중인 스크롤 애니메이션 참조를 저장할 변수
  let currentScrollAnimation: gsap.core.Tween | null = null

  // 스크롤 위치를 확인하고 스크롤 애니메이션을 실행하는 함수
  function smoothScrollToTop() {
    if (!mainElement) return

    // 현재 스크롤 위치가 1픽셀 이상인 경우에만 애니메이션 적용
    if (mainElement.scrollTop > 1) {
      // 이미 진행 중인 애니메이션이 있으면 중단
      if (currentScrollAnimation) {
        currentScrollAnimation.kill()
        currentScrollAnimation = null
      }

      // GSAP을 사용하여 부드러운 스크롤 애니메이션 적용
      currentScrollAnimation = gsap.to(mainElement, {
        scrollTop: 0,
        duration: 0.6, // 애니메이션 지속 시간 (초)
        ease: "power2.out", // 가속도 곡선 (easing)
        onComplete: () => {
          // 애니메이션 완료 시 참조 제거
          currentScrollAnimation = null
        },
      })
    }
  }

  // 마우스 휠 이벤트 시 애니메이션 중단 함수
  function handleWheel() {
    if (currentScrollAnimation) {
      currentScrollAnimation.kill()
      currentScrollAnimation = null
    }
  }

  // --- Window control functions (개선된 버전) ---
  async function minimizeWindow() {
    try {
      // tauriWindow가 없으면 현재 창 직접 가져오기
      const window = tauriWindow || WebviewWindow.getCurrent()
      if (window) {
        await window.minimize()
      } else {
        console.error("창을 찾을 수 없어 최소화할 수 없습니다")
        showToast("창을 최소화할 수 없습니다", "error")
      }
    } catch (e) {
      console.error("창 최소화 오류:", e)
      showToast("창 최소화 중 오류가 발생했습니다", "error")
    }
  }
  
  async function maximizeWindow() {
    try {
      // tauriWindow가 없으면 현재 창 직접 가져오기
      const window = tauriWindow || WebviewWindow.getCurrent()
      if (window) {
        const isMaximized = await window.isMaximized()
        if (isMaximized) {
          await window.unmaximize()
        } else {
          await window.maximize()
        }
      } else {
        console.error("창을 찾을 수 없어 최대화할 수 없습니다")
        showToast("창을 최대화할 수 없습니다", "error")
      }
    } catch (e) {
      console.error("창 최대화 오류:", e)
      showToast("창 최대화 중 오류가 발생했습니다", "error")
    }
  }
  
  async function hideToTray() {
    try {
      // tauriWindow가 없으면 현재 창 직접 가져오기
      const window = tauriWindow || WebviewWindow.getCurrent()
      if (window) {
        await window.hide()
      } else {
        console.error("창을 찾을 수 없어 숨길 수 없습니다")
        showToast("창을 숨길 수 없습니다", "error")
      }
    } catch (e) {
      console.error("창 숨기기 오류:", e)
      showToast("창 숨기기 중 오류가 발생했습니다", "error")
    }
  }

  // --- Reactive computations for styling ---
  $: currentActivePageConfig = (() => {
    if (activeTabPath === settingsTab.path) return settingsTab

    // 정확히 일치하는 탭 찾기
    const foundTab = tabs.find((t) => activeTabPath === t.path)
    return foundTab || tabs.find((t) => t.path === "/Installed-MCP") || tabs[0] // 기본값
  })()

  // 배경색과 글자색을 활성화된 탭에 따라 반응형으로 계산
  $: activeMainAreaBackgroundColor = isFirstInstallPage || isPopupPage ? "var(--color-base-100)" : `var(--${currentActivePageConfig.mainClass.replace("bg-", "color-")})`

  $: activeMainAreaContentColor = isFirstInstallPage || isPopupPage ? "var(--color-base-content)" : `var(--${currentActivePageConfig.mainContentClass.replace("text-", "color-")})`

  // 윈도우 컨트롤 버튼 스타일은 CSS에서 처리
</script>

<!-- Outermost container. If an overall app background different from main content is needed, apply it here. -->
<!-- For now, it's just a flex container. -->
<div class="flex flex-col h-screen overflow-hidden">
  <!-- Top Area: Title Bar and Tab Bar container -->
  {#if !isPopupPage}
    <!-- Window Title Bar -->
    <div class="{windowBarBackgroundClass} {windowBarContentColorClass}">
      <div class="h-8 flex items-center text-xs select-none" data-tauri-drag-region>
        <!-- 왼쪽 공간 -->
        <div class="w-[100px]"></div>

        <!-- 중앙 제목 -->
        <div class="absolute left-0 right-0 mx-auto flex justify-center items-center" data-tauri-drag-region>
          <img src="/favicon.png" alt="App Icon" class="w-4 h-4 mr-2" />
          <slot name="title">MCPLINK</slot>
        </div>

        <!-- 투명 버튼 위에 아이콘을 장식으로 배치 -->
        <div class="ml-auto flex">
          <!-- 아이콘은 버튼 내부에 절대 위치로 배치하여 장식으로 처리 -->
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
                // 탭 클릭 시 다른 모든 탭의 활성화 상태를 초기화하고 현재 탭만 활성화
                activeTabPath = tab.path
                // 부드러운 스크롤 애니메이션 적용
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
              // 설정 탭 클릭 시 모든 탭 초기화하고 설정 탭만 활성화
              activeTabPath = settingsTab.path
              // 부드러운 스크롤 애니메이션 적용
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
  /* 탭 기본 스타일 */
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
    min-width: 150px; /* "Installed MCP"보다 살짝 넓은 너비 */
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

  /* 윈도우 컨트롤 버튼 스타일 */
  .window-btn {
    width: 46px;
    height: 32px;
    background-color: transparent;
    cursor: default; /* 일반 마우스 커서 유지 */
    position: relative; /* 아이콘의 기준점 */
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
    pointer-events: none; /* 아이콘은 이벤트를 처리하지 않음 */
  }

  .min-btn:hover,
  .max-btn:hover {
    background-color: oklch(var(--color-base-300)) !important; /* base-300 색상 */
  }

  .close-btn:hover {
    background-color: oklch(var(--color-error)) !important; /* error 색상 */
  }

  /* 모든 SVG 아이콘에 대해 포인터 이벤트 차단 */
  svg {
    pointer-events: none;
  }
</style>
