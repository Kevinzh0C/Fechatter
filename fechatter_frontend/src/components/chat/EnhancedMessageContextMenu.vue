<!--
  EnhancedMessageContextMenu.vue
  Production-Grade Enhanced Message Context Menu
  
  Features:
  - 完整的消息操作支持
  - 智能Z-Index管理
  - Bot功能集成
  - 翻译功能集成
  - 无缝用户体验
-->
<template>
  <teleport to="body">
    <div v-if="visible" 
         class="enhanced-context-menu-overlay" 
         :style="{ zIndex: menuZIndex - 1 }"
         @click="handleOverlayClick"
         @contextmenu.prevent>
      
      <div ref="menuRef" 
           class="enhanced-context-menu" 
           :style="menuStyle"
           :class="menuClasses"
           @click.stop
           @contextmenu.prevent>
        
        <!-- 基础操作区 -->
        <div class="menu-section">
          <MenuItem 
            icon="reply" 
            label="Reply"
            shortcut="R"
            @click="handleReply" 
          />
          
          <MenuItem 
            v-if="canEdit"
            icon="edit" 
            label="Edit"
            shortcut="E"
            @click="handleEdit" 
          />
          
          <MenuItem 
            icon="copy" 
            label="Copy Message"
            shortcut="⌘C"
            @click="handleCopy" 
          />
          
          <MenuItem 
            icon="link" 
            label="Copy Link"
            @click="handleCopyLink" 
          />
        </div>
        
        <!-- 翻译操作区 -->
        <div class="menu-section" v-if="hasTranslationCapability">
          <MenuItem 
            icon="language" 
            label="Translate"
            :badge="quotaInfo.remaining > 0 ? quotaInfo.remaining : null"
            :disabled="quotaInfo.remaining === 0"
            @click="handleTranslate" 
          />
          
          <MenuItem 
            v-if="showTranslationSettings"
            icon="settings" 
            label="Translation Settings"
            @click="handleTranslateSettings" 
          />
        </div>
        
        <!-- Bot操作区 -->
        <div class="menu-section" v-if="hasBotCapability">
          <MenuItem 
            icon="brain" 
            label="AI Analyze"
            :loading="isAnalyzing"
            @click="handleBotAnalyze" 
          />
          
          <MenuItem 
            icon="sparkles" 
            label="AI Summarize"
            :loading="isSummarizing"
            @click="handleBotSummarize" 
          />
          
          <MenuItem 
            icon="robot" 
            label="Custom Bot Actions"
            @click="handleCustomBot" 
          />
        </div>
        
        <!-- 高级操作区 -->
        <div class="menu-section" v-if="hasAdvancedActions">
          <MenuItem 
            icon="forward" 
            label="Forward"
            @click="handleForward" 
          />
          
          <MenuItem 
            icon="bookmark" 
            label="Bookmark"
            @click="handleBookmark" 
          />
          
          <MenuItem 
            icon="flag" 
            label="Report"
            @click="handleReport" 
          />
        </div>
        
        <!-- 危险操作区 -->
        <div class="menu-section danger-section" v-if="hasDangerousActions">
          <MenuItem 
            v-if="canDelete"
            icon="trash" 
            label="Delete"
            variant="danger"
            @click="handleDelete" 
          />
        </div>
      </div>
    </div>
  </teleport>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { useMessageUIStore } from '@/stores/messageUI'
import { useAuthStore } from '@/stores/auth'
import { botService } from '@/services/botService'
import { useNotifications } from '@/composables/useNotifications'
import MenuItem from '@/components/ui/MenuItem.vue'

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
    required: true,
    validator: (value) => value && typeof value.x === 'number' && typeof value.y === 'number'
  },
  menuType: {
    type: String,
    default: 'default',
    validator: (value) => ['default', 'compact', 'admin'].includes(value)
  },
  capabilities: {
    type: Object,
    default: () => ({
      translation: true,
      bot: true,
      advanced: true,
      dangerous: true
    })
  }
})

const emit = defineEmits([
  'reply',
  'edit', 
  'delete',
  'forward',
  'copy',
  'close',
  'action'
])

// ================================
// Composables & Stores
// ================================

const messageUIStore = useMessageUIStore()
const authStore = useAuthStore()
const { notifySuccess, notifyError, notifyInfo } = useNotifications()

// ================================
// Reactive State
// ================================

const menuRef = ref(null)
const isAnalyzing = ref(false)
const isSummarizing = ref(false)
const analysisResult = ref(null)
const summaryResult = ref(null)

// ================================
// Computed Properties
// ================================

// 用户权限检查
const currentUser = computed(() => authStore.user)
const isOwnMessage = computed(() => {
  return props.message.sender?.id === currentUser.value?.id
})

const canEdit = computed(() => {
  return isOwnMessage.value && !props.message.is_deleted
})

const canDelete = computed(() => {
  return isOwnMessage.value || authStore.hasPermission('delete_messages')
})

// 功能能力检查
const hasTranslationCapability = computed(() => {
  return props.capabilities.translation && props.message.content?.trim()
})

const hasBotCapability = computed(() => {
  return props.capabilities.bot && props.message.content?.trim()
})

const hasAdvancedActions = computed(() => {
  return props.capabilities.advanced
})

const hasDangerousActions = computed(() => {
  return props.capabilities.dangerous
})

// 翻译相关
const quotaInfo = computed(() => {
  return botService.getQuotaInfo()
})

const showTranslationSettings = computed(() => {
  return authStore.hasPermission('manage_translation_settings')
})

// 菜单样式计算
const menuZIndex = computed(() => {
  return messageUIStore.activeContextMenu?.zIndex || 3000
})

const menuStyle = computed(() => {
  const { x, y } = props.position
  const estimatedWidth = 240
  const estimatedHeight = getEstimatedMenuHeight()
  
  let left = x
  let top = y
  
  // 边界检测和调整
  if (typeof window !== 'undefined') {
    const viewportWidth = window.innerWidth
    const viewportHeight = window.innerHeight
    
    // 右边界检测
    if (left + estimatedWidth > viewportWidth - 20) {
      left = x - estimatedWidth
    }
    
    // 下边界检测
    if (top + estimatedHeight > viewportHeight - 20) {
      top = y - estimatedHeight
    }
    
    // 确保不超出左边界和上边界
    left = Math.max(10, left)
    top = Math.max(10, top)
  }
  
  return {
    left: `${left}px`,
    top: `${top}px`,
    zIndex: menuZIndex.value
  }
})

const menuClasses = computed(() => [
  'enhanced-context-menu',
  `menu-type-${props.menuType}`,
  {
    'has-analysis': analysisResult.value,
    'has-summary': summaryResult.value,
    'is-loading': isAnalyzing.value || isSummarizing.value
  }
])

// ================================
// Menu Estimation
// ================================

const getEstimatedMenuHeight = () => {
  let itemCount = 3 // 基础操作：Reply, Edit, Copy
  
  if (hasTranslationCapability.value) {
    itemCount += showTranslationSettings.value ? 2 : 1
  }
  
  if (hasBotCapability.value) {
    itemCount += 3
  }
  
  if (hasAdvancedActions.value) {
    itemCount += 3
  }
  
  if (hasDangerousActions.value && canDelete.value) {
    itemCount += 1
  }
  
  // 每个项目约40px高度，加上分组间距
  const itemHeight = 40
  const sectionSpacing = 8
  const sectionCount = [
    true, // 基础操作
    hasTranslationCapability.value,
    hasBotCapability.value,
    hasAdvancedActions.value,
    hasDangerousActions.value && canDelete.value
  ].filter(Boolean).length
  
  return (itemCount * itemHeight) + (sectionCount * sectionSpacing) + 16 // padding
}

// ================================
// Event Handlers
// ================================

const handleOverlayClick = () => {
  handleClose()
}

const handleClose = () => {
  messageUIStore.closeContextMenu()
  emit('close')
}

// 基础操作
const handleReply = () => {
  emit('reply', props.message)
  emit('action', { type: 'reply', message: props.message })
  handleClose()
}

const handleEdit = () => {
  if (!canEdit.value) return
  
  emit('edit', props.message)
  emit('action', { type: 'edit', message: props.message })
  handleClose()
}

const handleCopy = async () => {
  try {
    await navigator.clipboard.writeText(props.message.content)
    notifySuccess('Message copied to clipboard')
    emit('copy', props.message)
    emit('action', { type: 'copy', message: props.message })
  } catch (error) {
    notifyError('Failed to copy message')
  }
  handleClose()
}

const handleCopyLink = async () => {
  try {
    const url = `${window.location.origin}${window.location.pathname}?message=${props.message.id}`
    await navigator.clipboard.writeText(url)
    notifySuccess('Message link copied to clipboard')
    emit('action', { type: 'copy-link', message: props.message })
  } catch (error) {
    notifyError('Failed to copy message link')
  }
  handleClose()
}

// 翻译操作
const handleTranslate = () => {
  if (quotaInfo.value.remaining === 0) {
    notifyError('Daily translation limit reached. Try again tomorrow.')
    return
  }
  
  // 使用统一状态管理打开翻译面板
  messageUIStore.openTranslationPanel(props.message.id)
  emit('action', { type: 'translate', message: props.message })
  handleClose()
}

const handleTranslateSettings = () => {
  messageUIStore.openBotPanel('translation', { 
    mode: 'settings',
    messageId: props.message.id 
  })
  emit('action', { type: 'translation-settings', message: props.message })
  handleClose()
}

// Bot操作
const handleBotAnalyze = async () => {
  if (isAnalyzing.value) return
  
  isAnalyzing.value = true
  try {
    const result = await botService.analyzeMessage(props.message.id, 'comprehensive')
    analysisResult.value = result
    
    notifySuccess('AI analysis completed')
    emit('action', { 
      type: 'bot-analyze', 
      message: props.message, 
      result 
    })
    
    // 可以选择显示结果面板或直接关闭
    handleClose()
    
  } catch (error) {
    notifyError(`AI analysis failed: ${error.message}`)
  } finally {
    isAnalyzing.value = false
  }
}

const handleBotSummarize = async () => {
  if (isSummarizing.value) return
  
  isSummarizing.value = true
  try {
    const result = await botService.summarizeMessage(props.message.id, 'brief')
    summaryResult.value = result
    
    notifySuccess('AI summary completed')
    emit('action', { 
      type: 'bot-summarize', 
      message: props.message, 
      result 
    })
    
    handleClose()
    
  } catch (error) {
    notifyError(`AI summarization failed: ${error.message}`)
  } finally {
    isSummarizing.value = false
  }
}

const handleCustomBot = () => {
  messageUIStore.openBotPanel('custom', { 
    messageId: props.message.id,
    action: 'analyze'
  })
  emit('action', { type: 'custom-bot', message: props.message })
  handleClose()
}

// 高级操作
const handleForward = () => {
  emit('forward', props.message)
  emit('action', { type: 'forward', message: props.message })
  handleClose()
}

const handleBookmark = () => {
  // TODO: 实现书签功能
  notifyInfo('Bookmark feature coming soon')
  emit('action', { type: 'bookmark', message: props.message })
  handleClose()
}

const handleReport = () => {
  // TODO: 实现举报功能
  notifyInfo('Report feature coming soon')
  emit('action', { type: 'report', message: props.message })
  handleClose()
}

// 危险操作
const handleDelete = () => {
  if (!canDelete.value) return
  
  emit('delete', props.message)
  emit('action', { type: 'delete', message: props.message })
  handleClose()
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
      
    case 'r':
    case 'R':
      if (!event.ctrlKey && !event.metaKey) {
        event.preventDefault()
        handleReply()
      }
      break
      
    case 'e':
    case 'E':
      if (!event.ctrlKey && !event.metaKey && canEdit.value) {
        event.preventDefault()
        handleEdit()
      }
      break
      
    case 'c':
    case 'C':
      if (event.ctrlKey || event.metaKey) {
        event.preventDefault()
        handleCopy()
      }
      break
  }
}

// ================================
// Lifecycle
// ================================

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
  
  // 聚焦菜单以便键盘导航
  nextTick(() => {
    if (menuRef.value) {
      menuRef.value.focus()
    }
  })
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})

// 监听visible变化
watch(() => props.visible, (newVisible) => {
  if (newVisible) {
    // 重置状态
    analysisResult.value = null
    summaryResult.value = null
    isAnalyzing.value = false
    isSummarizing.value = false
  }
})
</script>

<style scoped>
/*
  Enhanced Context Menu Styles
  Production-grade design with perfect UX
*/

.enhanced-context-menu-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: transparent;
  z-index: var(--z-context-menu-overlay, 2999);
}

.enhanced-context-menu {
  position: fixed;
  min-width: 240px;
  max-width: 320px;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(0, 0, 0, 0.08);
  border-radius: 12px;
  box-shadow: 
    0 8px 32px rgba(0, 0, 0, 0.12),
    0 2px 8px rgba(0, 0, 0, 0.08);
  padding: 8px;
  z-index: var(--z-context-menu, 3000);
  overflow: hidden;
  outline: none;
  
  /* Typography */
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
  font-size: 14px;
  line-height: 1.4;
  
  /* Animation */
  animation: contextMenuAppear 0.15s ease-out;
  transform-origin: top left;
}

@keyframes contextMenuAppear {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(-4px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

/* Menu Type Variants */
.menu-type-compact {
  min-width: 200px;
  padding: 6px;
}

.menu-type-admin {
  border-color: rgba(255, 107, 107, 0.2);
  box-shadow: 
    0 8px 32px rgba(255, 107, 107, 0.15),
    0 2px 8px rgba(0, 0, 0, 0.08);
}

/* Menu Sections */
.menu-section {
  margin-bottom: 6px;
}

.menu-section:last-child {
  margin-bottom: 0;
}

.danger-section {
  border-top: 1px solid rgba(255, 107, 107, 0.2);
  margin-top: 8px;
  padding-top: 6px;
}

/* States */
.is-loading {
  pointer-events: none;
}

.has-analysis .menu-section:first-child::after,
.has-summary .menu-section:first-child::after {
  content: '';
  display: block;
  height: 1px;
  background: rgba(0, 122, 255, 0.2);
  margin: 6px 0;
}

/* Dark Mode */
@media (prefers-color-scheme: dark) {
  .enhanced-context-menu {
    background: rgba(28, 28, 30, 0.95);
    border-color: rgba(255, 255, 255, 0.12);
    box-shadow: 
      0 8px 32px rgba(0, 0, 0, 0.3),
      0 2px 8px rgba(0, 0, 0, 0.2);
  }
  
  .menu-type-admin {
    border-color: rgba(255, 107, 107, 0.3);
    box-shadow: 
      0 8px 32px rgba(255, 107, 107, 0.2),
      0 2px 8px rgba(0, 0, 0, 0.2);
  }
  
  .danger-section {
    border-top-color: rgba(255, 107, 107, 0.3);
  }
}

/* High Contrast Mode */
@media (prefers-contrast: high) {
  .enhanced-context-menu {
    border-width: 2px;
    border-color: currentColor;
  }
}

/* Reduced Motion */
@media (prefers-reduced-motion: reduce) {
  .enhanced-context-menu {
    animation: none;
  }
}

/* Focus Management */
.enhanced-context-menu:focus-visible {
  outline: 2px solid #007AFF;
  outline-offset: -2px;
}
</style> 