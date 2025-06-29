<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="modelValue" class="search-overlay" @click="handleOverlayClick" @keydown.esc="close">
        <div class="search-modal" @click.stop role="dialog" aria-modal="true" aria-label="Search messages">
          <div class="search-header">
            <h2 class="search-title">Search in #{{ chatName }}</h2>
            <button @click="close" class="close-btn" aria-label="Close search">×</button>
          </div>

          <div class="search-input-section">
            <form @submit.prevent="performSearch">
              <input ref="searchInput" v-model="localQuery" type="text" class="search-input"
                :placeholder="`Search messages in #${chatName}...`" aria-label="Search query" />
              <button type="submit" class="search-btn">Search</button>
            </form>
          </div>

          <div class="search-results">
            <div v-if="loading" class="loading">Searching...</div>
            <div v-else-if="!searchPerformed" class="empty">Type to search messages</div>
            <div v-else-if="results.length === 0" class="no-results">No messages found</div>
            <div v-else class="results-list">
              <div v-for="message in results" :key="message.id" class="message-item" @click="selectMessage(message)">
                <div class="message-content">{{ message.content }}</div>
                <div class="message-meta">{{ message.sender_name }} • {{ formatTime(message.created_at) }}</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { ref, watch, nextTick, onMounted, onUnmounted } from 'vue';

const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false
  },
  chatId: {
    type: [Number, String],
    required: true
  },
  chatName: {
    type: String,
    default: ''
  }
});

const emit = defineEmits(['update:modelValue', 'select-message']);

// Refs
const searchInput = ref(null);
const localQuery = ref('');
const loading = ref(false);
const searchPerformed = ref(false);
const results = ref([]);

// Methods
const performSearch = async () => {
  if (!localQuery.value.trim()) return;

  loading.value = true;
  searchPerformed.value = true;

  try {
    // Try ChatService first
    const ChatService = (await import('@/services/ChatService')).default;
    const searchResults = await ChatService.searchMessages(
      props.chatId,
      localQuery.value.trim(),
      { limit: 20, offset: 0, sort: 'relevance' }
    );
    
    // Convert to expected format
    results.value = searchResults.hits.map(hit => ({
      id: hit.id,
      content: hit.content,
      sender_name: hit.sender_name || hit.sender?.fullname || 'Unknown',
      created_at: hit.created_at,
      chat_id: hit.chat_id || props.chatId
    }));
    
    console.log(`Backend search successful: ${results.value.length} results`);
    
  } catch (error) {
    console.warn('ERROR: Backend search failed, trying local fallback:', error);
    
    // Fallback: Local search in current chat messages
    try {
      const query = localQuery.value.toLowerCase();
      let localMessages = [];
      
      // Try multiple sources for local messages
      
      // 1. Try UnifiedMessageService
      if (window.unifiedMessageService) {
        localMessages = window.unifiedMessageService.getMessagesForChat(props.chatId) || [];
        console.log(`📋 UnifiedMessageService found ${localMessages.length} messages for chat ${props.chatId}`);
      }
      
      // 2. Try chat store if UnifiedMessageService is empty
      if (localMessages.length === 0) {
        try {
          const { useChatStore } = await import('@/stores/chat');
          const chatStore = useChatStore();
          if (chatStore.currentChatId == props.chatId) {
            localMessages = chatStore.messages || [];
            console.log(`📋 Chat store found ${localMessages.length} messages`);
          }
        } catch (storeError) {
          console.warn('Chat store access failed:', storeError);
        }
      }
      
      // 3. Try global message cache as last resort
      if (localMessages.length === 0 && window.messageCache) {
        localMessages = window.messageCache[props.chatId] || [];
        console.log(`📋 Global cache found ${localMessages.length} messages`);
      }
      
      // Filter messages that match the query
      const filteredMessages = localMessages.filter(msg => {
        return msg && 
               msg.content && 
               typeof msg.content === 'string' && 
               msg.content.toLowerCase().includes(query);
      });
      
      results.value = filteredMessages.map(msg => ({
        id: msg.id,
        content: msg.content,
        sender_name: msg.sender_name || msg.sender?.fullname || 'Unknown',
        created_at: msg.created_at,
        chat_id: msg.chat_id || props.chatId
      }));
      
      console.log(`Local fallback search: ${results.value.length} results from ${localMessages.length} total messages`);
      
      // If still no results, show helpful message
      if (results.value.length === 0 && localMessages.length === 0) {
        console.log('No local messages available for search');
        // Don't set results to show "No messages found" instead of error
      }
      
    } catch (fallbackError) {
      console.error('ERROR: Local fallback search also failed:', fallbackError);
      results.value = [];
    }
  } finally {
    loading.value = false;
  }
};

const selectMessage = (message) => {
  emit('select-message', {
    messageId: message.id,
    chatId: message.chat_id || props.chatId
  });
  close();
};

const close = () => {
  emit('update:modelValue', false);
  localQuery.value = '';
  results.value = [];
  searchPerformed.value = false;
};

const handleOverlayClick = (event) => {
  if (event.target === event.currentTarget) {
    close();
  }
};

const formatTime = (timestamp) => {
  const date = new Date(timestamp);
  return date.toLocaleTimeString();
};

// Watch for modal open
watch(() => props.modelValue, (modelValue) => {
  if (modelValue) {
    nextTick(() => {
      searchInput.value?.focus();
    });
  }
});

// Keyboard shortcut
const handleKeyDown = (event) => {
  if (event.key === 'Escape' && props.modelValue) {
    close();
  }
};

onMounted(() => {
  document.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown);
});
</script>

<style scoped>
.search-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.search-modal {
  background: white;
  border-radius: 12px;
  width: 90%;
  max-width: 600px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
}

.search-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px;
  border-bottom: 1px solid #e5e7eb;
}

.search-title {
  font-size: 18px;
  font-weight: 600;
  margin: 0;
}

.close-btn {
  width: 32px;
  height: 32px;
  border: none;
  background: #f3f4f6;
  border-radius: 8px;
  cursor: pointer;
  font-size: 18px;
}

.search-input-section {
  padding: 20px;
  border-bottom: 1px solid #e5e7eb;
}

.search-input-section form {
  display: flex;
  gap: 12px;
}

.search-input {
  flex: 1;
  padding: 12px;
  border: 2px solid #e5e7eb;
  border-radius: 8px;
  font-size: 16px;
}

.search-btn {
  padding: 12px 24px;
  background: #3b82f6;
  color: white;
  border: none;
  border-radius: 8px;
  cursor: pointer;
}

.search-results {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
}

.loading,
.empty,
.no-results {
  text-align: center;
  padding: 40px 20px;
  color: #6b7280;
}

.message-item {
  padding: 12px;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  margin-bottom: 8px;
  cursor: pointer;
}

.message-item:hover {
  background: #f9fafb;
}

.message-content {
  font-size: 14px;
  margin-bottom: 4px;
}

.message-meta {
  font-size: 12px;
  color: #6b7280;
}

.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>