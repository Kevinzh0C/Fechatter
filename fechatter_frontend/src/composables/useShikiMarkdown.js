// Composable for integrating Shiki with existing markdown rendering
import { ref, computed, watch, nextTick } from 'vue';
import { useMarkdownRenderer } from './useMarkdownRenderer.js';
// Remove static import to avoid conflicts with dynamic imports
// import { highlightMarkdownCode } from '../utils/codeHighlight.js';

export function useShikiMarkdown(options = {}) {
  const {
    theme = 'dark',
    lineNumbers = true,
    useWorker = true,
    fallbackToOriginal = true
  } = options;

  // Get original markdown renderer
  const markdownRenderer = useMarkdownRenderer({
    immediate: options.immediate,
    preserveScroll: options.preserveScroll,
    maxConcurrent: options.maxConcurrent
  });

  // Additional state for Shiki
  const isHighlighting = ref(false);
  const highlightError = ref(null);
  const currentTheme = ref(theme);

  // Override render method to add Shiki highlighting
  const originalRender = markdownRenderer.render;

  // Cached highlight functions
  let highlightMarkdownCode = null;
  let highlightSingleCodeBlock = null;
  let generateHighlightStyles = null;

  /**
   * Initialize Shiki with dynamic imports
   */
  const initialize = async () => {
    if (isHighlighting.value || highlightError.value) return;

    isHighlighting.value = true;
    highlightError.value = null;

    try {
      // Load all highlight functions dynamically
      const codeHighlightModule = await import('../utils/codeHighlight.js');

      highlightMarkdownCode = codeHighlightModule.highlightMarkdownCode;
      highlightSingleCodeBlock = codeHighlightModule.highlightSingleCodeBlock;
      generateHighlightStyles = codeHighlightModule.generateHighlightStyles;

      // Initialize themes and languages if needed
      if (codeHighlightModule.initializeShiki) {
        await codeHighlightModule.initializeShiki();
      }

      // Update styles
      const styles = await generateHighlightStyles(currentTheme.value);
      updateDynamicStyles(styles);

      // Re-highlight all visible code blocks
      const containers = document.querySelectorAll('[data-markdown-id]');
      for (const container of containers) {
        const id = container.getAttribute('data-markdown-id');
        await applyShikiHighlighting(id);
      }
    } catch (err) {
      highlightError.value = err;
      console.error('Failed to initialize Shiki:', err);

      if (fallbackToOriginal) {
        // Fallback to original rendering without Shiki
        await originalRender.call(markdownRenderer, id, content);
      }
    } finally {
      isHighlighting.value = false;
    }
  };

  async function renderWithShiki(id, content) {
    try {
      await initialize();

      if (useWorker) {
        // Use worker for markdown parsing and Shiki for highlighting
        // First, let the worker process the markdown structure
        await originalRender.call(markdownRenderer, id, content);

        // Then apply Shiki highlighting to code blocks in the DOM
        await applyShikiHighlighting(id);
      } else {
        // Process markdown with Shiki before rendering
        const highlightedContent = await highlightMarkdownCode(content, {
          theme: currentTheme.value,
          lineNumbers,
          cache: true
        });

        // Render the pre-highlighted content
        await originalRender.call(markdownRenderer, id, highlightedContent);
      }
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Shiki highlighting error:', error);
      }
      highlightError.value = error;

      if (fallbackToOriginal) {
        // Fallback to original rendering without Shiki
        await originalRender.call(markdownRenderer, id, content);
      }
    }
  }

  // Apply Shiki highlighting to already rendered code blocks
  async function applyShikiHighlighting(id) {
    const container = document.querySelector(`[data-markdown-id="${id}"]`);
    if (!container) return;

    const codeBlocks = container.querySelectorAll('pre > code');
    const highlightPromises = [];

    for (const codeBlock of codeBlocks) {
      const pre = codeBlock.parentElement;
      const lang = extractLanguageFromClass(codeBlock.className);
      const code = codeBlock.textContent;

      if (!code || !lang) continue;

      highlightPromises.push(
        highlightCodeBlock(pre, codeBlock, code, lang)
      );
    }

    await Promise.all(highlightPromises);
  }

  // Highlight individual code block
  async function highlightCodeBlock(pre, codeBlock, code, lang) {
    try {
      if (!highlightSingleCodeBlock) {
        // Fallback: load function dynamically
        const { highlightSingleCodeBlock: loadedFunction } = await import('../utils/codeHighlight.js');
        highlightSingleCodeBlock = loadedFunction;
      }

      // Extract metadata from data attributes if available
      const meta = pre.getAttribute('data-meta') || '';

      const highlighted = await highlightSingleCodeBlock(code, lang, meta, {
        theme: currentTheme.value,
        lineNumbers
      });

      // Create a temporary container to parse the highlighted HTML
      const temp = document.createElement('div');
      temp.innerHTML = highlighted;

      // Replace the pre element with the highlighted version
      const highlightedElement = temp.firstElementChild;
      if (highlightedElement) {
        pre.parentNode.replaceChild(highlightedElement, pre);
      }
    } catch (error) {
      if (import.meta.env.DEV) {
        console.warn(`Failed to highlight ${lang} code block:`, error);
      }
      // Keep original block on error
    }
  }

  // Extract language from class name
  function extractLanguageFromClass(className) {
    if (!className) return null;
    const match = className.match(/language-(\w+)/);
    return match ? match[1] : null;
  }

  // Change theme dynamically
  async function setTheme(newTheme) {
    currentTheme.value = newTheme;

    // Regenerate styles for new theme
    try {
      const styles = await generateHighlightStyles(newTheme);

      // Apply styles to document
      let styleElement = document.getElementById('shiki-theme-styles');
      if (!styleElement) {
        styleElement = document.createElement('style');
        styleElement.id = 'shiki-theme-styles';
        document.head.appendChild(styleElement);
      }
      styleElement.textContent = styles;

      // Re-highlight all visible code blocks
      const containers = document.querySelectorAll('[data-markdown-id]');
      for (const container of containers) {
        const id = container.getAttribute('data-markdown-id');
        await applyShikiHighlighting(id);
      }
    } catch (err) {
      console.error('Failed to switch theme:', err);
    }
  }

  // Update dynamic styles
  function updateDynamicStyles(styles) {
    let styleEl = document.getElementById('shiki-markdown-styles');
    if (!styleEl) {
      styleEl = document.createElement('style');
      styleEl.id = 'shiki-markdown-styles';
      document.head.appendChild(styleEl);
    }
    styleEl.textContent = styles;
  }

  // Override the render method
  markdownRenderer.render = renderWithShiki;

  return {
    // All original markdown renderer methods and state
    ...markdownRenderer,

    // Additional Shiki-specific state and methods
    isHighlighting,
    highlightError,
    currentTheme,
    setTheme,
    applyShikiHighlighting,

    // New Shiki-specific state and methods
    initialize,
    generateHighlightStyles,
    extractLanguageFromClass
  };
}

// Export a singleton version for easy use
let sharedInstance = null;

export function getSharedShikiMarkdown(options = {}) {
  if (!sharedInstance) {
    sharedInstance = useShikiMarkdown(options);
  }
  return sharedInstance;
}