import { ref, nextTick } from 'vue';

/**
 * M4: "焦点锚定"滚动 + 偏移补偿
 * 点击 → 虚拟列表先保证元素渲染 → scrollIntoView({block:'center'}) → focus-ring
 */
export function useFocusAnchoredScroll() {
  // 当前焦点消息
  const focusedMessageId = ref(null);
  const scrollContainer = ref(null);
  const isScrolling = ref(false);
  
  // 滚动配置
  const SCROLL_DURATION = 300; // ms
  const HIGHLIGHT_DURATION = 2000; // ms
  const CENTER_OFFSET = 100; // 距离顶部的偏移量
  
  /**
   * 滚动到指定消息并高亮
   * @param {string|number} messageId - 消息ID
   * @param {Object} options - 滚动选项
   */
  const scrollToMessage = async (messageId, options = {}) => {
    const {
      highlight = true,
      smooth = true,
      block = 'center',
      ensureVisible = true,
      onComplete = null
    } = options;
    
    if (!messageId || !scrollContainer.value) {
      console.warn('[FocusAnchoredScroll] Missing messageId or container');
      return false;
    }
    
    isScrolling.value = true;
    focusedMessageId.value = messageId;
    
    try {
      // 1. 确保消息已渲染
      if (ensureVisible) {
        await ensureMessageVisible(messageId);
      }
      
      // 2. 等待DOM更新
      await nextTick();
      await waitForRender();
      
      // 3. 查找目标元素
      const targetElement = findMessageElement(messageId);
      if (!targetElement) {
        console.warn(`[FocusAnchoredScroll] Message element not found: ${messageId}`);
        return false;
      }
      
      // 4. 计算滚动位置
      const scrollPosition = calculateScrollPosition(targetElement, block);
      
      // 5. 执行滚动
      await performScroll(scrollPosition, smooth);
      
      // 6. 应用高亮效果
      if (highlight) {
        applyHighlight(targetElement);
      }
      
      // 7. 设置焦点（用于键盘导航）
      targetElement.setAttribute('tabindex', '-1');
      targetElement.focus({ preventScroll: true });
      
      // 8. 回调
      if (onComplete) {
        onComplete(targetElement);
      }
      
      return true;
      
    } catch (error) {
      console.error('[FocusAnchoredScroll] Error:', error);
      return false;
    } finally {
      isScrolling.value = false;
    }
  };
  
  /**
   * 确保消息可见（处理虚拟滚动）
   */
  const ensureMessageVisible = async (messageId) => {
    // 触发虚拟列表加载该消息
    const event = new CustomEvent('ensure-message-visible', {
      detail: { messageId }
    });
    scrollContainer.value.dispatchEvent(event);
    
    // 等待加载完成
    await new Promise(resolve => {
      const checkVisible = () => {
        if (findMessageElement(messageId)) {
          resolve();
        } else {
          requestAnimationFrame(checkVisible);
        }
      };
      checkVisible();
    });
  };
  
  /**
   * 查找消息元素
   */
  const findMessageElement = (messageId) => {
    if (!scrollContainer.value) return null;
    
    // 尝试多种选择器
    const selectors = [
      `[data-message-id="${messageId}"]`,
      `#message-${messageId}`,
      `.message-item[data-id="${messageId}"]`
    ];
    
    for (const selector of selectors) {
      const element = scrollContainer.value.querySelector(selector);
      if (element) return element;
    }
    
    return null;
  };
  
  /**
   * 计算滚动位置
   */
  const calculateScrollPosition = (element, block) => {
    const container = scrollContainer.value;
    const containerRect = container.getBoundingClientRect();
    const elementRect = element.getBoundingClientRect();
    
    // 当前滚动位置
    const currentScroll = container.scrollTop;
    
    // 元素相对于容器的位置
    const elementTop = elementRect.top - containerRect.top + currentScroll;
    const elementHeight = elementRect.height;
    const containerHeight = containerRect.height;
    
    let targetScroll;
    
    switch (block) {
      case 'start':
        targetScroll = elementTop - CENTER_OFFSET;
        break;
      case 'end':
        targetScroll = elementTop + elementHeight - containerHeight + CENTER_OFFSET;
        break;
      case 'center':
      default:
        targetScroll = elementTop - (containerHeight / 2) + (elementHeight / 2);
        break;
    }
    
    // 确保不超出边界
    const maxScroll = container.scrollHeight - containerHeight;
    return Math.max(0, Math.min(targetScroll, maxScroll));
  };
  
  /**
   * 执行平滑滚动
   */
  const performScroll = async (targetPosition, smooth) => {
    const container = scrollContainer.value;
    
    if (!smooth) {
      container.scrollTop = targetPosition;
      return;
    }
    
    // 使用 requestAnimationFrame 实现平滑滚动
    const startPosition = container.scrollTop;
    const distance = targetPosition - startPosition;
    const startTime = performance.now();
    
    return new Promise(resolve => {
      const scrollStep = (currentTime) => {
        const elapsed = currentTime - startTime;
        const progress = Math.min(elapsed / SCROLL_DURATION, 1);
        
        // 缓动函数
        const easeProgress = easeInOutCubic(progress);
        container.scrollTop = startPosition + (distance * easeProgress);
        
        if (progress < 1) {
          requestAnimationFrame(scrollStep);
        } else {
          resolve();
        }
      };
      
      requestAnimationFrame(scrollStep);
    });
  };
  
  /**
   * 应用高亮效果
   */
  const applyHighlight = (element) => {
    // 移除之前的高亮
    const previousHighlight = scrollContainer.value.querySelector('.message-highlight');
    if (previousHighlight) {
      previousHighlight.classList.remove('message-highlight');
    }
    
    // 添加高亮类
    element.classList.add('message-highlight');
    
    // 自动移除高亮
    setTimeout(() => {
      element.classList.remove('message-highlight');
    }, HIGHLIGHT_DURATION);
  };
  
  /**
   * 等待渲染完成
   */
  const waitForRender = () => {
    return new Promise(resolve => {
      requestAnimationFrame(() => {
        requestAnimationFrame(resolve);
      });
    });
  };
  
  /**
   * 缓动函数
   */
  const easeInOutCubic = (t) => {
    return t < 0.5
      ? 4 * t * t * t
      : 1 - Math.pow(-2 * t + 2, 3) / 2;
  };
  
  /**
   * 批量滚动到多个消息
   */
  const scrollToMessages = async (messageIds, options = {}) => {
    for (const messageId of messageIds) {
      await scrollToMessage(messageId, { ...options, smooth: false });
      await new Promise(resolve => setTimeout(resolve, 500));
    }
  };
  
  /**
   * 清除焦点
   */
  const clearFocus = () => {
    focusedMessageId.value = null;
    
    const highlightedElement = scrollContainer.value?.querySelector('.message-highlight');
    if (highlightedElement) {
      highlightedElement.classList.remove('message-highlight');
    }
  };
  
  /**
   * 设置滚动容器
   */
  const setScrollContainer = (container) => {
    scrollContainer.value = container;
  };
  
  return {
    // 状态
    focusedMessageId,
    isScrolling,
    
    // 方法
    scrollToMessage,
    scrollToMessages,
    clearFocus,
    setScrollContainer,
    
    // 工具方法
    findMessageElement,
    ensureMessageVisible
  };
}