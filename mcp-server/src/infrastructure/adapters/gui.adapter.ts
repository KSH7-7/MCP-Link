import axios, { AxiosInstance } from 'axios';
import { RecommendationNotifier } from '../../core/ports/recommendation.notifier.port';
import { config } from '../config/index';

/**
 * Adapts the RecommendationNotifier port to interact with the GUI Backend API.
 */
export class GuiAdapter implements RecommendationNotifier {
    private readonly http: AxiosInstance;

    constructor() {
        this.http = axios.create({
            baseURL: config.guiBeApiBaseUrl,
        });
    }

    async notifyKeywords(keywords: string[]): Promise<void> {
        try {
            await this.http.post('/recommendations', { keywords });
            console.log(`Keywords notification sent: ${keywords.join(', ')}`);
        } catch (error) {
            console.error('Error sending keywords notification to GUI backend:', error);
            // Depending on error handling strategy, you might throw a custom error.
            // For now, we just log the error.
            // throw new Error('Failed to send keywords to GUI backend.');
        }
    }
}
