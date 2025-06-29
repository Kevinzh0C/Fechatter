/**
 * 🏥 System Health Validator - Validate system health check fixes
 * 
 * Provides tools to validate that the SystemMonitor health check issues are resolved
 */

class SystemHealthValidator {
  constructor() {
    this.isEnabled = import.meta.env.DEV;
    this.validationResults = [];
  }

  /**
   * Run complete system health validation
   */
  async runCompleteValidation() {
    if (!this.isEnabled) {
      console.log('SystemHealthValidator only available in development');
      return { error: 'Not available in production' };
    }

    console.group('🏥 System Health Validation Started');

    const results = {
      timestamp: new Date().toISOString(),
      overall: 'unknown',
      checks: {},
      recommendations: []
    };

    try {
      // Check 1: UnifiedMessageService Global Availability
      results.checks.unifiedMessageService = this._validateUnifiedMessageService();

      // Check 2: MessageDisplayGuarantee Global Availability  
      results.checks.messageDisplayGuarantee = this._validateMessageDisplayGuarantee();

      // Check 3: SystemMonitor Health Logic
      results.checks.systemMonitorHealth = this._validateSystemMonitorHealth();

      // Check 4: DOM Element Detection (non-critical)
      results.checks.domElementDetection = this._validateDOMElementDetection();

      // Check 5: Monitoring Resilience
      results.checks.monitoringResilience = this._validateMonitoringResilience();

      // Determine overall result
      const criticalChecks = ['unifiedMessageService', 'messageDisplayGuarantee', 'systemMonitorHealth'];
      const criticalPassed = criticalChecks.every(check => results.checks[check].status === 'pass');

      results.overall = criticalPassed ? 'healthy' : 'unhealthy';

      // Generate recommendations
      results.recommendations = this._generateRecommendations(results.checks);

      console.log('Validation Results:', results);

    } catch (error) {
      console.error('ERROR: Validation failed:', error);
      results.overall = 'error';
      results.error = error.message;
    } finally {
      console.groupEnd();
    }

    return results;
  }

  /**
   * Validate UnifiedMessageService global availability
   */
  _validateUnifiedMessageService() {
    const result = {
      name: 'UnifiedMessageService Global Availability',
      status: 'unknown',
      details: {},
      critical: true
    };

    try {
      // Check global availability
      result.details.globalAvailable = typeof window.unifiedMessageService !== 'undefined';

      // Check if it's the correct instance
      if (window.unifiedMessageService) {
        result.details.hasRequiredMethods = [
          'fetchMoreMessages',
          'getMessagesForChat',
          'addMessage',
          'initialize'
        ].every(method => typeof window.unifiedMessageService[method] === 'function');

        result.details.isInitialized = window.unifiedMessageService.isInitialized?.value || false;
        result.details.hasCache = !!window.unifiedMessageService.messageCache;
      }

      result.status = result.details.globalAvailable && result.details.hasRequiredMethods ? 'pass' : 'fail';

    } catch (error) {
      result.status = 'error';
      result.error = error.message;
    }

    return result;
  }

  /**
   * Validate MessageDisplayGuarantee global availability
   */
  _validateMessageDisplayGuarantee() {
    const result = {
      name: 'MessageDisplayGuarantee Global Availability',
      status: 'unknown',
      details: {},
      critical: true
    };

    try {
      result.details.globalAvailable = typeof window.messageDisplayGuarantee !== 'undefined';

      if (window.messageDisplayGuarantee) {
        result.details.hasRequiredMethods = [
          'setEnabled',
          'getMetrics',
          'exportDebugInfo'
        ].every(method => typeof window.messageDisplayGuarantee[method] === 'function');

        result.details.isEnabled = window.messageDisplayGuarantee.isEnabled || false;
      }

      result.status = result.details.globalAvailable && result.details.hasRequiredMethods ? 'pass' : 'fail';

    } catch (error) {
      result.status = 'error';
      result.error = error.message;
    }

    return result;
  }

  /**
   * Validate SystemMonitor health check logic
   */
  _validateSystemMonitorHealth() {
    const result = {
      name: 'SystemMonitor Health Check Logic',
      status: 'unknown',
      details: {},
      critical: true
    };

    try {
      result.details.systemMonitorAvailable = typeof window.systemMonitor !== 'undefined';

      if (window.systemMonitor) {
        // Test the health check method
        const healthStatus = window.systemMonitor.getSystemStatus();
        result.details.healthCheckWorks = typeof healthStatus === 'object' && 'isHealthy' in healthStatus;
        result.details.currentHealthStatus = healthStatus.isHealthy;
        result.details.services = healthStatus.services;
      }

      result.status = result.details.systemMonitorAvailable && result.details.healthCheckWorks ? 'pass' : 'fail';

    } catch (error) {
      result.status = 'error';
      result.error = error.message;
    }

    return result;
  }

  /**
   * Validate DOM element detection (non-critical)
   */
  _validateDOMElementDetection() {
    const result = {
      name: 'DOM Element Detection',
      status: 'unknown',
      details: {},
      critical: false
    };

    try {
      const messageListElements = document.querySelectorAll('.simple-message-list');
      result.details.elementsFound = messageListElements.length;
      result.details.currentRoute = window.location.hash || window.location.pathname;
      result.details.isOnChatRoute = result.details.currentRoute.includes('chat');

      // This is non-critical - pass if we're not on a chat route
      result.status = result.details.elementsFound > 0 || !result.details.isOnChatRoute ? 'pass' : 'warning';

    } catch (error) {
      result.status = 'error';
      result.error = error.message;
    }

    return result;
  }

  /**
   * Validate monitoring resilience improvements
   */
  _validateMonitoringResilience() {
    const result = {
      name: 'Monitoring Resilience',
      status: 'unknown',
      details: {},
      critical: false
    };

    try {
      // Check if the improved health check logic is in place
      if (window.systemMonitor && typeof window.systemMonitor._checkSystemHealth === 'function') {
        const healthCheckSource = window.systemMonitor._checkSystemHealth.toString();
        result.details.hasImprovedLogic = healthCheckSource.includes('critical') && healthCheckSource.includes('failedChecks');
        result.details.hasLogging = healthCheckSource.includes('console.log');
      }

      result.status = result.details.hasImprovedLogic ? 'pass' : 'warning';

    } catch (error) {
      result.status = 'error';
      result.error = error.message;
    }

    return result;
  }

  /**
   * Generate recommendations based on validation results
   */
  _generateRecommendations(checks) {
    const recommendations = [];

    Object.values(checks).forEach(check => {
      if (check.status === 'fail' && check.critical) {
        recommendations.push({
          priority: 'high',
          issue: check.name,
          action: `Fix critical issue: ${check.name}`
        });
      } else if (check.status === 'warning') {
        recommendations.push({
          priority: 'medium',
          issue: check.name,
          action: `Address warning: ${check.name}`
        });
      } else if (check.status === 'error') {
        recommendations.push({
          priority: 'high',
          issue: check.name,
          action: `Investigate error in: ${check.name}`
        });
      }
    });

    if (recommendations.length === 0) {
      recommendations.push({
        priority: 'info',
        issue: 'System Health',
        action: 'All checks passed - system is healthy!'
      });
    }

    return recommendations;
  }

  /**
   * Quick health check - simplified version
   */
  quickHealthCheck() {
    if (!this.isEnabled) return { error: 'Not available in production' };

    const checks = {
      unifiedMessageService: !!window.unifiedMessageService,
      messageDisplayGuarantee: !!window.messageDisplayGuarantee,
      systemMonitor: !!window.systemMonitor
    };

    const overallHealth = Object.values(checks).every(check => check);

    console.log('🏥 Quick Health Check:', {
      overall: overallHealth ? 'Healthy' : 'ERROR: Unhealthy',
      services: checks
    });

    return { overall: overallHealth, services: checks };
  }
}

// Create global instance
const systemHealthValidator = new SystemHealthValidator();

// Export for module usage
export { systemHealthValidator };

// Make available globally in development
if (import.meta.env.DEV) {
  window.systemHealthValidator = systemHealthValidator;

  // Add convenient global functions
  window.validateSystemHealth = () => systemHealthValidator.runCompleteValidation();
  window.quickHealthCheck = () => systemHealthValidator.quickHealthCheck();

  console.log('🏥 System Health Validator loaded - use validateSystemHealth() or quickHealthCheck()');
} 