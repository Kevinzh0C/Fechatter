<template>
  <div class="chat-info">
    <!-- Chat Header -->
    <div class="flex items-center justify-between p-4 border-b border-gray-200">
      <div class="flex items-center space-x-3">
        <!-- Chat Type Icon -->
        <div class="flex-shrink-0">
          <div :class="`${getChatTypeIconClass(chat.chat_type)} w-10 h-10 rounded-lg flex items-center justify-center`">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path v-if="chat.chat_type === 'Single'" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"></path>
              <path v-else-if="chat.chat_type === 'Group'" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"></path>
              <path v-else-if="chat.chat_type === 'PublicChannel'" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 20l4-16m2 16l4-16M6 9h14M4 15h14"></path>
              <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"></path>
            </svg>
          </div>
        </div>

        <!-- Chat Details -->
        <div class="flex-1 min-w-0">
          <h1 class="text-xl font-semibold text-gray-900 truncate">{{ chat.name }}</h1>
          <div class="flex items-center space-x-2 mt-1">
            <span :class="`${getChatTypeBadgeClass(chat.chat_type)} inline-flex items-center px-2 py-1 rounded-full text-xs font-medium`">
              {{ getChatTypeLabel(chat.chat_type) }}
            </span>
            <span class="text-sm text-gray-500">
              {{ memberCount }} {{ memberCount === 1 ? 'member' : 'members' }}
            </span>
            <span v-if="chat.created_at" class="text-sm text-gray-500">
              • Created {{ formatDate(chat.created_at) }}
            </span>
          </div>
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="flex items-center space-x-2">
        <button @click="$emit('toggle-search')" 
          class="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-lg"
          title="Search messages">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
          </svg>
        </button>
        
        <button @click="$emit('toggle-members')" 
          class="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-lg"
          title="Manage members">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197m13.5-9a2.5 2.5 0 11-5 0 2.5 2.5 0 015 0z"></path>
          </svg>
        </button>

        <div class="relative">
          <button @click="showMenu = !showMenu" 
            class="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-lg"
            title="Chat options">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z"></path>
            </svg>
          </button>

          <!-- Dropdown Menu -->
          <div v-if="showMenu" class="absolute right-0 mt-2 w-48 bg-white rounded-md shadow-lg py-1 z-50 border border-gray-200">
            <button @click="$emit('edit-chat'); showMenu = false" 
              class="block w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100">
              Edit Chat
            </button>
            <button @click="$emit('chat-settings'); showMenu = false" 
              class="block w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100">
              Chat Settings
            </button>
            <div class="border-t border-gray-100 my-1"></div>
            <button @click="$emit('leave-chat'); showMenu = false" 
              class="block w-full text-left px-4 py-2 text-sm text-red-600 hover:bg-gray-100">
              Leave Chat
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Chat Description -->
    <div v-if="chat.description" class="px-4 py-3 bg-gray-50 border-b border-gray-200">
      <p class="text-sm text-gray-600">{{ chat.description }}</p>
    </div>

    <!-- Chat Statistics -->
    <div v-if="showStats" class="px-4 py-3 bg-blue-50 border-b border-gray-200">
      <div class="grid grid-cols-3 gap-4 text-center">
        <div>
          <div class="text-lg font-semibold text-blue-600">{{ messageCount || 0 }}</div>
          <div class="text-xs text-gray-500">Messages</div>
        </div>
        <div>
          <div class="text-lg font-semibold text-blue-600">{{ memberCount }}</div>
          <div class="text-xs text-gray-500">Members</div>
        </div>
        <div>
          <div class="text-lg font-semibold text-blue-600">{{ fileCount || 0 }}</div>
          <div class="text-xs text-gray-500">Files</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';

const props = defineProps({
  chat: {
    type: Object,
    required: true
  },
  memberCount: {
    type: Number,
    default: 0
  },
  messageCount: {
    type: Number,
    default: 0
  },
  fileCount: {
    type: Number,
    default: 0
  },
  showStats: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits([
  'toggle-search',
  'toggle-members', 
  'edit-chat',
  'chat-settings',
  'leave-chat'
]);

const showMenu = ref(false);

function getChatTypeLabel(chatType) {
  const labels = {
    'Single': 'Direct Message',
    'Group': 'Group Chat',
    'PrivateChannel': 'Private Channel',
    'PublicChannel': 'Public Channel'
  };
  return labels[chatType] || chatType;
}

function getChatTypeBadgeClass(chatType) {
  const classes = {
    'Single': 'bg-purple-100 text-purple-800',
    'Group': 'bg-blue-100 text-blue-800',
    'PrivateChannel': 'bg-gray-100 text-gray-800',
    'PublicChannel': 'bg-green-100 text-green-800'
  };
  return classes[chatType] || 'bg-gray-100 text-gray-800';
}

function getChatTypeIconClass(chatType) {
  const classes = {
    'Single': 'bg-purple-100 text-purple-600',
    'Group': 'bg-blue-100 text-blue-600',
    'PrivateChannel': 'bg-gray-100 text-gray-600',
    'PublicChannel': 'bg-green-100 text-green-600'
  };
  return classes[chatType] || 'bg-gray-100 text-gray-600';
}

function formatDate(dateString) {
  const date = new Date(dateString);
  const now = new Date();
  const diffInDays = Math.floor((now - date) / (1000 * 60 * 60 * 24));
  
  if (diffInDays === 0) {
    return 'today';
  } else if (diffInDays === 1) {
    return 'yesterday';
  } else if (diffInDays < 7) {
    return `${diffInDays} days ago`;
  } else {
    return date.toLocaleDateString([], { 
      month: 'short', 
      day: 'numeric',
      year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined
    });
  }
}

// Close menu when clicking outside
function handleClickOutside(event) {
  if (!event.target.closest('.relative')) {
    showMenu.value = false;
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
});
</script>

<style scoped>
.chat-info {
  @apply bg-white;
}
</style> 