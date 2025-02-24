/**
 * Historical Message Jump Test
 * 
 * Tests the accuracy of message navigation, especially for historical messages
 * that require loading before jumping to their position.
 */

import { messageNavigationManager } from './messageNavigationManager.js'

export class HistoricalMessageJumpTest {
  constructor() {
    this.testResults = []
    this.currentTestId = 0
  }

  /**
   * Run comprehensive jump test
   */
  async runFullTest() {
    console.log('🧪 [HistoricalJumpTest] Starting comprehensive message jump test...')

    const testScenarios = [
      {
        name: 'Recent Message Jump',
        type: 'recent',
        description: 'Jump to a message visible in current DOM'
      },
      {
        name: 'Historical Message Jump',
        type: 'historical',
        description: 'Jump to an old message requiring loading'
      },
      {
        name: 'Cross-Chat Message Jump',
        type: 'cross_chat',
        description: 'Jump to message in different chat'
      },
      {
        name: 'Search Result Jump',
        type: 'search_result',
        description: 'Jump from search results to message'
      }
    ]

    for (const scenario of testScenarios) {
      await this.runTestScenario(scenario)
    }

    return this.generateTestReport()
  }

  /**
   * Test message jump accuracy for different scenarios
   */
  async runTestScenario(scenario) {
    const testId = ++this.currentTestId
    console.log(`🎯 [Test ${testId}] Running: ${scenario.name}`)

    const startTime = Date.now()
    let testResult = {
      id: testId,
      name: scenario.name,
      type: scenario.type,
      startTime,
      success: false,
      error: null,
      duration: 0,
      steps: {}
    }

    try {
      // Generate test message data
      const testMessage = this.generateTestMessage(scenario.type)

      // Perform navigation test
      const navigationResult = await this.testMessageNavigation(testMessage, scenario)

      testResult.success = navigationResult.success
      testResult.steps = navigationResult.steps
      testResult.messageFound = navigationResult.messageFound
      testResult.scrollSuccessful = navigationResult.scrollSuccessful
      testResult.highlightApplied = navigationResult.highlightApplied

    } catch (error) {
      testResult.error = error.message
      console.error(`❌ [Test ${testId}] Failed:`, error)
    } finally {
      testResult.duration = Date.now() - startTime
      this.testResults.push(testResult)

      console.log(`${testResult.success ? '✅' : '❌'} [Test ${testId}] ${scenario.name}: ${testResult.duration}ms`)
    }
  }

  /**
   * Test message navigation with accuracy verification
   */
  async testMessageNavigation(testMessage, scenario) {
    const verificationSteps = {
      messageLoading: false,
      scrollPositioning: false,
      highlighting: false,
      domPresence: false
    }

    // Step 1: Check initial message presence
    const initialElement = document.querySelector(`[data-message-id="${testMessage.messageId}"]`)
    const requiresLoading = !initialElement

    console.log(`📍 [Navigation] Message ${testMessage.messageId} requires loading: ${requiresLoading}`)

    // Step 2: Execute navigation
    const navigationParams = {
      messageId: testMessage.messageId,
      chatId: testMessage.chatId,
      searchQuery: testMessage.searchQuery || null,
      source: 'test',
      highlightDuration: 2000
    }

    const navigationResult = await messageNavigationManager.navigateToMessage(navigationParams)

    // Step 3: Verify message loading (if required)
    if (requiresLoading) {
      await new Promise(resolve => setTimeout(resolve, 1000)) // Wait for loading
      const loadedElement = document.querySelector(`[data-message-id="${testMessage.messageId}"]`)
      verificationSteps.messageLoading = !!loadedElement

      if (loadedElement) {
        console.log('📦 [Navigation] Message successfully loaded into DOM')
      } else {
        console.warn('⚠️ [Navigation] Message failed to load')
      }
    } else {
      verificationSteps.messageLoading = true // Already present
    }

    // Step 4: Verify final DOM presence
    await new Promise(resolve => setTimeout(resolve, 500))
    const finalElement = document.querySelector(`[data-message-id="${testMessage.messageId}"]`)
    verificationSteps.domPresence = !!finalElement

    // Step 5: Verify scroll positioning
    if (finalElement) {
      const rect = finalElement.getBoundingClientRect()
      const isInViewport = rect.top >= 0 && rect.bottom <= window.innerHeight
      const isNearCenter = Math.abs(rect.top - window.innerHeight / 2) < 200

      verificationSteps.scrollPositioning = isInViewport && isNearCenter

      console.log(`📏 [Navigation] Scroll positioning - In viewport: ${isInViewport}, Near center: ${isNearCenter}`)
    }

    // Step 6: Verify highlighting
    if (finalElement) {
      const hasHighlight = finalElement.classList.contains('message-navigation-highlight')
      const hasIndicator = !!finalElement.querySelector('.navigation-indicator')

      verificationSteps.highlighting = hasHighlight || hasIndicator

      console.log(`✨ [Navigation] Highlighting - CSS class: ${hasHighlight}, Indicator: ${hasIndicator}`)
    }

    const overallSuccess = Object.values(verificationSteps).every(step => step === true)

    return {
      success: overallSuccess,
      navigationResult,
      steps: verificationSteps,
      messageFound: !!finalElement,
      scrollSuccessful: verificationSteps.scrollPositioning,
      highlightApplied: verificationSteps.highlighting
    }
  }

  /**
   * Generate test message data for different scenarios
   */
  generateTestMessage(type) {
    const baseMessage = {
      messageId: `test_msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      chatId: this.getCurrentChatId() || 1,
      content: 'Test message for navigation accuracy',
      sender: 'Test User',
      created_at: new Date().toISOString()
    }

    switch (type) {
      case 'recent':
        // Use a message that should be in current DOM
        const recentElements = document.querySelectorAll('[data-message-id]')
        if (recentElements.length > 0) {
          const randomElement = recentElements[Math.floor(Math.random() * recentElements.length)]
          baseMessage.messageId = randomElement.getAttribute('data-message-id')
        }
        break

      case 'historical':
        // Simulate old message ID that likely isn't loaded
        baseMessage.messageId = `hist_${Date.now() - 86400000}_${Math.random().toString(36).substr(2, 5)}`
        baseMessage.created_at = new Date(Date.now() - 86400000).toISOString() // 1 day ago
        break

      case 'cross_chat':
        // Different chat ID
        baseMessage.chatId = (parseInt(baseMessage.chatId) || 1) + 1
        break

      case 'search_result':
        // Include search context
        baseMessage.searchQuery = 'test navigation'
        break
    }

    return baseMessage
  }

  /**
   * Get current chat ID from URL or DOM
   */
  getCurrentChatId() {
    const pathMatch = window.location.pathname.match(/\/chat\/(\d+)/)
    return pathMatch ? pathMatch[1] : null
  }

  /**
   * Generate comprehensive test report
   */
  generateTestReport() {
    const totalTests = this.testResults.length
    const successfulTests = this.testResults.filter(result => result.success).length
    const failedTests = totalTests - successfulTests
    const averageDuration = this.testResults.reduce((sum, result) => sum + result.duration, 0) / totalTests

    const report = {
      summary: {
        totalTests,
        successfulTests,
        failedTests,
        successRate: ((successfulTests / totalTests) * 100).toFixed(1) + '%',
        averageDuration: Math.round(averageDuration) + 'ms'
      },
      details: this.testResults,
      recommendations: this.generateRecommendations()
    }

    console.log('📊 [HistoricalJumpTest] Test Results Summary:')
    console.log(`  Total Tests: ${totalTests}`)
    console.log(`  Successful: ${successfulTests}`)
    console.log(`  Failed: ${failedTests}`)
    console.log(`  Success Rate: ${report.summary.successRate}`)
    console.log(`  Average Duration: ${report.summary.averageDuration}`)

    return report
  }

  /**
   * Generate improvement recommendations based on test results
   */
  generateRecommendations() {
    const recommendations = []

    const failedTests = this.testResults.filter(result => !result.success)

    // Analyze failure patterns
    const loadingFailures = failedTests.filter(result =>
      result.steps && !result.steps.messageLoading
    ).length

    const scrollFailures = failedTests.filter(result =>
      result.steps && !result.steps.scrollPositioning
    ).length

    const highlightFailures = failedTests.filter(result =>
      result.steps && !result.steps.highlighting
    ).length

    if (loadingFailures > 0) {
      recommendations.push({
        priority: 'HIGH',
        issue: 'Message Loading Accuracy',
        description: `${loadingFailures} tests failed due to message loading issues`,
        solution: 'Improve message context loading strategies and add more robust retry mechanisms'
      })
    }

    if (scrollFailures > 0) {
      recommendations.push({
        priority: 'MEDIUM',
        issue: 'Scroll Positioning Accuracy',
        description: `${scrollFailures} tests failed due to scroll positioning issues`,
        solution: 'Refine scroll calculation algorithm and add viewport detection'
      })
    }

    if (highlightFailures > 0) {
      recommendations.push({
        priority: 'LOW',
        issue: 'Message Highlighting',
        description: `${highlightFailures} tests failed due to highlighting issues`,
        solution: 'Ensure highlight application timing and CSS class management'
      })
    }

    return recommendations
  }

  /**
   * Run a quick validation test
   */
  async quickValidation() {
    console.log('⚡ [HistoricalJumpTest] Running quick validation...')

    // Test with a known message if available
    const existingMessage = document.querySelector('[data-message-id]')

    if (existingMessage) {
      const messageId = existingMessage.getAttribute('data-message-id')
      const chatId = this.getCurrentChatId()

      const result = await messageNavigationManager.jumpToMessage({
        messageId,
        chatId,
        source: 'quick_test'
      })

      console.log('⚡ Quick validation result:', result.success ? '✅ PASS' : '❌ FAIL')
      return result
    } else {
      console.warn('⚡ No messages available for quick validation')
      return { success: false, error: 'No messages in DOM' }
    }
  }
}

// Export for global access
window.HistoricalMessageJumpTest = HistoricalMessageJumpTest

// Create global test runner instance
export const historicalJumpTest = new HistoricalMessageJumpTest()

// Quick test function for console
window.testMessageJump = () => historicalJumpTest.quickValidation()
window.runFullMessageJumpTest = () => historicalJumpTest.runFullTest()

export default historicalJumpTest 