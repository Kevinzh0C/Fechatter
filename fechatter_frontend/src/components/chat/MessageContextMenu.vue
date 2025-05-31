<template>
  <teleport to="body">
    <div v-if="visible" 
         class="fixed bg-white rounded-lg shadow-xl border border-gray-200 py-1 z-50 min-w-[160px]"
         :style="menuStyle"
         @click.stop>
      <button @click="handleReply" 
              class="w-full px-4 py-2 text-left text-sm text-gray-700 hover:bg-gray-100 flex items-center space-x-2">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6"></path>
        </svg>
        <span>回复</span>
      </button>
      
      <button @click="handleForward"
              class="w-full px-4 py-2 text-left text-sm text-gray-700 hover:bg-gray-100 flex items-center space-x-2">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"></path>
        </svg>
        <span>转发</span>
      </button>
      
      <button @click="handleCopy"
              class="w-full px-4 py-2 text-left text-sm text-gray-700 hover:bg-gray-100 flex items-center space-x-2">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"></path>
        </svg>
        <span>复制</span>
      </button>
      
      <button v-if="canEdit"
              @click="handleEdit"
              class="w-full px-4 py-2 text-left text-sm text-gray-700 hover:bg-gray-100 flex items-center space-x-2">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"></path>
        </svg>
        <span>编辑</span>
      </button>
      
      <button @click="handleSelect"
              class="w-full px-4 py-2 text-left text-sm text-gray-700 hover:bg-gray-100 flex items-center space-x-2">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4"></path>
        </svg>
        <span>选择</span>
      </button>
      
      <div class="border-t border-gray-200 my-1"></div>
      
      <button v-if="canDelete"
              @click="handleDelete"
              class="w-full px-4 py-2 text-left text-sm text-red-600 hover:bg-red-50 flex items-center space-x-2">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
        </svg>
        <span>删除</span>
      </button>
    </div>
  </teleport>
</template>

<script setup>
import { computed, onMounted, onUnmounted } from 'vue';

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  x: {
    type: Number,
    default: 0
  },
  y: {
    type: Number,
    default: 0
  },
  message: {
    type: Object,
    default: null
  },
  canEdit: {
    type: Boolean,
    default: false
  },
  canDelete: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits(['reply', 'edit', 'delete', 'forward', 'copy', 'select', 'close']);

// 计算菜单位置，确保不超出屏幕边界
const menuStyle = computed(() => {
  const style = {
    top: `${props.y}px`,
    left: `${props.x}px`
  };
  
  // TODO: 计算实际菜单尺寸，调整位置避免超出屏幕
  if (typeof window !== 'undefined') {
    const menuWidth = 200; // 估计宽度
    const menuHeight = 300; // 估计高度
    
    if (props.x + menuWidth > window.innerWidth) {
      style.left = `${props.x - menuWidth}px`;
    }
    
    if (props.y + menuHeight > window.innerHeight) {
      style.top = `${props.y - menuHeight}px`;
    }
  }
  
  return style;
});

// 处理函数
const handleReply = () => {
  emit('reply', props.message);
  emit('close');
};

const handleEdit = () => {
  emit('edit', props.message);
  emit('close');
};

const handleDelete = () => {
  emit('delete', props.message);
  emit('close');
};

const handleForward = () => {
  emit('forward', props.message);
  emit('close');
};

const handleCopy = () => {
  emit('copy', props.message);
  emit('close');
};

const handleSelect = () => {
  emit('select', props.message);
  emit('close');
};

// 点击外部关闭
const handleClickOutside = (e) => {
  if (props.visible) {
    emit('close');
  }
};

// 按ESC关闭
const handleEscape = (e) => {
  if (e.key === 'Escape' && props.visible) {
    emit('close');
  }
};

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
  document.addEventListener('keydown', handleEscape);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
  document.removeEventListener('keydown', handleEscape);
});
</script> 