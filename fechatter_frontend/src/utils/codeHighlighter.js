/**
 * Code Highlighter Singleton
 * Following Frontend Design Principles:
 * - Component State Layer: Singleton pattern for resource management
 * - Build Resource Layer: Minimize instance creation
 */

import { createHighlighter } from 'shiki';

class CodeHighlighterManager {
  constructor() {
    this.highlighter = null;
    this.initPromise = null;
    this.disposed = false;
  }

  /**
   * Get or create highlighter instance
   * YAGNI: Only load themes/languages we actually use
   */
  async getInstance() {
    // If already disposed, create new instance
    if (this.disposed) {
      this.disposed = false;
      this.highlighter = null;
      this.initPromise = null;
    }

    // Return existing instance
    if (this.highlighter) {
      return this.highlighter;
    }

    // Return ongoing initialization
    if (this.initPromise) {
      return this.initPromise;
    }

    // Initialize new instance
    this.initPromise = this.initialize();
    return this.initPromise;
  }

  /**
   * Initialize highlighter with minimal configuration
   */
  async initialize() {
    try {
      this.highlighter = await createHighlighter({
        themes: ['github-light', 'github-dark'], // Only load needed themes
        langs: ['javascript', 'typescript', 'python', 'sql', 'json', 'bash'], // Common languages only
      });

      if (import.meta.env.DEV) {
        console.log('✅ Code highlighter initialized as singleton');
      return this.highlighter;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Failed to initialize code highlighter:', error);
      this.initPromise = null;
      throw error;
    }

  /**
   * Highlight code with caching
   */
  async highlight(code, lang = 'javascript', theme = 'github-light') {
    const highlighter = await this.getInstance();

    // Fallback to text if language not supported
    const supportedLangs = highlighter.getLoadedLanguages();
    const actualLang = supportedLangs.includes(lang) ? lang : 'text';

    return highlighter.codeToHtml(code, {
      lang: actualLang,
      theme
    });

  /**
   * Dispose highlighter instance
   */
  dispose() {
    if (this.highlighter) {
      this.highlighter.dispose();
      this.highlighter = null;
      this.initPromise = null;
      this.disposed = true;
      if (import.meta.env.DEV) {
        console.log('🧹 Code highlighter disposed');
      }

  /**
   * Get current status
   */
  getStatus() {
    return {
      initialized: !!this.highlighter,
      disposed: this.disposed,
      loading: !!this.initPromise && !this.highlighter
    };
  }

// Export singleton instance
const codeHighlighter = new CodeHighlighterManager();

// Auto-cleanup on page unload
if (typeof window !== 'undefined') {
  window.addEventListener('beforeunload', () => {
    codeHighlighter.dispose();
  });

export default codeHighlighter;

/**
 * Vue composable for code highlighting
 */
export function useCodeHighlighter() {
  const highlight = async (code, lang, theme) => {
    return codeHighlighter.highlight(code, lang, theme);
  };

  const dispose = () => {
    codeHighlighter.dispose();
  };

  const status = () => {
    return codeHighlighter.getStatus();
  };

  return {
    highlight,
    dispose,
    status
  };
} 