<script lang="ts">
  import { onMount, onDestroy } from "svelte"
  import { page } from "$app/stores"
  import { goto } from "$app/navigation"
  import { useNotifications } from "./toast-handler.js"
  import { listen } from "@tauri-apps/api/event"

  // 알림 핸들러 초기화
  const { showNotification, triggerSearch } = useNotifications()

  let unlistenNavigate: (() => void) | null = null
  let unlistenKeywords: (() => void) | null = null
  let unlistenBasicNavigate: (() => void) | null = null

  // Search execution function from notification
  function executeSearch(keyword: string) {
    if (!keyword) {
      return
    }

    // URL update
    try {
      goto(`/MCP-list?keyword=${encodeURIComponent(keyword)}`, { replaceState: false })
    } catch (e) {}

    // Show toast message (user feedback)
    try {
      const toastEvent = new CustomEvent("show-toast", {
        detail: {
          message: `'${keyword}' 키워드로 검색합니다`,
          type: "info",
          duration: 3000,
        },
      })
      document.dispatchEvent(toastEvent)
    } catch (e) {}

    // Save keyword to local storage (used when app restarts)
    // Important: Do not save if the current page is already MCP-list and the keyword is in the URL
    // Consider already running search
    if (window.location.pathname === "/MCP-list") {
      const urlParams = new URLSearchParams(window.location.search)
      const urlKeyword = urlParams.get("keyword")
      if (urlKeyword === keyword) {
        return
      }
    }

    // Save to session storage (valid only for current session)
    sessionStorage.setItem("pendingSearchKeyword", keyword)

    // Also save to local storage as a backup (with timestamp)
    const keywordData = {
      keyword: keyword,
      timestamp: Date.now(),
      used: false,
    }
    localStorage.setItem("lastNotificationKeyword", keyword)
  }

  onMount(async () => {
    // Set event listeners
    try {
      // navigate-to-mcp-list-with-keyword event listener
      unlistenNavigate = await listen("navigate-to-mcp-list-with-keyword", (event) => {
        const payload = event.payload

        if (typeof payload === "string" && payload.includes("keyword=")) {
          const urlParams = new URLSearchParams(payload.split("?")[1])
          const keyword = urlParams.get("keyword")
          if (keyword) {
            executeSearch(keyword)
          }
        } else {
        }
      })

      // navigate-to event listener (for basic navigation)
      const unlistenBasicNavigate = await listen("navigate-to", (event) => {
        const payload = event.payload

        if (typeof payload === "string") {
          if (payload.includes("keyword=")) {
            const urlParams = new URLSearchParams(payload.split("?")[1])
            const keyword = urlParams.get("keyword")
            if (keyword) {
              executeSearch(keyword)
              return
            }
          }

          // If there is no keyword, perform general navigation
          goto(payload)
        }
      })

      // new-keywords event listener
      unlistenKeywords = await listen("new-keywords", (event) => {
        const keywords = event.payload

        if (Array.isArray(keywords) && keywords.length > 0) {
          // Use the first keyword
          executeSearch(keywords[0])
        } else {
        }
      })

      // URI scheme processing listener
      window.addEventListener("DOMContentLoaded", () => {
        const uri = localStorage.getItem("pendingUriScheme")

        if (uri) {
          localStorage.removeItem("pendingUriScheme")

          if (uri.includes("keyword=")) {
            const parts = uri.split("keyword=")
            if (parts.length > 1) {
              const keyword = parts[1].split(/[?&]/)[0] // ? 또는 & 이후 부분 제거
              executeSearch(keyword)
            }
          }
        } else {
        }
      })

      // Check if DOM is already loaded
      if (document.readyState === "complete") {
        const uri = localStorage.getItem("pendingUriScheme")
        if (uri) {
          localStorage.removeItem("pendingUriScheme")

          if (uri.includes("keyword=")) {
            const parts = uri.split("keyword=")
            if (parts.length > 1) {
              const keyword = parts[1].split(/[?&]/)[0]
              executeSearch(keyword)
            }
          }
        }
      }
    } catch (e) {}
  })

  onDestroy(() => {
    // Clean up listeners
    if (unlistenNavigate) {
      unlistenNavigate()
    }
    if (unlistenKeywords) {
      unlistenKeywords()
    }
    if (typeof unlistenBasicNavigate === "function") {
      unlistenBasicNavigate()
    }
  })
</script>

<!-- This component only handles event processing and has no UI -->
