<template>
  <div class="fechatter-modal-overlay" @click="handleOverlayClick">
    <div class="fechatter-modal" @click.stop>
      <div class="fechatter-modal-header">
        <h2 class="fechatter-modal-title">Start a direct message</h2>
        <button @click="$emit('close')" class="fechatter-icon-btn">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
          </svg>
        </button>
      </div>

      <div class="fechatter-modal-body">
        <div class="fechatter-form-group">
          <label for="userSearch" class="fechatter-label">
            Search for people
          </label>
          <input
            id="userSearch"
            v-model="searchQuery"
            type="text"
            class="fechatter-input"
            placeholder="Type a name or email"
            @input="searchUsers"
            @focus="showResults = true"
            autofocus
          />
          <div class="fechatter-help-text">
            Start a direct message with someone in your workspace.
          </div>
        </div>

        <!-- Search Results -->
        <div v-if="showResults && searchResults.length > 0" class="fechatter-user-list">
          <h3 class="fechatter-section-title">People</h3>
          <div class="fechatter-user-results">
            <button
              v-for="user in searchResults"
              :key="user.id"
              @click="createDM(user)"
              :disabled="creating"
              class="fechatter-user-item"
            >
              <div class="fechatter-user-avatar">
                {{ user.fullname.charAt(0).toUpperCase() }}
              </div>
              <div class="fechatter-user-info">
                <div class="fechatter-user-name">{{ user.fullname }}</div>
                <div class="fechatter-user-email">{{ user.email }}</div>
                <div v-if="user.status" class="fechatter-user-status">
                  <span 
                    class="fechatter-status-indicator"
                    :class="getStatusClass(user.status)"
                  ></span>
                  {{ getStatusText(user.status) }}
                </div>
              </div>
              <div v-if="creating && selectedUser?.id === user.id" class="fechatter-spinner"></div>
            </button>
          </div>
        </div>

        <!-- Empty State -->
        <div v-else-if="showResults && searchQuery && searchResults.length === 0" class="fechatter-empty-state">
          <div class="fechatter-empty-icon">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" opacity="0.3">
              <path d="M12 2c5.523 0 10 4.477 10 10s-4.477 10-10 10S2 17.523 2 12 6.477 2 12 2zm0 2a8 8 0 100 16 8 8 0 000-16zm0 3a1 1 0 01.993.883L13 8v4a1 1 0 01-1.993.117L11 12V8a1 1 0 011-1zm0 8a1 1 0 110 2 1 1 0 010-2z"/>
            </svg>
          </div>
          <h3 class="fechatter-empty-title">No people found</h3>
          <p class="fechatter-empty-description">
            We couldn't find anyone matching "{{ searchQuery }}". Try a different search term.
          </p>
        </div>

        <!-- Initial State -->
        <div v-else-if="!showResults || !searchQuery" class="fechatter-initial-state">
          <div class="fechatter-initial-icon">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="currentColor" opacity="0.3">
              <path d="M12 2C13.1 2 14 2.9 14 4C14 5.1 13.1 6 12 6C10.9 6 10 5.1 10 4C10 2.9 10.9 2 12 2ZM21 9V7L15 7C13.9 7 13 7.9 13 9V10L11 10V9C11 7.9 10.1 7 9 7L3 7V9L9 9V16C9 17.1 9.9 18 11 18L13 18C14.1 18 15 17.1 15 16V11L18 11V16C18 17.1 18.9 18 20 18S22 17.1 22 16V11C22 10.4 21.6 10 21 10S20 10.4 20 11V9L21 9Z"/>
            </svg>
          </div>
          <h3 class="fechatter-initial-title">Direct messages</h3>
          <p class="fechatter-initial-description">
            Send a message directly to someone in your workspace. Start typing their name above to get started.
          </p>
        </div>

        <!-- Recent Conversations -->
        <div v-if="recentConversations.length > 0 && !searchQuery" class="fechatter-recent-section">
          <h3 class="fechatter-section-title">Recent conversations</h3>
          <div class="fechatter-user-results">
            <button
              v-for="conversation in recentConversations"
              :key="conversation.id"
              @click="openExistingDM(conversation)"
              class="fechatter-user-item"
            >
              <div class="fechatter-user-avatar">
                {{ getConversationAvatar(conversation) }}
              </div>
              <div class="fechatter-user-info">
                <div class="fechatter-user-name">{{ conversation.name }}</div>
                <div class="fechatter-user-email">
                  Last message: {{ formatLastMessage(conversation.last_message_at) }}
                </div>
              </div>
            </button>
          </div>
        </div>
      </div>

      <div class="fechatter-modal-footer">
        <button type="button" @click="$emit('close')" class="fechatter-btn fechatter-btn-ghost">
          Cancel
        </button>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, computed, onMounted } from 'vue'
import { useChatStore } from '../../stores/chat'
import { useUserStore } from '../../stores/user'

export default {
  name: 'CreateDMModal',
  emits: ['close', 'created'],
  setup(props, { emit }) {
    const chatStore = useChatStore()
    const userStore = useUserStore()

    const searchQuery = ref('')
    const searchResults = ref([])
    const showResults = ref(false)
    const creating = ref(false)
    const selectedUser = ref(null)
    const recentConversations = ref([])

    const allUsers = ref([])

    const searchUsers = () => {
      if (!searchQuery.value.trim()) {
        searchResults.value = []
        showResults.value = false
        return
      }

      showResults.value = true
      const query = searchQuery.value.toLowerCase()
      
      searchResults.value = allUsers.value
        .filter(user => 
          user.fullname.toLowerCase().includes(query) ||
          user.email.toLowerCase().includes(query)
        )
        .slice(0, 10) // Limit to 10 results
    }

    const createDM = async (user) => {
      if (creating.value) return

      creating.value = true
      selectedUser.value = user

      try {
        // Create a direct message (Single chat type)
        const dm = await chatStore.createChat(
          `${user.fullname}`, // DM name is usually the other person's name
          [user.id], // Members array with the selected user
          '', // No description for DMs
          'Single' // Chat type for direct messages
        )

        emit('created', dm)
      } catch (error) {
        console.error('Failed to create DM:', error)
        // Show error message to user
      } finally {
        creating.value = false
        selectedUser.value = null
      }
    }

    const openExistingDM = (conversation) => {
      emit('created', conversation)
    }

    const loadData = async () => {
      try {
        // Load all workspace users
        const users = await userStore.getWorkspaceUsers()
        allUsers.value = users || []

        // Load recent DM conversations
        const chats = await chatStore.fetchChats()
        recentConversations.value = (chats || [])
          .filter(chat => chat.chat_type === 'Single')
          .sort((a, b) => new Date(b.last_message_at || 0) - new Date(a.last_message_at || 0))
          .slice(0, 5) // Show last 5 DM conversations
      } catch (error) {
        console.error('Failed to load data:', error)
      }
    }

    const getStatusClass = (status) => {
      const statusClasses = {
        'Active': 'fechatter-status-online',
        'Away': 'fechatter-status-away',
        'Busy': 'fechatter-status-busy',
        'Offline': 'fechatter-status-offline'
      }
      return statusClasses[status] || 'fechatter-status-offline'
    }

    const getStatusText = (status) => {
      const statusTexts = {
        'Active': 'Active',
        'Away': 'Away',
        'Busy': 'Busy',
        'Offline': 'Offline'
      }
      return statusTexts[status] || 'Offline'
    }

    const getConversationAvatar = (conversation) => {
      // For DMs, show the other person's initial
      return conversation.name.charAt(0).toUpperCase()
    }

    const formatLastMessage = (timestamp) => {
      if (!timestamp) return 'No messages yet'
      
      const date = new Date(timestamp)
      const now = new Date()
      const diffInHours = (now - date) / (1000 * 60 * 60)
      
      if (diffInHours < 1) {
        return 'Just now'
      } else if (diffInHours < 24) {
        return `${Math.floor(diffInHours)} hours ago`
      } else if (diffInHours < 24 * 7) {
        return date.toLocaleDateString([], { weekday: 'short' })
      } else {
        return date.toLocaleDateString([], { month: 'short', day: 'numeric' })
      }
    }

    const handleOverlayClick = () => {
      emit('close')
    }

    onMounted(() => {
      loadData()
    })

    return {
      searchQuery,
      searchResults,
      showResults,
      creating,
      selectedUser,
      recentConversations,
      searchUsers,
      createDM,
      openExistingDM,
      getStatusClass,
      getStatusText,
      getConversationAvatar,
      formatLastMessage,
      handleOverlayClick
    }
  }
}
</script>

<style scoped>
/* User List Styles */
.fechatter-user-list {
  margin-top: var(--space-6);
}

.fechatter-section-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin: 0 0 var(--space-3);
}

.fechatter-user-results {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.fechatter-user-item {
  display: flex;
  align-items: center;
  width: 100%;
  padding: var(--space-3) var(--space-4);
  border: none;
  background: transparent;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.15s ease;
  text-align: left;
}

.fechatter-user-item:hover:not(:disabled) {
  background: var(--color-background-secondary);
}

.fechatter-user-item:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.fechatter-user-item .fechatter-user-avatar {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-md);
  background: var(--color-primary);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 14px;
  margin-right: var(--space-3);
  flex-shrink: 0;
}

.fechatter-user-info {
  flex: 1;
  min-width: 0;
}

.fechatter-user-name {
  font-weight: 600;
  color: var(--color-text);
  font-size: 15px;
  margin-bottom: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fechatter-user-email {
  font-size: 13px;
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fechatter-user-status {
  display: flex;
  align-items: center;
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-top: 2px;
}

.fechatter-status-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-right: 6px;
}

.fechatter-status-online {
  background: #00d26a;
}

.fechatter-status-away {
  background: #fbbf24;
}

.fechatter-status-busy {
  background: #ef4444;
}

.fechatter-status-offline {
  background: #6b7280;
}

/* Empty State */
.fechatter-empty-state {
  text-align: center;
  padding: var(--space-8) var(--space-4);
}

.fechatter-empty-icon {
  margin: 0 auto var(--space-4);
  color: var(--color-text-secondary);
}

.fechatter-empty-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 var(--space-2);
}

.fechatter-empty-description {
  font-size: 14px;
  color: var(--color-text-secondary);
  margin: 0;
  line-height: 1.4;
}

/* Initial State */
.fechatter-initial-state {
  text-align: center;
  padding: var(--space-8) var(--space-4);
}

.fechatter-initial-icon {
  margin: 0 auto var(--space-4);
  color: var(--color-text-secondary);
}

.fechatter-initial-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 var(--space-3);
}

.fechatter-initial-description {
  font-size: 15px;
  color: var(--color-text-secondary);
  margin: 0;
  line-height: 1.5;
  max-width: 300px;
  margin-left: auto;
  margin-right: auto;
}

/* Recent Section */
.fechatter-recent-section {
  margin-top: var(--space-6);
  border-top: 1px solid var(--color-border);
  padding-top: var(--space-6);
}

/* Loading spinner in button */
.fechatter-user-item .fechatter-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--color-border);
  border-top: 2px solid var(--color-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-left: var(--space-2);
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
</style> 