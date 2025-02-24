<template>
  <div v-if="visible" ref="debugPanel" class="draggable-debug-panel" :style="{
    top: position.y + 'px',
    left: position.x + 'px',
    zIndex: getActiveZIndex()
  }" @mousedown="startDrag" @click.stop="preventEventBubbling">
    <!-- 🎯 拖拽手柄 -->
    <div class="drag-handle" @mousedown.stop="startDrag">
      <div class="drag-dots">
        <div></div>
        <div></div>
        <div></div>
        <div></div>
        <div></div>
        <div></div>
      </div>
      <span class="panel-title">🐛 Debug Panel</span>
      <button class="close-btn" @click.stop="$emit('close')">✕</button>
    </div>

    <!-- 调试内容 -->
    <div class="debug-content">
      <div class="debug-tabs">
        <button v-for="tab in tabs" :key="tab.id" :class="['tab-btn', { active: activeTab === tab.id }]"
          @click="activeTab = tab.id">
          {{ tab.label }}
        </button>
      </div>

      <div class="tab-content">
        <!-- Chat 状态标签 -->
        <div v-if="activeTab === 'chat'" class="tab-panel">
          <div class="debug-section">
            <h4>📱 聊天状态</h4>
            <div class="debug-item">
              <span class="label">当前聊天ID:</span>
              <span class="value">{{ chatStore.currentChatId || 'None' }}</span>
            </div>
            <div class="debug-item">
              <span class="label">消息数量:</span>
              <span class="value">{{ chatStore.messages.length }}</span>
            </div>
            <div class="debug-item">
              <span class="label">加载状态:</span>
              <span class="value">{{ chatStore.loading ? '是' : '否' }}</span>
            </div>
            <div class="debug-item">
              <span class="label">切换处理中:</span>
              <span class="value">{{ chatStore.chatSwitchDebouncer.isProcessing ? '是' : '否' }}</span>
            </div>
          </div>
        </div>

        <!-- 连接状态标签 -->
        <div v-if="activeTab === 'connection'" class="tab-panel">
          <div class="debug-section">
            <h4>🔗 连接状态</h4>
            <div class="debug-item">
              <span class="label">SSE连接:</span>
              <span class="value" :class="sseStatus.class">{{ sseStatus.text }}</span>
            </div>
            <div class="debug-item">
              <span class="label">重连次数:</span>
              <span class="value">{{ connectionStats.reconnectAttempts }}</span>
            </div>
            <div class="debug-item">
              <span class="label">延迟:</span>
              <span class="value">{{ connectionStats.latency }}ms</span>
            </div>
          </div>
        </div>

        <!-- 性能标签 -->
        <div v-if="activeTab === 'performance'" class="tab-panel">
          <div class="debug-section">
            <h4>⚡ 性能监控</h4>
            <div class="debug-item">
              <span class="label">内存使用:</span>
              <span class="value">{{ performanceStats.memory }}MB</span>
            </div>
            <div class="debug-item">
              <span class="label">FPS:</span>
              <span class="value">{{ performanceStats.fps }}</span>
            </div>
            <div class="debug-item">
              <span class="label">消息缓存:</span>
              <span class="value">{{ Object.keys(chatStore.messageCache).length }} 个聊天</span>
            </div>
          </div>
        </div>

        <!-- 布局稳定性标签 -->
        <div v-if="activeTab === 'layout'" class="tab-panel">
          <div class="debug-section">
            <h4>📐 布局稳定性</h4>
            <div class="debug-item">
              <span class="label">布局偏移:</span>
              <span class="value" :class="layoutStats.stable ? 'status-good' : 'status-bad'">
                {{ layoutStats.stable ? '稳定' : '检测到偏移' }}
              </span>
            </div>
            <div class="debug-item">
              <span class="label">重排次数:</span>
              <span class="value">{{ layoutStats.reflows }}</span>
            </div>
            <div class="debug-item">
              <span class="label">重绘次数:</span>
              <span class="value">{{ layoutStats.repaints }}</span>
            </div>
          </div>

          <div class="debug-actions">
            <button class="action-btn" @click="resetLayoutStats">重置统计</button>
            <button class="action-btn" @click="measureLayoutStability">检测稳定性</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue';
import { useChatStore } from '@/stores/chat';

defineEmits(['close']);

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  initialPosition: {
    type: Object,
    default: () => ({ x: 20, y: 200 }) // 向下移动到200px
  }
});

const chatStore = useChatStore();

// 拖拽相关
const debugPanel = ref(null);
const isDragging = ref(false);
const dragOffset = reactive({ x: 0, y: 0 });
const position = reactive({ ...props.initialPosition });
const zIndex = ref(9999);

// 状态数据
const activeTab = ref('chat');
const tabs = [
  { id: 'chat', label: '聊天' },
  { id: 'connection', label: '连接' },
  { id: 'performance', label: '性能' },
  { id: 'layout', label: '布局' }
];

// 连接状态
const connectionStats = reactive({
  reconnectAttempts: 0,
  latency: 42
});

const sseStatus = computed(() => {
  // 这里可以从SSE service获取实际状态
  return {
    text: '已连接',
    class: 'status-good'
  };
});

// 性能统计
const performanceStats = reactive({
  memory: 0,
  fps: 60
});

// 布局稳定性统计
const layoutStats = reactive({
  stable: true,
  reflows: 0,
  repaints: 0,
  lastCheck: Date.now()
});

// 🔧 新增：动态z-index管理和事件防穿透
const getActiveZIndex = () => {
  // 智能层级管理系统
  const maxContextMenuZ = 9000; // 给消息菜单预留空间
  const baseDebugZ = 15000; // Debug面板基础层级
  const dragZ = 20000; // 拖拽时的层级
  
  return isDragging.value ? dragZ : baseDebugZ;
};

// 🔧 新增：阻止事件冒泡和穿透
const preventEventBubbling = (e) => {
  e.stopPropagation();
  e.preventDefault();
};

const startDrag = (e) => {
  if (e.target.classList.contains('close-btn')) return;

  isDragging.value = true;
  // 移除硬编码的zIndex，改用动态计算

  const rect = debugPanel.value.getBoundingClientRect();
  dragOffset.x = e.clientX - rect.left;
  dragOffset.y = e.clientY - rect.top;

  document.addEventListener('mousemove', onDrag);
  document.addEventListener('mouseup', stopDrag);

  // 防止文本选择
  e.preventDefault();
};

const onDrag = (e) => {
  if (!isDragging.value) return;

  position.x = e.clientX - dragOffset.x;
  position.y = e.clientY - dragOffset.y;

  // 限制在窗口范围内
  const rect = debugPanel.value.getBoundingClientRect();
  const maxX = window.innerWidth - rect.width;
  const maxY = window.innerHeight - rect.height;

  position.x = Math.max(0, Math.min(position.x, maxX));
  position.y = Math.max(0, Math.min(position.y, maxY));
};

const stopDrag = () => {
  isDragging.value = false;
  // 移除硬编码的zIndex，改用动态计算

  document.removeEventListener('mousemove', onDrag);
  document.removeEventListener('mouseup', stopDrag);
};

// 性能监控
const updatePerformanceStats = () => {
  if (performance.memory) {
    performanceStats.memory = Math.round(performance.memory.usedJSHeapSize / 1024 / 1024);
  }
};

// 布局稳定性检测
const measureLayoutStability = () => {
  // 简单的布局稳定性检测
  const startTime = performance.now();
  const messageList = document.querySelector('.message-list');

  if (messageList) {
    const rect1 = messageList.getBoundingClientRect();

    // 强制重排
    messageList.style.transform = 'translateZ(0)';

    requestAnimationFrame(() => {
      const rect2 = messageList.getBoundingClientRect();
      const shifted = Math.abs(rect1.top - rect2.top) > 1 || Math.abs(rect1.left - rect2.left) > 1;

      layoutStats.stable = !shifted;
      layoutStats.reflows++;
      layoutStats.lastCheck = Date.now();

      messageList.style.transform = '';
    });
  }
};

const resetLayoutStats = () => {
  layoutStats.reflows = 0;
  layoutStats.repaints = 0;
  layoutStats.stable = true;
  layoutStats.lastCheck = Date.now();
};

// 定时器
let performanceTimer = null;
let layoutTimer = null;

onMounted(() => {
  // 性能监控定时器
  performanceTimer = setInterval(updatePerformanceStats, 5000);

  // 布局检测定时器
  layoutTimer = setInterval(measureLayoutStability, 10000);
});

onUnmounted(() => {
  if (performanceTimer) clearInterval(performanceTimer);
  if (layoutTimer) clearInterval(layoutTimer);

  document.removeEventListener('mousemove', onDrag);
  document.removeEventListener('mouseup', stopDrag);
});
</script>

<style scoped>
.draggable-debug-panel {
  position: fixed;
  background: rgba(26, 26, 26, 0.95);
  border: 1px solid rgba(124, 58, 237, 0.3);
  border-radius: 12px;
  min-width: 320px;
  max-width: 420px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  backdrop-filter: blur(10px);
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  user-select: none;
  transition: box-shadow 0.2s ease;
}

.draggable-debug-panel:hover {
  box-shadow: 0 12px 48px rgba(0, 0, 0, 0.4);
}

.drag-handle {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  background: rgba(124, 58, 237, 0.1);
  border-radius: 12px 12px 0 0;
  cursor: move;
  border-bottom: 1px solid rgba(124, 58, 237, 0.2);
}

.drag-dots {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 3px;
  width: 16px;
  height: 12px;
}

.drag-dots div {
  width: 3px;
  height: 3px;
  background: rgba(124, 58, 237, 0.6);
  border-radius: 50%;
}

.panel-title {
  flex: 1;
  color: #ffffff;
  font-weight: 600;
  font-size: 14px;
}

.close-btn {
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  transition: all 0.2s ease;
  font-size: 16px;
  line-height: 1;
}

.close-btn:hover {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
}

.debug-content {
  padding: 0;
}

.debug-tabs {
  display: flex;
  border-bottom: 1px solid rgba(124, 58, 237, 0.2);
}

.tab-btn {
  flex: 1;
  padding: 10px 12px;
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.2s ease;
  border-bottom: 2px solid transparent;
}

.tab-btn:hover {
  background: rgba(124, 58, 237, 0.1);
  color: rgba(255, 255, 255, 0.9);
}

.tab-btn.active {
  color: #7c3aed;
  border-bottom-color: #7c3aed;
  background: rgba(124, 58, 237, 0.05);
}

.tab-content {
  max-height: 300px;
  overflow-y: auto;
}

.tab-panel {
  padding: 16px;
}

.debug-section {
  margin-bottom: 16px;
}

.debug-section h4 {
  color: #ffffff;
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 10px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.debug-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.debug-item:last-child {
  border-bottom: none;
}

.debug-item .label {
  color: rgba(255, 255, 255, 0.7);
  font-size: 12px;
  font-weight: 500;
}

.debug-item .value {
  color: #ffffff;
  font-size: 12px;
  font-weight: 600;
  font-family: 'Monaco', 'Menlo', monospace;
}

.status-good {
  color: #10b981 !important;
}

.status-bad {
  color: #ef4444 !important;
}

.debug-actions {
  display: flex;
  gap: 8px;
  margin-top: 12px;
}

.action-btn {
  flex: 1;
  padding: 8px 12px;
  background: rgba(124, 58, 237, 0.2);
  border: 1px solid rgba(124, 58, 237, 0.4);
  border-radius: 6px;
  color: #ffffff;
  cursor: pointer;
  font-size: 11px;
  font-weight: 500;
  transition: all 0.2s ease;
}

.action-btn:hover {
  background: rgba(124, 58, 237, 0.3);
  border-color: rgba(124, 58, 237, 0.6);
}

/* 滚动条样式 */
.tab-content::-webkit-scrollbar {
  width: 4px;
}

.tab-content::-webkit-scrollbar-track {
  background: transparent;
}

.tab-content::-webkit-scrollbar-thumb {
  background: rgba(124, 58, 237, 0.3);
  border-radius: 2px;
}

.tab-content::-webkit-scrollbar-thumb:hover {
  background: rgba(124, 58, 237, 0.5);
}

/* 响应式 */
@media (max-width: 768px) {
  .draggable-debug-panel {
    min-width: 280px;
    max-width: calc(100vw - 40px);
  }

  .tab-btn {
    font-size: 11px;
    padding: 8px 6px;
  }
}
</style>