import { ref, computed } from 'vue';
import { useAuthStore } from '@/stores/auth';
import minimalSSE from './sse-minimal'; // Import the actual SSE service

const onlineUserIds = ref(new Set());
const status = ref('disconnected');

const presenceService = {
  isInitialized: false,

  /**
   * Initialize the presence service and connect to the SSE stream.
   */
  initialize() {
    if (this.isInitialized) return;

    // Listen for raw SSE messages
    minimalSSE.on('message', (event) => {
      if (event.type === 'presence_update') {
        this.handlePresenceUpdate(event.data);
      }
    });

    // Listen for connection status changes
    minimalSSE.on('connect', () => {
      status.value = 'connected';
      this.updateMyPresence('online');
    });

    minimalSSE.on('disconnect', () => {
      status.value = 'disconnected';
    });

    this.isInitialized = true;
    // console.log('Presence Service Initialized');
  },

  /**
   * Handle incoming presence updates from the server.
   * @param {object} update - The presence update data.
   */
  handlePresenceUpdate(update) {
    if (!update || !update.user_id) return;
    if (update.status === 'online') {
      onlineUserIds.value.add(update.user_id);
    } else {
      onlineUserIds.value.delete(update.user_id);
    }
  },

  /**
   * Check if a user is online.
   * @param {number} userId - The ID of the user to check.
   * @returns {boolean} - True if the user is online, false otherwise.
   */
  isUserOnline(userId) {
    return onlineUserIds.value.has(userId);
  },

  /**
   * Get the current presence status (connected, disconnected).
   */
  getStatus() {
    return status.value;
  },

  /**
   * Manually update the current user's presence status.
   * @param {'online' | 'offline'} newStatus - The new presence status.
   */
  updateMyPresence(newStatus) {
    const authStore = useAuthStore();
    if (authStore.isAuthenticated) {
      // This would typically send an update to the backend
      // console.log(`[Presence] Broadcasting presence: ${newStatus}`);
      // For now, we simulate by adding/removing from the local set
      if (newStatus === 'online') {
        onlineUserIds.value.add(authStore.userId);
      } else {
        onlineUserIds.value.delete(authStore.userId);
      }
    }
  },

  // Export reactive properties for use in components
  onlineUserIds: computed(() => onlineUserIds.value),
  status: computed(() => status.value),
};

export default presenceService; 