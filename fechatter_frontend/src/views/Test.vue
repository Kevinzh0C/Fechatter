<template>
  <div class="min-h-screen bg-gray-100 py-12 px-4">
    <div class="max-w-4xl mx-auto space-y-6">
      <!-- Original Test Content -->
      <div class="bg-white rounded-lg shadow-md p-6">
        <h1 class="text-2xl font-bold text-gray-900 mb-4">Fechatter Test Page</h1>
        <p class="text-gray-600 mb-4">If you can see this, Vue is working correctly!</p>

        <div class="space-y-2">
          <p class="text-sm text-gray-500">
            <strong>Status:</strong>
            <span class="text-green-600">✓ Vue is loaded</span>
          </p>
          <p class="text-sm text-gray-500">
            <strong>Router:</strong>
            <span class="text-green-600">✓ Routing is working</span>
          </p>
          <p class="text-sm text-gray-500">
            <strong>Tailwind:</strong>
            <span class="text-green-600">✓ Styles are applied</span>
          </p>
        </div>

        <div class="mt-6 space-y-2">
          <router-link to="/login"
            class="block w-full text-center bg-blue-500 text-white py-2 px-4 rounded hover:bg-blue-600">
            Go to Login
          </router-link>
          <router-link to="/register"
            class="block w-full text-center bg-gray-500 text-white py-2 px-4 rounded hover:bg-gray-600">
            Go to Register
          </router-link>
        </div>
      </div>

      <!-- Login Test Component -->
      <div class="bg-white rounded-lg shadow-md p-6">
        <LoginTest />
      </div>
    </div>
  </div>
</template>

<script setup>
import LoginTest from '@/components/LoginTest.vue';

console.log('Test component loaded successfully!');
</script>