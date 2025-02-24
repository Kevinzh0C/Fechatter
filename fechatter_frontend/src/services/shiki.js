/**
 * XSS-Safe Code Highlighting Service
 * Provides secure syntax highlighting with DOMPurify sanitization
 */

import { createHighlighter } from 'shiki';

class SecureShikiService {
  constructor() {
    this.initPromise = null;
    this.highlighter = null;
    this.isReady = false;
    this.defaultLang = 'javascript';
    this.defaultTheme = 'github-dark';

    // Import DOMPurify for HTML sanitization
    this.purify = null;
    this.initializePurify();
  }

  /**
   * Initialize DOMPurify for HTML sanitization
   */
  async initializePurify() {
    try {
      const DOMPurify = await import('dompurify');
      this.purify = DOMPurify.default;

      // Configure DOMPurify for code highlighting
      this.purifyConfig = {
        ALLOWED_TAGS: [
          'span', 'div', 'pre', 'code', 'br'
        ],
        ALLOWED_ATTR: [
          'class', 'style', 'data-*'
        ],
        ALLOWED_URI_REGEXP: /^$/,
        KEEP_CONTENT: true,
        FORBID_CONTENTS: ['script', 'style'],
        CUSTOM_ELEMENT_HANDLING: {
          tagNameCheck: () => false,
          attributeNameCheck: () => false,
          allowCustomizedBuiltInElements: false
        }
      };

      // Remove console.log in production
      if (import.meta.env.DEV) {
        console.log('🧼 DOMPurify initialized for secure code highlighting');
      }
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ Failed to initialize DOMPurify:', error);
      }
      // Fallback to text-only highlighting
      this.purify = null;
    }
  }

  /**
   * Initialize the Shiki highlighter, ensuring it's a singleton.
   */
  async initialize() {
    if (this.initPromise) {
      return this.initPromise;
    }

    this.initPromise = createHighlighter({
      themes: [this.defaultTheme, 'github-light'],
      langs: [
        'javascript',
        'typescript',
        'json',
        'html',
        'css',
        'python',
        'rust',
        'sql',
        'bash',
        'markdown',
        'yaml',
        'plaintext'
      ]
    }).then(h => {
      this.highlighter = h;
      this.isReady = true;
      return this; // Return the service instance, not the highlighter
    }).catch(error => {
      if (import.meta.env.DEV) {
        console.error('❌ Failed to initialize Shiki:', error);
      }
      this.initPromise = null; // Allow retries if initialization fails
      throw error;
    });

    return this.initPromise;
  }

  /**
   * Sanitize input code before highlighting
   */
  sanitizeInput(code) {
    if (!code || typeof code !== 'string') {
      return '';
    }

    // Remove potentially dangerous content
    let sanitized = code
      // Remove script tags completely
      .replace(/<script[\s\S]*?<\/script>/gi, '')
      // Remove style tags
      .replace(/<style[\s\S]*?<\/style>/gi, '')
      // Remove event handlers
      .replace(/on\w+\s*=\s*["'][^"']*["']/gi, '')
      // Remove javascript: URLs
      .replace(/javascript:/gi, '')
      // Remove data: URLs with scripts
      .replace(/data:text\/html/gi, '')
      // Limit length to prevent DoS
      .substring(0, 10000);

    return sanitized;
  }

  /**
   * Sanitize HTML output from Shiki
   */
  sanitizeOutput(html) {
    if (!this.purify) {
      // Fallback: escape HTML if DOMPurify not available
      return this.escapeHtml(html);
    }

    try {
      const sanitized = this.purify.sanitize(html, this.purifyConfig);
      return sanitized;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ DOMPurify sanitization failed:', error);
      }
      return this.escapeHtml(html);
    }
  }

  /**
   * Fallback HTML escaping
   */
  escapeHtml(unsafe) {
    return unsafe
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#039;");
  }

  /**
   * Detect programming language from code content
   */
  detectLanguage(code) {
    if (!code) return this.defaultLang;

    const patterns = {
      javascript: [/console\.|function\s+\w+/, /const\s+\w+\s*=/, /=>\s*{/],
      typescript: [/interface\s+\w+/, /type\s+\w+\s*=/, /:\s*string\|number/],
      python: [/def\s+\w+\(/, /import\s+\w+/, /print\(/],
      rust: [/fn\s+\w+\(/, /let\s+mut/, /impl\s+\w+/],
      sql: [/SELECT\s+.*FROM/i, /INSERT\s+INTO/i, /UPDATE\s+.*SET/i],
      json: [/^\s*[\{\[]/, /"\w+":\s*["\d\{\[]/, /^\s*\{[\s\S]*\}\s*$/],
      html: [/<\w+.*?>/, /<\/\w+>/, /<!DOCTYPE/i],
      css: [/\w+\s*\{[\s\S]*\}/, /\.\w+\s*\{/, /@media/],
      bash: [/#!/, /\$\w+/, /^\s*#/m],
      yaml: [/^---/, /^\w+:/, /^- /m]
    };

    for (const [lang, langPatterns] of Object.entries(patterns)) {
      if (langPatterns.some(pattern => pattern.test(code))) {
        return lang;
      }
    }

    return this.defaultLang;
  }

  /**
   * Highlight code using the actual Shiki highlighter instance
   */
  async codeToHtml(code, options = {}) {
    await this.initialize();

    if (!this.isReady || !this.highlighter) {
      throw new Error('Shiki highlighter not ready');
    }

    // Sanitize input
    const sanitizedCode = this.sanitizeInput(code);
    if (!sanitizedCode.trim()) {
      return '<pre><code></code></pre>';
    }

    // Extract language and theme from options
    const lang = options.lang || options.language || this.detectLanguage(sanitizedCode);
    const theme = options.theme || this.defaultTheme;

    try {
      // Generate highlighted HTML using the actual highlighter
      const highlightedHtml = this.highlighter.codeToHtml(sanitizedCode, {
        lang,
        theme
      });

      // Sanitize the output HTML
      const safeHtml = this.sanitizeOutput(highlightedHtml);

      return safeHtml;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ Code highlighting failed:', error);
      }

      // Fallback to escaped text
      const escaped = this.escapeHtml(sanitizedCode);
      return `<pre><code class="language-${lang}">${escaped}</code></pre>`;
    }
  }

  /**
   * Legacy highlight method for backward compatibility
   */
  async highlight(code, language = null, theme = null) {
    return this.codeToHtml(code, { lang: language, theme });
  }

  /**
   * Get available languages
   */
  getAvailableLanguages() {
    if (!this.isReady || !this.highlighter) return [];
    return this.highlighter.getLoadedLanguages();
  }

  /**
   * Get available themes
   */
  getAvailableThemes() {
    if (!this.isReady || !this.highlighter) return [];
    return this.highlighter.getLoadedThemes();
  }

  /**
   * Clean up resources
   */
  dispose() {
    if (this.highlighter && typeof this.highlighter.dispose === 'function') {
      this.highlighter.dispose();
    }
    this.highlighter = null;
    this.isReady = false;
    this.initPromise = null;
  }
}

// Create singleton instance
const secureShiki = new SecureShikiService();

// Export secure wrapper function for backward compatibility
export async function getHighlighter(options = {}) {
  await secureShiki.initialize();
  return secureShiki;
}

// Export getShiki for backward compatibility
export async function getShiki() {
  await secureShiki.initialize();
  return secureShiki;
}

// Export the service and secure highlighting function
export { createHighlighter };
export default secureShiki; 