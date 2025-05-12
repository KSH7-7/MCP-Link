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
            // Assuming the API returns an object with a 'tags' property (Array<string>)
            // e.g., { tags: ["tag1", "tag2"] }
            // Or if it returns the array directly, use: const response = await this.http.get<string[]>('/tags');
            const response = await this.http.get<{ tags: string[] }>('/tags');
            return response.data.tags; 
        } catch (error) {
            console.error('Error fetching tags from crawler service:', error);
            // Depending on error handling strategy, you might throw a custom error
            // or return an empty array / default set of tags.
            throw new Error('Failed to fetch tags from crawler service.');
        }
    }
}
