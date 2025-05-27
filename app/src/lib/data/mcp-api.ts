import { invoke } from "@tauri-apps/api/core"
import { handleTauriError } from "../error-handler"

export interface MCPCard {
  id: number
  title: string
  description: string
  url: string // GitHub repository URL
  stars: number // GitHub stars count
  installed?: boolean // Installation status (optional property added)
  scanned?: boolean // Security scan status (v2+ API)
  security_rank?: string // Security rank from backend
  securityRank?: "CRITICAL" | "HIGH" | "MODERATE" | "LOW" | "UNRATE" // Mapped security rank for frontend
}

export interface PageInfo {
  has_next_page: boolean
  end_cursor: number | null
  total_items: number
}

export interface MCPCardResponse {
  cards: MCPCard[]
  page_info: PageInfo
}

// Interface for detailed MCP card information
export interface MCPCardDetail extends MCPCard {
  args?: string[]
  env?: Record<string, any>
  command?: string
  // scanned is already included from MCPCard
  // security_rank and securityRank are already included from MCPCard
}

/**
 * Fetch MCP card data from the external server via the Rust backend.
 * @returns MCP card data array
 */
export async function fetchMCPCards(searchTerm?: string, cursorId?: number): Promise<MCPCardResponse> {
  // get card data
  return handleTauriError(
    async () => {
      // configure API call parameters
      let params = {}
      if (searchTerm && searchTerm.trim() !== "") {
        params = { searchTerm }
      } else if (cursorId) {
        params = { cursorId }
      }

      // Tauri API call
      const response = Object.keys(params).length > 0 ? await invoke<MCPCardResponse>("get_mcp_data", params) : await invoke<MCPCardResponse>("get_mcp_data")

      // check installation status and set security rank
      await Promise.all(
        response.cards.map(async (card) => {
          try {
            card.installed = await invoke<boolean>("is_mcp_server_installed", {
              serverName: card.title,
            })

            // convert backend security_rank to frontend securityRank
            card.securityRank = (card.security_rank as any) || "UNRATE"
          } catch (err) {
            card.installed = false
            card.securityRank = "UNRATE" // fallback value when error occurs
          }
        })
      )

      return response
    },
    {
      cards: [],
      page_info: {
        has_next_page: false,
        end_cursor: null,
        total_items: 0,
      },
    }
  )
}

/**
 * Define the MCPServerConfig interface
 */
export interface MCPServerConfig {
  command: string
  args?: string[]
  env?: Record<string, any>
  cwd?: string | null
}

/**
 * Fetch detailed MCP card data from the Rust backend.
 * @param id The ID of the MCP card
 * @param title The title (name) of the MCP card (for checking installation status)
 * @returns Detailed MCP card data
 */
export async function fetchMCPCardDetail(id: number, title?: string): Promise<MCPCardDetail> {
  // default empty card (fallback value when error occurs)
  const emptyCard: MCPCardDetail = {
    id: id,
    title: title || "Unknown",
    description: "Error occurred while fetching information.",
    url: "",
    stars: 0,
    securityRank: "UNRATE",
    args: [],
    env: {},
    command: "",
    scanned: false,
  }

  return handleTauriError(async () => {
    // get basic info from API
    const detailFromAPI = await invoke<MCPCardDetail>("get_mcp_detail_data", { id })

    // convert backend security_rank to frontend securityRank
    detailFromAPI.securityRank = (detailFromAPI.security_rank as any) || "UNRATE"

    // if title is provided, check installation status
    if (title) {
      const isInstalled = await handleTauriError(async () => await invoke<boolean>("is_mcp_server_installed", { serverName: title }), false, { showToast: false })

      // if already installed, get config from config file
      if (isInstalled) {
        try {
          const config = await invoke<MCPServerConfig>("read_mcp_server_config", { serverName: title })

          // merge API result and installed config (installed config has priority)
          return {
            ...detailFromAPI,
            installed: true,
            command: config.command || detailFromAPI.command || "",
            args: config.args || detailFromAPI.args || [],
            env: config.env || detailFromAPI.env || {},
            securityRank: detailFromAPI.securityRank || "UNRATE", // keep security rank
            security_rank: detailFromAPI.security_rank, // keep original security_rank
          }
        } catch (configErr) {
          return {
            ...detailFromAPI,
            installed: true,
          }
        }
      } else {
        // if not installed
        return {
          ...detailFromAPI,
          installed: false,
        }
      }
    }

    // return API result by default
    return {
      ...detailFromAPI,
      installed: false,
    }
  }, emptyCard)
}
