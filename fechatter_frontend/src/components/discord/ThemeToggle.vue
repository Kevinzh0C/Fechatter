<template>
  <div class="theme-toggle-container">
    <button
      @click="toggleTheme"
      class="theme-toggle-btn"
      :class="{ 'dark': isDark }"
      :title="isDark ? 'Switch to Light Mode' : 'Switch to Dark Mode'"
      :aria-label="isDark ? 'Switch to Light Mode' : 'Switch to Dark Mode'"
    >
      <div class="toggle-track">
        <div class="toggle-handle" :class="{ 'dark': isDark }">
          <Transition name="icon" mode="out-in">
            <SunIcon v-if="!isDark" class="w-4 h-4 text-yellow-500" />
            <MoonIcon v-else class="w-4 h-4 text-blue-300" />
          </Transition>
        </div>
      </div>
      <span class="theme-label">{{ isDark ? 'Dark' : 'Light' }}</span>
    </button>
  </div>
</template>

<script setup>
import { ref, onMounted, watch } from 'vue';
import { SunIcon, MoonIcon } from '@heroicons/vue/24/outline';

// 响应式主题状态
const isDark = ref(false);

// 获取系统主题偏好
const getSystemTheme = () => {
  return window.matchMedia('(prefers-color-scheme: dark)').matches;
};

// 从localStorage获取保存的主题
const getSavedTheme = () => {
  const saved = localStorage.getItem('fechatter-theme');
  if (saved) {
    return saved === 'dark';
  }
  return getSystemTheme();
};

// 保存主题到localStorage
const saveTheme = (theme) => {
  localStorage.setItem('fechatter-theme', theme);
};

// 应用主题到DOM
const applyTheme = (dark) => {
  const html = document.documentElement;
  
  if (dark) {
    html.setAttribute('data-theme', 'dark');
    html.classList.add('dark');
  } else {
    html.setAttribute('data-theme', 'light');
    html.classList.remove('dark');
  }
  
  // 触发主题变化事件
  window.dispatchEvent(new CustomEvent('theme-changed', {
    detail: { theme: dark ? 'dark' : 'light' }
  }));
};

// 切换主题
const toggleTheme = () => {
  isDark.value = !isDark.value;
  applyTheme(isDark.value);
  saveTheme(isDark.value ? 'dark' : 'light');
  
  // 触发haptic feedback (if supported)
  if ('vibrate' in navigator) {
    navigator.vibrate(50);
  }
};

// 监听系统主题变化
const watchSystemTheme = () => {
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  
  const handleChange = (e) => {
    // 只有在没有手动设置主题时才跟随系统
    const savedTheme = localStorage.getItem('fechatter-theme');
    if (!savedTheme) {
      isDark.value = e.matches;
      applyTheme(isDark.value);
    }
  };
  
  mediaQuery.addEventListener('change', handleChange);
  
  return () => {
    mediaQuery.removeEventListener('change', handleChange);
  };
};

// 监听主题变化，更新Shiki等依赖主题的组件
watch(isDark, (newValue) => {
  // 通知其他组件主题已变化
  window.dispatchEvent(new CustomEvent('discord-theme-changed', {
    detail: { 
      isDark: newValue,
      theme: newValue ? 'github-dark' : 'github-light'
    }
  }));
}, { immediate: true });

// 组件挂载时初始化主题
onMounted(() => {
  isDark.value = getSavedTheme();
  applyTheme(isDark.value);
  
  // 监听系统主题变化
  const unwatch = watchSystemTheme();
  
  // 清理
  return unwatch;
});

// 暴露方法给父组件
defineExpose({
  toggleTheme,
  isDark
});
</script>

<style scoped>
.theme-toggle-container {
  display: flex;
  align-items: center;
}

.theme-toggle-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  background: transparent;
  border: none;
  padding: 8px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  color: var(--text-secondary);
  font-size: 0.875rem;
  font-weight: 500;
}

.theme-toggle-btn:hover {
  background: var(--bg-message-hover);
  color: var(--text-primary);
}

.theme-toggle-btn:focus {
  outline: 2px solid var(--border-focus);
  outline-offset: 2px;
}

.toggle-track {
  position: relative;
  width: 40px;
  height: 20px;
  background: var(--bg-tertiary);
  border-radius: 12px;
  border: 1px solid var(--border-primary);
  transition: all 0.2s ease;
}

.theme-toggle-btn.dark .toggle-track {
  background: var(--discord-primary);
  border-color: var(--discord-primary);
}

.toggle-handle {
  position: absolute;
  top: 1px;
  left: 1px;
  width: 16px;
  height: 16px;
  background: var(--bg-primary);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  box-shadow: var(--shadow-low);
}

.toggle-handle.dark {
  transform: translateX(18px);
}

.theme-label {
  user-select: none;
  transition: color 0.2s ease;
}

/* 图标过渡动画 */
.icon-enter-active,
.icon-leave-active {
  transition: all 0.2s ease;
}

.icon-enter-from {
  opacity: 0;
  transform: rotate(-90deg) scale(0.8);
}

.icon-leave-to {
  opacity: 0;
  transform: rotate(90deg) scale(0.8);
}

/* 无障碍支持 */
@media (prefers-reduced-motion: reduce) {
  .theme-toggle-btn,
  .toggle-track,
  .toggle-handle,
  .theme-label,
  .icon-enter-active,
  .icon-leave-active {
    transition: none;
  }
  
  .icon-enter-from,
  .icon-leave-to {
    transform: none;
  }
}

/* 高对比度模式 */
@media (prefers-contrast: high) {
  .toggle-track {
    border-width: 2px;
  }
  
  .theme-toggle-btn:focus {
    outline-width: 3px;
  }
}

/* 移动端优化 */
@media (max-width: 768px) {
  .theme-toggle-btn {
    padding: 6px 8px;
  }
  
  .theme-label {
    display: none;
  }
}

/* 打印模式隐藏 */
@media print {
  .theme-toggle-container {
    display: none;
  }
}
</style> 