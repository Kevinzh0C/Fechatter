import { createRouter, createWebHistory } from 'vue-router';
import Home from '../views/Home.vue';
import Login from '../views/Login.vue';
import Register from '../views/Register.vue';
import Chat from '../views/Chat.vue';
import Demo from '../views/Demo.vue';
import Test from '../views/Test.vue';

// 简化的路由，直接导入组件，无懒加载，无复杂逻辑
const routes = [
  {
    path: '/',
    redirect: '/demo' // 暂时跳转到demo避免认证问题
  },
  {
    path: '/home',
    name: 'Home',
    component: Home,
  },
  {
    path: '/login',
    name: 'Login',
    component: Login,
  },
  {
    path: '/register',
    name: 'Register',
    component: Register,
  },
  {
    path: '/chat/:id',
    name: 'Chat',
    component: Chat,
  },
  {
    path: '/demo',
    name: 'Demo',
    component: Demo,
  },
  {
    path: '/test',
    name: 'Test',
    component: Test,
  },
  {
    path: '/error/:code?',
    name: 'Error',
    component: () => {
      // 简单的错误组件，避免复杂的Error.vue
      return {
        template: `
          <div style="padding: 20px; text-align: center;">
            <h1>Error {{ $route.params.code || '500' }}</h1>
            <p>Something went wrong</p>
            <button @click="$router.push('/demo')">Go to Demo</button>
          </div>
        `
      };
    }
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/error/404'
  }
];

const router = createRouter({
  history: createWebHistory(),
  routes
});

// 简化的路由守卫，只打印日志
router.beforeEach((to, from, next) => {
  // 设置简单的标题
  document.title = `${to.name || 'Page'} - Fechatter`;
  
  next();
});

router.afterEach((to, from) => {
  });

router.onError((error) => {
  if (import.meta.env.DEV) {
    console.error('Router error:', error);
  }
  // 不要重定向，避免循环
});

export default router;