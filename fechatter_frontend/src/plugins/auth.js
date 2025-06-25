/**
 * Auth Plugin
 * 初始化认证系统
 */
import { useAuthStore } from '@/stores/auth';

export default {
  install(app) {
    // 在应用启动时初始化认证
    const authStore = useAuthStore();
    
    // 初始化认证状态
    authStore.initialize();
    
    // 全局错误处理
    app.config.errorHandler = (err, instance, info) => {
      if (err.code === 'UNAUTHORIZED') {
        console.error('Unauthorized access, redirecting to login...');
        authStore.clearAuthState();
        window.location.href = '/login';
      }
    };
    
    // 提供全局访问
    app.config.globalProperties.$auth = authStore;
    app.provide('auth', authStore);
  }
};