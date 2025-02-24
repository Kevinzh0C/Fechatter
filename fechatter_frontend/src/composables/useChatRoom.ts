import { ref, computed, readonly, watch, onMounted, onUnmounted, nextTick } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import { useWorkspaceStore } from '@/stores/workspace';
import { chatService } from '@/services/ChatService';
import { useToast } from '@/composables/useToast';
import type { Chat, User } from '@/types/chat';

// Chat Room State Machine
export type ChatRoomState = 'INIT' | 'LOADING' | 'READY' | 'SCROLLING' | 'SENDING' | 'CLOSED';

// Events
export type ChatRoomEvent =
  | { type: 'LOAD'; payload: { chatId: string } }
  | { type: 'LOADED' }
  | { type: 'SEND_MESSAGE'; payload: { content: string; files?: File[] } }
  | { type: 'MESSAGE_SENT' }
  | { type: 'SCROLL' }
  | { type: 'SCROLL_COMPLETE' }
  | { type: 'ERROR'; payload: { message: string } }
  | { type: 'CLOSE' };

export interface ChatMessage {
  id: string;
  content: string;
  sender_id: string;
  sender_name: string;
  created_at: string;
  files?: Array<{
    id: string;
    filename: string;
    url: string;
    size: number;
  }>;
}

export interface SendMessagePayload {
  content: string;
  files?: File[];
}

interface ChatRoomComposableState {
  state: ChatRoomState;
  currentChat: Chat | null;
  messages: ChatMessage[];
  chatMembers: User[];
  loading: boolean;
  error: string | null;
  hasMoreMessages: boolean;
}

const MESSAGE_LIMIT = 50;
const MAX_MESSAGE_LENGTH = 4000;
const MIN_MESSAGE_INTERVAL = 500; // milliseconds

export function useChatRoom() {
  const route = useRoute();
  const router = useRouter();
  const authStore = useAuthStore();
  const workspaceStore = useWorkspaceStore();
  const toast = useToast();

  // State
  const state = ref<ChatRoomComposableState>({
    state: 'INIT',
    currentChat: null,
    messages: [],
    chatMembers: [],
    loading: false,
    error: null,
    hasMoreMessages: true,
  });

  // UI State
  const showMemberManagement = ref(false);
  const showSearchModal = ref(false);
  const showChatSettings = ref(false);
  const showWorkspaceSettings = ref(false);
  const showUserMenu = ref(false);
  const editChatName = ref('');
  const editChatDescription = ref('');

  // Rate limiting
  let lastMessageTime = 0;

  // Computed
  const currentChatId = computed(() => {
    const id = route.params.id;
    return typeof id === 'string' ? parseInt(id, 10) : null;
  });

  const currentChat = computed(() => state.value.currentChat);
  const messages = computed(() => state.value.messages);
  const chatMembers = computed(() => state.value.chatMembers);
  const isLoading = computed(() => state.value.loading);
  const chatError = computed(() => state.value.error);

  const chatOwner = computed(() => {
    if (!currentChat.value || !chatMembers.value.length) return null;
    return chatMembers.value.find(member => member.id === currentChat.value!.owner_id);
  });

  const canDeleteChat = computed(() => {
    return currentChat.value &&
      authStore.user?.id === currentChat.value.owner_id;
  });

  const canSendMessage = computed(() => {
    return state.value.state === 'READY' && currentChatId.value;
  });

  // State machine transition
  const transition = (event: ChatRoomEvent): void => {
    const currentState = state.value.state;

    console.log(`ChatRoom transition: ${currentState} + ${event.type} = ?`);

    switch (currentState) {
      case 'INIT':
        if (event.type === 'LOAD') {
          state.value.state = 'LOADING';
          loadChatData(event.payload.chatId);
        }
        break;

      case 'LOADING':
        if (event.type === 'LOADED') {
          state.value.state = 'READY';
        } else if (event.type === 'ERROR') {
          state.value.state = 'READY';
          state.value.error = event.payload.message;
          toast.error(event.payload.message);
        }
        break;

      case 'READY':
        if (event.type === 'SEND_MESSAGE') {
          state.value.state = 'SENDING';
          sendMessage(event.payload);
        } else if (event.type === 'SCROLL') {
          state.value.state = 'SCROLLING';
        } else if (event.type === 'LOAD') {
          state.value.state = 'LOADING';
          loadChatData(event.payload.chatId);
        }
        break;

      case 'SENDING':
        if (event.type === 'MESSAGE_SENT') {
          state.value.state = 'READY';
        } else if (event.type === 'ERROR') {
          state.value.state = 'READY';
          state.value.error = event.payload.message;
          toast.error(event.payload.message);
        }
        break;

      case 'SCROLLING':
        if (event.type === 'SCROLL_COMPLETE') {
          state.value.state = 'READY';
        }
        break;

      default:
        if (event.type === 'CLOSE') {
          state.value.state = 'CLOSED';
        }
    }
  };

  // Load chat data
  const loadChatData = async (chatId: string): Promise<void> => {
    if (!chatId) return;

    try {
      state.value.loading = true;
      state.value.error = null;

      // Clear previous state
      state.value.messages = [];
      state.value.chatMembers = [];
      state.value.currentChat = null;

      // Load chat data in parallel
      const [chat, messagesData, members] = await Promise.all([
        chatService.getChat(chatId),
        chatService.getChatMessages(chatId, MESSAGE_LIMIT),
        chatService.getChatMembers(chatId),
      ]);

      state.value.currentChat = chat;
      state.value.messages = messagesData.messages;
      state.value.chatMembers = members;
      state.value.hasMoreMessages = messagesData.hasMore;

      // Update edit form
      editChatName.value = chat.name;
      editChatDescription.value = chat.description || '';

      transition({ type: 'LOADED' });
    } catch (error) {
      console.error('Failed to load chat data:', error);
      transition({
        type: 'ERROR',
        payload: { message: 'Failed to load chat data' }
      });
    } finally {
      state.value.loading = false;
    }
  };

  // Send message with validation and rate limiting
  const sendMessage = async (payload: SendMessagePayload): Promise<void> => {
    const { content, files } = payload;

    // Validation
    if (!content.trim() && (!files || files.length === 0)) {
      toast.warning('Please enter a message or attach a file');
      transition({ type: 'ERROR', payload: { message: 'Empty message' } });
      return;
    }

    if (content.length > MAX_MESSAGE_LENGTH) {
      toast.error(`Message is too long. Maximum ${MAX_MESSAGE_LENGTH} characters allowed.`);
      transition({ type: 'ERROR', payload: { message: 'Message too long' } });
      return;
    }

    // Rate limiting
    const now = Date.now();
    if (now - lastMessageTime < MIN_MESSAGE_INTERVAL) {
      toast.warning('Please wait before sending another message');
      transition({ type: 'ERROR', payload: { message: 'Rate limited' } });
      return;
    }

    if (!currentChatId.value) {
      transition({ type: 'ERROR', payload: { message: 'No chat selected' } });
      return;
    }

    try {
      const message = await chatService.sendMessage(
        currentChatId.value.toString(),
        content.trim(),
        files
      );

      // Add message to local state
      state.value.messages.push(message);
      lastMessageTime = now;

      toast.success('Message sent');
      transition({ type: 'MESSAGE_SENT' });
    } catch (error) {
      console.error('Failed to send message:', error);
      transition({
        type: 'ERROR',
        payload: { message: 'Failed to send message. Please try again.' }
      });
    }
  };

  // Load more messages (pagination)
  const loadMoreMessages = async (): Promise<void> => {
    if (!currentChatId.value || !state.value.hasMoreMessages || state.value.loading) {
      return;
    }

    try {
      state.value.loading = true;
      const oldestMessage = state.value.messages[0];
      const beforeId = oldestMessage?.id;

      const messagesData = await chatService.getChatMessages(
        currentChatId.value.toString(),
        MESSAGE_LIMIT,
        beforeId
      );

      // Prepend older messages
      state.value.messages = [...messagesData.messages, ...state.value.messages];
      state.value.hasMoreMessages = messagesData.hasMore;
    } catch (error) {
      console.error('Failed to load more messages:', error);
      toast.error('Failed to load more messages');
    } finally {
      state.value.loading = false;
    }
  };

  // Chat management actions
  const updateChat = async (): Promise<void> => {
    if (!currentChatId.value) return;

    try {
      const updatedChat = await chatService.updateChat(
        currentChatId.value.toString(),
        editChatName.value.trim(),
        editChatDescription.value.trim()
      );

      state.value.currentChat = updatedChat;
      showChatSettings.value = false;
      toast.success('Chat updated successfully');
    } catch (error) {
      console.error('Failed to update chat:', error);
      toast.error('Failed to update chat');
    }
  };

  const deleteChat = async (): Promise<void> => {
    if (!currentChat.value) return;

    const confirmed = confirm(`Are you sure you want to delete "${currentChat.value.name}"? This action cannot be undone.`);
    if (!confirmed) return;

    try {
      await chatService.deleteChat(currentChatId.value!.toString());
      toast.success('Chat deleted successfully');
      router.push('/');
    } catch (error) {
      console.error('Failed to delete chat:', error);
      toast.error('Failed to delete chat');
    }
  };

  const leaveChat = async (): Promise<void> => {
    if (!currentChat.value) return;

    const confirmed = confirm('Are you sure you want to leave this chat?');
    if (!confirmed) return;

    try {
      await chatService.leaveChat(currentChatId.value!.toString());
      toast.success('Left chat successfully');
      router.push('/');
    } catch (error) {
      console.error('Failed to leave chat:', error);
      toast.error('Failed to leave chat');
    }
  };

  // Search functionality
  const searchMessages = async (query: string): Promise<ChatMessage[]> => {
    if (!currentChatId.value) return [];

    try {
      return await chatService.searchMessages(currentChatId.value.toString(), query);
    } catch (error) {
      console.error('Failed to search messages:', error);
      toast.error('Search failed');
      return [];
    }
  };

  // UI action handlers
  const handleSendMessage = (payload: SendMessagePayload): void => {
    if (!canSendMessage.value) return;
    transition({ type: 'SEND_MESSAGE', payload });
  };

  const handleScrollStart = (): void => {
    transition({ type: 'SCROLL' });
  };

  const handleScrollComplete = (): void => {
    transition({ type: 'SCROLL_COMPLETE' });
  };

  // Watchers
  watch(currentChatId, (newChatId) => {
    if (newChatId) {
      transition({ type: 'LOAD', payload: { chatId: newChatId.toString() } });
    }
  }, { immediate: true });

  // Lifecycle
  onMounted(async () => {
    // Check authentication
    if (!authStore.token) {
      router.push('/login');
      return;
    }

    // Initialize workspace
    try {
      await workspaceStore.initializeWorkspace();
    } catch (error) {
      console.error('Failed to initialize workspace:', error);
    }
  });

  onUnmounted(() => {
    transition({ type: 'CLOSE' });
  });

  return {
    // State
    state: readonly(state),
    currentChat: readonly(currentChat),
    messages: readonly(messages),
    chatMembers: readonly(chatMembers),
    isLoading: readonly(isLoading),
    chatError: readonly(chatError),
    chatOwner: readonly(chatOwner),
    canDeleteChat: readonly(canDeleteChat),
    canSendMessage: readonly(canSendMessage),

    // UI State
    showMemberManagement,
    showSearchModal,
    showChatSettings,
    showWorkspaceSettings,
    showUserMenu,
    editChatName,
    editChatDescription,

    // Actions
    handleSendMessage,
    handleScrollStart,
    handleScrollComplete,
    loadMoreMessages,
    updateChat,
    deleteChat,
    leaveChat,
    searchMessages,

    // State machine
    transition,
  };
}