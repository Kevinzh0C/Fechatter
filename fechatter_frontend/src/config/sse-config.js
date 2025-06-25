/**
 * SSE Configuration Manager - Simplified Version
 * Following Occam's Razor: Use the simplest solution that works
 */

// Import only what we need
import minimalSSE from '@/services/sse-minimal';

// Remove complex configuration - YAGNI
const SSE_CONFIG = {
  // One flag to rule them all
  USE_MINIMAL_SSE: true
};

/**
 * SSE Service Factory - Simplified
 * Always return the minimal service
 */
class SSEServiceFactory {
  static getService() {
    // Occam's Razor: One service, no complex switching
    return minimalSSE;
  }
}

// Export the minimal service directly
export default minimalSSE;

// Also export factory for compatibility
export { SSEServiceFactory, SSE_CONFIG }; 