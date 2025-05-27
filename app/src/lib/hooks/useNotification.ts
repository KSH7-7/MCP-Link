import { onMount, onDestroy } from "svelte"
import { listen } from "@tauri-apps/api/event"
import { appWindow } from "@tauri-apps/api/window"
import type { UnlistenFn } from "@tauri-apps/api/event"

// handle notification related events
export function useNotification(onNavigateToMcpList: (keyword: string) => void) {
  let unlistenFns: UnlistenFn[] = []

  onMount(async () => {
    // handle general navigation event
    const unlisten1 = await listen("navigate-to", (event) => {
      // implementation needed
    })

    // handle MCP list page navigation event
    const unlisten2 = await listen("navigate-to-mcp-list-with-keyword", (event) => {
      const url = event.payload as string
      if (url && url.includes("keyword=")) {
        const keyword = url.split("keyword=")[1]
        if (keyword) {
          onNavigateToMcpList(keyword)
        }
      }
    })

    // handle new keyword event
    const unlisten3 = await listen("new-keywords", (event) => {
      // implementation needed
    })

    unlistenFns = [unlisten1, unlisten2, unlisten3]
  })

  onDestroy(() => {
    // clean up all event listeners
    unlistenFns.forEach((fn) => fn())
  })

  return {
    // utility functions will be added here as needed
  }
}
