<!--
  TranslationPanelDebug.vue
  Debug component to verify translation panel fixes
-->
<template>
  <div class="translation-debug">
    <div class="debug-header">
      <h3>🌐 Translation Panel Debug</h3>
      <button @click="visible = !visible" class="toggle-btn">
        {{ visible ? 'Hide' : 'Show' }} Debug Panel
      </button>
    </div>
    
    <div v-if="visible" class="debug-content">
      <!-- State Information -->
      <div class="debug-section">
        <h4>State Information</h4>
        <div class="state-grid">
          <div class="state-item">
            <label>Active Translation Panel:</label>
            <span>{{ activePanel ? `Message ${activePanel.messageId}` : 'None' }}</span>
          </div>
          <div class="state-item">
            <label>Z-Index:</label>
            <span>{{ activePanel?.zIndex || 'Not set' }}</span>
          </div>
          <div class="state-item">
            <label>Component ID:</label>
            <span>{{ activePanel?.componentId || 'None' }}</span>
          </div>
          <div class="state-item">
            <label>Position:</label>
            <span>{{ activePanel?.options?.position ? `(${activePanel.options.position.x}, ${activePanel.options.position.y})` : 'Not set' }}</span>
          </div>
        </div>
      </div>
      
      <!-- Z-Index Layer Information -->
      <div class="debug-section">
        <h4>Z-Index Layer Information</h4>
        <div class="layer-info">
          <div v-for="(zIndex, layer) in zIndexLayers" :key="layer" class="layer-item">
            <span class="layer-name">{{ layer }}:</span>
            <span class="layer-value">{{ zIndex }}</span>
          </div>
        </div>
      </div>
      
      <!-- Active Allocations -->
      <div class="debug-section">
        <h4>Active Z-Index Allocations</h4>
        <div class="allocations-list">
          <div v-for="(allocation, id) in activeAllocations" :key="id" class="allocation-item">
            <div class="allocation-id">{{ id }}</div>
            <div class="allocation-details">
              <span>Layer: {{ allocation.layer }}</span>
              <span>Z-Index: {{ allocation.zIndex }}</span>
              <span>Active: {{ allocation.active }}</span>
            </div>
          </div>
          <div v-if="Object.keys(activeAllocations).length === 0" class="no-allocations">
            No active allocations
          </div>
        </div>
      </div>
      
      <!-- Actions -->
      <div class="debug-section">
        <h4>Debug Actions</h4>
        <div class="action-buttons">
          <button @click="testTranslationPanel" class="action-btn">
            Test Translation Panel
          </button>
          <button @click="forceCloseAll" class="action-btn danger">
            Force Close All Panels
          </button>
          <button @click="refreshDebugInfo" class="action-btn">
            Refresh Debug Info
          </button>
        </div>
      </div>
      
      <!-- Test Results -->
      <div v-if="testResults.length > 0" class="debug-section">
        <h4>Test Results</h4>
        <div class="test-results">
          <div v-for="(result, index) in testResults" :key="index" 
               :class="['test-result', result.status]">
            <span class="test-name">{{ result.name }}</span>
            <span class="test-status">{{ result.status }}</span>
            <span class="test-message">{{ result.message }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useMessageUIStore } from '@/stores/messageUI'
import { ZIndexManager, useZIndex } from '@/utils/ZIndexManager'

// State
const visible = ref(false)
const testResults = ref([])

// Store
const messageUIStore = useMessageUIStore()
const { layers } = useZIndex()

// Computed
const activePanel = computed(() => messageUIStore.activeTranslationPanel)

const zIndexLayers = computed(() => layers)

const activeAllocations = computed(() => {
  const layerInfo = ZIndexManager.getLayerInfo()
  return layerInfo.allocations
})

// Methods
const testTranslationPanel = () => {
  testResults.value = []
  
  // Test 1: Z-Index Management
  const testMessage = { id: 'test-123', content: 'Test message for translation' }
  
  try {
    messageUIStore.openTranslationPanel(testMessage.id, {
      position: { x: 100, y: 100 }
    })
    
    testResults.value.push({
      name: 'Open Translation Panel',
      status: 'pass',
      message: 'Panel opened successfully'
    })
    
    // Test Z-Index allocation
    const panel = messageUIStore.activeTranslationPanel
    if (panel && panel.zIndex >= 5000) {
      testResults.value.push({
        name: 'Z-Index Allocation',
        status: 'pass',
        message: `Z-Index ${panel.zIndex} is correct (>= 5000)`
      })
    } else {
      testResults.value.push({
        name: 'Z-Index Allocation',
        status: 'fail',
        message: `Z-Index ${panel?.zIndex} is too low`
      })
    }
    
    // Test component ID
    if (panel && panel.componentId) {
      testResults.value.push({
        name: 'Component ID',
        status: 'pass',
        message: `Component ID: ${panel.componentId}`
      })
    } else {
      testResults.value.push({
        name: 'Component ID',
        status: 'fail',
        message: 'Component ID not set'
      })
    }
    
    // Clean up
    setTimeout(() => {
      messageUIStore.closeTranslationPanel()
      testResults.value.push({
        name: 'Close Translation Panel',
        status: 'pass',
        message: 'Panel closed successfully'
      })
    }, 1000)
    
  } catch (error) {
    testResults.value.push({
      name: 'Translation Panel Test',
      status: 'fail',
      message: error.message
    })
  }
}

const forceCloseAll = () => {
  try {
    messageUIStore.closeAllPanels()
    ZIndexManager.emergencyReset()
    
    testResults.value.push({
      name: 'Force Close All',
      status: 'pass',
      message: 'All panels closed and Z-Index reset'
    })
  } catch (error) {
    testResults.value.push({
      name: 'Force Close All',
      status: 'fail',
      message: error.message
    })
  }
}

const refreshDebugInfo = () => {
  // Force reactivity update
  testResults.value = []
}

onMounted(() => {
  console.log('Translation Panel Debug component mounted')
})
</script>

<style scoped>
.translation-debug {
  position: fixed;
  top: 10px;
  right: 10px;
  width: 400px;
  max-height: 80vh;
  overflow-y: auto;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(0, 0, 0, 0.1);
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
  z-index: 15000; /* Higher than translation panel for debugging */
  font-family: monospace;
  font-size: 12px;
}

.debug-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: rgba(0, 0, 0, 0.05);
  border-bottom: 1px solid rgba(0, 0, 0, 0.1);
}

.debug-header h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
}

.toggle-btn {
  padding: 4px 8px;
  background: #007AFF;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 11px;
}

.debug-content {
  padding: 16px;
}

.debug-section {
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.1);
}

.debug-section:last-child {
  border-bottom: none;
}

.debug-section h4 {
  margin: 0 0 8px 0;
  font-size: 13px;
  font-weight: 600;
  color: #333;
}

.state-grid {
  display: grid;
  gap: 6px;
}

.state-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 8px;
  background: rgba(0, 0, 0, 0.02);
  border-radius: 4px;
}

.state-item label {
  font-weight: 500;
  color: #666;
}

.state-item span {
  color: #333;
  font-family: monospace;
}

.layer-info {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 4px;
}

.layer-item {
  display: flex;
  justify-content: space-between;
  padding: 2px 6px;
  background: rgba(0, 122, 255, 0.05);
  border-radius: 3px;
}

.layer-name {
  font-weight: 500;
  color: #007AFF;
}

.layer-value {
  color: #333;
  font-weight: 600;
}

.allocations-list {
  max-height: 150px;
  overflow-y: auto;
}

.allocation-item {
  padding: 6px 8px;
  margin-bottom: 4px;
  background: rgba(0, 0, 0, 0.02);
  border-radius: 4px;
  border-left: 3px solid #007AFF;
}

.allocation-id {
  font-weight: 600;
  color: #333;
  margin-bottom: 2px;
}

.allocation-details {
  display: flex;
  gap: 8px;
  font-size: 11px;
  color: #666;
}

.no-allocations {
  text-align: center;
  color: #999;
  font-style: italic;
  padding: 12px;
}

.action-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.action-btn {
  padding: 6px 12px;
  background: #f0f0f0;
  border: 1px solid #ccc;
  border-radius: 4px;
  cursor: pointer;
  font-size: 11px;
  transition: all 0.2s ease;
}

.action-btn:hover {
  background: #e0e0e0;
}

.action-btn.danger {
  background: #ff6b6b;
  color: white;
  border-color: #ff6b6b;
}

.action-btn.danger:hover {
  background: #ff5252;
}

.test-results {
  max-height: 150px;
  overflow-y: auto;
}

.test-result {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  gap: 8px;
  padding: 6px 8px;
  margin-bottom: 4px;
  border-radius: 4px;
  font-size: 11px;
}

.test-result.pass {
  background: rgba(76, 175, 80, 0.1);
  border-left: 3px solid #4CAF50;
}

.test-result.fail {
  background: rgba(244, 67, 54, 0.1);
  border-left: 3px solid #F44336;
}

.test-name {
  font-weight: 500;
  color: #333;
}

.test-status {
  padding: 2px 6px;
  border-radius: 3px;
  font-weight: 600;
  font-size: 10px;
  text-transform: uppercase;
}

.test-result.pass .test-status {
  background: #4CAF50;
  color: white;
}

.test-result.fail .test-status {
  background: #F44336;
  color: white;
}

.test-message {
  color: #666;
  font-size: 10px;
}

/* Dark mode */
@media (prefers-color-scheme: dark) {
  .translation-debug {
    background: rgba(28, 28, 30, 0.95);
    border-color: rgba(255, 255, 255, 0.2);
  }
  
  .debug-header {
    background: rgba(255, 255, 255, 0.05);
    border-bottom-color: rgba(255, 255, 255, 0.1);
  }
  
  .debug-header h3 {
    color: #fff;
  }
  
  .state-item {
    background: rgba(255, 255, 255, 0.02);
  }
  
  .state-item label {
    color: #ccc;
  }
  
  .state-item span {
    color: #fff;
  }
}
</style> 