<template>
  <div 
    :id="`message-${message.id}`"
    class="message-item"
    :class="{ 
      'is-own': isOwnMessage,
      'is-highlighted': isHighlighted,
      'is-editing': isEditing
    }"
    @contextmenu.prevent="handleContextMenu"
  >
    <!-- Message Header -->
    <div class="message-header">
      <img 
        :src="avatarUrl" 
        :alt="message.sender_name"
        class="message-avatar"
        loading="lazy"
      >
      <div class="message-meta">
        <span class="message-sender">{{ message.sender_name }}</span>
        <span class="message-time">{{ formatTime(message.created_at) }}</span>
        <span v-if="message.edited_at" class="message-edited">(edited)</span>
      </div>
    </div>

    <!-- Message Content with High-Performance Markdown -->
    <div class="message-body">
      <ShikiMarkdownMessage
        v-if="!isEditing"
        ref="markdownRef"
        :content="message.content"
        :message-id="message.id"
        :immediate="isRecentMessage"
        theme="dark"
        :line-numbers="true"
      />
      
      <!-- Edit Mode -->
      <div v-else class="message-edit">
        <textarea
          v-model="editContent"
          @keydown.enter.ctrl="saveEdit"
          @keydown.esc="cancelEdit"
          class="edit-textarea"
          :rows="Math.min(editContent.split('\n').length + 1, 10)"
        />
        <div class="edit-actions">
          <button @click="saveEdit" class="btn-save">Save</button>
          <button @click="cancelEdit" class="btn-cancel">Cancel</button>
        </div>
      </div>
    </div>

    <!-- Message Actions -->
    <div v-if="showActions" class="message-actions">
      <button @click="toggleReaction('👍')" class="action-btn">👍</button>
      <button @click="toggleReaction('❤️')" class="action-btn">❤️</button>
      <button @click="handleReply" class="action-btn">Reply</button>
      <button v-if="canEdit" @click="startEdit" class="action-btn">Edit</button>
      <button v-if="canDelete" @click="handleDelete" class="action-btn">Delete</button>
    </div>

    <!-- Reactions -->
    <div v-if="message.reactions?.length" class="message-reactions">
      <span 
        v-for="reaction in groupedReactions" 
        :key="reaction.emoji"
        @click="toggleReaction(reaction.emoji)"
        class="reaction-badge"
        :class="{ 'is-active': reaction.hasReacted }"
      >
        {{ reaction.emoji }} {{ reaction.count }}
      </span>
    </div>

    <!-- Performance Metrics (dev only) -->
    <div v-if="showPerformanceMetrics" class="performance-metrics">
      <span>Render time: {{ performanceInfo.processingTime?.toFixed(2) }}ms</span>
      <span>Renders: {{ performanceInfo.renderCount }}</span>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue';
import { useAuthStore } from '@/stores/auth';
import { useChatStore } from '@/stores/chat';
import ShikiMarkdownMessage from './ShikiMarkdownMessage.vue';
import { format, formatDistanceToNow, isToday } from 'date-fns';

const props = defineProps({
  message: {
    type: Object,
    required: true
  },
  isHighlighted: {
    type: Boolean,
    default: false
  },
  showActions: {
    type: Boolean,
    default: true
  },
  showPerformanceMetrics: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits(['reply', 'edit', 'delete', 'reaction']);

// Stores
const authStore = useAuthStore();
const chatStore = useChatStore();

// Refs
const markdownRef = ref(null);
const isEditing = ref(false);
const editContent = ref('');

// Computed
const isOwnMessage = computed(() => 
  props.message.sender_id === authStore.user?.id
);

const canEdit = computed(() => 
  isOwnMessage.value && !props.message.deleted_at
);

const canDelete = computed(() => 
  isOwnMessage.value || authStore.user?.role === 'admin'
);

const avatarUrl = computed(() => 
  props.message.sender_avatar || 
  `https://api.dicebear.com/7.x/initials/svg?seed=${props.message.sender_name}`
);

const isRecentMessage = computed(() => {
  const messageTime = new Date(props.message.created_at);
  const now = new Date();
  return (now - messageTime) < 60000; // Less than 1 minute old
});

const groupedReactions = computed(() => {
  if (!props.message.reactions) return [];
  
  const grouped = {};
  props.message.reactions.forEach(reaction => {
    if (!grouped[reaction.emoji]) {
      grouped[reaction.emoji] = {
        emoji: reaction.emoji,
        count: 0,
        users: [],
        hasReacted: false
      };
    }
    grouped[reaction.emoji].count++;
    grouped[reaction.emoji].users.push(reaction.user_id);
    if (reaction.user_id === authStore.user?.id) {
      grouped[reaction.emoji].hasReacted = true;
    }
  });
  
  return Object.values(grouped);
});

const performanceInfo = computed(() => 
  markdownRef.value?.performanceInfo || {}
);

// Methods
function formatTime(timestamp) {
  const date = new Date(timestamp);
  
  if (isToday(date)) {
    return format(date, 'HH:mm');
  }
  
  const distance = formatDistanceToNow(date, { addSuffix: true });
  return distance;
}

function handleContextMenu(event) {
  // Emit context menu event for parent to handle
  emit('contextmenu', { event, message: props.message });
}

function startEdit() {
  isEditing.value = true;
  editContent.value = props.message.content;
}

function cancelEdit() {
  isEditing.value = false;
  editContent.value = '';
}

async function saveEdit() {
  if (editContent.value.trim() === props.message.content) {
    cancelEdit();
    return;
  }
  
  try {
    await chatStore.editMessage(props.message.id, editContent.value);
    isEditing.value = false;
  } catch (error) {
    console.error('Failed to edit message:', error);
  }
}

function handleReply() {
  emit('reply', props.message);
}

async function handleDelete() {
  if (confirm('Are you sure you want to delete this message?')) {
    try {
      await chatStore.deleteMessage(props.message.id);
    } catch (error) {
      console.error('Failed to delete message:', error);
    }
  }
}

async function toggleReaction(emoji) {
  try {
    await chatStore.toggleReaction(props.message.id, emoji);
  } catch (error) {
    console.error('Failed to toggle reaction:', error);
  }
}

// Watch for content changes to force re-render if needed
watch(() => props.message.content, (newContent, oldContent) => {
  if (newContent !== oldContent && markdownRef.value) {
    markdownRef.value.forceRender();
  }
});
</script>

<style scoped>
.message-item {
  padding: 0.75rem 1rem;
  transition: background-color 0.2s;
}

.message-item:hover {
  background-color: var(--bg-hover);
}

.message-item.is-highlighted {
  background-color: var(--highlight-bg);
  animation: highlight-fade 2s ease-out;
}

@keyframes highlight-fade {
  from { background-color: var(--highlight-bg-strong); }
  to { background-color: var(--highlight-bg); }
}

.message-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.25rem;
}

.message-avatar {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  object-fit: cover;
}

.message-meta {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
  font-size: 0.875rem;
}

.message-sender {
  font-weight: 600;
  color: var(--text-primary);
}

.message-time {
  color: var(--text-secondary);
  font-size: 0.75rem;
}

.message-edited {
  color: var(--text-tertiary);
  font-size: 0.75rem;
}

.message-body {
  margin-left: 44px; /* Avatar width + gap */
  position: relative;
}

.message-edit {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.edit-textarea {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-family: inherit;
  font-size: inherit;
  resize: vertical;
  min-height: 3rem;
}

.edit-actions {
  display: flex;
  gap: 0.5rem;
}

.btn-save,
.btn-cancel {
  padding: 0.25rem 0.75rem;
  border-radius: 4px;
  font-size: 0.875rem;
  border: none;
  cursor: pointer;
  transition: background-color 0.2s;
}

.btn-save {
  background-color: var(--primary-color);
  color: white;
}

.btn-save:hover {
  background-color: var(--primary-hover);
}

.btn-cancel {
  background-color: var(--bg-secondary);
  color: var(--text-primary);
}

.btn-cancel:hover {
  background-color: var(--bg-tertiary);
}

.message-actions {
  display: flex;
  gap: 0.25rem;
  margin-top: 0.5rem;
  margin-left: 44px;
  opacity: 0;
  transition: opacity 0.2s;
}

.message-item:hover .message-actions {
  opacity: 1;
}

.action-btn {
  padding: 0.25rem 0.5rem;
  background: var(--bg-secondary);
  border: none;
  border-radius: 4px;
  font-size: 0.75rem;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.message-reactions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
  margin-top: 0.5rem;
  margin-left: 44px;
}

.reaction-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.125rem 0.5rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.2s;
}

.reaction-badge:hover {
  background: var(--bg-tertiary);
  border-color: var(--border-hover);
}

.reaction-badge.is-active {
  background: var(--primary-bg);
  border-color: var(--primary-color);
  color: var(--primary-color);
}

.performance-metrics {
  margin-top: 0.5rem;
  margin-left: 44px;
  font-size: 0.75rem;
  color: var(--text-tertiary);
  display: flex;
  gap: 1rem;
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
  :root {
    --bg-primary: #1a1a1a;
    --bg-secondary: #2a2a2a;
    --bg-tertiary: #3a3a3a;
    --bg-hover: #252525;
    --text-primary: #ffffff;
    --text-secondary: #a0a0a0;
    --text-tertiary: #707070;
    --border-color: #3a3a3a;
    --border-hover: #4a4a4a;
    --primary-color: #4a9eff;
    --primary-hover: #3a8eef;
    --primary-bg: rgba(74, 158, 255, 0.1);
    --highlight-bg: rgba(255, 255, 255, 0.05);
    --highlight-bg-strong: rgba(255, 255, 255, 0.1);
  }
}
</style>