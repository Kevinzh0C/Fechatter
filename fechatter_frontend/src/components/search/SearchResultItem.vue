<template>
  <!-- Search Result Item - Cognitive Design Optimized -->
  <article 
    class="search-result-item"
    :class="{ 'highlighted': isHighlighted }"
    @click="handleNavigate"
    @mouseenter="handleHover"
    role="button"
    tabindex="0"
    @keydown.enter="handleNavigate"
    @keydown.space.prevent="handleNavigate"
  >
    <!-- Result Header -->
    <header class="result-header">
      <div class="sender-info">
        <div class="sender-avatar">
          <img 
            v-if="result.sender_avatar" 
            :src="result.sender_avatar" 
            :alt="result.sender_name"
            class="avatar-image"
          />
          <div v-else class="avatar-fallback">
            {{ getInitials(result.sender_name) }}
          </div>
        </div>
        
        <div class="sender-details">
          <span class="sender-name">{{ result.sender_name || 'Unknown User' }}</span>
          <time class="message-time" :datetime="result.created_at">
            {{ formatTime(result.created_at) }}
          </time>
        </div>
      </div>
      
      <div class="result-meta">
        <div class="relevance-score" v-if="showScore">
          <span class="score-label">{{ Math.round(result.relevance_score * 100) }}%</span>
        </div>
        
        <div class="strategy-badge" v-if="result.strategy_used">
          <span class="strategy-label">{{ strategyLabels[result.strategy_used] || result.strategy_used }}</span>
        </div>
      </div>
    </header>
    
    <!-- Message Content -->
    <main class="result-content">
      <div 
        class="message-text" 
        v-html="highlightedContent"
      ></div>
      
      <!-- File Information -->
      <div v-if="result.file_info" class="file-info">
        <div class="file-icon">
          <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
        </div>
        <div class="file-details">
          <span class="file-name">{{ result.file_info.name }}</span>
          <span class="file-size">{{ formatFileSize(result.file_info.size) }}</span>
        </div>
      </div>
      
      <!-- Context Preview -->
      <div v-if="result.context" class="context-preview">
        <span class="context-label">Context:</span>
        <span class="context-text">{{ result.context }}</span>
      </div>
    </main>
    
    <!-- Result Footer -->
    <footer class="result-footer" v-if="showActions">
      <div class="result-actions">
        <button 
          @click.stop="handlePreview"
          class="action-button preview-button"
          type="button"
        >
          <svg class="action-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
          </svg>
          Preview
        </button>
        
        <button 
          @click.stop="handleCopy"
          class="action-button copy-button"
          type="button"
        >
          <svg class="action-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
          </svg>
          Copy
        </button>
        
        <button 
          @click.stop="handleNavigate"
          class="action-button navigate-button primary"
          type="button"
        >
          <svg class="action-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
          </svg>
          Go to Message
        </button>
      </div>
    </footer>
  </article>
</template>

<script setup>
import { computed, ref } from 'vue';

// Props
const props = defineProps({
  result: {
    type: Object,
    required: true
  },
  query: {
    type: String,
    default: ''
  },
  isHighlighted: {
    type: Boolean,
    default: false
  },
  showScore: {
    type: Boolean,
    default: false
  },
  showActions: {
    type: Boolean,
    default: true
  }
});

// Emits
const emit = defineEmits(['navigate', 'preview', 'copy', 'hover']);

// Strategy Labels
const strategyLabels = {
  full_text: 'Full Text',
  semantic: 'Semantic',
  fuzzy: 'Fuzzy',
  exact: 'Exact',
  boolean: 'Boolean',
  temporal: 'Recent',
  user_scoped: 'User',
  file_content: 'File',
  fallback_local: 'Local'
};

// Computed Properties
const highlightedContent = computed(() => {
  if (props.result.content_highlighted) {
    return props.result.content_highlighted;
  }
  
  // Fallback highlighting
  if (props.query && props.result.content) {
    return highlightSearchTerms(props.result.content, props.query);
  }
  
  return props.result.content || '';
});

// Methods
const handleNavigate = () => {
  emit('navigate', props.result);
};

const handlePreview = () => {
  emit('preview', props.result);
};

const handleCopy = async () => {
  try {
    await navigator.clipboard.writeText(props.result.content);
    // You could add a toast notification here
    emit('copy', props.result);
  } catch (error) {
    console.error('Failed to copy:', error);
  }
};

const handleHover = () => {
  emit('hover', props.result);
};

const getInitials = (name) => {
  if (!name) return '?';
  return name
    .split(' ')
    .map(word => word.charAt(0))
    .slice(0, 2)
    .join('')
    .toUpperCase();
};

const formatTime = (timestamp) => {
  const date = new Date(timestamp);
  const now = new Date();
  const diffInHours = (now - date) / (1000 * 60 * 60);
  
  if (diffInHours < 1) {
    return 'Just now';
  } else if (diffInHours < 24) {
    return `${Math.floor(diffInHours)}h ago`;
  } else if (diffInHours < 168) { // 7 days
    return `${Math.floor(diffInHours / 24)}d ago`;
  } else {
    return date.toLocaleDateString();
  }
};

const formatFileSize = (bytes) => {
  if (!bytes) return '';
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${sizes[i]}`;
};

const highlightSearchTerms = (content, query) => {
  if (!query || !content) return content;
  
  const terms = query.toLowerCase().split(/\s+/).filter(term => term.length > 0);
  let highlighted = content;
  
  terms.forEach(term => {
    const regex = new RegExp(`(${term.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi');
    highlighted = highlighted.replace(regex, '<mark class="search-highlight">$1</mark>');
  });
  
  return highlighted;
};
</script>

<style scoped>
/* Search Result Item - Cognitive Design System */

.search-result-item {
  /* Golden Ratio Layout System */
  --golden-ratio: 1.618;
  --base-unit: 16px;
  --spacing-xs: 4px;
  --spacing-sm: 8px;
  --spacing-md: 12px;
  --spacing-lg: 20px;
  --spacing-xl: 32px;
  
  /* Ergonomic Touch Targets */
  --touch-target-min: 44px;
  --touch-target-optimal: 48px;
  
  /* Cognitive Color System */
  --color-primary: #3b82f6;
  --color-secondary: #6b7280;
  --color-success: #10b981;
  --color-warning: #f59e0b;
  --color-error: #ef4444;
  
  /* Neutral Palette */
  --color-bg-primary: #ffffff;
  --color-bg-secondary: #f9fafb;
  --color-bg-tertiary: #f3f4f6;
  --color-border-light: #e5e7eb;
  --color-border-medium: #d1d5db;
  --color-border-dark: #9ca3af;
  
  /* Text Colors */
  --color-text-primary: #111827;
  --color-text-secondary: #6b7280;
  --color-text-tertiary: #9ca3af;
  
  /* Component Layout */
  display: flex;
  flex-direction: column;
  background: var(--color-bg-primary);
  border: 1px solid var(--color-border-light);
  border-radius: 12px; /* Golden ratio based */
  padding: var(--spacing-lg);
  margin-bottom: var(--spacing-md);
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  
  /* Hover and Focus States - Immediate Feedback */
  &:hover {
    border-color: var(--color-primary);
    box-shadow: 
      0 4px 6px -1px rgba(0, 0, 0, 0.1),
      0 2px 4px -1px rgba(0, 0, 0, 0.06);
    transform: translateY(-1px);
  }
  
  &:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 
      0 0 0 3px rgba(59, 130, 246, 0.1),
      0 4px 6px -1px rgba(0, 0, 0, 0.1);
  }
  
  &:active {
    transform: translateY(0);
    box-shadow: 
      0 2px 4px -1px rgba(0, 0, 0, 0.1),
      0 1px 2px -1px rgba(0, 0, 0, 0.06);
  }
}

/* Highlighted state */
.search-result-item.highlighted {
  border-color: var(--color-primary);
  background: var(--color-primary-light);
  box-shadow: var(--shadow-lg);
}

/* Result Header */
.result-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--spacing-md);
}

.sender-info {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  flex: 1;
  min-width: 0; /* Allows text truncation */
}

.sender-avatar {
  width: 40px; /* Optimized for recognition */
  height: 40px;
  border-radius: 50%;
  overflow: hidden;
  flex-shrink: 0;
  background: var(--color-bg-tertiary);
  display: flex;
  align-items: center;
  justify-content: center;
}

.avatar-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.avatar-fallback {
  width: 100%;
  height: 100%;
  background: var(--color-primary);
  color: var(--color-text-inverse);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  font-weight: 600;
}

.sender-details {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.sender-name {
  font-weight: 600;
  color: var(--color-text-primary);
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.message-time {
  font-size: 12px;
  color: var(--color-text-tertiary);
  white-space: nowrap;
}

.result-meta {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  flex-shrink: 0;
}

.relevance-score {
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-success);
  color: var(--color-text-inverse);
  border-radius: var(--radius-sm);
  font-size: 11px;
  font-weight: 600;
}

.strategy-badge {
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-surface);
  border: 1px solid var(--color-border-medium);
  border-radius: var(--radius-sm);
  font-size: 11px;
  font-weight: 500;
  color: var(--color-text-secondary);
}

/* Result Content */
.result-content {
  margin-bottom: var(--spacing-lg);
}

.message-text {
  color: var(--color-text-primary);
  line-height: 1.6; /* Optimal for reading */
  margin-bottom: var(--spacing-md);
  
  /* Enhanced readability */
  font-size: 14px;
  word-wrap: break-word;
  overflow-wrap: break-word;
}

/* Search Highlight */
.message-text :deep(.search-highlight) {
  background: var(--color-warning);
  color: var(--color-text-primary);
  padding: 2px 4px;
  border-radius: 3px;
  font-weight: 600;
}

/* File Information */
.file-info {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-bg-secondary);
  border-radius: 8px;
  margin-bottom: var(--spacing-md);
  border: 1px solid var(--color-border-light);
}

.file-icon {
  width: 16px;
  height: 16px;
  color: var(--color-warning);
  flex-shrink: 0;
}

.file-details {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.file-name {
  font-size: 13px;
  color: var(--color-text-primary);
  font-weight: 500;
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-size {
  font-size: 12px;
  color: var(--color-text-tertiary);
  flex-shrink: 0;
}

/* Context Preview */
.context-preview {
  padding: var(--spacing-md);
  background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
  border-radius: 8px;
  border-left: 3px solid var(--color-primary);
}

.context-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 4px;
}

.context-text {
  font-size: 13px;
  color: var(--color-text-secondary);
  line-height: 1.5;
  font-style: italic;
}

/* Result Footer */
.result-footer {
  border-top: 1px solid var(--color-border-light);
  padding-top: var(--spacing-md);
  margin-top: auto;
}

.result-actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  justify-content: flex-end;
}

.action-button {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs) var(--spacing-sm);
  border: 1px solid transparent;
  background: transparent;
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.15s ease;
  min-height: 32px; /* Sufficient for secondary actions */
  
  &:hover {
    background: var(--color-bg-secondary);
  }
  
  &:focus {
    outline: none;
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
  }
}

.action-button.primary {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
  min-height: var(--touch-target-min); /* Primary action gets full touch target */
  
  &:hover {
    background: #2563eb;
    border-color: #2563eb;
  }
}

.action-button.preview-button {
  border-color: var(--color-border-medium);
  color: var(--color-text-secondary);
  
  &:hover {
    border-color: var(--color-primary);
    color: var(--color-primary);
    background: rgba(59, 130, 246, 0.05);
  }
}

.action-button.copy-button {
  width: 32px;
  height: 32px;
  padding: 0;
  justify-content: center;
  color: var(--color-text-tertiary);
  
  &:hover {
    color: var(--color-text-secondary);
    background: var(--color-bg-secondary);
  }
}

.action-icon {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
}

/* Responsive Design - Mobile First Approach */
@media (max-width: 640px) {
  .search-result-item {
    padding: var(--spacing-md);
    margin-bottom: var(--spacing-sm);
  }
  
  .result-header {
    flex-direction: column;
    gap: var(--spacing-sm);
  }
  
  .result-meta {
    flex-direction: row;
    align-items: center;
    align-self: stretch;
    justify-content: space-between;
  }
  
  .sender-info {
    gap: var(--spacing-sm);
  }
  
  .sender-avatar {
    width: 32px;
    height: 32px;
  }
  
  .result-actions {
    flex-wrap: wrap;
    gap: var(--spacing-xs);
  }
  
  .action-button {
    min-height: var(--touch-target-min); /* Full touch targets on mobile */
  }
}

/* Reduced Motion Support */
@media (prefers-reduced-motion: reduce) {
  .search-result-item {
    transition: none;
  }
}

/* High Contrast Mode */
@media (prefers-contrast: high) {
  .search-result-item {
    border-width: 2px;
  }
  
  .message-text :deep(.search-highlight) {
    background: #ffff00;
    color: #000000;
    outline: 2px solid #000000;
  }
}

/* Dark Mode Support */
@media (prefers-color-scheme: dark) {
  .search-result-item {
    --color-bg-primary: #1f2937;
    --color-bg-secondary: #374151;
    --color-bg-tertiary: #4b5563;
    --color-border-light: #4b5563;
    --color-border-medium: #6b7280;
    --color-border-dark: #9ca3af;
    --color-text-primary: #f9fafb;
    --color-text-secondary: #d1d5db;
    --color-text-tertiary: #9ca3af;
  }
  
  .search-result-item.highlighted {
    background: linear-gradient(135deg, #1f2937 0%, #064e3b 100%);
  }
}
</style> 