// Shiki configuration for build-time syntax highlighting
import { getHighlighter } from 'shiki';

// Cache for highlighters to avoid re-initialization
const highlighterCache = new Map();

// Language aliases mapping
const languageAliases = {
  js: 'javascript',
  ts: 'typescript',
  jsx: 'javascript',
  tsx: 'typescript',
  yml: 'yaml',
  md: 'markdown',
  sh: 'bash',
  shell: 'bash',
  console: 'bash',
  terminal: 'bash',
  py: 'python',
  rb: 'ruby',
  rs: 'rust',
  go: 'golang',
  dockerfile: 'docker',
  makefile: 'make'
};

// Common languages to bundle
const commonLanguages = [
  'javascript',
  'typescript',
  'jsx',
  'tsx',
  'vue',
  'html',
  'css',
  'scss',
  'json',
  'yaml',
  'markdown',
  'bash',
  'python',
  'rust',
  'go',
  'java',
  'cpp',
  'c',
  'sql',
  'docker',
  'make',
  'toml',
  'ini',
  'xml',
  'diff',
  'plaintext'
];

// Theme configuration
const themes = {
  light: 'vitesse-light',
  dark: 'one-dark-pro'
};

// Create a shared highlighter instance
export async function createShikiHighlighter(options = {}) {
  const {
    langs = commonLanguages,
    theme = 'dark',
    cache = true
  } = options;

  const cacheKey = `${theme}-${langs.sort().join(',')}`;

  // Return cached instance if available
  if (cache && highlighterCache.has(cacheKey)) {
    return highlighterCache.get(cacheKey);
  }

  try {
    // Create highlighter with specified languages and theme (v0.14.x API)
    const highlighter = await getHighlighter({
      theme: themes[theme],
      langs: langs.filter(lang => commonLanguages.includes(lang))
    });

    // Cache the instance
    if (cache) {
      highlighterCache.set(cacheKey, highlighter);
    }

    return highlighter;
  } catch (error) {
    console.error('Failed to create Shiki highlighter:', error);
    throw error;
  }
}

// Get or create a highlighter instance  
export async function getShikiHighlighterInstance(theme = 'dark') {
  return createShikiHighlighter({ theme });
}

// Resolve language from alias or file extension
export function resolveLanguage(lang) {
  if (!lang) return 'plaintext';

  const normalized = lang.toLowerCase().trim();
  return languageAliases[normalized] || normalized;
}

// Parse code block metadata (e.g., ```js {1,3-5} title="example.js")
export function parseCodeBlockMeta(meta = '') {
  const metadata = {
    highlightLines: [],
    title: null,
    showLineNumbers: true,
    startLine: 1
  };

  if (!meta) return metadata;

  // Parse highlighted lines {1,3-5}
  const highlightMatch = meta.match(/\{([\d,-]+)\}/);
  if (highlightMatch) {
    metadata.highlightLines = parseLineNumbers(highlightMatch[1]);
  }

  // Parse title
  const titleMatch = meta.match(/title="([^"]+)"/);
  if (titleMatch) {
    metadata.title = titleMatch[1];
  }

  // Parse line numbers option
  if (meta.includes('no-line-numbers')) {
    metadata.showLineNumbers = false;
  }

  // Parse start line
  const startLineMatch = meta.match(/start=(\d+)/);
  if (startLineMatch) {
    metadata.startLine = parseInt(startLineMatch[1], 10);
  }

  return metadata;
}

// Parse line number ranges (e.g., "1,3-5,7" -> [1,3,4,5,7])
function parseLineNumbers(str) {
  const lines = [];
  const parts = str.split(',');

  for (const part of parts) {
    const trimmed = part.trim();
    if (trimmed.includes('-')) {
      const [start, end] = trimmed.split('-').map(n => parseInt(n.trim(), 10));
      for (let i = start; i <= end; i++) {
        lines.push(i);
      }
    } else {
      lines.push(parseInt(trimmed, 10));
    }
  }
  return [...new Set(lines)].sort((a, b) => a - b);
}

// Format highlighted code with caching
const highlightCache = new Map();

export async function highlightCode(code, lang, options = {}) {
  const {
    theme = 'dark',
    lineNumbers = true,
    highlightLines = [],
    title = null,
    startLine = 1,
    cache = true
  } = options;

  // Generate cache key
  const cacheKey = JSON.stringify({
    code,
    lang,
    theme,
    lineNumbers,
    highlightLines,
    title,
    startLine
  });

  // Return cached result if available
  if (cache && highlightCache.has(cacheKey)) {
    return highlightCache.get(cacheKey);
  }

  try {
    const highlighter = await getShikiHighlighterInstance(theme);
    const resolvedLang = resolveLanguage(lang);

    // Get highlighted HTML (v0.14.x API)
    const html = highlighter.codeToHtml(code, {
      lang: resolvedLang
    });

    // Post-process HTML to add features
    const processedHtml = postProcessHighlightedCode(html, {
      lineNumbers,
      highlightLines,
      title,
      startLine,
      lang: resolvedLang
    });

    // Cache the result
    if (cache) {
      highlightCache.set(cacheKey, processedHtml);
    }

    return processedHtml;
  } catch (error) {
    console.error('Code highlighting failed:', error);
    // Fallback to escaped HTML
    return `<pre class="shiki"><code>${escapeHtml(code)}</code></pre>`;
  }
}

// Post-process highlighted code to add additional features
function postProcessHighlightedCode(html, options) {
  const {
    lineNumbers,
    highlightLines,
    title,
    startLine,
    lang
  } = options;

  // Create a safe parser for both browser and build environments
  let doc, pre, code;

  if (typeof DOMParser !== 'undefined') {
    // Browser environment
    const parser = new DOMParser();
    doc = parser.parseFromString(html, 'text/html');
    pre = doc.querySelector('pre');
    code = pre.querySelector('code');
  } else {
    // Build environment - return enhanced HTML string
    return postProcessHighlightedCodeString(html, options);
  }

  // Add language class
  pre.classList.add(`language-${lang}`);

  // Add title if provided
  if (title) {
    const titleEl = doc.createElement('div');
    titleEl.className = 'code-title';
    titleEl.textContent = title;
    pre.insertBefore(titleEl, code);
  }

  // Process lines for line numbers and highlighting
  if (lineNumbers || highlightLines.length > 0) {
    const lines = code.innerHTML.split('\n');
    const processedLines = lines.map((line, index) => {
      const lineNum = startLine + index;
      const isHighlighted = highlightLines.includes(lineNum);

      let lineHtml = '<span class="line">';
      if (lineNumbers) {
        lineHtml += `<span class="line-number" data-line="${lineNum}">${lineNum}</span>`;
      }
      lineHtml += `<span class="line-content${isHighlighted ? ' highlighted' : ''}">${line}</span>`;
      lineHtml += '</span>';

      return lineHtml;
    });

    code.innerHTML = processedLines.join('\n');
  }

  // Add wrapper div with metadata
  const wrapper = doc.createElement('div');
  wrapper.className = 'code-block-wrapper';
  wrapper.setAttribute('data-lang', lang);
  if (title) wrapper.setAttribute('data-title', title);

  // Move pre into wrapper
  pre.parentNode.insertBefore(wrapper, pre);
  wrapper.appendChild(pre);

  return wrapper.outerHTML;
}

// Post-process highlighted code string (for build environment)
function postProcessHighlightedCodeString(html, options) {
  const { lineNumbers, highlightLines, title, startLine, lang } = options;

  // Extract code content from pre/code tags
  const codeMatch = html.match(/<pre[^>]*><code[^>]*>([\s\S]*?)<\/code><\/pre>/);
  if (!codeMatch) return html;

  const codeContent = codeMatch[1];
  const lines = codeContent.split('\n');

  let processedCode = '';
  if (lineNumbers || highlightLines.length > 0) {
    processedCode = lines.map((line, index) => {
      const lineNum = startLine + index;
      const isHighlighted = highlightLines.includes(lineNum);

      let lineHtml = '<span class="line">';
      if (lineNumbers) {
        lineHtml += `<span class="line-number" data-line="${lineNum}">${lineNum}</span>`;
      }
      lineHtml += `<span class="line-content${isHighlighted ? ' highlighted' : ''}">${line}</span>`;
      lineHtml += '</span>';

      return lineHtml;
    }).join('\n');
  } else {
    processedCode = codeContent;
  }

  // Build wrapper
  let result = `<div class="code-block-wrapper" data-lang="${lang}"`;
  if (title) result += ` data-title="${escapeHtmlString(title)}"`;
  result += '>';

  if (title) {
    result += `<div class="code-title">${escapeHtmlString(title)}</div>`;
  }

  result += `<pre class="shiki language-${lang}"><code>${processedCode}</code></pre>`;
  result += '</div>';

  return result;
}

// Escape HTML entities (safe for both environments)
function escapeHtml(str) {
  if (typeof document !== 'undefined') {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
  }
  return escapeHtmlString(str);
}

// String-based HTML escaping
function escapeHtmlString(str) {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

// Clear caches
export function clearHighlightCache() {
  highlightCache.clear();
}

export function clearHighlighterCache() {
  highlighterCache.clear();
}

export function clearAllCaches() {
  clearHighlightCache();
  clearHighlighterCache();
}

// Export theme names for external use
export { themes };