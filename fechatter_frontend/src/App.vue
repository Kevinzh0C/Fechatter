<template>
  <div id="app" class="discord-chat-container">
    <!-- Global notifications -->
    <NotificationContainer />

    <!-- Auth Loading Overlay -->
    <div v-if="isAuthLoading" class="fixed inset-0 bg-white z-50 flex items-center justify-center">
      <div class="text-center">
        <div class="animate-spin rounded-full h-32 w-32 border-b-2 border-violet-600 mx-auto mb-4"></div>
        <h2 class="text-xl font-semibold text-gray-700">Loading...</h2>
        <p class="text-gray-500 mt-2">Initializing your workspace</p>
      </div>
    </div>

    <!-- Main App -->
    <div v-else>
      <!-- Public Routes -->
      <template v-if="!authStore.isAuthenticated">
        <router-view />
      </template>

      <!-- Protected Routes -->
      <template v-else>
        <div class="app-container">
          <router-view />
        </div>
      </template>
    </div>

    <!-- Error Boundary for unknown errors -->
    <div v-if="hasGlobalError" class="fixed inset-0 bg-red-50 z-50 flex items-center justify-center">
      <div class="text-center p-8">
        <h2 class="text-2xl font-bold text-red-600 mb-4">Application Error</h2>
        <p class="text-gray-700 mb-4">{{ globalError }}</p>
        <button @click="reloadPage" class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700">
          Reload Page
        </button>
      </div>
    </div>

    <!-- Notification Toast Container -->
    <ToastContainer />
    <!-- Keyboard Shortcuts Modal -->
    <KeyboardShortcutsModal v-model="showShortcutsModal" :shortcuts="keyboardShortcuts.shortcuts" />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import KeyboardShortcutsModal from './components/modals/KeyboardShortcutsModal.vue';
import ToastContainer from './components/ui/ToastContainer.vue';
import NotificationContainer from './components/ui/NotificationContainer.vue';
import { useKeyboardShortcuts } from './composables/useKeyboardShortcuts';
import healthCheck from './utils/healthCheck';
import { useAuthStore } from './stores/auth';

// Keyboard shortcuts state
const showShortcutsModal = ref(false);

// Initialize keyboard shortcuts
const keyboardShortcuts = useKeyboardShortcuts({
  enableGlobalShortcuts: true,
  enableNavigationShortcuts: true,
  enableChatShortcuts: true,
  enableSearchShortcuts: true
});

// Global event listeners for keyboard shortcuts
const handleGlobalEvents = (event) => {
  switch (event.type.replace('fechatter:', '')) {
    case 'show-shortcuts-help':
      showShortcutsModal.value = true;
      break;
    case 'open-settings':
      // This will be handled by specific components
      break;
    default:
      // Let other components handle their specific events
      break;
  }
};

// Initialize theme system
const initializeTheme = () => {
  // Get saved theme from localStorage
  const savedTheme = localStorage.getItem('fechatter-theme');
  const systemPrefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;

  // Determine theme to use
  const useDark = savedTheme ? savedTheme === 'dark' : systemPrefersDark;

  // Apply theme
  const html = document.documentElement;
  if (useDark) {
    html.setAttribute('data-theme', 'dark');
    html.classList.add('dark');
  } else {
    html.setAttribute('data-theme', 'light');
    html.classList.remove('dark');
  }
};

// Store
const authStore = useAuthStore();

// State
const isInitialized = ref(false);
const hasGlobalError = ref(false);
const globalError = ref('');
const isAuthLoading = ref(true);

// Methods
const reloadPage = () => {
  window.location.reload();
};

// Initialize and validate authentication state on app startup
const initializeAuthState = async () => {
  try {
    isAuthLoading.value = true;
    // console.log('🔐 [APP] Initializing authentication state...');

    // Initialize auth store - let it handle all token validation and refresh logic
    const isInitialized = await authStore.initialize();

    if (isInitialized) {
      // console.log('✅ [APP] Authentication initialized successfully');
    } else {
      // console.log('ℹ️ [APP] No valid authentication found - user will need to login');
    }

  } catch (error) {
    console.error('❌ [APP] Error during auth initialization:', error);
    hasGlobalError.value = true;
    globalError.value = error.message || 'Authentication initialization failed';

    // Clear any potentially corrupted auth state as fallback
    try {
      authStore.clearAuth();
      // console.log('🧹 [APP] Cleared corrupted auth state');
    } catch (clearError) {
      console.error('❌ [APP] Failed to clear auth state:', clearError);
    }
  } finally {
    isAuthLoading.value = false;
  }
};

// Ensure health check runs after app is fully mounted
onMounted(async () => {
  // First initialize theme
  initializeTheme();

  // Setup global keyboard shortcut event listeners
  window.addEventListener('fechatter:show-shortcuts-help', handleGlobalEvents);
  window.addEventListener('fechatter:open-settings', handleGlobalEvents);

  // Initialize authentication state validation
  await initializeAuthState();
});

onUnmounted(() => {
  // Cleanup global event listeners
  window.removeEventListener('fechatter:show-shortcuts-help', handleGlobalEvents);
  window.removeEventListener('fechatter:open-settings', handleGlobalEvents);
});
</script>

<style>
/* 引入Discord主题系统 */
@import './styles/theme.css';

/* 引入Channel List设计系统变量 */
@import './styles/channel-list-variables.css';

/* 引入自定义CSS */
@import './style.css';

/* 🎨 应用级别样式重置 */
#app {
  height: 100vh;              /* ✅ 恢复固定视口高度 */
  width: 100vw;
  overflow: hidden;           /* ✅ 控制总体溢出 */
  position: fixed;            /* ✅ 固定定位，防止滚动问题 */
  top: 0;
  left: 0;
  /* 确保应用使用主题系统 */
  background-color: var(--bg-primary);
  color: var(--text-primary);
  /* 使用Discord字体 */
  font-family: "gg sans", "Noto Sans", "Helvetica Neue", Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 1.375;
  /* 启用硬件加速 */
  transform: translateZ(0);
  backface-visibility: hidden;
  /* 防止用户选择 */
  -webkit-touch-callout: none;
  -webkit-tap-highlight-color: transparent;
  /* 阻止页面级滚动 */
  overscroll-behavior: none;
}

/* 🎯 全局滚动条样式 */
* {
  scrollbar-width: thin;
  scrollbar-color: var(--scrollbar-thumb) var(--scrollbar-track);
}

*::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

*::-webkit-scrollbar-track {
  background: var(--scrollbar-track);
}

*::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
  border-radius: 4px;
  border: 2px solid transparent;
  background-clip: content-box;
}

*::-webkit-scrollbar-thumb:hover {
  background: var(--scrollbar-thumb-hover);
  background-clip: content-box;
}

*::-webkit-scrollbar-corner {
  background: var(--scrollbar-track);
}

/* 🎯 全局选择样式 */
::selection {
  background: rgba(88, 101, 242, 0.2);
  color: var(--text-primary);
}

::-moz-selection {
  background: rgba(88, 101, 242, 0.2);
  color: var(--text-primary);
}

/* 🎯 全局焦点样式 */
*:focus {
  outline: 2px solid var(--border-focus);
  outline-offset: 2px;
}

*:focus:not(:focus-visible) {
  outline: none;
}

/* 🎯 图片优化 */
img {
  max-width: 100%;
  height: auto;
  border-radius: 4px;
}

/* 🎯 按钮重置 */
button {
  font-family: inherit;
  font-size: inherit;
  line-height: inherit;
  letter-spacing: inherit;
}

/* 🎯 链接样式 */
a {
  color: var(--text-link);
  text-decoration: none;
}

a:hover {
  text-decoration: underline;
}

/* 🎯 表单元素优化 */
input,
textarea,
select {
  font-family: inherit;
  font-size: inherit;
  line-height: inherit;
  background: var(--bg-input);
  border: 1px solid var(--border-primary);
  color: var(--text-primary);
}

input:focus,
textarea:focus,
select:focus {
  border-color: var(--border-focus);
  outline: none;
  box-shadow: 0 0 0 2px rgba(88, 101, 242, 0.2);
}

/* 🎯 代码元素 */
code,
pre {
  font-family: "Consolas", "Andale Mono WT", "Andale Mono", "Lucida Console", "Lucida Sans Typewriter", "DejaVu Sans Mono", "Bitstream Vera Sans Mono", "Liberation Mono", "Nimbus Mono L", "Monaco", "Courier New", Courier, monospace;
}

/* 🎯 移动端优化 */
@media (max-width: 768px) {
  #app {
    font-size: 14px;
  }
}

/* 🎯 打印优化 */
@media print {
  #app {
    background: white !important;
    color: black !important;
    font-size: 12pt;
  }

  * {
    background: transparent !important;
    color: black !important;
    box-shadow: none !important;
    text-shadow: none !important;
  }
}

/* 🎯 高对比度模式 */
@media (prefers-contrast: high) {
  #app {
    border: 1px solid var(--border-primary);
  }
}

/* 🎯 减少动画模式 */
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
}
</style>
