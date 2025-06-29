/**
 * Enhanced Code Highlighting Service with Light Theme
 * Migrated from Shiki to highlight.js for better performance and browser compatibility
 */

import hljs from 'highlight.js'
// Import highlight.js CSS theme (backup for CSS loading)
import 'highlight.js/styles/github.css'

// 支持的编程语言配置
const SUPPORTED_LANGUAGES = {
  javascript: { name: 'JavaScript', aliases: ['js'] },
  typescript: { name: 'TypeScript', aliases: ['ts'] },
  python: { name: 'Python', aliases: ['py'] },
  java: { name: 'Java', aliases: [] },
  cpp: { name: 'C++', aliases: ['c++', 'cxx'] },
  c: { name: 'C', aliases: [] },
  csharp: { name: 'C#', aliases: ['cs'] },
  php: { name: 'PHP', aliases: [] },
  ruby: { name: 'Ruby', aliases: ['rb'] },
  go: { name: 'Go', aliases: ['golang'] },
  rust: { name: 'Rust', aliases: ['rs'] },
  swift: { name: 'Swift', aliases: [] },
  kotlin: { name: 'Kotlin', aliases: ['kt'] },
  scala: { name: 'Scala', aliases: [] },
  shell: { name: 'Shell', aliases: ['bash', 'sh', 'zsh'] },
  sql: { name: 'SQL', aliases: [] },
  json: { name: 'JSON', aliases: [] },
  xml: { name: 'XML', aliases: [] },
  html: { name: 'HTML', aliases: [] },
  css: { name: 'CSS', aliases: [] },
  yaml: { name: 'YAML', aliases: ['yml'] },
  markdown: { name: 'Markdown', aliases: ['md'] },
  dockerfile: { name: 'Dockerfile', aliases: [] },
  nginx: { name: 'Nginx', aliases: [] },
  apache: { name: 'Apache', aliases: [] }
}

// 缓存已高亮的代码
const highlightCache = new Map()

/**
 * 获取语言的标准化名称
 */
function normalizeLanguage(lang) {
  if (!lang) return 'plaintext'

  const normalized = lang.toLowerCase()

  // 直接匹配
  if (SUPPORTED_LANGUAGES[normalized]) {
    return normalized
  }

  // 别名匹配
  for (const [key, config] of Object.entries(SUPPORTED_LANGUAGES)) {
    if (config.aliases.includes(normalized)) {
      return key
    }
  }

  return 'plaintext'
}

/**
 * 安全的缓存键生成函数 - 支持Unicode字符
 */
function getCacheKey(code, language, options = {}) {
  try {
    const key = JSON.stringify({ code, language, options });

    // 方案1: 使用 encodeURIComponent + btoa (安全但较长)
    // return btoa(encodeURIComponent(key)).replace(/[+/=]/g, '');

    // 方案2: 使用简单哈希算法 (更快，更可靠)
    return hashString(key);
  } catch (error) {
    // 如果所有方法都失败，使用时间戳作为fallback
    console.warn('Cache key generation failed, using timestamp fallback:', error);
    return `fallback_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}

/**
 * 简单字符串哈希函数 - 支持所有Unicode字符
 */
function hashString(str) {
  let hash = 0;
  if (str.length === 0) return hash.toString(36);

  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash; // 转换为32位整数
  }

  // 转换为正数并使用36进制表示（更短）
  return Math.abs(hash).toString(36);
}

/**
 * 创建增强的代码块包装器 - 精美容器 + 彩色高亮
 */
function createCodeWrapper(highlightedCode, language, options = {}) {
  const languageConfig = SUPPORTED_LANGUAGES[language];
  const displayName = languageConfig?.name || language.toUpperCase();

  const showHeader = options.showHeader !== false;
  const showCopy = options.showCopy !== false;
  const showLineNumbers = options.showLineNumbers || false;

  // 语言特定的图标和颜色
  const languageInfo = getLanguageInfo(language);

  if (!showHeader) {
    return `<div class="enhanced-code-container">
      <div class="code-content-area">
        <pre class="hljs language-${language} ${showLineNumbers ? 'with-line-numbers' : ''}">${highlightedCode}</pre>
      </div>
    </div>`;
  }

  return `<div class="enhanced-code-container" data-language="${language}">
    <div class="code-header-enhanced">
      <div class="code-header-left">
        <div class="language-indicator" style="background-color: ${languageInfo.color};">
          <span class="language-icon">${languageInfo.icon}</span>
          <span class="language-name">${displayName}</span>
        </div>
        <div class="code-meta">
          <span class="lines-count">${getLineCount(highlightedCode)} lines</span>
        </div>
      </div>
      <div class="code-header-right">
        ${showCopy ? `<button class="copy-button-enhanced" onclick="copyCodeToClipboard(this)" title="Copy code">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
            <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"></path>
          </svg>
          <span class="copy-text">Copy</span>
        </button>` : ''}
      </div>
    </div>
    <div class="code-content-area">
      <pre class="hljs language-${language} ${showLineNumbers ? 'with-line-numbers' : ''}">${highlightedCode}</pre>
    </div>
  </div>`;
}

/**
 * 获取编程语言的图标和颜色信息
 */
function getLanguageInfo(language) {
  const languageStyles = {
    javascript: { icon: '', color: '#f7df1e' },
    typescript: { icon: '🔷', color: '#3178c6' },
    python: { icon: '🐍', color: '#3776ab' },
    java: { icon: '☕', color: '#ed8b00' },
    cpp: { icon: '⚙️', color: '#00599c' },
    c: { icon: '', color: '#a8b9cc' },
    csharp: { icon: '💜', color: '#239120' },
    php: { icon: '🐘', color: '#777bb4' },
    ruby: { icon: '💎', color: '#cc342d' },
    go: { icon: '', color: '#00add8' },
    rust: { icon: '🦀', color: '#dea584' },
    swift: { icon: '🦉', color: '#fa7343' },
    kotlin: { icon: '', color: '#7f52ff' },
    scala: { icon: '🔺', color: '#dc322f' },
    shell: { icon: '📟', color: '#89e051' },
    sql: { icon: '🗄️', color: '#336791' },
    json: { icon: '📋', color: '#292929' },
    xml: { icon: '📄', color: '#0060ac' },
    html: { icon: '🌐', color: '#e34f26' },
    css: { icon: '', color: '#1572b6' },
    yaml: { icon: '', color: '#cb171e' },
    markdown: { icon: '📖', color: '#083fa1' },
    dockerfile: { icon: '🐳', color: '#384d54' },
    nginx: { icon: '', color: '#009639' },
    apache: { icon: '🪶', color: '#d22128' },
    plaintext: { icon: '', color: '#6b7280' }
  };

  return languageStyles[language] || languageStyles.plaintext;
}

/**
 * 计算代码行数
 */
function getLineCount(code) {
  return (code.match(/\n/g) || []).length + 1;
}

/**
 * 添加行号支持
 */
function addLineNumbers(code) {
  const lines = code.split('\n')
  return lines.map((line, index) => {
    const lineNumber = index + 1
    return `<span class="line-number">${lineNumber}</span><span class="line-content">${line}</span>`
  }).join('\n')
}

/**
 * 主要的代码高亮函数 - 支持Unicode字符
 */
export async function highlightCodeAsync(code, language = 'plaintext', options = {}) {
  try {
    if (!code || typeof code !== 'string') {
      throw new Error('Invalid code input');
    }

    const normalizedLang = normalizeLanguage(language);

    // 安全生成缓存键 - 现在支持Unicode字符
    let cacheKey;
    try {
      cacheKey = getCacheKey(code, normalizedLang, options);
    } catch (cacheKeyError) {
      console.warn('Failed to generate cache key, proceeding without cache:', cacheKeyError);
      cacheKey = null;
    }

    // 检查缓存（如果缓存键生成成功）
    if (cacheKey && highlightCache.has(cacheKey)) {
      return highlightCache.get(cacheKey);
    }

    let highlightedCode;

    if (normalizedLang === 'plaintext') {
      // 纯文本，安全转义HTML字符（包括Unicode）
      highlightedCode = code
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');
    } else {
      try {
        // 使用highlight.js进行语法高亮
        const result = hljs.highlight(code, { language: normalizedLang });
        highlightedCode = result.value;
      } catch (hlError) {
        console.warn(`Highlight.js failed for language ${normalizedLang}:`, hlError);
        // Fallback到纯文本处理
        highlightedCode = code
          .replace(/&/g, '&amp;')
          .replace(/</g, '&lt;')
          .replace(/>/g, '&gt;')
          .replace(/"/g, '&quot;')
          .replace(/'/g, '&#39;');
      }
    }

    // 添加行号（如果需要）
    if (options.showLineNumbers) {
      try {
        highlightedCode = addLineNumbers(highlightedCode);
      } catch (lineNumberError) {
        console.warn('Failed to add line numbers:', lineNumberError);
        // 继续而不添加行号
      }
    }

    // 创建包装器
    const wrappedCode = createCodeWrapper(highlightedCode, normalizedLang, options);

    // 安全缓存结果
    if (cacheKey) {
      try {
        if (highlightCache.size > 100) {
          // 清理最老的缓存项
          const firstKey = highlightCache.keys().next().value;
          highlightCache.delete(firstKey);
        }
        highlightCache.set(cacheKey, wrappedCode);
      } catch (cacheError) {
        console.warn('Failed to cache result:', cacheError);
        // 缓存失败不影响功能
      }
    }

    return wrappedCode;

  } catch (error) {
    console.warn('Code highlighting failed:', error);

    // 错误回退：返回安全的纯文本块
    const escapedCode = (code || '')
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');

    return `<div class="code-block-wrapper">
      <div class="code-block-header">
        <span class="code-language">Text</span>
        <span style="color: #d1242f; font-size: 0.75rem;">Highlighting failed</span>
      </div>
      <pre class="hljs">${escapedCode}</pre>
    </div>`;
  }
}

/**
 * 同步版本的代码高亮函数（向后兼容）
 */
export function highlightCode(code, language = 'plaintext', options = {}) {
  try {
    if (!code || typeof code !== 'string') {
      return code
    }

    const normalizedLang = normalizeLanguage(language)

    let highlightedCode

    if (normalizedLang === 'plaintext') {
      highlightedCode = code
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;')
    } else {
      const result = hljs.highlight(code, { language: normalizedLang })
      highlightedCode = result.value
    }

    if (options.showLineNumbers) {
      highlightedCode = addLineNumbers(highlightedCode)
    }

    return createCodeWrapper(highlightedCode, normalizedLang, options)

  } catch (error) {
    console.warn('Sync code highlighting failed:', error)
    return `<pre class="hljs">${code}</pre>`
  }
}

/**
 * 高亮消息内容中的代码块
 */
export function highlightCodeInContent(content) {
  if (!content) return content

  // 匹配代码块 ```language\ncode\n```
  const codeBlockRegex = /```(\w+)?\n([\s\S]*?)```/g

  return content.replace(codeBlockRegex, (match, language, code) => {
    try {
      return highlightCode(code.trim(), language || 'plaintext', {
        showHeader: true,
        showCopy: true
      })
    } catch (error) {
      console.warn('Code block highlighting failed:', error)
      return match // 返回原始内容
    }
  })
}

/**
 * 获取支持的语言列表
 */
export function getSupportedLanguages() {
  return Object.entries(SUPPORTED_LANGUAGES).map(([key, config]) => ({
    value: key,
    label: config.name,
    aliases: config.aliases
  }))
}

/**
 * 检测代码语言
 */
export function detectLanguage(code) {
  if (!code || typeof code !== 'string') {
    return 'plaintext'
  }

  try {
    const result = hljs.highlightAuto(code)
    return result.language || 'plaintext'
  } catch (error) {
    console.warn('Language detection failed:', error)
    return 'plaintext'
  }
}

/**
 * 清理高亮缓存
 */
export function clearHighlightCache() {
  highlightCache.clear()
}

/**
 * 获取缓存统计信息
 */
export function getCacheStats() {
  return {
    size: highlightCache.size,
    maxSize: 100
  }
}

// 全局函数：复制代码到剪贴板（增强版）
window.copyCodeToClipboard = function (button) {
  const codeContainer = button.closest('.enhanced-code-container') || button.closest('.code-block-wrapper');
  const codeBlock = codeContainer.querySelector('.hljs');
  const code = codeBlock.textContent || codeBlock.innerText;

  navigator.clipboard.writeText(code).then(() => {
    const copyText = button.querySelector('.copy-text');
    const originalText = copyText ? copyText.textContent : button.textContent;

    if (copyText) {
      copyText.textContent = 'Copied!';
    } else {
      button.textContent = 'Copied!';
    }

    button.classList.add('copied');

    setTimeout(() => {
      if (copyText) {
        copyText.textContent = originalText;
      } else {
        button.textContent = originalText;
      }
      button.classList.remove('copied');
    }, 2000);
  }).catch(err => {
    console.error('Failed to copy code:', err);
    const copyText = button.querySelector('.copy-text');

    if (copyText) {
      copyText.textContent = 'Copy failed';
    } else {
      button.textContent = 'Copy failed';
    }

    setTimeout(() => {
      if (copyText) {
        copyText.textContent = 'Copy';
      } else {
        button.textContent = 'Copy';
      }
    }, 2000);
  });
}

/**
 * 高亮Markdown内容中的代码块 - 专为消息显示优化，支持Unicode字符
 */
export async function highlightMarkdownCode(markdown, options = {}) {
  if (!markdown || typeof markdown !== 'string') {
    return markdown || '';
  }

  try {
    // 预检查：确保输入是有效的字符串
    if (typeof markdown !== 'string') {
      console.warn('highlightMarkdownCode: Invalid input type, expected string');
      return String(markdown || '');
    }

    // 匹配所有代码块 ```language\ncode\n```
    const codeBlockRegex = /```(\w+)?\n?([\s\S]*?)\n?```/g;

    let result = markdown;
    let match;
    const replacements = [];
    let processedBlocks = 0;

    // 收集所有需要替换的代码块
    while ((match = codeBlockRegex.exec(markdown)) !== null) {
      const [fullMatch, language, code] = match;
      const lang = normalizeLanguage(language || 'plaintext');
      processedBlocks++;

      try {
        // 高亮代码块 - 使用安全的缓存键生成
        const highlightedBlock = await highlightCodeAsync(code.trim(), lang, {
          showHeader: options.showHeader !== false,
          showCopy: options.showCopy !== false,
          showLineNumbers: options.lineNumbers || false,
          theme: options.theme || 'light' // 默认使用浅色主题
        });

        replacements.push({
          original: fullMatch,
          replacement: highlightedBlock
        });

        if (import.meta.env.DEV) {
          console.log(`Code block ${processedBlocks} highlighted successfully (${lang})`);
        }
      } catch (blockError) {
        console.warn(`Failed to highlight code block ${processedBlocks} in language ${lang}:`, blockError);

        // 创建一个简单的代码块作为fallback
        const fallbackBlock = `
<div class="code-block-wrapper">
  <div class="code-block-header">
    <span class="code-language">${lang.toUpperCase()}</span>
    <span style="color: #d1242f; font-size: 0.75rem;">Highlighting failed</span>
  </div>
  <pre class="hljs">${code.trim()
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#39;')}</pre>
</div>`.trim();

        replacements.push({
          original: fullMatch,
          replacement: fallbackBlock
        });
      }
    }

    // 应用所有替换
    replacements.forEach(({ original, replacement }) => {
      result = result.replace(original, replacement);
    });

    // 处理内联代码 `code` - 安全地处理Unicode字符
    const inlineCodeRegex = /`([^`\n]+)`/g;
    result = result.replace(inlineCodeRegex, (match, code) => {
      try {
        // 转义HTML字符
        const escapedCode = code
          .replace(/&/g, '&amp;')
          .replace(/</g, '&lt;')
          .replace(/>/g, '&gt;')
          .replace(/"/g, '&quot;')
          .replace(/'/g, '&#39;');

        return `<code class="inline-code">${escapedCode}</code>`;
      } catch (inlineError) {
        console.warn('Failed to process inline code:', inlineError);
        return match; // 返回原始内容
      }
    });

    // 缓存结果 - 使用新的安全缓存键生成
    if (options.cache !== false) {
      try {
        const cacheKey = getCacheKey(markdown, 'markdown', options);
        if (highlightCache.size < 50) { // 限制Markdown缓存大小
          highlightCache.set(cacheKey, result);
        }
      } catch (cacheError) {
        console.warn('Failed to cache markdown result:', cacheError);
        // 缓存失败不影响功能，继续执行
      }
    }

    if (import.meta.env.DEV && processedBlocks > 0) {
      console.log(`✨ Markdown processing completed: ${processedBlocks} code blocks processed`);
    }

    return result;

  } catch (error) {
    console.error('Markdown code highlighting failed:', error);

    // 错误回退：至少处理内联代码
    try {
      const inlineCodeRegex = /`([^`\n]+)`/g;
      return markdown.replace(inlineCodeRegex, (match, code) => {
        try {
          const escapedCode = code
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#39;');

          return `<code class="inline-code">${escapedCode}</code>`;
        } catch (escapeError) {
          console.warn('Failed to escape inline code:', escapeError);
          return match;
        }
      });
    } catch (fallbackError) {
      console.error('Fallback markdown processing also failed:', fallbackError);
      return markdown; // 返回原始内容
    }
  }
}

/**
 * 处理消息内容中的Markdown和代码高亮（别名函数，向后兼容）
 */
export const processMarkdownWithHighlight = highlightMarkdownCode;

// 默认导出
export default {
  highlightCodeAsync,
  highlightCode,
  highlightCodeInContent,
  highlightMarkdownCode,
  processMarkdownWithHighlight,
  getSupportedLanguages,
  detectLanguage,
  clearHighlightCache,
  getCacheStats
}