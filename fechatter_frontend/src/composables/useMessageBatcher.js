import { ref, onUnmounted, getCurrentInstance } from 'vue';

/**
 * M1: 消息管道去抖（Debounce/Batch）
 * 多源事件 (push + 下拉 + 重连补包) → 统一进 RxJS Subject / tokio::mpsc
 * 以 16 ms 批量 flush；保证 < 1 frame
 */
export function useMessageBatcher() {
  const batchedMessages = ref([]);
  const messageQueue = [];
  let flushTimer = null;
  let lastFlushTime = 0;

  // 16ms = 1 frame at 60fps
  const BATCH_INTERVAL = 16;

  // Performance monitoring
  const performanceMetrics = ref({
    totalBatches: 0,
    totalMessages: 0,
    avgBatchSize: 0,
    maxBatchSize: 0,
    lastBatchTime: 0
  });

  /**
   * Add message to batch queue
   */
  const addMessage = (message) => {
    messageQueue.push({
      ...message,
      queuedAt: performance.now()
    });

    scheduleFlush();
  };

  /**
   * Add multiple messages at once
   */
  const addMessages = (messages) => {
    const now = performance.now();
    messages.forEach(msg => {
      messageQueue.push({
        ...msg,
        queuedAt: now
      });
    });

    scheduleFlush();
  };

  // 防止并发刷新的标志
  let flushInProgress = false;

  /**
   * Schedule batch flush
   */
  const scheduleFlush = () => {
    // 防止重复调度
    if (flushTimer || flushInProgress) return;

    const now = performance.now();
    const timeSinceLastFlush = now - lastFlushTime;

    // If we just flushed, wait for next frame
    const delay = timeSinceLastFlush < BATCH_INTERVAL
      ? BATCH_INTERVAL - timeSinceLastFlush
      : 0;

    flushTimer = requestAnimationFrame(() => {
      flushTimer = null;
      if (!flushInProgress) {
        flushBatch();
      }
    });
  };

  /**
   * Flush message batch
   */
  const flushBatch = () => {
    if (messageQueue.length === 0 || flushInProgress) return;

    flushInProgress = true;
    const now = performance.now();

    // 安全地复制和清空队列
    const batch = messageQueue.splice(0, messageQueue.length);

    // Update metrics
    performanceMetrics.value.totalBatches++;
    performanceMetrics.value.totalMessages += batch.length;
    performanceMetrics.value.avgBatchSize =
      performanceMetrics.value.totalMessages / performanceMetrics.value.totalBatches;
    performanceMetrics.value.maxBatchSize =
      Math.max(performanceMetrics.value.maxBatchSize, batch.length);
    performanceMetrics.value.lastBatchTime = now - lastFlushTime;

    lastFlushTime = now;

    // Sort messages by timestamp to maintain order
    batch.sort((a, b) => {
      // First by created_at
      const timeA = new Date(a.created_at).getTime();
      const timeB = new Date(b.created_at).getTime();
      if (timeA !== timeB) return timeA - timeB;

      // Then by queue time
      return a.queuedAt - b.queuedAt;
    });

    // Remove duplicate messages
    const uniqueMessages = deduplicateMessages(batch);

    // Emit batch
    batchedMessages.value = uniqueMessages;

    // Log performance
    if (import.meta.env.DEV) {
      // console.log(`[MessageBatcher] Flushed ${uniqueMessages.length} messages in ${(now - batch[0].queuedAt).toFixed(2)}ms`);
    }

    // 重置刷新标志
    flushInProgress = false;
  };

  /**
   * Deduplicate messages by ID
   */
  const deduplicateMessages = (messages) => {
    const seen = new Map();
    const result = [];

    for (const msg of messages) {
      // 确保消息有有效的标识符
      const key = msg.id || msg.temp_id || `temp_${Date.now()}_${Math.floor(Math.random() * 16777215).toString(16)}`;

      if (!key || key.toString().startsWith('temp_') && !msg.temp_id) {
        // 为没有ID的消息生成临时ID
        msg.temp_id = key;
      }

      if (!seen.has(key)) {
        seen.set(key, true);
        result.push(msg);
      } else {
        // 如果是重复消息，保留更新的版本
        const index = result.findIndex(m => (m.id || m.temp_id) === key);
        if (index !== -1) {
          const existingTime = new Date(result[index].created_at || 0).getTime();
          const newTime = new Date(msg.created_at || 0).getTime();

          // 只有当新消息确实更新时才替换
          if (newTime > existingTime || (!result[index].id && msg.id)) {
            result[index] = msg;
          }
        }
      }
    }

    return result;
  };

  /**
   * Force immediate flush
   */
  const forceFlush = () => {
    if (flushTimer) {
      cancelAnimationFrame(flushTimer);
      flushTimer = null;
    }
    flushBatch();
  };

  /**
   * Reset batcher
   */
  const reset = () => {
    if (flushTimer) {
      cancelAnimationFrame(flushTimer);
      flushTimer = null;
    }
    messageQueue.length = 0;
    batchedMessages.value = [];
    lastFlushTime = 0;
  };

  /**
   * Get current queue size
   */
  const getQueueSize = () => messageQueue.length;

  // Cleanup on unmount - only if called within a component
  const instance = getCurrentInstance();
  if (instance) {
    onUnmounted(() => {
      if (flushTimer) {
        cancelAnimationFrame(flushTimer);
      }
    });
  }

  // Manual cleanup method for non-component usage
  const cleanup = () => {
    if (flushTimer) {
      cancelAnimationFrame(flushTimer);
      flushTimer = null;
    }
  };

  return {
    batchedMessages,
    addMessage,
    addMessages,
    forceFlush,
    reset,
    getQueueSize,
    performanceMetrics,
    cleanup
  };
}