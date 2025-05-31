<template>
  <div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
    <div class="max-w-md w-full space-y-8">
      <div>
        <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">Sign in to Fechatter</h2>
        <p class="mt-2 text-center text-sm text-gray-600">
          Welcome back! Please sign in to continue.
        </p>
      </div>
      
      <!-- Error Display -->
      <div v-if="authStore.error" class="rounded-md bg-red-50 p-4">
        <div class="flex">
          <div class="flex-shrink-0">
            <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
            </svg>
          </div>
          <div class="ml-3">
            <h3 class="text-sm font-medium text-red-800">Login Error</h3>
            <p class="mt-1 text-sm text-red-700">{{ authStore.error }}</p>
          </div>
        </div>
      </div>

      <form class="mt-8 space-y-6" @submit.prevent="handleSubmit">
        <div class="rounded-md shadow-sm -space-y-px">
          <div>
            <label for="email" class="sr-only">Email address</label>
            <input 
              v-model="email" 
              id="email" 
              name="email" 
              type="email" 
              required
              :disabled="authStore.loading"
              class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-t-md focus:outline-none focus:ring-purple-500 focus:border-purple-500 focus:z-10 sm:text-sm"
              placeholder="Email address" />
          </div>
          <div>
            <label for="password" class="sr-only">Password</label>
            <input 
              v-model="password" 
              id="password" 
              name="password" 
              type="password" 
              required
              :disabled="authStore.loading"
              class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-b-md focus:outline-none focus:ring-purple-500 focus:border-purple-500 focus:z-10 sm:text-sm"
              placeholder="Password" />
          </div>
        </div>

        <div class="flex items-center justify-between">
          <div class="text-sm">
            <router-link to="/register" class="font-medium text-purple-600 hover:text-purple-500">
              Don't have an account? Sign up
            </router-link>
          </div>
          <div class="text-sm">
            <button type="button" @click="fillTestCredentials" 
              class="font-medium text-gray-600 hover:text-gray-500">
              Use test account
            </button>
          </div>
        </div>

        <div>
          <button type="submit" 
            :disabled="authStore.loading || !email || !password"
            class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-purple-600 hover:bg-purple-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-500 disabled:opacity-50 disabled:cursor-not-allowed">
            <span v-if="authStore.loading" class="absolute left-0 inset-y-0 flex items-center pl-3">
              <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
            </span>
            {{ authStore.loading ? 'Signing in...' : 'Sign in' }}
          </button>
        </div>

        <!-- Demo credentials info -->
        <div class="mt-4 p-3 bg-blue-50 rounded-md">
          <h4 class="text-sm font-medium text-blue-800 mb-2">Test Accounts:</h4>
          <div class="text-xs text-blue-700 space-y-1">
            <p><strong>Super User:</strong> super@test.com / super123</p>
            <p><strong>Test User:</strong> testuser@example.com / password123</p>
          </div>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';

const router = useRouter();
const authStore = useAuthStore();

const email = ref('');
const password = ref('');
const mounted = ref(false);

// Prevent navigation while on login page
let preventNavigation = false;

const fillTestCredentials = () => {
  email.value = 'super@test.com';
  password.value = 'super123';
};

const handleSubmit = async () => {
  if (!email.value || !password.value || authStore.loading) {
    return;
  }

  console.log('Login attempt:', { email: email.value });
  
  try {
    preventNavigation = true;
    const success = await authStore.login(email.value, password.value);
    
    if (success) {
      console.log('Login successful, navigating to home');
      await router.push('/');
    } else {
      console.log('Login failed:', authStore.error);
    }
  } catch (error) {
    console.error('Login error:', error);
  } finally {
    preventNavigation = false;
  }
};

onMounted(() => {
  mounted.value = true;
  console.log('Login page mounted');
  
  // Clear any existing auth errors
  authStore.error = null;
  
  // Auto-fill for development
  if (import.meta.env.DEV) {
    email.value = 'super@test.com';
    password.value = 'super123';
  }
});

onUnmounted(() => {
  mounted.value = false;
  console.log('Login page unmounted');
});
</script>