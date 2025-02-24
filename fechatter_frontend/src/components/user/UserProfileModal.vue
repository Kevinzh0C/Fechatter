<template>
  <div v-if="user" class="profile-modal-overlay" @click="handleOverlayClick">
    <div class="profile-modal-container" @click.stop>
      <!-- Header with Close Button -->
      <div class="profile-header">
        <button @click="$emit('close')" class="close-button">
          <svg class="close-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- User Avatar Section -->
      <div class="profile-avatar-section">
        <div class="avatar-container">
          <img v-if="user.avatar_url && !showFallbackAvatar" :src="user.avatar_url" :alt="user.fullname"
            class="user-avatar" @error="showFallbackAvatar = true" />
          <div v-else class="fallback-avatar">
            <span class="avatar-initials">{{ getUserInitials(user.fullname) }}</span>
          </div>
          <div class="online-status" :class="getStatusClass(user.status)">
            <div class="status-dot"></div>
          </div>
        </div>
      </div>

      <!-- User Information -->
      <div class="profile-info-section">
        <h2 class="user-name">{{ user.fullname || 'Unknown User' }}</h2>

        <div class="user-details">
          <!-- Employee ID -->
          <div class="detail-item">
            <svg class="detail-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M10 6H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V8a2 2 0 00-2-2h-5m-4 0V5a2 2 0 114 0v1m-4 0a2 2 0 104 0m-5 8a2 2 0 100-4 2 2 0 000 4zm0 0c1.306 0 2.417.835 2.83 2M9 14a3.001 3.001 0 00-6 0m6 0a3.001 3.001 0 006 0" />
            </svg>
            <div class="detail-content">
              <span class="detail-label">Employee</span>
              <span class="detail-value">{{ user.id || 'N/A' }}</span>
            </div>
          </div>

          <!-- Email -->
          <div class="detail-item">
            <svg class="detail-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M3 8l7.89 4.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
            </svg>
            <div class="detail-content">
              <span class="detail-label">Email</span>
              <span class="detail-value">{{ user.email || 'No email provided' }}</span>
            </div>
          </div>

          <!-- Department (if available) -->
          <div v-if="user.department" class="detail-item">
            <svg class="detail-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
            </svg>
            <div class="detail-content">
              <span class="detail-label">Department</span>
              <span class="detail-value">{{ user.department }}</span>
            </div>
          </div>

          <!-- Title (if available) -->
          <div v-if="user.title" class="detail-item">
            <svg class="detail-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M21 13.255A23.931 23.931 0 0112 15c-3.183 0-6.22-.62-9-1.745M16 6V4a2 2 0 00-2-2h-4a2 2 0 00-2-2v2m8 0V6a2 2 0 00-2 2H8a2 2 0 00-2-2V6m8 0H8m0 0v.01M8 6v.01m8-.01V6m0 .01V6" />
            </svg>
            <div class="detail-content">
              <span class="detail-label">Title</span>
              <span class="detail-value">{{ user.title }}</span>
            </div>
          </div>

          <!-- Status -->
          <div class="detail-item">
            <svg class="detail-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <div class="detail-content">
              <span class="detail-label">Status</span>
              <span class="detail-value status-text" :class="getStatusClass(user.status)">
                {{ getStatusText(user.status) }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="profile-actions">
        <button @click="createDM" class="action-button primary" :disabled="isCreatingDM">
          <svg v-if="isCreatingDM" class="button-icon animate-spin" fill="none" stroke="currentColor"
            viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          <svg v-else class="button-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
          </svg>
          <span>{{ isCreatingDM ? 'Creating...' : 'Send Message' }}</span>
        </button>

        <button @click="viewFullProfile" class="action-button secondary">
          <svg class="button-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
          </svg>
          <span>View Profile</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'

const props = defineProps({
  user: {
    type: Object,
    default: null
  }
})

const emit = defineEmits(['close', 'dm-created', 'view-profile'])

// State
const isCreatingDM = ref(false)
const showFallbackAvatar = ref(false)

// Methods
const createDM = async () => {
  if (isCreatingDM.value) return

  isCreatingDM.value = true
  try {
    await new Promise(resolve => setTimeout(resolve, 500)) // Simulate API call
    emit('dm-created', props.user.id)
  } catch (error) {
    console.error('Failed to create DM:', error)
  } finally {
    isCreatingDM.value = false
  }
}

const viewFullProfile = () => {
  emit('view-profile', props.user)
}

const handleOverlayClick = (event) => {
  if (event.target === event.currentTarget) {
    emit('close')
  }
}

const getUserInitials = (fullname) => {
  if (!fullname) return '?'
  return fullname
    .split(' ')
    .map(name => name.charAt(0))
    .join('')
    .toUpperCase()
    .slice(0, 2)
}

const getStatusClass = (status) => {
  switch (status?.toLowerCase()) {
    case 'online':
    case 'active':
      return 'status-online'
    case 'away':
      return 'status-away'
    case 'busy':
    case 'dnd':
      return 'status-busy'
    case 'offline':
    case 'inactive':
    default:
      return 'status-offline'
  }
}

const getStatusText = (status) => {
  switch (status?.toLowerCase()) {
    case 'online':
    case 'active':
      return 'Online'
    case 'away':
      return 'Away'
    case 'busy':
    case 'dnd':
      return 'Busy'
    case 'offline':
    case 'inactive':
    default:
      return 'Offline'
  }
}
</script>

<style scoped>
/* Modern Profile Modal Styles */
.profile-modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
  animation: fadeIn 0.2s ease-out;
}

.profile-modal-container {
  background: linear-gradient(145deg, #ffffff 0%, #f8fafc 100%);
  border-radius: 24px;
  box-shadow:
    0 25px 50px rgba(0, 0, 0, 0.25),
    0 0 0 1px rgba(255, 255, 255, 0.05);
  width: 100%;
  max-width: 420px;
  overflow: hidden;
  position: relative;
  animation: slideUp 0.3s ease-out;
}

/* Header */
.profile-header {
  position: absolute;
  top: 16px;
  right: 16px;
  z-index: 10;
}

.close-button {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  border: none;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(10px);
  color: #6b7280;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.close-button:hover {
  background: rgba(255, 255, 255, 1);
  color: #374151;
  transform: scale(1.05);
}

.close-icon {
  width: 20px;
  height: 20px;
}

/* Avatar Section */
.profile-avatar-section {
  padding: 40px 32px 24px;
  text-align: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  position: relative;
}

.avatar-container {
  position: relative;
  display: inline-block;
}

.user-avatar,
.fallback-avatar {
  width: 120px;
  height: 120px;
  border-radius: 50%;
  border: 4px solid rgba(255, 255, 255, 0.9);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  object-fit: cover;
}

.fallback-avatar {
  background: linear-gradient(135deg, #f59e0b, #d97706);
  display: flex;
  align-items: center;
  justify-content: center;
}

.avatar-initials {
  font-size: 36px;
  font-weight: 700;
  color: white;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
}

.online-status {
  position: absolute;
  bottom: 8px;
  right: 8px;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: white;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.status-dot {
  width: 16px;
  height: 16px;
  border-radius: 50%;
}

.status-online .status-dot {
  background: #10b981;
  box-shadow: 0 0 0 2px #10b981;
}

.status-away .status-dot {
  background: #f59e0b;
  box-shadow: 0 0 0 2px #f59e0b;
}

.status-busy .status-dot {
  background: #ef4444;
  box-shadow: 0 0 0 2px #ef4444;
}

.status-offline .status-dot {
  background: #6b7280;
  box-shadow: 0 0 0 2px #6b7280;
}

/* User Information */
.profile-info-section {
  padding: 24px 32px;
}

.user-name {
  font-size: 24px;
  font-weight: 700;
  color: #1f2937;
  margin: 0 0 24px 0;
  text-align: center;
}

.user-details {
  space-y: 16px;
}

.detail-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px 0;
}

.detail-icon {
  width: 20px;
  height: 20px;
  color: #6366f1;
  flex-shrink: 0;
  margin-top: 2px;
}

.detail-content {
  flex: 1;
  min-width: 0;
}

.detail-label {
  display: block;
  font-size: 12px;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 2px;
}

.detail-value {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: #374151;
  word-wrap: break-word;
}

.status-text.status-online {
  color: #10b981;
}

.status-text.status-away {
  color: #f59e0b;
}

.status-text.status-busy {
  color: #ef4444;
}

.status-text.status-offline {
  color: #6b7280;
}

/* Action Buttons */
.profile-actions {
  padding: 24px 32px 32px;
  display: flex;
  gap: 12px;
}

.action-button {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 14px 20px;
  border-radius: 12px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  border: none;
  position: relative;
  overflow: hidden;
}

.action-button.primary {
  background: linear-gradient(135deg, #6366f1, #8b5cf6);
  color: white;
  box-shadow: 0 4px 14px rgba(99, 102, 241, 0.4);
}

.action-button.primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(99, 102, 241, 0.5);
}

.action-button.primary:disabled {
  opacity: 0.7;
  cursor: not-allowed;
  transform: none;
}

.action-button.secondary {
  background: rgba(107, 114, 128, 0.1);
  color: #374151;
  border: 1px solid rgba(107, 114, 128, 0.2);
}

.action-button.secondary:hover {
  background: rgba(107, 114, 128, 0.15);
  border-color: rgba(107, 114, 128, 0.3);
  transform: translateY(-1px);
}

.button-icon {
  width: 18px;
  height: 18px;
}

/* Animations */
@keyframes fadeIn {
  from {
    opacity: 0;
  }

  to {
    opacity: 1;
  }
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px) scale(0.95);
  }

  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}

.animate-spin {
  animation: spin 1s linear infinite;
}

/* Responsive Design */
@media (max-width: 480px) {
  .profile-modal-overlay {
    padding: 16px;
  }

  .profile-modal-container {
    max-width: none;
    width: 100%;
  }

  .profile-avatar-section {
    padding: 32px 24px 20px;
  }

  .user-avatar,
  .fallback-avatar {
    width: 100px;
    height: 100px;
  }

  .avatar-initials {
    font-size: 30px;
  }

  .profile-info-section,
  .profile-actions {
    padding-left: 24px;
    padding-right: 24px;
  }

  .profile-actions {
    flex-direction: column;
  }
}
</style>