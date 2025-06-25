import type { AppRouteRecord } from '../types';

export const appRoutes: AppRouteRecord[] = [
  {
    path: '/home',
    name: 'Home',
    component: () => import('../../views/Home.vue'),
    meta: { 
      title: 'Home',
      requiresAuth: true,
      icon: '🏠',
      order: 1,
    }
  },
  {
    path: '/chat/:id',
    name: 'Chat',
    component: () => import('../../views/Chat.vue'),
    meta: { 
      title: 'Chat',
      requiresAuth: true,
      keepAlive: true,
      hideInMenu: true,
    }
  },
];