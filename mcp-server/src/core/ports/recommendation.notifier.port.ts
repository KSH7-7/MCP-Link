/**
 * Contract (Output Port) with the external system for sending recommendation notifications based on extracted keywords.
 * Defines how the core logic requests notifications.
 */
export interface RecommendationNotifier {
    /**
     * Sends the list of extracted keywords to request relevant processing.
     * @param keywords An array of extracted keyword strings.
     * @returns Promise<void> A Promise indicating the completion of the notification request dispatch.
     */
    notifyKeywords(keywords: string[]): Promise<void>;
  }
  