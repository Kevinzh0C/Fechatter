import { ref, computed } from 'vue';

// Global notification state
const notifications = ref([]);
let notificationIdCounter = 0;

export function useNotifications() {
  // Add a new notification
  const addNotification = (notification) => {
    const id = ++notificationIdCounter;
    const newNotification = {
      id,
      title: notification.title || '',
      message: notification.message || '',
      type: notification.type || 'info', // 'success', 'error', 'warning', 'info'
      duration: notification.duration ?? 5000, // null for persistent
      icon: notification.icon,
      actions: notification.actions || [],
      timestamp: Date.now(),
    };
    
    notifications.value.unshift(newNotification);
    
    // Auto-remove after duration
    if (newNotification.duration > 0) {
      setTimeout(() => {
        removeNotification(id);
      }, newNotification.duration);
    }
    
    return id;
  };

  // Remove a notification by id
  const removeNotification = (id) => {
    const index = notifications.value.findIndex(n => n.id === id);
    if (index > -1) {
      notifications.value.splice(index, 1);
    }
  };

  // Clear all notifications
  const clearAllNotifications = () => {
    notifications.value = [];
  };

  // Convenience methods for different types
  const notifySuccess = (message, options = {}) => {
    return addNotification({
      type: 'success',
      message,
      title: options.title || 'Success',
      icon: '',
      ...options
    });
  };

  const notifyError = (message, options = {}) => {
    return addNotification({
      type: 'error',
      message,
      title: options.title || 'Error',
      icon: '❌',
      duration: options.duration ?? 8000, // Errors stay longer
      ...options
    });
  };

  const notifyWarning = (message, options = {}) => {
    return addNotification({
      type: 'warning',
      message,
      title: options.title || 'Warning',
      icon: '⚠️',
      ...options
    });
  };

  const notifyInfo = (message, options = {}) => {
    return addNotification({
      type: 'info',
      message,
      title: options.title || 'Info',
      icon: 'ℹ️',
      ...options
    });
  };

  // Computed values
  const hasNotifications = computed(() => notifications.value.length > 0);
  const notificationCount = computed(() => notifications.value.length);

  return {
    notifications: computed(() => notifications.value),
    hasNotifications,
    notificationCount,
    addNotification,
    removeNotification,
    clearAllNotifications,
    notifySuccess,
    notifyError,
    notifyWarning,
    notifyInfo,
  };
} 