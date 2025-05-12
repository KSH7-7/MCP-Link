import { invoke } from "@tauri-apps/api/core"

export interface MCPCard {
  id: number
  title: string
  description: string
  url: string // GitHub 리포지토리 URL
  stars: number // GitHub 스타 수
}

// MCP 카드의 상세 정보 인터페이스
export interface MCPCardDetail extends MCPCard {
  args?: string[]
  env?: Record<string, any>
  command?: string
}

/**
 * Fetch MCP card data from the external server via the Rust backend.
 * @returns MCP card data array
 */
export async function fetchMCPCards(searchTerm?: string): Promise<MCPCard[]> {
  try {
    if (searchTerm === undefined || searchTerm.trim() === "") {
      console.log("[mcp-api.ts] fetchMCPCards: No search term or empty. Invoking get_mcp_data WITHOUT params.")
      return await invoke<MCPCard[]>("get_mcp_data") // 인자 없이 호출
    } else {
      console.log(`[mcp-api.ts] fetchMCPCards: Searching for "${searchTerm}". Invoking get_mcp_data WITH searchTerm (camelCase).`)
      return await invoke<MCPCard[]>("get_mcp_data", { searchTerm: searchTerm }) // 'searchTerm' (camelCase)으로 전달
    }
  } catch (error) {
    console.error("[mcp-api.ts] Error fetching MCP card data:", error)
    throw error
  }
}

/**
 * Fetch detailed MCP card data from the Rust backend.
 * @param id The ID of the MCP card
 * @returns Detailed MCP card data
 */
export async function fetchMCPCardDetail(id: number): Promise<MCPCardDetail> {
  try {
    console.log(`[mcp-api.ts] fetchMCPCardDetail: Fetching detail for ID ${id}.`)
    return await invoke<MCPCardDetail>("get_mcp_detail_data", { id }) // id를 객체로 감싸서 전달
  } catch (error) {
    console.error(`[mcp-api.ts] Error fetching MCP card detail for ID ${id}:`, error)
    throw error
  }
}
