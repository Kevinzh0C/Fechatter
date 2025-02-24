// Code highlighting utilities using Shiki
import {
  parseCodeBlockMeta,
  resolveLanguage,
  themes,
  clearHighlightCache
} from '../plugins/shiki.js';

// Import async version for internal use
import { highlightCode as highlightCodeAsync } from '../plugins/shiki.js';

// Export utilities
export { clearHighlightCache, highlightCodeAsync };

// Regular expression to match code blocks
const CODE_BLOCK_REGEX = /```(\w+)?(\s+[^\n]+)?\n([\s\S]*?)```/g;
const INLINE_CODE_REGEX = /`([^`]+)`/g;

// ✨ Enhanced markdown code highlighting for Discord messages
export async function highlightMarkdownCode(markdown, options = {}) {
  const {
    theme = 'dark',
    lineNumbers = true,
    cache = true
  } = options;

  if (!markdown || typeof markdown !== 'string') {
    return markdown;
  }

  try {
    // Process code blocks
    const codeBlocks = [];
    let match;

    // Reset regex lastIndex
    CODE_BLOCK_REGEX.lastIndex = 0;

    // Extract all code blocks first
    while ((match = CODE_BLOCK_REGEX.exec(markdown)) !== null) {
      codeBlocks.push({
        fullMatch: match[0],
        lang: match[1] || 'plaintext',
        meta: match[2] || '',
        code: match[3] || '',
        index: match.index
      });
    }

    if (codeBlocks.length === 0) {
      return markdown; // No code blocks to process
    }

    // Highlight all code blocks in parallel
    const highlightedBlocks = await Promise.all(
      codeBlocks.map(async (block) => {
        const metadata = parseCodeBlockMeta(block.meta);
        const html = await highlightCodeAsync(block.code, block.lang, {
          theme,
          lineNumbers: metadata.showLineNumbers !== false && lineNumbers,
          highlightLines: metadata.highlightLines,
          title: metadata.title,
          startLine: metadata.startLine,
          cache
        });

        return {
          ...block,
          html
        };
      })
    );

    // Replace code blocks with highlighted HTML (reverse order to maintain indices)
    let result = markdown;
    for (let i = highlightedBlocks.length - 1; i >= 0; i--) {
      const block = highlightedBlocks[i];
      result =
        result.slice(0, block.index) +
        block.html +
        result.slice(block.index + block.fullMatch.length);
    }

    if (import.meta.env.DEV) {
      console.log(`✨ Highlighted ${codeBlocks.length} code blocks`);
    }

    return result;
  } catch (error) {
    console.error('💥 Markdown code highlighting failed:', error);
    return markdown; // Return original on error
  }
}

// Highlight a single code block
export async function highlightSingleCodeBlock(code, lang, meta = '', options = {}) {
  const metadata = parseCodeBlockMeta(meta);
  const resolvedLang = resolveLanguage(lang);

  return highlightCodeAsync(code, resolvedLang, {
    ...options,
    ...metadata
  });
}

// ✨ Smart code detection for automatic highlighting
export function hasCodeBlocks(content) {
  if (!content) return false;
  return CODE_BLOCK_REGEX.test(content);
}

export function hasInlineCode(content) {
  if (!content) return false;
  return INLINE_CODE_REGEX.test(content);
}

export function hasAnyCode(content) {
  return hasCodeBlocks(content) || hasInlineCode(content);
}

// Generate static CSS for highlighted code
export function generateHighlightStyles(theme = 'dark') {
  const isDark = theme === 'dark';

  return `
    /* Code block wrapper */
    .code-block-wrapper {
      position: relative;
      margin: 1rem 0;
      border-radius: 0.5rem;
      overflow: hidden;
      background-color: ${isDark ? '#282c34' : '#fafafa'};
      box-shadow: 0 2px 4px rgba(0, 0, 0, ${isDark ? '0.2' : '0.1'});
      font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', 'Monaco', monospace;
    }

    /* Code title */
    .code-title {
      padding: 0.5rem 1rem;
      font-size: 0.875rem;
      font-weight: 500;
      color: ${isDark ? '#abb2bf' : '#666'};
      background-color: ${isDark ? '#21252b' : '#f0f0f0'};
      border-bottom: 1px solid ${isDark ? '#3e4451' : '#e0e0e0'};
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    /* Pre and code elements */
    .shiki {
      margin: 0;
      padding: 1rem;
      overflow-x: auto;
      font-size: 0.875rem;
      line-height: 1.5;
      max-height: 400px;
    }

    .shiki code {
      display: block;
      width: fit-content;
      min-width: 100%;
      font-family: inherit;
    }

    /* Line styling */
    .line {
      display: table-row;
    }

    .line-number {
      display: table-cell;
      padding-right: 1rem;
      text-align: right;
      color: ${isDark ? '#5c6370' : '#999'};
      user-select: none;
      width: 1%;
      white-space: nowrap;
    }

    .line-content {
      display: table-cell;
      padding-left: 0.5rem;
    }

    /* Highlighted lines */
    .line-content.highlighted {
      background-color: ${isDark ? 'rgba(255, 255, 255, 0.1)' : 'rgba(0, 0, 0, 0.05)'};
      display: inline-block;
      width: 100%;
      margin: 0 -1rem;
      padding: 0 1rem;
    }

    /* Inline code */
    code:not(.shiki code) {
      padding: 0.125rem 0.25rem;
      font-size: 0.875em;
      color: ${isDark ? '#e06c75' : '#d14'};
      background-color: ${isDark ? 'rgba(255, 255, 255, 0.1)' : 'rgba(0, 0, 0, 0.05)'};
      border-radius: 0.25rem;
      font-family: 'Consolas', 'Monaco', 'Andale Mono', monospace;
    }

    /* Copy button */
    .code-copy-button {
      padding: 0.25rem 0.5rem;
      font-size: 0.75rem;
      color: ${isDark ? '#abb2bf' : '#666'};
      background-color: transparent;
      border: 1px solid ${isDark ? '#3e4451' : '#e0e0e0'};
      border-radius: 0.25rem;
      cursor: pointer;
      transition: all 0.2s;
    }

    .code-copy-button:hover {
      color: ${isDark ? '#fff' : '#000'};
      background-color: ${isDark ? '#3e4451' : '#e0e0e0'};
    }

    /* Scrollbar styling */
    .shiki::-webkit-scrollbar {
      height: 8px;
    }

    .shiki::-webkit-scrollbar-track {
      background: ${isDark ? '#282c34' : '#fafafa'};
    }

    .shiki::-webkit-scrollbar-thumb {
      background: ${isDark ? '#5c6370' : '#ccc'};
      border-radius: 4px;
    }

    .shiki::-webkit-scrollbar-thumb:hover {
      background: ${isDark ? '#abb2bf' : '#999'};
    }

    /* Language badge */
    .code-block-wrapper::after {
      content: attr(data-lang);
      position: absolute;
      top: 0.5rem;
      right: 0.5rem;
      padding: 0.125rem 0.5rem;
      font-size: 0.75rem;
      color: ${isDark ? '#5c6370' : '#999'};
      background-color: ${isDark ? 'rgba(255, 255, 255, 0.1)' : 'rgba(0, 0, 0, 0.05)'};
      border-radius: 0.25rem;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      pointer-events: none;
      font-family: inherit;
    }

    /* Loading states */
    .loading-code {
      background-color: ${isDark ? '#21252b' : '#f5f5f5'};
      color: ${isDark ? '#5c6370' : '#999'};
      margin: 0;
      padding: 1rem;
      border-radius: 0.5rem;
    }

    /* Error states */
    .code-error {
      background-color: ${isDark ? '#2d1b1b' : '#fef2f2'};
      border: 1px solid ${isDark ? '#5c2626' : '#fecaca'};
      color: ${isDark ? '#f87171' : '#dc2626'};
      padding: 1rem;
      border-radius: 0.5rem;
      margin: 1rem 0;
    }
  `;
}

// Markdown processor with syntax highlighting
export async function processMarkdownWithHighlight(markdown, options = {}) {
  const {
    theme = 'dark',
    lineNumbers = true,
    cache = true,
    processInlineCode = true
  } = options;

  // Highlight code blocks
  let processed = await highlightMarkdownCode(markdown, {
    theme,
    lineNumbers,
    cache
  });

  // Process inline code if enabled
  if (processInlineCode) {
    processed = processed.replace(INLINE_CODE_REGEX, (match, code) => {
      return `<code class="inline-code">${escapeHtml(code)}</code>`;
    });
  }

  return processed;
}

// Escape HTML entities
function escapeHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

// Theme switcher utility
export async function switchTheme(theme) {
  // Clear cache to force re-highlighting with new theme
  clearHighlightCache();

  // Return new styles
  return generateHighlightStyles(theme);
}

// Get available themes
export function getAvailableThemes() {
  return Object.keys(themes);
}

// Simple HTML escape for sync version
function simpleEscapeHtml(str) {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

// Synchronous version for computed properties - DEPRECATED
export function highlightCodeSync(content) {
  console.warn('⚠️ highlightCodeSync is deprecated. Use async version instead.');
  // Simple fallback for immediate rendering
  return content.replace(/```(\w+)?\n([\s\S]*?)```/g, (match, lang, code) => {
    return `<pre class="loading-code"><code class="language-${lang || 'text'}">${simpleEscapeHtml(code)}</code></pre>`;
  });
}

// Also export as highlightCode for compatibility - DEPRECATED
export function highlightCode(content) {
  console.warn('⚠️ highlightCode (sync) is deprecated. Use highlightCodeAsync instead.');
  return highlightCodeSync(content);
}

// ✨ Enhanced async code highlighting for messages
export async function highlightMessageContent(content, options = {}) {
  if (!content || !hasAnyCode(content)) {
    return content; // No code to highlight
  }

  const {
    theme = 'dark',
    lineNumbers = true,
    cache = true
  } = options;

  try {
    // Process markdown with code highlighting
    const { renderMarkdown } = await import('./markdown.js');
    let html = renderMarkdown(content);

    // Apply code highlighting to any remaining code blocks
    html = await highlightMarkdownCode(html, { theme, lineNumbers, cache });

    return html;
  } catch (error) {
    console.error('💥 Message content highlighting failed:', error);
    return content;
  }
}

// highlightCodeAsync is already available from the local import above