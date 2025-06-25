<template>
  <div 
    class="virtual-message-list" 
    ref="scrollContainer"
    role="log"
    aria-label="Chat messages"
    aria-live="polite"
    aria-atomic="false"
  >
    <!-- M3: 虚拟滚动占位器 -->
    <div 
      class="virtual-spacer" 
      :style="{ height: `${totalHeight}px` }"
    >
      <!-- M3: 仅渲染可视范围内的消息 -->
      <div
        v-for="virtualItem in virtualItems"
        :key="virtualItem.id || virtualItem.temp_id"
        :data-virtual-item="true"
        :data-item-id="virtualItem.id || virtualItem.temp_id"
        :data-message-id="virtualItem.id || virtualItem.temp_id"
        class="virtual-message-wrapper"
        :style="{
          position: 'absolute',
          top: `${virtualItem.offset}px`,
          left: 0,
          right: 0,
          minHeight: `${virtualItem.height}px`
        }"
      >
        <MessageItem 
          :message="virtualItem"
          :current-user-id="currentUserId"
          :chat-id="chatId"
          @user-profile-opened="handleUserProfileOpened"
          @dm-created="handleDMCreated"
          @mounted="onMessageMounted(virtualItem)"
        />
      </div>
    </div>

    <!-- 加载指示器 - 改进的骨架屏 -->
    <div v-if="loadingHistory" class="loading-history-container">
      <!-- 消息骨架屏 -->
      <MessageSkeleton 
        :count="3" 
        :variant="'normal'"
        :animated="true"
        class="history-skeletons"
      />
      
      <!-- 传统加载指示器（作为后备） -->
      <div class="loading-history-fallback">
        <div class="loading-dots">
          <div></div>
          <div></div>
          <div></div>
        </div>
        <span>Loading older messages...</span>
      </div>
    </div>

    <!-- 性能指标（开发模式） -->
    <div v-if="showMetrics" class="performance-metrics">
      <div>Visible: {{ visibleRange.start }}-{{ visibleRange.end }} of {{ messages.length }}</div>
      <div>Cache hits: {{ performanceMetrics.cacheHits }}</div>
      <div>Render time: {{ performanceMetrics.lastRenderTime }}ms</div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, toRefs } from 'vue';
import MessageItem from './MessageItem.vue';
import MessageSkeleton from '@/components/ui/MessageSkeleton.vue';
import { useChatStore } from '@/stores/chat';
import { useVirtualScroll } from '@/composables/useVirtualScroll';

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
  },
  chatId: {
    type: [Number, String],
    default: null
  }
});

const emit = defineEmits(['user-profile-opened', 'dm-created', 'load-more-messages']);

const chatStore = useChatStore();
const scrollContainer = ref(null);
const loadingHistory = ref(false);
const showMetrics = ref(import.meta.env.DEV);

// M3: 使用虚拟滚动
const {
  virtualItems,
  totalHeight,
  visibleRange,
  scrollToItem,
  setMeasuredHeight,
  performanceMetrics
} = useVirtualScroll({
  items: toRefs(props).messages,
  containerRef: scrollContainer,
  itemHeight: 80, // 估算的消息高度
  overscan: 3 // 额外渲染3个项目
});

// 消息挂载时测量高度
const onMessageMounted = (virtualItem) => {
  // 延迟测量以确保渲染完成
  requestAnimationFrame(() => {
    const element = scrollContainer.value?.querySelector(
      `[data-item-id="${virtualItem.id || virtualItem.temp_id}"]`
    );
    
    if (element) {
      const height = element.offsetHeight;
      setMeasuredHeight(virtualItem.id || virtualItem.temp_id, height);
    }
  });
};

// 监听滚动以加载历史消息
const handleScroll = () => {
  const container = scrollContainer.value;
  if (!container) return;
  
  // 接近顶部时加载更多
  if (container.scrollTop <= 100 && !loadingHistory.value && chatStore.hasMoreMessages) {
    loadMoreMessages();
  }
};

// 加载更多消息
const loadMoreMessages = async () => {
  if (loadingHistory.value) return;
  
  try {
    loadingHistory.value = true;
    
    // 保存当前滚动状态
    const container = scrollContainer.value;
    const previousHeight = totalHeight.value;
    const previousScroll = container.scrollTop;
    
    // 加载消息
    emit('load-more-messages');
    await chatStore.fetchMoreMessages(props.chatId);
    
    // 等待虚拟列表更新
    await new Promise(resolve => requestAnimationFrame(resolve));
    
    // 调整滚动位置
    const heightDiff = totalHeight.value - previousHeight;
    if (heightDiff > 0) {
      container.scrollTop = previousScroll + heightDiff;
    }
    
  } finally {
    loadingHistory.value = false;
  }
};

// 滚动到底部
const scrollToBottom = (smooth = false) => {
  if (props.messages.length > 0) {
    scrollToItem(props.messages.length - 1, 'end');
  }
};

// 监听新消息
watch(() => props.messages.length, (newLength, oldLength) => {
  if (newLength > oldLength) {
    // 延迟滚动以确保高度计算完成
    requestAnimationFrame(() => {
      scrollToBottom();
    });
  }
});

// Event handlers
const handleUserProfileOpened = (user) => {
  emit('user-profile-opened', user);
};

const handleDMCreated = (dm) => {
  emit('dm-created', dm);
};

// 组件挂载
onMounted(() => {
  const container = scrollContainer.value;
  if (container) {
    container.addEventListener('scroll', handleScroll, { passive: true });
    
    // 初始滚动到底部
    scrollToBottom();
  }
});

// 暴露方法
defineExpose({
  scrollToBottom,
  scrollToItem
});
</script>

<style scoped>
.virtual-message-list {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  position: relative;
  background: #fdfcfa;
  
  /* 性能优化 */
  contain: layout style paint;
  will-change: scroll-position;
}

.virtual-spacer {
  position: relative;
  width: 100%;
  /* 高度由 totalHeight 动态设置 */
}

.virtual-message-wrapper {
  width: 100%;
  max-width: 720px;
  margin: 0 auto;
  padding: 0 1.5rem;
  box-sizing: border-box;
  
  /* 防止内容影响虚拟滚动计算 */
  contain: layout style;
}

.loading-history-container {
  position: sticky;
  top: 0;
  width: 100%;
  z-index: 10;
  background: linear-gradient(to bottom, rgba(253, 252, 250, 0.9), rgba(253, 252, 250, 0.7));
  backdrop-filter: blur(8px);
  border-bottom: 1px solid rgba(124, 58, 237, 0.08);
}

.history-skeletons {
  padding: 1rem;
  max-width: 720px;
  margin: 0 auto;
}

.loading-history-fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  margin: 0 auto;
  max-width: 690px;
  font-size: 0.75rem;
  color: #6b6b6b;
  opacity: 0.8;
}

.loading-dots {
  display: flex;
  gap: 0.25rem;
}

.loading-dots div {
  width: 6px;
  height: 6px;
  background: rgba(124, 58, 237, 0.4);
  border-radius: 50%;
  animation: dot-pulse 1.5s ease-in-out infinite;
}

.loading-dots div:nth-child(2) {
  animation-delay: 0.15s;
}

.loading-dots div:nth-child(3) {
  animation-delay: 0.3s;
}

@keyframes dot-pulse {
  0%, 80%, 100% {
    opacity: 0.4;
    transform: scale(1);
  }
  40% {
    opacity: 1;
    transform: scale(1.2);
  }
}

.performance-metrics {
  position: fixed;
  bottom: 20px;
  right: 20px;
  background: rgba(0, 0, 0, 0.8);
  color: #00ff00;
  font-family: monospace;
  font-size: 12px;
  padding: 10px;
  border-radius: 4px;
  z-index: 9999;
}

/* 滚动条样式 */
.virtual-message-list::-webkit-scrollbar {
  width: 6px;
}

.virtual-message-list::-webkit-scrollbar-track {
  background: transparent;
}

.virtual-message-list::-webkit-scrollbar-thumb {
  background: rgba(124, 58, 237, 0.2);
  border-radius: 3px;
}

.virtual-message-list::-webkit-scrollbar-thumb:hover {
  background: rgba(124, 58, 237, 0.35);
}

/* 移动端优化 */
@media (max-width: 768px) {
  .virtual-message-wrapper {
    max-width: 100%;
    padding: 0 1rem;
  }
}
</style>