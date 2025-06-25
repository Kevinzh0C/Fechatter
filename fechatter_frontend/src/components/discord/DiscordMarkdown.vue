<template>
  <div class="discord-markdown" v-html="renderedContent" @click="handleClick" />
</template>

<script setup>
import { ref, computed, onMounted, nextTick } from 'vue';
import { remark } from 'remark';
import remarkGfm from 'remark-gfm';
import remarkBreaks from 'remark-breaks';
import remarkHtml from 'remark-html';
import { createHighlighter } from 'shiki';

const props = defineProps({
  content: {
    type: String,
    required: true
  },
  enableGfm: {
    type: Boolean,
    default: true
  },
  enableBreaks: {
    type: Boolean,
    default: true
  },
  theme: {
    type: String,
    default: 'github-light'
  },
  darkTheme: {
    type: String,
    default: 'github-dark'
  }
});

const emit = defineEmits(['link-click', 'mention-click', 'channel-click']);

// Shiki高亮器
let highlighter = null;
const isHighlighterReady = ref(false);

// 当前主题
const currentTheme = computed(() => {
  // 检测页面主要背景色来决定主题
  const bodyBg = getComputedStyle(document.body).backgroundColor;
  const isDark = document.documentElement.getAttribute('data-theme') === 'dark' || 
                 window.matchMedia('(prefers-color-scheme: dark)').matches ||
                 bodyBg.includes('rgb(26, 27, 29)'); // 检测深色背景
  
  // 针对白色聊天背景，使用适中的主题
  return isDark ? 'github-dark' : 'github-light';
});

// 初始化Shiki高亮器
const initializeHighlighter = async () => {
  try {
    highlighter = await createHighlighter({
      themes: ['github-dark', 'github-light', 'monokai', 'dracula', 'nord'],
      langs: [
        'javascript',
        'typescript',
        'python',
        'rust',
        'java',
        'cpp',
        'c',
        'csharp',
        'go',
        'ruby',
        'php',
        'html',
        'css',
        'scss',
        'json',
        'yaml',
        'toml',
        'xml',
        'sql',
        'bash',
        'shell',
        'powershell',
        'dockerfile',
        'markdown',
        'vue',
        'jsx',
        'tsx',
        'svelte',
        'kotlin',
        'swift',
        'dart',
        'lua',
        'perl',
        'r'
      ]
    });
    isHighlighterReady.value = true;
  } catch (error) {
    console.error('Failed to initialize Shiki highlighter:', error);
    isHighlighterReady.value = false;
  }
};

// 代码高亮函数
const highlightCode = (code, lang) => {
  if (!highlighter || !isHighlighterReady.value) {
    return `<pre class="discord-code-block"><code class="discord-code">${escapeHtml(code)}</code></pre>`;
  }

  try {
    // Language mapping for common aliases
    const languageMap = {
      'react': 'jsx',
      'reactjs': 'jsx',
      'ts': 'typescript',
      'js': 'javascript',
      'py': 'python',
      'rs': 'rust',
      'sh': 'bash',
      'yml': 'yaml',
      'htm': 'html'
    };

    // Map language or default to 'text'
    const mappedLang = languageMap[lang] || lang || 'text';

    const highlighted = highlighter.codeToHtml(code, {
      lang: mappedLang,
      theme: currentTheme.value,
      transformers: [{
        pre(node) {
          node.properties.class = 'discord-code-block shiki';
          // 移除Shiki的内联样式，使用我们的CSS
          node.properties.style = '';
        },
        code(node) {
          node.properties.class = 'discord-code shiki-code';
        }
      }]
    });

    // 添加复制按钮和语言标签
    const languageLabel = lang ? lang.toUpperCase() : 'TEXT';
    const copyButton = `
      <div class="code-header">
        <span class="code-language">${languageLabel}</span>
        <button class="copy-code-btn" onclick="copyCodeToClipboard(this)" data-code="${encodeURIComponent(code)}">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2 2v1"></path>
          </svg>
          Copy
        </button>
      </div>
    `;

    return `<div class="code-container">${copyButton}${highlighted}</div>`;
  } catch (error) {
    console.warn(`Code highlighting failed for language '${lang}':`, error.message);
    // Fallback to plain text with styling
    const languageLabel = lang ? lang.toUpperCase() : 'TEXT';
    const copyButton = `
      <div class="code-header">
        <span class="code-language">${languageLabel}</span>
        <button class="copy-code-btn" onclick="copyCodeToClipboard(this)" data-code="${encodeURIComponent(code)}">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2 2v1"></path>
          </svg>
          Copy
        </button>
      </div>
    `;
    return `<div class="code-container">${copyButton}<pre class="discord-code-block"><code class="discord-code">${escapeHtml(code)}</code></pre></div>`;
  }
};

// HTML转义
const escapeHtml = (text) => {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
};

// Markdown处理器
const processor = computed(() => {
  let proc = remark();

  if (props.enableGfm) {
    proc = proc.use(remarkGfm);
  }

  if (props.enableBreaks) {
    proc = proc.use(remarkBreaks);
  }

  return proc.use(remarkHtml, {
    sanitize: false
  });
});

// 渲染内容
const renderedContent = computed(() => {
  if (!props.content) return '';

  try {
    // 预处理Discord特殊语法
    let processedContent = props.content
      // 处理Discord风格的mentions
      .replace(/<@(\d+)>/g, '[@$1](@$1)')
      // 处理Discord风格的channels
      .replace(/<#(\d+)>/g, '[#$1](#$1)')
      // 处理Discord风格的roles
      .replace(/<@&(\d+)>/g, '[@&$1](@&$1)')
      // 处理Discord风格的emojis
      .replace(/<:([\w_]+):(\d+)>/g, ':$1:')
      // 处理spoiler标签
      .replace(/\|\|(.*?)\|\|/g, '<span class="discord-spoiler" onclick="this.classList.toggle(\'revealed\')">$1</span>');

    // 处理markdown
    const result = processor.value.processSync(processedContent);
    let html = result.toString();

    // 后处理：增强代码块
    html = html.replace(/<pre><code class="language-(\w+)">([\s\S]*?)<\/code><\/pre>/g, (match, lang, code) => {
      const decodedCode = code
        .replace(/&lt;/g, '<')
        .replace(/&gt;/g, '>')
        .replace(/&amp;/g, '&')
        .replace(/&quot;/g, '"')
        .replace(/&#39;/g, "'");
      
      return highlightCode(decodedCode, lang);
    });

    // 处理无语言的代码块
    html = html.replace(/<pre><code>([\s\S]*?)<\/code><\/pre>/g, (match, code) => {
      const decodedCode = code
        .replace(/&lt;/g, '<')
        .replace(/&gt;/g, '>')
        .replace(/&amp;/g, '&')
        .replace(/&quot;/g, '"')
        .replace(/&#39;/g, "'");
      
      return highlightCode(decodedCode, '');
    });

    // 处理行内代码
    html = html.replace(/<code>(.*?)<\/code>/g, '<code class="discord-inline-code">$1</code>');

    // 处理链接
    html = html.replace(/<a href="([^"]*)"[^>]*>(.*?)<\/a>/g, (match, href, text) => {
      if (href.startsWith('#')) {
        return `<span class="discord-channel-link" data-channel="${href.slice(1)}">${text}</span>`;
      } else if (href.startsWith('@')) {
        return `<span class="discord-mention" data-user="${href.slice(1)}">${text}</span>`;
      } else {
        return `<a href="${href}" class="discord-link" target="_blank" rel="noopener noreferrer">${text}</a>`;
      }
    });

    // 处理引用块
    html = html.replace(/<blockquote>([\s\S]*?)<\/blockquote>/g, '<div class="discord-quote">$1</div>');

    // 处理表格
    html = html.replace(/<table>([\s\S]*?)<\/table>/g, '<div class="discord-table-container"><table class="discord-table">$1</table></div>');

    return html;
  } catch (error) {
    console.error('Markdown processing failed:', error);
    return escapeHtml(props.content);
  }
});

// 点击事件处理
const handleClick = (event) => {
  const target = event.target;

  if (target.classList.contains('discord-link')) {
    event.preventDefault();
    emit('link-click', target.href);
  } else if (target.classList.contains('discord-mention')) {
    event.preventDefault();
    emit('mention-click', target.dataset.user);
  } else if (target.classList.contains('discord-channel-link')) {
    event.preventDefault();
    emit('channel-click', target.dataset.channel);
  }
};

// 全局复制代码函数
const setupCopyFunction = () => {
  if (!window.copyCodeToClipboard) {
    window.copyCodeToClipboard = async (button) => {
      try {
        const code = decodeURIComponent(button.dataset.code);
        await navigator.clipboard.writeText(code);

        const originalContent = button.innerHTML;
        button.innerHTML = `
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M20 6L9 17l-5-5"></path>
          </svg>
          Copied!
        `;
        button.classList.add('copied');

        setTimeout(() => {
          button.innerHTML = originalContent;
          button.classList.remove('copied');
        }, 2000);
      } catch (error) {
        console.error('Failed to copy code:', error);

        button.innerHTML = `
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12"></path>
          </svg>
          Failed
        `;
        button.classList.add('failed');

        setTimeout(() => {
          button.innerHTML = `
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
            </svg>
            Copy
          `;
          button.classList.remove('failed');
        }, 2000);
      }
    };
  }
};

onMounted(async () => {
  await initializeHighlighter();
  setupCopyFunction();
  
  // 监听主题变化
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  const handleThemeChange = () => {
    // 触发重新渲染以应用新主题
    if (highlighter && isHighlighterReady.value) {
      nextTick(() => {
        // 重新处理代码块
        const codeBlocks = document.querySelectorAll('.discord-code-block.shiki');
        codeBlocks.forEach(block => {
          block.classList.add('theme-updated');
        });
      });
    }
  };
  
  mediaQuery.addEventListener('change', handleThemeChange);
  
  // 清理
  return () => {
    mediaQuery.removeEventListener('change', handleThemeChange);
  };
});
</script>

<style scoped>
/* 🎨 Discord Markdown 样式 */
.discord-markdown {
  color: var(--text-primary);
  line-height: 1.375;
  word-wrap: break-word;
  font-size: 1rem;
}

/* 段落 */
.discord-markdown :deep(p) {
  margin: 0;
  line-height: 1.375;
}

.discord-markdown :deep(p + p) {
  margin-top: 4px;
}

/* 标题 */
.discord-markdown :deep(h1),
.discord-markdown :deep(h2),
.discord-markdown :deep(h3),
.discord-markdown :deep(h4),
.discord-markdown :deep(h5),
.discord-markdown :deep(h6) {
  margin: 8px 0 4px 0;
  font-weight: 600;
  color: var(--text-primary);
  line-height: 1.25;
}

.discord-markdown :deep(h1) {
  font-size: 1.5rem;
}

.discord-markdown :deep(h2) {
  font-size: 1.25rem;
}

.discord-markdown :deep(h3) {
  font-size: 1.125rem;
}

/* 强调 */
.discord-markdown :deep(strong) {
  font-weight: 700;
  color: var(--text-primary);
}

.discord-markdown :deep(em) {
  font-style: italic;
}

.discord-markdown :deep(del) {
  text-decoration: line-through;
  opacity: 0.6;
}

/* 行内代码 */
.discord-markdown :deep(.discord-inline-code) {
  background: rgba(175, 184, 193, 0.2);
  border: 1px solid rgba(175, 184, 193, 0.4);
  border-radius: 4px;
  padding: 0.2em 0.4em;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', 'Menlo', 'SF Mono', 'Monaco', monospace;
  font-size: 0.9em;
  color: #d1242f;
  white-space: pre-wrap;
  font-weight: 600;
  letter-spacing: 0.3px;
}

/* 代码块容器 */
.discord-markdown :deep(.code-container) {
  margin: 8px 0;
  border-radius: 8px;
  overflow: hidden;
  background: #f8f9fa;
  border: 1px solid #e9ecef;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
  transition: box-shadow 0.2s ease;
}

/* 代码头部 */
.discord-markdown :deep(.code-header) {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: linear-gradient(135deg, #f5f6f7 0%, #e9ecef 100%);
  border-bottom: 1px solid #dee2e6;
  font-size: 0.75rem;
  font-weight: 600;
}

.discord-markdown :deep(.code-language) {
  color: #0969da;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', 'Source Code Pro', 'Fira Code', monospace;
  font-weight: 600;
  background: rgba(9, 105, 218, 0.1);
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid rgba(9, 105, 218, 0.2);
}

.discord-markdown :deep(.copy-code-btn) {
  display: flex;
  align-items: center;
  gap: 6px;
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid #0969da;
  border-radius: 6px;
  padding: 8px 12px;
  font-size: 0.75rem;
  color: #0969da;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
  font-weight: 500;
}

.discord-markdown :deep(.copy-code-btn:hover) {
  background: rgba(9, 105, 218, 0.1);
  color: #0969da;
  border-color: #0969da;
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(9, 105, 218, 0.2);
}

.discord-markdown :deep(.copy-code-btn.copied) {
  background: rgba(26, 127, 55, 0.1);
  color: #1a7f37;
  border-color: #1a7f37;
  box-shadow: 0 2px 8px rgba(26, 127, 55, 0.2);
}

.discord-markdown :deep(.copy-code-btn.failed) {
  background: rgba(209, 36, 47, 0.1);
  color: #d1242f;
  border-color: #d1242f;
  box-shadow: 0 2px 8px rgba(209, 36, 47, 0.2);
}

/* 代码块 */
.discord-markdown :deep(.discord-code-block) {
  margin: 0;
  padding: 18px 22px;
  background: #f4f5f6 !important;
  overflow-x: auto;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', 'Menlo', 'DejaVu Sans Mono', 'Liberation Mono', 'Courier New', monospace;
  font-size: 15px;
  line-height: 1.7;
  border: none;
  border-radius: 0;
  font-weight: 600;
  letter-spacing: 0.5px;
  font-feature-settings: 'liga' 1, 'calt' 1, 'zero' 1;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.discord-markdown :deep(.discord-code-block.shiki) {
  background: #f4f5f6 !important;
}

.discord-markdown :deep(.shiki-code) {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', 'Menlo', 'DejaVu Sans Mono', 'Liberation Mono', 'Courier New', monospace !important;
  font-size: 15px !important;
  line-height: 1.7 !important;
  font-weight: 600 !important;
  letter-spacing: 0.5px !important;
  font-feature-settings: 'liga' 1, 'calt' 1, 'zero' 1 !important;
  text-rendering: optimizeLegibility !important;
  -webkit-font-smoothing: antialiased !important;
  -moz-osx-font-smoothing: grayscale !important;
}

/* 引用块 */
.discord-markdown :deep(.discord-quote) {
  border-left: 4px solid var(--quote-border);
  background: var(--quote-bg);
  padding: 8px 16px;
  margin: 4px 0;
  border-radius: 0 4px 4px 0;
  position: relative;
}

.discord-markdown :deep(.discord-quote)::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 4px;
  background: var(--quote-border);
  border-radius: 2px;
}

/* 列表 */
.discord-markdown :deep(ul),
.discord-markdown :deep(ol) {
  margin: 4px 0;
  padding-left: 20px;
}

.discord-markdown :deep(li) {
  margin: 2px 0;
  line-height: 1.375;
}

.discord-markdown :deep(li::marker) {
  color: var(--text-secondary);
}

/* 表格 */
.discord-markdown :deep(.discord-table-container) {
  overflow-x: auto;
  margin: 8px 0;
  border-radius: 4px;
  border: 1px solid var(--border-primary);
}

.discord-markdown :deep(.discord-table) {
  width: 100%;
  border-collapse: collapse;
  background: var(--bg-primary);
}

.discord-markdown :deep(.discord-table th) {
  background: var(--bg-secondary);
  padding: 8px 12px;
  text-align: left;
  font-weight: 600;
  color: var(--text-primary);
  border-bottom: 1px solid var(--border-primary);
}

.discord-markdown :deep(.discord-table td) {
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-primary);
  color: var(--text-primary);
}

.discord-markdown :deep(.discord-table tr:last-child td) {
  border-bottom: none;
}

/* 链接 */
.discord-markdown :deep(.discord-link) {
  color: var(--text-link);
  text-decoration: none;
  cursor: pointer;
}

.discord-markdown :deep(.discord-link:hover) {
  text-decoration: underline;
}

/* Discord特殊元素 */
.discord-markdown :deep(.discord-mention) {
  background: rgba(88, 101, 242, 0.15);
  color: var(--text-brand);
  padding: 0 2px;
  border-radius: 3px;
  cursor: pointer;
  font-weight: 500;
}

.discord-markdown :deep(.discord-mention:hover) {
  background: rgba(88, 101, 242, 0.3);
}

.discord-markdown :deep(.discord-channel-link) {
  background: rgba(88, 101, 242, 0.15);
  color: var(--text-brand);
  padding: 0 2px;
  border-radius: 3px;
  cursor: pointer;
  font-weight: 500;
}

.discord-markdown :deep(.discord-channel-link:hover) {
  background: rgba(88, 101, 242, 0.3);
}

.discord-markdown :deep(.discord-spoiler) {
  background: var(--text-secondary);
  color: transparent;
  border-radius: 2px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.discord-markdown :deep(.discord-spoiler.revealed) {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.discord-markdown :deep(.discord-spoiler:hover) {
  background: var(--text-muted);
}

/* 分隔线 */
.discord-markdown :deep(hr) {
  border: none;
  height: 1px;
  background: var(--border-primary);
  margin: 16px 0;
}

/* 任务列表 */
.discord-markdown :deep(input[type="checkbox"]) {
  margin-right: 8px;
  accent-color: var(--discord-primary);
}

/* Shiki 语法高亮增强 */
.discord-markdown :deep(.shiki) {
  background: #f4f5f6 !important;
  color: #24292f !important;
}

.discord-markdown :deep(.shiki span) {
  font-family: inherit !important;
  font-size: inherit !important;
  line-height: inherit !important;
  font-weight: inherit !important;
  letter-spacing: inherit !important;
}

/* 确保Shiki的语法高亮色彩正确显示 */
.discord-markdown :deep(.shiki .token) {
  color: inherit !important;
}

/* 强制Shiki主题色彩生效 - 移除任何颜色覆盖 */
.discord-markdown :deep(.shiki [style*="color"]) {
  color: unset !important;
}

/* 允许Shiki的内联样式生效 */
.discord-markdown :deep(.shiki span[style]) {
  color: unset !important;
}

/* 确保语法高亮元素使用其内联样式 */
.discord-markdown :deep(.shiki .token),
.discord-markdown :deep(.shiki span) {
  color: revert !important;
}

/* 自定义滚动条 */
.discord-markdown :deep(.discord-code-block) {
  scrollbar-width: thin;
  scrollbar-color: #ced4da #f4f5f6;
}

.discord-markdown :deep(.discord-code-block::-webkit-scrollbar) {
  width: 8px;
  height: 8px;
}

.discord-markdown :deep(.discord-code-block::-webkit-scrollbar-track) {
  background: #f4f5f6;
  border-radius: 4px;
}

.discord-markdown :deep(.discord-code-block::-webkit-scrollbar-thumb) {
  background: #ced4da;
  border-radius: 4px;
}

.discord-markdown :deep(.discord-code-block::-webkit-scrollbar-thumb:hover) {
  background: #adb5bd;
}

/* 代码行高亮 */
.discord-markdown :deep(.line) {
  display: inline-block;
  width: 100%;
}

.discord-markdown :deep(.line:hover) {
  background: rgba(9, 105, 218, 0.05);
}

/* 确保代码块的舒适间距 */
.discord-markdown :deep(.code-container + .code-container) {
  margin-top: 16px;
}

.discord-markdown :deep(p + .code-container) {
  margin-top: 12px;
}

.discord-markdown :deep(.code-container + p) {
  margin-top: 12px;
}

/* 代码块选中效果 */
.discord-markdown :deep(.discord-code-block) {
  user-select: all;
}

.discord-markdown :deep(.discord-code-block::selection) {
  background: rgba(9, 105, 218, 0.2);
}

.discord-markdown :deep(.discord-code-block *::selection) {
  background: rgba(9, 105, 218, 0.2);
}

/* 提升代码可读性 */
.discord-markdown :deep(.shiki .line) {
  padding: 0;
  margin: 0;
}

/* 代码块焦点状态 */
.discord-markdown :deep(.code-container:hover) {
  border-color: #0969da;
  box-shadow: 0 2px 8px rgba(9, 105, 218, 0.1);
}

/* 代码高亮动画 */
.discord-markdown :deep(.shiki span) {
  transition: color 0.2s ease;
}

.discord-markdown :deep(.theme-updated) {
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0.8;
  }
  to {
    opacity: 1;
  }
}

/* 响应式 */
@media (max-width: 768px) {
  .discord-markdown :deep(.code-header) {
    flex-direction: column;
    gap: 8px;
    align-items: flex-start;
    padding: 10px 12px;
  }

  .discord-markdown :deep(.discord-table-container) {
    font-size: 0.875rem;
  }

  .discord-markdown :deep(.discord-code-block) {
    padding: 12px 16px;
    font-size: 13px;
  }

  .discord-markdown :deep(.copy-code-btn) {
    padding: 6px 10px;
    font-size: 0.7rem;
  }
}
</style>