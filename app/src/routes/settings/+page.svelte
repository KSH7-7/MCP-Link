<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  
  // 설정 값들
  let configPath = '';
  
  // 디렉토리 선택 함수 (실제로는 구현되지 않음)
  function findPath() {
    alert('찾기 기능은 현재 구현되지 않았습니다. 직접 경로를 입력해주세요.');
  }
  
  // Config 경로 저장
  function saveConfigPath() {
    if (!configPath.trim()) {
      alert('경로를 입력해주세요');
      return;
    }
    
    // 여기에 실제 저장 로직 추가
    console.log('Config 경로 저장:', configPath);
    alert('Config 경로가 저장되었습니다');
  }
  
  // MCP 초기화 함수
  async function resetMCPs() {
    if (!confirm('정말로 설치된 모든 MCP를 초기화하시겠습니까? 이 작업은 되돌릴 수 없습니다.')) {
      return;
    }
    
    try {
      // 여기에 실제 초기화 API 호출
      // await invoke('reset_installed_mcps');
      alert('모든 MCP가 성공적으로 초기화되었습니다');
    } catch (error) {
      console.error('MCP 초기화 중 오류 발생:', error);
      alert('MCP 초기화 중 오류가 발생했습니다');
    }
  }
</script>

<div class="p-8 max-w-2xl mx-auto">
  <!-- Config 경로 재설정 섹션 -->
  <div class="bg-white rounded-lg p-6 shadow-sm mb-8 border border-gray-200">
    <h2 class="text-xl font-semibold mb-6">Config 경로 재설정</h2>
    
    <div class="flex items-center">
      <input 
        type="text" 
        class="input input-bordered w-full rounded-md" 
        placeholder="설정 파일 경로를 입력하세요" 
        bind:value={configPath}
      />
      <div class="flex ml-2 space-x-2">
        <button 
          class="btn btn-sm btn-outline border border-gray-300 rounded-md px-4" 
          on:click={findPath}
        >
          찾기
        </button>
        <button 
          class="btn btn-sm btn-primary rounded-md px-4" 
          on:click={saveConfigPath}
        >
          수정
        </button>
      </div>
    </div>
  </div>
  
  <!-- 설치된 MCP 초기화 섹션 -->
  <div class="bg-white rounded-lg p-6 shadow-sm border border-gray-200">
    <h2 class="text-xl font-semibold mb-6">설치된 MCP 초기화</h2>
    
    <div class="mb-8">
      <p class="text-red-600 font-medium text-center mb-4">
        경고! 이 옵션은 (이 GUI를 통해) 설치된 MCP 서버를 전부 삭제합니다. 유의하여주세요
      </p>
    </div>
    
    <div class="flex justify-center">
      <button 
        class="btn btn-error btn-sm rounded-md border border-red-500 px-6" 
        on:click={resetMCPs}
      >
        초기화
      </button>
    </div>
  </div>
</div>