<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  
  let testKeyword = "test-keyword";
  let notificationTitle = "테스트 알림";
  let notificationBody = "이것은 테스트 알림입니다.";
  let testStatus = "";
  let forcedActivateStatus = "";
  
  async function showTestNotification() {
    testStatus = "알림 표시 중...";
    try {
      await invoke("show_notification", {
        title: notificationTitle,
        body: notificationBody,
        keyword: testKeyword
      });
      testStatus = "알림 표시 완료. 알림을 클릭해보세요.";
    } catch (error) {
      testStatus = `오류 발생: ${error}`;
      console.error("알림 표시 오류:", error);
    }
  }
  
  async function testForceActivate() {
    forcedActivateStatus = "앱 활성화 테스트 중...";
    try {
      await invoke("test_force_activate");
      forcedActivateStatus = "앱 활성화 명령 실행 완료";
    } catch (error) {
      forcedActivateStatus = `오류 발생: ${error}`;
      console.error("앱 활성화 오류:", error);
    }
  }
  
  async function testSearchKeyword() {
    forcedActivateStatus = "키워드 검색 테스트 중...";
    try {
      await invoke("test_search_keyword", {
        keyword: testKeyword
      });
      forcedActivateStatus = "키워드 검색 명령 실행 완료";
    } catch (error) {
      forcedActivateStatus = `오류 발생: ${error}`;
      console.error("키워드 검색 오류:", error);
    }
  }
  
  onMount(() => {
    // 이벤트 리스너 설정
    document.addEventListener('storage-updated', () => {
      console.log('스토리지 업데이트 이벤트 수신');
    });
  });
</script>

<div class="p-4">
  <h2 class="text-xl font-bold mb-4">알림 테스트</h2>
  
  <div class="mb-6 bg-base-200 p-4 rounded-lg">
    <p class="text-sm text-base-content/80 mb-2">
      알림을 테스트하려면 아래 필드를 작성하고 '알림 표시' 버튼을 클릭하세요.
      알림이 표시되면 알림을 클릭하여 앱이 활성화되고 키워드 검색이 실행되는지 확인하세요.
    </p>
    
    <div class="flex flex-col gap-2">
      <div class="form-control">
        <label class="label">
          <span class="label-text">알림 제목</span>
        </label>
        <input type="text" class="input input-bordered" bind:value={notificationTitle} placeholder="알림 제목" />
      </div>
      
      <div class="form-control">
        <label class="label">
          <span class="label-text">알림 내용</span>
        </label>
        <input type="text" class="input input-bordered" bind:value={notificationBody} placeholder="알림 내용" />
      </div>
      
      <div class="form-control">
        <label class="label">
          <span class="label-text">테스트 키워드</span>
        </label>
        <input type="text" class="input input-bordered" bind:value={testKeyword} placeholder="검색할 키워드" />
      </div>
      
      <button class="btn btn-primary mt-2" on:click={showTestNotification}>
        알림 표시
      </button>
      
      {#if testStatus}
        <div class="mt-2 p-2 text-sm bg-info text-info-content rounded">
          {testStatus}
        </div>
      {/if}
    </div>
  </div>
  
  <div class="mb-6 bg-base-200 p-4 rounded-lg">
    <h3 class="text-lg font-semibold mb-2">앱 활성화 테스트</h3>
    <p class="text-sm text-base-content/80 mb-2">
      앱을 최소화한 후 이 버튼을 다른 응용 프로그램에서 클릭하면 앱이 활성화되는지 테스트할 수 있습니다.
    </p>
    
    <div class="flex gap-2">
      <button class="btn btn-secondary" on:click={testForceActivate}>
        앱 강제 활성화
      </button>
      
      <button class="btn btn-accent" on:click={testSearchKeyword}>
        키워드 검색 테스트
      </button>
    </div>
    
    {#if forcedActivateStatus}
      <div class="mt-2 p-2 text-sm bg-info text-info-content rounded">
        {forcedActivateStatus}
      </div>
    {/if}
  </div>
  
  <div class="bg-base-200 p-4 rounded-lg">
    <h3 class="text-lg font-semibold mb-2">디버그 로그 위치</h3>
    <ul class="list-disc ml-5 text-sm">
      <li><code>%TEMP%\mcplink_debug.log</code>: 앱 디버그 로그</li>
      <li><code>%TEMP%\mcplink_notification.log</code>: 알림 시스템 로그</li>
      <li><code>%TEMP%\mcplink_activation.log</code>: 앱 활성화 로그</li>
      <li><code>%TEMP%\mcplink_notification_click.log</code>: 알림 클릭 로그</li>
      <li><code>%TEMP%\mcplink_test.log</code>: 테스트 명령 로그</li>
    </ul>
  </div>
</div>