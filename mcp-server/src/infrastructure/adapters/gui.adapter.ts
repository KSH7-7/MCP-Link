import axios, { AxiosInstance } from 'axios';
import { RecommendationNotifier } from '../../core/ports/recommendation.notifier.port';
import { config } from '../config';

/**
 * Adapts the RecommendationNotifier port to interact with the GUI Backend API.
 */
export class GuiAdapter implements RecommendationNotifier {
  private readonly http: AxiosInstance;

  constructor() {
    this.http = axios.create({
      baseURL: config.guiBeApiBaseUrl,
      timeout: 5000, // Example timeout
      headers: { 'Content-Type': 'application/json' },
    });
  }

  /**
   * Sends the extracted keywords to the GUI Backend API.
   * Assumes the API endpoint is `/notify-keywords` relative to the base URL.
   * Sends the keywords as a JSON payload in the request body.
   * @param keywords An array of extracted keyword strings.
   * @returns A promise that resolves when the notification is sent (or fails).
   */
  async notifyKeywords(keywords: string[]): Promise<void> {
    const endpoint = '/notify-keywords'; // Assumption, adjust if needed
    const payload = { keywords };

    try {
      console.log(`[GUI Adapter] Notifying GUI BE at ${config.guiBeApiBaseUrl}${endpoint} with keywords:`, keywords);
      const response = await this.http.post(endpoint, payload);

      if (response.status >= 200 && response.status < 300) {
        console.log('[GUI Adapter] Successfully notified GUI BE.');
        // No return value needed for void promise on success
      } else {
        console.error(`[GUI Adapter] Unexpected status code from GUI BE: ${response.status}`, response.data);
        // Still resolve the promise, but log the error. 
        // Or throw an error if the caller needs to know about notification failures.
      }
    } catch (error) {
      // Type check for error
      if (error instanceof Error) {
        console.error('[GUI Adapter] Error notifying GUI BE:', error.message);
      } else {
        console.error('[GUI Adapter] An unknown error occurred while notifying GUI BE:', error);
      }
      // Decide if errors should be thrown or just logged. For now, just log.
      // throw new Error(`Failed to notify GUI BE: ${error.message || error}`);
    }
  }
}
