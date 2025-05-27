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

  // prevent duplicate calls
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
    } finally {
      loadingMore = false
    }
  }

  // search and display function
  async function searchAndDisplay(term: string, scrollToTop: boolean = false) {
    // remove whitespace
    term = term.trim()

    // prevent consecutive calls with the same search term
    if (isSearching && term === lastSearchTerm) {
      return
    }

    // increase search request ID
    const currentRequestId = ++searchRequestId

    // set search status
    isSearching = true
    lastSearchTerm = term

    // UI initialize
    loading = true
    allLoaded = false
    justLoadedNewData = false
    mcpCards = []
    pageInfo = { has_next_page: false, end_cursor: null, total_items: 0 }

    // scroll position initialize
    if (scrollToTop && mainElement) {
      mainElement.scrollTo(0, 0)
    }

    try {
      // API call
      const response = await fetchMCPCards(term || undefined)

      // check if current request is the latest
      if (currentRequestId !== searchRequestId) {
        return
      }

      // apply results
      mcpCards = response.cards
      pageInfo = response.page_info
      justLoadedNewData = true

      // update total count
      if (!term) {
        updateCount("listCount", pageInfo.total_items)
      }

      // check if page is loaded
      if (!pageInfo.has_next_page || mcpCards.length >= pageInfo.total_items) {
        allLoaded = true
      } else {
        allLoaded = false
      }

      // search completed toast
      if (term && isRecommendedSearch) {
        const toastEvent = new CustomEvent("show-toast", {
          detail: {
            message: `'${term}' search completed: ${mcpCards.length}`,
            type: mcpCards.length > 0 ? "success" : "info",
            duration: 2000,
          },
        })
        document.dispatchEvent(toastEvent)
      }
    } catch (error) {
      mcpCards = []
      pageInfo = { has_next_page: false, end_cursor: null, total_items: 0 }
      allLoaded = true

      // error toast
      const toastEvent = new CustomEvent("show-toast", {
        detail: {
          message: "Error occurred during search",
          type: "error",
          duration: 3000,
        },
      })
      document.dispatchEvent(toastEvent)
    } finally {
      // update status if current request is the latest
      if (currentRequestId === searchRequestId) {
        loading = false
        isSearching = false
      }
    }
  }

  // Search event handler
  const handleSearch = (event: CustomEvent<{ value: string }>) => {
    const term = event.detail.value.trim()

    // cancel debounce timer
    if (searchDebounceTimer) {
      clearTimeout(searchDebounceTimer)
    }

    // update status
    searchTermFromQuery = term
    isRecommendedSearch = false

    // 300ms debounce then search
    searchDebounceTimer = setTimeout(() => {
      searchAndDisplay(term, true)
    }, 300)
  }

  const handleClearSearch = () => {
    // cancel debounce timer
    if (searchDebounceTimer) {
      clearTimeout(searchDebounceTimer)
    }

    searchTermFromQuery = ""
    isRecommendedSearch = false

    searchAndDisplay("", true)
  }

  // NotificationHandler props
  const notificationHandlerProps = {
    setSearchTerm: (keyword: string) => {
      // check if keyword is already processed
      const now = Date.now()
      if (processedKeywords.has(keyword) && now - lastProcessedTime < 5000) {
        return
      }

      // display processed keyword
      processedKeywords.add(keyword)
      lastProcessedTime = now

      // clean old keywords
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
    // find the main element that actually scrolls
    mainElement = document.querySelector("main.flex-1.overflow-y-auto")

    if (mainElement) {
      mainElement.addEventListener("scroll", handleScroll)
    }

    // check keyword from URL parameters
    const urlParams = new URLSearchParams(window.location.search)
    const urlKeyword = urlParams.get("keyword")

    async function deleteKeywordFile() {
      try {
        await invoke("delete_keyword_file")
      } catch (e) {}
    }

    if (urlKeyword) {
      searchTermFromQuery = urlKeyword
      isRecommendedSearch = urlParams.get("recommended") === "true"

      // update search input
      const event = new CustomEvent("set-search-term", { detail: urlKeyword })
      document.dispatchEvent(event)

      // initial load then search without debounce
      searchAndDisplay(urlKeyword, true)
    } else {
      // if no keyword, load all list
      searchAndDisplay("", false)
      deleteKeywordFile()
    }

    // subscribe $page store
    const unsubscribePage = page.subscribe((currentPage) => {
      const urlKeyword = currentPage.url.searchParams.get("keyword")

      // if URL keyword is changed and not searching, process
      if (urlKeyword && urlKeyword !== searchTermFromQuery && !isSearching) {
        searchTermFromQuery = urlKeyword
        isRecommendedSearch = !!currentPage.url.searchParams.get("recommended")

        // update search input
        const searchEvent = new CustomEvent("set-search-term", { detail: urlKeyword })
        document.dispatchEvent(searchEvent)

        // URL change then search immediately
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
          await deleteKeywordFile()
        }
      } catch (e) {}
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
