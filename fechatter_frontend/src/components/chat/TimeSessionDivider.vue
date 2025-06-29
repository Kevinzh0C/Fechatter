<template>
  <div class="time-session-divider" :class="dividerClasses">
    <!-- 日期分割线 - 采用Discord/Slack风格的极简设计 -->
    <div v-if="divider.type === 'date-divider'" class="date-divider">
      <div class="date-line"></div>
      <div class="date-content">
        <span class="date-text">{{ divider.displayText }}</span>
      </div>
      <div class="date-line"></div>
    </div>
    
    <!-- 副日期分割线 - 必须跟在主日期分界线后面 -->
    <div v-else-if="divider.type === 'sub-date-divider'" class="sub-date-divider">
      <div class="sub-date-line"></div>
      <div class="sub-date-content">
        <span class="sub-date-text">{{ divider.displayText }}</span>
      </div>
      <div class="sub-date-line"></div>
    </div>
    
    <!-- 会话分割线 - 更subtle的设计 -->
    <div v-else-if="divider.type === 'session-divider'" class="session-divider">
      <div class="session-line"></div>
      <div v-if="divider.displayText" class="session-content">
        <span class="session-text">{{ divider.displayText }}</span>
      </div>
      <div class="session-line"></div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  divider: {
    type: Object,
    required: true
  },
  showDateDetails: {
    type: Boolean,
    default: false
  },
  compact: {
    type: Boolean,
    default: false
  }
});

const dividerClasses = computed(() => {
  const classes = [`divider-${props.divider.type}`];
  if (props.compact) classes.push('compact');
  if (props.divider.metadata?.isToday) classes.push('today');
  if (props.divider.metadata?.isYesterday) classes.push('yesterday');
  return classes;
});
</script>

<style scoped>
/* 基础容器 - 最小化垂直空间占用 */
.time-session-divider {
  margin: 0.25rem 0;
  position: relative;
}

.compact {
  margin: 0.125rem 0;
}

/* 📅 日期分割线 - 基于Discord/Slack的最佳实践 */
.date-divider {
  display: flex;
  align-items: center;
  margin: 0.375rem 0;
  opacity: 0.8;
}

.date-line {
  flex: 1;
  height: 1px;
  background: #e5e7eb; /* 极淡的灰色，不干扰内容 */
}

.date-content {
  padding: 0 0.875rem;
  background: transparent; /* 去掉背景色，减少视觉重量 */
}

.date-text {
  font-size: 0.75rem; /* 更小的字体，降低视觉优先级 */
  font-weight: 500; /* 中等字重，不过分突出 */
  color: #6b7280; /* 中性灰色，符合认知科学的对比度原则 */
  text-transform: uppercase;
  letter-spacing: 0.05em;
  white-space: nowrap;
}

/* 📅 副日期分割线 - 介于主日期和会话分割线之间的设计 */
.sub-date-divider {
  display: flex;
  align-items: center;
  margin: 0.21875rem 0 0.25rem 0;
  opacity: 0.7;
}

.sub-date-line {
  flex: 1;
  height: 1px;
  background: linear-gradient(90deg,
    transparent 0%,
    #e5e7eb 25%,
    #d1d5db 50%,
    #e5e7eb 75%,
    transparent 100%
  ); /* 渐变线条，更精致 */
}

.sub-date-content {
  padding: 0 0.75rem;
  background: transparent;
}

.sub-date-text {
  font-size: 0.6875rem; /* 比主日期小，比会话分割线大 */
  font-weight: 450; /* 中等偏轻字重 */
  color: #8b5cf6; /* 紫色系，区别于其他分割线 */
  text-transform: none; /* 不使用大写 */
  letter-spacing: 0.025em;
  white-space: nowrap;
  font-style: normal;
}

/* 特殊状态 - 保持低调但有意义的差异 */
.divider-today .date-text {
  color: #059669; /* 绿色但不过分鲜艳 */
  font-weight: 600; /* 今天可以稍微突出 */
}

.divider-yesterday .date-text {
  color: #7c2d12; /* 棕色，暖色调但不刺眼 */
}

/* 🌙 暗色主题适配 */
@media (prefers-color-scheme: dark) {
  .date-line {
    background: #374151;
  }
  
  .sub-date-line {
    background: linear-gradient(90deg,
      transparent 0%,
      rgba(139, 92, 246, 0.3) 25%,
      rgba(139, 92, 246, 0.5) 50%,
      rgba(139, 92, 246, 0.3) 75%,
      transparent 100%
    ); /* 暗色主题下的紫色渐变 */
  }
  
  .session-line {
    background: #1f2937;
  }
  
  .date-text {
    color: #9ca3af;
  }
  
  .sub-date-text {
    color: #a78bfa; /* 暗色主题下的浅紫色 */
  }
  
  .session-text {
    color: #6b7280;
  }
  
  .divider-today .date-text {
    color: #10b981;
  }
  
  .divider-yesterday .date-text {
    color: #f59e0b;
  }
}

/* 📱 响应式设计 - 移动端优化 */
@media (max-width: 768px) {
  .time-session-divider {
    margin: 0.1875rem 0;
  }
  
  .date-divider {
    margin: 0.25rem 0;
  }
  
  .sub-date-divider {
    margin: 0.15625rem 0 0.1875rem 0;
  }
  
  .date-content {
    padding: 0 0.5rem;
  }
  
  .sub-date-content {
    padding: 0 0.5rem;
  }
  
  .date-text {
    font-size: 0.7rem;
  }
  
  .sub-date-text {
    font-size: 0.625rem;
  }
  
  .session-text {
    font-size: 0.625rem;
  }
}

/* ✨ 微妙的悬停效果 - 不干扰但提供反馈 */
.date-divider:hover .date-text {
  color: #374151;
  transition: color 0.2s ease;
}

@media (prefers-color-scheme: dark) {
  .date-divider:hover .date-text {
    color: #d1d5db;
  }
}

/* 🎭 进入动画 - 非常subtle */
.time-session-divider {
  animation: fadeIn 0.3s ease-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

/* 辅助功能优化 */
@media (prefers-reduced-motion: reduce) {
  .time-session-divider {
    animation: none;
  }
  
  .date-divider:hover .date-text {
    transition: none;
  }
}

/* ⏰ 会话分割线 - 更subtle的设计 */
.session-divider {
  display: flex;
  align-items: center;
  margin: 0.75rem 0;
  opacity: 0.6; /* 更低的不透明度，减少干扰 */
}

.session-line {
  flex: 1;
  height: 1px;
  background: #f3f4f6; /* 非常淡的灰色，几乎不可见 */
}

.session-content {
  padding: 0 0.75rem;
  background: transparent;
}

.session-text {
  font-size: 0.6875rem; /* 非常小的字体 */
  color: #9ca3af; /* 更淡的颜色 */
  font-style: italic;
  font-weight: 400; /* 正常字重 */
  white-space: nowrap;
}
</style> 