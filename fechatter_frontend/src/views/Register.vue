<template>
  <div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
    <div class="max-w-md w-full space-y-8">
      <div>
        <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">Create your account</h2>
        <p class="mt-2 text-center text-sm text-gray-600">
          Join Fechatter and start collaborating
        </p>
      </div>
      <form class="mt-8 space-y-6" @submit.prevent="handleSubmit">
        <div class="rounded-md shadow-sm -space-y-px">
          <div>
            <label for="fullname" class="sr-only">Full Name</label>
            <input 
              v-model="fullname" 
              id="fullname" 
              name="fullname" 
              type="text" 
              required
              :disabled="loading"
              class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-t-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm disabled:opacity-50"
              placeholder="Full Name" />
          </div>
          <div>
            <label for="email" class="sr-only">Email address</label>
            <input 
              v-model="email" 
              id="email" 
              name="email" 
              type="email" 
              required
              :disabled="loading"
              class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm disabled:opacity-50"
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
              :disabled="loading"
              minlength="6"
              class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm disabled:opacity-50"
              placeholder="Password (minimum 6 characters)" />
          </div>
          <div>
            <label for="workspace" class="sr-only">Workspace</label>
            <input 
              v-model="workspace" 
              id="workspace" 
              name="workspace" 
              type="text" 
              required
              :disabled="loading"
              class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-b-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm disabled:opacity-50"
              placeholder="Workspace Name" />
          </div>
        </div>

        <div v-if="error" class="rounded-md bg-red-50 p-4">
          <div class="flex">
            <div class="ml-3">
              <h3 class="text-sm font-medium text-red-800">
                {{ error }}
              </h3>
            </div>
          </div>
        </div>

        <div>
          <button 
            type="submit"
            :disabled="loading || !isFormValid"
            class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed">
            <span v-if="loading">Creating account...</span>
            <span v-else>Sign up</span>
          </button>
        </div>
      </form>
      
      <div class="text-center">
        <router-link to="/login" class="text-indigo-600 hover:text-indigo-500">
          Already have an account? Sign in
        </router-link>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';

const router = useRouter();
const authStore = useAuthStore();

const fullname = ref('');
const email = ref('');
const password = ref('');
const workspace = ref('');
const error = ref('');
const loading = ref(false);

const isFormValid = computed(() => {
  return fullname.value.trim().length > 0 &&
         email.value.trim().length > 0 &&
         password.value.length >= 6 &&
         workspace.value.trim().length > 0;
});

async function handleSubmit() {
  if (!isFormValid.value) {
    error.value = 'Please fill in all fields correctly';
    return;
  }

  error.value = '';
  loading.value = true;
  
  try {
    const success = await authStore.register(
      fullname.value,
      email.value,
      password.value,
      workspace.value
    );
    
    if (success) {
      // Wait for router to be ready before navigation
      await router.isReady();
      await router.push('/');
    } else {
      error.value = authStore.error || 'Registration failed';
    }
  } catch (err) {
    error.value = err.message || 'Registration failed';
  } finally {
    loading.value = false;
  }
}
</script>