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

    <!-- 可拖动Debug Panel -->
    <DraggableDebugPanel 
      v-if="showDebugPanel"
      :visible="showDebugPanel"
      @close="showDebugPanel = false"
    />

    <!-- Debug控制按钮 -->
    <button 
      v-if="isDevelopment"
      class="debug-toggle-btn" 
      @click="showDebugPanel = !showDebugPanel"
      :class="{ active: showDebugPanel }"
    >
      🐛
    </button>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import DraggableDebugPanel from '@/components/debug/DraggableDebugPanel.vue';

// 开发环境检测
const isDevelopment = ref(process.env.NODE_ENV === 'development');
const showDebugPanel = ref(false);

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
/* 🔒 超稳定布局基础 */
.stable-layout-container {
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  position: fixed;
  top: 0;
  left: 0;
  /* 🔧 关键：使用contain来隔离布局影响 */
  contain: layout style paint;
  /* 🔧 启用硬件加速，减少重排 */
  transform: translateZ(0);
  will-change: auto;
}

/* 🔒 Grid布局：绝对稳定，永不偏移 */
.chat-grid-layout {
  display: grid;
  grid-template-columns: 280px 1fr; /* 固定左侧宽度，右侧自适应 */
  height: 100vh;
  width: 100vw;
  /* 🔧 防止任何形式的偏移 */
  position: fixed;
  top: 0;
  left: 0;
  /* 🔧 隔离布局影响 */
  contain: layout style;
  /* 🔧 禁用所有可能导致偏移的属性 */
  transform: none;
  margin: 0;
  padding: 0;
  border: none;
  outline: none;
}

/* 🔒 左侧边栏：绝对固定 */
.chat-sidebar-stable {
  grid-column: 1;
  height: 100vh;
  width: 280px; /* 固定宽度 */
  overflow-y: auto;
  overflow-x: hidden;
  /* 🔧 完全隔离，防止影响其他区域 */
  contain: strict;
  /* 🔧 固定定位，防止任何偏移 */
  position: relative;
  /* 🔧 防止内容溢出导致布局变化 */
  box-sizing: border-box;
  /* 🔧 防止子元素影响 */
  isolation: isolate;
  /* 🔧 优化滚动性能 */
  scroll-behavior: auto;
  overscroll-behavior: contain;
}

/* 🔒 右侧主区域：弹性稳定 */
.chat-main-stable {
  grid-column: 2;
  display: grid;
  grid-template-rows: 60px 1fr 80px; /* 头部60px，内容自适应，输入80px */
  height: 100vh;
  /* 🔧 防止溢出和偏移 */
  contain: layout style;
  overflow: hidden;
  position: relative;
  /* 🔧 确保子元素不会逃逸 */
  isolation: isolate;
}

/* 🔒 聊天头部：固定高度，绝不改变 */
.chat-header-stable {
  grid-row: 1;
  height: 60px; /* 固定高度 */
  /* 🔧 防止内容影响高度 */
  min-height: 60px;
  max-height: 60px;
  overflow: hidden;
  /* 🔧 隔离布局 */
  contain: layout style size;
  /* 🔧 防止任何形式的变化 */
  flex-shrink: 0;
  flex-grow: 0;
  position: relative;
  z-index: 1;
}

/* 🔒 消息区域：占满剩余空间，独立滚动 */
.message-area-stable {
  grid-row: 2;
  /* 🔧 关键：使用flex: 1来占满剩余空间 */
  flex: 1;
  min-height: 0; /* 重要：允许flex子元素收缩 */
  overflow: hidden;
  /* 🔧 完全隔离滚动区域 */
  contain: strict;
  position: relative;
  /* 🔧 确保内容不会影响外部布局 */
  isolation: isolate;
}

/* 🔒 输入区域：固定高度，绝不改变 */
.chat-input-stable {
  grid-row: 3;
  height: 80px; /* 固定高度 */
  /* 🔧 防止内容影响高度 */
  min-height: 80px;
  max-height: 80px;
  overflow: hidden;
  /* 🔧 隔离布局 */
  contain: layout style size;
  /* 🔧 防止任何形式的变化 */
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

/* 🔧 Debug控制按钮：固定位置 */
.debug-toggle-btn {
  position: fixed;
  top: 180px; /* 向下移动，避免遮挡其他元素 */
  right: 20px;
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: rgba(124, 58, 237, 0.9);
  border: 2px solid rgba(124, 58, 237, 0.3);
  color: white;
  font-size: 20px;
  cursor: pointer;
  z-index: 9998;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  backdrop-filter: blur(10px);
  /* 🔧 确保按钮不影响布局 */
  contain: strict;
  user-select: none;
}

.debug-toggle-btn:hover {
  background: rgba(124, 58, 237, 1);
  transform: scale(1.05);
  box-shadow: 0 4px 12px rgba(124, 58, 237, 0.4);
}

.debug-toggle-btn.active {
  background: rgba(239, 68, 68, 0.9);
  border-color: rgba(239, 68, 68, 0.3);
}

/* 🔧 响应式：保持稳定性 */
@media (max-width: 768px) {
  .chat-grid-layout {
    grid-template-columns: 1fr; /* 移动端单列布局 */
  }
  
  .chat-sidebar-stable {
    display: none; /* 移动端隐藏侧边栏 */
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