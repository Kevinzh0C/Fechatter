<template>
  <div 
    class="discord-avatar-container"
    :class="[
      `size-${size}`,
      { 
        'clickable': clickable,
        'has-status': showStatus && status
      }
    ]"
    @click="handleClick"
  >
    <!-- 头像主体 -->
    <div 
      class="discord-avatar"
      :style="avatarStyle"
      :title="displayName"
    >
      <!-- 图片头像 -->
      <img 
        v-if="src && !imageError"
        :src="src"
        :alt="displayName"
        class="avatar-image"
        @error="handleImageError"
        @load="handleImageLoad"
        loading="lazy"
      />
      
      <!-- 文字头像（fallback） -->
      <span 
        v-else
        class="avatar-text"
        :style="textStyle"
      >
        {{ initials }}
      </span>
      
      <!-- 加载状态 -->
      <div v-if="loading" class="avatar-loading">
        <div class="loading-spinner"></div>
      </div>
    </div>
    
    <!-- 状态指示器 -->
    <div 
      v-if="showStatus && status"
      class="status-indicator"
      :class="`status-${status}`"
      :title="statusText"
    >
      <div class="status-dot"></div>
    </div>
    
    <!-- 自定义徽章 -->
    <div 
      v-if="badge"
      class="avatar-badge"
      :class="badge.type || 'default'"
      :title="badge.title"
    >
      <component :is="badge.icon" v-if="badge.icon" class="badge-icon" />
      <span v-else-if="badge.text" class="badge-text">{{ badge.text }}</span>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue';

const props = defineProps({
  // 头像图片URL
  src: {
    type: String,
    default: null
  },
  
  // 显示名称
  name: {
    type: String,
    default: 'User'
  },
  
  // 用户ID（用于生成一致的颜色）
  userId: {
    type: [String, Number],
    default: null
  },
  
  // 头像尺寸
  size: {
    type: String,
    default: 'medium',
    validator: (value) => ['small', 'medium', 'large', 'xlarge'].includes(value)
  },
  
  // 在线状态
  status: {
    type: String,
    default: null,
    validator: (value) => !value || ['online', 'idle', 'dnd', 'offline'].includes(value)
  },
  
  // 是否显示状态指示器
  showStatus: {
    type: Boolean,
    default: false
  },
  
  // 是否可点击
  clickable: {
    type: Boolean,
    default: false
  },
  
  // 自定义背景色
  backgroundColor: {
    type: String,
    default: null
  },
  
  // 自定义文字色
  textColor: {
    type: String,
    default: null
  },
  
  // 徽章配置
  badge: {
    type: Object,
    default: null
  },
  
  // 加载状态
  loading: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits(['click', 'image-load', 'image-error']);

// 响应式状态
const imageError = ref(false);
const imageLoaded = ref(false);

// Discord风格的用户颜色池
const discordColors = [
  '#f23f43', // 红色
  '#f0b232', // 黄色
  '#23a55a', // 绿色
  '#5865f2', // Discord蓝
  '#eb459e', // 粉色
  '#3ba55c', // 深绿
  '#faa61a', // 橙色
  '#ed4245', // 深红
  '#9146ff', // 紫色
  '#00d4aa', // 青色
];

// 计算显示名称
const displayName = computed(() => {
  return props.name || 'User';
});

// 计算首字母
const initials = computed(() => {
  const name = displayName.value;
  if (!name) return '?';
  
  // 支持中英文混合
  const words = name.split(/\s+/);
  if (words.length >= 2) {
    return (words[0].charAt(0) + words[1].charAt(0)).toUpperCase();
  } else {
    // 单个词，取前两个字符
    return name.slice(0, 2).toUpperCase();
  }
});

// 计算头像背景色
const avatarBackgroundColor = computed(() => {
  if (props.backgroundColor) {
    return props.backgroundColor;
  }
  
  if (props.userId) {
    // 基于用户ID生成一致的颜色
    const hash = String(props.userId).split('').reduce((a, b) => {
      a = ((a << 5) - a) + b.charCodeAt(0);
      return a & a;
    }, 0);
    const index = Math.abs(hash) % discordColors.length;
    return discordColors[index];
  }
  
  // 基于名称生成颜色
  const hash = displayName.value.split('').reduce((a, b) => {
    a = ((a << 5) - a) + b.charCodeAt(0);
    return a & a;
  }, 0);
  const index = Math.abs(hash) % discordColors.length;
  return discordColors[index];
});

// 头像样式
const avatarStyle = computed(() => {
  const styles = {};
  
  if (!props.src || imageError.value) {
    styles.backgroundColor = avatarBackgroundColor.value;
  }
  
  return styles;
});

// 文字样式
const textStyle = computed(() => {
  return {
    color: props.textColor || '#ffffff'
  };
});

// 状态文本
const statusText = computed(() => {
  const statusMap = {
    online: 'Online',
    idle: 'Away',
    dnd: 'Do Not Disturb',
    offline: 'Offline'
  };
  return statusMap[props.status] || '';
});

// 事件处理
const handleClick = () => {
  if (props.clickable) {
    emit('click');
  }
};

const handleImageError = () => {
  imageError.value = true;
  emit('image-error');
};

const handleImageLoad = () => {
  imageError.value = false;
  imageLoaded.value = true;
  emit('image-load');
};

// 监听src变化，重置错误状态
watch(() => props.src, () => {
  imageError.value = false;
  imageLoaded.value = false;
});
</script>

<style scoped>
/* 🎨 Discord头像容器 */
.discord-avatar-container {
  position: relative;
  display: inline-block;
  flex-shrink: 0;
}

.discord-avatar-container.clickable {
  cursor: pointer;
}

/* 🎯 头像尺寸 */
.size-small .discord-avatar {
  width: 24px;
  height: 24px;
  font-size: 10px;
}

.size-medium .discord-avatar {
  width: 40px;
  height: 40px;
  font-size: 16px;
}

.size-large .discord-avatar {
  width: 80px;
  height: 80px;
  font-size: 32px;
}

.size-xlarge .discord-avatar {
  width: 120px;
  height: 120px;
  font-size: 48px;
}

/* 🎯 头像主体 */
.discord-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  overflow: hidden;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--bg-secondary);
  border: 2px solid var(--avatar-border);
  transition: all 0.2s ease;
  box-shadow: var(--shadow-low);
}

.discord-avatar-container.clickable .discord-avatar:hover {
  border-color: var(--border-focus);
  transform: scale(1.05);
  box-shadow: var(--shadow-medium);
}

/* 🎯 头像图片 */
.avatar-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 50%;
}

/* 🎯 文字头像 */
.avatar-text {
  font-weight: 600;
  font-family: "gg sans", "Noto Sans", "Helvetica Neue", Helvetica, Arial, sans-serif;
  line-height: 1;
  user-select: none;
  color: #ffffff;
}

/* 🎯 加载状态 */
.avatar-loading {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.5);
  border-radius: 50%;
}

.loading-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top: 2px solid #ffffff;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* 🎯 状态指示器 */
.status-indicator {
  position: absolute;
  bottom: -2px;
  right: -2px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 3px solid var(--bg-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1;
}

.size-small .status-indicator {
  width: 10px;
  height: 10px;
  border-width: 2px;
  bottom: -1px;
  right: -1px;
}

.size-large .status-indicator {
  width: 24px;
  height: 24px;
  border-width: 4px;
  bottom: -3px;
  right: -3px;
}

.size-xlarge .status-indicator {
  width: 32px;
  height: 32px;
  border-width: 5px;
  bottom: -4px;
  right: -4px;
}

.status-dot {
  width: 100%;
  height: 100%;
  border-radius: 50%;
}

/* 🎯 状态颜色 */
.status-online .status-dot {
  background-color: var(--status-online);
}

.status-idle .status-dot {
  background-color: var(--status-idle);
}

.status-dnd .status-dot {
  background-color: var(--status-dnd);
}

.status-offline .status-dot {
  background-color: var(--status-offline);
}

/* 🎯 徽章 */
.avatar-badge {
  position: absolute;
  top: -4px;
  right: -4px;
  min-width: 18px;
  height: 18px;
  border-radius: 12px;
  background: var(--text-danger);
  color: white;
  font-size: 11px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 6px;
  border: 2px solid var(--bg-primary);
  z-index: 2;
}

.avatar-badge.admin {
  background: var(--discord-primary);
}

.avatar-badge.moderator {
  background: var(--status-online);
}

.avatar-badge.premium {
  background: linear-gradient(45deg, #ff6b6b, #feca57);
}

.badge-icon {
  width: 12px;
  height: 12px;
}

.badge-text {
  font-size: 10px;
  line-height: 1;
}

.size-small .avatar-badge {
  top: -2px;
  right: -2px;
  min-width: 14px;
  height: 14px;
  font-size: 9px;
  padding: 0 4px;
}

.size-large .avatar-badge {
  top: -6px;
  right: -6px;
  min-width: 24px;
  height: 24px;
  font-size: 14px;
  padding: 0 8px;
}

.size-xlarge .avatar-badge {
  top: -8px;
  right: -8px;
  min-width: 32px;
  height: 32px;
  font-size: 16px;
  padding: 0 10px;
}

/* 🎯 悬停效果 */
.discord-avatar-container.clickable:hover .status-indicator {
  transform: scale(1.1);
}

.discord-avatar-container.clickable:hover .avatar-badge {
  transform: scale(1.1);
}

/* 🎯 无障碍支持 */
.discord-avatar-container:focus {
  outline: 2px solid var(--border-focus);
  outline-offset: 2px;
  border-radius: 50%;
}

/* 🎯 动画 */
.discord-avatar-container {
  animation: discord-fade-in 0.2s ease-out;
}

/* 🎯 响应式 */
@media (max-width: 768px) {
  .size-large .discord-avatar {
    width: 60px;
    height: 60px;
    font-size: 24px;
  }
  
  .size-xlarge .discord-avatar {
    width: 80px;
    height: 80px;
    font-size: 32px;
  }
}

/* 🎯 高对比度模式 */
@media (prefers-contrast: high) {
  .discord-avatar {
    border-width: 3px;
  }
  
  .status-indicator {
    border-width: 4px;
  }
}

/* 🎯 打印模式 */
@media print {
  .discord-avatar-container {
    break-inside: avoid;
  }
  
  .status-indicator,
  .avatar-badge {
    display: none;
  }
}
</style> 