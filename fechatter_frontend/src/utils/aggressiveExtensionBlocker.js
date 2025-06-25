/**
 * Aggressive Extension Blocker
 * Prevents browser extensions from interfering with the application
 */

class AggressiveExtensionBlocker {
  constructor() {
    this.blockedCount = 0;
    this.detectedExtensions = new Set();
    this.originalFetch = window.fetch;
    this.initialized = false;
  }

  /**
   * Initialize the blocker
   */
  initialize() {
    if (this.initialized) return;

    console.log('🛡️ Initializing Aggressive Extension Blocker...');

    // 1. Override window.fetch to intercept extension requests
    this.overrideFetch();

    // 2. Block known extension patterns
    this.blockKnownPatterns();

    // 3. Monitor for extension-injected elements
    this.monitorDOMChanges();

    // 4. Protect XMLHttpRequest
    this.protectXHR();

    // 5. Create isolated communication channel
    this.createIsolatedChannel();

    this.initialized = true;
    console.log('✅ Aggressive Extension Blocker initialized');
  }

  /**
   * Override fetch to block extension requests
   */
  overrideFetch() {
    const self = this;

    window.fetch = function (...args) {
      const [resource, config] = args;

      // Check if this is an extension request
      if (self.isExtensionRequest(resource)) {
        self.blockedCount++;
        console.warn(`🚫 Blocked extension request: ${resource}`);

        // Return a rejected promise to prevent the request
        return Promise.reject(new Error('Extension request blocked'));
      }

      // For our own requests, use the original fetch
      return self.originalFetch.apply(window, args);
    };
  }

  /**
   * Check if a request is from an extension
   */
  isExtensionRequest(url) {
    if (!url) return false;

    const urlStr = url.toString();

    // Known extension patterns
    const extensionPatterns = [
      'chrome-extension://',
      'moz-extension://',
      'extension://',
      'content_script.js',
      'content script.js',
      'inject.js',
      'injected.js',
      'immersive-translate',
      'translate.google',
      'papago',
      'grammarly'
    ];

    return extensionPatterns.some(pattern =>
      urlStr.toLowerCase().includes(pattern)
    );
  }

  /**
   * Block known problematic patterns
   */
  blockKnownPatterns() {
    // Block postMessage from extensions
    const originalPostMessage = window.postMessage;
    window.postMessage = function (message, targetOrigin, ...args) {
      if (typeof message === 'object' && message?.source?.includes('extension')) {
        console.warn('🚫 Blocked extension postMessage');
        return;
      }
      return originalPostMessage.apply(window, [message, targetOrigin, ...args]);
    };

    // Block addEventListener for extension events
    const originalAddEventListener = EventTarget.prototype.addEventListener;
    EventTarget.prototype.addEventListener = function (type, listener, ...args) {
      // Check if listener is from an extension
      const listenerStr = listener?.toString() || '';
      if (listenerStr.includes('chrome.runtime') ||
        listenerStr.includes('browser.runtime') ||
        listenerStr.includes('sendMessage')) {
        console.warn('🚫 Blocked extension event listener');
        return;
      }
      return originalAddEventListener.apply(this, [type, listener, ...args]);
    };
  }

  /**
   * Monitor DOM for extension-injected elements
   */
  monitorDOMChanges() {
    const observer = new MutationObserver((mutations) => {
      mutations.forEach((mutation) => {
        mutation.addedNodes.forEach((node) => {
          if (node.nodeType === 1) { // Element node
            // Check for extension signatures
            if (this.isExtensionElement(node)) {
              console.warn('🚫 Removing extension-injected element:', node);
              node.remove();
              this.blockedCount++;
            }
          }
        });
      });
    });

    observer.observe(document.body, {
      childList: true,
      subtree: true
    });
  }

  /**
   * Check if an element is from an extension
   */
  isExtensionElement(element) {
    const signatures = [
      'immersive-translate',
      'grammarly',
      'translate-',
      'extension-',
      '__translator',
      'papago'
    ];

    // Check class names
    const className = element.className?.toString() || '';
    if (signatures.some(sig => className.includes(sig))) {
      return true;
    }

    // Check id
    const id = element.id || '';
    if (signatures.some(sig => id.includes(sig))) {
      return true;
    }

    // Check data attributes
    for (let attr of element.attributes || []) {
      if (attr.name.startsWith('data-') &&
        signatures.some(sig => attr.value.includes(sig))) {
        return true;
      }
    }

    return false;
  }

  /**
   * Protect XMLHttpRequest
   */
  protectXHR() {
    const OriginalXHR = window.XMLHttpRequest;
    const self = this;

    window.XMLHttpRequest = function () {
      const xhr = new OriginalXHR();
      const originalOpen = xhr.open;

      xhr.open = function (method, url, ...args) {
        if (self.isExtensionRequest(url)) {
          console.warn('🚫 Blocked XHR request from extension:', url);
          self.blockedCount++;
          throw new Error('Extension XHR blocked');
        }
        return originalOpen.apply(xhr, [method, url, ...args]);
      };

      return xhr;
    };
  }

  /**
   * Create isolated communication channel
   */
  createIsolatedChannel() {
    // Create a unique namespace for our app's communications
    window.__fechatter_isolated__ = {
      requests: new Map(),
      messageChannel: new MessageChannel()
    };

    // Use MessageChannel for isolated communication
    window.__fechatter_isolated__.messageChannel.port1.onmessage = (event) => {
      // Handle internal messages safely
      if (event.data?.type === 'fechatter_internal') {
        // Process internal messages
        console.log('📨 Processing isolated message:', event.data);
      }
    };
  }

  /**
   * Get blocker statistics
   */
  getStats() {
    return {
      initialized: this.initialized,
      blockedCount: this.blockedCount,
      detectedExtensions: Array.from(this.detectedExtensions),
      isActive: this.initialized && this.blockedCount > 0
    };
  }

  /**
   * Reset blocker (for testing)
   */
  reset() {
    window.fetch = this.originalFetch;
    this.blockedCount = 0;
    this.detectedExtensions.clear();
    this.initialized = false;
  }
}

// Create singleton instance
const blocker = new AggressiveExtensionBlocker();

// Auto-initialize on load
if (typeof window !== 'undefined') {
  // Wait for DOM ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
      blocker.initialize();
    });
  } else {
    blocker.initialize();
  }

  // Expose for debugging
  window.extensionBlocker = blocker;

  console.log('🛡️ Aggressive Extension Blocker loaded');
  console.log('Usage:');
  console.log('  - window.extensionBlocker.getStats() - View blocking statistics');
  console.log('  - window.extensionBlocker.reset() - Reset blocker (testing only)');
}

export default blocker; 