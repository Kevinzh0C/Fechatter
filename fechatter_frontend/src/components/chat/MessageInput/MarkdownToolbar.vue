<template>
  <div class="markdown-toolbar">
    <!-- Text Formatting -->
    <div class="toolbar-group">
      <button @click="$emit('insert-markdown', '**', '**')" class="toolbar-btn" title="Bold (Ctrl+B)">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M6 4h8a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6zM6 12h9a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z"></path>
        </svg>
      </button>
      <button @click="$emit('insert-markdown', '*', '*')" class="toolbar-btn" title="Italic (Ctrl+I)">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="19" y1="4" x2="10" y2="4"></line>
          <line x1="14" y1="20" x2="5" y2="20"></line>
          <line x1="15" y1="4" x2="9" y2="20"></line>
        </svg>
      </button>
      <button @click="$emit('insert-markdown', '~~', '~~')" class="toolbar-btn" title="Strikethrough">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M16 4H9a3 3 0 0 0-2.83 4M14 12a4 4 0 0 1 0 8H6m-2-6h16"></path>
        </svg>
      </button>
      <button @click="$emit('insert-markdown', '==', '==')" class="toolbar-btn" title="Highlight">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M9 11H6a2 2 0 0 0-2 2v3c0 1.1.9 2 2 2h13c1.1 0 2-.9 2-2v-3a2 2 0 0 0-2-2h-3"></path>
          <path d="M9 7l1.5-1.5L12 7l1.5-1.5L15 7v4H9V7z"></path>
        </svg>
      </button>
      <button @click="$emit('insert-markdown', '`', '`')" class="toolbar-btn" title="Inline Code">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="16,18 22,12 16,6"></polyline>
          <polyline points="8,6 2,12 8,18"></polyline>
        </svg>
      </button>
    </div>

    <!-- Headers -->
    <div class="toolbar-group">
      <button @click="$emit('insert-markdown', '# ', '')" class="toolbar-btn" title="Header 1">H1</button>
      <button @click="$emit('insert-markdown', '## ', '')" class="toolbar-btn" title="Header 2">H2</button>
      <button @click="$emit('insert-markdown', '### ', '')" class="toolbar-btn" title="Header 3">H3</button>
    </div>

    <!-- Lists -->
    <div class="toolbar-group">
      <button @click="$emit('insert-markdown', '- ', '')" class="toolbar-btn" title="Bullet List">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="8" y1="6" x2="21" y2="6"></line>
          <line x1="8" y1="12" x2="21" y2="12"></line>
          <line x1="8" y1="18" x2="21" y2="18"></line>
          <line x1="3" y1="6" x2="3.01" y2="6"></line>
          <line x1="3" y1="12" x2="3.01" y2="12"></line>
          <line x1="3" y1="18" x2="3.01" y2="18"></line>
        </svg>
      </button>
      <button @click="$emit('insert-markdown', '1. ', '')" class="toolbar-btn" title="Numbered List">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="10" y1="6" x2="21" y2="6"></line>
          <line x1="10" y1="12" x2="21" y2="12"></line>
          <line x1="10" y1="18" x2="21" y2="18"></line>
          <path d="M4 6h1v4"></path>
          <path d="M4 10h2"></path>
          <path d="M6 18H4c0-1 2-2 2-3s-1-1.5-2-1"></path>
        </svg>
      </button>
      <button @click="$emit('insert-markdown', '- [ ] ', '')" class="toolbar-btn" title="Task List">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
          <polyline points="9,11 12,14 22,4"></polyline>
        </svg>
      </button>
    </div>

    <!-- Content -->
    <div class="toolbar-group">
      <button @click="$emit('insert-markdown', '> ', '')" class="toolbar-btn" title="Quote">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M3 21c3 0 7-1 7-8V5c0-1.25-.756-2.017-2-2H4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2 1 0 1 0 1 1v1c0 1-1 2-2 2s-1 .008-1 1.031V20c0 1 0 1 1 1z"></path>
          <path d="M15 21c3 0 7-1 7-8V5c0-1.25-.757-2.017-2-2h-4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2h.75c0 2.25.25 4-2.75 4v3c0 1 0 1 1 1z"></path>
        </svg>
      </button>
      <button @click="$emit('insert-code-block')" class="toolbar-btn" title="Code Block">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect>
          <line x1="8" y1="21" x2="16" y2="21"></line>
          <line x1="12" y1="17" x2="12" y2="21"></line>
        </svg>
      </button>
      <button @click="$emit('insert-markdown', '[', '](url)')" class="toolbar-btn" title="Link (Ctrl+K)">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
          <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
        </svg>
      </button>
      <button @click="$emit('insert-markdown', '![alt](', ')')" class="toolbar-btn" title="Image">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
          <circle cx="8.5" cy="8.5" r="1.5"></circle>
          <polyline points="21,15 16,10 5,21"></polyline>
        </svg>
      </button>
      <button @click="$emit('insert-table')" class="toolbar-btn" title="Table">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M9 9h6v6h-6zM21 14V8a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v6M21 14v2a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-2M3 10h18"></path>
        </svg>
      </button>
      <button @click="$emit('insert-markdown', '\n---\n', '')" class="toolbar-btn" title="Horizontal Rule">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="3" y1="12" x2="21" y2="12"></line>
        </svg>
      </button>
    </div>

    <!-- Actions -->
    <div class="toolbar-group actions">
      <button @click="$emit('toggle-preview')" class="toolbar-btn" :class="{ active: showPreview }" title="Toggle Preview">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
          <circle cx="12" cy="12" r="3"></circle>
        </svg>
      </button>
      <button @click="$emit('clear-content')" class="toolbar-btn danger" title="Clear All">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="3,6 5,6 21,6"></polyline>
          <path d="M19,6v14a2,2 0 0,1-2,2H7a2,2 0 0,1-2-2V6m3,0V4a2,2 0 0,1,2-2h4a2,2 0 0,1,2,2v2"></path>
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup>
defineProps({
  showPreview: {
    type: Boolean,
    default: false
  }
});

defineEmits([
  'insert-markdown',
  'insert-code-block',
  'insert-table',
  'toggle-preview',
  'clear-content'
]);
</script>

<style scoped>
.markdown-toolbar {
  display: flex;
  gap: 8px;
  padding: 8px;
  background: var(--color-bg-primary);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  margin-bottom: 8px;
  flex-wrap: wrap;
}

.toolbar-group {
  display: flex;
  gap: 4px;
  padding: 0 4px;
  border-right: 1px solid var(--color-border);
}

.toolbar-group:last-child {
  border-right: none;
}

.toolbar-group.actions {
  margin-left: auto;
}

.toolbar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  border-radius: 6px;
  cursor: pointer;
  color: var(--color-text-secondary);
  transition: all 0.2s;
  font-size: 12px;
  font-weight: 600;
}

.toolbar-btn:hover {
  background: var(--color-bg-secondary);
  color: var(--color-text-primary);
}

.toolbar-btn.active {
  background: var(--color-primary);
  color: white;
}

.toolbar-btn.danger {
  color: var(--color-danger);
}

.toolbar-btn.danger:hover {
  background: var(--color-danger);
  color: white;
}

@media (max-width: 768px) {
  .markdown-toolbar {
    overflow-x: auto;
    flex-wrap: nowrap;
  }
  
  .toolbar-group {
    flex-shrink: 0;
  }
}
</style>
