// handle notifications on MCP list page
import { showNotification, setupNotificationListeners } from "../../lib/toast-system.js"
import { onMount, onDestroy } from "svelte"
import { goto } from "$app/navigation"
import { page } from "$app/stores"
import { get } from "svelte/store"

// notification handler hook
export function useNotifications() {
  let cleanup = null

  onMount(async () => {
    // setup notification event listeners
    cleanup = await setupNotificationListeners()

    // extract keyword parameter from URL
    const currentPage = get(page)
    if (currentPage.url.searchParams.has("keyword")) {
      const keyword = currentPage.url.searchParams.get("keyword")
      // keyword processing logic should be implemented in the component
    }
  })

  onDestroy(() => {
    // clean up listeners when component unmounts
    if (cleanup) {
      cleanup()
    }
  })

  return {
    showNotification,
  }
}
