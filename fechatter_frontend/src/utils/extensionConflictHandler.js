/**
 * Extension Conflict Handler
 * Detects and handles conflicts with browser extensions
 * Production-grade implementation with proactive detection
 */

import requestIsolation from './requestIsolation';

class ExtensionConflictHandler {
  constructor() {
    this.conflictPatterns = [
      // Password managers
      { pattern: /password|1password|lastpass|bitwarden|dashlane/i, type: 'password_manager' },
      // Ad blockers
      { pattern: /adblock|ublock|adguard|privacy badger/i, type: 'ad_blocker' },
      // VPN/Proxy extensions
      { pattern: /vpn|proxy|tunnel|anonymizer/i, type: 'vpn_proxy' },
      // Developer tools
      { pattern: /react devtools|vue devtools|redux/i, type: 'dev_tools' },
      // Other security extensions
      { pattern: /noscript|https everywhere|privacy/i, type: 'security' }
    ];

    this.detectedConflicts = new Set();
    this.errorCounts = new Map();
    this.notificationShown = false;

    // Known extension-specific issues
    this.knownIssues = {
      'password_manager': {
        message: 'Password manager extensions may interfere with form submissions',
        solution: 'Try disabling autofill for this site'
      },
      'ad_blocker': {
        message: 'Ad blocker may block legitimate API requests',
        solution: 'Add this site to your ad blocker whitelist'
      },
      'vpn_proxy': {
        message: 'VPN/Proxy extensions may cause connection issues',
        solution: 'Try disabling the VPN temporarily'
      }
    };
  }

  /**
   * Initialize conflict handler
   */
  initialize() {
    // Listen for specific error patterns
    this.setupErrorInterceptor();

    // Proactive extension detection
    this.detectInstalledExtensions();

    // Monitor for new extensions
    this.monitorExtensionChanges();

    // Override console methods
    this.overrideConsoleMethods();
  }

  /**
   * Setup error interceptor
   */
  setupErrorInterceptor() {
    // Only listen for unhandled promise rejections
    // Console error handling is done via unified handler
    window.addEventListener('unhandledrejection', (event) => {
      const error = event.reason;
      const errorString = error?.message || error?.toString() || '';

      // Skip if this is a content script error
      const contentScriptSuppressor = window.contentScriptSuppressor;
      if (contentScriptSuppressor && contentScriptSuppressor.isContentScriptError(errorString)) {
        return;
      }

      if (error && this.isExtensionConflict(errorString)) {
        this.handleConflict(errorString);
      }
    });

  /**
   * Detect installed extensions proactively
   */
  async detectInstalledExtensions() {
    // Check for common extension indicators
    const extensionChecks = [
      // Password managers
      { check: () => document.querySelector('[data-lastpass-icon-root]'), type: 'lastpass' },
      { check: () => window.__REACT_DEVTOOLS_GLOBAL_HOOK__, type: 'react_devtools' },
      { check: () => window.__VUE_DEVTOOLS_GLOBAL_HOOK__, type: 'vue_devtools' },
      // Ad blockers (check for blocked resources) - 更优雅的检测方式
      {
        check: async () => {
          try {
            // 创建一个隐藏的测试元素而不是实际请求
            const testDiv = document.createElement('div');
            testDiv.className = 'adsbox ad-test';
            testDiv.style.cssText = 'position:absolute;left:-9999px;width:1px;height:1px;';
            document.body.appendChild(testDiv);

            // 检查元素是否被隐藏（广告拦截器通常会隐藏这些类名）
            const isHidden = window.getComputedStyle(testDiv).display === 'none' ||
              window.getComputedStyle(testDiv).visibility === 'hidden' ||
              testDiv.offsetHeight === 0;

            document.body.removeChild(testDiv);
            return isHidden;
          } catch {
            return false; // 检测失败，假设没有广告拦截器
          }
        },
        type: 'ad_blocker'
      }
    ];

    for (const { check, type } of extensionChecks) {
      try {
        const result = await check();
        if (result) {
          this.detectedConflicts.add(type);
          if (import.meta.env.DEV) {
            console.log(`[ExtensionConflict] Detected: ${type}`);
          }
      } catch {
        // Ignore check failures
      }

  /**
   * Monitor for extension changes
   */
  monitorExtensionChanges() {
    // Use MutationObserver to detect DOM changes by extensions
    const observer = new MutationObserver((mutations) => {
      mutations.forEach((mutation) => {
        if (mutation.addedNodes.length) {
          mutation.addedNodes.forEach((node) => {
            if (node.nodeType === 1) { // Element node
              const nodeString = node.outerHTML || '';
              this.conflictPatterns.forEach(({ pattern, type }) => {
                if (pattern.test(nodeString)) {
                  this.detectedConflicts.add(type);
                }
              });
          });
      });
    });

    // Start observing
    observer.observe(document.body, {
      childList: true,
      subtree: true,
      attributes: false,
      characterData: false
    });

    // Store observer for cleanup
    this.observer = observer;
  }

  /**
   * Check if error is extension-related
   */
  isExtensionConflict(errorMessage) {
    // First check if this is a content script error (handled by ContentScriptErrorSuppressor)
    if (errorMessage.includes('content script.js') ||
      errorMessage.includes('fetchError: Failed to fetch')) {
      return false; // Let ContentScriptErrorSuppressor handle these
    }

    const conflictIndicators = [
      /listener .* indicated an asynchronous response/i,
      /message channel closed/i,
      /Extension context invalidated/i,
      // Removed generic /Failed to fetch/i - too broad
      // Only match extension URLs directly
      /chrome-extension:\/\/[^\/]+\/(?!content script)/i,  // Extension URL but not content script
      /moz-extension:\/\/[^\/]+\/(?!content script)/i
    ];

    return conflictIndicators.some(pattern => pattern.test(errorMessage));
  }

  /**
   * Handle detected conflict
   */
  handleConflict(errorMessage) {
    // Track error frequency
    const count = (this.errorCounts.get(errorMessage) || 0) + 1;
    this.errorCounts.set(errorMessage, count);

    // Log conflict
    if (import.meta.env.DEV) {
      console.warn('[ExtensionConflict] Detected conflict:', {
        message: errorMessage,
      count,
      detectedExtensions: Array.from(this.detectedConflicts)
    });

    // Show notification after threshold
    if (count >= 3 && !this.notificationShown) {
      this.showConflictNotification();
    }

    // Notify request isolation layer
    if (requestIsolation) {
      // Request isolation will handle retries
    }

  /**
   * Show conflict notification to user
   */
  showConflictNotification() {
    if (this.notificationShown) return;
    this.notificationShown = true;

    const detectedTypes = Array.from(this.detectedConflicts);
    let message = 'Browser extensions may be interfering with the application. ';
    let solution = 'Try disabling extensions or using incognito/private mode.';

    // Provide specific guidance based on detected extensions
    if (detectedTypes.length > 0) {
      const issue = this.knownIssues[detectedTypes[0]];
      if (issue) {
        message = issue.message + '. ';
        solution = issue.solution;
      }

    // Use error handler if available
    if (window.errorHandler?.showNotification) {
      window.errorHandler.showNotification('warning', message + solution, {
        duration: 10000,
        action: {
          text: 'Learn More',
          handler: () => this.showConflictReport()
        }
      });
    } else {
      if (import.meta.env.DEV) {
        console.warn('[ExtensionConflict]', message + solution);
      }

  /**
   * Show detailed conflict report
   */
  showConflictReport() {
    const report = {
      detectedExtensions: Array.from(this.detectedConflicts),
      errorCounts: Object.fromEntries(this.errorCounts),
      recommendations: this.getRecommendations()
    };

    console.group('🔍 Extension Conflict Report');
    if (import.meta.env.DEV) {
      console.log('Detected Extensions:', report.detectedExtensions);
    if (import.meta.env.DEV) {
      console.log('Error Counts:', report.errorCounts);
    if (import.meta.env.DEV) {
      console.log('Recommendations:', report.recommendations);
    console.groupEnd();

    return report;
  }

  /**
   * Get recommendations based on detected conflicts
   */
  getRecommendations() {
    const recommendations = [];

    if (this.detectedConflicts.has('password_manager')) {
      recommendations.push({
        type: 'Password Manager',
        action: 'Disable autofill for this site in your password manager settings'
      });

    if (this.detectedConflicts.has('ad_blocker')) {
      recommendations.push({
        type: 'Ad Blocker',
        action: 'Add this site to your ad blocker\'s whitelist'
      });

    recommendations.push({
      type: 'General',
      action: 'Use incognito/private browsing mode which disables most extensions'
    });

    return recommendations;
  }

  /**
   * Cleanup resources
   */
  cleanup() {
    if (this.observer) {
      this.observer.disconnect();
    this.detectedConflicts.clear();
    this.errorCounts.clear();
  }

  /**
   * Override console methods to catch extension errors early
   */
  overrideConsoleMethods() {
    // Register with unified handler instead of direct override
    if (window.unifiedErrorHandler) {
      window.unifiedErrorHandler.registerHandler(
        'extensionConflictHandler',
        ({ args, errorString }) => {
          // Check if this is an extension conflict
          if (this.isExtensionConflict(errorString)) {
            this.handleConflict(errorString);
            // Don't suppress - let other handlers decide
            return 'pass';
          return 'pass';
        },
        30 // Higher priority than content script suppressor
      );

      if (import.meta.env.DEV) {
        console.log('[ExtensionConflictHandler] Registered with unified handler');
      }

// Export singleton instance
const extensionConflictHandler = new ExtensionConflictHandler();

// Auto-initialize if in browser
if (typeof window !== 'undefined') {
  window.extensionConflictHandler = extensionConflictHandler;
}

export default extensionConflictHandler; 