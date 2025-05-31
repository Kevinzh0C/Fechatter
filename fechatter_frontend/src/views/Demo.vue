<template>
  <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
    <!-- Compact Header -->
    <div class="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700">
      <div class="max-w-4xl mx-auto px-4 py-3 flex items-center justify-between">
        <div class="flex items-center space-x-3">
          <router-link to="/" class="text-purple-600 hover:text-purple-700">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18"></path>
            </svg>
          </router-link>
          <h1 class="text-lg font-semibold text-gray-900 dark:text-gray-100">Feature Demo</h1>
        </div>
        <button @click="toggleTheme" 
                class="p-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded">
          <svg v-if="isDarkMode" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"></path>
          </svg>
          <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"></path>
          </svg>
        </button>
      </div>
    </div>

    <div class="max-w-4xl mx-auto p-4">
      <!-- Quick Actions Grid -->
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3 mb-6">
        <!-- Notifications -->
        <div class="bg-white dark:bg-gray-800 p-3 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
          <h3 class="text-sm font-medium text-gray-900 dark:text-gray-100 mb-2">Notifications</h3>
          <div class="space-y-1">
            <button @click="requestNotificationPermission" 
                    class="w-full px-2 py-1 bg-blue-500 text-white text-xs rounded hover:bg-blue-600">
              Request Permission
            </button>
            <button @click="showTestNotification" 
                    class="w-full px-2 py-1 bg-green-500 text-white text-xs rounded hover:bg-green-600">
              Test Notification
            </button>
          </div>
        </div>

        <!-- Toasts -->
        <div class="bg-white dark:bg-gray-800 p-3 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
          <h3 class="text-sm font-medium text-gray-900 dark:text-gray-100 mb-2">Toast Messages</h3>
          <div class="space-y-1">
            <button @click="showToast('success', 'Success!')" 
                    class="w-full px-2 py-1 bg-green-500 text-white text-xs rounded hover:bg-green-600">
              Success
            </button>
            <button @click="showToast('error', 'Error!')" 
                    class="w-full px-2 py-1 bg-red-500 text-white text-xs rounded hover:bg-red-600">
              Error
            </button>
          </div>
        </div>

        <!-- Touch Gestures -->
        <div class="bg-white dark:bg-gray-800 p-3 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
          <h3 class="text-sm font-medium text-gray-900 dark:text-gray-100 mb-2">Touch Test</h3>
          <div class="h-16 bg-gray-100 dark:bg-gray-700 rounded border-2 border-dashed border-gray-300 dark:border-gray-600 flex items-center justify-center text-xs text-gray-500 dark:text-gray-400"
               ref="touchArea">
            {{ lastGesture || 'Swipe here' }}
          </div>
        </div>

        <!-- Data Test -->
        <div class="bg-white dark:bg-gray-800 p-3 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
          <h3 class="text-sm font-medium text-gray-900 dark:text-gray-100 mb-2">Test Data</h3>
          <button @click="createTestData" 
                  :disabled="creatingData"
                  class="w-full px-2 py-1 bg-purple-500 text-white text-xs rounded hover:bg-purple-600 disabled:opacity-50">
            {{ creatingData ? 'Creating...' : 'Create Data' }}
          </button>
        </div>
      </div>

      <!-- Search Demo -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 mb-6">
        <div class="p-4 border-b border-gray-200 dark:border-gray-700">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">Compact Search</h2>
        </div>
        <div class="p-4">
          <CompactSearch @search="handleSearch" @select="handleSelectMessage" />
        </div>
      </div>

      <!-- Message Actions Demo -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
        <div class="p-4 border-b border-gray-200 dark:border-gray-700">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">Message Demo</h2>
        </div>
        <div class="p-4">
          <MessageItem
            :message="demoMessage"
            :current-user-id="1"
            :show-actions="true"
            @edit="handleEdit"
            @delete="handleDelete"
            @reply="handleReply"
            @reaction="handleReaction"
          />
        </div>
      </div>
    </div>

    <!-- Toast Container -->
    <ToastContainer />
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { useNotifications } from '@/composables/useNotifications';
import { useTouch } from '@/composables/useTouch';
import CompactSearch from '@/components/search/CompactSearch.vue';
import MessageItem from '@/components/chat/MessageItem.vue';
import ToastContainer from '@/components/ui/ToastContainer.vue';

const { showToast, requestNotificationPermission, showBrowserNotification } = useNotifications();

const isDarkMode = ref(localStorage.getItem('theme') === 'dark');
const lastGesture = ref('');
const touchArea = ref(null);
const creatingData = ref(false);

// Demo message
const demoMessage = ref({
  id: 1,
  content: 'This is a **demo message** with `code` and [links](https://example.com)! 🎉',
  author: {
    id: 1,
    fullname: 'Demo User',
    email: 'demo@example.com'
  },
  created_at: new Date().toISOString(),
  reactions: [
    { emoji: '👍', count: 3, hasUserReacted: false },
    { emoji: '❤️', count: 1, hasUserReacted: true }
  ]
});

// Touch gestures
const { onSwipeUp, onSwipeDown, onSwipeLeft, onSwipeRight } = useTouch(touchArea);

onSwipeUp(() => {
  lastGesture.value = '↑ Swipe Up';
  showToast('info', 'Swipe Up detected!');
});

onSwipeDown(() => {
  lastGesture.value = '↓ Swipe Down';
  showToast('info', 'Swipe Down detected!');
});

onSwipeLeft(() => {
  lastGesture.value = '← Swipe Left';
  showToast('info', 'Swipe Left detected!');
});

onSwipeRight(() => {
  lastGesture.value = '→ Swipe Right';
  showToast('info', 'Swipe Right detected!');
});

function toggleTheme() {
  isDarkMode.value = !isDarkMode.value;
  const theme = isDarkMode.value ? 'dark' : 'light';
  localStorage.setItem('theme', theme);
  document.documentElement.classList.toggle('dark', isDarkMode.value);
}

function showTestNotification() {
  showBrowserNotification('Test Notification', {
    body: 'This is a test notification from Fechatter!',
    icon: '/vite.svg'
  });
}

async function createTestData() {
  creatingData.value = true;
  try {
    // Call the test data creation function
    if (window.createTestData) {
      await window.createTestData();
      showToast('success', 'Test data created successfully!');
    } else {
      showToast('error', 'Test data creation function not available');
    }
  } catch (error) {
    showToast('error', 'Failed to create test data');
    console.error('Error creating test data:', error);
  } finally {
    creatingData.value = false;
  }
}

function handleSearch(params) {
  console.log('Search params:', params);
  showToast('info', `Searching for: ${params.query || 'all messages'}`);
}

function handleSelectMessage(message) {
  console.log('Selected message:', message);
  showToast('success', 'Message selected!');
}

function handleEdit(message) {
  showToast('info', 'Edit message functionality');
}

function handleDelete(message) {
  showToast('warning', 'Delete message functionality');
}

function handleReply(message) {
  showToast('info', 'Reply to message functionality');
}

function handleReaction(emoji, message) {
  showToast('success', `Added reaction: ${emoji}`);
}

onMounted(() => {
  // Initialize theme
  document.documentElement.classList.toggle('dark', isDarkMode.value);
  
  // Load test data creation script
  const script = document.createElement('script');
  script.src = '/create_test_data.js';
  document.head.appendChild(script);
});
</script> 