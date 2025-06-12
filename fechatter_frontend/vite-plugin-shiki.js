// Vite plugin for build-time Shiki syntax highlighting
import { createHighlighter } from 'shiki';
import { parseCodeBlockMeta, resolveLanguage } from './src/plugins/shiki.js';

// Cache for build process
const buildCache = new Map();
let shikiHighlighter = null;

// Common languages for build
const buildLanguages = [
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
  'sql',
  'xml',
  'plaintext'
];

// Initialize Shiki highlighter for build
async function initializeHighlighter() {
  if (!shikiHighlighter) {
    shikiHighlighter = await createHighlighter({
      themes: ['vitesse-light', 'one-dark-pro'],
      langs: buildLanguages
    });
  }
  return shikiHighlighter;
}

// Process markdown content
async function processMarkdown(content, id, options = {}) {
  const {
    theme = 'dark',
    lineNumbers = true,
    cache = true
  } = options;

  // Check cache
  const cacheKey = `${id}:${theme}:${content}`;
  if (cache && buildCache.has(cacheKey)) {
    return buildCache.get(cacheKey);
  }

  const highlighter = await initializeHighlighter();
  const themeKey = theme === 'dark' ? 'one-dark-pro' : 'vitesse-light';

  // Process code blocks
  const codeBlockRegex = /```(\w+)?(\s+[^\n]+)?\n([\s\S]*?)```/g;
  let processed = content;
  let match;
  const replacements = [];

  while ((match = codeBlockRegex.exec(content)) !== null) {
    const lang = match[1] || 'plaintext';
    const meta = match[2] || '';
    const code = match[3] || '';
    const metadata = parseCodeBlockMeta(meta);

    try {
      // Generate highlighted HTML
      const html = highlighter.codeToHtml(code, {
        lang: resolveLanguage(lang),
        theme: themeKey
      });

      // Post-process HTML
      const processedHtml = postProcessHtml(html, {
        lang,
        lineNumbers: metadata.showLineNumbers !== false && lineNumbers,
        highlightLines: metadata.highlightLines,
        title: metadata.title,
        startLine: metadata.startLine
      });

      replacements.push({
        start: match.index,
        end: match.index + match[0].length,
        replacement: processedHtml
      });
    } catch (error) {
      console.warn(`Failed to highlight ${lang} code block:`, error);
      // Fallback to plain code block
      replacements.push({
        start: match.index,
        end: match.index + match[0].length,
        replacement: `<pre class="shiki"><code class="language-${lang}">${escapeHtml(code)}</code></pre>`
      });
    }
  }

  // Apply replacements in reverse order
  for (let i = replacements.length - 1; i >= 0; i--) {
    const { start, end, replacement } = replacements[i];
    processed = processed.slice(0, start) + replacement + processed.slice(end);
  }

  // Cache result
  if (cache) {
    buildCache.set(cacheKey, processed);
  }

  return processed;
}

// Post-process highlighted HTML
function postProcessHtml(html, options) {
  const { lang, lineNumbers, highlightLines, title, startLine } = options;

  // Create wrapper structure
  let result = '<div class="code-block-wrapper" data-lang="' + lang + '"';
  if (title) result += ' data-title="' + escapeHtml(title) + '"';
  result += '>';

  // Add title if provided
  if (title) {
    result += '<div class="code-title">' + escapeHtml(title) + '</div>';
  }

  // Parse the HTML to add line numbers and highlighting
  const parser = new DOMParser();
  const doc = parser.parseFromString(html, 'text/html');
  const pre = doc.querySelector('pre');
  const code = pre.querySelector('code');

  // Process lines
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

  result += pre.outerHTML;
  result += '</div>';

  return result;
}

// Escape HTML
function escapeHtml(str) {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

// Vite plugin factory
export default function viteShikiPlugin(options = {}) {
  const {
    theme = 'dark',
    lineNumbers = true,
    cache = true,
    include = /\.(md|markdown|vue)$/,
    exclude = /node_modules/
  } = options;

  return {
    name: 'vite-plugin-shiki',
    enforce: 'pre',

    async buildStart() {
      // Initialize highlighter at build start
      await initializeHighlighter();
    },

    async transform(code, id) {
      // Check if file should be processed
      if (!include.test(id) || exclude.test(id)) {
        return null;
      }

      let transformed = code;
      let hasChanges = false;

      // Process .md and .markdown files
      if (/\.(md|markdown)$/.test(id)) {
        transformed = await processMarkdown(code, id, { theme, lineNumbers, cache });
        hasChanges = transformed !== code;
      }

      // Process .vue files (extract and process template)
      if (/\.vue$/.test(id)) {
        const templateMatch = code.match(/<template[^>]*>([\s\S]*?)<\/template>/);
        if (templateMatch) {
          const template = templateMatch[1];
          
          // Process markdown in template (look for v-html with markdown)
          const processedTemplate = await processVueTemplate(template, id, { theme, lineNumbers, cache });
          
          if (processedTemplate !== template) {
            transformed = code.replace(templateMatch[0], `<template>${processedTemplate}</template>`);
            hasChanges = true;
          }
        }

        // Also process <script> sections that might contain markdown strings
        const scriptMatch = code.match(/<script[^>]*>([\s\S]*?)<\/script>/);
        if (scriptMatch) {
          const script = scriptMatch[1];
          const processedScript = await processScriptMarkdown(script, id, { theme, lineNumbers, cache });
          
          if (processedScript !== script) {
            transformed = transformed.replace(scriptMatch[0], `<script${scriptMatch[0].match(/<script([^>]*)>/)[1]}>${processedScript}</script>`);
            hasChanges = true;
          }
        }
      }

      return hasChanges ? {
        code: transformed,
        map: null
      } : null;
    },

    // Generate styles for SSR/SSG
    generateBundle() {
      const styles = generateStaticStyles(theme);
      
      this.emitFile({
        type: 'asset',
        fileName: 'shiki-styles.css',
        source: styles
      });
    },

    // Clear cache on build end
    buildEnd() {
      buildCache.clear();
    }
  };
}

// Process Vue template content
async function processVueTemplate(template, id, options) {
  // Look for markdown content in v-html directives
  const vHtmlRegex = /v-html="([^"]+)"/g;
  let processed = template;
  let match;

  while ((match = vHtmlRegex.exec(template)) !== null) {
    const expression = match[1];
    
    // Skip if it's a complex expression or not a markdown variable
    if (expression.includes('(') || expression.includes('[') || !expression.includes('markdown')) {
      continue;
    }

    // This is a simplified approach - in production, you'd want more sophisticated detection
    // For now, we'll just mark it for runtime processing
    processed = processed.replace(match[0], `v-html="${expression}" data-shiki-process="true"`);
  }

  return processed;
}

// Process markdown strings in script sections
async function processScriptMarkdown(script, id, options) {
  // Look for markdown template literals
  const markdownRegex = /const\s+\w+Markdown\s*=\s*`([\s\S]*?)`/g;
  let processed = script;
  let match;
  const replacements = [];

  while ((match = markdownRegex.exec(script)) !== null) {
    const markdown = match[1];
    
    // Check if it contains code blocks
    if (markdown.includes('```')) {
      const highlighted = await processMarkdown(markdown, `${id}:script:${match.index}`, options);
      
      replacements.push({
        start: match.index,
        end: match.index + match[0].length,
        replacement: match[0].replace(markdown, highlighted.replace(/`/g, '\\`'))
      });
    }
  }

  // Apply replacements
  for (let i = replacements.length - 1; i >= 0; i--) {
    const { start, end, replacement } = replacements[i];
    processed = processed.slice(0, start) + replacement + processed.slice(end);
  }

  return processed;
}

// Generate static CSS
function generateStaticStyles(theme) {
  const isDark = theme === 'dark';
  
  return `
/* Shiki Syntax Highlighting Styles */
.code-block-wrapper {
  position: relative;
  margin: 1rem 0;
  border-radius: 0.5rem;
  overflow: hidden;
  background-color: ${isDark ? '#282c34' : '#fafafa'};
  box-shadow: 0 2px 4px rgba(0, 0, 0, ${isDark ? '0.2' : '0.1'});
}

.code-title {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  font-weight: 500;
  color: ${isDark ? '#abb2bf' : '#666'};
  background-color: ${isDark ? '#21252b' : '#f0f0f0'};
  border-bottom: 1px solid ${isDark ? '#3e4451' : '#e0e0e0'};
}

.shiki {
  margin: 0;
  padding: 1rem;
  overflow-x: auto;
  font-size: 0.875rem;
  line-height: 1.5;
  font-family: 'Consolas', 'Monaco', 'Andale Mono', 'Ubuntu Mono', monospace;
}

.shiki code {
  display: block;
  width: fit-content;
  min-width: 100%;
}

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

/* Scrollbar */
.shiki::-webkit-scrollbar {
  height: 8px;
}

.shiki::-webkit-scrollbar-track {
  background: transparent;
}

.shiki::-webkit-scrollbar-thumb {
  background: ${isDark ? '#5c6370' : '#ccc'};
  border-radius: 4px;
}

.shiki::-webkit-scrollbar-thumb:hover {
  background: ${isDark ? '#abb2bf' : '#999'};
}

/* Language indicator */
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
  pointer-events: none;
}

.code-block-wrapper[data-title]::before {
  top: 3rem;
}

/* Dark theme specific */
.dark .code-block-wrapper {
  background-color: #282c34;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.dark .code-title {
  color: #abb2bf;
  background-color: #21252b;
  border-bottom-color: #3e4451;
}

.dark .line-number {
  color: #5c6370;
}

.dark .line-content.highlighted {
  background-color: rgba(255, 255, 255, 0.1);
}

.dark code:not(.shiki code) {
  color: #e06c75;
  background-color: rgba(255, 255, 255, 0.1);
}

/* Light theme specific */
.light .code-block-wrapper {
  background-color: #fafafa;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.light .code-title {
  color: #666;
  background-color: #f0f0f0;
  border-bottom-color: #e0e0e0;
}

.light .line-number {
  color: #999;
}

.light .line-content.highlighted {
  background-color: rgba(0, 0, 0, 0.05);
}

.light code:not(.shiki code) {
  color: #d14;
  background-color: rgba(0, 0, 0, 0.05);
}
`;
}

// Export utilities
export { processMarkdown, generateStaticStyles };