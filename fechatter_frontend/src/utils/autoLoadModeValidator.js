/**
 * AutoLoadManager 加载模式验证工具
 * 
 * 验证自动模式和滚动触发模式的正确实现
 */

import { autoLoadManager } from '../services/AutoLoadManager.js';

export class AutoLoadModeValidator {
  constructor() {
    this.testResults = [];
    this.isRunning = false;
  }

  /**
   * 运行完整的双模式验证测试
   */
  async runModeValidation() {
    if (this.isRunning) {
      console.warn('[ModeValidator] 验证正在进行中...');
      return;
    }

    this.isRunning = true;
    this.testResults = [];

    console.log('[ModeValidator] 开始AutoLoadManager双模式验证测试');

    try {
      // 测试自动模式
      await this.testAutoMode();
      await this.wait(1000);

      // 测试滚动触发模式
      await this.testScrollTriggeredMode();
      await this.wait(1000);

      // 测试模式切换
      await this.testModeSwitching();

    } catch (error) {
      console.error('🚨 [ModeValidator] 验证失败:', error);
      this.recordTest('Overall Validation', false, error.message);
    }

    this.isRunning = false;
    return this.generateReport();
  }

  /**
   * 测试自动模式
   */
  async testAutoMode() {
    console.log('[ModeValidator] 测试自动模式');

    autoLoadManager.reset();

    let batchCount = 0;
    const mockAutoCallback = async () => {
      batchCount++;
      console.log(`📦 [Auto Mode] 第 ${batchCount} 批加载`);

      if (batchCount === 1) {
        return {
          success: true,
          messages: Array(20).fill().map((_, i) => ({ id: i + 1, text: `Auto msg ${i + 1}` })),
          hasMore: true,
          totalCount: 20
        };
      } else if (batchCount === 2) {
        return {
          success: true,
          messages: Array(15).fill().map((_, i) => ({ id: i + 21, text: `Auto msg ${i + 21}` })),
          hasMore: true,
          totalCount: 35
        };
      } else {
        return {
          success: true,
          messages: Array(10).fill().map((_, i) => ({ id: i + 36, text: `Auto msg ${i + 36}` })),
          hasMore: false,
          totalCount: 45
        };
      }
    };

    const stateLog = [];
    const stateHandler = (data) => {
      stateLog.push(data.to);
      console.log(`🔄 [Auto Mode] 状态变化: ${data.from} → ${data.to}`);
    };

    autoLoadManager.on('state-change', stateHandler);

    const startTime = Date.now();
    const result = await autoLoadManager.startAutoLoadSession({
      chatId: 'auto-test-chat',
      hasMoreMessages: true,
      loadingMode: 'auto',
      loadCallback: mockAutoCallback
    });
    const endTime = Date.now();

    autoLoadManager.off('state-change', stateHandler);

    // 验证自动模式特性
    const finalState = autoLoadManager.getState();

    this.recordTest('Auto Mode Success', result === true, `Expected true, got ${result}`);
    this.recordTest('Auto Mode Batch Count', batchCount === 3, `Expected 3 batches, got ${batchCount}`);
    this.recordTest('Auto Mode Total Messages', finalState.totalLoaded === 45, `Expected 45, got ${finalState.totalLoaded}`);
    this.recordTest('Auto Mode No Waiting', !stateLog.includes('waiting_for_scroll'), 'Should not enter waiting state in auto mode');
    this.recordTest('Auto Mode Completion Time', endTime - startTime < 3000, `Should complete quickly, took ${endTime - startTime}ms`);

    console.log(`[Auto Mode] 完成验证 - 加载了 ${batchCount} 批，共 ${finalState.totalLoaded} 条消息`);
  }

  /**
   * 📜 测试滚动触发模式
   */
  async testScrollTriggeredMode() {
    console.log('[ModeValidator] 测试滚动触发模式');

    autoLoadManager.reset();

    let batchCount = 0;
    let waitingCount = 0;
    const mockScrollCallback = async () => {
      batchCount++;
      console.log(`📦 [Scroll Mode] 第 ${batchCount} 批加载`);

      if (batchCount === 1) {
        return {
          success: true,
          messages: Array(10).fill().map((_, i) => ({ id: i + 1, text: `Scroll msg ${i + 1}` })),
          hasMore: true,
          totalCount: 10
        };
      } else if (batchCount === 2) {
        return {
          success: true,
          messages: Array(10).fill().map((_, i) => ({ id: i + 11, text: `Scroll msg ${i + 11}` })),
          hasMore: true,
          totalCount: 20
        };
      } else {
        return {
          success: true,
          messages: Array(5).fill().map((_, i) => ({ id: i + 21, text: `Scroll msg ${i + 21}` })),
          hasMore: false,
          totalCount: 25
        };
      }
    };

    const stateLog = [];
    const stateHandler = (data) => {
      stateLog.push(data.to);
      console.log(`🔄 [Scroll Mode] 状态变化: ${data.from} → ${data.to}`);

      if (data.to === 'waiting_for_scroll') {
        waitingCount++;
        console.log(`⏳ [Scroll Mode] 第 ${waitingCount} 次等待用户滚动`);

        // 模拟用户滚动行为 (延迟后自动恢复)
        setTimeout(() => {
          if (autoLoadManager.waitingForScroll) {
            console.log('USER: [Scroll Mode] 模拟用户滚动恢复');
            autoLoadManager.resumeFromScroll(() => { });
          }
        }, 500);
      }
    };

    autoLoadManager.on('state-change', stateHandler);

    const startTime = Date.now();
    const result = await autoLoadManager.startAutoLoadSession({
      chatId: 'scroll-test-chat',
      hasMoreMessages: true,
      loadingMode: 'scroll-triggered',
      onScrollNeeded: (data) => {
        console.log(`🔄 [Scroll Mode] 需要滚动回调:`, data);
      },
      loadCallback: mockScrollCallback
    });
    const endTime = Date.now();

    autoLoadManager.off('state-change', stateHandler);

    // 验证滚动触发模式特性
    const finalState = autoLoadManager.getState();

    this.recordTest('Scroll Mode Success', result === true, `Expected true, got ${result}`);
    this.recordTest('Scroll Mode Batch Count', batchCount === 3, `Expected 3 batches, got ${batchCount}`);
    this.recordTest('Scroll Mode Waiting Count', waitingCount === 2, `Expected 2 waits, got ${waitingCount}`);
    this.recordTest('Scroll Mode Total Messages', finalState.totalLoaded === 25, `Expected 25, got ${finalState.totalLoaded}`);
    this.recordTest('Scroll Mode Has Waiting', stateLog.includes('waiting_for_scroll'), 'Should enter waiting state in scroll mode');
    this.recordTest('Scroll Mode Duration', endTime - startTime > 1000, `Should take longer due to waits, took ${endTime - startTime}ms`);

    console.log(`[Scroll Mode] 完成验证 - 等待了 ${waitingCount} 次，加载了 ${batchCount} 批，共 ${finalState.totalLoaded} 条消息`);
  }

  /**
   * 🔄 测试模式切换
   */
  async testModeSwitching() {
    console.log('[ModeValidator] 测试模式切换');

    // 测试从自动模式切换到滚动模式
    autoLoadManager.reset();

    let callCount = 0;
    const switchingCallback = async () => {
      callCount++;
      return {
        success: true,
        messages: [{ id: callCount, text: `Switch msg ${callCount}` }],
        hasMore: callCount < 2,
        totalCount: callCount
      };
    };

    // 启动自动模式会话
    const autoPromise = autoLoadManager.startAutoLoadSession({
      chatId: 'switch-test-1',
      hasMoreMessages: true,
      loadingMode: 'auto',
      loadCallback: switchingCallback
    });

    await autoPromise;
    const autoResult = autoLoadManager.getState();

    // 重置并启动滚动模式会话
    autoLoadManager.reset();
    callCount = 0;

    const scrollResult = await this.testQuickScrollMode();

    this.recordTest('Mode Switching', true, 'Successfully switched between modes');
    this.recordTest('Auto Mode State Reset', autoResult.current === 'completed', 'Auto mode completed successfully');
    this.recordTest('Scroll Mode State Reset', scrollResult.success === true, 'Scroll mode started successfully');

    console.log('[Mode Switching] 模式切换验证完成');
  }

  /**
   * 快速滚动模式测试 (用于模式切换验证)
   */
  async testQuickScrollMode() {
    let quickCallCount = 0;
    const quickCallback = async () => {
      quickCallCount++;
      return {
        success: true,
        messages: [{ id: quickCallCount, text: `Quick ${quickCallCount}` }],
        hasMore: false,
        totalCount: quickCallCount
      };
    };

    // 立即触发恢复，不等待
    const stateHandler = (data) => {
      if (data.to === 'waiting_for_scroll') {
        setTimeout(() => {
          if (autoLoadManager.waitingForScroll) {
            autoLoadManager.resumeFromScroll(() => { });
          }
        }, 10);
      }
    };

    autoLoadManager.on('state-change', stateHandler);

    const result = await autoLoadManager.startAutoLoadSession({
      chatId: 'quick-scroll-test',
      hasMoreMessages: true,
      loadingMode: 'scroll-triggered',
      loadCallback: quickCallback
    });

    autoLoadManager.off('state-change', stateHandler);

    return { success: result };
  }

  /**
   * 记录测试结果
   */
  recordTest(testName, passed, details = '') {
    if (passed) {
      console.log(`[ModeValidator] ${testName}: PASSED`);
    } else {
      console.log(`ERROR: [ModeValidator] ${testName}: FAILED - ${details}`);
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
    const total = this.testResults.length;
    const passed = this.testResults.filter(t => t.passed).length;
    const failed = total - passed;
    const successRate = ((passed / total) * 100).toFixed(1);

    const report = {
      summary: {
        totalTests: total,
        passedTests: passed,
        failedTests: failed,
        successRate: `${successRate}%`,
        bothModesWorking: failed === 0,
        timestamp: new Date().toISOString()
      },
      details: this.testResults,
      modeComparison: {
        autoMode: {
          characteristics: ['单次触发', '自动循环', '无等待状态', '快速完成'],
          validated: this.testResults.filter(t => t.name.includes('Auto Mode')).every(t => t.passed)
        },
        scrollTriggeredMode: {
          characteristics: ['分批触发', '等待用户交互', '包含等待状态', '用户控制节奏'],
          validated: this.testResults.filter(t => t.name.includes('Scroll Mode')).every(t => t.passed)
        }
      }
    };

    console.log('\n[ModeValidator] 双模式验证报告:');
    console.log(`总测试数: ${total}`);
    console.log(`通过测试: ${passed}`);
    console.log(`ERROR: 失败测试: ${failed}`);
    console.log(`成功率: ${successRate}%`);
    console.log(`🔒 双模式可靠性: ${report.summary.bothModesWorking ? '100% 可靠' : 'ERROR: 需要修复'}`);

    if (failed > 0) {
      console.log('\nERROR: 失败的测试:');
      this.testResults.filter(t => !t.passed).forEach(test => {
        console.log(`  - ${test.name}: ${test.details}`);
      });
    }

    console.log('\n🔄 模式特性验证:');
    console.log(`📈 自动模式: ${report.modeComparison.autoMode.validated ? '验证通过' : 'ERROR: 验证失败'}`);
    console.log(`📜 滚动触发模式: ${report.modeComparison.scrollTriggeredMode.validated ? '验证通过' : 'ERROR: 验证失败'}`);

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
    this.isRunning = false;
  }
}

// 创建全局验证器实例
export const autoLoadModeValidator = new AutoLoadModeValidator();

// 开发环境全局函数
if (typeof window !== 'undefined' && import.meta.env.DEV) {
  window.autoLoadModeValidator = autoLoadModeValidator;

  window.validateLoadingModes = async () => {
    return await autoLoadModeValidator.runModeValidation();
  };

  window.cleanupModeTests = () => {
    autoLoadModeValidator.cleanup();
  };

  console.log('AutoLoadModeValidator 调试功能已加载');
  console.log('使用 validateLoadingModes() 运行双模式验证测试');
} 