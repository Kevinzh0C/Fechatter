<template>
  <div class="protobuf-analytics-test">
    <div class="header">
      <h2>🔬 Protobuf Analytics Test Panel</h2>
      <div class="status-indicator" :class="{ active: analytics.enabled }">
        {{ analytics.enabled ? 'ENABLED' : 'DISABLED' }}
      </div>
    </div>

    <div class="status-panel">
      <h3>📊 Client Status</h3>
      <div class="status-grid">
        <div class="status-item">
          <span class="label">Protobuf Available:</span>
          <span class="value" :class="{ success: status.protobufAvailable, error: !status.protobufAvailable }">
            {{ status.protobufAvailable ? '✅ YES' : '❌ NO' }}
          </span>
        </div>
        <div class="status-item">
          <span class="label">Pending Events:</span>
          <span class="value">{{ status.pendingEvents }}</span>
        </div>
        <div class="status-item">
          <span class="label">Client ID:</span>
          <span class="value">{{ status.clientId }}</span>
        </div>
        <div class="status-item">
          <span class="label">Fallback Mode:</span>
          <span class="value" :class="{ warning: !status.protobufAvailable }">
            {{ status.protobufAvailable ? 'Protobuf' : 'JSON' }}
          </span>
        </div>
      </div>
    </div>

    <div class="test-panel">
      <h3>🧪 Test Events</h3>
      <div class="test-buttons">
        <button @click="testAppStart" class="test-btn primary">
          🚀 App Start
        </button>
        <button @click="testUserLogin" class="test-btn primary">
          👤 User Login
        </button>
        <button @click="testMessageSent" class="test-btn primary">
          💬 Message Sent
        </button>
        <button @click="testNavigation" class="test-btn primary">
          🧭 Navigation
        </button>
        <button @click="testError" class="test-btn danger">
          ⚠️ Error Event
        </button>
        <button @click="testSearch" class="test-btn primary">
          🔍 Search
        </button>
        <button @click="testFileUpload" class="test-btn primary">
          📁 File Upload
        </button>
        <button @click="flushEvents" class="test-btn success">
          🔄 Flush Batch
        </button>
      </div>
    </div>

    <div class="logs-panel">
      <h3>📝 Event Logs</h3>
      <div class="logs-container">
        <div v-for="(log, index) in logs" :key="index" class="log-entry" :class="log.type">
          <span class="timestamp">{{ formatTime(log.timestamp) }}</span>
          <span class="message">{{ log.message }}</span>
        </div>
      </div>
      <button @click="clearLogs" class="clear-logs-btn">
        🗑️ Clear Logs
      </button>
    </div>

    <div class="network-panel">
      <h3>🌐 Network Activity</h3>
      <div class="network-stats">
        <div class="stat-item">
          <span class="label">Total Requests:</span>
          <span class="value">{{ networkStats.totalRequests }}</span>
        </div>
        <div class="stat-item">
          <span class="label">Successful:</span>
          <span class="value success">{{ networkStats.successful }}</span>
        </div>
        <div class="stat-item">
          <span class="label">Failed:</span>
          <span class="value error">{{ networkStats.failed }}</span>
        </div>
        <div class="stat-item">
          <span class="label">Protobuf Requests:</span>
          <span class="value">{{ networkStats.protobufRequests }}</span>
        </div>
        <div class="stat-item">
          <span class="label">JSON Fallback:</span>
          <span class="value warning">{{ networkStats.jsonFallback }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, reactive, onMounted, onUnmounted } from 'vue'
// import { CompleteProtobufAnalyticsClient } from '../../lib/analytics-protobuf-complete.js'
// Temporarily disable analytics for build
const CompleteProtobufAnalyticsClient = class {
  constructor() { }
  getStatus() { return { enabled: false, protobufAvailable: false, pendingEvents: 0, clientId: 'disabled' } }
  trackAppStart() { return Promise.resolve() }
  trackUserLogin() { return Promise.resolve() }
  trackMessageSent() { return Promise.resolve() }
  trackNavigation() { return Promise.resolve() }
  trackError() { return Promise.resolve() }
  trackSearch() { return Promise.resolve() }
  trackFileUpload() { return Promise.resolve() }
  flush() { return Promise.resolve() }
}

export default {
  name: 'ProtobufAnalyticsTest',
  setup() {
    // Create analytics client for testing
    const analytics = new CompleteProtobufAnalyticsClient({
      enabled: true,
      debug: true,
      endpoint: 'http://127.0.0.1:6690',
      fallback_to_json: true,
      batch_size: 10, // Smaller batch for testing
      flush_interval: 10000, // 10 seconds for testing
    })

    const status = ref({
      enabled: false,
      protobufAvailable: false,
      pendingEvents: 0,
      clientId: '',
    })

    const logs = ref([])
    const networkStats = reactive({
      totalRequests: 0,
      successful: 0,
      failed: 0,
      protobufRequests: 0,
      jsonFallback: 0,
    })

    let statusUpdateInterval = null

    // Update status periodically
    const updateStatus = () => {
      status.value = analytics.getStatus()
    }

    // Add log entry
    const addLog = (message, type = 'info') => {
      logs.value.unshift({
        timestamp: Date.now(),
        message,
        type,
      })

      // Keep only last 50 logs
      if (logs.value.length > 50) {
        logs.value = logs.value.slice(0, 50)
      }
    }

    // Test functions
    const testAppStart = async () => {
      try {
        addLog('Testing app start event...', 'info')
        await analytics.trackAppStart()
        addLog('✅ App start event sent successfully', 'success')
        networkStats.totalRequests++
        networkStats.successful++
        if (status.value.protobufAvailable) {
          networkStats.protobufRequests++
        } else {
          networkStats.jsonFallback++
        }
      } catch (error) {
        addLog(`❌ App start event failed: ${error.message}`, 'error')
        networkStats.totalRequests++
        networkStats.failed++
      }
      updateStatus()
    }

    const testUserLogin = async () => {
      try {
        addLog('Testing user login event...', 'info')
        await analytics.trackUserLogin('test@fechatter.com', 'password')
        addLog('✅ User login event sent successfully', 'success')
        networkStats.totalRequests++
        networkStats.successful++
        if (status.value.protobufAvailable) {
          networkStats.protobufRequests++
        } else {
          networkStats.jsonFallback++
        }
      } catch (error) {
        addLog(`❌ User login event failed: ${error.message}`, 'error')
        networkStats.totalRequests++
        networkStats.failed++
      }
      updateStatus()
    }

    const testMessageSent = async () => {
      try {
        addLog('Testing message sent event...', 'info')
        await analytics.trackMessageSent('chat_test_123', 'Hello from protobuf test! @everyone', [])
        addLog('✅ Message sent event queued successfully', 'success')
      } catch (error) {
        addLog(`❌ Message sent event failed: ${error.message}`, 'error')
      }
      updateStatus()
    }

    const testNavigation = async () => {
      try {
        addLog('Testing navigation event...', 'info')
        const startTime = Date.now() - Math.random() * 2000 // Random navigation time
        await analytics.trackNavigation('/test-from', '/test-to', startTime)
        addLog('✅ Navigation event queued successfully', 'success')
      } catch (error) {
        addLog(`❌ Navigation event failed: ${error.message}`, 'error')
      }
      updateStatus()
    }

    const testError = async () => {
      try {
        addLog('Testing error event...', 'info')
        const testError = new Error('This is a test error from protobuf analytics')
        testError.stack = 'Error: Test error\n    at ProtobufAnalyticsTest.vue:testError:1:1'
        await analytics.trackError(testError, 'protobuf-test-component', 'TestError')
        addLog('✅ Error event sent successfully', 'success')
        networkStats.totalRequests++
        networkStats.successful++
        if (status.value.protobufAvailable) {
          networkStats.protobufRequests++
        } else {
          networkStats.jsonFallback++
        }
      } catch (error) {
        addLog(`❌ Error event failed: ${error.message}`, 'error')
        networkStats.totalRequests++
        networkStats.failed++
      }
      updateStatus()
    }

    const testSearch = async () => {
      try {
        addLog('Testing search event...', 'info')
        await analytics.trackSearch('global', 'protobuf test query', Math.floor(Math.random() * 20), Math.floor(Math.random() * 500), true)
        addLog('✅ Search event queued successfully', 'success')
      } catch (error) {
        addLog(`❌ Search event failed: ${error.message}`, 'error')
      }
      updateStatus()
    }

    const testFileUpload = async () => {
      try {
        addLog('Testing file upload event...', 'info')
        const mockFile = {
          type: 'image/png',
          size: 1024 * 512, // 512KB
          name: 'protobuf-test.png'
        }
        await analytics.trackFileUpload(mockFile, 'test-upload', Math.floor(Math.random() * 3000))
        addLog('✅ File upload event queued successfully', 'success')
      } catch (error) {
        addLog(`❌ File upload event failed: ${error.message}`, 'error')
      }
      updateStatus()
    }

    const flushEvents = async () => {
      try {
        addLog('Flushing pending events...', 'info')
        await analytics.flush()
        addLog('✅ Events flushed successfully', 'success')
        networkStats.totalRequests++
        networkStats.successful++
        if (status.value.protobufAvailable) {
          networkStats.protobufRequests++
        } else {
          networkStats.jsonFallback++
        }
      } catch (error) {
        addLog(`❌ Flush failed: ${error.message}`, 'error')
        networkStats.totalRequests++
        networkStats.failed++
      }
      updateStatus()
    }

    const clearLogs = () => {
      logs.value = []
      addLog('Logs cleared', 'info')
    }

    const formatTime = (timestamp) => {
      return new Date(timestamp).toLocaleTimeString()
    }

    onMounted(() => {
      updateStatus()
      statusUpdateInterval = setInterval(updateStatus, 1000)
      addLog('Protobuf Analytics Test Panel initialized', 'success')

      // Set user ID for testing
      analytics.setUserId('test-user-123')
      analytics.setSessionId('test-session-' + Date.now())
    })

    onUnmounted(() => {
      if (statusUpdateInterval) {
        clearInterval(statusUpdateInterval)
      }
      analytics.destroy()
    })

    return {
      analytics,
      status,
      logs,
      networkStats,
      testAppStart,
      testUserLogin,
      testMessageSent,
      testNavigation,
      testError,
      testSearch,
      testFileUpload,
      flushEvents,
      clearLogs,
      formatTime,
    }
  }
}
</script>

<style scoped>
.protobuf-analytics-test {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
  font-family: 'Monaco', 'Menlo', monospace;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 10px;
  border-bottom: 2px solid #333;
}

.header h2 {
  margin: 0;
  color: #333;
}

.status-indicator {
  padding: 8px 16px;
  border-radius: 4px;
  font-weight: bold;
  background-color: #ff4444;
  color: white;
}

.status-indicator.active {
  background-color: #44aa44;
}

.status-panel,
.test-panel,
.logs-panel,
.network-panel {
  margin-bottom: 20px;
  padding: 15px;
  border: 1px solid #ddd;
  border-radius: 8px;
  background-color: #f9f9f9;
}

.status-panel h3,
.test-panel h3,
.logs-panel h3,
.network-panel h3 {
  margin-top: 0;
  color: #333;
}

.status-grid,
.network-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 10px;
}

.status-item,
.stat-item {
  display: flex;
  justify-content: space-between;
  padding: 8px;
  background-color: white;
  border-radius: 4px;
  border: 1px solid #eee;
}

.label {
  font-weight: bold;
  color: #666;
}

.value {
  font-family: monospace;
}

.value.success {
  color: #28a745;
}

.value.error {
  color: #dc3545;
}

.value.warning {
  color: #ffc107;
}

.test-buttons {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 10px;
}

.test-btn {
  padding: 10px 15px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: bold;
  transition: all 0.2s;
}

.test-btn.primary {
  background-color: #007bff;
  color: white;
}

.test-btn.primary:hover {
  background-color: #0056b3;
}

.test-btn.success {
  background-color: #28a745;
  color: white;
}

.test-btn.success:hover {
  background-color: #1e7e34;
}

.test-btn.danger {
  background-color: #dc3545;
  color: white;
}

.test-btn.danger:hover {
  background-color: #c82333;
}

.logs-container {
  max-height: 300px;
  overflow-y: auto;
  background-color: #1a1a1a;
  color: #fff;
  padding: 10px;
  border-radius: 4px;
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 12px;
}

.log-entry {
  display: flex;
  margin-bottom: 5px;
  padding: 2px 0;
}

.log-entry.success {
  color: #4caf50;
}

.log-entry.error {
  color: #f44336;
}

.log-entry.warning {
  color: #ff9800;
}

.timestamp {
  color: #888;
  margin-right: 10px;
  min-width: 80px;
}

.message {
  flex: 1;
}

.clear-logs-btn {
  margin-top: 10px;
  padding: 8px 16px;
  background-color: #6c757d;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.clear-logs-btn:hover {
  background-color: #545b62;
}
</style>