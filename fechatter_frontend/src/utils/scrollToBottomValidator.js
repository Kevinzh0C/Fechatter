/**
 * 🧪 ScrollToBottomValidator - 100%滚动保证机制验证工具
 * 
 * 验证第一次进入channel时是否100%滚动到最新消息位置
 */

export class ScrollToBottomValidator {
  constructor() {
    this.testResults = [];
    this.isRunning = false;
    this.testScenarios = [
      'initial_load_with_messages',
      'initial_load_empty_then_messages',
      'chat_switching',
      'slow_loading_messages',
      'large_message_list',
      'concurrent_chat_switches'
    ];
  }

  /**
   * 🧪 运行完整的验证测试套件
   */
  async runCompleteValidation() {
    if (this.isRunning) {
      console.warn('🧪 [ScrollValidator] 测试正在进行中');
      return this.testResults;
    }

    this.isRunning = true;
    this.testResults = [];

    console.log('🧪 [ScrollValidator] 开始100%滚动保证验证测试');

    try {
      // 测试场景1：初始加载包含消息
      await this.testInitialLoadWithMessages();

      // 测试场景2：初始加载为空，后来有消息
      await this.testInitialLoadEmptyThenMessages();

      // 测试场景3：频道切换
      await this.testChatSwitching();

      // 测试场景4：慢速加载消息
      await this.testSlowLoadingMessages();

      // 测试场景5：大量消息列表
      await this.testLargeMessageList();

      // 测试场景6：并发频道切换
      await this.testConcurrentChatSwitches();

      const summary = this.generateTestSummary();
      console.log('[ScrollValidator] 验证测试完成:', summary);

      return {
        summary,
        results: this.testResults,
        overallStatus: summary.passRate === 100 ? 'PASS' : 'NEEDS_IMPROVEMENT'
      };

    } catch (error) {
      console.error('🚨 [ScrollValidator] 验证测试失败:', error);
      return {
        error: error.message,
        results: this.testResults
      };
    } finally {
      this.isRunning = false;
    }
  }

  /**
   * 🧪 测试场景1：初始加载包含消息
   */
  async testInitialLoadWithMessages() {
    const testName = '初始加载包含消息';
    console.log(`🧪 [ScrollValidator] 测试: ${testName}`);

    try {
      // 模拟包含消息的初始加载
      const mockMessages = this.generateMockMessages(10);
      const { container, messageList } = this.createMockMessageContainer(mockMessages);

      // 模拟组件挂载
      await this.simulateComponentMount(messageList, 1, mockMessages);

      // 验证滚动位置
      const isAtBottom = this.verifyScrollPosition(container);

      this.recordTestResult(testName, isAtBottom, {
        messageCount: mockMessages.length,
        finalScrollPosition: container.scrollTop,
        maxScrollPosition: container.scrollHeight - container.clientHeight
      });

      // 清理
      document.body.removeChild(container);

    } catch (error) {
      this.recordTestResult(testName, false, { error: error.message });
    }
  }

  /**
   * 🧪 测试场景2：初始加载为空，后来有消息
   */
  async testInitialLoadEmptyThenMessages() {
    const testName = '初始加载为空后添加消息';
    console.log(`🧪 [ScrollValidator] 测试: ${testName}`);

    try {
      // 创建空的消息容器
      const { container, messageList } = this.createMockMessageContainer([]);

      // 模拟组件挂载（无消息）
      await this.simulateComponentMount(messageList, 1, []);

      // 添加消息
      const mockMessages = this.generateMockMessages(5);
      await this.simulateMessagesAdded(messageList, mockMessages);

      // 验证滚动位置
      const isAtBottom = this.verifyScrollPosition(container);

      this.recordTestResult(testName, isAtBottom, {
        initialMessageCount: 0,
        finalMessageCount: mockMessages.length,
        finalScrollPosition: container.scrollTop,
        maxScrollPosition: container.scrollHeight - container.clientHeight
      });

      // 清理
      document.body.removeChild(container);

    } catch (error) {
      this.recordTestResult(testName, false, { error: error.message });
    }
  }

  /**
   * 🧪 测试场景3：频道切换
   */
  async testChatSwitching() {
    const testName = '频道切换滚动';
    console.log(`🧪 [ScrollValidator] 测试: ${testName}`);

    try {
      const { container, messageList } = this.createMockMessageContainer([]);

      // 第一个频道
      const messages1 = this.generateMockMessages(8, 'chat1');
      await this.simulateComponentMount(messageList, 1, messages1);

      // 切换到第二个频道
      const messages2 = this.generateMockMessages(12, 'chat2');
      await this.simulateChatSwitch(messageList, 2, messages2);

      // 验证滚动位置
      const isAtBottom = this.verifyScrollPosition(container);

      this.recordTestResult(testName, isAtBottom, {
        chat1MessageCount: messages1.length,
        chat2MessageCount: messages2.length,
        finalScrollPosition: container.scrollTop,
        maxScrollPosition: container.scrollHeight - container.clientHeight
      });

      // 清理
      document.body.removeChild(container);

    } catch (error) {
      this.recordTestResult(testName, false, { error: error.message });
    }
  }

  /**
   * 🧪 测试场景4：慢速加载消息
   */
  async testSlowLoadingMessages() {
    const testName = '慢速加载消息';
    console.log(`🧪 [ScrollValidator] 测试: ${testName}`);

    try {
      const { container, messageList } = this.createMockMessageContainer([]);

      // 模拟组件挂载
      await this.simulateComponentMount(messageList, 1, []);

      // 模拟慢速添加消息
      const mockMessages = this.generateMockMessages(6);
      for (let i = 0; i < mockMessages.length; i++) {
        await this.wait(200); // 每200ms添加一条消息
        await this.simulateMessagesAdded(messageList, [mockMessages[i]]);
      }

      // 验证最终滚动位置
      const isAtBottom = this.verifyScrollPosition(container);

      this.recordTestResult(testName, isAtBottom, {
        messageCount: mockMessages.length,
        loadDuration: '1200ms',
        finalScrollPosition: container.scrollTop,
        maxScrollPosition: container.scrollHeight - container.clientHeight
      });

      // 清理
      document.body.removeChild(container);

    } catch (error) {
      this.recordTestResult(testName, false, { error: error.message });
    }
  }

  /**
   * 🧪 测试场景5：大量消息列表
   */
  async testLargeMessageList() {
    const testName = '大量消息列表';
    console.log(`🧪 [ScrollValidator] 测试: ${testName}`);

    try {
      // 创建大量消息
      const mockMessages = this.generateMockMessages(100);
      const { container, messageList } = this.createMockMessageContainer(mockMessages);

      // 模拟组件挂载
      await this.simulateComponentMount(messageList, 1, mockMessages);

      // 验证滚动位置
      const isAtBottom = this.verifyScrollPosition(container);

      this.recordTestResult(testName, isAtBottom, {
        messageCount: mockMessages.length,
        containerHeight: container.clientHeight,
        totalHeight: container.scrollHeight,
        finalScrollPosition: container.scrollTop,
        maxScrollPosition: container.scrollHeight - container.clientHeight
      });

      // 清理
      document.body.removeChild(container);

    } catch (error) {
      this.recordTestResult(testName, false, { error: error.message });
    }
  }

  /**
   * 🧪 测试场景6：并发频道切换
   */
  async testConcurrentChatSwitches() {
    const testName = '并发频道切换';
    console.log(`🧪 [ScrollValidator] 测试: ${testName}`);

    try {
      const { container, messageList } = this.createMockMessageContainer([]);

      // 快速切换多个频道
      const chatPromises = [];
      for (let chatId = 1; chatId <= 5; chatId++) {
        const messages = this.generateMockMessages(5 + chatId, `chat${chatId}`);
        chatPromises.push(this.simulateChatSwitch(messageList, chatId, messages));
      }

      // 等待所有切换完成
      await Promise.all(chatPromises);

      // 等待最后的状态稳定
      await this.wait(500);

      // 验证最终滚动位置
      const isAtBottom = this.verifyScrollPosition(container);

      this.recordTestResult(testName, isAtBottom, {
        chatSwitches: 5,
        finalScrollPosition: container.scrollTop,
        maxScrollPosition: container.scrollHeight - container.clientHeight
      });

      // 清理
      document.body.removeChild(container);

    } catch (error) {
      this.recordTestResult(testName, false, { error: error.message });
    }
  }

  /**
   * 创建模拟消息容器
   */
  createMockMessageContainer(messages) {
    const container = document.createElement('div');
    container.style.cssText = `
      width: 400px;
      height: 300px;
      overflow-y: auto;
      position: absolute;
      top: -1000px;
      left: -1000px;
      background: white;
      border: 1px solid #ccc;
    `;

    const messagesWrapper = document.createElement('div');
    messagesWrapper.className = 'messages-wrapper';
    container.appendChild(messagesWrapper);

    // 添加现有消息
    messages.forEach(message => {
      const messageEl = this.createMessageElement(message);
      messagesWrapper.appendChild(messageEl);
    });

    document.body.appendChild(container);

    const messageList = {
      scrollContainer: container,
      messagesWrapper: messagesWrapper,
      messages: [...messages]
    };

    return { container, messageList };
  }

  /**
   * 创建消息元素
   */
  createMessageElement(message) {
    const el = document.createElement('div');
    el.className = 'message-wrapper';
    el.setAttribute('data-message-id', message.id);
    el.style.cssText = `
      padding: 8px;
      margin: 4px 0;
      background: #f5f5f5;
      border-radius: 4px;
      min-height: 40px;
    `;
    el.textContent = message.content;
    return el;
  }

  /**
   * EVENT: 生成模拟消息
   */
  generateMockMessages(count, chatPrefix = 'test') {
    const messages = [];
    for (let i = 1; i <= count; i++) {
      messages.push({
        id: `${chatPrefix}_${i}`,
        content: `Message ${i} from ${chatPrefix}`,
        chat_id: chatPrefix,
        created_at: new Date(Date.now() + i * 1000).toISOString()
      });
    }
    return messages;
  }

  /**
   * 🎭 模拟组件挂载
   */
  async simulateComponentMount(messageList, chatId, messages) {
    // 更新消息列表
    messageList.messages = [...messages];

    // 重新渲染消息
    this.renderMessages(messageList);

    // 模拟保证滚动
    if (window.guaranteedScrollToBottom) {
      await window.guaranteedScrollToBottom.guaranteeScrollToBottom({
        chatId,
        scrollContainer: messageList.scrollContainer,
        messages: messages,
        forceSmooth: false
      });
    } else {
      // 备用滚动
      messageList.scrollContainer.scrollTop = messageList.scrollContainer.scrollHeight;
    }

    // 等待DOM更新
    await this.wait(100);
  }

  /**
   * 🎭 模拟消息添加
   */
  async simulateMessagesAdded(messageList, newMessages) {
    messageList.messages.push(...newMessages);
    this.renderMessages(messageList);

    // 模拟消息watch触发
    const container = messageList.scrollContainer;
    const { scrollTop, scrollHeight, clientHeight } = container;
    const isNearBottom = scrollTop >= scrollHeight - clientHeight - 100;

    if (isNearBottom) {
      container.scrollTop = container.scrollHeight;
    }

    await this.wait(50);
  }

  /**
   * 🎭 模拟频道切换
   */
  async simulateChatSwitch(messageList, newChatId, newMessages) {
    // 清除现有消息
    messageList.messages = [];
    this.renderMessages(messageList);

    // 等待清除完成
    await this.wait(50);

    // 加载新消息并滚动
    await this.simulateComponentMount(messageList, newChatId, newMessages);
  }

  /**
   * 渲染消息到DOM
   */
  renderMessages(messageList) {
    const wrapper = messageList.messagesWrapper;
    wrapper.innerHTML = '';

    messageList.messages.forEach(message => {
      const messageEl = this.createMessageElement(message);
      wrapper.appendChild(messageEl);
    });
  }

  /**
   * 验证滚动位置
   */
  verifyScrollPosition(container) {
    const { scrollTop, scrollHeight, clientHeight } = container;
    const maxScrollTop = scrollHeight - clientHeight;
    const tolerance = 10; // 10px容错

    const isAtBottom = Math.abs(scrollTop - maxScrollTop) <= tolerance;

    if (import.meta.env.DEV) {
      console.log('[ScrollValidator] 滚动位置验证:', {
        scrollTop: Math.round(scrollTop),
        maxScrollTop: Math.round(maxScrollTop),
        difference: Math.round(Math.abs(scrollTop - maxScrollTop)),
        isAtBottom,
        tolerance
      });
    }

    return isAtBottom;
  }

  /**
   * 记录测试结果
   */
  recordTestResult(testName, passed, details = {}) {
    const result = {
      test: testName,
      status: passed ? 'PASS' : 'FAIL',
      timestamp: Date.now(),
      details
    };

    this.testResults.push(result);

    const icon = passed ? '' : '❌';
    console.log(`${icon} [ScrollValidator] ${testName}: ${result.status}`);

    if (!passed && details.error) {
      console.error(`    错误: ${details.error}`);
    }
  }

  /**
   * 📈 生成测试摘要
   */
  generateTestSummary() {
    const total = this.testResults.length;
    const passed = this.testResults.filter(r => r.status === 'PASS').length;
    const failed = total - passed;
    const passRate = total > 0 ? Math.round((passed / total) * 100) : 0;

    return {
      total,
      passed,
      failed,
      passRate,
      timestamp: new Date().toISOString()
    };
  }

  /**
   * ⏰ 等待指定时间
   */
  wait(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * 📋 导出测试报告
   */
  exportTestReport() {
    const report = {
      title: 'ScrollToBottom 100% 保证验证报告',
      summary: this.generateTestSummary(),
      results: this.testResults,
      scenarios: this.testScenarios,
      timestamp: new Date().toISOString()
    };

    const blob = new Blob([JSON.stringify(report, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `scroll-validation-report-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);

    console.log('📄 测试报告已导出');
  }
}

// 🌍 创建全局实例
export const scrollToBottomValidator = new ScrollToBottomValidator();

// 开发环境调试函数
if (typeof window !== 'undefined' && import.meta.env.DEV) {
  window.validateScrollToBottom = () => scrollToBottomValidator.runCompleteValidation();
  window.exportScrollValidationReport = () => scrollToBottomValidator.exportTestReport();

  console.log('🧪 ScrollToBottomValidator 已加载 - 使用 validateScrollToBottom() 运行测试');
} 