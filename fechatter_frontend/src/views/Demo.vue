<template>
  <div class="min-h-screen bg-gray-50">
    <!-- Simple Header -->
    <div class="bg-white shadow-sm border-b border-gray-200">
      <div class="max-w-4xl mx-auto px-4 py-3">
        <div class="flex items-center gap-2">
          <AppIcon 
            :size="24" 
            :preserve-gradient="true"
            title="Fechatter Demo"
          />
          <h1 class="text-lg font-semibold text-gray-900">Demo Page - 演示页面</h1>
        </div>
      </div>
    </div>

    <div class="max-w-4xl mx-auto p-4">
      <!-- Basic Status Card -->
      <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-200 mb-6">
        <h2 class="text-xl font-semibold text-gray-900 mb-4">Application Status</h2>
        <div class="space-y-3">
          <div class="flex items-center space-x-2">
            <svg class="w-5 h-5 text-green-500" viewBox="0 0 24 24" fill="currentColor">
              <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
            </svg>
            <span>Vue app mounted successfully</span>
          </div>
          <div class="flex items-center space-x-2">
            <svg class="w-5 h-5 text-green-500" viewBox="0 0 24 24" fill="currentColor">
              <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
            </svg>
            <span>Router working</span>
          </div>
          <div class="flex items-center space-x-2">
            <svg class="w-5 h-5 text-green-500" viewBox="0 0 24 24" fill="currentColor">
              <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
            </svg>
            <span>JavaScript router implementation</span>
          </div>
        </div>
      </div>

      <!-- Navigation Links -->
      <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-200 mb-6">
        <h2 class="text-xl font-semibold text-gray-900 mb-4">Navigation Test</h2>
        <div class="grid grid-cols-2 md:grid-cols-3 gap-3">
          <router-link to="/login" 
                       class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 text-center">
            Login
          </router-link>
          <router-link to="/register" 
                       class="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600 text-center">
            Register
          </router-link>
          <router-link to="/home" 
                       class="px-4 py-2 bg-purple-500 text-white rounded hover:bg-purple-600 text-center">
            Home
          </router-link>
          <router-link to="/test" 
                       class="px-4 py-2 bg-orange-500 text-white rounded hover:bg-orange-600 text-center">
            Test
          </router-link>
          <router-link to="/error/404" 
                       class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600 text-center">
            404 Error
          </router-link>
          <router-link to="/error/500" 
                       class="px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600 text-center">
            500 Error
          </router-link>
        </div>
      </div>

      <!-- Simple Functionality Test -->
      <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
        <h2 class="text-xl font-semibold text-gray-900 mb-4">Basic Functionality</h2>
        <div class="space-y-4">
          <button @click="showMessage" 
                  class="px-4 py-2 bg-indigo-500 text-white rounded hover:bg-indigo-600">
            Show Alert
          </button>
          <div v-if="message" class="p-3 bg-green-100 border border-green-400 text-green-700 rounded">
            {{ message }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import { AppIcon } from '@/components/icons';

const message = ref('');

function showMessage() {
  message.value = `Hello! Current time: ${new Date().toLocaleTimeString()}`;
  setTimeout(() => {
    message.value = '';
  }, 3000);
}
</script> 