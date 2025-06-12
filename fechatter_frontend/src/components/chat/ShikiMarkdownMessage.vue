<template>
  <div class="shiki-markdown-message">
    <div
      :data-markdown-id="messageId"
      class="markdown-content"
      :class="{ 'is-processing': isProcessing || isHighlighting }"
    ></div>
    
    <!-- Loading indicator -->
    <div v-if="isProcessing || isHighlighting" class="loading-indicator">
      <span class="spinner"></span>
      <span class="text">{{ isHighlighting ? 'Highlighting code...' : 'Processing...' }}</span>
    </div>
    
    <!-- Error display -->
    <div v-if="error || highlightError" class="error-message">
      <span class="icon">⚠️</span>
      <span class="text">{{ error || highlightError }}</span>
    </div>
  </div>
</template>

<script setup>
import { computed, watch, onMounted, onUnmounted } from 'vue';
import { useShikiMarkdown } from '@/composables/useShikiMarkdown.js';

const props = defineProps({
  content: {
    type: String,
    required: true
  },
  messageId: {
    type: String,
    default: () => `msg-${Date.now()}-${Math.random()}`
  },
  theme: {
    type: String,
    default: 'dark'
  },
  lineNumbers: {
    type: Boolean,
    default: true
  },
  immediate: {
    type: Boolean,
    default: false
  }
});

// Use Shiki-enhanced markdown renderer
const {
  render,
  cleanup,
  isProcessing,
  error,
  isHighlighting,
  highlightError,
  currentTheme,
  setTheme
} = useShikiMarkdown({
  theme: props.theme,
  lineNumbers: props.lineNumbers,
  immediate: props.immediate,
  preserveScroll: true,
  useWorker: true,
  fallbackToOriginal: true
});

// Watch for content changes
watch(() => props.content, (newContent) => {
  if (newContent) {
    render(props.messageId, newContent);
  }
}, { immediate: true });

// Watch for theme changes
watch(() => props.theme, (newTheme) => {
  if (newTheme !== currentTheme.value) {
    setTheme(newTheme);
  }
});

// Cleanup on unmount
onUnmounted(() => {
  cleanup(props.messageId);
});
</script>

<style scoped>
.shiki-markdown-message {
  position: relative;
}

.markdown-content {
  transition: opacity 0.2s;
}

.markdown-content.is-processing {
  opacity: 0.7;
}

/* Loading indicator */
.loading-indicator {
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.25rem 0.75rem;
  background: rgba(0, 0, 0, 0.1);
  border-radius: 1rem;
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.spinner {
  width: 12px;
  height: 12px;
  border: 2px solid transparent;
  border-top-color: currentColor;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Error message */
.error-message {
  margin-top: 0.5rem;
  padding: 0.5rem;
  background: #fee;
  border: 1px solid #fcc;
  border-radius: 0.25rem;
  color: #c00;
  font-size: 0.875rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.dark .error-message {
  background: rgba(239, 68, 68, 0.1);
  border-color: rgba(239, 68, 68, 0.3);
  color: #ef4444;
}

/* Markdown content styles */
.markdown-content :deep(p) {
  margin-bottom: 1rem;
}

.markdown-content :deep(p:last-child) {
  margin-bottom: 0;
}

.markdown-content :deep(h1),
.markdown-content :deep(h2),
.markdown-content :deep(h3) {
  margin-top: 1.5rem;
  margin-bottom: 0.75rem;
  font-weight: 600;
}

.markdown-content :deep(h1) {
  font-size: 1.5rem;
}

.markdown-content :deep(h2) {
  font-size: 1.25rem;
}

.markdown-content :deep(h3) {
  font-size: 1.125rem;
}

.markdown-content :deep(ul),
.markdown-content :deep(ol) {
  margin-bottom: 1rem;
  padding-left: 1.5rem;
}

.markdown-content :deep(li) {
  margin-bottom: 0.25rem;
}

.markdown-content :deep(blockquote) {
  margin: 1rem 0;
  padding-left: 1rem;
  border-left: 4px solid var(--border-color);
  color: var(--text-secondary);
}

.markdown-content :deep(a) {
  color: var(--primary-color);
  text-decoration: none;
}

.markdown-content :deep(a:hover) {
  text-decoration: underline;
}

/* Code block animations */
.markdown-content :deep(.code-block-wrapper) {
  animation: fadeIn 0.3s ease-in;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>