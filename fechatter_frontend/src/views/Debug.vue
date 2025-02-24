<template>
  <div class="debug-container">
    <h1>Debug Information</h1>

    <section>
      <h2>Current Route</h2>
      <pre>{{ currentRoute }}</pre>
    </section>

    <section>
      <h2>Auth Store State</h2>
      <pre>{{ authState }}</pre>
    </section>

    <section>
      <h2>Local Storage</h2>
      <pre>{{ localStorageData }}</pre>
    </section>

    <section>
      <h2>Session Storage</h2>
      <pre>{{ sessionStorageData }}</pre>
    </section>

    <section>
      <h2>Actions</h2>
      <button @click="clearStorage">Clear All Storage</button>
      <button @click="goToLogin">Go to Login</button>
      <button @click="goToHome">Go to Home</button>
      <button @click="refreshAuth">Refresh Auth</button>
    </section>
  </div>
</template>

<script setup>
import { computed, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();

const currentRoute = computed(() => ({
  path: route.path,
  name: route.name,
  params: route.params,
  query: route.query,
  meta: route.meta
}));

const authState = computed(() => ({
  isAuthenticated: authStore.isAuthenticated,
  isInitialized: authStore.isInitialized,
  isLoggedIn: authStore.isLoggedIn,
  isTokenExpired: authStore.isTokenExpired,
  token: authStore.token ? authStore.token.substring(0, 20) + '...' : null,
  user: authStore.user,
  workspace: authStore.workspace,
  tokens: {
    hasAccessToken: !!authStore.tokens.accessToken,
    hasRefreshToken: !!authStore.tokens.refreshToken,
    expiresAt: authStore.tokens.expiresAt ? new Date(authStore.tokens.expiresAt).toISOString() : null
  }
}));

const localStorageData = computed(() => {
  const data = {};
  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i);
    let value = localStorage.getItem(key);

    // 截断长值
    if (value && value.length > 100) {
      value = value.substring(0, 100) + '...';
    }

    // 尝试解析JSON
    try {
      data[key] = JSON.parse(value);
    } catch {
      data[key] = value;
    }
  }
  return data;
});

const sessionStorageData = computed(() => {
  const data = {};
  for (let i = 0; i < sessionStorage.length; i++) {
    const key = sessionStorage.key(i);
    let value = sessionStorage.getItem(key);

    // 截断长值
    if (value && value.length > 100) {
      value = value.substring(0, 100) + '...';
    }

    // 尝试解析JSON
    try {
      data[key] = JSON.parse(value);
    } catch {
      data[key] = value;
    }
  }
  return data;
});

const clearStorage = () => {
  localStorage.clear();
  sessionStorage.clear();
  console.log('Storage cleared!');
  window.location.reload();
};

const goToLogin = () => {
  router.push('/login');
};

const goToHome = () => {
  router.push('/home');
};

const refreshAuth = async () => {
  await authStore.initialize();
  console.log('Auth refreshed!');
};
</script>

<style scoped>
.debug-container {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

section {
  margin-bottom: 30px;
  padding: 20px;
  background: #f5f5f5;
  border-radius: 8px;
}

h1 {
  color: #333;
  margin-bottom: 20px;
}

h2 {
  color: #666;
  margin-bottom: 10px;
}

pre {
  background: #fff;
  padding: 10px;
  border-radius: 4px;
  overflow: auto;
  white-space: pre-wrap;
  word-wrap: break-word;
}

button {
  margin-right: 10px;
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
</style>