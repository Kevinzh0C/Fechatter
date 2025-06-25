// High-performance Markdown WebWorker with incremental diffing
import { unified } from 'unified';
import remarkParse from 'remark-parse';
import remarkGfm from 'remark-gfm';
import remarkRehype from 'remark-rehype';
import rehypeStringify from 'rehype-stringify';
import rehypeSanitize from 'rehype-sanitize';
import rehypeHighlight from 'rehype-highlight';

// Initialize the markdown processor
const processor = unified()
  .use(remarkParse)
  .use(remarkGfm)
  .use(remarkRehype, { allowDangerousHtml: false })
  .use(rehypeSanitize)
  .use(rehypeHighlight, { plainText: ['txt', 'text'] })
  .use(rehypeStringify);

// Cache for previous ASTs
const astCache = new Map();
const htmlCache = new Map();

// Debounce timer
let debounceTimer = null;
const pendingTasks = new Map();

// Process markdown with incremental diffing
async function processMarkdown(id, content, immediate = false) {
  // Clear existing timer for this ID
  if (pendingTasks.has(id)) {
    clearTimeout(pendingTasks.get(id));
  }

  // Debounce unless immediate
  if (!immediate) {
    return new Promise((resolve) => {
      const timer = setTimeout(() => {
        pendingTasks.delete(id);
        performProcessing(id, content).then(resolve);
      }, 50);
      pendingTasks.set(id, timer);
    });
  }

  return performProcessing(id, content);
}

async function performProcessing(id, content) {
  try {
    const startTime = performance.now();
    
    // Parse to AST
    const ast = processor.parse(content);
    const previousAst = astCache.get(id);
    
    // If no previous AST, render full content
    if (!previousAst) {
      const html = await processor.run(ast).then(tree => processor.stringify(tree));
      astCache.set(id, ast);
      htmlCache.set(id, html);
      
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
    
    // Apply patches and generate partial HTML
    const partialUpdates = await generatePartialUpdates(ast, patches);
    
    // Update caches
    astCache.set(id, ast);
    
    return {
      id,
      type: 'incremental',
      patches: partialUpdates,
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

// Diff two AST trees and return patches
function diffAST(oldNode, newNode, path = []) {
  const patches = [];
  
  // Both null/undefined
  if (!oldNode && !newNode) return patches;
  
  // Node added
  if (!oldNode && newNode) {
    patches.push({
      type: 'add',
      path: [...path],
      node: newNode
    });
    return patches;
  }
  
  // Node removed
  if (oldNode && !newNode) {
    patches.push({
      type: 'remove',
      path: [...path]
    });
    return patches;
  }
  
  // Different node types
  if (oldNode.type !== newNode.type) {
    patches.push({
      type: 'replace',
      path: [...path],
      oldNode,
      newNode
    });
    return patches;
  }
  
  // Compare node properties
  const propsChanged = hasPropertiesChanged(oldNode, newNode);
  if (propsChanged) {
    patches.push({
      type: 'update',
      path: [...path],
      oldNode,
      newNode,
      changes: propsChanged
    });
  }
  
  // Compare children
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

// Check if node properties have changed
function hasPropertiesChanged(oldNode, newNode) {
  const changes = {};
  
  // Check value changes (for text nodes)
  if (oldNode.value !== newNode.value) {
    changes.value = { old: oldNode.value, new: newNode.value };
  }
  
  // Check position changes
  if (JSON.stringify(oldNode.position) !== JSON.stringify(newNode.position)) {
    changes.position = { old: oldNode.position, new: newNode.position };
  }
  
  // Check data changes
  if (JSON.stringify(oldNode.data) !== JSON.stringify(newNode.data)) {
    changes.data = { old: oldNode.data, new: newNode.data };
  }
  
  return Object.keys(changes).length > 0 ? changes : null;
}

// Generate partial HTML updates from patches
async function generatePartialUpdates(fullAst, patches) {
  const updates = [];
  
  for (const patch of patches) {
    const node = getNodeByPath(fullAst, patch.path);
    
    if (patch.type === 'add' || patch.type === 'replace' || patch.type === 'update') {
      // Render the new/updated node to HTML
      const htmlNode = await processor.run(patch.newNode || node);
      const html = processor.stringify(htmlNode);
      
      updates.push({
        ...patch,
        html,
        selector: generateSelector(patch.path)
      });
    } else {
      updates.push({
        ...patch,
        selector: generateSelector(patch.path)
      });
    }
  }
  
  return updates;
}

// Get node by path in AST
function getNodeByPath(root, path) {
  let current = root;
  
  for (const segment of path) {
    if (segment === 'children' || typeof segment === 'number') {
      if (segment === 'children') continue;
      current = current.children?.[segment];
    } else {
      current = current[segment];
    }
    
    if (!current) break;
  }
  
  return current;
}

// Generate CSS selector from path
function generateSelector(path) {
  const segments = [];
  let depth = 0;
  
  for (let i = 0; i < path.length; i++) {
    if (path[i] === 'children' && typeof path[i + 1] === 'number') {
      depth++;
      segments.push(`:nth-child(${path[i + 1] + 1})`);
      i++; // Skip the number
    }
  }
  
  // Generate a data attribute selector for precise targeting
  return `[data-md-path="${path.join('-')}"]`;
}

// Clean up resources for a specific document
function cleanup(id) {
  astCache.delete(id);
  htmlCache.delete(id);
  if (pendingTasks.has(id)) {
    clearTimeout(pendingTasks.get(id));
    pendingTasks.delete(id);
  }
}

// Message handler
self.addEventListener('message', async (event) => {
  const { type, id, content, immediate } = event.data;
  
  switch (type) {
    case 'render':
      const result = await processMarkdown(id, content, immediate);
      self.postMessage(result);
      break;
      
    case 'cleanup':
      cleanup(id);
      self.postMessage({ type: 'cleanup-complete', id });
      break;
      
    case 'clear-all':
      astCache.clear();
      htmlCache.clear();
      pendingTasks.forEach(timer => clearTimeout(timer));
      pendingTasks.clear();
      self.postMessage({ type: 'clear-complete' });
      break;
  }
});

// Export for testing
if (typeof module !== 'undefined') {
  module.exports = { diffAST, processMarkdown };
}