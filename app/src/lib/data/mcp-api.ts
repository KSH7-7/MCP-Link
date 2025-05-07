import { invoke } from "@tauri-apps/api/core"

export interface MCPCard {
  id: number
  title: string
  description: string
  url: string // GitHub 리포지토리 URL
  stars: number // GitHub 스타 수
}

/**
 * Fetch MCP card data from the external server via the Rust backend.
 * @returns MCP card data array
 */
export async function fetchMCPCards(searchTerm?: string): Promise<MCPCard[]> {
  try {
    // searchTerm이 있을 때만 파라미터로 전달
    const response = searchTerm ? await invoke<MCPCard[]>("get_mcp_data", { searchTerm }) : await invoke<MCPCard[]>("get_mcp_data")
    return response
  } catch (error) {
    console.error("Error fetching MCP card data:", error)
    throw error
  }
}
