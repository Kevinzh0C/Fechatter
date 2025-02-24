/**
 * Message Display Diagnostics
 * 🔧 Production-grade debugging tools for message display issues
 */

class MessageDisplayDiagnostics {
  constructor() {
    this.diagnosticsHistory = [];
    this.isActive = false;
  }

  /**
   * Run comprehensive diagnostics on current chat
   */
  async runDiagnostics(chatId) {
    const startTime = Date.now();
    const result = {
      chatId,
      timestamp: startTime,
      checks: {},
      summary: {},
      recommendations: []
    };

    console.log(`🔬 [Diagnostics] Starting message display analysis for chat ${chatId}`);

    try {
      // Check 1: DOM Elements
      result.checks.domElements = this.checkDOMElements(chatId);

      // Check 2: MessageDisplayGuarantee State
      result.checks.guaranteeState = this.checkGuaranteeState(chatId);

      // Check 3: Viewport Analysis
      result.checks.viewport = this.checkViewportState();

      // Check 4: Message Tracking
      result.checks.tracking = this.checkMessageTracking(chatId);

      // Check 5: Performance Metrics
      result.checks.performance = this.checkPerformanceMetrics();

      // Generate summary and recommendations
      result.summary = this.generateSummary(result.checks);
      result.recommendations = this.generateRecommendations(result.checks);

      const duration = Date.now() - startTime;
      result.duration = duration;

      console.log(`✅ [Diagnostics] Analysis completed in ${duration}ms`);
      console.table(result.summary);

      if (result.recommendations.length > 0) {
        console.log(`💡 [Diagnostics] Recommendations:`);
        result.recommendations.forEach((rec, idx) => {
          console.log(`  ${idx + 1}. ${rec}`);
        });
      }

      this.diagnosticsHistory.push(result);
      return result;

    } catch (error) {
      console.error(`❌ [Diagnostics] Error during analysis:`, error);
      result.error = error.message;
      return result;
    }
  }

  /**
   * Check DOM elements for message display
   */
  checkDOMElements(chatId) {
    const messageContainer = document.querySelector('[data-test-id="message-list"]');
    const messageElements = document.querySelectorAll('[data-message-id]');

    const visibleMessages = Array.from(messageElements).filter(el => {
      const rect = el.getBoundingClientRect();
      return rect.height > 0 && rect.width > 0;
    });

    const messagesInViewport = Array.from(messageElements).filter(el => {
      const rect = el.getBoundingClientRect();
      const container = messageContainer?.getBoundingClientRect();

      if (!container) return false;

      return rect.bottom >= container.top &&
        rect.top <= container.bottom &&
        rect.right >= container.left &&
        rect.left <= container.right;
    });

    return {
      totalElements: messageElements.length,
      visibleElements: visibleMessages.length,
      inViewport: messagesInViewport.length,
      containerExists: !!messageContainer,
      containerHeight: messageContainer?.scrollHeight || 0,
      scrollPosition: messageContainer?.scrollTop || 0
    };
  }

  /**
   * Check MessageDisplayGuarantee state
   */
  checkGuaranteeState(chatId) {
    if (typeof window.messageDisplayGuarantee === 'undefined') {
      return { error: 'MessageDisplayGuarantee not available' };
    }

    const guarantee = window.messageDisplayGuarantee;
    const contexts = Array.from(guarantee.verificationQueue.values());
    const currentContext = contexts.find(ctx => ctx.chatId === chatId);

    return {
      hasContext: !!currentContext,
      totalContexts: contexts.length,
      currentTracking: currentContext ? {
        expectedMessages: currentContext.messageIds.size,
        displayedMessages: currentContext.displayedIds.size,
        retryAttempts: currentContext.retryAttempts,
        fetchTimestamp: currentContext.fetchTimestamp,
        timeElapsed: currentContext.fetchTimestamp ? Date.now() - currentContext.fetchTimestamp : 0
      } : null
    };
  }

  /**
   * Check viewport and scroll state
   */
  checkViewportState() {
    const container = document.querySelector('[data-test-id="message-list"]');

    if (!container) {
      return { error: 'Message container not found' };
    }

    const isAtBottom = Math.abs(
      (container.scrollTop + container.clientHeight) - container.scrollHeight
    ) < 10;

    return {
      scrollTop: container.scrollTop,
      scrollHeight: container.scrollHeight,
      clientHeight: container.clientHeight,
      isAtBottom,
      scrollPercentage: Math.round((container.scrollTop / (container.scrollHeight - container.clientHeight)) * 100)
    };
  }

  /**
   * Check message tracking effectiveness
   */
  checkMessageTracking(chatId) {
    const messageElements = document.querySelectorAll('[data-message-id]');
    const messageIds = Array.from(messageElements).map(el => el.dataset.messageId);

    const trackingStats = {
      totalMessages: messageIds.length,
      duplicateIds: messageIds.length - new Set(messageIds).size,
      missingDataAttributes: Array.from(messageElements).filter(el => !el.dataset.messageId).length
    };

    return trackingStats;
  }

  /**
   * Check performance metrics
   */
  checkPerformanceMetrics() {
    const perf = window.performance;
    const navigation = perf.getEntriesByType('navigation')[0];

    return {
      loadTime: navigation ? Math.round(navigation.loadEventEnd - navigation.loadEventStart) : 0,
      domInteractive: navigation ? Math.round(navigation.domInteractive - navigation.fetchStart) : 0,
      memoryUsage: (window.performance.memory ? {
        used: Math.round(window.performance.memory.usedJSHeapSize / 1024 / 1024),
        total: Math.round(window.performance.memory.totalJSHeapSize / 1024 / 1024)
      } : null)
    };
  }

  /**
   * Generate diagnostic summary
   */
  generateSummary(checks) {
    return {
      'DOM Elements': `${checks.domElements.visibleElements}/${checks.domElements.totalElements} visible`,
      'Viewport': `${checks.domElements.inViewport} in view`,
      'Tracking': checks.guaranteeState.hasContext ?
        `${checks.guaranteeState.currentTracking?.displayedMessages}/${checks.guaranteeState.currentTracking?.expectedMessages} tracked` :
        'No active tracking',
      'Container': checks.domElements.containerExists ? '✅ Present' : '❌ Missing',
      'Scroll Position': checks.viewport.error ? 'Error' : `${checks.viewport.scrollPercentage}%`
    };
  }

  /**
   * Generate recommendations based on diagnostics
   */
  generateRecommendations(checks) {
    const recommendations = [];

    if (!checks.domElements.containerExists) {
      recommendations.push('Message container missing - check component rendering');
    }

    if (checks.domElements.totalElements === 0) {
      recommendations.push('No message elements found - check data loading');
    }

    if (checks.domElements.visibleElements < checks.domElements.totalElements * 0.8) {
      recommendations.push('Many messages not visible - check CSS and layout');
    }

    if (checks.guaranteeState.hasContext &&
      checks.guaranteeState.currentTracking?.retryAttempts > 0) {
      recommendations.push('Message tracking has retry attempts - investigate visibility detection');
    }

    if (checks.tracking.duplicateIds > 0) {
      recommendations.push(`Found ${checks.tracking.duplicateIds} duplicate message IDs - check key uniqueness`);
    }

    if (checks.performance.memoryUsage?.used > 100) {
      recommendations.push(`High memory usage (${checks.performance.memoryUsage.used}MB) - monitor for leaks`);
    }

    return recommendations;
  }

  /**
   * Export diagnostic report
   */
  exportReport(chatId) {
    const reports = this.diagnosticsHistory.filter(r => r.chatId === chatId);
    const data = {
      chatId,
      timestamp: Date.now(),
      reports,
      summary: {
        totalRuns: reports.length,
        lastRun: reports[reports.length - 1]?.timestamp,
        avgDuration: reports.reduce((sum, r) => sum + (r.duration || 0), 0) / reports.length
      }
    };

    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `message-display-diagnostics-${chatId}-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);

    console.log(`📄 [Diagnostics] Report exported for chat ${chatId}`);
  }
}

// Create global instance
const messageDisplayDiagnostics = new MessageDisplayDiagnostics();

// Expose to window for console access
if (typeof window !== 'undefined') {
  window.messageDisplayDiagnostics = messageDisplayDiagnostics;

  // Convenience methods
  window.diagnoseDhat = (chatId) => messageDisplayDiagnostics.runDiagnostics(chatId);
  window.exportDiagReport = (chatId) => messageDisplayDiagnostics.exportReport(chatId);
}

export default messageDisplayDiagnostics; 