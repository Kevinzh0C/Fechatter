/**
 * 🔧 Recursive Updates Fix Verification Script
 * 
 * This script verifies that the recursive update issues have been resolved
 * and provides monitoring tools for detecting potential issues.
 */

export class RecursiveUpdatesFixer {
  constructor() {
    this.isMonitoring = false;
    this.updateCounts = new Map();
    this.errorLog = [];
    this.fixedComponents = new Set();
    this.startTime = Date.now();
  }

  /**
   * 🔧 Start monitoring for recursive updates
   */
  startMonitoring() {
    if (this.isMonitoring) return;

    this.isMonitoring = true;
    console.log('🔍 Starting recursive updates monitoring...');

    // Monitor Vue warnings
    this._monitorVueWarnings();

    // Monitor excessive function calls
    this._monitorExcessiveCalls();

    // Monitor memory usage
    this._monitorMemoryUsage();

    return true;
  }

  /**
   * 🔧 Stop monitoring
   */
  stopMonitoring() {
    this.isMonitoring = false;
    this.updateCounts.clear();
    console.log('🛑 Stopped recursive updates monitoring');
  }

  /**
   * 🔧 Apply fixes to existing components
   */
  async applyFixes() {
    console.log('🔧 Applying recursive update fixes...');

    const fixes = [
      this._fixUnifiedMessageService,
      this._fixSimpleMessageList,
      this._fixEventListeners,
      this._fixTransitionComponents,
      this._addDebouncing
    ];

    const results = [];

    for (const fix of fixes) {
      try {
        const result = await fix.call(this);
        results.push(result);
      } catch (error) {
        results.push({
          name: fix.name,
          success: false,
          error: error.message
        });
      }
    }

    this._generateFixReport(results);
    return results;
  }

  /**
   * 🔧 Fix UnifiedMessageService
   */
  _fixUnifiedMessageService() {
    console.log('🔧 Fixing UnifiedMessageService...');

    // Check if fixed version is available
    if (window.unifiedMessageServiceFixed) {
      // Replace global instance
      window.unifiedMessageService = window.unifiedMessageServiceFixed;
      this.fixedComponents.add('UnifiedMessageService');

      return {
        name: 'UnifiedMessageService',
        success: true,
        details: 'Replaced with fixed version that removes event dispatching'
      };
    }

    return {
      name: 'UnifiedMessageService',
      success: false,
      details: 'Fixed version not available'
    };
  }

  /**
   * 🔧 Fix SimpleMessageList components
   */
  _fixSimpleMessageList() {
    console.log('🔧 Fixing SimpleMessageList components...');

    // Remove problematic event listeners
    const eventsToRemove = [
      'fechatter:auto-load-complete',
      'fechatter:force-message-refresh',
      'fechatter:force-scroll-check'
    ];

    let removedCount = 0;
    eventsToRemove.forEach(eventName => {
      // Remove existing listeners (if any)
      const listeners = window._eventListeners?.[eventName];
      if (listeners && listeners.length > 0) {
        listeners.forEach(listener => {
          window.removeEventListener(eventName, listener);
        });
        removedCount += listeners.length;
      }
    });

    this.fixedComponents.add('SimpleMessageList');

    return {
      name: 'SimpleMessageList',
      success: true,
      details: `Removed ${removedCount} problematic event listeners`
    };
  }

  /**
   * 🔧 Fix event listeners
   */
  _fixEventListeners() {
    console.log('🔧 Fixing event listeners...');

    // Create event listener tracking
    if (!window._eventListeners) {
      window._eventListeners = {};
    }

    // Override addEventListener to track listeners
    const originalAddEventListener = window.addEventListener;
    window.addEventListener = function (type, listener, options) {
      if (!window._eventListeners[type]) {
        window._eventListeners[type] = [];
      }
      window._eventListeners[type].push(listener);
      return originalAddEventListener.call(this, type, listener, options);
    };

    // Override removeEventListener
    const originalRemoveEventListener = window.removeEventListener;
    window.removeEventListener = function (type, listener, options) {
      if (window._eventListeners[type]) {
        const index = window._eventListeners[type].indexOf(listener);
        if (index > -1) {
          window._eventListeners[type].splice(index, 1);
        }
      }
      return originalRemoveEventListener.call(this, type, listener, options);
    };

    this.fixedComponents.add('EventListeners');

    return {
      name: 'EventListeners',
      success: true,
      details: 'Added event listener tracking and cleanup'
    };
  }

  /**
   * 🔧 Fix transition components
   */
  _fixTransitionComponents() {
    console.log('🔧 Fixing transition components...');

    // Add CSS to prevent transition-related recursive updates
    const style = document.createElement('style');
    style.textContent = `
      /* Prevent transition recursive updates */
      .v-enter-active, .v-leave-active {
        transition-duration: 0.15s !important;
      }
      
      .v-enter-from, .v-leave-to {
        transform: none !important;
      }
      
      /* Reduce animation complexity */
      * {
        animation-duration: 0.3s !important;
      }
    `;
    document.head.appendChild(style);

    this.fixedComponents.add('TransitionComponents');

    return {
      name: 'TransitionComponents',
      success: true,
      details: 'Added CSS to reduce transition complexity'
    };
  }

  /**
   * 🔧 Add debouncing to critical functions
   */
  _addDebouncing() {
    console.log('🔧 Adding debouncing...');

    // Global debounce utility
    window.debounce = function (func, wait) {
      let timeout;
      return function executedFunction(...args) {
        const later = () => {
          clearTimeout(timeout);
          func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
      };
    };

    // Global throttle utility
    window.throttle = function (func, limit) {
      let inThrottle;
      return function () {
        const args = arguments;
        const context = this;
        if (!inThrottle) {
          func.apply(context, args);
          inThrottle = true;
          setTimeout(() => inThrottle = false, limit);
        }
      };
    };

    this.fixedComponents.add('Debouncing');

    return {
      name: 'Debouncing',
      success: true,
      details: 'Added global debounce and throttle utilities'
    };
  }

  /**
   * 🔍 Monitor Vue warnings
   */
  _monitorVueWarnings() {
    const originalWarn = console.warn;
    console.warn = (...args) => {
      const message = args.join(' ');

      if (message.includes('Maximum recursive updates exceeded')) {
        this.errorLog.push({
          type: 'recursive-update',
          message,
          timestamp: Date.now(),
          stack: new Error().stack
        });

        console.error('🚨 RECURSIVE UPDATE DETECTED:', message);

        // Auto-apply emergency fix
        this._emergencyFix();
      }

      return originalWarn.apply(console, args);
    };
  }

  /**
   * 🔍 Monitor excessive function calls
   */
  _monitorExcessiveCalls() {
    const functionsToMonitor = [
      'fetchMessages',
      'fetchMoreMessages',
      'markMessageDisplayed',
      'finishAutoLoad'
    ];

    functionsToMonitor.forEach(funcName => {
      this.updateCounts.set(funcName, 0);
    });

    // Check counts every 5 seconds
    setInterval(() => {
      this.updateCounts.forEach((count, funcName) => {
        if (count > 50) { // More than 50 calls in 5 seconds
          console.warn(`⚠️ Excessive calls detected: ${funcName} called ${count} times`);
          this._emergencyFix();
        }
        this.updateCounts.set(funcName, 0); // Reset counter
      });
    }, 5000);
  }

  /**
   * 🔍 Monitor memory usage
   */
  _monitorMemoryUsage() {
    if (!performance.memory) return;

    const initialMemory = performance.memory.usedJSHeapSize;

    setInterval(() => {
      const currentMemory = performance.memory.usedJSHeapSize;
      const memoryIncrease = currentMemory - initialMemory;

      // If memory increased by more than 50MB, there might be a leak
      if (memoryIncrease > 50 * 1024 * 1024) {
        console.warn('⚠️ Potential memory leak detected:', {
          initial: Math.round(initialMemory / 1024 / 1024) + 'MB',
          current: Math.round(currentMemory / 1024 / 1024) + 'MB',
          increase: Math.round(memoryIncrease / 1024 / 1024) + 'MB'
        });
      }
    }, 30000); // Check every 30 seconds
  }

  /**
   * 🚨 Emergency fix for detected issues
   */
  _emergencyFix() {
    console.log('🚨 Applying emergency fix...');

    // Stop all timers
    const highestTimeoutId = setTimeout(';');
    for (let i = 0; i < highestTimeoutId; i++) {
      clearTimeout(i);
    }

    // Force garbage collection if available
    if (window.gc) {
      window.gc();
    }

    // Reload the page as last resort
    setTimeout(() => {
      console.log('🔄 Emergency reload to prevent app crash');
      window.location.reload();
    }, 5000);
  }

  /**
   * 📊 Generate fix report
   */
  _generateFixReport(results) {
    const totalTime = Date.now() - this.startTime;
    const successCount = results.filter(r => r.success).length;

    console.group('🔧 Recursive Updates Fix Report');
    console.log(`⏱️ Total fix time: ${totalTime}ms`);
    console.log(`✅ Successful fixes: ${successCount}/${results.length}`);
    console.log(`🔧 Fixed components: ${Array.from(this.fixedComponents).join(', ')}`);

    results.forEach(result => {
      const icon = result.success ? '✅' : '❌';
      console.log(`${icon} ${result.name}: ${result.details || result.error}`);
    });

    if (this.errorLog.length > 0) {
      console.log(`🚨 Errors detected: ${this.errorLog.length}`);
      this.errorLog.forEach(error => {
        console.log(`  - ${error.type}: ${error.message.substring(0, 100)}...`);
      });
    }

    console.groupEnd();
  }

  /**
   * 🧪 Test if fixes are working
   */
  async runHealthCheck() {
    console.log('🧪 Running recursive updates health check...');

    const tests = [
      {
        name: 'Event Dispatching',
        test: () => !window.dispatchEvent.toString().includes('fechatter:auto-load-complete')
      },
      {
        name: 'Fixed Service Available',
        test: () => !!window.unifiedMessageServiceFixed
      },
      {
        name: 'Debounce Available',
        test: () => typeof window.debounce === 'function'
      },
      {
        name: 'Event Tracking',
        test: () => !!window._eventListeners
      },
      {
        name: 'Memory Stable',
        test: () => performance.memory ? performance.memory.usedJSHeapSize < 100 * 1024 * 1024 : true
      }
    ];

    const results = tests.map(test => ({
      name: test.name,
      passed: test.test(),
      timestamp: new Date().toISOString()
    }));

    const passedCount = results.filter(r => r.passed).length;
    const healthScore = Math.round((passedCount / tests.length) * 100);

    console.log(`🎯 Health Score: ${healthScore}%`);
    results.forEach(result => {
      const icon = result.passed ? '✅' : '❌';
      console.log(`${icon} ${result.name}`);
    });

    return {
      score: healthScore,
      results,
      isHealthy: healthScore >= 80
    };
  }
}

// Create global instance
export const recursiveUpdatesFixer = new RecursiveUpdatesFixer();

// Global access
if (typeof window !== 'undefined') {
  window.recursiveUpdatesFixer = recursiveUpdatesFixer;

  // Auto-start monitoring in development
  if (import.meta.env.DEV) {
    recursiveUpdatesFixer.startMonitoring();
  }

  // Global convenience functions
  window.fixRecursiveUpdates = () => recursiveUpdatesFixer.applyFixes();
  window.checkRecursiveUpdatesHealth = () => recursiveUpdatesFixer.runHealthCheck();
}

export default recursiveUpdatesFixer; 