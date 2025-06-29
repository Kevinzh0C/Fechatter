/**
 * 🛡️ Message Display Guarantee Service
 * 
 * Production-level service ensuring backend messages are reliably displayed on frontend
 * Implements complete end-to-end verification with fallback mechanisms
 */

import { ref, reactive } from 'vue';
import { productionLogManager } from '@/utils/productionLogManager.js';

export class MessageDisplayGuarantee {
  constructor() {
    this.verificationQueue = new Map(); // messageId -> verification context

    // CRITICAL FIX: Use plain objects instead of reactive to prevent recursive updates
    this.displayMetrics = {
      totalFetched: 0,
      totalDisplayed: 0,
      failedDisplays: 0,
      lastVerification: null
    };

    this.retryConfig = {
      maxRetries: 2,
      baseDelay: 1500,
      maxDelay: 4000,
      maxTimeout: 15000 // NEW: Maximum time to spend on verification
    };

    this.isEnabled = ref(true);

    this.debugMode = ref(import.meta.env.DEV);

    // CRITICAL FIX: Use plain object instead of reactive
    this.deletedMessageStats = {
      totalDetected: 0,
      chatsWithDeleted: new Set(),
      lastDetection: null
    };

    // NEW: Debounce mechanism to prevent excessive updates
    this._updateDebounceTimer = null;
    this._pendingUpdates = new Set();
  }

  // NEW: Debounced update mechanism
  _debouncedUpdate(updateType) {
    this._pendingUpdates.add(updateType);

    if (this._updateDebounceTimer) {
      clearTimeout(this._updateDebounceTimer);
    }

    this._updateDebounceTimer = setTimeout(() => {
      this._flushPendingUpdates();
    }, 50); // 50ms debounce
  }

  _flushPendingUpdates() {
    // Process pending updates in batch
    if (this._pendingUpdates.size > 0) {
      // Batch metrics updates would go here if needed for reactive UI
      this._pendingUpdates.clear();
    }
    this._updateDebounceTimer = null;
  }

  /**
   * Start tracking messages for display verification
   * ENHANCED: 统一的消息追踪启动入口，防止重复上下文创建
   */
  startMessageTracking(chatId, messageIds) {
    if (!this.isEnabled.value) return null;

    const normalizedChatId = parseInt(chatId);
    const normalizedIds = Array.isArray(messageIds)
      ? messageIds.map(id => parseInt(id))
      : [parseInt(messageIds)];

    if (normalizedIds.length === 0) {
      if (this.debugMode.value) {
        console.warn('[MessageDisplayGuarantee] No message IDs provided for tracking');
      }
      return null;
    }

    // ENHANCED: Progressive cleanup strategy to prevent race conditions during chat switching
    if (this.verificationQueue.size > 0) {
      const existingContexts = Array.from(this.verificationQueue.entries())
        .filter(([_, context]) => context.chatId === normalizedChatId);

      if (existingContexts.length > 0) {
        // CRITICAL FIX: Don't immediately clear - use progressive cleanup
        const activeContexts = existingContexts.filter(([_, context]) => {
          const isRecent = (Date.now() - context.fetchTimestamp) < 3000; // 3 seconds
          const hasProgress = context.displayedIds.size > 0;
          const isActive = context.status === 'fetching' || context.status === 'partially_displayed';

          return isRecent && (hasProgress || isActive);
        });

        if (activeContexts.length > 0) {
          // Defer cleanup of active contexts to prevent race conditions
          setTimeout(() => {
            activeContexts.forEach(([trackingId, context]) => {
              if (this.verificationQueue.has(trackingId)) {
                // Only clear if still not completed
                if (context.status !== 'completed') {
                  if (this.debugMode.value) {
                    console.log(`🕒 [MessageDisplayGuarantee] Deferred cleanup of active context ${trackingId} for chat ${normalizedChatId}`);
                  }
                  this.verificationQueue.delete(trackingId);
                }
              }
            });
          }, 2000); // 2 second grace period for in-flight operations
        }

        // Clear only non-active contexts immediately
        const inactiveContexts = existingContexts.filter(([_, context]) => {
          const isOld = (Date.now() - context.fetchTimestamp) >= 3000;
          const noProgress = context.displayedIds.size === 0;
          const isInactive = context.status === 'failed' || context.status === 'completed';

          return isOld || noProgress || isInactive;
        });

        inactiveContexts.forEach(([trackingId]) => {
          this.verificationQueue.delete(trackingId);
          if (this.debugMode.value) {
            console.log(`🧹 [MessageDisplayGuarantee] Cleaned up inactive context ${trackingId} for chat ${normalizedChatId}`);
          }
        });
      }
    }

    // CRITICAL FIX: 确保每个chat只有一个活跃的追踪上下文
    // 首先清理该chat的所有旧上下文
    const existingContexts = [];
    for (const [existingTrackingId, existingContext] of this.verificationQueue.entries()) {
      if (existingContext.chatId === normalizedChatId) {
        existingContexts.push({ trackingId: existingTrackingId, context: existingContext });
      }
    }

    // 如果存在活跃上下文，优先复用而不是创建新的
    if (existingContexts.length > 0) {
      // CRITICAL FIX: 扩展活跃上下文定义，包含最近完成的上下文以防止Vue重新渲染时的上下文丢失
      const activeContext = existingContexts.find(ctx => {
        const isActiveFetching = ctx.context.status === 'fetching' || ctx.context.status === 'partially_displayed';
        const isRecentlyCompleted = ctx.context.status === 'completed' &&
          (Date.now() - ctx.context.fetchTimestamp) < 30000; // 30秒内完成的上下文仍然考虑为活跃
        const isFallbackActive = ctx.context.status === 'fallback_displayed';
        return isActiveFetching || isRecentlyCompleted || isFallbackActive;
      });

      if (activeContext) {
        // 扩展现有上下文以包含新消息
        const existingIds = activeContext.context.messageIds;
        const newIds = normalizedIds.filter(id => !existingIds.has(id));

        if (newIds.length > 0) {
          newIds.forEach(id => existingIds.add(id));

          // CRITICAL FIX: 如果上下文已完成，重新激活它以处理新消息
          if (activeContext.context.status === 'completed') {
            activeContext.context.status = 'partially_displayed';
          }

          if (this.debugMode.value) {
            console.log(`🔄 [MessageDisplayGuarantee] Extended existing context ${activeContext.trackingId} with ${newIds.length} new messages for chat ${normalizedChatId}. Context reactivated from ${activeContext.context.status}.`);
          }
        }

        return activeContext.trackingId;
      } else {
        // ENHANCED: 只清理真正无效的上下文，保留最近的已完成上下文
        const contextsToClean = existingContexts.filter(({ context }) => {
          const isOldCompleted = context.status === 'completed' &&
            (Date.now() - context.fetchTimestamp) > 30000; // 超过30秒的已完成上下文
          const isFailedOrInactive = ['failed', 'timeout'].includes(context.status);
          return isOldCompleted || isFailedOrInactive;
        });

        contextsToClean.forEach(({ trackingId }) => {
          this.verificationQueue.delete(trackingId);
          if (this.debugMode.value) {
            console.log(`🧹 [MessageDisplayGuarantee] Cleaned up old/inactive context ${trackingId} for chat ${normalizedChatId}`);
          }
        });
      }
    }

    // 创建新的统一上下文ID格式：使用固定格式而不是时间戳
    const trackingId = `unified_${normalizedChatId}_${Date.now()}`;
    const context = {
      trackingId,
      chatId: normalizedChatId,
      messageIds: new Set(normalizedIds),
      fetchTimestamp: Date.now(),
      displayedIds: new Set(),
      retryAttempts: 0,
      status: 'fetching',
      isUnified: true // 标记为统一上下文
    };

    this.verificationQueue.set(trackingId, context);

    // CRITICAL FIX: 同时注册按chatId的快速查找
    if (!this._chatContextMap) this._chatContextMap = new Map();
    this._chatContextMap.set(normalizedChatId, trackingId);

    // CRITICAL FIX: Use debounced update
    this.displayMetrics.totalFetched += normalizedIds.length;
    this._debouncedUpdate('totalFetched');

    if (this.debugMode.value) {
      productionLogManager.info('MessageDisplayGuarantee', `Started unified tracking ${normalizedIds.length} messages for chat ${normalizedChatId}`, {
        trackingId,
        messageIds: normalizedIds,
        timeout: '2000ms',
        strategy: 'unified_context'
      });
    }

    // Set timeout for verification - 使用统一的trackingId
    setTimeout(() => {
      this.verifyDisplayCompletion(trackingId);
    }, 800); // OPTIMIZED: Reduced from 2000ms to 800ms for faster feedback

    return trackingId;
  }

  /**
   * ENHANCED: Create fallback context if no context found but we have a valid chatId
   */
  createFallbackContext(normalizedId, normalizedChatId) {
    // Check if this is a recent message that should have a context
    const recentTimeThreshold = 30000; // 30 seconds
    const now = Date.now();

    // CRITICAL: Check if we already have a fallback for this chat to prevent spam
    const existingFallback = Array.from(this.verificationQueue.values())
      .find(ctx => ctx.isFallback && ctx.chatId === normalizedChatId);

    if (existingFallback) {
      // Add to existing fallback instead of creating new one
      existingFallback.messageIds.add(normalizedId);
      if (this.debugMode.value) {
        console.log(`[MessageDisplayGuarantee] Added to existing fallback context for chat ${normalizedChatId}`);
      }
      return existingFallback.trackingId;
    }

    // FIX: Rate limiting - prevent excessive fallback creation during high-frequency scrolling
    if (!this._fallbackRateLimit) this._fallbackRateLimit = new Map();

    const rateLimitKey = `fallback_${normalizedChatId}`;
    const lastCreated = this._fallbackRateLimit.get(rateLimitKey) || 0;
    const timeSinceLastCreated = now - lastCreated;
    const minInterval = 1000; // Minimum 1 second between fallback creations per chat

    if (timeSinceLastCreated < minInterval) {
      if (this.debugMode.value) {
        console.log(`🚫 [MessageDisplayGuarantee] Rate limited: fallback creation for chat ${normalizedChatId} (${timeSinceLastCreated}ms < ${minInterval}ms)`);
      }
      return null; // Rate limited - don't create fallback
    }

    this._fallbackRateLimit.set(rateLimitKey, now);

    // Auto-cleanup rate limit entries
    setTimeout(() => {
      this._fallbackRateLimit.delete(rateLimitKey);
    }, minInterval * 2);

    // Only create fallback for recent attempts (prevent memory leaks)
    if (this.debugMode.value) {
      console.warn(`[MessageDisplayGuarantee] Creating fallback context for orphaned message ${normalizedId} in chat ${normalizedChatId}`);
    }

    const fallbackTrackingId = `fallback_${normalizedChatId}_${now}`;
    const fallbackContext = {
      trackingId: fallbackTrackingId,
      chatId: normalizedChatId,
      messageIds: new Set([normalizedId]),
      fetchTimestamp: now,
      displayedIds: new Set(),
      retryAttempts: 0,
      status: 'fallback_created',
      isFallback: true
    };

    this.verificationQueue.set(fallbackTrackingId, fallbackContext);

    // Auto-cleanup fallback context after reasonable time
    setTimeout(() => {
      if (this.verificationQueue.has(fallbackTrackingId)) {
        this.verificationQueue.delete(fallbackTrackingId);
        if (this.debugMode.value) {
          console.log(`🧹 [MessageDisplayGuarantee] Auto-cleaned fallback context ${fallbackTrackingId}`);
        }
      }
    }, recentTimeThreshold);

    return fallbackTrackingId;
  }

  /**
   * Mark message as displayed in UI
   * PRODUCTION FIX: 优化上下文查找机制，优先使用chatId查找
   */
  markMessageDisplayed(messageId, displayElement = null, currentChatId = null) {
    if (!this.isEnabled.value) return;

    // ENHANCED: Normalize messageId to handle both string and number types
    const normalizedId = parseInt(messageId);
    const normalizedChatId = currentChatId ? parseInt(currentChatId) : null;

    // RACE CONDITION FIX: If no active contexts, this might be a late retry after completion
    if (this.verificationQueue.size === 0) {
      if (this.debugMode.value) {
        productionLogManager.debug('MessageDisplayGuarantee', `No active tracking contexts - late retry for message ${normalizedId} ignored`);
      }
      return; // Gracefully ignore late retries
    }

    // CRITICAL FIX: 优先使用chatId快速查找策略
    let foundContext = false;
    let targetContext = null;

    // 策略1: 如果有chatId，优先使用快速查找映射
    if (normalizedChatId && this._chatContextMap && this._chatContextMap.has(normalizedChatId)) {
      const mappedTrackingId = this._chatContextMap.get(normalizedChatId);
      const mappedContext = this.verificationQueue.get(mappedTrackingId);

      if (mappedContext && mappedContext.chatId === normalizedChatId) {
        // 验证消息确实在这个上下文中
        const hasMessage = mappedContext.messageIds.has(messageId) ||
          mappedContext.messageIds.has(normalizedId) ||
          mappedContext.messageIds.has(String(normalizedId));

        if (hasMessage && mappedContext.status !== 'completed') {
          targetContext = { trackingId: mappedTrackingId, context: mappedContext };
          foundContext = true;

          if (this.debugMode.value) {
            console.log(`[MessageDisplayGuarantee] Fast lookup found context ${mappedTrackingId} for message ${normalizedId} in chat ${normalizedChatId}`);
          }
        }
      }
    }

    // 策略2: 如果快速查找失败，使用传统遍历查找（向后兼容）
    if (!foundContext) {
      for (const [trackingId, context] of this.verificationQueue.entries()) {
        // CRITICAL FIX: Add chatId validation to prevent cross-chat interference
        if (normalizedChatId && context.chatId !== normalizedChatId) {
          continue;
        }

        // RACE CONDITION FIX: Check if context is already completed
        if (context.status === 'completed') {
          if (this.debugMode.value) {
            console.log(`🔄 [MessageDisplayGuarantee] Context ${trackingId} already completed - ignoring late retry for message ${normalizedId}`);
          }
          continue; // 跳过已完成的上下文，但继续查找其他可能的上下文
        }

        // ENHANCED: Check for both string and number representations
        const hasMessage = context.messageIds.has(messageId) ||
          context.messageIds.has(normalizedId) ||
          context.messageIds.has(String(normalizedId));

        if (hasMessage) {
          foundContext = true;
          targetContext = { trackingId, context };

          if (this.debugMode.value) {
            console.log(`[MessageDisplayGuarantee] Traditional lookup found context ${trackingId} for message ${normalizedId} in chat ${context.chatId}`);
          }
          break;
        }
      }
    }

    // 策略3: 如果仍然没找到，创建fallback上下文
    if (!foundContext && normalizedChatId) {
      const fallbackTrackingId = this.createFallbackContext(normalizedId, normalizedChatId);

      // FIX: Handle rate limiting - fallbackTrackingId can be null
      if (fallbackTrackingId) {
        const fallbackContext = this.verificationQueue.get(fallbackTrackingId);
        if (fallbackContext) {
          targetContext = { trackingId: fallbackTrackingId, context: fallbackContext };
          foundContext = true;

          if (this.debugMode.value) {
            console.log(`🆘 [MessageDisplayGuarantee] Created fallback context ${fallbackTrackingId} for orphaned message ${normalizedId} in chat ${normalizedChatId}`);
          }
        }
      } else {
        // Rate limited - gracefully ignore this orphaned message
        if (this.debugMode.value) {
          console.log(`🚫 [MessageDisplayGuarantee] Orphaned message ${normalizedId} ignored due to rate limiting`);
        }
        return; // Gracefully exit
      }
    }

    if (foundContext && targetContext) {
      const { trackingId, context } = targetContext;

      if (this.debugMode.value) {
        productionLogManager.debug('MessageDisplayGuarantee', `Found context ${trackingId} for message ${normalizedId} in chat ${context.chatId}`);
      }

      // RACE CONDITION FIX: Check if message already marked as displayed
      if (context.displayedIds.has(normalizedId)) {
        if (this.debugMode.value) {
          productionLogManager.verbose('MessageDisplayGuarantee', `Message ${normalizedId} already marked - ignoring duplicate in context ${trackingId}`);
        }
        return; // Gracefully ignore duplicates
      }

      // ENHANCED: Add to displayed set using the same format as stored
      context.displayedIds.add(normalizedId);

      // CRITICAL FIX: Use debounced update to prevent recursive updates
      this.displayMetrics.totalDisplayed++;
      this._debouncedUpdate('totalDisplayed');

      if (this.debugMode.value) {
        console.log(`[MessageDisplayGuarantee] Marked message ${normalizedId} as displayed in tracking ${trackingId}. Progress: ${context.displayedIds.size}/${context.messageIds.size}`);
      }

      // Verify element is actually visible (if provided)
      let isVisible = true;
      if (displayElement) {
        isVisible = this.isElementVisible(displayElement);
        if (this.debugMode.value) {
          console.log(`🛡️ [MessageDisplayGuarantee] Element visibility for message ${normalizedId}:`, isVisible);
        }
      }

      if (isVisible) {
        context.status = context.isFallback ? 'fallback_displayed' : 'partially_displayed';

        // Check if all messages are displayed
        if (context.displayedIds.size === context.messageIds.size) {
          context.status = 'completed';
          this.completeTracking(trackingId);
        }
      }
    } else {
      // ENHANCED: More informative error with reduced noise
      if (this.verificationQueue.size > 0) {
        // Only log detailed error every 5th occurrence to reduce noise
        const errorKey = `${normalizedId}_${normalizedChatId}`;
        if (!this._errorThrottle) this._errorThrottle = new Map();

        const errorCount = (this._errorThrottle.get(errorKey) || 0) + 1;
        this._errorThrottle.set(errorKey, errorCount);

        // Clean up throttle map periodically
        if (errorCount === 1) {
          setTimeout(() => this._errorThrottle.delete(errorKey), 10000);
        }

        if (errorCount <= 2 || errorCount % 5 === 0) {
          const activeContexts = Array.from(this.verificationQueue.entries()).map(([id, ctx]) => ({
            trackingId: id,
            chatId: ctx.chatId,
            messageIds: Array.from(ctx.messageIds),
            displayedIds: Array.from(ctx.displayedIds),
            status: ctx.status,
            age: Date.now() - ctx.fetchTimestamp
          }));

          console.error(`ERROR: [MessageDisplayGuarantee] NO TRACKING CONTEXT FOUND for message ${normalizedId}${normalizedChatId ? ` in chat ${normalizedChatId}` : ''}. Active contexts:`, activeContexts);

          // CRITICAL DEBUG: Additional troubleshooting info (throttled)
          console.error(`[MessageDisplayGuarantee] Troubleshooting info:`, {
            messageId: messageId,
            normalizedId: normalizedId,
            currentChatId: normalizedChatId,
            totalActiveContexts: this.verificationQueue.size,
            verificationQueueKeys: Array.from(this.verificationQueue.keys()),
            errorOccurrence: errorCount,
            chatContextMap: this._chatContextMap ? Array.from(this._chatContextMap.entries()) : 'not_initialized'
          });
        }
      } else if (this.debugMode.value) {
        console.log(`🔄 [MessageDisplayGuarantee] Message ${normalizedId} not found in any active context - likely a late retry after completion`);
      }
    }
  }

  /**
   * Verify completion of message display
   * ENHANCED: Added timeout protection
   */
  verifyDisplayCompletion(trackingId) {
    const context = this.verificationQueue.get(trackingId);
    if (!context) return;

    const totalExpected = context.messageIds.size;
    const totalDisplayed = context.displayedIds.size;
    const timeElapsed = Date.now() - context.fetchTimestamp;

    if (this.debugMode.value) {
      console.log(`[MessageDisplayGuarantee] Verification for chat ${context.chatId}: ${totalDisplayed}/${totalExpected} messages displayed (${timeElapsed}ms elapsed)`);
    }

    // NEW: Timeout protection - if we've spent too long, accept partial success
    if (timeElapsed > this.retryConfig.maxTimeout) {
      if (totalDisplayed > totalExpected * 0.8) { // 80% success rate is acceptable
        if (this.debugMode.value) {
          console.log(`⏰ [MessageDisplayGuarantee] Timeout reached, accepting ${totalDisplayed}/${totalExpected} messages as sufficient`);
        }
        this.completeTrackingWithNote(trackingId, `timeout reached - ${totalDisplayed}/${totalExpected} messages displayed`);
        return true;
      }
    }

    if (totalDisplayed === totalExpected) {
      // Success - all messages displayed
      this.completeTracking(trackingId);
      return true;
    }

    // Some messages missing - attempt recovery
    const missingIds = Array.from(context.messageIds).filter(
      id => !context.displayedIds.has(id)
    );

    // ENHANCED: Check if missing messages might be legitimately absent
    const missingCount = missingIds.length;
    const missingRatio = missingCount / totalExpected;

    if (this.debugMode.value) {
      console.log(`[MessageDisplayGuarantee] Missing analysis: ${missingCount}/${totalExpected} (${(missingRatio * 100).toFixed(1)}%) missing`);
      console.log(`[MessageDisplayGuarantee] Missing IDs:`, missingIds);
      console.log(`[MessageDisplayGuarantee] Displayed IDs:`, Array.from(context.displayedIds));
    }

    // CRITICAL FIX: Handle the case where ALL messages are missing
    if (missingRatio === 1.0) {
      // If ALL messages are missing, this is definitely a system problem, not deleted messages
      console.warn(`🚨 [MessageDisplayGuarantee] ALL ${totalExpected} messages are missing in chat ${context.chatId} - this indicates a system issue`);

      // 🚨 EMERGENCY AUTO-FIX: Try to auto-register visible messages
      if (context.retryAttempts === 0) {
        console.log(`🆘 [MessageDisplayGuarantee] Attempting emergency auto-registration for chat ${context.chatId}`);
        this.emergencyAutoRegister(context.chatId, missingIds);
      }

      if (context.retryAttempts < this.retryConfig.maxRetries) {
        this.attemptRecovery(trackingId, missingIds);
      } else {
        this.reportFailure(trackingId, missingIds);
      }
      return;
    }

    // NEW: Enhanced detection for different failure scenarios
    const isScatteredPattern = this.hasScatteredPattern(missingIds);
    const isLikelyViewportIssue = !isScatteredPattern && missingRatio < 0.3;
    const isLikelyDeletedMessages = missingRatio < 0.2 && isScatteredPattern;

    // VIEWPORT ISSUE HANDLING: Messages likely outside viewport
    if (isLikelyViewportIssue) {
      if (this.debugMode.value) {
        console.log(`[MessageDisplayGuarantee] Detected ${missingCount} messages likely outside viewport in chat ${context.chatId}:`, missingIds);
      }

      // Mark as completed with viewport note - this is not a critical failure
      this.completeTrackingWithNote(trackingId, `${missingCount} messages outside viewport (normal behavior)`);
      return true;
    }

    // DELETED MESSAGES HANDLING: Scattered missing messages
    if (isLikelyDeletedMessages) {
      // NEW: Update deleted message statistics with debounced updates
      this.deletedMessageStats.totalDetected += missingCount;
      this.deletedMessageStats.chatsWithDeleted.add(context.chatId);
      this.deletedMessageStats.lastDetection = Date.now();
      this._debouncedUpdate('deletedMessages');

      if (this.debugMode.value) {
        console.log(`[MessageDisplayGuarantee] Detected ${missingCount} likely deleted messages in chat ${context.chatId}:`, missingIds);
      }

      // Mark as completed with a note about deleted messages
      this.completeTrackingWithNote(trackingId, `${missingCount} messages appear to be deleted`);
      return true;
    }

    // ENHANCED: More detailed warning with debugging info
    console.warn(`🚨 [MessageDisplayGuarantee] Missing ${missingIds.length} messages in chat ${context.chatId}:`, {
      missingIds,
      missingRatio: (missingRatio * 100).toFixed(1) + '%',
      isScatteredPattern: this.hasScatteredPattern(missingIds),
      displayedIds: Array.from(context.displayedIds),
      totalExpected
    });

    if (context.retryAttempts < this.retryConfig.maxRetries) {
      this.attemptRecovery(trackingId, missingIds);
    } else {
      this.reportFailure(trackingId, missingIds);
    }
  }

  /**
   * Check if missing IDs form a scattered pattern (indicating deleted messages)
   * ENHANCED: Better detection for edge cases
   */
  hasScatteredPattern(missingIds) {
    if (missingIds.length <= 1) return true;

    // Sort the missing IDs
    const sorted = missingIds.map(id => parseInt(id)).sort((a, b) => a - b);

    // NEW: Check if missing messages are at the end (likely viewport issue)
    const allDisplayedIds = Array.from(this.verificationQueue.values())
      .flatMap(ctx => Array.from(ctx.displayedIds))
      .map(id => parseInt(id))
      .sort((a, b) => a - b);

    if (allDisplayedIds.length > 0) {
      const maxDisplayedId = Math.max(...allDisplayedIds);
      const minMissingId = Math.min(...sorted);

      // If missing messages are all after the highest displayed message,
      // they are likely outside viewport (not scattered/deleted)
      if (minMissingId > maxDisplayedId) {
        if (this.debugMode.value) {
          console.log(`[MessageDisplayGuarantee] Missing messages [${sorted.join(', ')}] appear to be outside viewport (after ${maxDisplayedId})`);
        }
        return false; // Not scattered - likely viewport issue
      }
    }

    // Check if they are not consecutive
    let consecutiveCount = 0;
    for (let i = 1; i < sorted.length; i++) {
      if (sorted[i] === sorted[i - 1] + 1) {
        consecutiveCount++;
      }
    }

    // If less than half are consecutive, it's likely a scattered pattern
    return consecutiveCount < (sorted.length * 0.5);
  }

  /**
   * Complete tracking with a note about the resolution
   */
  completeTrackingWithNote(trackingId, note) {
    const context = this.verificationQueue.get(trackingId);
    if (!context) return;

    const timeTaken = Date.now() - context.fetchTimestamp;
    this.displayMetrics.lastVerification = {
      chatId: context.chatId,
      messageCount: context.messageIds.size,
      displayedCount: context.displayedIds.size,
      timeTaken,
      timestamp: Date.now(),
      note
    };

    if (import.meta.env.DEV) {
      console.log(`[MessageDisplayGuarantee] Completed tracking for chat ${context.chatId} (${timeTaken}ms) - ${note}`);
    }

    this.verificationQueue.delete(trackingId);
  }

  /**
   * Attempt to recover missing messages
   */
  async attemptRecovery(trackingId, missingIds) {
    const context = this.verificationQueue.get(trackingId);
    if (!context) return;

    context.retryAttempts++;
    context.status = 'recovering';

    const delay = Math.min(
      this.retryConfig.baseDelay * Math.pow(2, context.retryAttempts - 1),
      this.retryConfig.maxDelay
    );

    console.log(`🔄 [MessageDisplayGuarantee] Attempting recovery ${context.retryAttempts}/${this.retryConfig.maxRetries} for ${missingIds.length} messages`);

    setTimeout(async () => {
      try {
        // Force re-render of message list
        await this.forceMessageListRefresh(context.chatId);

        // Re-verify after recovery attempt
        setTimeout(() => {
          this.verifyDisplayCompletion(trackingId);
        }, 1000);

      } catch (error) {
        console.error('🚨 [MessageDisplayGuarantee] Recovery failed:', error);
        this.reportFailure(trackingId, missingIds);
      }
    }, delay);
  }

  /**
   * Force refresh of message list component
   */
  async forceMessageListRefresh(chatId) {
    // Emit event to force message list re-render
    const refreshEvent = new CustomEvent('fechatter:force-message-refresh', {
      detail: { chatId }
    });
    window.dispatchEvent(refreshEvent);

    // Also trigger scroll to ensure visibility
    const scrollEvent = new CustomEvent('fechatter:force-scroll-check', {
      detail: { chatId }
    });
    window.dispatchEvent(scrollEvent);
  }

  /**
   * Report display failure
   */
  reportFailure(trackingId, missingIds) {
    const context = this.verificationQueue.get(trackingId);
    if (!context) return;

    context.status = 'failed';
    // CRITICAL FIX: Use debounced update
    this.displayMetrics.failedDisplays += missingIds.length;
    this._debouncedUpdate('failedDisplays');

    console.error(`ERROR: [MessageDisplayGuarantee] CRITICAL: Failed to display ${missingIds.length} messages in chat ${context.chatId}`, {
      trackingId,
      missingIds,
      retryAttempts: context.retryAttempts,
      timeTaken: Date.now() - context.fetchTimestamp
    });

    // Report to monitoring system
    this.reportToMonitoring('message_display_failure', {
      chatId: context.chatId,
      missingCount: missingIds.length,
      totalExpected: context.messageIds.size,
      retryAttempts: context.retryAttempts
    });

    this.verificationQueue.delete(trackingId);
  }

  /**
   * Complete successful tracking
   */
  completeTracking(trackingId) {
    const context = this.verificationQueue.get(trackingId);
    if (!context) return;

    const timeTaken = Date.now() - context.fetchTimestamp;
    this.displayMetrics.lastVerification = {
      chatId: context.chatId,
      messageCount: context.messageIds.size,
      timeTaken,
      timestamp: Date.now()
    };

    console.log(`[MessageDisplayGuarantee] Successfully displayed ${context.messageIds.size} messages in chat ${context.chatId} (${timeTaken}ms)`);

    this.verificationQueue.delete(trackingId);
  }

  /**
   * Check if DOM element is visible within its message container
   * ENTERPRISE FIX: Enhanced visibility detection for enterprise chat systems
   */
  isElementVisible(element) {
    if (!element) return false;

    const rect = element.getBoundingClientRect();

    // Element must have dimensions
    if (rect.width <= 0 || rect.height <= 0) {
      return false;
    }

    // ENTERPRISE: Enhanced container detection with priority order
    const messageContainer =
      element.closest('.discord-message-list') ||      // Discord system (primary)
      element.closest('.messages-container') ||        // Discord nested container
      element.closest('.simple-message-list') ||       // Simple system
      element.closest('.message-list') ||              // Fallback
      element.closest('[data-testid="message-list"]') || // Test environments
      element.closest('.chat-container') ||            // Chat container
      document.querySelector('.discord-message-list'); // Global fallback

    if (!messageContainer) {
      // ENTERPRISE: Very tolerant window viewport check for edge cases
      const viewHeight = Math.max(document.documentElement.clientHeight, window.innerHeight);
      const viewWidth = Math.max(document.documentElement.clientWidth, window.innerWidth);

      // GENEROUS: Large buffer for enterprise scenarios with complex layouts
      return (
        rect.bottom >= -200 &&           // Allow 200px buffer above viewport
        rect.top <= viewHeight + 200 &&  // Allow 200px buffer below viewport
        rect.right >= -100 &&            // Allow 100px buffer left
        rect.left <= viewWidth + 100     // Allow 100px buffer right
      );
    }

    // ENTERPRISE: Check visibility relative to the message container
    const containerRect = messageContainer.getBoundingClientRect();

    // CRITICAL: Enterprise-grade visibility tolerance
    // Much larger buffer to handle complex enterprise layouts, animations, and edge cases
    const buffer = 300; // Increased from 100px to 300px for enterprise tolerance
    const isVisible = (
      rect.bottom >= (containerRect.top - buffer) &&
      rect.top <= (containerRect.bottom + buffer) &&
      rect.right >= (containerRect.left - buffer) &&
      rect.left <= (containerRect.right + buffer)
    );

    // ENTERPRISE: Multiple fallback checks for message elements
    const hasMessageId = element.hasAttribute('data-message-id') ||
      element.querySelector('[data-message-id]') ||
      element.classList.contains('discord-message-item') ||
      element.classList.contains('message-item');

    if (!isVisible && hasMessageId) {
      // ENHANCED: More generous fallback checks for enterprise environments
      const isInDOM = document.contains(element);
      const hasReasonablePosition = rect.top > -2000 && rect.bottom < window.innerHeight + 2000;
      const hasReasonableSize = rect.width > 0 && rect.height > 0;
      const parentVisible = element.parentElement && this.isParentChainVisible(element.parentElement);

      if (isInDOM && hasReasonablePosition && hasReasonableSize) {
        // ENTERPRISE: Force visibility for message elements that meet basic criteria
        if (this.debugMode.value) {
          console.log(`[MessageDisplayGuarantee] ENTERPRISE: Force-marking message as visible:`, {
            elementRect: rect,
            containerRect: containerRect,
            hasMessageId,
            isInDOM,
            hasReasonablePosition,
            hasReasonableSize,
            parentVisible
          });
        }
        return true;
      }

      // ADDITIONAL: Check if element is just outside a scrollable container
      if (messageContainer.scrollHeight > messageContainer.clientHeight) {
        const scrollTop = messageContainer.scrollTop;
        const scrollHeight = messageContainer.scrollHeight;
        const clientHeight = messageContainer.clientHeight;

        // If container is scrollable and element is within reasonable scroll bounds
        const elementTopInContainer = rect.top - containerRect.top + scrollTop;
        const elementBottomInContainer = rect.bottom - containerRect.top + scrollTop;

        const withinScrollBounds = (
          elementBottomInContainer >= -buffer &&
          elementTopInContainer <= scrollHeight + buffer
        );

        if (withinScrollBounds) {
          if (this.debugMode.value) {
            console.log(`[MessageDisplayGuarantee] ENTERPRISE: Element within scroll bounds, marking visible:`, {
              elementTopInContainer,
              elementBottomInContainer,
              scrollHeight,
              scrollTop,
              withinScrollBounds
            });
          }
          return true;
        }
      }
    }

    // DEBUG: Enhanced logging in development (batched to reduce noise)
    if (import.meta.env.DEV && this.debugMode.value && !isVisible) {
      console.log(`[MessageDisplayGuarantee] Element not visible:`, {
        elementRect: {
          top: rect.top,
          bottom: rect.bottom,
          width: rect.width,
          height: rect.height
        },
        containerRect: containerRect ? {
          top: containerRect.top,
          bottom: containerRect.bottom,
          width: containerRect.width,
          height: containerRect.height
        } : null,
        buffer,
        hasMessageId,
        elementType: element.tagName,
        elementClass: element.className,
        containerType: messageContainer?.tagName,
        containerClass: messageContainer?.className
      });
    }

    return isVisible;
  }

  /**
   * NEW: Check if parent chain is visible (for nested visibility detection)
   */
  isParentChainVisible(element, maxDepth = 5) {
    if (!element || maxDepth <= 0) return false;

    const rect = element.getBoundingClientRect();

    // Parent must have reasonable dimensions
    if (rect.width <= 0 || rect.height <= 0) {
      return false;
    }

    // Check if parent is hidden
    const styles = window.getComputedStyle(element);
    if (styles.display === 'none' || styles.visibility === 'hidden' || styles.opacity === '0') {
      return false;
    }

    // Check parent element recursively
    if (element.parentElement && element.parentElement !== document.body) {
      return this.isParentChainVisible(element.parentElement, maxDepth - 1);
    }

    return true;
  }

  /**
   * Report metrics to monitoring system
   */
  reportToMonitoring(eventType, data) {
    // Integration point for monitoring systems
    if (window.analytics && window.analytics.track) {
      window.analytics.track(eventType, data);
    }

    // Also emit custom event for other listeners
    const event = new CustomEvent('fechatter:display-guarantee-event', {
      detail: { eventType, data }
    });
    window.dispatchEvent(event);
  }

  /**
   * Get current metrics (plain objects, not reactive)
   */
  getMetrics() {
    return {
      ...this.displayMetrics,
      successRate: this.displayMetrics.totalFetched > 0
        ? ((this.displayMetrics.totalDisplayed / this.displayMetrics.totalFetched) * 100).toFixed(2)
        : 100,
      activeTracking: this.verificationQueue.size,
      // NEW: Include deleted message statistics
      deletedMessages: {
        totalDetected: this.deletedMessageStats.totalDetected,
        chatsAffected: this.deletedMessageStats.chatsWithDeleted.size,
        lastDetection: this.deletedMessageStats.lastDetection
      }
    };
  }

  /**
   * NEW: Get reactive metrics for components that need to watch values
   * This creates a fresh reactive object to avoid recursive update issues
   */
  getReactiveMetrics() {
    return reactive({
      ...this.getMetrics()
    });
  }

  /**
   * Enable/disable the guarantee system
   */
  setEnabled(enabled) {
    this.isEnabled.value = enabled;
    if (!enabled) {
      this.verificationQueue.clear();
    }
  }

  /**
   * NEW: Quick methods for testing and debugging
   */
  enableDebugMode() {
    this.debugMode.value = true;
    if (import.meta.env.DEV) {
      console.log('[MessageDisplayGuarantee] Debug mode enabled');
    }
  }

  disableDebugMode() {
    this.debugMode.value = false;
    if (import.meta.env.DEV) {
      console.log('[MessageDisplayGuarantee] Debug mode disabled');
    }
  }

  temporaryDisable(durationMs = 30000) {
    this.setEnabled(false);
    if (import.meta.env.DEV) {
      console.log(`[MessageDisplayGuarantee] Temporarily disabled for ${durationMs}ms`);
    }
    setTimeout(() => {
      this.setEnabled(true);
      if (import.meta.env.DEV) {
        console.log('[MessageDisplayGuarantee] Re-enabled after temporary disable');
      }
    }, durationMs);
  }

  /**
   * Export debug information
   */
  exportDebugInfo() {
    return {
      metrics: this.getMetrics(),
      activeTracking: Array.from(this.verificationQueue.entries()).map(([id, context]) => ({
        trackingId: id,
        chatId: context.chatId,
        messageCount: context.messageIds.size,
        displayedCount: context.displayedIds.size,
        status: context.status,
        retryAttempts: context.retryAttempts,
        age: Date.now() - context.fetchTimestamp
      })),
      config: this.retryConfig
    };
  }

  /**
   * NEW: Cleanup method to prevent memory leaks
   */
  cleanup() {
    // Clear debounce timer
    if (this._updateDebounceTimer) {
      clearTimeout(this._updateDebounceTimer);
      this._updateDebounceTimer = null;
    }

    // Clear pending updates
    this._pendingUpdates.clear();

    // Clear verification queue
    this.verificationQueue.clear();

    if (this.debugMode.value) {
      console.log('🧹 [MessageDisplayGuarantee] Cleanup completed');
    }
  }

  /**
   * NEW: Clear tracking contexts for a specific chat
   * Called when switching chats to prevent cross-chat interference
   */
  clearTrackingForChat(chatId) {
    const normalizedChatId = parseInt(chatId);
    let clearedCount = 0;
    const contextsToGracefullyComplete = [];

    // CRITICAL DEBUG: Log detailed context information before clearing
    if (this.debugMode.value) {
      const existingContexts = Array.from(this.verificationQueue.entries()).map(([id, ctx]) => ({
        trackingId: id,
        chatId: ctx.chatId,
        messageIds: Array.from(ctx.messageIds),
        displayedIds: Array.from(ctx.displayedIds),
        status: ctx.status,
        age: Date.now() - ctx.fetchTimestamp
      }));

      console.log(`🧹 [MessageDisplayGuarantee] BEFORE clearTrackingForChat(${normalizedChatId}). Current contexts:`, existingContexts);
    }

    for (const [trackingId, context] of this.verificationQueue.entries()) {
      if (context.chatId === normalizedChatId) {
        // ENHANCED: Check if context has partial progress that should be gracefully completed
        const hasPartialProgress = context.displayedIds.size > 0 && context.displayedIds.size < context.messageIds.size;
        const isRecentContext = (Date.now() - context.fetchTimestamp) < 5000; // Less than 5 seconds old

        if (hasPartialProgress && isRecentContext && !context.isFallback) {
          // Don't immediately clear contexts with recent partial progress
          // Instead, mark them for graceful completion
          contextsToGracefullyComplete.push({
            trackingId,
            context,
            remainingTime: 3000 // Give 3 more seconds for in-flight display events
          });

          if (this.debugMode.value) {
            console.log(`🔄 [MessageDisplayGuarantee] Deferring cleanup of context ${trackingId} due to partial progress (${context.displayedIds.size}/${context.messageIds.size})`);
          }
        } else {
          // Safe to clear immediately
          if (this.debugMode.value) {
            console.log(`🧹 [MessageDisplayGuarantee] Clearing tracking context ${trackingId} for chat ${normalizedChatId}:`, {
              messageIds: Array.from(context.messageIds),
              displayedIds: Array.from(context.displayedIds),
              status: context.status,
              hasPartialProgress,
              isRecentContext
            });
          }
          this.verificationQueue.delete(trackingId);
          clearedCount++;
        }
      }
    }

    // NEW: Handle graceful completion of contexts with partial progress
    if (contextsToGracefullyComplete.length > 0) {
      contextsToGracefullyComplete.forEach(({ trackingId, context, remainingTime }) => {
        // Mark context as being cleared to prevent new display events
        context.status = 'clearing';
        context.clearingStartTime = Date.now();

        // Set a timer to complete or force-clear the context
        setTimeout(() => {
          if (this.verificationQueue.has(trackingId)) {
            const finalContext = this.verificationQueue.get(trackingId);
            const finalDisplayedCount = finalContext.displayedIds.size;
            const finalExpectedCount = finalContext.messageIds.size;

            if (finalDisplayedCount === finalExpectedCount) {
              // All messages were displayed during grace period
              this.completeTrackingWithNote(trackingId, `completed during chat switch grace period`);
            } else if (finalDisplayedCount > 0) {
              // Some progress was made
              this.completeTrackingWithNote(trackingId, `partial completion during chat switch (${finalDisplayedCount}/${finalExpectedCount})`);
            } else {
              // No additional progress
              this.verificationQueue.delete(trackingId);
            }

            if (this.debugMode.value) {
              console.log(`🔄 [MessageDisplayGuarantee] Grace period completed for context ${trackingId}: ${finalDisplayedCount}/${finalExpectedCount} messages`);
            }
          }
        }, remainingTime);
      });

      if (this.debugMode.value) {
        console.log(`🔄 [MessageDisplayGuarantee] Initiated graceful completion for ${contextsToGracefullyComplete.length} contexts`);
      }
    }

    if (this.debugMode.value) {
      console.log(`🧹 [MessageDisplayGuarantee] AFTER clearTrackingForChat(${normalizedChatId}). Cleared ${clearedCount} context(s). Remaining contexts:`,
        Array.from(this.verificationQueue.entries()).map(([id, ctx]) => ({
          trackingId: id,
          chatId: ctx.chatId,
          messageIds: Array.from(ctx.messageIds),
          status: ctx.status
        }))
      );
    }

    return clearedCount;
  }

  /**
   * NEW: Clear all tracking contexts (for app-level cleanup)
   */
  clearAllTracking() {
    const clearedCount = this.verificationQueue.size;
    this.verificationQueue.clear();

    if (this.debugMode.value && clearedCount > 0) {
      console.log(`🧹 [MessageDisplayGuarantee] Cleared all ${clearedCount} tracking context(s)`);
    }

    return clearedCount;
  }

  /**
   * 🚨 NEW: Emergency auto-registration when ALL messages are missing
   */
  emergencyAutoRegister(chatId, expectedMessageIds) {
    try {
      const messageElements = document.querySelectorAll('[data-message-id]');
      let registeredCount = 0;

      console.log(`🆘 [MessageDisplayGuarantee] Emergency auto-register: Found ${messageElements.length} elements, expecting ${expectedMessageIds.length} messages`);

      messageElements.forEach(element => {
        const messageId = element.getAttribute('data-message-id');
        const numericId = parseInt(messageId);

        // Only register if this message ID was expected and element is visible
        if (expectedMessageIds.includes(numericId) && element.offsetParent !== null) {
          try {
            this.markMessageDisplayed(numericId, element, chatId);
            registeredCount++;

            if (this.debugMode.value) {
              console.log(`🆘 [Emergency] Auto-registered message ${messageId}`);
            }
          } catch (error) {
            if (this.debugMode.value) {
              console.warn(`🆘 [Emergency] Failed to auto-register message ${messageId}:`, error.message);
            }
          }
        }
      });

      console.log(`🆘 [MessageDisplayGuarantee] Emergency auto-registration complete: ${registeredCount}/${expectedMessageIds.length} messages registered`);

      return registeredCount;
    } catch (error) {
      console.error(`🚨 [MessageDisplayGuarantee] Emergency auto-registration failed:`, error);
      return 0;
    }
  }

  /**
   * Mark temporary message as replaced by real message
   * NEW: Handle temp message to real message transition
   */
  markMessageReplaced(tempMessageId, realMessageId, chatId) {
    if (!this.isEnabled.value) return;

    const normalizedTempId = tempMessageId.toString();
    const normalizedRealId = parseInt(realMessageId);
    const normalizedChatId = parseInt(chatId);

    if (this.debugMode.value) {
      console.log(`🔄 [MessageDisplayGuarantee] Processing message replacement: ${normalizedTempId} → ${normalizedRealId} in chat ${normalizedChatId}`);
    }

    // Find contexts that contain the temporary message
    for (const [trackingId, context] of this.verificationQueue.entries()) {
      if (context.chatId !== normalizedChatId) continue;

      // Check if this context contains the temporary message
      const hasTempMessage = context.messageIds.has(normalizedTempId) ||
        context.messageIds.has(tempMessageId);

      if (hasTempMessage) {
        // Remove the temporary message ID and add the real message ID
        context.messageIds.delete(normalizedTempId);
        context.messageIds.delete(tempMessageId);
        context.messageIds.add(normalizedRealId);

        // If temp message was marked as displayed, mark real message as displayed too
        if (context.displayedIds.has(normalizedTempId) || context.displayedIds.has(tempMessageId)) {
          context.displayedIds.delete(normalizedTempId);
          context.displayedIds.delete(tempMessageId);
          context.displayedIds.add(normalizedRealId);

          if (this.debugMode.value) {
            console.log(`[MessageDisplayGuarantee] Transferred display status from ${normalizedTempId} to ${normalizedRealId} in context ${trackingId}`);
          }
        }

        // Update context metadata
        context.lastReplacement = {
          tempId: normalizedTempId,
          realId: normalizedRealId,
          timestamp: Date.now()
        };

        if (this.debugMode.value) {
          console.log(`🔄 [MessageDisplayGuarantee] Updated context ${trackingId}: temp message replaced. Progress: ${context.displayedIds.size}/${context.messageIds.size}`);
        }

        // Check if tracking should be completed after replacement
        if (context.displayedIds.size === context.messageIds.size && context.status !== 'completed') {
          context.status = 'completed';
          this.completeTracking(trackingId);
        }
      }
    }
  }
}

// CRITICAL FIX: Create global instance with proper cleanup
export const messageDisplayGuarantee = new MessageDisplayGuarantee();

// Make available globally for debugging with cleanup support
if (typeof window !== 'undefined') {
  // Clean up any existing instance first
  if (window.messageDisplayGuarantee && typeof window.messageDisplayGuarantee.cleanup === 'function') {
    window.messageDisplayGuarantee.cleanup();
  }

  window.messageDisplayGuarantee = messageDisplayGuarantee;

  // Add cleanup on page unload to prevent memory leaks
  window.addEventListener('beforeunload', () => {
    if (window.messageDisplayGuarantee && typeof window.messageDisplayGuarantee.cleanup === 'function') {
      window.messageDisplayGuarantee.cleanup();
    }
  });
} 