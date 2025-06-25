<template>
  <div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
    <div class="max-w-md w-full space-y-8">
      <div>
        <div class="flex justify-center mb-4">
          <AppIcon 
            :size="64" 
            :preserve-gradient="true"
            start-color="#6366f1"
            end-color="#8b5cf6"
            title="Fechatter Logo"
          />
        </div>
        <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">Create your account</h2>
        <p class="mt-2 text-center text-sm text-gray-600">
          Join Fechatter and start collaborating
        </p>
      </div>

      <!-- Error Display -->
      <AuthError 
        :error="error" 
        title="Registration Error"
        :suggestion="errorSuggestion"
        @dismiss="dismissError"
        dismissible
      />

      <form 
        class="mt-8 space-y-6" 
        @submit.prevent="handleSubmit"
        data-testid="register-form"
      >
        <div class="rounded-md shadow-sm -space-y-px">
          <div>
            <label for="fullname" class="sr-only">Full Name</label>
            <input 
              v-model="fullname" 
              id="fullname" 
              name="fullname" 
              type="text" 
              required
              autocomplete="name"
              :disabled="loading || isSubmitting"
              class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-t-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm disabled:opacity-50 disabled:cursor-not-allowed"
              placeholder="Full Name"
              data-testid="fullname-input"
            />
          </div>
          <div>
            <label for="email" class="sr-only">Email address</label>
            <input 
              v-model="email" 
              id="email" 
              name="email" 
              type="email" 
              required
              autocomplete="email"
              :disabled="loading || isSubmitting"
              class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm disabled:opacity-50 disabled:cursor-not-allowed"
              placeholder="Email address"
              data-testid="email-input"
            />
          </div>
          <div>
            <label for="password" class="sr-only">Password</label>
            <input 
              v-model="password" 
              id="password" 
              name="password" 
              type="password" 
              required
              autocomplete="new-password"
              :disabled="loading || isSubmitting"
              minlength="8"
              @input="validatePassword"
              :class="[
                'appearance-none rounded-none relative block w-full px-3 py-2 border placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm disabled:opacity-50 disabled:cursor-not-allowed',
                passwordError ? 'border-red-300' : 'border-gray-300'
              ]"
              placeholder="Password (8+ chars with number)"
              data-testid="password-input"
            />
            <p v-if="passwordError" class="mt-1 text-xs text-red-600">
              {{ passwordError }}
            </p>
          </div>
          <div>
            <label for="workspace" class="sr-only">Workspace Name</label>
            <input 
              v-model="workspace" 
              id="workspace" 
              name="workspace" 
              type="text" 
              required
              autocomplete="organization"
              :disabled="loading || isSubmitting"
              class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-b-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm disabled:opacity-50 disabled:cursor-not-allowed"
              placeholder="Workspace Name"
              data-testid="workspace-input"
            />
          </div>
        </div>

        <div class="flex items-center justify-between">
          <div class="text-sm">
            <router-link 
              to="/login" 
              class="font-medium text-indigo-600 hover:text-indigo-500"
              data-testid="login-link"
            >
              Already have an account? Sign in
            </router-link>
          </div>
        </div>

        <div>
          <button 
            type="submit"
            :disabled="loading || !isFormValid || isSubmitting"
            class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
            data-testid="register-submit"
          >
            <span v-if="loading || isSubmitting" class="absolute left-0 inset-y-0 flex items-center pl-3">
              <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
            </span>
            {{ (loading || isSubmitting) ? 'Creating account...' : 'Sign up' }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import { AppIcon } from '@/components/icons';
// import { useRegisterGuard } from '@/composables/useRegisterGuard';
import AuthError from '@/components/common/AuthError.vue';


const router = useRouter();
const authStore = useAuthStore();
// const { startRegisterGuard, stopRegisterGuard } = useRegisterGuard();

// Form data
const fullname = ref('');
const email = ref('');
const password = ref('');
const workspace = ref('');
const error = ref('');
const errorSuggestion = ref('');
const passwordError = ref('');
const loading = ref(false);
const isSubmitting = ref(false);

// Password validation
const validatePassword = () => {
  const pwd = password.value;
  if (pwd.length === 0) {
    passwordError.value = '';
  } else if (pwd.length < 8) {
    passwordError.value = 'Password must be at least 8 characters';
  } else if (!/\d/.test(pwd)) {
    passwordError.value = 'Password must contain at least one number';
  } else {
    passwordError.value = '';
  }
};

// Form validation
const isFormValid = computed(() => {
  return fullname.value.trim().length > 0 &&
         email.value.trim().length > 0 &&
         password.value.length >= 8 &&
         /\d/.test(password.value) &&
         workspace.value.trim().length > 0 &&
         !passwordError.value;
});

// CSRF token handling
const getCsrfToken = () => {
  const metaToken = document.querySelector('meta[name="csrf-token"]');
  return metaToken?.content || null;
};

const setCsrfToken = (token) => {
  // Update meta tag if exists, or create new one
  let metaToken = document.querySelector('meta[name="csrf-token"]');
  if (!metaToken) {
    metaToken = document.createElement('meta');
    metaToken.name = 'csrf-token';
    document.head.appendChild(metaToken);
  }
  metaToken.content = token;
};

const dismissError = () => {
  error.value = '';
  errorSuggestion.value = '';
};

const handleSubmit = async () => {
  // Prevent duplicate submissions
  if (!isFormValid.value || loading.value || isSubmitting.value) {
    return;
  }

  error.value = '';
  errorSuggestion.value = '';
  isSubmitting.value = true;
  loading.value = true;
  
  // Start navigation guard
  // startRegisterGuard();
  
  try {
    // Get CSRF token if available
    const csrfToken = getCsrfToken();
    if (csrfToken) {
      console.log('CSRF token detected for registration');
    }
    
    const success = await authStore.register(
      fullname.value.trim(),
      email.value.trim(),
      password.value,
      workspace.value.trim()
    );
    
    if (success) {
      console.log('Registration successful');
      
      // Handle CSRF token from response if provided
      // This would typically be set by the auth store after successful registration
      if (authStore.csrfToken) {
        setCsrfToken(authStore.csrfToken);
      }
      
      // Auto-login is handled by the auth store
      // Small delay for smooth transition
      await new Promise(resolve => setTimeout(resolve, 100));
      
      // Navigate to onboarding or home
      await router.push('/onboarding').catch(() => {
        // Fallback to home if onboarding route doesn't exist
        router.push('/');
      });
    } else {
      error.value = authStore.error || 'Registration failed';
      errorSuggestion.value = getErrorSuggestion(error.value);
    }
  } catch (err) {
    console.error('Registration error:', err);
    error.value = err.message || 'Registration failed';
    errorSuggestion.value = getErrorSuggestion(error.value);
  } finally {
    loading.value = false;
    isSubmitting.value = false;
    // stopRegisterGuard();
  }
};

const getErrorSuggestion = (errorMessage) => {
  if (errorMessage.toLowerCase().includes('email')) {
    return 'Please check that the email address is valid and not already in use.';
  } else if (errorMessage.toLowerCase().includes('password')) {
    return 'Ensure your password meets all requirements: 8+ characters with at least one number.';
  } else if (errorMessage.toLowerCase().includes('workspace')) {
    return 'The workspace name may already be taken. Try a different name.';
  }
  return 'Please check your information and try again.';
};

onMounted(() => {
  console.log('Register page mounted');
  
  // Clear any existing errors
  error.value = '';
  authStore.error = null;
});

onUnmounted(() => {
  stopRegisterGuard();
  console.log('Register page unmounted');
});
</script>