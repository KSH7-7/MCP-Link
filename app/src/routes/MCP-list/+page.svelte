<script lang="ts">
  import { onMount, onDestroy } from "svelte"
  import Search from "../../lib/components/search.svelte"
  import MCPCard from "../../lib/components/mcp-card.svelte"
  import { fetchMCPCards } from "../../lib/data/mcp-api"
  import { page } from "$app/stores"
  import { browser } from "$app/environment"
  import NotificationHandler from "./NotificationHandler.svelte"
  import { sharedDataStore, updateCount } from "$lib/stores/data-store"
  import { invoke } from "@tauri-apps/api/core"


  // define data type to receive from backend
  import type { MCPCard as MCPCardType, PageInfo } from "../../lib/data/mcp-api"

  // MCP card data
  let mcpCards: MCPCardType[] = []
  let pageInfo: PageInfo = { has_next_page: false, end_cursor: null, total_items: 0 }
  let mainElement: HTMLElement | null = null

  // data loading state
  let loading = true
  let loadingMore = false
  let allLoaded = false
  let justLoadedNewData = false

  // 중복 호출 방지를 위한 상태
  let isSearching = false
  let lastSearchTerm = ""
  let searchRequestId = 0
  let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null

  // Added variables
  let searchTermFromQuery = ""
  let isRecommendedSearch = false
  let processedKeywords = new Set<string>()
  let lastProcessedTime = 0

  // Scroll event handler
  function handleScroll() {
    if (!mainElement) {
      mainElement = document.querySelector("main.flex-1.overflow-y-auto")
      if (!mainElement) return
    }

    if (loadingMore || !pageInfo.has_next_page || allLoaded) {
      return
    }

    if (justLoadedNewData) {
      justLoadedNewData = false
      return
    }

    const scrollPosition = mainElement.scrollTop + mainElement.clientHeight
    const scrollHeight = mainElement.scrollHeight

    if (scrollHeight - scrollPosition < 300) {
      loadNextPage()
    }
  }

  // Function to load the next page
  async function loadNextPage() {
    if (!pageInfo.has_next_page || !pageInfo.end_cursor || loadingMore || allLoaded) {
      return
    }

    loadingMore = true
    try {
      await new Promise((resolve) => setTimeout(resolve, 100))

      const response = await fetchMCPCards(searchTermFromQuery || undefined, pageInfo.end_cursor)

      if (response.cards && response.cards.length > 0) {
        mcpCards = [...mcpCards, ...response.cards]
        pageInfo = response.page_info
        justLoadedNewData = true
      } else {
        pageInfo.has_next_page = false
      }

      if (!pageInfo.has_next_page || mcpCards.length >= pageInfo.total_items) {
        allLoaded = true
      } else {
        allLoaded = false
      }
    } catch (error) {
      console.error("[MCP-list] Error loading next page:", error)
    } finally {
      loadingMore = false
    }
  }

  // 검색 실행 함수 (개선된 버전)
  async function searchAndDisplay(term: string, scrollToTop: boolean = false) {
    // 공백 제거
    term = term.trim()
    
    // 동일한 검색어로 연속 호출 방지
    if (isSearching && term === lastSearchTerm) {
      console.log("검색 중복 방지: 이미 같은 검색어로 검색 중", term)
      return
    }
    
    // 검색 요청 ID 증가
    const currentRequestId = ++searchRequestId
    
    // 검색 상태 설정
    isSearching = true
    lastSearchTerm = term
    
    console.log(`검색 시작 (ID: ${currentRequestId}):`, term || "전체 목록")
    
    // UI 초기화
    loading = true
    allLoaded = false
    justLoadedNewData = false
    mcpCards = []
    pageInfo = { has_next_page: false, end_cursor: null, total_items: 0 }
    
    // 스크롤 위치 초기화
    if (scrollToTop && mainElement) {
      mainElement.scrollTo(0, 0)
    }
    
    try {
      // API 호출
      const response = await fetchMCPCards(term || undefined)
      
      // 최신 요청인지 확인
      if (currentRequestId !== searchRequestId) {
        console.log(`검색 결과 무시 (구 요청 ID: ${currentRequestId})`)
        return
      }
      
      // 결과 적용
      mcpCards = response.cards
      pageInfo = response.page_info
      justLoadedNewData = true
      
      // 전체 개수 업데이트
      if (!term) {
        updateCount("listCount", pageInfo.total_items)
      }
      
      // 페이지 로드 완료 체크
      if (!pageInfo.has_next_page || mcpCards.length >= pageInfo.total_items) {
        allLoaded = true
      } else {
        allLoaded = false
      }
      
      // 검색 완료 토스트
      if (term && isRecommendedSearch) {
        const toastEvent = new CustomEvent("show-toast", {
          detail: {
            message: `'${term}' 검색 완료: ${mcpCards.length}개`,
            type: mcpCards.length > 0 ? "success" : "info",
            duration: 2000,
          },
        })
        document.dispatchEvent(toastEvent)
      }
      
    } catch (error) {
      console.error("검색 오류:", error)
      mcpCards = []
      pageInfo = { has_next_page: false, end_cursor: null, total_items: 0 }
      allLoaded = true
      
      // 오류 토스트
      const toastEvent = new CustomEvent("show-toast", {
        detail: {
          message: "검색 중 오류가 발생했습니다",
          type: "error",
          duration: 3000,
        },
      })
      document.dispatchEvent(toastEvent)
      
    } finally {
      // 현재 요청이 최신 요청인 경우에만 상태 업데이트
      if (currentRequestId === searchRequestId) {
        loading = false
        isSearching = false
        console.log(`검색 완료 (ID: ${currentRequestId})`)
      }
    }
  }

  // Search event handler
  const handleSearch = (event: CustomEvent<{ value: string }>) => {
    const term = event.detail.value.trim()
    
    console.log("handleSearch 호출됨:", term)
    
    // 디바운스 타이머 취소
    if (searchDebounceTimer) {
      clearTimeout(searchDebounceTimer)
    }
    
    // 상태 업데이트
    searchTermFromQuery = term
    isRecommendedSearch = false
    
    // 300ms 디바운스 후 검색
    searchDebounceTimer = setTimeout(() => {
      searchAndDisplay(term, true)
    }, 300)
  }

  const handleClearSearch = () => {
    console.log("handleClearSearch 호출됨")
    
    // 디바운스 타이머 취소
    if (searchDebounceTimer) {
      clearTimeout(searchDebounceTimer)
    }
    
    searchTermFromQuery = ""
    isRecommendedSearch = false
    
    searchAndDisplay("", true)
  }

  // NotificationHandler props (개선된 버전)
  const notificationHandlerProps = {
    setSearchTerm: (keyword: string) => {
    console.log("setSearchTerm 호출됨:", keyword)
    
    // 이미 처리된 키워드인지 확인
    const now = Date.now()
    if (processedKeywords.has(keyword) && now - lastProcessedTime < 5000) {
      console.log("이미 처리된 키워드:", keyword)
      return
    }
    
    // 처리된 키워드로 표시
    processedKeywords.add(keyword)
    lastProcessedTime = now
    
    // 오래된 키워드 정리
    if (processedKeywords.size > 10) {
      processedKeywords.clear()
    }
    
    if (searchDebounceTimer) {
      clearTimeout(searchDebounceTimer)
    }
    
    searchTermFromQuery = keyword
    isRecommendedSearch = true
    
    const event = new CustomEvent("set-search-term", { detail: keyword })
    document.dispatchEvent(event)
    
    searchDebounceTimer = setTimeout(() => {
      searchAndDisplay(keyword, true)
    }, 300)
  },
}

  // get data when component is mounted
  onMount(() => {
    // 실제 스크롤이 발생하는 메인 요소 찾기
    mainElement = document.querySelector("main.flex-1.overflow-y-auto")

    if (mainElement) {
      mainElement.addEventListener("scroll", handleScroll)
    }

    // URL 파라미터에서 키워드 확인
    const urlParams = new URLSearchParams(window.location.search)
    const urlKeyword = urlParams.get("keyword")
    
    async function deleteKeywordFile() {
      try {
        await invoke("delete_keyword_file")
        console.log("키워드 파일 삭제됨")
      } catch (e) {
    console.log("키워드 파일 삭제 실패:", e)
  }
}

    if (urlKeyword) {
      console.log("MCP-list: URL에서 키워드 발견:", urlKeyword)
      searchTermFromQuery = urlKeyword
      isRecommendedSearch = urlParams.get("recommended") === "true"
      
      // 검색창 업데이트
      const event = new CustomEvent("set-search-term", { detail: urlKeyword })
      document.dispatchEvent(event)
      
      // 초기 로드 시에는 디바운스 없이 즉시 검색
      searchAndDisplay(urlKeyword, true)
    } else {
      // 키워드가 없으면 전체 목록 로드
      searchAndDisplay("", false)
      deleteKeywordFile()
    }

    // $page 스토어 구독
    const unsubscribePage = page.subscribe((currentPage) => {
      const urlKeyword = currentPage.url.searchParams.get("keyword")
      
      // URL 키워드가 변경되었고, 현재 검색 중이 아닐 때만 처리
      if (urlKeyword && urlKeyword !== searchTermFromQuery && !isSearching) {
        console.log("URL 키워드 변경 감지:", urlKeyword)
        
        searchTermFromQuery = urlKeyword
        isRecommendedSearch = !!currentPage.url.searchParams.get("recommended")
        
        // 검색창 업데이트
        const searchEvent = new CustomEvent("set-search-term", { detail: urlKeyword })
        document.dispatchEvent(searchEvent)
        
        // URL 변경은 즉시 검색
        searchAndDisplay(urlKeyword, true)
        deleteKeywordFile()
      }
    })

    return () => {
      if (mainElement) {
        mainElement.removeEventListener("scroll", handleScroll)
      }
      if (unsubscribePage) unsubscribePage()
      if (searchDebounceTimer) clearTimeout(searchDebounceTimer)
    }
  
  
    const checkAndCleanOldKeyword = async () => {
      try {
        const result = await invoke<{ keyword: string; age_ms: number } | null>("check_keyword_file_age")
        if (result && result.age_ms > 10000) {
          console.log("오래된 키워드 파일 감지, 삭제합니다:", result.keyword)
          await deleteKeywordFile()
        }
      } catch (e) {
        console.log("키워드 파일 체크 실패:", e)
      }
    }
    checkAndCleanOldKeyword()
 
  })

  // Cleanup
  onDestroy(() => {
    if (searchDebounceTimer) {
      clearTimeout(searchDebounceTimer)
    }
  })
</script>

<NotificationHandler {...notificationHandlerProps} />

<div class="container mx-auto pb-4">
  <!-- Top header area -->
  <div class="py-2 px-4 sticky top-0 z-10 bg-[var(--color-secondary)]">
    <div class="flex flex-col sm:flex-row justify-between items-center w-full px-4">
      <h1 class="text-2xl font-bold text-center sm:text-left sm:mr-auto">
        MCP List ({$sharedDataStore.loaded ? (searchTermFromQuery ? mcpCards.length : $sharedDataStore.counts.listCount || pageInfo.total_items) : pageInfo.total_items})
      </h1>

      <!-- Search UI -->
      <div class="relative w-full max-w-xs mx-auto sm:mx-0 sm:w-64 mt-2 sm:mt-0 sm:ml-auto">
        {#if isRecommendedSearch}
          <span class="absolute left-[-20px] top-3 text-yellow-500" title="Recommended Search">✨</span>
        {/if}

        <Search 
          initialValue={searchTermFromQuery} 
          placeholder="Search servers..." 
          customClass="input input-bordered w-full" 
          on:search={handleSearch} 
        />

        {#if loading && !mcpCards.length}
          <span class="loading loading-spinner loading-xs absolute right-3 top-3"></span>
        {/if}
      </div>
    </div>
  </div>

  <!-- Content area -->
  <div class="mt-3 px-4">
    <!-- Loading indicator -->
    {#if loading && mcpCards.length === 0 && !searchTermFromQuery}
      <div class="text-center py-10">
        <div class="flex flex-col items-center gap-4">
          <span class="loading loading-spinner loading-lg text-secondary"></span>
          <p>Loading MCP servers...</p>
        </div>
      </div>
    <!-- MCP cards grid -->
    {:else if mcpCards.length > 0}
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        {#each mcpCards as card (card.id)}
          <MCPCard {...card} mode="" />
        {/each}
      </div>

      <!-- Loading more indicator -->
      {#if loadingMore}
        <div class="flex justify-center items-center py-4">
          <span class="loading loading-sm loading-spinner text-secondary"></span>
        </div>
      {/if}

      <!-- All loaded message -->
      {#if allLoaded && !loadingMore}
        <div class="text-center py-4 text-gray-500">All MCPs loaded.</div>
      {/if}
    <!-- No results message -->
    {:else if searchTermFromQuery}
      <div class="text-center py-10 text-gray-500">
        <p>No search results for "{searchTermFromQuery}"</p>
        <button class="btn btn-sm btn-outline mt-3" on:click={handleClearSearch}>Delete search</button>
      </div>
    <!-- Empty state -->
    {:else}
      <div class="text-center py-10 text-gray-500">
        <p>No MCPs found.</p>
        <p class="mt-2">Search or filter to find MCPs.</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .container {
    max-width: 1200px;
  }

  :global(::-webkit-scrollbar) {
    width: 8px;
  }

  :global(::-webkit-scrollbar-track) {
    background: transparent;
  }

  :global(::-webkit-scrollbar-thumb) {
    background: #888;
    border-radius: 4px;
  }

  :global(::-webkit-scrollbar-thumb:hover) {
    background: #555;
  }
</style>