<template>
  <div class="search-integration">
    <div class="search-header">
      <h1 class="title">Production Search System</h1>
      <p class="subtitle">Complete search functionality with backend API integration</p>
    </div>

    <div class="search-demo">
      <div class="demo-controls">
        <button 
          class="demo-button primary"
          @click="openChatSearch"
        >
          Open Chat Search
        </button>
        <button 
          class="demo-button secondary"
          @click="testSearchAPI"
          :disabled="isTestingAPI"
        >
          {{ isTestingAPI ? 'Testing...' : 'Test Search API' }}
        </button>
        <button 
          class="demo-button"
          @click="clearMetrics"
        >
          Clear Metrics
        </button>
      </div>

      <div class="demo-info" v-if="currentChat">
        <h3>Current Chat Context</h3>
        <p><strong>Chat ID:</strong> {{ currentChat.id }}</p>
        <p><strong>Chat Name:</strong> {{ currentChat.name }}</p>
        <p><strong>Members:</strong> {{ currentChat.memberCount || 0 }}</p>
      </div>

      <div class="api-status">
        <h3>API Status</h3>
        <div class="status-grid">
          <div class="status-item">
            <span class="status-label">Search Service:</span>
            <span class="status-value" :class="apiStatus.searchService">
              {{ apiStatus.searchService }}
            </span>
          </div>
          <div class="status-item">
            <span class="status-label">Backend API:</span>
            <span class="status-value" :class="apiStatus.backend">
              {{ apiStatus.backend }}
            </span>
          </div>
          <div class="status-item">
            <span class="status-label">Cache Status:</span>
            <span class="status-value">
              {{ metrics.cacheSize }} entries
            </span>
          </div>
        </div>
      </div>

      <div class="metrics" v-if="metrics.searchCount > 0">
        <h3>Performance Metrics</h3>
        <div class="metrics-grid">
          <div class="metric-card">
            <div class="metric-value">{{ metrics.searchCount }}</div>
            <div class="metric-label">Total Searches</div>
          </div>
          <div class="metric-card">
            <div class="metric-value">{{ Math.round(metrics.avgResponseTime) }}ms</div>
            <div class="metric-label">Avg Response Time</div>
          </div>
          <div class="metric-card">
            <div class="metric-value">{{ Math.round(metrics.cacheHitRate * 100) }}%</div>
            <div class="metric-label">Cache Hit Rate</div>
          </div>
          <div class="metric-card">
            <div class="metric-value">{{ Math.round(metrics.errorRate * 100) }}%</div>
            <div class="metric-label">Error Rate</div>
          </div>
        </div>
      </div>

      <div class="test-results" v-if="testResults.length > 0">
        <h3>API Test Results</h3>
        <div class="results-list">
          <div 
            v-for="(result, index) in testResults"
            :key="index"
            class="result-item"
            :class="result.status.toLowerCase()"
          >
            <div class="result-status">{{ result.status }}</div>
            <div class="result-test">{{ result.test }}</div>
            <div class="result-details" v-if="result.details">{{ result.details }}</div>
            <div class="result-error" v-if="result.error">{{ result.error }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Search Modal -->
    <ProductionSearchModal
      v-model="showSearchModal"
      :chat-id="currentChat?.id"
      :chat-name="currentChat?.name"
      @navigate-to-message="handleNavigateToMessage"
      @close="handleSearchClose"
    />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useChatStore } from '@/stores/chat';
import ProductionSearchModal from './ProductionSearchModal.vue';
import searchService from '@/services/searchService.js';

// State
const showSearchModal = ref(false);
const isTestingAPI = ref(false);
const testResults = ref([]);
const metrics = ref({
  searchCount: 0,
  avgResponseTime: 0,
  cacheHitRate: 0,
  errorRate: 0,
  cacheSize: 0,
  activeRequests: 0
});

const apiStatus = ref({
  searchService: 'unknown',
  backend: 'unknown'
});

// Stores
const chatStore = useChatStore();

// Computed
const currentChat = computed(() => {
  // Get current chat from store or create mock data for testing
  return chatStore.currentChat || {
    id: 1,
    name: '日本市場チャンネル',
    memberCount: 5
  };
});

// Methods
function openChatSearch() {
  showSearchModal.value = true;
}

function handleNavigateToMessage(result) {
  console.log('Navigate to message:', result);
  // Here you would implement actual navigation
}

function handleSearchClose() {
  updateMetrics();
}

function updateMetrics() {
  metrics.value = searchService.getMetrics();
}

async function testSearchAPI() {
  isTestingAPI.value = true;
  testResults.value = [];
  
  const tests = [
    {
      name: 'Search Service Initialization',
      test: () => searchService !== null
    },
    {
      name: 'Chat Search API',
      test: async () => {
        if (!currentChat.value?.id) {
          throw new Error('No current chat available');
        }
        const result = await searchService.searchInChat({
          chatId: currentChat.value.id,
          query: 'test',
          limit: 5
        });
        return result.hits !== undefined;
      }
    },
    {
      name: 'Global Message Search API',
      test: async () => {
        const result = await searchService.searchGlobalMessages({
          query: 'test',
          limit: 5
        });
        return result.hits !== undefined;
      }
    },
    {
      name: 'Search Suggestions API',
      test: async () => {
        const suggestions = await searchService.getSearchSuggestions('test', 3);
        return Array.isArray(suggestions);
      }
    },
    {
      name: 'Cache Functionality',
      test: () => {
        const cacheKey = searchService.generateCacheKey('test', { query: 'test' });
        searchService.setCache(cacheKey, { test: 'data' });
        const cached = searchService.getFromCache(cacheKey);
        return cached !== null;
      }
    }
  ];

  for (const testCase of tests) {
    try {
      const startTime = performance.now();
      const result = await testCase.test();
      const endTime = performance.now();
      
      testResults.value.push({
        test: testCase.name,
        status: result ? 'PASS' : 'FAIL',
        details: `Completed in ${Math.round(endTime - startTime)}ms`,
        error: null
      });
      
      // Update API status
      if (testCase.name.includes('API')) {
        apiStatus.value.backend = 'connected';
      }
      
    } catch (error) {
      testResults.value.push({
        test: testCase.name,
        status: 'ERROR',
        details: null,
        error: error.message
      });
      
      // Update API status on error
      if (testCase.name.includes('API')) {
        apiStatus.value.backend = 'error';
      }
    }
  }

  // Mark search service as initialized
  apiStatus.value.searchService = 'initialized';
  isTestingAPI.value = false;
  updateMetrics();
}

function clearMetrics() {
  searchService.clearCache();
  metrics.value = {
    searchCount: 0,
    avgResponseTime: 0,
    cacheHitRate: 0,
    errorRate: 0,
    cacheSize: 0,
    activeRequests: 0
  };
  testResults.value = [];
}

// Lifecycle
onMounted(() => {
  updateMetrics();
  
  // Set up metric update interval
  const metricsInterval = setInterval(updateMetrics, 5000);
  
  // Cleanup on unmount
  onUnmounted(() => {
    clearInterval(metricsInterval);
  });
});
</script>

<style scoped>
.search-integration {
  max-width: 1200px;
  margin: 0 auto;
  padding: 24px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

.search-header {
  text-align: center;
  margin-bottom: 40px;
}

.title {
  font-size: 32px;
  font-weight: 700;
  color: #1f2937;
  margin: 0 0 8px 0;
}

.subtitle {
  font-size: 18px;
  color: #6b7280;
  margin: 0;
}

.search-demo {
  display: grid;
  gap: 24px;
}

.demo-controls {
  display: flex;
  gap: 12px;
  justify-content: center;
  flex-wrap: wrap;
}

.demo-button {
  padding: 12px 24px;
  border: 2px solid #e5e7eb;
  border-radius: 8px;
  background: white;
  color: #374151;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.demo-button:hover:not(:disabled) {
  border-color: #d1d5db;
  background: #f9fafb;
}

.demo-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.demo-button.primary {
  background: #3b82f6;
  color: white;
  border-color: #3b82f6;
}

.demo-button.primary:hover:not(:disabled) {
  background: #2563eb;
  border-color: #2563eb;
}

.demo-button.secondary {
  background: #10b981;
  color: white;
  border-color: #10b981;
}

.demo-button.secondary:hover:not(:disabled) {
  background: #059669;
  border-color: #059669;
}

.demo-info,
.api-status,
.metrics,
.test-results {
  background: white;
  border: 1px solid #e5e7eb;
  border-radius: 12px;
  padding: 20px;
}

.demo-info h3,
.api-status h3,
.metrics h3,
.test-results h3 {
  font-size: 18px;
  font-weight: 600;
  color: #1f2937;
  margin: 0 0 16px 0;
}

.demo-info p {
  margin: 8px 0;
  color: #374151;
}

.status-grid {
  display: grid;
  gap: 12px;
}

.status-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
}

.status-label {
  font-weight: 500;
  color: #374151;
}

.status-value {
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  text-transform: uppercase;
}

.status-value.connected,
.status-value.initialized {
  background: #d1fae5;
  color: #065f46;
}

.status-value.error {
  background: #fee2e2;
  color: #991b1b;
}

.status-value.unknown {
  background: #f3f4f6;
  color: #6b7280;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 16px;
}

.metric-card {
  text-align: center;
  padding: 16px;
  background: #f9fafb;
  border-radius: 8px;
}

.metric-value {
  font-size: 24px;
  font-weight: 700;
  color: #1f2937;
  margin-bottom: 4px;
}

.metric-label {
  font-size: 12px;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.results-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.result-item {
  display: grid;
  grid-template-columns: 80px 1fr;
  gap: 12px;
  padding: 12px;
  border-radius: 6px;
  align-items: start;
}

.result-item.pass {
  background: #f0fdf4;
  border: 1px solid #bbf7d0;
}

.result-item.fail {
  background: #fef2f2;
  border: 1px solid #fecaca;
}

.result-item.error {
  background: #fef2f2;
  border: 1px solid #fecaca;
}

.result-status {
  font-size: 12px;
  font-weight: 600;
  text-align: center;
  padding: 4px 8px;
  border-radius: 4px;
}

.result-item.pass .result-status {
  background: #16a34a;
  color: white;
}

.result-item.fail .result-status {
  background: #dc2626;
  color: white;
}

.result-item.error .result-status {
  background: #ea580c;
  color: white;
}

.result-test {
  font-weight: 500;
  color: #1f2937;
}

.result-details {
  font-size: 14px;
  color: #6b7280;
  margin-top: 4px;
}

.result-error {
  font-size: 14px;
  color: #dc2626;
  margin-top: 4px;
  font-family: monospace;
}

@media (max-width: 768px) {
  .search-integration {
    padding: 16px;
  }
  
  .title {
    font-size: 24px;
  }
  
  .subtitle {
    font-size: 16px;
  }
  
  .demo-controls {
    flex-direction: column;
  }
  
  .metrics-grid {
    grid-template-columns: repeat(2, 1fr);
  }
  
  .result-item {
    grid-template-columns: 1fr;
  }
}
</style> 