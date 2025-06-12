<template>
  <div class="login-test p-4">
    <h2 class="text-xl font-bold mb-4">Login Test Component</h2>
    
    <div class="space-y-4">
      <input 
        v-model="email" 
        type="email" 
        placeholder="Email"
        class="w-full p-2 border rounded"
      />
      
      <input 
        v-model="password" 
        type="password" 
        placeholder="Password"
        class="w-full p-2 border rounded"
      />
      
      <div class="flex gap-2 flex-wrap">
        <button 
          @click="testDirectAPI" 
          class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        >
          Test Direct API
        </button>
        
        <button 
          @click="testWithAuthStore" 
          class="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600"
        >
          Test Auth Store
        </button>
        
        <button 
          @click="fillTestCredentials" 
          class="px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600"
        >
          Fill Test Data
        </button>
      </div>
      
      <div v-if="result" class="mt-4 p-4 rounded" :class="resultClass">
        <h3 class="font-bold">Result:</h3>
        <pre class="text-sm overflow-auto">{{ result }}</pre>
      </div>
      
      <div v-if="logs.length > 0" class="mt-4">
        <h3 class="font-bold">Logs:</h3>
        <div class="bg-gray-100 p-2 rounded max-h-64 overflow-y-auto">
          <div v-for="(log, index) in logs" :key="index" class="text-xs font-mono">
            [{{ log.time }}] {{ log.message }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import { useAuthStore } from '@/stores/auth';
import axios from 'axios';

const authStore = useAuthStore();

const email = ref('admin@test.com');
const password = ref('super123');
const result = ref('');
const resultClass = ref('');
const logs = ref([]);

function log(message) {
  const time = new Date().toTimeString().split(' ')[0];
  logs.value.push({ time, message });
  console.log(`[LoginTest] ${message}`);
}

function fillTestCredentials() {
  email.value = 'admin@test.com';
  password.value = 'super123';
  log('Filled test credentials');
}

async function testDirectAPI() {
  logs.value = [];
  log('Starting direct API test');
  
  try {
    log(`Making request to: http://127.0.0.1:8080/api/signin`);
    
    const response = await axios.post('http://127.0.0.1:8080/api/signin', {
      email: email.value,
      password: password.value
    }, {
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json'
      },
      timeout: 10000
    });
    
    log('Request successful!');
    result.value = JSON.stringify(response.data, null, 2);
    resultClass.value = 'bg-green-100 border border-green-300';
  } catch (error) {
    log(`Error: ${error.message}`);
    log(`Error code: ${error.code}`);
    log(`Response status: ${error.response?.status}`);
    
    result.value = JSON.stringify({
      message: error.message,
      code: error.code,
      response: error.response?.data,
      config: {
        url: error.config?.url,
        method: error.config?.method,
        baseURL: error.config?.baseURL
      }
    }, null, 2);
    resultClass.value = 'bg-red-100 border border-red-300';
  }
}

async function testWithAuthStore() {
  logs.value = [];
  log('Starting auth store test');
  
  try {
    log('Calling authStore.login()');
    const success = await authStore.login(email.value, password.value);
    
    if (success) {
      log('Login successful via auth store!');
      result.value = JSON.stringify({
        success: true,
        user: authStore.user,
        token: authStore.tokens?.accessToken ? 'Present' : 'Missing'
      }, null, 2);
      resultClass.value = 'bg-green-100 border border-green-300';
    } else {
      log('Login failed via auth store');
      result.value = JSON.stringify({
        success: false,
        error: authStore.error
      }, null, 2);
      resultClass.value = 'bg-red-100 border border-red-300';
    }
  } catch (error) {
    log(`Exception: ${error.message}`);
    result.value = JSON.stringify({
      exception: error.message,
      error: authStore.error
    }, null, 2);
    resultClass.value = 'bg-red-100 border border-red-300';
  }
}
</script>