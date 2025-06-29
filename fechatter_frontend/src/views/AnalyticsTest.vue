<template>
  <div class="analytics-test-container">
    <div class="test-header">
      <h1>Analytics Batch Testing</h1>
      <p>Test the Analytics client batch sending mechanism and various event types</p>
    </div>

    <div class="test-grid">
      <!-- Analytics Status -->
      <div class="test-card">
        <h3>📈 Analytics Status</h3>
        <div class="status-grid">
          <div class="status-item">
            <span class="label">Client Enabled:</span>
            <span :class="['value', analytics.isEnabled ? 'success' : 'error']">
              {{ analytics.isEnabled ? 'Yes' : 'No' }}
            </span>
          </div>
          <div class="status-item">
            <span class="label">Queue Size:</span>
            <span class="value">{{ queueSize }}</span>
          </div>
          <div class="status-item">
            <span class="label">Events Sent:</span>
            <span class="value success">{{ eventsSent }}</span>
          </div>
          <div class="status-item">
            <span class="label">Batch Size:</span>
            <span class="value">{{ analytics.getClient()?.batchSize || 'N/A' }}</span>
          </div>
        </div>
      </div>

      <!-- Quick Actions -->
      <div class="test-card">
        <h3>Quick Actions</h3>
        <div class="action-buttons">
          <button @click="sendSingleEvent" class="btn btn-primary">
            Send Single Event
          </button>
          <button @click="sendBatchEvents" class="btn btn-secondary">
            Send Batch (10 events)
          </button>
          <button @click="sendMixedEvents" class="btn btn-accent">
            Send Mixed Event Types
          </button>
          <button @click="forceFlush" class="btn btn-warning">
            Force Flush Queue
          </button>
        </div>
      </div>

      <!-- Event Type Testing -->
      <div class="test-card full-width">
        <h3>Event Type Testing</h3>
        <div class="event-type-grid">
          <button 
            v-for="eventType in eventTypes" 
            :key="eventType.name"
            @click="sendEventType(eventType)"
            class="event-btn"
            :class="eventType.category"
          >
            <span class="icon">{{ eventType.icon }}</span>
            <span class="name">{{ eventType.name }}</span>
          </button>
        </div>
      </div>

      <!-- Performance Testing -->
      <div class="test-card">
        <h3>Performance Testing</h3>
        <div class="perf-controls">
          <div class="input-group">
            <label>Event Count:</label>
            <input v-model.number="perfEventCount" type="number" min="1" max="1000" />
          </div>
          <div class="input-group">
            <label>Interval (ms):</label>
            <input v-model.number="perfInterval" type="number" min="10" max="5000" />
          </div>
          <button @click="runPerformanceTest" :disabled="isRunningPerfTest" class="btn btn-danger">
            {{ isRunningPerfTest ? 'Running...' : 'Run Performance Test' }}
          </button>
        </div>
        <div v-if="perfResults" class="perf-results">
          <h4>Performance Results:</h4>
          <div class="result-item">
            <span>Total Time:</span>
            <span>{{ perfResults.totalTime }}ms</span>
          </div>
          <div class="result-item">
            <span>Events/Second:</span>
            <span>{{ perfResults.eventsPerSecond }}</span>
          </div>
          <div class="result-item">
            <span>Success Rate:</span>
            <span>{{ perfResults.successRate }}%</span>
          </div>
        </div>
      </div>

      <!-- Event Log -->
      <div class="test-card full-width">
        <h3>Event Log</h3>
        <div class="log-controls">
          <button @click="clearLog" class="btn btn-small">Clear Log</button>
          <label class="checkbox-label">
            <input v-model="autoScroll" type="checkbox" />
            Auto-scroll
          </label>
        </div>
        <div class="event-log" ref="logContainer">
          <div 
            v-for="log in eventLog" 
            :key="log.id"
            class="log-entry"
            :class="log.type"
          >
            <span class="timestamp">{{ formatTime(log.timestamp) }}</span>
            <span class="event-type">{{ log.eventType }}</span>
            <span class="message">{{ log.message }}</span>
            <span v-if="log.data" class="data">{{ JSON.stringify(log.data, null, 2) }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, nextTick, computed } from 'vue'
import { useAnalytics } from '@/composables/useAnalytics'

const analytics = useAnalytics()

// Reactive state
const eventsSent = ref(0)
const queueSize = ref(0)
const eventLog = ref([])
const autoScroll = ref(true)
const logContainer = ref(null)

// Performance testing
const perfEventCount = ref(100)
const perfInterval = ref(100)
const isRunningPerfTest = ref(false)
const perfResults = ref(null)

// Event types for testing
const eventTypes = ref([
  { name: 'App Start', icon: '', category: 'system', event: 'app_start', data: {} },
  { name: 'User Login', icon: '🔐', category: 'auth', event: 'user_login', data: { email: 'test@example.com', login_method: 'password' } },
  { name: 'Message Sent', icon: '💬', category: 'chat', event: 'message_sent', data: { chat_id: 'test_chat', message_type: 'text', message_size: 50 } },
  { name: 'Navigation', icon: '🧭', category: 'navigation', event: 'navigation', data: { from: 'home', to: 'chat', duration_ms: 250 } },
  { name: 'File Upload', icon: '📁', category: 'file', event: 'file_uploaded', data: { file_type: 'image', file_size: 1024000, upload_method: 'drag_drop' } },
  { name: 'Search', icon: '', category: 'search', event: 'search_performed', data: { search_type: 'messages', query_length: 10, results_count: 5 } },
  { name: 'Error', icon: '❌', category: 'error', event: 'error_occurred', data: { error_type: 'network', error_code: '500', error_message: 'Server error' } },
  { name: 'Bot Response', icon: '🤖', category: 'ai', event: 'bot_response', data: { bot_id: 'gpt-4', response_type: 'chat', response_time_ms: 1500, success: true } }
])

// Computed
const client = computed(() => analytics.getClient())

// Methods
function addLog(type, eventType, message, data = null) {
  const log = {
    id: Date.now() + Math.random(),
    timestamp: Date.now(),
    type,
    eventType,
    message,
    data
  }
  
  eventLog.value.push(log)
  
  // Keep only last 100 logs
  if (eventLog.value.length > 100) {
    eventLog.value = eventLog.value.slice(-100)
  }
  
  if (autoScroll.value) {
    nextTick(() => {
      if (logContainer.value) {
        logContainer.value.scrollTop = logContainer.value.scrollHeight
      }
    })
  }
}

function clearLog() {
  eventLog.value = []
}

function formatTime(timestamp) {
  return new Date(timestamp).toLocaleTimeString()
}

async function sendSingleEvent() {
  try {
    await analytics.track('app_start', { test_mode: true, timestamp: Date.now() })
    eventsSent.value++
    addLog('success', 'app_start', 'Single event sent successfully')
  } catch (error) {
    addLog('error', 'app_start', `Failed to send event: ${error.message}`)
  }
}

async function sendBatchEvents() {
  try {
    const promises = []
    for (let i = 0; i < 10; i++) {
      promises.push(analytics.track('message_sent', {
        chat_id: `test_chat_${i}`,
        message_type: 'text',
        message_size: Math.floor(Math.random() * 200) + 10,
        batch_test: true,
        index: i
      }))
    }
    
    await Promise.all(promises)
    eventsSent.value += 10
    addLog('success', 'batch', '10 events sent in batch')
  } catch (error) {
    addLog('error', 'batch', `Failed to send batch: ${error.message}`)
  }
}

async function sendMixedEvents() {
  try {
    const events = [
      analytics.trackUserLogin('test@example.com', 'password'),
      analytics.trackMessageSent('chat_123', 'text', 45, { hasMentions: true }),
      analytics.trackNavigation('home', 'chat', 300),
      analytics.trackFileUpload('image', 2048000, 'button', 1500),
      analytics.trackSearch('messages', 15, 8, 250, true)
    ]
    
    await Promise.all(events)
    eventsSent.value += 5
    addLog('success', 'mixed', '5 mixed event types sent')
  } catch (error) {
    addLog('error', 'mixed', `Failed to send mixed events: ${error.message}`)
  }
}

async function sendEventType(eventType) {
  try {
    await analytics.track(eventType.event, eventType.data)
    eventsSent.value++
    addLog('success', eventType.event, `${eventType.name} event sent`, eventType.data)
  } catch (error) {
    addLog('error', eventType.event, `Failed to send ${eventType.name}: ${error.message}`)
  }
}

async function forceFlush() {
  try {
    await analytics.flush()
    addLog('info', 'flush', 'Queue flushed manually')
  } catch (error) {
    addLog('error', 'flush', `Failed to flush queue: ${error.message}`)
  }
}

async function runPerformanceTest() {
  if (isRunningPerfTest.value) return
  
  isRunningPerfTest.value = true
  perfResults.value = null
  
  const startTime = Date.now()
  let successCount = 0
  let errorCount = 0
  
  try {
    addLog('info', 'perf_test', `Starting performance test: ${perfEventCount.value} events with ${perfInterval.value}ms interval`)
    
    for (let i = 0; i < perfEventCount.value; i++) {
      try {
        await analytics.track('performance_test', {
          index: i,
          timestamp: Date.now(),
          test_id: `perf_${Date.now()}`
        })
        successCount++
      } catch (error) {
        errorCount++
      }
      
      if (i < perfEventCount.value - 1) {
        await new Promise(resolve => setTimeout(resolve, perfInterval.value))
      }
    }
    
    const endTime = Date.now()
    const totalTime = endTime - startTime
    const eventsPerSecond = Math.round((perfEventCount.value / totalTime) * 1000)
    const successRate = Math.round((successCount / perfEventCount.value) * 100)
    
    perfResults.value = {
      totalTime,
      eventsPerSecond,
      successRate,
      successCount,
      errorCount
    }
    
    eventsSent.value += successCount
    addLog('success', 'perf_test', `Performance test completed: ${successCount}/${perfEventCount.value} events sent`)
    
  } catch (error) {
    addLog('error', 'perf_test', `Performance test failed: ${error.message}`)
  } finally {
    isRunningPerfTest.value = false
  }
}

// Update queue size periodically
let queueUpdateInterval = null

onMounted(() => {
  addLog('info', 'system', 'Analytics test page loaded')
  
  // Update queue size every second
  queueUpdateInterval = setInterval(() => {
    if (client.value && client.value.queue) {
      queueSize.value = client.value.queue.length
    }
  }, 1000)
})

onUnmounted(() => {
  if (queueUpdateInterval) {
    clearInterval(queueUpdateInterval)
  }
})
</script>

<style scoped>
.analytics-test-container {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
  background: #f8f9fa;
  min-height: 100vh;
}

.test-header {
  text-align: center;
  margin-bottom: 30px;
}

.test-header h1 {
  color: #2d3748;
  margin-bottom: 10px;
}

.test-header p {
  color: #718096;
  font-size: 16px;
}

.test-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 20px;
}

.test-card {
  background: white;
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  border: 1px solid #e2e8f0;
}

.test-card.full-width {
  grid-column: 1 / -1;
}

.test-card h3 {
  margin: 0 0 15px 0;
  color: #2d3748;
  font-size: 18px;
}

.status-grid {
  display: grid;
  gap: 10px;
}

.status-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid #f0f0f0;
}

.status-item:last-child {
  border-bottom: none;
}

.label {
  font-weight: 500;
  color: #4a5568;
}

.value {
  font-weight: 600;
}

.value.success {
  color: #38a169;
}

.value.error {
  color: #e53e3e;
}

.action-buttons {
  display: grid;
  gap: 10px;
}

.btn {
  padding: 10px 16px;
  border: none;
  border-radius: 8px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-primary {
  background: #4299e1;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #3182ce;
}

.btn-secondary {
  background: #718096;
  color: white;
}

.btn-secondary:hover:not(:disabled) {
  background: #4a5568;
}

.btn-accent {
  background: #9f7aea;
  color: white;
}

.btn-accent:hover:not(:disabled) {
  background: #805ad5;
}

.btn-warning {
  background: #ed8936;
  color: white;
}

.btn-warning:hover:not(:disabled) {
  background: #dd6b20;
}

.btn-danger {
  background: #f56565;
  color: white;
}

.btn-danger:hover:not(:disabled) {
  background: #e53e3e;
}

.btn-small {
  padding: 6px 12px;
  font-size: 14px;
}

.event-type-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 10px;
}

.event-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 15px;
  border: 2px solid #e2e8f0;
  border-radius: 10px;
  background: white;
  cursor: pointer;
  transition: all 0.2s ease;
}

.event-btn:hover {
  border-color: #4299e1;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.event-btn .icon {
  font-size: 24px;
}

.event-btn .name {
  font-weight: 500;
  color: #2d3748;
}

.event-btn.system:hover {
  border-color: #38a169;
}

.event-btn.auth:hover {
  border-color: #3182ce;
}

.event-btn.chat:hover {
  border-color: #805ad5;
}

.event-btn.navigation:hover {
  border-color: #dd6b20;
}

.event-btn.file:hover {
  border-color: #38b2ac;
}

.event-btn.search:hover {
  border-color: #ed8936;
}

.event-btn.error:hover {
  border-color: #e53e3e;
}

.event-btn.ai:hover {
  border-color: #9f7aea;
}

.perf-controls {
  display: grid;
  gap: 15px;
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.input-group label {
  font-weight: 500;
  color: #4a5568;
}

.input-group input {
  padding: 8px 12px;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  font-size: 14px;
}

.perf-results {
  margin-top: 15px;
  padding: 15px;
  background: #f7fafc;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
}

.perf-results h4 {
  margin: 0 0 10px 0;
  color: #2d3748;
}

.result-item {
  display: flex;
  justify-content: space-between;
  padding: 5px 0;
  border-bottom: 1px solid #e2e8f0;
}

.result-item:last-child {
  border-bottom: none;
}

.log-controls {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: #4a5568;
  cursor: pointer;
}

.event-log {
  max-height: 400px;
  overflow-y: auto;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  background: #f8f9fa;
}

.log-entry {
  display: grid;
  grid-template-columns: auto auto 1fr auto;
  gap: 10px;
  padding: 8px 12px;
  border-bottom: 1px solid #e2e8f0;
  font-size: 12px;
  font-family: 'Monaco', 'Menlo', monospace;
}

.log-entry:last-child {
  border-bottom: none;
}

.log-entry.success {
  background: #f0fff4;
  border-left: 3px solid #38a169;
}

.log-entry.error {
  background: #fef5e7;
  border-left: 3px solid #e53e3e;
}

.log-entry.info {
  background: #ebf8ff;
  border-left: 3px solid #4299e1;
}

.timestamp {
  color: #718096;
  font-weight: 500;
}

.event-type {
  color: #4a5568;
  font-weight: 600;
  background: #edf2f7;
  padding: 2px 6px;
  border-radius: 4px;
}

.message {
  color: #2d3748;
}

.data {
  color: #718096;
  font-size: 10px;
  white-space: pre-wrap;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style> 