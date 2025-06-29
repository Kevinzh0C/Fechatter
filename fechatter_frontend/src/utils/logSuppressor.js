/**
 * Log Suppressor
 * Aggressive noise reduction for development environment
 * Targets specific noisy sources without breaking functionality
 */

class LogSuppressor {
  constructor() {
    this.suppressedPatterns = [
      // Development optimizer noise - MORE AGGRESSIVE
      /\[DEV_OPTIMIZER\]/,
      /Development mode optimizations/,
      /Development helpers available/,
      /\[DEV\]/,
      /\[AUTH\]/,
      /\[ROUTER\]/,
      /API Request \(via Gateway\)/,
      /API Response \(via Gateway\)/,
      /🔑 Added auth token/,
      /developmentOptimizer\.js:\d+/,  // Any line from developmentOptimizer.js
      /\[\d+:\d+:\d+ [AP]M\]/,  // Any timestamp pattern

      // Navigation spam
      /Channel navigation fixed/,
      /Logout fixed/,
      /Router fixed/,
      /All navigation fixes applied/,
      /Applying navigation fixes/,

      // Health check spam
      /Health monitoring/,
      /Health check/,
      /Health monitoring started/,

      // Test loading spam and auto-execution
      /test loaded/i,
      /available at window\./,
      /Use window\./,
      /loaded - use window\./,
      /loaded - use/,
      /🧪 Testing/,
      /🧪 Test available/,
      /Extension Pattern Test/,
      /🧪 Extension Conflict Fix Test/,
      /Error Source Preservation Test/,
      /🧪 Testing Pragmatic Error Suppressor/,
      /🧪 Testing Content Script Error Suppression/,
      /🧪 Testing Unified Error Handler/,
      /🔬 Transparent Error Handling Verification/,
      /Verifying All Error Suppression Fixes/,
      /🧪 Test Manager Initialization/,
      /📋 ChannelsList Analysis/,
      /SUBSCRIPTION: API Response Analysis/,
      /Duplicate Detection/,
      /🏠 Home\.vue Computed Properties/,
      /Data Flow Trace/,
      /Direct error from/,
      /Error from level\d+ function/,
      /Error \d+ - rapid/,
      /Test error with stack trace/,

      // Additional test script loading messages - ENHANCED
      /🧪.*Test loaded.*available for manual execution/,
      /🛡️.*Test loaded.*available for manual execution/,
      /🤝.*Test loaded.*available for manual execution/,
      /.*Test loaded.*available for manual execution/,
      /🔐.*Test loaded.*available for manual execution/,
      /loaded.*available for manual execution/,
      /Use window\.test/,
      /Use window\..*\(\) to test/,
      /Test.*loaded.*manual execution/,
      /available for manual execution/,

      // SSE connection spam
      /SSE.*loaded/,
      /Connection.*loaded/,

      // Authentication noise
      /\[AUTH\] Checking auth state/,
      /\[AUTH\] Restoring token/,
      /🔐 \[AUTH\] Initializing/,
      /Authentication initialized/,
      /Auth state/,

      // API noise - but keep real errors visible
      /API Request/,
      /API Response/,
      /🔑 Added auth token to request/,

      // Router noise
      /\[ROUTER\] Navigation/,
      /\[ROUTER\] Public route/,
      /\[ROUTER\] Navigation allowed/,

      // Channel analysis noise
      /📋 ChannelsList Analysis/,
      /Total channels:/,
      /No duplicates/,

      // Extension blocker noise
      /🛡️ Aggressive Extension Blocker/,
      /Extension Blocker initialized/,
      /Extension coordination test/,

      // Extension conflicts - these should be suppressed
      /A listener indicated an asynchronous response/,
      /message channel closed before a response/,
      /Extension context invalidated/,

      // Mark as read API errors - these are expected until backend implements endpoint
      /POST.*\/chat\/\d+\/read 404/,
      /Mark chat as read.*not_found/,
      /请求的资源未找到/,

      // Auth 401 errors - specific patterns for authentication failures
      /GET.*\/api\/users 401/,
      /GET.*\/api\/workspace\/chats 401/,
      /GET.*\/api\/chat\/\d+ 401/,
      /POST.*\/api\/logout 401/,
      /401 \(Unauthorized\)/,
      /Authentication failure/,
      /🚨 API Error \(via Gateway\).*401/,
      /🔐 \[AUTH\] Authentication failure/,
      /net::ERR_ABORTED 401/,
      /session has expired/i,
      /unauthorized/i,

      // Various utility loads
      /loaded - use window\./,
      /available for manual execution/,
      /Commands:/,
      /available:/,
      /loaded\./,

      // Token management noise
      /⏰ Next token refresh/,
      /🔐 Tokens updated/,
      /expiresIn:/,

      // ENHANCED: Token activity monitoring spam
      /⏰ User inactive for too long/,
      /User activity timeout/,
      /Token activity check/,
      /Activity manager/,
      /Inactivity detected/,

      // Debug tools auto-execution spam
      /📋 ChannelsList Analysis/,
      /SUBSCRIPTION: API Response Analysis/,
      /Duplicate Detection/,
      /🏠 Home\.vue Computed Properties/,
      /Data Flow Trace/,
      /Starting automatic duplicate channels analysis/,
      /Duplicate Channels Debug Tool loaded/,

      // Home layout noise
      /🏠 \[HOME\]/,
      /Starting home layout/,
      /Authentication verified/,

      // Chat noise
      /📍 \[ChatStore\]/,
      /🔄 \[Chat\.vue\]/,
      /📥 \[FETCH_MESSAGES\]/,
      /Loading workspace users/,

      // Test automation noise
      /🧪 Test available/,
      /Test loaded/,
      /available for manual execution/,
      /Run.*test/,

      // Configuration noise
      /🎛️ Fechatter Configuration Initialized/,
      /Gateway URL:/,
      /Debug Mode:/,
      /Environment:/,

      // Handler registration noise
      /\[UnifiedErrorHandler\] Registered handler/,
      /Registered with unified/,

      // Validator noise
      /🛡️ Strict Channel Message Validator loaded/,
      /Usage:/,
      /window\.channelMessageValidator/,

      // Token manager noise
      /🔐 Token Manager initialized/,

      // API server noise
      /SUBSCRIPTION: API Server \(via Gateway\)/,
      /🔌 SSE Server \(via Gateway\)/,

      // Error suppressor noise
      /🔇 Suppressed Content Script Errors/,

      // Log suppressor self-noise
      /Added suppression pattern/,
      /🔇 Log suppressor activated/,

      // Duplicate loading messages
      /loaded - use/,
      /available:/,
      /loaded\./
    ];

    this.isActive = false;
    this.originalConsole = null;
    this.logCount = 0;
    this.suppressedCount = 0;
  }

  /**
   * Check if message should be suppressed
   */
  shouldSuppress(message) {
    const messageStr = typeof message === 'string' ? message : String(message);

    // Check against all patterns
    for (const pattern of this.suppressedPatterns) {
      if (pattern.test(messageStr)) {
        this.suppressedCount++;
        return true;
      }

    return false;
  }

  /**
   * Activate suppression
   */
  activate() {
    if (this.isActive) return;

    // Store original console methods
    this.originalConsole = {
      log: console.log.bind(console),
      info: console.info.bind(console),
      warn: console.warn.bind(console),
      error: console.error.bind(console),
      debug: console.debug.bind(console)
    };

    // Override console methods
    console.log = (...args) => {
      this.logCount++;
      if (!this.shouldSuppress(args[0])) {
        this.originalConsole.log(...args);
      }
    };

    console.info = (...args) => {
      this.logCount++;
      if (!this.shouldSuppress(args[0])) {
        this.originalConsole.info(...args);
      }
    };

    console.warn = (...args) => {
      this.logCount++;
      // Always show warnings
      this.originalConsole.warn(...args);
    };

    console.error = (...args) => {
      this.logCount++;
      // Always show errors
      this.originalConsole.error(...args);
    };

    console.debug = (...args) => {
      this.logCount++;
      if (!this.shouldSuppress(args[0])) {
        this.originalConsole.debug(...args);
      }
    };

    this.isActive = true;
    // Make activation message silent
    // this.originalConsole.log('🔇 Log suppressor activated - noise reduction active');
  }

  /**
   * Deactivate suppression
   */
  deactivate() {
    if (!this.isActive || !this.originalConsole) return;

    // Restore original console methods
    console.log = this.originalConsole.log;
    console.info = this.originalConsole.info;
    console.warn = this.originalConsole.warn;
    console.error = this.originalConsole.error;
    console.debug = this.originalConsole.debug;

    this.isActive = false;
    if (import.meta.env.DEV) {
      console.log('🔊 Log suppressor deactivated - full logging restored');
    }

  /**
   * Get suppression statistics
   */
  getStats() {
    return {
      isActive: this.isActive,
      totalLogs: this.logCount,
      suppressed: this.suppressedCount,
      shown: this.logCount - this.suppressedCount,
      suppressionRate: this.logCount > 0 ? (this.suppressedCount / this.logCount * 100).toFixed(1) + '%' : '0%'
    };
  }

  /**
   * Toggle suppression
   */
  toggle() {
    if (this.isActive) {
      this.deactivate();
    } else {
      this.activate();
    }

  /**
   * Reset counters
   */
  reset() {
    this.logCount = 0;
    this.suppressedCount = 0;
    if (import.meta.env.DEV) {
      console.log('Log suppressor counters reset');
    }

  /**
   * Add custom pattern
   */
  addPattern(pattern) {
    this.suppressedPatterns.push(pattern);
    // Remove the console.log to make suppressor itself silent
    // console.log('Added suppression pattern:', pattern);
  }

  /**
   * Show suppressed patterns
   */
  showPatterns() {
    console.group('🔇 Log Suppression Patterns');
    this.suppressedPatterns.forEach((pattern, index) => {
      if (import.meta.env.DEV) {
        console.log(`${index + 1}. ${pattern}`);
      }
    });
    console.groupEnd();
  }

// Create singleton instance
const logSuppressor = new LogSuppressor();

// Auto-activate in development
if (import.meta.env.DEV) {
  // Activate immediately instead of waiting
  logSuppressor.activate();

  // Add extra aggressive patterns for developmentOptimizer noise
  logSuppressor.addPattern(/developmentOptimizer\.js/);
  logSuppressor.addPattern(/\[5:\d{2}:\d{2} PM\]/); // Timestamp pattern
  logSuppressor.addPattern(/\[.*PM\]/); // Any PM timestamp
  logSuppressor.addPattern(/Total channels:/);
  logSuppressor.addPattern(/No duplicates/);
  logSuppressor.addPattern(/ChannelsList Analysis/);
  logSuppressor.addPattern(/Authentication initialized/);
  logSuppressor.addPattern(/Health monitoring/);
  logSuppressor.addPattern(/Next token refresh/);
  logSuppressor.addPattern(/Tokens updated/);
}

// Export for global use
if (typeof window !== 'undefined') {
  window.logSuppressor = logSuppressor;

  // Convenient global functions
  window.suppressLogs = () => logSuppressor.activate();
  window.allowLogs = () => logSuppressor.deactivate();
  window.toggleLogs = () => logSuppressor.toggle();
  window.logStats = () => {
    const stats = logSuppressor.getStats();
    console.table(stats);
    return stats;
  };

  // Make initial load message silent in production mode
  // console.log('🔇 Log Suppressor loaded - use window.suppressLogs(), window.allowLogs(), window.toggleLogs()');
}

export default logSuppressor; 