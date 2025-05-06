/**
 * Contract (Output Port) with the external system responsible for providing tag list data.
 * Defines how the core logic requests tag data.
 */
export interface TagProvider {
    /**
     * Gets the list of all available tags.
     * @returns Promise<string[]> A Promise resolving to an array of tag strings.
     */
    getAllTags(): Promise<string[]>;
  }