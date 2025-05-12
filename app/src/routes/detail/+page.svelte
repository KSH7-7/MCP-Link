<script lang="ts">
  import { onMount } from 'svelte';
  import { Star, Github, ArrowLeft } from 'lucide-svelte';
  import { fetchMCPCardDetail } from '$lib/data/mcp-api';
  import { invoke } from '@tauri-apps/api/core';
  
  // URL에서 MCP 정보 가져오기
  let id: number = 0;
  let title: string = '';
  let description: string = '';
  let url: string = '';
  let stars: number = 0;
  // 상세 설정 바인딩
  let args: string[] = [];
  let env: Record<string, any> = {};
  let command: string = '';
  let loading = true; // 초기 로딩 상태 true
  let error = '';
  let mode: 'install' | 'edit' = 'install'; // mode 변수 추가 및 기본값 설정
  
  // 스타 수 포맷팅 (1000 이상이면 K로 표시)
  function formatStars(count: number): string {
    if (count >= 1000) {
      return `${Math.round(count / 100) / 10}k`;
    }
    return count.toString();
  }
  
  // GitHub 링크 열기 함수
  async function openGitHub(urlToOpen: string) {
    if (!urlToOpen) return;
    
    // 프로토콜이 없는 경우 https:// 추가
    const finalUrl = urlToOpen.startsWith('http') ? urlToOpen : `https://${urlToOpen}`;
    
    try {
      // 브라우저의 기본 window.open 메서드 사용
      window.open(finalUrl, '_blank');
    } catch (error) {
      console.error('링크 열기 실패:', error);
      // 실패 시 사용자에게 알림
      alert(`링크를 열 수 없습니다. 수동으로 접속해 주세요: ${finalUrl}`);
    }
  }
  
  // 뒤로가기 함수
  function goBack() {
    window.history.back();
  }
  
  // 별점 배열 생성
  let starsArray: number[] = [];
  
  // 배열이나 객체를 문자열로 변환 (placeholder용)
  function formatValueForPlaceholder(value: any): string {
    if (Array.isArray(value)) {
      return value.join(', ');
    }
    if (typeof value === 'object' && value !== null) {
      return Object.entries(value)
        .map(([k, v]) => `${k}: ${v}`)
        .join(', ');
    }
    return String(value || '');
  }
  
  // MCP 상세 데이터 로드
  async function loadMCPDetail(mcpId: number) {
    if (!mcpId) {
      error = 'ID가 없습니다.';
      loading = false;
      return;
    }
    
    loading = true;
    error = '';
    
    try {
      const detail = await fetchMCPCardDetail(mcpId);
      console.log('상세 데이터 로드 성공:', detail);
      
      // 기본 정보 업데이트 (이미 URL에서 가져왔으므로 선택적)
      title = detail.title || title;
      description = detail.description || description;
      url = detail.url || url;
      stars = detail.stars || stars;
      
      // 상세 설정 업데이트
      args = detail.args || [];
      env = detail.env || {};
      command = detail.command || '';
      
      // 별점 배열 업데이트
      const starCount = Math.min(Math.round(stars / 1000), 5);
      starsArray = Array(5).fill(0).map((_, i) => i < starCount ? 1 : 0);
    } catch (err: any) {
      console.error('상세 데이터 로드 실패', err);
      error = `상세 데이터 로드 실패: ${err.message || err.toString()}`;
       // 예제 데이터 사용 (선택적 - 오류 발생 시 대체 UI 제공 목적)
      if (mcpId === 16) { // sentry 서버 예시
        args = ['mcp-server-sentry', '--auth-token', 'YOUR_SENTRY_TOKEN'];
        env = { SENTRY_DSN: 'https://example.sentry.io/123456' };
        command = 'uvx';
      } 
    } finally {
      loading = false;
    }
  }
  
  onMount(() => {
    // URL에서 파라미터 가져오기
    const params = new URLSearchParams(window.location.search);
    id = parseInt(params.get('id') || '0');
    
    // mode 파라미터 읽기
    const modeParam = params.get('mode');
    if (modeParam === 'edit' || modeParam === 'install') {
      mode = modeParam;
    }
    
    // 기본 정보는 URL 파라미터에서 가져오기
    title = params.get('title') || '제목 없음';
    description = params.get('description') || '설명 없음';
    url = params.get('url') || '';
    stars = parseInt(params.get('stars') || '0');
    
    // 별점 배열 생성 (최대 5개)
    const starCount = Math.min(Math.round(stars / 1000), 5);
    starsArray = Array(5).fill(0).map((_, i) => i < starCount ? 1 : 0);
    
    // ID가 있으면 상세 데이터 로드
    if (id) {
      loadMCPDetail(id);
    } else {
      error = '잘못된 접근입니다: ID가 없습니다.';
      loading = false;
    }
  });
</script>

<div class="p-8 max-w-5xl mx-auto">
  <div class="bg-white rounded-lg shadow-sm p-6 relative">
    <!-- 뒤로가기 버튼 -->
    <button 
      class="absolute top-4 right-4 btn btn-sm btn-ghost gap-1" 
      on:click={goBack}
    >
      <ArrowLeft size={18} />
      <span>뒤로</span>
    </button>
    
    <!-- 제목과 버전 -->
    <div class="mb-8 border-b pb-4">
      <div class="flex items-center gap-3">
        <h1 class="text-2xl font-bold">{title}</h1>
        <div class="flex items-center gap-1">
          <Star class="text-yellow-400 fill-yellow-400" size={22} />
          <span class="text-gray-600 font-medium">{stars > 0 ? formatStars(stars) : '0'}</span>
        </div>
        {#if url && url.includes('github.com')}
          <button
            on:click={() => openGitHub(url)}
            class="text-gray-500 hover:text-gray-700 transition-colors"
            title="GitHub 리포지토리 열기"
          >
            <Github size={20} />
          </button>
        {/if}
      </div>
    </div>
    
    {#if loading}
      <div class="flex justify-center p-8">
        <div class="loading loading-spinner loading-lg"></div>
      </div>
    {:else if error}
      <div class="alert alert-error">
        <span>{error}</span>
        {#if id}
        <div class="mt-2">
          <button class="btn btn-sm btn-outline" on:click={() => loadMCPDetail(id)}>다시 시도</button>
        </div>
        {/if}
      </div>
    {:else}
      <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
        <!-- 왼쪽 컬럼: MCP 기본 정보 -->
        <div>
          <div class="mb-4">
          </div>
          
          <div>
            <h2 class="text-xl font-semibold mb-4">설명</h2>
            <div class="prose max-w-none">
              <p>{description}</p>
            </div>
          </div>
        </div>
        
        <!-- 오른쪽 컬럼: 필수 설정 -->
        <div>
          <h2 class="text-xl font-semibold mb-4">필수 설정</h2>
          <div class="mb-6">
            {#if command}
              <div class="form-control mb-4">
                <div class="form-control">
                  <label class="label">
                    <span class="label-text">Command</span>
                  </label>
                  <input type="text" class="input input-bordered" value={command} />
                </div>
              </div>
            {/if}
            
            {#if args && args.length > 0}
              <div class="form-control mb-4">
                <div class="form-control">
                  <label class="label">
                    <span class="label-text">Arguments</span>
                  </label>
                  <!-- 비밀번호 타입 대신 readonly textarea 사용 -->
                  <textarea class="textarea textarea-bordered" rows="3">{formatValueForPlaceholder(args)}</textarea>
                </div>
              </div>
            {/if}
            
            {#if env && Object.keys(env).length > 0}
              <div class="form-control mb-4">
                <div class="form-control">
                  <label class="label">
                    <span class="label-text">Environment Variables</span>
                  </label>
                  <!-- 비밀번호 타입 대신 readonly textarea 사용 -->
                  <textarea class="textarea textarea-bordered" rows="3">{formatValueForPlaceholder(env)}</textarea>
                </div>
              </div>
            {/if}
            
            {#if (!command && (!args || args.length === 0) && (!env || Object.keys(env).length === 0))}
              <p class="text-gray-500">필수 설정 항목이 없습니다.</p>
            {/if}
          </div>
        </div>
      </div>
    {/if}
    
    <!-- 하단 버튼 영역 -->
    <div class="mt-8 flex justify-end gap-2 border-t pt-4">
      <button 
        class="btn btn-sm btn-outline" 
        on:click={goBack}
      >
        취소
      </button>
      <button 
        class="btn btn-sm btn-primary"
        on:click={() => {
          if (mode === 'edit') {
            alert('MCP가 수정되었습니다!');
            // 추후 실제 수정 로직 추가
          } else {
            alert('MCP가 설치되었습니다!');
          }
        }}
        disabled={loading || !!error}
      >
        {mode === 'edit' ? '수정하기' : '설치하기'}
      </button>
    </div>
  </div>
</div>