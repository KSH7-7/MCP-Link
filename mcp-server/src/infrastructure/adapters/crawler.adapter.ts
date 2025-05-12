import axios, { AxiosInstance } from 'axios';
import { TagProvider } from '../../core/ports/tag.provider.port.js';
import { config } from '../config/index.js';

/**
 * Adapts the TagProvider port to interact with the Crawler Service API.
 */
export class CrawlerAdapter implements TagProvider {
    private readonly http: AxiosInstance;

    constructor() {
        this.http = axios.create({
            baseURL: config.crawlerApiBaseUrl,
        });
    }

    async getAllTags(): Promise<string[]> {
        try {
            // Assuming the API returns an object with a 'data' property,
            // which in turn has an 'mcpTags' property (Array<string>)
            // e.g., { timestamp: "...", code: "...", message: "...", data: { mcpTags: ["tag1", "tag2"] } }
            const response = await this.http.get<{ data: { mcpTags: string[] } }>('/tags');
            
            // Validate the structure before accessing
            if (response.data && response.data.data && Array.isArray(response.data.data.mcpTags)) {
                // console.log('[CrawlerAdapter] Successfully fetched tags:', response.data.data.mcpTags);
                return response.data.data.mcpTags;
            } else {
                console.error('[CrawlerAdapter] Unexpected API response structure. `response.data.data.mcpTags` is not an array or is missing.');
                return []; // Return an empty array as a fallback
            }
        } catch (error) {
            console.error('Error fetching tags from crawler service:', error);
            // Depending on error handling strategy, you might throw a custom error
            // or return an empty array / default set of tags.
            throw new Error('Failed to fetch tags from crawler service.');
        }
    }
}
