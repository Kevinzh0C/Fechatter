/**
 * Message UI State Store - Unified Message UI State Management
 * Solves state fragmentation and conflict issues
 */

import { ref, readonly, computed } from 'vue'
import { defineStore } from 'pinia'
import { ZIndexManager, useZIndex } from '@/utils/ZIndexManager'

export const useMessageUIStore = defineStore('messageUI', () => {
  
  // ================================
  // Core State
  // ================================
  
  // Currently active context menu
  const activeContextMenu = ref(null)
  
  // Currently active translation panel
  const activeTranslationPanel = ref(null)
  
  // Currently active Bot panel
  const activeBotPanel = ref(null)
  
  // Modal stack
  const modalStack = ref([])
  
  // Message selection state
  const selectedMessages = ref(new Set())
  
  // Message editing state
  const editingMessage = ref(null)
  
  // ================================
  // Z-Index Integration
  // ================================
  
  const { allocate: allocateZIndex, release: releaseZIndex } = useZIndex()
  
  // ================================
  // Context Menu Management
  // ================================
  
  /**
   * Open context menu
   */
  const openContextMenu = (messageId, position, menuType = 'default', options = {}) => {
    // Close other UI panels to ensure context menu has priority
    closeAllPanels()
    
    // Allocate z-index
    const componentId = `context-menu-${messageId}`
    const zIndex = allocateZIndex(componentId, 'contextMenu')
    
    // Set menu state
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
   * Close context menu
   */
  const closeContextMenu = () => {
    if (activeContextMenu.value) {
      const { componentId } = activeContextMenu.value
      releaseZIndex(componentId)
      activeContextMenu.value = null
    }
  }
  
  // ================================
  // Translation Panel Management  
  // ================================
  
  /**
   * Open translation panel
   */
  const openTranslationPanel = (messageId, options = {}) => {
    // If same message, toggle state
    if (activeTranslationPanel.value?.messageId === messageId) {
      closeTranslationPanel()
      return
    }
    
    // Close other panels
    closeOtherPanels('translation')
    
    // Allocate z-index
    const componentId = `translation-panel-${messageId}`
    const zIndex = allocateZIndex(componentId, 'translation')
    
    // Set panel state
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
   * Close translation panel
   */
  const closeTranslationPanel = () => {
    if (activeTranslationPanel.value) {
      const { componentId } = activeTranslationPanel.value
      releaseZIndex(componentId)
      activeTranslationPanel.value = null
    }
  }
  
  // ================================
  // Bot Panel Management
  // ================================
  
  /**
   * Open Bot panel
   */
  const openBotPanel = (panelType, options = {}) => {
    // If same panel, toggle state
    if (activeBotPanel.value?.type === panelType) {
      closeBotPanel()
      return
    }
    
    // Close other panels
    closeOtherPanels('bot')
    
    // Allocate z-index
    const componentId = `bot-panel-${panelType}`
    const zIndex = allocateZIndex(componentId, 'botPanel')
    
    // Set panel state
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
   * Close Bot panel
   */
  const closeBotPanel = () => {
    if (activeBotPanel.value) {
      const { componentId } = activeBotPanel.value
      releaseZIndex(componentId)
      activeBotPanel.value = null
    }
  }
  
  // ================================
  // Panel Coordination
  // ================================
  
  /**
   * Close other panels (except specified type)
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
   * Close all panels
   */
  const closeAllPanels = () => {
    closeOtherPanels()
  }
  
  // ================================
  // Computed Properties
  // ================================
  
  // Whether there are active panels
  const hasActivePanels = computed(() => {
    return !!(
      activeContextMenu.value ||
      activeTranslationPanel.value ||
      activeBotPanel.value ||
      modalStack.value.length > 0
    )
  })
  
  // Selected message count
  const selectedCount = computed(() => {
    return selectedMessages.value.size
  })
  
  // ================================
  // Return API
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

// Global Debug Access
if (typeof window !== 'undefined' && import.meta.env.DEV) {
  window.messageUIStore = useMessageUIStore
}
