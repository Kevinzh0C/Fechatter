<template>
  <div class="dm-item" :class="{
    'active': isActive,
    'unread': hasUnread,
    'online': isUserOnline
  }" @click="$emit('click', dm)" @contextmenu="$emit('context-menu', $event)">
    <!-- User Avatar -->
    <div class="user-avatar">
      <img v-if="dm.avatar_url" :src="dm.avatar_url" :alt="dm.name" class="avatar-image" @error="handleImageError" />
      <div v-else class="avatar-placeholder" :class="{ 'group-avatar': isGroupChat }">
        <Icon v-if="isGroupChat" name="users" class="group-icon" />
        <span v-else>{{ getInitials(dm.name) }}</span>
      </div>

      <!-- Online Status Indicator (only for DMs, not groups) -->
      <div v-if="!isGroupChat" class="status-indicator" :class="getStatusClass()" :title="getStatusText()"></div>
      <div v-else class="group-indicator" :title="`${dm.member_count || 0} members`">
        <Icon name="users" class="group-count-icon" />
      </div>
    </div>

    <!-- DM Content -->
    <div class="dm-content">
      <div class="dm-header">
        <span class="user-name">{{ dm.name }}</span>
        <span v-if="lastMessage" class="last-time">
          {{ formatTime(lastMessage.created_at) }}
        </span>
      </div>

      <!-- Last Message Preview -->
      <div v-if="lastMessage" class="last-message">
        <Icon v-if="lastMessage.sender_id === currentUserId" name="arrow-right" class="sent-icon" />
        <span class="message-content">{{ truncateMessage(lastMessage.content) }}</span>
      </div>

      <!-- Status Text -->
      <div v-else class="user-status">
        {{ getStatusText() }}
      </div>
    </div>

    <!-- DM Status -->
    <div class="dm-status">
      <!-- Unread Badge -->
      <div v-if="hasUnread" class="unread-badge" :class="{ 'mention': hasMentions }">
        {{ unreadDisplay }}
      </div>

      <!-- Activity Indicator -->
      <div v-if="isActive && !hasUnread" class="activity-indicator"></div>

      <!-- Typing Indicator -->
      <div v-if="isTyping" class="typing-indicator">
        <Icon name="edit" class="typing-icon" />
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useAuthStore } from '@/stores/auth'
import Icon from '@/components/ui/Icon.vue'

// Props
const props = defineProps({
  dm: {
    type: Object,
    required: true
  },
  isActive: {
    type: Boolean,
    default: false
  },
  isTyping: {
    type: Boolean,
    default: false
  }
})

// Emits
defineEmits(['click', 'context-menu'])

// Store
const authStore = useAuthStore()

// Computed
const currentUserId = computed(() => authStore.user?.id)

const hasUnread = computed(() => {
  return props.dm.unread_count > 0
})

const hasMentions = computed(() => {
  return props.dm.mention_count > 0
})

const unreadDisplay = computed(() => {
  const count = props.dm.unread_count
  if (count > 99) return '99+'
  if (count > 0) return count.toString()
  return ''
})

const lastMessage = computed(() => {
  return props.dm.last_message
})

const isUserOnline = computed(() => {
  return props.dm.user_status === 'online'
})

const isGroupChat = computed(() => {
  return props.dm.chat_type === 'Group' || props.dm.user_status === 'group'
})

// Methods
const getInitials = (name) => {
  if (!name) return '?'
  const words = name.trim().split(' ')
  if (words.length >= 2) {
    return (words[0][0] + words[words.length - 1][0]).toUpperCase()
  }
  return name.substring(0, 2).toUpperCase()
}

const getStatusClass = () => {
  switch (props.dm.user_status) {
    case 'online':
      return 'online'
    case 'away':
      return 'away'
    case 'busy':
      return 'busy'
    case 'offline':
    default:
      return 'offline'
  }
}

const getStatusText = () => {
  switch (props.dm.user_status) {
    case 'online':
      return 'Online'
    case 'away':
      return 'Away'
    case 'busy':
      return 'Busy'
    case 'offline':
      return 'Offline'
    default:
      return 'Unknown'
  }
}

const truncateMessage = (content) => {
  if (!content) return ''
  const maxLength = 35
  return content.length > maxLength
    ? content.substring(0, maxLength) + '...'
    : content
}

const formatTime = (timestamp) => {
  if (!timestamp) return ''

  const now = new Date()
  const messageTime = new Date(timestamp)
  const diffInMinutes = Math.floor((now - messageTime) / (1000 * 60))

  if (diffInMinutes < 1) return 'now'
  if (diffInMinutes < 60) return `${diffInMinutes}m`
  if (diffInMinutes < 1440) return `${Math.floor(diffInMinutes / 60)}h`

  const diffInDays = Math.floor(diffInMinutes / 1440)
  if (diffInDays < 7) return `${diffInDays}d`

  return messageTime.toLocaleDateString()
}

const handleImageError = (event) => {
  event.target.style.display = 'none'
  // Fallback to placeholder will be shown automatically
}
</script>

<style scoped>
/* 🎨 Modern Slack-style DM Item */
.dm-item {
  display: flex;
  align-items: center;
  padding: 6px 16px;
  margin: 0 2px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
  min-height: 52px;
  /* Slightly taller for avatar */
  color: rgba(255, 255, 255, 0.7);
  background: transparent;
  border: 1px solid transparent;
  line-height: 1.4;
}

.dm-item:hover {
  background: rgba(255, 255, 255, 0.1);
  color: white;
  transform: none;
}

.dm-item:focus {
  outline: 2px solid rgba(255, 255, 255, 0.3);
  outline-offset: 2px;
}

.dm-item.active {
  background: #1264a3;
  color: white;
  font-weight: 600;
}

.dm-item.active:hover {
  background: #1264a3;
  transform: none;
}

.dm-item.unread {
  font-weight: 600;
  color: white;
}

/* 👤 User Avatar - Enhanced design */
.user-avatar {
  position: relative;
  width: 32px;
  height: 32px;
  margin-right: 12px;
  flex-shrink: 0;
  transition: transform 0.15s ease;
}

.dm-item:hover .user-avatar {
  transform: scale(1.05);
}

.avatar-image {
  width: 100%;
  height: 100%;
  border-radius: 50%;
  object-fit: cover;
  background: rgba(255, 255, 255, 0.1);
  border: 2px solid rgba(255, 255, 255, 0.1);
  transition: border-color 0.15s ease;
}

.dm-item:hover .avatar-image {
  border-color: rgba(255, 255, 255, 0.2);
}

.avatar-placeholder {
  width: 100%;
  height: 100%;
  border-radius: 50%;
  background: linear-gradient(135deg, #1264a3, #0e4f82);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 14px;
  font-weight: 700;
  border: 2px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 2px 8px rgba(18, 100, 163, 0.2);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

.dm-item:hover .avatar-placeholder {
  border-color: rgba(255, 255, 255, 0.2);
  box-shadow: 0 4px 12px rgba(18, 100, 163, 0.3);
}

/* 🟢 Status Indicator - Modern design */
.status-indicator {
  position: absolute;
  bottom: -1px;
  right: -1px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  border: 3px solid #3f0f40;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.status-indicator.online {
  background: #2eb67d;
  box-shadow: 0 0 8px rgba(46, 182, 125, 0.4);
}

.status-indicator.away {
  background: #ecb22e;
  box-shadow: 0 0 8px rgba(236, 178, 46, 0.4);
}

.status-indicator.busy {
  background: #e01e5a;
  box-shadow: 0 0 8px rgba(224, 30, 90, 0.4);
}

.status-indicator.offline {
  background: rgba(255, 255, 255, 0.3);
  box-shadow: none;
}

/* 📝 DM Content - Typography improvements */
.dm-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.dm-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.user-name {
  font-size: 15px;
  font-weight: 500;
  color: inherit;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.4;
}

.dm-item.active .user-name {
  font-weight: 600;
}

.last-time {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.4);
  flex-shrink: 0;
  margin-left: 8px;
}

.dm-item.active .last-time {
  color: rgba(255, 255, 255, 0.7);
}

.last-message {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  overflow: hidden;
  line-height: 1.3;
}

.dm-item.active .last-message {
  color: rgba(255, 255, 255, 0.8);
}

.sent-icon {
  width: 12px;
  height: 12px;
  color: rgba(255, 255, 255, 0.4);
  flex-shrink: 0;
}

.dm-item.active .sent-icon {
  color: rgba(255, 255, 255, 0.7);
}

.message-content {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.user-status {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
  font-style: italic;
  line-height: 1.3;
}

.dm-item.active .user-status {
  color: rgba(255, 255, 255, 0.7);
}

/* 🔔 DM Status - Premium indicators */
.dm-status {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-left: 8px;
  flex-shrink: 0;
}

.unread-badge {
  background: linear-gradient(135deg, #e01e5a, #c91346);
  color: white;
  font-size: 11px;
  font-weight: 700;
  padding: 0;
  border-radius: 10px;
  min-width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  box-shadow: 0 2px 4px rgba(224, 30, 90, 0.3);
  border: 2px solid #3f0f40;
}

.unread-badge.mention {
  background: linear-gradient(135deg, #f59e0b, #d97706);
  box-shadow: 0 2px 4px rgba(245, 158, 11, 0.4);
  animation: pulse-mention 2s infinite;
}

@keyframes pulse-mention {

  0%,
  100% {
    transform: scale(1);
    box-shadow: 0 2px 4px rgba(245, 158, 11, 0.4);
  }

  50% {
    transform: scale(1.05);
    box-shadow: 0 2px 8px rgba(245, 158, 11, 0.6);
  }
}

.activity-indicator {
  width: 8px;
  height: 8px;
  background: #2eb67d;
  border-radius: 50%;
  box-shadow: 0 0 6px rgba(46, 182, 125, 0.5);
  animation: pulse-activity 2s infinite;
}

@keyframes pulse-activity {

  0%,
  100% {
    opacity: 1;
    transform: scale(1);
  }

  50% {
    opacity: 0.7;
    transform: scale(1.1);
  }
}

/* ⌨️ Typing Indicator - Enhanced animation */
.typing-indicator {
  display: flex;
  align-items: center;
  padding: 2px 6px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 8px;
}

.typing-icon {
  width: 14px;
  height: 14px;
  color: #1264a3;
  animation: typing-pulse 1.5s infinite;
}

@keyframes typing-pulse {

  0%,
  100% {
    transform: scale(1);
    opacity: 0.7;
  }

  50% {
    transform: scale(1.1);
    opacity: 1;
  }
}

/* 🌟 Online State Enhancement */
.dm-item.online .user-avatar {
  box-shadow: 0 0 0 2px rgba(46, 182, 125, 0.3);
}

.dm-item.online:hover .user-avatar {
  box-shadow: 0 0 0 2px rgba(46, 182, 125, 0.5);
}

/* 📱 Responsive Design - Mobile optimizations */
@media (max-width: 768px) {
  .dm-item {
    padding: 8px 12px;
    min-height: 56px;
    /* Larger touch targets on mobile */
    margin: 1px 4px;
  }

  .user-avatar {
    width: 36px;
    height: 36px;
    margin-right: 10px;
  }

  .status-indicator {
    width: 16px;
    height: 16px;
  }

  .user-name {
    font-size: 16px;
  }

  .last-message {
    font-size: 13px;
  }

  .unread-badge {
    min-width: 22px;
    height: 22px;
    font-size: 12px;
  }
}

/* 🎯 High Contrast Mode - Accessibility */
@media (prefers-contrast: high) {
  .dm-item {
    border: 1px solid rgba(255, 255, 255, 0.2);
  }

  .dm-item.active {
    border: 2px solid white;
    background: #0a3d67;
  }

  .status-indicator {
    border-width: 4px;
    border-color: white;
  }

  .unread-badge {
    border: 2px solid white;
  }

  .avatar-placeholder,
  .avatar-image {
    border: 3px solid white;
  }
}

/* 🎭 Reduced Motion - Respect user preferences */
@media (prefers-reduced-motion: reduce) {

  .dm-item,
  .unread-badge,
  .activity-indicator,
  .typing-icon,
  .user-avatar,
  .avatar-image,
  .avatar-placeholder,
  .status-indicator {
    animation: none;
    transition: none;
  }

  .dm-item:hover {
    transform: none;
  }

  .dm-item:hover .user-avatar {
    transform: none;
  }
}

/* 🌙 Focus states for keyboard navigation */
.dm-item:focus-visible {
  outline: 2px solid rgba(255, 255, 255, 0.8);
  outline-offset: 2px;
  background: rgba(255, 255, 255, 0.05);
}

/* 💡 Enhanced visual feedback */
.dm-item.active .user-name {
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

/* 🎨 Avatar hover effects */
.dm-item:hover .avatar-placeholder {
  background: linear-gradient(135deg, #0e4f82, #0a3d67);
}

/* 👥 Group Chat Specific Styles */
.avatar-placeholder.group-avatar {
  background: linear-gradient(135deg, #36c5f0, #2eb67d);
}

.group-icon {
  width: 14px;
  height: 14px;
  color: white;
}

.group-indicator {
  position: absolute;
  bottom: -2px;
  right: -2px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: linear-gradient(135deg, #36c5f0, #2eb67d);
  border: 3px solid #3f0f40;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 4px rgba(54, 197, 240, 0.3);
}

.group-count-icon {
  width: 8px;
  height: 8px;
  color: white;
}

/* Group hover effects */
.dm-item:hover .avatar-placeholder.group-avatar {
  background: linear-gradient(135deg, #2eb67d, #36c5f0);
}
</style>