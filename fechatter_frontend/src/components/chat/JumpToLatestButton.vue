<template>
  <Transition name="fab">
    <button v-if="showButton" @click="jumpToLatest" class="jump-to-latest-fab"
      :class="{ 'has-unread': unreadCount > 0 }"
      :aria-label="`Jump to latest message${unreadCount > 0 ? ` (${unreadCount} unread)` : ''}`">
      <!-- 主图标 -->
      <svg class="fab-icon" width="20" height="20" viewBox="0 0 20 20" fill="currentColor">
        <path d="M10 14l-5-5 1.41-1.41L10 11.17l3.59-3.58L15 9l-5 5z" />
      </svg>

      <!-- 未读计数徽章 -->
      <span v-if="unreadCount > 0" class="unread-badge">
        {{ unreadCount > 99 ? '99+' : unreadCount }}
      </span>

      <!-- 涟漪效果 -->
      <span class="fab-ripple"></span>
    </button>
  </Transition>
</template>

<script setup>
import { ref, onMounted, onUnmounted, computed } from 'vue';

const props = defineProps({
  scrollContainer: {
    type: Object,
    default: null
  },
  unreadCount: {
    type: Number,
    default: 0
  },
  threshold: {
    type: Number,
    default: 0.9 // 当最后一条消息90%不可见时显示按钮
  }
});

const emit = defineEmits(['jump-to-latest']);

// M5: IntersectionObserver 监控
const showButton = ref(false);
let observer = null;
let latestAnchor = null;

// 🎯 NEW: 动态位置计算
const buttonPosition = ref({
  bottom: 24,
  right: 24
});

// 费兹定律优化：按钮大小和位置
const FAB_SIZE = 56; // px
const FAB_MARGIN = 24; // px
const MIN_BOTTOM_CLEARANCE = 80; // 最小底部间距

/**
 * 🎯 动态计算按钮位置，避开其他组件
 */
const calculateOptimalPosition = () => {
  const viewport = {
    width: window.innerWidth,
    height: window.innerHeight
  };

  let optimalBottom = FAB_MARGIN;
  let optimalRight = FAB_MARGIN;

  // 1. 检测消息输入框
  const messageInput = document.querySelector('.message-input, .input-container, [class*="input"]');
  if (messageInput) {
    const inputRect = messageInput.getBoundingClientRect();
    const inputHeight = inputRect.height;

    // 确保按钮在输入框上方至少80px
    const requiredBottom = viewport.height - inputRect.top + MIN_BOTTOM_CLEARANCE;
    optimalBottom = Math.max(optimalBottom, requiredBottom);
  }

  // 2. 检测其他浮动组件
  const floatingElements = document.querySelectorAll('.floating-toolbar, .emoji-picker, .modal, [class*="floating"]');
  floatingElements.forEach(element => {
    const rect = element.getBoundingClientRect();
    const isVisible = rect.width > 0 && rect.height > 0;

    if (isVisible) {
      // 如果浮动组件在右下角区域，向上调整按钮位置
      const isInBottomRight = rect.right > viewport.width - 200 && rect.bottom > viewport.height - 200;
      if (isInBottomRight) {
        const requiredBottom = viewport.height - rect.top + 20;
        optimalBottom = Math.max(optimalBottom, requiredBottom);
      }
    }
  });

  // 3. 检测滚动条（如果可见）
  if (props.scrollContainer) {
    const hasVerticalScrollbar = props.scrollContainer.scrollHeight > props.scrollContainer.clientHeight;
    if (hasVerticalScrollbar) {
      // 为滚动条留出空间
      optimalRight = Math.max(optimalRight, FAB_MARGIN + 20);
    }
  }

  // 4. 边界检查
  optimalBottom = Math.min(optimalBottom, viewport.height - FAB_SIZE - 20);
  optimalRight = Math.min(optimalRight, viewport.width - FAB_SIZE - 20);

  // 5. 更新位置
  buttonPosition.value = {
    bottom: Math.max(optimalBottom, FAB_MARGIN),
    right: Math.max(optimalRight, FAB_MARGIN)
  };

  if (import.meta.env.DEV) {
    console.log('🎯 [JumpToLatest] Position calculated:', buttonPosition.value);
  }
};

// 🎯 监听布局变化
let resizeObserver = null;
let mutationObserver = null;

const setupPositionTracking = () => {
  // 计算初始位置
  calculateOptimalPosition();

  // 监听窗口大小变化
  window.addEventListener('resize', calculateOptimalPosition);

  // 监听滚动容器变化
  if (props.scrollContainer) {
    props.scrollContainer.addEventListener('scroll', () => {
      // 滚动时重新计算（防抖）
      clearTimeout(scrollTimeout);
      scrollTimeout = setTimeout(calculateOptimalPosition, 100);
    });
  }

  // 使用ResizeObserver监听输入框大小变化
  if (typeof ResizeObserver !== 'undefined') {
    resizeObserver = new ResizeObserver(() => {
      calculateOptimalPosition();
    });

    // 观察输入框
    const messageInput = document.querySelector('.message-input, .input-container');
    if (messageInput) {
      resizeObserver.observe(messageInput);
    }
  }

  // 使用MutationObserver监听DOM变化
  mutationObserver = new MutationObserver(() => {
    // 延迟重新计算，避免频繁更新
    clearTimeout(mutationTimeout);
    mutationTimeout = setTimeout(calculateOptimalPosition, 200);
  });

  mutationObserver.observe(document.body, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: ['class', 'style']
  });
};

let scrollTimeout = null;
let mutationTimeout = null;

/**
 * 设置 IntersectionObserver
 */
const setupObserver = () => {
  if (!props.scrollContainer) return;

  // 查找或创建锚点元素
  latestAnchor = props.scrollContainer.querySelector('#latest-anchor');

  if (!latestAnchor) {
    // 创建锚点
    latestAnchor = document.createElement('div');
    latestAnchor.id = 'latest-anchor';
    latestAnchor.style.cssText = 'height: 1px; visibility: hidden;';
    props.scrollContainer.appendChild(latestAnchor);
  }

  // 配置观察器
  const options = {
    root: props.scrollContainer,
    rootMargin: '0px',
    threshold: [0, props.threshold] // 监控完全不可见和阈值
  };

  observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      // 当锚点不可见或可见度低于阈值时显示按钮
      const isInvisible = entry.intersectionRatio < props.threshold;

      // 添加滞后以避免频繁切换
      if (isInvisible !== showButton.value) {
        setTimeout(() => {
          showButton.value = isInvisible;
        }, 100);
      }
    });
  }, options);

  // 开始观察
  observer.observe(latestAnchor);
};

/**
 * 更新锚点位置
 */
const updateAnchorPosition = () => {
  if (!latestAnchor || !props.scrollContainer) return;

  // 将锚点移动到最后一条消息后
  const messages = props.scrollContainer.querySelectorAll('.message-wrapper-stable, .message-item');
  if (messages.length > 0) {
    const lastMessage = messages[messages.length - 1];
    lastMessage.insertAdjacentElement('afterend', latestAnchor);
  }
};

/**
 * 跳转到最新消息
 */
const jumpToLatest = () => {
  if (!props.scrollContainer) return;

  // 添加点击反馈
  const button = document.querySelector('.jump-to-latest-fab');
  if (button) {
    button.classList.add('clicked');
    setTimeout(() => button.classList.remove('clicked'), 300);
  }

  // 平滑滚动到底部
  const scrollHeight = props.scrollContainer.scrollHeight;
  const clientHeight = props.scrollContainer.clientHeight;
  const targetScroll = scrollHeight - clientHeight;

  // 使用原生平滑滚动
  props.scrollContainer.scrollTo({
    top: targetScroll,
    behavior: 'smooth'
  });

  // 触发事件
  emit('jump-to-latest');
};

// 监听消息变化以更新锚点位置
const observeMessages = () => {
  if (!props.scrollContainer) return;

  const messagesObserver = new MutationObserver(() => {
    updateAnchorPosition();
  });

  messagesObserver.observe(props.scrollContainer, {
    childList: true,
    subtree: true
  });

  return messagesObserver;
};

// 生命周期
onMounted(() => {
  if (props.scrollContainer) {
    setupObserver();
    updateAnchorPosition();

    // 🎯 NEW: 设置位置跟踪
    setupPositionTracking();

    // 监听消息变化
    const messagesObserver = observeMessages();

    // 清理函数
    onUnmounted(() => {
      if (observer) {
        observer.disconnect();
      }
      if (messagesObserver) {
        messagesObserver.disconnect();
      }

      // 🎯 NEW: 清理位置跟踪
      window.removeEventListener('resize', calculateOptimalPosition);

      if (resizeObserver) {
        resizeObserver.disconnect();
      }

      if (mutationObserver) {
        mutationObserver.disconnect();
      }

      // 清理定时器
      if (scrollTimeout) {
        clearTimeout(scrollTimeout);
      }
      if (mutationTimeout) {
        clearTimeout(mutationTimeout);
      }
    });
  }
});

// 监听容器变化
const containerWatcher = computed(() => props.scrollContainer);
containerWatcher.value && setupObserver();
</script>

<style scoped>
/* M5: Jump-to-Latest FAB - 费兹定律优化 */
.jump-to-latest-fab {
  position: fixed;
  bottom: v-bind('buttonPosition.bottom + "px"');
  right: v-bind('buttonPosition.right + "px"');
  width: v-bind('FAB_SIZE + "px"');
  height: v-bind('FAB_SIZE + "px"');
  border-radius: 50%;
  background: white;
  border: none;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15),
    0 2px 4px rgba(0, 0, 0, 0.1);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  color: #7c3aed;
  overflow: hidden;

  /* 增大点击区域（费兹定律） */
  &::before {
    content: '';
    position: absolute;
    inset: -8px;
  }
}

.jump-to-latest-fab:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(124, 58, 237, 0.3),
    0 4px 8px rgba(0, 0, 0, 0.1);
  background: #7c3aed;
  color: white;
}

.jump-to-latest-fab:active {
  transform: translateY(0);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.jump-to-latest-fab.clicked {
  animation: click-pulse 0.3s ease-out;
}

/* 图标 */
.fab-icon {
  position: relative;
  z-index: 2;
  transition: transform 0.3s ease;
}

.jump-to-latest-fab:hover .fab-icon {
  transform: translateY(2px);
}

/* 未读徽章 */
.unread-badge {
  position: absolute;
  top: -4px;
  right: -4px;
  min-width: 20px;
  height: 20px;
  padding: 0 6px;
  background: #ef4444;
  color: white;
  font-size: 11px;
  font-weight: 600;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  z-index: 3;
  animation: badge-bounce 0.3s ease;
}

.jump-to-latest-fab.has-unread {
  background: #7c3aed;
  color: white;
  animation: attention-pulse 2s ease-in-out infinite;
}

/* 涟漪效果 */
.fab-ripple {
  position: absolute;
  inset: 0;
  border-radius: 50%;
  background: currentColor;
  opacity: 0;
  transform: scale(0);
  z-index: 1;
}

.jump-to-latest-fab:active .fab-ripple {
  animation: ripple 0.6s ease-out;
}

/* 过渡动画 */
.fab-enter-active,
.fab-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.fab-enter-from {
  opacity: 0;
  transform: scale(0) translateY(20px);
}

.fab-leave-to {
  opacity: 0;
  transform: scale(0.8) translateY(10px);
}

/* 动画定义 */
@keyframes click-pulse {
  0% {
    transform: scale(1);
  }

  50% {
    transform: scale(0.95);
  }

  100% {
    transform: scale(1);
  }
}

@keyframes badge-bounce {

  0%,
  100% {
    transform: scale(1);
  }

  50% {
    transform: scale(1.1);
  }
}

@keyframes attention-pulse {

  0%,
  100% {
    box-shadow: 0 4px 12px rgba(124, 58, 237, 0.4),
      0 2px 4px rgba(0, 0, 0, 0.1);
  }

  50% {
    box-shadow: 0 4px 20px rgba(124, 58, 237, 0.6),
      0 2px 8px rgba(0, 0, 0, 0.15);
  }
}

@keyframes ripple {
  to {
    opacity: 0.3;
    transform: scale(1);
  }
}

/* 响应式调整 */
@media (max-width: 640px) {
  .jump-to-latest-fab {
    bottom: 16px;
    right: 16px;
    width: 48px;
    height: 48px;
  }

  .fab-icon {
    width: 18px;
    height: 18px;
  }
}

/* 暗色模式 */
@media (prefers-color-scheme: dark) {
  .jump-to-latest-fab {
    background: #2a2a2a;
    color: #e0e0e0;
  }

  .jump-to-latest-fab:hover {
    background: #7c3aed;
    color: white;
  }

  .jump-to-latest-fab.has-unread {
    background: #7c3aed;
  }
}

/* 减少动画 */
@media (prefers-reduced-motion: reduce) {

  .jump-to-latest-fab,
  .fab-icon,
  .unread-badge {
    transition: none;
    animation: none;
  }

  .fab-enter-active,
  .fab-leave-active {
    transition: opacity 0.2s;
  }

  .fab-enter-from,
  .fab-leave-to {
    transform: none;
  }
}
</style>