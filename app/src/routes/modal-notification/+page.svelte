<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { invoke } from "@tauri-apps/api/core";
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

  // Get keyword from URL
  let keyword = "";
  let timeLeft = 10; // 10 seconds auto-close countdown
  let intervalId: ReturnType<typeof setInterval>;

  onMount(() => {
    // Get keyword from URL params
    keyword = $page.url.searchParams.get("keyword") || "";

    // Start countdown for auto-close
    intervalId = setInterval(() => {
      timeLeft--;
      if (timeLeft <= 0) {
        clearInterval(intervalId);
        closeModal();
      }
    }, 1000);

    // Make window always on top and visible
    const currentWindow = WebviewWindow.getByLabel("notification-modal");
    if (currentWindow) {
      currentWindow.setAlwaysOnTop(true);
    }

    // Cleanup on unmount
    return () => {
      if (intervalId) clearInterval(intervalId);
    };
  });

  async function navigateToMcpListWithKeyword() {
    clearInterval(intervalId);
    
    // 1. Activate main window and show MCPList with keyword
    try {
      await invoke("activate_main_window_with_keyword", { keyword });
    } catch (error) {
      console.error("Error activating main window:", error);
    }
    
    // 2. Close this modal window
    closeModal();
  }

  function closeModal() {
    // Get the current window and close it
    const currentWindow = WebviewWindow.getByLabel("notification-modal");
    if (currentWindow) {
      currentWindow.close();
    }
  }
</script>

<div class="modal-notification p-6 bg-white rounded-lg shadow-lg border border-primary/20 max-w-md mx-auto">
  <div class="flex justify-between items-start mb-4">
    <h2 class="text-xl font-bold text-gray-800">키워드 추천 확인</h2>
    <button 
      class="text-gray-500 hover:text-gray-700" 
      on:click={closeModal}
      aria-label="Close"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  </div>
  
  <div class="mb-6">
    <p class="text-gray-700">
      추천 키워드: <span class="font-semibold text-primary">{keyword}</span>
    </p>
    <p class="text-gray-600 text-sm mt-2">
      이 키워드로 MCP 서버 목록을 검색하시겠습니까?
    </p>
    <p class="text-gray-500 text-xs mt-1">
      {timeLeft}초 후 자동으로 닫힙니다.
    </p>
  </div>
  
  <div class="flex justify-end space-x-2">
    <button 
      class="px-4 py-2 bg-gray-200 text-gray-800 rounded hover:bg-gray-300"
      on:click={closeModal}
    >
      취소
    </button>
    <button 
      class="px-4 py-2 bg-primary text-primary-content rounded hover:bg-primary-focus"
      on:click={navigateToMcpListWithKeyword}
    >
      검색하기
    </button>
  </div>
</div>

<style>
  /* Ensure modal is centered in window */
  :global(html, body) {
    height: 100%;
    margin: 0;
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: transparent;
  }

  .modal-notification {
    width: 100%;
    max-width: 400px;
    box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
  }
</style>