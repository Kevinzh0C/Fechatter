<template>
  <div class="code-highlight-container">
    <!-- 代码头部 -->
    <div v-if="hasLanguage || showHeader" class="code-header">
      <div class="header-left">
        <span v-if="hasLanguage" class="language-label">{{ displayLanguage }}</span>
        <span v-if="filename" class="filename">{{ filename }}</span>
      </div>
      <div class="header-right">
        <button 
          class="copy-button" 
          @click="copyCode" 
          :class="{ 'copied': copySuccess }"
          :title="copySuccess ? 'Copied!' : 'Copy code'"
        >
          <svg v-if="!copySuccess" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
          </svg>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="20,6 9,17 4,12"></polyline>
          </svg>
          <span class="copy-text">{{ copySuccess ? 'Copied' : 'Copy' }}</span>
        </button>
      </div>
    </div>

    <!-- 代码块 -->
    <div class="code-wrapper" :class="{ 'no-header': !hasLanguage && !showHeader }">
      <pre class="code-block"><code 
        ref="codeElement" 
        :class="languageClass" 
        v-html="highlightedCode"
      ></code></pre>
      
      <!-- 行号（可选） -->
      <div v-if="showLineNumbers" class="line-numbers">
        <span 
          v-for="(line, index) in codeLines" 
          :key="index" 
          class="line-number"
        >{{ index + 1 }}</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, nextTick } from 'vue';
import hljs from 'highlight.js/lib/core';

// Import commonly used languages
import javascript from 'highlight.js/lib/languages/javascript';
import typescript from 'highlight.js/lib/languages/typescript';
import python from 'highlight.js/lib/languages/python';
import java from 'highlight.js/lib/languages/java';
import cpp from 'highlight.js/lib/languages/cpp';
import csharp from 'highlight.js/lib/languages/csharp';
import php from 'highlight.js/lib/languages/php';
import go from 'highlight.js/lib/languages/go';
import rust from 'highlight.js/lib/languages/rust';
import ruby from 'highlight.js/lib/languages/ruby';
import sql from 'highlight.js/lib/languages/sql';
import json from 'highlight.js/lib/languages/json';
import xml from 'highlight.js/lib/languages/xml';
import css from 'highlight.js/lib/languages/css';
import bash from 'highlight.js/lib/languages/bash';
import yaml from 'highlight.js/lib/languages/yaml';
import markdown from 'highlight.js/lib/languages/markdown';

// Register languages
hljs.registerLanguage('javascript', javascript);
hljs.registerLanguage('js', javascript);
hljs.registerLanguage('typescript', typescript);
hljs.registerLanguage('ts', typescript);
hljs.registerLanguage('python', python);
hljs.registerLanguage('py', python);
hljs.registerLanguage('java', java);
hljs.registerLanguage('cpp', cpp);
hljs.registerLanguage('c++', cpp);
hljs.registerLanguage('c', cpp);
hljs.registerLanguage('csharp', csharp);
hljs.registerLanguage('cs', csharp);
hljs.registerLanguage('php', php);
hljs.registerLanguage('go', go);
hljs.registerLanguage('golang', go);
hljs.registerLanguage('rust', rust);
hljs.registerLanguage('rs', rust);
hljs.registerLanguage('ruby', ruby);
hljs.registerLanguage('rb', ruby);
hljs.registerLanguage('sql', sql);
hljs.registerLanguage('json', json);
hljs.registerLanguage('xml', xml);
hljs.registerLanguage('html', xml);
hljs.registerLanguage('css', css);
hljs.registerLanguage('bash', bash);
hljs.registerLanguage('sh', bash);
hljs.registerLanguage('shell', bash);
hljs.registerLanguage('yaml', yaml);
hljs.registerLanguage('yml', yaml);
hljs.registerLanguage('markdown', markdown);
hljs.registerLanguage('md', markdown);

const props = defineProps({
  code: {
    type: String,
    required: true
  },
  language: {
    type: String,
    default: null
  },
  filename: {
    type: String,
    default: null
  },
  showLineNumbers: {
    type: Boolean,
    default: false
  },
  showHeader: {
    type: Boolean,
    default: true
  },
  autoDetect: {
    type: Boolean,
    default: true
  },
  theme: {
    type: String,
    default: 'github',
    validator: (value) => ['github', 'vs2015', 'atom-one-dark', 'github-dark'].includes(value)
  }
});

const codeElement = ref(null);
const copySuccess = ref(false);

// Language detection and display
const detectedLanguage = computed(() => {
  if (props.language) {
    return props.language.toLowerCase();
  }

  if (props.autoDetect && props.code.trim()) {
    try {
      const result = hljs.highlightAuto(props.code.trim(), [
        'javascript', 'typescript', 'python', 'java', 'cpp', 'csharp',
        'php', 'go', 'rust', 'ruby', 'sql', 'json', 'xml', 'css',
        'bash', 'yaml', 'markdown'
      ]);
      if (result.language && result.relevance > 3) {
        return result.language;
      }
    } catch (error) {
      console.warn('Language detection failed:', error);
    }
  }

  return null;
});

const hasLanguage = computed(() => !!detectedLanguage.value);

const displayLanguage = computed(() => {
  if (!detectedLanguage.value) return '';

  const languageMap = {
    'javascript': 'JavaScript',
    'js': 'JavaScript',
    'typescript': 'TypeScript',
    'ts': 'TypeScript',
    'python': 'Python',
    'py': 'Python',
    'java': 'Java',
    'cpp': 'C++',
    'c++': 'C++',
    'c': 'C',
    'csharp': 'C#',
    'cs': 'C#',
    'php': 'PHP',
    'go': 'Go',
    'golang': 'Go',
    'rust': 'Rust',
    'rs': 'Rust',
    'ruby': 'Ruby',
    'rb': 'Ruby',
    'sql': 'SQL',
    'json': 'JSON',
    'xml': 'XML',
    'html': 'HTML',
    'css': 'CSS',
    'bash': 'Bash',
    'sh': 'Shell',
    'shell': 'Shell',
    'yaml': 'YAML',
    'yml': 'YAML',
    'markdown': 'Markdown',
    'md': 'Markdown'
  };

  return languageMap[detectedLanguage.value] || detectedLanguage.value.toUpperCase();
});

const languageClass = computed(() => {
  return detectedLanguage.value ? `language-${detectedLanguage.value}` : '';
});

const codeLines = computed(() => {
  return props.code.trim().split('\n');
});

const highlightedCode = computed(() => {
  if (!props.code.trim()) return '';

  try {
    if (detectedLanguage.value && hljs.getLanguage(detectedLanguage.value)) {
      const result = hljs.highlight(props.code.trim(), {
        language: detectedLanguage.value,
        ignoreIllegals: true
      });
      return result.value;
    } else {
      // Auto-detect if no specific language
      const result = hljs.highlightAuto(props.code.trim());
      return result.value;
    }
  } catch (error) {
    console.warn('Code highlighting failed:', error);
    // Fallback to escaped plain text
    return props.code.trim()
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }
});

// Copy functionality
const copyCode = async () => {
  try {
    await navigator.clipboard.writeText(props.code.trim());
    copySuccess.value = true;
    setTimeout(() => {
      copySuccess.value = false;
    }, 2000);
  } catch (error) {
    console.warn('Copy failed:', error);
    // Fallback for older browsers
    try {
      const textArea = document.createElement('textarea');
      textArea.value = props.code.trim();
      textArea.style.position = 'fixed';
      textArea.style.opacity = '0';
      document.body.appendChild(textArea);
      textArea.select();
      document.execCommand('copy');
      document.body.removeChild(textArea);
      copySuccess.value = true;
      setTimeout(() => {
        copySuccess.value = false;
      }, 2000);
    } catch (fallbackError) {
      console.error('Copy fallback failed:', fallbackError);
    }
  }
};

onMounted(() => {
  // Apply highlighting after mount
  nextTick(() => {
    if (codeElement.value && !highlightedCode.value) {
      hljs.highlightElement(codeElement.value);
    }
  });
});
</script>

<style scoped>
/* Import highlight.js theme based on props */
@import 'highlight.js/styles/github.css';

.code-highlight-container {
  margin: 12px 0;
  border-radius: 8px;
  background: #ffffff;
  border: 1px solid #e1e4e8;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
  transition: all 0.2s ease;
  overflow: hidden;
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', 'Source Code Pro', monospace;
}

.code-highlight-container:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  border-color: #d0d7de;
}

.code-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: linear-gradient(135deg, #f6f8fa 0%, #f1f3f4 100%);
  border-bottom: 1px solid #e1e4e8;
  min-height: 44px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.language-label {
  font-weight: 600;
  color: #24292f;
  text-transform: uppercase;
  font-size: 11px;
  letter-spacing: 0.5px;
  background: #e1e4e8;
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid #d0d7de;
  transition: all 0.2s ease;
}

.filename {
  font-size: 13px;
  color: #656d76;
  font-weight: 500;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
}

.copy-button {
  display: flex;
  align-items: center;
  padding: 6px 12px;
  gap: 6px;
  background: #ffffff;
  border: 1px solid #d0d7de;
  border-radius: 6px;
  color: #24292f;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 12px;
  font-weight: 500;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
  min-width: 80px;
  justify-content: center;
}

.copy-button:hover {
  background: #f3f4f6;
  border-color: #8c959f;
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.copy-button.copied {
  background: #dcfce7;
  border-color: #86efac;
  color: #166534;
  animation: copySuccess 0.3s ease;
}

@keyframes copySuccess {
  0% { transform: scale(1); }
  50% { transform: scale(1.05); }
  100% { transform: scale(1); }
}

.copy-button:active {
  transform: scale(0.95);
}

.copy-text {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.2px;
}

.code-wrapper {
  position: relative;
  display: flex;
}

.code-wrapper.no-header {
  border-radius: 8px;
}

.code-block {
  flex: 1;
  margin: 0;
  padding: 16px 20px;
  background: #ffffff;
  overflow-x: auto;
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', 'Source Code Pro', monospace;
  font-size: 14px;
  line-height: 1.6;
  border-radius: 0;
  color: #24292e;
}

.code-block code {
  background: transparent;
  padding: 0;
  border: none;
  font-family: inherit;
  color: inherit;
  font-variant-ligatures: common-ligatures;
  display: block;
  white-space: pre;
}

.line-numbers {
  display: flex;
  flex-direction: column;
  padding: 16px 8px 16px 16px;
  background: #f6f8fa;
  border-right: 1px solid #e1e4e8;
  color: #656d76;
  font-size: 12px;
  line-height: 1.6;
  user-select: none;
  min-width: 40px;
  text-align: right;
}

.line-number {
  height: 22.4px; /* Match line-height * font-size */
  display: flex;
  align-items: center;
  justify-content: flex-end;
}

/* Enhanced highlight.js styles */
.code-block :deep(.hljs) {
  background: transparent;
  color: #24292e;
}

.code-block :deep(.hljs-comment),
.code-block :deep(.hljs-quote) {
  color: #6a737d;
  font-style: italic;
}

.code-block :deep(.hljs-keyword),
.code-block :deep(.hljs-selector-tag),
.code-block :deep(.hljs-literal) {
  color: #d73a49;
  font-weight: 600;
}

.code-block :deep(.hljs-string),
.code-block :deep(.hljs-doctag) {
  color: #032f62;
}

.code-block :deep(.hljs-number),
.code-block :deep(.hljs-variable),
.code-block :deep(.hljs-template-variable),
.code-block :deep(.hljs-tag .hljs-attr) {
  color: #005cc5;
}

.code-block :deep(.hljs-function),
.code-block :deep(.hljs-title) {
  color: #6f42c1;
  font-weight: 600;
}

.code-block :deep(.hljs-attr),
.code-block :deep(.hljs-name),
.code-block :deep(.hljs-attribute) {
  color: #e36209;
}

.code-block :deep(.hljs-builtin-name),
.code-block :deep(.hljs-built_in) {
  color: #005cc5;
  font-weight: 500;
}

/* Enhanced scrollbar */
.code-block::-webkit-scrollbar {
  height: 8px;
}

.code-block::-webkit-scrollbar-track {
  background: #f6f8fa;
  border-radius: 4px;
}

.code-block::-webkit-scrollbar-thumb {
  background: #d0d7de;
  border-radius: 4px;
}

.code-block::-webkit-scrollbar-thumb:hover {
  background: #8c959f;
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
  .code-highlight-container {
    background: #0d1117;
    border-color: #30363d;
  }

  .code-header {
    background: linear-gradient(135deg, #161b22 0%, #21262d 100%);
    border-color: #30363d;
  }

  .language-label {
    color: #f0f6fc;
    background: #30363d;
    border-color: #484f58;
  }

  .filename {
    color: #8b949e;
  }

  .copy-button {
    background: #21262d;
    color: #f0f6fc;
    border-color: #484f58;
  }

  .copy-button:hover {
    background: #30363d;
    border-color: #6e7681;
  }

  .copy-button.copied {
    background: #1a2e1a;
    border-color: #238636;
    color: #7ee787;
  }

  .code-block {
    background: #0d1117;
    color: #e6edf3;
  }

  .line-numbers {
    background: #161b22;
    border-color: #30363d;
    color: #7d8590;
  }

  .code-block :deep(.hljs) {
    color: #e6edf3;
  }

  .code-block :deep(.hljs-comment),
  .code-block :deep(.hljs-quote) {
    color: #8b949e;
  }

  .code-block :deep(.hljs-keyword),
  .code-block :deep(.hljs-selector-tag),
  .code-block :deep(.hljs-literal) {
    color: #ff7b72;
  }

  .code-block :deep(.hljs-string),
  .code-block :deep(.hljs-doctag) {
    color: #a5d6ff;
  }

  .code-block :deep(.hljs-number),
  .code-block :deep(.hljs-variable),
  .code-block :deep(.hljs-template-variable) {
    color: #79c0ff;
  }

  .code-block :deep(.hljs-function),
  .code-block :deep(.hljs-title) {
    color: #d2a8ff;
  }

  .code-block :deep(.hljs-attr),
  .code-block :deep(.hljs-name),
  .code-block :deep(.hljs-attribute) {
    color: #ffa657;
  }
}

/* Responsive design */
@media (max-width: 768px) {
  .code-header {
    padding: 8px 12px;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    min-height: auto;
  }

  .header-left,
  .header-right {
    width: 100%;
    justify-content: space-between;
  }

  .code-block {
    padding: 12px 16px;
    font-size: 13px;
  }

  .line-numbers {
    padding: 12px 6px 12px 12px;
    min-width: 35px;
    font-size: 11px;
  }
}
</style> 