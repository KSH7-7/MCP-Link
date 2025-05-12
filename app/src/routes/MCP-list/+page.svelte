<script lang="ts">
  import { onMount } from "svelte"
  import Search from "../../lib/components/search.svelte"
  import MCPCard from "../../lib/components/mcp-card.svelte"
  import { fetchMCPCards } from "../../lib/data/mcp-api"
  import { page } from "$app/stores"
  import { listen } from "@tauri-apps/api/event"
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow"

  // define data type to receive from backend
  import type { MCPCard as MCPCardType, PageInfo } from "../../lib/data/mcp-api"

  // MCP card data
  let mcpCards: MCPCardType[] = []
  let pageInfo: PageInfo = { hasNextPage: false, endCursor: null, totalItems: 0 }

  // data loading state
  let loading = true
  let loadingMore = false
  let allLoaded = false // 모든 데이터가 로드되었는지 확인하는 플래그 추가

  // START: 추가된 변수
  let searchTermFromQuery = "" // URL에서 가져온 검색어를 저장할 변수
  let isRecommendedSearch = false // 추천 검색 여부 상태 변수
  // END: 추가된 변수

  // 스크롤 이벤트 핸들러
  function handleScroll() {
    if (loadingMore || !pageInfo.hasNextPage || allLoaded) return

    // 문서 전체의 스크롤 위치 확인
    const scrollPosition = window.innerHeight + window.scrollY
    const scrollHeight = document.body.offsetHeight

    // 스크롤이 하단에서 300px 위치에 도달하면 다음 페이지 로드 (더 빠르게 감지)
    if (scrollHeight - scrollPosition < 300) {
      console.log("스크롤 하단 근처 감지: 다음 페이지 로드")
      loadNextPage()
    }
  }

  // 다음 페이지 로드 함수
  async function loadNextPage() {
    if (!pageInfo.hasNextPage || !pageInfo.endCursor || loadingMore || allLoaded) return

    loadingMore = true
    try {
      console.log(`다음 페이지 로드 시작: cursorId=${pageInfo.endCursor}`)
      const response = await fetchMCPCards(searchTermFromQuery || undefined, pageInfo.endCursor)
      console.log(`다음 페이지 로드 완료: ${response.cards.length}개 항목 추가`)

      // 기존 카드에 새 카드 추가
      mcpCards = [...mcpCards, ...response.cards]
      pageInfo = response.pageInfo

      // 모든 데이터를 로드했는지 확인
      if (!pageInfo.hasNextPage || mcpCards.length >= pageInfo.totalItems) {
        console.log("모든 데이터 로드 완료")
        allLoaded = true
      }
    } catch (error) {
      console.error("다음 페이지 로드 중 오류 발생:", error)
    } finally {
      loadingMore = false
    }
  }

  // 페이지 초기화 및 데이터 로드 함수
  function initPage() {
    // 전역 스크롤 이벤트 리스너 추가 (디바운스 적용)
    let scrollTimer: ReturnType<typeof setTimeout> | null = null
    const scrollHandler = () => {
      if (scrollTimer) clearTimeout(scrollTimer)
      scrollTimer = setTimeout(handleScroll, 100)
    }

    window.addEventListener("scroll", scrollHandler)

    // 초기 로드 후 페이지 하단 확인
    setTimeout(() => {
      handleScroll()
    }, 500)

    return scrollHandler
  }

  // get data when component is mounted
  onMount(async () => {
    // START: 메인 윈도우 이벤트 리스너 추가 (네비게이션 및 중앙 이동)
    const unlistenNavigate = await listen("navigate-to-mcp-list-with-keyword", async (event) => {
      const newUrl = event.payload as string
      if (newUrl && typeof newUrl === "string") {
        const url = new URL(newUrl, window.location.origin) // Full URL로 만듬
        const keyword = url.searchParams.get("keyword")
        if (keyword) {
          searchTermFromQuery = keyword
          isRecommendedSearch = true
          await searchAndDisplay(keyword)
        } else {
          searchTermFromQuery = "" // 키워드가 없으면 초기화
          isRecommendedSearch = false
        }
      }
    })

    // START: URL query parameter에서 keyword 읽어오기
    const unsubscribePage = page.subscribe((p) => {
      console.log("[MCP-list] $page store 구독 콜백 실행, URL:", p.url.toString())
      const keyword = p.url.searchParams.get("keyword")
      console.log("[MCP-list] URL에서 추출한 keyword:", keyword)
      if (keyword) {
        if (searchTermFromQuery !== keyword) {
          // 이벤트보다 늦게 실행되거나, 직접 URL 접근 시
          searchTermFromQuery = keyword
          isRecommendedSearch = true
          console.log("[MCP-list] searchTermFromQuery 할당됨:", searchTermFromQuery)
          searchAndDisplay(searchTermFromQuery)
        }
      } else {
        console.log("[MCP-list] URL에 keyword 없음, 전체 목록 로드 시도")
        if (searchTermFromQuery !== "") {
          // 이전 검색어가 있었다면 초기화
          searchTermFromQuery = ""
        }
        isRecommendedSearch = false
        fetchAllMCPs()
      }
    })
    // END: URL query parameter에서 keyword 읽어오기

    // 스크롤 이벤트 초기화
    const scrollHandler = initPage()

    // 컴포넌트 파괴 시 리스너 및 구독 해제
    return () => {
      unlistenNavigate()
      unsubscribePage()
      window.removeEventListener("scroll", scrollHandler)
    }
  })

  // 페이지 변경 감지 및 재초기화
  $: {
    if ($page) {
      console.log("[MCP-list] 페이지 변경 감지:", $page.url.pathname)
      if ($page.url.pathname === "/MCP-list") {
        console.log("[MCP-list] MCP-list 페이지 감지, 스크롤 이벤트 재초기화")
        setTimeout(() => {
          handleScroll()
        }, 500)
      }
    }
  }

  // START: 검색 실행 및 결과 표시 함수
  async function searchAndDisplay(term: string) {
    if (!term) {
      isRecommendedSearch = false // 검색어가 없으면 추천 상태도 해제
      return fetchAllMCPs() // 용어가 없으면 전체 로드
    }
    // isRecommendedSearch는 term이 있을 때만 true일 수 있으므로, 여기서는 변경하지 않음
    loading = true
    allLoaded = false // 검색 시 allLoaded 초기화
    try {
      const response = await fetchMCPCards(term)
      mcpCards = response.cards
      pageInfo = response.pageInfo
      console.log(`검색 결과: ${mcpCards.length}개 항목, 총 ${pageInfo.totalItems}개 중`)
      console.log(`다음 페이지 여부: ${pageInfo.hasNextPage}, 다음 커서: ${pageInfo.endCursor}`)

      // 모든 데이터를 로드했는지 확인
      if (!pageInfo.hasNextPage || mcpCards.length >= pageInfo.totalItems) {
        console.log("모든 데이터 로드 완료")
        allLoaded = true
      } else {
        // 초기 로드 후 스크롤 확인
        setTimeout(handleScroll, 500)
      }
    } catch (error) {
      console.error("검색 중 오류 발생:", error)
      mcpCards = []
      pageInfo = { hasNextPage: false, endCursor: null, totalItems: 0 }
      allLoaded = true
    } finally {
      loading = false
      // 페이지를 맨 위로 스크롤
      window.scrollTo(0, 0)
    }
  }
  // END: 검색 실행 및 결과 표시 함수

  // START: 모든 MCP 카드 가져오는 함수
  async function fetchAllMCPs() {
    isRecommendedSearch = false // 전체 목록 조회 시 추천 상태 해제
    loading = true
    allLoaded = false // 새로운 데이터 로드 시 allLoaded 초기화
    try {
      const response = await fetchMCPCards()
      mcpCards = response.cards
      pageInfo = response.pageInfo
      console.log(`전체 목록: ${mcpCards.length}개 항목, 총 ${pageInfo.totalItems}개 중`)
      console.log(`다음 페이지 여부: ${pageInfo.hasNextPage}, 다음 커서: ${pageInfo.endCursor}`)

      // 모든 데이터를 로드했는지 확인
      if (!pageInfo.hasNextPage || mcpCards.length >= pageInfo.totalItems) {
        console.log("모든 데이터 로드 완료")
        allLoaded = true
      } else {
        // 초기 로드 후 스크롤 확인
        setTimeout(handleScroll, 500)
      }
    } catch (error) {
      console.error("MCP 데이터를 가져오는 중 오류 발생:", error)
      mcpCards = []
      pageInfo = { hasNextPage: false, endCursor: null, totalItems: 0 }
      allLoaded = true
    } finally {
      loading = false
      // 페이지를 맨 위로 스크롤
      window.scrollTo(0, 0)
    }
  }
  // END: 모든 MCP 카드 가져오는 함수

  // Search event handler
  async function handleSearchEvent(event: CustomEvent<{ value: string }>) {
    const searchTerm = event.detail.value
    // 사용자가 직접 검색어를 입력/변경하여 검색했으므로 추천 상태 해제
    if (isRecommendedSearch && searchTerm !== searchTermFromQuery) {
      isRecommendedSearch = false
    }
    isRecommendedSearch = false

    // URL을 업데이트하지 않으므로, searchTermFromQuery는 변경하지 않음.
    // searchAndDisplay는 현재 검색창의 값(searchTerm)으로 검색.
    await searchAndDisplay(searchTerm)
  }
</script>

<div class="p-8">
  <!-- 고정 헤더 영역 - 배경색을 탭 색상과 동일하게 설정 -->
  <div class="sticky top-0 z-10 pb-4">
    <div class="flex justify-between items-center">
      <p class="text-xl font-semibold">Search results ({pageInfo.totalItems} MCPs)</p>
      <div class="search-area-wrapper flex items-center ml-auto">
        {#if isRecommendedSearch}
          <span class="mr-2 text-yellow-500" title="추천 검색">✨</span>
        {/if}
        <div class="search-component-wrapper w-72 rounded-[10px]">
          {#key searchTermFromQuery}
            <Search on:search={handleSearchEvent} initialValue={searchTermFromQuery} />
          {/key}
        </div>
      </div>
    </div>
  </div>

  <!-- 콘텐츠 영역 -->
  <div class="mt-2">
    {#if loading}
      <div class="flex justify-center items-center h-64">
        <span class="loading loading-spinner loading-lg text-primary"></span>
      </div>
    {:else if mcpCards.length === 0}
      <div class="flex justify-center items-center h-64">
        <p class="text-gray-500">No search results found.</p>
      </div>
    {:else}
      <div class="grid grid-cols-1 lg:grid-cols-1 gap-2">
        {#each mcpCards as card (card.id)}
          <MCPCard id={card.id} title={card.title} description={card.description} url={card.url} stars={card.stars} />
        {/each}

        <!-- 로딩 표시기 -->
        {#if loadingMore}
          <div class="flex justify-center items-center py-4">
            <span class="loading loading-spinner loading-md text-primary"></span>
          </div>
        {/if}

        <!-- 더 이상 데이터가 없을 때 -->
        {#if allLoaded && mcpCards.length > 0}
          <div class="flex justify-center items-center py-4 text-gray-500">모든 결과를 불러왔습니다.</div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  /* 페이지 배경색을 탭 색상과 일치시킴 (#ecfdf5) */
  :global(body) {
    background-color: #ecfdf5;
  }

  /* 파란 선 제거 */
  :global(.sticky) {
    border-bottom: none !important;
  }

  /* 스크롤바 스타일 커스터마이징 */
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
