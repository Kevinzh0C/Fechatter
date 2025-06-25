<template>
  <div 
    ref="containerRef"
    :data-markdown-id="messageId"
    class="markdown-content"
    :class="{ 'is-processing': isProcessing }"
  >
    <!-- Initial content while processing -->
    <div v-if="!hasRendered" class="markdown-placeholder">
      {{ content }}
    </div>
  </div>
</template>

<script setup>
import { ref, watch, onMounted, computed } from 'vue';
import { useMarkdownRenderer } from '@/composables/useMarkdownRenderer';

const props = defineProps({
  content: {
    type: String,
    required: true
  },
  messageId: {
    type: String,
    required: true
  },
  immediate: {
    type: Boolean,
    default: false
  }
});

// Refs
const containerRef = ref(null);
const hasRendered = ref(false);

// Use markdown renderer
const {
  render,
  cleanup,
  isProcessing,
  processingTime,
  error,
  renderCount
} = useMarkdownRenderer({
  immediate: props.immediate,
  preserveScroll: true,
  maxConcurrent: 3
});

// Watch for content changes
watch(() => props.content, (newContent) => {
  if (newContent) {
    render(props.messageId, newContent);
    hasRendered.value = true;
  }
}, { immediate: true });

// Performance metrics
const performanceInfo = computed(() => ({
  processingTime: processingTime.value,
  renderCount: renderCount.value,
  hasError: !!error.value
}));

// Cleanup on unmount
onMounted(() => {
  return () => {
    cleanup(props.messageId);
  };
});

// Expose for parent components
defineExpose({
  performanceInfo,
  forceRender: () => render(props.messageId, props.content)
});
</script>

<style scoped>
.markdown-content {
  position: relative;
  word-wrap: break-word;
  overflow-wrap: break-word;
}

.markdown-content.is-processing {
  opacity: 0.95;
}

.markdown-placeholder {
  white-space: pre-wrap;
  font-family: inherit;
  color: var(--text-secondary);
}

/* Markdown styles */
.markdown-content :deep(h1),
.markdown-content :deep(h2),
.markdown-content :deep(h3),
.markdown-content :deep(h4),
.markdown-content :deep(h5),
.markdown-content :deep(h6) {
  margin-top: 1em;
  margin-bottom: 0.5em;
  font-weight: 600;
  line-height: 1.25;
}

.markdown-content :deep(h1) { font-size: 1.5em; }
.markdown-content :deep(h2) { font-size: 1.3em; }
.markdown-content :deep(h3) { font-size: 1.1em; }

.markdown-content :deep(p) {
  margin-top: 0;
  margin-bottom: 0.75em;
}

.markdown-content :deep(p:last-child) {
  margin-bottom: 0;
}

.markdown-content :deep(code) {
  background-color: var(--bg-secondary);
  padding: 0.125em 0.25em;
  border-radius: 3px;
  font-size: 0.875em;
  font-family: 'SF Mono', Monaco, Consolas, monospace;
}

.markdown-content :deep(pre) {
  background-color: var(--bg-secondary);
  padding: 0.75em;
  border-radius: 6px;
  overflow-x: auto;
  margin: 0.5em 0;
}

.markdown-content :deep(pre code) {
  background: none;
  padding: 0;
  font-size: 0.875em;
}

.markdown-content :deep(blockquote) {
  border-left: 4px solid var(--border-color);
  padding-left: 1em;
  margin: 0.5em 0;
  color: var(--text-secondary);
}

.markdown-content :deep(ul),
.markdown-content :deep(ol) {
  padding-left: 1.5em;
  margin: 0.5em 0;
}

.markdown-content :deep(li) {
  margin: 0.25em 0;
}

.markdown-content :deep(a) {
  color: var(--primary-color);
  text-decoration: none;
}

.markdown-content :deep(a:hover) {
  text-decoration: underline;
}

.markdown-content :deep(img) {
  max-width: 100%;
  height: auto;
  border-radius: 4px;
}

.markdown-content :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 0.5em 0;
}

.markdown-content :deep(th),
.markdown-content :deep(td) {
  border: 1px solid var(--border-color);
  padding: 0.5em;
  text-align: left;
}

.markdown-content :deep(th) {
  background-color: var(--bg-secondary);
  font-weight: 600;
}

/* Syntax highlighting */
.markdown-content :deep(.hljs) {
  background: transparent;
}

/* Smooth transitions for updates */
.markdown-content :deep([data-md-path]) {
  transition: opacity 0.15s ease;
}

.markdown-content :deep([data-md-path].updating) {
  opacity: 0.7;
}
</style>