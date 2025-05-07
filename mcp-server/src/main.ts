import { McpServer } from '@modelcontextprotocol/sdk/server/mcp';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio';
import { FallbackService } from './core/services/fallback.service';
import { CrawlerAdapter } from './infrastructure/adapters/crawler.adapter';
import { GuiAdapter } from './infrastructure/adapters/gui.adapter';
import { McpAdapter } from './infrastructure/adapters/mcp.adapter';
import { config } from './infrastructure/config/index';

async function bootstrap() {
  console.log('[Main] Initializing application for Stdio transport...');

  // Instantiate Adapters
  const crawlerAdapter = new CrawlerAdapter();
  const guiAdapter = new GuiAdapter();
  console.log('[Main] Adapters initialized.');

  // Instantiate Core Service
  const fallbackService = new FallbackService(crawlerAdapter, guiAdapter);
  console.log('[Main] Core service initialized.');

  // Instantiate MCP Adapter
  const mcpAdapter = new McpAdapter(fallbackService);
  console.log('[Main] MCP adapter initialized.');

  // Create MCP Server
  console.log(`[Main] Creating MCP Server: ${config.serverName || 'FallbackMCPDevServer'} v${config.serverVersion || '0.0.1'}`);
  const mcpServer = new McpServer({
    name: config.serverName || 'FallbackMCPDevServer',
    version: config.serverVersion || '0.0.1',
  });
  console.log('[Main] MCP Server instance created.');

  // Register Tools with MCP Server
  console.log(`[Main] Registering tool: ${mcpAdapter.toolName}...`);
  mcpServer.tool(
    mcpAdapter.toolName,
    mcpAdapter.description,
    mcpAdapter.inputSchema.shape, // Pass the .shape of the Zod schema
    mcpAdapter.handleCallTool
  );
  console.log(`[Main] Tool '${mcpAdapter.toolName}' registered successfully.`);

  // Create Stdio Transport and Connect
  console.log('[Main] Creating StdioServerTransport...');
  const transport = new StdioServerTransport(); // Uses default stdin/stdout
  console.log('[Main] StdioServerTransport created.');

  console.log('[Main] Connecting MCP Server to Stdio transport...');
  try {
    // McpServer.connect() internally calls transport.start().
    await mcpServer.connect(transport);
    console.log('[Main] MCP Server successfully connected and listening via Stdio.');
    // Stdio server waits for input from the parent process, so no separate 'keep alive' logic is needed.
  } catch (error) {
    console.error('[Main] Failed to connect MCP Server to Stdio transport:', error);
    process.exit(1);
  }
}

// --- Application Entry Point ---
bootstrap().catch((error) => {
  console.error('[Main] Unhandled error during bootstrap:', error);
  process.exit(1);
});