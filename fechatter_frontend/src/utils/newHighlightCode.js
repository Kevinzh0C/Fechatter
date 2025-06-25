/**
 * ✨ Modern Code Highlighting System
 * Based on highlight.js - lightweight, no WASM, production-ready
 */

import hljs from 'highlight.js/lib/core'

// Import common languages
import javascript from 'highlight.js/lib/languages/javascript'
import typescript from 'highlight.js/lib/languages/typescript'
import python from 'highlight.js/lib/languages/python'
import java from 'highlight.js/lib/languages/java'
import cpp from 'highlight.js/lib/languages/cpp'
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
  html: 'xml',
  jsx: 'javascript',
  tsx: 'typescript',
  c: 'cpp',
  'c++': 'cpp'
}

// Cache for highlighted code
const highlightCache = new Map()

/**
 * Resolve language alias
 */
function resolveLanguage(lang) {
  if (!lang) return 'plaintext'
  const normalized = lang.toLowerCase().trim()
  return LANGUAGE_ALIASES[normalized] || normalized
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
 * ✨ Main highlight function
 */
export async function highlightCode(code, language = 'plaintext', options = {}) {
  const {
    theme = 'dark',
    lineNumbers = true,
    cache = true
  } = options

  // Generate cache key
  const cacheKey = JSON.stringify({ code, language, theme, lineNumbers })

  // Return cached result
  if (cache && highlightCache.has(cacheKey)) {
    return highlightCache.get(cacheKey)
  }

  try {
    const resolvedLang = resolveLanguage(language)

    // Highlight code
    let highlighted
    if (hljs.getLanguage(resolvedLang)) {
      highlighted = hljs.highlight(code, { language: resolvedLang })
    } else {
      highlighted = hljs.highlightAuto(code)
    }

    // Split into lines
    const lines = highlighted.value.split('\n')

    // Build HTML structure
    let html = `<div class="code-block-wrapper ${theme}" data-lang="${resolvedLang}">`
    html += `<pre class="hljs ${theme}"><code class="language-${resolvedLang}">`

    // Add lines with numbers
    lines.forEach((line, index) => {
      const lineNum = index + 1
      html += '<span class="line">'

      if (lineNumbers) {
        html += `<span class="line-number">${lineNum}</span>`
      }

      html += `<span class="line-content">${line || ' '}</span>`
      html += '</span>\n'
    })

    html += '</code></pre></div>'

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
  if (!markdown || typeof markdown !== 'string') {
    return markdown
  }

  try {
    // Regular expression to match code blocks
    const codeBlockRegex = /```(\w+)?\n([\s\S]*?)```/g
    const codeBlocks = []
    let match

    // Extract all code blocks
    while ((match = codeBlockRegex.exec(markdown)) !== null) {
      codeBlocks.push({
        fullMatch: match[0],
        lang: match[1] || 'plaintext',
        code: match[2] || '',
        index: match.index
      })
    }

    if (codeBlocks.length === 0) {
      return markdown
    }

    // Highlight all code blocks
    const highlightedBlocks = await Promise.all(
      codeBlocks.map(async (block) => {
        const html = await highlightCode(block.code, block.lang, options)
        return { ...block, html }
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
 * ✨ Generate CSS styles
 */
export function generateHighlightStyles(theme = 'dark') {
  const isDark = theme === 'dark'

  return `
    .code-block-wrapper {
      position: relative;
      margin: 1rem 0;
      border-radius: 8px;
      overflow: hidden;
      background-color: ${isDark ? '#0d1117' : '#f6f8fa'};
      border: 1px solid ${isDark ? '#21262d' : '#d1d9e0'};
      font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
    }
    
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
    
    .line {
      display: table-row;
    }
    
    .line-number {
      display: table-cell;
      padding-right: 1rem;
      text-align: right;
      color: ${isDark ? '#7d8590' : '#656d76'};
      user-select: none;
      width: 1%;
    }
    
    .line-content {
      display: table-cell;
      padding-left: 0.5rem;
      white-space: pre;
    }
    
    .code-error {
      background-color: ${isDark ? '#2d1b1b' : '#fef2f2'};
      color: ${isDark ? '#f87171' : '#dc2626'};
      padding: 1rem;
      border-radius: 8px;
      margin: 1rem 0;
    }
  `
}

// Aliases for backward compatibility
export const highlightCodeAsync = highlightCode
export const clearHighlightCache = () => highlightCache.clear()

export default highlightCode 