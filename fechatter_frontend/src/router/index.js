import { createRouter, createWebHistory } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import { analytics } from '../lib/analytics-protobuf';
import { setupGlobalRouterErrorHandling } from '@/composables/useRouterGuard';
import authStateManager from '../utils/authStateManager';

// Lazy loaded components - Only import components that actually exist
const Home = () => import('@/views/Home.vue');
const Login = () => import('@/views/Login.vue');
const Register = () => import('@/views/Register.vue');
const Chat = () => import('@/views/Chat.vue');
const Demo = () => import('@/views/Demo.vue');
const Test = () => import('@/views/Test.vue');
const Error = () => import('@/views/Error.vue');
const Debug = () => import('@/views/Debug.vue');
const SimpleLogin = () => import('@/views/SimpleLogin.vue');

const routes = [
  // 根路径重定向
  {
    path: '/',
    redirect: (to) => {
      if (import.meta.env.DEV) {
        console.log('🔍 [ROUTER] Root redirect triggered, checking auth state...');
      }

      // 避免在路由初始化时调用 store，直接检查 localStorage
      // 检查两种可能的键名（兼容性）
      const token = localStorage.getItem('auth_token') || localStorage.getItem('fechatter_access_token');
      const tokenExpiry = localStorage.getItem('token_expires_at') || localStorage.getItem('fechatter_token_expiry');
      const rememberMe = localStorage.getItem('remember_me') === 'true';

      // 如果没有勾选记住我，检查 sessionStorage
      const sessionToken = !rememberMe ? sessionStorage.getItem('auth_token') : null;
      const sessionExpiry = !rememberMe ? sessionStorage.getItem('token_expires_at') : null;

      const finalToken = token || sessionToken;
      const finalExpiry = tokenExpiry || sessionExpiry;

      // 检查 token 是否存在且未过期
      const isTokenValid = finalToken && finalExpiry && new Date().getTime() < parseInt(finalExpiry);

      const redirectTarget = isTokenValid ? '/home' : '/login';

      console.log('🔍 [ROUTER] Root redirect decision:', {
        hasLocalToken: !!token,
        hasSessionToken: !!sessionToken,
        hasExpiry: !!finalExpiry,
        rememberMe,
        isValid: isTokenValid,
        currentTime: new Date().getTime(),
        expiryTime: finalExpiry ? parseInt(finalExpiry) : null,
        redirectTo: redirectTarget
      });

      return redirectTarget;
    }
  },

  // 认证页面
  {
    path: '/login',
    name: 'Login',
    component: Login,
    meta: { requiresGuest: true }
  },
  {
    path: '/register',
    name: 'Register',
    component: Register,
    meta: { requiresGuest: true }
  },

  // 主应用布局 (Slack-like layout with sidebar)
  {
    path: '/home',
    name: 'Home',
    component: Home,
    meta: { requiresAuth: true },
    children: [
      // 欢迎页面 (默认右侧内容)
      {
        path: '',
        name: 'Welcome',
        component: () => import('../components/common/WelcomeContent.vue'),
        meta: { requiresAuth: true }
      },
      // 管理员页面 (嵌套在Home布局内)
      {
        path: '/admin',
        name: 'Admin',
        component: () => import('../components/admin/AdminDashboard.vue'),
        meta: { requiresAuth: true, requiresAdmin: true }
      }
    ]
  },

  // 🔧 CRITICAL FIX: 独立的聊天路由 (不嵌套在Home内)
  {
    path: '/chat/:id',
    name: 'Chat',
    component: Chat,
    meta: { requiresAuth: true }
  },

  // 独立功能页面 (不需要侧边栏)
  {
    path: '/demo',
    name: 'Demo',
    component: Demo
  },
  {
    path: '/demo/search',
    name: 'SearchDemo',
    component: () => import('@/views/PerfectSearchDemo.vue'),
    meta: { title: 'Perfect Search Demo' }
  },
  {
    path: '/test',
    name: 'Test',
    component: Test
  },
  {
    path: '/debug',
    name: 'Debug',
    component: Debug
  },
  {
    path: '/debug/protobuf-analytics',
    name: 'ProtobufAnalyticsTest',
    component: () => import('../components/debug/ProtobufAnalyticsTest.vue'),
    meta: { requiresAuth: false }
  },
  {
    path: '/simple-login',
    name: 'SimpleLogin',
    component: SimpleLogin
  },

  // 错误页面
  {
    path: '/error/:code',
    name: 'Error',
    component: Error,
    props: true
  },

  // 404 处理
  {
    path: '/:pathMatch(.*)*',
    redirect: '/error/404'
  }
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

// 路由初始化日志
if (import.meta.env.DEV) {
  console.log('🔍 [ROUTER] Router initialized with routes:', routes.length);
}

// Global navigation guard
// 存储导航开始时间
let navigationStartTime = 0;

// 🔧 PERFORMANCE: 认证初始化缓存
let authInitPromise = null;
let isAuthInitialized = false;

router.beforeEach(async (to, from, next) => {
  // 记录导航开始时间
  navigationStartTime = Date.now();

  if (import.meta.env.DEV) {
    console.log('🔍 [ROUTER] Navigation:', { from: from.path, to: to.path });
  }

  // 🔧 PERFORMANCE: 公开路由快速通道
  const publicRoutes = ['/login', '/register', '/demo', '/test', '/error', '/debug', '/simple-login'];
  const isPublicRoute = publicRoutes.some(route => to.path.startsWith(route));

  if (isPublicRoute) {
    if (import.meta.env.DEV) {
      console.log('🔍 [ROUTER] Public route, allowing access');
    }
    return next();
  }

  // 🔧 PERFORMANCE: 避免重复初始化认证
  const authStore = useAuthStore();

  if (!isAuthInitialized && !authInitPromise) {
    if (import.meta.env.DEV) {
      console.log('🔍 [ROUTER] Initializing auth store...');
    }

    authInitPromise = authStore.initialize()
      .then(() => {
        isAuthInitialized = true;
        if (import.meta.env.DEV) {
          console.log('🔍 [ROUTER] ✅ Auth store initialized');
        }
      })
      .catch(error => {
        if (import.meta.env.DEV) {
          console.error('🔍 [ROUTER] ❌ Auth store initialization failed:', error);
        }
        isAuthInitialized = false; // 允许重试
        throw error;
      })
      .finally(() => {
        authInitPromise = null; // 清理promise
      });
  }

  // 等待认证初始化（如果正在进行）
  if (authInitPromise) {
    try {
      await authInitPromise;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('🔍 [ROUTER] Auth initialization failed, redirecting to login');
      }
      return next('/login');
    }
  }

  // 🔧 SIMPLIFIED: 基本认证检查
  const requiresAuth = to.matched.some(record => record.meta.requiresAuth);

  if (requiresAuth) {
    // 🔧 CRITICAL FIX: Simplified auth check - remove complex consensus logic
    let authState = authStore.isAuthenticated;
    let hasToken = !!authStore.token;
    let hasUser = !!authStore.user;
    let isTokenExpired = authStore.isTokenExpired;

    // 🔧 SIMPLIFIED: Direct functional check - if we have token + user, accept it
    const hasFunctionalAuth = hasToken && hasUser && !isTokenExpired;

    // 🔧 FALLBACK: Check storage consistency for edge cases
    let hasStorageBackup = false;
    if (!hasFunctionalAuth) {
      try {
        const storageToken = localStorage.getItem('auth_token');
        const storageUser = localStorage.getItem('auth_user');
        hasStorageBackup = !!(storageToken && storageUser);
      } catch (error) {
        console.warn('🔍 [ROUTER] Storage check failed:', error);
      }
    }

    // 🔧 TOLERANT: Accept authentication if we have functional auth OR authStore says we're auth OR storage backup
    const isAuthenticated = hasFunctionalAuth || authState || hasStorageBackup;

    if (!isAuthenticated) {
      if (import.meta.env.DEV) {
        console.warn('🔍 [ROUTER] Access denied - redirecting to login');
        console.warn('🔍 [ROUTER] Auth state:', {
          authState,
          hasToken,
          hasUser,
          isTokenExpired,
          hasFunctionalAuth,
          hasStorageBackup,
          finalDecision: isAuthenticated,
          route: to.path
        });
      }

      // 🔧 ENHANCED: Prevent redirect loops and save target path
      if (to.path !== '/login') {
        sessionStorage.setItem('redirectPath', to.fullPath);
        return next('/login');
      } else {
        // Already on login page, allow access
        return next();
      }
    } else {
      if (import.meta.env.DEV) {
        console.log('✅ [ROUTER] Authentication verified successfully');
      }
    }
  }

  // 🔧 SIMPLIFIED: 管理员权限检查
  const requiresAdmin = to.matched.some(record => record.meta.requiresAdmin);
  if (requiresAdmin && !authStore.isAuthenticated) {
    if (import.meta.env.DEV) {
      console.warn('🔍 [ROUTER] Admin access denied - not authenticated');
    }
    return next('/login');
  }

  // 🔧 SIMPLIFIED: 访客路由检查
  const requiresGuest = to.matched.some(record => record.meta.requiresGuest);
  if (requiresGuest && authStore.isAuthenticated) {
    if (import.meta.env.DEV) {
      console.log('🔍 [ROUTER] Guest route but user is authenticated, redirecting to home');
    }
    return next('/home');
  }

  if (import.meta.env.DEV) {
    console.log('🔍 [ROUTER] ✅ Navigation allowed');
  }

  next();
});

// 导航完成后的处理
router.afterEach((to, from, failure) => {
  if (failure) {
    if (import.meta.env.DEV) {
      console.error('❌ Navigation failed:', failure);
    }
  } else {
    if (import.meta.env.DEV) {
      console.log('🔍 [ROUTER] Navigation completed:', { from: from.path, to: to.path });
    }

    // 跟踪导航事件
    if (navigationStartTime && from.path !== to.path) {
      analytics.trackNavigation(from.path, to.path, navigationStartTime).catch(err => {
        if (import.meta.env.DEV) {
          console.warn('Failed to track navigation:', err);
        }
      });
    }
  }
});

// 🔧 CRITICAL FIX: 简化错误处理，避免无限重定向
router.onError((error) => {
  if (import.meta.env.DEV) {
    console.error('🔍 [ROUTER] Router error:', error);
  }

  // 🔧 忽略常见的无害错误
  if (error.name === 'NavigationDuplicated' ||
    error.message?.includes('redundant navigation') ||
    error.message?.includes('Avoided redundant')) {
    console.log('ℹ️ [ROUTER] Ignoring redundant navigation error');
    return;
  }

  if (error.name === 'NavigationAborted' || error.name === 'NavigationCancelled') {
    console.log('ℹ️ [ROUTER] Navigation was cancelled or aborted');
    return;
  }

  // 🔧 只有在真正严重的错误时才重定向，并且避免循环
  if (!window.location.pathname.startsWith('/error/')) {
    console.error('❌ [ROUTER] Serious error, redirecting to error page:', error);
    router.push('/error/500').catch(() => {
      console.error('❌ [ROUTER] Failed to navigate to error page');
    });
  }
});

// Setup enhanced global router error handling
setupGlobalRouterErrorHandling(router);

export default router;