// Code highlighting utilities using Shiki
import { 
  highlightCode, 
  parseCodeBlockMeta, 
  resolveLanguage,
  themes,
  clearHighlightCache 
} from '../plugins/shiki.js';

// Regular expression to match code blocks
const CODE_BLOCK_REGEX = /```(\w+)?(\s+[^\n]+)?\n([\s\S]*?)```/g;
const INLINE_CODE_REGEX = /`([^`]+)`/g;

// Highlight all code blocks in markdown content
export async function highlightMarkdownCode(markdown, options = {}) {
  const {
    theme = 'dark',
    lineNumbers = true,
    cache = true
  } = options;

  // Process code blocks
  const codeBlocks = [];
  let match;

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

  // Highlight all code blocks in parallel
  const highlightedBlocks = await Promise.all(
    codeBlocks.map(async (block) => {
      const metadata = parseCodeBlockMeta(block.meta);
      const html = await highlightCode(block.code, block.lang, {
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

  // Replace code blocks with highlighted HTML
  let result = markdown;
  for (let i = highlightedBlocks.length - 1; i >= 0; i--) {
    const block = highlightedBlocks[i];
    result = 
      result.slice(0, block.index) + 
      block.html + 
      result.slice(block.index + block.fullMatch.length);
  }

  return result;
}

// Highlight a single code block
export async function highlightSingleCodeBlock(code, lang, meta = '', options = {}) {
  const metadata = parseCodeBlockMeta(meta);
  const resolvedLang = resolveLanguage(lang);
  
  return highlightCode(code, resolvedLang, {
    ...options,
    ...metadata
  });
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
    }

    /* Code title */
    .code-title {
      padding: 0.5rem 1rem;
      font-size: 0.875rem;
      font-weight: 500;
      color: ${isDark ? '#abb2bf' : '#666'};
      background-color: ${isDark ? '#21252b' : '#f0f0f0'};
      border-bottom: 1px solid ${isDark ? '#3e4451' : '#e0e0e0'};
    }

    /* Pre and code elements */
    .shiki {
      margin: 0;
      padding: 1rem;
      overflow-x: auto;
      font-size: 0.875rem;
      line-height: 1.5;
    }

    .shiki code {
      display: block;
      width: fit-content;
      min-width: 100%;
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
    .code-block-wrapper::before {
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
    }

    /* Copy button (if added later) */
    .code-copy-button {
      position: absolute;
      top: 0.5rem;
      right: 0.5rem;
      padding: 0.25rem 0.5rem;
      font-size: 0.75rem;
      color: ${isDark ? '#abb2bf' : '#666'};
      background-color: ${isDark ? '#21252b' : '#f0f0f0'};
      border: 1px solid ${isDark ? '#3e4451' : '#e0e0e0'};
      border-radius: 0.25rem;
      cursor: pointer;
      transition: all 0.2s;
    }

    .code-copy-button:hover {
      color: ${isDark ? '#fff' : '#000'};
      background-color: ${isDark ? '#3e4451' : '#e0e0e0'};
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
      return `<code>${escapeHtml(code)}</code>`;
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

// Export for external use
export { clearHighlightCache } from '../plugins/shiki.js';