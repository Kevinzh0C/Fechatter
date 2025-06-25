<template>
  <div class="stable-layout-container">
    <!-- 🔒 超稳定聊天布局：Grid + 固定定位 -->
    <div class="chat-grid-layout">
      <!-- 左侧聊天列表：固定宽度，绝对稳定 -->
      <div class="chat-sidebar-stable">
        <slot name="sidebar" />
      </div>

      <!-- 右侧聊天区域：弹性宽度，稳定布局 -->
      <div class="chat-main-stable">
        <!-- 聊天头部：固定高度 -->
        <div class="chat-header-stable">
          <slot name="header" />
        </div>

        <!-- 消息列表：占满剩余空间，独立滚动 -->
        <div class="message-area-stable">
          <slot name="messages" />
        </div>

        <!-- 输入区域：固定高度 -->
        <div class="chat-input-stable">
          <slot name="input" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { onMounted } from 'vue';

// Debug panel removed for production

// 检测布局稳定性
const detectLayoutShifts = () => {
  if ('LayoutShift' in window) {
    const observer = new PerformanceObserver((list) => {
      for (const entry of list.getEntries()) {
        if (entry.hadRecentInput) continue;

        console.warn('🚨 Layout shift detected:', {
          value: entry.value,
          sources: entry.sources,
          lastInputTime: entry.lastInputTime
        });
      }
    });

    observer.observe({ entryTypes: ['layout-shift'] });
  }
};

onMounted(() => {
  detectLayoutShifts();
});
</script>

<style scoped>
/* 🔒 弹性布局基础 - 支持内容扩展 */
.stable-layout-container {
  min-height: 100vh;
  width: 100vw;
  overflow: auto;
  position: relative;
  top: 0;
  left: 0;
  contain: layout style paint;
  transform: translateZ(0);
  will-change: auto;
}

/* 🔒 Grid布局：弹性稳定，支持内容扩展 */
.chat-grid-layout {
  display: grid;
  grid-template-columns: 280px 1fr;
  min-height: 100vh;
  width: 100vw;
  position: relative;
  contain: layout style;
  margin: 0;
  padding: 0;
  border: none;
  outline: none;
}

/* 🔒 左侧边栏：固定宽度，支持滚动 */
.chat-sidebar-stable {
  grid-column: 1;
  height: 100vh;
  width: 280px;
  overflow-y: auto;
  overflow-x: hidden;
  contain: layout style;
  position: relative;
  box-sizing: border-box;
  isolation: isolate;
  scroll-behavior: auto;
  overscroll-behavior: contain;
}

/* 🔒 右侧主区域：弹性布局，支持内容扩展 */
.chat-main-stable {
  grid-column: 2;
  display: grid;
  grid-template-rows: 60px 1fr 80px;
  min-height: 100vh;
  contain: layout style;
  overflow: visible;
  position: relative;
  isolation: isolate;
}

/* 🔒 聊天头部：固定高度 */
.chat-header-stable {
  grid-row: 1;
  height: 60px;
  min-height: 60px;
  max-height: 60px;
  overflow: hidden;
  contain: layout style size;
  flex-shrink: 0;
  flex-grow: 0;
  position: relative;
  z-index: 1;
}

/* 🔒 消息区域：弹性扩展，支持内容滚动 */
.message-area-stable {
  grid-row: 2;
  flex: 1;
  min-height: 0;
  overflow: auto;
  contain: layout style;
  position: relative;
  isolation: isolate;
}

/* 🔒 输入区域：固定高度 */
.chat-input-stable {
  grid-row: 3;
  height: 80px;
  min-height: 80px;
  max-height: 80px;
  overflow: hidden;
  contain: layout style size;
  flex-shrink: 0;
  flex-grow: 0;
  position: relative;
  z-index: 1;
}

/* 🔒 滚动条优化：防止布局影响 */
.chat-sidebar-stable::-webkit-scrollbar,
.message-area-stable::-webkit-scrollbar {
  width: 6px;
  /* 🔧 确保滚动条不占用内容空间 */
  position: absolute;
}

.chat-sidebar-stable::-webkit-scrollbar-track,
.message-area-stable::-webkit-scrollbar-track {
  background: transparent;
}

.chat-sidebar-stable::-webkit-scrollbar-thumb,
.message-area-stable::-webkit-scrollbar-thumb {
  background: rgba(124, 58, 237, 0.2);
  border-radius: 3px;
  /* 🔧 防止滚动条变化影响布局 */
  border: none;
  outline: none;
}

/* 🔧 响应式：保持稳定性 */
@media (max-width: 768px) {
  .chat-grid-layout {
    grid-template-columns: 1fr;
    /* 移动端单列布局 */
  }

  .chat-sidebar-stable {
    display: none;
    /* 移动端隐藏侧边栏 */
  }

  .chat-main-stable {
    grid-column: 1;
  }
}

/* 🔧 强制GPU加速，减少重排 */
.chat-grid-layout,
.chat-sidebar-stable,
.chat-main-stable,
.chat-header-stable,
.message-area-stable,
.chat-input-stable {
  transform: translateZ(0);
  backface-visibility: hidden;
  perspective: 1000px;
}

/* 🔧 防止字体加载导致的布局偏移 */
.stable-layout-container {
  font-display: swap;
  font-synthesis: none;
}

/* 🔧 禁用可能导致布局变化的动画 */
@media (prefers-reduced-motion: reduce) {
  .debug-toggle-btn {
    transition: none;
    transform: none;
  }

  .debug-toggle-btn:hover {
    transform: none;
  }
}

/* 🔧 高对比度模式下的稳定性 */
@media (prefers-contrast: high) {
  .debug-toggle-btn {
    border-width: 3px;
  }
}

/* 🔧 打印时的布局稳定性 */
@media print {
  .stable-layout-container {
    position: static;
    height: auto;
    contain: none;
  }

  .chat-grid-layout {
    position: static;
    display: block;
  }

  .debug-toggle-btn {
    display: none;
  }
}
</style>