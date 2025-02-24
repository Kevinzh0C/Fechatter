// Simplified Vite plugin for Shiki syntax highlighting
import { getHighlighter } from 'shiki';

// Simple language mapping
const languageMap = {
  js: 'javascript',
  ts: 'typescript',
  jsx: 'javascript',
  tsx: 'typescript',
  yml: 'yaml',
  md: 'markdown',
  sh: 'bash',
  py: 'python',
  rs: 'rust'
};

// Basic languages only
const basicLanguages = [
  'javascript',
  'typescript',
  'vue',
  'html',
  'css',
  'json',
  'yaml',
  'markdown',
  'bash',
  'python',
  'rust',
  'sql',
  'plaintext'
];

let highlighter = null;

async function getShikiHighlighter() {
  if (!highlighter) {
    try {
      highlighter = await getHighlighter({
        theme: 'one-dark-pro',
        langs: basicLanguages
      });
    } catch (error) {
      console.error('Failed to initialize Shiki highlighter:', error);
      return null;
    }
  }
  return highlighter;
}

// Resolve language
function resolveLanguage(lang) {
  if (!lang) return 'plaintext';
  const normalized = lang.toLowerCase().trim();
  return languageMap[normalized] || normalized;
}

// Simple vite plugin
export default function viteShikiSimple(options = {}) {
  return {
    name: 'vite-plugin-shiki-simple',
    enforce: 'pre',

    async buildStart() {
      // Initialize highlighter at build start
      await getShikiHighlighter();
    },

    async transform(code, id) {
      // Only process .vue files that contain code blocks
      if (!id.endsWith('.vue') || !code.includes('```')) {
        return null;
      }

      // Simple processing - just mark for runtime highlighting
      const processed = code.replace(
        /```(\w+)?\n([\s\S]*?)```/g,
        (match, lang, content) => {
          const resolvedLang = resolveLanguage(lang);
          return `<pre class="shiki-code" data-lang="${resolvedLang}"><code>${content.trim()}</code></pre>`;
        }
      );

      return processed !== code ? { code: processed } : null;
    }
  };
} 