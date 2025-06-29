/**
 * Perfect Navigation Controller Test Suite
 * 
 * Comprehensive testing for the Perfect Navigation System
 * Validates 95%+ success rate target and all edge cases
 */

import { perfectNavigationController } from './PerfectNavigationController.js'

export class PerfectNavigationTest {
  constructor() {
    this.testResults = []
    this.testId = 0
  }

  /**
   * Run comprehensive Perfect Navigation tests
   */
  async runFullTest() {
    console.log('🧪 [PerfectNavigationTest] Starting comprehensive test suite...')

    const testScenarios = [
      {
        name: 'Recent Message Jump',
        type: 'recent',
        description: 'Test jumping to recently loaded messages',
        expectedSuccessRate: 99
      },
      {
        name: 'Historical Message Jump',
        type: 'historical',
        description: 'Test jumping to old messages requiring loading',
        expectedSuccessRate: 95
      },
      {
        name: 'Cross-Chat Jump',
        type: 'cross_chat',
        description: 'Test jumping across different chats',
        expectedSuccessRate: 98
      },
      {
        name: 'Search Context Jump',
        type: 'search_context',
        description: 'Test jumping from search results with highlighting',
        expectedSuccessRate: 97
      },
      {
        name: 'Concurrent Navigation',
        type: 'concurrent',
        description: 'Test multiple simultaneous navigation requests',
        expectedSuccessRate: 90
      },
      {
        name: 'Edge Case Scenarios',
        type: 'edge_cases',
        description: 'Test error conditions and fallbacks',
        expectedSuccessRate: 85
      }
    ]

    const results = {
      totalTests: 0,
      passedTests: 0,
      failedTests: 0,
      scenarios: []
    }

    for (const scenario of testScenarios) {
      console.log(`\n[Test] Starting: ${scenario.name}`)
      const scenarioResult = await this.runScenarioTest(scenario)
      results.scenarios.push(scenarioResult)
      results.totalTests += scenarioResult.attempts
      results.passedTests += scenarioResult.successes
      results.failedTests += scenarioResult.failures
    }

    // Generate final report
    const finalReport = this.generateFinalReport(results)
    console.log('\n[PerfectNavigationTest] Test Suite Complete!')
    console.table(finalReport.scenarioSummary)
    console.log('\nOverall Results:', finalReport.overall)

    return finalReport
  }

  /**
   * 🧪 Run individual scenario test
   */
  async runScenarioTest(scenario) {
    const attempts = scenario.type === 'concurrent' ? 10 : 5
    const results = {
      name: scenario.name,
      type: scenario.type,
      attempts,
      successes: 0,
      failures: 0,
      averageTime: 0,
      errors: [],
      details: []
    }

    const times = []

    for (let i = 0; i < attempts; i++) {
      const testCase = this.generateTestCase(scenario.type)
      const startTime = Date.now()

      try {
        let navigationResult

        if (scenario.type === 'concurrent') {
          // Test concurrent navigation
          navigationResult = await this.testConcurrentNavigation(testCase)
        } else {
          // Test normal navigation
          navigationResult = await perfectNavigationController.navigateToMessage(testCase)
        }

        const duration = Date.now() - startTime
        times.push(duration)

        if (navigationResult.success) {
          results.successes++
          console.log(`  Test ${i + 1}: SUCCESS (${duration}ms)`)
        } else {
          results.failures++
          results.errors.push(navigationResult.error || 'Unknown error')
          console.log(`  ERROR: Test ${i + 1}: FAILED (${duration}ms) - ${navigationResult.error}`)
        }

        results.details.push({
          testIndex: i + 1,
          success: navigationResult.success,
          duration,
          error: navigationResult.error,
          stages: navigationResult.stages
        })

      } catch (error) {
        const duration = Date.now() - startTime
        times.push(duration)
        results.failures++
        results.errors.push(error.message)
        console.log(`  💥 Test ${i + 1}: EXCEPTION (${duration}ms) - ${error.message}`)

        results.details.push({
          testIndex: i + 1,
          success: false,
          duration,
          error: error.message,
          exception: true
        })
      }

      // Small delay between tests
      await new Promise(resolve => setTimeout(resolve, 100))
    }

    results.averageTime = times.reduce((sum, time) => sum + time, 0) / times.length
    results.successRate = (results.successes / attempts * 100).toFixed(1)

    console.log(`Scenario "${scenario.name}": ${results.successRate}% success rate (${results.averageTime.toFixed(0)}ms avg)`)

    return results
  }

  /**
   * 🔀 Test concurrent navigation requests
   */
  async testConcurrentNavigation(baseTestCase) {
    const concurrentRequests = 3
    const promises = []

    for (let i = 0; i < concurrentRequests; i++) {
      const testCase = {
        ...baseTestCase,
        messageId: `${baseTestCase.messageId}_${i}`,
        source: `concurrent_test_${i}`
      }
      promises.push(perfectNavigationController.navigateToMessage(testCase))
    }

    const results = await Promise.allSettled(promises)
    const successes = results.filter(r => r.status === 'fulfilled' && r.value.success).length

    return {
      success: successes >= concurrentRequests - 1, // Allow 1 failure
      concurrentRequests,
      successes,
      error: successes < concurrentRequests - 1 ? 'Too many concurrent failures' : null
    }
  }

  /**
   * 🎲 Generate test case for different scenarios
   */
  generateTestCase(type) {
    const baseCase = {
      messageId: `test_msg_${Date.now()}_${Math.random().toString(36).substr(2, 8)}`,
      chatId: this.getCurrentChatId() || '1',
      searchQuery: 'test query',
      scrollBehavior: 'smooth',
      highlightDuration: 1000, // Shorter for tests
      source: `test_${type}`
    }

    switch (type) {
      case 'recent':
        // Use an existing message if available
        const existingMessage = this.getRandomExistingMessage()
        if (existingMessage) {
          baseCase.messageId = existingMessage.id
          baseCase.chatId = existingMessage.chatId
        }
        break

      case 'historical':
        // Simulate old message
        baseCase.messageId = `hist_${Date.now() - 86400000}_${Math.random().toString(36).substr(2, 5)}`
        break

      case 'cross_chat':
        // Different chat
        baseCase.chatId = String(parseInt(baseCase.chatId) + 1)
        break

      case 'search_context':
        // With search highlighting
        baseCase.searchQuery = 'important message'
        baseCase.pulseAnimation = true
        baseCase.showIndicator = true
        break

      case 'edge_cases':
        // Invalid data
        if (Math.random() > 0.5) {
          baseCase.messageId = 'invalid_message_id_12345'
        } else {
          baseCase.chatId = 'nonexistent_chat_999'
        }
        break
    }

    return baseCase
  }

  /**
   * 🎲 Get a random existing message for testing
   */
  getRandomExistingMessage() {
    const messageElements = document.querySelectorAll('[data-message-id]')
    if (messageElements.length === 0) return null

    const randomElement = messageElements[Math.floor(Math.random() * messageElements.length)]
    return {
      id: randomElement.getAttribute('data-message-id'),
      chatId: this.getCurrentChatId()
    }
  }

  /**
   * Get current chat ID
   */
  getCurrentChatId() {
    const pathMatch = window.location.pathname.match(/\/chat\/(\d+)/)
    return pathMatch ? pathMatch[1] : null
  }

  /**
   * Generate comprehensive test report
   */
  generateFinalReport(results) {
    const overallSuccessRate = (results.passedTests / results.totalTests * 100).toFixed(1)

    const scenarioSummary = results.scenarios.map(scenario => ({
      Scenario: scenario.name,
      'Success Rate': `${scenario.successRate}%`,
      'Avg Time': `${scenario.averageTime.toFixed(0)}ms`,
      'Tests': `${scenario.successes}/${scenario.attempts}`,
      'Status': parseFloat(scenario.successRate) >= 90 ? 'PASS' :
        parseFloat(scenario.successRate) >= 75 ? 'WARNING: WARN' : 'ERROR: FAIL'
    }))

    const overall = {
      'Total Tests': results.totalTests,
      'Passed': results.passedTests,
      'Failed': results.failedTests,
      'Success Rate': `${overallSuccessRate}%`,
      'Target Met': parseFloat(overallSuccessRate) >= 95 ? 'YES' : 'ERROR: NO',
      'Grade': this.calculateGrade(overallSuccessRate)
    }

    // Performance analytics
    const analytics = perfectNavigationController.getAnalytics()

    return {
      overall,
      scenarioSummary,
      analytics,
      recommendations: this.generateRecommendations(results),
      timestamp: new Date().toISOString()
    }
  }

  /**
   * Calculate performance grade
   */
  calculateGrade(successRate) {
    const rate = parseFloat(successRate)
    if (rate >= 98) return 'A+ (Exceptional)'
    if (rate >= 95) return 'A (Excellent)'
    if (rate >= 90) return 'B+ (Good)'
    if (rate >= 85) return 'B (Satisfactory)'
    if (rate >= 75) return 'C (Needs Improvement)'
    return 'F (Critical Issues)'
  }

  /**
   * Generate improvement recommendations
   */
  generateRecommendations(results) {
    const recommendations = []

    results.scenarios.forEach(scenario => {
      const successRate = parseFloat(scenario.successRate)

      if (successRate < 90) {
        recommendations.push({
          priority: successRate < 75 ? 'HIGH' : 'MEDIUM',
          scenario: scenario.name,
          issue: `Success rate ${scenario.successRate}% below target`,
          suggestion: this.getScenarioSuggestion(scenario.type),
          errors: scenario.errors.slice(0, 3) // Top 3 errors
        })
      }
    })

    return recommendations
  }

  /**
   * Get scenario-specific suggestions
   */
  getScenarioSuggestion(type) {
    const suggestions = {
      recent: 'Improve DOM element detection and scroll container registration',
      historical: 'Enhance message loading strategies and add more fallback mechanisms',
      cross_chat: 'Optimize chat switching and route stabilization logic',
      search_context: 'Refine search term highlighting and scroll positioning accuracy',
      concurrent: 'Implement better queuing and prevent race conditions',
      edge_cases: 'Add more robust error handling and validation'
    }

    return suggestions[type] || 'Review and optimize the specific scenario logic'
  }

  /**
   * Quick validation test
   */
  async quickTest() {
    console.log('[PerfectNavigationTest] Running quick validation test...')

    const existingMessage = this.getRandomExistingMessage()
    if (!existingMessage) {
      console.warn('WARNING: No existing messages found for quick test')
      return { success: false, error: 'No messages available' }
    }

    const testCase = {
      messageId: existingMessage.id,
      chatId: existingMessage.chatId,
      source: 'quick_test',
      highlightDuration: 2000
    }

    const startTime = Date.now()
    const result = await perfectNavigationController.navigateToMessage(testCase)
    const duration = Date.now() - startTime

    console.log(`Quick test result: ${result.success ? 'PASS' : 'ERROR: FAIL'} (${duration}ms)`)

    if (result.success) {
      console.log('Perfect Navigation Controller is working correctly!')
    } else {
      console.warn('WARNING: Perfect Navigation Controller has issues:', result.error)
    }

    return { ...result, duration }
  }

  /**
   * 📈 Performance benchmark
   */
  async benchmarkTest(iterations = 20) {
    console.log(`🏃 [PerfectNavigationTest] Running performance benchmark (${iterations} iterations)...`)

    const times = []
    let successes = 0

    for (let i = 0; i < iterations; i++) {
      const testCase = this.generateTestCase('recent')
      const startTime = Date.now()

      try {
        const result = await perfectNavigationController.navigateToMessage(testCase)
        const duration = Date.now() - startTime
        times.push(duration)

        if (result.success) successes++

      } catch (error) {
        times.push(Date.now() - startTime)
      }

      // Small delay
      await new Promise(resolve => setTimeout(resolve, 50))
    }

    const stats = {
      iterations,
      successes,
      successRate: (successes / iterations * 100).toFixed(1) + '%',
      averageTime: (times.reduce((sum, time) => sum + time, 0) / times.length).toFixed(0) + 'ms',
      minTime: Math.min(...times) + 'ms',
      maxTime: Math.max(...times) + 'ms',
      medianTime: times.sort()[Math.floor(times.length / 2)] + 'ms'
    }

    console.log('Performance Benchmark Results:')
    console.table(stats)

    return stats
  }
}

// Global instance
export const perfectNavigationTest = new PerfectNavigationTest()

// Console commands for easy testing
window.testPerfectNavigation = () => perfectNavigationTest.quickTest()
window.runPerfectNavigationSuite = () => perfectNavigationTest.runFullTest()
window.benchmarkPerfectNavigation = (iterations) => perfectNavigationTest.benchmarkTest(iterations)

// Analytics access
window.getPerfectNavigationAnalytics = () => perfectNavigationController.getAnalytics()

console.log('🧪 Perfect Navigation Test Suite loaded!')
console.log('Available commands:')
console.log('  - window.testPerfectNavigation() - Quick test')
console.log('  - window.runPerfectNavigationSuite() - Full test suite')
console.log('  - window.benchmarkPerfectNavigation(20) - Performance benchmark')
console.log('  - window.getPerfectNavigationAnalytics() - View analytics')

export default perfectNavigationTest 