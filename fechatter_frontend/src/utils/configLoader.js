/**
 * Configuration Loader for Fechatter Frontend
 * Loads YAML configuration files based on environment
 */

import axios from 'axios';

let configCache = null;
let configPromise = null;

/**
 * 
 * Load configuration from YAML file
 * @param {string} environment - Environment name (development, production, staging)
 * @returns {Promise<Object>} Configuration object
 */
export async function loadConfig(environment = null) {
  // Determine environment
  const env = environment || import.meta.env.MODE || 'development';

  // Return cached config if available
  if (configCache && configCache.app?.environment === env) {
    return configCache;
  }

  // Return existing promise if loading
  if (configPromise) {
    return configPromise;
  }

  configPromise = loadConfigFromFile(env);

  try {
    configCache = await configPromise;
    return configCache;
  } finally {
    configPromise = null;
  }
}

/**
 * Load configuration from YAML file
 * @private
 */
async function loadConfigFromFile(env) {
  try {
    // Try to load YAML config file
    const configUrl = `/config/${env}.yml`;

    const response = await axios.get(configUrl, {
      timeout: 5000,
      headers: {
        'Accept': 'text/yaml, application/x-yaml, text/plain',
      }
    });

    // Parse YAML content
    const config = parseYAML(response.data);

    return config;

  } catch (error) {
    console.warn(`⚠️ Failed to load ${env}.yml config, falling back to default configuration`);
    console.warn('Error details:', error.message);

    // Fallback to default configuration
    return getDefaultConfig(env);
  }
}

/**
 * Simple YAML parser for basic configuration
 * @private
 */
function parseYAML(yamlText) {
  const lines = yamlText.split('\n');
  const result = {};
  const stack = [result];
  let currentLevel = 0;

  for (let line of lines) {
    // Skip comments and empty lines
    if (line.trim().startsWith('#') || line.trim() === '') continue;

    // Calculate indentation level
    const indent = line.length - line.trimStart().length;
    const level = Math.floor(indent / 2);

    // Adjust stack based on indentation
    while (stack.length > level + 1) {
      stack.pop();
    }

    const trimmedLine = line.trim();

    if (trimmedLine.includes(':')) {
      const [key, ...valueParts] = trimmedLine.split(':');
      const value = valueParts.join(':').trim();

      const current = stack[stack.length - 1];

      if (value === '' || value === null) {
        // Object/array declaration
        current[key.trim()] = {};
        stack.push(current[key.trim()]);
      } else if (value.startsWith('[') && value.endsWith(']')) {
        // Array value
        const arrayContent = value.slice(1, -1);
        if (arrayContent.trim()) {
          current[key.trim()] = arrayContent.split(',').map(item =>
            item.trim().replace(/^["']|["']$/g, '')
          );
        } else {
          current[key.trim()] = [];
        }
      } else if (value.startsWith('"') || value.startsWith("'")) {
        // String value
        current[key.trim()] = value.replace(/^["']|["']$/g, '');
      } else if (value === 'true' || value === 'false') {
        // Boolean value
        current[key.trim()] = value === 'true';
      } else if (!isNaN(value) && value !== '') {
        // Number value
        current[key.trim()] = parseInt(value) || parseFloat(value);
      } else {
        // String value without quotes
        current[key.trim()] = value;
      }
    } else if (trimmedLine.startsWith('- ')) {
      // Array item
      const current = stack[stack.length - 1];
      const value = trimmedLine.substring(2).trim().replace(/^["']|["']$/g, '');

      // Convert parent to array if it's not already
      const parentKey = Object.keys(current)[Object.keys(current).length - 1];
      if (!Array.isArray(current[parentKey])) {
        current[parentKey] = [];
      }
      current[parentKey].push(value);
    }
  }

  return result;
}

/**
 * Get default configuration when YAML loading fails
 * @private
 */
function getDefaultConfig(env) {
  const isDev = env === 'development';

  return {
    app: {
      name: "Fechatter",
      version: "1.0.0",
      environment: env,
      debug: isDev
    },
    frontend: {
      port: 1420,
      host: "localhost"
    },
    api: {
      gateway_url: "http://45.77.178.85:8080",
      base_url: "http://45.77.178.85:8080/api",
      file_url: "http://45.77.178.85:8080/files",
      // 使用SSE而不是WebSocket进行实时通信
      sse_url: "http://45.77.178.85:8080/events",
      notify_url: "http://45.77.178.85:8080",
      // 禁用SSE可以根据需要调整
      disable_sse: false,
      timeout: isDev ? 15000 : 30000,
      max_retries: isDev ? 3 : 5,
      retry_delay: isDev ? 1000 : 2000
    },
    cors: {
      enabled: true,
      origins: isDev
        ? ["http://localhost:1420", "http://127.0.0.1:1420"]
        : ["https://fechatter.v0.app", "https://*.v0.app"],
      credentials: true
    },
    logging: {
      level: isDev ? "debug" : "info",
      enable_network_logs: isDev,
      enable_api_logs: true
    }
  };
}

/**
 * Get current configuration (sync)
 * Returns cached config or null if not loaded
 */
export function getConfig() {
  return configCache;
}

/**
 * Get API configuration
 */
export function getApiConfig() {
  const config = getConfig();
  return config?.api || {};
}

/**
 * Get CORS configuration
 */
export function getCorsConfig() {
  const config = getConfig();
  return config?.cors || {};
}

/**
 * Get logging configuration
 */
export function getLoggingConfig() {
  const config = getConfig();
  return config?.logging || {};
}

/**
 * Check if debug mode is enabled
 */
export function isDebugEnabled() {
  const config = getConfig();
  return config?.app?.debug || false;
}

/**
 * Initialize configuration system
 * Should be called once at app startup
 */
export async function initializeConfig() {
  try {
    const config = await loadConfig();

    console.group('🎛️ Fechatter Configuration Initialized');
    console.log('Environment:', config.app?.environment);
    console.log('Gateway URL:', config.api?.gateway_url);
    console.log('Debug Mode:', config.app?.debug);
    console.log('CORS Origins:', config.cors?.origins);
    console.groupEnd();

    return config;
  } catch (error) {
    console.error('❌ Failed to initialize configuration:', error);
    throw error;
  }
}

export default {
  loadConfig,
  getConfig,
  getApiConfig,
  getCorsConfig,
  getLoggingConfig,
  isDebugEnabled,
  initializeConfig
}; 