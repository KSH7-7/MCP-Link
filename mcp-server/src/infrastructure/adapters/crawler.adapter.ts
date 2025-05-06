import axios, { AxiosInstance } from 'axios';
import { TagProvider } from '../../core/ports/tag.provider.port';
import { config } from '../config';

/**
 * Adapts the TagProvider port to interact with the Crawler API.
 */
export class CrawlerAdapter implements TagProvider {
  private readonly http: AxiosInstance;

  constructor() {
    this.http = axios.create({
      baseURL: config.crawlerApiBaseUrl,
      timeout: 5000, // Example timeout of 5 seconds
      headers: { 'Content-Type': 'application/json' },
    });
  }

  /**
   * Fetches the list of all tags from the Crawler API.
   * Assumes the API endpoint is `/servers/tags` relative to the base URL.
   * Assumes the API response structure has the tags in `response.data.data`.
   * @returns A promise resolving to an array of tag strings.
   */
  async getAllTags(): Promise<string[]> {
    try {
      console.log(`[Crawler Adapter] Fetching tags from ${config.crawlerApiBaseUrl}/servers/tags`);
      // The full URL will be constructed by axios based on baseURL + '/servers/tags'
      const response = await this.http.get<{ data: string[] }>('/servers/tags');

      if (response.status === 200 && Array.isArray(response.data?.data)) {
        console.log(`[Crawler Adapter] Successfully fetched ${response.data.data.length} tags.`);
        return response.data.data;
      } else {
        console.error('[Crawler Adapter] Unexpected response structure or status: ', response.status, response.data);
        return []; // Return empty array on unexpected structure
      }
    } catch (error) {
      // Type check for error before accessing properties
      if (error instanceof Error) {
        console.error('[Crawler Adapter] Error fetching tags:', error.message);
      } else {
        console.error('[Crawler Adapter] An unknown error occurred while fetching tags:', error);
      }
      // In case of network error or other issues, return an empty array
      // or re-throw a custom error if the service layer needs to handle it differently.
      return [];
    }
  }
}
