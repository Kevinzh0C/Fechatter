import type { RouterMiddleware } from '../types';
import { useAuthStore } from '../../stores/auth';

export const authMiddleware: RouterMiddleware = async (to, _from, next) => {
  const authStore = useAuthStore();

  // 初始化认证状态 (只执行一次)
  if (!authStore.isInitialized) {
    try {
      await authStore.initialize();
    } catch (error) {
      console.error('Auth initialization failed:', error);
    }
  }

  const isAuthenticated = authStore.isLoggedIn;
  const requiresAuth = to.meta?.requiresAuth;
  const requiresGuest = to.meta?.requiresGuest;

  // 需要认证但未登录
  if (requiresAuth && !isAuthenticated) {
    // 保存目标路径用于登录后重定向
    sessionStorage.setItem('redirectPath', to.fullPath);
    return next('/login');
  }

  // 需要游客状态但已登录
  if (requiresGuest && isAuthenticated) {
    return next('/home');
  }

  next();
};