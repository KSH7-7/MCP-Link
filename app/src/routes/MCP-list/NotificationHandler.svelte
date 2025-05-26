<script lang="ts">
  import { onMount, onDestroy } from "svelte"
  import { goto } from "$app/navigation"
  import { listen } from "@tauri-apps/api/event"
  import { invoke } from "@tauri-apps/api/core"
  
  // 부모 컴포넌트로부터 받은 props
  export let setSearchTerm: ((keyword: string) => void) | undefined = undefined

  let unlistenNotificationClick: (() => void) | null = null
  let unlistenSearchKeyword: (() => void) | null = null
  let unlistenAppActivated: (() => void) | null = null
  let processingKeyword = false
  let lastProcessedKeyword = ""
  let globalLastProcessTime = 0
  
  // 개선된 중복 방지 시스템
  let processedKeywords = new Map<string, number>() // 키워드별 마지막 처리 시간
  let sessionStartTime = Date.now() // 세션 시작 시간
  let isInitialLoad = true // 초기 로드 여부

  // 키워드 처리 함수 (강화된 중복 방지 적용)
  async function handleKeywordSearch(keyword: string, fromNotification: boolean = false) {
    if (!keyword || !keyword.trim()) {
      console.log("NotificationHandler: 빈 키워드")
      return
    }
    
    keyword = keyword.trim()
    const now = Date.now()
    
    // 강화된 중복 처리 방지
    const lastProcessTime = processedKeywords.get(keyword) || 0
    const timeSinceLastProcess = now - lastProcessTime
    const timeSinceSessionStart = now - sessionStartTime
    
    // 1. 현재 처리 중인 경우
    if (processingKeyword) {
      console.log("NotificationHandler: 이미 키워드 처리 중")
      return
    }
    
    // 2. 같은 키워드를 최근에 처리한 경우 (5분 이내)
    if (timeSinceLastProcess < 300000) { // 5분 = 300초
      console.log("NotificationHandler: 최근에 처리된 키워드", { 
        keyword, 
        timeSinceLastProcess: Math.round(timeSinceLastProcess / 1000) + "초 전"
      })
      return
    }
    
    // 3. 초기 로드 후 즉시 app-activated가 온 경우 (자동 이벤트)
    if (!fromNotification && timeSinceSessionStart < 10000 && isInitialLoad) {
      console.log("NotificationHandler: 초기 로드 중 자동 이벤트 무시")
      return
    }

    try {
      processingKeyword = true
      lastProcessedKeyword = keyword
      globalLastProcessTime = now
      processedKeywords.set(keyword, now)
      isInitialLoad = false
      
      console.log("NotificationHandler: 키워드 처리 시작", keyword, "알림에서:", fromNotification)

      // 알림에서 온 경우 앱 활성화 시도
      if (fromNotification) {
        try {
          const result = await invoke("activate_app_window")
          console.log("NotificationHandler: 앱 활성화 결과", result)
        } catch (e) {
          console.error("NotificationHandler: 앱 활성화 실패", e)
        }
      }

      // 1. 부모 컴포넌트의 setSearchTerm 함수가 있으면 사용
      if (setSearchTerm) {
        console.log("NotificationHandler: 부모의 setSearchTerm 호출")
        setSearchTerm(keyword)
        
        // 키워드 처리 완료 후 상태 정리
        await clearKeywordState()
      } else {
        // 2. setSearchTerm이 없는 경우 직접 페이지 이동
        console.log("NotificationHandler: 직접 페이지 이동")
        await goto(`/MCP-list?keyword=${encodeURIComponent(keyword)}&recommended=true`)
        
        // 키워드 처리 완료 후 상태 정리
        await clearKeywordState()
      }

      // 3. 토스트 메시지 표시
      const toastEvent = new CustomEvent("show-toast", {
        detail: {
          message: `'${keyword}' 검색 중...`,
          type: "info",
          duration: 2000,
        },
      })
      document.dispatchEvent(toastEvent)

    } catch (error) {
      console.error("NotificationHandler: 키워드 처리 오류", error)
      
      const toastEvent = new CustomEvent("show-toast", {
        detail: {
          message: "키워드 처리 중 오류가 발생했습니다",
          type: "error",
          duration: 3000,
        },
      })
      document.dispatchEvent(toastEvent)
      
    } finally {
      setTimeout(() => {
        processingKeyword = false
      }, 1000) // 1초 후 처리 완료로 마크
    }
  }
  
  // 키워드 상태 완전 정리 함수
  async function clearKeywordState() {
    try {
      // 1. 파일 삭제
      await deleteKeywordFile()
      
      // 2. Rust 상태 정리 
      await invoke("clear_keyword_state")
      
      console.log("NotificationHandler: 키워드 상태 완전 정리 완료")
    } catch (e) {
      console.log("NotificationHandler: 키워드 상태 정리 중 오류 (무시)", e)
    }
  }
  
  // 키워드 파일 삭제 함수
  async function deleteKeywordFile() {
    try {
      const path = await import("@tauri-apps/api/path")
      const fs = await import("@tauri-apps/plugin-fs")
      
      const tempDir = await path.tempDir()
      const keywordPath = `${tempDir}mcplink_last_keyword.txt`
      
      await fs.remove(keywordPath)
      console.log("NotificationHandler: 키워드 파일 삭제됨")
    } catch (e) {
      console.log("NotificationHandler: 키워드 파일 삭제 실패 (무시)", e)
    }
  }

  onMount(async () => {
    console.log("NotificationHandler: 마운트됨")
    sessionStartTime = Date.now()
    isInitialLoad = true

    // 1. notification-clicked 이벤트 리스너 (알림 전용)
    unlistenNotificationClick = await listen("notification-clicked", async (event) => {
      const keyword = event.payload as string
      console.log("NotificationHandler: notification-clicked 이벤트 수신", keyword)
      await handleKeywordSearch(keyword, true) // 알림에서 온 것임을 표시
    })

    // 2. search-keyword 이벤트 리스너 (기존 호환성)
    unlistenSearchKeyword = await listen("search-keyword", async (event) => {
      const keyword = event.payload as string
      console.log("NotificationHandler: search-keyword 이벤트 수신", keyword)
      await handleKeywordSearch(keyword, false)
    })

    // 3. app-activated 이벤트 리스너 (개선된 버전)
    unlistenAppActivated = await listen("app-activated", async (event) => {
      console.log("NotificationHandler: app-activated 이벤트 수신", event)
      
      // 이벤트 페이로드에서 알림 여부 확인
      const payload = event.payload as any
      const fromNotification = payload?.fromNotification || false
      
      // 자동 이벤트는 초기 로드 시에만 무시하고, 이후에는 처리
      const timeSinceSessionStart = Date.now() - sessionStartTime
      if (!fromNotification && isInitialLoad && timeSinceSessionStart < 5000) {
        console.log("NotificationHandler: 초기 자동 이벤트 무시")
        isInitialLoad = false // 첫 자동 이벤트 후 플래그 해제
        return
      }
      
      // 키워드 파일 확인 (알림에서 온 경우만)
      if (fromNotification) {
        try {
          const fs = await import("@tauri-apps/plugin-fs")
          const path = await import("@tauri-apps/api/path")
          const tempDir = await path.tempDir()
          const keywordPath = `${tempDir}mcplink_last_keyword.txt`
          
          try {
            const keyword = await fs.readTextFile(keywordPath)
            if (keyword && keyword.trim()) {
              console.log("NotificationHandler: app-activated에서 키워드 발견", keyword)
              await handleKeywordSearch(keyword.trim(), true)
            }
          } catch (e) {
            // 파일이 없으면 무시
          }
        } catch (e) {
          console.error("NotificationHandler: app-activated 키워드 확인 오류", e)
        }
      }
    })

    // 4. 앱 시작 시 키워드 파일 확인 (한 번만, 조건부)
    setTimeout(async () => {
      try {
        const fs = await import("@tauri-apps/plugin-fs")
        const path = await import("@tauri-apps/api/path")
        const tempDir = await path.tempDir()
        const keywordPath = `${tempDir}mcplink_last_keyword.txt`
        
        try {
          const keyword = await fs.readTextFile(keywordPath)
          if (keyword && keyword.trim()) {
            console.log("NotificationHandler: 시작 시 파일에서 키워드 발견", keyword)
            
            // 파일 생성 시간 확인
            try {
              const stat = await fs.stat(keywordPath)
              const mtime = stat.mtime
              const fileTime = mtime instanceof Date ? mtime.getTime() : (typeof mtime === 'number' ? mtime : 0)
              const fileAge = Date.now() - fileTime
              
              // 30초 이내의 파일만 처리
              if (fileAge < 30000) {
                await handleKeywordSearch(keyword.trim(), true) // 알림에서 온 것으로 처리
              } else {
                console.log("NotificationHandler: 오래된 키워드 파일 무시", fileAge)
                await fs.remove(keywordPath)
              }
            } catch (e) {
              // stat 실패 시 그냥 처리
              await handleKeywordSearch(keyword.trim(), true)
            }
          }
        } catch (e) {
          // 파일이 없으면 무시
        }
      } catch (e) {
        console.error("NotificationHandler: 초기 키워드 확인 오류", e)
      }
    }, 1000) // 1초 후에 확인
  })

  onDestroy(() => {
    console.log("NotificationHandler: 언마운트됨")
    if (unlistenNotificationClick) unlistenNotificationClick()
    if (unlistenSearchKeyword) unlistenSearchKeyword()
    if (unlistenAppActivated) unlistenAppActivated()
  })
</script>

<!-- UI가 없는 컴포넌트 -->