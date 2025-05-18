<script lang="ts">
  import { onMount } from "svelte"
  import Search from "../../lib/components/search.svelte"
  import MCPCard from "../../lib/components/mcp-card.svelte"
  import { fetchMCPCards } from "../../lib/data/mcp-api"
  import { page } from "$app/stores"
  import { listen } from "@tauri-apps/api/event"
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow"
  import NotificationHandler from "./NotificationHandler.svelte"

  // define data type to receive from backend
  import type { MCPCard as MCPCardType, PageInfo } from "../../lib/data/mcp-api"

  // MCP card data
  let mcpCards: MCPCardType[] = []
  let pageInfo: PageInfo = { has_next_page: false, end_cursor: null, total_items: 0 }

  // data loading state
  let loading = true
  let loadingMore = false
  let allLoaded = false // Flag to check if all data has been loaded

  // START: Added variables
  let searchTermFromQuery = "" // Variable to store the search term from the URL
  let isRecommendedSearch = false // State variable for recommended search
  let debounceTimer: ReturnType<typeof setTimeout> | null = null; // Debounce timer
  // END: Added variables

  // Scroll event handler
  function handleScroll() {
    if (loadingMore || !pageInfo.has_next_page || allLoaded) return

    // Check scroll position of the entire document
    const scrollPosition = window.innerHeight + window.scrollY
    const scrollHeight = document.body.offsetHeight

    // Load next page if scroll is 300px from the bottom (detect faster)
    if (scrollHeight - scrollPosition < 300) {
      loadNextPage()
    }
  }

  // Function to load the next page
  async function loadNextPage() {
    if (!pageInfo.has_next_page || !pageInfo.end_cursor || loadingMore || allLoaded) return

    loadingMore = true
    try {
      const response = await fetchMCPCards(searchTermFromQuery || undefined, pageInfo.end_cursor)

      // Add new cards to existing cards
      mcpCards = [...mcpCards, ...response.cards]
      pageInfo = response.page_info

      // Check if all data has been loaded
      if (!pageInfo.has_next_page || mcpCards.length >= pageInfo.total_items) {
        allLoaded = true
      }
    } catch (error) {
      console.error("Error loading next page:", error)
    } finally {
      loadingMore = false
    }
  }

  // Page initialization and data loading function
  function initPage() {
    // Add global scroll event listener (with debounce)
    let scrollTimer: ReturnType<typeof setTimeout> | null = null
    const scrollHandler = () => {
      if (scrollTimer) clearTimeout(scrollTimer)
      scrollTimer = setTimeout(handleScroll, 100)
    }

    window.addEventListener("scroll", scrollHandler)

    // Check page bottom after initial load
    setTimeout(() => {
      handleScroll()
    }, 500)

    return scrollHandler
  }

  // get data when component is mounted
  onMount(() => {
    // 세션 스토리지에서 알림 키워드 확인 (일회성 처리)
    const pendingKeyword = sessionStorage.getItem('pendingSearchKeyword')
    if (pendingKeyword) {
      console.log('Found pending keyword in session storage:', pendingKeyword)
      searchTermFromQuery = pendingKeyword
      isRecommendedSearch = true
      
      // 페이지 로드 시 자동 검색 실행
      setTimeout(() => {
        searchAndDisplay(pendingKeyword)
      }, 300)
      
      // 사용한 키워드는 세션 스토리지에서 제거
      sessionStorage.removeItem('pendingSearchKeyword')
      
      // 마지막 알림 키워드도 제거 (페이지 새로고침에서 재사용 방지)
      localStorage.removeItem('lastNotificationKeyword')
    } else {
      // URL 파라미터에서 키워드를 확인
      const urlParams = new URLSearchParams(window.location.search);
      const urlKeyword = urlParams.get('keyword');
      
      // URL에 키워드가 있으면 마지막 키워드 검사는 건너뜀
      if (urlKeyword) {
        console.log('Found keyword in URL parameters:', urlKeyword);
        // 마지막 알림 키워드가 URL에 있는 키워드와 같으면 제거
        if (localStorage.getItem('lastNotificationKeyword') === urlKeyword) {
          localStorage.removeItem('lastNotificationKeyword');
        }
        return;
      }
      
      // 로컬 스토리지에서 마지막 알림 키워드 확인 (백업 처리)
      const lastKeyword = localStorage.getItem('lastNotificationKeyword')
      if (lastKeyword && !pendingKeyword && !searchTermFromQuery) {
        console.log('Using last notification keyword as fallback:', lastKeyword)
        searchTermFromQuery = lastKeyword
        isRecommendedSearch = true
        
        // 페이지 로드 시 자동 검색 실행
        setTimeout(() => {
          searchAndDisplay(lastKeyword)
          // 사용 후 키워드 제거 (재사용 방지)
          localStorage.removeItem('lastNotificationKeyword')
        }, 300)
      }
    }
    
    // 1. 추가: search-keyword 이벤트 리스닝
    try {
      // @ts-ignore
      window.__TAURI__.event.listen('search-keyword', (event) => {
        try {
          // 키워드 추출 (문자열이나 객체 형태에 대응)
          const keyword = typeof event.payload === 'string' 
            ? event.payload  // 이미 문자열이면 그대로 사용
            : (typeof event.payload === 'object' && event.payload && event.payload.keyword 
              ? event.payload.keyword   // 객체에서 keyword 속성 추출
              : null)
          
          if (keyword) {
            console.log('수신된 검색 키워드:', keyword)
            
            // 1. 검색창에 키워드 설정 (set-search-term 이벤트 발생)
            try {
              // 검색창에 키워드 설정
              const searchEvent = new CustomEvent('set-search-term', { detail: keyword })
              document.dispatchEvent(searchEvent)
              console.log('검색창에 키워드 설정됨:', keyword)
              
              // 2. 검색 상태 업데이트 (URL은 업데이트하지 않고 시각적으로만 검색 상태 표시)
              searchTermFromQuery = keyword
              isRecommendedSearch = true
              
              // 3. 검색 실행 - 검색창에 키워드가 표시된 후 약간 지연시켜 검색 실행
              setTimeout(() => {
                searchAndDisplay(keyword)
              }, 200)
            } catch (e) {
              console.error('검색창 키워드 설정 오류:', e)
            }
          } else {
            console.warn('search-keyword 이벤트에서 키워드를 찾을 수 없음')
          }
        } catch (e) {
          console.error('search-keyword 이벤트 처리 오류:', e)
        }
      })
    } catch (e) {
      console.error('Error setting up search-keyword listener:', e)
    }
    
    // 2. 추가: activation-complete 이벤트 리스닝
    try {
      // @ts-ignore
      window.__TAURI__.event.listen('activation-complete', () => {
        // 앱이 활성화되었을 때 추가 조치
        // URL 파라미터로 키워드가 있는지 확인
        const urlParams = new URLSearchParams(window.location.search);
        const urlKeyword = urlParams.get('keyword');
        
        // 이미 URL에 키워드가 있다면, 기존 키워드 처리는 무시
        if (urlKeyword) {
          console.log('이미 URL에 키워드가 있어 알림 키워드 처리 무시:', urlKeyword);
          
          // 마지막 알림 키워드가 현재 URL과 같으면 제거
          const lastKeyword = localStorage.getItem('lastNotificationKeyword');
          if (lastKeyword === urlKeyword) {
            localStorage.removeItem('lastNotificationKeyword');
          }
          return;
        }
        
        // 세션 스토리지에서 먼저 확인
        const pendingKeyword = sessionStorage.getItem('pendingSearchKeyword');
        if (pendingKeyword) {
          console.log('앱 활성화: 세션 스토리지에서 키워드 발견:', pendingKeyword);
          searchTermFromQuery = pendingKeyword;
          isRecommendedSearch = true;
          searchAndDisplay(pendingKeyword);
          
          // 사용 후 제거
          sessionStorage.removeItem('pendingSearchKeyword');
          return;
        }
        
        // 로컬 스토리지에서 확인
        const lastKeyword = localStorage.getItem('lastNotificationKeyword');
        if (lastKeyword && (!searchTermFromQuery || searchTermFromQuery !== lastKeyword)) {
          console.log('앱 활성화: 로컬 스토리지에서 키워드 발견:', lastKeyword);
          searchTermFromQuery = lastKeyword;
          isRecommendedSearch = true;
          searchAndDisplay(lastKeyword);
          
          // 사용 후 제거
          localStorage.removeItem('lastNotificationKeyword');
        }
      })
    } catch (e) {
      console.error('Error setting up activation-complete listener:', e)
    }
    // START: Add main window event listener (navigation and centering)
    let unlistenNavigate: (() => void) | undefined
    listen("navigate-to-mcp-list-with-keyword", async (event) => {
      const newUrl = event.payload as string
      if (newUrl && typeof newUrl === "string") {
        const url = new URL(newUrl, window.location.origin) // Create full URL
        const keyword = url.searchParams.get("keyword")
        if (keyword) {
          searchTermFromQuery = keyword
          isRecommendedSearch = true
          await searchAndDisplay(keyword)
        } else {
          searchTermFromQuery = "" // Initialize if keyword is missing
          isRecommendedSearch = false
        }
      }
    }).then((fn) => (unlistenNavigate = fn)) // Save unlisten function

    // START: Read keyword from URL query parameter
    const unsubscribePage = page.subscribe((p) => {
      const keyword = p.url.searchParams.get("keyword")
      if (keyword) {
        if (searchTermFromQuery !== keyword) {
          // Execute later than event or for direct URL access
          searchTermFromQuery = keyword
          isRecommendedSearch = true
          searchAndDisplay(searchTermFromQuery)
        }
      } else {
        if (searchTermFromQuery !== "") {
          // Initialize if there was a previous search term
          searchTermFromQuery = ""
        }
        isRecommendedSearch = false
        fetchAllMCPs()
      }
    })
    // END: Read keyword from URL query parameter

    // Initialize scroll event
    const scrollHandler = initPage()

    // Unregister listener and unsubscribe on component destroy
    return () => {
      if (unlistenNavigate) unlistenNavigate() // Call saved unlisten function
      unsubscribePage()
      window.removeEventListener("scroll", scrollHandler)
    }
  })

  // Detect page change and reinitialize
  $: {
    if ($page) {
      if ($page.url.pathname === "/MCP-list") {
        setTimeout(() => {
          handleScroll()
        }, 500)
      }
    }
  }

  // START: Search execution and result display function
  async function searchAndDisplay(term: string) {
    if (!term) {
      isRecommendedSearch = false // Clear recommendation state if search term is empty
      return fetchAllMCPs() // Load all if term is empty
    }
    
    // 키워드를 검색창에도 설정 (사용자 피드백용)
    try {
      // 이벤트 발생
      const searchEvent = new CustomEvent('set-search-term', { detail: term })
      document.dispatchEvent(searchEvent)
    } catch (e) {
      console.error('Failed to dispatch search event:', e)
    }
    
    // isRecommendedSearch can only be true if term exists, so don't change it here
    loading = true
    allLoaded = false // Reset allLoaded when loading new data
    try {
      // 알림이 처리되었음을 사용자에게 표시 (타이밍 잠깐 지연)
      setTimeout(() => {
        if (isRecommendedSearch) {
          try {
            // 토스트 메시지 표시
            const toastEvent = new CustomEvent('show-toast', {
              detail: {
                message: `'${term}' 키워드로 검색중..`,
                type: 'info',
                duration: 3000
              }
            })
            document.dispatchEvent(toastEvent)
          } catch (e) {
            console.error('Toast event error:', e)
          }
        }
      }, 100)
      
      const response = await fetchMCPCards(term)
      mcpCards = response.cards
      pageInfo = response.page_info

      // Check if all data has been loaded
      if (!pageInfo.has_next_page || mcpCards.length >= pageInfo.total_items) {
        allLoaded = true
      } else {
        // Check scroll after initial load
        setTimeout(handleScroll, 500)
      }
      
      // 검색 결과 표시
      if (isRecommendedSearch) {
        try {
          // 결과 토스트 메시지
          const resultToast = new CustomEvent('show-toast', {
            detail: {
              message: `'${term}' 키워드 검색 결과: ${mcpCards.length}개 발견`,
              type: mcpCards.length > 0 ? 'success' : 'warning',
              duration: 3000
            }
          })
          setTimeout(() => document.dispatchEvent(resultToast), 1000)
        } catch (e) {
          console.error('Result toast error:', e)
        }
      }
    } catch (error) {
      console.error("Error during search:", error)
      mcpCards = []
      pageInfo = { has_next_page: false, end_cursor: null, total_items: 0 }
      allLoaded = true
      
      // 검색 오류 표시
      if (isRecommendedSearch) {
        try {
          const errorToast = new CustomEvent('show-toast', {
            detail: {
              message: `'${term}' 키워드 검색 오류 발생`,
              type: 'error',
              duration: 3000
            }
          })
          document.dispatchEvent(errorToast)
        } catch (e) {
          console.error('Error toast error:', e)
        }
      }
    } finally {
      loading = false
      // Scroll to the top of the page
      window.scrollTo(0, 0)
    }
  }
  // END: Search execution and result display function

  // START: Function to fetch all MCP cards
  async function fetchAllMCPs() {
    isRecommendedSearch = false // Clear recommendation state when fetching all list
    allLoaded = false // Reset allLoaded when loading new data
    try {
      const response = await fetchMCPCards()
      mcpCards = response.cards
      pageInfo = response.page_info

      // Check if all data has been loaded
      if (!pageInfo.has_next_page || mcpCards.length >= pageInfo.total_items) {
        allLoaded = true
      } else {
        // Check scroll after initial load
        setTimeout(handleScroll, 500)
      }
    } catch (error) {
      console.error("Error fetching MCP data:", error)
      mcpCards = []
      pageInfo = { has_next_page: false, end_cursor: null, total_items: 0 }
      allLoaded = true
    } finally {
      loading = false
      // Scroll to the top of the page
      window.scrollTo(0, 0)
    }
  }
  // END: Function to fetch all MCP cards

  // Search event handler
  async function handleSearchEvent(event: CustomEvent<{ value: string }>) {
    const searchTerm = event.detail.value
    // User directly entered/changed search term, so clear recommendation state
    if (isRecommendedSearch && searchTerm !== searchTermFromQuery) {
      isRecommendedSearch = false
    }
    isRecommendedSearch = false

    // Do not update URL, so searchTermFromQuery remains unchanged.
    // searchAndDisplay searches with the current value in the search bar (searchTerm).
    await searchAndDisplay(searchTerm)
  }
</script>

<NotificationHandler />

<div class="container mx-auto pb-8">
  <!-- Top header area (not fixed) - background color same as page background -->
  <div class="pt-1 pb-2 border-b border-primary-content/10">
    <div class="flex flex-col sm:flex-row justify-between items-center w-full px-4">
      <h1 class="text-2xl font-bold text-center sm:text-left sm:mr-auto">MCP List ({pageInfo.total_items})</h1>
      
      <!-- Search UI -->
      <div class="relative w-full max-w-xs mx-auto sm:mx-0 sm:w-64 mt-2 sm:mt-0 sm:ml-auto">
        {#if isRecommendedSearch}
          <span class="absolute left-[-20px] top-3 text-yellow-500" title="Recommended Search">✨</span>
        {/if}
        
        <Search 
          initialValue={searchTermFromQuery}
          placeholder="Search MCPs..."
          customClass="input input-bordered w-full pr-10"
          on:search={(event) => handleSearchEvent(event)} 
        />
        
        {#if loading && !mcpCards.length}
          <span class="loading loading-spinner loading-xs absolute right-3 top-3"></span>
        {:else}
          <svg 
            xmlns="http://www.w3.org/2000/svg" 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round" 
            class="w-5 h-5 absolute right-3 top-3"
          >
            <circle cx="11" cy="11" r="8"></circle>
            <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
          </svg>
        {/if}
      </div>
    </div>
  </div>

  <!-- Content area -->
  <div class="mt-3 px-4">
    {#if loading}
      <div class="flex justify-center items-center h-64">
        <span class="loading loading-spinner loading-lg text-primary"></span>
      </div>
    {:else if mcpCards.length === 0}
      <div class="flex justify-center items-center h-64">
        <p class="text-gray-500">No MCPs found.</p>
      </div>
    {:else}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {#each mcpCards as card (card.id)}
          <MCPCard {...card} />
        {/each}
      </div>

      <!-- Loading spinner (loading more data) -->
      {#if loadingMore}
        <div class="flex justify-center items-center py-4">
          <span class="loading loading-spinner loading-md text-primary"></span>
        </div>
      {/if}

      <!-- When no more data -->
      {#if allLoaded && mcpCards.length > 0}
        <div class="flex justify-center items-center py-4 text-gray-500">All results loaded.</div>
      {/if}
    {/if}
  </div>
</div>

<style>
  /* Customize scrollbar style */
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
  
  /* Container style */
  .container {
    max-width: 1200px; /* Match with Installed-MCP page */
  }
</style>
