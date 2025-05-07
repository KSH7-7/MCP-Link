<script lang="ts">
  import { onMount } from 'svelte';
  import { Star, Github, ArrowLeft } from 'lucide-svelte';
  
  // URL에서 MCP 정보 가져오기
  let id: number = 0;
  let title: string = '';
  let description: string = '';
  let url: string = '';
  let stars: number = 0;
  
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
  
  onMount(() => {
    // URL에서 파라미터 가져오기
    const params = new URLSearchParams(window.location.search);
    id = parseInt(params.get('id') || '0');
    title = params.get('title') || '';
    description = params.get('description') || '';
    url = params.get('url') || '';
    stars = parseInt(params.get('stars') || '0');
    
    // 별점 배열 생성 (최대 5개)
    const starCount = Math.min(Math.round(stars / 1000), 5);
    starsArray = Array(5).fill(0).map((_, i) => i < starCount ? 1 : 0);
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
      </div>
    </div>
    
    <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
      <!-- 왼쪽 컬럼: MCP 기본 정보 -->
      <div>
        <div class="mb-4">
          <p class="text-gray-600 mb-1">이름: {title}</p>
          <p class="text-gray-600 mb-3">제공자: {url ? (url.includes('github.com') ? 'GitHub' : url.split('/')[2] || 'Unknown') : 'Unknown'}</p>
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
          <div class="form-control mb-4">
            <div class="form-control">
              <label class="label">
                <span class="label-text">API 키</span>
              </label>
              <input type="password" class="input input-bordered" value="" placeholder="GitHub에서 발급받은 API 키를 입력하세요" />
            </div>
          </div>
          
          <div class="form-control">
            <label class="label">
              <span class="label-text">저장소 URL</span>
            </label>
            <div class="flex">
              <input 
                type="text" 
                class="input input-bordered flex-1" 
                value={url} 
                readonly
              />
            </div>
          </div>
        </div>
      </div>
    </div>
    
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
        on:click={() => alert('MCP가 설치되었습니다!')}
      >
        설치하기
      </button>
    </div>
  </div>
</div>