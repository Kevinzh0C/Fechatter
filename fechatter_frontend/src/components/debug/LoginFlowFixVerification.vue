<!--
  LoginFlowFixVerification.vue - Login flow fix verification
-->
<template>
  <div class="login-fix-verification">
    <div class="verification-header">
      <h3>Login Flow Fix</h3>
      <button @click="visible = !visible" class="toggle-btn">
        {{ visible ? 'Hide' : 'Show' }}
      </button>
    </div>
    
    <div v-if="visible" class="verification-content">
      <!-- Auth State -->
      <div class="section">
        <h4>Auth State</h4>
        <div class="state-grid">
          <div class="state-item">
            <span>Auth Store:</span>
            <span :class="authStore.isAuthenticated ? 'success' : 'error'">
              {{ authStore.isAuthenticated ? '' : '❌' }}
            </span>
          </div>
          <div class="state-item">
            <span>User Data:</span>
            <span :class="authStore.user ? 'success' : 'error'">
              {{ authStore.user ? '' : '❌' }}
            </span>
          </div>
        </div>
      </div>
      
      <!-- Fix Status -->
      <div class="section">
        <h4>Fix Status</h4>
        <div class="fix-list">
          <div class="fix-item">Simplified auth checking</div>
          <div class="fix-item">Added nextTick() delay</div>
          <div class="fix-item">Enhanced error handling</div>
          <div class="fix-item">Trust router guard</div>
        </div>
      </div>
      
      <!-- Test Button -->
      <div class="section">
        <button @click="runTest" class="test-btn">Run Flow Test</button>
      </div>
      
      <!-- Test Results -->
      <div v-if="testResult" class="section">
        <h4>Test Result</h4>
        <div :class="['result', testResult.success ? 'success' : 'error']">
          {{ testResult.message }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { useRoute } from 'vue-router'

const authStore = useAuthStore()
const route = useRoute()

const visible = ref(false)
const testResult = ref(null)

const runTest = () => {
  const isAuthValid = authStore.isAuthenticated && authStore.user
  const isOnCorrectRoute = route.path === '/home'
  
  const success = isAuthValid && isOnCorrectRoute
  
  testResult.value = {
    success,
    message: success 
      ? 'Login flow working correctly!' 
      : `Issues: Auth=${isAuthValid}, Route=${isOnCorrectRoute}`
  }
}
</script>

<style scoped>
.login-fix-verification {
  position: fixed;
  top: 120px;
  right: 10px;
  width: 350px;
  background: white;
  border: 1px solid #ddd;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.1);
  z-index: 13000;
  font-size: 13px;
}

.verification-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: #007AFF;
  color: white;
  border-radius: 8px 8px 0 0;
}

.toggle-btn {
  padding: 4px 8px;
  background: rgba(255,255,255,0.2);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.verification-content {
  padding: 16px;
}

.section {
  margin-bottom: 16px;
}

.section h4 {
  margin: 0 0 8px 0;
  font-size: 14px;
}

.state-grid {
  display: grid;
  gap: 6px;
}

.state-item {
  display: flex;
  justify-content: space-between;
  padding: 6px 10px;
  background: #f8f9fa;
  border-radius: 4px;
}

.success {
  color: #28a745;
}

.error {
  color: #dc3545;
}

.fix-list {
  display: grid;
  gap: 4px;
}

.fix-item {
  font-size: 12px;
  color: #666;
}

.test-btn {
  width: 100%;
  padding: 8px 16px;
  background: #007AFF;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.result {
  padding: 8px 12px;
  border-radius: 4px;
  font-weight: 500;
}

.result.success {
  background: rgba(40, 167, 69, 0.1);
  color: #28a745;
}

.result.error {
  background: rgba(220, 53, 69, 0.1);
  color: #dc3545;
}
</style> 