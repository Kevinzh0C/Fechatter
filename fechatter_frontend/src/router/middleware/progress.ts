import type { RouterMiddleware } from '../types';
import NProgress from 'nprogress';
import 'nprogress/nprogress.css';

// 配置NProgress
NProgress.configure({
  showSpinner: false,
  minimum: 0.1,
  easing: 'ease',
  speed: 500,
  trickleSpeed: 200,
});

// 自定义样式
const style = document.createElement('style');
style.textContent = `
  #nprogress .bar {
    background: #6366f1 !important;
    height: 3px !important;
  }
  #nprogress .peg {
    box-shadow: 0 0 10px #6366f1, 0 0 5px #6366f1 !important;
  }
`;
document.head.appendChild(style);

export const progressMiddleware: RouterMiddleware = async (_to, _from, next) => {
  // 开始进度条
  NProgress.start();

  next();
};