/**
 * 🎯 Unified Markdown Engine
 * 
 * 统一的 Markdown 处理引擎，集成：
 * - Shiki 代码高亮
 * - KaTeX 数学公式渲染
 * - 标准 Markdown 格式化
 * - 实时预览同步
 */

import { unified } from 'unified';
import remarkParse from 'remark-parse';
import remarkGfm from 'remark-gfm';
import remarkBreaks from 'remark-breaks';
import remarkMath from 'remark-math';
import remarkRehype from 'remark-rehype';
import rehypeKatex from 'rehype-katex';
import rehypeStringify from 'rehype-stringify';
import rehypeSanitize from 'rehype-sanitize';
// import { createHighlighter } from 'shiki';
// Temporarily disable shiki for build
const createHighlighter = () => Promise.resolve({
  codeToHtml: (code, options) => `<pre><code class="language-${options?.lang || 'text'}">${code}</code></pre>`
});
import katex from 'katex';

export class UnifiedMarkdownEngine {
  constructor() {
    this.highlighter = null;
    this.processor = null;
    this.isInitialized = false;
    this.renderCache = new Map();
    this.previewCache = new Map();

    // Performance metrics
    this.metrics = {
      renderTime: 0,
      cacheHits: 0,
      cacheMisses: 0
    };

    // Configuration
    this.config = {
      theme: 'dark',
      lineNumbers: true,
      mathDisplay: true,
      enableCache: true,
      maxCacheSize: 100
    };
  }

  /**
   * 初始化引擎
   */
  async initialize() {
    if (this.isInitialized) return;

    try {
      // 初始化 Shiki 高亮器
      this.highlighter = await createHighlighter({
        themes: ['vitesse-light', 'one-dark-pro'],
        langs: [
          'javascript', 'typescript', 'jsx', 'tsx', 'vue',
          'html', 'css', 'scss', 'json', 'yaml', 'markdown',
          'bash', 'shell', 'python', 'rust', 'go', 'java',
          'cpp', 'c', 'sql', 'docker', 'xml', 'toml'
        ]
      });

      // 创建统一的 Markdown 处理器
      this.processor = unified()
        .use(remarkParse)
        .use(remarkGfm)
        .use(remarkBreaks)
        .use(remarkMath) // 数学公式支持
        .use(remarkRehype, { allowDangerousHtml: false })
        .use(rehypeKatex, { // KaTeX 数学公式渲染
          displayMode: false,
          throwOnError: false,
          errorColor: '#cc0000'
        })
        .use(this.createShikiPlugin()) // 自定义 Shiki 插件
        .use(rehypeSanitize, this.createSanitizeSchema())
        .use(rehypeStringify);

      this.isInitialized = true;
      console.log('✅ UnifiedMarkdownEngine initialized');
    } catch (error) {
      console.error('❌ Failed to initialize UnifiedMarkdownEngine:', error);
      throw error;
    }
  }

  /**
   * 渲染 Markdown 内容（用于消息显示）
   */
  async renderMarkdown(content, options = {}) {
    await this.initialize();

    const {
      cache = this.config.enableCache,
      theme = this.config.theme,
      lineNumbers = this.config.lineNumbers
    } = options;

    const cacheKey = this.generateCacheKey(content, { theme, lineNumbers });

    // 检查缓存
    if (cache && this.renderCache.has(cacheKey)) {
      this.metrics.cacheHits++;
      return this.renderCache.get(cacheKey);
    }

    const startTime = Date.now();

    try {
      // 更新配置
      this.updateConfig({ theme, lineNumbers });

      // 处理 Markdown
      const result = await this.processor.process(content);
      const html = String(result);

      // 缓存结果
      if (cache) {
        this.setCache(this.renderCache, cacheKey, html);
        this.metrics.cacheMisses++;
      }

      this.metrics.renderTime = Date.now() - startTime;
      return html;

    } catch (error) {
      console.error('❌ Markdown rendering failed:', error);
      // 返回安全的纯文本
      return `<p>${this.escapeHtml(content)}</p>`;
    }
  }

  /**
   * 渲染预览（优化版本，用于实时预览）
   */
  async renderPreview(content, options = {}) {
    await this.initialize();

    const {
      cache = true,
      theme = this.config.theme,
      debounce = 150
    } = options;

    const cacheKey = this.generateCacheKey(content, { theme, preview: true });

    // 检查预览缓存
    if (cache && this.previewCache.has(cacheKey)) {
      return this.previewCache.get(cacheKey);
    }

    try {
      // 对于预览，使用轻量级处理
      const html = await this.renderMarkdown(content, { ...options, cache: false });

      // 缓存预览结果
      if (cache) {
        this.setCache(this.previewCache, cacheKey, html);
      }

      return html;

    } catch (error) {
      console.error('❌ Preview rendering failed:', error);
      return `<p>${this.escapeHtml(content)}</p>`;
    }
  }

  /**
   * 创建自定义 Shiki 插件
   */
  createShikiPlugin() {
    const engine = this;

    return function rehypeShiki() {
      return async function transformer(tree) {
        const { visit } = await import('unist-util-visit');
        const promises = [];

        visit(tree, 'element', (node) => {
          if (node.tagName === 'pre' && node.children?.[0]?.tagName === 'code') {
            const codeNode = node.children[0];
            const lang = engine.extractLanguage(codeNode);
            const code = engine.extractCode(codeNode);

            if (code && lang) {
              promises.push(
                engine.highlightCodeBlock(node, codeNode, code, lang, engine.config)
              );
            }
          }
        });

        await Promise.all(promises);
      };
    };
  }

  /**
   * 高亮代码块
   */
  async highlightCodeBlock(preNode, codeNode, code, lang, config) {
    try {
      const theme = config.theme === 'dark' ? 'one-dark-pro' : 'vitesse-light';

      const html = this.highlighter.codeToHtml(code, {
        lang: this.resolveLanguage(lang),
        theme: theme
      });

      // 解析生成的 HTML 并替换原节点
      const { parse } = await import('parse5');
      const { fromParse5 } = await import('hast-util-from-parse5');

      const parsed = parse(html);
      const hast = fromParse5(parsed);
      const preElement = hast.children.find(child =>
        child.type === 'element' && child.tagName === 'pre'
      );

      if (preElement) {
        // 添加额外的类和属性
        preElement.properties = {
          ...preElement.properties,
          className: [
            ...(preElement.properties.className || []),
            'unified-code-block',
            `lang-${lang}`
          ],
          'data-lang': lang,
          'data-theme': config.theme
        };

        // 替换原节点
        Object.assign(preNode, preElement);
      }

    } catch (error) {
      console.warn(`Code highlighting failed for ${lang}:`, error);
      // 保持原样但添加基本样式
      preNode.properties = {
        ...preNode.properties,
        className: ['unified-code-block', 'fallback', `lang-${lang}`],
        'data-lang': lang
      };
    }
  }

  /**
   * 提取代码语言
   */
  extractLanguage(codeNode) {
    const className = codeNode.properties?.className || [];
    const langClass = className.find(cls => cls.startsWith('language-'));
    return langClass ? langClass.replace('language-', '') : 'plaintext';
  }

  /**
   * 提取代码内容
   */
  extractCode(codeNode) {
    if (!codeNode.children) return '';
    return codeNode.children
      .filter(child => child.type === 'text')
      .map(child => child.value)
      .join('');
  }

  /**
   * 解析语言别名
   */
  resolveLanguage(lang) {
    const aliases = {
      'js': 'javascript',
      'ts': 'typescript',
      'jsx': 'javascript',
      'tsx': 'typescript',
      'yml': 'yaml',
      'md': 'markdown',
      'sh': 'bash',
      'py': 'python',
      'rs': 'rust'
    };
    return aliases[lang] || lang;
  }

  /**
   * 创建安全化模式
   */
  createSanitizeSchema() {
    return {
      tagNames: [
        'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
        'p', 'br', 'strong', 'em', 'del', 'ins',
        'ul', 'ol', 'li', 'blockquote',
        'pre', 'code', 'span',
        'a', 'img',
        'table', 'thead', 'tbody', 'tr', 'th', 'td',
        'div', // 用于 KaTeX
        'annotation', 'math', 'mrow', 'mi', 'mn', 'mo' // KaTeX 元素
      ],
      attributes: {
        '*': ['className', 'id', 'style', 'data*'],
        'a': ['href', 'title', 'target', 'rel'],
        'img': ['src', 'alt', 'title', 'width', 'height'],
        'pre': ['data-lang', 'data-theme'],
        'code': ['data-lang'],
        'div': ['title'] // KaTeX
      },
      protocols: {
        'href': ['http', 'https', 'mailto'],
        'src': ['http', 'https', 'data']
      }
    };
  }

  /**
   * 更新配置
   */
  updateConfig(newConfig) {
    Object.assign(this.config, newConfig);
  }

  /**
   * 生成缓存键
   */
  generateCacheKey(content, options = {}) {
    const optionsStr = JSON.stringify(options);
    return `${this.hashString(content)}_${this.hashString(optionsStr)}`;
  }

  /**
   * 简单哈希函数
   */
  hashString(str) {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // 转为32位整数
    }
    return hash.toString(36);
  }

  /**
   * 设置缓存（带大小限制）
   */
  setCache(cache, key, value) {
    if (cache.size >= this.config.maxCacheSize) {
      // 删除最旧的条目
      const firstKey = cache.keys().next().value;
      cache.delete(firstKey);
    }
    cache.set(key, value);
  }

  /**
   * HTML 转义
   */
  escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  /**
   * 清理缓存
   */
  clearCache() {
    this.renderCache.clear();
    this.previewCache.clear();
    this.metrics.cacheHits = 0;
    this.metrics.cacheMisses = 0;
  }

  /**
   * 获取性能指标
   */
  getMetrics() {
    return {
      ...this.metrics,
      cacheSize: this.renderCache.size + this.previewCache.size,
      cacheHitRate: this.metrics.cacheHits / (this.metrics.cacheHits + this.metrics.cacheMisses) || 0
    };
  }

  /**
   * 销毁引擎
   */
  destroy() {
    this.clearCache();
    this.highlighter = null;
    this.processor = null;
    this.isInitialized = false;
  }
}

// 创建全局实例
export const unifiedMarkdownEngine = new UnifiedMarkdownEngine();

// 默认导出
export default unifiedMarkdownEngine; 