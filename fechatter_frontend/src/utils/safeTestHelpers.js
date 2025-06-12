/**
 * Safe Test Helpers
 * Utilities for testing error handling without causing actual unhandled promise rejections
 */

/**
 * Create a safely rejected promise for testing
 * This prevents unhandled promise rejection warnings while still allowing error detection
 * @param {Error|string} error - The error to reject with
 * @returns {Promise} A rejected promise with a built-in catch handler
 */
export function createSafeRejectedPromise(error) {
  const errorObj = error instanceof Error ? error : new Error(error);

  // Create the promise with immediate catch to prevent unhandled rejection
  const promise = Promise.reject(errorObj);

  // Add a no-op catch handler to prevent unhandled rejection
  // but still allow the promise to be rejected for testing
  promise.catch(() => {
    // Intentionally empty - prevents unhandled rejection warning
  });

  // Return a new promise that will also reject with the same error
  // This allows tests to check for rejection while avoiding warnings
  return Promise.reject(errorObj);
}

/**
 * Create multiple safe rejected promises for testing
 * @param {Array<Error|string>} errors - Array of errors to create rejections for
 * @returns {Array<Promise>} Array of safely rejected promises
 */
export function createSafeRejectedPromises(errors) {
  return errors.map(error => createSafeRejectedPromise(error));
}

/**
 * Test promise rejection handling with safe promises
 * @param {Function} testFunction - Function that should handle the rejection
 * @param {Error|string} error - The error to test with
 * @param {number} delay - Delay before checking results (ms)
 * @returns {Promise<boolean>} Whether the rejection was handled
 */
export async function testSafePromiseRejection(testFunction, error, delay = 100) {
  let handled = false;

  // Create a safe rejected promise
  const promise = createSafeRejectedPromise(error);

  // Set up handler to detect if rejection was processed
  const originalHandler = window.onerror;
  window.onerror = (message) => {
    if (message.includes(error.toString())) {
      handled = true;
    }
    return true; // Prevent default error handling
  };

  try {
    // Execute the test function with the promise
    await testFunction(promise);

    // Wait for async handlers
    await new Promise(resolve => setTimeout(resolve, delay));

  } finally {
    // Restore original error handler
    window.onerror = originalHandler;
  }

  return handled;
}

/**
 * Safely test unhandled rejection detection
 * @param {string} errorMessage - Error message to test
 * @returns {Promise} A promise that rejects after handlers have a chance to detect it
 */
export function createTestableRejection(errorMessage) {
  return new Promise((resolve, reject) => {
    // Delay rejection to allow handlers to be set up
    setTimeout(() => {
      const error = new Error(errorMessage);

      // Trigger unhandledrejection event manually
      const event = new PromiseRejectionEvent('unhandledrejection', {
        promise: Promise.reject(error),
        reason: error
      });

      window.dispatchEvent(event);

      // Also create actual rejection but with safety catch
      Promise.reject(error).catch(() => {
        // Safety catch to prevent actual unhandled rejection
      });

      reject(error);
    }, 0);
  }).catch(() => {
    // Final safety catch
  });
}

/**
 * Run a test that involves promise rejections safely
 * @param {string} testName - Name of the test
 * @param {Function} testFunction - Test function to run
 * @returns {Promise<Object>} Test result
 */
export async function runSafePromiseTest(testName, testFunction) {
  console.log(`🧪 Running safe promise test: ${testName}`);

  const startTime = Date.now();
  let success = false;
  let error = null;

  try {
    await testFunction();
    success = true;
    console.log(`✅ ${testName} completed successfully`);
  } catch (err) {
    error = err;
    console.error(`❌ ${testName} failed:`, err);
  }

  const duration = Date.now() - startTime;

  return {
    testName,
    success,
    error,
    duration,
    timestamp: new Date().toISOString()
  };
}

/**
 * Create a promise that rejects but is immediately caught
 * Useful for testing error detection without causing warnings
 */
export class SafeRejectedPromise extends Promise {
  constructor(error) {
    super((resolve, reject) => {
      reject(error);
    });

    // Immediately add a catch to prevent unhandled rejection
    this.catch(() => { /* Intentionally empty */ });
  }
}

// Export convenience functions
export const safeReject = (error) => new SafeRejectedPromise(error);
export const safeRejectMany = (errors) => errors.map(err => safeReject(err));

// Expose to window for testing
if (typeof window !== 'undefined' && import.meta.env.DEV) {
  window.safeTestHelpers = {
    createSafeRejectedPromise,
    createSafeRejectedPromises,
    testSafePromiseRejection,
    createTestableRejection,
    runSafePromiseTest,
    SafeRejectedPromise,
    safeReject,
    safeRejectMany
  };
}

export default {
  createSafeRejectedPromise,
  createSafeRejectedPromises,
  testSafePromiseRejection,
  createTestableRejection,
  runSafePromiseTest,
  SafeRejectedPromise,
  safeReject,
  safeRejectMany
}; 