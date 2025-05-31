<template>
  <div class="message-list">
    <!-- 消息容器 -->
    <div class="messages-container" ref="scrollContainer">
      <!-- 空状态 -->
      <div v-if="messages.length === 0 && !loading" class="empty-state">
        <h3>开始对话</h3>
        <p>发送第一条消息开始聊天</p>
      </div>

      <!-- 加载状态 -->
      <div v-if="loading" class="loading-state">
        <div class="loading-dots">
          <div></div>
          <div></div>
          <div></div>
        </div>
        <p>加载消息中...</p>
      </div>

      <!-- 消息列表 -->
      <div v-if="messages.length > 0" class="messages">
        <MessageItem 
          v-for="message in messages" 
          :key="message.id" 
          :message="message"
          :current-user-id="currentUserId"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, nextTick, watch } from 'vue';
import MessageItem from './MessageItem.vue';

const props = defineProps({
  messages: {
    type: Array,
    default: () => []
  },
  currentUserId: {
    type: Number,
    required: true
  },
  loading: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits(['reply-message']);

// 组件引用
const scrollContainer = ref(null);

// 滚动到底部
const scrollToBottom = (smooth = true) => {
  nextTick(() => {
    const container = scrollContainer.value;
    if (container) {
      container.scrollTo({
        top: container.scrollHeight,
        behavior: smooth ? 'smooth' : 'instant'
      });
    }
  });
};

// 监听消息变化，自动滚动到底部
watch(() => props.messages.length, () => {
  scrollToBottom();
});

// 暴露方法给父组件
defineExpose({
  scrollToBottom
});
</script>

<style scoped>
.message-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: linear-gradient(to bottom, rgba(243, 244, 246, 0.3), white);
}

.messages-container {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  text-align: center;
  color: #6b7280;
}

.empty-state h3 {
  font-size: 1.125rem;
  font-weight: 500;
  margin-bottom: 0.5rem;
  color: #374151;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  text-align: center;
}

.loading-dots {
  display: flex;
  gap: 0.25rem;
  margin-bottom: 1rem;
}

.loading-dots div {
  width: 0.5rem;
  height: 0.5rem;
  background: linear-gradient(135deg, #3b82f6, #8b5cf6);
  border-radius: 50%;
  animation: bounce 1.4s ease-in-out infinite both;
}

.loading-dots div:nth-child(1) { animation-delay: -0.32s; }
.loading-dots div:nth-child(2) { animation-delay: -0.16s; }

@keyframes bounce {
  0%, 80%, 100% {
    transform: scale(0);
  }
  40% {
    transform: scale(1);
  }
}

.messages {
  max-width: 64rem;
  margin: 0 auto;
}

/* 自定义滚动条 */
.messages-container::-webkit-scrollbar {
  width: 6px;
}

.messages-container::-webkit-scrollbar-track {
  background: transparent;
}

.messages-container::-webkit-scrollbar-thumb {
  background: rgba(156, 163, 175, 0.3);
  border-radius: 3px;
}

.messages-container::-webkit-scrollbar-thumb:hover {
  background: rgba(156, 163, 175, 0.5);
}
</style> 