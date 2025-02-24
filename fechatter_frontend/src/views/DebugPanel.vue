<template>
  <!-- Debug Panel -->
  <Transition name="debug-panel">
    <div v-if="isVisible" class="debug-panel">
      <div class="debug-header">
        <h3>🐛 Debug Panel</h3>
        <button @click="isVisible = false" class="close-btn">×</button>
      </div>

      <div class="debug-content">
        <!-- Health Status -->
        <div class="debug-section">
          <h4>🏥 Health Status</h4>
          <div class="status-grid">
            <div v-for="check in healthChecks" :key="check.checkId"
              :class="['status-item', check.success ? 'success' : 'error']">
              <span class="status-name">{{ check.checkName }}</span>
              <span class="status-icon">{{ check.success ? '✅' : '❌' }}</span>
            </div>
          </div>
          <button @click="runHealthCheck" class="action-btn">Run Health Check</button>
        </div>

        <!-- Error Monitor -->
        <div class="debug-section">
          <h4>📊 Error Stats</h4>
          <div class="stats-grid">
            <div class="stat-item">
              <span class="stat-label">Total Errors</span>
              <span class="stat-value">{{ errorStats.total }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">Critical</span>
              <span class="stat-value critical">{{ errorStats.critical }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">Warnings</span>
              <span class="stat-value warning">{{ errorStats.warnings }}</span>
            </div>
          </div>
          <div class="action-buttons">
            <button @click="viewErrors" class="action-btn">View Errors</button>
            <button @click="clearErrors" class="action-btn secondary">Clear</button>
          </div>
        </div>

        <!-- Current State -->
        <div class="debug-section">
          <h4>🔍 Current State</h4>
          <div class="state-info">
            <div class="state-item">
              <span>Route:</span>
              <code>{{ currentRoute }}</code>
            </div>
            <div class="state-item">
              <span>User ID:</span>
              <code>{{ userId || 'Not logged in' }}</code>
            </div>
            <div class="state-item">
              <span>Active Workspace:</span>
              <code>{{ activeWorkspace }}</code>
            </div>
            <div class="state-item">
              <span>Current Chat:</span>
              <code>{{ currentChat }}</code>
            </div>
            <div class="state-item">
              <span>Messages Count:</span>
              <code>{{ messagesCount }}</code>
            </div>
          </div>
        </div>

        <!-- Quick Actions -->
        <div class="debug-section">
          <h4>⚡ Quick Actions</h4>
          <div class="action-grid">
            <button @click="testApi" class="action-btn">Test API</button>
            <button @click="diagnoseApi" class="action-btn">Diagnose API</button>
            <button @click="exportReport" class="action-btn">Export Report</button>
            <button @click="refreshStores" class="action-btn">Refresh Stores</button>
          </div>
        </div>

        <!-- Recent Errors -->
        <div v-if="showErrors && recentErrors.length > 0" class="debug-section">
          <h4>❌ Recent Errors</h4>
          <div class="error-list">
            <div v-for="error in recentErrors" :key="error.id" class="error-item">
              <div class="error-header">
                <span class="error-type">{{ error.error.type }}</span>
                <span class="error-time">{{ formatTime(error.timestamp) }}</span>
              </div>
              <div class="error-message">{{ error.error.message }}</div>
              <div v-if="error.context.component" class="error-component">
                Component: {{ error.context.component }}
              </div>
            </div>
          </div>
        </div>

        <!-- SSE Connection Test -->
        <div class="debug-section">
          <h3>🔌 Real-time Connection (SSE)</h3>
          <div class="debug-content">
            <div class="connection-status">
              <div class="status-indicator"
                :class="{ 'connected': sseStatus.isConnected, 'disconnected': !sseStatus.isConnected }">
                {{ sseStatus.isConnected ? 'Connected' : 'Disconnected' }}
              </div>
              <div class="connection-details">
                <div>State: {{ sseStatus.connectionState }}</div>
                <div>Reconnect Attempts: {{ sseStatus.reconnectAttempts }}</div>
                <div v-if="sseStatus.lastActivityTime">
                  Last Activity: {{ formatTime(sseStatus.lastActivityTime) }}
                </div>
              </div>
            </div>

            <div class="test-controls">
              <button @click="testSSEConnection" :disabled="testingSSE" class="test-btn">
                {{ testingSSE ? 'Testing...' : 'Test SSE Connection' }}
              </button>
              <button @click="reconnectSSE" class="test-btn">
                Reconnect SSE
              </button>
              <button @click="disconnectSSE" class="test-btn">
                Disconnect SSE
              </button>
            </div>

            <div v-if="sseTestResults.length > 0" class="test-results">
              <h4>Recent SSE Events:</h4>
              <div class="log-container">
                <div v-for="(result, index) in sseTestResults.slice(-10)" :key="index" class="log-entry"
                  :class="result.type">
                  <span class="timestamp">{{ formatTime(result.timestamp) }}</span>
                  <span class="message">{{ result.message }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Transition>

  <!-- Toggle Button -->
  <button v-if="!isVisible" @click="isVisible = true" class="debug-toggle">
    🐛
  </button>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { useRoute } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import { useChatStore } from '@/stores/chat';
import { useWorkspaceStore } from '@/stores/workspace';
import errorMonitor from '@/utils/errorMonitor';
import healthCheck from '@/utils/healthCheck';
import api from '@/services/api';
import minimalSSE from '@/services/sse-minimal';

// Visibility
const isVisible = ref(false);
const showErrors = ref(false);

// Stores
const route = useRoute();
const authStore = useAuthStore();
const chatStore = useChatStore();
const workspaceStore = useWorkspaceStore();

// State
const healthChecks = ref([]);
const errorStats = ref({ total: 0, critical: 0, warnings: 0 });
const recentErrors = ref([]);
const sseStatus = ref({ isConnected: false, connectionState: 'Disconnected', reconnectAttempts: 0, lastActivityTime: null });
const testingSSE = ref(false);
const sseTestResults = ref([]);

// Computed
const currentRoute = computed(() => route.path);
const userId = computed(() => authStore.user?.id);
const activeWorkspace = computed(() => workspaceStore.activeWorkspaceId);
const currentChat = computed(() => chatStore.currentChatId);
const messagesCount = computed(() => chatStore.messages.length);

// Methods
async function runHealthCheck() {
  try {
    // 使用安全的健康检查方法
    const result = await healthCheck.runAllChecksSafely();
    if (result) {
      healthChecks.value = result.results;
      console.log('Health check completed:', result.summary);

      // 如果有应用未就绪的错误，显示提示
      if (result.summary.error) {
        console.warn('Health check warning:', result.summary.error);
      }
  } catch (error) {
    console.error('Health check failed:', error);

    // 创建错误结果显示
    healthChecks.value = [{
      checkId: 'health_system',
      checkName: 'Health Check System',
      success: false,
      error: error.message,
      critical: true,
      timestamp: new Date().toISOString()
    }];
  }

function updateErrorStats() {
  try {
    const stats = errorMonitor.getErrorStats();
    errorStats.value = stats;
    recentErrors.value = stats.recent || [];
  } catch (error) {
    console.error('Failed to update error stats:', error);
    errorStats.value = { total: 0, critical: 0, warnings: 0 };
    recentErrors.value = [];
  }

function viewErrors() {
  showErrors.value = !showErrors.value;
  if (showErrors.value) {
    updateErrorStats();
  }

function clearErrors() {
  errorMonitor.clearErrors();
  updateErrorStats();
  showErrors.value = false;
}

async function testApi() {
  try {
    const response = await api.get('/health');
    alert(`API Test Success!\nStatus: ${response.status}\nData: ${JSON.stringify(response.data)}`);
  } catch (error) {
    alert(`API Test Failed!\nError: ${error.message}`);
  }

function diagnoseApi() {
  if (window.diagnoseApi) {
    window.diagnoseApi.diagnoseApiConfigurations();
  } else {
    console.error('API diagnostic tools not loaded');
  }

function exportReport() {
  const report = {
    timestamp: new Date().toISOString(),
    health: healthCheck.getDetailedReport(),
    errors: errorMonitor.getErrorStats(),
    state: {
      route: currentRoute.value,
      userId: userId.value,
      workspace: activeWorkspace.value,
      chat: currentChat.value,
      messages: messagesCount.value
    }
  };

  const blob = new Blob([JSON.stringify(report, null, 2)], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `debug-report-${Date.now()}.json`;
  a.click();
  URL.revokeObjectURL(url);
}

async function refreshStores() {
  try {
    await Promise.all([
      authStore.fetchUser(),
      workspaceStore.fetchWorkspaces(),
      chatStore.fetchChats()
    ]);
    alert('Stores refreshed successfully!');
  } catch (error) {
    alert(`Failed to refresh stores: ${error.message}`);
  }

function formatTime(timestamp) {
  const date = new Date(timestamp);
  return date.toLocaleTimeString();
}

// Keyboard support
function handleKeydown(event) {
  if (event.key === 'Escape' && isVisible.value) {
    isVisible.value = false;
  }

// Auto-update
let updateInterval;
onMounted(() => {
  runHealthCheck();
  updateErrorStats();

  // Add keyboard listener
  document.addEventListener('keydown', handleKeydown);

  // Setup SSE debug listeners
  setupSSEDebugListeners();
  updateSSEStatus();

  updateInterval = setInterval(() => {
    if (isVisible.value) {
      updateErrorStats();
      updateSSEStatus(); // Update SSE status regularly
    }
  }, 5000);
});

// Cleanup
onUnmounted(() => {
  if (updateInterval) {
    clearInterval(updateInterval);
  document.removeEventListener('keydown', handleKeydown);

  // Clean up SSE debug listeners
  minimalSSE.off('connected');
  minimalSSE.off('disconnected');
  minimalSSE.off('message');
});

// Watch for visibility changes
watch(isVisible, (newVal) => {
  if (newVal) {
    runHealthCheck();
    updateErrorStats();
  }
});

async function testSSEConnection() {
  testingSSE.value = true;
  sseTestResults.value = [];

  function addLog(message, type = 'info') {
    sseTestResults.value.push({
      timestamp: new Date(),
      message,
      type
    });

  try {
    addLog('🔧 Starting SSE connection test...', 'info');

    // Check if we have auth token
    if (!authStore.token) {
      addLog('❌ No authentication token available', 'error');
      return;
    }

    // Check current SSE status
    const currentStatus = minimalSSE.getConnectionState();
    addLog(`📊 Current SSE state: ${currentStatus.connectionState}`, 'info');

    // If already connected, test by sending presence update
    if (currentStatus.isConnected) {
      addLog('✅ SSE already connected, testing with presence update...', 'success');
      await minimalSSE.sendPresenceUpdate('online');
      addLog('✅ Presence update sent successfully', 'success');
    } else {
      // Try to connect
      addLog('🔌 Attempting SSE connection...', 'info');
      await minimalSSE.connect(authStore.token);
      addLog('✅ SSE connection initiated', 'success');
    }

    // Update status display
    updateSSEStatus();

  } catch (error) {
    addLog(`❌ SSE test failed: ${error.message}`, 'error');
    console.error('SSE test error:', error);
  } finally {
    testingSSE.value = false;
  }

async function reconnectSSE() {
  try {
    sseTestResults.value.push({
      timestamp: new Date(),
      message: '🔄 Reconnecting SSE...',
      type: 'info'
    });

    // Disconnect first
    minimalSSE.disconnect();

    // Wait a moment
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Reconnect
    if (authStore.token) {
      await minimalSSE.connect(authStore.token);
      sseTestResults.value.push({
        timestamp: new Date(),
        message: '✅ SSE reconnected successfully',
        type: 'success'
      });
    } else {
      sseTestResults.value.push({
        timestamp: new Date(),
        message: '❌ No auth token for reconnection',
        type: 'error'
      });

    updateSSEStatus();
  } catch (error) {
    sseTestResults.value.push({
      timestamp: new Date(),
      message: `❌ Reconnection failed: ${error.message}`,
      type: 'error'
    });
    console.error('SSE reconnection error:', error);
  }

function disconnectSSE() {
  try {
    minimalSSE.disconnect();
    sseTestResults.value.push({
      timestamp: new Date(),
      message: '🔌 SSE disconnected',
      type: 'info'
    });

    updateSSEStatus();
  } catch (error) {
    sseTestResults.value.push({
      timestamp: new Date(),
      message: `❌ Disconnect failed: ${error.message}`,
      type: 'error'
    });
    console.error('SSE disconnect error:', error);
  }

function updateSSEStatus() {
  const status = minimalSSE.getConnectionState();
  sseStatus.value = {
    isConnected: status.isConnected || false,
    connectionState: status.connectionState || 'unknown',
    reconnectAttempts: status.reconnectAttempts || 0,
    lastActivityTime: status.lastActivityTime || null
  };
}

// Add SSE event listeners for debugging
function setupSSEDebugListeners() {
  minimalSSE.on('connected', () => {
    sseTestResults.value.push({
      timestamp: new Date(),
      message: '🔗 SSE Connected (event)',
      type: 'success'
    });
    updateSSEStatus();
  });

  minimalSSE.on('disconnected', () => {
    sseTestResults.value.push({
      timestamp: new Date(),
      message: '🔌 SSE Disconnected (event)',
      type: 'warning'
    });
    updateSSEStatus();
  });

  minimalSSE.on('message', (data) => {
    sseTestResults.value.push({
      timestamp: new Date(),
      message: `📨 SSE Message: ${JSON.stringify(data).substring(0, 100)}...`,
      type: 'info'
    });
  });
</script>

<style scoped>
/* Debug Overlay */
.debug-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  padding: 20px;
  box-sizing: border-box;
}

/* Debug Panel */
.debug-panel {
  position: fixed;
  bottom: 20px;
  right: 20px;
  width: 400px;
  max-width: 90vw;
  max-height: 80vh;
  background: #1a1a1a;
  color: #fff;
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  overflow: hidden;
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 12px;
}

.debug-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  background: #2a2a2a;
  border-bottom: 1px solid #3a3a3a;
}

.debug-header h3 {
  margin: 0;
  font-size: 16px;
}

.close-btn {
  background: none;
  border: none;
  color: #999;
  font-size: 24px;
  cursor: pointer;
  padding: 0;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: #3a3a3a;
  color: #fff;
}

.debug-content {
  padding: 16px;
  overflow-y: auto;
  max-height: calc(85vh - 60px);
}

.debug-section {
  margin-bottom: 20px;
  padding-bottom: 20px;
  border-bottom: 1px solid #3a3a3a;
}

.debug-section:last-child {
  border-bottom: none;
  margin-bottom: 0;
  padding-bottom: 0;
}

.debug-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  color: #00b4d8;
}

/* Status Grid */
.status-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  margin-bottom: 12px;
}

.status-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: #2a2a2a;
  border-radius: 6px;
  border: 1px solid #3a3a3a;
}

.status-item.success {
  border-color: #10b981;
}

.status-item.error {
  border-color: #ef4444;
}

.status-name {
  font-size: 11px;
}

/* Stats Grid */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
  margin-bottom: 12px;
}

.stat-item {
  text-align: center;
  padding: 12px;
  background: #2a2a2a;
  border-radius: 6px;
  border: 1px solid #3a3a3a;
}

.stat-label {
  display: block;
  font-size: 10px;
  color: #999;
  margin-bottom: 4px;
}

.stat-value {
  display: block;
  font-size: 20px;
  font-weight: bold;
}

.stat-value.critical {
  color: #ef4444;
}

.stat-value.warning {
  color: #f59e0b;
}

/* State Info */
.state-info {
  background: #2a2a2a;
  padding: 12px;
  border-radius: 6px;
  border: 1px solid #3a3a3a;
}

.state-item {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
}

.state-item:last-child {
  margin-bottom: 0;
}

.state-item span:first-child {
  color: #999;
}

.state-item code {
  color: #00b4d8;
  background: #1a1a1a;
  padding: 2px 6px;
  border-radius: 3px;
}

/* Action Buttons */
.action-btn {
  background: #00b4d8;
  color: #fff;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s;
}

.action-btn:hover {
  background: #0891b2;
}

.action-btn.secondary {
  background: #3a3a3a;
}

.action-btn.secondary:hover {
  background: #4a4a4a;
}

.action-buttons {
  display: flex;
  gap: 8px;
}

.action-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

/* Error List */
.error-list {
  max-height: 200px;
  overflow-y: auto;
}

.error-item {
  background: #2a2a2a;
  padding: 12px;
  border-radius: 6px;
  margin-bottom: 8px;
  border: 1px solid #3a3a3a;
}

.error-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
}

.error-type {
  color: #ef4444;
  font-weight: bold;
}

.error-time {
  color: #999;
  font-size: 10px;
}

.error-message {
  color: #fff;
  margin-bottom: 4px;
}

.error-component {
  color: #999;
  font-size: 10px;
}

/* Toggle Button */
.debug-toggle {
  position: fixed;
  top: 50%;
  right: 0;
  transform: translateY(-50%);
  width: 50px;
  height: 50px;
  background: #1a1a1a;
  border: 2px solid #00b4d8;
  border-radius: 25px 0 0 25px;
  color: #fff;
  font-size: 24px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: -4px 0 12px rgba(0, 0, 0, 0.3);
  transition: all 0.2s;
  z-index: 9998;
  border-right: none;
}

.debug-toggle:hover {
  transform: translateY(-50%) translateX(-5px);
  box-shadow: -6px 0 20px rgba(0, 0, 0, 0.4);
  background: #2a2a2a;
}

/* Scrollbar */
.debug-content::-webkit-scrollbar,
.error-list::-webkit-scrollbar {
  width: 6px;
}

.debug-content::-webkit-scrollbar-track,
.error-list::-webkit-scrollbar-track {
  background: #1a1a1a;
}

.debug-content::-webkit-scrollbar-thumb,
.error-list::-webkit-scrollbar-thumb {
  background: #4a4a4a;
  border-radius: 3px;
}

.debug-content::-webkit-scrollbar-thumb:hover,
.error-list::-webkit-scrollbar-thumb:hover {
  background: #5a5a5a;
}

/* Transition animations */
.debug-fade-enter-active,
.debug-fade-leave-active {
  transition: opacity 0.3s ease;
}

.debug-fade-enter-from,
.debug-fade-leave-to {
  opacity: 0;
}

.debug-panel-enter-active,
.debug-panel-leave-active {
  transition: all 0.3s ease;
}

.debug-panel-enter-from,
.debug-panel-leave-to {
  opacity: 0;
  transform: scale(0.8);
}

/* Responsive design */
@media (max-width: 768px) {
  .debug-panel {
    width: 95vw;
    margin: 10px;
  }

  .debug-content {
    max-height: calc(90vh - 60px);
  }

  .status-grid,
  .action-grid {
    grid-template-columns: 1fr;
  }

  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }

/* Connection Status */
.connection-status {
  margin-bottom: 12px;
  padding: 12px;
  background: #2a2a2a;
  border-radius: 6px;
  border: 1px solid #3a3a3a;
}

.status-indicator {
  font-weight: bold;
  margin-bottom: 8px;
  padding: 4px 8px;
  border-radius: 4px;
  text-align: center;
}

.status-indicator.connected {
  background: rgba(16, 185, 129, 0.2);
  color: #10b981;
  border: 1px solid #10b981;
}

.status-indicator.disconnected {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
  border: 1px solid #ef4444;
}

.connection-details {
  font-size: 11px;
  color: #999;
}

.connection-details>div {
  margin-bottom: 2px;
}

/* Test Controls */
.test-controls {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.test-btn {
  padding: 6px 12px;
  background: #3a3a3a;
  border: 1px solid #4a4a4a;
  color: #fff;
  border-radius: 4px;
  font-size: 11px;
  cursor: pointer;
  transition: all 0.2s;
}

.test-btn:hover:not(:disabled) {
  background: #4a4a4a;
  border-color: #5a5a5a;
}

.test-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Test Results */
.test-results {
  margin-top: 12px;
}

.test-results h4 {
  margin: 0 0 8px 0;
  font-size: 12px;
  color: #999;
}

.log-container {
  max-height: 200px;
  overflow-y: auto;
  background: #1a1a1a;
  border: 1px solid #3a3a3a;
  border-radius: 4px;
  padding: 8px;
}

.log-entry {
  display: flex;
  gap: 8px;
  margin-bottom: 4px;
  font-size: 10px;
  line-height: 1.4;
}

.log-entry:last-child {
  margin-bottom: 0;
}

.log-entry .timestamp {
  color: #666;
  min-width: 60px;
  flex-shrink: 0;
}

.log-entry .message {
  flex: 1;
}

.log-entry.success .message {
  color: #10b981;
}

.log-entry.error .message {
  color: #ef4444;
}

.log-entry.warning .message {
  color: #f59e0b;
}

.log-entry.info .message {
  color: #3b82f6;
}
</style>