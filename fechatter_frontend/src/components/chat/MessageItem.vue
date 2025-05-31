<template>
  <div class="message-item">
    <div class="message-content">
      <!-- 用户头像 -->
      <div v-if="!isOwnMessage(message)" class="avatar">
        <div class="avatar-circle">
          {{ getUserInitials(message.sender) }}
        </div>
      </div>
      
      <!-- 消息内容区 -->
      <div class="message-body">
        <!-- 发送者信息 -->
        <div v-if="!isOwnMessage(message)" class="sender-name">
          {{ message.sender?.username || 'Unknown User' }}
        </div>
        
        <!-- 消息气泡 -->
        <div class="message-bubble" :class="{ 'own-message': isOwnMessage(message) }">
          <!-- 文本内容 -->
          <div v-if="message.content" class="message-text">
            {{ message.content }}
          </div>
          
          <!-- 时间戳 -->
          <div class="message-time">
            {{ formatMessageTime(message.created_at) }}
          </div>
        </div>
      </div>
      
      <!-- 自己的头像 -->
      <div v-if="isOwnMessage(message)" class="avatar">
        <div class="avatar-circle own-avatar">
          {{ getUserInitials(message.sender) }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  message: {
    type: Object,
    required: true
  },
  currentUserId: {
    type: Number,
    required: true
  }
});

// 方法
function isOwnMessage(message) {
  return message.sender?.id === props.currentUserId;
}

function getUserInitials(sender) {
  if (!sender || !sender.username) return '?';
  return sender.username.charAt(0).toUpperCase();
}

function formatMessageTime(timestamp) {
  if (!timestamp) return '';
  
  const date = new Date(timestamp);
  const now = new Date();
  const diffInHours = (now - date) / (1000 * 60 * 60);
  
  if (diffInHours < 24) {
    return date.toLocaleTimeString('zh-CN', { 
      hour: '2-digit', 
      minute: '2-digit',
      hour12: false 
    });
  } else {
    return date.toLocaleDateString('zh-CN', { 
      month: 'short', 
      day: 'numeric',
      hour: '2-digit', 
      minute: '2-digit',
      hour12: false 
    });
  }
}
</script>

<style scoped>
.message-item {
  margin-bottom: 1rem;
}

.message-content {
  display: flex;
  align-items: flex-end;
  gap: 0.5rem;
}

.avatar {
  flex-shrink: 0;
}

.avatar-circle {
  width: 2.5rem;
  height: 2.5rem;
  border-radius: 50%;
  background: linear-gradient(135deg, #3b82f6, #8b5cf6);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 600;
  font-size: 0.875rem;
}

.own-avatar {
  background: linear-gradient(135deg, #10b981, #3b82f6);
}

.message-body {
  flex: 1;
  min-width: 0;
  max-width: 70%;
}

.sender-name {
  font-size: 0.75rem;
  color: #6b7280;
  margin-bottom: 0.25rem;
  padding-left: 0.75rem;
}

.message-bubble {
  background: white;
  border: 1px solid #e5e7eb;
  border-radius: 1rem;
  padding: 0.75rem 1rem;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

.message-bubble.own-message {
  background: #f1f5f9;
  border-color: #cbd5e1;
  margin-left: auto;
}

.message-text {
  color: #374151;
  line-height: 1.5;
  word-wrap: break-word;
}

.message-time {
  font-size: 0.75rem;
  color: #9ca3af;
  margin-top: 0.25rem;
  text-align: right;
}

.own-message .message-time {
  text-align: left;
}
</style> 