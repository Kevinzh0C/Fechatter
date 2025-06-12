<template>
  <div class="sse-performance-test">
    <div class="test-header">
      <h1>SSE Performance Testing Dashboard</h1>
      <p class="subtitle">比较原版和优化版SSE的性能表现</p>
    </div>

    <!-- Control Panel -->
    <div class="control-panel">
      <div class="current-implementation">
        <h3>当前实现</h3>
        <div class="implementation-status">
          <span class="badge" :class="currentImplementation">
            {{ currentImplementation === 'enhanced' ? '🚀 Enhanced SSE' : '📡 Original SSE' }}
          </span>
          <button @click="switchImplementation" :disabled="isSwitching" class="switch-btn">
            {{ isSwitching ? '切换中...' : '切换实现' }}
          </button>
        </div>
      </div>

      <div class="test-controls">
        <h3>测试控制</h3>
        <div class="control-buttons">
          <button @click="startTest" :disabled="isTestRunning" class="btn-primary">
            {{ isTestRunning ? '测试中...' : '开始性能测试' }}
          </button>
          <button @click="resetMetrics" :disabled="isTestRunning" class="btn-secondary">
            重置指标
          </button>
          <button @click="runComparison" :disabled="isTestRunning" class="btn-accent">
            对比测试 (1分钟)
          </button>
        </div>
      </div>
    </div>

    <!-- Real-time Metrics -->
    <div class="metrics-grid">
      <!-- Connection Status -->
      <div class="metric-card connection-status">
        <h4>连接状态</h4>
        <div class="status-display">
          <div class="status-item">
            <span class="label">状态:</span>
            <span class="value" :class="connectionState">{{ connectionState }}</span>
          </div>
          <div class="status-item">
            <span class="label">代理类型:</span>
            <span class="value">{{ proxyType }}</span>
          </div>
          <div class="status-item">
            <span class="label">连接质量:</span>
            <span class="value quality" :class="connectionQuality.toLowerCase()">
              {{ connectionQuality }}
            </span>
          </div>
        </div>
      </div>

      <!-- Performance Metrics -->
      <div class="metric-card performance-metrics">
        <h4>性能指标</h4>
        <div class="metrics-display">
          <div class="metric-row">
            <span class="metric-label">连接成功率:</span>
            <span class="metric-value success-rate">{{ metrics.connectionSuccessRate }}%</span>
          </div>
          <div class="metric-row">
            <span class="metric-label">平均延迟:</span>
            <span class="metric-value latency">{{ metrics.averageLatency }}ms</span>
          </div>
          <div class="metric-row">
            <span class="metric-label">错误率:</span>
            <span class="metric-value error-rate">{{ metrics.errorRate }}/min</span>
          </div>
          <div class="metric-row">
            <span class="metric-label">重连延迟:</span>
            <span class="metric-value reconnect-delay">{{ metrics.avgReconnectDelay }}ms</span>
          </div>
        </div>
      </div>

      <!-- Error Analytics -->
      <div class="metric-card error-analytics">
        <h4>错误分析</h4>
        <div class="error-display">
          <div class="error-item">
            <span class="error-type">总错误数:</span>
            <span class="error-count">{{ metrics.errorCount }}</span>
          </div>
          <div class="error-item">
            <span class="error-type">最近错误类型:</span>
            <span class="error-type-value">{{ lastErrorType || 'None' }}</span>
          </div>
          <div class="error-item">
            <span class="error-type">重连次数:</span>
            <span class="error-count">{{ metrics.totalReconnects }}</span>
          </div>
        </div>
      </div>

      <!-- Test Results -->
      <div class="metric-card test-results" v-if="comparisonResults">
        <h4>对比测试结果</h4>
        <div class="comparison-display">
          <div class="improvement-item" v-if="comparisonResults.improvement?.reconnectDelay">
            <span class="improvement-label">重连延迟改善:</span>
            <span class="improvement-value positive">-{{ comparisonResults.improvement.reconnectDelay }}%</span>
          </div>
          <div class="improvement-item" v-if="comparisonResults.improvement?.errorRate">
            <span class="improvement-label">错误率改善:</span>
            <span class="improvement-value positive">-{{ comparisonResults.improvement.errorRate }}%</span>
          </div>
          <div class="improvement-item" v-if="comparisonResults.improvement?.connectionSuccessRate">
            <span class="improvement-label">成功率提升:</span>
            <span class="improvement-value positive">+{{ comparisonResults.improvement.connectionSuccessRate }}%</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Debug Information -->
    <div class="debug-section" v-if="showDebugInfo">
      <h3>调试信息</h3>
      <div class="debug-grid">
        <div class="debug-card">
          <h4>连接历史</h4>
          <div class="debug-content">
            <pre>{{ JSON.stringify(debugInfo.latencyHistory, null, 2) }}</pre>
          </div>
        </div>
        <div class="debug-card">
          <h4>错误历史</h4>
          <div class="debug-content">
            <pre>{{ JSON.stringify(debugInfo.errorHistory, null, 2) }}</pre>
          </div>
        </div>
        <div class="debug-card">
          <h4>重连策略</h4>
          <div class="debug-content">
            <pre>{{ JSON.stringify(debugInfo.currentStrategy, null, 2) }}</pre>
          </div>
        </div>
      </div>
    </div>

    <!-- Toggle Debug -->
    <div class="debug-toggle">
      <button @click="showDebugInfo = !showDebugInfo" class="btn-debug">
        {{ showDebugInfo ? '隐藏调试信息' : '显示调试信息' }}
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, onUnmounted, computed } from 'vue';
import { sseFactory } from '@/config/sse-config';

// Reactive state
const currentImplementation = ref('enhanced');
const isSwitching = ref(false);
const isTestRunning = ref(false);
const showDebugInfo = ref(false);

const metrics = reactive({
  connectionSuccessRate: '0',
  averageLatency: '0',
  errorRate: '0.0',
  avgReconnectDelay: '0',
  errorCount: 0,
  totalReconnects: 0
});

const connectionState = ref('disconnected');
const connectionQuality = ref('GOOD');
const proxyType = ref('unknown');
const lastErrorType = ref(null);
const debugInfo = ref({});
const comparisonResults = ref(null);

// Update interval
let updateInterval = null;

/**
 * Switch between SSE implementations
 */
const switchImplementation = async () => {
  isSwitching.value = true;

  try {
    const useEnhanced = currentImplementation.value === 'original';
    const success = await sseFactory.switchSSEImplementation(useEnhanced);

    if (success) {
      currentImplementation.value = useEnhanced ? 'enhanced' : 'original';
      console.log(`✅ Switched to ${currentImplementation.value} implementation`);

      // Reset metrics after switch
      resetMetrics();
    }
  } catch (error) {
    console.error('❌ Failed to switch implementation:', error);
  } finally {
    isSwitching.value = false;
  }
};

/**
 * Start performance test
 */
const startTest = () => {
  isTestRunning.value = true;
  console.log('📊 Starting performance test...');

  // Reset metrics
  resetMetrics();

  // Run test for 30 seconds
  setTimeout(() => {
    isTestRunning.value = false;
    console.log('📊 Performance test completed');
  }, 30000);
};

/**
 * Reset metrics
 */
const resetMetrics = () => {
  const monitor = sseFactory.performanceMonitor;
  if (monitor) {
    monitor.resetMetrics();
  }

  // Reset display metrics
  Object.assign(metrics, {
    connectionSuccessRate: '0',
    averageLatency: '0',
    errorRate: '0.0',
    avgReconnectDelay: '0',
    errorCount: 0,
    totalReconnects: 0
  });

  comparisonResults.value = null;
};

/**
 * Run comparison test
 */
const runComparison = async () => {
  isTestRunning.value = true;

  try {
    console.log('📊 Starting comparison test...');
    const results = await sseFactory.runPerformanceComparison(60000); // 1 minute
    comparisonResults.value = results;
    console.log('📊 Comparison test completed:', results);
  } catch (error) {
    console.error('❌ Comparison test failed:', error);
  } finally {
    isTestRunning.value = false;
  }
};

/**
 * Update metrics display
 */
const updateMetrics = () => {
  const currentMetrics = sseFactory.getPerformanceMetrics();
  if (currentMetrics) {
    Object.assign(metrics, currentMetrics);
  }

  // Update connection state
  const sseService = sseFactory.getSSEService();
  if (sseService) {
    const state = sseService.getConnectionState?.();
    if (state) {
      connectionState.value = state.state || 'disconnected';
      connectionQuality.value = state.connectionQuality || 'GOOD';
      proxyType.value = state.proxyType || 'unknown';
      lastErrorType.value = state.lastErrorType;
    }

    // Update debug info
    if (showDebugInfo.value && sseService.getDebugInfo) {
      debugInfo.value = sseService.getDebugInfo();
    }
  }
};

/**
 * Lifecycle hooks
 */
onMounted(() => {
  console.log('📊 SSE Performance Test page mounted');

  // Determine current implementation
  const sseService = sseFactory.getSSEService();
  if (sseService.constructor.name.includes('Enhanced')) {
    currentImplementation.value = 'enhanced';
  } else {
    currentImplementation.value = 'original';
  }

  // Start updating metrics
  updateInterval = setInterval(updateMetrics, 1000);
  updateMetrics();
});

onUnmounted(() => {
  if (updateInterval) {
    clearInterval(updateInterval);
  }
});

// Computed properties
const connectionStateClass = computed(() => {
  return {
    'connected': 'success',
    'connecting': 'warning',
    'reconnecting': 'warning',
    'reconnecting_long_term': 'warning',
    'disconnected': 'error'
  }[connectionState.value] || 'error';
});
</script>

<style scoped>
.sse-performance-test {
  max-width: 1200px;
  margin: 0 auto;
  padding: 2rem;
  font-family: 'SF Pro Display', system-ui, sans-serif;
}

.test-header {
  text-align: center;
  margin-bottom: 2rem;
}

.test-header h1 {
  font-size: 2.5rem;
  font-weight: 700;
  color: #1a1a1a;
  margin-bottom: 0.5rem;
}

.subtitle {
  font-size: 1.1rem;
  color: #666;
  margin: 0;
}

.control-panel {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 2rem;
  margin-bottom: 2rem;
  padding: 1.5rem;
  background: #f8f9fa;
  border-radius: 12px;
  border: 1px solid #e9ecef;
}

.implementation-status {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.badge {
  padding: 0.5rem 1rem;
  border-radius: 20px;
  font-weight: 600;
  font-size: 0.9rem;
}

.badge.enhanced {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.badge.original {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
  color: white;
}

.control-buttons {
  display: flex;
  gap: 1rem;
  flex-wrap: wrap;
}

.btn-primary,
.btn-secondary,
.btn-accent,
.switch-btn,
.btn-debug {
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-primary {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.btn-secondary {
  background: #6c757d;
  color: white;
}

.btn-accent {
  background: linear-gradient(135deg, #ffecd2 0%, #fcb69f 100%);
  color: #333;
}

.switch-btn {
  background: #28a745;
  color: white;
}

.btn-debug {
  background: #17a2b8;
  color: white;
}

.btn-primary:hover,
.btn-secondary:hover,
.btn-accent:hover,
.switch-btn:hover,
.btn-debug:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.btn-primary:disabled,
.btn-secondary:disabled,
.btn-accent:disabled,
.switch-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1.5rem;
  margin-bottom: 2rem;
}

.metric-card {
  background: white;
  border-radius: 12px;
  padding: 1.5rem;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.08);
  border: 1px solid #e9ecef;
}

.metric-card h4 {
  margin: 0 0 1rem 0;
  font-size: 1.2rem;
  font-weight: 600;
  color: #333;
}

.status-display,
.metrics-display,
.error-display,
.comparison-display {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.status-item,
.metric-row,
.error-item,
.improvement-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0;
  border-bottom: 1px solid #f0f0f0;
}

.status-item:last-child,
.metric-row:last-child,
.error-item:last-child,
.improvement-item:last-child {
  border-bottom: none;
}

.label,
.metric-label,
.error-type,
.improvement-label {
  font-weight: 500;
  color: #666;
}

.value,
.metric-value,
.error-count,
.error-type-value,
.improvement-value {
  font-weight: 600;
  color: #333;
}

.value.success,
.value.connected {
  color: #28a745;
}

.value.warning,
.value.connecting,
.value.reconnecting {
  color: #ffc107;
}

.value.error,
.value.disconnected {
  color: #dc3545;
}

.quality.excellent {
  color: #28a745;
}

.quality.good {
  color: #20c997;
}

.quality.fair {
  color: #ffc107;
}

.quality.poor {
  color: #dc3545;
}

.improvement-value.positive {
  color: #28a745;
  font-weight: 700;
}

.debug-section {
  margin-top: 2rem;
  padding: 1.5rem;
  background: #f8f9fa;
  border-radius: 12px;
  border: 1px solid #e9ecef;
}

.debug-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1rem;
  margin-top: 1rem;
}

.debug-card {
  background: white;
  border-radius: 8px;
  padding: 1rem;
  border: 1px solid #dee2e6;
}

.debug-card h4 {
  margin: 0 0 0.5rem 0;
  font-size: 1rem;
  color: #333;
}

.debug-content {
  max-height: 200px;
  overflow-y: auto;
}

.debug-content pre {
  font-size: 0.8rem;
  color: #666;
  margin: 0;
  white-space: pre-wrap;
}

.debug-toggle {
  text-align: center;
  margin-top: 2rem;
}

/* Responsive design */
@media (max-width: 768px) {
  .control-panel {
    grid-template-columns: 1fr;
  }

  .metrics-grid {
    grid-template-columns: 1fr;
  }

  .control-buttons {
    flex-direction: column;
  }

  .implementation-status {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>