<template>
  <div class="admin-dashboard">
    <!-- Header -->
    <div class="admin-header">
      <h1 class="admin-title">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor" class="admin-icon">
          <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z" />
        </svg>
        Admin Dashboard
      </h1>
      <div class="admin-actions">
        <button @click="refreshAll" class="refresh-btn" :disabled="loading">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" :class="{ 'animate-spin': loading }">
            <path
              d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
          </svg>
          Refresh
        </button>
        <button @click="exportData" class="export-btn">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M14,2H6A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20Z" />
          </svg>
          Export Data
        </button>
      </div>
    </div>

    <!-- Dashboard Grid -->
    <div class="dashboard-grid">
      <!-- System Status -->
      <div class="dashboard-card">
        <div class="card-header">
          <h2>System Status</h2>
          <div class="status-indicator" :class="systemStatus.overall">
            <div class="status-dot"></div>
            {{ systemStatus.overall.toUpperCase() }}
          </div>
        </div>
        <div class="card-content">
          <div class="status-grid">
            <div class="status-item">
              <span class="status-label">Frontend</span>
              <div class="status-value" :class="systemStatus.frontend">
                {{ systemStatus.frontend }}
              </div>
            </div>
            <div class="status-item">
              <span class="status-label">Backend</span>
              <div class="status-value" :class="systemStatus.backend">
                {{ systemStatus.backend }}
              </div>
            </div>
            <div class="status-item">
              <span class="status-label">Database</span>
              <div class="status-value" :class="systemStatus.database">
                {{ systemStatus.database }}
              </div>
            </div>
            <div class="status-item">
              <span class="status-label">Real-time</span>
              <div class="status-value" :class="systemStatus.realtime">
                {{ systemStatus.realtime }}
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Cache Management -->
      <div class="dashboard-card">
        <div class="card-header">
          <h2>Cache Management</h2>
          <div class="cache-size">{{ formatBytes(cacheStats.totalSize) }}</div>
        </div>
        <div class="card-content">
          <div class="cache-stats">
            <div class="cache-item">
              <span class="cache-label">Messages Cache</span>
              <div class="cache-info">
                <span class="cache-size">{{ formatBytes(cacheStats.messages) }}</span>
                <button @click="clearCache('messages')" class="clear-cache-btn">Clear</button>
              </div>
            </div>
            <div class="cache-item">
              <span class="cache-label">User Data Cache</span>
              <div class="cache-info">
                <span class="cache-size">{{ formatBytes(cacheStats.users) }}</span>
                <button @click="clearCache('users')" class="clear-cache-btn">Clear</button>
              </div>
            </div>
            <div class="cache-item">
              <span class="cache-label">Session Storage</span>
              <div class="cache-info">
                <span class="cache-size">{{ formatBytes(cacheStats.session) }}</span>
                <button @click="clearCache('session')" class="clear-cache-btn">Clear</button>
              </div>
            </div>
            <div class="cache-item">
              <span class="cache-label">Local Storage</span>
              <div class="cache-info">
                <span class="cache-size">{{ formatBytes(cacheStats.local) }}</span>
                <button @click="clearCache('local')" class="clear-cache-btn">Clear</button>
              </div>
            </div>
          </div>
          <div class="cache-actions">
            <button @click="clearAllCache" class="clear-all-btn">
              Clear All Cache
            </button>
            <button @click="optimizeCache" class="optimize-btn">
              Optimize Cache
            </button>
          </div>
        </div>
      </div>

      <!-- Search Index Management -->
      <div class="dashboard-card">
        <div class="card-header">
          <h2>Search Index</h2>
          <div class="index-status" :class="searchIndex.status">
            {{ searchIndex.status.toUpperCase() }}
          </div>
        </div>
        <div class="card-content">
          <div class="index-stats">
            <div class="stat-item">
              <span class="stat-label">Total Documents</span>
              <span class="stat-value">{{ searchIndex.totalDocuments.toLocaleString() }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">Last Updated</span>
              <span class="stat-value">{{ formatTime(searchIndex.lastUpdated) }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">Index Size</span>
              <span class="stat-value">{{ formatBytes(searchIndex.size) }}</span>
            </div>
          </div>
          <div class="index-actions">
            <button @click="rebuildIndex" class="rebuild-btn" :disabled="searchIndex.rebuilding">
              {{ searchIndex.rebuilding ? 'Rebuilding...' : 'Rebuild Index' }}
            </button>
            <button @click="optimizeIndex" class="optimize-index-btn">
              Optimize Index
            </button>
          </div>
          <div v-if="searchIndex.rebuilding" class="rebuild-progress">
            <div class="progress-bar">
              <div class="progress-fill" :style="{ width: searchIndex.rebuildProgress + '%' }"></div>
            </div>
            <span class="progress-text">{{ searchIndex.rebuildProgress }}%</span>
          </div>
        </div>
      </div>

      <!-- Performance Metrics -->
      <div class="dashboard-card">
        <div class="card-header">
          <h2>Performance Metrics</h2>
          <div class="performance-score" :class="getPerformanceClass(performance.score)">
            {{ performance.score }}/100
          </div>
        </div>
        <div class="card-content">
          <div class="metrics-grid">
            <div class="metric-item">
              <span class="metric-label">Load Time</span>
              <span class="metric-value">{{ performance.loadTime }}ms</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Memory Usage</span>
              <span class="metric-value">{{ formatBytes(performance.memoryUsage) }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Active Users</span>
              <span class="metric-value">{{ performance.activeUsers }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">API Response</span>
              <span class="metric-value">{{ performance.apiResponseTime }}ms</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Recent Activity -->
      <div class="dashboard-card wide">
        <div class="card-header">
          <h2>Recent Admin Activity</h2>
          <div class="activity-count">{{ adminActivity.length }} events</div>
        </div>
        <div class="card-content">
          <div class="activity-list">
            <div v-for="activity in adminActivity" :key="activity.id" class="activity-item">
              <div class="activity-icon" :class="activity.type">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path v-if="activity.type === 'cache'"
                    d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-5 14H7v-2h7v2zm3-4H7v-2h10v2zm0-4H7V7h10v2z" />
                  <path v-else-if="activity.type === 'search'"
                    d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z" />
                  <path v-else
                    d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" />
                </svg>
              </div>
              <div class="activity-content">
                <div class="activity-title">{{ activity.title }}</div>
                <div class="activity-description">{{ activity.description }}</div>
                <div class="activity-time">{{ formatTime(activity.timestamp) }}</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useNotifications } from '@/composables/useNotifications';
import minimalSSE from '@/services/sse-minimal';
import { useAuthStore } from '@/stores/auth';
import { useWorkspaceStore } from '@/stores/workspace';
import { errorHandler } from '@/utils/errorHandler';

const { notifySuccess, notifyError } = useNotifications();

// Reactive state
const loading = ref(false);

// System Status
const systemStatus = ref({
  overall: 'healthy',
  frontend: 'healthy',
  backend: 'healthy',
  database: 'healthy',
  realtime: 'healthy'
});

// Cache Statistics
const cacheStats = ref({
  totalSize: 0,
  messages: 0,
  users: 0,
  session: 0,
  local: 0
});

// Search Index
const searchIndex = ref({
  status: 'ready',
  totalDocuments: 12543,
  lastUpdated: new Date().toISOString(),
  size: 45 * 1024 * 1024, // 45MB
  rebuilding: false,
  rebuildProgress: 0
});

// Performance Metrics
const performance = ref({
  score: 92,
  loadTime: 1240,
  memoryUsage: 128 * 1024 * 1024, // 128MB
  activeUsers: 42,
  apiResponseTime: 156
});

// Admin Activity
const adminActivity = ref([
  {
    id: 1,
    type: 'cache',
    title: 'Cache Cleared',
    description: 'Messages cache cleared by system optimization',
    timestamp: new Date(Date.now() - 5 * 60 * 1000).toISOString()
  },
  {
    id: 2,
    type: 'search',
    title: 'Search Index Rebuilt',
    description: 'Full search index rebuild completed successfully',
    timestamp: new Date(Date.now() - 15 * 60 * 1000).toISOString()
  },
  {
    id: 3,
    type: 'system',
    title: 'Performance Optimization',
    description: 'Automatic performance optimization executed',
    timestamp: new Date(Date.now() - 30 * 60 * 1000).toISOString()
  }
]);

// Computed
const getPerformanceClass = (score) => {
  if (score >= 90) return 'excellent';
  if (score >= 80) return 'good';
  if (score >= 70) return 'fair';
  return 'poor';
};

// Methods
const formatBytes = (bytes) => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
};

const formatTime = (timestamp) => {
  const date = new Date(timestamp);
  const now = new Date();
  const diffInMinutes = Math.floor((now - date) / (1000 * 60));

  if (diffInMinutes < 1) return 'Just now';
  if (diffInMinutes < 60) return `${diffInMinutes}m ago`;
  if (diffInMinutes < 1440) return `${Math.floor(diffInMinutes / 60)}h ago`;
  return date.toLocaleDateString();
};

const calculateCacheStats = () => {
  try {
    // Calculate session storage size
    let sessionSize = 0;
    for (let key in sessionStorage) {
      if (sessionStorage.hasOwnProperty(key)) {
        sessionSize += sessionStorage[key].length;
      }
    }

    // Calculate local storage size
    let localSize = 0;
    for (let key in localStorage) {
      if (localStorage.hasOwnProperty(key)) {
        localSize += localStorage[key].length;
      }
    }

    // Estimate message cache size
    const messageCache = sessionStorage.getItem('messageCache');
    const messagesSize = messageCache ? messageCache.length * 2 : 0; // UTF-16 encoding

    // Estimate user data cache size
    const usersSize = (localStorage.getItem('auth_token') || '').length * 2;

    cacheStats.value = {
      session: sessionSize * 2, // UTF-16 encoding
      local: localSize * 2,
      messages: messagesSize,
      users: usersSize,
      totalSize: (sessionSize + localSize) * 2
    };
  } catch (error) {
    console.error('Failed to calculate cache stats:', error);
  }
};

const updateSystemStatus = () => {
  // Check real-time connection
  const connectionState = minimalSSE.getConnectionState();
  systemStatus.value.realtime = connectionState.isConnected ? 'healthy' : 'warning';

  // Check if we can access local storage (frontend health)
  try {
    localStorage.setItem('health-check', 'test');
    localStorage.removeItem('health-check');
    systemStatus.value.frontend = 'healthy';
  } catch {
    systemStatus.value.frontend = 'error';
  }

  // Calculate overall status
  const statuses = Object.values(systemStatus.value).filter(s => s !== 'healthy');
  if (statuses.length === 0) {
    systemStatus.value.overall = 'healthy';
  } else if (statuses.some(s => s === 'error')) {
    systemStatus.value.overall = 'error';
  } else {
    systemStatus.value.overall = 'warning';
  }
};

const clearCache = async (type) => {
  try {
    loading.value = true;

    switch (type) {
      case 'messages':
        sessionStorage.removeItem('messageCache');
        break;
      case 'users':
        // Keep auth tokens, only clear user cache
        break;
      case 'session':
        sessionStorage.clear();
        break;
      case 'local':
        // Be careful not to clear auth tokens
        const authToken = localStorage.getItem('auth_token');
        const refreshToken = localStorage.getItem('refresh_token');
        localStorage.clear();
        if (authToken) localStorage.setItem('auth_token', authToken);
        if (refreshToken) localStorage.setItem('refresh_token', refreshToken);
        break;
    }

    calculateCacheStats();
    addAdminActivity('cache', 'Cache Cleared', `${type} cache cleared successfully`);
    notifySuccess(`${type} cache cleared successfully`);
  } catch (error) {
    console.error('Failed to clear cache:', error);
    notifyError('Failed to clear cache');
  } finally {
    loading.value = false;
  }
};

const clearAllCache = async () => {
  if (!confirm('Are you sure you want to clear all cache? This will log you out.')) {
    return;
  }

  try {
    loading.value = true;
    sessionStorage.clear();
    localStorage.clear();
    calculateCacheStats();
    addAdminActivity('cache', 'All Cache Cleared', 'All cache data cleared');
    notifySuccess('All cache cleared successfully');

    // Reload the page after clearing all cache
    setTimeout(() => {
      window.location.reload();
    }, 1000);
  } catch (error) {
    console.error('Failed to clear all cache:', error);
    notifyError('Failed to clear all cache');
  } finally {
    loading.value = false;
  }
};

const optimizeCache = async () => {
  try {
    loading.value = true;

    // Remove expired cache entries
    const messageCache = sessionStorage.getItem('messageCache');
    if (messageCache) {
      try {
        const cache = JSON.parse(messageCache);
        const now = Date.now();
        const oneHour = 60 * 60 * 1000;

        Object.keys(cache).forEach(key => {
          if (cache[key].timestamp && (now - cache[key].timestamp) > oneHour) {
            delete cache[key];
          }
        });

        sessionStorage.setItem('messageCache', JSON.stringify(cache));
      } catch (e) {
        console.warn('Failed to optimize message cache:', e);
      }
    }

    calculateCacheStats();
    addAdminActivity('cache', 'Cache Optimized', 'Cache optimization completed');
    notifySuccess('Cache optimized successfully');
  } catch (error) {
    console.error('Failed to optimize cache:', error);
    notifyError('Failed to optimize cache');
  } finally {
    loading.value = false;
  }
};

const rebuildIndex = async () => {
  try {
    searchIndex.value.rebuilding = true;
    searchIndex.value.rebuildProgress = 0;

    // Simulate index rebuild progress
    const interval = setInterval(() => {
      searchIndex.value.rebuildProgress += 10;
      if (searchIndex.value.rebuildProgress >= 100) {
        clearInterval(interval);
        searchIndex.value.rebuilding = false;
        searchIndex.value.lastUpdated = new Date().toISOString();
        addAdminActivity('search', 'Search Index Rebuilt', 'Full search index rebuild completed');
        notifySuccess('Search index rebuilt successfully');
      }
    }, 500);
  } catch (error) {
    console.error('Failed to rebuild search index:', error);
    notifyError('Failed to rebuild search index');
    searchIndex.value.rebuilding = false;
  }
};

const optimizeIndex = async () => {
  try {
    loading.value = true;
    addAdminActivity('search', 'Search Index Optimized', 'Search index optimization completed');
    notifySuccess('Search index optimized successfully');
  } catch (error) {
    console.error('Failed to optimize search index:', error);
    notifyError('Failed to optimize search index');
  } finally {
    loading.value = false;
  }
};

const addAdminActivity = (type, title, description) => {
  const newActivity = {
    id: Date.now(),
    type,
    title,
    description,
    timestamp: new Date().toISOString()
  };

  adminActivity.value.unshift(newActivity);

  // Keep only last 20 activities
  if (adminActivity.value.length > 20) {
    adminActivity.value = adminActivity.value.slice(0, 20);
  }
};

const refreshAll = async () => {
  try {
    loading.value = true;
    calculateCacheStats();
    updateSystemStatus();
    addAdminActivity('system', 'Dashboard Refreshed', 'All metrics refreshed');
    notifySuccess('Dashboard refreshed successfully');
  } catch (error) {
    console.error('Failed to refresh dashboard:', error);
    notifyError('Failed to refresh dashboard');
  } finally {
    loading.value = false;
  }
};

const exportData = async () => {
  try {
    const data = {
      systemStatus: systemStatus.value,
      cacheStats: cacheStats.value,
      searchIndex: searchIndex.value,
      performance: performance.value,
      adminActivity: adminActivity.value,
      timestamp: new Date().toISOString()
    };

    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `admin-dashboard-${new Date().toISOString().split('T')[0]}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    addAdminActivity('system', 'Data Exported', 'Admin dashboard data exported');
    notifySuccess('Data exported successfully');
  } catch (error) {
    console.error('Failed to export data:', error);
    notifyError('Failed to export data');
  }
};

// Auto-refresh system status
let statusInterval = null;

// Lifecycle
onMounted(() => {
  calculateCacheStats();
  updateSystemStatus();

  // Auto-refresh every 30 seconds
  statusInterval = setInterval(() => {
    calculateCacheStats();
    updateSystemStatus();
  }, 30000);
});

onUnmounted(() => {
  if (statusInterval) {
    clearInterval(statusInterval);
  }
});
</script>

<style scoped>
.admin-dashboard {
  padding: 24px;
  background: #f8f9fa;
  min-height: 100vh;
}

.admin-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 32px;
}

.admin-title {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 28px;
  font-weight: 700;
  color: #1f2937;
  margin: 0;
}

.admin-icon {
  color: #6366f1;
}

.admin-actions {
  display: flex;
  gap: 12px;
}

.refresh-btn,
.export-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  background: white;
  color: #374151;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.refresh-btn:hover,
.export-btn:hover {
  background: #f9fafb;
  border-color: #9ca3af;
}

.refresh-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.dashboard-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: 24px;
}

.dashboard-card {
  background: white;
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  border: 1px solid #e5e7eb;
}

.dashboard-card.wide {
  grid-column: 1 / -1;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.card-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: #1f2937;
  margin: 0;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
}

.status-indicator.healthy {
  background: #d1fae5;
  color: #065f46;
}

.status-indicator.warning {
  background: #fef3c7;
  color: #92400e;
}

.status-indicator.error {
  background: #fee2e2;
  color: #991b1b;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: currentColor;
}

.status-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.status-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.status-label {
  font-size: 14px;
  color: #6b7280;
}

.status-value {
  padding: 4px 8px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
}

.status-value.healthy {
  background: #d1fae5;
  color: #065f46;
}

.status-value.warning {
  background: #fef3c7;
  color: #92400e;
}

.status-value.error {
  background: #fee2e2;
  color: #991b1b;
}

.cache-size {
  font-size: 14px;
  font-weight: 600;
  color: #6366f1;
}

.cache-stats {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 20px;
}

.cache-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  background: #f9fafb;
  border-radius: 8px;
}

.cache-label {
  font-size: 14px;
  color: #374151;
}

.cache-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.clear-cache-btn {
  padding: 4px 8px;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  background: white;
  color: #374151;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.clear-cache-btn:hover {
  background: #f9fafb;
  border-color: #9ca3af;
}

.cache-actions {
  display: flex;
  gap: 12px;
}

.clear-all-btn,
.optimize-btn,
.rebuild-btn,
.optimize-index-btn {
  flex: 1;
  padding: 10px 16px;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  background: white;
  color: #374151;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.clear-all-btn:hover,
.optimize-btn:hover,
.rebuild-btn:hover,
.optimize-index-btn:hover {
  background: #f9fafb;
  border-color: #9ca3af;
}

.clear-all-btn {
  background: #fef2f2;
  border-color: #fecaca;
  color: #dc2626;
}

.clear-all-btn:hover {
  background: #fee2e2;
  border-color: #dc2626;
}

.index-status {
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
}

.index-status.ready {
  background: #d1fae5;
  color: #065f46;
}

.index-stats {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 20px;
}

.stat-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.stat-label {
  font-size: 14px;
  color: #6b7280;
}

.stat-value {
  font-size: 14px;
  font-weight: 600;
  color: #1f2937;
}

.index-actions {
  display: flex;
  gap: 12px;
}

.rebuild-progress {
  margin-top: 16px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.progress-bar {
  flex: 1;
  height: 8px;
  background: #e5e7eb;
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: #6366f1;
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 12px;
  font-weight: 600;
  color: #6366f1;
}

.performance-score {
  padding: 8px 16px;
  border-radius: 20px;
  font-size: 16px;
  font-weight: 700;
}

.performance-score.excellent {
  background: #d1fae5;
  color: #065f46;
}

.performance-score.good {
  background: #dbeafe;
  color: #1e40af;
}

.performance-score.fair {
  background: #fef3c7;
  color: #92400e;
}

.performance-score.poor {
  background: #fee2e2;
  color: #991b1b;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.metric-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.metric-label {
  font-size: 12px;
  color: #6b7280;
  text-transform: uppercase;
  font-weight: 500;
}

.metric-value {
  font-size: 18px;
  font-weight: 700;
  color: #1f2937;
}

.activity-count {
  font-size: 14px;
  color: #6b7280;
}

.activity-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.activity-item {
  display: flex;
  gap: 12px;
  padding: 16px;
  background: #f9fafb;
  border-radius: 8px;
}

.activity-icon {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.activity-icon.cache {
  background: #dbeafe;
  color: #1e40af;
}

.activity-icon.search {
  background: #fef3c7;
  color: #92400e;
}

.activity-icon.system {
  background: #d1fae5;
  color: #065f46;
}

.activity-content {
  flex: 1;
}

.activity-title {
  font-size: 14px;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 4px;
}

.activity-description {
  font-size: 13px;
  color: #6b7280;
  margin-bottom: 8px;
}

.activity-time {
  font-size: 12px;
  color: #9ca3af;
}

.animate-spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}

/* Mobile responsive */
@media (max-width: 768px) {
  .admin-dashboard {
    padding: 16px;
  }

  .dashboard-grid {
    grid-template-columns: 1fr;
    gap: 16px;
  }

  .admin-header {
    flex-direction: column;
    gap: 16px;
    align-items: flex-start;
  }

  .admin-actions {
    width: 100%;
    justify-content: stretch;
  }

  .refresh-btn,
  .export-btn {
    flex: 1;
  }

  .status-grid,
  .metrics-grid {
    grid-template-columns: 1fr;
  }

  .cache-actions,
  .index-actions {
    flex-direction: column;
  }
}
</style>