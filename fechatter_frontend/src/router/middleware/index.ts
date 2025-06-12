import type { RouterMiddleware } from '../types';
import { authMiddleware } from './auth';
import { permissionsMiddleware } from './permissions';
import { loggerMiddleware } from './logger';
import { progressMiddleware } from './progress';
import { titleMiddleware } from './title';

export const defaultMiddleware: RouterMiddleware[] = [
  loggerMiddleware,      // 1. 日志记录
  progressMiddleware,    // 2. 进度条
  titleMiddleware,       // 3. 页面标题
  authMiddleware,        // 4. 认证检查
  permissionsMiddleware, // 5. 权限检查
];

export {
  authMiddleware,
  permissionsMiddleware,
  loggerMiddleware,
  progressMiddleware,
  titleMiddleware,
};