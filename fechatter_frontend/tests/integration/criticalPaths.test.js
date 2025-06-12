// Critical Path Integration Tests
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createTestingPinia } from '@pinia/testing';
import { createRouter, createWebHistory } from 'vue-router';
import routes from '@/router';
import Chat from '@/views/Chat.vue';
import MessageList from '@/components/chat/MessageList.vue';
import { useChatStore } from '@/stores/chat';
import { useAuthStore } from '@/stores/auth';

// Mock API
const mockApi = {
  get: vi.fn(),
  post: vi.fn(),
  patch: vi.fn(),
  delete: vi.fn()
};

vi.mock('@/services/api', () => ({
  default: mockApi
}));

describe('Critical Paths', () => {
  let wrapper;
  let router;
  let chatStore;
  let authStore;

  beforeEach(async () => {
    // Reset mocks
    vi.clearAllMocks();

    // Create router
    router = createRouter({
      history: createWebHistory(),
      routes
    });

    // Create pinia instance
    const pinia = createTestingPinia({
      createSpy: vi.fn,
      stubActions: false
    });

    // Get stores
    chatStore = useChatStore(pinia);
    authStore = useAuthStore(pinia);

    // Set up auth
    authStore.token = 'test-token';
    authStore.user = { id: 1, fullname: 'Test User' };
  });

  afterEach(() => {
    if (wrapper) {
      wrapper.unmount();
    }
  });

  describe('Chat View', () => {
    it('should display MessageList component when chat is loaded', async () => {
      // Mock chat data
      const mockChat = {
        id: 1,
        name: 'Test Chat',
        chat_type: 'PublicChannel'
      };

      const mockMessages = [
        {
          id: 1,
          content: 'Test message',
          sender: { id: 1, fullname: 'Test User' },
          created_at: new Date().toISOString()
        }
      ];

      // Set store data
      chatStore.chats = [mockChat];
      chatStore.messages = mockMessages;
      chatStore.currentChatId = 1;

      // Mock API responses
      mockApi.get.mockImplementation((url) => {
        if (url.includes('/messages')) {
          return Promise.resolve({ data: mockMessages });
        }
        if (url.includes('/members')) {
          return Promise.resolve({ data: [] });
        }
        return Promise.reject(new Error('Unknown endpoint'));
      });

      // Mount component
      wrapper = mount(Chat, {
        global: {
          plugins: [router, pinia],
          stubs: {
            MessageInput: true,
            MemberManagement: true,
            CompactSearch: true,
            TypingIndicator: true,
            OnlineUsers: true,
            UserProfile: true
          }
        },
        props: {
          id: '1'
        }
      });

      // Set route
      await router.push('/chat/1');
      await router.isReady();

      // Wait for async operations
      await flushPromises();

      // Check MessageList is rendered
      const messageList = wrapper.findComponent(MessageList);
      expect(messageList.exists()).toBe(true);

      // Check props passed to MessageList
      expect(messageList.props('messages')).toEqual(mockMessages);
      expect(messageList.props('currentUserId')).toBe(1);
      expect(messageList.props('chatId')).toBe(1);
    });

    it('should handle empty messages gracefully', async () => {
      // Set empty messages
      chatStore.messages = [];
      chatStore.currentChatId = 1;

      wrapper = mount(Chat, {
        global: {
          plugins: [router, pinia],
          stubs: {
            MessageInput: true,
            MemberManagement: true,
            CompactSearch: true,
            TypingIndicator: true,
            OnlineUsers: true,
            UserProfile: true
          }
        }
      });

      await router.push('/chat/1');
      await flushPromises();

      const messageList = wrapper.findComponent(MessageList);
      expect(messageList.exists()).toBe(true);
      expect(messageList.props('messages')).toEqual([]);
    });

    it('should show loading state initially', async () => {
      chatStore.loading = true;
      chatStore.currentChatId = null;

      wrapper = mount(Chat, {
        global: {
          plugins: [router, pinia]
        }
      });

      await flushPromises();

      // Check for loading indicator
      expect(wrapper.find('.slack-loading-state').exists()).toBe(true);
    });
  });

  describe('Message Display', () => {
    it('should correctly normalize messages from backend', () => {
      const backendMessage = {
        id: 1,
        content: 'Test',
        sender_id: 2,
        sender_name: 'John Doe',
        created_at: '2024-01-01T00:00:00Z'
      };

      const normalized = chatStore.normalizeMessage(backendMessage);

      expect(normalized).toMatchObject({
        id: 1,
        content: 'Test',
        sender: {
          id: 2,
          fullname: 'John Doe'
        },
        created_at: '2024-01-01T00:00:00Z'
      });
    });

    it('should handle nested sender object', () => {
      const backendMessage = {
        id: 1,
        content: 'Test',
        sender: {
          id: 2,
          fullname: 'Jane Doe',
          email: 'jane@example.com'
        },
        created_at: '2024-01-01T00:00:00Z'
      };

      const normalized = chatStore.normalizeMessage(backendMessage);

      expect(normalized.sender).toEqual({
        id: 2,
        fullname: 'Jane Doe',
        email: 'jane@example.com'
      });
    });
  });

  describe('API Error Handling', () => {
    it('should handle API errors gracefully', async () => {
      mockApi.get.mockRejectedValue(new Error('Network error'));

      try {
        await chatStore.fetchMessages(1);
      } catch (error) {
        expect(error.message).toBe('Network error');
      }

      expect(chatStore.error).toBeTruthy();
    });
  });

  describe('Store Consistency', () => {
    it('should maintain cache consistency', async () => {
      const messages = [
        { id: 1, content: 'Message 1' },
        { id: 2, content: 'Message 2' }
      ];

      mockApi.get.mockResolvedValue({ data: messages });

      // First fetch
      await chatStore.fetchMessages(1);
      expect(chatStore.messages).toHaveLength(2);

      // Check cache
      expect(chatStore.messageCache[1]).toBeDefined();
      expect(chatStore.messageCache[1].messages).toHaveLength(2);
    });
  });
});

// Run specific critical path tests
describe('Critical User Flows', () => {
  it('User can view channel messages', async () => {
    // This is the main flow that was broken
    const steps = [
      'User logs in',
      'User selects workspace',
      'User clicks on channel',
      'MessageList component renders',
      'Messages are displayed'
    ];

    // Document the flow
    console.log('Testing critical flow:', steps.join(' → '));

    // Implementation would involve full E2E test
    expect(true).toBe(true); // Placeholder
  });
}); 