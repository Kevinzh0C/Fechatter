<template>
  <div v-if="!isOnline || !serverHealthy" 
    class="fixed top-0 left-0 right-0 z-50 bg-red-600 text-white py-2 px-4 text-sm flex items-center justify-between">
    <div class="flex items-center">
      <svg v-if="!isOnline" class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
          d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728L5.636 5.636m12.728 12.728L5.636 5.636"></path>
      </svg>
      <svg v-else class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
          d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
      </svg>
      <span v-if="!isOnline">You are offline</span>
      <span v-else-if="!serverHealthy">Server connection lost</span>
    </div>
    <button v-if="isOnline && !serverHealthy" 
      @click="checkServerHealth" 
      :disabled="checking"
      class="text-xs underline hover:no-underline disabled:opacity-50">
      {{ checking ? 'Checking...' : 'Retry' }}
    </button>
  </div>
  
  <!-- Success notification when connection is restored -->
  <div v-if="showReconnected" 
    class="fixed top-0 left-0 right-0 z-50 bg-green-600 text-white py-2 px-4 text-sm flex items-center">
    <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
    </svg>
    <span>Connection restored</span>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { networkStatus, healthCheck } from '@/services/api';

const isOnline = ref(networkStatus.isOnline());
const serverHealthy = ref(true);
const checking = ref(false);
const showReconnected = ref(false);

let unsubscribeNetworkStatus = null;
let healthCheckInterval = null;
let reconnectedTimeout = null;

// 检查服务器健康状态
async function checkServerHealth() {
  if (!isOnline.value) return;
  
  checking.value = true;
  try {
    await healthCheck();
    const wasUnhealthy = !serverHealthy.value;
    serverHealthy.value = true;
    
    // 如果之前不健康，现在健康了，显示重连成功提示
    if (wasUnhealthy) {
      showReconnectedNotification();
    }
  } catch (error) {
    serverHealthy.value = false;
    console.warn('Server health check failed:', error);
  } finally {
    checking.value = false;
  }
}

// 显示重连成功通知
function showReconnectedNotification() {
  showReconnected.value = true;
  if (reconnectedTimeout) {
    clearTimeout(reconnectedTimeout);
  }
  reconnectedTimeout = setTimeout(() => {
    showReconnected.value = false;
  }, 3000);
}

// 网络状态变化处理
function handleNetworkChange(online) {
  const wasOffline = !isOnline.value;
  isOnline.value = online;
  
  if (online) {
    // 网络恢复，检查服务器健康状态
    if (wasOffline) {
      showReconnectedNotification();
    }
    checkServerHealth();
  } else {
    // 网络断开
    serverHealthy.value = false;
  }
}

onMounted(() => {
  // 订阅网络状态变化
  unsubscribeNetworkStatus = networkStatus.onStatusChange(handleNetworkChange);
  
  // 定期检查服务器健康状态
  checkServerHealth();
  healthCheckInterval = setInterval(checkServerHealth, 30000); // 每30秒检查一次
});

onUnmounted(() => {
  if (unsubscribeNetworkStatus) {
    unsubscribeNetworkStatus();
  }
  if (healthCheckInterval) {
    clearInterval(healthCheckInterval);
  }
  if (reconnectedTimeout) {
    clearTimeout(reconnectedTimeout);
  }
});
</script> 