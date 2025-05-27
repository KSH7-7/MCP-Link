<script lang="ts">
  import { onMount, onDestroy } from "svelte"
  import { goto } from "$app/navigation"
  import { listen } from "@tauri-apps/api/event"
  import { invoke } from "@tauri-apps/api/core"
  
  // props from parent component
  export let setSearchTerm: ((keyword: string) => void) | undefined = undefined

  let unlistenNotificationClick: (() => void) | null = null
  let unlistenSearchKeyword: (() => void) | null = null
  let unlistenAppActivated: (() => void) | null = null
  let processingKeyword = false
  let lastProcessedKeyword = ""
  let globalLastProcessTime = 0
  
  let processedKeywords = new Map<string, number>() // keyword last process time
  let sessionStartTime = Date.now() // session start time
  let isInitialLoad = true // initial load

  // keyboard search function
  async function handleKeywordSearch(keyword: string, fromNotification: boolean = false) {
    if (!keyword || !keyword.trim()) {
      return
    }
    
    keyword = keyword.trim()
    const now = Date.now()
    
    // enhanced duplicate processing prevention
    const lastProcessTime = processedKeywords.get(keyword) || 0
    const timeSinceLastProcess = now - lastProcessTime
    const timeSinceSessionStart = now - sessionStartTime
    
    // 1. currently processing
    if (processingKeyword) {
      return
    }
    
    // 2. same keyword processed recently (within 5 minutes)
    if (timeSinceLastProcess < 300000) { // 5 minutes = 300 seconds
      return
    }
    
    // 3. immediately app-activated after initial load (automatic event)
    if (!fromNotification && timeSinceSessionStart < 10000 && isInitialLoad) {
      return
    }

    try {
      processingKeyword = true
      lastProcessedKeyword = keyword
      globalLastProcessTime = now
      processedKeywords.set(keyword, now)
      isInitialLoad = false
      
      // if from notification, try to activate the app
      if (fromNotification) {
        try {
          const result = await invoke("activate_app_window")
        } catch (e) {
        }
      }

      // 1. if setSearchTerm function exists, use it
      if (setSearchTerm) {
        setSearchTerm(keyword)
        
        // after keyword processing, clear the state
        await clearKeywordState()
      } else {
        // 2. if setSearchTerm does not exist, go to the page directly
        await goto(`/MCP-list?keyword=${encodeURIComponent(keyword)}&recommended=true`)
        
        // after keyword processing, clear the state
        await clearKeywordState()
      }

      // 3. show toast message
      const toastEvent = new CustomEvent("show-toast", {
        detail: {
          message: `'${keyword}' searching...`,
          type: "info",
          duration: 2000,
        },
      })
      document.dispatchEvent(toastEvent)

    } catch (error) {
      
      const toastEvent = new CustomEvent("show-toast", {
        detail: {
          message: "An error occurred while processing the keyword",
          type: "error",
          duration: 3000,
        },
      })
      document.dispatchEvent(toastEvent)
      
    } finally {
      setTimeout(() => {
        processingKeyword = false
      }, 1000) // 1 second after processing is complete
    }
  }
  
  // clean up the keyword state
  async function clearKeywordState() {
    try {
      // 1. delete the file
      await deleteKeywordFile()
      
      // 2. clear the Rust state
      await invoke("clear_keyword_state")
    } catch (e) {
    }
  }
  
  // delete the keyword file
  async function deleteKeywordFile() {
    try {
      const path = await import("@tauri-apps/api/path")
      const fs = await import("@tauri-apps/plugin-fs")
      
      const tempDir = await path.tempDir()
      const keywordPath = `${tempDir}mcplink_last_keyword.txt`
      
      await fs.remove(keywordPath)
    } catch (e) {
    }
  }

  onMount(async () => {
    sessionStartTime = Date.now()
    isInitialLoad = true

    // 1. notification-clicked event listener (for notification only)
    unlistenNotificationClick = await listen("notification-clicked", async (event) => {
      const keyword = event.payload as string
      await handleKeywordSearch(keyword, true) // show that it's from notification
    })

    // 2. search-keyword event listener (for backward compatibility)
    unlistenSearchKeyword = await listen("search-keyword", async (event) => {
      const keyword = event.payload as string
      await handleKeywordSearch(keyword, false)
    })

    // 3. app-activated event listener (enhanced version)
    unlistenAppActivated = await listen("app-activated", async (event) => {
      
      // check if it's from notification
      const payload = event.payload as any
      const fromNotification = payload?.fromNotification || false
      
      // automatic event is ignored only during initial load, otherwise processed
      const timeSinceSessionStart = Date.now() - sessionStartTime
      if (!fromNotification && isInitialLoad && timeSinceSessionStart < 5000) {
        isInitialLoad = false // after the first automatic event, release the flag
        return
      }
      
      // check the keyword file (only from notification)
      if (fromNotification) {
        try {
          const fs = await import("@tauri-apps/plugin-fs")
          const path = await import("@tauri-apps/api/path")
          const tempDir = await path.tempDir()
          const keywordPath = `${tempDir}mcplink_last_keyword.txt`
          
          try {
            const keyword = await fs.readTextFile(keywordPath)
            if (keyword && keyword.trim()) {
              await handleKeywordSearch(keyword.trim(), true)
            }
          } catch (e) {
            // if the file does not exist, ignore
          }
        } catch (e) {
        }
      }
    })

    // 4. check the keyword file when the app starts (only once, conditional)
    setTimeout(async () => {
      try {
        const fs = await import("@tauri-apps/plugin-fs")
        const path = await import("@tauri-apps/api/path")
        const tempDir = await path.tempDir()
        const keywordPath = `${tempDir}mcplink_last_keyword.txt`
        
        try {
          const keyword = await fs.readTextFile(keywordPath)
          if (keyword && keyword.trim()) {
            
            // check the file creation time
            try {
              const stat = await fs.stat(keywordPath)
              const mtime = stat.mtime
              const fileTime = mtime instanceof Date ? mtime.getTime() : (typeof mtime === 'number' ? mtime : 0)
              const fileAge = Date.now() - fileTime
              
              // process the file if it's within 30 seconds
              if (fileAge < 30000) {
                await handleKeywordSearch(keyword.trim(), true) // process as from notification
              } else {
                await fs.remove(keywordPath)
              }
            } catch (e) {
              // if stat fails, process it
              await handleKeywordSearch(keyword.trim(), true)
            }
          }
        } catch (e) {
          // if the file does not exist, ignore
        }
      } catch (e) {
      }
    }, 1000) // check after 1 second
  })

  onDestroy(() => {
    if (unlistenNotificationClick) unlistenNotificationClick()
    if (unlistenSearchKeyword) unlistenSearchKeyword()
    if (unlistenAppActivated) unlistenAppActivated()
  })
</script>