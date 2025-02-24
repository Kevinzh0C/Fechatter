import { ref, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { useChatStore } from '@/stores/chat';
import { useAuthStore } from '@/stores/auth';

/**
 * Global keyboard shortcuts composable
 * Provides consistent keyboard navigation across the application
 */
export function useKeyboardShortcuts(options = {}) {
  const router = useRouter();
  const chatStore = useChatStore();
  const authStore = useAuthStore();

  // Options with defaults
  const {
    enableGlobalShortcuts = true,
    enableNavigationShortcuts = true,
    enableChatShortcuts = true,
    enableSearchShortcuts = true,
    contextElement = document
  } = options;

  // Reactive state
  const isEnabled = ref(true);
  const activeModals = ref(new Set());
  const shortcutHistory = ref([]);

  // Keyboard shortcuts registry
  const shortcuts = new Map([
    // Global navigation
    ['ctrl+h', { action: 'goHome', description: 'Go to Home' }],
    ['ctrl+,', { action: 'openSettings', description: 'Open Settings' }],
    ['ctrl+shift+a', { action: 'openAdmin', description: 'Open Admin Dashboard' }],
    ['ctrl+shift+/', { action: 'showHelp', description: 'Show keyboard shortcuts' }],

    // Search shortcuts
    ['ctrl+f', { action: 'openSearch', description: 'Search in current chat' }],
    ['ctrl+k', { action: 'openSearch', description: 'Search in current chat' }],
    ['ctrl+shift+f', { action: 'openGlobalSearch', description: 'Global search' }],

    // Chat navigation
    ['ctrl+1', { action: 'selectChat', chatIndex: 0, description: 'Select first chat' }],
    ['ctrl+2', { action: 'selectChat', chatIndex: 1, description: 'Select second chat' }],
    ['ctrl+3', { action: 'selectChat', chatIndex: 2, description: 'Select third chat' }],
    ['ctrl+4', { action: 'selectChat', chatIndex: 3, description: 'Select fourth chat' }],
    ['ctrl+5', { action: 'selectChat', chatIndex: 4, description: 'Select fifth chat' }],
    ['ctrl+6', { action: 'selectChat', chatIndex: 5, description: 'Select sixth chat' }],
    ['ctrl+7', { action: 'selectChat', chatIndex: 6, description: 'Select seventh chat' }],
    ['ctrl+8', { action: 'selectChat', chatIndex: 7, description: 'Select eighth chat' }],
    ['ctrl+9', { action: 'selectChat', chatIndex: 8, description: 'Select ninth chat' }],

    // Chat actions
    ['ctrl+n', { action: 'newChannel', description: 'Create new channel' }],
    ['ctrl+shift+n', { action: 'newDM', description: 'Create new direct message' }],
    ['ctrl+i', { action: 'showChatInfo', description: 'Show chat information' }],
    ['ctrl+m', { action: 'toggleMute', description: 'Toggle chat mute' }],

    // Message actions
    ['ctrl+enter', { action: 'sendMessage', description: 'Send message (alternative)' }],
    ['ctrl+shift+enter', { action: 'sendMessageToThread', description: 'Send message to thread' }],
    ['escape', { action: 'cancelAction', description: 'Cancel current action' }],
    ['ctrl+z', { action: 'undoLastAction', description: 'Undo last action' }],

    // Message navigation
    ['arrowup', { action: 'navigateMessageUp', description: 'Navigate to previous message' }],
    ['arrowdown', { action: 'navigateMessageDown', description: 'Navigate to next message' }],
    ['home', { action: 'navigateToFirstMessage', description: 'Go to first message' }],
    ['end', { action: 'navigateToLastMessage', description: 'Go to last message' }],
    ['pageup', { action: 'scrollUpPage', description: 'Scroll up one page' }],
    ['pagedown', { action: 'scrollDownPage', description: 'Scroll down one page' }],

    // Message interaction
    ['enter', { action: 'selectMessage', description: 'Select/activate current message' }],
    ['space', { action: 'selectMessage', description: 'Select/activate current message' }],
    ['r', { action: 'replyToMessage', description: 'Reply to selected message' }],
    ['e', { action: 'editMessage', description: 'Edit selected message (if own)' }],
    ['delete', { action: 'deleteMessage', description: 'Delete selected message (if own)' }],
    ['ctrl+c', { action: 'copyMessage', description: 'Copy selected message content' }],
    ['ctrl+shift+c', { action: 'copyMessageLink', description: 'Copy message link' }],

    // Navigation
    ['alt+up', { action: 'previousChat', description: 'Go to previous chat' }],
    ['alt+down', { action: 'nextChat', description: 'Go to next chat' }],
    ['alt+left', { action: 'goBack', description: 'Go back' }],
    ['alt+right', { action: 'goForward', description: 'Go forward' }],

    // View actions
    ['ctrl+shift+s', { action: 'toggleSidebar', description: 'Toggle sidebar' }],
    ['ctrl+shift+m', { action: 'toggleMembersList', description: 'Toggle members list' }],
    ['ctrl+shift+t', { action: 'toggleTheme', description: 'Toggle dark/light theme' }],

    // Quick actions
    ['ctrl+j', { action: 'jumpToMessage', description: 'Jump to specific message' }],
    ['ctrl+shift+u', { action: 'markAllAsRead', description: 'Mark all as read' }],
    ['ctrl+shift+i', { action: 'toggleInviteModal', description: 'Invite people to chat' }],

    // Debug and dev tools (only in development)
    ['ctrl+shift+d', { action: 'toggleDebugPanel', description: 'Toggle debug panel (dev only)' }],
    ['ctrl+shift+r', { action: 'forceReload', description: 'Force reload application' }]
  ]);

  // Platform detection
  const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;

  // Convert shortcuts for Mac (ctrl -> cmd)
  const normalizeShortcut = (shortcut) => {
    if (isMac) {
      return shortcut.replace(/ctrl/g, 'cmd');
    }
    return shortcut;
  };

  // Key combination parser
  const parseKeyEvent = (event) => {
    const parts = [];

    if (event.ctrlKey || event.metaKey) parts.push(isMac ? 'cmd' : 'ctrl');
    if (event.shiftKey) parts.push('shift');
    if (event.altKey) parts.push('alt');

    // Handle special keys - ensure event.key exists
    if (!event.key) return parts.join('+');
    const key = event.key.toLowerCase();
    if (key === ' ') parts.push('space');
    else if (key === 'escape') parts.push('escape');
    else if (key === 'enter') parts.push('enter');
    else if (key === 'backspace') parts.push('backspace');
    else if (key === 'delete') parts.push('delete');
    else if (key === 'tab') parts.push('tab');
    else if (key.startsWith('arrow')) parts.push(key.replace('arrow', ''));
    else if (key.length === 1) parts.push(key);
    else parts.push(key);

    return parts.join('+');
  };

  // Execute shortcut actions
  const executeAction = async (shortcut, event) => {
    if (!isEnabled.value) return false;

    const shortcutData = shortcuts.get(normalizeShortcut(shortcut));
    if (!shortcutData) return false;

    // Add to history
    shortcutHistory.value.unshift({
      shortcut,
      action: shortcutData.action,
      timestamp: new Date().toISOString()
    });

    // Keep only last 50 shortcuts
    if (shortcutHistory.value.length > 50) {
      shortcutHistory.value = shortcutHistory.value.slice(0, 50);
    }

    try {
      switch (shortcutData.action) {
        case 'goHome':
          router.push('/home');
          break;

        case 'openSettings':
          emitGlobalEvent('open-settings');
          break;

        case 'openAdmin':
          emitGlobalEvent('open-admin');
          break;

        case 'showHelp':
          showKeyboardShortcutsHelp();
          break;

        case 'openSearch':
          emitGlobalEvent('open-search');
          break;

        case 'openGlobalSearch':
          emitGlobalEvent('open-global-search');
          break;

        case 'selectChat':
          selectChatByIndex(shortcutData.chatIndex);
          break;

        case 'newChannel':
          emitGlobalEvent('create-channel');
          break;

        case 'newDM':
          emitGlobalEvent('create-dm');
          break;

        case 'showChatInfo':
          emitGlobalEvent('show-chat-info');
          break;

        case 'toggleMute':
          await toggleCurrentChatMute();
          break;

        case 'sendMessage':
          emitGlobalEvent('send-message');
          break;

        case 'cancelAction':
          emitGlobalEvent('cancel-action');
          break;

        case 'previousChat':
          navigateToAdjacentChat(-1);
          break;

        case 'nextChat':
          navigateToAdjacentChat(1);
          break;

        case 'goBack':
          router.back();
          break;

        case 'goForward':
          router.forward();
          break;

        case 'toggleSidebar':
          emitGlobalEvent('toggle-sidebar');
          break;

        case 'toggleMembersList':
          emitGlobalEvent('toggle-members-list');
          break;

        case 'toggleTheme':
          toggleApplicationTheme();
          break;

        case 'jumpToMessage':
          emitGlobalEvent('jump-to-message');
          break;

        case 'markAllAsRead':
          await markAllChatsAsRead();
          break;

        case 'toggleInviteModal':
          emitGlobalEvent('toggle-invite-modal');
          break;

        case 'toggleDebugPanel':
          if (import.meta.env.DEV) {
            emitGlobalEvent('toggle-debug-panel');
          }
          break;

        case 'forceReload':
          if (confirm('Force reload the application? Unsaved changes will be lost.')) {
            window.location.reload();
          }
          break;

        // Message navigation actions
        case 'navigateMessageUp':
          emitGlobalEvent('navigate-message-up');
          break;

        case 'navigateMessageDown':
          emitGlobalEvent('navigate-message-down');
          break;

        case 'navigateToFirstMessage':
          emitGlobalEvent('navigate-to-first-message');
          break;

        case 'navigateToLastMessage':
          emitGlobalEvent('navigate-to-last-message');
          break;

        case 'scrollUpPage':
          emitGlobalEvent('scroll-up-page');
          break;

        case 'scrollDownPage':
          emitGlobalEvent('scroll-down-page');
          break;

        case 'selectMessage':
          emitGlobalEvent('select-message');
          break;

        case 'replyToMessage':
          emitGlobalEvent('reply-to-message');
          break;

        case 'editMessage':
          emitGlobalEvent('edit-message');
          break;

        case 'deleteMessage':
          if (confirm('Are you sure you want to delete this message?')) {
            emitGlobalEvent('delete-message');
          }
          break;

        case 'copyMessage':
          emitGlobalEvent('copy-message');
          break;

        case 'copyMessageLink':
          emitGlobalEvent('copy-message-link');
          break;

        default:
          if (import.meta.env.DEV) {
            console.warn(`Unknown keyboard shortcut action: ${shortcutData.action}`);
          }
          return false;
      }

      return true;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error(`Error executing keyboard shortcut ${shortcut}:`, error);
      }
      return false;
    }
  };

  // Helper functions
  const selectChatByIndex = (index) => {
    const chats = chatStore.chats || [];
    if (index < chats.length) {
      const chat = chats[index];
      router.push(`/chat/${chat.id}`);
    }
  };

  const navigateToAdjacentChat = (direction) => {
    const chats = chatStore.chats || [];
    const currentPath = router.currentRoute.value.path;
    const currentChatId = currentPath.match(/\/chat\/(\d+)/)?.[1];

    if (!currentChatId) return;

    const currentIndex = chats.findIndex(chat => chat.id.toString() === currentChatId);
    if (currentIndex === -1) return;

    const newIndex = currentIndex + direction;
    if (newIndex >= 0 && newIndex < chats.length) {
      const nextChat = chats[newIndex];
      router.push(`/chat/${nextChat.id}`);
    }
  };

  const toggleCurrentChatMute = async () => {
    const currentPath = router.currentRoute.value.path;
    const currentChatId = currentPath.match(/\/chat\/(\d+)/)?.[1];

    if (currentChatId) {
      try {
        await chatStore.toggleChatMute(parseInt(currentChatId));
      } catch (error) {
        if (import.meta.env.DEV) {
          console.error('Failed to toggle chat mute:', error);
        }
      }
    }
  };

  const markAllChatsAsRead = async () => {
    try {
      await chatStore.markAllAsRead();
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Failed to mark all chats as read:', error);
      }
    }
  };

  const toggleApplicationTheme = () => {
    const html = document.documentElement;
    const currentTheme = html.getAttribute('data-theme');
    const newTheme = currentTheme === 'dark' ? 'light' : 'dark';

    html.setAttribute('data-theme', newTheme);
    if (newTheme === 'dark') {
      html.classList.add('dark');
    } else {
      html.classList.remove('dark');
    }

    localStorage.setItem('fechatter-theme', newTheme);
  };

  const emitGlobalEvent = (eventName, data = null) => {
    window.dispatchEvent(new CustomEvent(`fechatter:${eventName}`, { detail: data }));
  };

  const showKeyboardShortcutsHelp = () => {
    const shortcutsList = Array.from(shortcuts.entries()).map(([key, data]) => ({
      key: normalizeShortcut(key),
      description: data.description
    }));

    emitGlobalEvent('show-shortcuts-help', shortcutsList);
  };

  // Check if an element should ignore shortcuts (input fields, etc.)
  const shouldIgnoreShortcut = (event) => {
    const target = event.target;
    const tagName = target.tagName?.toLowerCase();
    const isInput = ['input', 'textarea', 'select'].includes(tagName);
    const isContentEditable = target.contentEditable === 'true';
    const isModal = activeModals.value.size > 0;

    // Allow certain shortcuts even in input fields
    const allowedInInputs = ['escape', 'ctrl+enter', 'ctrl+shift+enter', 'ctrl+f', 'ctrl+k'];
    const currentShortcut = parseKeyEvent(event);

    // Allow navigation shortcuts in message areas (not in input fields)
    const navigationShortcuts = [
      'arrowup', 'arrowdown', 'home', 'end', 'pageup', 'pagedown',
      'enter', 'space', 'r', 'e', 'delete', 'ctrl+c', 'ctrl+shift+c'
    ];

    // Check if we're in a message container (allow navigation)
    const isInMessageContainer = target.closest('.virtual-message-list, .message-list, .discord-message');

    if (isInput || isContentEditable) {
      return !allowedInInputs.includes(normalizeShortcut(currentShortcut));
    }

    // Allow navigation shortcuts in message areas
    if (isInMessageContainer && navigationShortcuts.includes(normalizeShortcut(currentShortcut))) {
      return false;
    }

    return false;
  };

  // Main keyboard event handler
  const handleKeyDown = (event) => {
    if (!isEnabled.value || shouldIgnoreShortcut(event)) {
      return;
    }

    const shortcut = parseKeyEvent(event);
    const executed = executeAction(shortcut, event);

    if (executed) {
      event.preventDefault();
      event.stopPropagation();
    }
  };

  // Public API
  const enable = () => {
    isEnabled.value = true;
  };

  const disable = () => {
    isEnabled.value = false;
  };

  const addModal = (modalId) => {
    activeModals.value.add(modalId);
  };

  const removeModal = (modalId) => {
    activeModals.value.delete(modalId);
  };

  const getShortcuts = () => {
    return Array.from(shortcuts.entries()).map(([key, data]) => ({
      key: normalizeShortcut(key),
      ...data
    }));
  };

  const getShortcutHistory = () => {
    return shortcutHistory.value;
  };

  // Setup and cleanup
  onMounted(() => {
    if (enableGlobalShortcuts) {
      contextElement.addEventListener('keydown', handleKeyDown);
    }
  });

  onUnmounted(() => {
    if (enableGlobalShortcuts) {
      contextElement.removeEventListener('keydown', handleKeyDown);
    }
  });

  return {
    // State
    isEnabled,
    shortcuts: getShortcuts(),
    shortcutHistory: getShortcutHistory(),

    // Methods
    enable,
    disable,
    addModal,
    removeModal,
    executeAction: (shortcut) => executeAction(shortcut, { preventDefault: () => { }, stopPropagation: () => { } }),

    // Utilities
    normalizeShortcut,
    parseKeyEvent,
    emitGlobalEvent
  };
}

export default useKeyboardShortcuts;