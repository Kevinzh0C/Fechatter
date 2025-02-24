/**
 * 🎯 Enhanced Bot Service - 增强Bot服务
 * 扩展原有botService，提供完整的Bot管理功能
 */

import { BotService } from './botService'
import api from './api'

export class EnhancedBotService extends BotService {

  constructor() {
    super()

    // 增强功能配置
    this.analysisTypes = ['sentiment', 'topic', 'intent', 'comprehensive']
    this.summaryStyles = ['brief', 'detailed', 'bullet-points', 'academic']
    this.customBotTypes = ['analyzer', 'summarizer', 'responder', 'translator']

    // 缓存配置
    this.cacheTimeout = 24 * 60 * 60 * 1000 // 24小时
    this.analysisCache = new Map()
    this.summaryCache = new Map()

    // 性能监控
    this.performanceMetrics = {
      translationRequests: 0,
      analysisRequests: 0,
      summaryRequests: 0,
      averageResponseTime: 0,
      errorRate: 0
    }
  }

  // ================================
  // 🎯 Translation Service Enhancement
  // ================================

  /**
   * 增强翻译功能 - 支持缓存、批量翻译、质量评估
   */
  async translateMessage(messageId, targetLang, options = {}) {
    const startTime = Date.now()

    const enhancedOptions = {
      preserveFormatting: options.preserveFormatting || false,
      showConfidence: options.showConfidence || true,
      cacheResult: options.cacheResult !== false,
      detectSource: options.detectSource !== false,
      alternatives: options.alternatives || false,
      ...options
    }

    try {
      // 检查缓存
      if (enhancedOptions.cacheResult) {
        const cachedResult = await this.getCachedTranslation(messageId, targetLang)
        if (cachedResult) {
          this.updatePerformanceMetrics('translation', Date.now() - startTime, true)
          return cachedResult
        }
      }

      // 执行翻译
      const result = await super.translateMessage(messageId, targetLang)

      // 增强结果处理
      const enhancedResult = {
        ...result,
        cached: false,
        processingTime: Date.now() - startTime,
        confidence: result.confidence || this.estimateConfidence(result),
        alternatives: enhancedOptions.alternatives ?
          await this.generateAlternatives(result) : null,
        qualityScore: this.calculateQualityScore(result),
        metadata: {
          timestamp: new Date().toISOString(),
          sourceDetected: enhancedOptions.detectSource,
          preservedFormatting: enhancedOptions.preserveFormatting
        }
      }

      // 缓存结果
      if (enhancedOptions.cacheResult) {
        await this.cacheTranslation(messageId, targetLang, enhancedResult)
      }

      this.updatePerformanceMetrics('translation', Date.now() - startTime, true)
      return enhancedResult

    } catch (error) {
      this.updatePerformanceMetrics('translation', Date.now() - startTime, false)
      return this.handleTranslationError(error, messageId, targetLang)
    }
  }

  /**
   * 批量翻译消息
   */
  async translateMessagesBatch(messageIds, targetLang, options = {}) {
    const batchSize = options.batchSize || 5
    const results = []

    for (let i = 0; i < messageIds.length; i += batchSize) {
      const batch = messageIds.slice(i, i + batchSize)
      const batchPromises = batch.map(id =>
        this.translateMessage(id, targetLang, options)
      )

      try {
        const batchResults = await Promise.allSettled(batchPromises)
        results.push(...batchResults.map(result =>
          result.status === 'fulfilled' ? result.value : { error: result.reason }
        ))
      } catch (error) {
        console.error('Batch translation error:', error)
      }
    }

    return {
      results,
      total: messageIds.length,
      successful: results.filter(r => !r.error).length,
      failed: results.filter(r => r.error).length
    }
  }

  // ================================
  // 🎯 AI Analysis Service
  // ================================

  /**
   * AI消息分析 - 支持多种分析类型
   */
  async analyzeMessage(messageId, analysisType = 'comprehensive', options = {}) {
    const startTime = Date.now()

    // 验证分析类型
    if (!this.analysisTypes.includes(analysisType)) {
      throw new Error(`Invalid analysis type: ${analysisType}`)
    }

    // 检查缓存
    const cacheKey = `${messageId}-${analysisType}`
    if (options.useCache !== false) {
      const cached = this.analysisCache.get(cacheKey)
      if (cached && (Date.now() - cached.timestamp) < this.cacheTimeout) {
        return { ...cached.result, cached: true }
      }
    }

    try {
      const response = await api.post('/bot/analyze', {
        message_id: String(messageId),
        analysis_type: analysisType,
        options: {
          includeMetadata: options.includeMetadata !== false,
          detailLevel: options.detailLevel || 'standard',
          language: options.language || 'auto'
        }
      })

      const result = {
        success: true,
        analysis: response.data.analysis,
        confidence: response.data.confidence,
        suggestions: response.data.suggestions || [],
        metadata: {
          type: analysisType,
          processingTime: Date.now() - startTime,
          timestamp: new Date().toISOString(),
          version: response.data.version || '1.0'
        },
        rawData: options.includeRawData ? response.data : null
      }

      // 缓存结果
      if (options.useCache !== false) {
        this.analysisCache.set(cacheKey, {
          result,
          timestamp: Date.now()
        })
      }

      this.updatePerformanceMetrics('analysis', Date.now() - startTime, true)
      return result

    } catch (error) {
      this.updatePerformanceMetrics('analysis', Date.now() - startTime, false)
      throw new Error(`Analysis failed: ${error.message}`)
    }
  }

  /**
   * 批量分析消息
   */
  async analyzeMessagesBatch(messageIds, analysisType = 'comprehensive', options = {}) {
    try {
      const response = await api.post('/bot/analyze/batch', {
        message_ids: messageIds.map(id => String(id)),
        analysis_type: analysisType,
        options
      })

      return {
        success: true,
        results: response.data.results,
        summary: response.data.summary,
        metadata: {
          totalAnalyzed: messageIds.length,
          processingTime: response.data.processing_time,
          timestamp: new Date().toISOString()
        }
      }
    } catch (error) {
      throw new Error(`Batch analysis failed: ${error.message}`)
    }
  }

  // ================================
  // 🎯 AI Summary Service
  // ================================

  /**
   * AI消息总结 - 支持多种总结风格
   */
  async summarizeMessage(messageId, style = 'brief', options = {}) {
    const startTime = Date.now()

    // 验证总结风格
    if (!this.summaryStyles.includes(style)) {
      throw new Error(`Invalid summary style: ${style}`)
    }

    // 检查缓存
    const cacheKey = `${messageId}-${style}`
    if (options.useCache !== false) {
      const cached = this.summaryCache.get(cacheKey)
      if (cached && (Date.now() - cached.timestamp) < this.cacheTimeout) {
        return { ...cached.result, cached: true }
      }
    }

    try {
      const response = await api.post('/bot/summarize', {
        message_id: String(messageId),
        style: style,
        options: {
          maxLength: options.maxLength || 200,
          includeKeyPoints: options.includeKeyPoints !== false,
          language: options.language || 'auto'
        }
      })

      const result = {
        success: true,
        summary: response.data.summary,
        keyPoints: response.data.key_points || [],
        wordCount: response.data.word_count || 0,
        readingTime: response.data.reading_time || 0,
        metadata: {
          style,
          processingTime: Date.now() - startTime,
          timestamp: new Date().toISOString(),
          compressionRatio: response.data.compression_ratio || 0
        }
      }

      // 缓存结果
      if (options.useCache !== false) {
        this.summaryCache.set(cacheKey, {
          result,
          timestamp: Date.now()
        })
      }

      this.updatePerformanceMetrics('summary', Date.now() - startTime, true)
      return result

    } catch (error) {
      this.updatePerformanceMetrics('summary', Date.now() - startTime, false)
      throw new Error(`Summarization failed: ${error.message}`)
    }
  }

  /**
   * 对话总结 - 总结整个对话
   */
  async summarizeConversation(chatId, options = {}) {
    try {
      const response = await api.post('/bot/summarize/conversation', {
        chat_id: chatId,
        options: {
          timeRange: options.timeRange || '24h',
          maxMessages: options.maxMessages || 100,
          style: options.style || 'detailed'
        }
      })

      return {
        success: true,
        summary: response.data.summary,
        highlights: response.data.highlights || [],
        participants: response.data.participants || [],
        topics: response.data.topics || [],
        metadata: {
          messageCount: response.data.message_count,
          timeSpan: response.data.time_span,
          timestamp: new Date().toISOString()
        }
      }
    } catch (error) {
      throw new Error(`Conversation summarization failed: ${error.message}`)
    }
  }

  // ================================
  // 🎯 Custom Bot Management
  // ================================

  /**
   * 创建自定义Bot
   */
  async createCustomBot(botConfig) {
    try {
      const response = await api.post('/bot/custom/create', {
        name: botConfig.name,
        type: botConfig.type,
        config: botConfig.config || {},
        description: botConfig.description || '',
        permissions: botConfig.permissions || []
      })

      return {
        success: true,
        bot: response.data.bot
      }
    } catch (error) {
      throw new Error(`Failed to create custom bot: ${error.message}`)
    }
  }

  /**
   * 更新自定义Bot
   */
  async updateCustomBot(botId, updates) {
    try {
      const response = await api.put(`/bot/custom/${botId}`, updates)

      return {
        success: true,
        bot: response.data.bot
      }
    } catch (error) {
      throw new Error(`Failed to update custom bot: ${error.message}`)
    }
  }

  /**
   * 删除自定义Bot
   */
  async deleteCustomBot(botId) {
    try {
      await api.delete(`/bot/custom/${botId}`)

      return { success: true }
    } catch (error) {
      throw new Error(`Failed to delete custom bot: ${error.message}`)
    }
  }

  /**
   * 获取自定义Bot列表
   */
  async getCustomBots() {
    try {
      const response = await api.get('/bot/custom')

      return {
        success: true,
        bots: response.data.bots || []
      }
    } catch (error) {
      throw new Error(`Failed to get custom bots: ${error.message}`)
    }
  }

  // ================================
  // 🎯 Bot Configuration Management
  // ================================

  /**
   * 更新Bot配置
   */
  async updateBotConfig(botType, config) {
    try {
      const response = await api.put(`/bot/config/${botType}`, config)
      return response.data
    } catch (error) {
      throw new Error(`Config update failed: ${error.message}`)
    }
  }

  /**
   * 获取Bot状态
   */
  async getBotStatus() {
    try {
      const response = await api.get('/bot/status')
      return {
        translation: response.data.translation_bot || { status: 'unknown' },
        assistant: response.data.ai_assistant || { status: 'unknown' },
        custom: response.data.custom_bots || []
      }
    } catch (error) {
      console.warn('Failed to get bot status:', error)
      return {
        translation: { status: 'unknown' },
        assistant: { status: 'unknown' },
        custom: []
      }
    }
  }

  // ================================
  // 🎯 Utility Methods
  // ================================

  /**
   * 缓存翻译结果
   */
  async cacheTranslation(messageId, targetLang, result) {
    const key = `${messageId}-${targetLang}`
    const cachedData = {
      ...result,
      cachedAt: Date.now(),
      expiresAt: Date.now() + this.cacheTimeout
    }

    try {
      localStorage.setItem(`translation_cache_${key}`, JSON.stringify(cachedData))
    } catch (error) {
      console.warn('Failed to cache translation:', error)
    }
  }

  /**
   * 获取缓存的翻译结果
   */
  async getCachedTranslation(messageId, targetLang) {
    const key = `${messageId}-${targetLang}`

    try {
      const cached = localStorage.getItem(`translation_cache_${key}`)
      if (cached) {
        const data = JSON.parse(cached)
        if (data.expiresAt > Date.now()) {
          return { ...data, cached: true }
        } else {
          localStorage.removeItem(`translation_cache_${key}`)
        }
      }
    } catch (error) {
      console.warn('Failed to get cached translation:', error)
    }

    return null
  }

  /**
   * 估算翻译置信度
   */
  estimateConfidence(result) {
    // 简单的置信度估算算法
    const textLength = result.translation?.length || 0
    const sourceLength = result.source_text?.length || 0

    if (textLength === 0) return 0

    const lengthRatio = Math.min(textLength / sourceLength, 2)
    const baseConfidence = 0.7
    const lengthAdjustment = (lengthRatio - 0.5) * 0.2

    return Math.max(0.3, Math.min(0.95, baseConfidence + lengthAdjustment))
  }

  /**
   * 生成翻译替代方案
   */
  async generateAlternatives(result) {
    // TODO: 实现替代翻译生成
    return []
  }

  /**
   * 计算翻译质量分数
   */
  calculateQualityScore(result) {
    const confidence = result.confidence || 0.7
    const length = result.translation?.length || 0

    // 简单的质量评分算法
    let score = confidence * 100

    // 长度调整
    if (length < 10) score *= 0.9
    if (length > 500) score *= 0.95

    return Math.round(score)
  }

  /**
   * 更新性能指标
   */
  updatePerformanceMetrics(type, responseTime, success) {
    const metrics = this.performanceMetrics
    const field = `${type}Requests`

    metrics[field]++

    // 更新平均响应时间
    const totalRequests = metrics.translationRequests +
      metrics.analysisRequests +
      metrics.summaryRequests

    metrics.averageResponseTime =
      (metrics.averageResponseTime * (totalRequests - 1) + responseTime) / totalRequests

    // 更新错误率
    if (!success) {
      const totalErrors = metrics.errorRate * (totalRequests - 1) + 1
      metrics.errorRate = totalErrors / totalRequests
    } else {
      metrics.errorRate = (metrics.errorRate * (totalRequests - 1)) / totalRequests
    }
  }

  /**
   * 获取性能指标
   */
  getPerformanceMetrics() {
    return { ...this.performanceMetrics }
  }

  /**
   * 清理缓存
   */
  clearCache() {
    this.analysisCache.clear()
    this.summaryCache.clear()

    // 清理localStorage中的翻译缓存
    const keys = Object.keys(localStorage)
    keys.forEach(key => {
      if (key.startsWith('translation_cache_')) {
        localStorage.removeItem(key)
      }
    })
  }
}

// 创建增强服务实例
export const enhancedBotService = new EnhancedBotService()

// 导出到全局，便于调试
if (typeof window !== 'undefined') {
  window.enhancedBotService = enhancedBotService
}

export default enhancedBotService 