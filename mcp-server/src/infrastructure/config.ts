export const config = {
  serverName: process.env.MCP_SERVER_NAME || 'FallbackMCPDevServer',
  serverVersion: process.env.MCP_SERVER_VERSION || '0.0.1',
  crawlerApiBaseUrl: process.env.CRAWLER_API_BASE_URL || 'http://localhost:8081/api/v1', // Example default
  guiBeApiBaseUrl: process.env.GUI_BE_API_BASE_URL || 'http://localhost:8082/api/v1', // Example default
  // Add other server-specific configurations here if needed
  // e.g., serverName: 'MCP Fallback Server',
};

// Optional: Export a type for better type checking in other parts of the application
export type AppConfig = typeof config; 