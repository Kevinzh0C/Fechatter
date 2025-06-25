// Markdown AST utilities for diffing and patch generation

// AST node types that should be treated as block elements
const BLOCK_TYPES = new Set([
  'root', 'paragraph', 'heading', 'blockquote', 'code', 'list',
  'listItem', 'table', 'tableRow', 'tableCell', 'thematicBreak',
  'html', 'definition', 'footnoteDefinition'
]);

// AST node types that are inline
const INLINE_TYPES = new Set([
  'text', 'emphasis', 'strong', 'delete', 'link', 'image',
  'inlineCode', 'break', 'footnote', 'footnoteReference'
]);

/**
 * Deep equality check for AST nodes
 */
export function isNodeEqual(a, b) {
  if (a === b) return true;
  if (!a || !b) return false;
  if (a.type !== b.type) return false;
  
  // Check value (for text nodes)
  if (a.value !== b.value) return false;
  
  // Check important properties
  if (a.url !== b.url) return false;
  if (a.title !== b.title) return false;
  if (a.alt !== b.alt) return false;
  if (a.identifier !== b.identifier) return false;
  if (a.label !== b.label) return false;
  if (a.depth !== b.depth) return false;
  if (a.ordered !== b.ordered) return false;
  if (a.start !== b.start) return false;
  if (a.checked !== b.checked) return false;
  
  // Check children length
  const aChildren = a.children || [];
  const bChildren = b.children || [];
  if (aChildren.length !== bChildren.length) return false;
  
  // Deep check children
  for (let i = 0; i < aChildren.length; i++) {
    if (!isNodeEqual(aChildren[i], bChildren[i])) return false;
  }
  
  return true;
}

/**
 * Calculate similarity between two nodes (0-1)
 */
export function nodeSimilarity(a, b) {
  if (!a || !b) return 0;
  if (a.type !== b.type) return 0;
  
  let score = 0.5; // Base score for same type
  
  // Value similarity for text nodes
  if (a.value && b.value) {
    const similarity = stringSimilarity(a.value, b.value);
    score += similarity * 0.3;
  }
  
  // Children similarity
  if (a.children && b.children) {
    const childScore = childrenSimilarity(a.children, b.children);
    score += childScore * 0.2;
  }
  
  return Math.min(score, 1);
}

/**
 * Calculate string similarity using Levenshtein distance
 */
function stringSimilarity(a, b) {
  if (a === b) return 1;
  if (!a || !b) return 0;
  
  const maxLen = Math.max(a.length, b.length);
  if (maxLen === 0) return 1;
  
  const distance = levenshteinDistance(a, b);
  return 1 - (distance / maxLen);
}

/**
 * Levenshtein distance between two strings
 */
function levenshteinDistance(a, b) {
  const matrix = [];
  
  for (let i = 0; i <= b.length; i++) {
    matrix[i] = [i];
  }
  
  for (let j = 0; j <= a.length; j++) {
    matrix[0][j] = j;
  }
  
  for (let i = 1; i <= b.length; i++) {
    for (let j = 1; j <= a.length; j++) {
      if (b.charAt(i - 1) === a.charAt(j - 1)) {
        matrix[i][j] = matrix[i - 1][j - 1];
      } else {
        matrix[i][j] = Math.min(
          matrix[i - 1][j - 1] + 1, // substitution
          matrix[i][j - 1] + 1,     // insertion
          matrix[i - 1][j] + 1      // deletion
        );
      }
    }
  }
  
  return matrix[b.length][a.length];
}

/**
 * Calculate similarity between two arrays of children
 */
function childrenSimilarity(aChildren, bChildren) {
  if (aChildren.length === 0 && bChildren.length === 0) return 1;
  if (aChildren.length === 0 || bChildren.length === 0) return 0;
  
  let totalSimilarity = 0;
  const maxLength = Math.max(aChildren.length, bChildren.length);
  
  for (let i = 0; i < maxLength; i++) {
    if (i < aChildren.length && i < bChildren.length) {
      totalSimilarity += nodeSimilarity(aChildren[i], bChildren[i]);
    }
  }
  
  return totalSimilarity / maxLength;
}

/**
 * Generate efficient patches between two AST trees
 */
export function generatePatches(oldAST, newAST) {
  const patches = [];
  const visited = new Set();
  
  function diff(oldNode, newNode, path = []) {
    const pathKey = path.join('.');
    if (visited.has(pathKey)) return;
    visited.add(pathKey);
    
    // Both null
    if (!oldNode && !newNode) return;
    
    // Node added
    if (!oldNode && newNode) {
      patches.push({
        type: 'insert',
        path: [...path],
        node: cloneNode(newNode),
        priority: calculatePriority(newNode, 'insert')
      });
      return;
    }
    
    // Node removed
    if (oldNode && !newNode) {
      patches.push({
        type: 'delete',
        path: [...path],
        priority: calculatePriority(oldNode, 'delete')
      });
      return;
    }
    
    // Nodes are equal - no change needed
    if (isNodeEqual(oldNode, newNode)) return;
    
    // Calculate similarity to decide between update and replace
    const similarity = nodeSimilarity(oldNode, newNode);
    
    if (similarity < 0.3) {
      // Too different - replace entirely
      patches.push({
        type: 'replace',
        path: [...path],
        oldNode: cloneNode(oldNode),
        newNode: cloneNode(newNode),
        priority: calculatePriority(newNode, 'replace')
      });
    } else {
      // Similar enough - try to update
      const changes = getNodeChanges(oldNode, newNode);
      if (Object.keys(changes).length > 0) {
        patches.push({
          type: 'update',
          path: [...path],
          changes,
          priority: calculatePriority(newNode, 'update')
        });
      }
      
      // Diff children
      if (oldNode.children || newNode.children) {
        diffChildren(
          oldNode.children || [],
          newNode.children || [],
          [...path, 'children']
        );
      }
    }
  }
  
  function diffChildren(oldChildren, newChildren, basePath) {
    // Use Myers' diff algorithm for optimal child matching
    const matches = findBestMatches(oldChildren, newChildren);
    
    matches.forEach(match => {
      if (match.type === 'match') {
        diff(
          oldChildren[match.oldIndex],
          newChildren[match.newIndex],
          [...basePath, match.newIndex]
        );
      } else if (match.type === 'insert') {
        diff(null, newChildren[match.newIndex], [...basePath, match.newIndex]);
      } else if (match.type === 'delete') {
        diff(oldChildren[match.oldIndex], null, [...basePath, match.oldIndex]);
      }
    });
  }
  
  diff(oldAST, newAST);
  
  // Sort patches by priority and depth
  patches.sort((a, b) => {
    if (a.priority !== b.priority) return b.priority - a.priority;
    return a.path.length - b.path.length;
  });
  
  return patches;
}

/**
 * Find best matches between old and new children arrays
 */
function findBestMatches(oldChildren, newChildren) {
  const matches = [];
  const oldMatched = new Set();
  const newMatched = new Set();
  
  // First pass: find exact matches
  for (let newIdx = 0; newIdx < newChildren.length; newIdx++) {
    for (let oldIdx = 0; oldIdx < oldChildren.length; oldIdx++) {
      if (oldMatched.has(oldIdx) || newMatched.has(newIdx)) continue;
      
      if (isNodeEqual(oldChildren[oldIdx], newChildren[newIdx])) {
        matches.push({
          type: 'match',
          oldIndex: oldIdx,
          newIndex: newIdx,
          score: 1
        });
        oldMatched.add(oldIdx);
        newMatched.add(newIdx);
        break;
      }
    }
  }
  
  // Second pass: find similar nodes
  for (let newIdx = 0; newIdx < newChildren.length; newIdx++) {
    if (newMatched.has(newIdx)) continue;
    
    let bestMatch = null;
    let bestScore = 0.3; // Minimum similarity threshold
    
    for (let oldIdx = 0; oldIdx < oldChildren.length; oldIdx++) {
      if (oldMatched.has(oldIdx)) continue;
      
      const score = nodeSimilarity(oldChildren[oldIdx], newChildren[newIdx]);
      if (score > bestScore) {
        bestScore = score;
        bestMatch = oldIdx;
      }
    }
    
    if (bestMatch !== null) {
      matches.push({
        type: 'match',
        oldIndex: bestMatch,
        newIndex: newIdx,
        score: bestScore
      });
      oldMatched.add(bestMatch);
      newMatched.add(newIdx);
    }
  }
  
  // Add unmatched nodes as inserts/deletes
  for (let newIdx = 0; newIdx < newChildren.length; newIdx++) {
    if (!newMatched.has(newIdx)) {
      matches.push({
        type: 'insert',
        newIndex: newIdx
      });
    }
  }
  
  for (let oldIdx = 0; oldIdx < oldChildren.length; oldIdx++) {
    if (!oldMatched.has(oldIdx)) {
      matches.push({
        type: 'delete',
        oldIndex: oldIdx
      });
    }
  }
  
  // Sort by index for sequential processing
  matches.sort((a, b) => {
    const aIdx = a.newIndex ?? a.oldIndex;
    const bIdx = b.newIndex ?? b.oldIndex;
    return aIdx - bIdx;
  });
  
  return matches;
}

/**
 * Get specific changes between two nodes
 */
function getNodeChanges(oldNode, newNode) {
  const changes = {};
  
  // Text value changes
  if (oldNode.value !== newNode.value) {
    changes.value = {
      old: oldNode.value,
      new: newNode.value
    };
  }
  
  // Property changes
  const props = ['url', 'title', 'alt', 'identifier', 'label', 'depth', 'ordered', 'start', 'checked'];
  props.forEach(prop => {
    if (oldNode[prop] !== newNode[prop]) {
      changes[prop] = {
        old: oldNode[prop],
        new: newNode[prop]
      };
    }
  });
  
  return changes;
}

/**
 * Calculate patch priority based on node type and operation
 */
function calculatePriority(node, operation) {
  let priority = 0;
  
  // Operation priorities
  switch (operation) {
    case 'delete': priority = 30; break;
    case 'insert': priority = 20; break;
    case 'replace': priority = 25; break;
    case 'update': priority = 10; break;
  }
  
  // Node type priorities
  if (node.type === 'heading') priority += 15;
  else if (node.type === 'code') priority += 10;
  else if (node.type === 'list') priority += 8;
  else if (node.type === 'blockquote') priority += 5;
  else if (INLINE_TYPES.has(node.type)) priority += 2;
  
  return priority;
}

/**
 * Clone AST node without circular references
 */
function cloneNode(node) {
  if (!node) return null;
  
  const clone = {
    type: node.type
  };
  
  // Copy properties
  const props = ['value', 'url', 'title', 'alt', 'identifier', 'label', 
                 'depth', 'ordered', 'start', 'checked', 'lang', 'meta'];
  
  props.forEach(prop => {
    if (node[prop] !== undefined) {
      clone[prop] = node[prop];
    }
  });
  
  // Clone children
  if (node.children) {
    clone.children = node.children.map(child => cloneNode(child));
  }
  
  return clone;
}

/**
 * Generate DOM patches from AST patches
 */
export function generateDOMPatches(patches, options = {}) {
  const { preserveWhitespace = false } = options;
  
  return patches.map(patch => {
    const domPatch = {
      ...patch,
      selector: pathToSelector(patch.path),
      xpath: pathToXPath(patch.path)
    };
    
    // Add DOM-specific instructions
    if (patch.type === 'update' && patch.changes.value) {
      domPatch.textContent = preserveWhitespace 
        ? patch.changes.value.new 
        : patch.changes.value.new.trim();
    }
    
    return domPatch;
  });
}

/**
 * Convert AST path to CSS selector
 */
function pathToSelector(path) {
  let selector = '';
  let currentDepth = 0;
  
  for (let i = 0; i < path.length; i += 2) {
    if (path[i] === 'children' && typeof path[i + 1] === 'number') {
      currentDepth++;
      const index = path[i + 1] + 1; // CSS uses 1-based indexing
      selector += `:nth-child(${index})`;
      
      if (i + 2 < path.length) {
        selector += ' > *';
      }
    }
  }
  
  return selector || '*';
}

/**
 * Convert AST path to XPath
 */
function pathToXPath(path) {
  let xpath = '';
  
  for (let i = 0; i < path.length; i += 2) {
    if (path[i] === 'children' && typeof path[i + 1] === 'number') {
      const index = path[i + 1] + 1; // XPath uses 1-based indexing
      xpath += `/*[${index}]`;
    }
  }
  
  return xpath || '/';
}

/**
 * Optimize patches by combining adjacent operations
 */
export function optimizePatches(patches) {
  const optimized = [];
  const grouped = new Map();
  
  // Group patches by parent path
  patches.forEach(patch => {
    const parentPath = patch.path.slice(0, -2).join('.');
    if (!grouped.has(parentPath)) {
      grouped.set(parentPath, []);
    }
    grouped.get(parentPath).push(patch);
  });
  
  // Combine adjacent operations
  grouped.forEach(group => {
    // Sort by index
    group.sort((a, b) => {
      const aIdx = a.path[a.path.length - 1];
      const bIdx = b.path[b.path.length - 1];
      return aIdx - bIdx;
    });
    
    let combined = null;
    
    group.forEach(patch => {
      if (!combined) {
        combined = patch;
      } else if (canCombine(combined, patch)) {
        combined = combinePatch(combined, patch);
      } else {
        optimized.push(combined);
        combined = patch;
      }
    });
    
    if (combined) {
      optimized.push(combined);
    }
  });
  
  return optimized;
}

/**
 * Check if two patches can be combined
 */
function canCombine(a, b) {
  // Same path prefix and adjacent indices
  if (a.path.length !== b.path.length) return false;
  
  for (let i = 0; i < a.path.length - 1; i++) {
    if (a.path[i] !== b.path[i]) return false;
  }
  
  const aIdx = a.path[a.path.length - 1];
  const bIdx = b.path[b.path.length - 1];
  
  return Math.abs(aIdx - bIdx) === 1 && a.type === b.type;
}

/**
 * Combine two patches into one
 */
function combinePatch(a, b) {
  return {
    type: 'bulk-' + a.type,
    path: a.path.slice(0, -1),
    startIndex: Math.min(a.path[a.path.length - 1], b.path[b.path.length - 1]),
    endIndex: Math.max(a.path[a.path.length - 1], b.path[b.path.length - 1]),
    patches: [a, b],
    priority: Math.max(a.priority, b.priority)
  };
}

// Export for testing
export const testUtils = {
  isNodeEqual,
  nodeSimilarity,
  findBestMatches,
  calculatePriority
};