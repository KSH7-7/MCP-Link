import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js"
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js"
import { FallbackService } from "./core/services/fallback.service.js"
import { CrawlerAdapter } from "./infrastructure/adapters/crawler.adapter.js"
import { GuiAdapter } from "./infrastructure/adapters/gui.adapter.js"
import { McpAdapter } from "./infrastructure/adapters/mcp.adapter.js"
import { config } from "./infrastructure/config/index.js"

async function bootstrap() {
  console.error("[Main] Initializing application for Stdio transport...")

  // Instantiate Adapters
  const crawlerAdapter = new CrawlerAdapter()
  const guiAdapter = new GuiAdapter()
  console.error("[Main] Adapters initialized.")

  // Instantiate Core Service
  const fallbackService = new FallbackService(crawlerAdapter, guiAdapter)
  console.error("[Main] Core service initialized.")

  // Instantiate MCP Adapter
  const mcpAdapter = new McpAdapter(fallbackService)
  console.error("[Main] MCP adapter initialized.")

  // Create MCP Server
  console.error(`[Main] Creating MCP Server: ${config.serverName || "FallbackMCPDevServer"} v${config.serverVersion || "0.0.1"}`)
  const mcpServer = new McpServer({
    name: config.serverName || "FallbackMCPDevServer",
    version: config.serverVersion || "0.0.1",
  })
  console.error("[Main] MCP Server instance created.")

  // Register Tools with MCP Server
  console.error(`[Main] Registering tool: ${mcpAdapter.toolName}...`)
  mcpServer.tool(
    mcpAdapter.toolName,
    mcpAdapter.description,
    mcpAdapter.inputSchema.shape, // Pass the .shape of the Zod schema
    mcpAdapter.handleCallTool
  )
  console.error(`[Main] Tool '${mcpAdapter.toolName}' registered successfully.`)

  // Create Stdio Transport and Connect
  console.error("[Main] Creating StdioServerTransport...")
  const transport = new StdioServerTransport() // Uses default stdin/stdout
  console.error("[Main] StdioServerTransport created.")

  console.error("[Main] Connecting MCP Server to Stdio transport...")
  try {
    // McpServer.connect() internally calls transport.start().
    await mcpServer.connect(transport)
    console.error("[Main] MCP Server successfully connected and listening via Stdio.")
    // Stdio server waits for input from the parent process, so no separate 'keep alive' logic is needed.
  } catch (error) {
    console.error("[Main] Failed to connect MCP Server to Stdio transport:", error)
    process.exit(1)
  }
}

// --- Application Entry Point ---
bootstrap().catch((error) => {
  console.error("[Main] Unhandled error during bootstrap:", error)
  process.exit(1)
})
