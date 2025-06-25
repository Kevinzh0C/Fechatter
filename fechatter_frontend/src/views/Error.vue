<template>
  <div class="min-h-screen bg-gray-50 flex flex-col justify-center py-12 sm:px-6 lg:px-8">
    <div class="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
      <div class="bg-white py-8 px-4 shadow sm:rounded-lg sm:px-10 text-center">
        <div class="mb-6">
          <div class="mx-auto h-24 w-24 bg-red-100 rounded-full flex items-center justify-center">
            <svg class="h-12 w-12 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path v-if="errorCode === '404'" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                    d="M9.172 16.172a4 4 0 015.656 0M9 12h6m-6-4h6m2 5.291A7.962 7.962 0 0112 15c-2.34 0-4.467-.881-6.072-2.33" />
              <path v-else-if="errorCode === '403'" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                    d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
              <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16c-.77.833.192 2.5 1.732 2.5z" />
            </svg>
          </div>
        </div>
        
        <h2 class="text-2xl font-bold text-gray-900 mb-2">
          {{ errorTitle }}
        </h2>
        
        <p class="text-gray-600 mb-6">
          {{ errorMessage }}
        </p>
        
        <div class="space-y-3">
          <button 
            @click="goBack" 
            class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
          >
            Go Back
          </button>
          
          <router-link 
            to="/demo" 
            class="w-full flex justify-center py-2 px-4 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
          >
            Go to Demo
          </router-link>
        </div>
        
        <!-- Debug info (development only) -->
        <div v-if="showDebugInfo" class="mt-6 p-4 bg-gray-100 rounded text-left text-xs">
          <h4 class="font-medium text-gray-700 mb-2">Debug Information:</h4>
          <pre class="text-gray-600">{{ debugInfo }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { useRouter, useRoute } from 'vue-router';

export default {
  name: 'Error',
  props: {
    code: {
      type: String,
      default: null
    }
  },
  setup(props) {
    const router = useRouter();
    const route = useRoute();
    
    return {
      router,
      route
    };
  },
  computed: {
    errorCode() {
      return this.code || this.route.params.code || '500';
    },
    showDebugInfo() {
      return import.meta.env.DEV;
    },
    errorTitle() {
      switch (this.errorCode) {
        case '403':
          return 'Access Denied';
        case '404':
          return 'Page Not Found';
        case '500':
          return 'Server Error';
        default:
          return 'Error';
      }
    },
    errorMessage() {
      switch (this.errorCode) {
        case '403':
          return "You don't have permission to access this resource.";
        case '404':
          return "The page you're looking for doesn't exist or has been moved.";
        case '500':
          return 'Something went wrong on our end. Please try again later.';
        default:
          return 'An unexpected error occurred.';
      }
    },
    debugInfo() {
      return {
        code: this.errorCode,
        route: this.route.fullPath,
        timestamp: new Date().toISOString(),
        userAgent: navigator.userAgent,
        referrer: document.referrer || 'direct'
      };
    }
  },
  methods: {
    goBack() {
      if (window.history.length > 1) {
        this.router.go(-1);
      } else {
        this.router.push('/demo');
      }
    }
  }
};
</script>