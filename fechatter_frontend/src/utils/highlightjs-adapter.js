/**
 * 🎨 Highlight.js Adapter - 轻量级代码高亮解决方案
 * 
 * 替代复杂的Shiki，提供：
 * - 轻量级，无WebAssembly依赖
 * - 同步渲染，不阻塞UI
 * - 支持常用编程语言
 * - 自动语言检测
 */

// 注意：需要先安装 highlight.js
// yarn add highlight.js

// 模拟highlight.js的API（如果未安装）
let hljs
try {
  // 尝试使用实际的highlight.js
  const hljsModule = await import('highlight.js/lib/core')
  hljs = hljsModule.default

  // 动态导入常用语言
  const languages = {
    javascript: () => import('highlight.js/lib/languages/javascript'),
    typescript: () => import('highlight.js/lib/languages/typescript'),
    python: () => import('highlight.js/lib/languages/python'),
    java: () => import('highlight.js/lib/languages/java'),
    cpp: () => import('highlight.js/lib/languages/cpp'),
    csharp: () => import('highlight.js/lib/languages/csharp'),
    php: () => import('highlight.js/lib/languages/php'),
    ruby: () => import('highlight.js/lib/languages/ruby'),
    go: () => import('highlight.js/lib/languages/go'),
    rust: () => import('highlight.js/lib/languages/rust'),
    sql: () => import('highlight.js/lib/languages/sql'),
    json: () => import('highlight.js/lib/languages/json'),
    xml: () => import('highlight.js/lib/languages/xml'),
    css: () => import('highlight.js/lib/languages/css'),
    yaml: () => import('highlight.js/lib/languages/yaml'),
    bash: () => import('highlight.js/lib/languages/bash'),
    diff: () => import('highlight.js/lib/languages/diff'),
    markdown: () => import('highlight.js/lib/languages/markdown')
  }

  // 注册语言
  for (const [name, loader] of Object.entries(languages)) {
    try {
      const lang = await loader()
      hljs.registerLanguage(name, lang.default)
    } catch (error) {
      console.warn(`Failed to load language ${name}:`, error)
    }
  }

} catch (error) {
  console.warn('highlight.js not available, using fallback:', error)

  // Fallback实现
  hljs = {
    highlight: (code, { language }) => ({
      value: escapeHtml(code),
      language: language
    }),
    highlightAuto: (code) => ({
      value: escapeHtml(code),
      language: 'text'
    }),
    getLanguage: () => true,
    listLanguages: () => ['javascript', 'python', 'java', 'css', 'html']
  }
}

// 语言别名映射
const languageAliases = {
  'js': 'javascript',
  'ts': 'typescript',
  'jsx': 'javascript',
  'tsx': 'typescript',
  'py': 'python',
  'rb': 'ruby',
  'rs': 'rust',
  'c++': 'cpp',
  'c#': 'csharp',
  'cs': 'csharp',
  'sh': 'bash',
  'shell': 'bash',
  'docker': 'dockerfile',
  'html': 'xml',
  'vue': 'xml',
  'md': 'markdown',
  'text': 'plaintext',
  'txt': 'plaintext'
}

// 缓存高亮结果以提升性能
const highlightCache = new Map()
const MAX_CACHE_SIZE = 500

/**
 * 高亮单个代码块 - 同步版本
 */
export function highlightCode(code, language = '', options = {}) {
  const {
    lineNumbers = false,
    maxLines = 500,
    cache = true,
    autoDetect = true
  } = options

  if (!code || !code.trim()) {
    return `<pre class="hljs"><code></code></pre>`
  }

  // 截断过长代码
  if (code.split('\n').length > maxLines) {
    const lines = code.split('\n')
    code = lines.slice(0, maxLines).join('\n') + '\n... (truncated)'
  }

  // 检查缓存
  const cacheKey = cache ? `${language}-${code.substring(0, 50)}-${lineNumbers}` : null
  if (cache && highlightCache.has(cacheKey)) {
    return highlightCache.get(cacheKey)
  }

  try {
    let result
    const normalizedLang = normalizeLang(language)

    if (normalizedLang && hljs.getLanguage && hljs.getLanguage(normalizedLang)) {
      // 指定语言高亮
      result = hljs.highlight(code, { language: normalizedLang })
    } else if (autoDetect && hljs.highlightAuto) {
      // 自动检测语言
      result = hljs.highlightAuto(code)
    } else {
      // 无高亮
      result = { value: escapeHtml(code), language: 'text' }
    }

    // 构建HTML
    let html = `<div class="code-block-wrapper" data-lang="${result.language || 'text'}">`

    if (result.language && result.language !== 'text') {
      html += `<div class="code-header">
        <span class="language-label">${result.language}</span>
        <button class="copy-button" onclick="copyCodeToClipboard(this)" title="Copy code">📋</button>
      </div>`
    }

    if (lineNumbers) {
      html += buildLineNumberedCode(result.value, result.language)
    } else {
      html += `<pre class="hljs language-${result.language || 'text'}"><code>${result.value}</code></pre>`
    }

    html += '</div>'

    // 更新缓存
    if (cache) {
      if (highlightCache.size >= MAX_CACHE_SIZE) {
        const firstKey = highlightCache.keys().next().value
        highlightCache.delete(firstKey)
      }
      highlightCache.set(cacheKey, html)
    }

    return html

  } catch (error) {
    console.warn('Code highlighting failed:', error)
    return `<pre class="hljs"><code>${escapeHtml(code)}</code></pre>`
  }
}

/**
 * 处理Markdown中的代码块
 */
export function highlightMarkdownCode(markdown, options = {}) {
  if (!markdown) return markdown

  // 匹配代码块
  const codeBlockRegex = /```(\w+)?\n?([\s\S]*?)```/g

  return markdown.replace(codeBlockRegex, (match, lang, code) => {
    return highlightCode(code.trim(), lang, options)
  })
}

/**
 * 处理内联代码
 */
export function highlightInlineCode(text) {
  if (!text) return text

  const inlineCodeRegex = /`([^`\n]+)`/g

  return text.replace(inlineCodeRegex, (match, code) => {
    return `<code class="inline-code">${escapeHtml(code)}</code>`
  })
}

/**
 * 规范化语言名称
 */
function normalizeLang(lang) {
  if (!lang) return ''
  const normalized = lang.toLowerCase().trim()
  return languageAliases[normalized] || normalized
}

/**
 * 构建带行号的代码
 */
function buildLineNumberedCode(highlightedCode, language) {
  const lines = highlightedCode.split('\n')

  let html = `<div class="code-container">
    <div class="line-numbers">`

  for (let i = 1; i <= lines.length; i++) {
    html += `<span class="line-number">${i}</span>\n`
  }

  html += `</div><pre class="hljs language-${language || 'text'}"><code>`

  lines.forEach((line, index) => {
    html += `<span class="code-line" data-line="${index + 1}">${line || ' '}</span>`
    if (index < lines.length - 1) html += '\n'
  })

  html += `</code></pre></div>`

  return html
}

/**
 * HTML转义
 */
function escapeHtml(text) {
  if (!text) return ''
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

/**
 * 复制代码到剪贴板的全局函数
 */
if (typeof window !== 'undefined') {
  window.copyCodeToClipboard = function (button) {
    const codeBlock = button.closest('.code-block-wrapper')
    if (!codeBlock) return

    const code = codeBlock.querySelector('code')?.textContent
    if (!code) return

    navigator.clipboard.writeText(code).then(() => {
      const originalText = button.textContent
      button.textContent = '✅'
      button.style.color = '#22c55e'

      setTimeout(() => {
        button.textContent = originalText
        button.style.color = ''
      }, 2000)
    }).catch(err => {
      console.error('Failed to copy code:', err)
      // Fallback for older browsers
      const textArea = document.createElement('textarea')
      textArea.value = code
      document.body.appendChild(textArea)
      textArea.select()
      document.execCommand('copy')
      document.body.removeChild(textArea)

      button.textContent = '✅'
      setTimeout(() => {
        button.textContent = '📋'
      }, 2000)
    })
  }
}

/**
 * 清理缓存
 */
export function clearHighlightCache() {
  highlightCache.clear()
}

/**
 * 获取支持的语言列表
 */
export function getSupportedLanguages() {
  return hljs.listLanguages ? hljs.listLanguages() : Object.keys(languageAliases)
}

// 默认导出
export default {
  highlightCode,
  highlightMarkdownCode,
  highlightInlineCode,
  clearHighlightCache,
  getSupportedLanguages
}
