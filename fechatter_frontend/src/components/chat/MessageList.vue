<template>
  <div class="message-list-stable">
    <!-- 🔒 超稳定消息容器：使用CSS Grid固定布局 -->
    <div class="messages-container-stable" ref="scrollContainer">
      <!-- 🚀 历史消息加载指示器：固定位置 -->
      <div v-if="loadingHistory" class="loading-history-stable">
        <div class="loading-dots">
          <div></div>
          <div></div>
          <div></div>
        </div>
        <span>Loading older messages...</span>
      </div>

      <!-- 🚀 骨架屏加载状态：固定布局 -->
      <div v-if="loading && messages.length === 0" class="loading-state-stable">
        <div class="message-skeleton-stable" v-for="i in 3" :key="`skeleton-${i}`">
          <div class="skeleton-avatar-stable"></div>
          <div class="skeleton-content-stable">
            <div class="skeleton-line skeleton-line-name"></div>
            <div class="skeleton-line skeleton-line-text"></div>
            <div class="skeleton-line skeleton-line-short"></div>
          </div>
        </div>
      </div>

      <!-- 空状态：居中固定 -->
      <div v-else-if="messages.length === 0 && !loading" class="empty-state-stable">
        <h3>开始对话</h3>
        <p>发送第一条消息开始聊天</p>
      </div>

      <!-- 🔒 消息列表：绝对稳定的布局 -->
      <div v-else class="messages-stable" ref="messagesContainer">
        <!-- 🔒 超稳定消息项：固定尺寸，防止偏移 -->
        <div
          v-for="message in displayedMessages" 
          :key="`msg-${message.id || message.temp_id}`"
          :data-message-id="message.id || message.temp_id"
          class="message-wrapper-stable"
          :class="{ 
            'new-message': isNewMessage(message),
            'optimistic': message.isOptimistic,
            'failed': message.status === 'failed',
            'confirmed': message.status === 'confirmed'
          }"
        >
          <MessageItem 
            :message="message"
            :current-user-id="currentUserId"
            :chat-id="chatId"
            @user-profile-opened="handleUserProfileOpened"
            @dm-created="handleDMCreated"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, nextTick, watch, onMounted, onUnmounted, computed } from 'vue';
import MessageItem from './MessageItem.vue';
import { useChatStore } from '@/stores/chat';

const props = defineProps({
  messages: {
    type: Array,
    default: () => []
  },
  currentUserId: {
    type: Number,
    required: false,
    default: 0
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

const emit = defineEmits(['reply-message', 'user-profile-opened', 'dm-created', 'load-more-messages']);

const chatStore = useChatStore();

// 组件引用
const scrollContainer = ref(null);
const messagesContainer = ref(null);

// 🔒 稳定状态管理：避免频繁变更
const loadingHistory = ref(false);
const isUserScrolling = ref(false);
const lastScrollTop = ref(0);
const newMessageIds = ref(new Set());
const currentChatId = ref(null);
const lastMessageCount = ref(0);

// 🔒 防抖和节流：减少重排次数
let scrollTimeout = null;
let historyLoadTimeout = null;
let scrollDebounceTimeout = null;
let layoutStabilityTimeout = null;

// 🔒 布局稳定性监控
const layoutMetrics = ref({
  lastWidth: 0,
  lastHeight: 0,
  stable: true
});

// 🚀 稳定的消息显示：最小化DOM变更
const displayedMessages = computed(() => {
  // 🔒 聊天切换时的稳定处理
  if (currentChatId.value !== props.chatId) {
    newMessageIds.value.clear();
    currentChatId.value = props.chatId;
    lastMessageCount.value = props.messages.length;
  }
  
  // 🔒 稳定的新消息标记：避免频繁DOM操作
  if (props.messages.length > lastMessageCount.value) {
    const newMessages = props.messages.slice(lastMessageCount.value);
    newMessages.forEach(msg => {
      const messageId = msg.id || msg.temp_id;
      if (messageId) {
        newMessageIds.value.add(messageId);
        // 🔒 稳定的清理：使用setTimeout而不是动画
        setTimeout(() => {
          newMessageIds.value.delete(messageId);
        }, 2000); // 减少到2秒，降低内存占用
      }
    });
    lastMessageCount.value = props.messages.length;
  }
  
  return props.messages;
});

// 🔒 超稳定聊天切换：无动画，无偏移
const handleChatSwitch = () => {
  const container = scrollContainer.value;
  if (!container) return;

  // 🔒 重置状态：最小化变更
  isUserScrolling.value = false;
  newMessageIds.value.clear();
  lastScrollTop.value = 0;
  
  // 🔒 无动画切换：直接设置，避免任何中间状态
  container.style.opacity = '1';
  container.style.transform = 'none';
  
  // 🔒 立即滚动：无过渡，无动画
  nextTick(() => {
    scrollToBottomStable(false, true);
  });
};

// 🔒 监听聊天ID变化：使用防抖切换
watch(() => props.chatId, (newChatId, oldChatId) => {
  if (newChatId !== oldChatId && newChatId) {
    handleChatSwitch();
  }
}, { immediate: true });

// 🔒 稳定的新消息检测
const isNewMessage = (message) => {
  const messageId = message.id || message.temp_id;
  return messageId && newMessageIds.value.has(messageId);
};

// 🔒 超稳定滚动处理：最小化重排
const handleScrollStable = () => {
  const container = scrollContainer.value;
  if (!container) return;
  
  const scrollTop = container.scrollTop;
  const scrollHeight = container.scrollHeight;
  const clientHeight = container.clientHeight;
  
  // 🔒 防抖处理：减少计算频率
  if (scrollDebounceTimeout) {
    clearTimeout(scrollDebounceTimeout);
  }
  
  scrollDebounceTimeout = setTimeout(() => {
    // 🔒 稳定的滚动检测：避免微小变化
    const scrollDelta = Math.abs(scrollTop - lastScrollTop.value);
    if (scrollDelta > 10) { // 增大阈值，减少误判
      isUserScrolling.value = true;
    }
    lastScrollTop.value = scrollTop;
    
    // 🔒 稳定的底部检测
    if (scrollTop + clientHeight >= scrollHeight - 30) { // 减小阈值，提高精度
      isUserScrolling.value = false;
    }
    
    // 🔒 稳定的历史加载：增大阈值
    if (scrollTop <= 50 && !loadingHistory.value && chatStore.hasMoreMessages) {
      loadHistoryMessagesStable();
    }
  }, 32); // 降低到30fps，减少计算负担
};

// M2: 锚点保持 + 偏移补偿
// 记录锚点信息
const anchorState = ref({
  element: null,
  id: null,
  offsetFromTop: 0,
  savedAt: 0
});

// 保存当前视觉锚点
const saveVisualAnchor = () => {
  const container = scrollContainer.value;
  if (!container || !messagesContainer.value) return null;
  
  const containerRect = container.getBoundingClientRect();
  const messages = messagesContainer.value.querySelectorAll('[data-message-id]');
  
  // 找到视口中最接近顶部的消息作为锚点
  let anchorElement = null;
  let minDistance = Infinity;
  
  messages.forEach(msgEl => {
    const rect = msgEl.getBoundingClientRect();
    const distance = Math.abs(rect.top - containerRect.top);
    
    if (rect.top >= containerRect.top && rect.top < containerRect.bottom && distance < minDistance) {
      minDistance = distance;
      anchorElement = msgEl;
    }
  });
  
  if (anchorElement) {
    const messageId = anchorElement.getAttribute('data-message-id');
    const offsetFromTop = anchorElement.getBoundingClientRect().top - containerRect.top;
    
    anchorState.value = {
      element: anchorElement,
      id: messageId,
      offsetFromTop,
      savedAt: performance.now()
    };
    
    return anchorState.value;
  }
  
  return null;
};

// 恢复视觉锚点
const restoreVisualAnchor = () => {
  const container = scrollContainer.value;
  if (!container || !anchorState.value.id) return;
  
  const anchorElement = container.querySelector(`[data-message-id="${anchorState.value.id}"]`);
  if (!anchorElement) return;
  
  const containerRect = container.getBoundingClientRect();
  const anchorRect = anchorElement.getBoundingClientRect();
  const currentOffsetFromTop = anchorRect.top - containerRect.top;
  const scrollAdjustment = currentOffsetFromTop - anchorState.value.offsetFromTop;
  
  if (Math.abs(scrollAdjustment) > 1) {
    container.scrollTop += scrollAdjustment;
  }
};

// 🔒 超稳定历史消息加载：防止任何偏移
const loadHistoryMessagesStable = async () => {
  if (loadingHistory.value || !chatStore.hasMoreMessages) return;
  
  try {
    loadingHistory.value = true;
    const container = scrollContainer.value;
    if (!container) return;
    
    // M2: 保存视觉锚点
    const anchor = saveVisualAnchor();
    
    // 🔒 记录精确的布局状态
    const rect = container.getBoundingClientRect();
    const previousScrollHeight = container.scrollHeight;
    const previousScrollTop = container.scrollTop;
    
    // 🔒 禁用所有过渡和动画
    const originalTransition = container.style.transition;
    const originalScrollBehavior = container.style.scrollBehavior;
    container.style.transition = 'none';
    container.style.scrollBehavior = 'auto';
    
    // 加载数据
    emit('load-more-messages');
    await chatStore.fetchMoreMessages(props.chatId);
    
    // 🔒 多重DOM稳定确认
    await new Promise(resolve => requestAnimationFrame(resolve));
    await nextTick();
    await new Promise(resolve => requestAnimationFrame(resolve));
    
    // M2: 使用锚点恢复，如果失败则使用高度差补偿
    if (anchor) {
      restoreVisualAnchor();
    } else {
      // 🔒 精确的滚动位置恢复
      const newScrollHeight = container.scrollHeight;
      const heightDifference = newScrollHeight - previousScrollHeight;
      
      if (heightDifference > 0) {
        const targetScrollTop = previousScrollTop + heightDifference;
        container.scrollTop = targetScrollTop;
        
        // 🔒 验证位置是否正确
        const actualScrollTop = container.scrollTop;
        if (Math.abs(actualScrollTop - targetScrollTop) > 1) {
          console.warn('🔒 [MessageList] Scroll position adjustment needed');
          container.scrollTop = targetScrollTop;
        }
      }
    }
    
    // 恢复样式
    container.style.transition = originalTransition;
    container.style.scrollBehavior = originalScrollBehavior;
    
  } catch (error) {
    console.error('🔒 [MessageList] History load error:', error);
  } finally {
    loadingHistory.value = false;
    anchorState.value = { element: null, id: null, offsetFromTop: 0, savedAt: 0 };
  }
};

// 🔒 超稳定滚动到底部：无动画，无偏移
const scrollToBottomStable = (smooth = false, force = false) => {
  const container = scrollContainer.value;
  if (!container) return;
  
  // 🔒 强制无动画滚动
  if (isUserScrolling.value && !force) return;
  
  const scrollHeight = container.scrollHeight;
  const clientHeight = container.clientHeight;
  
  if (scrollHeight > clientHeight) {
    // 🔒 直接设置，无过渡
    container.scrollTop = scrollHeight - clientHeight;
  }
};

// 🔒 稳定的消息变化监听：减少响应频率
watch(() => props.messages.length, (newLength, oldLength) => {
  if (newLength > oldLength && !isUserScrolling.value) {
    // 🔒 延迟滚动，确保DOM稳定
    setTimeout(() => {
      scrollToBottomStable(false);
    }, 16); // 一帧的时间
  }
}, { immediate: false });

// 🔒 布局稳定性监控
const monitorLayoutStability = () => {
  const container = scrollContainer.value;
  if (!container) return;
  
  const rect = container.getBoundingClientRect();
  const currentWidth = rect.width;
  const currentHeight = rect.height;
  
  if (layoutMetrics.value.lastWidth > 0) {
    const widthDiff = Math.abs(currentWidth - layoutMetrics.value.lastWidth);
    const heightDiff = Math.abs(currentHeight - layoutMetrics.value.lastHeight);
    
    if (widthDiff > 1 || heightDiff > 1) {
      console.warn('🔒 [MessageList] Layout shift detected:', {
        widthDiff,
        heightDiff,
        stable: false
      });
      layoutMetrics.value.stable = false;
    } else {
      layoutMetrics.value.stable = true;
    }
  }
  
  layoutMetrics.value.lastWidth = currentWidth;
  layoutMetrics.value.lastHeight = currentHeight;
};

// 组件生命周期
onMounted(() => {
  const container = scrollContainer.value;
  if (container) {
    // 🔒 被动滚动监听，减少性能影响
    container.addEventListener('scroll', handleScrollStable, { passive: true });
  }
  
  // 🔒 初始化：无动画滚动
  nextTick(() => {
    scrollToBottomStable(false, true);
  });
  
  // 🔒 定期检查布局稳定性
  layoutStabilityTimeout = setInterval(monitorLayoutStability, 1000);
});

onUnmounted(() => {
  const container = scrollContainer.value;
  if (container) {
    container.removeEventListener('scroll', handleScrollStable);
  }
  
  // 🔒 清理所有定时器
  [scrollTimeout, historyLoadTimeout, scrollDebounceTimeout, layoutStabilityTimeout].forEach(timer => {
    if (timer) clearTimeout(timer);
  });
});

// 滚动到指定消息
const scrollToMessage = async (messageId) => {
  const container = scrollContainer.value;
  if (!container || !messageId) return;
  
  // 查找消息元素
  const messageElement = container.querySelector(`[data-message-id="${messageId}"]`);
  if (!messageElement) {
    console.warn('🔍 [MessageList] Message element not found:', messageId);
    return;
  }
  
  // 保存锚点状态
  const anchor = saveVisualAnchor();
  
  // 滚动到消息
  messageElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
  
  // 高亮消息
  messageElement.classList.add('search-highlight');
  setTimeout(() => {
    messageElement.classList.remove('search-highlight');
  }, 3000);
};

// 暴露稳定的方法
defineExpose({
  scrollToBottom: scrollToBottomStable,
  scrollToMessage,
  resetScrollState: () => {
    isUserScrolling.value = false;
    lastScrollTop.value = 0;
    newMessageIds.value.clear();
  }
});

// Event handlers
const handleUserProfileOpened = (user) => {
  emit('user-profile-opened', user);
};

const handleDMCreated = (dm) => {
  emit('dm-created', dm);
};
</script>

<style scoped>
/* 🔒 超稳定消息列表布局 */
.message-list-stable {
  /* 🔧 关键：完全填充父容器，无边距无padding */
  width: 100%;
  height: 100%;
  margin: 0;
  padding: 0;
  /* 🔧 阻止外部滚动 */
  overscroll-behavior: contain;
  border: none;
  outline: none;
  /* 🔧 隔离布局影响 */
  contain: layout style;
  /* 🔧 固定背景，防止继承变化 */
  background: #fdfcfa;
  /* 🔧 防止溢出 */
  overflow: hidden;
  /* 🔧 GPU加速 */
  transform: translateZ(0);
  /* 🔧 防止子元素影响 */
  isolation: isolate;
}

/* 🔒 超稳定消息容器 */
.messages-container-stable {
  /* 🔧 完全填充 */
  width: 100%;
  height: 100%;
  /* 🔧 独立滚动，隔离影响 */
  overflow-y: auto;
  overflow-x: hidden;
  /* 🔧 完全隔离 */
  contain: strict;
  /* 🔧 固定样式 */
  background: #fdfcfa;
  /* 🔧 禁用平滑滚动，防止性能问题 */
  scroll-behavior: auto;
  /* 🔧 防止过度滚动影响 */
  overscroll-behavior: contain;
  /* 🔧 性能优化 */
  will-change: scroll-position;
}

/* 🔒 稳定的消息内容区域 */
.messages-stable,
.loading-state-stable,
.empty-state-stable {
  /* 🔧 固定宽度，居中显示 */
  width: 100%;
  max-width: 720px;
  margin: 0 auto;
  padding: 0 1.5rem;
  /* 🔧 防止内容影响布局 */
  box-sizing: border-box;
  /* 🔧 最小高度，防止塌陷 */
  min-height: 100px;
}

/* 🔒 超稳定历史加载指示器 */
.loading-history-stable {
  /* 🔧 固定尺寸和位置 */
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 1rem;
  margin: 0.5rem auto;
  width: calc(100% - 3rem);
  max-width: 690px; /* 720px - 30px padding */
  /* 🔧 固定高度，防止抖动 */
  height: 56px;
  min-height: 56px;
  max-height: 56px;
  /* 🔧 稳定样式 */
  font-size: 0.8125rem;
  color: #6b6b6b;
  background: rgba(124, 58, 237, 0.03);
  border: 1px solid rgba(124, 58, 237, 0.08);
  border-radius: 0.5rem;
  /* 🔧 隔离变化 */
  contain: layout style size;
  /* 🔧 防止动画影响 */
  animation: none;
  transition: none;
}

/* 🔒 稳定的骨架屏 */
.loading-state-stable {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1rem 1.5rem;
}

.message-skeleton-stable {
  display: flex;
  gap: 0.75rem;
  /* 🔧 固定高度，防止抖动 */
  height: 64px;
  min-height: 64px;
  max-height: 64px;
  padding: 0.5rem 0;
  /* 🔧 隔离布局 */
  contain: layout style size;
}

.skeleton-avatar-stable {
  /* 🔧 固定尺寸 */
  width: 32px;
  height: 32px;
  min-width: 32px;
  min-height: 32px;
  max-width: 32px;
  max-height: 32px;
  border-radius: 0.375rem;
  background: linear-gradient(90deg, #f5f4f2 25%, #ebe9e6 50%, #f5f4f2 75%);
  background-size: 200% 100%;
  /* 🔧 减少动画，降低重排风险 */
  animation: shimmerStable 3s infinite linear;
  flex-shrink: 0;
  flex-grow: 0;
}

.skeleton-content-stable {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  /* 🔧 防止内容变化 */
  min-width: 0;
}

/* 🔒 超稳定消息包装器 */
.message-wrapper-stable {
  /* 🔧 固定布局，防止任何偏移 */
  width: 100%;
  /* 🔧 最小高度，防止塌陷 */
  min-height: 40px;
  /* 🔧 稳定间距 */
  margin-bottom: 0.25rem;
  padding: 0.375rem 0;
  /* 🔧 隔离变化 */
  contain: layout style;
  /* 🔧 防止变换影响 */
  transform: none;
  transition: none;
  /* 🔧 稳定边框 */
  border-bottom: 1px solid rgba(124, 58, 237, 0.04);
  /* 🔧 防止动画 */
  animation: none;
}

.message-wrapper-stable:last-child {
  border-bottom: none;
  /* 🔧 防止最后一个元素的边距影响 */
  margin-bottom: 0;
}

/* 🔒 稳定的消息状态样式 */
.message-wrapper-stable.optimistic {
  /* 🔧 最小化视觉变化 */
  opacity: 0.85;
  background: rgba(124, 58, 237, 0.02);
}

.message-wrapper-stable.failed {
  background: rgba(239, 68, 68, 0.05);
  border-left: 2px solid rgba(239, 68, 68, 0.3);
}

.message-wrapper-stable.confirmed {
  background: transparent;
}

/* 🔒 新消息指示：最小化动画 */
.message-wrapper-stable.new-message {
  background: rgba(124, 58, 237, 0.03);
  /* 🔧 移除可能导致重排的动画 */
  animation: none;
}

/* 🔒 稳定的空状态 */
.empty-state-stable {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  /* 🔧 固定高度，防止变化 */
  height: 200px;
  min-height: 200px;
  max-height: 200px;
  text-align: center;
  color: #6b6b6b;
  /* 🔧 隔离布局 */
  contain: layout style size;
}

.empty-state-stable h3 {
  font-size: 1rem;
  font-weight: 600;
  margin: 0 0 0.5rem 0;
  color: #3a3a3a;
}

.empty-state-stable p {
  margin: 0;
  opacity: 0.8;
}

/* 🔒 稳定的滚动条 */
.messages-container-stable::-webkit-scrollbar {
  width: 6px;
  /* 🔧 确保滚动条不影响布局 */
  position: absolute;
}

.messages-container-stable::-webkit-scrollbar-track {
  background: transparent;
}

.messages-container-stable::-webkit-scrollbar-thumb {
  background: rgba(124, 58, 237, 0.2);
  border-radius: 3px;
  /* 🔧 防止滚动条变化影响 */
  border: none;
  outline: none;
}

.messages-container-stable::-webkit-scrollbar-thumb:hover {
  background: rgba(124, 58, 237, 0.35);
}

/* 🔧 稳定的动画定义 */
@keyframes shimmerStable {
  0% { background-position: -200% 0; }
  100% { background-position: 200% 0; }
}

/* 🔧 减少动画的响应式适配 */
@media (prefers-reduced-motion: reduce) {
  .skeleton-avatar-stable,
  .message-wrapper-stable {
    animation: none;
    transition: none;
  }
}

/* 🔧 移动端稳定性 */
@media (max-width: 768px) {
  .messages-stable,
  .loading-state-stable,
  .empty-state-stable {
    max-width: 100%;
    padding: 0 1rem;
  }
}

/* 🔧 高对比度下的稳定性 */
@media (prefers-contrast: high) {
  .message-wrapper-stable {
    border-bottom-color: rgba(124, 58, 237, 0.15);
  }
}

/* 🔧 打印时的稳定性 */
@media print {
  .message-wrapper-stable {
    break-inside: avoid;
    page-break-inside: avoid;
  }
  
  .loading-history-stable,
  .loading-state-stable {
    display: none;
  }
}

/* 搜索高亮效果 */
.message-wrapper-stable.search-highlight {
  background: rgba(254, 243, 199, 0.8) !important;
  border-left: 3px solid #f59e0b;
  margin-left: -3px;
  animation: highlight-pulse 0.5s ease;
  transition: all 0.3s ease;
}

@keyframes highlight-pulse {
  0% {
    background: rgba(251, 191, 36, 0.4);
    transform: scale(1.01);
  }
  100% {
    background: rgba(254, 243, 199, 0.8);
    transform: scale(1);
  }
}
</style> 