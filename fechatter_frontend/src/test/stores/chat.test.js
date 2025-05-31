import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useChatStore } from '../../stores/chat';
import axios from 'axios';
import { vi } from 'vitest';

vi.mock('axios');

describe('Chat Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  describe('fetchChats', () => {
    it('should fetch chats successfully', async () => {
      const store = useChatStore();
      const mockChats = [
        { id: 1, name: 'Chat 1' },
        { id: 2, name: 'Chat 2' }
      ];
      axios.get.mockResolvedValueOnce({ data: mockChats });

      await store.fetchChats();

      expect(store.chats).toEqual(mockChats);
      expect(store.error).toBeNull();
    });

    it('should handle fetch error', async () => {
      const store = useChatStore();
      axios.get.mockRejectedValueOnce({ response: { data: { message: 'Failed to fetch' } } });

      await store.fetchChats();

      expect(store.error).toBe('Failed to fetch');
      expect(store.chats).toEqual([]);
    });
  });

  describe('createChat', () => {
    it('should create chat successfully', async () => {
      const store = useChatStore();
      const mockChat = { id: 1, name: 'New Chat' };
      axios.post.mockResolvedValueOnce({ data: mockChat });

      const result = await store.createChat('New Chat', [1, 2]);

      expect(result).toEqual(mockChat);
      expect(store.chats).toContainEqual(mockChat);
    });
  });

  describe('sendMessage', () => {
    it('should send message successfully', async () => {
      const store = useChatStore();
      const mockMessage = { id: 1, content: 'Hello' };
      axios.post.mockResolvedValueOnce({ data: mockMessage });

      const result = await store.sendMessage(1, 'Hello');

      expect(result).toEqual(mockMessage);
      expect(store.messages).toContainEqual(mockMessage);
    });

    it('should include idempotency key', async () => {
      const store = useChatStore();
      await store.sendMessage(1, 'Hello');

      expect(axios.post).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({
          idempotency_key: expect.any(String)
        })
      );
    });
  });

  describe('uploadFiles', () => {
    it('should upload files successfully', async () => {
      const store = useChatStore();
      const mockUrls = ['/files/1/test.jpg'];
      axios.post.mockResolvedValueOnce({ data: mockUrls });

      const files = [new File([''], 'test.jpg')];
      const result = await store.uploadFiles(files);

      expect(result).toEqual(mockUrls);
    });
  });
});