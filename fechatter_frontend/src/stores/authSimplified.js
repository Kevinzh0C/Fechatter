/**
 * 🎯 Simplified Auth Store - 基于状态机的现代前端设计
 * 
 * 设计原则：
 * - KISS: 保持简单直接
 * - Single Source of Truth: 唯一状态源
 * - Event-Driven: 事件驱动
 * - State Machine: 明确的状态转换
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import authService from '@/services/auth.service';
import { authEventBus } from '@/services/auth-events';

// 🎪 认证状态机定义
const AUTH_STATES = {
  LOGGED_OUT: 'logged_out',
  LOGGING_IN: 'logging_in',
  AUTHENTICATED: 'authenticated',
  REFRESHING: 'refreshing',
  ERROR: 'error'
};

// 🔄 状态转换规则
const STATE_TRANSITIONS = {
  [AUTH_STATES.LOGGED_OUT]: [AUTH_STATES.LOGGING_IN],
  [AUTH_STATES.LOGGING_IN]: [AUTH_STATES.AUTHENTICATED, AUTH_STATES.ERROR],
  [AUTH_STATES.AUTHENTICATED]: [AUTH_STATES.REFRESHING, AUTH_STATES.LOGGED_OUT, AUTH_STATES.ERROR],
  [AUTH_STATES.REFRESHING]: [AUTH_STATES.AUTHENTICATED, AUTH_STATES.LOGGED_OUT],
  [AUTH_STATES.ERROR]: [AUTH_STATES.LOGGED_OUT, AUTH_STATES.LOGGING_IN]
};

export const useSimplifiedAuthStore = defineStore('auth-simplified', () => {
  // 📦 核心状态 (Single Source of Truth)
  const currentState = ref(AUTH_STATES.LOGGED_OUT);
  const user = ref(null);
  const token = ref(null);
  const error = ref(null);
  const lastActivity = ref(Date.now());

  // 🎯 计算属性 (简化的状态检查)
  const isAuthenticated = computed(() =>
    currentState.value === AUTH_STATES.AUTHENTICATED && !!token.value
  );

  const isLoading = computed(() =>
    [AUTH_STATES.LOGGING_IN, AUTH_STATES.REFRESHING].includes(currentState.value)
  );

  const canAttemptAuth = computed(() =>
    STATE_TRANSITIONS[currentState.value]?.includes(AUTH_STATES.LOGGING_IN)
  );

  // 🔄 状态转换函数
  const transitionTo = (newState) => {
    const allowedTransitions = STATE_TRANSITIONS[currentState.value] || [];

    if (!allowedTransitions.includes(newState)) {
      console.warn(`🚫 Invalid state transition: ${currentState.value} → ${newState}`);
      return false;
    }

    const oldState = currentState.value;
    currentState.value = newState;

    // 🎪 发送状态变化事件
    authEventBus.emit('auth:state-change', {
      from: oldState,
      to: newState,
      timestamp: Date.now()
    });

    if (import.meta.env.DEV) {
      console.log(`🔄 Auth state: ${oldState} → ${newState}`);
    }

    return true;
  };

  // 📦 简化的状态设置 (原子操作)
  const setAuthData = (authResult) => {
    // 🎯 Single operation - 不需要多步验证
    token.value = authResult.accessToken;
    user.value = authResult.user;

    // 🔧 简化存储 - 只存储必要数据
    const authData = {
      token: authResult.accessToken,
      user: authResult.user,
      timestamp: Date.now()
    };

    try {
      localStorage.setItem('auth', JSON.stringify(authData));
      // 🎪 发送认证成功事件
      authEventBus.emit('auth:login', authResult.user);
    } catch (error) {
      console.warn('Failed to persist auth data:', error);
      // 不阻塞登录流程
    }
  };

  // 🚀 简化的登录流程 (乐观更新)
  const login = async (email, password) => {
    if (!canAttemptAuth.value) {
      throw new Error('Cannot attempt login in current state');
    }

    // 🎯 状态转换
    transitionTo(AUTH_STATES.LOGGING_IN);
    error.value = null;

    try {
      // 🌐 API调用
      const authResult = await authService.login(email, password);

      // 📦 设置认证数据 (简单、直接)
      setAuthData(authResult);

      // ✅ 转换到已认证状态
      transitionTo(AUTH_STATES.AUTHENTICATED);

      return true;
    } catch (loginError) {
      // ❌ 转换到错误状态
      error.value = loginError.message;
      transitionTo(AUTH_STATES.ERROR);
      throw loginError;
    }
  };

  // 🔄 智能状态恢复 (延迟加载)
  const attemptRecovery = async () => {
    if (currentState.value === AUTH_STATES.AUTHENTICATED) {
      return true; // 已经认证，无需恢复
    }

    try {
      const authData = JSON.parse(localStorage.getItem('auth') || '{}');

      // 🕐 检查数据新鲜度 (24小时内)
      const isRecent = authData.timestamp &&
        (Date.now() - authData.timestamp) < 24 * 60 * 60 * 1000;

      if (isRecent && authData.token && authData.user) {
        // 🎯 乐观恢复
        token.value = authData.token;
        user.value = authData.user;
        transitionTo(AUTH_STATES.AUTHENTICATED);

        // 🔄 后台验证token有效性 (非阻塞)
        validateTokenInBackground(authData.token);

        return true;
      }
    } catch (error) {
      console.warn('Recovery attempt failed:', error);
    }

    return false;
  };

  // 🔄 后台token验证 (非阻塞)
  const validateTokenInBackground = async (tokenToValidate) => {
    try {
      // 🌐 轻量级验证请求
      await authService.validateToken(tokenToValidate);

      if (import.meta.env.DEV) {
        console.log('✅ Token validation successful');
      }
    } catch (error) {
      // 🔄 静默刷新尝试
      if (authService.refreshToken) {
        try {
          const refreshed = await authService.refreshToken();
          setAuthData(refreshed);

          if (import.meta.env.DEV) {
            console.log('🔄 Token refreshed successfully');
          }
        } catch (refreshError) {
          // 🚪 优雅退出
          logout('Session expired');
        }
      } else {
        logout('Authentication failed');
      }
    }
  };

  // 🚪 简化的登出
  const logout = (reason = 'User logout') => {
    // 🧹 清理状态
    token.value = null;
    user.value = null;
    error.value = null;

    // 🧹 清理存储
    localStorage.removeItem('auth');

    // 🔄 状态转换
    transitionTo(AUTH_STATES.LOGGED_OUT);

    // 🎪 发送登出事件
    authEventBus.emit('auth:logout', { reason });

    if (import.meta.env.DEV) {
      console.log(`🚪 Logged out: ${reason}`);
    }
  };

  // 🔄 活动时间更新
  const updateActivity = () => {
    lastActivity.value = Date.now();
  };

  // 🎯 初始化 (简化版)
  const initialize = async () => {
    if (currentState.value !== AUTH_STATES.LOGGED_OUT) {
      return isAuthenticated.value;
    }

    // 🔄 尝试从存储恢复
    const recovered = await attemptRecovery();

    if (import.meta.env.DEV) {
      console.log(`🎯 Auth initialized: ${recovered ? 'recovered' : 'logged out'}`);
    }

    return recovered;
  };

  // 🎪 导出接口
  return {
    // 状态
    currentState: readonly(currentState),
    user: readonly(user),
    error: readonly(error),

    // 计算属性
    isAuthenticated,
    isLoading,
    canAttemptAuth,

    // 方法
    login,
    logout,
    initialize,
    updateActivity,
    attemptRecovery,

    // 调试
    getState: () => ({
      currentState: currentState.value,
      hasToken: !!token.value,
      hasUser: !!user.value,
      lastActivity: lastActivity.value
    })
  };
});

// 🎪 全局事件监听器设置
if (typeof window !== 'undefined') {
  // 🔄 页面活动监听
  ['click', 'keypress', 'scroll', 'mousemove'].forEach(eventType => {
    document.addEventListener(eventType, () => {
      const store = useSimplifiedAuthStore();
      if (store.isAuthenticated) {
        store.updateActivity();
      }
    }, { passive: true });
  });

  // 🎪 调试接口
  window.authState = () => useSimplifiedAuthStore().getState();
} 