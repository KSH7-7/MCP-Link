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

  // 알림에서 검색 실행 함수
  function executeSearch(keyword: string) {
    if (!keyword) {
      return
    }

    // URL 업데이트
    try {
      goto(`/MCP-list?keyword=${encodeURIComponent(keyword)}`, { replaceState: false })
    } catch (e) {
      // 실패시 조용히 실패
    }

    // 토스트 메시지 표시 (사용자 피드백)
    try {
      const toastEvent = new CustomEvent("show-toast", {
        detail: {
          message: `'${keyword}' 키워드로 검색합니다`,
          type: "info",
          duration: 3000,
        },
      })
      document.dispatchEvent(toastEvent)
    } catch (e) {
      // 실패시 조용히 실패
    }

    // 키워드 로컬 스토리지에 저장 (앱 다시 시작 시 사용)
    // 중요: 현재 페이지가 이미 MCP-list이고 키워드가 URL에 있다면 저장하지 않음
    // 이미 검색을 실행 중인 경우를 고려
    if (window.location.pathname === "/MCP-list") {
      const urlParams = new URLSearchParams(window.location.search)
      const urlKeyword = urlParams.get("keyword")
      if (urlKeyword === keyword) {
        return
      }
    }

    // 세션 스토리지에 저장 (현재 세션에서만 유효하도록)
    sessionStorage.setItem("pendingSearchKeyword", keyword)

    // 로컬 스토리지에도 백업으로 저장 (타임스탬프도 함께 저장)
    const keywordData = {
      keyword: keyword,
      timestamp: Date.now(),
      used: false,
    }
    localStorage.setItem("lastNotificationKeyword", keyword)
  }

  onMount(async () => {
    // 이벤트 리스너 설정
    try {
      // navigate-to-mcp-list-with-keyword 이벤트 리스너
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

      // navigate-to 이벤트 리스너 (기본 탐색용)
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

          // 키워드가 없으면 일반 탐색
          goto(payload)
        }
      })

      // new-keywords 이벤트 리스너
      unlistenKeywords = await listen("new-keywords", (event) => {
        const keywords = event.payload

        if (Array.isArray(keywords) && keywords.length > 0) {
          // 첫 번째 키워드 사용
          executeSearch(keywords[0])
        } else {
        }
      })

      // URI 스킴 처리 리스너 설정
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

      // 이미 DOM이 로드된 경우를 위한 체크
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
    } catch (e) {
    }
  })

  onDestroy(() => {
    // 리스너 정리
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

<!-- 이 컴포넌트는 UI가 없고 이벤트 처리만 담당합니다 -->
