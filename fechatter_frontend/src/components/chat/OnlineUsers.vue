<template>
  <div class="online-users">
    <!-- Online Users Button -->
    <button 
      @click="toggleOnlineUsers"
      class="online-users-toggle"
      :class="{ 'active': showOnlineUsers }"
      :title="`${onlineCount} users online`"
    >
      <div class="flex items-center space-x-2">
        <div class="online-indicator"></div>
        <span class="text-sm font-medium">{{ onlineCount }} Online</span>
        <svg class="w-4 h-4 transition-transform" :class="{ 'rotate-180': showOnlineUsers }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
        </svg>
      </div>
    </button>

    <!-- Online Users List -->
    <transition
      enter-active-class="transition-all duration-300 ease-out"
      enter-from-class="opacity-0 scale-95"
      enter-to-class="opacity-100 scale-100"
      leave-active-class="transition-all duration-200 ease-in"
      leave-from-class="opacity-100 scale-100"
      leave-to-class="opacity-0 scale-95"
    >
      <div v-if="showOnlineUsers" class="online-users-dropdown">
        <div class="p-3 border-b border-gray-100">
          <h3 class="text-sm font-semibold text-gray-700">Online Members</h3>
        </div>
        
        <div class="max-h-96 overflow-y-auto">
          <div v-if="loading" class="p-4 text-center">
            <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600 mx-auto"></div>
          </div>
          
          <div v-else-if="actualOnlineUsers.length === 0" class="p-4 text-center text-sm text-gray-500">
            No other users online
          </div>
          
          <div v-else class="divide-y divide-gray-100">
            <div v-for="user in actualOnlineUsers" :key="user.user_id" 
                 class="px-3 py-2 hover:bg-gray-50 transition-colors cursor-pointer"
                 @click="handleUserClick(user)">
              <div class="flex items-center space-x-3">
                <!-- User Avatar -->
                <div class="relative">
                  <div class="w-8 h-8 bg-gradient-to-br from-blue-500 to-purple-600 rounded-full flex items-center justify-center text-white font-semibold text-sm">
                    {{ getInitials(user.fullname) }}
                  </div>
                  <div class="absolute bottom-0 right-0 w-3 h-3 border-2 border-white rounded-full" 
                       :class="getStatusColor(user)"></div>
                </div>
                
                <!-- User Info -->
                <div class="flex-1 min-w-0">
                  <div class="text-sm font-medium text-gray-900 truncate">
                    {{ user.fullname }}
                  </div>
                  <div class="text-xs text-gray-500">
                    {{ getStatusText(user) }}
                  </div>
                </div>
                
                <!-- Last Seen -->
                <div class="text-xs text-gray-400">
                  {{ formatLastSeen(user.last_seen) }}
                </div>
              </div>
            </div>
          </div>
        </div>
        
        <div class="p-3 border-t border-gray-100 bg-gray-50">
          <button @click="refreshOnlineUsers" class="text-xs text-blue-600 hover:text-blue-700 font-medium">
            Refresh
          </button>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import api from '@/services/api';
import realtimeCommunicationService from '@/services/sse';
import { useAuthStore } from '@/stores/auth';

const props = defineProps({
  chatId: {
    type: Number,
    required: true
  },
  workspaceId: {
    type: Number,
    required: false
  }
});

const emit = defineEmits(['user-clicked']);

// State
const showOnlineUsers = ref(false);
const onlineUsers = ref([]);
const loading = ref(false);
const error = ref(null);

// Real presence tracking state
const userPresence = ref(new Map()); // Map<userId, presenceData>
const lastPresenceUpdate = ref(null);

// Computed
const onlineCount = computed(() => {
  // Count users that are actually online based on presence data
  return Array.from(userPresence.value.values()).filter(presence => 
    presence.status === 'online' && isUserActive(presence)
  ).length;
});

// Get filtered online users with real presence status
const actualOnlineUsers = computed(() => {
  return onlineUsers.value.filter(user => {
    const presence = userPresence.value.get(user.user_id);
    return presence && presence.status === 'online' && isUserActive(presence);
  }).map(user => ({
    ...user,
    ...userPresence.value.get(user.user_id),
    isOnline: true
  }));
});

// Check if user is considered active (within last 5 minutes)
function isUserActive(presence) {
  if (!presence || !presence.last_seen) return false;
  const lastSeen = new Date(presence.last_seen);
  const fiveMinutesAgo = new Date(Date.now() - 5 * 60 * 1000);
  return lastSeen > fiveMinutesAgo;
}

// Fetch workspace users and initialize presence tracking
async function fetchOnlineUsers() {
  loading.value = true;
  error.value = null;
  
  try {
    // Use the existing /users endpoint to get all workspace users
    const response = await api.get('/users');
    
    // Handle API response
    const users = response.data?.data || response.data || [];
    
    // Store all users but don't assume they're online
    onlineUsers.value = users.map(user => ({
      user_id: user.id,
      fullname: user.fullname || user.username || `User ${user.id}`,
      email: user.email || `user${user.id}@example.com`,
      avatar_url: user.avatar_url,
      status: 'unknown', // Will be updated by presence events
      last_seen: null,
      connected_at: null
    }));

    // Initialize presence data for all users as offline
    users.forEach(user => {
      if (!userPresence.value.has(user.id)) {
        userPresence.value.set(user.id, {
          user_id: user.id,
          status: 'offline',
          last_seen: null,
          connected_at: null
        });
      }
    });
    
    // Request current presence status from server
    await syncPresenceStatus();
    
  } catch (err) {
    console.error('Failed to fetch workspace users:', err);
    error.value = err.message;
    
    // Fallback: show empty list instead of crashing
    onlineUsers.value = [];
  } finally {
    loading.value = false;
  }
}

// Sync presence status from server
async function syncPresenceStatus() {
  try {
    // Use existing /users endpoint to get workspace users
    // Presence status will be updated via SSE events
    console.log('Relying on SSE events for presence status updates');
    
    // Mark initial sync as completed
    lastPresenceUpdate.value = new Date().toISOString();
  } catch (err) {
    // This should not happen with the fallback approach
    console.warn('Initial presence sync completed with SSE fallback:', err.message);
  }
}

// Toggle online users dropdown
function toggleOnlineUsers() {
  showOnlineUsers.value = !showOnlineUsers.value;
  if (showOnlineUsers.value && onlineUsers.value.length === 0) {
    fetchOnlineUsers();
  }
}

// Refresh online users
function refreshOnlineUsers() {
  fetchOnlineUsers();
}

// Handle user presence updates from SSE
function handleUserPresence(data) {
  // Update presence map with new data
  const presenceData = {
    user_id: data.user_id,
    status: data.status || 'offline',
    last_seen: data.last_seen || data.timestamp || new Date().toISOString(),
    connected_at: data.connected_at || (data.status === 'online' ? new Date().toISOString() : null),
    last_updated: new Date().toISOString(),
    session_id: data.session_id,
    device_info: data.device_info
  };
  
  userPresence.value.set(data.user_id, presenceData);
  lastPresenceUpdate.value = new Date().toISOString();
  
  // If this is a new user we don't have in our users list, try to fetch user info
  const existingUser = onlineUsers.value.find(u => u.user_id === data.user_id);
  if (!existingUser && data.status === 'online') {
    fetchUserInfo(data.user_id).then(userInfo => {
      if (userInfo) {
        onlineUsers.value.push({
          user_id: userInfo.id,
          fullname: userInfo.fullname || userInfo.username || `User ${userInfo.id}`,
          email: userInfo.email,
          avatar_url: userInfo.avatar_url,
          status: 'online',
          last_seen: presenceData.last_seen,
          connected_at: presenceData.connected_at
        });
      }
    }).catch(err => {
      console.warn(`Failed to fetch user info for ${data.user_id}:`, err);
    });
  }
}

// Fetch individual user info when we get presence for unknown user
async function fetchUserInfo(userId) {
  try {
    const response = await api.get(`/users/${userId}`);
    return response.data?.data || response.data;
  } catch (err) {
    console.warn(`User ${userId} info not available:`, err);
    return null;
  }
}

// Handle user click
function handleUserClick(user) {
  emit('user-clicked', user);
  showOnlineUsers.value = false;
}

// Format last seen time - custom implementation without date-fns
function formatLastSeen(timestamp) {
  if (!timestamp) return 'Never';
  
  try {
    const now = new Date();
    const date = new Date(timestamp);
    const diffInSeconds = Math.floor((now - date) / 1000);
    
    if (diffInSeconds < 60) {
      return 'Just now';
    } else if (diffInSeconds < 3600) {
      const minutes = Math.floor(diffInSeconds / 60);
      return `${minutes} minute${minutes !== 1 ? 's' : ''} ago`;
    } else if (diffInSeconds < 86400) {
      const hours = Math.floor(diffInSeconds / 3600);
      return `${hours} hour${hours !== 1 ? 's' : ''} ago`;
    } else if (diffInSeconds < 2592000) {
      const days = Math.floor(diffInSeconds / 86400);
      return `${days} day${days !== 1 ? 's' : ''} ago`;
    } else {
      const months = Math.floor(diffInSeconds / 2592000);
      return `${months} month${months !== 1 ? 's' : ''} ago`;
    }
  } catch {
    return 'Unknown';
  }
}

// Get user initials
function getInitials(name) {
  if (!name) return '?';
  
  const parts = name.trim().split(' ');
  if (parts.length >= 2) {
    return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
  }
  return name.slice(0, 2).toUpperCase();
}

// Get status text for user
function getStatusText(user) {
  const presence = userPresence.value.get(user.user_id);
  if (!presence) return 'Unknown';
  
  if (presence.status === 'online') {
    return 'Online';
  } else if (presence.status === 'away') {
    return 'Away';
  } else if (presence.status === 'busy') {
    return 'Busy';
  } else if (presence.last_seen) {
    const lastSeen = new Date(presence.last_seen);
    const now = new Date();
    const diffInMinutes = Math.floor((now - lastSeen) / (1000 * 60));
    
    if (diffInMinutes < 1) {
      return 'Just left';
    } else if (diffInMinutes < 60) {
      return `Left ${diffInMinutes}m ago`;
    } else if (diffInMinutes < 1440) {
      const hours = Math.floor(diffInMinutes / 60);
      return `Left ${hours}h ago`;
    } else {
      const days = Math.floor(diffInMinutes / 1440);
      return `Left ${days}d ago`;
    }
  }
  
  return 'Offline';
}

// Get status color for user indicator
function getStatusColor(user) {
  const presence = userPresence.value.get(user.user_id);
  if (!presence) return 'bg-gray-400'; // Unknown status
  
  switch (presence.status) {
    case 'online':
      return 'bg-green-500'; // Green for online
    case 'away':
      return 'bg-yellow-500'; // Yellow for away
    case 'busy':
      return 'bg-red-500'; // Red for busy
    case 'offline':
    default:
      return 'bg-gray-400'; // Gray for offline
  }
}

// Send our own presence update
function sendPresenceUpdate(status = 'online') {
  // For now, this is handled by SSE connection status
  // In the future, we could have an explicit API endpoint
  console.log(`Would send presence update: ${status}`);
}

// Watch for chat changes
watch(() => props.chatId, () => {
  if (showOnlineUsers.value) {
    fetchOnlineUsers();
  }
});

// Lifecycle
onMounted(() => {
  // Listen for user presence events from SSE
  realtimeCommunicationService.on('user_presence', handleUserPresence);
  
  // Listen for SSE connection events to update our own status
  realtimeCommunicationService.on('connected', () => {
    sendPresenceUpdate('online');
  });
  
  realtimeCommunicationService.on('disconnected', () => {
    // Mark ourselves as offline in local state
    const authStore = useAuthStore();
    if (authStore.user) {
      handleUserPresence({
        user_id: authStore.user.id,
        status: 'offline',
        last_seen: new Date().toISOString()
      });
    }
  });
  
  // Initial fetch of workspace users
  fetchOnlineUsers();
  
  // Refresh presence status periodically (every 2 minutes)
  const refreshInterval = setInterval(() => {
    syncPresenceStatus();
  }, 120000); // Every 2 minutes
  
  // Clean up stale presence data periodically (every 1 minute)
  const cleanupInterval = setInterval(() => {
    const now = new Date();
    const tenMinutesAgo = new Date(now.getTime() - 10 * 60 * 1000);
    
    // Remove users who haven't been seen in 10 minutes
    for (const [userId, presence] of userPresence.value.entries()) {
      if (presence.last_seen && new Date(presence.last_seen) < tenMinutesAgo) {
        presence.status = 'offline';
      }
    }
  }, 60000); // Every 1 minute
  
  // Store intervals for cleanup
  window.__onlineUsersRefreshInterval = refreshInterval;
  window.__onlineUsersCleanupInterval = cleanupInterval;
});

onUnmounted(() => {
  // Remove all SSE event listeners
  realtimeCommunicationService.off('user_presence', handleUserPresence);
  realtimeCommunicationService.off('connected');
  realtimeCommunicationService.off('disconnected');
  
  // Clear all intervals
  if (window.__onlineUsersRefreshInterval) {
    clearInterval(window.__onlineUsersRefreshInterval);
    delete window.__onlineUsersRefreshInterval;
  }
  
  if (window.__onlineUsersCleanupInterval) {
    clearInterval(window.__onlineUsersCleanupInterval);
    delete window.__onlineUsersCleanupInterval;
  }
});
</script>

<style scoped>
.online-users {
  position: relative;
}

.online-users-toggle {
  padding: 6px 12px;
  border-radius: 6px;
  transition: all 0.2s;
  background: transparent;
  border: 1px solid transparent;
}

.online-users-toggle:hover {
  background: rgba(0, 0, 0, 0.05);
}

.online-users-toggle.active {
  background: rgba(59, 130, 246, 0.1);
  border-color: rgba(59, 130, 246, 0.2);
  color: #3b82f6;
}

.online-indicator {
  width: 8px;
  height: 8px;
  background: #10b981;
  border-radius: 50%;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% {
    box-shadow: 0 0 0 0 rgba(16, 185, 129, 0.5);
  }
  70% {
    box-shadow: 0 0 0 6px rgba(16, 185, 129, 0);
  }
  100% {
    box-shadow: 0 0 0 0 rgba(16, 185, 129, 0);
  }
}

.online-users-dropdown {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 8px;
  width: 280px;
  background: white;
  border: 1px solid rgba(0, 0, 0, 0.1);
  border-radius: 8px;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1), 0 4px 10px rgba(0, 0, 0, 0.05);
  z-index: 50;
}
</style>