import dotenv from 'dotenv';
import { z } from 'zod';
import path from 'path';

// Determine the environment and load the appropriate .env file
const env = process.env.NODE_ENV || 'development';
const envPath = path.resolve(process.cwd(), `.env.${env}`);

dotenv.config({ path: envPath });
// Also load .env file if it exists (for general, non-environment-specific settings)
dotenv.config({ path: path.resolve(process.cwd(), '.env'), override: false });


// Define the schema for environment variables
const appSchema = z.object({
  NODE_ENV: z.enum(['development', 'production', 'test']).default('development'),
  GUI_BE_API_BASE_URL: z.string().url(),
  CRAWLER_API_BASE_URL: z.string().url(),
  SERVER_NAME: z.string().default('MCP Fallback Server'),
  SERVER_VERSION: z.string().default('0.1.0'),
  // Add other environment variables here
});

// Validate and export the configuration
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let validatedConfig: any;

try {
  validatedConfig = appSchema.parse(process.env);
} catch (error) {
  if (error instanceof z.ZodError) {
    console.error('Environment variable validation error:', error.errors);
    process.exit(1);
  }
  throw error; // Re-throw other errors
}

export const config = {
  nodeEnv: validatedConfig.NODE_ENV,
  crawlerApiBaseUrl: validatedConfig.CRAWLER_API_BASE_URL,
  guiBeApiBaseUrl: validatedConfig.GUI_BE_API_BASE_URL,
  serverName: validatedConfig.SERVER_NAME,
  serverVersion: validatedConfig.SERVER_VERSION,
  // Add other config properties here
};

// Define a type for the validated environment schema
type EnvSchemaType = z.infer<typeof appSchema>;

// More accurately type the exported 'config' object
export type AppConfig = {
  nodeEnv: EnvSchemaType['NODE_ENV'];
  crawlerApiBaseUrl: EnvSchemaType['CRAWLER_API_BASE_URL'];
  guiBeApiBaseUrl: EnvSchemaType['GUI_BE_API_BASE_URL'];
  serverName: EnvSchemaType['SERVER_NAME'];
  serverVersion: EnvSchemaType['SERVER_VERSION'];
  // Add other config properties here, mirroring the structure of the 'config' object
};

console.log('Configuration loaded:', config); // Optional: Log loaded config for verification
