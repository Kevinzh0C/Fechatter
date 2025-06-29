<template>
  <div class="code-container">
    <div v-if="language" class="code-header">
      <span class="lang-tag">{{ language.toUpperCase() }}</span>
      <button @click="handleCopy" class="copy-btn" :class="{ copied: isCopied }">
        {{ isCopied ? 'Copied!' : 'Copy' }}
      </button>
    </div>
    <pre class="code-block"><code>{{ code }}</code></pre>
  </div>
</template>

<script setup>
import { ref } from 'vue';

const props = defineProps({
  code: {
    type: String,
    required: true
  },
  language: {
    type: String,
    default: null
  },
  autoDetect: {
    type: Boolean,
    default: true
  }
});

const isCopied = ref(false);

const handleCopy = async () => {
  try {
    await navigator.clipboard.writeText(props.code);
    isCopied.value = true;
    setTimeout(() => {
      isCopied.value = false;
    }, 2000);
  } catch (error) {
    console.warn('Copy failed:', error);
  }
};
</script>

<style scoped>
.code-container {
  margin: 8px 0;
  border-radius: 6px;
  background: #f8f9fa;
  border: 1px solid #e1e4e8;
  overflow: hidden;
}

.code-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: #f1f3f4;
  border-bottom: 1px solid #e1e4e8;
}

.lang-tag {
  font-size: 10px;
  font-weight: 600;
  color: #24292f;
  background: #e1e4e8;
  padding: 2px 6px;
  border-radius: 3px;
}

.copy-btn {
  padding: 4px 8px;
  font-size: 11px;
  background: transparent;
  border: 1px solid #d0d7de;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.copy-btn:hover {
  background: #f3f4f6;
}

.copy-btn.copied {
  background: #dcfce7;
  color: #166534;
  border-color: #86efac;
}

.code-block {
  margin: 0;
  padding: 12px;
  background: #ffffff;
  overflow-x: auto;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 13px;
  line-height: 1.4;
}

.code-block code {
  background: transparent;
  padding: 0;
  border: none;
  font-family: inherit;
}
</style>