import { 
  createRouter, 
  createWebHistory, 
  createMemoryHistory,
  type Router,
  type NavigationGuardNext,
  type RouteLocationNormalized
} from 'vue-router';
import type { RouterConfig, RouterMiddleware } from './types';
import { routes } from './routes';
import { defaultMiddleware } from './middleware';
import { setupRouterAfterHooks, SessionRecovery, RouterPerformance } from './utils';

/**
 * 创建路由器实例
 */
export function createAppRouter(config: RouterConfig = {}): Router {
  const {
    baseUrl = import.meta.env.BASE_URL,
    enableProgress = true,
    enableDebug = import.meta.env.VITE_DEBUG === 'true',
    middleware = defaultMiddleware,
  } = config;

  // 根据环境选择历史模式
  const history = typeof window !== 'undefined' 
    ? createWebHistory(baseUrl)
    : createMemoryHistory();

  const router = createRouter({
    history,
    routes,
    scrollBehavior(to, from, savedPosition) {
      // 恢复滚动位置
      if (savedPosition) {
        return savedPosition;
      }
      
      // 锚点跳转
      if (to.hash) {
        return { el: to.hash };
      }
      
      // 顶部滚动
      return { top: 0 };
    },
  });

  // 应用中间件管道
  if (middleware.length > 0) {
    router.beforeEach(async (to, from, next) => {
      RouterPerformance.startNavigation();
      
      try {
        // 执行中间件链
        await executeMiddlewareChain(middleware, to, from, next);
      } catch (error) {
        console.error('Middleware execution failed:', error);
        next('/error/500');
      }
    });
  }

  // 设置后置钩子
  setupRouterAfterHooks(router);

  return router;
}

/**
 * 执行中间件链
 */
async function executeMiddlewareChain(
  middleware: RouterMiddleware[],
  to: RouteLocationNormalized,
  from: RouteLocationNormalized,
  next: NavigationGuardNext
): Promise<void> {
  let index = 0;

  const dispatch = async (): Promise<void> => {
    if (index >= middleware.length) {
      return next();
    }

    const middlewareFunc = middleware[index++];
    
    // 创建next函数，支持中间件链
    const nextFunc: NavigationGuardNext = (to?: any) => {
      if (to === false || typeof to === 'string' || (to && typeof to === 'object')) {
        // 中间件想要重定向或阻止导航
        return next(to);
      }
      
      // 继续执行下一个中间件
      return dispatch();
    };

    await middlewareFunc(to, from, nextFunc);
  };

  await dispatch();
}

/**
 * 创建测试路由器
 */
export function createTestRouter(initialRoute: string = '/'): Router {
  const router = createAppRouter({
    enableProgress: false,
    enableDebug: false,
    middleware: [], // 测试时不使用中间件
  });

  // 设置初始路由
  router.push(initialRoute);
  
  return router;
}

/**
 * 路由会话恢复
 */
export async function restoreSession(router: Router): Promise<void> {
  await SessionRecovery.execute(router);
}

// 创建默认路由器实例
const router = createAppRouter();

// 导出默认实例
export default router;

// 导出工具函数
export { SessionRecovery, RouterPerformance };
export type { RouterConfig, RouterMiddleware } from './types';