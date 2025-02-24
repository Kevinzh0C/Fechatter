/**
 * 🎯 Message UI State Store - 消息UI统一状态管理
 * 解决状态分散、冲突等问题
 */

import { ref, readonly, computed } from 'vue'
import { defineStore } from 'pinia'
import { ZIndexManager, useZIndex } from '@/utils/ZIndexManager'

export const useMessageUIStore = defineStore('messageUI', () => {
  
  // ================================
  // 🎯 Core State
  // ================================
  
  // 当前活跃的上下文菜单
  const activeContextMenu = ref(null)
  
  // 当前活跃的翻译面板
  const activeTranslationPanel = ref(null)
  
  // 当前活跃的Bot面板
  const activeBotPanel = ref(null)
  
  // 模态框堆栈
  const modalStack = ref([])
  
  // 消息选择状态
  const selectedMessages = ref(new Set())
  
  // 消息编辑状态
  const editingMessage = ref(null)
  
  // ================================
  // 🎯 Z-Index Integration
  // ================================
  
  const { allocate: allocateZIndex, release: releaseZIndex } = useZIndex()
  
  // ================================
  // 🎯 Context Menu Management
  // ================================
  
  /**
   * 打开上下文菜单
   */
  const openContextMenu = (messageId, position, menuType = 'default', options = {}) => {
    // 关闭其他UI面板，确保上下文菜单优先显示
    closeAllPanels()
    
    // 分配z-index
    const componentId = `context-menu-${messageId}`
    const zIndex = allocateZIndex(componentId, 'contextMenu')
    
    // 设置菜单状态
    activeContextMenu.value = {
      messageId,
      position: { ...position },
      menuType,
      zIndex,
      componentId,
      timestamp: Date.now(),
      options: { ...options }
    }
  }
  
  /**
   * 关闭上下文菜单
   */
  const closeContextMenu = () => {
    if (activeContextMenu.value) {
      const { componentId } = activeContextMenu.value
      releaseZIndex(componentId)
      activeContextMenu.value = null
    }
  }
  
  // ================================
  // 🎯 Translation Panel Management  
  // ================================
  
  /**
   * 打开翻译面板
   */
  const openTranslationPanel = (messageId, options = {}) => {
    // 如果是同一个消息，则切换状态
    if (activeTranslationPanel.value?.messageId === messageId) {
      closeTranslationPanel()
      return
    }
    
    // 关闭其他面板
    closeOtherPanels('translation')
    
    // 分配z-index
    const componentId = `translation-panel-${messageId}`
    const zIndex = allocateZIndex(componentId, 'translation')
    
    // 设置面板状态
    activeTranslationPanel.value = {
      messageId,
      zIndex,
      componentId,
      timestamp: Date.now(),
      options: {
        showAdvanced: false,
        preserveFormatting: true,
        showConfidence: true,
        ...options
      }
    }
  }
  
  /**
   * 关闭翻译面板
   */
  const closeTranslationPanel = () => {
    if (activeTranslationPanel.value) {
      const { componentId } = activeTranslationPanel.value
      releaseZIndex(componentId)
      activeTranslationPanel.value = null
    }
  }
  
  // ================================
  // 🎯 Bot Panel Management
  // ================================
  
  /**
   * 打开Bot面板
   */
  const openBotPanel = (panelType, options = {}) => {
    // 如果是同一个面板，则切换状态
    if (activeBotPanel.value?.type === panelType) {
      closeBotPanel()
      return
    }
    
    // 关闭其他面板
    closeOtherPanels('bot')
    
    // 分配z-index
    const componentId = `bot-panel-${panelType}`
    const zIndex = allocateZIndex(componentId, 'botPanel')
    
    // 设置面板状态
    activeBotPanel.value = {
      type: panelType,
      zIndex,
      componentId,
      timestamp: Date.now(),
      options: {
        width: 400,
        height: 600,
        position: 'right',
        ...options
      }
    }
  }
  
  /**
   * 关闭Bot面板
   */
  const closeBotPanel = () => {
    if (activeBotPanel.value) {
      const { componentId } = activeBotPanel.value
      releaseZIndex(componentId)
      activeBotPanel.value = null
    }
  }
  
  // ================================
  // 🎯 Panel Coordination
  // ================================
  
  /**
   * 关闭其他面板（除了指定类型）
   */
  const closeOtherPanels = (exceptType = null) => {
    if (exceptType !== 'context') {
      closeContextMenu()
    }
    
    if (exceptType !== 'translation') {
      closeTranslationPanel()
    }
    
    if (exceptType !== 'bot') {
      closeBotPanel()
    }
  }
  
  /**
   * 关闭所有面板
   */
  const closeAllPanels = () => {
    closeOtherPanels()
  }
  
  // ================================
  // 🎯 Computed Properties
  // ================================
  
  // 是否有活跃的面板
  const hasActivePanels = computed(() => {
    return !!(
      activeContextMenu.value ||
      activeTranslationPanel.value ||
      activeBotPanel.value ||
      modalStack.value.length > 0
    )
  })
  
  // 选中消息数量
  const selectedCount = computed(() => {
    return selectedMessages.value.size
  })
  
  // ================================
  // 🎯 Return API
  // ================================
  
  return {
    // State (readonly)
    activeContextMenu: readonly(activeContextMenu),
    activeTranslationPanel: readonly(activeTranslationPanel),
    activeBotPanel: readonly(activeBotPanel),
    modalStack: readonly(modalStack),
    
    // Computed
    hasActivePanels,
    selectedCount,
    
    // Context Menu Actions
    openContextMenu,
    closeContextMenu,
    
    // Translation Panel Actions
    openTranslationPanel,
    closeTranslationPanel,
    
    // Bot Panel Actions
    openBotPanel,
    closeBotPanel,
    
    // Panel Coordination
    closeOtherPanels,
    closeAllPanels
  }
})

// 🌐 Global Debug Access
if (typeof window !== 'undefined' && import.meta.env.DEV) {
  window.messageUIStore = useMessageUIStore
}
