#!/usr/bin/env node

/**
 * Mock Bot API Server
 * Provides local bot services for development
 */

const express = require('express');
const cors = require('cors');
const app = express();
const PORT = 3001;

// Middleware
app.use(cors());
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

// Request logging
app.use((req, res, next) => {
  console.log(`🤖 [Bot Mock API] ${req.method} ${req.path}`, req.body || '');
  next();
});

// ================================
// 🎯 Bot API Endpoints
// ================================

// GET /api/bot/languages - Get supported languages
app.get('/api/bot/languages', (req, res) => {
  const languages = [
    { code: 'en', name: 'English', flag: '🇺🇸', native: 'English' },
    { code: 'zh', name: 'Chinese', flag: '🇨🇳', native: '中文' },
    { code: 'ja', name: 'Japanese', flag: '🇯🇵', native: '日本語' },
    { code: 'ko', name: 'Korean', flag: '🇰🇷', native: '한국어' },
    { code: 'es', name: 'Spanish', flag: '🇪🇸', native: 'Español' },
    { code: 'fr', name: 'French', flag: '🇫🇷', native: 'Français' },
    { code: 'de', name: 'German', flag: '🇩🇪', native: 'Deutsch' },
    { code: 'ru', name: 'Russian', flag: '🇷🇺', native: 'Русский' },
    { code: 'pt', name: 'Portuguese', flag: '🇵🇹', native: 'Português' },
    { code: 'it', name: 'Italian', flag: '🇮🇹', native: 'Italiano' }
  ];

  res.json({
    success: true,
    languages,
    total: languages.length,
    provider: 'mock-bot-api'
  });
});

// POST /api/bot/translate - Translate message
app.post('/api/bot/translate', (req, res) => {
  const { message_id, target_language, text } = req.body;

  if (!message_id || !target_language) {
    return res.status(400).json({
      error: 'Missing required fields: message_id, target_language'
    });
  }

  // Mock translation database
  const translations = {
    'zh': {
      'Hello': '你好',
      'Hello world': '你好世界',
      'Good morning': '早上好',
      'How are you?': '你好吗？',
      'Thank you': '谢谢',
      'Welcome': '欢迎',
      'Test message debug': '测试消息调试',
      'Message': '消息'
    },
    'ja': {
      'Hello': 'こんにちは',
      'Hello world': 'こんにちは世界',
      'Good morning': 'おはようございます',
      'How are you?': 'お元気ですか？',
      'Thank you': 'ありがとうございます',
      'Welcome': 'いらっしゃいませ',
      'Test message debug': 'テストメッセージデバッグ',
      'Message': 'メッセージ'
    },
    'ko': {
      'Hello': '안녕하세요',
      'Hello world': '안녕하세요 세상',
      'Good morning': '좋은 아침',
      'How are you?': '어떻게 지내세요？',
      'Thank you': '감사합니다',
      'Welcome': '환영합니다',
      'Test message debug': '테스트 메시지 디버그',
      'Message': '메시지'
    },
    'es': {
      'Hello': 'Hola',
      'Hello world': 'Hola mundo',
      'Good morning': 'Buenos días',
      'How are you?': '¿Cómo estás?',
      'Thank you': 'Gracias',
      'Welcome': 'Bienvenido',
      'Test message debug': 'Mensaje de prueba de depuración',
      'Message': 'Mensaje'
    }
  };

  // Get message content (mock - in real app would fetch from DB)
  const messageContent = text || 'Hello world';

  // Get translation
  const targetLangTranslations = translations[target_language] || {};
  let translation = targetLangTranslations[messageContent];

  if (!translation) {
    // Generate fallback translation
    translation = `[${target_language.toUpperCase()}] ${messageContent}`;
  }

  // Simulate processing delay
  setTimeout(() => {
    res.json({
      success: true,
      translation,
      source_language: 'en',
      target_language,
      confidence: 0.85 + Math.random() * 0.1,
      message_id,
      quota_used: Math.floor(Math.random() * 15) + 1,
      quota_remaining: Math.floor(Math.random() * 10) + 5,
      quota_limit: 20,
      provider: 'mock-bot-api',
      processing_time_ms: 300 + Math.random() * 200
    });
  }, 200 + Math.random() * 300);
});

// POST /api/bot/detect-language - Detect language
app.post('/api/bot/detect-language', (req, res) => {
  const { text } = req.body;

  if (!text) {
    return res.status(400).json({
      error: 'Missing required field: text'
    });
  }

  // Simple pattern-based detection
  let detectedLanguage = 'en';

  if (/[\u4e00-\u9fff]/.test(text)) {
    detectedLanguage = 'zh';
  } else if (/[\u3040-\u309f\u30a0-\u30ff]/.test(text)) {
    detectedLanguage = 'ja';
  } else if (/[\uac00-\ud7af]/.test(text)) {
    detectedLanguage = 'ko';
  } else if (/[\u0400-\u04ff]/.test(text)) {
    detectedLanguage = 'ru';
  }

  res.json({
    success: true,
    language: detectedLanguage,
    confidence: 0.9 + Math.random() * 0.1,
    provider: 'mock-bot-api'
  });
});

// GET /api/bot/status - Bot service status
app.get('/api/bot/status', (req, res) => {
  res.json({
    success: true,
    translation_bot: {
      status: 'active',
      version: '1.0.0',
      uptime: process.uptime(),
      endpoints: ['translate', 'detect-language', 'languages']
    },
    ai_assistant: {
      status: 'active',
      model: 'mock-gpt-3.5',
      capabilities: ['analysis', 'summarization']
    },
    custom_bots: [],
    provider: 'mock-bot-api',
    timestamp: new Date().toISOString()
  });
});

// Health check
app.get('/health', (req, res) => {
  res.json({
    status: 'healthy',
    uptime: process.uptime(),
    timestamp: new Date().toISOString()
  });
});

// 404 handler
app.use('*', (req, res) => {
  res.status(404).json({
    error: 'Endpoint not found',
    path: req.originalUrl,
    available_endpoints: [
      'GET /api/bot/languages',
      'POST /api/bot/translate',
      'POST /api/bot/detect-language',
      'GET /api/bot/status',
      'GET /health'
    ]
  });
});

// Error handler
app.use((err, req, res, next) => {
  console.error('🚨 [Bot Mock API] Error:', err);
  res.status(500).json({
    error: 'Internal server error',
    message: err.message
  });
});

// Start server
app.listen(PORT, () => {
  console.log(`🚀 [Bot Mock API] Server running on http://localhost:${PORT}`);
  console.log(`📋 Available endpoints:`);
  console.log(`   GET  /api/bot/languages`);
  console.log(`   POST /api/bot/translate`);
  console.log(`   POST /api/bot/detect-language`);
  console.log(`   GET  /api/bot/status`);
  console.log(`   GET  /health`);
});

// Graceful shutdown
process.on('SIGTERM', () => {
  console.log('🛑 [Bot Mock API] Server shutting down...');
  process.exit(0);
});

process.on('SIGINT', () => {
  console.log('🛑 [Bot Mock API] Server shutting down...');
  process.exit(0);
}); 