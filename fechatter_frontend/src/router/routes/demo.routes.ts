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
      hideInMenu: import.meta.env.PROD, // 生产环境隐藏
    }
  },
];