/**
 * Developer Console Helpers
 * 
 * Quick utilities for managing the cleaned-up logging system during development
 * Access via window.devHelpers in browser console
 */

import { productionLogManager } from './productionLogManager.js';

class DevConsoleHelpers {
  constructor() {
    this.logManager = productionLogManager;
  }

  /**
   * Quick log level presets
   */
  quiet() {
    this.logManager.setLevel(this.logManager.logLevels.ERROR);
    console.log('🔇 Quiet mode: Only errors will be shown');
    return this;
  }

  normal() {
    this.logManager.setLevel(this.logManager.logLevels.INFO);
    console.log('📢 Normal mode: Info level and above');
    return this;
  }

  verbose() {
    this.logManager.setLevel(this.logManager.logLevels.VERBOSE);
    console.log('📣 Verbose mode: All logs will be shown');
    return this;
  }

  /**
   * Category management shortcuts
   */
  hideMessageLogs() {
    this.logManager.disableCategory('MessageDisplayGuarantee');
    this.logManager.disableCategory('SimpleMessageList');
    console.log('🙈 Message-related logs hidden');
    return this;
  }

  showMessageLogs() {
    this.logManager.enableCategory('MessageDisplayGuarantee');
    this.logManager.enableCategory('SimpleMessageList');
    console.log('👁️ Message-related logs enabled');
    return this;
  }

  hideNavigationLogs() {
    this.logManager.disableCategory('NavigationManager');
    this.logManager.disableCategory('Router');
    console.log('🙈 Navigation logs hidden');
    return this;
  }

  showNavigationLogs() {
    this.logManager.enableCategory('NavigationManager');
    this.logManager.enableCategory('Router');
    console.log('👁️ Navigation logs enabled');
    return this;
  }

  /**
   * Quick debugging presets
   */
  debugCleanConsole() {
    console.clear();
    this.logManager.setLevel(this.logManager.logLevels.INFO);
    this.hideMessageLogs();
    console.log('🧹 Clean console mode: Core functionality only');
    return this;
  }

  debugMessageSystem() {
    console.clear();
    this.logManager.setLevel(this.logManager.logLevels.DEBUG);
    this.showMessageLogs();
    this.logManager.disableCategory('Router');
    this.logManager.disableCategory('NavigationManager');
    console.log('🛡️ Message system debug mode');
    return this;
  }

  debugPerformance() {
    console.clear();
    this.logManager.setLevel(this.logManager.logLevels.WARN);
    console.log('⚡ Performance mode: Warnings and errors only');
    return this;
  }

  /**
   * Show current status
   */
  status() {
    const stats = this.logManager.getStats();
    console.group('📊 Current Logging Status');
    console.log('Log Level:', stats.currentLevel);
    console.log('Suppressed Categories:', stats.suppressedCategories);
    console.log('Batched Logs Count:', stats.batchedLogsCount);
    console.log('Total Categories:', stats.totalCategories);
    console.log('Most Frequent Logs:', stats.mostFrequentLogs);
    console.groupEnd();
    return this;
  }

  /**
   * Reset to defaults
   */
  reset() {
    this.logManager.setLevel(this.logManager.logLevels.DEBUG);
    this.logManager.suppressedLogs.clear();
    console.log('🔄 Reset to default logging state');
    return this;
  }

  /**
   * Show help
   */
  help() {
    console.group('💡 Dev Console Helper Commands');
    console.log('Quick Presets:');
    console.log('  devHelpers.quiet()        - Only show errors');
    console.log('  devHelpers.normal()       - Show info and above');
    console.log('  devHelpers.verbose()      - Show all logs');
    console.log('');
    console.log('Category Controls:');
    console.log('  devHelpers.hideMessageLogs()    - Hide message system logs');
    console.log('  devHelpers.showMessageLogs()    - Show message system logs');
    console.log('  devHelpers.hideNavigationLogs() - Hide navigation logs');
    console.log('  devHelpers.showNavigationLogs() - Show navigation logs');
    console.log('');
    console.log('Debug Presets:');
    console.log('  devHelpers.debugCleanConsole()  - Clean console mode');
    console.log('  devHelpers.debugMessageSystem() - Focus on message system');
    console.log('  devHelpers.debugPerformance()   - Performance mode');
    console.log('');
    console.log('Utilities:');
    console.log('  devHelpers.status()      - Show current status');
    console.log('  devHelpers.reset()       - Reset to defaults');
    console.log('  devHelpers.help()        - Show this help');
    console.groupEnd();
    return this;
  }
}

export const devConsoleHelpers = new DevConsoleHelpers();

// Make available globally in development
if (import.meta.env.DEV) {
  window.devHelpers = devConsoleHelpers;

  // Auto-show help on first load
  setTimeout(() => {
    console.log('💡 Use devHelpers.help() to see available console commands for managing logs');
  }, 1000);
} 