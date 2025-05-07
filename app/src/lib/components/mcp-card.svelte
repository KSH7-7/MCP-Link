<script lang="ts">
  import { Star, Github, Info } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { goto } from '$app/navigation';
  
  // 카드 컴포넌트에 필요한 props 정의
  export let id: number;
  export let title: string;
  export let description: string;
  export let url: string = ""; // GitHub URL
  export let stars: number = 0; // GitHub Stars 수
  export let onClick: (() => void) | undefined = undefined; // 클릭 핸들러(선택적)
  export let className: string = ""; // 추가 CSS 클래스(선택적)
  export let variant: "default" | "primary" | "accent" = "default"; // 카드 스타일 변형
  export let maxDescLength: number = 150; // 설명 최대 길이
  
  // 스타 수 포맷팅 (1000 이상이면 K로 표시)
  function formatStars(count: number): string {
    if (count >= 1000) {
      return `${Math.round(count / 100) / 10}k`;
    }
    return count.toString();
  }
  
  // 설명 텍스트 줄이기
  function truncateDescription(text: string, maxLength: number): string {
    if (text.length <= maxLength) return text;
    return text.substring(0, maxLength) + "...";
  }
  
  // 최종 표시할 설명
  const displayDescription = truncateDescription(description, maxDescLength);
  
  // GitHub 링크 열기 함수
  async function openGitHub(urlToOpen: string) {
    console.log('GitHub 링크 열기 시도:', urlToOpen);
    if (!urlToOpen) return;
    
    // 프로토콜이 없는 경우 https:// 추가
    const finalUrl = urlToOpen.startsWith('http') ? urlToOpen : `https://${urlToOpen}`;
    console.log('최종 URL:', finalUrl);
    
    try {
      // 브라우저의 기본 window.open 메서드 사용
      window.open(finalUrl, '_blank');
    } catch (error) {
      console.error('링크 열기 실패:', error);
      // 실패 시 사용자에게 알림
      alert(`링크를 열 수 없습니다. 수동으로 접속해 주세요: ${finalUrl}`);
    }
  }
  
  // 상세 페이지로 이동하는 함수
  function goToDetail() {
    const params = new URLSearchParams({
      id: id.toString(),
      title,
      description,
      url: url || '',
      stars: stars.toString()
    });
    
    window.location.href = `/detail?${params.toString()}`;
  }
</script>

<!-- 재사용 가능한 MCP 카드 컴포넌트 -->
{#if onClick}
  <button 
    class="card card-border w-full shadow-sm {
      variant === 'default' ? 'bg-base-100' : 
      variant === 'primary' ? 'bg-primary text-primary-content' : 
      'bg-accent text-accent-content'
    } {className} text-left"
    on:click={onClick}
  >
    <div class="card-body h-[160px] flex flex-col">
      <div class="flex justify-between items-start">
        <h2 class="card-title text-lg">{title}</h2>
        <div class="flex items-center gap-1 flex-shrink-0">
          <!-- 스타 아이콘과 개수 -->
          {#if stars > 0}
            <Star class="text-yellow-400" size={18} />
            <span>{formatStars(stars)}</span>
          {/if}
          
          <!-- GitHub 링크 -->
          {#if url}
            <!-- URL이 있는 경우: 클릭 가능한 링크 -->
            <span
              on:click|stopPropagation={() => openGitHub(url)}
              class="ml-2 text-gray-600 hover:text-black focus:outline-none cursor-pointer"
            >
              <Github size={18} />
            </span>
          {:else}
            <!-- URL이 없는 경우: 비활성화된 상태로 표시 -->
            <span class="ml-2 text-gray-400 opacity-40">
              <Github size={18} />
            </span>
          {/if}
        </div>
      </div>
      <div class="w-full">
        <p class="text-sm overflow-hidden line-clamp-2 max-w-[85%]">{displayDescription}</p>
      </div>
      <div class="card-actions justify-end mt-1">
        <span
          on:click|stopPropagation={goToDetail}
          class="btn btn-sm btn-primary cursor-pointer"
        >
          상세보기
        </span>
      </div>
    </div>
  </button>
{:else}
  <div class="card card-border w-full shadow-sm {
      variant === 'default' ? 'bg-base-100' : 
      variant === 'primary' ? 'bg-primary text-primary-content' : 
      'bg-accent text-accent-content'
    } {className}">
    <div class="card-body h-[160px] flex flex-col">
      <div class="flex justify-between items-start">
        <h2 class="card-title text-lg">{title}</h2>
        <div class="flex items-center gap-1 flex-shrink-0">
          <!-- 스타 아이콘과 개수 -->
          {#if stars > 0}
            <Star class="text-yellow-400" size={18} />
            <span>{formatStars(stars)}</span>
          {/if}
          
          <!-- GitHub 링크 -->
          {#if url}
            <!-- URL이 있는 경우: 클릭 가능한 링크 -->
            <button
              on:click={() => openGitHub(url)}
              class="ml-2 text-gray-600 hover:text-black focus:outline-none"
            >
              <Github size={18} />
            </button>
          {:else}
            <!-- URL이 없는 경우: 비활성화된 상태로 표시 -->
            <span class="ml-2 text-gray-400 opacity-40">
              <Github size={18} />
            </span>
          {/if}
        </div>
      </div>
      <div class="w-full">
        <p class="text-sm overflow-hidden line-clamp-2 max-w-[85%]">{displayDescription}</p>
      </div>
      <div class="card-actions justify-end mt-1">
        <button
          on:click={goToDetail}
          class="btn btn-sm btn-primary"
        >
          상세보기
        </button>
      </div>
    </div>
  </div>
{/if}