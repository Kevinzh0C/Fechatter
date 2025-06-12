/**
 * Performance Monitoring Composable
 * Tracks application performance metrics and provides optimization insights
 */

import { ref, onMounted, onUnmounted } from 'vue';
import { isDebugEnabled, getEnvConfig } from '@/utils/config';

export function usePerformanceMonitor() {
  // State
  const metrics = ref({
    // Core Web Vitals
    fcp: null, // First Contentful Paint
    lcp: null, // Largest Contentful Paint
    fid: null, // First Input Delay
    cls: null, // Cumulative Layout Shift

    // Custom Metrics
    timeToInteractive: null,
    navigationTiming: null,
    resourceTiming: [],
    memoryUsage: null,
    connectionInfo: null,

    // Application Metrics
    renderCount: 0,
    apiCallCount: 0,
    errorCount: 0,
    lastUpdated: null,
  });

  const isMonitoring = ref(false);
  const performanceObserver = ref(null);

  // Methods
  const startMonitoring = () => {
    if (isMonitoring.value || !window.performance) return;

    isMonitoring.value = true;
    collectInitialMetrics();
    observePerformanceEntries();
    monitorMemoryUsage();
    trackNetworkInformation();

    if (isDebugEnabled()) {
      }
  };

  const stopMonitoring = () => {
    if (!isMonitoring.value) return;

    if (performanceObserver.value) {
      performanceObserver.value.disconnect();
      performanceObserver.value = null;
    }

    isMonitoring.value = false;

    if (isDebugEnabled()) {
      }
  };

  const collectInitialMetrics = () => {
    // Navigation Timing
    const navigationTiming = performance.getEntriesByType('navigation')[0];
    if (navigationTiming) {
      metrics.value.navigationTiming = {
        domContentLoaded: navigationTiming.domContentLoadedEventEnd - navigationTiming.domContentLoadedEventStart,
        loadComplete: navigationTiming.loadEventEnd - navigationTiming.loadEventStart,
        totalTime: navigationTiming.loadEventEnd - navigationTiming.fetchStart,
        dnsLookup: navigationTiming.domainLookupEnd - navigationTiming.domainLookupStart,
        tcpConnection: navigationTiming.connectEnd - navigationTiming.connectStart,
        serverResponse: navigationTiming.responseEnd - navigationTiming.requestStart,
        domProcessing: navigationTiming.domComplete - navigationTiming.domLoading,
      };
    }

    // Resource Timing
    const resourceEntries = performance.getEntriesByType('resource');
    metrics.value.resourceTiming = resourceEntries.map(entry => ({
      name: entry.name,
      type: getResourceType(entry.name),
      duration: entry.duration,
      size: entry.transferSize || 0,
      cached: entry.transferSize === 0 && entry.decodedBodySize > 0,
    }));

    updateTimestamp();
  };

  const observePerformanceEntries = () => {
    if (!window.PerformanceObserver) return;

    // Observe Core Web Vitals
    try {
      performanceObserver.value = new PerformanceObserver((list) => {
        list.getEntries().forEach((entry) => {
          switch (entry.entryType) {
            case 'paint':
              if (entry.name === 'first-contentful-paint') {
                metrics.value.fcp = entry.startTime;
              }
              break;

            case 'largest-contentful-paint':
              metrics.value.lcp = entry.startTime;
              break;

            case 'first-input':
              metrics.value.fid = entry.processingStart - entry.startTime;
              break;

            case 'layout-shift':
              if (!entry.hadRecentInput) {
                metrics.value.cls = (metrics.value.cls || 0) + entry.value;
              }
              break;

            case 'measure':
              if (entry.name === 'fechatter-tti') {
                metrics.value.timeToInteractive = entry.duration;
              }
              break;

            case 'navigation':
              collectInitialMetrics();
              break;
          }
        });

        updateTimestamp();
      });

      // Start observing
      performanceObserver.value.observe({
        entryTypes: ['paint', 'largest-contentful-paint', 'first-input', 'layout-shift', 'measure', 'navigation']
      });
    } catch (error) {
      console.warn('Performance Observer not supported:', error);
    }
  };

  const monitorMemoryUsage = () => {
    if (!performance.memory) return;

    const updateMemory = () => {
      metrics.value.memoryUsage = {
        used: performance.memory.usedJSHeapSize,
        total: performance.memory.totalJSHeapSize,
        limit: performance.memory.jsHeapSizeLimit,
        percentage: Math.round((performance.memory.usedJSHeapSize / performance.memory.jsHeapSizeLimit) * 100),
      };
      updateTimestamp();
    };

    updateMemory();

    // Update memory info every 30 seconds
    const memoryInterval = setInterval(updateMemory, 30000);

    onUnmounted(() => {
      clearInterval(memoryInterval);
    });
  };

  const trackNetworkInformation = () => {
    if (!navigator.connection) return;

    const updateConnection = () => {
      metrics.value.connectionInfo = {
        effectiveType: navigator.connection.effectiveType,
        downlink: navigator.connection.downlink,
        rtt: navigator.connection.rtt,
        saveData: navigator.connection.saveData,
      };
      updateTimestamp();
    };

    updateConnection();
    navigator.connection.addEventListener('change', updateConnection);

    onUnmounted(() => {
      navigator.connection.removeEventListener('change', updateConnection);
    });
  };

  const getResourceType = (url) => {
    if (url.includes('.js')) return 'script';
    if (url.includes('.css')) return 'stylesheet';
    if (url.match(/\.(png|jpg|jpeg|gif|svg|webp)$/)) return 'image';
    if (url.match(/\.(woff|woff2|ttf|eot)$/)) return 'font';
    if (url.includes('/api/')) return 'api';
    return 'other';
  };

  const updateTimestamp = () => {
    metrics.value.lastUpdated = Date.now();
  };

  // Tracking methods
  const trackRender = () => {
    metrics.value.renderCount++;
    updateTimestamp();
  };

  const trackApiCall = () => {
    metrics.value.apiCallCount++;
    updateTimestamp();
  };

  const trackError = () => {
    metrics.value.errorCount++;
    updateTimestamp();
  };

  const measureTimeToInteractive = () => {
    // Mark TTI when the component is fully interactive
    setTimeout(() => {
      performance.mark('fechatter-tti-start');
      requestIdleCallback(() => {
        performance.mark('fechatter-tti-end');
        performance.measure('fechatter-tti', 'fechatter-tti-start', 'fechatter-tti-end');
      });
    }, 0);
  };

  // Analysis methods
  const getPerformanceScore = () => {
    const scores = {
      fcp: getScoreForFCP(metrics.value.fcp),
      lcp: getScoreForLCP(metrics.value.lcp),
      fid: getScoreForFID(metrics.value.fid),
      cls: getScoreForCLS(metrics.value.cls),
    };

    const validScores = Object.values(scores).filter(score => score !== null);
    if (validScores.length === 0) return null;

    const averageScore = validScores.reduce((sum, score) => sum + score, 0) / validScores.length;

    return {
      overall: Math.round(averageScore),
      breakdown: scores,
      grade: getPerformanceGrade(averageScore),
    };
  };

  const getScoreForFCP = (fcp) => {
    if (fcp === null) return null;
    if (fcp <= 1800) return 90;
    if (fcp <= 3000) return 50;
    return 0;
  };

  const getScoreForLCP = (lcp) => {
    if (lcp === null) return null;
    if (lcp <= 2500) return 90;
    if (lcp <= 4000) return 50;
    return 0;
  };

  const getScoreForFID = (fid) => {
    if (fid === null) return null;
    if (fid <= 100) return 90;
    if (fid <= 300) return 50;
    return 0;
  };

  const getScoreForCLS = (cls) => {
    if (cls === null) return null;
    if (cls <= 0.1) return 90;
    if (cls <= 0.25) return 50;
    return 0;
  };

  const getPerformanceGrade = (score) => {
    if (score >= 90) return 'A';
    if (score >= 70) return 'B';
    if (score >= 50) return 'C';
    if (score >= 30) return 'D';
    return 'F';
  };

  const getOptimizationSuggestions = () => {
    const suggestions = [];

    if (metrics.value.lcp > 2500) {
      suggestions.push({
        type: 'performance',
        priority: 'high',
        message: 'Largest Contentful Paint is slow. Consider optimizing images and reducing server response time.',
      });
    }

    if (metrics.value.fid > 100) {
      suggestions.push({
        type: 'performance',
        priority: 'medium',
        message: 'First Input Delay is high. Consider reducing JavaScript execution time.',
      });
    }

    if (metrics.value.cls > 0.1) {
      suggestions.push({
        type: 'layout',
        priority: 'medium',
        message: 'Cumulative Layout Shift is high. Ensure elements have defined dimensions.',
      });
    }

    if (metrics.value.memoryUsage?.percentage > 80) {
      suggestions.push({
        type: 'memory',
        priority: 'high',
        message: 'Memory usage is high. Consider optimizing component lifecycle and data management.',
      });
    }

    const largeResources = metrics.value.resourceTiming.filter(resource => resource.size > 1000000);
    if (largeResources.length > 0) {
      suggestions.push({
        type: 'resources',
        priority: 'medium',
        message: `${largeResources.length} large resources detected. Consider compression and lazy loading.`,
      });
    }

    return suggestions;
  };

  const exportMetrics = () => {
    const exportData = {
      timestamp: new Date().toISOString(),
      url: window.location.href,
      userAgent: navigator.userAgent,
      metrics: metrics.value,
      score: getPerformanceScore(),
      suggestions: getOptimizationSuggestions(),
    };

    return exportData;
  };

  const logMetrics = () => {
    if (!isDebugEnabled()) return;

    const score = getPerformanceScore();
    const suggestions = getOptimizationSuggestions();

    console.group('📊 Performance Metrics');
    console.log('Overall Score:', score?.overall || 'N/A', `(Grade: ${score?.grade || 'N/A'})`);
    console.log('Core Web Vitals:', {
      FCP: metrics.value.fcp ? `${Math.round(metrics.value.fcp)}ms` : 'N/A',
      LCP: metrics.value.lcp ? `${Math.round(metrics.value.lcp)}ms` : 'N/A',
      FID: metrics.value.fid ? `${Math.round(metrics.value.fid)}ms` : 'N/A',
      CLS: metrics.value.cls ? metrics.value.cls.toFixed(3) : 'N/A',
    });
    console.log('Memory Usage:', metrics.value.memoryUsage?.percentage ? `${metrics.value.memoryUsage.percentage}%` : 'N/A');
    console.log('Network:', metrics.value.connectionInfo?.effectiveType || 'N/A');

    if (suggestions.length > 0) {
      console.warn('Optimization Suggestions:', suggestions);
    }

    console.groupEnd();
  };

  // Lifecycle
  onMounted(() => {
    startMonitoring();
    measureTimeToInteractive();
  });

  onUnmounted(() => {
    stopMonitoring();
  });

  return {
    // State
    metrics,
    isMonitoring,

    // Methods
    startMonitoring,
    stopMonitoring,
    trackRender,
    trackApiCall,
    trackError,
    measureTimeToInteractive,

    // Analysis
    getPerformanceScore,
    getOptimizationSuggestions,
    exportMetrics,
    logMetrics,
  };
} 