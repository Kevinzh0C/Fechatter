// High-performance Markdown WebWorker with Shiki syntax highlighting
import { unified } from 'unified';
import remarkParse from 'remark-parse';
import remarkGfm from 'remark-gfm';
import remarkBreaks from 'remark-breaks';
import remarkRehype from 'remark-rehype';
import rehypeStringify from 'rehype-stringify';
import rehypeSanitize from 'rehype-sanitize';
import { createHighlighter } from 'shiki';
import { parseCodeBlockMeta, resolveLanguage } from '../plugins/shiki.js';

// Initialize Shiki highlighter
let shikiHighlighter = null;
const initHighlighter = async () => {
  if (!shikiHighlighter) {
    shikiHighlighter = await createHighlighter({
      themes: ['vitesse-light', 'one-dark-pro'],
      langs: [
        'javascript',
        'typescript',
        'jsx',
        'tsx',
        'vue',
        'html',
        'css',
        'json',
        'yaml',
        'markdown',
        'bash',
        'python',
        'rust',
        'go',
        'sql',
        'plaintext'
      ]
    });
  }
  return shikiHighlighter;
};

// Custom rehype plugin for Shiki highlighting
function rehypeShiki(options = {}) {
  const { theme = 'dark', lineNumbers = true } = options;
  const themeKey = theme === 'dark' ? 'one-dark-pro' : 'vitesse-light';

  return async (tree) => {
    const highlighter = await initHighlighter();
    const { visit } = await import('unist-util-visit');

    const codeBlocks = [];
    
    // Find all code blocks
    visit(tree, 'element', (node, index, parent) => {
      if (node.tagName === 'pre' && node.children?.[0]?.tagName === 'code') {
        codeBlocks.push({ node, index, parent });
      }
    });

    // Process code blocks in parallel
    const processPromises = codeBlocks.map(async ({ node, index, parent }) => {
      const codeNode = node.children[0];
      const lang = extractLanguage(codeNode);
      const code = extractCode(codeNode);
      const meta = codeNode.data?.meta || '';

      if (!code) return;

      try {
        const metadata = parseCodeBlockMeta(meta);
        const resolvedLang = resolveLanguage(lang);

        // Get highlighted HTML from Shiki
        const html = highlighter.codeToHtml(code, {
          lang: resolvedLang,
          theme: themeKey
        });

        // Create wrapper structure
        const wrapper = {
          type: 'element',
          tagName: 'div',
          properties: {
            className: ['code-block-wrapper'],
            'data-lang': resolvedLang
          },
          children: []
        };

        // Add title if provided
        if (metadata.title) {
          wrapper.properties['data-title'] = metadata.title;
          wrapper.children.push({
            type: 'element',
            tagName: 'div',
            properties: { className: ['code-title'] },
            children: [{ type: 'text', value: metadata.title }]
          });
        }

        // Parse the highlighted HTML
        const { fromHtml } = await import('hast-util-from-html');
        const highlightedAst = fromHtml(html, { fragment: true });
        const preNode = highlightedAst.children.find(child => child.tagName === 'pre');

        if (preNode && metadata.showLineNumbers !== false && lineNumbers) {
          // Add line numbers and highlighting
          addLineNumbers(preNode, {
            highlightLines: metadata.highlightLines,
            startLine: metadata.startLine
          });
        }

        wrapper.children.push(preNode);

        // Replace the original node
        parent.children[index] = wrapper;
      } catch (error) {
        console.error('Failed to highlight code:', error);
        // Keep original node on error
      }
    });

    await Promise.all(processPromises);
  };
}

// Extract language from code node
function extractLanguage(codeNode) {
  const className = codeNode.properties?.className;
  if (Array.isArray(className)) {
    const langClass = className.find(c => c.startsWith('language-'));
    return langClass ? langClass.replace('language-', '') : 'plaintext';
  }
  return 'plaintext';
}

// Extract code content from node
function extractCode(node) {
  if (node.children?.[0]?.type === 'text') {
    return node.children[0].value;
  }
  
  // Recursively extract text
  const extractText = (n) => {
    if (n.type === 'text') return n.value;
    if (n.children) return n.children.map(extractText).join('');
    return '';
  };
  
  return extractText(node);
}

// Add line numbers to highlighted code
function addLineNumbers(preNode, options) {
  const { highlightLines = [], startLine = 1 } = options;
  const codeNode = preNode.children.find(child => child.tagName === 'code');
  
  if (!codeNode) return;

  // Split content into lines
  const lines = [];
  const processNode = (node) => {
    if (node.type === 'text') {
      const nodeLines = node.value.split('\n');
      for (let i = 0; i < nodeLines.length; i++) {
        if (i > 0) lines.push({ type: 'newline', elements: [] });
        if (nodeLines[i]) {
          const currentLine = lines[lines.length - 1] || { type: 'line', elements: [] };
          if (currentLine.type !== 'line') {
            lines.push({ type: 'line', elements: [{ ...node, value: nodeLines[i] }] });
          } else {
            currentLine.elements.push({ ...node, value: nodeLines[i] });
          }
        }
      }
    } else if (node.children) {
      node.children.forEach(processNode);
    } else {
      const currentLine = lines[lines.length - 1] || { type: 'line', elements: [] };
      if (currentLine.type === 'line') {
        currentLine.elements.push(node);
      }
    }
  };

  codeNode.children.forEach(processNode);

  // Rebuild code node with line numbers
  codeNode.children = [];
  lines.forEach((line, index) => {
    if (line.type === 'newline') {
      codeNode.children.push({ type: 'text', value: '\n' });
      return;
    }

    const lineNum = startLine + index;
    const isHighlighted = highlightLines.includes(lineNum);

    const lineElement = {
      type: 'element',
      tagName: 'span',
      properties: { className: ['line'] },
      children: [
        {
          type: 'element',
          tagName: 'span',
          properties: { 
            className: ['line-number'],
            'data-line': lineNum
          },
          children: [{ type: 'text', value: String(lineNum) }]
        },
        {
          type: 'element',
          tagName: 'span',
          properties: { 
            className: ['line-content', ...(isHighlighted ? ['highlighted'] : [])]
          },
          children: line.elements
        }
      ]
    };

    codeNode.children.push(lineElement);
  });
}

// Initialize the markdown processor with Shiki
const createProcessor = (theme = 'dark') => {
  return unified()
    .use(remarkParse)
    .use(remarkGfm)
    .use(remarkBreaks)
    .use(remarkRehype, { allowDangerousHtml: false })
    .use(rehypeSanitize)
    .use(rehypeShiki, { theme, lineNumbers: true })
    .use(rehypeStringify);
};

// Cache for processors and ASTs
const processorCache = new Map();
const astCache = new Map();
const htmlCache = new Map();

// Debounce timer
const pendingTasks = new Map();

// Process markdown with incremental diffing
async function processMarkdown(id, content, options = {}) {
  const { immediate = false, theme = 'dark' } = options;

  // Clear existing timer for this ID
  if (pendingTasks.has(id)) {
    clearTimeout(pendingTasks.get(id));
  }

  // Debounce unless immediate
  if (!immediate) {
    return new Promise((resolve) => {
      const timer = setTimeout(() => {
        pendingTasks.delete(id);
        performProcessing(id, content, theme).then(resolve);
      }, 50);
      pendingTasks.set(id, timer);
    });
  }

  return performProcessing(id, content, theme);
}

async function performProcessing(id, content, theme) {
  try {
    const startTime = performance.now();
    
    // Get or create processor for theme
    const processorKey = `processor-${theme}`;
    let processor = processorCache.get(processorKey);
    if (!processor) {
      processor = createProcessor(theme);
      processorCache.set(processorKey, processor);
    }

    // Parse to AST
    const ast = processor.parse(content);
    const previousAst = astCache.get(id);
    
    // If no previous AST or theme changed, render full content
    const previousTheme = htmlCache.get(`${id}-theme`);
    if (!previousAst || previousTheme !== theme) {
      const processedAst = await processor.run(ast);
      const html = processor.stringify(processedAst);
      
      astCache.set(id, ast);
      htmlCache.set(id, html);
      htmlCache.set(`${id}-theme`, theme);
      
      return {
        id,
        type: 'full',
        html,
        processingTime: performance.now() - startTime
      };
    }
    
    // Perform incremental diff
    const patches = diffAST(previousAst, ast);
    
    // If no changes, return early
    if (patches.length === 0) {
      return {
        id,
        type: 'no-change',
        processingTime: performance.now() - startTime
      };
    }
    
    // For now, do a full re-render with Shiki
    // (incremental updates with Shiki would require more complex logic)
    const processedAst = await processor.run(ast);
    const html = processor.stringify(processedAst);
    
    astCache.set(id, ast);
    htmlCache.set(id, html);
    
    return {
      id,
      type: 'full',
      html,
      processingTime: performance.now() - startTime
    };
    
  } catch (error) {
    console.error('Markdown processing error:', error);
    return {
      id,
      type: 'error',
      error: error.message
    };
  }
}

// Simplified AST diffing (reuse from original)
function diffAST(oldNode, newNode, path = []) {
  const patches = [];
  
  if (!oldNode && !newNode) return patches;
  
  if (!oldNode && newNode) {
    patches.push({ type: 'add', path: [...path], node: newNode });
    return patches;
  }
  
  if (oldNode && !newNode) {
    patches.push({ type: 'remove', path: [...path] });
    return patches;
  }
  
  if (oldNode.type !== newNode.type) {
    patches.push({ type: 'replace', path: [...path], oldNode, newNode });
    return patches;
  }
  
  if (oldNode.value !== newNode.value || 
      JSON.stringify(oldNode.position) !== JSON.stringify(newNode.position)) {
    patches.push({ type: 'update', path: [...path], oldNode, newNode });
  }
  
  if (oldNode.children && newNode.children) {
    const maxLength = Math.max(oldNode.children.length, newNode.children.length);
    for (let i = 0; i < maxLength; i++) {
      const childPatches = diffAST(
        oldNode.children[i],
        newNode.children[i],
        [...path, 'children', i]
      );
      patches.push(...childPatches);
    }
  }
  
  return patches;
}

// Clean up resources
function cleanup(id) {
  astCache.delete(id);
  htmlCache.delete(id);
  htmlCache.delete(`${id}-theme`);
  if (pendingTasks.has(id)) {
    clearTimeout(pendingTasks.get(id));
    pendingTasks.delete(id);
  }
}

// Message handler
self.addEventListener('message', async (event) => {
  const { type, id, content, immediate, theme } = event.data;
  
  switch (type) {
    case 'render':
      const result = await processMarkdown(id, content, { immediate, theme });
      self.postMessage(result);
      break;
      
    case 'cleanup':
      cleanup(id);
      self.postMessage({ type: 'cleanup-complete', id });
      break;
      
    case 'clear-all':
      astCache.clear();
      htmlCache.clear();
      processorCache.clear();
      pendingTasks.forEach(timer => clearTimeout(timer));
      pendingTasks.clear();
      self.postMessage({ type: 'clear-complete' });
      break;
      
    case 'init':
      // Pre-initialize highlighter
      await initHighlighter();
      self.postMessage({ type: 'init-complete' });
      break;
  }
});

// Export for testing
if (typeof module !== 'undefined') {
  module.exports = { diffAST, processMarkdown };
}