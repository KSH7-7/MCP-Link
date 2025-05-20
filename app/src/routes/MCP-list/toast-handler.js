// MCP 목록 페이지에서 알림 처리
import { showNotification, setupNotificationListeners } from "../../lib/toast-system.js"
import { onMount, onDestroy } from "svelte"
import { goto } from "$app/navigation"
import { page } from "$app/stores"
import { get } from "svelte/store"

// Notification handler hook
export function useNotifications() {
  let cleanup = null

  onMount(async () => {
    // Set up notification event listeners
    cleanup = await setupNotificationListeners()

    // Extract keyword parameter from URL
    const currentPage = get(page)
    if (currentPage.url.searchParams.has("keyword")) {
      const keyword = currentPage.url.searchParams.get("keyword")
      // Trigger search functionality with keyword
      triggerSearch(keyword)
    }
  })

  onDestroy(() => {
    // Clean up listeners when component unmounts
    if (cleanup) {
      cleanup()
    }
  })

  // Search trigger function (must be implemented in the MCP-list page)
  function triggerSearch(keyword) {
    // This function must be overridden in the MCP-list page
    // The default implementation is an empty function
  }

  // Expose notification display function
  return {
    showNotification,
    triggerSearch,
  }
}
