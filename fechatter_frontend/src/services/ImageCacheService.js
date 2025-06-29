/**
 * 🖼️ Smart Image Cache Service
 * 智能图片缓存服务 - 避免重复远端请求，提升用户体验
 */

class ImageCacheService {
  constructor() {
    // 🗄️ 内存缓存 - 存储blob URLs
    this.cache = new Map()

    // ⏰ 缓存过期时间 (30分钟)
    this.CACHE_EXPIRY = 30 * 60 * 1000

    // 缓存统计
    this.stats = {
      hits: 0,
      misses: 0,
      totalRequests: 0,
      cacheSize: 0
    }

    // 🧹 定期清理过期缓存
    this.startCleanupTimer()

    // 💾 localStorage持久化配置
    this.STORAGE_KEY = 'fechatter_image_cache_metadata'
    this.MAX_CACHE_SIZE = 50 * 1024 * 1024 // 50MB
    this.MAX_CACHE_ITEMS = 100 // 最大缓存项目数

    if (this.isDevelopment()) {
      console.log('🖼️ [ImageCache] Service initialized with intelligent caching')
    }
  }

  /**
   * 获取缓存的图片URL (主要接口)
   * @param {string} imageUrl - 原始图片URL
   * @param {object} options - 配置选项
   * @returns {Promise<string>} - Blob URL或原始URL
   */
  async getCachedImageUrl(imageUrl, options = {}) {
    this.stats.totalRequests++

    if (!imageUrl) {
      return null
    }

    // 对于非API图片，直接返回原始URL
    if (!this.isApiImage(imageUrl)) {
      return imageUrl
    }

    const cacheKey = this.getCacheKey(imageUrl)
    const cached = this.cache.get(cacheKey)

    // 缓存命中
    if (cached && this.isCacheValid(cached)) {
      this.stats.hits++
      cached.lastAccessed = Date.now()

      if (this.isDevelopment()) {
        console.log(`[ImageCache] Cache hit for: ${this.truncateUrl(imageUrl)}`)
      }

      return cached.blobUrl
    }

    // ERROR: 缓存未命中，从远端获取
    this.stats.misses++

    try {
      if (this.isDevelopment()) {
        console.log(`📥 [ImageCache] Fetching from remote: ${this.truncateUrl(imageUrl)}`)
      }

      const blobUrl = await this.fetchAndCacheImage(imageUrl, options)
      return blobUrl
    } catch (error) {
      console.error('ERROR: [ImageCache] Failed to fetch image:', error)

      // 🔄 降级处理：返回原始URL
      return imageUrl
    }
  }

  /**
   * SUBSCRIPTION: 从远端获取图片并缓存
   * @param {string} imageUrl - 图片URL
   * @param {object} options - 请求选项
   * @returns {Promise<string>} - Blob URL
   */
  async fetchAndCacheImage(imageUrl, options = {}) {
    const cacheKey = this.getCacheKey(imageUrl)

    // 🔒 检查是否需要认证
    const needsAuth = this.needsAuthentication(imageUrl)

    let response
    if (needsAuth) {
      // 🔐 使用认证请求
      const token = await this.getAuthToken()
      response = await fetch(imageUrl, {
        headers: {
          'Authorization': `Bearer ${token}`,
          ...options.headers
        },
        ...options
      })
    } else {
      // 🌐 普通请求
      response = await fetch(imageUrl, options)
    }

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`)
    }

    const blob = await response.blob()
    const blobUrl = URL.createObjectURL(blob)

    // 💾 存储到缓存
    const cacheItem = {
      blobUrl,
      originalUrl: imageUrl,
      timestamp: Date.now(),
      lastAccessed: Date.now(),
      size: blob.size,
      type: blob.type
    }

    // 🧹 检查缓存容量
    await this.ensureCacheCapacity(blob.size)

    this.cache.set(cacheKey, cacheItem)
    this.updateStats()

    if (this.isDevelopment()) {
      console.log(`💾 [ImageCache] Cached: ${this.truncateUrl(imageUrl)} (${this.formatSize(blob.size)})`)
    }

    return blobUrl
  }

  /**
   * 判断是否为API图片URL
   * @param {string} url - 图片URL
   * @returns {boolean}
   */
  isApiImage(url) {
    return url && (
      url.includes('/api/files/') ||
      url.startsWith('http://localhost') ||
      url.startsWith('http://45.77.178.85')
    )
  }

  /**
   * 🔐 判断是否需要认证
   * @param {string} url - 图片URL
   * @returns {boolean}
   */
  needsAuthentication(url) {
    return url && url.includes('/api/files/')
  }

  /**
   * 🔑 获取认证token
   * @returns {string|null}
   */
  async getAuthToken() {
    try {
      // CRITICAL FIX: 使用与api.js相同的token获取逻辑

      // Priority 1: tokenManager (in-memory, fast)
      try {
        const { default: tokenManager } = await import('./tokenManager')
        const token = tokenManager.getAccessToken()
        if (token) {
          if (this.isDevelopment()) {
            console.log('🔑 [ImageCache] Token from tokenManager')
          }
          return token
        }
      } catch (error) {
        if (this.isDevelopment()) {
          console.warn('WARNING: [ImageCache] TokenManager access failed:', error)
        }
      }

      // Priority 2: authStateManager (localStorage, persistent) 
      try {
        const { default: authStateManager } = await import('../utils/authStateManager')
        const authState = authStateManager.getAuthState()
        const token = authState.token
        if (token) {
          if (this.isDevelopment()) {
            console.log('🔑 [ImageCache] Token from authStateManager')
          }
          return token
        }
      } catch (error) {
        if (this.isDevelopment()) {
          console.warn('WARNING: [ImageCache] AuthStateManager access failed:', error)
        }
      }

      // Priority 3: Direct localStorage access (fallback)
      try {
        // CRITICAL: Use correct localStorage key from auth.js
        const authToken = localStorage.getItem('auth_token')
        if (authToken) {
          if (this.isDevelopment()) {
            console.log('🔑 [ImageCache] Token from localStorage (direct)')
          }
          return authToken
        }

        // Legacy fallback
        const stored = localStorage.getItem('fechatter_auth')
        if (stored) {
          const auth = JSON.parse(stored)
          if (auth.token) {
            if (this.isDevelopment()) {
              console.log('🔑 [ImageCache] Token from localStorage (legacy)')
            }
            return auth.token
          }
        }
      } catch (error) {
        if (this.isDevelopment()) {
          console.warn('WARNING: [ImageCache] localStorage access failed:', error)
        }
      }

      if (this.isDevelopment()) {
        console.warn('WARNING: [ImageCache] No authentication token found')
      }
      return null
    } catch (error) {
      console.error('ERROR: [ImageCache] Failed to get auth token:', error)
      return null
    }
  }

  /**
   * 🔑 生成缓存键
   * @param {string} url - 原始URL
   * @returns {string}
   */
  getCacheKey(url) {
    // 使用URL的hash作为缓存键，忽略query参数
    try {
      const urlObj = new URL(url, window.location.origin)
      return urlObj.pathname
    } catch {
      return url
    }
  }

  /**
   * ⏰ 检查缓存是否有效
   * @param {object} cacheItem - 缓存项
   * @returns {boolean}
   */
  isCacheValid(cacheItem) {
    if (!cacheItem || !cacheItem.blobUrl) {
      return false
    }

    // 🕒 检查是否过期
    const age = Date.now() - cacheItem.timestamp
    return age < this.CACHE_EXPIRY
  }

  /**
   * 🧹 确保缓存容量不超限
   * @param {number} newItemSize - 新项目大小
   */
  async ensureCacheCapacity(newItemSize) {
    // 计算当前缓存大小
    let currentSize = 0
    for (const item of this.cache.values()) {
      currentSize += item.size || 0
    }

    // 🚨 如果超出限制，清理最旧的项目
    const wouldExceed = currentSize + newItemSize > this.MAX_CACHE_SIZE
    const tooManyItems = this.cache.size >= this.MAX_CACHE_ITEMS

    if (wouldExceed || tooManyItems) {
      const itemsToRemove = Array.from(this.cache.entries())
        .sort(([, a], [, b]) => a.lastAccessed - b.lastAccessed)
        .slice(0, Math.max(10, Math.floor(this.cache.size * 0.3))) // 清理30%或至少10个

      for (const [key, item] of itemsToRemove) {
        this.removeCacheItem(key, item)
      }

      if (this.isDevelopment()) {
        console.log(`🧹 [ImageCache] Cleaned ${itemsToRemove.length} items from cache`)
      }
    }
  }

  /**
   * 🗑️ 移除缓存项
   * @param {string} key - 缓存键
   * @param {object} item - 缓存项
   */
  removeCacheItem(key, item) {
    if (item?.blobUrl && item.blobUrl.startsWith('blob:')) {
      URL.revokeObjectURL(item.blobUrl)
    }
    this.cache.delete(key)
  }

  /**
   * 🧹 定期清理过期缓存
   */
  startCleanupTimer() {
    setInterval(() => {
      this.cleanupExpiredCache()
    }, 5 * 60 * 1000) // 每5分钟清理一次
  }

  /**
   * 🗑️ 清理过期缓存
   */
  cleanupExpiredCache() {
    let removedCount = 0

    for (const [key, item] of this.cache.entries()) {
      if (!this.isCacheValid(item)) {
        this.removeCacheItem(key, item)
        removedCount++
      }
    }

    if (removedCount > 0) {
      this.updateStats()

      if (this.isDevelopment()) {
        console.log(`🧹 [ImageCache] Cleaned ${removedCount} expired items`)
      }
    }
  }

  /**
   * 更新统计信息
   */
  updateStats() {
    this.stats.cacheSize = this.cache.size
  }

  /**
   * 📈 获取缓存统计
   * @returns {object}
   */
  getStats() {
    const hitRate = this.stats.totalRequests > 0
      ? (this.stats.hits / this.stats.totalRequests * 100).toFixed(1)
      : '0.0'

    return {
      ...this.stats,
      hitRate: `${hitRate}%`,
      memoryUsage: this.getMemoryUsage()
    }
  }

  /**
   * 💾 计算内存使用量
   * @returns {string}
   */
  getMemoryUsage() {
    let totalSize = 0
    for (const item of this.cache.values()) {
      totalSize += item.size || 0
    }
    return this.formatSize(totalSize)
  }

  /**
   * 📏 格式化文件大小
   * @param {number} bytes - 字节数
   * @returns {string}
   */
  formatSize(bytes) {
    const units = ['B', 'KB', 'MB', 'GB']
    let size = bytes
    let unitIndex = 0

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024
      unitIndex++
    }

    return `${size.toFixed(1)}${units[unitIndex]}`
  }

  /**
   * ✂️ 截断URL用于日志显示
   * @param {string} url - 完整URL
   * @returns {string}
   */
  truncateUrl(url) {
    if (!url) return ''
    if (url.length <= 50) return url
    return url.substring(0, 30) + '...' + url.substring(url.length - 20)
  }

  /**
   * 检查是否为开发环境
   * @returns {boolean}
   */
  isDevelopment() {
    return import.meta.env.DEV || import.meta.env.MODE === 'development'
  }

  /**
   * 🗑️ 清空所有缓存
   */
  clearCache() {
    for (const [key, item] of this.cache.entries()) {
      this.removeCacheItem(key, item)
    }

    this.stats = {
      hits: 0,
      misses: 0,
      totalRequests: 0,
      cacheSize: 0
    }

    if (this.isDevelopment()) {
      console.log('🗑️ [ImageCache] Cache cleared')
    }
  }

  /**
   * 📤 预加载图片
   * @param {string[]} urls - 图片URL数组
   */
  async preloadImages(urls) {
    if (!Array.isArray(urls) || urls.length === 0) return

    const preloadPromises = urls
      .filter(url => url && this.isApiImage(url))
      .map(url => this.getCachedImageUrl(url).catch(error => {
        console.warn(`WARNING: [ImageCache] Preload failed for ${url}:`, error)
      }))

    await Promise.allSettled(preloadPromises)

    if (this.isDevelopment()) {
      console.log(`📤 [ImageCache] Preloaded ${urls.length} images`)
    }
  }
}

// 创建全局单例
const imageCacheService = new ImageCacheService()

export default imageCacheService 