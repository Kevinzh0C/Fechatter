/**
 * Analytics Composable for Vue Components
 * Provides easy access to analytics tracking functionality
 */

import { inject, getCurrentInstance } from 'vue'

export function useAnalytics() {
  // Try to get analytics from injection first
  let analytics = inject('analytics', null)

  // Fallback to global properties
  if (!analytics) {
    const instance = getCurrentInstance()
    analytics = instance?.appContext?.config?.globalProperties?.$analytics
  }

  if (!analytics) {
    console.warn('[Analytics] Analytics client not found. Make sure it\'s properly initialized.')
    // Return mock functions to prevent errors
    return {
      track: () => { },
      trackUserLogin: () => { },
      trackUserLogout: () => { },
      trackMessageSent: () => { },
      trackNavigation: () => { },
      trackError: () => { },
      trackFileUpload: () => { },
      trackSearch: () => { },
      trackChatJoined: () => { },
      trackChatLeft: () => { },
      flush: () => { },
      isEnabled: false
    }
  }

  return {
    // Generic tracking method
    track: (eventType, eventData = {}) => {
      return analytics.track(eventType, eventData)
    },

    // Specific event tracking methods
    trackUserLogin: (email, method = 'password') => {
      return analytics.track('user_login', { email, login_method: method })
    },

    trackUserLogout: (email) => {
      return analytics.track('user_logout', { email })
    },

    trackUserRegister: (email, workspaceId, method = 'email') => {
      return analytics.track('user_register', {
        email,
        workspace_id: workspaceId,
        registration_method: method
      })
    },

    trackMessageSent: (chatId, messageType = 'text', messageSize = 0, options = {}) => {
      return analytics.track('message_sent', {
        chat_id: chatId,
        message_type: messageType,
        message_size: messageSize,
        message_total_files: options.totalFiles || 0,
        message_has_mentions: options.hasMentions || false,
        message_has_links: options.hasLinks || false
      })
    },

    trackChatCreated: (workspaceId, chatType = 'group', membersCount = 1) => {
      return analytics.track('chat_created', {
        workspace_id: workspaceId,
        chat_type: chatType,
        initial_members_count: membersCount
      })
    },

    trackChatJoined: (chatId, method = 'invited') => {
      return analytics.track('chat_joined', {
        chat_id: chatId,
        join_method: method
      })
    },

    trackChatLeft: (chatId, reason = 'voluntary') => {
      return analytics.track('chat_left', {
        chat_id: chatId,
        leave_reason: reason
      })
    },

    trackNavigation: (from, to, duration = 0) => {
      return analytics.track('navigation', {
        from,
        to,
        duration_ms: duration
      })
    },

    trackFileUpload: (fileType, fileSize, method = 'button', duration = 0) => {
      return analytics.track('file_uploaded', {
        file_type: fileType,
        file_size: fileSize,
        upload_method: method,
        upload_duration_ms: duration
      })
    },

    trackFileDownload: (fileType, fileSize, duration = 0) => {
      return analytics.track('file_downloaded', {
        file_type: fileType,
        file_size: fileSize,
        download_duration_ms: duration
      })
    },

    trackSearch: (searchType, queryLength, resultsCount, duration = 0, hasFilters = false) => {
      return analytics.track('search_performed', {
        search_type: searchType,
        query_length: queryLength,
        results_count: resultsCount,
        search_duration_ms: duration,
        has_filters: hasFilters
      })
    },

    trackNotification: (type, source, wasClicked = false) => {
      return analytics.track('notification_received', {
        notification_type: type,
        source,
        was_clicked: wasClicked
      })
    },

    trackError: (errorType, errorCode, errorMessage, stackTrace = '', context = '') => {
      return analytics.track('error_occurred', {
        error_type: errorType,
        error_code: errorCode,
        error_message: errorMessage,
        stack_trace: stackTrace,
        context
      })
    },

    trackBotResponse: (botId, chatId, responseType, responseTime, tokensUsed = 0, success = true, errorMessage = '') => {
      return analytics.track('bot_response', {
        bot_id: botId,
        chat_id: chatId,
        response_type: responseType,
        response_time_ms: responseTime,
        tokens_used: tokensUsed,
        success,
        error_message: errorMessage
      })
    },

    // Utility methods
    flush: () => {
      return analytics.flush()
    },

    isEnabled: analytics.enabled,

    // Get analytics client instance (for advanced usage)
    getClient: () => analytics
  }
} 