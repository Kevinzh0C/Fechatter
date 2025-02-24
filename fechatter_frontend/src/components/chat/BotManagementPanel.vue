<!--
  BotManagementPanel.vue
  Production-Grade Bot Management Panel
-->
<template>
  <teleport to="body">
    <div v-if="visible" 
         class="bot-panel-overlay" 
         :style="{ zIndex: panelZIndex - 1 }"
         @click="handleOverlayClick">
      
      <div class="bot-management-panel" 
           :style="panelStyle"
           @click.stop>
        
        <!-- 面板头部 -->
        <div class="panel-header">
          <h3 class="panel-title">Bot Management</h3>
          <div class="panel-controls">
            <button @click="handleClose" class="close-btn">×</button>
          </div>
        </div>
        
        <!-- 面板导航 -->
        <div class="panel-nav">
          <button 
            v-for="tab in tabs" 
            :key="tab.id"
            :class="['nav-tab', { active: activeTab === tab.id }]"
            @click="switchTab(tab.id)"
          >
            <span class="tab-icon">{{ tab.icon }}</span>
            <span>{{ tab.name }}</span>
          </button>
        </div>
        
        <!-- 翻译Bot -->
        <div v-if="activeTab === 'translation'" class="bot-section">
          <div class="bot-header">
            <span class="bot-icon">🌐</span>
            <h4>Translation Assistant</h4>
            <div class="status-indicator" :class="translationBot.status">
              {{ translationBot.status }}
            </div>
          </div>
          
          <div class="bot-controls">
            <div class="control-group">
              <label>Daily Quota</label>
              <div class="quota-display">
                <span class="quota-used">{{ quotaInfo.used }}</span>
                <span class="quota-separator">/</span>
                <span class="quota-total">{{ quotaInfo.limit }}</span>
              </div>
            </div>
            
            <div class="control-group">
              <label>Supported Languages</label>
              <div class="language-grid">
                <span 
                  v-for="lang in supportedLanguages" 
                  :key="lang.code"
                  class="language-chip"
                >
                  {{ lang.name }}
                </span>
              </div>
            </div>
            
            <div class="control-group">
              <label>Settings</label>
              <div class="setting-item">
                <input 
                  type="checkbox" 
                  id="auto-detect"
                  v-model="translationSettings.autoDetect"
                  @change="updateTranslationSettings"
                >
                <label for="auto-detect">Auto-detect language</label>
              </div>
              <div class="setting-item">
                <input 
                  type="checkbox" 
                  id="preserve-format"
                  v-model="translationSettings.preserveFormatting"
                  @change="updateTranslationSettings"
                >
                <label for="preserve-format">Preserve formatting</label>
              </div>
            </div>
          </div>
        </div>
        
        <!-- AI助手Bot -->
        <div v-if="activeTab === 'assistant'" class="bot-section">
          <div class="bot-header">
            <span class="bot-icon">🤖</span>
            <h4>AI Assistant</h4>
            <div class="status-indicator" :class="assistantBot.status">
              {{ assistantBot.status }}
            </div>
          </div>
          
          <div class="bot-controls">
            <div class="control-group">
              <label>Model Selection</label>
              <select v-model="assistantSettings.model" @change="updateAssistantSettings">
                <option v-for="model in availableModels" :key="model.id" :value="model.id">
                  {{ model.name }}
                </option>
              </select>
            </div>
            
            <div class="control-group">
              <label>Response Length</label>
              <input 
                type="range" 
                :min="100" 
                :max="2000" 
                v-model="assistantSettings.maxLength"
                @input="updateAssistantSettings"
              >
              <span class="range-value">{{ assistantSettings.maxLength }} characters</span>
            </div>
            
            <div class="control-group">
              <label>Features</label>
              <div class="feature-list">
                <div class="feature-item" v-for="feature in assistantFeatures" :key="feature.id">
                  <input 
                    type="checkbox" 
                    :id="feature.id"
                    v-model="feature.enabled"
                    @change="updateAssistantSettings"
                  >
                  <label :for="feature.id">{{ feature.name }}</label>
                  <span class="feature-desc">{{ feature.description }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
        
        <!-- 自定义Bot -->
        <div v-if="activeTab === 'custom'" class="bot-section">
          <div class="bot-header">
            <span class="bot-icon">⚙️</span>
            <h4>Custom Bots</h4>
          </div>
          
          <div class="custom-bot-list">
            <div v-for="bot in customBots" :key="bot.id" class="custom-bot-item">
              <div class="bot-info">
                <span class="bot-name">{{ bot.name }}</span>
                <span class="bot-type">{{ bot.type }}</span>
              </div>
              <div class="bot-actions">
                <button @click="editBot(bot)" class="btn-secondary">Edit</button>
                <button @click="deleteBot(bot)" class="btn-danger">Delete</button>
              </div>
            </div>
          </div>
          
          <div class="custom-bot-creator">
            <h5>Create New Bot</h5>
            <div class="creator-form">
              <input 
                type="text" 
                placeholder="Bot name"
                v-model="newBot.name"
              >
              <select v-model="newBot.type">
                <option value="analyzer">Analyzer</option>
                <option value="summarizer">Summarizer</option>
                <option value="responder">Auto Responder</option>
              </select>
              <button @click="createBot" class="btn-primary" :disabled="!canCreateBot">
                Create Bot
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </teleport>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { useMessageUIStore } from '@/stores/messageUI'
import { botService } from '@/services/botService'

const props = defineProps({
  visible: Boolean,
  panelType: String,
  options: Object
})

const emit = defineEmits(['close', 'bot-updated'])

const messageUIStore = useMessageUIStore()

// State
const activeTab = ref(props.panelType || 'translation')
const botStatus = ref({})
const translationSettings = ref({
  autoDetect: true,
  preserveFormatting: true
})
const assistantSettings = ref({
  model: 'gpt-3.5-turbo',
  maxLength: 500
})
const customBots = ref([])
const newBot = ref({
  name: '',
  type: 'analyzer'
})

// Computed
const panelZIndex = computed(() => messageUIStore.activeBotPanel?.zIndex || 4000)

const panelStyle = computed(() => ({
  right: '20px',
  top: '100px',
  width: `${props.options?.width || 400}px`,
  height: `${props.options?.height || 600}px`,
  zIndex: panelZIndex.value
}))

const tabs = computed(() => [
  { id: 'translation', name: 'Translation', icon: '🌐' },
  { id: 'assistant', name: 'AI Assistant', icon: '🤖' },
  { id: 'custom', name: 'Custom', icon: '⚙️' }
])

const quotaInfo = computed(() => botService.getQuotaInfo())

const supportedLanguages = ref([
  { code: 'en', name: 'English' },
  { code: 'zh', name: '中文' },
  { code: 'ja', name: '日本語' },
  { code: 'ko', name: '한국어' }
])

const availableModels = ref([
  { id: 'gpt-3.5-turbo', name: 'GPT-3.5 Turbo' },
  { id: 'gpt-4', name: 'GPT-4' }
])

const assistantFeatures = ref([
  { id: 'analysis', name: 'Message Analysis', description: 'Analyze message sentiment and content', enabled: true },
  { id: 'summary', name: 'Auto Summary', description: 'Generate automatic summaries', enabled: true },
  { id: 'translation', name: 'Smart Translation', description: 'Context-aware translation', enabled: false }
])

const translationBot = computed(() => ({
  status: 'active'
}))

const assistantBot = computed(() => ({
  status: 'active'
}))

const canCreateBot = computed(() => {
  return newBot.value.name.trim() && newBot.value.type
})

// Methods
const handleOverlayClick = () => handleClose()

const handleClose = () => {
  messageUIStore.closeBotPanel()
  emit('close')
}

const switchTab = (tabId) => {
  activeTab.value = tabId
}

const updateTranslationSettings = async () => {
  try {
    await botService.updateBotConfig('translation', translationSettings.value)
    emit('bot-updated', { type: 'translation', settings: translationSettings.value })
  } catch (error) {
    console.error('Failed to update translation settings:', error)
  }
}

const updateAssistantSettings = async () => {
  try {
    await botService.updateBotConfig('assistant', assistantSettings.value)
    emit('bot-updated', { type: 'assistant', settings: assistantSettings.value })
  } catch (error) {
    console.error('Failed to update assistant settings:', error)
  }
}

const createBot = async () => {
  try {
    const bot = await botService.createCustomBot(newBot.value)
    customBots.value.push(bot)
    newBot.value = { name: '', type: 'analyzer' }
    emit('bot-updated', { type: 'custom', action: 'created', bot })
  } catch (error) {
    console.error('Failed to create bot:', error)
  }
}

const editBot = (bot) => {
  // TODO: Implement bot editing
  console.log('Edit bot:', bot)
}

const deleteBot = async (bot) => {
  try {
    await botService.deleteCustomBot(bot.id)
    customBots.value = customBots.value.filter(b => b.id !== bot.id)
    emit('bot-updated', { type: 'custom', action: 'deleted', bot })
  } catch (error) {
    console.error('Failed to delete bot:', error)
  }
}

// Lifecycle
onMounted(async () => {
  try {
    const status = await botService.getBotStatus()
    botStatus.value = status
    
    if (status.custom) {
      customBots.value = status.custom
    }
  } catch (error) {
    console.error('Failed to load bot status:', error)
  }
})

watch(() => props.panelType, (newType) => {
  if (newType) {
    activeTab.value = newType
  }
})
</script>

<style scoped>
.bot-panel-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.3);
}

.bot-management-panel {
  position: fixed;
  background: white;
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid #e0e0e0;
  background: #f8f9fa;
}

.panel-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: #666;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  background: #e0e0e0;
}

.panel-nav {
  display: flex;
  border-bottom: 1px solid #e0e0e0;
  background: #fafafa;
}

.nav-tab {
  flex: 1;
  padding: 12px 16px;
  border: none;
  background: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.2s;
}

.nav-tab:hover {
  background: #f0f0f0;
}

.nav-tab.active {
  background: white;
  border-bottom: 2px solid #2196f3;
}

.bot-section {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
}

.bot-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 24px;
}

.bot-icon {
  font-size: 24px;
}

.bot-header h4 {
  margin: 0;
  flex: 1;
}

.status-indicator {
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 500;
  text-transform: uppercase;
}

.status-indicator.active {
  background: #e8f5e8;
  color: #2e7d32;
}

.control-group {
  margin-bottom: 24px;
}

.control-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
}

.quota-display {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 18px;
  font-weight: 600;
}

.quota-used {
  color: #2196f3;
}

.quota-separator {
  color: #ccc;
}

.language-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.language-chip {
  padding: 4px 12px;
  background: #f0f0f0;
  border-radius: 16px;
  font-size: 12px;
}

.setting-item {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.feature-list {
  space-y: 12px;
}

.feature-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
}

.feature-desc {
  font-size: 12px;
  color: #666;
}

.custom-bot-list {
  margin-bottom: 24px;
}

.custom-bot-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  margin-bottom: 8px;
}

.bot-info {
  display: flex;
  flex-direction: column;
}

.bot-name {
  font-weight: 500;
}

.bot-type {
  font-size: 12px;
  color: #666;
}

.bot-actions {
  display: flex;
  gap: 8px;
}

.creator-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.creator-form input,
.creator-form select {
  padding: 8px 12px;
  border: 1px solid #e0e0e0;
  border-radius: 6px;
}

.btn-primary,
.btn-secondary,
.btn-danger {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary {
  background: #2196f3;
  color: white;
}

.btn-primary:hover {
  background: #1976d2;
}

.btn-primary:disabled {
  background: #ccc;
  cursor: not-allowed;
}

.btn-secondary {
  background: #f0f0f0;
  color: #333;
}

.btn-secondary:hover {
  background: #e0e0e0;
}

.btn-danger {
  background: #f44336;
  color: white;
}

.btn-danger:hover {
  background: #d32f2f;
}

@media (prefers-color-scheme: dark) {
  .bot-management-panel {
    background: #2d2d2d;
    color: white;
  }
  
  .panel-header {
    background: #333;
    border-bottom-color: #444;
  }
  
  .panel-nav {
    background: #383838;
    border-bottom-color: #444;
  }
  
  .nav-tab:hover {
    background: #404040;
  }
  
  .nav-tab.active {
    background: #2d2d2d;
    border-bottom-color: #64b5f6;
  }
  
  .language-chip {
    background: #404040;
    color: white;
  }
}
</style>
