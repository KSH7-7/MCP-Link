import { FallbackUseCase } from '../../core/ports/fallback.input.port.js';
import { z } from 'zod';
import {
  type CallToolResult,
  // type RequestHandlerExtra, // Use if needed in the callback
} from '@modelcontextprotocol/sdk/types.js';

// Zod schema definition
const FallbackToolInputSchema = z.object({
  query: z.string().describe('The original user query that Claude could not process or match to any other tool.'),
});

// Type inferred from the input schema
type FallbackToolInput = z.infer<typeof FallbackToolInputSchema>;

const FALLBACK_TOOL_NAME = 'fallback_handler';
const FALLBACK_TOOL_DESCRIPTION = `
This tool MUST be used to initiate a search for a more specific tool if no other listed tool can directly fulfill the user's entire request. Its primary function is to trigger a process that helps the user discover and install the appropriate tool.

**When to use this tool (Call this tool IMMEDIATELY if):**
1.  The user's request implies a specific action or capability (e.g., writing to Notion, accessing local files, interacting with a specific API or service like Google Maps) AND no other tool in your current list explicitly provides this capability.
2.  No other tool is a clear and direct match for the user's **entire concrete task**. Even if you understand the request, if fulfilling it requires a specific tool you don't have, use this.
3.  You would otherwise tell the user you cannot perform the action directly, or offer generic instructions instead of performing the action (e.g., "To do X, you would need to open application Y and do Z...").

**Crucial Instructions:**
-   **DO NOT** attempt to answer the user's request directly with generic information, by stating your limitations, or by providing step-by-step instructions if their request clearly implies a task that could be performed by a specialized (potentially installable) tool.
-   **ALWAYS prioritize calling this \`fallback_handler\`** in such cases. This allows the system to check for and recommend a suitable tool to the user.
-   This handler is your **default and immediate action** for unhandled tool-requiring tasks. It is better to call this handler than to say "I can't do that."

Expected output: A confirmation message indicating the fallback and tool recommendation process has started.

**Warning:** Do NOT say anything else. Just call this tool with the original user query.
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

    // console.log(`[MCP Adapter Callback] Executing fallback service for query: "${originalQuery}"`);

    try {
      // Delegate execution to the core fallback service
      await this.fallbackService.execute(originalQuery);
      // console.log('[MCP Adapter Callback] Fallback service execution finished successfully.');

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
