<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte"

  // Create event dispatcher
  const dispatch = createEventDispatcher<{
    search: { value: string }
  }>()

  // START: Add initialValue prop
  export let initialValue: string = ""
  export let placeholder: string = "MCP name or keyword search..."
  export let customClass: string = "input input-bordered rounded-[10px]"
  // END: Add initialValue prop

  // Search value
  let searchValue = ""

  // programmatic change flag
  let isProgrammaticChange = false

  // Debounce timer
  let debounceTimer: ReturnType<typeof setTimeout> | null = null

  // Debounce delay (milliseconds)
  const DEBOUNCE_DELAY = 300

  // START: Set searchValue with initialValue in onMount and listen for set-search-term event
  onMount(() => {
    // set initial value
    if (initialValue) {
      searchValue = initialValue
    }

    // add set-search-term event listener - for keyword setting from notification
    const handleSetSearchTerm = (event: CustomEvent<string>) => {
      if (event?.detail) {
        // set programmatic change flag
        isProgrammaticChange = true
        searchValue = event.detail

        // set programmatic change flag after debounce time
        setTimeout(() => {
          isProgrammaticChange = false
        }, DEBOUNCE_DELAY + 50) // 350ms after debounce time

        // highlight search effect
        setTimeout(() => {
          const inputElement = document.querySelector('input[type="search"]')
          if (inputElement) {
            inputElement.classList.add("highlight-search")
            // set focus
            inputElement.focus()
            // remove highlight effect after 1.5 seconds
            setTimeout(() => {
              inputElement.classList.remove("highlight-search")
            }, 1500)
          }
        }, 10)
      }
    }

    // add event listener
    document.addEventListener("set-search-term", handleSetSearchTerm as EventListener)

    // cleanup function
    return () => {
      document.removeEventListener("set-search-term", handleSetSearchTerm as EventListener)
    }
  })
  // END: Set searchValue with initialValue in onMount

  // Debounced search function
  function debouncedSearch() {
    // if programmatic change, do not trigger search event
    if (isProgrammaticChange) {
      return
    }

    if (debounceTimer) clearTimeout(debounceTimer)
    debounceTimer = setTimeout(() => {
      dispatch("search", { value: searchValue })
    }, DEBOUNCE_DELAY)
  }

  // Search event handler
  function handleSearch() {
    dispatch("search", { value: searchValue })
  }

  // Keyboard event handler (Enter key pressed to search)
  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      if (debounceTimer) clearTimeout(debounceTimer)
      handleSearch()
    }
  }
</script>

<label class="{customClass} flex items-center relative">
  <input type="search" class="w-full" {placeholder} bind:value={searchValue} on:input={debouncedSearch} on:keydown={handleKeyDown} />
  <div class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-5 h-5">
      <circle cx="11" cy="11" r="8"></circle>
      <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
    </svg>
  </div>
</label>

<style>
  label {
    width: 100%; /* Control width from parent element if necessary */
  }

  /* highlight search style */
  :global(.highlight-search) {
    border: 2px solid #ffd700 !important; /* gold border */
    box-shadow: 0 0 10px rgba(255, 215, 0, 0.5) !important; /* gold shadow */
    transition: all 0.3s ease-in-out !important;
    animation: pulse 1.5s ease-out !important;
  }

  /* pulse animation */
  @keyframes pulse {
    0% {
      transform: scale(1);
    }
    25% {
      transform: scale(1.03);
    }
    50% {
      transform: scale(1);
    }
    75% {
      transform: scale(1.02);
    }
    100% {
      transform: scale(1);
    }
  }
</style>
