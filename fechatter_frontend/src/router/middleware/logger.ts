import type { RouterMiddleware, NavigationContext } from '../types';
import { v4 as uuidv4 } from 'uuid';

const isDebugEnabled = import.meta.env.VITE_DEBUG === 'true';

export const loggerMiddleware: RouterMiddleware = async (to, from, next) => {
  if (!isDebugEnabled) {
    return next();
  }

  const traceId = uuidv4();
  const timestamp = Date.now();

  const context: NavigationContext = {
    traceId,
    timestamp,
    to,
    from,
  };

  // 导航开始日志
  console.group(`🚀 [${traceId}] Route Navigation`);
  console.log('From:', from.fullPath);
  console.log('To:', to.fullPath);
  console.log('Meta:', to.meta);
  console.log('Params:', to.params);
  console.log('Query:', to.query);
  // 保存上下文到路由实例
  (to as any).navigationContext = context;

  next();
};