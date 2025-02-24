<template>
  <div class="enhanced-message-content">
    <template v-for="(segment, index) in contentSegments" :key="`segment-${index}`">
      <!-- Code block -->
      <div v-if="segment.type === 'code'" class="code-block">
        <div v-if="segment.language" class="code-header">
          <span class="lang-tag">{{ segment.language.toUpperCase() }}</span>
        </div>
        <pre class="code-content"><code>{{ segment.content }}</code></pre>
      </div>
      <!-- Inline code -->
      <code v-else-if="segment.type === 'inline-code'" class="inline-code">{{ segment.content }}</code>
      <!-- Regular text (with markdown support) -->
      <div v-else class="text-content" v-html="renderMarkdown(segment.content)"></div>
    </template>
  </div>
</template>

<script setup>
import { computed } from 'vue';
import { marked } from 'marked';
import DOMPurify from 'dompurify';

const props = defineProps({
  content: {
    type: String,
    required: true
  },
  enableMarkdown: {
    type: Boolean,
    default: true
  }
});

// Enhanced regex patterns for code detection
const CODE_BLOCK_REGEX = /```(\w+)?\n?([\s\S]*?)```/g;
const INLINE_CODE_REGEX = /`([^`\n]+)`/g;

// Parse content into segments
const contentSegments = computed(() => {
  if (!props.content?.trim()) return [];

  const segments = [];
  let lastIndex = 0;
  let content = props.content;

  // Find all code blocks first
  const codeBlocks = [];
  let match;

  // Reset regex state
  CODE_BLOCK_REGEX.lastIndex = 0;

  while ((match = CODE_BLOCK_REGEX.exec(content)) !== null) {
    codeBlocks.push({
      start: match.index,
      end: match.index + match[0].length,
      language: match[1] || null,
      content: match[2]?.trim() || '',
      fullMatch: match[0]
    });
  }

  // Sort code blocks by position
  codeBlocks.sort((a, b) => a.start - b.start);

  // Process content segment by segment
  codeBlocks.forEach(codeBlock => {
    // Add text before code block
    if (lastIndex < codeBlock.start) {
      const textSegment = content.slice(lastIndex, codeBlock.start);
      if (textSegment.trim()) {
        // Check for inline code in text segment
        parseInlineCode(textSegment, segments);
      }
    }

    // Add code block
    if (codeBlock.content.trim()) {
      segments.push({
        type: 'code',
        content: codeBlock.content,
        language: codeBlock.language
      });
    }

    lastIndex = codeBlock.end;
  });

  // Add remaining text after last code block
  if (lastIndex < content.length) {
    const remainingText = content.slice(lastIndex);
    if (remainingText.trim()) {
      parseInlineCode(remainingText, segments);
    }
  }

  // If no code blocks found, parse the entire content for inline code
  if (codeBlocks.length === 0) {
    parseInlineCode(content, segments);
  }

  return segments.length > 0 ? segments : [{ type: 'text', content: content }];
});

// Parse inline code within text
function parseInlineCode(text, segments) {
  let lastIndex = 0;
  let match;

  // Reset regex state
  INLINE_CODE_REGEX.lastIndex = 0;

  const inlineCodes = [];
  while ((match = INLINE_CODE_REGEX.exec(text)) !== null) {
    inlineCodes.push({
      start: match.index,
      end: match.index + match[0].length,
      content: match[1]
    });
  }

  if (inlineCodes.length === 0) {
    // No inline code, add as regular text
    segments.push({
      type: 'text',
      content: text
    });
    return;
  }

  inlineCodes.forEach(inlineCode => {
    // Add text before inline code
    if (lastIndex < inlineCode.start) {
      const textPart = text.slice(lastIndex, inlineCode.start);
      if (textPart.trim()) {
        segments.push({
          type: 'text',
          content: textPart
        });
      }
    }

    // Add inline code
    segments.push({
      type: 'inline-code',
      content: inlineCode.content
    });

    lastIndex = inlineCode.end;
  });

  // Add remaining text
  if (lastIndex < text.length) {
    const remainingText = text.slice(lastIndex);
    if (remainingText.trim()) {
      segments.push({
        type: 'text',
        content: remainingText
      });
    }
  }
}

// Render markdown for text segments
function renderMarkdown(text) {
  if (!props.enableMarkdown || !text?.trim()) return text;

  try {
    // Configure marked for security and Discord-like rendering
    const renderer = new marked.Renderer();

    // Override code rendering to prevent conflicts
    renderer.code = (code, language) => {
      // This shouldn't be called since we handle code blocks separately
      return `<pre><code>${code}</code></pre>`;
    };

    renderer.codespan = (code) => {
      // This shouldn't be called since we handle inline code separately
      return `<code>${code}</code>`;
    };

    // Discord-style link rendering
    renderer.link = (href, title, text) => {
      const safeHref = DOMPurify.sanitize(href);
      const safeTitle = title ? DOMPurify.sanitize(title) : '';
      const safeText = DOMPurify.sanitize(text);
      return `<a href="${safeHref}" ${safeTitle ? `title="${safeTitle}"` : ''} target="_blank" rel="noopener noreferrer" class="message-link">${safeText}</a>`;
    };

    // Basic formatting
    renderer.strong = (text) => `<strong class="message-bold">${text}</strong>`;
    renderer.em = (text) => `<em class="message-italic">${text}</em>`;
    renderer.del = (text) => `<del class="message-strikethrough">${text}</del>`;

    // Simple list rendering
    renderer.list = (body, ordered) => {
      const tag = ordered ? 'ol' : 'ul';
      return `<${tag} class="message-list">${body}</${tag}>`;
    };

    renderer.listitem = (text) => `<li class="message-list-item">${text}</li>`;

    // Configure marked options
    marked.setOptions({
      renderer: renderer,
      gfm: true,
      breaks: true,
      sanitize: false, // We'll use DOMPurify instead
      smartLists: true,
      smartypants: false
    });

    const dirty = marked.parse(text);
    return DOMPurify.sanitize(dirty, {
      ALLOWED_TAGS: ['strong', 'em', 'del', 'a', 'br', 'p', 'ul', 'ol', 'li'],
      ALLOWED_ATTR: ['href', 'title', 'target', 'rel', 'class'],
      ALLOW_DATA_ATTR: false
    });
  } catch (error) {
    console.warn('Markdown rendering failed:', error);
    return DOMPurify.sanitize(text);
  }
}
</script>

<style scoped>
.enhanced-message-content {
  word-wrap: break-word;
  white-space: pre-wrap;
  line-height: 1.4;
}

.text-content {
  display: inline;
}

/* Code block styles */
.code-block {
  margin: 8px 0;
  border-radius: 6px;
  background: #f8f9fa;
  border: 1px solid #e1e4e8;
  overflow: hidden;
}

.code-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: #f1f3f4;
  border-bottom: 1px solid #e1e4e8;
}

.lang-tag {
  font-size: 10px;
  font-weight: 600;
  color: #24292f;
  background: #e1e4e8;
  padding: 2px 6px;
  border-radius: 3px;
}

.code-content {
  margin: 0;
  padding: 12px;
  background: #ffffff;
  overflow-x: auto;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 13px;
  line-height: 1.4;
}

.code-content code {
  background: transparent;
  padding: 0;
  border: none;
  font-family: inherit;
}

.inline-code {
  background: #f6f8fa;
  color: #cf222e;
  padding: 3px 6px;
  border-radius: 6px;
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', 'Source Code Pro', monospace;
  font-size: 0.85em;
  font-weight: 600;
  border: 1px solid #d0d7de;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
  transition: all 0.2s ease;
  display: inline-block;
  margin: 0 1px;
  vertical-align: baseline;
}

.inline-code:hover {
  background: #f3f4f6;
  border-color: #8c959f;
  transform: translateY(-0.5px);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

/* Markdown styling */
.enhanced-message-content :deep(.message-bold) {
  font-weight: 700;
}

.enhanced-message-content :deep(.message-italic) {
  font-style: italic;
}

.enhanced-message-content :deep(.message-strikethrough) {
  text-decoration: line-through;
  opacity: 0.7;
}

.enhanced-message-content :deep(.message-link) {
  color: #1264a3;
  text-decoration: none;
  border-bottom: 1px solid transparent;
  transition: border-bottom-color 0.2s ease;
}

.enhanced-message-content :deep(.message-link:hover) {
  border-bottom-color: #1264a3;
}

.enhanced-message-content :deep(.message-list) {
  margin: 8px 0;
  padding-left: 20px;
}

.enhanced-message-content :deep(.message-list-item) {
  margin: 2px 0;
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
  .code-block {
    background: #21262d;
    border-color: #30363d;
  }

  .code-header {
    background: #30363d;
    border-bottom-color: #30363d;
  }

  .lang-tag {
    background: #30363d;
    color: #79c0ff;
  }

  .code-content {
    background: #0d1117;
    color: #e6edf3;
  }

  .inline-code {
    background: #21262d;
    color: #79c0ff;
    border-color: #30363d;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
  }

  .inline-code:hover {
    background: #30363d;
    border-color: #525252;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.4);
  }

  .enhanced-message-content :deep(.message-link) {
    color: #79c0ff;
  }

  .enhanced-message-content :deep(.message-link:hover) {
    border-bottom-color: #79c0ff;
  }

  /* Handle empty content gracefully */
  .enhanced-message-content:empty::before {
    content: "";
    display: block;
  }

  /* Ensure proper spacing between segments */
  .enhanced-message-content>*+* {
    margin-top: 4px;
  }

  /* Handle very long code lines */
  .enhanced-message-content :deep(pre) {
    max-width: 100%;
    overflow-x: auto;
  }
}

/* Handle empty content gracefully */
.enhanced-message-content:empty::before {
  content: "";
  display: block;
}

/* Ensure proper spacing between segments */
.enhanced-message-content>*+* {
  margin-top: 4px;
}

/* Handle very long code lines */
.enhanced-message-content :deep(pre) {
  max-width: 100%;
  overflow-x: auto;
}
</style>