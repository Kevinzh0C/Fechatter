import type { AppRouteRecord } from '../types';

export const demoRoutes: AppRouteRecord[] = [
  {
    path: '/demo',
    name: 'Demo',
    component: () => import('../../views/Demo.vue'),
    meta: {
      title: 'Feature Demo',
      icon: '🎮',
      order: 90,
    }
  },
  {
    path: '/test',
    name: 'Test',
    component: () => import('../../views/Test.vue'),
    meta: {
      title: 'Test Page',
      icon: '🧪',
      order: 99,
      hideInMenu: false, // 开发环境显示
    }
  },
  {
    path: '/code-highlight-test',
    name: 'CodeHighlightTest',
    component: () => import('../../components/demo/CodeHighlightTest.vue'),
    meta: {
      title: 'Code Highlight Test',
      icon: '🎨',
      order: 91,
      hideInMenu: false, // 开发环境显示
    }
  },
];