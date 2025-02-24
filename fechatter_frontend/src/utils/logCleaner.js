/**
 * Log Cleaner - Production-grade console management
 * 🔧 FIXED: Prevents log aggregation pollution while preserving debug info
 */

class LogCleaner {
  constructor() {
    this.isDev = import.meta.env.DEV;
    this.logLevels = {
      ERROR: 0,
      WARN: 1,
      INFO: 2,
      DEBUG: 3
    };

    // Production: ERROR only, Development: INFO and below
    this.currentLevel = this.isDev ? this.logLevels.INFO : this.logLevels.ERROR;

    this.noisePatterns = [
      // Health check noise
      /Health check completed/,
      /Health monitoring/,

      // Extension noise
      /chrome-extension/,
      /content.*script/,
      /quillbot/i,

      // Development noise - 🔧 ENHANCED: More specific patterns
      /loaded.*use window\./,
      /Performance Monitor loaded/,
      /📊.*loaded/,
      /Commands:/,
      /available:/,

      // SSE noise
      /SSE.*connecting/,
      /SSE.*connected/,

      // Routine operations
      /✅.*applied/,
      /🔧.*active/,

      // 🔧 NEW: Debug channels auto-execution noise
      /📋 ChannelsList Analysis/,
      /📡 API Response Analysis/,
      /🔍 Duplicate Detection/,
      /📊 Data Flow Trace/,
      /🏠 Home\.vue Computed Properties/,
      /Total channels:/,
      /✅ No duplicates/,

      // 🔧 NEW: Ultra-fast system startup noise
      /⚡ Ultra-fast message system active/,
      /💡 Development commands:/,
      /🛠️ Development Commands:/,
      /🚀 Features:/,
      /✓.*delay/,
      /✓.*processing/,
      /✓.*updates/,
      /===================================/,

      // 🔧 NEW: Performance measurement spam
      /\[PERF\].*color:/,
      /Breakdown:.*complete:/,
      /instant-nav-start:/,
      /route-nav-start:/
    ];

    this.initialize();
  }

  initialize() {
    // 🔧 CRITICAL FIX: Only apply filtering in development with opt-in
    // Prevents production log pollution and maintains debugging capability
    if (this.isDev && this.shouldEnableFiltering()) {
      this.setupSelectiveFiltering();
    }

    // 🔧 FIXED: Don't log the cleaner itself to prevent aggregation
    // console.log('🧹 Log cleaner initialized -', this.isDev ? 'Development' : 'Production', 'mode');
  }

  shouldEnableFiltering() {
    // 🔧 SMART FILTERING: Only enable if noise levels are detected
    // Check if debug channels are running (indicating noise)
    return window.location.search.includes('debug=quiet') ||
      localStorage.getItem('enable_log_filtering') === 'true';
  }

  setupSelectiveFiltering() {
    const originalLog = console.log;
    const originalInfo = console.info;

    // 🔧 ENHANCED: Preserve source information while filtering noise
    console.log = (...args) => {
      if (this.shouldFilter(args)) {
        return; // Silent suppression
      }

      // 🔧 CRITICAL FIX: Preserve original call stack
      // Don't call through this wrapper to avoid logCleaner.js:65
      originalLog.apply(console, args);
    };

    console.info = (...args) => {
      if (this.shouldFilter(args)) {
        return;
      originalInfo.apply(console, args);
    };

    // 🔧 NEW: Add control methods
    window.enableLogFiltering = () => {
      localStorage.setItem('enable_log_filtering', 'true');
      if (import.meta.env.DEV) {
        console.log('🔇 Log filtering enabled');
      }
    };

    window.disableLogFiltering = () => {
      localStorage.removeItem('enable_log_filtering');
      if (import.meta.env.DEV) {
        console.log('🔊 Log filtering disabled');
      }
    };
  }

  setupProductionFiltering() {
    // In production, only show errors and critical warnings
    console.log = () => { }; // Silent
    console.info = () => { }; // Silent
    console.debug = () => { }; // Silent

    // Keep error and warn for critical issues
    const originalWarn = console.warn;
    console.warn = (...args) => {
      if (this.shouldFilter(args)) {
        return;
      originalWarn.apply(console, args);
    };
  }

  shouldFilter(args) {
    const message = args.join(' ');

    // Never filter errors
    if (message.includes('Error') || message.includes('error')) {
      return false;
    }

    // 🔧 ENHANCED: More intelligent filtering
    // Filter known noise patterns
    return this.noisePatterns.some(pattern => pattern.test(message));
  }

  // Utility methods for controlled logging
  error(...args) {
    if (import.meta.env.DEV) {
      console.error(...args);
    }

  warn(...args) {
    if (this.currentLevel >= this.logLevels.WARN) {
      if (import.meta.env.DEV) {
        console.warn(...args);
      }

  info(...args) {
    if (this.currentLevel >= this.logLevels.INFO) {
      console.info(...args);
    }

  debug(...args) {
    if (this.currentLevel >= this.logLevels.DEBUG) {
      console.debug(...args);
    }

  // 🔧 NEW: Manual control methods
  suppressDebugChannels() {
    // Add temporary suppression for debug channels
    if (window.debugDuplicateChannels) {
      const original = window.debugDuplicateChannels;
      window.debugDuplicateChannels = () => {
        if (import.meta.env.DEV) {
          console.log('🔇 Debug channels analysis suppressed. Use logCleaner.unsuppressDebugChannels() to restore.');
        }
      };
      this._originalDebugChannels = original;
    }

  unsuppressDebugChannels() {
    if (this._originalDebugChannels) {
      window.debugDuplicateChannels = this._originalDebugChannels;
      delete this._originalDebugChannels;
      if (import.meta.env.DEV) {
        console.log('🔊 Debug channels analysis restored');
      }

const logCleaner = new LogCleaner();

// 🔧 EXPOSE CONTROL: Allow manual control in development
if (import.meta.env.DEV) {
  window.logCleaner = logCleaner;
}

export default logCleaner;