    import { FallbackUseCase } from "../ports/fallback.input.port";
    import { TagProvider } from "../ports/tag.provider.port";
    import { RecommendationNotifier } from "../ports/recommendation.notifier.port";

    /**
     * Implements the core fallback logic.
     * This class depends only on the port interfaces, not concrete implementations.
     */
    export class FallbackService implements FallbackUseCase {
      private loadedTags: string[] = []; // For simple in-memory tag caching
      private tagsLoaded: boolean = false;

      /**
       * Injects dependencies (concrete adapters will be provided in main.ts).
       * @param tagProvider Port for fetching tags.
       * @param recommendationNotifier Port for notifying about keywords.
       */
      constructor(
        private readonly tagProvider: TagProvider,
        private readonly recommendationNotifier: RecommendationNotifier
      ) {
        // Optionally, load tags eagerly when the service is instantiated
        // this.loadTags(); // Or load lazily inside execute
      }

      /**
       * Executes the core fallback logic: load tags, extract keywords, notify GUI.
       * @param originalQuery The original user query.
       */
      async execute(originalQuery: string): Promise<void> {
        console.log(`[Core Service] Executing fallback for query: "${originalQuery}"`);

        // 1. Load tags (with simple caching)
        try {
          await this.ensureTagsLoaded();
        } catch (error) {
          console.error("[Core Service] Failed to load tags. Aborting fallback logic.", error);
          // Depending on requirements, you might want to proceed without tags
          // or throw an error to be caught by the adapter.
          return; // Stop execution if tags are essential
        }

        // 2. Extract keywords
        const keywords = this.extractKeywords(originalQuery, this.loadedTags);
        console.log(`[Core Service] Extracted keywords: ${JSON.stringify(keywords)}`);

        // 3. Notify GUI if keywords are found
        if (keywords.length > 0) {
          try {
            await this.recommendationNotifier.notifyKeywords(keywords);
            console.log("[Core Service] Successfully notified recommendation system.");
          } catch (error) {
            console.error("[Core Service] Failed to notify recommendation system.", error);
            // Log the error, but maybe don't stop the whole process? Depends on requirements.
          }
        } else {
            console.log("[Core Service] No keywords found, skipping notification.");
        }

        // The MCP response itself will be handled by the McpAdapter
        console.log("[Core Service] Fallback execution finished.");
      }

      /**
       * Ensures tags are loaded into memory, fetching if necessary.
       */
      private async ensureTagsLoaded(): Promise<void> {
        if (!this.tagsLoaded) {
          console.log("[Core Service] Loading tags...");
          this.loadedTags = await this.tagProvider.getAllTags();
          this.tagsLoaded = true;
          console.log(`[Core Service] Loaded ${this.loadedTags.length} tags.`);
        } else {
           console.log("[Core Service] Tags already loaded.");
        }
      }

      /**
       * Extracts keywords from the query based on the loaded tag list.
       * (This is the core algorithm to implement)
       * @param query The user's query string.
       * @param tags The list of available tags.
       * @returns An array of found keywords.
       */
      private extractKeywords(query: string, tags: string[]): string[] {
        // --- TODO: Implement Keyword Extraction Logic ---
        // This is a placeholder implementation.
        // You need a robust way to find relevant tags in the query.
        // Consider:
        // - Case-insensitive matching?
        // - Whole word matching?
        // - Handling synonyms or aliases (might need richer tag data)?
        // - Performance for large tag lists?

        const foundKeywords: string[] = [];
        const lowerCaseQuery = query.toLowerCase(); // Simple case-insensitive approach

        for (const tag of tags) {
          // Simple check if the query includes the tag (case-insensitive)
          // This might match partial words (e.g., "graph" in "photograph")
          // A more robust solution might use regex with word boundaries (\b)
          if (lowerCaseQuery.includes(tag.toLowerCase())) {
            foundKeywords.push(tag); // Store the original tag casing
          }
        }

        // Optional: Remove duplicates if a query might contain the same tag multiple times
        // return [...new Set(foundKeywords)];
        return foundKeywords;
        // --- End of TODO ---
      }
    }
