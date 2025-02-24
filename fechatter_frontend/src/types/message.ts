/**
 * Message System Type Definitions
 * Following TypeScript best practices for type safety
 */

// Core message interface
export interface Message {
  id: number
  content: string
  senderId: number
  senderName?: string
  chatId: number
  createdAt: string
  updatedAt?: string
  status: MessageStatus
  files?: MessageFile[]
  replyTo?: number
  mentions?: number[]
}

export enum MessageStatus {
  PENDING = 'pending',
  SENT = 'sent',
  DELIVERED = 'delivered',
  READ = 'read',
  FAILED = 'failed'
}

export interface MessageFile {
  id: string
  url: string
  filename: string
  mimeType: string
  size: number
}

// Scroll position tracking
export interface ScrollPosition {
  scrollTop: number
  scrollHeight: number
  clientHeight: number
  messageId?: number
}

// Message visibility tracking
export interface MessageVisibility {
  messageId: number
  isVisible: boolean
  visiblePercentage: number
  element?: HTMLElement
}

// Component props interfaces
export interface MessageListProps {
  messages: Message[]
  chatId: number
  currentUserId: number
  loading?: boolean
  hasMore?: boolean
}

export interface MessageListEmits {
  'message-displayed': [messageId: number]
  'scroll-changed': [position: ScrollPosition]
  'load-more': []
  'user-profile-opened': [userId: number]
  'dm-created': [chatId: number]
  'auto-load-more': []
}

// 🔥 NEW: Extended props for enhanced message list
export interface ExtendedMessageListProps extends MessageListProps {
  autoLoadIndicatorVisible?: boolean
  isNearTop?: boolean
  canAutoLoad?: boolean
  scrollPosition?: ScrollPosition | null
  showDebugInfo?: boolean
}

// 🔥 NEW: Load more trigger types
export type LoadMoreTriggerType = 'manual' | 'auto' | 'scroll'

// 🔥 NEW: Load more state interface
export interface LoadMoreState {
  isLoadingMore: boolean
  indicatorVisible: boolean
  canAutoLoad: boolean
  lastTriggerTime?: number
  triggerType?: LoadMoreTriggerType
}

// 🔥 NEW: Auto load more event detail
export interface AutoLoadMoreEventDetail {
  chatId: number
  triggerType: LoadMoreTriggerType
  scrollPosition?: ScrollPosition
}

// Service interfaces
export interface IMessageDisplayService {
  markDisplayed(messageId: number): void
  isVisible(element: HTMLElement): boolean
  getVisibilityPercentage(element: HTMLElement): number
}

export interface IScrollManagerService {
  scrollToBottom(smooth?: boolean): void
  scrollToMessage(messageId: number): void
  preservePosition(): ScrollPosition
  restorePosition(position: ScrollPosition): void
}

export interface IMessageTrackingService {
  startTracking(chatId: number, messageIds: number[]): string
  stopTracking(trackingId: string): void
  getTrackingStatus(trackingId: string): TrackingStatus
}

export interface TrackingStatus {
  chatId: number
  totalMessages: number
  displayedMessages: number
  percentage: number
  isComplete: boolean
}

// Store state interfaces
export interface MessageStoreState {
  messagesByChat: Map<number, Message[]>
  loading: boolean
  error: string | null
  hasMoreByChat: Map<number, boolean>
}

export interface ViewportStoreState {
  scrollPositions: Map<number, ScrollPosition>
  visibleMessages: Map<number, Set<number>>
  activeViewport: number | null
} 