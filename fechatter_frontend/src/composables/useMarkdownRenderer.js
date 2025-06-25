// High-performance Markdown renderer with WebWorker and incremental updates
import { ref, onUnmounted, watch, shallowRef } from 'vue';
import MarkdownWorker from '../workers/markdown.worker.js?worker';

// Shared worker instance for better resource management
let sharedWorker = null;
let workerRefCount = 0;

export function useMarkdownRenderer(options = {}) {
  const {
    immediate = false,
    preserveScroll = true,
    maxConcurrent = 3
  } = options;

  // State
  const isProcessing = ref(false);
  const processingTime = ref(0);
  const error = ref(null);
  const renderCount = ref(0);
  
  // Virtual DOM state
  const virtualDOM = shallowRef(new Map());
  const anchorPositions = new Map();
  const pendingPatches = [];
  
  // Worker management
  let worker = null;
  let messageQueue = [];
  let activeRequests = 0;
  
  // Initialize worker
  function initWorker() {
    if (sharedWorker && workerRefCount > 0) {
      worker = sharedWorker;
      workerRefCount++;
    } else {
      worker = new MarkdownWorker();
      sharedWorker = worker;
      workerRefCount = 1;
      
      // Set up message handler
      worker.addEventListener('message', handleWorkerMessage);
      worker.addEventListener('error', handleWorkerError);
    }
  }
  
  // Handle worker messages
  function handleWorkerMessage(event) {
    const data = event.data;
    activeRequests = Math.max(0, activeRequests - 1);
    
    // Process next queued message if any
    if (messageQueue.length > 0 && activeRequests < maxConcurrent) {
      const next = messageQueue.shift();
      sendToWorker(next);
    }
    
    // Update processing state
    if (activeRequests === 0) {
      isProcessing.value = false;
    }
    
    // Handle response based on type
    switch (data.type) {
      case 'full':
        handleFullRender(data);
        break;
        
      case 'incremental':
        handleIncrementalUpdate(data);
        break;
        
      case 'no-change':
        // No updates needed
        break;
        
      case 'error':
        error.value = data.error;
        break;
    }
    
    // Update metrics
    if (data.processingTime) {
      processingTime.value = data.processingTime;
    }
  }
  
  // Handle worker errors
  function handleWorkerError(event) {
    console.error('Markdown worker error:', event);
    error.value = 'Worker error: ' + event.message;
    isProcessing.value = false;
  }
  
  // Send message to worker with queueing
  function sendToWorker(message) {
    if (activeRequests >= maxConcurrent) {
      messageQueue.push(message);
      return;
    }
    
    activeRequests++;
    isProcessing.value = true;
    worker.postMessage(message);
  }
  
  // Render markdown content
  function render(id, content) {
    if (!worker) {
      initWorker();
    }
    
    error.value = null;
    
    sendToWorker({
      type: 'render',
      id,
      content,
      immediate
    });
  }
  
  // Handle full render response
  function handleFullRender(data) {
    const container = document.querySelector(`[data-markdown-id="${data.id}"]`);
    if (!container) return;
    
    // Save scroll position and anchors
    const scrollTop = preserveScroll ? container.scrollTop : 0;
    saveAnchorPositions(container);
    
    // Apply full HTML
    container.innerHTML = annotateHTML(data.html, []);
    
    // Restore scroll and anchors
    if (preserveScroll) {
      container.scrollTop = scrollTop;
      restoreAnchorPositions(container);
    }
    
    // Update virtual DOM
    rebuildVirtualDOM(container, data.id);
    renderCount.value++;
  }
  
  // Handle incremental update
  function handleIncrementalUpdate(data) {
    const container = document.querySelector(`[data-markdown-id="${data.id}"]`);
    if (!container) return;
    
    // Batch patches for optimal rendering
    pendingPatches.push(...data.patches);
    
    // Apply patches using requestAnimationFrame for smooth updates
    requestAnimationFrame(() => {
      const scrollTop = preserveScroll ? container.scrollTop : 0;
      const activeElement = document.activeElement;
      
      // Apply all pending patches
      applyPatches(container, pendingPatches);
      pendingPatches.length = 0;
      
      // Restore focus and scroll
      if (preserveScroll) {
        container.scrollTop = scrollTop;
      }
      if (activeElement && container.contains(activeElement)) {
        activeElement.focus();
      }
      
      renderCount.value++;
    });
  }
  
  // Apply patches to DOM
  function applyPatches(container, patches) {
    // Sort patches by depth (deepest first) to avoid conflicts
    patches.sort((a, b) => b.path.length - a.path.length);
    
    for (const patch of patches) {
      try {
        const element = container.querySelector(patch.selector);
        
        switch (patch.type) {
          case 'add':
            if (element) {
              const newElement = createElementFromHTML(
                annotateHTML(patch.html, patch.path)
              );
              element.parentNode.insertBefore(newElement, element.nextSibling);
            }
            break;
            
          case 'remove':
            if (element) {
              element.remove();
            }
            break;
            
          case 'replace':
          case 'update':
            if (element) {
              const newElement = createElementFromHTML(
                annotateHTML(patch.html, patch.path)
              );
              morphElement(element, newElement);
            }
            break;
        }
      } catch (err) {
        console.error('Patch application error:', err, patch);
      }
    }
  }
  
  // Morphing algorithm for smooth DOM updates
  function morphElement(oldEl, newEl) {
    // If elements are different types, replace
    if (oldEl.tagName !== newEl.tagName) {
      oldEl.parentNode.replaceChild(newEl, oldEl);
      return;
    }
    
    // Update attributes
    const oldAttrs = oldEl.attributes;
    const newAttrs = newEl.attributes;
    
    // Remove old attributes
    for (let i = oldAttrs.length - 1; i >= 0; i--) {
      const attr = oldAttrs[i];
      if (!newEl.hasAttribute(attr.name)) {
        oldEl.removeAttribute(attr.name);
      }
    }
    
    // Add/update new attributes
    for (let i = 0; i < newAttrs.length; i++) {
      const attr = newAttrs[i];
      if (oldEl.getAttribute(attr.name) !== attr.value) {
        oldEl.setAttribute(attr.name, attr.value);
      }
    }
    
    // Update children
    morphChildren(oldEl, newEl);
  }
  
  // Morph child nodes
  function morphChildren(oldParent, newParent) {
    const oldChildren = Array.from(oldParent.childNodes);
    const newChildren = Array.from(newParent.childNodes);
    
    const maxLength = Math.max(oldChildren.length, newChildren.length);
    
    for (let i = 0; i < maxLength; i++) {
      const oldChild = oldChildren[i];
      const newChild = newChildren[i];
      
      if (!oldChild && newChild) {
        oldParent.appendChild(newChild.cloneNode(true));
      } else if (oldChild && !newChild) {
        oldChild.remove();
      } else if (oldChild && newChild) {
        if (oldChild.nodeType === 3 && newChild.nodeType === 3) {
          // Text nodes
          if (oldChild.textContent !== newChild.textContent) {
            oldChild.textContent = newChild.textContent;
          }
        } else if (oldChild.nodeType === 1 && newChild.nodeType === 1) {
          // Element nodes
          morphElement(oldChild, newChild);
        } else {
          // Different node types
          oldParent.replaceChild(newChild.cloneNode(true), oldChild);
        }
      }
    }
  }
  
  // Annotate HTML with path data attributes
  function annotateHTML(html, basePath) {
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, 'text/html');
    
    function annotateNode(node, path) {
      if (node.nodeType === 1) { // Element node
        node.setAttribute('data-md-path', path.join('-'));
        
        const children = Array.from(node.children);
        children.forEach((child, index) => {
          annotateNode(child, [...path, 'children', index]);
        });
      }
    }
    
    const body = doc.body;
    Array.from(body.children).forEach((child, index) => {
      annotateNode(child, [...basePath, 'children', index]);
    });
    
    return body.innerHTML;
  }
  
  // Create element from HTML string
  function createElementFromHTML(html) {
    const template = document.createElement('template');
    template.innerHTML = html.trim();
    return template.content.firstChild;
  }
  
  // Save anchor positions for stability
  function saveAnchorPositions(container) {
    anchorPositions.clear();
    
    const anchors = container.querySelectorAll('[id], h1, h2, h3, h4, h5, h6');
    anchors.forEach(anchor => {
      const rect = anchor.getBoundingClientRect();
      const containerRect = container.getBoundingClientRect();
      
      anchorPositions.set(anchor, {
        id: anchor.id || anchor.textContent,
        relativeTop: rect.top - containerRect.top,
        scrollTop: container.scrollTop
      });
    });
  }
  
  // Restore anchor positions after update
  function restoreAnchorPositions(container) {
    anchorPositions.forEach((position, anchor) => {
      const newAnchor = anchor.id 
        ? container.querySelector(`#${CSS.escape(anchor.id)}`)
        : Array.from(container.querySelectorAll(anchor.tagName))
            .find(el => el.textContent === position.id);
            
      if (newAnchor) {
        const rect = newAnchor.getBoundingClientRect();
        const containerRect = container.getBoundingClientRect();
        const currentRelativeTop = rect.top - containerRect.top;
        const drift = currentRelativeTop - position.relativeTop;
        
        if (Math.abs(drift) > 5) {
          container.scrollTop = position.scrollTop + drift;
        }
      }
    });
  }
  
  // Rebuild virtual DOM representation
  function rebuildVirtualDOM(container, id) {
    const vdom = new Map();
    
    function buildVDOM(element, path = []) {
      const node = {
        type: element.tagName,
        attributes: {},
        children: [],
        path
      };
      
      // Copy attributes
      for (const attr of element.attributes) {
        node.attributes[attr.name] = attr.value;
      }
      
      // Process children
      for (let i = 0; i < element.children.length; i++) {
        const child = element.children[i];
        const childPath = [...path, 'children', i];
        node.children.push(buildVDOM(child, childPath));
      }
      
      vdom.set(path.join('-'), node);
      return node;
    }
    
    buildVDOM(container);
    virtualDOM.value.set(id, vdom);
  }
  
  // Cleanup function
  function cleanup(id) {
    if (worker) {
      worker.postMessage({ type: 'cleanup', id });
      virtualDOM.value.delete(id);
      anchorPositions.clear();
    }
  }
  
  // Destroy and cleanup all resources
  function destroy() {
    if (worker) {
      workerRefCount--;
      
      if (workerRefCount === 0) {
        worker.postMessage({ type: 'clear-all' });
        worker.removeEventListener('message', handleWorkerMessage);
        worker.removeEventListener('error', handleWorkerError);
        worker.terminate();
        sharedWorker = null;
      }
      
      worker = null;
    }
    
    virtualDOM.value.clear();
    anchorPositions.clear();
    pendingPatches.length = 0;
    messageQueue.length = 0;
  }
  
  // Auto cleanup on unmount
  onUnmounted(() => {
    destroy();
  });
  
  return {
    // Methods
    render,
    cleanup,
    destroy,
    
    // State
    isProcessing,
    processingTime,
    error,
    renderCount,
    virtualDOM
  };
}