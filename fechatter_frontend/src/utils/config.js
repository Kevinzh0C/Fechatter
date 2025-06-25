/**
 * Application Configuration Management
 * Centralizes environment variables and provides type-safe configuration access
 */

// =============================================================================
// Environment Configuration
// =============================================================================

// 统一配置文件 - 通过vite代理访问，避免CORS
export const config = {
  // API配置 - 使用相对路径
  baseURL: import.meta.env.VITE_API_BASE_URL || '/api',
  sseURL: import.meta.env.VITE_SSE_URL || '/events',
  fileBaseURL: import.meta.env.VITE_FILE_BASE_URL || '/files',
  timeout: parseInt(import.meta.env.VITE_REQUEST_TIMEOUT) || 15000,
  maxRetries: parseInt(import.meta.env.VITE_MAX_RETRIES) || 3,
  retryDelay: parseInt(import.meta.env.VITE_RETRY_DELAY) || 1000,

  // Application Configuration
  app: {
    name: import.meta.env.VITE_APP_NAME || 'Fechatter',
    version: import.meta.env.VITE_APP_VERSION || '1.0.0',
    description: import.meta.env.VITE_APP_DESCRIPTION || 'Modern team chat application',
    environment: import.meta.env.MODE || 'development',
  },

  // Development Configuration
  dev: {
    debug: import.meta.env.VITE_DEBUG === 'true',
    enableDevtools: import.meta.env.VITE_ENABLE_DEVTOOLS === 'true',
    isDevelopment: import.meta.env.DEV,
    isProduction: import.meta.env.PROD,
  },

  // Feature Flags
  features: {
    fileUpload: import.meta.env.VITE_ENABLE_FILE_UPLOAD !== 'false',
    voiceMessages: import.meta.env.VITE_ENABLE_VOICE_MESSAGES === 'true',
    videoCalls: import.meta.env.VITE_ENABLE_VIDEO_CALLS === 'true',
    notifications: import.meta.env.VITE_ENABLE_NOTIFICATIONS !== 'false',
    darkMode: import.meta.env.VITE_ENABLE_DARK_MODE !== 'false',
    animations: import.meta.env.VITE_ENABLE_ANIMATIONS !== 'false',
  },

  // UI Configuration
  ui: {
    defaultTheme: import.meta.env.VITE_DEFAULT_THEME || 'light',
    lazyLoading: import.meta.env.VITE_LAZY_LOADING !== 'false',
    imageOptimization: import.meta.env.VITE_IMAGE_OPTIMIZATION !== 'false',
  },

  // Performance Configuration
  performance: {
    cacheDuration: parseInt(import.meta.env.VITE_CACHE_DURATION) || 3600,
  },

  // Security Configuration
  security: {
    enableCSP: import.meta.env.VITE_ENABLE_CSP !== 'false',
    allowedOrigins: (import.meta.env.VITE_ALLOWED_ORIGINS || 'http://localhost:*,http://127.0.0.1:*').split(','),
  },
};

// =============================================================================
// Configuration Utilities
// =============================================================================

/**
 * Check if a feature is enabled
 * @param {string} featureName - Name of the feature
 * @returns {boolean} Whether the feature is enabled
 */
export const isFeatureEnabled = (featureName) => {
  return config.features[featureName] || false;
};

/**
 * Get API configuration
 * @returns {object} API configuration object
 */
export const getApiConfig = () => ({ ...config.api });

/**
 * Get app configuration
 * @returns {object} App configuration object
 */
export const getAppConfig = () => ({ ...config.app });

/**
 * Check if running in development mode
 * @returns {boolean} True if in development mode
 */
export const isDevelopment = () => config.dev.isDevelopment;

/**
 * Check if running in production mode
 * @returns {boolean} True if in production mode
 */
export const isProduction = () => config.dev.isProduction;

/**
 * Check if debug mode is enabled
 * @returns {boolean} True if debug mode is enabled
 */
export const isDebugEnabled = () => config.dev.debug;

/**
 * Get environment-specific configuration
 * @param {string} key - Configuration key
 * @param {*} defaultValue - Default value if key not found
 * @returns {*} Configuration value
 */
export const getEnvConfig = (key, defaultValue = null) => {
  const keys = key.split('.');
  let current = config;

  for (const k of keys) {
    if (current[k] === undefined) {
      return defaultValue;
    }
    current = current[k];
  }

  return current;
};

/**
 * Validate required environment variables
 * @returns {object} Validation result with missing variables
 */
export const validateEnvironment = () => {
  const required = [
    'VITE_API_BASE_URL',
  ];

  const missing = required.filter(key => !import.meta.env[key]);

  return {
    isValid: missing.length === 0,
    missing,
    warnings: missing.length > 0 ? ['Some environment variables are missing'] : [],
  };
};

/**
 * Get configuration summary for debugging
 * @returns {object} Configuration summary
 */
export const getConfigSummary = () => {
  if (!config.dev.debug) {
    return { debug: 'Debug mode disabled' };
  }

  return {
    api: {
      baseURL: config.baseURL,
      timeout: config.timeout,
      maxRetries: config.maxRetries,
    },
    app: config.app,
    features: config.features,
    environment: config.app.environment,
    validation: validateEnvironment(),
  };
};

/**
 * Initialize configuration and perform validations
 */
export const initializeConfig = () => {
  const validation = validateEnvironment();

  if (config.dev.debug) {
    console.group('🔧 Fechatter Configuration');
    console.log('Environment:', config.app.environment);
    console.log('API Base URL:', config.baseURL);
    console.log('Features:', config.features);

    if (!validation.isValid) {
      console.warn('Missing environment variables:', validation.missing);
    }

    console.groupEnd();
  }

  return validation;
};

// =============================================================================
// Exports
// =============================================================================

export default config; 