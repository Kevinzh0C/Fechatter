<template>
  <div class="online-users">
    <!-- Online Users Button -->
    <button @click="toggleOnlineUsers" class="online-users-toggle" :class="{ 'active': showOnlineUsers }"
      :title="`${onlineCount} users online`">
      <div class="flex items-center space-x-2">
        <div class="online-indicator"></div>
        <span class="text-sm font-medium">{{ onlineCount }} Online</span>
        <svg class="w-4 h-4 transition-transform" :class="{ 'rotate-180': showOnlineUsers }" fill="none"
          stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
        </svg>
      </div>
    </button>

    <!-- Online Users List -->
    <transition enter-active-class="transition-all duration-300 ease-out" enter-from-class="opacity-0 scale-95"
      enter-to-class="opacity-100 scale-100" leave-active-class="transition-all duration-200 ease-in"
      leave-from-class="opacity-100 scale-100" leave-to-class="opacity-0 scale-95">
      <div v-if="showOnlineUsers" class="online-users-dropdown">
        <div class="p-3 border-b border-gray-100">
          <h3 class="text-sm font-semibold text-gray-700">Online Members</h3>
        </div>

        <div class="max-h-96 overflow-y-auto">
          <div v-if="loading" class="p-4 text-center">
            <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600 mx-auto"></div>
          </div>

          <div v-else-if="onlineUsers.length === 0" class="p-4 text-center text-sm text-gray-500">
            No other users online
          </div>

          <div v-else class="divide-y divide-gray-100">
            <div v-for="user in onlineUsers" :key="user.user_id"
              class="px-3 py-2 hover:bg-gray-50 transition-colors cursor-pointer" @click="handleUserClick(user)">
              <div class="flex items-center space-x-3">
                <!-- User Avatar -->
                <div class="relative">
                  <div
                    class="w-8 h-8 bg-gradient-to-br from-blue-500 to-purple-600 rounded-full flex items-center justify-center text-white font-semibold text-sm">
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

// Mock data for development
const mockUsers = [
  {
    user_id: 1,
    fullname: 'John Doe',
    email: 'john@example.com',
    status: 'online',
    last_seen: new Date().toISOString()
  },
  {
    user_id: 2,
    fullname: 'Jane Smith',
    email: 'jane@example.com',
    status: 'away',
    last_seen: new Date(Date.now() - 5 * 60 * 1000).toISOString()
  }
];

// Computed
const onlineCount = computed(() => {
  return onlineUsers.value.filter(user => user.status === 'online').length;
});

// Fetch online users
async function fetchOnlineUsers() {
  loading.value = true;
  error.value = null;

  try {
    // TODO: Replace with actual API call
    await new Promise(resolve => setTimeout(resolve, 500));
    onlineUsers.value = [...mockUsers];
  } catch (err) {
    console.error('Failed to fetch online users:', err);
    error.value = err.message;
    onlineUsers.value = [];
  } finally {
    loading.value = false;
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

// Handle user click
function handleUserClick(user) {
  emit('user-clicked', user);
  showOnlineUsers.value = false;
}

// Format last seen time
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
    } else {
      const days = Math.floor(diffInSeconds / 86400);
      return `${days} day${days !== 1 ? 's' : ''} ago`;
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
  if (!user) return 'Unknown';

  if (user.status === 'online') {
    return 'Online';
  } else if (user.status === 'away') {
    return 'Away';
  } else if (user.status === 'busy') {
    return 'Busy';
  }
  return 'Offline';
}

// Get status color for user indicator
function getStatusColor(user) {
  if (!user) return 'bg-gray-400';

  switch (user.status) {
    case 'online':
      return 'bg-green-500';
    case 'away':
      return 'bg-yellow-500';
    case 'busy':
      return 'bg-red-500';
    case 'offline':
    default:
      return 'bg-gray-400';
  }
}

// Watch for chat changes
watch(() => props.chatId, () => {
  if (showOnlineUsers.value) {
    fetchOnlineUsers();
  }
});

// Lifecycle
onMounted(() => {
  fetchOnlineUsers();
});

onUnmounted(() => {
  // Cleanup
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