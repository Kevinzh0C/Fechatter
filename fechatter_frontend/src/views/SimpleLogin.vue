<template>
  <div class="simple-login">
    <h1>Simple Login Page</h1>
    <form @submit.prevent="handleLogin">
      <div>
        <label>Email:</label>
        <input v-model="email" type="email" required />
      </div>
      <div>
        <label>Password:</label>
        <input v-model="password" type="password" required />
      </div>
      <button type="submit" :disabled="loading">
        {{ loading ? 'Logging in...' : 'Login' }}
      </button>
    </form>
    <div v-if="error" style="color: red;">{{ error }}</div>
    
    <hr />
    
    <div>
      <h3>Auth Store State:</h3>
      <pre>{{ authState }}</pre>
    </div>
    
    <div>
      <h3>Actions:</h3>
      <button @click="clearAndGoToDebug">Clear Storage & Go to Debug</button>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';

const router = useRouter();
const authStore = useAuthStore();

const email = ref('admin@fechatter.com');
const password = ref('Admin123!@#');
const loading = ref(false);
const error = ref('');

const authState = computed(() => ({
  isAuthenticated: authStore.isAuthenticated,
  isInitialized: authStore.isInitialized,
  isLoggedIn: authStore.isLoggedIn,
  hasToken: !!authStore.token,
  user: authStore.user
}));

const handleLogin = async () => {
  loading.value = true;
  error.value = '';
  
  try {
    console.log('Attempting login...');
    const success = await authStore.login(email.value, password.value, true);
    
    if (success) {
      console.log('Login successful, redirecting to home...');
      router.push('/home');
    } else {
      error.value = authStore.error || 'Login failed';
    }
  } catch (err) {
    console.error('Login error:', err);
    error.value = err.message || 'Login failed';
  } finally {
    loading.value = false;
  }
};

const clearAndGoToDebug = () => {
  localStorage.clear();
  sessionStorage.clear();
  router.push('/debug');
};
</script>

<style scoped>
.simple-login {
  padding: 20px;
  max-width: 600px;
  margin: 0 auto;
}

form {
  margin: 20px 0;
}

div {
  margin: 10px 0;
}

input {
  width: 100%;
  padding: 8px;
  margin: 5px 0;
  border: 1px solid #ddd;
  border-radius: 4px;
}

button {
  padding: 10px 20px;
  background: #007bff;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

button:hover {
  background: #0056b3;
}

button:disabled {
  background: #ccc;
  cursor: not-allowed;
}

pre {
  background: #f5f5f5;
  padding: 10px;
  border-radius: 4px;
  overflow: auto;
}
</style>