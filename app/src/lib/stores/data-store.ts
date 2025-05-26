import { writable } from "svelte/store"

// 페이지 간 공유될 데이터를 위한 스토어
export const sharedDataStore = writable({
  counts: {}, // 각종 카운트 데이터를 저장하는 객체
  loaded: false, // 데이터 로딩 상태
})

// 데이터 업데이트 함수
export function updateCount(key: string, value: number): void {
  sharedDataStore.update((data) => {
    const updatedData = {
      ...data,
      counts: {
        ...data.counts,
        [key]: value,
      },
    }
    return updatedData
  })
}

// 데이터 로드 완료 표시 함수
export function setLoaded(isLoaded: boolean): void {
  sharedDataStore.update((data) => ({
    ...data,
    loaded: isLoaded,
  }))
}

// 데이터 초기화 함수
export function initializeData(): void {
  sharedDataStore.set({
    counts: {},
    loaded: false,
  })
}
