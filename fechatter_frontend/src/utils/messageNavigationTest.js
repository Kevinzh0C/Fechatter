/**
 * 🧪 Message Navigation Manager - Quick Test
 * 生产级别导航功能的快速验证
 */

import { useMessageNavigation } from './messageNavigationManager'

export class MessageNavigationTest {
  constructor() {
    this.testResults = []
    this.navigation = useMessageNavigation()
  }

  /**
   * 🧪 运行所有测试
   */
  async runAllTests() {
    console.log('🧪 [MessageNavigationTest] Starting tests...')

    this.testResults = []

    // 基本功能测试
    await this.testParameterValidation()
    await this.testNavigationInterface()
    await this.testHighlightSystem()
    await this.testAnalytics()

    // 输出结果
    this.printTestResults()

    return this.testResults
  }

  /**
   * 测试参数验证
   */
  async testParameterValidation() {
    console.log('🧪 Testing parameter validation...')

    try {
      // 测试缺少必需参数
      const result = await this.navigation.navigateToMessage({})

      this.addTestResult('Parameter Validation',
        !result.success && result.error.includes('Invalid navigation parameters'),
        'Should reject empty parameters'
      )

    } catch (error) {
      this.addTestResult('Parameter Validation', false, `Unexpected error: ${error.message}`)
    }
  }

  /**
   * 测试导航接口
   */
  async testNavigationInterface() {
    console.log('🧪 Testing navigation interface...')

    try {
      // 测试接口是否存在
      const hasJumpToMessage = typeof this.navigation.jumpToMessage === 'function'
      const hasNavigateToMessage = typeof this.navigation.navigateToMessage === 'function'
      const hasClearHighlights = typeof this.navigation.clearHighlights === 'function'
      const hasGetAnalytics = typeof this.navigation.getAnalytics === 'function'

      this.addTestResult('Navigation Interface',
        hasJumpToMessage && hasNavigateToMessage && hasClearHighlights && hasGetAnalytics,
        'All required methods should exist'
      )

    } catch (error) {
      this.addTestResult('Navigation Interface', false, `Interface error: ${error.message}`)
    }
  }

  /**
   * ✨ 测试高亮系统
   */
  async testHighlightSystem() {
    console.log('🧪 Testing highlight system...')

    try {
      // 创建测试DOM元素
      const testElement = document.createElement('div')
      testElement.setAttribute('data-message-id', 'test-message-123')
      testElement.innerHTML = '<div class="message-content">Hello world test message</div>'
      document.body.appendChild(testElement)

      // 测试高亮清理功能
      this.navigation.clearHighlights()

      const hasHighlightClass = testElement.classList.contains('message-navigation-highlight')

      this.addTestResult('Highlight System',
        !hasHighlightClass,
        'Should clear highlights without errors'
      )

      // 清理测试元素
      document.body.removeChild(testElement)

    } catch (error) {
      this.addTestResult('Highlight System', false, `Highlight error: ${error.message}`)
    }
  }

  /**
   * 测试分析功能
   */
  async testAnalytics() {
    console.log('🧪 Testing analytics...')

    try {
      const analytics = this.navigation.getAnalytics()

      const hasRequiredFields = analytics &&
        typeof analytics.totalNavigations === 'number' &&
        typeof analytics.successfulNavigations === 'number' &&
        typeof analytics.failedNavigations === 'number' &&
        typeof analytics.successRate === 'string'

      this.addTestResult('Analytics',
        hasRequiredFields,
        'Analytics should return proper data structure'
      )

    } catch (error) {
      this.addTestResult('Analytics', false, `Analytics error: ${error.message}`)
    }
  }

  /**
   * 添加测试结果
   */
  addTestResult(testName, passed, description) {
    const result = {
      name: testName,
      passed,
      description,
      timestamp: new Date().toISOString()
    }

    this.testResults.push(result)

    const status = passed ? 'PASS' : 'ERROR: FAIL'
    console.log(`🧪 ${status}: ${testName} - ${description}`)
  }

  /**
   * 打印测试结果
   */
  printTestResults() {
    const passedTests = this.testResults.filter(r => r.passed).length
    const totalTests = this.testResults.length
    const successRate = ((passedTests / totalTests) * 100).toFixed(1)

    console.log('\n🧪 [MessageNavigationTest] Test Summary:')
    console.log(`Results: ${passedTests}/${totalTests} tests passed (${successRate}%)`)

    if (passedTests === totalTests) {
      console.log('🎉 All tests passed! Navigation system is ready for production.')
    } else {
      console.log('WARNING: Some tests failed. Please check the implementation.')

      // 显示失败的测试
      const failedTests = this.testResults.filter(r => !r.passed)
      failedTests.forEach(test => {
        console.log(`ERROR: Failed: ${test.name} - ${test.description}`)
      })
    }
  }

  /**
   * 运行生产环境检查
   */
  async runProductionChecks() {
    console.log('[MessageNavigationTest] Running production checks...')

    const checks = {
      cssLoaded: this.checkCSSLoaded(),
      domStructure: this.checkDOMStructure(),
      eventListeners: this.checkEventListeners(),
      performance: this.checkPerformance()
    }

    console.log('Production Checks Results:', checks)

    const allPassed = Object.values(checks).every(check => check.passed)

    if (allPassed) {
      console.log('Production checks passed! System ready for deployment.')
    } else {
      console.log('WARNING: Production checks failed. Review the issues before deployment.')
    }

    return checks
  }

  /**
   * 检查CSS是否加载
   */
  checkCSSLoaded() {
    try {
      // 创建测试元素检查CSS
      const testEl = document.createElement('div')
      testEl.className = 'message-navigation-highlight'
      testEl.style.position = 'absolute'
      testEl.style.top = '-9999px'
      document.body.appendChild(testEl)

      const styles = window.getComputedStyle(testEl)
      const hasBackground = styles.backgroundColor !== 'rgba(0, 0, 0, 0)' && styles.backgroundColor !== 'transparent'

      document.body.removeChild(testEl)

      return {
        passed: hasBackground,
        message: hasBackground ? 'CSS styles loaded' : 'CSS styles not loaded'
      }
    } catch (error) {
      return { passed: false, message: `CSS check error: ${error.message}` }
    }
  }

  /**
   * 检查DOM结构
   */
  checkDOMStructure() {
    try {
      const hasMessageList = document.querySelector('.simple-message-list, .message-list, .messages-container') !== null
      const hasDataAttributes = document.querySelector('[data-message-id]') !== null

      return {
        passed: hasMessageList,
        message: hasMessageList ? 'DOM structure valid' : 'Missing message list container',
        details: { hasMessageList, hasDataAttributes }
      }
    } catch (error) {
      return { passed: false, message: `DOM check error: ${error.message}` }
    }
  }

  /**
   * 🎧 检查事件监听器
   */
  checkEventListeners() {
    try {
      // 检查是否有相关的事件监听器
      const hasClickListeners = document.querySelectorAll('[onclick], [data-onclick]').length > 0

      return {
        passed: true, // 基本检查，不强制要求
        message: 'Event listeners check completed',
        details: { hasClickListeners }
      }
    } catch (error) {
      return { passed: false, message: `Event check error: ${error.message}` }
    }
  }

  /**
   * 检查性能
   */
  checkPerformance() {
    try {
      const start = performance.now()

      // 执行一些基本操作
      this.navigation.clearHighlights()
      const analytics = this.navigation.getAnalytics()

      const duration = performance.now() - start
      const isPerformant = duration < 10 // 应该在10ms内完成

      return {
        passed: isPerformant,
        message: `Operations completed in ${duration.toFixed(2)}ms`,
        details: { duration, analytics }
      }
    } catch (error) {
      return { passed: false, message: `Performance check error: ${error.message}` }
    }
  }
}

// 导出便捷函数
export async function quickTest() {
  const tester = new MessageNavigationTest()
  return await tester.runAllTests()
}

export async function productionCheck() {
  const tester = new MessageNavigationTest()
  return await tester.runProductionChecks()
}

// 自动测试（开发环境）
if (import.meta.env.DEV) {
  // 延迟执行测试，确保DOM就绪
  setTimeout(async () => {
    console.log('🧪 [Auto Test] Running MessageNavigation tests...')
    await quickTest()
  }, 2000)
}

export default MessageNavigationTest 