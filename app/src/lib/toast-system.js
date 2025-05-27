// notification system utility functions
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { goto } from "$app/navigation"
import toastStore from "./stores/toast"
import { handleTauriError } from "./error-handler"

// display notification via Tauri's notification system
export async function showNotification(title, body, keyword) {
  return handleTauriError(
    async () => {
      await invoke("show_notification", { title, body, keyword })
      return true
    },
    false,
    {
      showToast: true,
      position: "top-right",
    }
  )
}

// setup notification event listeners
export async function setupNotificationListeners() {
  // listen for navigation events
  const unlistenNavigate = await listen("navigate-to", (event) => {
    const url = event.payload
    if (url) {
      goto(url)
    }
  })

  // listen for keyword search events in the MCP list page
  const unlistenKeyword = await listen("navigate-to-mcp-list-with-keyword", (event) => {
    const url = event.payload
    if (url && url.includes("keyword=")) {
      goto(url)
    }
  })

  // return cleanup function
  return () => {
    unlistenNavigate()
    unlistenKeyword()
  }
}

// handle URI scheme
export async function handleUriScheme(uri) {
  try {
    // Deep Link plugin automatically handles URLs
    // additional processing may be needed if needed
    return true
  } catch (error) {
    return false
  }
}

// initialize toast system
export function initToastSystem() {
  // add global event listener
  document.addEventListener("show-toast", (event) => {
    const { message, type, duration, position } = event.detail
    showToast(message, { type, duration, position })
  })
}

// display toast message
export function showToast(message, options = {}) {
  const { title = "", type = "info", duration = 3000, position = "bottom-center" } = options

  // update Svelte store
  toastStore.update((state) => ({
    ...state,
    show: true,
    title,
    message,
    type,
    duration,
    position,
  }))

  // automatically hide
  setTimeout(() => {
    toastStore.update((state) => ({
      ...state,
      show: false,
    }))
  }, duration)

  return true
}

// display error toast message (avoid name duplication with showErrorToast in error-handler.ts)
export function showSimpleErrorToast(error, options = {}) {
  const message = error instanceof Error ? error.message : String(error)
  return showToast(message, {
    type: "error",
    duration: 5000,
    position: "bottom-center",
    ...options,
  })
}
