<!--
  FloatingFormattingToolbar.vue
  完全悬浮的格式化工具栏 - 彻底迁移成悬浮菜单
  
  Features:
  - 完全悬浮定位，不占用页面布局空间
  - 智能定位算法，自动调整位置避免超出屏幕
  - 完整的格式化功能集
  - 响应式设计，适配不同屏幕尺寸
  - 键盘快捷键支持
  - 优雅的动画效果
-->
<template>
  <Teleport to="body">
    <div v-if="visible" class="floating-toolbar-overlay" :style="{ zIndex: toolbarZIndex - 1 }"
      @click="handleOverlayClick" @contextmenu.prevent>

      <div ref="toolbarRef" class="floating-formatting-toolbar" :style="toolbarStyle" :class="toolbarClasses"
        @click.stop @contextmenu.prevent>

        <!-- 工具栏拖拽手柄 -->
        <div class="toolbar-handle" @mousedown="startDrag" @touchstart="startDrag">
          <div class="handle-icon">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path
                d="M3 9.5a1.5 1.5 0 1 1 0-3 1.5 1.5 0 0 1 0 3zm5 0a1.5 1.5 0 1 1 0-3 1.5 1.5 0 0 1 0 3zm5 0a1.5 1.5 0 1 1 0-3 1.5 1.5 0 0 1 0 3z" />
            </svg>
          </div>
          <div class="toolbar-title">Format</div>
          <button @click="handleClose" class="toolbar-close">×</button>
        </div>

        <!-- 主工具栏 -->
        <div class="main-toolbar">
          <!-- 基本格式化 -->
          <div class="toolbar-group basic-formatting">
            <button @click="applyFormat('bold')" class="toolbar-btn" title="Bold (⌘B)"
              :class="{ active: isFormatActive('bold') }">
              <strong>B</strong>
            </button>
            <button @click="applyFormat('italic')" class="toolbar-btn" title="Italic (⌘I)"
              :class="{ active: isFormatActive('italic') }">
              <em>I</em>
            </button>
            <button @click="applyFormat('strikethrough')" class="toolbar-btn" title="Strikethrough"
              :class="{ active: isFormatActive('strikethrough') }">
              <del>S</del>
            </button>
            <button @click="applyFormat('code')" class="toolbar-btn" title="Inline Code (`)"
              :class="{ active: isFormatActive('code') }">
              <code>{}</code>
            </button>
          </div>

          <!-- 分隔线 -->
          <div class="toolbar-divider"></div>

          <!-- 标题 -->
          <div class="toolbar-group headers">
            <div class="dropdown-container" :class="{ open: showHeaderDropdown }" @click="toggleHeaderDropdown">
              <button class="toolbar-btn dropdown-btn" title="Headers">
                <span class="header-icon">H</span>
                <svg class="dropdown-arrow" width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
                  <path d="M6 8L2 4h8l-4 4z" />
                </svg>
              </button>
              <div class="dropdown-menu header-menu">
                <button @click="applyHeader(1)" class="dropdown-item">
                  <span class="header-preview h1">Heading 1</span>
                </button>
                <button @click="applyHeader(2)" class="dropdown-item">
                  <span class="header-preview h2">Heading 2</span>
                </button>
                <button @click="applyHeader(3)" class="dropdown-item">
                  <span class="header-preview h3">Heading 3</span>
                </button>
                <button @click="applyHeader(4)" class="dropdown-item">
                  <span class="header-preview h4">Heading 4</span>
                </button>
              </div>
            </div>
          </div>

          <!-- 列表 -->
          <div class="toolbar-group lists">
            <button @click="applyFormat('unordered-list')" class="toolbar-btn" title="Bullet List"
              :class="{ active: isFormatActive('unordered-list') }">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path
                  d="M2 4a1 1 0 1 0 0-2 1 1 0 0 0 0 2zm2.5-1.5H14v1H4.5v-1zm0 5H14v1H4.5v-1zm0 5H14v1H4.5v-1zM2 9a1 1 0 1 0 0-2 1 1 0 0 0 0 2zm0 5a1 1 0 1 0 0-2 1 1 0 0 0 0 2z" />
              </svg>
            </button>
            <button @click="applyFormat('ordered-list')" class="toolbar-btn" title="Numbered List"
              :class="{ active: isFormatActive('ordered-list') }">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path
                  d="M1 2.5A.5.5 0 0 1 1.5 2h3A.5.5 0 0 1 5 2.5v3A.5.5 0 0 1 4.5 6h-3A.5.5 0 0 1 1 5.5v-3zM1.5 3v2h2V3h-2zm5.5-.5H14v1H7v-1zm0 5H14v1H7v-1zm0 5H14v1H7v-1z" />
              </svg>
            </button>
            <button @click="applyFormat('task-list')" class="toolbar-btn" title="Task List"
              :class="{ active: isFormatActive('task-list') }">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path
                  d="M14 1a1 1 0 0 1 1 1v12a1 1 0 0 1-1 1H2a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1h12zM2 0a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V2a2 2 0 0 0-2-2H2z" />
                <path
                  d="M10.854 5.146a.5.5 0 0 1 0 .708l-3 3a.5.5 0 0 1-.708 0l-1.5-1.5a.5.5 0 1 1 .708-.708L7.5 7.793l2.646-2.647a.5.5 0 0 1 .708 0z" />
              </svg>
            </button>
          </div>

          <!-- 分隔线 -->
          <div class="toolbar-divider"></div>

          <!-- 插入元素 -->
          <div class="toolbar-group insert-elements">
            <button @click="applyFormat('link')" class="toolbar-btn" title="Link (⌘K)"
              :class="{ active: isFormatActive('link') }">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path
                  d="M6.354 5.5H4a3 3 0 0 0 0 6h3a3 3 0 0 0 2.83-4H9c-.086 0-.17.01-.25.031A2 2 0 0 1 7 10.5H4a2 2 0 1 1 0-4h1.535c.218-.376.495-.714.82-1z" />
                <path
                  d="M9 5.5a3 3 0 0 0-2.83 4h1.098A2 2 0 0 1 9 6.5h3a2 2 0 1 1 0 4h-1.535a4.02 4.02 0 0 1-.82 1H12a3 3 0 1 0 0-6H9z" />
              </svg>
            </button>
            <button @click="applyFormat('image')" class="toolbar-btn" title="Image">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M6.002 5.5a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0z" />
                <path
                  d="M2.002 1a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V3a2 2 0 0 0-2-2h-12zm12 1a1 1 0 0 1 1 1v6.5l-3.777-1.947a.5.5 0 0 0-.577.093l-3.71 3.71-2.66-1.772a.5.5 0 0 0-.63.062L1.002 12V3a1 1 0 0 1 1-1h12z" />
              </svg>
            </button>
            <button @click="applyFormat('blockquote')" class="toolbar-btn" title="Blockquote"
              :class="{ active: isFormatActive('blockquote') }">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path
                  d="M12 12a1 1 0 0 0 1-1V8.558a1 1 0 0 0-1-1h-1.388c0-.351.021-.703.062-1.054.062-.372.166-.703.31-.992.145-.29.331-.517.559-.683.227-.186.516-.279.868-.279V3c-.579 0-1.085.124-1.52.372a3.322 3.322 0 0 0-1.085.992 4.92 4.92 0 0 0-.62 1.458A7.712 7.712 0 0 0 9 7.558V11a1 1 0 0 0 1 1h2Zm-6 0a1 1 0 0 0 1-1V8.558a1 1 0 0 0-1-1H4.612c0-.351.021-.703.062-1.054.062-.372.166-.703.31-.992.145-.29.331-.517.559-.683.227-.186.516-.279.868-.279V3c-.579 0-1.085.124-1.52.372a3.322 3.322 0 0 0-1.085.992 4.92 4.92 0 0 0-.62 1.458A7.712 7.712 0 0 0 3 7.558V11a1 1 0 0 0 1 1h2Z" />
              </svg>
            </button>
          </div>

          <!-- 展开/收起按钮 -->
          <div class="toolbar-group expand-toggle">
            <button @click="toggleExpanded" class="toolbar-btn expand-btn" title="More Options">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" :class="{ rotated: isExpanded }">
                <path
                  d="M8 4a.5.5 0 0 1 .5.5v3h3a.5.5 0 0 1 0 1h-3v3a.5.5 0 0 1-1 0v-3h-3a.5.5 0 0 1 0-1h3v-3A.5.5 0 0 1 8 4z" />
              </svg>
            </button>
          </div>
        </div>

        <!-- 扩展工具栏 -->
        <Transition name="expand">
          <div v-if="isExpanded" class="expanded-toolbar">
            <!-- 代码和数学 -->
            <div class="toolbar-group code-math">
              <span class="group-label">Code & Math</span>
              <button @click="applyFormat('code-block')" class="toolbar-btn" title="Code Block"
                :class="{ active: isFormatActive('code-block') }">
                <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                  <path
                    d="M10.478 1.647a.5.5 0 1 0-.956-.294l-4 13a.5.5 0 0 0 .956.294l4-13zM4.854 4.146a.5.5 0 0 1 0 .708L1.707 8l3.147 3.146a.5.5 0 0 1-.708.708l-3.5-3.5a.5.5 0 0 1 0-.708l3.5-3.5a.5.5 0 0 1 .708 0zm6.292 0a.5.5 0 0 0 0 .708L14.293 8l-3.147 3.146a.5.5 0 0 0 .708.708l3.5-3.5a.5.5 0 0 0 0-.708l-3.5-3.5a.5.5 0 0 0-.708 0z" />
                </svg>
              </button>
              <button @click="applyFormat('math-inline')" class="toolbar-btn" title="Inline Math"
                :class="{ active: isFormatActive('math-inline') }">
                <span class="math-symbol">∑</span>
              </button>
              <button @click="applyFormat('math-block')" class="toolbar-btn" title="Math Block"
                :class="{ active: isFormatActive('math-block') }">
                <span class="math-symbol-block">∫</span>
              </button>
            </div>

            <!-- 表格工具 -->
            <div class="toolbar-group table-tools">
              <span class="group-label">Table</span>
              <button @click="insertTable(2, 2)" class="toolbar-btn" title="Insert Table">
                <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                  <path
                    d="M0 2a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H2a2 2 0 0 1-2-2V2zm15 2h-4v3h4V4zm0 4h-4v3h4V8zm0 4h-4v3h3a1 1 0 0 0 1-1v-2zm-5 3v-3H6v3h4zm-5 0v-3H1v2a1 1 0 0 0 1 1h3zm-4-4h4V8H1v3zm0-4h4V4H1v3zm5-3v3h4V4H6zm4 4H6v3h4V8z" />
                </svg>
              </button>
              <button @click="applyFormat('table-row')" class="toolbar-btn" title="Add Row">
                <span class="table-icon">+Row</span>
              </button>
            </div>

            <!-- 特殊格式 -->
            <div class="toolbar-group special-formats">
              <span class="group-label">Special</span>
              <button @click="applyFormat('horizontal-rule')" class="toolbar-btn" title="Horizontal Rule">
                <span class="hr-icon">---</span>
              </button>
              <button @click="applyFormat('spoiler')" class="toolbar-btn" title="Spoiler">
                <span class="spoiler-icon">||</span>
              </button>
              <button @click="applyFormat('highlight')" class="toolbar-btn" title="Highlight">
                <span class="highlight-icon">==</span>
              </button>
            </div>
          </div>
        </Transition>

        <!-- 快捷键提示 -->
        <div class="keyboard-shortcuts" v-if="showShortcuts">
          <div class="shortcuts-title">Keyboard Shortcuts</div>
          <div class="shortcuts-list">
            <div class="shortcut-item">
              <kbd>⌘B</kbd> <span>Bold</span>
            </div>
            <div class="shortcut-item">
              <kbd>⌘I</kbd> <span>Italic</span>
            </div>
            <div class="shortcut-item">
              <kbd>⌘K</kbd> <span>Link</span>
            </div>
            <div class="shortcut-item">
              <kbd>⌘`</kbd> <span>Code</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'

// ================================
// 🎯 Props & Emits
// ================================

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  position: {
    type: Object,
    default: () => ({ x: 100, y: 100 })
  },
  textareaRef: {
    type: Object,
    default: null
  },
  currentContent: {
    type: String,
    default: ''
  },
  disabled: {
    type: Boolean,
    default: false
  },
  zIndex: {
    type: Number,
    default: 9999
  }
})

const emit = defineEmits([
  'close',
  'format-applied',
  'content-changed',
  'position-changed'
])

// ================================
// 🎯 Reactive State
// ================================

const toolbarRef = ref(null)
const isExpanded = ref(false)
const showHeaderDropdown = ref(false)
const showShortcuts = ref(false)
const activeFormats = ref(new Set())
const isDragging = ref(false)
const dragOffset = ref({ x: 0, y: 0 })

// ================================
// 🎯 Computed Properties
// ================================

const toolbarZIndex = computed(() => props.zIndex)

const toolbarStyle = computed(() => {
  const baseStyle = {
    position: 'fixed',
    zIndex: toolbarZIndex.value,
    left: `${Math.max(20, Math.min(window.innerWidth - 400, props.position.x))}px`,
    top: `${Math.max(20, Math.min(window.innerHeight - 200, props.position.y))}px`,
  }

  if (isDragging.value) {
    baseStyle.transition = 'none'
    baseStyle.userSelect = 'none'
  }

  return baseStyle
})

const toolbarClasses = computed(() => [
  'floating-formatting-toolbar',
  {
    'toolbar-expanded': isExpanded.value,
    'toolbar-dragging': isDragging.value,
    'toolbar-with-shortcuts': showShortcuts.value
  }
])

// ================================
// 🎯 Core Methods
// ================================

const handleClose = () => {
  emit('close')
}

const handleOverlayClick = () => {
  handleClose()
}

const isFormatActive = (format) => {
  return activeFormats.value.has(format)
}

const toggleExpanded = () => {
  isExpanded.value = !isExpanded.value
}

const toggleHeaderDropdown = () => {
  showHeaderDropdown.value = !showHeaderDropdown.value
}

// ================================
// 🎯 Format Application Methods
// ================================

const applyFormat = (format) => {
  if (props.disabled) return

  const textarea = props.textareaRef
  if (!textarea) return

  const start = textarea.selectionStart
  const end = textarea.selectionEnd
  const selectedText = props.currentContent.substring(start, end)

  let beforeText = ''
  let afterText = ''
  let newCursorPos = start

  switch (format) {
    case 'bold':
      beforeText = '**'
      afterText = '**'
      break
    case 'italic':
      beforeText = '*'
      afterText = '*'
      break
    case 'strikethrough':
      beforeText = '~~'
      afterText = '~~'
      break
    case 'code':
      beforeText = '`'
      afterText = '`'
      break
    case 'link':
      beforeText = '['
      afterText = '](url)'
      break
    case 'image':
      beforeText = '!['
      afterText = '](url)'
      break
    case 'blockquote':
      beforeText = '> '
      afterText = ''
      break
    case 'code-block':
      beforeText = '```\n'
      afterText = '\n```'
      break
    case 'math-inline':
      beforeText = '$'
      afterText = '$'
      break
    case 'math-block':
      beforeText = '$$\n'
      afterText = '\n$$'
      break
    case 'unordered-list':
      beforeText = '- '
      afterText = ''
      break
    case 'ordered-list':
      beforeText = '1. '
      afterText = ''
      break
    case 'task-list':
      beforeText = '- [ ] '
      afterText = ''
      break
    case 'horizontal-rule':
      beforeText = '\n---\n'
      afterText = ''
      break
    case 'spoiler':
      beforeText = '||'
      afterText = '||'
      break
    case 'highlight':
      beforeText = '=='
      afterText = '=='
      break
  }

  // Apply format
  const newText = beforeText + selectedText + afterText
  const newContent = props.currentContent.substring(0, start) + newText + props.currentContent.substring(end)

  // Calculate new cursor position
  if (selectedText) {
    newCursorPos = start + beforeText.length + selectedText.length + afterText.length
  } else {
    newCursorPos = start + beforeText.length
  }

  emit('content-changed', newContent)
  emit('format-applied', { format, beforeText, afterText, cursorPos: newCursorPos })

  // Close dropdowns
  showHeaderDropdown.value = false
}

const applyHeader = (level) => {
  if (props.disabled) return

  const prefix = '#'.repeat(level) + ' '
  const textarea = props.textareaRef
  if (!textarea) return

  const start = textarea.selectionStart
  const end = textarea.selectionEnd
  const selectedText = props.currentContent.substring(start, end) || 'Header text'

  const newText = prefix + selectedText
  const newContent = props.currentContent.substring(0, start) + newText + props.currentContent.substring(end)

  emit('content-changed', newContent)
  emit('format-applied', {
    format: `header-${level}`,
    beforeText: prefix,
    afterText: '',
    cursorPos: start + prefix.length + selectedText.length
  })

  showHeaderDropdown.value = false
}

const insertTable = (rows, cols) => {
  let tableMarkdown = '\n'

  // Table header
  tableMarkdown += '|'
  for (let i = 0; i < cols; i++) {
    tableMarkdown += ` Header ${i + 1} |`
  }
  tableMarkdown += '\n'

  // Separator
  tableMarkdown += '|'
  for (let i = 0; i < cols; i++) {
    tableMarkdown += ' --- |'
  }
  tableMarkdown += '\n'

  // Data rows
  for (let i = 0; i < rows; i++) {
    tableMarkdown += '|'
    for (let j = 0; j < cols; j++) {
      tableMarkdown += ` Cell ${i + 1}-${j + 1} |`
    }
    tableMarkdown += '\n'
  }

  const textarea = props.textareaRef
  if (!textarea) return

  const start = textarea.selectionStart
  const newContent = props.currentContent.substring(0, start) + tableMarkdown + props.currentContent.substring(start)

  emit('content-changed', newContent)
  emit('format-applied', {
    format: 'table',
    beforeText: tableMarkdown,
    afterText: '',
    cursorPos: start + tableMarkdown.length
  })
}

// ================================
// 🎯 Drag & Drop Functions
// ================================

const startDrag = (event) => {
  if (!toolbarRef.value) return

  isDragging.value = true
  const rect = toolbarRef.value.getBoundingClientRect()

  const clientX = event.clientX || (event.touches && event.touches[0].clientX)
  const clientY = event.clientY || (event.touches && event.touches[0].clientY)

  dragOffset.value = {
    x: clientX - rect.left,
    y: clientY - rect.top
  }

  document.addEventListener('mousemove', handleDrag)
  document.addEventListener('mouseup', stopDrag)
  document.addEventListener('touchmove', handleDrag)
  document.addEventListener('touchend', stopDrag)
}

const handleDrag = (event) => {
  if (!isDragging.value) return

  event.preventDefault()

  const clientX = event.clientX || (event.touches && event.touches[0].clientX)
  const clientY = event.clientY || (event.touches && event.touches[0].clientY)

  const newX = clientX - dragOffset.value.x
  const newY = clientY - dragOffset.value.y

  // Boundary constraints
  const maxX = window.innerWidth - 400
  const maxY = window.innerHeight - 200

  const constrainedX = Math.max(20, Math.min(maxX, newX))
  const constrainedY = Math.max(20, Math.min(maxY, newY))

  emit('position-changed', { x: constrainedX, y: constrainedY })
}

const stopDrag = () => {
  isDragging.value = false
  document.removeEventListener('mousemove', handleDrag)
  document.removeEventListener('mouseup', stopDrag)
  document.removeEventListener('touchmove', handleDrag)
  document.removeEventListener('touchend', stopDrag)
}

// ================================
// 🎯 Keyboard Shortcuts
// ================================

const handleKeyDown = (event) => {
  if (props.disabled || !props.visible) return

  const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0
  const ctrlKey = isMac ? event.metaKey : event.ctrlKey

  if (ctrlKey) {
    switch (event.key.toLowerCase()) {
      case 'b':
        event.preventDefault()
        applyFormat('bold')
        break
      case 'i':
        event.preventDefault()
        applyFormat('italic')
        break
      case 'k':
        event.preventDefault()
        applyFormat('link')
        break
      case '`':
        event.preventDefault()
        applyFormat('code')
        break
    }
  }

  // Toggle shortcuts help
  if (event.key === 'F1' || (event.key === '?' && event.ctrlKey)) {
    event.preventDefault()
    showShortcuts.value = !showShortcuts.value
  }

  // Close on Escape
  if (event.key === 'Escape') {
    event.preventDefault()
    handleClose()
  }
}

const handleClickOutside = (event) => {
  if (!toolbarRef.value?.contains(event.target)) {
    showHeaderDropdown.value = false
    showShortcuts.value = false
  }
}

// ================================
// 🎯 Lifecycle Hooks
// ================================

onMounted(() => {
  document.addEventListener('keydown', handleKeyDown)
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown)
  document.removeEventListener('click', handleClickOutside)
  document.removeEventListener('mousemove', handleDrag)
  document.removeEventListener('mouseup', stopDrag)
  document.removeEventListener('touchmove', handleDrag)
  document.removeEventListener('touchend', stopDrag)
})

// Reset state when visibility changes
watch(() => props.visible, (newVisible) => {
  if (!newVisible) {
    isExpanded.value = false
    showHeaderDropdown.value = false
    showShortcuts.value = false
    isDragging.value = false
  }
})
</script>

<style scoped>
/* ================================
   🎯 Overlay & Container
   ================================ */

.floating-toolbar-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(2px);
  pointer-events: all;
}

.floating-formatting-toolbar {
  min-width: 380px;
  max-width: 450px;
  background: #ffffff;
  border: 1px solid #e5e7eb;
  border-radius: 12px;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
  user-select: none;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  transform-gpu: true;
  will-change: transform;
}

.floating-formatting-toolbar.toolbar-dragging {
  transform: scale(1.02);
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
}

/* ================================
   🎯 Toolbar Handle
   ================================ */

.toolbar-handle {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
  border-bottom: 1px solid #e2e8f0;
  border-radius: 12px 12px 0 0;
  cursor: move;
  position: relative;
}

.toolbar-handle:hover {
  background: linear-gradient(135deg, #f1f5f9 0%, #e2e8f0 100%);
}

.handle-icon {
  color: #64748b;
  display: flex;
  align-items: center;
}

.toolbar-title {
  font-size: 14px;
  font-weight: 600;
  color: #374151;
  margin: 0 12px;
}

.toolbar-close {
  background: none;
  border: none;
  color: #6b7280;
  cursor: pointer;
  font-size: 20px;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s ease;
}

.toolbar-close:hover {
  background: #f3f4f6;
  color: #374151;
}

/* ================================
   🎯 Main Toolbar
   ================================ */

.main-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  padding: 12px 16px;
}

.toolbar-group {
  display: flex;
  align-items: center;
  gap: 4px;
  position: relative;
}

.group-label {
  font-size: 11px;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-right: 6px;
  min-width: fit-content;
}

.toolbar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 8px;
  cursor: pointer;
  color: #374151;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
}

.toolbar-btn:hover {
  background: #f9fafb;
  border-color: #e5e7eb;
  color: #111827;
  transform: translateY(-1px);
}

.toolbar-btn.active {
  background: #eff6ff;
  border-color: #3b82f6;
  color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.toolbar-btn:active {
  transform: translateY(0) scale(0.95);
}

.toolbar-divider {
  width: 1px;
  height: 24px;
  background: #e5e7eb;
  margin: 0 4px;
}

/* ================================
   🎯 Special Buttons
   ================================ */

.expand-btn svg {
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.expand-btn svg.rotated {
  transform: rotate(45deg);
}

.dropdown-btn {
  position: relative;
}

.dropdown-arrow {
  position: absolute;
  top: 50%;
  right: 2px;
  transform: translateY(-50%);
  transition: transform 0.2s ease;
}

.dropdown-container.open .dropdown-arrow {
  transform: translateY(-50%) rotate(180deg);
}

/* ================================
   🎯 Dropdown Menus
   ================================ */

.dropdown-menu {
  position: absolute;
  top: 100%;
  left: 0;
  min-width: 160px;
  background: #ffffff;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
  z-index: 10;
  opacity: 0;
  visibility: hidden;
  transform: translateY(-8px);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.dropdown-container.open .dropdown-menu {
  opacity: 1;
  visibility: visible;
  transform: translateY(4px);
}

.dropdown-item {
  display: block;
  width: 100%;
  padding: 8px 12px;
  background: none;
  border: none;
  text-align: left;
  color: #374151;
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.dropdown-item:hover {
  background: #f9fafb;
}

.dropdown-item:first-child {
  border-radius: 8px 8px 0 0;
}

.dropdown-item:last-child {
  border-radius: 0 0 8px 8px;
}

/* ================================
   🎯 Header Previews
   ================================ */

.header-preview {
  font-weight: bold;
}

.header-preview.h1 {
  font-size: 18px;
}

.header-preview.h2 {
  font-size: 16px;
}

.header-preview.h3 {
  font-size: 14px;
}

.header-preview.h4 {
  font-size: 12px;
}

/* ================================
   🎯 Special Icons
   ================================ */

.math-symbol,
.math-symbol-block {
  font-size: 16px;
  font-weight: 500;
}

.table-icon,
.hr-icon,
.spoiler-icon,
.highlight-icon {
  font-size: 10px;
  font-weight: 600;
  font-family: 'SF Mono', Monaco, monospace;
}

/* ================================
   🎯 Expanded Toolbar
   ================================ */

.expanded-toolbar {
  border-top: 1px solid #e5e7eb;
  padding: 12px 16px;
  background: #f9fafb;
  border-radius: 0 0 12px 12px;
}

.expanded-toolbar .toolbar-group {
  margin-bottom: 8px;
  padding-bottom: 8px;
  border-bottom: 1px solid #e5e7eb;
}

.expanded-toolbar .toolbar-group:last-child {
  margin-bottom: 0;
  padding-bottom: 0;
  border-bottom: none;
}

/* ================================
   🎯 Keyboard Shortcuts
   ================================ */

.keyboard-shortcuts {
  border-top: 1px solid #e5e7eb;
  padding: 12px 16px;
  background: #f8fafc;
  border-radius: 0 0 12px 12px;
}

.shortcuts-title {
  font-size: 12px;
  font-weight: 600;
  color: #374151;
  margin-bottom: 8px;
}

.shortcuts-list {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
}

.shortcut-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11px;
  color: #6b7280;
}

.shortcut-item kbd {
  background: #e5e7eb;
  color: #374151;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 10px;
  font-family: ui-monospace, SFMono-Regular, monospace;
}

/* ================================
   🎯 Animations
   ================================ */

.expand-enter-active,
.expand-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.expand-enter-from,
.expand-leave-to {
  opacity: 0;
  transform: translateY(-10px);
  max-height: 0;
}

.expand-enter-to,
.expand-leave-from {
  opacity: 1;
  transform: translateY(0);
  max-height: 200px;
}

/* ================================
   🎯 Responsive Design
   ================================ */

@media (max-width: 480px) {
  .floating-formatting-toolbar {
    min-width: 320px;
    max-width: 90vw;
  }

  .main-toolbar {
    padding: 8px 12px;
    gap: 6px;
  }

  .toolbar-btn {
    width: 32px;
    height: 32px;
  }

  .shortcuts-list {
    grid-template-columns: 1fr;
  }
}

/* ================================
   🎯 Dark Mode Support
   ================================ */

@media (prefers-color-scheme: dark) {
  .floating-formatting-toolbar {
    background: #1f2937;
    border-color: #374151;
    color: #f9fafb;
  }

  .toolbar-handle {
    background: linear-gradient(135deg, #374151 0%, #4b5563 100%);
    border-bottom-color: #4b5563;
  }

  .toolbar-handle:hover {
    background: linear-gradient(135deg, #4b5563 0%, #6b7280 100%);
  }

  .toolbar-btn {
    color: #d1d5db;
  }

  .toolbar-btn:hover {
    background: #374151;
    border-color: #4b5563;
    color: #f9fafb;
  }

  .toolbar-btn.active {
    background: #1e40af;
    border-color: #3b82f6;
    color: #93c5fd;
  }

  .expanded-toolbar {
    background: #374151;
    border-top-color: #4b5563;
  }

  .keyboard-shortcuts {
    background: #1f2937;
    border-top-color: #374151;
  }

  .dropdown-menu {
    background: #1f2937;
    border-color: #374151;
  }

  .dropdown-item {
    color: #d1d5db;
  }

  .dropdown-item:hover {
    background: #374151;
  }
}

/* ================================
   🎯 Accessibility
   ================================ */

.toolbar-btn:focus-visible {
  outline: 2px solid #3b82f6;
  outline-offset: 2px;
}

@media (prefers-reduced-motion: reduce) {

  .floating-formatting-toolbar,
  .toolbar-btn,
  .dropdown-menu,
  .expand-enter-active,
  .expand-leave-active {
    transition: none;
  }

  .expand-btn svg {
    transition: none;
  }
}

/* ================================
   🎯 High Contrast Mode
   ================================ */

@media (prefers-contrast: high) {
  .floating-formatting-toolbar {
    border: 2px solid;
  }

  .toolbar-btn {
    border: 1px solid;
  }

  .toolbar-btn:hover,
  .toolbar-btn.active {
    border-width: 2px;
  }
}
</style>