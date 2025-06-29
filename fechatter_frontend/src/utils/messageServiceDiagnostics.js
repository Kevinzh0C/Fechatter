/**
 * UnifiedMessageService Diagnostics
 * Comprehensive monitoring and debugging for message service issues
 */

class MessageServiceDiagnostics {
  constructor() {
    this.errorHistory = [];
    this.performanceMetrics = [];
    this.isMonitoring = false;
  }

  /**
   * Run comprehensive UnifiedMessageService diagnostics
   */
  async runDiagnostics(chatId = null) {
    console.log('EVENT: [MessageService Diagnostics] Starting comprehensive message service analysis...');

    const startTime = Date.now();
    const result = {
      timestamp: startTime,
      chatId,
      checks: {},
      issues: [],
      recommendations: [],
      autoFixable: []
    };

    try {
      // Check 1: Service Availability
      result.checks.serviceAvailability = this.checkServiceAvailability();

      // Check 2: Message Cache Health
      result.checks.cacheHealth = this.checkCacheHealth();

      // Check 3: API Integration
      result.checks.apiIntegration = await this.checkAPIIntegration();

      // Check 4: Store Integration
      result.checks.storeIntegration = this.checkStoreIntegration();

      // Check 5: Memory Usage
      result.checks.memoryUsage = this.checkMemoryUsage();

      // Check 6: Specific Chat Analysis (if chatId provided)
      if (chatId) {
        result.checks.chatSpecific = await this.checkSpecificChat(chatId);
      }

      // Generate analysis
      result.issues = this.identifyIssues(result.checks);
      result.recommendations = this.generateRecommendations(result.checks);
      result.autoFixable = this.identifyAutoFixes(result.checks);

      const duration = Date.now() - startTime;
      result.duration = duration;

      console.log(`[MessageService Diagnostics] Analysis completed in ${duration}ms`);
      this.displayResults(result);

      return result;

    } catch (error) {
      console.error('ERROR: [MessageService Diagnostics] Error during analysis:', error);
      result.error = error.message;
      return result;
    }
  }

  /**
   * Check UnifiedMessageService availability
   */
  checkServiceAvailability() {
    const result = {
      serviceExists: false,
      isInitialized: false,
      hasAllMethods: false,
      methodsAvailable: []
    };

    // Check if service exists globally
    if (window.unifiedMessageService) {
      result.serviceExists = true;
      result.isInitialized = window.unifiedMessageService.isInitialized?.value || false;

      // Check critical methods
      const requiredMethods = ['fetchMessages', 'fetchMoreMessages', 'sendMessage', 'clearMessagesForChat'];
      const availableMethods = [];

      requiredMethods.forEach(method => {
        if (typeof window.unifiedMessageService[method] === 'function') {
          availableMethods.push(method);
        }
      });

      result.methodsAvailable = availableMethods;
      result.hasAllMethods = availableMethods.length === requiredMethods.length;
    }

    return result;
  }

  /**
   * Check message cache health
   */
  checkCacheHealth() {
    const result = {
      cacheSize: 0,
      chatCount: 0,
      totalMessages: 0,
      cacheDetails: {},
      hasStaleEntries: false
    };

    if (window.unifiedMessageService) {
      try {
        const messagesByChat = window.unifiedMessageService.messagesByChat;
        const messageCache = window.unifiedMessageService.messageCache;

        if (messagesByChat instanceof Map) {
          result.chatCount = messagesByChat.size;
          result.totalMessages = Array.from(messagesByChat.values())
            .reduce((total, messages) => total + (messages?.length || 0), 0);
        }

        if (messageCache && typeof messageCache === 'object') {
          const now = Date.now();
          const staleThreshold = 10 * 60 * 1000; // 10 minutes

          Object.entries(messageCache).forEach(([chatId, cache]) => {
            const age = now - (cache.timestamp || 0);
            const isStale = age > staleThreshold;

            result.cacheDetails[chatId] = {
              messageCount: cache.messages?.length || 0,
              age: Math.round(age / 1000), // seconds
              isStale,
              isFresh: cache.isFresh || false
            };

            if (isStale) result.hasStaleEntries = true;
          });

          result.cacheSize = Object.keys(messageCache).length;
        }
      } catch (error) {
        result.error = error.message;
      }
    }

    return result;
  }

  /**
   * Check API integration health
   */
  async checkAPIIntegration() {
    const result = {
      apiServiceAvailable: false,
      canImportAPI: false,
      endpointsWorking: {}
    };

    try {
      // Test dynamic import of API service
      const { default: api } = await import('../services/api');
      result.apiServiceAvailable = !!api;
      result.canImportAPI = true;

      // Test basic endpoints
      const endpointsToTest = [
        { name: 'health', url: '/api/health' },
        { name: 'workspaceChats', url: '/api/workspace/chats' }
      ];

      for (const endpoint of endpointsToTest) {
        try {
          const response = await fetch(endpoint.url, {
            headers: {
              'Authorization': `Bearer ${localStorage.getItem('access_token')}`
            }
          });

          result.endpointsWorking[endpoint.name] = {
            status: response.status,
            ok: response.ok,
            available: true
          };
        } catch (error) {
          result.endpointsWorking[endpoint.name] = {
            available: false,
            error: error.message
          };
        }
      }

    } catch (importError) {
      result.canImportAPI = false;
      result.importError = importError.message;
    }

    return result;
  }

  /**
   * Check store integration
   */
  checkStoreIntegration() {
    const result = {
      piniaStoresAvailable: false,
      userStoreWorking: false,
      workspaceStoreWorking: false,
      chatStoreWorking: false,
      storeDetails: {}
    };

    if (window.__pinia_stores__) {
      result.piniaStoresAvailable = true;

      // Check user store
      try {
        const userStore = window.__pinia_stores__.user();
        result.userStoreWorking = !!userStore;
        result.storeDetails.user = {
          workspaceUsers: userStore.workspaceUsers?.length || 0,
          loading: userStore.loading || false,
          error: userStore.error || null
        };
      } catch (error) {
        result.storeDetails.user = { error: error.message };
      }

      // Check workspace store
      try {
        const workspaceStore = window.__pinia_stores__.workspace();
        result.workspaceStoreWorking = !!workspaceStore;
        result.storeDetails.workspace = {
          workspaceUsers: workspaceStore.workspaceUsers?.length || 0,
          currentWorkspace: !!workspaceStore.currentWorkspace,
          loading: workspaceStore.loading || false
        };
      } catch (error) {
        result.storeDetails.workspace = { error: error.message };
      }

      // Check chat store
      try {
        const chatStore = window.__pinia_stores__.chat();
        result.chatStoreWorking = !!chatStore;
        result.storeDetails.chat = {
          currentChatId: chatStore.currentChatId || null,
          messages: chatStore.messages?.length || 0,
          loading: chatStore.loading || false
        };
      } catch (error) {
        result.storeDetails.chat = { error: error.message };
      }
    }

    return result;
  }

  /**
   * Check memory usage
   */
  checkMemoryUsage() {
    const result = {
      memorySupported: false,
      jsHeapSize: null,
      estimatedMessageMemory: 0
    };

    if (window.performance && window.performance.memory) {
      result.memorySupported = true;
      result.jsHeapSize = {
        used: Math.round(window.performance.memory.usedJSHeapSize / 1024 / 1024), // MB
        total: Math.round(window.performance.memory.totalJSHeapSize / 1024 / 1024) // MB
      };
    }

    // Estimate message cache memory usage
    if (window.unifiedMessageService?.messagesByChat) {
      try {
        const messageData = Array.from(window.unifiedMessageService.messagesByChat.values());
        const jsonSize = JSON.stringify(messageData).length;
        result.estimatedMessageMemory = Math.round(jsonSize / 1024); // KB
      } catch (error) {
        result.estimatedMessageMemory = 'calculation_failed';
      }
    }

    return result;
  }

  /**
   * Check specific chat health
   */
  async checkSpecificChat(chatId) {
    const result = {
      chatId,
      hasMessages: false,
      messageCount: 0,
      hasMore: false,
      cacheStatus: 'unknown',
      canFetchMore: false,
      fetchMoreErrors: []
    };

    if (window.unifiedMessageService) {
      try {
        // Check current messages
        const messages = window.unifiedMessageService.getMessagesForChat(chatId);
        result.hasMessages = messages && messages.length > 0;
        result.messageCount = messages ? messages.length : 0;

        // Check hasMore status
        result.hasMore = window.unifiedMessageService.hasMoreMessages(chatId);

        // Check cache status
        const cache = window.unifiedMessageService.messageCache[chatId];
        if (cache) {
          const age = Date.now() - cache.timestamp;
          result.cacheStatus = age < 60000 ? 'fresh' : age < 300000 ? 'stale' : 'old';
        } else {
          result.cacheStatus = 'none';
        }

        // Test fetchMoreMessages (safely)
        try {
          // Don't actually fetch, just check if method exists and is callable
          result.canFetchMore = typeof window.unifiedMessageService.fetchMoreMessages === 'function';
        } catch (error) {
          result.fetchMoreErrors.push(error.message);
        }

      } catch (error) {
        result.error = error.message;
      }
    }

    return result;
  }

  /**
   * Identify issues from diagnostic results
   */
  identifyIssues(checks) {
    const issues = [];

    if (!checks.serviceAvailability.serviceExists) {
      issues.push({
        severity: 'critical',
        category: 'service',
        issue: 'UnifiedMessageService not available',
        impact: 'No message functionality'
      });
    }

    if (!checks.serviceAvailability.hasAllMethods) {
      issues.push({
        severity: 'high',
        category: 'service',
        issue: 'Missing required methods',
        impact: 'Limited message functionality',
        details: checks.serviceAvailability.methodsAvailable
      });
    }

    if (checks.cacheHealth.hasStaleEntries) {
      issues.push({
        severity: 'medium',
        category: 'performance',
        issue: 'Stale cache entries detected',
        impact: 'Potential memory waste'
      });
    }

    if (!checks.apiIntegration.canImportAPI) {
      issues.push({
        severity: 'critical',
        category: 'integration',
        issue: 'Cannot import API service',
        impact: 'No network communication possible'
      });
    }

    if (!checks.storeIntegration.piniaStoresAvailable) {
      issues.push({
        severity: 'high',
        category: 'integration',
        issue: 'Pinia stores not accessible',
        impact: 'User data resolution fails'
      });
    }

    if (checks.memoryUsage.jsHeapSize && checks.memoryUsage.jsHeapSize.used > 100) {
      issues.push({
        severity: 'medium',
        category: 'performance',
        issue: 'High memory usage detected',
        impact: 'Potential performance degradation',
        details: `${checks.memoryUsage.jsHeapSize.used}MB used`
      });
    }

    return issues;
  }

  /**
   * Generate recommendations
   */
  generateRecommendations(checks) {
    const recommendations = [];

    if (!checks.serviceAvailability.serviceExists) {
      recommendations.push({
        category: 'service',
        action: 'Reinitialize UnifiedMessageService',
        priority: 'critical'
      });
    }

    if (checks.cacheHealth.hasStaleEntries) {
      recommendations.push({
        category: 'performance',
        action: 'Clear stale cache entries',
        priority: 'medium'
      });
    }

    if (!checks.storeIntegration.userStoreWorking) {
      recommendations.push({
        category: 'integration',
        action: 'Refresh user store data',
        priority: 'high'
      });
    }

    return recommendations;
  }

  /**
   * Identify auto-fixable issues
   */
  identifyAutoFixes(checks) {
    const autoFixes = [];

    if (checks.cacheHealth.hasStaleEntries) {
      autoFixes.push('clear-stale-cache');
    }

    if (!checks.storeIntegration.userStoreWorking) {
      autoFixes.push('refresh-user-store');
    }

    return autoFixes;
  }

  /**
   * Auto-fix identified issues
   */
  async autoFixIssues() {
    console.log('[MessageService Diagnostics] Starting auto-fix process...');

    const fixes = [];

    try {
      // Fix 1: Clear stale cache
      if (window.unifiedMessageService?.messageCache) {
        const now = Date.now();
        const staleThreshold = 10 * 60 * 1000; // 10 minutes
        let clearedCount = 0;

        Object.keys(window.unifiedMessageService.messageCache).forEach(chatId => {
          const cache = window.unifiedMessageService.messageCache[chatId];
          if (cache && (now - cache.timestamp) > staleThreshold) {
            delete window.unifiedMessageService.messageCache[chatId];
            clearedCount++;
          }
        });

        if (clearedCount > 0) {
          fixes.push({
            type: 'clear-stale-cache',
            status: 'success',
            details: `Cleared ${clearedCount} stale entries`
          });
        }
      }

      // Fix 2: Refresh user store
      if (window.__pinia_stores__?.user) {
        try {
          const userStore = window.__pinia_stores__.user();
          if (userStore.fetchWorkspaceUsers) {
            await userStore.fetchWorkspaceUsers();
            fixes.push({
              type: 'refresh-user-store',
              status: 'success'
            });
          }
        } catch (error) {
          fixes.push({
            type: 'refresh-user-store',
            status: 'failed',
            error: error.message
          });
        }
      }

    } catch (error) {
      fixes.push({
        type: 'auto-fix',
        status: 'failed',
        error: error.message
      });
    }

    console.log('[MessageService Diagnostics] Auto-fix completed:', fixes);
    return fixes;
  }

  /**
   * Display diagnostic results
   */
  displayResults(result) {
    console.group('EVENT: UnifiedMessageService Diagnostics Results');

    console.log(`⏱️ Analysis Time: ${result.duration}ms`);

    if (result.issues.length > 0) {
      console.log('\n🚨 Issues Found:');
      result.issues.forEach((issue, idx) => {
        const icon = issue.severity === 'critical' ? '🚨' : issue.severity === 'high' ? '⚠️' : 'ℹ️';
        console.log(`  ${idx + 1}. ${icon} [${issue.category}] ${issue.issue}`);
        console.log(`     Impact: ${issue.impact}`);
        if (issue.details) console.log(`     Details:`, issue.details);
      });
    } else {
      console.log('No critical issues found');
    }

    if (result.recommendations.length > 0) {
      console.log('\nRecommendations:');
      result.recommendations.forEach((rec, idx) => {
        console.log(`  ${idx + 1}. [${rec.category}] ${rec.action} (${rec.priority})`);
      });
    }

    if (result.autoFixable.length > 0) {
      console.log('\nAuto-fixable issues found:', result.autoFixable);
      console.log('Run fixDMessageService() to attempt automatic repairs');
    }

    console.groupEnd();
  }

  /**
   * Start monitoring for real-time issues
   */
  startMonitoring() {
    if (this.isMonitoring) return;

    this.isMonitoring = true;
    console.log('SUBSCRIPTION: [MessageService Diagnostics] Started real-time monitoring');

    // Monitor for errors
    const originalConsoleError = console.error;
    console.error = (...args) => {
      if (args.some(arg => String(arg).includes('UnifiedMessageService'))) {
        this.errorHistory.push({
          timestamp: Date.now(),
          error: args.join(' ')
        });
      }
      originalConsoleError.apply(console, args);
    };
  }

  /**
   * Stop monitoring
   */
  stopMonitoring() {
    this.isMonitoring = false;
    console.log('SUBSCRIPTION: [MessageService Diagnostics] Stopped monitoring');
  }

  /**
   * Get monitoring data
   */
  getMonitoringData() {
    return {
      isMonitoring: this.isMonitoring,
      errorHistory: this.errorHistory.slice(-10), // Last 10 errors
      performanceMetrics: this.performanceMetrics.slice(-10)
    };
  }
}

// Create global instance
const messageServiceDiagnostics = new MessageServiceDiagnostics();

// Expose to window for console access
if (typeof window !== 'undefined') {
  window.messageServiceDiagnostics = messageServiceDiagnostics;

  // Convenience commands
  window.diagnoseDMessageService = (chatId) => messageServiceDiagnostics.runDiagnostics(chatId);
  window.fixDMessageService = () => messageServiceDiagnostics.autoFixIssues();
  window.monitorDMessageService = () => messageServiceDiagnostics.startMonitoring();
}

export default messageServiceDiagnostics; 