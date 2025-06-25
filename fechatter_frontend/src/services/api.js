import axios from 'axios';
import { getApiConfig, isDebugEnabled } from '@/utils/configLoader';
import tokenManager from '@/services/tokenManager';

// 默认配置 - 在配置加载之前使用
const defaultConfig = {
  baseURL: 'http://127.0.0.1:8080/api',      // 通过Gateway
  sseURL: 'http://127.0.0.1:6687/events',      // 临时直连notify-server绕过Gateway路由问题
  timeout: 30000,
  enableDebug: true,
  maxRetries: 3,
  retryDelay: 1000,
};

// 获取当前配置
function getCurrentConfig() {
  const yamlConfig = getApiConfig();

  // 如果YAML配置可用，使用YAML配置
  if (yamlConfig && Object.keys(yamlConfig).length > 0) {
    return {
      baseURL: yamlConfig.base_url || defaultConfig.baseURL,
      sseURL: yamlConfig.sse_url || defaultConfig.sseURL,
      timeout: yamlConfig.timeout || defaultConfig.timeout,
      enableDebug: isDebugEnabled(),
      maxRetries: yamlConfig.max_retries || defaultConfig.maxRetries,
      retryDelay: yamlConfig.retry_delay || defaultConfig.retryDelay,
    };
  }

  // 否则使用默认配置
  return defaultConfig;
}

// 动态获取配置
const getConfig = () => getCurrentConfig();

console.log('  📡 API Server (via Gateway):', getConfig().baseURL);
console.log('  🔌 SSE Server (via Gateway):', getConfig().sseURL);

// 创建axios实例 - 使用动态配置
const createApiInstance = () => {
  const config = getConfig();

  return axios.create({
    baseURL: config.baseURL,
    timeout: config.timeout,
    headers: {
      'Content-Type': 'application/json',
      'Accept': 'application/json'
    }
  });
};

// 初始API实例
let api = createApiInstance();

// 配置更新时重新创建API实例
export function updateApiInstance() {
  api = createApiInstance();
  return api;
}

// 网络状态监控
let isOnline = navigator.onLine;
let networkStatusListeners = [];

// 监听网络状态变化
window.addEventListener('online', () => {
  isOnline = true;
  networkStatusListeners.forEach(callback => callback(true));
  if (getConfig().enableDebug) console.log('🌐 Network connection restored');
});

window.addEventListener('offline', () => {
  isOnline = false;
  networkStatusListeners.forEach(callback => callback(false));
  if (getConfig().enableDebug) console.warn('🌐 Network connection lost');
});

// 重试配置 - 动态获取
const getRetryConfig = () => {
  const config = getConfig();
  return {
    maxRetries: config.maxRetries || 3,
    retryDelay: config.retryDelay || 1000
  };
};

// 延迟函数
const delay = (ms) => new Promise(resolve => setTimeout(resolve, ms));

// 性能监控
let requestCounter = 0;
const performanceMetrics = {
  requests: 0,
  errors: 0,
  totalTime: 0,
  avgTime: 0,
};

// 请求拦截器 - 极简版本
api.interceptors.request.use(
  async (config) => {
    // 调试日志
    if (getConfig().enableDebug) {
      console.log(`🚀 API Request (via Gateway):`, {
        method: config.method,
        url: config.url,
        baseURL: config.baseURL,
        fullURL: config.baseURL + config.url,
        data: config.data
      });
    }

    // 关键: 如果是 FormData，让浏览器/axios自动设置 Content-Type，包含 boundary
    if (config.data instanceof FormData) {
      delete config.headers['Content-Type'];
    }

    // Skip auth refresh for certain requests
    if (config.skipAuthRefresh) {
      return config;
    }

    // 添加认证令牌 - 使用 Token Manager
    if (!config.headers.Authorization) {
      const token = tokenManager.getAccessToken();

      if (token) {
        // Check if token needs refresh before making request
        if (tokenManager.shouldRefreshToken() && !tokenManager.state.isRefreshing) {
          try {
            await tokenManager.refreshToken();
          } catch (error) {
            console.error('Failed to refresh token before request:', error);
          }
        }

        // Get potentially updated token
        const currentToken = tokenManager.getAccessToken();
        if (currentToken) {
          config.headers.Authorization = `Bearer ${currentToken}`;
          if (getConfig().enableDebug) {
            console.log('🔑 Added auth token to request', {
              url: config.url,
              hasToken: true
            });
          }
        }
      } else {
        if (getConfig().enableDebug) {
          console.warn('⚠️ No auth token available for request:', config.url);
        }
      }
    }

    // Track user activity
    tokenManager.updateActivity();

    return config;
  },
  (error) => {
    console.error('🚫 Request interceptor error:', error);
    return Promise.reject(error);
  }
);

// 响应拦截器 - 极简版本
api.interceptors.response.use(
  (response) => {
    if (getConfig().enableDebug) {
      console.log('✅ API Response (via Gateway):', {
        status: response.status,
        url: response.config.url,
        data: response.data
      });
    }
    return response;
  },
  async (error) => {
    console.error('🚨 API Error (via Gateway):', {
      message: error.message,
      code: error.code,
      response: error.response?.data,
      status: error.response?.status,
      config: error.config
    });

    // Handle 401 Unauthorized with token manager
    if (error.response?.status === 401 && !error.config.skipAuthRefresh) {
      // Check if this is a retry after refresh
      if (error.config._retry) {
        console.error('🔐 [AUTH] Token refresh already attempted, giving up');
        return handleAuthFailure(error);
      }

      // Try to refresh token
      try {
        error.config._retry = true;
        await tokenManager.refreshToken();

        // Retry original request with new token
        const newToken = tokenManager.getAccessToken();
        if (newToken) {
          error.config.headers.Authorization = `Bearer ${newToken}`;
          return api.request(error.config);
        }
      } catch (refreshError) {
        console.error('🔐 [AUTH] Token refresh failed:', refreshError);
        return handleAuthFailure(error);
      }
    }

    return Promise.reject(error);
  }
);

// Handle authentication failure
async function handleAuthFailure(error) {
  const url = error.config?.url;
  const responseData = error.response?.data;

  console.log('🔐 [AUTH] Authentication failure:', {
    url,
    responseData
  });

  // Don't logout for missing endpoints
  if (responseData?.error?.includes('Not Found') ||
    responseData?.message?.includes('not found')) {
    console.warn('🔐 [AUTH] 401 due to missing API endpoint');
    return Promise.reject(error);
  }

  // Clear auth and redirect
  try {
    const { useAuthStore } = await import('@/stores/auth');
    const authStore = useAuthStore();
    await authStore.logout('Your session has expired. Please login again.');
  } catch (logoutError) {
    console.error('Failed to logout:', logoutError);
    // Fallback: clear tokens and redirect
    tokenManager.clearTokens();
    if (window.location.pathname !== '/login') {
      window.location.href = '/login';
    }
  }

  return Promise.reject(error);
}

// 判断是否应该重试 - 更严格的重试条件
function shouldRetry(error) {
  // 网络错误且不是超时
  if (!error.response && !error.code?.includes('TIMEOUT')) {
    return true;
  }

  // 5xx服务器错误，但不包括502/503（可能是服务停止）
  if (error.response?.status >= 500 && error.response?.status < 502) {
    return true;
  }

  // 429 请求过多
  if (error.response?.status === 429) {
    return true;
  }

  return false;
}

// 统一错误处理 - 优化错误消息
function handleApiError(error) {
  console.error('❌ API Error:', {
    message: error.message,
    status: error.response?.status,
    url: error.config?.url,
    method: error.config?.method,
    data: error.response?.data
  });

  // 网络错误
  if (!error.response) {
    console.error('🔍 Network Error Details:', {
      code: error.code,
      message: error.message,
      config: error.config,
      isOnline: isOnline,
      request: error.request
    });

    // 简化错误消息处理
    if (error.code === 'ERR_NETWORK' || error.message?.includes('Network Error')) {
      error.message = 'Network error. Please check your connection and try again.';
    }

    return Promise.reject(error);
  }

  // HTTP状态码处理
  const status = error.response.status;
  const responseData = error.response.data;

  switch (status) {
    case 401:
      if (responseData?.error?.code === 'INVALID_CREDENTIALS') {
        error.message = 'Invalid email or password. Please check your credentials.';
      } else {
        error.message = 'Your session has expired. Please log in again.';
        clearAuthState();
        redirectToLogin();
      }
      break;

    case 403:
      error.message = 'You do not have permission to perform this action.';
      break;

    case 404:
      error.message = 'The requested resource was not found.';
      break;

    case 422:
      error.message = responseData?.message || responseData?.error?.message || 'Validation error.';
      break;

    case 429:
      error.message = 'Too many requests. Please wait a moment and try again.';
      break;

    case 500:
      error.message = 'Internal server error. Please try again later.';
      break;

    case 502:
      error.message = 'Service temporarily unavailable. Please ensure the backend server is running.';
      break;

    case 503:
      error.message = 'Service unavailable. Please try again later.';
      break;

    default:
      // 尝试从响应中解析错误消息
      if (responseData?.error?.message) {
        error.message = responseData.error.message;
      } else if (responseData?.message) {
        error.message = responseData.message;
      } else if (responseData?.error) {
        error.message = typeof responseData.error === 'string' ? responseData.error : 'An unexpected error occurred.';
      } else {
        error.message = 'An unexpected error occurred.';
      }
  }

  return Promise.reject(error);
}

// 清除认证状态
function clearAuthState() {
  const keysToRemove = ['auth_token', 'token', 'refreshToken', 'user'];
  keysToRemove.forEach(key => localStorage.removeItem(key));
}

// 重定向到登录页
function redirectToLogin() {
  if (window.location.pathname !== '/login' && window.location.pathname !== '/register') {
    window.location.href = '/login';
  }
}

// 导出网络状态监控API
export const networkStatus = {
  isOnline: () => isOnline,
  onStatusChange: (callback) => {
    networkStatusListeners.push(callback);
    // 返回取消订阅函数
    return () => {
      networkStatusListeners = networkStatusListeners.filter(cb => cb !== callback);
    };
  }
};

// 健康检查函数
export const healthCheck = async () => {
  try {
    const config = getConfig();
    const healthUrl = `${config.baseURL.replace('/api', '')}/health`; // 通过Gateway访问健康检查

    const response = await axios.get(healthUrl, {
      timeout: 5000,
      headers: {
        'Content-Type': 'application/json',
      },
    });
    return response.data;
  } catch (error) {
    if (getConfig().enableDebug) console.error('❌ Health check failed:', error);
    throw error;
  }
};

// 性能监控API
export const getPerformanceMetrics = () => ({
  ...performanceMetrics,
  uptime: performance.now(),
  isOnline,
});

// API 配置获取 - 导出当前配置
export const getCurrentApiConfig = () => ({ ...getConfig() });

// 文件上传工具
export const uploadFile = async (file, onProgress = null) => {
  const formData = new FormData();
  formData.append('file', file);

  return api.post('/files/single', formData, {
    onUploadProgress: onProgress ? (progressEvent) => {
      const progress = Math.round((progressEvent.loaded * 100) / progressEvent.total);
      onProgress(progress);
    } : undefined,
  });
};

// 批量文件上传
export const uploadFiles = async (files, onProgress = null) => {
  const formData = new FormData();
  files.forEach(file => formData.append('files', file));

  return api.post('/upload', formData, {
    onUploadProgress: onProgress ? (progressEvent) => {
      const progress = Math.round((progressEvent.loaded * 100) / progressEvent.total);
      onProgress(progress);
    } : undefined,
  });
};

// SSE 连接管理 (由专门的实时通信服务处理)
// 注意：SSE连接由 @/services/sse.js 中的 realtimeCommunicationService 管理
// 此处不再提供SSE连接方法，保持API服务的单一职责

export const getSSEConfig = () => ({
  sseURL: getConfig().sseURL,
});

// 搜索服务
export const SearchService = {
  // 搜索消息 - 支持全局搜索和特定聊天搜索
  async search(params) {
    try {
      const {
        query,
        chatId = null,
        timeRange = 'all',
        messageType = 'all',
        fromUser = '',
        sortBy = 'newest',
        page = 1,
        limit = 20
      } = params;

      // 如果提供了 chatId，使用聊天特定的搜索端点
      if (chatId) {
        const searchParams = new URLSearchParams();
        searchParams.append('q', query);
        searchParams.append('limit', limit.toString());
        if (page > 1) searchParams.append('offset', ((page - 1) * limit).toString());
        if (sortBy && sortBy !== 'relevance') {
          // Map frontend sortBy to backend sort parameter
          const sortMap = {
            'newest': 'date_desc',
            'oldest': 'date_asc',
            'relevance': 'relevance'
          };
          searchParams.append('sort', sortMap[sortBy] || 'relevance');
        }

        console.log('🔍 [SearchService] Making search request:', {
          url: `/chat/${chatId}/messages/search?${searchParams.toString()}`,
          method: 'GET',
          params: searchParams.toString()
        });

        try {
          const response = await api.get(`/chat/${chatId}/messages/search?${searchParams.toString()}`);

          // 处理 ApiResponse 格式 - 后端返回的是 SearchResponse { success, data: MessageSearchResults, message }
          const responseData = response.data?.data || response.data;

          // MessageSearchResults 包含 hits (消息数组), total, took_ms, query, page
          if (responseData && responseData.hits) {
            return {
              results: responseData.hits || [],
              total: responseData.total || 0,
              page: responseData.page?.offset ? Math.floor(responseData.page.offset / limit) + 1 : page,
              limit: responseData.page?.limit || limit,
              hasMore: responseData.page?.has_more || false
            };
          }

          // 兼容旧格式
          return {
            results: responseData.results || responseData || [],
            total: responseData.total || responseData.length || 0,
            page: page,
            limit: limit,
            hasMore: responseData.has_more || false
          };
        } catch (error) {
          // If chat search fails with 401, try global search and filter
          if (error.response?.status === 401) {
            console.warn('🔍 Chat search failed with 401, falling back to global search');

            // Use global search
            const globalSearchParams = new URLSearchParams();
            globalSearchParams.append('q', query);
            globalSearchParams.append('limit', limit * 2); // Get more results to filter
            if (page > 1) globalSearchParams.append('offset', ((page - 1) * limit).toString());

            try {
              const globalResponse = await api.get(`/search/messages?${globalSearchParams.toString()}`);
              const globalData = globalResponse.data?.data || globalResponse.data;

              if (globalData && globalData.hits) {
                // Filter results by chatId
                const filteredHits = globalData.hits.filter(msg => msg.chat_id === chatId);

                return {
                  results: filteredHits.slice(0, limit),
                  total: filteredHits.length,
                  page: page,
                  limit: limit,
                  hasMore: filteredHits.length > limit
                };
              }
            } catch (globalError) {
              console.error('🔍 Global search also failed:', globalError);
              throw globalError;
            }
          }

          throw error;
        }
      }

      // 全局搜索 - 注意：后端暂时只支持基本参数
      const searchParams = new URLSearchParams();
      if (query) searchParams.append('q', query);
      searchParams.append('limit', limit.toString());
      if (page > 1) searchParams.append('offset', ((page - 1) * limit).toString());
      if (sortBy && sortBy !== 'relevance') {
        // Map frontend sortBy to backend sort parameter
        const sortMap = {
          'newest': 'date_desc',
          'oldest': 'date_asc',
          'relevance': 'relevance'
        };
        searchParams.append('sort', sortMap[sortBy] || 'relevance');
      }

      const response = await api.get(`/search/messages?${searchParams.toString()}`);

      // 处理 ApiResponse 格式 - 后端返回的是 SearchResponse { success, data: MessageSearchResults, message }
      const responseData = response.data?.data || response.data;

      // MessageSearchResults 包含 hits (消息数组), total, took_ms, query, page
      if (responseData && responseData.hits) {
        return {
          results: responseData.hits || [],
          total: responseData.total || 0,
          page: responseData.page?.offset ? Math.floor(responseData.page.offset / limit) + 1 : page,
          limit: responseData.page?.limit || limit,
          hasMore: responseData.page?.has_more || false
        };
      }

      // 兼容旧格式
      return {
        results: responseData.results || responseData || [],
        total: responseData.total || responseData.pagination?.total || 0,
        page: responseData.page || responseData.pagination?.page || page,
        limit: responseData.limit || responseData.pagination?.limit || limit,
        hasMore: responseData.has_more || responseData.pagination?.has_more || false
      };
    } catch (error) {
      if (getConfig().enableDebug) {
        console.error('🔍 Search API error:', error);
      }
      // 返回空结果而不是抛出错误，让 UI 可以优雅处理
      return {
        results: [],
        total: 0,
        page: 1,
        limit: 20,
        hasMore: false
      };
    }
  },

  // 搜索建议
  async suggestions(query, limit = 5) {
    try {
      const response = await api.get(`/search/suggestions?q=${encodeURIComponent(query)}&limit=${limit}`);
      const data = response.data?.data || response.data;
      return data.suggestions || data || [];
    } catch (error) {
      if (getConfig().enableDebug) {
        console.error('🔍 Search suggestions error:', error);
      }
      return [];
    }
  },

  /*
  // TODO: Backend API for search history is not defined in lib.rs.
  // 获取搜索历史
  async getHistory(limit = 10) {
    try {
      const response = await api.get(`/search/history?limit=${limit}`);
      return response.data.history || response.data.data || [];
    } catch (error) {
      if (getConfig().enableDebug) {
        console.error('🔍 Search history error:', error);
      }
      return [];
    }
  },

  // 保存搜索历史
  async saveSearch(query, results) {
    try {
      await api.post('/search/history', {
        query,
        results_count: results.length,
        timestamp: new Date().toISOString()
      });
    } catch (error) {
      if (getConfig().enableDebug) {
        console.error('🔍 Save search history error:', error);
      }
      // 非关键错误，不抛出
    }
  },

  // 清除搜索历史
  async clearHistory() {
    try {
      await api.delete('/search/history');
    } catch (error) {
      if (getConfig().enableDebug) {
        console.error('🔍 Clear search history error:', error);
      }
      throw error;
    }
  },
  */

  /*
  // TODO: Backend API for advanced search is not defined in lib.rs.
  // 高级搜索
  async advancedSearch(filters) {
    try {
      const response = await api.post('/search/advanced', filters);
      const data = response.data;

      return {
        results: data.results || data.data || [],
        total: data.total || data.pagination?.total || 0,
        page: data.page || data.pagination?.page || 1,
        limit: data.limit || data.pagination?.limit || 20,
        hasMore: data.has_more || data.pagination?.has_more || false,
        facets: data.facets || {},
        suggestions: data.suggestions || []
      };
    } catch (error) {
      if (getConfig().enableDebug) {
        console.error('🔍 Advanced search error:', error);
      }
      throw error;
    }
  }
  */
};

export default api; 