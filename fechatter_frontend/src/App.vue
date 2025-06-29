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
      // console.log('[APP] Authentication initialized successfully');
    } else {
      // console.log('INFO: [APP] No valid authentication found - user will need to login');
    }

  } catch (error) {
    console.error('ERROR: [APP] Error during auth initialization:', error);
    hasGlobalError.value = true;
    globalError.value = error.message || 'Authentication initialization failed';

    // Clear any potentially corrupted auth state as fallback
    try {
      authStore.clearAuth();
      // console.log('🧹 [APP] Cleared corrupted auth state');
    } catch (clearError) {
      console.error('ERROR: [APP] Failed to clear auth state:', clearError);
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
/* Import Discord theme system */
@import './styles/theme.css';

/* Import Channel List design system variables */
@import './styles/channel-list-variables.css';

/* Import enhanced code highlighting styles */
@import './styles/enhanced-code-highlight.css';

/* Import custom CSS */
@import './style.css';

/* Application-level style reset */
#app {
  height: 100vh;              /* Fixed viewport height */
  width: 100vw;
  overflow: hidden;           /* Control overall overflow */
  position: fixed;            /* Fixed positioning to prevent scroll issues */
  top: 0;
  left: 0;
  /* Ensure app uses theme system */
  background-color: var(--bg-primary);
  color: var(--text-primary);
  /* Use Discord fonts */
  font-family: "gg sans", "Noto Sans", "Helvetica Neue", Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 1.375;
  /* Enable hardware acceleration */
  transform: translateZ(0);
  backface-visibility: hidden;
  /* Prevent user selection */
  -webkit-touch-callout: none;
  -webkit-tap-highlight-color: transparent;
  /* Prevent page-level scrolling */
  overscroll-behavior: none;
}

/* Global scrollbar styles */
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

/* Global selection styles */
::selection {
  background: rgba(88, 101, 242, 0.2);
  color: var(--text-primary);
}

::-moz-selection {
  background: rgba(88, 101, 242, 0.2);
  color: var(--text-primary);
}

/* Global focus styles */
*:focus {
  outline: 2px solid var(--border-focus);
  outline-offset: 2px;
}

*:focus:not(:focus-visible) {
  outline: none;
}

/* Image optimization */
img {
  max-width: 100%;
  height: auto;
  border-radius: 4px;
}

/* Button reset */
button {
  font-family: inherit;
  font-size: inherit;
  line-height: inherit;
  letter-spacing: inherit;
}

/* Link styles */
a {
  color: var(--text-link);
  text-decoration: none;
}

a:hover {
  text-decoration: underline;
}

/* Form element optimization */
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

/* Code elements */
code,
pre {
  font-family: "Consolas", "Andale Mono WT", "Andale Mono", "Lucida Console", "Lucida Sans Typewriter", "DejaVu Sans Mono", "Bitstream Vera Sans Mono", "Liberation Mono", "Nimbus Mono L", "Monaco", "Courier New", Courier, monospace;
}

/* Mobile optimization */
@media (max-width: 768px) {
  #app {
    font-size: 14px;
  }
}

/* Print optimization */
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

/* High contrast mode */
@media (prefers-contrast: high) {
  #app {
    border: 1px solid var(--border-primary);
  }
}

/* Reduced motion mode */
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
}

/* ✨ Global Styles */
body, html {
  margin: 0;
  padding: 0;
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
}

#app {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

/* ✨ Highlight.js Theme for Code Highlighting */
.hljs.dark {
  color: #f0f6fc;
  background: #0d1117;
}

.hljs.dark .hljs-comment,
.hljs.dark .hljs-quote {
  color: #8b949e;
  font-style: italic;
}

.hljs.dark .hljs-keyword,
.hljs.dark .hljs-selector-tag,
.hljs.dark .hljs-literal,
.hljs.dark .hljs-type {
  color: #ff7b72;
}

.hljs.dark .hljs-string,
.hljs.dark .hljs-doctag {
  color: #a5d6ff;
}

.hljs.dark .hljs-title,
.hljs.dark .hljs-section,
.hljs.dark .hljs-selector-id {
  color: #7ee787;
}

.hljs.dark .hljs-variable,
.hljs.dark .hljs-template-variable,
.hljs.dark .hljs-class .hljs-title {
  color: #ffa657;
}

.hljs.dark .hljs-number {
  color: #79c0ff;
}

.hljs.dark .hljs-function {
  color: #d2a8ff;
}

.hljs.dark .hljs-built_in,
.hljs.dark .hljs-builtin-name {
  color: #79c0ff;
}

/* Light theme */
.hljs.light {
  color: #24292f;
  background: #f6f8fa;
}

.hljs.light .hljs-comment,
.hljs.light .hljs-quote {
  color: #6a737d;
  font-style: italic;
}

.hljs.light .hljs-keyword,
.hljs.light .hljs-selector-tag,
.hljs.light .hljs-literal,
.hljs.light .hljs-type {
  color: #d73a49;
}

.hljs.light .hljs-string,
.hljs.light .hljs-doctag {
  color: #032f62;
}

.hljs.light .hljs-title,
.hljs.light .hljs-section,
.hljs.light .hljs-selector-id {
  color: #6f42c1;
}

.hljs.light .hljs-variable,
.hljs.light .hljs-template-variable,
.hljs.light .hljs-class .hljs-title {
  color: #6f42c1;
}

.hljs.light .hljs-number {
  color: #005cc5;
}

.hljs.light .hljs-function {
  color: #6f42c1;
}

.hljs.light .hljs-built_in,
.hljs.light .hljs-builtin-name {
  color: #005cc5;
}
</style>
