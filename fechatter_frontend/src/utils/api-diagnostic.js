// API诊断工具
import api from '@/services/api';
import axios from 'axios';

export function diagnoseApiConfigurations() {
  console.log('=== API Configurations ===');

  // 检查API配置
  const config = api.defaults;

  console.log('Current API config:', {
    baseURL: config.baseURL,
    timeout: config.timeout,
    headers: config.headers,
    withCredentials: config.withCredentials,
    validateStatus: config.validateStatus
  });

  // 检查拦截器数量
  console.log('Interceptors count:', {
    request: api.interceptors.request.handlers.length,
    response: api.interceptors.response.handlers.length
  });

  // 检查环境变量
  console.log('Environment:', {
    MODE: import.meta.env.MODE,
    DEV: import.meta.env.DEV,
    VITE_API_BASE_URL: import.meta.env.VITE_API_BASE_URL
  });
}

// 测试直接请求
export async function testDirectRequests() {
  const testData = {
    email: 'admin@test.com',
    password: 'super123'
  };

  console.log('=== Direct API Requests Test ===');

  // Test 1: Raw fetch
  try {
    console.log('Test 1: Raw fetch...');
    const response = await fetch('http://127.0.0.1:8080/api/signin', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json'
      },
      body: JSON.stringify(testData)
    });

    const result = {
      ok: response.ok,
      status: response.status,
      statusText: response.statusText
    };

    if (response.ok) {
      const data = await response.json();
      result.data = data;
    }

    console.log('Raw fetch result:', result);
  } catch (error) {
    console.error('Raw fetch error:', error);
  }

  // Test 2: Raw axios (no interceptors)
  try {
    console.log('\nTest 2: Raw axios...');
    const response = await axios.post('http://127.0.0.1:8080/api/signin', testData, {
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json'
      },
      timeout: 10000
    });
    console.log('Raw axios success:', {
      status: response.status,
      data: response.data
    });
  } catch (error) {
    console.error('Raw axios error:', {
      message: error.message,
      code: error.code,
      status: error.response?.status
    });
  }

  // Test 3: Configured API instance
  try {
    console.log('\nTest 3: Configured API...');
    const response = await api.post('/signin', testData);
    console.log('Configured API success:', {
      status: response.status,
      data: response.data
    });
  } catch (error) {
    console.error('Configured API error:', {
      message: error.message,
      code: error.code,
      status: error.response?.status
    });
  }
}

// 测试健康检查
export async function testHealthCheck() {
  console.log('=== Health Check Test ===');

  const endpoints = [
    'http://127.0.0.1:8080/health',
    'http://127.0.0.1:8080/api/health',
    'http://127.0.0.1:8080/ping'
  ];

  for (const endpoint of endpoints) {
    try {
      console.log(`Testing: ${endpoint}`);
      const response = await fetch(endpoint, {
        method: 'GET',
        headers: { 'Accept': 'application/json' },
        timeout: 5000
      });

      console.log(`✅ ${endpoint}:`, {
        status: response.status,
        ok: response.ok
      });
    } catch (error) {
      console.log(`❌ ${endpoint}:`, error.message);
    }
  }
}

// 导出诊断工具到全局对象
window.diagnoseApi = {
  diagnoseApiConfigurations,
  testDirectRequests,
  testHealthCheck
};