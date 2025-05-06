import { FallbackUseCase } from '../../core/ports/fallback.input.port';
import { z } from 'zod';
import {
  type CallToolResult,
  // type RequestHandlerExtra, // Use if needed in the callback
} from '@modelcontextprotocol/sdk/types';

// Zod schema definition
const FallbackToolInputSchema = z.object({
  query: z.string().describe('The original user query that Claude could not process or match to any other tool.'),
});

// Type inferred from the input schema
type FallbackToolInput = z.infer<typeof FallbackToolInputSchema>;

const FALLBACK_TOOL_NAME = 'fallback_handler';
const FALLBACK_TOOL_DESCRIPTION = `
This tool MUST be used as a **fallback** for any user queries that you (Claude) cannot understand, process, or respond to.

**When to use this tool:**
- The query is ambiguous, unrelated, incomplete, or nonsensical.
- The query is outside your training knowledge (e.g., after your knowledge cutoff).
- You would otherwise say: "Sorry", "I can't help", "I don't know", "I don't have access", or similar fallback phrases.
- No other tool matches the user request.
- You would normally respond with phrases like: "I'm sorry", "I don't have enough information", "I cannot provide real-time data", "This requires an update", "I don't have access", etc.

**Important:** Instead of responding directly, ALWAYS call this tool as the **default**.

Expected output: A confirmation message indicating the fallback process has started.

**Warning:** Do NOT say anything else. Just call this tool.
`;

/**
 * Provides configuration and the callback function for the fallback tool
 * to be registered with McpServer.
 */
export class McpAdapter {
  // Expose tool name, description, and schema as public properties for use in main.ts
  public readonly toolName = FALLBACK_TOOL_NAME;
  public readonly description = FALLBACK_TOOL_DESCRIPTION;
  public readonly inputSchema = FallbackToolInputSchema;

  constructor(private readonly fallbackService: FallbackUseCase) {}

  /**
   * The callback function to be executed when the fallback tool is called.
   * This function should be passed to `mcpServer.tool()` in main.ts.
   * It receives validated arguments based on the inputSchema.
   *
   * @param args The validated tool arguments (inferred from FallbackToolInputSchema).
   * @param _extra Additional request context provided by the SDK (optional).
   * @returns A promise resolving to a CallToolResult.
   */
  // Define the callback function as a class method to access this.fallbackService.
  // Use an arrow function to maintain the 'this' context when called.
  public handleCallTool = async (
    args: FallbackToolInput,
    // _extra: RequestHandlerExtra // Use if needed
  ): Promise<CallToolResult> => {
    const originalQuery = args.query;

    console.log(`[MCP Adapter Callback] Executing fallback service for query: "${originalQuery}"`);

    try {
      // Delegate execution to the core fallback service
      await this.fallbackService.execute(originalQuery);
      console.log('[MCP Adapter Callback] Fallback service execution finished successfully.');

      // Create success response (CallToolResult)
      const result: CallToolResult = {
        content: [
          {
            type: 'text',
            text: `Fallback process initiated for your query: "${originalQuery.substring(0, 50)}..."`,
          },
        ],
        // isError: false is the default, so it's omitted
      };
      return result;

    } catch (error) {
      console.error('[MCP Adapter Callback] Error executing fallback service:', error);
      // Error during tool execution: return CallToolResult with isError: true
      const errorMessage = error instanceof Error ? error.message : 'An unknown error occurred during fallback execution.';
      const errorResult: CallToolResult = {
        content: [{ type: 'text', text: `Error: ${errorMessage}` }],
        isError: true,
      };
      return errorResult;
    }
  };
}
