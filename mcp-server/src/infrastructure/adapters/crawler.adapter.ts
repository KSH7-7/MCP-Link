import axios, { AxiosInstance } from "axios"
import { TagProvider } from "../../core/ports/tag.provider.port.js"
import { config } from "../config/index.js"

/**
 * Adapts the TagProvider port to interact with the Crawler Service API.
 */
export class CrawlerAdapter implements TagProvider {
  private readonly http: AxiosInstance

  constructor() {
    this.http = axios.create({
      baseURL: config.crawlerApiBaseUrl,
    })
  }

  async getAllTags(): Promise<string[]> {
    try {
      // Crawler 서버의 실제 응답 구조에 맞게 수정합니다.
      // 예상 구조: { data: { mcpTags: string[] } }
      const response = await this.http.get<{ data?: { mcpTags?: string[] } }>("/tags")

      if (response && response.data && response.data.data && Array.isArray(response.data.data.mcpTags)) {
        return response.data.data.mcpTags
      } else {
        console.error("Error fetching tags: Unexpected response structure from /tags endpoint. Response data:", response?.data)
        return [] // TypeError 방지를 위해 빈 배열 반환
      }
    } catch (error) {
      console.error("Error fetching tags from crawler service:", error)
      return [] // TypeError 방지를 위해 빈 배열 반환
    }
  }
}
