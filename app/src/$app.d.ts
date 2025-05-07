// temporary navigation module
// (to delete 'Cannot find module '$app/navigation' or its corresponding type declarations.' warning)
declare module "$app/navigation" {
  export function goto(
    url: string | URL,
    opts?: {
      replaceState?: boolean
      noscroll?: boolean
      keepfocus?: boolean
      state?: any
      invalidateAll?: boolean
    }
  ): Promise<void>

  export function invalidate(url: string | RegExp): Promise<void>
  export function prefetch(url: string): Promise<void>
  export function prefetchRoutes(urls?: string[]): Promise<void>
}
