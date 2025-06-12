<template>
  <div class="channel-item" :class="{
    'active': isActive,
    'unread': hasUnread,
    'muted': channel.is_muted
  }" @click="$emit('click', channel)" @contextmenu="$emit('context-menu', $event)">
    <!-- Channel Icon -->
    <div class="channel-icon">
      <Icon :name="getChannelIcon()" />
    </div>

    <!-- Channel Content -->
    <div class="channel-content">
      <div class="channel-name">
        <span class="name-text">{{ channel.name }}</span>
        <Icon v-if="channel.is_muted" name="bell-off" class="muted-icon" />
      </div>

      <!-- Last Message Preview -->
      <div v-if="lastMessage" class="last-message">
        <span class="message-sender">{{ lastMessage.sender_name }}:</span>
        <span class="message-content">{{ truncateMessage(lastMessage.content) }}</span>
        <span class="message-time">{{ formatTime(lastMessage.created_at) }}</span>
      </div>
    </div>

    <!-- Channel Status -->
    <div class="channel-status">
      <!-- Unread Badge -->
      <div v-if="hasUnread" class="unread-badge" :class="{ 'mention': hasMentions }">
        {{ unreadDisplay }}
      </div>

      <!-- Activity Indicator -->
      <div v-if="isActive && !hasUnread" class="activity-indicator"></div>

      <!-- Typing Indicator -->
      <div v-if="isTyping" class="typing-indicator">
        <div class="typing-dots">
          <span></span>
          <span></span>
          <span></span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import Icon from '@/components/ui/Icon.vue'

// Props
const props = defineProps({
  channel: {
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

// Computed
const hasUnread = computed(() => {
  return props.channel.unread_count > 0
})

const hasMentions = computed(() => {
  return props.channel.mention_count > 0
})

const unreadDisplay = computed(() => {
  const count = props.channel.unread_count
  if (count > 99) return '99+'
  if (count > 0) return count.toString()
  return ''
})

const lastMessage = computed(() => {
  return props.channel.last_message
})

// Methods
const getChannelIcon = () => {
  switch (props.channel.chat_type) {
    case 'PublicChannel':
      return 'hash'
    case 'PrivateChannel':
      return 'lock'
    case 'AnnouncementChannel':
      return 'megaphone'
    default:
      return 'hash'
  }
}

const truncateMessage = (content) => {
  if (!content) return ''
  const maxLength = 30
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
</script>

<style scoped>
/* 🎨 Modern Channel Item - Discord & Linear Inspired */
.channel-item {
  display: flex;
  align-items: center;
  padding: 6px 8px;
  margin: 0 0 1px 0;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.1s ease;
  position: relative;
  min-height: 32px;
  color: rgba(255, 255, 255, 0.6);
  background: transparent;
  border: none;
  line-height: 1.4;
}

.channel-item:hover {
  background: rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.9);
}

.channel-item.active {
  background: rgba(255, 255, 255, 0.1);
  color: white;
}

.channel-item.active:hover {
  background: rgba(255, 255, 255, 0.12);
}

.channel-item.unread {
  color: rgba(255, 255, 255, 0.95);
}

.channel-item.unread .name-text {
  font-weight: 600;
}

.channel-item.muted {
  opacity: 0.5;
}

.channel-item.muted:hover {
  opacity: 0.7;
}

/* 📱 Channel Icon */
.channel-icon {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgba(255, 255, 255, 0.4);
  margin-right: 8px;
  flex-shrink: 0;
}

.channel-item:hover .channel-icon {
  color: rgba(255, 255, 255, 0.7);
}

.channel-item.active .channel-icon {
  color: rgba(255, 255, 255, 0.8);
}

/* 📝 Channel Content */
.channel-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.channel-name {
  display: flex;
  align-items: center;
  gap: 6px;
}

.name-text {
  font-size: 14px;
  font-weight: 500;
  color: inherit;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.4;
}

.muted-icon {
  width: 12px;
  height: 12px;
  color: rgba(255, 255, 255, 0.3);
  flex-shrink: 0;
}

/* Hide last message preview for cleaner look */
.last-message {
  display: none;
}

/* 🔔 Channel Status */
.channel-status {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-left: 8px;
  flex-shrink: 0;
}

.unread-badge {
  background: #ED4245;
  color: white;
  font-size: 11px;
  font-weight: 600;
  padding: 0 5px;
  border-radius: 8px;
  min-width: 18px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
}

.unread-badge.mention {
  background: #F0B232;
}

.activity-indicator {
  width: 6px;
  height: 6px;
  background: #3BA55C;
  border-radius: 50%;
}

/* ⏳ Typing Indicator */
.typing-indicator {
  display: flex;
  align-items: center;
  padding: 2px 6px;
  background: rgba(255, 255, 255, 0.06);
  border-radius: 8px;
}

.typing-dots {
  display: flex;
  gap: 2px;
}

.typing-dots span {
  width: 3px;
  height: 3px;
  background: rgba(255, 255, 255, 0.4);
  border-radius: 50%;
  animation: typing-bounce 1.4s infinite ease-in-out;
}

.typing-dots span:nth-child(1) {
  animation-delay: 0s;
}

.typing-dots span:nth-child(2) {
  animation-delay: 0.2s;
}

.typing-dots span:nth-child(3) {
  animation-delay: 0.4s;
}

@keyframes typing-bounce {

  0%,
  60%,
  100% {
    transform: translateY(0);
    opacity: 0.4;
  }

  30% {
    transform: translateY(-4px);
    opacity: 1;
  }
}

/* 📱 Mobile Responsive */
@media (max-width: 768px) {
  .channel-item {
    padding: 8px 10px;
    min-height: 40px;
  }

  .channel-icon {
    margin-right: 10px;
  }

  .name-text {
    font-size: 15px;
  }

  .unread-badge {
    min-width: 20px;
    height: 18px;
    font-size: 12px;
  }
}

/* 🎯 Touch-friendly */
@media (hover: none) and (pointer: coarse) {
  .channel-item {
    min-height: 44px;
  }
}

/* 🎭 Reduced Motion */
@media (prefers-reduced-motion: reduce) {

  .channel-item,
  .typing-dots span {
    animation: none;
    transition: none;
  }
}

/* 🌙 Focus States */
.channel-item:focus-visible {
  outline: 2px solid rgba(88, 101, 242, 0.5);
  outline-offset: 2px;
}

/* 🎨 High Contrast */
@media (prefers-contrast: high) {
  .channel-item {
    border: 1px solid transparent;
  }

  .channel-item:hover {
    border-color: rgba(255, 255, 255, 0.2);
  }

  .channel-item.active {
    border-color: white;
    background: rgba(255, 255, 255, 0.15);
  }

  .unread-badge {
    border: 1px solid white;
  }
}
</style>