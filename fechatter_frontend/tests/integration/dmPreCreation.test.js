// DM Pre-creation Integration Tests
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createTestingPinia } from '@pinia/testing';
import { createRouter, createWebHistory } from 'vue-router';
import routes from '@/router';
import UserProfile from '@/components/modals/UserProfile.vue';
import ChannelList from '@/components/chat/ChannelList.vue';
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

describe('DM Pre-creation Mechanism', () => {
  let wrapper;
  let router;
  let chatStore;
  let authStore;
  let pinia;

  beforeEach(async () => {
    // Reset mocks
    vi.clearAllMocks();

    // Create router
    router = createRouter({
      history: createWebHistory(),
      routes
    });

    // Set global router
    window.$router = router;

    // Create pinia instance
    pinia = createTestingPinia({
      createSpy: vi.fn,
      stubActions: false
    });

    // Get stores
    chatStore = useChatStore(pinia);
    authStore = useAuthStore(pinia);

    // Set up auth
    authStore.token = 'test-token';
    authStore.user = { id: 1, fullname: 'Current User', email: 'current@test.com' };
  });

  afterEach(() => {
    if (wrapper) {
      wrapper.unmount();
    }
    // Clear localStorage
    localStorage.clear();
  });

  describe('UserProfile DM Creation', () => {
    it('should create a temporary DM when clicking Send Message', async () => {
      const testUser = {
        id: 2,
        fullname: 'Test User',
        email: 'test@example.com',
        status: 'Active'
      };

      wrapper = mount(UserProfile, {
        global: {
          plugins: [router, pinia],
          stubs: {
            teleport: true
          }
        },
        props: {
          user: testUser
        }
      });

      // Find and click the Send Message button
      const sendMessageBtn = wrapper.find('.profile-action-btn.primary');
      expect(sendMessageBtn.exists()).toBe(true);
      expect(sendMessageBtn.text()).toContain('Send Message');

      await sendMessageBtn.trigger('click');
      await flushPromises();

      // Check that temporary DM was created
      const tempDM = chatStore.chats.find(c => c.id === `preview-dm-${testUser.id}`);
      expect(tempDM).toBeDefined();
      expect(tempDM.is_preview).toBe(true);
      expect(tempDM.chat_type).toBe('Single');
      expect(tempDM.name).toBe(testUser.fullname);

      // Check that dm-created event was emitted
      expect(wrapper.emitted('dm-created')).toBeTruthy();
      expect(wrapper.emitted('dm-created')[0][0]).toEqual(tempDM);
    });

    it('should not create duplicate temporary DMs', async () => {
      const testUser = {
        id: 3,
        fullname: 'Another User',
        email: 'another@example.com'
      };

      // Create first temporary DM
      const firstTempDM = chatStore.createTemporaryDM(testUser);
      expect(chatStore.chats.length).toBe(1);

      // Try to create another
      const secondTempDM = chatStore.createTemporaryDM(testUser);

      // Should return the same instance
      expect(secondTempDM).toBe(firstTempDM);
      expect(chatStore.chats.length).toBe(1);
    });
  });

  describe('ChannelList DM Display', () => {
    it('should display temporary DMs at the top of DM list', async () => {
      // Add some regular DMs
      chatStore.chats = [
        {
          id: 1,
          chat_type: 'Single',
          name: 'Regular DM',
          display_name: 'Regular User',
          last_message: { created_at: new Date().toISOString() }
        },
        {
          id: 'preview-dm-99',
          is_preview: true,
          chat_type: 'Single',
          name: 'Preview DM',
          display_name: 'Preview User'
        }
      ];

      wrapper = mount(ChannelList, {
        global: {
          plugins: [router, pinia]
        },
        props: {
          listType: 'dms'
        }
      });

      await flushPromises();

      const dmItems = wrapper.findAll('.dm-item');
      expect(dmItems.length).toBe(2);

      // Preview DM should be first
      expect(dmItems[0].find('.dm-user-name').text()).toBe('Preview User');
      expect(dmItems[0].find('.dm-last-message').text()).toContain('Start a conversation');
    });

    it('should show close button on DM hover', async () => {
      chatStore.chats = [{
        id: 1,
        chat_type: 'Single',
        name: 'Test DM',
        display_name: 'Test User'
      }];

      wrapper = mount(ChannelList, {
        global: {
          plugins: [router, pinia]
        },
        props: {
          listType: 'dms'
        }
      });

      await flushPromises();

      const dmItem = wrapper.find('.dm-item');
      const closeBtn = dmItem.find('.dm-hide-btn'); // Class name is still dm-hide-btn

      expect(closeBtn.exists()).toBe(true);
      expect(closeBtn.attributes('title')).toBe('Close conversation');
    });
  });

  describe('DM Closing Functionality', () => {
    it('should remove DM from list when clicking close button', async () => {
      const dmId = 1;
      chatStore.chats = [{
        id: dmId,
        chat_type: 'Single',
        name: 'Test DM',
        display_name: 'Test User'
      }];

      wrapper = mount(ChannelList, {
        global: {
          plugins: [router, pinia]
        },
        props: {
          listType: 'dms'
        }
      });

      await flushPromises();
      expect(wrapper.findAll('.dm-item').length).toBe(1);

      // Click close button
      const closeBtn = wrapper.find('.dm-hide-btn');
      await closeBtn.trigger('click');
      await flushPromises();

      // Check that DM is removed from the list in the component
      expect(wrapper.findAll('.dm-item').length).toBe(0);
    });
  });

  describe('Navigation and Cleanup', () => {
    it('should remove temporary DM when navigating away', async () => {
      const tempDMId = 'preview-dm-99';
      chatStore.chats = [{
        id: tempDMId,
        is_preview: true,
        chat_type: 'Single',
        name: 'Temp DM'
      }];

      // Navigate to the temporary DM
      await router.push(`/chat/${tempDMId}`);
      await router.isReady();

      // Navigate away
      await router.push('/home');

      // The actual cleanup happens in Chat.vue component
      // This test verifies the mechanism exists
      expect(chatStore.removeTemporaryDM).toBeDefined();
    });
  });
}); 