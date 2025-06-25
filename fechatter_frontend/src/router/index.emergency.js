import { createRouter, createWebHistory } from 'vue-router';

// 紧急简化版本 - 无认证检查
const routes = [
  {
    path: '/',
    redirect: '/demo' // 直接跳转到demo，避免认证循环
  },
  {
    path: '/demo',
    name: 'Demo',
    component: () => import('../views/Demo.vue'),
  },
  {
    path: '/home',
    name: 'Home',
    component: () => import('../views/Home.vue'),
  },
  {
    path: '/login',
    name: 'Login',
    component: () => import('../views/Login.vue'),
  },
  {
    path: '/test',
    name: 'Test',
    component: () => import('../views/Test.vue'),
  },
  {
    path: '/error/:code',
    name: 'Error',
    component: () => import('../views/Error.vue'),
    props: true,
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/demo'
  }
];

const router = createRouter({
  history: createWebHistory(),
  routes
});

export default router;