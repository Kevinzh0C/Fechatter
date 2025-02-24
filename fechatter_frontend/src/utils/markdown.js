/**
 * Markdown rendering utilities for message content
 * Provides safe HTML rendering of markdown-formatted text
 */

// Essential imports for markdown rendering
import { marked } from 'marked'
import DOMPurify from 'dompurify'

// 🎨 Configure marked with enhanced code handling
const renderer = {
  code(code, infostring, escaped) {
    const lang = (infostring || '').match(/\S*/)
    const language = lang ? lang[0] : 'plaintext'

    // Extract title from infostring (e.g., "javascript title='example.js'")
    const titleMatch = infostring && infostring.match(/title=['"]([^'"]+)['"]/)
    const title = titleMatch ? titleMatch[1] : null

    // Create a placeholder that will be replaced by Vue component
    const escapedCode = escaped ? code : escapeHtml(code)

    return `<div class="code-block-placeholder" 
      data-code="${escapeAttribute(escapedCode)}"
      data-language="${escapeAttribute(language)}"
      ${title ? `data-title="${escapeAttribute(title)}"` : ''}
      data-line-numbers="true">
      <pre class="loading-code"><code class="language-${language}">${escapedCode}</code></pre>
    </div>`
  },

  codespan(code) {
    return `<code class="inline-code">${escapeHtml(code)}</code>`
  }
}

// Configure marked options using v15 API
marked.use({
  renderer,
  gfm: true,
  breaks: true,
  pedantic: false,
  sanitize: false, // We'll use DOMPurify instead
  smartLists: true,
  smartypants: false,
  xhtml: false
})

// 🧼 Configure DOMPurify for enhanced security
const purifyConfig = {
  ALLOWED_TAGS: [
    'p', 'br', 'strong', 'em', 'u', 's', 'del', 'ins', 'mark',
    'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
    'blockquote', 'code', 'pre',
    'ul', 'ol', 'li',
    'a', 'img',
    'table', 'thead', 'tbody', 'tr', 'th', 'td',
    'div', 'span',
    'hr'
  ],
  ALLOWED_ATTR: [
    'href', 'title', 'alt', 'src', 'class', 'id',
    'data-code', 'data-language', 'data-title', 'data-line-numbers'
  ],
  ALLOWED_URI_REGEXP: /^(?:(?:(?:f|ht)tps?|mailto|tel|callto|cid|xmpp):|[^a-z]|[a-z+.\-]+(?:[^a-z+.\-:]|$))/i,
  ADD_TAGS: ['mark'],
  ADD_ATTR: ['target']
}

/**
 * Simple markdown renderer for chat messages
 * @param {string} content - Markdown content to render
 * @returns {string} HTML string
 */
export function renderMarkdown(content) {
  if (!content || typeof content !== 'string') {
    return ''
  }

  try {
    // First pass: Convert markdown to HTML using marked v15 API
    let html = marked.parse(content)

    // Second pass: Sanitize with DOMPurify
    html = DOMPurify.sanitize(html, purifyConfig)

    return html
  } catch (error) {
    console.error('❌ Markdown rendering failed:', error)
    // Fallback to escaped plain text
    return `<p>${escapeHtml(content)}</p>`
  }
}

/**
 * Escape HTML characters to prevent XSS
 * @param {string} text - Text to escape
 * @returns {string} Escaped text
 */
function escapeHtml(text) {
  const div = document.createElement('div')
  div.textContent = text
  return div.innerHTML
}

/**
 * Basic emoji rendering
 * @param {string} text - Text with emoji codes
 * @returns {string} Text with emoji unicode
 */
function renderEmojis(text) {
  const emojiMap = {
    ':smile:': '😄',
    ':laughing:': '😆',
    ':heart:': '❤️',
    ':thumbsup:': '👍',
    ':thumbsdown:': '👎',
    ':fire:': '🔥',
    ':rocket:': '🚀',
    ':star:': '⭐',
    ':eyes:': '👀',
    ':thinking:': '🤔',
    ':crying:': '😢',
    ':angry:': '😠',
    ':confused:': '😕',
    ':wink:': '😉',
    ':kiss:': '😘',
    ':sunglasses:': '😎',
    ':party:': '🎉',
    ':clap:': '👏',
    ':wave:': '👋',
    ':ok:': '👌'
  }

  let result = text
  for (const [code, emoji] of Object.entries(emojiMap)) {
    result = result.replace(new RegExp(escapeRegex(code), 'g'), emoji)
  }

  return result
}

/**
 * Escape special regex characters
 * @param {string} string - String to escape
 * @returns {string} Escaped string
 */
function escapeRegex(string) {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

/**
 * Parse mentions from markdown content
 * @param {string} content - Content to parse
 * @returns {Array} Array of mentioned usernames
 */
export function parseMentions(content) {
  if (!content) return []

  const mentionRegex = /@(\w+)/g
  const mentions = []
  let match

  while ((match = mentionRegex.exec(content)) !== null) {
    mentions.push(match[1])
  }

  return [...new Set(mentions)] // Remove duplicates
}

/**
 * Parse channel mentions from markdown content
 * @param {string} content - Content to parse
 * @returns {Array} Array of mentioned channel names
 */
export function parseChannelMentions(content) {
  if (!content) return []

  const channelRegex = /#(\w+)/g
  const channels = []
  let match

  while ((match = channelRegex.exec(content)) !== null) {
    channels.push(match[1])
  }

  return [...new Set(channels)] // Remove duplicates
}

/**
 * Extract plain text from markdown content
 * @param {string} content - Markdown content
 * @returns {string} Plain text without markdown formatting
 */
export function extractPlainText(content) {
  if (!content) return ''

  let text = content

  // Remove code blocks
  text = text.replace(/```[\s\S]*?```/g, '')

  // Remove inline code
  text = text.replace(/`[^`]+`/g, '')

  // Remove bold/italic/strikethrough
  text = text.replace(/[*_~]{1,2}([^*_~]+)[*_~]{1,2}/g, '$1')

  // Remove links but keep text
  text = text.replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')

  // Remove URLs
  text = text.replace(/https?:\/\/[^\s<>"']+/g, '')

  // Remove mentions and channels (keep the text)
  text = text.replace(/[@#](\w+)/g, '$1')

  // Remove emojis
  text = text.replace(/:[a-z_]+:/g, '')

  // Clean up whitespace
  text = text.replace(/\s+/g, ' ').trim()

  return text
}

/**
 * Check if content contains markdown formatting
 * @param {string} content - Content to check
 * @returns {boolean} True if content has markdown
 */
export function hasMarkdown(content) {
  if (!content) return false

  const markdownPatterns = [
    /```[\s\S]*?```/,     // Code blocks
    /`[^`]+`/,            // Inline code
    /\*\*[^*]+\*\*/,      // Bold
    /__[^_]+__/,          // Bold
    /\*[^*]+\*/,          // Italic
    /_[^_]+_/,            // Italic
    /~~[^~]+~~/,          // Strikethrough
    /\[([^\]]+)\]\([^)]+\)/, // Links
    /@\w+/,               // Mentions
    /#\w+/,               // Channels
    /:[a-z_]+:/           // Emojis
  ]

  return markdownPatterns.some(pattern => pattern.test(content))
}

/**
 * Truncate markdown content while preserving formatting
 * @param {string} content - Markdown content
 * @param {number} maxLength - Maximum length
 * @returns {string} Truncated content
 */
export function truncateMarkdown(content, maxLength = 100) {
  if (!content) return ''

  const plainText = extractPlainText(content)

  if (plainText.length <= maxLength) {
    return content
  }

  // Find truncation point that doesn't break markdown
  let truncateAt = maxLength
  let inCodeBlock = false
  let inInlineCode = false
  let formatting = []

  for (let i = 0; i < content.length && i < maxLength; i++) {
    const char = content[i]
    const next = content[i + 1]
    const prev = content[i - 1]

    // Track code blocks
    if (char === '`') {
      if (next === '`' && content[i + 2] === '`') {
        inCodeBlock = !inCodeBlock
        i += 2
      } else if (!inCodeBlock) {
        inInlineCode = !inInlineCode
      }
    }

    // Track other formatting
    if (!inCodeBlock && !inInlineCode) {
      if (char === '*' || char === '_' || char === '~') {
        formatting.push(char)
      }
    }

    // Find a good breaking point
    if (char === ' ' && i >= maxLength - 10) {
      truncateAt = i
      break
    }
  }

  let truncated = content.substring(0, truncateAt).trim()

  // Close any open formatting
  if (inCodeBlock) {
    truncated += '```'
  } else if (inInlineCode) {
    truncated += '`'
  }

  // Add ellipsis
  truncated += '...'

  return truncated
}

/**
 * Validate markdown content for safety
 * @param {string} content - Content to validate
 * @returns {Object} Validation result
 */
export function validateMarkdown(content) {
  const result = {
    valid: true,
    errors: [],
    warnings: []
  }

  if (!content) return result

  // Check for potentially dangerous content
  if (content.includes('<script')) {
    result.valid = false
    result.errors.push('Script tags are not allowed')
  }

  if (content.includes('javascript:')) {
    result.valid = false
    result.errors.push('JavaScript URLs are not allowed')
  }

  // Check for malformed markdown
  const codeBlockMatches = content.match(/```/g)
  if (codeBlockMatches && codeBlockMatches.length % 2 !== 0) {
    result.warnings.push('Unclosed code block detected')
  }

  const backtickMatches = content.match(/(?<!`)`(?!`)/g)
  if (backtickMatches && backtickMatches.length % 2 !== 0) {
    result.warnings.push('Unclosed inline code detected')
  }

  return result
}

// Export utility object for easier imports
export const markdownUtils = {
  render: renderMarkdown,
  parseMentions,
  parseChannelMentions,
  extractPlainText,
  hasMarkdown,
  truncateMarkdown,
  validateMarkdown
}

function escapeAttribute(text) {
  return text
    .replace(/&/g, '&amp;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
}

/**
 * 🎨 Post-process rendered HTML to replace code placeholders with highlighted code
 */
export async function processCodeBlocks(html) {
  if (!html || typeof html !== 'string') {
    return html
  }

  try {
    // Create a temporary DOM to process code blocks
    const parser = new DOMParser()
    const doc = parser.parseFromString(html, 'text/html')
    const placeholders = doc.querySelectorAll('.code-block-placeholder')

    if (placeholders.length === 0) {
      return html
    }

    // Import highlighting function dynamically
    const { highlightCodeAsync } = await import('./codeHighlight.js')

    // Process all code blocks in parallel
    const highlightPromises = Array.from(placeholders).map(async (placeholder) => {
      const code = placeholder.getAttribute('data-code') || ''
      const language = placeholder.getAttribute('data-language') || 'plaintext'
      const title = placeholder.getAttribute('data-title') || null
      const lineNumbers = placeholder.getAttribute('data-line-numbers') === 'true'

      try {
        const highlighted = await highlightCodeAsync(code, language, {
          theme: 'dark',
          lineNumbers,
          title,
          cache: true
        })

        // Replace placeholder with highlighted code
        const wrapper = doc.createElement('div')
        wrapper.innerHTML = highlighted
        placeholder.parentNode.replaceChild(wrapper.firstElementChild, placeholder)
      } catch (error) {
        console.warn(`Failed to highlight ${language} code block:`, error)
        // Keep the placeholder as fallback
      }
    })

    await Promise.all(highlightPromises)

    // Return processed HTML
    return doc.body.innerHTML
  } catch (error) {
    console.error('❌ Code block processing failed:', error)
    return html
  }
}

/**
 * 🚀 Complete markdown processing with code highlighting
 */
export async function renderMarkdownWithCodeHighlight(content) {
  const basicHtml = renderMarkdown(content)
  return await processCodeBlocks(basicHtml)
}

/**
 * 🔍 Extract code blocks from markdown content
 */
export function extractCodeBlocks(content) {
  if (!content) return []

  const codeBlocks = []
  const codeBlockRegex = /```(\w+)?(\s+[^\n]+)?\n([\s\S]*?)```/g
  let match

  while ((match = codeBlockRegex.exec(content)) !== null) {
    const lang = match[1] || 'plaintext'
    const meta = match[2] || ''
    const code = match[3] || ''

    // Extract title from meta
    const titleMatch = meta.match(/title=['"]([^'"]+)['"]/)
    const title = titleMatch ? titleMatch[1] : null

    codeBlocks.push({
      language: lang,
      code: code.trim(),
      title,
      raw: match[0],
      index: match.index
    })
  }

  return codeBlocks
}

/**
 * 🧹 Clean HTML for preview (strip code blocks for performance)
 */
export function renderMarkdownPreview(content, maxLength = 200) {
  if (!content) return ''

  // Remove code blocks for preview
  const withoutCodeBlocks = content.replace(/```[\s\S]*?```/g, '[Code Block]')

  // Truncate if too long
  const truncated = withoutCodeBlocks.length > maxLength
    ? withoutCodeBlocks.substring(0, maxLength) + '...'
    : withoutCodeBlocks

  return renderMarkdown(truncated)
}

// Export default function for backward compatibility
export default renderMarkdown 