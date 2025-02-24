<template>
  <div class="home-simplified">
    <!-- 🔄 Loading状态 -->
    <div v-if="isInitializing" class="loading-container">
      <div class="loading-spinner"></div>
      <p>Initializing...</p>
    </div>

    <!-- ❌ 错误状态 -->
    <div v-else-if="initError" class="error-container">
      <div class="error-icon">⚠️</div>
      <h3>Initialization Failed</h3>
      <p>{{ initError }}</p>
      <button @click="retryInitialization" class="retry-button">
        Retry
      </button>
    </div>

    <!-- ✅ 正常状态 -->
    <div v-else class="home-content">
      <header class="home-header">
        <h1>Welcome to Fechatter</h1>
        <div class="user-info">
          <span>{{ authStore.user?.fullname || 'User' }}</span>
          <button @click="handleLogout" class="logout-button">
            Logout
          </button>
        </div>
      </header>

      <main class="home-main">
        <!-- 🎯 聊天区域 -->
        <div class="chat-section">
          <div v-if="isLoadingChats" class="loading-chats">
            <div class="loading-spinner small"></div>
            <span>Loading chats...</span>
          </div>
          <div v-else-if="chats.length === 0" class="no-chats">
            <p>No chats available</p>
          </div>
          <div v-else class="chat-list">
            <h2>Your Chats</h2>
            <div v-for="chat in chats" :key="chat.id" class="chat-item">
              <div class="chat-avatar">
                {{ chat.name.charAt(0).toUpperCase() }}
              </div>
              <div class="chat-info">
                <h3>{{ chat.name }}</h3>
                <p>{{ chat.lastMessage || 'No messages yet' }}</p>
              </div>
            </div>
          </div>
        </div>
      </main>

      <!-- 🔍 开发模式调试面板 -->
      <div v-if="showDebugInfo" class="debug-panel">
        <h4>Debug Info</h4>
        <div class="debug-grid">
          <div class="debug-item">
            <strong>Auth State:</strong> {{ authStore.currentState }}
          </div>
          <div class="debug-item">
            <strong>User:</strong> {{ authStore.user?.email || 'None' }}
          </div>
          <div class="debug-item">
            <strong>Recovery Method:</strong> {{ recoveryMethod }}
          </div>
          <div class="debug-item">
            <strong>Init Time:</strong> {{ initializationTime }}ms
          </div>
        </div>
        <button @click="runAuthDiagnostics" class="debug-button">
          Run Diagnostics
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useSimplifiedAuthStore } from '@/stores/authSimplified';
import { authRecoveryManager } from '@/services/authRecoveryStrategies';

// 🎯 组合式API设置
const router = useRouter();
const authStore = useSimplifiedAuthStore();

// 🎯 响应式状态
const isInitializing = ref(true);
const initError = ref(null);
const isLoadingChats = ref(false);
const chats = ref([]);
const recoveryMethod = ref('none');
const initializationTime = ref(0);

// 🎯 计算属性
const showDebugInfo = computed(() => import.meta.env.DEV);

// 🔄 智能初始化 (延迟验证)
const initializeHome = async () => {
  const startTime = Date.now();

  try {
    // 🎯 Step 1: 快速认证检查
    if (authStore.isAuthenticated) {
      // 已认证，直接进入
      await loadChats();
      return;
    }

    // 🎯 Step 2: 尝试恢复认证状态
    const recoveryResult = await authRecoveryManager.attemptRecovery({
      tokenExpired: true,
      hasRememberedCredentials: false
    });

    if (recoveryResult.success) {
      recoveryMethod.value = recoveryResult.method;

      // 🔄 更新认证store状态
      if (recoveryResult.data && recoveryResult.data.accessToken) {
        // 这里应该有一个方法来设置恢复的认证数据
        // 现在简化为直接设置状态
        await authStore.attemptRecovery();
      }

      // 🎯 恢复成功，加载聊天
      await loadChats();
    } else {
      // 🚪 恢复失败，重定向到登录
      if (import.meta.env.DEV) {
        console.log('🚪 Recovery failed, redirecting to login');
      }
      router.push('/login');
    }

  } catch (error) {
    initError.value = error.message;
    console.error('Home initialization failed:', error);
  } finally {
    initializationTime.value = Date.now() - startTime;
    isInitializing.value = false;
  }
};

// 🎯 加载聊天数据 (延迟加载)
const loadChats = async () => {
  isLoadingChats.value = true;

  try {
    // 🔄 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 1000));

    // 🎯 模拟聊天数据
    chats.value = [
      {
        id: 1,
        name: 'General Chat',
        lastMessage: 'Welcome to Fechatter!'
      },
      {
        id: 2,
        name: 'Development',
        lastMessage: 'Working on new features...'
      },
      {
        id: 3,
        name: 'Random',
        lastMessage: 'How is everyone doing?'
      }
    ];

  } catch (error) {
    console.error('Failed to load chats:', error);
  } finally {
    isLoadingChats.value = false;
  }
};

// 🚪 简化的登出逻辑
const handleLogout = async () => {
  try {
    authStore.logout('User initiated logout');
    router.push('/login');
  } catch (error) {
    console.error('Logout failed:', error);
  }
};

// 🔄 重试初始化
const retryInitialization = () => {
  isInitializing.value = true;
  initError.value = null;
  initializeHome();
};

// 🧪 运行认证诊断
const runAuthDiagnostics = async () => {
  try {
    const diagnostics = {
      authState: authStore.getState(),
      recoveryStrategies: authRecoveryManager.getStrategies(),
      localStorage: {
        hasAuth: !!localStorage.getItem('auth'),
        authSize: localStorage.getItem('auth')?.length || 0
      },
      timestamp: new Date().toISOString()
    };

    console.group('🔍 Auth Diagnostics');
    console.log('Current State:', diagnostics.authState);
    console.log('Recovery Strategies:', diagnostics.recoveryStrategies);
    console.log('Storage:', diagnostics.localStorage);
    console.groupEnd();

  } catch (error) {
    console.error('Diagnostics failed:', error);
  }
};

// 🎯 组件生命周期
onMounted(() => {
  initializeHome();
});
</script>

<style scoped>
/* 🎨 简化的样式设计 */
.home-simplified {
  min-height: 100vh;
  background: #f9fafb;
}

/* 🔄 Loading状态 */
.loading-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
  gap: 1rem;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 4px solid #e5e7eb;
  border-top: 4px solid #6366f1;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.loading-spinner.small {
  width: 20px;
  height: 20px;
  border-width: 2px;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }

  100% {
    transform: rotate(360deg);
  }
}

/* ❌ 错误状态 */
.error-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
  gap: 1rem;
  padding: 2rem;
  text-align: center;
}

.error-icon {
  font-size: 3rem;
}

.error-container h3 {
  color: #dc2626;
  margin: 0;
}

.error-container p {
  color: #6b7280;
  margin: 0;
}

.retry-button {
  background: #6366f1;
  color: white;
  border: none;
  padding: 0.75rem 1.5rem;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.retry-button:hover {
  background: #4f46e5;
  transform: translateY(-1px);
}

/* ✅ 正常状态 */
.home-content {
  max-width: 1200px;
  margin: 0 auto;
  padding: 2rem;
}

.home-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2rem;
  padding-bottom: 1rem;
  border-bottom: 2px solid #e5e7eb;
}

.home-header h1 {
  color: #1f2937;
  font-size: 2rem;
  font-weight: 700;
  margin: 0;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.user-info span {
  color: #6b7280;
  font-weight: 500;
}

.logout-button {
  background: #ef4444;
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 6px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.logout-button:hover {
  background: #dc2626;
}

/* 🎯 聊天区域 */
.chat-section {
  background: white;
  border-radius: 12px;
  padding: 2rem;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.05);
  min-height: 400px;
}

.loading-chats {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 2rem;
  color: #6b7280;
}

.no-chats {
  text-align: center;
  padding: 2rem;
  color: #6b7280;
}

.chat-list h2 {
  color: #1f2937;
  margin-bottom: 1.5rem;
  font-size: 1.5rem;
  font-weight: 600;
}

.chat-item {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem;
  border-radius: 8px;
  transition: all 0.2s ease;
  cursor: pointer;
}

.chat-item:hover {
  background: #f3f4f6;
}

.chat-avatar {
  width: 40px;
  height: 40px;
  background: #6366f1;
  color: white;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 1.1rem;
}

.chat-info {
  flex: 1;
}

.chat-info h3 {
  margin: 0 0 0.25rem 0;
  color: #1f2937;
  font-size: 1rem;
  font-weight: 600;
}

.chat-info p {
  margin: 0;
  color: #6b7280;
  font-size: 0.9rem;
}

/* 🔍 调试面板 */
.debug-panel {
  margin-top: 2rem;
  background: white;
  border-radius: 12px;
  padding: 1.5rem;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.05);
  border-left: 4px solid #6366f1;
}

.debug-panel h4 {
  margin: 0 0 1rem 0;
  color: #1f2937;
  font-size: 1.2rem;
}

.debug-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 1rem;
  margin-bottom: 1rem;
}

.debug-item {
  padding: 0.75rem;
  background: #f9fafb;
  border-radius: 6px;
  font-size: 0.9rem;
}

.debug-item strong {
  color: #374151;
}

.debug-button {
  background: #6366f1;
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 6px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.debug-button:hover {
  background: #4f46e5;
}

/* 🎯 响应式设计 */
@media (max-width: 768px) {
  .home-content {
    padding: 1rem;
  }

  .home-header {
    flex-direction: column;
    gap: 1rem;
    text-align: center;
  }

  .chat-section {
    padding: 1rem;
  }

  .debug-grid {
    grid-template-columns: 1fr;
  }
}

/* 🎯 可访问性优化 */
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}

button:focus-visible {
  outline: 2px solid #6366f1;
  outline-offset: 2px;
}
</style>