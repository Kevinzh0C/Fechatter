/**
 * Message Confirmation Service
 * 处理消息发送确认和SSE delivery确认
 * 确保消息状态正确更新为delivered状态
 */

import { useChatStore } from '@/stores/chat'

class MessageConfirmationService {
  constructor() {
    this.confirmedMessages = new Set()
    this.pendingConfirmations = new Map()
    this.isEnabled = true
    
    if (import.meta.env.DEV) {
      console.log('📨 Message Confirmation Service initialized')
    }
  }

  /**
   * 模拟SSE确认消息delivery
   * Simulate SSE message delivery confirmation
   */
  simulateMessageDeliveryConfirmation(messageId, chatId, delay = 2000) {
    if (!messageId || !chatId) return

    // 避免重复确认
    if (this.confirmedMessages.has(messageId)) {
      return
    }

    // 标记为待确认
    this.pendingConfirmations.set(messageId, {
      chatId,
      timestamp: Date.now()
    })

    if (import.meta.env.DEV) {
      console.log(`📨 Scheduling delivery confirmation for message ${messageId} in ${delay}ms`)
    }

    // 模拟网络延迟后的SSE确认
    setTimeout(() => {
      this.confirmMessageDelivery(messageId, chatId)
    }, delay)
  }

  /**
   * 确认消息已delivery
   * Confirm message delivery
   */
  confirmMessageDelivery(messageId, chatId) {
    if (!messageId || !chatId) return

    // 避免重复确认
    if (this.confirmedMessages.has(messageId)) {
      if (import.meta.env.DEV) {
        console.log(`📨 Message ${messageId} already confirmed, skipping`)
      }
      return
    }

    try {
      const chatStore = useChatStore()
      
      // 调用chat store的updateRealtimeMessage来更新状态
      const updated = chatStore.updateRealtimeMessage(messageId, {
        status: 'delivered',
        delivered_at: new Date().toISOString(),
        confirmed_via_sse: true
      })

      if (updated) {
        // 标记为已确认
        this.confirmedMessages.add(messageId)
        this.pendingConfirmations.delete(messageId)

        if (import.meta.env.DEV) {
          console.log(`✅ Message ${messageId} delivery confirmed via simulated SSE`)
        }
      } else if (import.meta.env.DEV) {
        console.warn(`⚠️ Could not find message ${messageId} to confirm delivery`)
      }

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error(`❌ Failed to confirm delivery for message ${messageId}:`, error)
      }
    }
  }

  /**
   * 手动触发消息确认（用于测试）
   * Manually trigger message confirmation (for testing)
   */
  manuallyConfirmMessage(messageId, chatId) {
    if (import.meta.env.DEV) {
      console.log(`🔧 Manually confirming message ${messageId}`)
    }
    this.confirmMessageDelivery(messageId, chatId)
  }
}

// 创建单例实例
const messageConfirmationService = new MessageConfirmationService()

// 暴露到window对象用于调试
if (typeof window !== 'undefined' && import.meta.env.DEV) {
  window.msgConfirm = {
    confirm: (messageId, chatId) => messageConfirmationService.manuallyConfirmMessage(messageId, chatId),
    stats: () => ({ confirmedMessages: messageConfirmationService.confirmedMessages.size })
  }
  console.log('📨 Message Confirmation Service available at window.msgConfirm')
}

export default messageConfirmationService
