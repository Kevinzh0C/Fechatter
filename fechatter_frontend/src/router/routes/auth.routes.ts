import type { AppRouteRecord } from '../types';

export const authRoutes: AppRouteRecord[] = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('../../views/Login.vue'),
    meta: { 
      title: 'Sign In',
      requiresGuest: true,
      layout: 'auth',
      hideInMenu: true,
    }
  },
  {
    path: '/register',
    name: 'Register',
    component: () => import('../../views/Register.vue'),
    meta: { 
      title: 'Sign Up',
      requiresGuest: true,
      layout: 'auth',
      hideInMenu: true,
    }
  },
];