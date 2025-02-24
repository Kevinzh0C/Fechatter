<template>
  <div class="code-block-wrapper" :class="{ 'is-loading': isLoading, 'has-error': hasError }" :data-lang="language"
    :data-title="title">
    <!-- Code Title -->
    <div v-if="title" class="code-title">
      <span class="title-text">{{ title }}</span>
      <div class="title-actions">
        <button class="copy-button" @click="copyCode" :disabled="!code" title="Copy code">
          <ClipboardDocumentIcon class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="isLoading" class="code-loading">
      <div class="loading-spinner"></div>
      <span>Highlighting code...</span>
    </div>

    <!-- Error State -->
    <div v-else-if="hasError" class="code-error">
      <div class="error-content">
        <ExclamationTriangleIcon class="w-5 h-5 text-red-500" />
        <span>Failed to highlight code</span>
        <button @click="retryHighlight" class="retry-button">
          Retry
        </button>
      </div>
      <pre class="fallback-code"><code>{{ code }}</code></pre>
    </div>

    <!-- Highlighted Code -->
    <div v-else class="code-content" v-html="highlightedCode"></div>

    <!-- Copy Success Toast -->
    <Transition name="toast">
      <div v-if="showCopySuccess" class="copy-toast">
        <CheckIcon class="w-4 h-4" />
        Copied!
      </div>
    </Transition>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { ClipboardDocumentIcon, CheckIcon, ExclamationTriangleIcon } from '@heroicons/vue/24/outline'

// Props
const props = defineProps({
  code: {
    type: String,
    required: true
  },
  language: {
    type: String,
    default: 'plaintext'
  },
  title: {
    type: String,
    default: null
  },
  lineNumbers: {
    type: Boolean,
    default: true
  },
  theme: {
    type: String,
    default: 'dark',
    validator: (value) => ['light', 'dark'].includes(value)
  },
  maxHeight: {
    type: String,
    default: '400px'
  }
})

// Reactive State
const highlightedCode = ref('')
const isLoading = ref(true)
const hasError = ref(false)
const errorMessage = ref('')
const showCopySuccess = ref(false)

// Computed
const codeStyle = computed(() => ({
  maxHeight: props.maxHeight
}))

// Methods
const highlightCode = async () => {
  if (!props.code.trim()) {
    highlightedCode.value = '<pre><code></code></pre>'
    isLoading.value = false
    return
  }

  try {
    isLoading.value = true
    hasError.value = false
    errorMessage.value = ''

    // Dynamic import to avoid bundle size issues
    const { highlightCodeAsync } = await import('@/utils/codeHighlight')

    const result = await highlightCodeAsync(props.code, props.language, {
      theme: props.theme,
      lineNumbers: props.lineNumbers,
      cache: true
    })

    highlightedCode.value = result

    if (import.meta.env.DEV) {
      console.log(`✨ Code highlighted: ${props.language}`)
    }
  } catch (error) {
    console.error('💥 Code highlighting failed:', error)
    hasError.value = true
    errorMessage.value = error.message || 'Unknown error'

    // Fallback to plain code
    highlightedCode.value = `<pre class="shiki"><code class="language-${props.language}">${escapeHtml(props.code)}</code></pre>`
  } finally {
    isLoading.value = false
  }
}

const retryHighlight = () => {
  hasError.value = false
  highlightCode()
}

const copyCode = async () => {
  if (!props.code) return

  try {
    await navigator.clipboard.writeText(props.code)
    showCopySuccess.value = true

    // Hide toast after 2 seconds
    setTimeout(() => {
      showCopySuccess.value = false
    }, 2000)
  } catch (error) {
    console.error('Failed to copy code:', error)
  }
}

const escapeHtml = (str) => {
  const div = document.createElement('div')
  div.textContent = str
  return div.innerHTML
}

// Watchers
watch([() => props.code, () => props.language, () => props.theme], () => {
  highlightCode()
}, { immediate: false })

// Lifecycle
onMounted(() => {
  highlightCode()
})
</script>

<style scoped>
.code-block-wrapper {
  @apply relative rounded-lg overflow-hidden shadow-sm border;
  background: var(--code-bg, #1e1e1e);
  border-color: var(--code-border, #333);
  margin: 1rem 0;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', 'Monaco', monospace;
}

.code-title {
  @apply flex items-center justify-between px-4 py-2 border-b;
  background: var(--code-title-bg, #252526);
  border-color: var(--code-border, #333);
}

.title-text {
  @apply text-sm font-medium;
  color: var(--code-title-text, #cccccc);
}

.title-actions {
  @apply flex items-center gap-2;
}

.copy-button {
  @apply p-1 rounded transition-colors;
  color: var(--code-muted, #858585);
}

.copy-button:hover:not(:disabled) {
  @apply bg-gray-700;
  color: var(--code-text, #cccccc);
}

.copy-button:disabled {
  @apply opacity-50 cursor-not-allowed;
}

.code-loading {
  @apply flex items-center justify-center gap-3 p-8;
  color: var(--code-muted, #858585);
}

.loading-spinner {
  @apply w-5 h-5 border-2 border-current border-t-transparent rounded-full animate-spin;
}

.code-error {
  @apply p-4;
}

.error-content {
  @apply flex items-center gap-2 mb-3 text-red-400;
}

.retry-button {
  @apply ml-auto px-3 py-1 text-xs bg-red-500/20 text-red-400 rounded hover:bg-red-500/30 transition-colors;
}

.fallback-code {
  @apply p-4 bg-gray-800 rounded text-gray-300 text-sm overflow-x-auto;
}

.code-content {
  @apply overflow-hidden;
}

.copy-toast {
  @apply absolute top-2 right-2 flex items-center gap-1 px-3 py-1 bg-green-500 text-white text-xs rounded shadow-lg;
  z-index: 10;
}

/* Transitions */
.toast-enter-active,
.toast-leave-active {
  @apply transition-all duration-200;
}

.toast-enter-from,
.toast-leave-to {
  @apply opacity-0 transform translate-y-1;
}

/* Loading state */
.is-loading {
  @apply pointer-events-none;
}

/* Error state */
.has-error .code-content {
  @apply hidden;
}

/* Global code styling overrides */
:deep(.shiki) {
  @apply m-0 p-4 overflow-x-auto text-sm leading-relaxed;
  max-height: v-bind('codeStyle.maxHeight');
}

:deep(.shiki code) {
  @apply block w-full min-w-full;
}

:deep(.line) {
  @apply table-row;
}

:deep(.line-number) {
  @apply table-cell pr-4 text-right select-none w-1 whitespace-nowrap;
  color: var(--code-line-number, #858585);
}

:deep(.line-content) {
  @apply table-cell pl-2;
}

:deep(.line-content.highlighted) {
  @apply bg-blue-500/20 -mx-4 px-4;
}

/* Scrollbar styling */
:deep(.shiki::-webkit-scrollbar) {
  @apply h-2;
}

:deep(.shiki::-webkit-scrollbar-track) {
  @apply bg-transparent;
}

:deep(.shiki::-webkit-scrollbar-thumb) {
  @apply bg-gray-600 rounded;
}

:deep(.shiki::-webkit-scrollbar-thumb:hover) {
  @apply bg-gray-500;
}

/* Theme variables */
.code-block-wrapper {
  --code-bg: theme('colors.gray.900');
  --code-border: theme('colors.gray.700');
  --code-title-bg: theme('colors.gray.800');
  --code-title-text: theme('colors.gray.200');
  --code-text: theme('colors.gray.100');
  --code-muted: theme('colors.gray.400');
  --code-line-number: theme('colors.gray.500');
}

/* Light theme */
.light .code-block-wrapper {
  --code-bg: theme('colors.gray.50');
  --code-border: theme('colors.gray.200');
  --code-title-bg: theme('colors.gray.100');
  --code-title-text: theme('colors.gray.700');
  --code-text: theme('colors.gray.900');
  --code-muted: theme('colors.gray.500');
  --code-line-number: theme('colors.gray.400');
}
</style>