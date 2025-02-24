<template>
  <div class="enhanced-formatting-toolbar" :class="{ 'toolbar-expanded': isExpanded }">
    <!-- 主工具栏 -->
    <div class="main-toolbar">
      <!-- 基本格式化 -->
      <div class="toolbar-group basic-formatting">
        <button @click="applyFormat('bold')" class="toolbar-btn" title="Bold (⌘B)" :class="{ active: isFormatActive('bold') }">
          <strong>B</strong>
        </button>
        <button @click="applyFormat('italic')" class="toolbar-btn" title="Italic (⌘I)" :class="{ active: isFormatActive('italic') }">
          <em>I</em>
        </button>
        <button @click="applyFormat('strikethrough')" class="toolbar-btn" title="Strikethrough" :class="{ active: isFormatActive('strikethrough') }">
          <del>S</del>
        </button>
        <button @click="applyFormat('code')" class="toolbar-btn" title="Inline Code (`)" :class="{ active: isFormatActive('code') }">
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
              <path d="M6 8L2 4h8l-4 4z"/>
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
        <button @click="applyFormat('unordered-list')" class="toolbar-btn" title="Bullet List" :class="{ active: isFormatActive('unordered-list') }">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M2 4a1 1 0 1 0 0-2 1 1 0 0 0 0 2zm2.5-1.5H14v1H4.5v-1zm0 5H14v1H4.5v-1zm0 5H14v1H4.5v-1zM2 9a1 1 0 1 0 0-2 1 1 0 0 0 0 2zm0 5a1 1 0 1 0 0-2 1 1 0 0 0 0 2z"/>
          </svg>
        </button>
        <button @click="applyFormat('ordered-list')" class="toolbar-btn" title="Numbered List" :class="{ active: isFormatActive('ordered-list') }">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M1 2.5A.5.5 0 0 1 1.5 2h3A.5.5 0 0 1 5 2.5v3A.5.5 0 0 1 4.5 6h-3A.5.5 0 0 1 1 5.5v-3zM1.5 3v2h2V3h-2zm5.5-.5H14v1H7v-1zm0 5H14v1H7v-1zm0 5H14v1H7v-1z"/>
          </svg>
        </button>
        <button @click="applyFormat('task-list')" class="toolbar-btn" title="Task List" :class="{ active: isFormatActive('task-list') }">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M14 1a1 1 0 0 1 1 1v12a1 1 0 0 1-1 1H2a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1h12zM2 0a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V2a2 2 0 0 0-2-2H2z"/>
            <path d="M10.854 5.146a.5.5 0 0 1 0 .708l-3 3a.5.5 0 0 1-.708 0l-1.5-1.5a.5.5 0 1 1 .708-.708L7.5 7.793l2.646-2.647a.5.5 0 0 1 .708 0z"/>
          </svg>
        </button>
      </div>

      <!-- 分隔线 -->
      <div class="toolbar-divider"></div>

      <!-- 插入元素 -->
      <div class="toolbar-group insert-elements">
        <button @click="applyFormat('link')" class="toolbar-btn" title="Link (⌘K)" :class="{ active: isFormatActive('link') }">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M6.354 5.5H4a3 3 0 0 0 0 6h3a3 3 0 0 0 2.83-4H9c-.086 0-.17.01-.25.031A2 2 0 0 1 7 10.5H4a2 2 0 1 1 0-4h1.535c.218-.376.495-.714.82-1z"/>
            <path d="M9 5.5a3 3 0 0 0-2.83 4h1.098A2 2 0 0 1 9 6.5h3a2 2 0 1 1 0 4h-1.535a4.02 4.02 0 0 1-.82 1H12a3 3 0 1 0 0-6H9z"/>
          </svg>
        </button>
        <button @click="applyFormat('image')" class="toolbar-btn" title="Image">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M6.002 5.5a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0z"/>
            <path d="M2.002 1a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V3a2 2 0 0 0-2-2h-12zm12 1a1 1 0 0 1 1 1v6.5l-3.777-1.947a.5.5 0 0 0-.577.093l-3.71 3.71-2.66-1.772a.5.5 0 0 0-.63.062L1.002 12V3a1 1 0 0 1 1-1h12z"/>
          </svg>
        </button>
        <button @click="applyFormat('blockquote')" class="toolbar-btn" title="Blockquote" :class="{ active: isFormatActive('blockquote') }">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M12 12a1 1 0 0 0 1-1V8.558a1 1 0 0 0-1-1h-1.388c0-.351.021-.703.062-1.054.062-.372.166-.703.31-.992.145-.29.331-.517.559-.683.227-.186.516-.279.868-.279V3c-.579 0-1.085.124-1.52.372a3.322 3.322 0 0 0-1.085.992 4.92 4.92 0 0 0-.62 1.458A7.712 7.712 0 0 0 9 7.558V11a1 1 0 0 0 1 1h2Zm-6 0a1 1 0 0 0 1-1V8.558a1 1 0 0 0-1-1H4.612c0-.351.021-.703.062-1.054.062-.372.166-.703.31-.992.145-.29.331-.517.559-.683.227-.186.516-.279.868-.279V3c-.579 0-1.085.124-1.52.372a3.322 3.322 0 0 0-1.085.992 4.92 4.92 0 0 0-.62 1.458A7.712 7.712 0 0 0 3 7.558V11a1 1 0 0 0 1 1h2Z"/>
          </svg>
        </button>
      </div>

      <!-- 分隔线 -->
      <div class="toolbar-divider"></div>

      <!-- 代码和数学 -->
      <div class="toolbar-group code-math">
        <button @click="applyFormat('code-block')" class="toolbar-btn" title="Code Block" :class="{ active: isFormatActive('code-block') }">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M10.478 1.647a.5.5 0 1 0-.956-.294l-4 13a.5.5 0 0 0 .956.294l4-13zM4.854 4.146a.5.5 0 0 1 0 .708L1.707 8l3.147 3.146a.5.5 0 0 1-.708.708l-3.5-3.5a.5.5 0 0 1 0-.708l3.5-3.5a.5.5 0 0 1 .708 0zm6.292 0a.5.5 0 0 0 0 .708L14.293 8l-3.147 3.146a.5.5 0 0 0 .708.708l3.5-3.5a.5.5 0 0 0 0-.708l-3.5-3.5a.5.5 0 0 0-.708 0z"/>
          </svg>
        </button>
        <button @click="applyFormat('math-inline')" class="toolbar-btn" title="Inline Math" :class="{ active: isFormatActive('math-inline') }">
          <span class="math-symbol">∑</span>
        </button>
        <button @click="applyFormat('math-block')" class="toolbar-btn" title="Math Block" :class="{ active: isFormatActive('math-block') }">
          <span class="math-symbol-block">∫</span>
        </button>
      </div>

      <!-- 展开/收起按钮 -->
      <div class="toolbar-group expand-toggle">
        <button @click="toggleExpanded" class="toolbar-btn expand-btn" title="More Options">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" :class="{ rotated: isExpanded }">
            <path d="M8 4a.5.5 0 0 1 .5.5v3h3a.5.5 0 0 1 0 1h-3v3a.5.5 0 0 1-1 0v-3h-3a.5.5 0 0 1 0-1h3v-3A.5.5 0 0 1 8 4z"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- 扩展工具栏 -->
    <Transition name="expand">
      <div v-if="isExpanded" class="expanded-toolbar">
        <!-- 表格工具 -->
        <div class="toolbar-group table-tools">
          <span class="group-label">Table</span>
          <button @click="insertTable(2, 2)" class="toolbar-btn" title="Insert Table">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M0 2a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H2a2 2 0 0 1-2-2V2zm15 2h-4v3h4V4zm0 4h-4v3h4V8zm0 4h-4v3h3a1 1 0 0 0 1-1v-2zm-5 3v-3H6v3h4zm-5 0v-3H1v2a1 1 0 0 0 1 1h3zm-4-4h4V8H1v3zm0-4h4V4H1v3zm5-3v3h4V4H6zm4 4H6v3h4V8z"/>
            </svg>
          </button>
          <button @click="applyFormat('table-row')" class="toolbar-btn" title="Add Row">
            <span class="table-icon">+Row</span>
          </button>
          <button @click="applyFormat('table-col')" class="toolbar-btn" title="Add Column">
            <span class="table-icon">+Col</span>
          </button>
        </div>

        <!-- 快速数学符号 -->
        <div class="toolbar-group math-symbols">
          <span class="group-label">Math</span>
          <button v-for="symbol in mathSymbols" :key="symbol.symbol" @click="insertMathSymbol(symbol)"
            class="toolbar-btn math-btn" :title="symbol.name">
            <span class="math-symbol">{{ symbol.symbol }}</span>
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
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';

const props = defineProps({
  textareaRef: {
    type: Object,
    required: true
  },
  currentContent: {
    type: String,
    default: ''
  },
  disabled: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits(['format-applied', 'content-changed']);

// State
const isExpanded = ref(false);
const showHeaderDropdown = ref(false);
const activeFormats = ref(new Set());

// 数学符号快捷列表
const mathSymbols = [
  { symbol: '∑', name: 'Sum', latex: '\\sum' },
  { symbol: '∫', name: 'Integral', latex: '\\int' },
  { symbol: '∞', name: 'Infinity', latex: '\\infty' },
  { symbol: 'α', name: 'Alpha', latex: '\\alpha' },
  { symbol: 'β', name: 'Beta', latex: '\\beta' },
  { symbol: 'π', name: 'Pi', latex: '\\pi' },
  { symbol: '≤', name: 'Less or equal', latex: '\\leq' },
  { symbol: '≥', name: 'Greater or equal', latex: '\\geq' },
  { symbol: '±', name: 'Plus minus', latex: '\\pm' },
  { symbol: '√', name: 'Square root', latex: '\\sqrt' }
];

// Methods
const toggleExpanded = () => {
  isExpanded.value = !isExpanded.value;
};

const toggleHeaderDropdown = () => {
  showHeaderDropdown.value = !showHeaderDropdown.value;
};

const isFormatActive = (format) => {
  return activeFormats.value.has(format);
};

const applyFormat = (format) => {
  if (props.disabled) return;

  const textarea = props.textareaRef;
  if (!textarea) return;

  const start = textarea.selectionStart;
  const end = textarea.selectionEnd;
  const selectedText = props.currentContent.substring(start, end);
  
  let beforeText = '';
  let afterText = '';
  let newCursorPos = start;

  switch (format) {
    case 'bold':
      beforeText = '**';
      afterText = '**';
      break;
    case 'italic':
      beforeText = '*';
      afterText = '*';
      break;
    case 'strikethrough':
      beforeText = '~~';
      afterText = '~~';
      break;
    case 'code':
      beforeText = '`';
      afterText = '`';
      break;
    case 'link':
      beforeText = '[';
      afterText = '](url)';
      break;
    case 'image':
      beforeText = '![';
      afterText = '](url)';
      break;
    case 'blockquote':
      beforeText = '> ';
      afterText = '';
      break;
    case 'code-block':
      beforeText = '```\n';
      afterText = '\n```';
      break;
    case 'math-inline':
      beforeText = '$';
      afterText = '$';
      break;
    case 'math-block':
      beforeText = '$$\n';
      afterText = '\n$$';
      break;
    case 'unordered-list':
      beforeText = '- ';
      afterText = '';
      break;
    case 'ordered-list':
      beforeText = '1. ';
      afterText = '';
      break;
    case 'task-list':
      beforeText = '- [ ] ';
      afterText = '';
      break;
    case 'horizontal-rule':
      beforeText = '\n---\n';
      afterText = '';
      break;
    case 'spoiler':
      beforeText = '||';
      afterText = '||';
      break;
    case 'highlight':
      beforeText = '==';
      afterText = '==';
      break;
  }

  // 应用格式
  const newText = beforeText + selectedText + afterText;
  const newContent = props.currentContent.substring(0, start) + newText + props.currentContent.substring(end);
  
  // 计算新的光标位置
  if (selectedText) {
    newCursorPos = start + beforeText.length + selectedText.length + afterText.length;
  } else {
    newCursorPos = start + beforeText.length;
  }

  emit('content-changed', newContent);
  emit('format-applied', { format, beforeText, afterText, cursorPos: newCursorPos });

  // 关闭下拉菜单
  showHeaderDropdown.value = false;
};

const applyHeader = (level) => {
  if (props.disabled) return;

  const prefix = '#'.repeat(level) + ' ';
  applyFormat('header');
  
  // 自定义标题处理
  const textarea = props.textareaRef;
  if (!textarea) return;

  const start = textarea.selectionStart;
  const end = textarea.selectionEnd;
  const selectedText = props.currentContent.substring(start, end) || 'Header text';
  
  const newText = prefix + selectedText;
  const newContent = props.currentContent.substring(0, start) + newText + props.currentContent.substring(end);
  
  emit('content-changed', newContent);
  emit('format-applied', { 
    format: `header-${level}`, 
    beforeText: prefix, 
    afterText: '', 
    cursorPos: start + prefix.length + selectedText.length 
  });

  showHeaderDropdown.value = false;
};

const insertTable = (rows, cols) => {
  let tableMarkdown = '\n';
  
  // 表头
  tableMarkdown += '|';
  for (let i = 0; i < cols; i++) {
    tableMarkdown += ` Header ${i + 1} |`;
  }
  tableMarkdown += '\n';
  
  // 分隔符
  tableMarkdown += '|';
  for (let i = 0; i < cols; i++) {
    tableMarkdown += ' --- |';
  }
  tableMarkdown += '\n';
  
  // 数据行
  for (let i = 0; i < rows; i++) {
    tableMarkdown += '|';
    for (let j = 0; j < cols; j++) {
      tableMarkdown += ` Cell ${i + 1}-${j + 1} |`;
    }
    tableMarkdown += '\n';
  }
  
  const textarea = props.textareaRef;
  if (!textarea) return;

  const start = textarea.selectionStart;
  const newContent = props.currentContent.substring(0, start) + tableMarkdown + props.currentContent.substring(start);
  
  emit('content-changed', newContent);
  emit('format-applied', { 
    format: 'table', 
    beforeText: tableMarkdown, 
    afterText: '', 
    cursorPos: start + tableMarkdown.length 
  });
};

const insertMathSymbol = (symbol) => {
  const textarea = props.textareaRef;
  if (!textarea) return;

  const start = textarea.selectionStart;
  const symbolText = symbol.latex || symbol.symbol;
  const newContent = props.currentContent.substring(0, start) + symbolText + props.currentContent.substring(start);
  
  emit('content-changed', newContent);
  emit('format-applied', { 
    format: 'math-symbol', 
    beforeText: symbolText, 
    afterText: '', 
    cursorPos: start + symbolText.length 
  });
};

// 键盘快捷键
const handleKeyDown = (event) => {
  if (props.disabled) return;

  const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
  const ctrlKey = isMac ? event.metaKey : event.ctrlKey;

  if (ctrlKey) {
    switch (event.key.toLowerCase()) {
      case 'b':
        event.preventDefault();
        applyFormat('bold');
        break;
      case 'i':
        event.preventDefault();
        applyFormat('italic');
        break;
      case 'k':
        event.preventDefault();
        applyFormat('link');
        break;
      case '`':
        event.preventDefault();
        applyFormat('code');
        break;
    }
  }
};

// 点击外部关闭下拉菜单
const handleClickOutside = (event) => {
  if (!event.target.closest('.dropdown-container')) {
    showHeaderDropdown.value = false;
  }
};

onMounted(() => {
  document.addEventListener('keydown', handleKeyDown);
  document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown);
  document.removeEventListener('click', handleClickOutside);
});
</script>

<style scoped>
.enhanced-formatting-toolbar {
  background: #ffffff;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  padding: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
  user-select: none;
}

.main-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.toolbar-group {
  display: flex;
  align-items: center;
  gap: 4px;
}

.toolbar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 6px;
  cursor: pointer;
  color: #374151;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s ease;
  position: relative;
}

.toolbar-btn:hover {
  background: #f3f4f6;
  border-color: #d1d5db;
  color: #111827;
}

.toolbar-btn.active {
  background: #eff6ff;
  border-color: #3b82f6;
  color: #1d4ed8;
}

.toolbar-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.toolbar-divider {
  width: 1px;
  height: 24px;
  background: #e5e7eb;
  margin: 0 4px;
}

/* 下拉菜单 */
.dropdown-container {
  position: relative;
}

.dropdown-btn {
  width: auto;
  padding: 0 8px;
  gap: 4px;
}

.header-icon {
  font-weight: 700;
  font-size: 16px;
}

.dropdown-arrow {
  transition: transform 0.2s ease;
}

.dropdown-container.open .dropdown-arrow {
  transform: rotate(180deg);
}

.dropdown-menu {
  position: absolute;
  top: 100%;
  left: 0;
  background: #ffffff;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 100;
  opacity: 0;
  visibility: hidden;
  transform: translateY(-4px);
  transition: all 0.2s ease;
  min-width: 160px;
}

.dropdown-container.open .dropdown-menu {
  opacity: 1;
  visibility: visible;
  transform: translateY(0);
}

.dropdown-item {
  display: flex;
  align-items: center;
  width: 100%;
  padding: 8px 12px;
  background: transparent;
  border: none;
  cursor: pointer;
  color: #374151;
  transition: background-color 0.2s ease;
  text-align: left;
}

.dropdown-item:hover {
  background: #f3f4f6;
}

.header-preview {
  font-weight: 600;
}

.header-preview.h1 { font-size: 18px; }
.header-preview.h2 { font-size: 16px; }
.header-preview.h3 { font-size: 14px; }
.header-preview.h4 { font-size: 12px; }

/* 数学符号 */
.math-symbol {
  font-size: 18px;
  font-family: 'Times New Roman', serif;
}

.math-symbol-block {
  font-size: 20px;
  font-family: 'Times New Roman', serif;
}

/* 表格图标 */
.table-icon {
  font-size: 10px;
  font-weight: 600;
}

.hr-icon {
  font-family: monospace;
  font-size: 12px;
}

.spoiler-icon,
.highlight-icon {
  font-family: monospace;
  font-weight: 700;
}

/* 展开按钮 */
.expand-btn svg {
  transition: transform 0.2s ease;
}

.expand-btn svg.rotated {
  transform: rotate(45deg);
}

/* 扩展工具栏 */
.expanded-toolbar {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid #e5e7eb;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.group-label {
  font-size: 10px;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-right: 8px;
}

.math-btn {
  width: 28px;
  height: 28px;
}

/* 动画 */
.expand-enter-active,
.expand-leave-active {
  transition: all 0.3s ease;
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  opacity: 0;
  max-height: 0;
  margin-top: 0;
  padding-top: 0;
  border-top-width: 0;
}

.expand-enter-to,
.expand-leave-from {
  opacity: 1;
  max-height: 200px;
}

/* 响应式 */
@media (max-width: 768px) {
  .main-toolbar {
    gap: 4px;
  }
  
  .toolbar-btn {
    width: 28px;
    height: 28px;
    font-size: 12px;
  }
  
  .toolbar-group {
    gap: 2px;
  }
}
</style> 