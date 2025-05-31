<template>
  <div class="min-h-screen bg-gray-100">
    <!-- Header -->
    <div class="bg-white shadow">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex justify-between items-center py-6">
          <div>
            <h1 class="text-3xl font-bold text-gray-900">API Test Suite</h1>
            <p class="mt-1 text-sm text-gray-500">Test all Fechatter API endpoints</p>
          </div>
          <div class="flex items-center space-x-4">
            <router-link to="/" 
              class="px-4 py-2 bg-purple-600 text-white rounded hover:bg-purple-700">
              Back to Chat
            </router-link>
            <button @click="runAllTests" 
                    :disabled="runningAllTests"
                    class="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 disabled:opacity-50">
              {{ runningAllTests ? 'Running...' : 'Run All Tests' }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Test Results Summary -->
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
      <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
        <div class="bg-white rounded-lg shadow p-6">
          <div class="flex items-center">
            <div class="flex-shrink-0">
              <svg class="h-8 w-8 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 00-2 2"></path>
              </svg>
            </div>
            <div class="ml-5 w-0 flex-1">
              <dl>
                <dt class="text-sm font-medium text-gray-500 truncate">Total Tests</dt>
                <dd class="text-lg font-medium text-gray-900">{{ totalTests }}</dd>
              </dl>
            </div>
          </div>
        </div>

        <div class="bg-white rounded-lg shadow p-6">
          <div class="flex items-center">
            <div class="flex-shrink-0">
              <svg class="h-8 w-8 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
              </svg>
            </div>
            <div class="ml-5 w-0 flex-1">
              <dl>
                <dt class="text-sm font-medium text-gray-500 truncate">Passed</dt>
                <dd class="text-lg font-medium text-gray-900">{{ passedTests }}</dd>
              </dl>
            </div>
          </div>
        </div>

        <div class="bg-white rounded-lg shadow p-6">
          <div class="flex items-center">
            <div class="flex-shrink-0">
              <svg class="h-8 w-8 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
              </svg>
            </div>
            <div class="ml-5 w-0 flex-1">
              <dl>
                <dt class="text-sm font-medium text-gray-500 truncate">Failed</dt>
                <dd class="text-lg font-medium text-gray-900">{{ failedTests }}</dd>
              </dl>
            </div>
          </div>
        </div>

        <div class="bg-white rounded-lg shadow p-6">
          <div class="flex items-center">
            <div class="flex-shrink-0">
              <svg class="h-8 w-8 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"></path>
              </svg>
            </div>
            <div class="ml-5 w-0 flex-1">
              <dl>
                <dt class="text-sm font-medium text-gray-500 truncate">Running</dt>
                <dd class="text-lg font-medium text-gray-900">{{ runningTests }}</dd>
              </dl>
            </div>
          </div>
        </div>
      </div>

      <!-- Test Categories -->
      <div class="space-y-6">
        <!-- Authentication Tests -->
        <div class="bg-white shadow rounded-lg">
          <div class="px-6 py-4 border-b border-gray-200">
            <h3 class="text-lg font-medium text-gray-900">Authentication Tests</h3>
          </div>
          <div class="p-6">
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              <TestButton 
                v-for="test in authTests" 
                :key="test.name"
                :test="test"
                @run="runTest" />
            </div>
          </div>
        </div>

        <!-- User Management Tests -->
        <div class="bg-white shadow rounded-lg">
          <div class="px-6 py-4 border-b border-gray-200">
            <h3 class="text-lg font-medium text-gray-900">User Management Tests</h3>
          </div>
          <div class="p-6">
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              <TestButton 
                v-for="test in userTests" 
                :key="test.name"
                :test="test"
                @run="runTest" />
            </div>
          </div>
        </div>

        <!-- Chat Management Tests -->
        <div class="bg-white shadow rounded-lg">
          <div class="px-6 py-4 border-b border-gray-200">
            <h3 class="text-lg font-medium text-gray-900">Chat Management Tests</h3>
          </div>
          <div class="p-6">
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              <TestButton 
                v-for="test in chatTests" 
                :key="test.name"
                :test="test"
                @run="runTest" />
            </div>
          </div>
        </div>

        <!-- Message Tests -->
        <div class="bg-white shadow rounded-lg">
          <div class="px-6 py-4 border-b border-gray-200">
            <h3 class="text-lg font-medium text-gray-900">Message Tests</h3>
          </div>
          <div class="p-6">
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              <TestButton 
                v-for="test in messageTests" 
                :key="test.name"
                :test="test"
                @run="runTest" />
            </div>
          </div>
        </div>

        <!-- File Management Tests -->
        <div class="bg-white shadow rounded-lg">
          <div class="px-6 py-4 border-b border-gray-200">
            <h3 class="text-lg font-medium text-gray-900">File Management Tests</h3>
          </div>
          <div class="p-6">
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              <TestButton 
                v-for="test in fileTests" 
                :key="test.name"
                :test="test"
                @run="runTest" />
            </div>
          </div>
        </div>

        <!-- Member Management Tests -->
        <div class="bg-white shadow rounded-lg">
          <div class="px-6 py-4 border-b border-gray-200">
            <h3 class="text-lg font-medium text-gray-900">Member Management Tests</h3>
          </div>
          <div class="p-6">
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              <TestButton 
                v-for="test in memberTests" 
                :key="test.name"
                :test="test"
                @run="runTest" />
            </div>
          </div>
        </div>
      </div>

      <!-- Test Results -->
      <div v-if="testResults.length > 0" class="mt-8 bg-white shadow rounded-lg">
        <div class="px-6 py-4 border-b border-gray-200 flex justify-between items-center">
          <h3 class="text-lg font-medium text-gray-900">Test Results</h3>
          <button @click="clearResults" class="text-sm text-gray-500 hover:text-gray-700">
            Clear Results
          </button>
        </div>
        <div class="max-h-96 overflow-y-auto">
          <div v-for="result in testResults" :key="result.id" 
               class="px-6 py-4 border-b border-gray-100 last:border-b-0">
            <div class="flex items-start justify-between">
              <div class="flex-1">
                <div class="flex items-center">
                  <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium"
                        :class="result.status === 'passed' 
                          ? 'bg-green-100 text-green-800' 
                          : result.status === 'failed' 
                          ? 'bg-red-100 text-red-800' 
                          : 'bg-yellow-100 text-yellow-800'">
                    {{ result.status }}
                  </span>
                  <span class="ml-2 font-medium text-gray-900">{{ result.name }}</span>
                  <span class="ml-2 text-sm text-gray-500">{{ result.duration }}ms</span>
                </div>
                <div v-if="result.error" class="mt-2 text-sm text-red-600">
                  {{ result.error }}
                </div>
                <div v-if="result.response" class="mt-2">
                  <details class="text-sm">
                    <summary class="cursor-pointer text-gray-600 hover:text-gray-800">
                      Response Data
                    </summary>
                    <pre class="mt-2 p-2 bg-gray-50 rounded text-xs overflow-x-auto">{{ JSON.stringify(result.response, null, 2) }}</pre>
                  </details>
                </div>
              </div>
              <div class="text-sm text-gray-500">
                {{ formatTime(result.timestamp) }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
import axios from 'axios';
import TestButton from '../components/TestButton.vue';

// Test state
const testResults = ref([]);
const runningAllTests = ref(false);
const testTokens = ref({
  accessToken: null,
  refreshToken: null,
  testChatId: null,
  testUserId: null,
  uploadedFiles: []
});

// Computed properties
const totalTests = computed(() => {
  return authTests.value.length + userTests.value.length + chatTests.value.length + 
         messageTests.value.length + fileTests.value.length + memberTests.value.length;
});

const passedTests = computed(() => {
  return testResults.value.filter(r => r.status === 'passed').length;
});

const failedTests = computed(() => {
  return testResults.value.filter(r => r.status === 'failed').length;
});

const runningTests = computed(() => {
  return testResults.value.filter(r => r.status === 'running').length;
});

// Test definitions
const authTests = ref([
  {
    name: 'User Signup',
    status: 'pending',
    async run() {
      const response = await axios.post('/api/signup', {
        fullname: 'Test User',
        email: 'testuser@example.com',
        password: 'password123',
        workspace: 'Test Workspace'
      });
      testTokens.value.accessToken = response.data.access_token;
      testTokens.value.refreshToken = response.data.refresh_token;
      return response.data;
    }
  },
  {
    name: 'User Signin',
    status: 'pending',
    async run() {
      const response = await axios.post('/api/signin', {
        email: 'testuser@example.com',
        password: 'password123'
      });
      testTokens.value.accessToken = response.data.access_token;
      testTokens.value.refreshToken = response.data.refresh_token;
      return response.data;
    }
  },
  {
    name: 'Token Refresh',
    status: 'pending',
    async run() {
      if (!testTokens.value.refreshToken) {
        throw new Error('No refresh token available');
      }
      const response = await axios.post('/api/refresh', {}, {
        headers: { Authorization: `Bearer ${testTokens.value.refreshToken}` }
      });
      testTokens.value.accessToken = response.data.access_token;
      testTokens.value.refreshToken = response.data.refresh_token;
      return response.data;
    }
  },
  {
    name: 'User Logout',
    status: 'pending',
    async run() {
      if (!testTokens.value.accessToken) {
        throw new Error('No access token available');
      }
      const response = await axios.post('/api/logout', {}, {
        headers: { Authorization: `Bearer ${testTokens.value.accessToken}` }
      });
      return response.data;
    }
  },
  {
    name: 'Logout All Sessions',
    status: 'pending',
    async run() {
      if (!testTokens.value.accessToken) {
        throw new Error('No access token available');
      }
      const response = await axios.post('/api/logout-all', {}, {
        headers: { Authorization: `Bearer ${testTokens.value.accessToken}` }
      });
      return response.data;
    }
  }
]);

const userTests = ref([
  {
    name: 'List Users',
    status: 'pending',
    async run() {
      if (!testTokens.value.accessToken) {
        throw new Error('No access token available');
      }
      const response = await axios.get('/api/users', {
        headers: { Authorization: `Bearer ${testTokens.value.accessToken}` }
      });
      return response.data;
    }
  }
]);

const chatTests = ref([
  {
    name: 'List Chats',
    status: 'pending',
    async run() {
      if (!testTokens.value.accessToken) {
        throw new Error('No access token available');
      }
      const response = await axios.get('/api/chat', {
        headers: { Authorization: `Bearer ${testTokens.value.accessToken}` }
      });
      return response.data;
    }
  },
  {
    name: 'Create Chat',
    status: 'pending',
    async run() {
      if (!testTokens.value.accessToken) {
        throw new Error('No access token available');
      }
      const response = await axios.post('/api/chat', {
        name: 'Test Chat',
        chat_type: 'PrivateChannel',
        members: [],
        description: 'A test chat for API testing'
      }, {
        headers: { Authorization: `Bearer ${testTokens.value.accessToken}` }
      });
      testTokens.value.testChatId = response.data.id;
      return response.data;
    }
  },
  {
    name: 'Update Chat',
    status: 'pending',
    async run() {
      if (!testTokens.value.accessToken || !testTokens.value.testChatId) {
        throw new Error('No access token or chat ID available');
      }
      const response = await axios.patch(`/api/chat/${testTokens.value.testChatId}`, {
        name: 'Updated Test Chat',
        description: 'Updated description for test chat'
      }, {
        headers: { Authorization: `Bearer ${testTokens.value.accessToken}` }
      });
      return response.data;
    }
  },
  {
    name: 'Delete Chat',
    status: 'pending',
    async run() {
      if (!testTokens.value.accessToken || !testTokens.value.testChatId) {
        throw new Error('No access token or chat ID available');
      }
      const response = await axios.delete(`/api/chat/${testTokens.value.testChatId}`, {
        headers: { Authorization: `Bearer ${testTokens.value.accessToken}` }
      });
      return response.data;
    }
  }
]);

const messageTests = ref([
  {
    name: 'Send Message',
    status: 'pending',
    async run() {
      if (!testTokens.value.accessToken || !testTokens.value.testChatId) {
        throw new Error('No access token or chat ID available');
      }
      const response = await axios.post(`/api/chat/${testTokens.value.testChatId}/messages`, {
        content: 'Hello, this is a test message!',
        files: []
      }, {
        headers: { Authorization: `Bearer ${testTokens.value.accessToken}` }
      });
      return response.data;
    }
  },
  {
    name: 'Send Message with Idempotency',
    status: 'pending',
    async run() {
      if (!testTokens.value.accessToken || !testTokens.value.testChatId) {
        throw new Error('No access token or chat ID available');
      }
      const idempotencyKey = '01970000-0000-7000-0000-000000000001';
      const response = await axios.post(`/api/chat/${testTokens.value.testChatId}/messages`, {
        content: 'Idempotent test message',
        files: [],
        idempotency_key: idempotencyKey
      }, {
        headers: { Authorization: `Bearer ${testTokens.value.accessToken}` }
      });
      return response.data;
    }
  },
  {
    name: 'List Messages',
    status: 'pending',
    async run() {
      if (!testTokens.value.accessToken || !testTokens.value.testChatId) {
        throw new Error('No access token or chat ID available');
      }
      const response = await axios.get(`/api/chat/${testTokens.value.testChatId}/messages`, {
        headers: { Authorization: `Bearer ${testTokens.value.accessToken}` }
      });
      return response.data;
    }
  },
  {
    name: 'List Messages with Pagination',
    status: 'pending',
    async run() {
      if (!testTokens.value.accessToken || !testTokens.value.testChatId) {
        throw new Error('No access token or chat ID available');
      }
      const response = await axios.get(`/api/chat/${testTokens.value.testChatId}/messages?limit=5`, {
        headers: { Authorization: `Bearer ${testTokens.value.accessToken}` }
      });
      return response.data;
    }
  }
]);

const fileTests = ref([
  {
    name: 'Upload File',
    status: 'pending',
    async run() {
      if (!testTokens.value.accessToken) {
        throw new Error('No access token available');
      }
      
      // Create a test file
      const testContent = 'This is a test file content for API testing';
      const blob = new Blob([testContent], { type: 'text/plain' });
      const file = new File([blob], 'test.txt', { type: 'text/plain' });
      
      const formData = new FormData();
      formData.append('files', file);
      
      const response = await axios.post('/api/upload', formData, {
        headers: { 
          Authorization: `Bearer ${testTokens.value.accessToken}`,
          'Content-Type': 'multipart/form-data'
        }
      });
      
      testTokens.value.uploadedFiles = response.data;
      return response.data;
    }
  },
  {
    name: 'Send Message with File',
    status: 'pending',
    async run() {
      if (!testTokens.value.accessToken || !testTokens.value.testChatId || !testTokens.value.uploadedFiles.length) {
        throw new Error('No access token, chat ID, or uploaded files available');
      }
      const response = await axios.post(`/api/chat/${testTokens.value.testChatId}/messages`, {
        content: 'Message with file attachment',
        files: testTokens.value.uploadedFiles
      }, {
        headers: { Authorization: `Bearer ${testTokens.value.accessToken}` }
      });
      return response.data;
    }
  },
  {
    name: 'Access File',
    status: 'pending',
    async run() {
      if (!testTokens.value.accessToken || !testTokens.value.uploadedFiles.length) {
        throw new Error('No access token or uploaded files available');
      }
      const fileUrl = testTokens.value.uploadedFiles[0];
      const response = await axios.get(`/api${fileUrl}`, {
        headers: { Authorization: `Bearer ${testTokens.value.accessToken}` }
      });
      return { status: 'File accessed successfully', size: response.data.length };
    }
  }
]);

const memberTests = ref([
  {
    name: 'List Chat Members',
    status: 'pending',
    async run() {
      if (!testTokens.value.accessToken || !testTokens.value.testChatId) {
        throw new Error('No access token or chat ID available');
      }
      const response = await axios.get(`/api/chat/${testTokens.value.testChatId}/members`, {
        headers: { Authorization: `Bearer ${testTokens.value.accessToken}` }
      });
      return response.data;
    }
  },
  {
    name: 'Add Chat Members',
    status: 'pending',
    async run() {
      if (!testTokens.value.accessToken || !testTokens.value.testChatId) {
        throw new Error('No access token or chat ID available');
      }
      // This test might fail if there are no other users to add
      const response = await axios.post(`/api/chat/${testTokens.value.testChatId}/members`, [2, 3], {
        headers: { Authorization: `Bearer ${testTokens.value.accessToken}` }
      });
      return response.data;
    }
  },
  {
    name: 'Remove Chat Members',
    status: 'pending',
    async run() {
      if (!testTokens.value.accessToken || !testTokens.value.testChatId) {
        throw new Error('No access token or chat ID available');
      }
      const response = await axios.delete(`/api/chat/${testTokens.value.testChatId}/members`, {
        data: [3],
        headers: { Authorization: `Bearer ${testTokens.value.accessToken}` }
      });
      return response.data;
    }
  }
]);

// Methods
async function runTest(test) {
  const startTime = Date.now();
  const resultId = Date.now() + Math.random();
  
  // Add running test result
  testResults.value.unshift({
    id: resultId,
    name: test.name,
    status: 'running',
    timestamp: new Date(),
    duration: 0
  });
  
  test.status = 'running';
  
  try {
    const response = await test.run();
    const duration = Date.now() - startTime;
    
    // Update result
    const resultIndex = testResults.value.findIndex(r => r.id === resultId);
    if (resultIndex !== -1) {
      testResults.value[resultIndex] = {
        id: resultId,
        name: test.name,
        status: 'passed',
        timestamp: new Date(),
        duration,
        response
      };
    }
    
    test.status = 'passed';
  } catch (error) {
    const duration = Date.now() - startTime;
    
    // Update result
    const resultIndex = testResults.value.findIndex(r => r.id === resultId);
    if (resultIndex !== -1) {
      testResults.value[resultIndex] = {
        id: resultId,
        name: test.name,
        status: 'failed',
        timestamp: new Date(),
        duration,
        error: error.response?.data?.message || error.message
      };
    }
    
    test.status = 'failed';
  }
}

async function runAllTests() {
  runningAllTests.value = true;
  
  try {
    // Run tests in sequence to maintain dependencies
    const allTests = [
      ...authTests.value.slice(0, 2), // Signup and Signin
      ...userTests.value,
      ...chatTests.value.slice(0, 2), // List and Create chat
      ...messageTests.value,
      ...fileTests.value,
      ...memberTests.value,
      ...authTests.value.slice(2), // Token refresh and logout tests
      ...chatTests.value.slice(2) // Update and Delete chat
    ];
    
    for (const test of allTests) {
      await runTest(test);
      // Small delay between tests
      await new Promise(resolve => setTimeout(resolve, 500));
    }
  } finally {
    runningAllTests.value = false;
  }
}

function clearResults() {
  testResults.value = [];
  // Reset all test statuses
  [...authTests.value, ...userTests.value, ...chatTests.value, ...messageTests.value, ...fileTests.value, ...memberTests.value]
    .forEach(test => test.status = 'pending');
}

function formatTime(date) {
  return date.toLocaleTimeString();
}

onMounted(() => {
  // Auto-run signin if we have stored credentials
  const storedToken = localStorage.getItem('auth_token');
  if (storedToken) {
    testTokens.value.accessToken = storedToken;
  }
});
</script> 