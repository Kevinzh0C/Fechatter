<template>
  <div class="code-block">
    <div class="code-header">
      <span class="language">{{ displayLanguage }}</span>
      <button @click="copyCode" class="copy-button">
        {{ copied ? '✓ Copied' : 'Copy' }}
      </button>
    </div>
    <div class="code-content" v-html="highlightedCode" v-if="!loading"></div>
    <div v-else class="code-loading">
      <div class="loading-shimmer"></div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue';
import { useCodeHighlighter } from '@/utils/codeHighlighter';

const props = defineProps({
  code: {
    type: String,
    required: true
  },
  language: {
    type: String,
    default: 'javascript'
  },
  theme: {
    type: String,
    default: 'github-light'
  }
});

// Use singleton code highlighter
const { highlight } = useCodeHighlighter();

const highlightedCode = ref('');
const loading = ref(true);
const copied = ref(false);

const displayLanguage = computed(() => {
  const langMap = {
    js: 'JavaScript',
    javascript: 'JavaScript',
    ts: 'TypeScript',
    typescript: 'TypeScript',
    py: 'Python',
    python: 'Python',
    sql: 'SQL',
    json: 'JSON',
    bash: 'Bash',
    sh: 'Shell'
  };
  return langMap[props.language] || props.language.toUpperCase();
});

// Highlight code with singleton
const highlightCode = async () => {
  if (!props.code) {
    highlightedCode.value = '';
    loading.value = false;
    return;
  }

  loading.value = true;
  try {
    // Use singleton highlighter
    highlightedCode.value = await highlight(props.code, props.language, props.theme);
  } catch (error) {
    console.error('Code highlighting failed:', error);
    // Fallback to plain text
    highlightedCode.value = `<pre><code>${escapeHtml(props.code)}</code></pre>`;
  } finally {
    loading.value = false;
  }
};

// Escape HTML for fallback
const escapeHtml = (text) => {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
};

// Copy code to clipboard
const copyCode = async () => {
  try {
    await navigator.clipboard.writeText(props.code);
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 2000);
  } catch (error) {
    console.error('Failed to copy code:', error);
  }
};

// Watch for code changes
watch(() => props.code, highlightCode);
watch(() => props.language, highlightCode);
watch(() => props.theme, highlightCode);

// Initial highlight
onMounted(() => {
  highlightCode();
});
</script>

<style scoped>
.code-block {
  border: 1px solid #e1e4e8;
  border-radius: 6px;
  overflow: hidden;
  margin: 8px 0;
  background: #f6f8fa;
}

.code-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  background: #f6f8fa;
  border-bottom: 1px solid #e1e4e8;
  font-size: 12px;
}

.language {
  color: #586069;
  font-weight: 500;
}

.copy-button {
  padding: 4px 8px;
  background: white;
  border: 1px solid #e1e4e8;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.copy-button:hover {
  background: #f3f4f6;
  border-color: #d1d5db;
}

.code-content {
  overflow-x: auto;
}

.code-content :deep(pre) {
  margin: 0;
  padding: 16px;
  overflow-x: auto;
}

.code-content :deep(code) {
  font-family: 'Monaco', 'Consolas', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.5;
}

.code-loading {
  padding: 16px;
  min-height: 100px;
}

.loading-shimmer {
  width: 100%;
  height: 20px;
  background: linear-gradient(90deg, #e1e4e8 25%, #f0f3f6 50%, #e1e4e8 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
  border-radius: 4px;
}

@keyframes shimmer {
  0% {
    background-position: 200% 0;
  }

  100% {
    background-position: -200% 0;
  }
}
</style>