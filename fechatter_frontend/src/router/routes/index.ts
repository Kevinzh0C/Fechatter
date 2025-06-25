import type { AppRouteRecord } from '../types';
import { authRoutes } from './auth.routes';
import { appRoutes } from './app.routes';
import { demoRoutes } from './demo.routes';
import { errorRoutes } from './error.routes';

// 根路由重定向
export const rootRoute: AppRouteRecord = {
  path: '/',
  redirect: '/login'
};

// 合并所有路由模块
export const routes: AppRouteRecord[] = [
  rootRoute,
  ...authRoutes,
  ...appRoutes,
  ...demoRoutes,
  ...errorRoutes,
];

// 导出路由模块供按需使用
export {
  authRoutes,
  appRoutes,
  demoRoutes,
  errorRoutes,
};