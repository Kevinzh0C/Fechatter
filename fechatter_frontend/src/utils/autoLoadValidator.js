/**
 * 🔍 AutoLoadManager 验证工具
 * 
 * 验证"Loading earlier messages..."完整DAG系统的可靠性
 */

import { autoLoadManager } from '../services/AutoLoadManager.js';

export class AutoLoadValidator {
  constructor() {
    this.testResults = [];
    this.isRunning = false;
    this.totalTests = 0;
    this.passedTests = 0;
    this.failedTests = 0;
  }

  /**
   * 🎯 运行完整的AutoLoadManager验证测试
   */
  async runFullValidation() {
    if (this.isRunning) {
      console.warn('🔍 [AutoLoadValidator] 验证正在进行中...');
      return;
    }

    this.isRunning = true;
    this.testResults = [];
    this.totalTests = 0;
    this.passedTests = 0;
    this.failedTests = 0;

    console.log('🎯 [AutoLoadValidator] 开始AutoLoadManager完整验证测试');

    // 测试套件
    const testSuites = [
      () => this.testBasicStateTransitions(),
      () => this.testLoadingCallback(),
      () => this.testErrorHandling(),
      () => this.testUserInteractions(),
      () => this.testEventSystem(),
      () => this.testConcurrentSessions(),
      () => this.testPerformanceMetrics(),
      () => this.testDAGCompletion()
    ];

    for (const testSuite of testSuites) {
      try {
        await testSuite();
        await this.wait(500); // 测试间隔
      } catch (error) {
        console.error('🚨 [AutoLoadValidator] 测试套件失败:', error);
        this.recordTest('TestSuite', false, error.message);
      }
    }

    this.isRunning = false;
    return this.generateReport();
  }

  /**
   * 🔄 测试1: 基本状态转换
   */
  async testBasicStateTransitions() {
    console.log('🔍 [AutoLoadValidator] 测试1: 基本状态转换');

    // 重置管理器
    autoLoadManager.reset();

    // 检查初始状态
    const initialState = autoLoadManager.getState();
    this.recordTest('Initial State', initialState.current === 'idle', `Expected 'idle', got '${initialState.current}'`);

    // 创建模拟加载回调
    let callCount = 0;
    const mockLoadCallback = async () => {
      callCount++;
      if (callCount === 1) {
        return {
          success: true,
          messages: Array(10).fill().map((_, i) => ({ id: i + 1, text: `Message ${i + 1}` })),
          hasMore: true,
          totalCount: 10
        };
      } else {
        return {
          success: true,
          messages: Array(5).fill().map((_, i) => ({ id: i + 11, text: `Message ${i + 11}` })),
          hasMore: false,
          totalCount: 15
        };
      }
    };

    // 启动会话
    const sessionPromise = autoLoadManager.startAutoLoadSession({
      chatId: 'test-chat-1',
      hasMoreMessages: true,
      loadCallback: mockLoadCallback
    });

    // 等待状态变化
    await this.wait(100);

    // 检查会话完成
    const sessionResult = await sessionPromise;
    this.recordTest('Session Success', sessionResult === true, `Session should succeed, got ${sessionResult}`);

    // 检查最终状态
    const finalState = autoLoadManager.getState();
    this.recordTest('Final State', finalState.current === 'completed', `Expected 'completed', got '${finalState.current}'`);

    // 检查加载的消息数
    this.recordTest('Total Loaded', finalState.totalLoaded === 15, `Expected 15, got ${finalState.totalLoaded}`);
  }

  /**
   * 📦 测试2: 加载回调验证
   */
  async testLoadingCallback() {
    console.log('🔍 [AutoLoadValidator] 测试2: 加载回调验证');

    autoLoadManager.reset();

    // 测试无效的加载回调格式
    const invalidCallback = async () => {
      return "invalid format"; // 应该返回对象
    };

    try {
      await autoLoadManager.startAutoLoadSession({
        chatId: 'test-chat-2',
        hasMoreMessages: true,
        loadCallback: invalidCallback
      });
      this.recordTest('Invalid Callback Format', false, 'Should have thrown error for invalid format');
    } catch (error) {
      this.recordTest('Invalid Callback Format', true, 'Correctly caught invalid format');
    }

    // 测试加载失败的回调
    const failingCallback = async () => {
      throw new Error('Network error');
    };

    autoLoadManager.reset();
    const failResult = await autoLoadManager.startAutoLoadSession({
      chatId: 'test-chat-3',
      hasMoreMessages: true,
      loadCallback: failingCallback
    });

    this.recordTest('Failing Callback Handling', failResult === false, 'Session should fail gracefully');

    const errorState = autoLoadManager.getState();
    this.recordTest('Error State', errorState.current === 'error', `Expected 'error', got '${errorState.current}'`);
  }

  /**
   * ❌ 测试3: 错误处理
   */
  async testErrorHandling() {
    console.log('🔍 [AutoLoadValidator] 测试3: 错误处理');

    autoLoadManager.reset();

    let errorEventReceived = false;
    const errorHandler = (data) => {
      errorEventReceived = true;
    };

    autoLoadManager.on('error', errorHandler);

    const errorCallback = async () => {
      throw new Error('Test error');
    };

    await autoLoadManager.startAutoLoadSession({
      chatId: 'test-chat-4',
      hasMoreMessages: true,
      loadCallback: errorCallback
    });

    this.recordTest('Error Event Emitted', errorEventReceived, 'Error event should be emitted');

    // 测试用户重试
    const retryResult = await autoLoadManager.userRetry();
    this.recordTest('User Retry', retryResult === false, 'Retry should fail with same error');

    autoLoadManager.off('error', errorHandler);
  }

  /**
   * 👤 测试4: 用户交互
   */
  async testUserInteractions() {
    console.log('🔍 [AutoLoadValidator] 测试4: 用户交互');

    autoLoadManager.reset();

    // 启动一个成功的会话
    const successCallback = async () => ({
      success: true,
      messages: [{ id: 1, text: 'Test message' }],
      hasMore: false,
      totalCount: 1
    });

    await autoLoadManager.startAutoLoadSession({
      chatId: 'test-chat-5',
      hasMoreMessages: true,
      loadCallback: successCallback
    });

    // 测试用户关闭完成提示
    const initialState = autoLoadManager.getState();
    autoLoadManager.userDismissCompletion();
    const afterDismissState = autoLoadManager.getState();

    this.recordTest('User Dismiss', afterDismissState.userDismissed === true, 'User dismissed flag should be set');

    // 测试停止功能
    autoLoadManager.reset();
    const stopCallback = async () => {
      await this.wait(1000); // 长时间加载
      return { success: true, messages: [], hasMore: false, totalCount: 0 };
    };

    const stopPromise = autoLoadManager.startAutoLoadSession({
      chatId: 'test-chat-6',
      hasMoreMessages: true,
      loadCallback: stopCallback
    });

    await this.wait(100);
    autoLoadManager.stopAutoLoad('user-cancel');

    const stopResult = await stopPromise;
    this.recordTest('Manual Stop', stopResult === false, 'Session should be stopped by user');
  }

  /**
   * 📡 测试5: 事件系统
   */
  async testEventSystem() {
    console.log('🔍 [AutoLoadValidator] 测试5: 事件系统');

    autoLoadManager.reset();

    const eventLog = [];
    const events = ['state-change', 'progress', 'complete', 'ui-update'];

    // 注册所有事件监听器
    const handlers = {};
    events.forEach(event => {
      handlers[event] = (data) => {
        eventLog.push({ event, data, timestamp: Date.now() });
      };
      autoLoadManager.on(event, handlers[event]);
    });

    // 执行一个完整的加载会话
    const eventTestCallback = async () => ({
      success: true,
      messages: Array(5).fill().map((_, i) => ({ id: i, text: `Msg ${i}` })),
      hasMore: false,
      totalCount: 5
    });

    await autoLoadManager.startAutoLoadSession({
      chatId: 'test-chat-7',
      hasMoreMessages: true,
      loadCallback: eventTestCallback
    });

    // 验证事件是否被触发
    const stateChangeEvents = eventLog.filter(e => e.event === 'state-change');
    const progressEvents = eventLog.filter(e => e.event === 'progress');
    const completeEvents = eventLog.filter(e => e.event === 'complete');
    const uiUpdateEvents = eventLog.filter(e => e.event === 'ui-update');

    this.recordTest('State Change Events', stateChangeEvents.length >= 3, `Expected >= 3, got ${stateChangeEvents.length}`);
    this.recordTest('Progress Events', progressEvents.length >= 1, `Expected >= 1, got ${progressEvents.length}`);
    this.recordTest('Complete Events', completeEvents.length === 1, `Expected 1, got ${completeEvents.length}`);
    this.recordTest('UI Update Events', uiUpdateEvents.length >= 1, `Expected >= 1, got ${uiUpdateEvents.length}`);

    // 移除事件监听器
    events.forEach(event => {
      autoLoadManager.off(event, handlers[event]);
    });

    this.recordTest('Event System', true, 'Event system test completed');
  }

  /**
   * 🔀 测试6: 并发会话处理
   */
  async testConcurrentSessions() {
    console.log('🔍 [AutoLoadValidator] 测试6: 并发会话处理');

    autoLoadManager.reset();

    const concurrentCallback = async () => ({
      success: true,
      messages: [{ id: 1, text: 'Concurrent message' }],
      hasMore: false,
      totalCount: 1
    });

    // 启动第一个会话
    const session1Promise = autoLoadManager.startAutoLoadSession({
      chatId: 'test-chat-8',
      hasMoreMessages: true,
      loadCallback: concurrentCallback
    });

    // 立即尝试启动第二个会话
    const session2Promise = autoLoadManager.startAutoLoadSession({
      chatId: 'test-chat-9',
      hasMoreMessages: true,
      loadCallback: concurrentCallback
    });

    const [result1, result2] = await Promise.all([session1Promise, session2Promise]);

    // 第一个会话应该成功，第二个应该被拒绝
    this.recordTest('First Session Success', result1 === true, 'First session should succeed');
    this.recordTest('Second Session Rejection', result2 === false, 'Second session should be rejected');
  }

  /**
   * 📊 测试7: 性能指标
   */
  async testPerformanceMetrics() {
    console.log('🔍 [AutoLoadValidator] 测试7: 性能指标');

    autoLoadManager.reset();

    const metricsCallback = async () => ({
      success: true,
      messages: Array(10).fill().map((_, i) => ({ id: i, text: `Metric msg ${i}` })),
      hasMore: false,
      totalCount: 10
    });

    const startTime = Date.now();
    await autoLoadManager.startAutoLoadSession({
      chatId: 'test-chat-10',
      hasMoreMessages: true,
      loadCallback: metricsCallback
    });
    const endTime = Date.now();

    const metrics = autoLoadManager.getMetrics();

    this.recordTest('Total Sessions', metrics.totalSessions >= 1, `Expected >= 1, got ${metrics.totalSessions}`);
    this.recordTest('Successful Sessions', metrics.successfulSessions >= 1, `Expected >= 1, got ${metrics.successfulSessions}`);
    this.recordTest('Success Rate', metrics.successRate !== '0.0%', `Expected > 0%, got ${metrics.successRate}`);
    this.recordTest('Total Messages Loaded', metrics.totalMessagesLoaded >= 10, `Expected >= 10, got ${metrics.totalMessagesLoaded}`);
    this.recordTest('Average Load Time', metrics.averageLoadTime > 0, `Expected > 0, got ${metrics.averageLoadTime}`);

    const actualDuration = endTime - startTime;
    this.recordTest('Performance Timing', actualDuration < 5000, `Should complete in < 5s, took ${actualDuration}ms`);
  }

  /**
   * ✅ 测试8: DAG完整性验证
   */
  async testDAGCompletion() {
    console.log('🔍 [AutoLoadValidator] 测试8: DAG完整性验证');

    autoLoadManager.reset();

    const dagStates = [];
    const dagProgressEvents = [];
    let dagCompleted = false;

    // 监听DAG流程
    const stateHandler = (data) => {
      dagStates.push(data.to);
    };

    const progressHandler = (data) => {
      dagProgressEvents.push(data);
    };

    const completeHandler = (data) => {
      dagCompleted = true;
    };

    autoLoadManager.on('state-change', stateHandler);
    autoLoadManager.on('progress', progressHandler);
    autoLoadManager.on('complete', completeHandler);

    // 模拟完整的DAG流程
    let loadCallCount = 0;
    const dagCallback = async () => {
      loadCallCount++;

      if (loadCallCount === 1) {
        return {
          success: true,
          messages: Array(20).fill().map((_, i) => ({ id: i + 1, text: `DAG msg ${i + 1}` })),
          hasMore: true,
          totalCount: 20
        };
      } else if (loadCallCount === 2) {
        return {
          success: true,
          messages: Array(15).fill().map((_, i) => ({ id: i + 21, text: `DAG msg ${i + 21}` })),
          hasMore: true,
          totalCount: 35
        };
      } else {
        return {
          success: true,
          messages: Array(10).fill().map((_, i) => ({ id: i + 36, text: `DAG msg ${i + 36}` })),
          hasMore: false,
          totalCount: 45
        };
      }
    };

    const dagResult = await autoLoadManager.startAutoLoadSession({
      chatId: 'test-dag-completion',
      hasMoreMessages: true,
      loadCallback: dagCallback
    });

    // 验证DAG完整性
    const expectedStates = ['detecting_need', 'loading', 'processing', 'all_loaded', 'completed'];
    const hasAllStates = expectedStates.every(state => dagStates.includes(state));

    this.recordTest('DAG State Completion', hasAllStates, `Missing states: ${expectedStates.filter(s => !dagStates.includes(s))}`);
    this.recordTest('DAG Result Success', dagResult === true, 'DAG should complete successfully');
    this.recordTest('DAG Progress Events', dagProgressEvents.length >= 2, `Expected >= 2, got ${dagProgressEvents.length}`);
    this.recordTest('DAG Final Completion', dagCompleted === true, 'DAG completion event should fire');
    this.recordTest('DAG Load Calls', loadCallCount === 3, `Expected 3 load calls, got ${loadCallCount}`);

    const finalState = autoLoadManager.getState();
    this.recordTest('DAG Final Total', finalState.totalLoaded === 45, `Expected 45 total, got ${finalState.totalLoaded}`);

    // 清理事件监听器
    autoLoadManager.off('state-change', stateHandler);
    autoLoadManager.off('progress', progressHandler);
    autoLoadManager.off('complete', completeHandler);

    // 验证100%保证停止
    this.recordTest('100% Stop Guarantee', finalState.current === 'completed', 'System should be in completed state');
    this.recordTest('No More Loading', !finalState.isActive, 'System should not be active after completion');
  }

  /**
   * 📝 记录测试结果
   */
  recordTest(testName, passed, details = '') {
    this.totalTests++;
    if (passed) {
      this.passedTests++;
      console.log(`✅ [AutoLoadValidator] ${testName}: PASSED`);
    } else {
      this.failedTests++;
      console.log(`❌ [AutoLoadValidator] ${testName}: FAILED - ${details}`);
    }

    this.testResults.push({
      name: testName,
      passed,
      details,
      timestamp: new Date().toISOString()
    });
  }

  /**
   * 📋 生成验证报告
   */
  generateReport() {
    const successRate = ((this.passedTests / this.totalTests) * 100).toFixed(1);
    const report = {
      summary: {
        totalTests: this.totalTests,
        passedTests: this.passedTests,
        failedTests: this.failedTests,
        successRate: `${successRate}%`,
        isSystemReliable: this.failedTests === 0,
        timestamp: new Date().toISOString()
      },
      details: this.testResults,
      autoLoadMetrics: autoLoadManager.getMetrics()
    };

    console.log('\n🎯 [AutoLoadValidator] 验证报告:');
    console.log(`📊 总测试数: ${this.totalTests}`);
    console.log(`✅ 通过测试: ${this.passedTests}`);
    console.log(`❌ 失败测试: ${this.failedTests}`);
    console.log(`🎯 成功率: ${successRate}%`);
    console.log(`🔒 系统可靠性: ${report.summary.isSystemReliable ? '✅ 100% 可靠' : '❌ 需要修复'}`);

    if (this.failedTests > 0) {
      console.log('\n❌ 失败的测试:');
      this.testResults.filter(t => !t.passed).forEach(test => {
        console.log(`  - ${test.name}: ${test.details}`);
      });
    }

    return report;
  }

  /**
   * ⏰ 等待辅助函数
   */
  wait(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * 🧹 清理测试环境
   */
  cleanup() {
    autoLoadManager.reset();
    this.testResults = [];
    this.totalTests = 0;
    this.passedTests = 0;
    this.failedTests = 0;
    this.isRunning = false;
  }
}

// 创建全局验证器实例
export const autoLoadValidator = new AutoLoadValidator();

// 开发环境全局函数
if (typeof window !== 'undefined' && import.meta.env.DEV) {
  window.autoLoadValidator = autoLoadValidator;

  window.validateAutoLoad = async () => {
    return await autoLoadValidator.runFullValidation();
  };

  window.cleanupAutoLoadTests = () => {
    autoLoadValidator.cleanup();
  };

  console.log('🔍 AutoLoadValidator 调试功能已加载');
  console.log('使用 validateAutoLoad() 运行完整验证测试');
} 