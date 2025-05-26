import { invoke } from "@tauri-apps/api/core"
import { handleTauriError } from '../error-handler'

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
  // 카드 데이터 가져오기
  return handleTauriError(async () => {
    // API 호출 파라미터 구성
    let params = {};
    if (searchTerm && searchTerm.trim() !== "") {
      params = { searchTerm };
    } else if (cursorId) {
      params = { cursorId };
    }
    
    // Tauri API 호출
    const response = Object.keys(params).length > 0
      ? await invoke<MCPCardResponse>("get_mcp_data", params)
      : await invoke<MCPCardResponse>("get_mcp_data");

    // 설치 상태 확인 및 보안 등급 설정
    await Promise.all(response.cards.map(async (card) => {
      try {
        card.installed = await invoke<boolean>("is_mcp_server_installed", {
          serverName: card.title,
        });
        
        // 백엔드의 security_rank를 프론트엔드용 securityRank로 변환
        card.securityRank = (card.security_rank as any) || "UNRATE";
      } catch (err) {
        console.error(`${card.title} 설치 상태 확인 실패:`, err);
        card.installed = false;
        card.securityRank = "UNRATE"; // 오류 시 기본값
      }
    }));

    return response;
  }, { 
    cards: [], 
    page_info: { 
      has_next_page: false, 
      end_cursor: null, 
      total_items: 0 
    } 
  });
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
  // 기본 빈 카드 (오류 발생 시 폴백 값)
  const emptyCard: MCPCardDetail = {
    id: id,
    title: title || "알 수 없음",
    description: "정보를 가져오는 중 오류가 발생했습니다.",
    url: "",
    stars: 0,
    securityRank: "UNRATE",
    args: [],
    env: {},
    command: "",
    scanned: false
  };

  return handleTauriError(async () => {
    // API에서 기본 정보 가져오기
    const detailFromAPI = await invoke<MCPCardDetail>("get_mcp_detail_data", { id });
    
    // 백엔드의 security_rank를 프론트엔드용 securityRank로 변환
    detailFromAPI.securityRank = (detailFromAPI.security_rank as any) || "UNRATE";

    // 제목이 제공된 경우 설치 상태 확인
    if (title) {
      const isInstalled = await handleTauriError(
        async () => await invoke<boolean>("is_mcp_server_installed", { serverName: title }),
        false,
        { showToast: false }
      );

      // 이미 설치된 경우 설정 파일에서 값 가져오기
      if (isInstalled) {
        try {
          const config = await invoke<MCPServerConfig>("read_mcp_server_config", { serverName: title });

          // API 결과와 설치된 구성 병합 (설치된 구성이 우선함)
          return {
            ...detailFromAPI,
            installed: true,
            command: config.command || detailFromAPI.command || "",
            args: config.args || detailFromAPI.args || [],
            env: config.env || detailFromAPI.env || {},
            securityRank: detailFromAPI.securityRank || "UNRATE", // 보안 랭크 유지
            security_rank: detailFromAPI.security_rank // 원본 security_rank도 유지
          };
        } catch (configErr) {
          console.warn(`설치된 구성이 있지만 읽기 실패: ${configErr}`);
          return {
            ...detailFromAPI,
            installed: true
          };
        }
      } else {
        // 설치되지 않은 경우
        return {
          ...detailFromAPI,
          installed: false
        };
      }
    }

    // 기본적으로 API 결과 반환
    return {
      ...detailFromAPI,
      installed: false
    };
  }, emptyCard);
}
