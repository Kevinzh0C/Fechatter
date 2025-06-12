<template>
  <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
    <!-- Monitor Panel -->
    <div v-if="showMonitor"
      class="fixed top-0 right-0 w-96 h-full bg-gray-900 shadow-xl z-50 overflow-hidden flex flex-col">
      <div class="p-4 bg-gray-800 flex items-center justify-between">
        <h3 class="text-white font-semibold">Login Flow Monitor</h3>
        <button @click="showMonitor = false" class="text-gray-400 hover:text-white">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div class="p-4 bg-gray-800 border-t border-gray-700">
        <div class="grid grid-cols-3 gap-4 text-center">
          <div>
            <div class="text-2xl font-bold text-blue-400">{{ monitorEvents.length }}</div>
            <div class="text-xs text-gray-400">Events</div>
          </div>
          <div>
            <div class="text-2xl font-bold text-red-400">{{ errorCount }}</div>
            <div class="text-xs text-gray-400">Errors</div>
          </div>
          <div>
            <div class="text-2xl font-bold text-green-400">{{ duration }}ms</div>
            <div class="text-xs text-gray-400">Duration</div>
          </div>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto p-4 space-y-2">
        <div v-for="event in monitorEvents" :key="event.id" class="bg-gray-800 rounded p-3 text-sm"
          :class="{ 'border-l-4 border-red-500': event.status === 'error' }">
          <div class="flex items-center justify-between mb-1">
            <span class="font-semibold" :class="getEventColor(event.type)">{{ event.type }}</span>
            <span class="text-xs text-gray-500">+{{ event.elapsed }}ms</span>
          </div>
          <pre v-if="event.data && Object.keys(event.data).length"
            class="text-xs text-gray-400 overflow-x-auto">{{ JSON.stringify(event.data, null, 2) }}</pre>
        </div>
      </div>

      <div class="p-4 bg-gray-800 border-t border-gray-700 space-y-2">
        <button @click="monitor.clear(); monitorEvents = []"
          class="w-full py-2 bg-gray-700 text-white rounded hover:bg-gray-600 text-sm">
          Clear Events
        </button>
        <button @click="monitor.downloadReport()"
          class="w-full py-2 bg-blue-600 text-white rounded hover:bg-blue-700 text-sm">
          Download Report
        </button>
      </div>
    </div>

    <!-- Monitor Toggle -->
    <button @click="showMonitor = !showMonitor"
      class="fixed top-4 right-4 p-3 bg-blue-600 text-white rounded-full shadow-lg hover:bg-blue-700 z-40">
      <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
      </svg>
    </button>

    <!-- Login Form -->
    <div class="flex items-center justify-center min-h-screen px-4">
      <div class="w-full max-w-md">
        <div class="bg-white dark:bg-gray-800 shadow-lg rounded-lg px-8 pt-6 pb-8 mb-4">
          <div class="mb-8 text-center">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-white">Welcome Back</h1>
            <p class="text-gray-600 dark:text-gray-400 mt-2">Sign in to your account</p>
          </div>

          <form @submit.prevent="handleLogin" class="space-y-6">
            <div>
              <label for="username" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Email Address
              </label>
              <input id="username" v-model="loginData.username" type="email" required
                @focus="logInputEvent('focus', 'username')" @blur="logInputEvent('blur', 'username')"
                @input="logInputEvent('input', 'username')"
                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                placeholder="Enter your email address" />
            </div>

            <div>
              <label for="password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Password
              </label>
              <input id="password" v-model="loginData.password" type="password" required
                @focus="logInputEvent('focus', 'password')" @blur="logInputEvent('blur', 'password')"
                @input="logInputEvent('input', 'password')"
                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                placeholder="Enter your password" />
            </div>

            <div v-if="error"
              class="bg-red-50 dark:bg-red-900/20 border border-red-400 text-red-700 dark:text-red-400 px-4 py-3 rounded">
              {{ error }}
            </div>

            <button type="submit" :disabled="loading"
              class="w-full bg-blue-600 text-white py-2 px-4 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors">
              <span v-if="!loading">Sign In</span>
              <span v-else class="flex items-center justify-center">
                <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z">
                  </path>
                </svg>
                Processing...
              </span>
            </button>
          </form>

          <div class="mt-6 text-center">
            <p class="text-sm text-gray-600 dark:text-gray-400">
              Don't have an account?
              <router-link to="/register" class="text-blue-600 hover:text-blue-700 dark:text-blue-400 font-medium">
                Sign up
              </router-link>
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useToast } from '@/composables/useToast'
import LoginFlowMonitor from '@/utils/login-monitor'
import extensionConflictHandler from '@/utils/extensionConflictHandler'

export default {
  name: 'LoginMonitored',
  setup() {
    const router = useRouter()
    const authStore = useAuthStore()
    const { showToast } = useToast()

    // 监控器
    const monitor = new LoginFlowMonitor()
    const monitorEvents = ref([])
    const showMonitor = ref(true)

    // 表单数据
    const loginData = ref({
      username: '',
      password: ''
    })
    const loading = ref(false)
    const error = ref('')

    // 计算属性
    const errorCount = computed(() =>
      monitorEvents.value.filter(e => e.status === 'error').length
    )

    const duration = computed(() => {
      if (monitorEvents.value.length === 0) return 0
      const first = monitorEvents.value[0]
      const last = monitorEvents.value[monitorEvents.value.length - 1]
      return last.elapsed
    })

    // 获取事件颜色
    const getEventColor = (type) => {
      const colors = {
        'USER_ACTION': 'text-blue-400',
        'API_REQUEST': 'text-orange-400',
        'API_RESPONSE': 'text-green-400',
        'STORE_UPDATE': 'text-purple-400',
        'ROUTE_CHANGE': 'text-cyan-400',
        'ERROR': 'text-red-400',
        'VALIDATION': 'text-yellow-400',
        'EXTENSION_CONFLICT': 'text-pink-400'
      }

      for (const [prefix, color] of Object.entries(colors)) {
        if (type.startsWith(prefix)) return color
      }
      return 'text-gray-400'
    }

    // 记录输入事件
    const logInputEvent = (event, field) => {
      monitor.logEvent(`INPUT_${event.toUpperCase()}`, {
        field,
        value: event === 'input' ? loginData.value[field].length : undefined
      })
    }

    // 处理登录
    const handleLogin = async () => {
      // 清空之前的错误
      error.value = ''

      // 开始监控
      monitor.start()
      monitor.logEvent('LOGIN_START', {
        username: loginData.value.username,
        timestamp: new Date().toISOString()
      })

      // 监控Store
      monitor.monitorStore(authStore, 'auth')

      // 验证表单
      if (!loginData.value.username || !loginData.value.password) {
        monitor.logEvent('VALIDATION_ERROR', {
          message: 'Username and password are required',
          fields: {
            username: !loginData.value.username,
            password: !loginData.value.password
          }
        }, 'error')

        error.value = 'Please fill in all fields'
        return
      }

      monitor.logEvent('VALIDATION_SUCCESS', {
        message: 'Form validation passed'
      })

      loading.value = true

      try {
        // 执行登录
        monitor.logEvent('AUTH_ACTION_CALL', {
          action: 'login',
          email: loginData.value.username
        })

        const success = await authStore.login(
          loginData.value.username,
          loginData.value.password
        )

        if (success) {
          monitor.logEvent('LOGIN_SUCCESS', {
            user: authStore.user,
            redirectTo: '/chat'
          })

          showToast('Login successful!', 'success')

          // 验证token状态后再导航 - 修复时序竞争问题
          monitor.logEvent('TOKEN_VERIFICATION', {
            hasToken: !!authStore.token,
            isAuthenticated: authStore.isAuthenticated,
            hasUser: !!authStore.user,
            tokenPreview: authStore.token ? authStore.token.substring(0, 20) + '...' : null
          })

          // 确保token已正确设置后再导航
          const verifyTokenAndNavigate = async () => {
            // Give a moment for async operations to complete
            await new Promise(resolve => setTimeout(resolve, 50))

            const token = authStore.token
            const isAuthenticated = authStore.isAuthenticated

            monitor.logEvent('TOKEN_CHECK', {
              hasToken: !!token,
              isAuthenticated,
              tokenLength: token ? token.length : 0
            })

            if (token && isAuthenticated) {
              monitor.logEvent('NAVIGATION_START', {
                from: '/login',
                to: '/chat',
                tokenStatus: 'verified'
              })
              router.push('/chat')
            } else {
              // Log the specific issue
              monitor.logEvent('TOKEN_MISSING', {
                hasToken: !!token,
                isAuthenticated,
                willRetry: true
              }, 'warning')

              // Retry up to 5 times with exponential backoff
              let retryCount = 0
              const maxRetries = 5

              const retryCheck = async () => {
                retryCount++
                await new Promise(resolve => setTimeout(resolve, 100 * retryCount))

                const retryToken = authStore.token
                if (retryToken && authStore.isAuthenticated) {
                  monitor.logEvent('TOKEN_RETRY_SUCCESS', {
                    attempt: retryCount,
                    tokenLength: retryToken.length
                  })
                  router.push('/chat')
                } else if (retryCount < maxRetries) {
                  monitor.logEvent('TOKEN_RETRY', {
                    attempt: retryCount,
                    maxRetries
                  }, 'warning')
                  retryCheck()
                } else {
                  // Final failure - something is wrong
                  monitor.logEvent('TOKEN_SETUP_FAILED', {
                    attempts: retryCount,
                    finalState: {
                      hasToken: !!authStore.token,
                      isAuthenticated: authStore.isAuthenticated,
                      hasUser: !!authStore.user
                    }
                  }, 'error')

                  error.value = 'Authentication setup failed. Please try again.'
                  showToast('Authentication setup failed', 'error')

                  // Clear auth state to allow retry
                  authStore.clearAuth()
                }
              }

              retryCheck()
            }
          }

          // Start verification process
          verifyTokenAndNavigate()
        } else {
          throw new Error(authStore.error || 'Login failed')
        }
      } catch (err) {
        monitor.logEvent('LOGIN_ERROR', {
          error: err.message,
          stack: err.stack
        }, 'error')

        error.value = err.message
        showToast(err.message, 'error')
      } finally {
        loading.value = false

        // 生成摘要
        setTimeout(() => {
          monitor.showSummary()
        }, 1000)
      }
    }

    // 监听监控事件
    const handleMonitorEvent = (e) => {
      monitorEvents.value.push(e.detail)
    }

    // Handle extension conflicts
    const handleExtensionConflict = (e) => {
      console.log('🔧 [LOGIN] Extension conflict detected:', e.detail)

      // Add conflict info to monitor events
      monitor.logEvent('EXTENSION_CONFLICT', {
        conflictId: e.detail.id,
        source: e.detail.source,
        message: e.detail.message.substring(0, 100)
      }, 'warning')

      // Show user-friendly message
      if (e.detail.source === 'promise_rejection') {
        console.log('🔧 [LOGIN] Browser extension interference detected and handled')
      }
    }

    onMounted(() => {
      window.addEventListener('login-monitor-event', handleMonitorEvent)
      window.addEventListener('extension-conflict-detected', handleExtensionConflict)

      // Initialize extension conflict handler
      extensionConflictHandler.initialize()

      // 监控路由
      monitor.monitorRouter(router)

      // 预填充测试数据（开发环境）
      if (import.meta.env.DEV) {
        loginData.value.username = 'admin@test.com'
        loginData.value.password = 'super123'

        // Show extension conflict report in development
        setTimeout(() => {
          extensionConflictHandler.showConflictReport()
        }, 3000)
      }
    })

    onUnmounted(() => {
      window.removeEventListener('login-monitor-event', handleMonitorEvent)
      window.removeEventListener('extension-conflict-detected', handleExtensionConflict)

      // Clean up extension conflict handler
      extensionConflictHandler.disable()
    })

    return {
      loginData,
      loading,
      error,
      handleLogin,
      logInputEvent,

      // 监控相关
      monitor,
      monitorEvents,
      showMonitor,
      errorCount,
      duration,
      getEventColor
    }
  }
}
</script>