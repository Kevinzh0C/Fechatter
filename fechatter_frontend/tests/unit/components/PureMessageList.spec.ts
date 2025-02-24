/**
 * PureMessageList Component Unit Tests
 * Testing the pure presentation component
 */

import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import PureMessageList from '@/components/chat/PureMessageList.vue'
import MessageItem from '@/components/chat/MessageItem.vue'
import type { Message } from '@/types/message'

// Mock messages
const mockMessages: Message[] = [
  {
    id: 1,
    content: 'Hello World',
    senderId: 1,
    senderName: 'John Doe',
    chatId: 1,
    createdAt: '2024-01-01T00:00:00Z',
    status: 'sent'
  },
  {
    id: 2,
    content: 'How are you?',
    senderId: 2,
    senderName: 'Jane Smith',
    chatId: 1,
    createdAt: '2024-01-01T00:01:00Z',
    status: 'sent'
  }
]

describe('PureMessageList', () => {
  it('renders messages correctly', () => {
    const wrapper = mount(PureMessageList, {
      props: {
        messages: mockMessages,
        chatId: 1,
        currentUserId: 1,
        loading: false,
        hasMore: false
      },
      global: {
        stubs: {
          MessageItem: true
        }
      }
    })

    const messageWrappers = wrapper.findAll('.message-wrapper')
    expect(messageWrappers).toHaveLength(2)
    expect(messageWrappers[0].attributes('data-message-id')).toBe('1')
    expect(messageWrappers[1].attributes('data-message-id')).toBe('2')
  })

  it('shows loading indicator when loading', () => {
    const wrapper = mount(PureMessageList, {
      props: {
        messages: [],
        chatId: 1,
        currentUserId: 1,
        loading: true,
        hasMore: false
      }
    })

    expect(wrapper.find('.loading-indicator').exists()).toBe(true)
    expect(wrapper.find('.loading-text').text()).toBe('Loading messages...')
  })

  it('shows load more button when hasMore is true', () => {
    const wrapper = mount(PureMessageList, {
      props: {
        messages: mockMessages,
        chatId: 1,
        currentUserId: 1,
        loading: false,
        hasMore: true
      }
    })

    const loadMoreBtn = wrapper.find('.load-more-btn')
    expect(loadMoreBtn.exists()).toBe(true)
    expect(loadMoreBtn.text()).toBe('Load More Messages')
  })

  it('emits load-more event when button clicked', async () => {
    const wrapper = mount(PureMessageList, {
      props: {
        messages: mockMessages,
        chatId: 1,
        currentUserId: 1,
        loading: false,
        hasMore: true
      }
    })

    await wrapper.find('.load-more-btn').trigger('click')
    expect(wrapper.emitted('load-more')).toBeTruthy()
    expect(wrapper.emitted('load-more')).toHaveLength(1)
  })

  it('emits message-displayed when message element is registered', () => {
    const wrapper = mount(PureMessageList, {
      props: {
        messages: mockMessages,
        chatId: 1,
        currentUserId: 1
      }
    })

    // Component should emit message-displayed for each message
    const emitted = wrapper.emitted('message-displayed')
    expect(emitted).toBeTruthy()
  })

  it('emits scroll-changed when container scrolls', async () => {
    const wrapper = mount(PureMessageList, {
      props: {
        messages: mockMessages,
        chatId: 1,
        currentUserId: 1
      }
    })

    const scrollContainer = wrapper.find('.pure-message-list')
    await scrollContainer.trigger('scroll')

    const emitted = wrapper.emitted('scroll-changed')
    expect(emitted).toBeTruthy()
    expect(emitted![0][0]).toHaveProperty('scrollTop')
    expect(emitted![0][0]).toHaveProperty('scrollHeight')
    expect(emitted![0][0]).toHaveProperty('clientHeight')
  })

  it('shows empty state when no messages and not loading', () => {
    const wrapper = mount(PureMessageList, {
      props: {
        messages: [],
        chatId: 1,
        currentUserId: 1,
        loading: false,
        hasMore: false
      }
    })

    expect(wrapper.find('.empty-state').exists()).toBe(true)
    expect(wrapper.find('.empty-state').text()).toContain('No messages yet')
  })

  it('disables load more button when loading', () => {
    const wrapper = mount(PureMessageList, {
      props: {
        messages: mockMessages,
        chatId: 1,
        currentUserId: 1,
        loading: true,
        hasMore: true
      }
    })

    const loadMoreBtn = wrapper.find('.load-more-btn')
    expect(loadMoreBtn.attributes('disabled')).toBeDefined()
  })

  it('exposes scrollContainer ref', () => {
    const wrapper = mount(PureMessageList, {
      props: {
        messages: mockMessages,
        chatId: 1,
        currentUserId: 1
      }
    })

    const exposed = wrapper.vm as any
    expect(exposed.scrollContainer).toBeDefined()
  })

  it('passes correct props to MessageItem components', () => {
    const wrapper = mount(PureMessageList, {
      props: {
        messages: mockMessages,
        chatId: 1,
        currentUserId: 1
      }
    })

    const messageItems = wrapper.findAllComponents(MessageItem)
    expect(messageItems[0].props()).toMatchObject({
      message: mockMessages[0],
      currentUserId: 1,
      chatId: 1
    })
  })
}) 