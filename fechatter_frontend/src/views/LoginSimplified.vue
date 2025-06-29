<template>
  <div class="login-simplified">
    <div class="login-container">
      <div class="login-header">
        <h1>Welcome Back</h1>
        <p>Sign in to continue to Fechatter</p>
      </div>

      <!-- 简化的登录表单 -->
      <form @submit.prevent="handleLogin" class="login-form">
        <div class="form-group">
          <label for="email">Email</label>
          <input id="email" v-model="email" type="email" required :disabled="authStore.isLoading"
            @keydown.enter="handleLogin" class="form-input" placeholder="Enter your email" />
        </div>

        <div class="form-group">
          <label for="password">Password</label>
          <input id="password" v-model="password" type="password" required :disabled="authStore.isLoading"
            @keydown.enter="handleLogin" class="form-input" placeholder="Enter your password" />
        </div>

        <!-- 简化的错误显示 -->
        <div v-if="authStore.error" class="error-message">
          {{ authStore.error }}
        </div>

        <!-- 乐观的登录按钮 -->
        <button type="submit" :disabled="!canSubmit" class="login-button" :class="{ loading: authStore.isLoading }">
          <span v-if="!authStore.isLoading">Sign In</span>
          <span v-else class="loading-content">
            <svg class="spinner" viewBox="0 0 24 24">
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none" opacity="0.25" />
              <path fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
            </svg>
            Signing In...
          </span>
        </button>
      </form>

      <!-- 开发模式的状态指示器 -->
      <div v-if="showDebugInfo" class="debug-info">
        <div class="debug-title">Auth State</div>
        <div class="debug-content">
          State: {{ authStore.currentState }}<br>
          Loading: {{ authStore.isLoading }}<br>
          Can Auth: {{ authStore.canAttemptAuth }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useSimplifiedAuthStore } from '@/stores/authSimplified';

// 组合式API设置
const router = useRouter();
const authStore = useSimplifiedAuthStore();

// 响应式数据
const email = ref('');
const password = ref('');

// 计算属性 (简化逻辑)
const canSubmit = computed(() => {
  return email.value.trim() &&
    password.value.trim() &&
    authStore.canAttemptAuth &&
    !authStore.isLoading;
});

const showDebugInfo = computed(() => {
  return import.meta.env.DEV;
});

// 简化的登录逻辑 (乐观更新)
const handleLogin = async () => {
  if (!canSubmit.value) return;

  try {
    // 直接调用登录，不需要复杂的验证
    await authStore.login(email.value.trim(), password.value);

    // 乐观导航 - 相信认证状态已正确设置
    router.push('/home');

  } catch (error) {
    // 错误已由store处理，组件只需要显示
    console.error('Login failed:', error);
  }
};

// 组件初始化
onMounted(async () => {
  // 🔄 如果已经认证，直接跳转
  if (authStore.isAuthenticated) {
    router.push('/home');
    return;
  }

  // 🔄 尝试恢复认证状态 (非阻塞)
  authStore.initialize().then(recovered => {
    if (recovered) {
      router.push('/home');
    }
  });
});
</script>

<style scoped>
/* 简化的样式设计 */
.login-simplified {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 20px;
}

.login-container {
  background: white;
  padding: 2rem;
  border-radius: 12px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
  width: 100%;
  max-width: 400px;
}

.login-header {
  text-align: center;
  margin-bottom: 2rem;
}

.login-header h1 {
  font-size: 1.8rem;
  font-weight: 700;
  color: #1f2937;
  margin-bottom: 0.5rem;
}

.login-header p {
  color: #6b7280;
  font-size: 0.9rem;
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.form-group label {
  font-weight: 600;
  color: #374151;
  font-size: 0.9rem;
}

.form-input {
  padding: 0.75rem;
  border: 2px solid #e5e7eb;
  border-radius: 8px;
  font-size: 1rem;
  transition: all 0.2s ease;
  outline: none;
}

.form-input:focus {
  border-color: #6366f1;
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
}

.form-input:disabled {
  background: #f9fafb;
  color: #9ca3af;
  cursor: not-allowed;
}

.error-message {
  background: #fef2f2;
  border: 1px solid #fecaca;
  color: #dc2626;
  padding: 0.75rem;
  border-radius: 6px;
  font-size: 0.9rem;
}

.login-button {
  background: #6366f1;
  color: white;
  border: none;
  padding: 0.875rem;
  border-radius: 8px;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  min-height: 48px;
}

.login-button:hover:not(:disabled) {
  background: #4f46e5;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3);
}

.login-button:disabled {
  background: #d1d5db;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

.loading-content {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.spinner {
  width: 20px;
  height: 20px;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}

/* 开发模式调试信息 */
.debug-info {
  margin-top: 1.5rem;
  padding: 1rem;
  background: #f3f4f6;
  border-radius: 6px;
  border-left: 4px solid #6366f1;
}

.debug-title {
  font-weight: 600;
  color: #374151;
  margin-bottom: 0.5rem;
  font-size: 0.9rem;
}

.debug-content {
  font-family: monospace;
  font-size: 0.8rem;
  color: #6b7280;
  line-height: 1.4;
}

/* 响应式设计 */
@media (max-width: 480px) {
  .login-container {
    padding: 1.5rem;
    margin: 0 10px;
  }

  .login-header h1 {
    font-size: 1.5rem;
  }
}

/* 可访问性优化 */
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}

.form-input:focus,
.login-button:focus {
  outline: 2px solid #6366f1;
  outline-offset: 2px;
}
</style>