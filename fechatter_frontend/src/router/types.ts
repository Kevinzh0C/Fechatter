import type { RouteRecordRaw, NavigationGuardNext, RouteLocationNormalized } from 'vue-router';

// 扩展路由meta类型
export interface RouteMeta {
  title?: string;
  requiresAuth?: boolean;
  requiresGuest?: boolean;
  roles?: string[];
  permissions?: string[];
  layout?: 'default' | 'auth' | 'chat';
  keepAlive?: boolean;
  hideInMenu?: boolean;
  icon?: string;
  order?: number;
}

// 扩展路由记录类型
export interface AppRouteRecord extends Omit<RouteRecordRaw, 'meta' | 'children'> {
  meta?: RouteMeta;
  children?: AppRouteRecord[];
}

// 中间件函数类型
export interface RouterMiddleware {
  (to: RouteLocationNormalized, from: RouteLocationNormalized, next: NavigationGuardNext): Promise<void> | void;
}

// 路由配置选项
export interface RouterConfig {
  baseUrl?: string;
  enableProgress?: boolean;
  enableDebug?: boolean;
  middleware?: RouterMiddleware[];
}

// 导航上下文
export interface NavigationContext {
  traceId: string;
  timestamp: number;
  to: RouteLocationNormalized;
  from: RouteLocationNormalized;
  user?: any;
  permissions?: string[];
}

// 错误类型
export interface RouterError {
  code: number;
  message: string;
  originalError?: Error;
  context?: NavigationContext;
}