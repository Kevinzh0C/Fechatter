import api from './api';

// Bot service for translation and other bot features
export class BotService {
  constructor() {
    this.quotaKey = 'bot_translation_quota';
    this.quotaResetKey = 'bot_quota_reset_time';
    this.dailyLimit = 20;
  }

  // Get current quota info
  getQuotaInfo() {
    const today = new Date().toDateString();
    const resetTime = localStorage.getItem(this.quotaResetKey);

    // Reset quota if it's a new day
    if (resetTime !== today) {
      localStorage.setItem(this.quotaResetKey, today);
      localStorage.setItem(this.quotaKey, '0');
    }

    const used = parseInt(localStorage.getItem(this.quotaKey) || '0');
    const remaining = Math.max(0, this.dailyLimit - used);

    return {
      used,
      remaining,
      limit: this.dailyLimit,
      resetTime: today
    };
  }

  // Increment quota usage
  incrementQuota() {
    const quota = this.getQuotaInfo();
    localStorage.setItem(this.quotaKey, String(quota.used + 1));
  }

  // Check if user can make request
  canMakeRequest() {
    const quota = this.getQuotaInfo();
    return quota.remaining > 0;
  }

  // Translate message with automatic fallback to mock
  async translateMessage(messageId, targetLanguage = 'en') {
    // FIX: Ensure message_id is always a string for backend API compatibility
    const messageIdString = String(messageId);

    // Check quota
    if (!this.canMakeRequest()) {
      const quota = this.getQuotaInfo();
      throw new Error(`Daily translation limit exceeded. You have used ${quota.used}/${quota.limit} translations today. Resets tomorrow.`);
    }

    try {
      if (import.meta.env.DEV) {
        console.log('🔄 [BotService] Attempting translation API call...', {
          messageId: messageIdString,
          targetLanguage,
          url: '/bot/translate'
        });
      }

      // CRITICAL FIX: Get message content to send with translation request
      let messageContent = '';

      try {
        // Try to get message content from various sources
        if (typeof window !== 'undefined') {
          // Method 1: Check global message data
          if (window.currentTranslatingMessage && window.currentTranslatingMessage.id === messageIdString) {
            messageContent = window.currentTranslatingMessage.content || '';
          }

          // Method 2: Find message in DOM
          if (!messageContent) {
            const messageElement = document.querySelector(`[data-message-id="${messageIdString}"]`);
            if (messageElement) {
              const contentElement = messageElement.querySelector('.message-content, .discord-message-content, [class*="content"]');
              if (contentElement) {
                messageContent = contentElement.textContent?.trim() || '';
              }
            }
          }

          // Method 3: Check for debug/test message
          if (!messageContent && window.testMessage && window.testMessage.id === messageIdString) {
            messageContent = window.testMessage.content || '';
          }
        }

        // Fallback if no content found
        if (!messageContent) {
          messageContent = `Message content for ID ${messageIdString}`;
        }

        if (import.meta.env.DEV) {
          console.log('[BotService] Message content for translation:', {
            messageId: messageIdString,
            contentLength: messageContent.length,
            contentPreview: messageContent.substring(0, 50) + (messageContent.length > 50 ? '...' : '')
          });
        }
      } catch (error) {
        if (import.meta.env.DEV) {
          console.warn('WARNING: [BotService] Could not extract message content:', error);
        }
        messageContent = `Content for message ${messageIdString}`;
      }

      const response = await api.post('/bot/translate', {
        message_id: messageIdString,
        target_language: targetLanguage,
        text: messageContent
      });

      if (import.meta.env.DEV) {
        console.log('[BotService] Translation API call successful');
      }

      // FIXED: Use server-side quota management and update local cache
      const serverQuota = {
        used: response.data.quota_used || 0,
        remaining: response.data.quota_remaining || this.dailyLimit,
        limit: response.data.quota_limit || this.dailyLimit,
        resetTime: new Date().toDateString()
      };

      // Update local quota cache with server data
      if (response.data.quota_used !== undefined) {
        localStorage.setItem(this.quotaKey, String(response.data.quota_used));
        localStorage.setItem(this.quotaResetKey, new Date().toDateString());
      }

      return {
        success: true,
        translation: response.data.translation,
        source_language: response.data.source_language,
        target_language: response.data.target_language,
        confidence: response.data.confidence,
        quota_used: serverQuota.used,
        quota_remaining: serverQuota.remaining,
        quota_limit: serverQuota.limit,
        quota: serverQuota,
        provider: response.data.provider || 'server'
      };
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('ERROR: [BotService] Translation API error:', {
          message: error.message,
          status: error.response?.status,
          statusText: error.response?.statusText,
          data: error.response?.data,
          url: error.config?.url
        });
      }

      // ENHANCED: More comprehensive fallback detection
      const shouldFallbackToMock = (
        error.response?.status === 404 ||
        error.response?.status === 501 ||
        error.response?.status === 502 ||
        error.response?.status === 503 ||
        error.code === 'ERR_NETWORK' ||
        error.code === 'ECONNREFUSED' ||
        !error.response // Network error
      );

      if (shouldFallbackToMock) {
        if (import.meta.env.DEV) {
          console.log('🔄 [BotService] Falling back to mock translation service...', {
            reason: error.response?.status || error.code || 'unknown',
            messageId: messageIdString,
            targetLanguage
          });
        }

        try {
          const mockResult = await this.mockTranslateMessage(messageIdString, targetLanguage);

          if (import.meta.env.DEV) {
            console.log('[BotService] Mock translation successful');
          }

          return mockResult;
        } catch (mockError) {
          if (import.meta.env.DEV) {
            console.error('ERROR: [BotService] Mock translation failed:', mockError);
          }
          throw new Error(`Translation failed: ${mockError.message}`);
        }
      }

      // Handle other specific error cases without fallback
      if (error.response?.status === 429) {
        throw new Error('Rate limit exceeded. Please try again later.');
      } else if (error.response?.status === 400) {
        const message = error.response?.data?.message || 'Invalid translation request.';
        throw new Error(message);
      } else if (error.response?.status === 401) {
        throw new Error('Authentication required for translation service.');
      } else if (error.response?.status === 403) {
        throw new Error('Access denied. Please check your permissions.');
      }

      // Generic error for unhandled cases
      throw new Error(`Translation service error: ${error.message}`);
    }
  }

  // Mock translation service for development/fallback
  async mockTranslateMessage(messageId, targetLanguage = 'en') {
    if (import.meta.env.DEV) {
      console.log('🎭 [BotService] Starting mock translation...', { messageId, targetLanguage });
    }

    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 300 + Math.random() * 500));

    // ENHANCED: Try to get actual message content
    let messageContent = 'Default message content'; // Fallback

    try {
      // Try to get message from DOM or global state
      if (typeof window !== 'undefined') {
        // Method 0: Check for test message first (for debug panel)
        if (window.testMessage && window.testMessage.id === messageId) {
          messageContent = window.testMessage.content || messageContent;
          if (import.meta.env.DEV) {
            console.log('🧪 [BotService] Using test message content:', messageContent);
          }
        }

        // Method 1: Try to find message in chat store
        if (messageContent === 'Default message content' && window.useLatestMessage) {
          const latestMessage = window.useLatestMessage();
          if (latestMessage && latestMessage.id === messageId) {
            messageContent = latestMessage.content || latestMessage.text || messageContent;
          }
        }

        // Method 2: Try to get from message elements in DOM
        if (messageContent === 'Default message content') {
          const messageElements = document.querySelectorAll('[data-message-id]');
          for (const element of messageElements) {
            if (element.dataset.messageId === String(messageId)) {
              const contentElement = element.querySelector('.message-content, .discord-message-content, [class*="content"]');
              if (contentElement) {
                messageContent = contentElement.textContent?.trim() || messageContent;
                break;
              }
            }
          }
        }

        // Method 3: Try to get from any existing message data
        if (messageContent === 'Default message content') {
          const messageElement = document.querySelector(`[data-message-id="${messageId}"]`);
          if (messageElement) {
            messageContent = messageElement.textContent?.trim() || messageContent;
          }
        }
      }
    } catch (error) {
      if (import.meta.env.DEV) {
        console.warn('WARNING: [BotService] Could not retrieve actual message content:', error);
      }
    }

    // Fallback to a more generic message if still default
    if (messageContent === 'Default message content') {
      messageContent = `Message ${messageId} content`;
    }

    if (import.meta.env.DEV) {
      console.log('[BotService] Message content for translation:', {
        messageId,
        content: messageContent.substring(0, 100) + (messageContent.length > 100 ? '...' : ''),
        length: messageContent.length
      });
    }

    // Enhanced mock translations database
    const mockTranslations = {
      'zh': {
        'Hello': '你好',
        'Hello world': '你好世界',
        'Good morning': '早上好',
        'How are you?': '你好吗？',
        'Thank you': '谢谢',
        'Good night': '晚安',
        'See you later': '待会见',
        'How much does this cost?': '这个多少钱？',
        'Where is the bathroom?': '洗手间在哪里？',
        'I love you': '我爱你',
        'Test message debug': '测试消息调试',
        'Message': '消息',
        'content': '内容'
      },
      'ja': {
        'Hello': 'こんにちは',
        'Hello world': 'こんにちは世界',
        'Good morning': 'おはようございます',
        'How are you?': 'お元気ですか？',
        'Thank you': 'ありがとうございます',
        'Good night': 'おやすみなさい',
        'See you later': 'また後で',
        'How much does this cost?': 'これはいくらですか？',
        'Where is the bathroom?': 'トイレはどこですか？',
        'I love you': '愛してます',
        'Test message debug': 'テストメッセージデバッグ',
        'Message': 'メッセージ',
        'content': 'コンテンツ'
      },
      'ko': {
        'Hello': '안녕하세요',
        'Hello world': '안녕하세요 세상',
        'Good morning': '좋은 아침',
        'How are you?': '어떻게 지내세요？',
        'Thank you': '감사합니다',
        'Good night': '잘 자요',
        'See you later': '나중에 봐요',
        'How much does this cost?': '이거 얼마예요?',
        'Where is the bathroom?': '화장실이 어디예요?',
        'I love you': '사랑해요',
        'Test message debug': '테스트 메시지 디버그',
        'Message': '메시지',
        'content': '내용'
      },
      'es': {
        'Hello': 'Hola',
        'Hello world': 'Hola mundo',
        'Good morning': 'Buenos días',
        'How are you?': '¿Cómo estás?',
        'Thank you': 'Gracias',
        'Good night': 'Buenas noches',
        'See you later': 'Hasta luego',
        'How much does this cost?': '¿Cuánto cuesta esto?',
        'Where is the bathroom?': '¿Dónde está el baño?',
        'I love you': 'Te amo',
        'Test message debug': 'Mensaje de prueba de depuración',
        'Message': 'Mensaje',
        'content': 'contenido'
      },
      'fr': {
        'Hello': 'Bonjour',
        'Hello world': 'Bonjour le monde',
        'Good morning': 'Bonjour',
        'How are you?': 'Comment allez-vous?',
        'Thank you': 'Merci',
        'Good night': 'Bonne nuit',
        'See you later': 'À plus tard',
        'How much does this cost?': 'Combien ça coûte?',
        'Where is the bathroom?': 'Où sont les toilettes?',
        'I love you': 'Je t\'aime',
        'Test message debug': 'Message de test de débogage',
        'Message': 'Message',
        'content': 'contenu'
      },
      'de': {
        'Hello': 'Hallo',
        'Hello world': 'Hallo Welt',
        'Good morning': 'Guten Morgen',
        'How are you?': 'Wie geht es dir?',
        'Thank you': 'Danke',
        'Good night': 'Gute Nacht',
        'See you later': 'Bis später',
        'How much does this cost?': 'Wie viel kostet das?',
        'Where is the bathroom?': 'Wo ist das Badezimmer?',
        'I love you': 'Ich liebe dich',
        'Test message debug': 'Test-Nachrichten-Debug',
        'Message': 'Nachricht',
        'content': 'Inhalt'
      },
      'ru': {
        'Hello': 'Привет',
        'Hello world': 'Привет мир',
        'Good morning': 'Доброе утро',
        'How are you?': 'Как дела?',
        'Thank you': 'Спасибо',
        'Good night': 'Спокойной ночи',
        'See you later': 'Увидимся позже',
        'How much does this cost?': 'Сколько это стоит?',
        'Where is the bathroom?': 'Где туалет?',
        'I love you': 'Я тебя люблю',
        'Test message debug': 'Отладка тестового сообщения',
        'Message': 'Сообщение',
        'content': 'содержание'
      }
    };

    // Get translation or generate intelligent fallback
    const targetTranslations = mockTranslations[targetLanguage] || {};
    let translation;

    // First try exact match
    if (targetTranslations[messageContent]) {
      translation = targetTranslations[messageContent];
    } else {
      // Try partial matches for common phrases
      const lowerContent = messageContent.toLowerCase();
      let foundMatch = false;

      for (const [key, value] of Object.entries(targetTranslations)) {
        if (lowerContent.includes(key.toLowerCase()) || key.toLowerCase().includes(lowerContent)) {
          translation = value;
          foundMatch = true;
          break;
        }
      }

      // If no match found, generate smart fallback
      if (!foundMatch) {
        const languageNames = {
          'zh': '中文',
          'ja': '日本語',
          'ko': '한국어',
          'es': 'Español',
          'fr': 'Français',
          'de': 'Deutsch',
          'ru': 'Русский'
        };

        const langName = languageNames[targetLanguage] || targetLanguage.toUpperCase();

        // For longer text, provide a more realistic mock translation
        if (messageContent.length > 20) {
          translation = `[${langName} 翻译] ${messageContent}`;
        } else {
          translation = `[${langName}] ${messageContent}`;
        }
      }
    }

    // Increment local quota for mock
    this.incrementQuota();
    const quota = this.getQuotaInfo();

    const result = {
      success: true,
      translation: translation,
      source_language: this.detectLanguageFallback(messageContent),
      target_language: targetLanguage,
      confidence: 0.85 + Math.random() * 0.1, // 0.85-0.95
      quota: {
        used: quota.used,
        remaining: quota.remaining,
        limit: quota.limit,
        resetTime: quota.resetTime
      },
      mock: true, // Indicate this is a mock response
      provider: 'mock-service',
      originalContent: messageContent
    };

    if (import.meta.env.DEV) {
      console.log('[BotService] Mock translation completed:', {
        original: messageContent.substring(0, 50) + (messageContent.length > 50 ? '...' : ''),
        translated: translation.substring(0, 50) + (translation.length > 50 ? '...' : ''),
        confidence: result.confidence,
        quota: result.quota
      });
    }

    return result;
  }

  // Get supported languages with fallback
  async getSupportedLanguages() {
    try {
      const response = await api.get('/bot/languages');
      return response.data.languages || [];
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Failed to fetch supported languages:', error);
        if (error.response?.status === 404) {
          console.log('🔄 Using fallback language list...');
        }
      }
      // Return enhanced languages as fallback with flags
      return [
        { code: 'en', name: 'English', flag: '🇺🇸' },
        { code: 'zh', name: '中文', flag: '🇨🇳' },
        { code: 'ja', name: '日本語', flag: '🇯🇵' },
        { code: 'ko', name: '한국어', flag: '🇰🇷' },
        { code: 'es', name: 'Español', flag: '🇪🇸' },
        { code: 'fr', name: 'Français', flag: '🇫🇷' },
        { code: 'de', name: 'Deutsch', flag: '🇩🇪' },
        { code: 'ru', name: 'Русский', flag: '🇷🇺' }
      ];
    }
  }

  // Detect language of text with fallback
  async detectLanguage(text) {
    try {
      const response = await api.post('/bot/detect-language', { text });
      return response.data.language;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Language detection error:', error);
        if (error.response?.status === 404) {
          console.log('🔄 Using fallback language detection...');
        }
      }

      // Fallback language detection using simple patterns
      return this.detectLanguageFallback(text);
    }
  }

  // Simple pattern-based language detection fallback
  detectLanguageFallback(text) {
    if (!text || typeof text !== 'string') return 'en';

    const patterns = {
      'zh': /[\u4e00-\u9fff]/,
      'ja': /[\u3040-\u309f\u30a0-\u30ff]/,
      'ko': /[\uac00-\ud7af]/,
      'ru': /[\u0400-\u04ff]/,
      'ar': /[\u0600-\u06ff]/,
      'en': /^[a-zA-Z\s.,!?'"]+$/
    };

    // Check each pattern
    for (const [lang, pattern] of Object.entries(patterns)) {
      if (pattern.test(text)) {
        return lang;
      }
    }

    // Default to English
    return 'en';
  }
}

// Export singleton instance
export const botService = new BotService();

// Export to window for debugging
if (typeof window !== 'undefined') {
  window.botService = botService;
} 