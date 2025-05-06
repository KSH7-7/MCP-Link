/**
 * Input Port (Use Case Interface) for executing the fallback logic.
 * Defines how the external world (Adapter) calls the core logic.
 */
export interface FallbackUseCase {
    /**
     * Executes the fallback logic.
     * @param originalQuery The original user query string that Claude could not process.
     * @returns Promise<void> A Promise indicating the completion of the operation. (Modify if a return value is needed)
     */
    execute(originalQuery: string): Promise<void>;
  }