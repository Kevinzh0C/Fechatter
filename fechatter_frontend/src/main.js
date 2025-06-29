import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import router from "./router";
import authPlugin from "./plugins/auth";
import { errorHandler } from "./utils/errorHandler";
// Removed unused protobuf analytics import to avoid variable name conflict
import sseConnectionManager from './utils/sseConnectionManager';
import minimalMessagePersistence from './utils/minimalMessagePersistence';
import minimalSSE from './services/sse-minimal';
// Import extension error suppressor to handle browser extension errors
import './utils/extensionErrorSuppressor.js';

// AESTHETIC RESTORATION: Import unified design system FIRST to override conflicts
import './styles/unified-aesthetic-system.css';
import './styles/unified-channel-fonts.css';
import './style.css';
import './styles/z-index.css';

// CODE HIGHLIGHTING: Import enhanced code highlighting styles
import './styles/enhanced-code-highlight.css';

// 🛡️ Security Initializations - MUST load first (restored after aesthetic fix)
import secureLogger from './utils/secureLogger'
import './utils/extensionErrorSuppressor'
import requestConflictResolver from './utils/requestConflictResolver'

// Performance monitoring (development only)
import performanceMonitor from './utils/performanceMonitor'

// NEW: System Monitor for comprehensive debugging and performance monitoring
import { systemMonitor } from './utils/systemMonitor.js';

// 🏥 NEW: Quick Health Check for instant system status
import { quickHealthCheck } from './utils/quickHealthCheck.js';

// Service imports
import { useAuthStore } from './stores/auth'
import { useChatStore } from './stores/chat'
import { useUserStore } from './stores/user'
import { useWorkspaceStore } from './stores/workspace'

// 🛡️ NEW: Import Message Display Guarantee system
import { messageDisplayGuarantee } from './services/messageSystem/MessageDisplayGuarantee.js';
import { messageDisplayGuaranteeVerification } from './utils/messageDisplayGuaranteeVerification.js';

// NEW: Import UnifiedMessageService for global availability
import { unifiedMessageService } from './services/messageSystem/UnifiedMessageService.js';

import { guaranteedScrollToBottom } from '@/services/GuaranteedScrollToBottom.js'
import { scrollToBottomValidator } from '@/utils/scrollToBottomValidator.js'

// Import Analytics client instance
import analytics from './services/analytics-client.js'

// Enhanced console state detection function
function detectConsoleAccess() {
  let devtools = {
    open: false,
    orientation: null
  };

  const threshold = 160;

  if (window.outerHeight - window.innerHeight > threshold ||
    window.outerWidth - window.innerWidth > threshold) {
    devtools.open = true;
    devtools.orientation = window.outerHeight - window.innerHeight > threshold ? 'vertical' : 'horizontal';
  }

  return devtools;
}

// Enhanced console state detection
const devtools = detectConsoleAccess();

// --- 🛡️ SIMPLIFIED SAFETY INITIALIZATION ---
// Simplified safety - no external dependency needed
const simplifiedSafety = {
  initialize: (devtools) => {
    if (import.meta.env.DEV) {
      console.log('🛡️ Simplified Safety active (Development Mode)');
    }
  },
  getSecurityStatus: () => ({
    debugSuppressed: import.meta.env.PROD,
    environment: import.meta.env.MODE,
    devtools: detectConsoleAccess()
  })
};

simplifiedSafety.initialize(devtools);
console.log('🛡️ Production Safety Wrapper active');

// --- PERFORMANCE MONITORING ---
performanceMonitor.initialize();
console.log('Performance Monitor loaded - use window.perfAnalytics() to view stats');

// --- SYSTEM MONITORING ---
if (import.meta.env.DEV) {
  console.log('System Monitor initialized - use window.debugSystem() for health check');
  console.log('Available debug commands: debugSystem(), debugMessageService(), debugCache(), debugComponents(), debugPerformance(), debugMemory(), debugFullReport()');
  console.log('🏥 Quick Health Check available - use window.quickHealthCheck() for instant status overview');
}

// --- INITIALIZE CORE APPLICATION ---
const app = createApp(App);
const pinia = createPinia();

app.config.errorHandler = (err, instance, info) => {
  errorHandler.handle(err, {
    context: 'Vue Error Handler',
    component: instance?.$options?.name || 'Unknown Component',
    info
  });
};

app.use(pinia);
app.use(router);
app.use(authPlugin);

// Make analytics available globally
app.config.globalProperties.$analytics = analytics
app.provide('analytics', analytics)

// Load debug tools in development
if (import.meta.env.DEV) {
  import('./utils/messageDebugger');
  import('./utils/performanceMonitor');
  import('./utils/duplicateDebugger');
  import('./utils/devConsoleHelpers'); // Import dev console helpers for log management
  import('./utils/messageDisplayDiagnostics'); // Import message display diagnostics
  import('./utils/authDiagnostics'); // Import authentication diagnostics
  import('./utils/systemDiagnostics'); // Import master system diagnostics
  import('./utils/messageServiceDiagnostics'); // Import UnifiedMessageService diagnostics
  import('./utils/fetchMoreMessagesTest'); // Import fetchMoreMessages fix verification
  import('./utils/systemHealthValidator'); // Import system health validator
  import('./utils/sseDebugger'); // Import SSE debugger for comprehensive SSE diagnosis

  // 🆕 NEW: Import advanced UX systems for development
  import('./services/ProgressiveLoadManager.js'); // Import progressive loading system
  import('./services/MessageSessionGrouper.js'); // Import message grouping system
}

// Import performance validator for development testing
if (import.meta.env.DEV) {
  import('./utils/navigationPerformanceValidator.js').then(module => {
    console.log('Navigation Performance Validator loaded - use window.testNavigationPerformance(fromChatId, toChatId) to test');
  });
}

// Initialize security measures
if (import.meta.env.DEV) {
  console.log('🔒 Initializing Fechatter Security Layer...')
}

// Apply simplified safety policies
const securityStatus = simplifiedSafety.getSecurityStatus()
if (import.meta.env.DEV) {
  console.log('🛡️ Security Status:', securityStatus)
}

// Set up global error handling with security considerations
window.addEventListener('error', (event) => {
  errorHandler.handle(event.error, {
    context: 'Global Error Handler',
    filename: event.filename
  })
})

window.addEventListener('unhandledrejection', (event) => {
  errorHandler.handle(event.reason, {
    context: 'Unhandled Promise Rejection'
  })
})

// Initialize stores with error handling
async function initializeStores() {
  try {
    const authStore = useAuthStore()
    const chatStore = useChatStore()
    const userStore = useUserStore()
    const workspaceStore = useWorkspaceStore()

    // Initialize auth state from storage
    await authStore.initialize()

    // ENHANCED: More strict SSE connection validation
    if (import.meta.env.DEV) {
      console.log('[SSE] Checking SSE initialization conditions...');
      console.log('[SSE] authStore.isAuthenticated:', authStore.isAuthenticated);
      console.log('[SSE] authStore.token exists:', !!authStore.token);
      console.log('[SSE] Current route:', router.currentRoute.value?.path);
      if (authStore.token) {
        console.log('[SSE] Token preview:', authStore.token.substring(0, 20) + '...');
      }
    }

    // CRITICAL FIX: More comprehensive authentication validation
    const isUserLoggedIn = authStore.isAuthenticated &&
      authStore.token &&
      authStore.user &&
      authStore.token.length > 50; // Ensure token is substantial

    // ADDITIONAL: Don't initialize SSE on login/register pages
    const currentPath = router.currentRoute.value?.path || window.location.pathname;
    const isOnAuthPage = currentPath.includes('/login') ||
      currentPath.includes('/register') ||
      currentPath === '/';

    if (import.meta.env.DEV) {
      console.log('[SSE] Enhanced validation:');
      console.log('  - isUserLoggedIn:', isUserLoggedIn);
      console.log('  - isOnAuthPage:', isOnAuthPage);
      console.log('  - currentPath:', currentPath);
    }

    if (isUserLoggedIn && !isOnAuthPage) {
      try {
        if (import.meta.env.DEV) {
          console.log('🔗 [SSE] ALL CONDITIONS MET! Initializing SSE connection...');
          console.log('🔗 [SSE] User:', authStore.user?.email || 'Unknown');
          console.log('🔗 [SSE] minimalSSE service:', !!minimalSSE);
        }

        minimalSSE.connect(authStore.token);

        // Wait a moment and verify connection
        setTimeout(() => {
          const sseStatus = minimalSSE.getStatus();
          if (import.meta.env.DEV) {
            console.log('SUBSCRIPTION: [SSE] Startup connection status:', sseStatus);
            console.log('SUBSCRIPTION: [SSE] EventSource exists:', !!minimalSSE.eventSource);
            if (minimalSSE.eventSource) {
              console.log('SUBSCRIPTION: [SSE] EventSource URL:', minimalSSE.eventSource.url);
              console.log('SUBSCRIPTION: [SSE] EventSource readyState:', minimalSSE.eventSource.readyState);
            }
          }
        }, 2000);

      } catch (sseError) {
        if (import.meta.env.DEV) {
          console.error('ERROR: [SSE] Failed to initialize SSE on startup:', sseError);
        }
      }
    } else {
      if (import.meta.env.DEV) {
        console.warn('WARNING: [SSE] SKIPPING SSE initialization:');
        console.warn('  - isAuthenticated:', authStore.isAuthenticated);
        console.warn('  - token exists:', !!authStore.token);
        console.warn('  - user exists:', !!authStore.user);
        console.warn('  - token length:', authStore.token?.length || 0);
        console.warn('  - isOnAuthPage:', isOnAuthPage);
        console.warn('  - currentPath:', currentPath);
        console.warn('SSE will initialize after successful login and navigation to chat');
      }
    }

    if (import.meta.env.DEV) {
      console.log('Application stores initialized')
    }

    if (import.meta.env.DEV) {
      performanceMonitor.endOperation('app-initialization', {
        status: 'success',
        storesInitialized: true
      })
    }

  } catch (error) {
    if (import.meta.env.DEV) {
      console.error('ERROR: Failed to initialize stores:', error)
    }
    errorHandler.handle(error, {
      context: 'Store Initialization',
      critical: true
    })
  }
}

// --- 🛡️ MESSAGE DISPLAY GUARANTEE INITIALIZATION ---
if (import.meta.env.DEV) {
  console.log('🛡️ Initializing Message Display Guarantee system...');
}

// Enable the guarantee system
messageDisplayGuarantee.setEnabled(true);

// Make globally available for debugging
window.messageDisplayGuarantee = messageDisplayGuarantee;

// 🛡️ NEW: Make UnifiedMessageService globally available for system monitoring
window.unifiedMessageService = unifiedMessageService;

if (import.meta.env.DEV) {
  console.log('Message Display Guarantee system ready');
  console.log('UnifiedMessageService globally available');

  // Add debugging helper
  window.debugMessageGuarantee = () => {
    console.group('🛡️ Message Display Guarantee Debug Info');
    console.log('Metrics:', messageDisplayGuarantee.getMetrics());
    console.log('Full Debug Info:', messageDisplayGuarantee.exportDebugInfo());
    console.groupEnd();
  };

  // NEW: Add quick control methods
  window.disableMessageGuarantee = (duration = 30000) => {
    messageDisplayGuarantee.temporaryDisable(duration);
  };

  window.enableMessageGuaranteeDebug = () => {
    messageDisplayGuarantee.enableDebugMode();
  };

  window.disableMessageGuaranteeDebug = () => {
    messageDisplayGuarantee.disableDebugMode();
  };

  // 🛡️ NEW: Initialize verification system
  console.log('Initializing Message Display Guarantee Verification...');
  window.messageDisplayGuaranteeVerification = messageDisplayGuaranteeVerification;

  // Add verification command
  window.verifyMessageDisplayGuarantee = async () => {
    console.log('🛡️ Starting Message Display Guarantee Verification...');
    return await messageDisplayGuaranteeVerification.runCompleteVerification();
  };

  console.log('Verification system ready - use window.verifyMessageDisplayGuarantee() to test');
  console.log('🔬 Message Display Diagnostics ready - use diagnoseDhat(chatId) to analyze message display issues');
  console.log('🔐 Authentication Diagnostics ready - use diagnoseDAuth() to analyze auth issues, fixDAuth() to auto-fix');
  console.log('EVENT: Message Service Diagnostics ready - use diagnoseDMessageService(chatId) to analyze service issues, fixDMessageService() to auto-fix');
  console.log('FetchMore Fix Test ready - use testFetchMoreFix(chatId) to verify the ReferenceError fix, quickTestFetchMore(chatId) for quick check');
  console.log('🏥 Master System Diagnostics ready - use diagnoseDSystem(chatId) for complete health check, fixDSystem() to auto-fix all');
}

// Initialize stores with guarantee system integration
initializeStores().then(() => {
  console.log('Application stores initialized');
}).catch(error => {
  console.error('Failed to initialize stores:', error);
});

// Mount application with security checks
async function mountApp() {
  try {
    await initializeStores()

    // REFACTORED: Use dependency injection instead of global access
    const authStore = useAuthStore()
    const chatStore = useChatStore()
    const userStore = useUserStore()
    const workspaceStore = useWorkspaceStore()

    // NEW: Configure dependency injection container
    if (import.meta.env.DEV) {
      import('./services/DIContainer.js').then(({ container, configureServices }) => {
        configureServices(container)
        container.install(app)
        console.log('Dependency injection container configured')
      }).catch(error => {
        console.warn('WARNING: DIContainer failed to load:', error)
      })
    }

    // DEPRECATED: Keep minimal global access for backward compatibility
    // Will be removed in future refactoring phases
    if (import.meta.env.DEV) {
      window.__pinia_stores__ = {
        auth: () => authStore,
        chat: () => chatStore,
        user: () => userStore,
        workspace: () => workspaceStore,
      };
      console.warn('WARNING: Global store access is deprecated. Use dependency injection instead.')
    }

    // Security validation
    if (import.meta.env.PROD && !securityStatus.debugSuppressed) {
      if (import.meta.env.DEV) {
        console.warn('WARNING: Security Warning: Debug logs not properly suppressed in production')
      }
    }

    app.mount('#app')
    if (import.meta.env.DEV) {
      console.log('Fechatter application mounted successfully')
    }

    // Cleanup on page unload
    window.addEventListener('beforeunload', () => {
      if (import.meta.env.DEV) {
        performanceMonitor.clearReports()
      }
      requestConflictResolver.abortAllRequests()
    })

  } catch (error) {
    if (import.meta.env.DEV) {
      console.error('ERROR: Failed to mount application:', error)
    }
    errorHandler.handle(error, {
      context: 'Application Mount',
      critical: true
    })

    // Show user-friendly error
    document.body.innerHTML = `
      <div style="
        display: flex; 
        justify-content: center; 
        align-items: center; 
        height: 100vh; 
        font-family: Arial, sans-serif;
        background: #f5f5f5;
      ">
        <div style="
          text-align: center; 
          padding: 2rem; 
          background: white; 
          border-radius: 8px; 
          box-shadow: 0 2px 10px rgba(0,0,0,0.1);
          max-width: 400px;
        ">
          <h2 style="color: #e74c3c; margin-bottom: 1rem;">Application Error</h2>
          <p style="color: #666; margin-bottom: 1.5rem;">
            Unable to start the application. Please refresh the page or contact support.
          </p>
          <button 
            onclick="window.location.reload()" 
            style="
              background: #3498db; 
              color: white; 
              border: none; 
              padding: 0.75rem 1.5rem; 
              border-radius: 4px; 
              cursor: pointer;
              font-size: 1rem;
            "
          >
            Refresh Page
          </button>
        </div>
      </div>
    `
  }
}

// Start application
mountApp()

window.app = app;
window.pinia = pinia;
window.$router = router;
window.errorHandler = errorHandler;
window.sseConnectionManager = sseConnectionManager;

// CRITICAL FIX: Expose tokenManager to window for SSE service access
import tokenManager from './services/tokenManager.js';
window.tokenManager = tokenManager;

// Expose security utilities for API parameter limiting
window.securityUtils = {
  enforceApiLimits: (params) => {
    // Limit API request parameters to prevent DoS
    const MAX_LIMIT = 100;
    const sanitized = { ...params };

    if (sanitized.limit && sanitized.limit > MAX_LIMIT) {
      sanitized.limit = MAX_LIMIT;
      if (import.meta.env.DEV) {
        console.warn(`🚨 API limit parameter capped from ${params.limit} to ${MAX_LIMIT}`);
      }
    }

    return sanitized;
  },

  sanitizeDisplayId: (id) => {
    // Obfuscate IDs for display in logs
    if (!id) return 'unknown';
    return `${String(id).slice(0, 2)}***`;
  }
};

window.addEventListener('beforeunload', () => {
  analytics.track('app_exit', { exit_code: 1 });
  analytics.destroy();
});

// Initialize guaranteed scroll system
if (import.meta.env.DEV) {
  window.guaranteedScrollToBottom = guaranteedScrollToBottom;
  console.log('GuaranteedScrollToBottom system loaded');
}
