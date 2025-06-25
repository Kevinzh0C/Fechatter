import { createRouter, createWebHistory } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import { analytics } from '../lib/analytics-protobuf';
import { setupGlobalRouterErrorHandling } from '@/composables/useRouterGuard';

// Lazy loaded components - 只导入实际存在的组件
const Home = () => import('@/views/Home.vue');
const Login = () => import('@/views/Login.vue');
const Register = () => import('@/views/Register.vue');
const Chat = () => import('@/views/Chat.vue');
const Demo = () => import('@/views/Demo.vue');
const Test = () => import('@/views/Test.vue');
const Error = () => import('@/views/Error.vue');
const Debug = () => import('@/views/Debug.vue');
const SimpleLogin = () => import('@/views/SimpleLogin.vue');
const SSEPerformanceTest = () => import('@/views/SSEPerformanceTest.vue');

const routes = [
  // 根路径重定向
  {
    path: '/',
    redirect: (to) => {
      console.log('🔍 [ROUTER] Root redirect triggered, checking auth state...');

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
      // 聊天页面 (嵌套在Home布局内)
      {
        path: '/chat/:id',
        name: 'Chat',
        component: Chat,
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

  // 独立功能页面 (不需要侧边栏)
  {
    path: '/demo',
    name: 'Demo',
    component: Demo
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

  {
    path: '/sse-performance-test',
    name: 'SSEPerformanceTest',
    component: SSEPerformanceTest,
    meta: {
      requiresAuth: false,  // Allow access without authentication for testing
      title: 'SSE Performance Test'
    }
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
console.log('🔍 [ROUTER] Router initialized with routes:', routes.length);

// Global navigation guard
// 存储导航开始时间
let navigationStartTime = 0;

router.beforeEach(async (to, from, next) => {
  // 记录导航开始时间
  navigationStartTime = Date.now();

  console.log('🔍 [ROUTER] Navigation:', { from: from.path, to: to.path });

  // Public routes that don't require authentication
  const publicRoutes = ['/login', '/register', '/demo', '/test', '/error', '/debug', '/simple-login'];
  const isPublicRoute = publicRoutes.some(route => to.path.startsWith(route));

  if (isPublicRoute) {
    console.log('🔍 [ROUTER] Public route, allowing access');
    return next();
  }

  // Initialize auth store if not already done
  const authStore = useAuthStore();
  if (!authStore.isInitialized) {
    console.log('🔍 [ROUTER] Initializing auth store...');
    try {
      await authStore.initialize();
      console.log('🔍 [ROUTER] Auth store initialized, current state:', {
        isAuthenticated: authStore.isAuthenticated,
        hasToken: !!authStore.token,
        hasUser: !!authStore.user,
        isTokenExpired: authStore.isTokenExpired
      });
    } catch (error) {
      console.error('🔍 [ROUTER] Auth store initialization failed:', error);
      // If initialization fails, redirect to login
      return next('/login');
    }
  }

  // Check if route requires authentication
  const requiresAuth = to.matched.some(record => record.meta.requiresAuth);

  if (requiresAuth) {
    // Check authentication for protected routes
    if (!authStore.isLoggedIn || authStore.isTokenExpired) {
      console.warn('🔍 [ROUTER] Access denied - redirecting to login');
      console.warn('🔍 [ROUTER] Auth state:', {
        isLoggedIn: authStore.isLoggedIn,
        isAuthenticated: authStore.isAuthenticated,
        hasToken: !!authStore.token,
        hasUser: !!authStore.user,
        isTokenExpired: authStore.isTokenExpired
      });

      // Store the intended path for redirect after login
      if (to.path !== '/login') {
        sessionStorage.setItem('redirectPath', to.fullPath);
      }

      return next('/login');
    }
  }

  // Check if route requires admin privileges
  const requiresAdmin = to.matched.some(record => record.meta.requiresAdmin);

  if (requiresAdmin) {
    // For now, allow all authenticated users to access admin features
    // In a real application, you would check the user's role/permissions
    if (!authStore.isAuthenticated) {
      console.warn('🔍 [ROUTER] Admin access denied - not authenticated');
      return next('/login');
    }

    // TODO: Add proper role-based access control
    // const isAdmin = authStore.user?.role === 'admin' || authStore.user?.permissions?.includes('admin');
    // if (!isAdmin) {
    //   console.warn('🔍 [ROUTER] Admin access denied - insufficient permissions');
    //   return next('/home');
    // }
  }

  // Check if route requires guest (not authenticated)
  const requiresGuest = to.matched.some(record => record.meta.requiresGuest);

  if (requiresGuest && authStore.isAuthenticated) {
    console.log('🔍 [ROUTER] Guest route but user is authenticated, redirecting to home');
    return next('/home');
  }

  console.log('🔍 [ROUTER] Navigation allowed');
  next();
});

// 导航完成后的处理
router.afterEach((to, from, failure) => {
  if (failure) {
    console.error('Navigation failed:', failure);
  } else {
    if (import.meta.env.VITE_DEBUG === 'true') {
    }

    // 跟踪导航事件
    if (navigationStartTime && from.path !== to.path) {
      analytics.trackNavigation(from.path, to.path, navigationStartTime).catch(err => {
        console.warn('Failed to track navigation:', err);
      });
    }
  }
});

// 全局错误处理
router.onError((error) => {
  console.error('Router error:', error);
  // 避免无限重定向
  if (window.location.pathname !== '/error/500') {
    router.push('/error/500');
  }
});

// Setup enhanced global router error handling
setupGlobalRouterErrorHandling(router);

export default router;