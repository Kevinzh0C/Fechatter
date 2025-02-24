<template>
  <div v-if="showMonitor && isDev" class="performance-monitor">
    <div class="monitor-header" @click="expanded = !expanded">
      <span class="monitor-icon">⚡</span>
      <span>Channel Performance</span>
      <span class="toggle-icon">{{ expanded ? '▼' : '▶' }}</span>
    </div>

    <div v-if="expanded" class="monitor-content">
      <div class="stat-row">
        <span class="stat-label">Last Switch:</span>
        <span class="stat-value" :class="getSpeedClass(lastSwitchTime)">
          {{ lastSwitchTime }}ms
        </span>
      </div>

      <div class="stat-row">
        <span class="stat-label">Cached Channels:</span>
        <span class="stat-value">{{ stats.cachedChannels }}</span>
      </div>

      <div class="stat-row">
        <span class="stat-label">Preloaded:</span>
        <span class="stat-value">{{ stats.activePreloads }}</span>
      </div>

      <div class="stat-row">
        <span class="stat-label">Top Channels:</span>
      </div>
      <div class="top-channels">
        <div v-for="chatId in stats.topChannels" :key="chatId" class="channel-item">
          #{{ getChatName(chatId) }}
        </div>
      </div>

      <button @click="clearCache" class="clear-cache-btn">
        Clear Cache
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import channelOptimizer from '@/utils/channelOptimizer';
import { useChatStore } from '@/stores/chat';

const chatStore = useChatStore();
const isDev = import.meta.env.DEV;

const showMonitor = ref(true);
const expanded = ref(false);
const lastSwitchTime = ref(0);
const stats = ref({
  cachedChannels: 0,
  cachedMembers: 0,
  activePreloads: 0,
  topChannels: []
});

let updateInterval = null;

const getSpeedClass = (time) => {
  if (time < 100) return 'speed-fast';
  if (time < 300) return 'speed-good';
  if (time < 500) return 'speed-ok';
  return 'speed-slow';
};

const getChatName = (chatId) => {
  const chat = chatStore.getChatById(chatId);
  return chat?.name || chatId;
};

const updateStats = () => {
  stats.value = channelOptimizer.getStats();
};

const clearCache = () => {
  channelOptimizer.reset();
  updateStats();
};

// Listen for channel switch events
const originalSwitch = channelOptimizer.switchChannel.bind(channelOptimizer);
channelOptimizer.switchChannel = async function (...args) {
  const start = performance.now();
  const result = await originalSwitch(...args);
  lastSwitchTime.value = Math.round(performance.now() - start);
  updateStats();
  return result;
};

onMounted(() => {
  updateStats();
  updateInterval = setInterval(updateStats, 10000);
});

onUnmounted(() => {
  if (updateInterval) {
    clearInterval(updateInterval);
  }
});
</script>

<style scoped>
.performance-monitor {
  position: fixed;
  bottom: 20px;
  right: 20px;
  background: rgba(0, 0, 0, 0.9);
  color: white;
  border-radius: 8px;
  font-size: 12px;
  font-family: monospace;
  z-index: 9999;
  min-width: 200px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.monitor-header {
  padding: 8px 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  user-select: none;
}

.monitor-icon {
  font-size: 14px;
}

.toggle-icon {
  margin-left: auto;
  font-size: 10px;
}

.monitor-content {
  padding: 12px;
}

.stat-row {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
  align-items: center;
}

.stat-label {
  color: rgba(255, 255, 255, 0.7);
}

.stat-value {
  font-weight: bold;
}

.speed-fast {
  color: #10b981;
}

.speed-good {
  color: #3b82f6;
}

.speed-ok {
  color: #f59e0b;
}

.speed-slow {
  color: #ef4444;
}

.top-channels {
  margin-top: 4px;
  margin-bottom: 12px;
}

.channel-item {
  padding: 4px 8px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  margin-bottom: 4px;
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.clear-cache-btn {
  width: 100%;
  padding: 6px;
  background: rgba(239, 68, 68, 0.2);
  border: 1px solid rgba(239, 68, 68, 0.5);
  color: #fca5a5;
  border-radius: 4px;
  cursor: pointer;
  font-size: 11px;
  transition: all 0.2s;
}

.clear-cache-btn:hover {
  background: rgba(239, 68, 68, 0.3);
  border-color: #ef4444;
  color: white;
}
</style>