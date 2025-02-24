/**
 * Message Tracking Service Implementation
 * Concrete implementation of IMessageTrackingService
 */

import { v4 as uuidv4 } from 'uuid'
import type { IMessageTrackingService, TrackingStatus } from '@/types/message'

export class MessageTrackingService implements IMessageTrackingService {
  private trackingContexts = new Map<string, TrackingContext>()

  startTracking(chatId: number, messageIds: number[]): string {
    const trackingId = uuidv4()

    const context: TrackingContext = {
      trackingId,
      chatId,
      messageIds: new Set(messageIds),
      displayedIds: new Set(),
      startTime: Date.now(),
      status: 'active'
    }

    this.trackingContexts.set(trackingId, context)

    return trackingId
  }

  stopTracking(trackingId: string): void {
    const context = this.trackingContexts.get(trackingId)
    if (context) {
      context.status = 'stopped'
      // Clean up after a delay
      setTimeout(() => {
        this.trackingContexts.delete(trackingId)
      }, 5000)
    }
  }

  getTrackingStatus(trackingId: string): TrackingStatus {
    const context = this.trackingContexts.get(trackingId)
    if (!context) {
      throw new Error(`Tracking context ${trackingId} not found`)
    }

    const totalMessages = context.messageIds.size
    const displayedMessages = context.displayedIds.size
    const percentage = totalMessages > 0 ? (displayedMessages / totalMessages) * 100 : 0

    return {
      chatId: context.chatId,
      totalMessages,
      displayedMessages,
      percentage,
      isComplete: displayedMessages === totalMessages
    }
  }

  markMessageDisplayed(trackingId: string, messageId: number): void {
    const context = this.trackingContexts.get(trackingId)
    if (!context) return

    if (context.messageIds.has(messageId)) {
      context.displayedIds.add(messageId)

      // Check if complete
      if (context.displayedIds.size === context.messageIds.size) {
        context.status = 'completed'
        context.completedTime = Date.now()
      }
    }
  }

  getActiveTrackings(): string[] {
    return Array.from(this.trackingContexts.entries())
      .filter(([_, context]) => context.status === 'active')
      .map(([id]) => id)
  }

  clearCompleted(): void {
    const now = Date.now()
    const maxAge = 5 * 60 * 1000 // 5 minutes

    for (const [id, context] of this.trackingContexts.entries()) {
      if (context.status === 'completed' && context.completedTime) {
        if (now - context.completedTime > maxAge) {
          this.trackingContexts.delete(id)
        }
      }
    }
  }
}

interface TrackingContext {
  trackingId: string
  chatId: number
  messageIds: Set<number>
  displayedIds: Set<number>
  startTime: number
  completedTime?: number
  status: 'active' | 'completed' | 'stopped'
} 