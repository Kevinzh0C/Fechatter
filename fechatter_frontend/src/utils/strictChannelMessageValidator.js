/**
 * Strict Channel Message Validator
 * Ensures messages are ONLY displayed in their correct channel
 * This is FUNDAMENTAL to the software - messages MUST match their channel
 */

class StrictChannelMessageValidator {
  constructor() {
    this.validationEnabled = true;
    this.violations = [];
    this.maxViolations = 100;
  }

  /**
   * Validate that a message belongs to the specified channel
   * @param {Object} message - The message to validate
   * @param {number} expectedChatId - The expected chat ID
   * @returns {boolean} - True if valid, throws error if not
   */
  validateMessage(message, expectedChatId) {
    if (!this.validationEnabled) return true;

    // Convert to numbers for comparison
    const messageChatId = parseInt(message.chat_id, 10);
    const expected = parseInt(expectedChatId, 10);

    if (messageChatId !== expected) {
      const violation = {
        timestamp: new Date().toISOString(),
        messageId: message.id,
        messageChatId: messageChatId,
        expectedChatId: expected,
        messageContent: message.content?.substring(0, 50) + '...',
        stack: new Error().stack
      };

      this.violations.push(violation);

      // Keep only recent violations
      if (this.violations.length > this.maxViolations) {
        this.violations.shift();
      }

      // This is CRITICAL - throw error to prevent wrong display
      throw new Error(
        `CRITICAL: Message ${message.id} belongs to chat ${messageChatId} but trying to display in chat ${expected}. ` +
        `This is a fundamental violation. Message content: "${violation.messageContent}"`
      );
    }

    return true;
  }

  /**
   * Validate an entire message array
   * @param {Array} messages - Array of messages
   * @param {number} expectedChatId - The expected chat ID
   * @returns {Array} - Only messages that belong to the channel
   */
  validateMessageArray(messages, expectedChatId) {
    if (!Array.isArray(messages)) return [];

    const validMessages = [];
    const invalidMessages = [];

    messages.forEach(message => {
      try {
        if (this.validateMessage(message, expectedChatId)) {
          validMessages.push(message);
        }
      } catch (error) {
        console.error('❌ [VALIDATOR]', error.message);
        invalidMessages.push({
          message,
          error: error.message
        });
      }
    });

    if (invalidMessages.length > 0) {
      console.error(`❌ [VALIDATOR] Found ${invalidMessages.length} messages in wrong channel!`);
      console.table(invalidMessages.map(item => ({
        messageId: item.message.id,
        messageChatId: item.message.chat_id,
        expectedChatId,
        content: item.message.content?.substring(0, 30) + '...'
      })));
    }

    return validMessages;
  }

  /**
   * Get validation report
   */
  getReport() {
    return {
      enabled: this.validationEnabled,
      violationCount: this.violations.length,
      recentViolations: this.violations.slice(-10),
      summary: this.generateSummary()
    };
  }

  /**
   * Generate summary of violations
   */
  generateSummary() {
    if (this.violations.length === 0) {
      return 'No violations detected';
    }

    // Group by chat pairs
    const pairMap = new Map();

    this.violations.forEach(v => {
      const key = `${v.messageChatId}->${v.expectedChatId}`;
      if (!pairMap.has(key)) {
        pairMap.set(key, 0);
      }
      pairMap.set(key, pairMap.get(key) + 1);
    });

    const pairs = Array.from(pairMap.entries())
      .sort((a, b) => b[1] - a[1])
      .map(([pair, count]) => `${pair}: ${count} times`);

    return {
      totalViolations: this.violations.length,
      uniquePairs: pairMap.size,
      topViolations: pairs.slice(0, 5)
    };
  }

  /**
   * Clear violations history
   */
  clearViolations() {
    this.violations = [];
    console.log('✅ [VALIDATOR] Violations history cleared');
  }

  /**
   * Enable/disable validation (for debugging)
   */
  setEnabled(enabled) {
    this.validationEnabled = enabled;
    console.log(`[VALIDATOR] Validation ${enabled ? 'enabled' : 'disabled'}`);
  }
}

// Create singleton instance
const validator = new StrictChannelMessageValidator();

// Export for use in stores and components
export default validator;

// Also expose globally for debugging
if (typeof window !== 'undefined') {
  window.channelMessageValidator = validator;

  console.log('🛡️ Strict Channel Message Validator loaded');
  console.log('Usage:');
  console.log('  - window.channelMessageValidator.getReport() - View validation report');
  console.log('  - window.channelMessageValidator.clearViolations() - Clear violation history');
  console.log('  - window.channelMessageValidator.setEnabled(false) - Disable validation temporarily');
} 