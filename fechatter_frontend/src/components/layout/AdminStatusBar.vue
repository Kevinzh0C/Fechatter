<template>
  <div class="admin-status-bar">
    <div class="admin-info">
      <div class="admin-indicator" :class="isSetupMode ? 'setup-mode' : 'online-mode'">
        <span class="status-icon">{{ isSetupMode ? '⚙️' : '🟢' }}</span>
        <span class="status-text">
          Super Administrator
          <span class="status-mode">({{ isSetupMode ? 'Setup Mode' : (isOnline ? 'Online' : 'Offline') }})</span>
        </span>
      </div>
    </div>

    <div class="admin-controls">
      <!-- Online/Offline Toggle -->
      <button v-if="!isSetupMode" @click="toggleOnlineStatus" class="status-toggle-btn"
        :class="{ 'online': isOnline, 'offline': !isOnline }" :title="`Switch to ${isOnline ? 'Offline' : 'Online'}`">
        <span class="toggle-icon">{{ isOnline ? '🟢' : '⚫' }}</span>
        <span class="toggle-text">{{ isOnline ? 'Online' : 'Offline' }}</span>
      </button>

      <!-- Setup Mode Indicator -->
      <div v-else class="setup-indicator">
        <span class="setup-icon">⚙️</span>
        <span class="setup-text">Initializing workspace...</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { useWorkspaceStore } from '@/stores/workspace'

// Stores
const authStore = useAuthStore()
const workspaceStore = useWorkspaceStore()

// State
const isOnline = ref(true)
const isInitializing = ref(false)

// Computed
const isSetupMode = computed(() => {
  // Check if user is admin and workspace is in setup mode
  const user = authStore.user
  const workspace = workspaceStore.currentWorkspace

  return user?.role === 'admin' && (!workspace?.is_initialized || isInitializing.value)
})

// Methods
const toggleOnlineStatus = () => {
  isOnline.value = !isOnline.value

  // Emit presence update event
  window.dispatchEvent(new CustomEvent('admin-presence-change', {
    detail: { status: isOnline.value ? 'online' : 'offline' }
  }))

  // Store preference
  localStorage.setItem('admin_online_status', isOnline.value.toString())
}

// Initialize
onMounted(() => {
  // Restore last online status
  const savedStatus = localStorage.getItem('admin_online_status')
  if (savedStatus !== null) {
    isOnline.value = savedStatus === 'true'
  }

  // Check workspace initialization status
  if (workspaceStore.currentWorkspace) {
    isInitializing.value = !workspaceStore.currentWorkspace.is_initialized
  }
})
</script>

<style scoped>
/* Admin Status Bar Design */
.admin-status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: linear-gradient(135deg, rgba(88, 101, 242, 0.1), rgba(139, 92, 246, 0.1));
  border: 1px solid rgba(88, 101, 242, 0.2);
  border-radius: 8px;
  margin: 16px;
  backdrop-filter: blur(10px);
  transition: all 0.3s ease;
}

.admin-status-bar:hover {
  background: linear-gradient(135deg, rgba(88, 101, 242, 0.15), rgba(139, 92, 246, 0.15));
  border-color: rgba(88, 101, 242, 0.3);
}

/* Admin Info Section */
.admin-info {
  display: flex;
  align-items: center;
  flex: 1;
}

.admin-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-icon {
  font-size: 16px;
  animation: pulse 2s infinite;
}

.status-text {
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
  line-height: 1.2;
}

.status-mode {
  font-weight: 400;
  color: rgba(255, 255, 255, 0.7);
}

/* Setup Mode Styles */
.setup-mode .status-icon {
  animation: rotate 2s linear infinite;
}

/* Online Mode Styles */
.online-mode .status-icon {
  filter: drop-shadow(0 0 4px rgba(34, 197, 94, 0.5));
}

/* Admin Controls */
.admin-controls {
  display: flex;
  align-items: center;
}

/* Status Toggle Button */
.status-toggle-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.8);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  backdrop-filter: blur(5px);
}

.status-toggle-btn:hover {
  background: rgba(255, 255, 255, 0.15);
  border-color: rgba(255, 255, 255, 0.3);
  color: rgba(255, 255, 255, 0.9);
  transform: translateY(-1px);
}

.status-toggle-btn:active {
  transform: translateY(0);
}

.status-toggle-btn.online {
  border-color: rgba(34, 197, 94, 0.3);
  background: rgba(34, 197, 94, 0.1);
  color: rgba(34, 197, 94, 0.9);
}

.status-toggle-btn.online:hover {
  background: rgba(34, 197, 94, 0.15);
  border-color: rgba(34, 197, 94, 0.4);
}

.status-toggle-btn.offline {
  border-color: rgba(156, 163, 175, 0.3);
  background: rgba(156, 163, 175, 0.1);
  color: rgba(156, 163, 175, 0.9);
}

.status-toggle-btn.offline:hover {
  background: rgba(156, 163, 175, 0.15);
  border-color: rgba(156, 163, 175, 0.4);
}

.toggle-icon {
  font-size: 10px;
}

.toggle-text {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

/* Setup Indicator */
.setup-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border: 1px solid rgba(251, 191, 36, 0.3);
  border-radius: 6px;
  background: rgba(251, 191, 36, 0.1);
  color: rgba(251, 191, 36, 0.9);
  font-size: 12px;
  font-weight: 500;
}

.setup-icon {
  font-size: 10px;
  animation: rotate 2s linear infinite;
}

.setup-text {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

/* Animations */
@keyframes pulse {

  0%,
  100% {
    opacity: 1;
    transform: scale(1);
  }

  50% {
    opacity: 0.8;
    transform: scale(1.05);
  }
}

@keyframes rotate {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}

/* Responsive Design */
@media (max-width: 768px) {
  .admin-status-bar {
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    margin: 12px;
  }

  .admin-controls {
    width: 100%;
    justify-content: center;
  }

  .status-toggle-btn,
  .setup-indicator {
    flex: 1;
    justify-content: center;
    max-width: 200px;
  }
}

/* Dark Mode Optimizations */
@media (prefers-color-scheme: dark) {
  .admin-status-bar {
    background: linear-gradient(135deg, rgba(88, 101, 242, 0.15), rgba(139, 92, 246, 0.15));
    border-color: rgba(88, 101, 242, 0.25);
  }

  .admin-status-bar:hover {
    background: linear-gradient(135deg, rgba(88, 101, 242, 0.2), rgba(139, 92, 246, 0.2));
    border-color: rgba(88, 101, 242, 0.35);
  }
}

/* Reduced Motion Support */
@media (prefers-reduced-motion: reduce) {

  .status-icon,
  .setup-icon {
    animation: none;
  }

  .status-toggle-btn:hover {
    transform: none;
  }

  .admin-status-bar {
    transition: none;
  }
}
</style>