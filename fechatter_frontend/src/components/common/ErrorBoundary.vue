<template>
  <div v-if="hasError" class="error-boundary">
    <div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
      <div class="max-w-md w-full space-y-8">
        <div class="text-center">
          <svg class="mx-auto h-16 w-16 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z"></path>
          </svg>
          <h2 class="mt-6 text-3xl font-extrabold text-gray-900">
            Something went wrong
          </h2>
          <p class="mt-2 text-sm text-gray-600">
            We're sorry, but something unexpected happened. Please try refreshing the page.
          </p>
          
          <!-- Error Details (only in development) -->
          <div v-if="isDevelopment && errorInfo" class="mt-4 p-4 bg-red-50 border border-red-200 rounded-md text-left">
            <h3 class="text-sm font-medium text-red-800 mb-2">Error Details:</h3>
            <pre class="text-xs text-red-700 whitespace-pre-wrap">{{ errorInfo }}</pre>
          </div>
        </div>
        
        <div class="space-y-3">
          <button @click="retry" 
            class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-purple-600 hover:bg-purple-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-500">
            Try Again
          </button>
          
          <button @click="goHome" 
            class="group relative w-full flex justify-center py-2 px-4 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-500">
            Go to Home
          </button>
          
          <button @click="reportError" 
            class="group relative w-full flex justify-center py-2 px-4 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-500">
            Report Issue
          </button>
        </div>
      </div>
    </div>
  </div>
  
  <slot v-else />
</template>

<script setup>
import { ref, onErrorCaptured } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();

const hasError = ref(false);
const errorInfo = ref('');
const isDevelopment = ref(import.meta.env.DEV);

onErrorCaptured((error, instance, info) => {
  console.error('Error caught by ErrorBoundary:', error);
  console.error('Component info:', info);
  
  hasError.value = true;
  errorInfo.value = `${error.message}\n\nStack trace:\n${error.stack}\n\nComponent info: ${info}`;
  
  // Report error to monitoring service in production
  if (!isDevelopment.value) {
    reportErrorToService(error, info);
  }
  
  return false; // Prevent the error from propagating further
});

function retry() {
  hasError.value = false;
  errorInfo.value = '';
  // Force component re-render
  window.location.reload();
}

function goHome() {
  hasError.value = false;
  errorInfo.value = '';
  router.push('/');
}

function reportError() {
  // In a real application, this would open a support ticket or feedback form
  const subject = encodeURIComponent('Fechatter Application Error');
  const body = encodeURIComponent(`I encountered an error in the Fechatter application.\n\nError details:\n${errorInfo.value}`);
  
  // Open email client (you could also use a support ticket system)
  window.open(`mailto:support@fechatter.com?subject=${subject}&body=${body}`);
}

function reportErrorToService(error, info) {
  // In a real application, you would send this to your error monitoring service
  // Examples: Sentry, Bugsnag, LogRocket, etc.
  
  const errorData = {
    message: error.message,
    stack: error.stack,
    componentInfo: info,
    url: window.location.href,
    userAgent: navigator.userAgent,
    timestamp: new Date().toISOString()
  };
  
  // Example: Send to monitoring service
  // fetch('/api/errors', {
  //   method: 'POST',
  //   headers: { 'Content-Type': 'application/json' },
  //   body: JSON.stringify(errorData)
  // }).catch(console.error);
  
  console.log('Error reported to monitoring service:', errorData);
}
</script>

<style scoped>
.error-boundary {
  /* Ensure error boundary takes full viewport */
  min-height: 100vh;
  width: 100%;
}
</style> 