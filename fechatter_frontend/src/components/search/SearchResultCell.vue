<template>
  <!-- M2: 微缩 MessageList - 高度 ≤ 48px 优化水平扫描 -->
  <div class="search-result-cell" :class="{
    'is-selected': isSelected,
    'is-focused': isFocused
  }" @click="handleClick" @keydown.enter="handleClick" @keydown.space.prevent="handleClick" :tabindex="tabIndex"
    role="option" :aria-selected="isSelected" :aria-label="`Message from ${message.sender_name}: ${cleanContent}`">
    <!-- 紧凑布局：头像 + 内容 + 时间 -->
    <div class="cell-avatar">
      <div class="avatar-mini" :title="message.sender_name">
        {{ getInitials(message.sender_name) }}
      </div>
    </div>

    <div class="cell-content">
      <!-- 发送者 + 高亮内容 -->
      <div class="content-main">
        <span class="sender-name">{{ message.sender_name }}:</span>
        <span class="message-preview" v-html="highlightedContent"></span>
      </div>

      <!-- 元信息：时间 + 匹配信息 -->
      <div class="content-meta">
        <time class="message-time" :datetime="message.created_at">
          {{ formatTime(message.created_at) }}
        </time>
        <span v-if="matchInfo" class="match-info">
          {{ matchInfo }}
        </span>
      </div>
    </div>

    <!-- 快速操作 -->
    <div class="cell-actions">
      <button class="action-button" @click.stop="$emit('jump-to', message)" :aria-label="`Jump to message`">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 2L13 7L8 12M3 7H13" stroke="currentColor" stroke-width="2" fill="none" />
        </svg>
      </button>
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
  searchQuery: {
    type: String,
    default: ''
  },
  isSelected: {
    type: Boolean,
    default: false
  },
  isFocused: {
    type: Boolean,
    default: false
  },
  tabIndex: {
    type: Number,
    default: 0
  },
  highlightKeyword: {
    type: Function,
    default: (text, keyword) => text
  }
});

const emit = defineEmits(['click', 'jump-to']);

// 清理内容（移除HTML标签）
const cleanContent = computed(() => {
  return props.message.content
    .replace(/<[^>]*>/g, '')
    .replace(/\s+/g, ' ')
    .trim();
});

// 高亮内容
const highlightedContent = computed(() => {
  const maxLength = 80; // 最大显示字符数
  const content = cleanContent.value;

  if (!props.searchQuery) {
    return content.length > maxLength
      ? content.substring(0, maxLength) + '...'
      : content;
  }

  // 找到关键词位置
  const lowerContent = content.toLowerCase();
  const lowerQuery = props.searchQuery.toLowerCase();
  const matchIndex = lowerContent.indexOf(lowerQuery);

  if (matchIndex === -1) {
    // 没找到，返回截断的内容
    return content.length > maxLength
      ? content.substring(0, maxLength) + '...'
      : content;
  }

  // 计算显示窗口
  const contextLength = 30;
  let start = Math.max(0, matchIndex - contextLength);
  let end = Math.min(content.length, matchIndex + lowerQuery.length + contextLength);

  // 调整到词边界
  if (start > 0) {
    const spaceIndex = content.lastIndexOf(' ', start);
    if (spaceIndex > start - 10) {
      start = spaceIndex + 1;
    }
  }

  if (end < content.length) {
    const spaceIndex = content.indexOf(' ', end);
    if (spaceIndex !== -1 && spaceIndex < end + 10) {
      end = spaceIndex;
    }
  }

  // 构建显示文本
  let displayText = content.substring(start, end);
  if (start > 0) displayText = '...' + displayText;
  if (end < content.length) displayText = displayText + '...';

  // 高亮
  return props.highlightKeyword(displayText, props.searchQuery);
});

// 匹配信息
const matchInfo = computed(() => {
  if (!props.message.match_count) return null;

  const count = props.message.match_count;
  return count > 1 ? `${count} matches` : '1 match';
});

// 获取姓名缩写
const getInitials = (name) => {
  if (!name) return '?';

  const parts = name.split(' ');
  if (parts.length >= 2) {
    return parts[0][0] + parts[parts.length - 1][0];
  }
  return name.substring(0, 2).toUpperCase();
};

// 格式化时间
const formatTime = (timestamp) => {
  const date = new Date(timestamp);
  const now = new Date();
  const diff = now - date;

  // Today
  if (date.toDateString() === now.toDateString()) {
    return date.toLocaleTimeString([], { hour: 'numeric', minute: '2-digit' });
  }

  const yesterday = new Date(now);
  yesterday.setDate(now.getDate() - 1);
  if (date.toDateString() === yesterday.toDateString()) {
    // Yesterday
    return 'Yesterday';
  }

  // 本周
  if (diff < 7 * 24 * 60 * 60 * 1000) {
    return date.toLocaleDateString('en-US', { weekday: 'short' });
  }

  // 更早
  return date.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric'
  });
};

// 处理点击
const handleClick = () => {
  emit('click', props.message);
};
</script>

<style scoped>
/* M2: 紧凑设计 - 高度严格控制在48px内 */
.search-result-cell {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  height: 48px;
  padding: 0 1rem;
  background: transparent;
  border: none;
  cursor: pointer;
  transition: all 0.15s ease;

  /* 费兹定律：增大可点击区域 */
  position: relative;
  overflow: hidden;
}

.search-result-cell::before {
  content: '';
  position: absolute;
  left: 0;
  right: 0;
  top: 0;
  bottom: 0;
  background: transparent;
  transition: background 0.15s ease;
}

.search-result-cell:hover::before {
  background: rgba(124, 58, 237, 0.04);
}

.search-result-cell.is-selected::before {
  background: rgba(124, 58, 237, 0.08);
}

.search-result-cell.is-focused {
  outline: 2px solid rgba(124, 58, 237, 0.5);
  outline-offset: -2px;
}

/* 头像 - 迷你版 */
.cell-avatar {
  flex-shrink: 0;
}

.avatar-mini {
  width: 24px;
  height: 24px;
  border-radius: 4px;
  background: rgba(124, 58, 237, 0.1);
  color: #7c3aed;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
}

/* 内容区域 */
.cell-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.content-main {
  display: flex;
  align-items: baseline;
  gap: 0.375rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.sender-name {
  flex-shrink: 0;
  font-weight: 600;
  color: #1a1a1a;
  font-size: 0.8125rem;
}

.message-preview {
  flex: 1;
  color: #4a4a4a;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 高亮样式 - M3关联 */
.message-preview :deep(mark) {
  background: #fef3c7;
  color: inherit;
  padding: 0 0.125rem;
  border-radius: 2px;
  font-weight: 500;
}

/* 元信息 */
.content-meta {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.75rem;
  color: #9a9a9a;
}

.message-time {
  flex-shrink: 0;
}

.match-info {
  color: #7c3aed;
  font-weight: 500;
}

/* 操作按钮 */
.cell-actions {
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.15s ease;
}

.search-result-cell:hover .cell-actions,
.search-result-cell.is-focused .cell-actions {
  opacity: 1;
}

.action-button {
  width: 28px;
  height: 28px;
  border-radius: 4px;
  border: none;
  background: transparent;
  color: #7c3aed;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.15s ease;
}

.action-button:hover {
  background: rgba(124, 58, 237, 0.1);
}

.action-button:active {
  transform: scale(0.95);
}

/* 响应式 */
@media (max-width: 640px) {
  .search-result-cell {
    padding: 0 0.75rem;
  }

  .content-meta {
    display: none;
  }

  .cell-actions {
    opacity: 1;
  }
}

/* 暗色模式 */
@media (prefers-color-scheme: dark) {
  .sender-name {
    color: #e0e0e0;
  }

  .message-preview {
    color: #b0b0b0;
  }

  .message-preview :deep(mark) {
    background: #854d0e;
    color: #fef3c7;
  }
}

/* 动画 */
@keyframes slide-in {
  from {
    opacity: 0;
    transform: translateX(-10px);
  }

  to {
    opacity: 1;
    transform: translateX(0);
  }
}

.search-result-cell {
  animation: slide-in 0.2s ease both;
}
</style>