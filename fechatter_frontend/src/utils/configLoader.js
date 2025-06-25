/**
 * Static Configuration for Fechatter Frontend
 * Provides a reliable, single source of truth for configuration.
 */

// Determine environment
const env = import.meta.env.MODE || 'development';
const isDev = env === 'development';

const staticConfig = {
  app: {
    name: "Fechatter",
    version: "1.0.0",
    environment: env,
    debug: isDev
  },
  api: {
    base_url: "/api",
    timeout: isDev ? 15000 : 30000,
  },
  logging: {
    level: isDev ? "debug" : "info",
    enable_api_logs: true
  }
};

/**
 * Get current configuration (sync)
 */
export function getConfig() {
  return staticConfig;
}

/**
 * Get API configuration
 */
export function getApiConfig() {
  return staticConfig.api;
}

/**
 * Check if debug mode is enabled
 */
export function isDebugEnabled() {
  return staticConfig.app.debug;
}

if (import.meta.env.DEV) {
  console.log('🎛️ Fechatter Static Configuration Loaded');
  console.log('Environment:', staticConfig.app.environment);
}

export default {
  getConfig,
  getApiConfig,
  isDebugEnabled
}; 