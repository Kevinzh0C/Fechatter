/**
 * Performance optimization utilities
 * Provides debounce and throttle functions for efficient event handling
 */

/**
 * Debounce function - executes after delay when calls stop
 * @param {Function} func - Function to debounce
 * @param {number} delay - Delay in milliseconds
 * @param {boolean} immediate - Execute immediately on first call
 * @returns {Function} Debounced function
 */
export function debounce(func, delay = 300, immediate = false) {
  let timeoutId;
  let lastCallTime = 0;

  return function executedFunction(...args) {
    const context = this;

    const callNow = immediate && !timeoutId;

    const later = () => {
      timeoutId = null;
      lastCallTime = Date.now();
      if (!immediate) {
        func.apply(context, args);
      }
    };

    clearTimeout(timeoutId);
    timeoutId = setTimeout(later, delay);

    if (callNow) {
      func.apply(context, args);
    }
  };
}

/**
 * Throttle function - executes at most once per interval
 * @param {Function} func - Function to throttle
 * @param {number} limit - Time limit in milliseconds
 * @param {Object} options - Configuration options
 * @returns {Function} Throttled function
 */
export function throttle(func, limit = 100, options = {}) {
  let lastCallTime = 0;
  let timeoutId = null;
  const { leading = true, trailing = true } = options;

  return function executedFunction(...args) {
    const context = this;
    const now = Date.now();

    const callFunction = () => {
      lastCallTime = now;
      func.apply(context, args);
    };

    const timeSinceLastCall = now - lastCallTime;
    const shouldCallLeading = leading && timeSinceLastCall >= limit;
    const shouldCallTrailing = trailing && timeSinceLastCall < limit;

    if (shouldCallLeading) {
      callFunction();
    } else if (shouldCallTrailing && !timeoutId) {
      timeoutId = setTimeout(() => {
        timeoutId = null;
        if (trailing) {
          callFunction();
        }
      }, limit - timeSinceLastCall);
    }
  };
}

/**
 * Request animation frame throttle - executes at most once per frame
 * @param {Function} func - Function to throttle
 * @returns {Function} Throttled function
 */
export function rafThrottle(func) {
  let frameId = null;
  let lastArgs = null;

  return function executedFunction(...args) {
    const context = this;
    lastArgs = args;

    if (frameId === null) {
      frameId = requestAnimationFrame(() => {
        frameId = null;
        func.apply(context, lastArgs);
      });
    }
  };
}

/**
 * Idle callback throttle - executes when browser is idle
 * @param {Function} func - Function to throttle
 * @param {Object} options - Idle options
 * @returns {Function} Throttled function
 */
export function idleThrottle(func, options = {}) {
  let idleId = null;
  let lastArgs = null;
  const { timeout = 50 } = options;

  return function executedFunction(...args) {
    const context = this;
    lastArgs = args;

    if (idleId === null) {
      const callback = () => {
        idleId = null;
        func.apply(context, lastArgs);
      };

      if (window.requestIdleCallback) {
        idleId = requestIdleCallback(callback, { timeout });
      } else {
        // Fallback for browsers without requestIdleCallback
        idleId = setTimeout(callback, 16); // ~60fps
      }
    }
  };
}

/**
 * Performance monitoring decorator
 * @param {Function} func - Function to monitor
 * @param {string} name - Performance marker name
 * @returns {Function} Monitored function
 */
export function withPerformanceMonitoring(func, name) {
  return function monitoredFunction(...args) {
    const startTime = performance.now();

    try {
      const result = func.apply(this, args);

      const endTime = performance.now();
      const duration = endTime - startTime;

      console.log(`⚡ [Performance] ${name}: ${duration.toFixed(2)}ms`);

      // Mark performance if function takes too long
      if (duration > 16) { // More than one frame at 60fps
        console.warn(`⚠️ [Performance] Slow function detected: ${name} (${duration.toFixed(2)}ms)`);
      }

      return result;
    } catch (error) {
      const endTime = performance.now();
      const duration = endTime - startTime;
      console.error(`❌ [Performance] ${name} failed after ${duration.toFixed(2)}ms:`, error);
      throw error;
    }
  };
}

/**
 * Batch function calls to reduce frequency
 * @param {Function} func - Function to batch
 * @param {number} batchSize - Number of calls to batch
 * @param {number} delay - Delay between batches
 * @returns {Function} Batched function
 */
export function batchCalls(func, batchSize = 10, delay = 16) {
  let batch = [];
  let timeoutId = null;

  const processBatch = () => {
    if (batch.length > 0) {
      const currentBatch = batch.slice();
      batch = [];

      currentBatch.forEach(({ context, args }) => {
        func.apply(context, args);
      });
    }
    timeoutId = null;
  };

  return function batchedFunction(...args) {
    const context = this;

    batch.push({ context, args });

    if (batch.length >= batchSize) {
      if (timeoutId) {
        clearTimeout(timeoutId);
      }
      processBatch();
    } else if (!timeoutId) {
      timeoutId = setTimeout(processBatch, delay);
    }
  };
}

/**
 * Create a function that caches results based on arguments
 * @param {Function} func - Function to memoize
 * @param {Function} keyResolver - Function to generate cache key
 * @param {number} maxCacheSize - Maximum cache size
 * @returns {Function} Memoized function
 */
export function memoize(func, keyResolver = (...args) => JSON.stringify(args), maxCacheSize = 100) {
  const cache = new Map();

  return function memoizedFunction(...args) {
    const key = keyResolver(...args);

    if (cache.has(key)) {
      return cache.get(key);
    }

    const result = func.apply(this, args);

    // Implement LRU cache behavior
    if (cache.size >= maxCacheSize) {
      const firstKey = cache.keys().next().value;
      cache.delete(firstKey);
    }

    cache.set(key, result);
    return result;
  };
}

/**
 * Performance profiler for measuring execution time
 */
export class PerformanceProfiler {
  constructor() {
    this.measurements = new Map();
  }

  start(label) {
    this.measurements.set(label, {
      startTime: performance.now(),
      endTime: null,
      duration: null
    });
  }

  end(label) {
    const measurement = this.measurements.get(label);
    if (measurement) {
      measurement.endTime = performance.now();
      measurement.duration = measurement.endTime - measurement.startTime;

      console.log(`⚡ [Profiler] ${label}: ${measurement.duration.toFixed(2)}ms`);

      return measurement.duration;
    }
    return null;
  }

  getResults() {
    const results = {};
    for (const [label, measurement] of this.measurements) {
      if (measurement.duration !== null) {
        results[label] = measurement.duration;
      }
    }
    return results;
  }

  clear() {
    this.measurements.clear();
  }
}

// Create default profiler instance
export const profiler = new PerformanceProfiler();

/**
 * Utility to check if browser supports certain performance features
 */
export const performanceSupport = {
  requestIdleCallback: typeof window !== 'undefined' && 'requestIdleCallback' in window,
  requestAnimationFrame: typeof window !== 'undefined' && 'requestAnimationFrame' in window,
  performanceObserver: typeof window !== 'undefined' && 'PerformanceObserver' in window,
  performanceNow: typeof performance !== 'undefined' && 'now' in performance
};

// Console debug helper
if (typeof window !== 'undefined') {
  window.fechatterPerformance = {
    debounce,
    throttle,
    rafThrottle,
    idleThrottle,
    profiler,
    performanceSupport
  };
} 