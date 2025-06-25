<template>
  <!-- @ts-nocheck -->
  <div class="live-markdown-preview" :class="{ 'is-loading': isLoading, 'has-error': error }">
    <!-- 预览头部 -->
    <div class="preview-header">
      <div class="preview-title">
        <svg class="preview-icon" width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path
            d="M14.85 3H1.15C.52 3 0 3.52 0 4.15v7.69C0 12.48.52 13 1.15 13h13.69c.64 0 1.15-.52 1.15-1.15v-7.7C16 3.52 15.48 3 14.85 3z" />
        </svg>
        <span>Live Preview</span>
        <div v-if="isLoading" class="loading-spinner"></div>
      </div>

      <div class="preview-controls">
        <!-- 主题切换 -->
        <button @click="toggleTheme" class="control-btn"
          :title="`Switch to ${theme === 'dark' ? 'light' : 'dark'} theme`">
          <svg v-if="theme === 'dark'" width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8z" />
          </svg>
          <svg v-else width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
            <path
              d="M6 .278a.768.768 0 0 1 .08.858 7.208 7.208 0 0 0-.878 3.46c0 4.021 3.278 7.277 7.318 7.277.527 0 1.04-.055 1.533-.16a.787.787 0 0 1 .81.316.733.733 0 0 1-.031.893A8.349 8.349 0 0 1 8.344 16C3.734 16 0 12.286 0 7.71 0 4.266 2.114 1.312 5.124.06A.752.752 0 0 1 6 .278z" />
          </svg>
        </button>

        <!-- 同步滚动 -->
        <button @click="toggleSyncScroll" class="control-btn" :class="{ active: syncScroll }" title="Sync scroll">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
            <path
              d="M2 4a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V4zm2-1a1 1 0 0 0-1 1v8a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1V4a1 1 0 0 0-1-1H4z" />
            <path d="M8 15A7 7 0 1 1 8 1a7 7 0 0 1 0 14zm0 1A8 8 0 1 0 8 0a8 8 0 0 0 0 16z" />
          </svg>
        </button>

        <!-- 性能指标 -->
        <div v-if="showMetrics && metrics" class="metrics-display" title="Render time">
          <span class="metric-value">{{ metrics.renderTime }}ms</span>
        </div>
      </div>
    </div>

    <!-- 预览内容 -->
    <div class="preview-content" ref="previewContentRef" @scroll="handleScroll">
      <!-- 加载状态 -->
      <div v-if="isLoading && !renderedContent" class="loading-state">
        <div class="loading-spinner"></div>
        <span>Rendering preview...</span>
      </div>

      <!-- 错误状态 -->
      <div v-else-if="error" class="error-state">
        <svg class="error-icon" width="24" height="24" viewBox="0 0 16 16" fill="currentColor">
          <path
            d="M16 8A8 8 0 1 1 0 8a8 8 0 0 1 16 0zM8 4a.905.905 0 0 0-.9.995l.35 3.507a.552.552 0 0 0 1.1 0l.35-3.507A.905.905 0 0 0 8 4zm.002 6a1 1 0 1 0 0 2 1 1 0 0 0 0-2z" />
        </svg>
        <span class="error-message">Preview failed to render</span>
        <button @click="retryRender" class="retry-btn">Retry</button>
      </div>

      <!-- 空状态 -->
      <div v-else-if="!content.trim()" class="empty-state">
        <svg class="empty-icon" width="48" height="48" viewBox="0 0 16 16" fill="currentColor">
          <path
            d="M14.5 3a.5.5 0 0 1 .5.5v9a.5.5 0 0 1-.5.5h-13a.5.5 0 0 1-.5-.5v-9a.5.5 0 0 1 .5-.5h13zm-13-1A1.5 1.5 0 0 0 0 3.5v9A1.5 1.5 0 0 0 1.5 14h13a1.5 1.5 0 0 0 1.5-1.5v-9A1.5 1.5 0 0 0 14.5 2h-13z" />
          <path
            d="M3 5.5a.5.5 0 0 1 .5-.5h9a.5.5 0 0 1 0 1h-9a.5.5 0 0 1-.5-.5zM3 8a.5.5 0 0 1 .5-.5h9a.5.5 0 0 1 0 1h-9A.5.5 0 0 1 3 8zm0 2.5a.5.5 0 0 1 .5-.5h6a.5.5 0 0 1 0 1h-6a.5.5 0 0 1-.5-.5z" />
        </svg>
        <span class="empty-message">Start typing to see preview...</span>
      </div>

      <!-- 渲染内容 -->
      <div v-else class="rendered-content" :class="`theme-${theme}`" v-html="renderedContent">
      </div>
    </div>

    <!-- 底部状态栏 -->
    <div v-if="showStatus" class="preview-status">
      <div class="status-left">
        <span v-if="wordCount" class="word-count">{{ wordCount }} words</span>
        <span v-if="lineCount" class="line-count">{{ lineCount }} lines</span>
        <span v-if="charCount" class="char-count">{{ charCount }} chars</span>
      </div>

      <div class="status-right">
        <span v-if="cacheHitRate !== null" class="cache-info" title="Cache hit rate">
          Cache: {{ Math.round(cacheHitRate * 100) }}%
        </span>
        <span v-if="lastRenderTime" class="render-time" title="Last render time">
          {{ lastRenderTime }}ms
        </span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import { unifiedMarkdownEngine } from '@/services/UnifiedMarkdownEngine.js';

const props = defineProps({
  content: {
    type: String,
    default: ''
  },
  theme: {
    type: String,
    default: 'dark',
    validator: (value) => ['light', 'dark'].includes(value)
  },
  syncScroll: {
    type: Boolean,
    default: true
  },
  showMetrics: {
    type: Boolean,
    default: false
  },
  showStatus: {
    type: Boolean,
    default: true
  },
  debounceMs: {
    type: Number,
    default: 300
  },
  immediate: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits(['theme-changed', 'scroll', 'render-complete', 'render-error']);

// Refs
const previewContentRef = ref(null);

// State
const renderedContent = ref('');
const isLoading = ref(false);
const error = ref(null);
const metrics = ref(null);
const lastRenderTime = ref(0);
const renderCount = ref(0);

// Debounce timer
let debounceTimer = null;
let isScrolling = false;

// Computed properties
const wordCount = computed(() => {
  if (!props.content.trim()) return 0;
  return props.content.trim().split(/\s+/).length;
});

const lineCount = computed(() => {
  return props.content.split('\n').length;
});

const charCount = computed(() => {
  return props.content.length;
});

const cacheHitRate = computed(() => {
  const engineMetrics = unifiedMarkdownEngine.getMetrics();
  return engineMetrics.cacheHitRate || null;
});

// Methods
const renderPreview = async (immediate = false) => {
  // Clear existing timer
  if (debounceTimer) {
    clearTimeout(debounceTimer);
    debounceTimer = null;
  }

  const render = async () => {
    if (!props.content.trim()) {
      renderedContent.value = '';
      isLoading.value = false;
      error.value = null;
      return;
    }

    isLoading.value = true;
    error.value = null;
    const startTime = Date.now();

    try {
      // Update engine theme
      unifiedMarkdownEngine.updateConfig({ theme: props.theme });

      // Render content
      const html = await unifiedMarkdownEngine.renderPreview(props.content, {
        theme: props.theme,
        cache: true
      });

      renderedContent.value = html;
      lastRenderTime.value = Date.now() - startTime;
      renderCount.value++;

      // Get metrics
      metrics.value = unifiedMarkdownEngine.getMetrics();

      emit('render-complete', {
        renderTime: lastRenderTime.value,
        renderCount: renderCount.value,
        metrics: metrics.value
      });

    } catch (err) {
      console.error('Preview render failed:', err);
      error.value = err;
      emit('render-error', err);
    } finally {
      isLoading.value = false;
    }
  };

  if (immediate || props.debounceMs <= 0) {
    await render();
  } else {
    debounceTimer = setTimeout(render, props.debounceMs);
  }
};

const retryRender = async () => {
  error.value = null;
  await renderPreview(true);
};

const toggleTheme = () => {
  const newTheme = props.theme === 'dark' ? 'light' : 'dark';
  emit('theme-changed', newTheme);
};

const toggleSyncScroll = () => {
  emit('scroll-sync-changed', !props.syncScroll);
};

const handleScroll = (event) => {
  if (!props.syncScroll || isScrolling) return;

  const element = event.target;
  const scrollRatio = element.scrollTop / (element.scrollHeight - element.clientHeight);

  emit('scroll', { scrollRatio, direction: 'preview' });
};

// External scroll sync
const syncScrollPosition = (scrollRatio) => {
  if (!props.syncScroll || !previewContentRef.value) return;

  isScrolling = true;

  const element = previewContentRef.value;
  const maxScroll = element.scrollHeight - element.clientHeight;
  element.scrollTop = maxScroll * scrollRatio;

  nextTick(() => {
    isScrolling = false;
  });
};

// Watch for content changes
watch(() => props.content, () => {
  renderPreview();
}, { immediate: props.immediate });

// Watch for theme changes
watch(() => props.theme, (newTheme) => {
  unifiedMarkdownEngine.updateConfig({ theme: newTheme });
  renderPreview(true);
});

// Initial render
onMounted(() => {
  if (props.content.trim()) {
    renderPreview(props.immediate);
  }
});

// Cleanup
onUnmounted(() => {
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
});

// Expose methods
defineExpose({
  renderPreview,
  retryRender,
  syncScrollPosition,
  getMetrics: () => metrics.value
});
</script>

<style scoped>
.live-markdown-preview {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #ffffff;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  overflow: hidden;
}

.preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: #f8fafc;
  border-bottom: 1px solid #e5e7eb;
}

.preview-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
  color: #374151;
  font-size: 14px;
}

.preview-icon {
  color: #6366f1;
}

.preview-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

.control-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 4px;
  cursor: pointer;
  color: #6b7280;
  transition: all 0.2s ease;
}

.control-btn:hover {
  background: #f3f4f6;
  border-color: #d1d5db;
  color: #374151;
}

.control-btn.active {
  background: #eff6ff;
  border-color: #3b82f6;
  color: #1d4ed8;
}

.metrics-display {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #6b7280;
}

.metric-value {
  font-family: monospace;
  font-weight: 600;
}

.preview-content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  line-height: 1.6;
}

/* 状态显示 */
.loading-state,
.error-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  gap: 12px;
  color: #6b7280;
}

.loading-spinner {
  width: 20px;
  height: 20px;
  border: 2px solid #e5e7eb;
  border-top: 2px solid #6366f1;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }

  100% {
    transform: rotate(360deg);
  }
}

.error-icon {
  color: #dc2626;
}

.retry-btn {
  padding: 6px 12px;
  background: #eff6ff;
  border: 1px solid #3b82f6;
  border-radius: 4px;
  color: #1d4ed8;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s ease;
}

.retry-btn:hover {
  background: #dbeafe;
}

.empty-icon {
  color: #9ca3af;
}

/* 渲染内容样式 */
.rendered-content {
  min-height: 100%;
}

/* KaTeX 数学公式样式 */
.rendered-content :deep(.katex) {
  font-size: 1.1em;
}

.rendered-content :deep(.katex-display) {
  margin: 1em 0;
  text-align: center;
}

/* 代码高亮样式 */
.rendered-content :deep(.unified-code-block) {
  margin: 1em 0;
  border-radius: 6px;
  overflow: hidden;
}

.rendered-content :deep(.unified-code-block pre) {
  margin: 0;
  padding: 1em;
  overflow-x: auto;
}

/* 主题样式 */
.theme-dark {
  color: #e5e7eb;
}

.theme-dark :deep(h1),
.theme-dark :deep(h2),
.theme-dark :deep(h3),
.theme-dark :deep(h4),
.theme-dark :deep(h5),
.theme-dark :deep(h6) {
  color: #f9fafb;
}

.theme-dark :deep(code) {
  background: #374151;
  color: #f9fafb;
}

.theme-dark :deep(blockquote) {
  border-left-color: #6b7280;
  color: #d1d5db;
}

.theme-light {
  color: #374151;
}

.theme-light :deep(h1),
.theme-light :deep(h2),
.theme-light :deep(h3),
.theme-light :deep(h4),
.theme-light :deep(h5),
.theme-light :deep(h6) {
  color: #111827;
}

.theme-light :deep(code) {
  background: #f3f4f6;
  color: #374151;
}

.theme-light :deep(blockquote) {
  border-left-color: #d1d5db;
  color: #6b7280;
}

/* 底部状态栏 */
.preview-status {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  background: #f8fafc;
  border-top: 1px solid #e5e7eb;
  font-size: 12px;
  color: #6b7280;
}

.status-left,
.status-right {
  display: flex;
  gap: 12px;
}

.word-count,
.line-count,
.char-count,
.cache-info,
.render-time {
  font-family: monospace;
}

/* 自定义滚动条 */
.preview-content::-webkit-scrollbar {
  width: 8px;
}

.preview-content::-webkit-scrollbar-track {
  background: #f1f5f9;
}

.preview-content::-webkit-scrollbar-thumb {
  background: #cbd5e1;
  border-radius: 4px;
}

.preview-content::-webkit-scrollbar-thumb:hover {
  background: #94a3b8;
}

/* 响应式 */
@media (max-width: 768px) {
  .preview-header {
    padding: 8px 12px;
  }

  .preview-content {
    padding: 12px;
  }

  .preview-status {
    padding: 6px 12px;
    flex-direction: column;
    gap: 4px;
  }

  .status-left,
  .status-right {
    gap: 8px;
  }
}
</style>