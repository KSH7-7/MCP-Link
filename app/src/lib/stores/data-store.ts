import { writable } from "svelte/store"

// store for data shared between pages
export const sharedDataStore = writable({
  counts: {}, // object to store various count data
  loaded: false, // data loading status
})

// update data function
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

// display data loading complete function
export function setLoaded(isLoaded: boolean): void {
  sharedDataStore.update((data) => ({
    ...data,
    loaded: isLoaded,
  }))
}

// initialize data function
export function initializeData(): void {
  sharedDataStore.set({
    counts: {},
    loaded: false,
  })
}
