<!--
  TranslationPanel.vue
  Production-Grade Translation Panel Component
  
  Features:
  - Unified z-index management with ZIndexManager
  - Teleport to body for proper stacking
  - Dynamic positioning to prevent overlap
  - Professional UI with quotas and language selection
-->
<template>
  <teleport to="body">
    <div v-if="visible" class="translation-panel-overlay" :style="{ zIndex: panelZIndex - 1 }"
      @click="handleOverlayClick" @contextmenu.prevent>

      <div ref="panelRef" class="translation-panel" :style="panelStyle" :class="panelClasses" @click.stop
        @contextmenu.prevent>

        <!-- Panel Header -->
        <div class="panel-header">
          <div class="header-left">
            <span class="panel-icon">🌐</span>
            <span class="panel-title">Translate Message</span>
          </div>
          <div class="header-right">
            <span class="quota-info">
              {{ quotaInfo.remaining }}/{{ quotaInfo.limit }} left today
            </span>
            <button class="close-btn" @click="handleClose" :aria-label="'Close translation panel'">
              <XMarkIcon class="w-4 h-4" />
            </button>
          </div>
        </div>

        <!-- Message Preview -->
        <div class="message-preview">
          <div class="preview-label">Original Message:</div>
          <div class="preview-content">{{ truncatedContent }}</div>
          <!-- DEBUG: Show message info in development -->
          <div v-if="isDev" class="debug-info">
            <small>Debug: ID={{ message.id }}, Length={{ (message.content || '').length }}</small>
          </div>
        </div>

        <!-- Language Selection - Show only when not translating and no result -->
        <div v-if="!isTranslating && !translationResult" class="language-selection">
          <div class="selection-label">Translate to:</div>
          <div class="language-grid">
            <button v-for="lang in supportedLanguages" :key="lang.code" :class="['language-btn', {
              'is-selected': selectedLanguage === lang.code
            }]" @click="handleTranslate(lang.code)">
              <span class="lang-flag">{{ lang.flag }}</span>
              <span class="lang-name">{{ lang.name }}</span>
            </button>
          </div>
        </div>

        <!-- Loading State - Show when translating -->
        <div v-if="isTranslating" class="loading-state">
          <div class="loading-spinner"></div>
          <span>Translating to {{ getLanguageName(selectedLanguage) }}...</span>
          <div class="loading-details">
            <small>Processing your message...</small>
          </div>
        </div>

        <!-- Translation Result - Show when translation is complete -->
        <div v-if="!isTranslating && translationResult" class="translation-result">
          <div class="result-header">
            <span class="result-label">Translation:</span>
            <span class="source-lang">{{ translationResult.sourceLanguage }} → {{ translationResult.targetLanguage
              }}</span>
          </div>
          <div class="result-content">{{ translationResult.translation }}</div>
          <div class="result-actions">
            <button class="copy-btn" @click="copyTranslation">
              Copy Translation
            </button>
            <button class="apply-btn" @click="applyTranslation">
              Apply to Message
            </button>
            <button class="translate-again-btn" @click="handleTranslateAgain">
              Translate Again
            </button>
          </div>
          <!-- DEBUG: Show result info in development -->
          <div v-if="isDev" class="debug-info">
            <small>Debug Result: Confidence={{ translationResult.confidence }}</small>
          </div>
        </div>

        <!-- DEBUG: Show state info in development -->
        <div v-if="isDev" class="debug-state">
          <div class="debug-info">
            <small>
              State Debug: isTranslating={{ isTranslating }}, hasResult={{ !!translationResult }},
              selectedLang={{ selectedLanguage }}, quotaRemaining={{ quotaInfo.remaining }}
            </small>
          </div>
        </div>
      </div>
    </div>
  </teleport>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { XMarkIcon } from '@heroicons/vue/24/outline'
import { useMessageUIStore } from '@/stores/messageUI'
import { botService } from '@/services/botService'
import { useNotifications } from '@/composables/useNotifications'
import { useZIndex } from '@/utils/ZIndexManager'

// ================================
// Props & Emits
// ================================

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  message: {
    type: Object,
    required: true
  },
  position: {
    type: Object,
    default: () => ({ x: 0, y: 0 })
  }
})

const emit = defineEmits([
  'close',
  'translated',
  'applied'
])

// ================================
// Composables & Stores
// ================================

const messageUIStore = useMessageUIStore()
const { notifySuccess, notifyError } = useNotifications()
const { allocate: allocateZIndex, release: releaseZIndex } = useZIndex()

// ================================
// Reactive State
// ================================

const panelRef = ref(null)
const isTranslating = ref(false)
const translationResult = ref(null)
const selectedLanguage = ref('en')
const componentId = ref(`translation-panel-${props.message.id}`)

// ================================
// Computed Properties
// ================================

const panelZIndex = computed(() => {
  // Use store's z-index if available, otherwise allocate new one
  if (messageUIStore.activeTranslationPanel?.zIndex) {
    return messageUIStore.activeTranslationPanel.zIndex
  }

  // Fallback to manual allocation with high priority
  return allocateZIndex(componentId.value, 'translation', 8)
})

const quotaInfo = ref(botService.getQuotaInfo())

// FIXED: Reactive quota info that updates after translation
const updateQuotaInfo = () => {
  quotaInfo.value = botService.getQuotaInfo()
}

const truncatedContent = computed(() => {
  const content = props.message.content || ''
  return content.length > 100 ? content.substring(0, 100) + '...' : content
})

const isDev = computed(() => import.meta.env.DEV)

const panelStyle = computed(() => {
  // 优化定位逻辑：直接使用Chat.vue计算的最优位置
  const basePosition = props.position
  const panelWidth = 400
  const panelHeight = 500

  // 使用传入的优化位置，只做基础的边界检查
  let left = basePosition.x
  let top = basePosition.y

  // 最终边界安全检查
  if (typeof window !== 'undefined') {
    const viewportWidth = window.innerWidth
    const viewportHeight = window.innerHeight

    // 确保面板不会超出视口
    left = Math.max(10, Math.min(left, viewportWidth - panelWidth - 10))
    top = Math.max(10, Math.min(top, viewportHeight - panelHeight - 10))
  }

  return {
    position: 'fixed',
    left: `${left}px`,
    top: `${top}px`,
    zIndex: panelZIndex.value,
    width: '400px',
    maxHeight: '480px' // 稍微减小高度，给更多安全空间
  }
})

const panelClasses = computed(() => [
  'translation-panel',
  {
    'is-translating': isTranslating.value,
    'has-result': !!translationResult.value
  }
])

const supportedLanguages = [
  { code: 'en', name: 'English', flag: '🇺🇸' },
  { code: 'zh', name: '中文', flag: '🇨🇳' },
  { code: 'ja', name: '日本語', flag: '🇯🇵' },
  { code: 'ko', name: '한국어', flag: '🇰🇷' },
  { code: 'es', name: 'Español', flag: '🇪🇸' },
  { code: 'fr', name: 'Français', flag: '🇫🇷' },
  { code: 'de', name: 'Deutsch', flag: '🇩🇪' },
  { code: 'ru', name: 'Русский', flag: '🇷🇺' }
]

// ================================
// Event Handlers
// ================================

const handleOverlayClick = () => {
  handleClose()
}

const handleClose = () => {
  // Release Z-index allocation
  releaseZIndex(componentId.value)

  // Clear translation result
  translationResult.value = null

  // Close via store
  messageUIStore.closeTranslationPanel()

  emit('close')
}

const handleTranslate = async (targetLang) => {
  if (isTranslating.value || quotaInfo.value.remaining === 0) {
    if (quotaInfo.value.remaining === 0) {
      notifyError('Daily translation limit reached. Try again tomorrow.')
    }
    return
  }

  // ENHANCED: Immediate UI feedback
  selectedLanguage.value = targetLang
  translationResult.value = null

  // Small delay to show UI state change clearly
  await nextTick()
  isTranslating.value = true

  try {
    // FIX: Convert message ID to string to match backend API expectation
    const messageId = String(props.message.id)

    // CRITICAL FIX: Store message content globally for botService access
    if (typeof window !== 'undefined') {
      window.currentTranslatingMessage = {
        id: messageId,
        content: props.message.content || props.message.text || ''
      }

      if (import.meta.env.DEV) {
        console.log('🌐 [TranslationPanel] Stored message for translation:', {
          messageId,
          contentLength: window.currentTranslatingMessage.content.length,
          contentPreview: window.currentTranslatingMessage.content.substring(0, 50)
        })
      }
    }

    if (import.meta.env.DEV) {
      console.log('🌐 [TranslationPanel] Starting translation...', {
        messageId,
        targetLang,
        messageContent: props.message.content?.substring(0, 50)
      })
    }

    const result = await botService.translateMessage(messageId, targetLang)

    if (import.meta.env.DEV) {
      console.log('🌐 [TranslationPanel] Translation API response:', result)
    }

    // SIMPLIFIED: Use direct result properties (API response is already correct)
    const translationText = result.translation
    const sourceLanguage = result.source_language || 'auto-detected'
    const confidence = result.confidence || 0.95

    translationResult.value = {
      translation: translationText,
      sourceLanguage: sourceLanguage,
      targetLanguage: getLanguageName(targetLang),
      confidence: confidence
    }

    if (import.meta.env.DEV) {
      console.log('🌐 [TranslationPanel] Final translation result:', translationResult.value)
    }

    // FIXED: Use server-provided quota information and update UI
    const remainingQuota = result.quota_remaining !== undefined ? result.quota_remaining :
      (result.quota?.remaining !== undefined ? result.quota.remaining : 'unknown')

    // Update quota display immediately
    updateQuotaInfo()

    notifySuccess(`Translation completed. ${remainingQuota} translations left today.`)
    emit('translated', translationResult.value)

  } catch (error) {
    console.error('🚨 [TranslationPanel] Translation error:', error)
    notifyError(error.message || 'Failed to translate message')

    // DEBUG: Log detailed error information
    if (import.meta.env.DEV) {
      console.log('🚨 [TranslationPanel] Error details:', {
        error: error.message,
        response: error.response?.data,
        status: error.response?.status
      })
    }
  } finally {
    isTranslating.value = false

    // CLEANUP: Remove global message reference
    if (typeof window !== 'undefined') {
      delete window.currentTranslatingMessage
    }
  }
}

const copyTranslation = async () => {
  if (!translationResult.value) return

  try {
    await navigator.clipboard.writeText(translationResult.value.translation)
    notifySuccess('Translation copied to clipboard')
  } catch (error) {
    notifyError('Failed to copy translation')
  }
}

const applyTranslation = () => {
  if (!translationResult.value) return

  emit('applied', {
    messageId: props.message.id,
    translation: translationResult.value.translation,
    sourceLanguage: translationResult.value.sourceLanguage
  })

  handleClose()
}

const handleTranslateAgain = () => {
  // Reset state to allow new translation
  translationResult.value = null
  selectedLanguage.value = 'en'
  isTranslating.value = false

  if (import.meta.env.DEV) {
    console.log('🔄 [TranslationPanel] Reset for new translation')
  }

  // Update quota info
  updateQuotaInfo()
}

const getLanguageName = (code) => {
  const lang = supportedLanguages.find(l => l.code === code)
  return lang ? lang.name : code.toUpperCase()
}

// ================================
// Keyboard Handling
// ================================

const handleKeydown = (event) => {
  if (!props.visible) return

  switch (event.key) {
    case 'Escape':
      event.preventDefault()
      handleClose()
      break
  }
}

// ================================
// Lifecycle
// ================================

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)

  // Focus panel for keyboard navigation
  nextTick(() => {
    if (panelRef.value) {
      panelRef.value.focus()
    }
  })
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  releaseZIndex(componentId.value)
})

// Watch for visibility changes
watch(() => props.visible, (newVisible) => {
  if (newVisible) {
    // Reset state when panel opens
    translationResult.value = null
    isTranslating.value = false
    selectedLanguage.value = 'en'

    // FIXED: Update quota info when panel opens
    updateQuotaInfo()
  }
})
</script>

<style scoped>
/*
  Translation Panel Styles
  Production-grade design with perfect UX
*/

.translation-panel-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: transparent;
  z-index: var(--z-translation-panel-overlay, 2999);
}

.translation-panel {
  background: rgba(255, 255, 255, 0.98);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(0, 0, 0, 0.08);
  border-radius: 16px;
  box-shadow:
    0 12px 48px rgba(0, 0, 0, 0.15),
    0 4px 16px rgba(0, 0, 0, 0.1);
  overflow: hidden;
  outline: none;

  /* Typography */
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
  font-size: 14px;
  line-height: 1.5;

  /* Animation */
  animation: panelAppear 0.2s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  transform-origin: top left;
}

@keyframes panelAppear {
  from {
    opacity: 0;
    transform: scale(0.9) translateY(-8px);
  }

  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

/* Panel Header */
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.02);
  border-bottom: 1px solid rgba(0, 0, 0, 0.06);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.panel-icon {
  font-size: 20px;
}

.panel-title {
  font-weight: 600;
  color: var(--text-primary);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.quota-info {
  font-size: 12px;
  color: var(--text-muted);
  background: rgba(0, 0, 0, 0.04);
  padding: 4px 8px;
  border-radius: 6px;
}

.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: transparent;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.close-btn:hover {
  background-color: rgba(0, 0, 0, 0.06);
}

/* Message Preview */
.message-preview {
  padding: 16px 20px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.06);
}

.preview-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
  margin-bottom: 8px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.preview-content {
  background: rgba(0, 0, 0, 0.02);
  padding: 12px;
  border-radius: 8px;
  font-size: 13px;
  line-height: 1.4;
  color: var(--text-secondary);
}

/* Language Selection */
.language-selection {
  padding: 16px 20px;
  transition: all 0.3s ease;
}

.selection-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
  margin-bottom: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.language-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 8px;
}

.language-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background: transparent;
  border: 1px solid rgba(0, 0, 0, 0.1);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
  font-size: 13px;
}

.language-btn:hover:not(.is-disabled) {
  background-color: rgba(0, 122, 255, 0.08);
  border-color: rgba(0, 122, 255, 0.3);
}

.language-btn.is-selected {
  background-color: #007AFF;
  border-color: #007AFF;
  color: white;
}

.language-btn.is-disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.lang-flag {
  font-size: 16px;
}

.lang-name {
  font-weight: 500;
}

/* Translation Result */
.translation-result {
  padding: 16px 20px;
  border-top: 1px solid rgba(0, 0, 0, 0.06);
  background: rgba(0, 122, 255, 0.02);
  animation: resultAppear 0.4s ease-out;
}

@keyframes resultAppear {
  from {
    opacity: 0;
    transform: translateY(10px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.result-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.result-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.source-lang {
  font-size: 11px;
  color: var(--text-muted);
  background: rgba(0, 0, 0, 0.04);
  padding: 2px 6px;
  border-radius: 4px;
}

.result-content {
  background: white;
  padding: 16px;
  border-radius: 8px;
  margin-bottom: 12px;
  font-size: 14px;
  line-height: 1.5;
  border: 1px solid rgba(0, 0, 0, 0.06);
}

.result-actions {
  display: flex;
  gap: 8px;
}

.copy-btn,
.apply-btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.copy-btn {
  background: transparent;
  border: 1px solid rgba(0, 0, 0, 0.2);
  color: var(--text-primary);
}

.copy-btn:hover {
  background-color: rgba(0, 0, 0, 0.04);
}

.apply-btn {
  background: #007AFF;
  border: 1px solid #007AFF;
  color: white;
}

.apply-btn:hover {
  background: #0056CC;
  border-color: #0056CC;
}

.translate-again-btn {
  background: transparent;
  border: 1px solid #007AFF;
  color: #007AFF;
}

.translate-again-btn:hover {
  background: rgba(0, 122, 255, 0.08);
  border-color: #0056CC;
  color: #0056CC;
}

/* Loading State */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 32px 20px;
  color: var(--text-muted);
  background: rgba(0, 122, 255, 0.02);
  border-top: 1px solid rgba(0, 0, 0, 0.06);
  text-align: center;
  animation: loadingAppear 0.3s ease-out;
}

@keyframes loadingAppear {
  from {
    opacity: 0;
    transform: scale(0.95);
  }

  to {
    opacity: 1;
    transform: scale(1);
  }
}

.loading-state>span {
  font-weight: 500;
  color: #007AFF;
  font-size: 15px;
}

.loading-details {
  margin-top: 4px;
}

.loading-details small {
  color: var(--text-muted);
  font-size: 12px;
}

.loading-spinner {
  width: 20px;
  height: 20px;
  border: 3px solid rgba(0, 122, 255, 0.2);
  border-top: 3px solid #007AFF;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* States */
.is-translating {
  pointer-events: none;
}

.has-result .language-selection {
  border-bottom: 1px solid rgba(0, 0, 0, 0.06);
}

/* Dark Mode */
@media (prefers-color-scheme: dark) {
  .translation-panel {
    background: rgba(28, 28, 30, 0.98);
    border-color: rgba(255, 255, 255, 0.12);
    box-shadow:
      0 12px 48px rgba(0, 0, 0, 0.4),
      0 4px 16px rgba(0, 0, 0, 0.3);
  }

  .panel-header {
    background: rgba(255, 255, 255, 0.02);
    border-bottom-color: rgba(255, 255, 255, 0.06);
  }

  .result-content {
    background: rgba(255, 255, 255, 0.04);
    border-color: rgba(255, 255, 255, 0.12);
  }
}

/* High Contrast Mode */
@media (prefers-contrast: high) {
  .translation-panel {
    border-width: 2px;
    border-color: currentColor;
  }
}

/* Reduced Motion */
@media (prefers-reduced-motion: reduce) {
  .translation-panel {
    animation: none;
  }

  .loading-spinner {
    animation: none;
  }
}

/* Focus Management */
.translation-panel:focus-visible {
  outline: 2px solid #007AFF;
  outline-offset: -2px;
}

/* DEBUG: Debug styles */
.debug-info {
  margin-top: 8px;
  padding: 4px 8px;
  background: rgba(255, 0, 0, 0.1);
  border: 1px solid rgba(255, 0, 0, 0.2);
  border-radius: 4px;
  font-family: monospace;
  font-size: 11px;
  color: #666;
}

.debug-state {
  padding: 8px 16px;
  background: rgba(0, 0, 255, 0.05);
  border-top: 1px solid rgba(0, 0, 255, 0.1);
}
</style>