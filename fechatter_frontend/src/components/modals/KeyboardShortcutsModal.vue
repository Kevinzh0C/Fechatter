<template>
  <div v-if="modelValue" class="keyboard-shortcuts-modal-overlay" @click="closeModal">
    <div class="keyboard-shortcuts-modal" @click.stop>
      <!-- Header -->
      <div class="modal-header">
        <h2 class="modal-title">
          <svg class="header-icon" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <path d="M20 5H4c-1.1 0-1.99.9-1.99 2L2 17c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm-9 3h2v2h-2V8zm0 3h2v2h-2v-2zM8 8h2v2H8V8zm0 3h2v2H8v-2zm-1 2H5v-2h2v2zm0-3H5V8h2v2zm3 3h2v2h-2v-2zm0 3h2v2h-2v-2zm-3 0h2v2H8v-2zm0-3H5v-2h2v2zm11 3h-2v-2h2v2zm0-3h-2v-2h2v2zm0-3h-2V8h2v2zm-3 6h-2v-2h2v2z"/>
          </svg>
          Keyboard Shortcuts
        </h2>
        <button @click="closeModal" class="close-button" title="Close (Esc)">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="modal-content">
        <!-- Search filter -->
        <div class="search-section">
          <div class="search-input-container">
            <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
            </svg>
            <input 
              v-model="searchQuery" 
              type="text" 
              placeholder="Search shortcuts..." 
              class="search-input"
              @keydown.escape="searchQuery = ''"
            >
            <button v-if="searchQuery" @click="searchQuery = ''" class="clear-search">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
              </svg>
            </button>
          </div>
        </div>

        <!-- Shortcuts organized by category -->
        <div class="shortcuts-container">
          <div v-for="category in filteredCategories" :key="category.name" class="shortcut-category">
            <h3 class="category-title">
              <span class="category-icon">{{ category.icon }}</span>
              {{ category.name }}
              <span class="category-count">({{ category.shortcuts.length }})</span>
            </h3>
            
            <div class="shortcuts-grid">
              <div v-for="shortcut in category.shortcuts" :key="shortcut.key" 
                   class="shortcut-item" 
                   :class="{ 'highlighted': isHighlighted(shortcut) }">
                <div class="shortcut-keys">
                  <kbd v-for="(key, index) in formatShortcutKeys(shortcut.key)" 
                       :key="index" 
                       class="key">{{ key }}</kbd>
                </div>
                <div class="shortcut-description">
                  {{ shortcut.description }}
                </div>
              </div>
            </div>
          </div>

          <!-- No results state -->
          <div v-if="filteredCategories.length === 0" class="no-results">
            <svg class="no-results-icon" width="48" height="48" viewBox="0 0 24 24" fill="currentColor">
              <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
            </svg>
            <p>No shortcuts found for "{{ searchQuery }}"</p>
            <button @click="searchQuery = ''" class="clear-search-button">Clear search</button>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="modal-footer">
        <div class="footer-info">
          <span class="platform-note">
            <strong>{{ platformName }}</strong> shortcuts shown
          </span>
          <span class="tip">
            Tip: Most shortcuts work across all views
          </span>
        </div>
        <div class="footer-actions">
          <button @click="printShortcuts" class="action-button secondary">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 8H5c-1.66 0-3 1.34-3 3v6h4v4h12v-4h4v-6c0-1.66-1.34-3-3-3zm-3 11H8v-5h8v5zm3-7c-.55 0-1-.45-1-1s.45-1 1-1 1 .45 1 1-.45 1-1 1zm-1-9H6v4h12V3z"/>
            </svg>
            Print
          </button>
          <button @click="closeModal" class="action-button primary">Close</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';

const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false
  },
  shortcuts: {
    type: Array,
    default: () => []
  }
});

const emit = defineEmits(['update:modelValue']);

// State
const searchQuery = ref('');

// Platform detection
const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
const platformName = computed(() => isMac ? 'macOS' : 'Windows/Linux');

// Organize shortcuts by category
const categories = computed(() => [
  {
    name: 'Navigation',
    icon: '🧭',
    keywords: ['home', 'back', 'forward', 'chat', 'navigate'],
    shortcuts: props.shortcuts.filter(s => 
      s.action?.includes('go') || 
      s.action?.includes('navigate') || 
      s.action?.includes('Chat') ||
      s.key?.includes('h') ||
      ['ctrl+h', 'alt+left', 'alt+right', 'alt+up', 'alt+down'].some(k => s.key?.includes(k.split('+').pop()))
    )
  },
  {
    name: 'Search',
    icon: '',
    keywords: ['search', 'find'],
    shortcuts: props.shortcuts.filter(s => 
      s.action?.includes('search') || s.action?.includes('Search') || s.description?.toLowerCase().includes('search')
    )
  },
  {
    name: 'Chat Actions',
    icon: '💬',
    keywords: ['message', 'chat', 'send', 'channel', 'dm'],
    shortcuts: props.shortcuts.filter(s => 
      s.action?.includes('Chat') || 
      s.action?.includes('message') || 
      s.action?.includes('channel') || 
      s.action?.includes('DM') ||
      s.description?.toLowerCase().includes('chat') ||
      s.description?.toLowerCase().includes('message')
    )
  },
  {
    name: 'Quick Actions',
    icon: '',
    keywords: ['create', 'new', 'toggle', 'quick'],
    shortcuts: props.shortcuts.filter(s => 
      s.action?.includes('new') || 
      s.action?.includes('create') || 
      s.action?.includes('toggle') ||
      s.description?.toLowerCase().includes('create') ||
      s.description?.toLowerCase().includes('new')
    )
  },
  {
    name: 'View Controls',
    icon: '👁️',
    keywords: ['view', 'sidebar', 'theme', 'toggle'],
    shortcuts: props.shortcuts.filter(s => 
      s.action?.includes('toggle') && !s.action?.includes('Chat') ||
      s.description?.toLowerCase().includes('sidebar') ||
      s.description?.toLowerCase().includes('theme') ||
      s.description?.toLowerCase().includes('view')
    )
  },
  {
    name: 'System',
    icon: '⚙️',
    keywords: ['settings', 'help', 'reload', 'debug'],
    shortcuts: props.shortcuts.filter(s => 
      s.action?.includes('settings') || 
      s.action?.includes('help') || 
      s.action?.includes('reload') ||
      s.action?.includes('debug') ||
      s.description?.toLowerCase().includes('settings') ||
      s.description?.toLowerCase().includes('help')
    )
  }
]);

// Filter categories based on search
const filteredCategories = computed(() => {
  if (!searchQuery.value) return categories.value;
  
  const query = searchQuery.value.toLowerCase();
  
  return categories.value
    .map(category => ({
      ...category,
      shortcuts: category.shortcuts.filter(shortcut => 
        shortcut.key?.toLowerCase().includes(query) ||
        shortcut.description?.toLowerCase().includes(query) ||
        shortcut.action?.toLowerCase().includes(query) ||
        category.keywords.some(keyword => keyword.includes(query))
      )
    }))
    .filter(category => category.shortcuts.length > 0);
});

// Format shortcut keys for display
const formatShortcutKeys = (keyString) => {
  if (!keyString) return [];
  
  return keyString.split('+').map(key => {
    // Capitalize and format keys nicely
    switch (key.toLowerCase()) {
      case 'ctrl': return isMac ? '⌃' : 'Ctrl';
      case 'cmd': return '⌘';
      case 'shift': return isMac ? '⇧' : 'Shift';
      case 'alt': return isMac ? '⌥' : 'Alt';
      case 'enter': return '↵';
      case 'escape': return 'Esc';
      case 'space': return 'Space';
      case 'up': return '↑';
      case 'down': return '↓';
      case 'left': return '←';
      case 'right': return '→';
      case 'backspace': return '⌫';
      case 'delete': return 'Del';
      case 'tab': return 'Tab';
      default: return key.toUpperCase();
    }
  });
};

// Check if shortcut should be highlighted (recently used)
const isHighlighted = (shortcut) => {
  // This could be enhanced to highlight recently used shortcuts
  return false;
};

// Close modal
const closeModal = () => {
  emit('update:modelValue', false);
};

// Print shortcuts
const printShortcuts = () => {
  const printContent = generatePrintContent();
  const printWindow = window.open('', '_blank');
  printWindow.document.write(printContent);
  printWindow.document.close();
  printWindow.print();
};

// Generate print-friendly content
const generatePrintContent = () => {
  const shortcuts = categories.value.flatMap(cat => 
    cat.shortcuts.map(s => `${s.key} - ${s.description}`)
  ).join('\n');
  
  return `
    <html>
      <head>
        <title>Fechatter Keyboard Shortcuts</title>
        <style>
          body { font-family: Arial, sans-serif; margin: 20px; }
          h1 { color: #333; border-bottom: 2px solid #333; }
          .shortcut { margin: 5px 0; }
          .key { background: #f0f0f0; padding: 2px 6px; border-radius: 3px; font-family: monospace; }
        </style>
      </head>
      <body>
        <h1>Fechatter Keyboard Shortcuts</h1>
        <p>Platform: ${platformName.value}</p>
        <div>
          ${categories.value.map(cat => `
            <h2>${cat.icon} ${cat.name}</h2>
            ${cat.shortcuts.map(s => `
              <div class="shortcut">
                <span class="key">${s.key}</span> - ${s.description}
              </div>
            `).join('')}
          `).join('')}
        </div>
      </body>
    </html>
  `;
};

// Keyboard event handling
const handleKeyDown = (event) => {
  if (event.key === 'Escape') {
    closeModal();
  }
};

// Lifecycle
onMounted(() => {
  document.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown);
});
</script>

<style scoped>
.keyboard-shortcuts-modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
  backdrop-filter: blur(4px);
}

.keyboard-shortcuts-modal {
  background: white;
  border-radius: 12px;
  width: 100%;
  max-width: 800px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.2);
  animation: modalSlideIn 0.3s ease-out;
}

@keyframes modalSlideIn {
  from {
    opacity: 0;
    transform: scale(0.9) translateY(-20px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

/* Header */
.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: 1px solid #e5e7eb;
  background: #f9fafb;
}

.modal-title {
  font-size: 20px;
  font-weight: 600;
  color: #111827;
  margin: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-icon {
  color: #6b7280;
}

.close-button {
  background: none;
  border: none;
  color: #6b7280;
  cursor: pointer;
  padding: 8px;
  border-radius: 6px;
  transition: all 0.2s;
}

.close-button:hover {
  background: #e5e7eb;
  color: #374151;
}

/* Content */
.modal-content {
  flex: 1;
  overflow-y: auto;
  padding: 0;
}

/* Search */
.search-section {
  padding: 20px 24px;
  border-bottom: 1px solid #e5e7eb;
  background: #f9fafb;
}

.search-input-container {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 12px;
  color: #9ca3af;
}

.search-input {
  width: 100%;
  padding: 10px 12px 10px 40px;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  font-size: 14px;
  background: white;
  transition: border-color 0.2s;
}

.search-input:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.clear-search {
  position: absolute;
  right: 8px;
  background: none;
  border: none;
  color: #9ca3af;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  transition: color 0.2s;
}

.clear-search:hover {
  color: #6b7280;
}

/* Shortcuts */
.shortcuts-container {
  padding: 24px;
}

.shortcut-category {
  margin-bottom: 32px;
}

.shortcut-category:last-child {
  margin-bottom: 0;
}

.category-title {
  font-size: 16px;
  font-weight: 600;
  color: #111827;
  margin: 0 0 16px 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.category-icon {
  font-size: 18px;
}

.category-count {
  font-size: 12px;
  font-weight: 400;
  color: #6b7280;
  background: #f3f4f6;
  padding: 2px 6px;
  border-radius: 10px;
}

.shortcuts-grid {
  display: grid;
  gap: 8px;
}

.shortcut-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: #f9fafb;
  border-radius: 8px;
  transition: all 0.2s;
}

.shortcut-item:hover {
  background: #f3f4f6;
}

.shortcut-item.highlighted {
  background: #eff6ff;
  border: 1px solid #bfdbfe;
}

.shortcut-keys {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
  margin-right: 16px;
}

.key {
  background: white;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  padding: 4px 8px;
  font-family: 'SF Mono', Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
  font-size: 12px;
  font-weight: 500;
  color: #374151;
  min-width: 24px;
  text-align: center;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

.shortcut-description {
  flex: 1;
  font-size: 14px;
  color: #374151;
}

/* No results */
.no-results {
  text-align: center;
  padding: 40px 20px;
  color: #6b7280;
}

.no-results-icon {
  margin: 0 auto 16px;
  opacity: 0.5;
}

.no-results p {
  font-size: 16px;
  margin: 0 0 16px 0;
}

.clear-search-button {
  background: #3b82f6;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
  transition: background 0.2s;
}

.clear-search-button:hover {
  background: #2563eb;
}

/* Footer */
.modal-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-top: 1px solid #e5e7eb;
  background: #f9fafb;
}

.footer-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.platform-note {
  font-size: 12px;
  color: #6b7280;
}

.tip {
  font-size: 12px;
  color: #059669;
}

.footer-actions {
  display: flex;
  gap: 12px;
}

.action-button {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  gap: 6px;
}

.action-button.primary {
  background: #3b82f6;
  color: white;
  border: none;
}

.action-button.primary:hover {
  background: #2563eb;
}

.action-button.secondary {
  background: white;
  color: #374151;
  border: 1px solid #d1d5db;
}

.action-button.secondary:hover {
  background: #f9fafb;
}

/* Mobile responsiveness */
@media (max-width: 768px) {
  .keyboard-shortcuts-modal-overlay {
    padding: 16px;
  }

  .keyboard-shortcuts-modal {
    max-height: 95vh;
  }

  .modal-header,
  .search-section,
  .shortcuts-container,
  .modal-footer {
    padding-left: 16px;
    padding-right: 16px;
  }

  .modal-title {
    font-size: 18px;
  }

  .shortcut-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }

  .shortcut-keys {
    margin-right: 0;
  }

  .footer-info {
    display: none;
  }

  .action-button {
    flex: 1;
    justify-content: center;
  }
}

@media (max-width: 480px) {
  .shortcuts-grid {
    gap: 4px;
  }

  .shortcut-item {
    padding: 8px 12px;
  }

  .key {
    padding: 2px 6px;
    font-size: 11px;
  }

  .shortcut-description {
    font-size: 13px;
  }
}
</style>