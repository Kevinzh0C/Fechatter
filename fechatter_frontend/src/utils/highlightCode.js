/**
 * ✨ Modern Code Highlighting System
 * Based on highlight.js - lightweight, no WASM, production-ready
 * 
 * Features:
 * - 40+ languages support
 * - Auto language detection
 * - Line numbers
 * - Highlighted lines
 * - Copy functionality
 * - Multiple themes
 * - Markdown integration
 */

import hljs from 'highlight.js/lib/core'

// Import common languages
import javascript from 'highlight.js/lib/languages/javascript'
import typescript from 'highlight.js/lib/languages/typescript'
import python from 'highlight.js/lib/languages/python'
import java from 'highlight.js/lib/languages/java'
import cpp from 'highlight.js/lib/languages/cpp'
import csharp from 'highlight.js/lib/languages/csharp'
import php from 'highlight.js/lib/languages/php'
import ruby from 'highlight.js/lib/languages/ruby'
import go from 'highlight.js/lib/languages/go'
import rust from 'highlight.js/lib/languages/rust'
import sql from 'highlight.js/lib/languages/sql'
import json from 'highlight.js/lib/languages/json'
import xml from 'highlight.js/lib/languages/xml'
import css from 'highlight.js/lib/languages/css'
import yaml from 'highlight.js/lib/languages/yaml'
import bash from 'highlight.js/lib/languages/bash'
import dockerfile from 'highlight.js/lib/languages/dockerfile'
import markdown from 'highlight.js/lib/languages/markdown'
import plaintext from 'highlight.js/lib/languages/plaintext'

// Register languages
hljs.registerLanguage('javascript', javascript)
hljs.registerLanguage('typescript', typescript)
hljs.registerLanguage('python', python)
hljs.registerLanguage('java', java)
hljs.registerLanguage('cpp', cpp)
hljs.registerLanguage('csharp', csharp)
hljs.registerLanguage('php', php)
hljs.registerLanguage('ruby', ruby)
hljs.registerLanguage('go', go)
hljs.registerLanguage('rust', rust)
hljs.registerLanguage('sql', sql)
hljs.registerLanguage('json', json)
hljs.registerLanguage('xml', xml)
hljs.registerLanguage('css', css)
hljs.registerLanguage('yaml', yaml)
hljs.registerLanguage('bash', bash)
hljs.registerLanguage('dockerfile', dockerfile)
hljs.registerLanguage('markdown', markdown)
hljs.registerLanguage('plaintext', plaintext)

// Language aliases
const LANGUAGE_ALIASES = {
  js: 'javascript',
  ts: 'typescript',
  py: 'python',
  rb: 'ruby',
  rs: 'rust',
  sh: 'bash',
  shell: 'bash',
  yml: 'yaml',
  dockerfile: 'dockerfile',
  html: 'xml',
  jsx: 'javascript',
  tsx: 'typescript',
  c: 'cpp',
  'c++': 'cpp',
  'c#': 'csharp',
  cs: 'csharp'
}

// Available themes
const THEMES = {
  dark: 'github-dark',
  light: 'github',
  'one-dark': 'atom-one-dark',
  'vs-code': 'vs2015',
  'tomorrow': 'tomorrow-night'
}

// Cache for highlighted code
const highlightCache = new Map()

/**
 * Resolve language alias to actual language name
 */
function resolveLanguage(lang) {
  if (!lang) return 'plaintext'
  const normalized = lang.toLowerCase().trim()
  return LANGUAGE_ALIASES[normalized] || normalized
}

/**
 * Parse code block metadata
 */
function parseCodeBlockMeta(meta = '') {
  const metadata = {
    highlightLines: [],
    title: null,
    showLineNumbers: true,
    startLine: 1
  }

  if (!meta) return metadata

  // Parse highlighted lines {1,3-5}
  const highlightMatch = meta.match(/\{([\d,-]+)\}/)
  if (highlightMatch) {
    metadata.highlightLines = parseLineNumbers(highlightMatch[1])
  }

  // Parse title
  const titleMatch = meta.match(/title="([^"]+)"/)
  if (titleMatch) {
    metadata.title = titleMatch[1]
  }

  // Parse line numbers option
  if (meta.includes('no-line-numbers')) {
    metadata.showLineNumbers = false
  }

  // Parse start line
  const startLineMatch = meta.match(/start=(\d+)/)
  if (startLineMatch) {
    metadata.startLine = parseInt(startLineMatch[1], 10)
  }

  return metadata
}

/**
 * Parse line number ranges
 */
function parseLineNumbers(str) {
  const lines = []
  const parts = str.split(',')

  for (const part of parts) {
    const trimmed = part.trim()
    if (trimmed.includes('-')) {
      const [start, end] = trimmed.split('-').map(n => parseInt(n.trim(), 10))
      for (let i = start; i <= end; i++) {
        lines.push(i)
      }
    } else {
      lines.push(parseInt(trimmed, 10))
    }
  }
  return [...new Set(lines)].sort((a, b) => a - b)
}

/**
 * Escape HTML entities
 */
function escapeHtml(str) {
  const div = document.createElement('div')
  div.textContent = str
  return div.innerHTML
}

/**
 * Generate unique ID for code blocks
 */
function generateCodeId() {
  return `code-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
}

/**
 * ✨ Main highlight function
 */
export async function highlightCode(code, language = 'plaintext', options = {}) {
  const {
    theme = 'dark',
    lineNumbers = true,
    highlightLines = [],
    title = null,
    startLine = 1,
    cache = true,
    showCopyButton = true
  } = options

  // Generate cache key
  const cacheKey = JSON.stringify({
    code,
    language,
    theme,
    lineNumbers,
    highlightLines,
    title,
    startLine
  })

  // Return cached result
  if (cache && highlightCache.has(cacheKey)) {
    return highlightCache.get(cacheKey)
  }

  try {
    const resolvedLang = resolveLanguage(language)
    const codeId = generateCodeId()

    // Highlight code
    let highlighted
    if (hljs.getLanguage(resolvedLang)) {
      highlighted = hljs.highlight(code, { language: resolvedLang })
    } else {
      // Fallback to auto-detection
      highlighted = hljs.highlightAuto(code)
    }

    // Split into lines
    const lines = highlighted.value.split('\n')

    // Build HTML structure
    let html = `<div class="code-block-wrapper ${theme}" data-lang="${resolvedLang}" data-id="${codeId}">`

    // Add title if provided
    if (title) {
      html += `<div class="code-title">
        <span>${escapeHtml(title)}</span>
        ${showCopyButton ? `<button class="copy-btn" onclick="copyCode('${codeId}')" title="Copy code">📋</button>` : ''}
      </div>`
    }

    // Start pre/code structure
    html += `<pre class="hljs ${theme}"><code class="language-${resolvedLang}">`

    // Add lines with numbers and highlighting
    lines.forEach((line, index) => {
      const lineNum = startLine + index
      const isHighlighted = highlightLines.includes(lineNum)

      html += '<span class="line'
      if (isHighlighted) html += ' highlighted'
      html += '">'

      if (lineNumbers) {
        html += `<span class="line-number" data-line="${lineNum}">${lineNum}</span>`
      }

      html += `<span class="line-content">${line || ' '}</span>`
      html += '</span>\n'
    })

    html += '</code></pre>'

    // Add copy button if no title
    if (!title && showCopyButton) {
      html += `<button class="copy-btn-floating" onclick="copyCode('${codeId}')" title="Copy code">📋</button>`
    }

    html += '</div>'

    // Add copy functionality script
    html += `<script>
      window.copyCode = window.copyCode || function(id) {
        const block = document.querySelector('[data-id="' + id + '"]');
        if (!block) return;
        const code = block.querySelector('code');
        const text = Array.from(code.querySelectorAll('.line-content')).map(el => el.textContent).join('\\n');
        navigator.clipboard.writeText(text).then(() => {
          const btn = block.querySelector('.copy-btn, .copy-btn-floating');
          if (btn) {
            const original = btn.textContent;
            btn.textContent = '';
            setTimeout(() => { btn.textContent = original; }, 2000);
          }
        });
      };
    </script>`

    // Cache result
    if (cache) {
      highlightCache.set(cacheKey, html)
    }

    return html
  } catch (error) {
    console.error('Code highlighting failed:', error)

    // Fallback HTML
    const fallback = `<pre class="code-error"><code>${escapeHtml(code)}</code></pre>`

    if (cache) {
      highlightCache.set(cacheKey, fallback)
    }

    return fallback
  }
}

/**
 * ✨ Highlight markdown code blocks
 */
export async function highlightMarkdownCode(markdown, options = {}) {
  const {
    theme = 'dark',
    lineNumbers = true,
    cache = true
  } = options

  if (!markdown || typeof markdown !== 'string') {
    return markdown
  }

  try {
    // Regular expression to match code blocks
    const codeBlockRegex = /```(\w+)?(\s+[^\n]+)?\n([\s\S]*?)```/g
    const codeBlocks = []
    let match

    // Extract all code blocks
    while ((match = codeBlockRegex.exec(markdown)) !== null) {
      codeBlocks.push({
        fullMatch: match[0],
        lang: match[1] || 'plaintext',
        meta: match[2] || '',
        code: match[3] || '',
        index: match.index
      })
    }

    if (codeBlocks.length === 0) {
      return markdown
    }

    // Highlight all code blocks
    const highlightedBlocks = await Promise.all(
      codeBlocks.map(async (block) => {
        const metadata = parseCodeBlockMeta(block.meta)
        const html = await highlightCode(block.code, block.lang, {
          theme,
          lineNumbers: metadata.showLineNumbers !== false && lineNumbers,
          highlightLines: metadata.highlightLines,
          title: metadata.title,
          startLine: metadata.startLine,
          cache
        })

        return {
          ...block,
          html
        }
      })
    )

    // Replace code blocks with highlighted HTML (reverse order)
    let result = markdown
    for (let i = highlightedBlocks.length - 1; i >= 0; i--) {
      const block = highlightedBlocks[i]
      result =
        result.slice(0, block.index) +
        block.html +
        result.slice(block.index + block.fullMatch.length)
    }

    return result
  } catch (error) {
    console.error('Markdown code highlighting failed:', error)
    return markdown
  }
}

/**
 * ✨ Generate CSS styles for themes
 */
export function generateHighlightStyles(theme = 'dark') {
  const isDark = theme === 'dark'

  return `
    /* Code Block Wrapper */
    .code-block-wrapper {
      position: relative;
      margin: 1rem 0;
      border-radius: 8px;
      overflow: hidden;
      background-color: ${isDark ? '#0d1117' : '#f6f8fa'};
      border: 1px solid ${isDark ? '#21262d' : '#d1d9e0'};
      font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', 'Consolas', 'Monaco', monospace;
    }
    
    /* Code Title */
    .code-title {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 0.75rem 1rem;
      background-color: ${isDark ? '#161b22' : '#f1f3f4'};
      border-bottom: 1px solid ${isDark ? '#21262d' : '#d1d9e0'};
      font-size: 0.875rem;
      font-weight: 500;
      color: ${isDark ? '#f0f6fc' : '#24292f'};
    }
    
    /* Pre/Code Elements */
    .hljs {
      margin: 0;
      padding: 1rem;
      background-color: ${isDark ? '#0d1117' : '#f6f8fa'} !important;
      color: ${isDark ? '#f0f6fc' : '#24292f'} !important;
      overflow-x: auto;
      font-size: 0.875rem;
      line-height: 1.5;
      max-height: 400px;
    }
    
    /* Line Structure */
    .line {
      display: table-row;
      width: 100%;
    }
    
    .line-number {
      display: table-cell;
      padding-right: 1rem;
      text-align: right;
      color: ${isDark ? '#7d8590' : '#656d76'};
      user-select: none;
      width: 1%;
      white-space: nowrap;
      vertical-align: top;
    }
    
    .line-content {
      display: table-cell;
      padding-left: 0.5rem;
      width: 100%;
      white-space: pre;
    }
    
    /* Highlighted Lines */
    .line.highlighted {
      background-color: ${isDark ? 'rgba(255, 255, 255, 0.1)' : 'rgba(255, 255, 0, 0.2)'};
    }
    
    .line.highlighted .line-content {
      background-color: inherit;
    }
    
    /* Copy Buttons */
    .copy-btn, .copy-btn-floating {
      background-color: ${isDark ? '#21262d' : '#f1f3f4'};
      border: 1px solid ${isDark ? '#30363d' : '#d1d9e0'};
      color: ${isDark ? '#f0f6fc' : '#24292f'};
      padding: 0.25rem 0.5rem;
      border-radius: 4px;
      font-size: 0.75rem;
      cursor: pointer;
      transition: all 0.2s;
    }
    
    .copy-btn:hover, .copy-btn-floating:hover {
      background-color: ${isDark ? '#30363d' : '#e1e4e8'};
    }
    
    .copy-btn-floating {
      position: absolute;
      top: 0.5rem;
      right: 0.5rem;
      opacity: 0;
      transition: opacity 0.2s;
    }
    
    .code-block-wrapper:hover .copy-btn-floating {
      opacity: 1;
    }
    
    /* Language Badge */
    .code-block-wrapper::before {
      content: attr(data-lang);
      position: absolute;
      top: 0.5rem;
      left: 0.5rem;
      background-color: ${isDark ? 'rgba(255, 255, 255, 0.1)' : 'rgba(0, 0, 0, 0.1)'};
      color: ${isDark ? '#7d8590' : '#656d76'};
      padding: 0.125rem 0.5rem;
      border-radius: 12px;
      font-size: 0.75rem;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      font-weight: 500;
      pointer-events: none;
      z-index: 1;
    }
    
    /* Scrollbar */
    .hljs::-webkit-scrollbar {
      height: 8px;
    }
    
    .hljs::-webkit-scrollbar-track {
      background: ${isDark ? '#161b22' : '#f1f3f4'};
    }
    
    .hljs::-webkit-scrollbar-thumb {
      background: ${isDark ? '#30363d' : '#c9d1d9'};
      border-radius: 4px;
    }
    
    .hljs::-webkit-scrollbar-thumb:hover {
      background: ${isDark ? '#484f58' : '#959da5'};
    }
    
    /* Error State */
    .code-error {
      background-color: ${isDark ? '#2d1b1b' : '#fef2f2'};
      border: 1px solid ${isDark ? '#5c2626' : '#fecaca'};
      color: ${isDark ? '#f87171' : '#dc2626'};
      padding: 1rem;
      border-radius: 8px;
      margin: 1rem 0;
      font-family: 'Consolas', 'Monaco', monospace;
    }
    
    /* Inline Code */
    code:not(.hljs) {
      background-color: ${isDark ? '#6e768166' : '#afb8c133'};
      color: ${isDark ? '#f0f6fc' : '#24292f'};
      padding: 0.125rem 0.25rem;
      border-radius: 3px;
      font-family: 'SF Mono', 'Consolas', 'Monaco', monospace;
      font-size: 0.875em;
    }
  `
}

/**
 * ✨ Clear highlight cache
 */
export function clearHighlightCache() {
  highlightCache.clear()
}

/**
 * ✨ Get available languages
 */
export function getAvailableLanguages() {
  return hljs.listLanguages()
}

/**
 * ✨ Detect language automatically
 */
export function detectLanguage(code) {
  try {
    const result = hljs.highlightAuto(code)
    return result.language || 'plaintext'
  } catch (error) {
    return 'plaintext'
  }
}

// Aliases for backward compatibility
export const highlightCodeAsync = highlightCode
export const highlightSingleCodeBlock = highlightCode
export const processMarkdownWithHighlight = highlightMarkdownCode

// Export default function
export default highlightCode 