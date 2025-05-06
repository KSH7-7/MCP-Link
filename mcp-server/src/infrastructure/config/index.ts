import dotenv from 'dotenv';
import path from 'path';

// Load environment variables from .env file
dotenv.config({ path: path.resolve(process.cwd(), '.env') });

/**
 * Application configuration extracted from environment variables.
 */
interface AppConfig {
  crawlerApiBaseUrl: string;
  guiBeApiBaseUrl: string;
  fallbackServerPort: number;
}

// Function to get environment variable or throw error if not set
function getEnvVariable(key: string, defaultValue?: string): string {
  const value = process.env[key] || defaultValue;
  if (value === undefined) {
    throw new Error(`Missing environment variable: ${key}`);
  }
  return value;
}

// Function to get environment variable as number or throw error
function getEnvVariableAsNumber(key: string, defaultValue?: number): number {
  const valueStr = getEnvVariable(key, defaultValue?.toString());
  const value = parseInt(valueStr, 10);
  if (isNaN(value)) {
    throw new Error(`Invalid number format for environment variable: ${key}`);
  }
  return value;
}

// Export the configuration object
export const config: AppConfig = {
  crawlerApiBaseUrl: getEnvVariable('CRAWLER_API_BASE_URL', 'http://localhost:8081/api/v1/mcp'), // Example default
  guiBeApiBaseUrl: getEnvVariable('GUI_BE_API_BASE_URL', 'http://localhost:8082/api/v1'),     // Example default
  fallbackServerPort: getEnvVariableAsNumber('FALLBACK_SERVER_PORT', 3001),              // Example default
};

console.log('Configuration loaded:', config); // Optional: Log loaded config for verification
