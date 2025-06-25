import type { AppRouteRecord } from '../types';

export const errorRoutes: AppRouteRecord[] = [
  {
    path: '/error/:code',
    name: 'Error',
    component: () => import('../../views/Error.vue'),
    props: true,
    meta: { 
      title: 'Error',
      hideInMenu: true,
    }
  },
  {
    path: '/:pathMatch(.*)*',
    name: 'NotFound',
    redirect: '/error/404'
  },
];