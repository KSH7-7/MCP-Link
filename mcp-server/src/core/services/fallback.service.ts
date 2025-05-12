import { FallbackUseCase } from "../ports/fallback.input.port.js";
import { TagProvider } from "../ports/tag.provider.port.js";
import { RecommendationNotifier } from "../ports/recommendation.notifier.port.js";

/**
 * Implements the core fallback logic using provided ports.
 * Assumes dependencies (TagProvider, RecommendationNotifier adapters) are injected.
 */
export class FallbackService implements FallbackUseCase {
  private loadedTags: string[] = [];
  private tagsLoaded: boolean = false;

  constructor(
    private readonly tagProvider: TagProvider,
    private readonly recommendationNotifier: RecommendationNotifier
  ) {}

  /**
   * Executes the fallback logic: load tags, extract keywords, and notify.
   * @param originalQuery The original user query.
   */
  async execute(originalQuery: string): Promise<void> {
    console.log(`[Core Service] Executing fallback for query: "${originalQuery}"`);

    try {
      await this.ensureTagsLoaded();
    } catch (error) {
      console.error("[Core Service] Failed to load tags. Aborting fallback logic.", error);
      // Stop execution if tags are essential for the fallback logic to proceed.
      return;
    }

    const keywords = this.extractKeywords(originalQuery, this.loadedTags);
    console.log(`[Core Service] Extracted keywords: ${JSON.stringify(keywords)}`);

    if (keywords.length > 0) {
      try {
        await this.recommendationNotifier.notifyKeywords(keywords);
        console.log("[Core Service] Successfully notified recommendation system.");
      } catch (error) {
        console.error("[Core Service] Failed to notify recommendation system.", error);
        // Continue execution even if notification fails, as the core logic might still be useful.
      }
    } else {
        console.log("[Core Service] No keywords found, skipping notification.");
    }

    console.log("[Core Service] Fallback execution finished.");
  }

  /**
   * Ensures tags are loaded into memory, fetching via TagProvider if necessary.
   * Uses a simple in-memory flag to avoid redundant fetches within the service instance lifetime.
   */
  private async ensureTagsLoaded(): Promise<void> {
    if (!this.tagsLoaded) {
      console.log("[Core Service] Loading tags...");
      // Assumes the TagProvider adapter handles the API response structure
      // and returns just the array of tag strings.
      this.loadedTags = await this.tagProvider.getAllTags();
      this.tagsLoaded = true;
      console.log(`[Core Service] Loaded ${this.loadedTags.length} tags.`);
    } else {
       console.log("[Core Service] Tags already loaded.");
    }
  }

  /**
   * Extracts keywords by checking for the presence of tags within the query.
   * Performs a simple case-insensitive substring match.
   * Relies on the TagProvider supplying comprehensive tag variations (e.g., case, synonyms).
   * @param query The user's query string.
   * @param tags The list of available tags provided by the TagProvider.
   * @returns An array of found keywords (maintaining original casing from the tag list).
   */
  private extractKeywords(query: string, tags: string[]): string[] {
    const foundKeywords: string[] = [];
    const lowerCaseQuery = query.toLowerCase(); // Prepare query for case-insensitive check

    for (const tag of tags) {
      // Check if the lowercased query includes the lowercased tag
      if (lowerCaseQuery.includes(tag.toLowerCase())) {
        foundKeywords.push(tag); // Store the original tag casing as the keyword
      }
    }
    // Returns all matches, including potential duplicates if a tag appears multiple times
    // or if variations of the same concept (e.g., "notion", "Notion") are matched.
    return foundKeywords;
  }
}
