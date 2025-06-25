<template>
  <div class="user-bottom-bar">
    <div class="user-section" @click="toggleUserMenu" ref="userSection">
      <!-- User Avatar -->
      <div class="user-avatar-container">
        <img v-if="currentUser?.avatar_url" :src="currentUser.avatar_url" :alt="currentUser.fullname"
          class="user-avatar" @error="handleAvatarError" />
        <div v-else class="user-avatar user-avatar-placeholder">
          <span class="avatar-initials">{{ getUserInitials(currentUser?.fullname) }}</span>
        </div>

        <!-- Online Status Indicator -->
        <div class="status-indicator" :class="getStatusClass()"></div>
      </div>

      <!-- User Info -->
      <div class="user-info">
        <div class="user-name">{{ currentUser?.fullname || 'User' }}</div>
        <div class="user-status">{{ getStatusText() }}</div>
      </div>

      <!-- Menu Indicator -->
      <div class="menu-indicator" :class="{ 'active': showUserMenu }">
        <Icon name="chevron-up" class="chevron-icon" />
      </div>
    </div>

    <!-- User Menu Dropdown -->
    <Transition name="menu-slide">
      <div v-if="showUserMenu" class="user-menu" @click.stop>
        <div class="menu-header">
          <div class="menu-user-info">
            <div class="menu-user-name">{{ currentUser?.fullname }}</div>
            <div class="menu-user-email">{{ currentUser?.email }}</div>
          </div>
        </div>

        <div class="menu-divider"></div>

        <div class="menu-items">
          <button @click="openProfile" class="menu-item">
            <Icon name="user" class="menu-icon" />
            <span>Profile</span>
          </button>

          <button @click="openSettings" class="menu-item">
            <Icon name="settings" class="menu-icon" />
            <span>Settings</span>
          </button>

          <button v-if="canSwitchRole" @click="switchRole" class="menu-item">
            <Icon name="shield" class="menu-icon" />
            <span>Switch role</span>
          </button>
        </div>

        <div class="menu-divider"></div>

        <button @click="signOut" class="menu-item menu-item-danger">
          <Icon name="log-out" class="menu-icon" />
          <span>Sign out</span>
        </button>
      </div>
    </Transition>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useUserStore } from '@/stores/user'
import Icon from '@/components/ui/Icon.vue'

// Props
const props = defineProps({
  showAdminFeatures: {
    type: Boolean,
    default: false
  }
})

// Emits
const emit = defineEmits(['profile-opened', 'settings-opened', 'role-switched'])

// Stores
const router = useRouter()
const authStore = useAuthStore()
const userStore = useUserStore()

// State
const showUserMenu = ref(false)
const userSection = ref(null)

// Computed
const currentUser = computed(() => authStore.user)

const canSwitchRole = computed(() => {
  return currentUser.value?.role === 'admin' || currentUser.value?.role === 'moderator'
})

// Methods
const toggleUserMenu = () => {
  showUserMenu.value = !showUserMenu.value
}

const closeUserMenu = () => {
  showUserMenu.value = false
}

const getUserInitials = (name) => {
  if (!name) return '?'
  return name
    .split(' ')
    .map(word => word.charAt(0))
    .join('')
    .toUpperCase()
    .slice(0, 2)
}

const handleAvatarError = () => {
  // Handle avatar loading error by showing initials
  console.warn('Failed to load user avatar')
}

const getStatusClass = () => {
  // Get user's online status - could be connected to presence system
  return 'status-online' // Default to online for now
}

const getStatusText = () => {
  // Return user status text
  if (currentUser.value?.role === 'admin') {
    return 'Administrator'
  }
  if (currentUser.value?.role === 'moderator') {
    return 'Moderator'
  }
  return 'Online'
}

const openProfile = () => {
  closeUserMenu()
  emit('profile-opened')
  // Could open profile modal or navigate to profile page
}

const openSettings = () => {
  closeUserMenu()
  emit('settings-opened')
  // Could open settings modal or navigate to settings page
}

const switchRole = () => {
  closeUserMenu()
  emit('role-switched')
  // Implementation for role switching
}

const signOut = async () => {
  closeUserMenu()
  try {
    await authStore.logout()
    router.push('/login')
  } catch (error) {
    console.error('Failed to sign out:', error)
  }
}

// Click outside handler
const handleClickOutside = (event) => {
  if (userSection.value && !userSection.value.contains(event.target)) {
    closeUserMenu()
  }
}

// Lifecycle
onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped>
/* User Bottom Bar Design */
.user-bottom-bar {
  position: relative;
  padding: 12px 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(0, 0, 0, 0.2);
  backdrop-filter: blur(10px);
}

/* User Section */
.user-section {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  background: transparent;
  position: relative;
}

.user-section:hover {
  background: rgba(255, 255, 255, 0.1);
}

.user-section:active {
  background: rgba(255, 255, 255, 0.15);
}

/* User Avatar */
.user-avatar-container {
  position: relative;
  flex-shrink: 0;
}

.user-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  object-fit: cover;
  border: 2px solid rgba(255, 255, 255, 0.2);
  transition: all 0.2s ease;
}

.user-avatar-placeholder {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: linear-gradient(135deg, #5865f2, #7983f5);
  display: flex;
  align-items: center;
  justify-content: center;
  border: 2px solid rgba(255, 255, 255, 0.2);
  transition: all 0.2s ease;
}

.avatar-initials {
  font-size: 16px;
  font-weight: 600;
  color: white;
  text-transform: uppercase;
}

.user-section:hover .user-avatar,
.user-section:hover .user-avatar-placeholder {
  border-color: rgba(255, 255, 255, 0.4);
  transform: scale(1.05);
}

/* Status Indicator */
.status-indicator {
  position: absolute;
  bottom: -1px;
  right: -1px;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 2px solid var(--bg-primary);
  transition: all 0.2s ease;
}

.status-indicator.status-online {
  background: #23a55a;
  box-shadow: 0 0 6px rgba(35, 165, 90, 0.5);
}

.status-indicator.status-away {
  background: #f0b132;
  box-shadow: 0 0 6px rgba(240, 177, 50, 0.5);
}

.status-indicator.status-offline {
  background: #80848e;
}

/* User Info */
.user-info {
  flex: 1;
  min-width: 0;
}

.user-name {
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
  line-height: 1.2;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.user-status {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.6);
  line-height: 1.2;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Menu Indicator */
.menu-indicator {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s ease;
}

.menu-indicator.active {
  background: rgba(255, 255, 255, 0.1);
}

.chevron-icon {
  width: 14px;
  height: 14px;
  color: rgba(255, 255, 255, 0.5);
  transition: all 0.2s ease;
}

.menu-indicator.active .chevron-icon {
  color: rgba(255, 255, 255, 0.8);
  transform: rotate(180deg);
}

/* User Menu Dropdown */
.user-menu {
  position: absolute;
  bottom: 100%;
  left: 16px;
  right: 16px;
  background: #ffffff;
  border: none;
  border-radius: 12px;
  box-shadow:
    0 16px 32px rgba(0, 0, 0, 0.2),
    0 8px 16px rgba(0, 0, 0, 0.1),
    0 0 0 1px rgba(0, 0, 0, 0.05);
  overflow: hidden;
  z-index: 1000;
  backdrop-filter: blur(20px);
  margin-bottom: 8px;
}

/* Menu Header */
.menu-header {
  padding: 16px;
  background: linear-gradient(135deg, #f8fafc, #f1f5f9);
  border-bottom: 1px solid #e2e8f0;
}

.menu-user-info {
  text-align: left;
}

.menu-user-name {
  font-size: 16px;
  font-weight: 600;
  color: #1a1a1a;
  line-height: 1.2;
  margin-bottom: 2px;
}

.menu-user-email {
  font-size: 13px;
  color: #666;
  line-height: 1.2;
}

/* Menu Divider */
.menu-divider {
  height: 1px;
  background: #e2e8f0;
  margin: 8px 0;
}

/* Menu Items */
.menu-items {
  padding: 8px 0;
}

.menu-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border: none;
  background: none;
  color: #1a1a1a;
  font-size: 14px;
  font-weight: 500;
  text-align: left;
  cursor: pointer;
  transition: all 0.15s ease;
  min-height: 44px;
  /* Touch-friendly */
}

.menu-item:hover {
  background: #f5f7fa;
}

.menu-item:active {
  background: #e8ecef;
}

.menu-item-danger {
  color: #dc2626;
}

.menu-item-danger:hover {
  background: #fef2f2;
  color: #b91c1c;
}

.menu-icon {
  width: 16px;
  height: 16px;
  color: #666;
  flex-shrink: 0;
}

.menu-item-danger .menu-icon {
  color: #dc2626;
}

/* Menu Transitions */
.menu-slide-enter-active,
.menu-slide-leave-active {
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  transform-origin: bottom center;
}

.menu-slide-enter-from,
.menu-slide-leave-to {
  opacity: 0;
  transform: translateY(8px) scale(0.95);
}

.menu-slide-enter-to,
.menu-slide-leave-from {
  opacity: 1;
  transform: translateY(0) scale(1);
}

/* Responsive Design */
@media (max-width: 768px) {
  .user-bottom-bar {
    padding: 16px;
  }

  .user-section {
    padding: 12px;
  }

  .user-avatar,
  .user-avatar-placeholder {
    width: 44px;
    height: 44px;
  }

  .avatar-initials {
    font-size: 18px;
  }

  .user-name {
    font-size: 15px;
  }

  .user-status {
    font-size: 13px;
  }

  .menu-item {
    padding: 16px;
    font-size: 15px;
    min-height: 48px;
  }

  .menu-icon {
    width: 18px;
    height: 18px;
  }
}

/* High contrast mode support */
@media (prefers-contrast: high) {
  .user-section {
    border: 1px solid rgba(255, 255, 255, 0.3);
  }

  .user-menu {
    border: 2px solid #333;
  }
}

/* Reduced motion support */
@media (prefers-reduced-motion: reduce) {

  .user-section,
  .user-avatar,
  .user-avatar-placeholder,
  .menu-indicator,
  .chevron-icon {
    transition: none;
  }

  .menu-slide-enter-active,
  .menu-slide-leave-active {
    transition: none;
  }
}
</style>