import { createRouter, createWebHistory } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import Home from '../views/Home.vue';
import Login from '../views/Login.vue';
import Register from '../views/Register.vue';
import Chat from '../views/Chat.vue';
import Demo from '../views/Demo.vue';

const routes = [
  {
    path: '/',
    name: 'Home',
    component: Home,
    meta: { requiresAuth: true }
  },
  {
    path: '/login',
    name: 'Login',
    component: Login,
    meta: { requiresGuest: true }
  },
  {
    path: '/register',
    name: 'Register',
    component: Register,
    meta: { requiresGuest: true }
  },
  {
    path: '/chat/:id',
    name: 'Chat',
    component: Chat,
    meta: { requiresAuth: true }
  },
  {
    path: '/demo',
    name: 'Demo',
    component: Demo
  }
];

const router = createRouter({
  history: createWebHistory(),
  routes
});

// Track initialization state more carefully
let authInitialized = false;
let initializationInProgress = false;

router.beforeEach(async (to, from, next) => {
  try {
    // Prevent multiple simultaneous initializations
    if (!authInitialized && !initializationInProgress) {
      initializationInProgress = true;
      
      const authStore = useAuthStore();
      
      // Initialize auth store only once with proper error handling
      try {
        await authStore.initializeAuth();
        authInitialized = true;
        console.log('Auth initialized successfully');
      } catch (error) {
        console.error('Auth initialization failed:', error);
        authInitialized = true; // Still mark as initialized to prevent loops
      } finally {
        initializationInProgress = false;
      }
    }
    
    // Wait for initialization to complete if in progress
    if (initializationInProgress) {
      let attempts = 0;
      while (initializationInProgress && attempts < 50) { // Max 5 seconds
        await new Promise(resolve => setTimeout(resolve, 100));
        attempts++;
      }
    }

    const authStore = useAuthStore();
    const isAuthenticated = authStore.isLoggedIn;
    
    // Enhanced debug logging
    console.log('Route Guard:', {
      to: to.path,
      from: from.path,
      isAuthenticated,
      hasToken: !!authStore.token,
      hasRefreshToken: !!authStore.refreshToken,
      requiresAuth: to.meta.requiresAuth,
      requiresGuest: to.meta.requiresGuest
    });
    
    // Handle routes that require authentication
    if (to.meta.requiresAuth) {
      if (!isAuthenticated) {
        console.log('Redirecting to login - authentication required');
        return next('/login');
      }
    }
    
    // Handle routes that require guest (not authenticated)
    if (to.meta.requiresGuest) {
      if (isAuthenticated) {
        console.log('Redirecting to home - already authenticated');
        return next('/');
      }
    }
    
    // Allow navigation
    console.log('Navigation allowed to:', to.path);
    next();
    
  } catch (error) {
    console.error('Router guard error:', error);
    
    // On error, handle gracefully
    if (to.path !== '/login' && to.meta.requiresAuth) {
      console.log('Error occurred, redirecting to login');
      next('/login');
    } else {
      console.log('Error occurred, allowing navigation');
      next();
    }
  }
});

export default router; 