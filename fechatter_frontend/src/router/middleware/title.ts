import type { RouterMiddleware } from '../types';

const DEFAULT_TITLE = 'Fechatter';

export const titleMiddleware: RouterMiddleware = async (to, _from, next) => {
  // 设置页面标题
  const title = to.meta?.title;
  if (title) {
    document.title = `${title} - ${DEFAULT_TITLE}`;
  } else {
    document.title = DEFAULT_TITLE;
  }

  next();
};