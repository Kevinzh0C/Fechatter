<template>
  <div class="loading-completion-container">
    <!-- 🔄 加载进度指示器 -->
    <Transition name="loading-fade">
      <div v-if="showLoadingIndicator" class="loading-progress">
        <div class="loading-content">
          <div class="loading-spinner" :class="{ 'paused': isPaused }"></div>
          <div class="loading-text">
            <span class="primary-text">{{ loadingText }}</span>
            <span v-if="progressInfo" class="progress-info">{{ progressInfo }}</span>
          </div>
          <button v-if="canCancel" @click="handleCancel" class="cancel-button">
            取消
          </button>
        </div>

        <!-- 进度条 -->
        <div v-if="showProgress" class="progress-bar-container">
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: `${progressPercentage}%` }"></div>
          </div>
          <span class="progress-text">{{ progressPercentage }}%</span>
        </div>
      </div>
    </Transition>

    <!-- 🔄 NEW: 等待滚动提示 -->
    <Transition name="scroll-wait-fade">
      <div v-if="showScrollWait" class="scroll-wait-indicator">
        <div class="scroll-wait-content">
          <div class="scroll-icon">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M8 3a5 5 0 0 1 5 5v8a5 5 0 0 1-5 5 5 5 0 0 1-5-5V8a5 5 0 0 1 5-5z"></path>
              <path d="m8 14 4-4 4 4"></path>
            </svg>
          </div>
          <div class="scroll-message">
            <h3 class="scroll-title">✋ 用户控制加载</h3>
            <p class="scroll-details">{{ scrollWaitMessage }}</p>
            <div class="scroll-stats">
              已加载 <strong>{{ totalLoaded }}</strong> 条消息
            </div>

            <!-- 🎯 NEW: 批次大小控制 (性能优化) -->
            <div class="batch-control">
              <label class="batch-label">每批加载数量:</label>
              <select v-model="selectedBatchSize" @change="handleBatchSizeChange" class="batch-select">
                <option value="10">10条 (省流量)</option>
                <option value="20">20条 (推荐)</option>
                <option value="50">50条 (快速)</option>
              </select>
            </div>
          </div>

          <div class="scroll-actions">
            <!-- 主要操作 -->
            <button @click="handleContinueLoading" class="continue-button">
              🚀 继续加载 {{ selectedBatchSize }} 条
            </button>

            <!-- 次要操作 -->
            <div class="secondary-actions">
              <button @click="handleScrollToLoad" class="scroll-load-button">
                📜 滚动到顶部加载
              </button>
              <button @click="handleStopLoading" class="stop-button">
                🛑 停止并保存进度
              </button>
            </div>
          </div>
        </div>
      </div>
    </Transition>

    <!-- ✅ 完成通知 -->
    <Transition name="completion-fade">
      <div v-if="showCompletion" class="completion-notification">
        <div class="completion-content">
          <div class="completion-icon">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
              <path d="M20 6L9 17L4 12" stroke="currentColor" stroke-width="2" stroke-linecap="round"
                stroke-linejoin="round" />
            </svg>
          </div>

          <div class="completion-message">
            <h3 class="completion-title">{{ completionTitle }}</h3>
            <p class="completion-details">{{ completionMessage }}</p>
            <div v-if="showStats" class="completion-stats">
              <span class="stat-item">
                <strong>{{ totalLoaded }}</strong> 条消息
              </span>
              <span class="stat-item">
                用时 <strong>{{ formatDuration(loadDuration) }}</strong>
              </span>
            </div>
          </div>

          <div class="completion-actions">
            <button @click="handleDismiss" class="dismiss-button">
              知道了
            </button>
            <button v-if="showScrollToTop" @click="handleScrollToTop" class="scroll-top-button">
              回到最早消息
            </button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- ❌ 错误状态 -->
    <Transition name="error-fade">
      <div v-if="showError" class="error-notification">
        <div class="error-content">
          <div class="error-icon">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2" />
              <line x1="15" y1="9" x2="9" y2="15" stroke="currentColor" stroke-width="2" />
              <line x1="9" y1="9" x2="15" y2="15" stroke="currentColor" stroke-width="2" />
            </svg>
          </div>

          <div class="error-message">
            <h3 class="error-title">加载失败</h3>
            <p class="error-details">{{ errorMessage }}</p>
          </div>

          <div class="error-actions">
            <button @click="handleRetry" class="retry-button">
              重试
            </button>
            <button @click="handleDismissError" class="dismiss-error-button">
              关闭
            </button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 🚫 禁用加载提示 -->
    <Transition name="disabled-fade">
      <div v-if="showDisabled" class="disabled-notification">
        <div class="disabled-content">
          <div class="disabled-icon">ℹ️</div>
          <span class="disabled-text">自动加载已停止</span>
          <button @click="handleEnableAutoLoad" class="enable-button">
            重新启用
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'

const props = defineProps({
  // 管理器状态
  autoLoadState: {
    type: String,
    default: 'idle'
  },

  // 加载进度
  totalLoaded: {
    type: Number,
    default: 0
  },

  currentPhase: {
    type: String,
    default: ''
  },

  // 错误信息
  errorMessage: {
    type: String,
    default: ''
  },

  // 完成信息
  completionData: {
    type: Object,
    default: () => ({})
  },

  // 配置选项
  showProgress: {
    type: Boolean,
    default: true
  },

  showStats: {
    type: Boolean,
    default: true
  },

  canCancel: {
    type: Boolean,
    default: true
  },

  autoHideDelay: {
    type: Number,
    default: 5000 // 5秒后自动隐藏
  }
})

const emit = defineEmits([
  'cancel',
  'retry',
  'dismiss',
  'scroll-to-top',
  'enable-auto-load',
  // 🔄 ENHANCED: 用户控制相关事件
  'continue-loading',
  'stop-loading',
  'scroll-to-load',
  'batch-size-change'
])

// 内部状态
const progressPercentage = ref(0)
const loadDuration = ref(0)
const autoHideTimer = ref(null)

// 🎯 NEW: 用户控制状态
const selectedBatchSize = ref(20) // 默认20条

// 显示状态计算
const showLoadingIndicator = computed(() =>
  ['detecting_need', 'loading', 'processing'].includes(props.autoLoadState)
)

const showScrollWait = computed(() =>
  props.autoLoadState === 'waiting_for_scroll'
)

const showCompletion = computed(() =>
  props.autoLoadState === 'completed' && props.completionData.totalLoaded > 0
)

const showError = computed(() =>
  props.autoLoadState === 'error'
)

const showDisabled = computed(() =>
  props.autoLoadState === 'disabled'
)

const isPaused = computed(() =>
  props.autoLoadState === 'waiting_for_scroll'
)

const showScrollToTop = computed(() =>
  props.totalLoaded > 20 // 超过20条消息才显示回到顶部
)

// 文本内容
const loadingText = computed(() => {
  switch (props.currentPhase) {
    case 'detecting_need':
      return '检测消息状态...'
    case 'loading':
      return 'Loading earlier messages...'
    case 'processing':
      return '处理消息数据...'
    case 'waiting_for_scroll':
      return '等待继续加载...'
    default:
      return '正在加载...'
  }
})

const scrollWaitMessage = computed(() => {
  const loaded = props.totalLoaded || 0
  return `已加载 ${loaded} 条消息。您可以选择继续加载更多历史消息，或随时停止。`
})

const progressInfo = computed(() => {
  if (props.totalLoaded > 0) {
    return `已加载 ${props.totalLoaded} 条消息`
  }
  return ''
})

const completionTitle = computed(() => '🎉 全部消息已加载')

const completionMessage = computed(() => {
  const total = props.completionData.totalLoaded || props.totalLoaded
  return `已成功加载频道中的全部 ${total} 条历史消息`
})

// 方法
const handleCancel = () => {
  emit('cancel')
}

const handleRetry = () => {
  emit('retry')
}

const handleDismiss = () => {
  clearAutoHideTimer()
  emit('dismiss')
}

const handleDismissError = () => {
  emit('dismiss')
}

const handleScrollToTop = () => {
  emit('scroll-to-top')
}

const handleEnableAutoLoad = () => {
  emit('enable-auto-load')
}

// 🔄 ENHANCED: 用户控制事件处理
const handleContinueLoading = () => {
  emit('continue-loading', { batchSize: selectedBatchSize.value })
}

const handleStopLoading = () => {
  emit('stop-loading', {
    reason: 'user-choice',
    totalLoaded: props.totalLoaded,
    saveProgress: true
  })
}

const handleScrollToLoad = () => {
  emit('scroll-to-load', {
    batchSize: selectedBatchSize.value,
    instruction: '请滚动到消息列表顶部以继续加载'
  })
}

const handleBatchSizeChange = () => {
  emit('batch-size-change', {
    newBatchSize: selectedBatchSize.value,
    reason: '用户调整性能参数'
  })
}

const formatDuration = (ms) => {
  if (ms < 1000) return `${ms}ms`
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`
  return `${Math.floor(ms / 60000)}m ${Math.floor((ms % 60000) / 1000)}s`
}

const clearAutoHideTimer = () => {
  if (autoHideTimer.value) {
    clearTimeout(autoHideTimer.value)
    autoHideTimer.value = null
  }
}

const setupAutoHide = () => {
  if (props.autoHideDelay > 0 && showCompletion.value) {
    clearAutoHideTimer()
    autoHideTimer.value = setTimeout(() => {
      handleDismiss()
    }, props.autoHideDelay)
  }
}

// 监听状态变化
watch(() => props.autoLoadState, (newState) => {
  if (newState === 'completed') {
    loadDuration.value = props.completionData.duration || 0
    setupAutoHide()
  }
})

watch(() => props.totalLoaded, (newTotal, oldTotal) => {
  // 简单的进度计算（基于已加载消息数）
  if (newTotal > oldTotal) {
    progressPercentage.value = Math.min(100, (newTotal / 100) * 100)
  }
})

// 生命周期
onMounted(() => {
  if (showCompletion.value) {
    setupAutoHide()
  }
})

onUnmounted(() => {
  clearAutoHideTimer()
})
</script>

<style scoped>
.loading-completion-container {
  position: relative;
  width: 100%;
}

/* 🔄 加载进度样式 */
.loading-progress {
  position: fixed;
  top: 80px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(255, 255, 255, 0.98);
  backdrop-filter: blur(12px);
  border-radius: 16px;
  padding: 20px 24px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
  border: 1px solid rgba(124, 58, 237, 0.15);
  z-index: 1000;
  min-width: 320px;
  max-width: 480px;
}

.loading-content {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 12px;
}

.loading-spinner {
  width: 20px;
  height: 20px;
  border: 2px solid #e0e7ff;
  border-top-color: #7c3aed;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.loading-spinner.paused {
  animation-play-state: paused;
  opacity: 0.6;
}

.loading-text {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.primary-text {
  font-size: 14px;
  font-weight: 600;
  color: #374151;
}

.progress-info {
  font-size: 12px;
  color: #6b7280;
}

.cancel-button {
  padding: 6px 12px;
  background: #f3f4f6;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  font-size: 12px;
  color: #6b7280;
  cursor: pointer;
  transition: all 0.2s ease;
}

.cancel-button:hover {
  background: #e5e7eb;
  color: #374151;
}

.progress-bar-container {
  display: flex;
  align-items: center;
  gap: 12px;
}

.progress-bar {
  flex: 1;
  height: 6px;
  background: #e5e7eb;
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #7c3aed, #a855f7);
  border-radius: 3px;
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 12px;
  color: #6b7280;
  font-weight: 500;
  min-width: 35px;
}

/* ✅ 完成通知样式 */
.completion-notification {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: rgba(255, 255, 255, 0.98);
  backdrop-filter: blur(16px);
  border-radius: 20px;
  padding: 32px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
  border: 1px solid rgba(16, 185, 129, 0.2);
  z-index: 1001;
  max-width: 440px;
  width: 90vw;
}

.completion-content {
  text-align: center;
}

.completion-icon {
  width: 48px;
  height: 48px;
  background: #10b981;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 auto 20px;
  color: white;
}

.completion-title {
  font-size: 20px;
  font-weight: 700;
  color: #111827;
  margin: 0 0 8px;
}

.completion-details {
  font-size: 14px;
  color: #6b7280;
  margin: 0 0 16px;
  line-height: 1.5;
}

.completion-stats {
  display: flex;
  justify-content: center;
  gap: 24px;
  margin-bottom: 24px;
  padding: 16px;
  background: #f9fafb;
  border-radius: 12px;
}

.stat-item {
  font-size: 13px;
  color: #6b7280;
}

.stat-item strong {
  color: #111827;
  font-weight: 600;
}

.completion-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.dismiss-button {
  padding: 10px 20px;
  background: #7c3aed;
  color: white;
  border: none;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.dismiss-button:hover {
  background: #6d28d9;
  transform: translateY(-1px);
}

.scroll-top-button {
  padding: 10px 20px;
  background: #f3f4f6;
  color: #6b7280;
  border: 1px solid #d1d5db;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.scroll-top-button:hover {
  background: #e5e7eb;
  color: #374151;
}

/* ❌ 错误状态样式 */
.error-notification {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: rgba(255, 255, 255, 0.98);
  backdrop-filter: blur(16px);
  border-radius: 16px;
  padding: 28px;
  box-shadow: 0 16px 32px rgba(0, 0, 0, 0.12);
  border: 1px solid rgba(239, 68, 68, 0.2);
  z-index: 1001;
  max-width: 400px;
  width: 90vw;
}

.error-content {
  text-align: center;
}

.error-icon {
  width: 40px;
  height: 40px;
  color: #ef4444;
  margin: 0 auto 16px;
}

.error-title {
  font-size: 18px;
  font-weight: 600;
  color: #111827;
  margin: 0 0 8px;
}

.error-details {
  font-size: 14px;
  color: #6b7280;
  margin: 0 0 20px;
}

.error-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.retry-button {
  padding: 10px 20px;
  background: #ef4444;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.retry-button:hover {
  background: #dc2626;
}

.dismiss-error-button {
  padding: 10px 20px;
  background: #f3f4f6;
  color: #6b7280;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.dismiss-error-button:hover {
  background: #e5e7eb;
}

/* 🔄 NEW: 等待滚动样式 */
.scroll-wait-indicator {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: rgba(255, 255, 255, 0.98);
  backdrop-filter: blur(16px);
  border-radius: 20px;
  padding: 32px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
  border: 1px solid rgba(59, 130, 246, 0.2);
  z-index: 1001;
  max-width: 440px;
  width: 90vw;
}

.scroll-wait-content {
  text-align: center;
}

.scroll-icon {
  width: 48px;
  height: 48px;
  background: #3b82f6;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 auto 20px;
  color: white;
  animation: pulse 2s ease-in-out infinite;
}

.scroll-title {
  font-size: 20px;
  font-weight: 700;
  color: #111827;
  margin: 0 0 8px;
}

.scroll-details {
  font-size: 14px;
  color: #6b7280;
  margin: 0 0 16px;
  line-height: 1.5;
}

.scroll-stats {
  display: flex;
  justify-content: center;
  margin-bottom: 24px;
  padding: 12px;
  background: #f0f9ff;
  border-radius: 12px;
  font-size: 13px;
  color: #6b7280;
}

.scroll-stats strong {
  color: #111827;
  font-weight: 600;
}

/* 🎯 NEW: 批次控制样式 */
.batch-control {
  margin-top: 16px;
  padding: 12px;
  background: #f8fafc;
  border-radius: 8px;
  display: flex;
  align-items: center;
  gap: 8px;
  justify-content: center;
}

.batch-label {
  font-size: 13px;
  color: #4b5563;
  font-weight: 500;
}

.batch-select {
  padding: 4px 8px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  background: white;
  font-size: 13px;
  color: #374151;
  cursor: pointer;
  transition: border-color 0.2s ease;
}

.batch-select:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
}

.scroll-actions {
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: center;
}

.secondary-actions {
  display: flex;
  gap: 8px;
  justify-content: center;
}

.continue-button {
  padding: 12px 24px;
  background: #3b82f6;
  color: white;
  border: none;
  border-radius: 12px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 2px 8px rgba(59, 130, 246, 0.3);
}

.continue-button:hover {
  background: #2563eb;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.4);
}

.scroll-load-button {
  padding: 8px 16px;
  background: #f8fafc;
  color: #4b5563;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.scroll-load-button:hover {
  background: #e2e8f0;
  color: #1e293b;
  border-color: #94a3b8;
}

.stop-button {
  padding: 8px 16px;
  background: #fef2f2;
  color: #dc2626;
  border: 1px solid #fecaca;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.stop-button:hover {
  background: #fee2e2;
  color: #b91c1c;
  border-color: #fca5a5;
}

/* 🚫 禁用状态样式 */
.disabled-notification {
  position: fixed;
  bottom: 20px;
  right: 20px;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(8px);
  border-radius: 12px;
  padding: 16px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
  border: 1px solid #d1d5db;
  z-index: 999;
}

.disabled-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.disabled-icon {
  font-size: 18px;
}

.disabled-text {
  font-size: 14px;
  color: #6b7280;
}

.enable-button {
  padding: 6px 12px;
  background: #7c3aed;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.enable-button:hover {
  background: #6d28d9;
}

/* 动画 */
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@keyframes pulse {

  0%,
  100% {
    transform: scale(1);
  }

  50% {
    transform: scale(1.05);
  }
}

.loading-fade-enter-active,
.loading-fade-leave-active {
  transition: all 0.3s ease;
}

.loading-fade-enter-from {
  opacity: 0;
  transform: translateX(-50%) translateY(-20px);
}

.loading-fade-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-20px);
}

.completion-fade-enter-active,
.completion-fade-leave-active {
  transition: all 0.4s ease;
}

.completion-fade-enter-from,
.completion-fade-leave-to {
  opacity: 0;
  transform: translate(-50%, -50%) scale(0.9);
}

.error-fade-enter-active,
.error-fade-leave-active {
  transition: all 0.3s ease;
}

.error-fade-enter-from,
.error-fade-leave-to {
  opacity: 0;
  transform: translate(-50%, -50%) scale(0.95);
}

.disabled-fade-enter-active,
.disabled-fade-leave-active {
  transition: all 0.3s ease;
}

.disabled-fade-enter-from,
.disabled-fade-leave-to {
  opacity: 0;
  transform: translateX(20px);
}

.scroll-wait-fade-enter-active,
.scroll-wait-fade-leave-active {
  transition: all 0.4s ease;
}

.scroll-wait-fade-enter-from,
.scroll-wait-fade-leave-to {
  opacity: 0;
  transform: translate(-50%, -50%) scale(0.9);
}

/* 响应式 */
@media (max-width: 768px) {
  .loading-progress {
    top: 60px;
    min-width: 280px;
    padding: 16px 20px;
  }

  .completion-notification {
    padding: 24px;
    max-width: 360px;
  }

  .disabled-notification {
    bottom: 16px;
    right: 16px;
    left: 16px;
    max-width: none;
  }
}
</style>