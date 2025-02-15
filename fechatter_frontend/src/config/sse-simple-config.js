/**
 * Simplified SSE Configuration
 * Following Occam's Razor: Less is More
 */

export const SIMPLE_SSE_CONFIG = {
  // Single retry limit - no complex multi-layer retries
  MAX_RETRIES: 2,  // Only retry twice, fail fast

  // Simple backoff - no complex strategies
  RETRY_DELAY: 3000,  // Fixed 3 second delay

  // User experience first
  SILENT_AFTER_FIRST_FAILURE: true,  // Don't spam users with errors

  // Progressive disclosure
  SHOW_CONNECTION_STATUS: false,  // Hide technical details by default

  // Performance budget
  CONNECTION_TIMEOUT: 10000,  // 10 seconds max

  // Feature flags
  DISABLE_IN_DEVELOPMENT: false,  // Keep it simple - same behavior in all envs
  USE_MOCK_IN_OFFLINE: true,  // Graceful degradation when offline
};

/**
 * Get simplified error message for users
 * No technical jargon, just actionable advice
 */
export function getUserFriendlyError(error) {
  // Check if offline
  if (!navigator.onLine) {
    return null;  // Silent when offline - expected behavior
  }

  // For all other errors, keep it simple
  return {
    message: "实时更新暂时不可用",
    action: "页面会自动重连",
    showRefresh: false  // Don't encourage page refresh for transient issues
  };
} 