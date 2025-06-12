/**
 * Auth Guard
 * 路由认证守卫
 */
import { useAuthStore } from '@/stores/auth';

/**
 * 需要认证的路由
 */
const requiresAuth = (to) => {
  return to.matched.some(record => record.meta.requiresAuth !== false);
};

/**
 * 公开路由（不需要认证）
 */
const publicRoutes = ['/login', '/register', '/forgot-password', '/reset-password'];

/**
 * 创建认证守卫
 */
export function createAuthGuard(router) {
  router.beforeEach(async (to, from, next) => {
    const authStore = useAuthStore();
    
    // 等待认证初始化完成
    if (!authStore.isInitialized) {
      await authStore.initialize();
    }
    
    const isPublicRoute = publicRoutes.includes(to.path) || to.meta.public === true;
    const isAuthenticated = authStore.isLoggedIn;
    
    // 记录导航
    console.log(`🔒 Auth Guard: ${from.path} → ${to.path}`, {
      isPublicRoute,
      isAuthenticated,
      requiresAuth: requiresAuth(to)
    });
    
    // 已登录用户访问登录/注册页面，重定向到首页
    if (isAuthenticated && (to.path === '/login' || to.path === '/register')) {
      return next('/home');
    }
    
    // 未登录用户访问需要认证的页面
    if (!isAuthenticated && !isPublicRoute && requiresAuth(to)) {
      // 保存原始目标路径
      sessionStorage.setItem('redirectPath', to.fullPath);
      return next('/login');
    }
    
    // 检查会话超时
    if (isAuthenticated && authStore.checkSessionTimeout()) {
      console.log('⏰ Session timeout detected');
      await authStore.logout();
      return next('/login');
    }
    
    // 更新最后活动时间
    if (isAuthenticated) {
      authStore.updateLastActivity();
    }
    
    next();
  });
  
  // 路由后置守卫
  router.afterEach((to, from) => {
    // 清除加载状态
    const authStore = useAuthStore();
    authStore.loading = false;
    
    // 页面标题更新
    const defaultTitle = 'Fechatter';
    document.title = to.meta.title ? `${to.meta.title} - ${defaultTitle}` : defaultTitle;
  });
}

/**
 * 路由元数据助手
 */
export const authMeta = {
  // 需要认证
  requiresAuth: () => ({ requiresAuth: true }),
  
  // 公开路由
  public: () => ({ requiresAuth: false, public: true }),
  
  // 需要特定角色
  requiresRole: (role) => ({ requiresAuth: true, requiresRole: role }),
  
  // 需要特定权限
  requiresPermission: (permission) => ({ requiresAuth: true, requiresPermission: permission })
};