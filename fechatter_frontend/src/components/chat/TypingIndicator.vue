<template>
  <transition enter-active-class="transition-all duration-300 ease-out" enter-from-class="opacity-0 translate-y-2"
    enter-to-class="opacity-100 translate-y-0" leave-active-class="transition-all duration-200 ease-in"
    leave-from-class="opacity-100 translate-y-0" leave-to-class="opacity-0 translate-y-2">
    <div v-if="typingUsers.length > 0" class="typing-indicator">
      <div class="flex items-center space-x-2 px-4 py-2 text-sm text-gray-600">
        <div class="typing-animation">
          <span></span>
          <span></span>
          <span></span>
        </div>
        <span>{{ typingText }}</span>
      </div>
    </div>
  </transition>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import minimalSSE from '@/services/sse-minimal';
import { useAuthStore } from '@/stores/auth';

const props = defineProps({
  chatId: {
    type: Number,
    required: true
  },
  currentUserId: {
    type: Number,
    required: true
  }
});

// State
const typingUsers = ref([]);

// Computed
const typingText = computed(() => {
  const count = typingUsers.value.length;
  if (count === 0) return '';

  if (count === 1) {
    return `${typingUsers.value[0].userName} is typing...`;
  } else if (count === 2) {
    return `${typingUsers.value[0].userName} and ${typingUsers.value[1].userName} are typing...`;
  } else {
    return `${typingUsers.value[0].userName} and ${count - 1} others are typing...`;
  }
});

// Handle typing status updates
function handleTypingStatus(data) {
  if (data.chatId !== props.chatId || data.userId === props.currentUserId) {
    // Ignore typing status from other chats or self
    return;
  }

  const userIndex = typingUsers.value.findIndex(u => u.userId === data.userId);

  if (data.isTyping) {
    // Add or update typing user
    if (userIndex === -1) {
      typingUsers.value.push({
        userId: data.userId,
        userName: data.userName || `User ${data.userId}`,
        timestamp: Date.now()
      });
    } else {
      typingUsers.value[userIndex].timestamp = Date.now();
    }
  } else {
    // Remove typing user
    if (userIndex !== -1) {
      typingUsers.value.splice(userIndex, 1);
    }
  }

  // Clean up stale typing indicators (older than 5 seconds)
  const now = Date.now();
  typingUsers.value = typingUsers.value.filter(u => now - u.timestamp < 5000);
}

// Cleanup stale typing indicators periodically
let cleanupTimer = null;

function startCleanup() {
  cleanupTimer = setInterval(() => {
    const now = Date.now();
    typingUsers.value = typingUsers.value.filter(u => now - u.timestamp < 5000);
  }, 1000);
}

function stopCleanup() {
  if (cleanupTimer) {
    clearInterval(cleanupTimer);
    cleanupTimer = null;
  }
}

// Lifecycle
onMounted(() => {
  // Listen for typing status events
  minimalSSE.on('typing_status', handleTypingStatus);
  startCleanup();
});

onUnmounted(() => {
  minimalSSE.off('typing_status', handleTypingStatus);
  stopCleanup();
});
</script>

<style scoped>
.typing-indicator {
  background: rgba(255, 255, 255, 0.95);
  border-top: 1px solid rgba(0, 0, 0, 0.08);
}

.typing-animation {
  display: inline-flex;
  align-items: center;
  gap: 2px;
}

.typing-animation span {
  width: 6px;
  height: 6px;
  background-color: #64748b;
  border-radius: 50%;
  animation: typing 1.4s infinite;
}

.typing-animation span:nth-child(1) {
  animation-delay: 0s;
}

.typing-animation span:nth-child(2) {
  animation-delay: 0.2s;
}

.typing-animation span:nth-child(3) {
  animation-delay: 0.4s;
}

@keyframes typing {

  0%,
  60%,
  100% {
    opacity: 0.3;
    transform: translateY(0);
  }

  30% {
    opacity: 1;
    transform: translateY(-4px);
  }
}
</style>