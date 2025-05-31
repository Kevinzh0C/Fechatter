import axios from 'axios';

// 创建axios实例
const api = axios.create({
  baseURL: 'http://127.0.0.1:6688/api',
  timeout: 15000, // 增加超时时间
  headers: {
    'Content-Type': 'application/json',
  },
});

// 网络状态监控
let isOnline = navigator.onLine;
let networkStatusListeners = [];

// 监听网络状态变化
window.addEventListener('online', () => {
  isOnline = true;
  networkStatusListeners.forEach(callback => callback(true));
});

window.addEventListener('offline', () => {
  isOnline = false;
  networkStatusListeners.forEach(callback => callback(false));
});

// 重试配置
const MAX_RETRIES = 3;
const RETRY_DELAY = 1000;

// 延迟函数
const delay = (ms) => new Promise(resolve => setTimeout(resolve, ms));

// 请求拦截器 - 添加认证token
api.interceptors.request.use(
  (config) => {
    // 检查网络状态
    if (!isOnline) {
      return Promise.reject(new Error('Network is offline'));
    }

    const token = localStorage.getItem('auth_token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    
    // 添加请求时间戳用于调试
    config.metadata = { startTime: new Date() };
    
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// 响应拦截器 - 处理错误和重试
api.interceptors.response.use(
  (response) => {
    // 添加响应时间信息用于调试
    if (response.config.metadata) {
      const endTime = new Date();
      const duration = endTime.getTime() - response.config.metadata.startTime.getTime();
      console.debug(`API Request to ${response.config.url} took ${duration}ms`);
    }
    return response;
  },
  async (error) => {
    const config = error.config;
    
    // 如果没有config或者已经重试过了，直接返回错误
    if (!config || config.__retryCount >= MAX_RETRIES) {
      return handleApiError(error);
    }

    // 初始化重试计数
    config.__retryCount = config.__retryCount || 0;
    config.__retryCount++;

    // 如果是网络错误或5xx错误，进行重试
    if (shouldRetry(error)) {
      console.warn(`API request failed, retrying (${config.__retryCount}/${MAX_RETRIES})...`);
      await delay(RETRY_DELAY * config.__retryCount);
      return api.request(config);
    }

    return handleApiError(error);
  }
);

// 判断是否应该重试
function shouldRetry(error) {
  // 网络错误
  if (!error.response) {
    return true;
  }
  
  // 5xx服务器错误
  if (error.response.status >= 500) {
    return true;
  }
  
  // 429 请求过多
  if (error.response.status === 429) {
    return true;
  }
  
  return false;
}

// 统一错误处理
function handleApiError(error) {
  console.error('API Error:', error);
  
  // 网络错误
  if (!error.response) {
    if (!isOnline) {
      error.message = 'You are offline. Please check your internet connection.';
    } else {
      error.message = 'Network error. Please check your connection and try again.';
    }
    return Promise.reject(error);
  }

  // 401 未授权
  if (error.response.status === 401) {
    // Token过期或无效，清除本地存储
    localStorage.removeItem('auth_token');
    localStorage.removeItem('token');
    localStorage.removeItem('refreshToken');
    localStorage.removeItem('user');
    
    // 如果不在登录页面，重定向到登录页
    if (window.location.pathname !== '/login') {
      window.location.href = '/login';
    }
    
    error.message = 'Your session has expired. Please log in again.';
  }
  
  // 403 禁止访问
  else if (error.response.status === 403) {
    error.message = 'You do not have permission to perform this action.';
  }
  
  // 404 未找到
  else if (error.response.status === 404) {
    error.message = 'The requested resource was not found.';
  }
  
  // 422 验证错误
  else if (error.response.status === 422) {
    error.message = error.response.data?.message || 'Validation error.';
  }
  
  // 429 请求过多
  else if (error.response.status === 429) {
    error.message = 'Too many requests. Please wait a moment and try again.';
  }
  
  // 5xx 服务器错误
  else if (error.response.status >= 500) {
    error.message = 'Server error. Please try again later.';
  }
  
  // 其他错误
  else {
    error.message = error.response.data?.message || 'An unexpected error occurred.';
  }

  return Promise.reject(error);
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
    // 强制使用IPv4地址避免IPv6连接问题
    const response = await axios.get('http://127.0.0.1:6688/health', {
      timeout: 5000,
      headers: {
        'Content-Type': 'application/json',
      },
    });
    return response.data;
  } catch (error) {
    console.error('Health check failed:', error);
    throw error;
  }
};

export default api; 