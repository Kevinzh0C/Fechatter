<template>
  <!-- 优化: 降低位置，减少闪烁，降低认知负担 -->
  <div v-if="isVisible" class="absolute -top-8 right-2 z-40 message-toolbar-optimized" @mouseenter="keepVisible"
    @mouseleave="hideToolbar">
    <div
      class="flex items-center bg-white/95 rounded-md shadow-md border border-gray-200/80 px-1 py-0.5 space-x-0.5 backdrop-blur-sm">
      <!-- Reply Button -->
      <button type="button"
        class="p-1.5 rounded text-gray-500 hover:text-blue-600 hover:bg-blue-50/80 transition-all duration-100 focus:outline-none"
        @click="handleReply" title="Reply">
        <ArrowUturnLeftIcon class="h-3.5 w-3.5" />
      </button>

      <!-- Translate Button -->
      <button type="button"
        class="p-1.5 rounded text-gray-500 hover:text-purple-600 hover:bg-purple-50/80 transition-all duration-100 focus:outline-none"
        @click="handleTranslate" title="Translate">
        <LanguageIcon class="h-3.5 w-3.5" />
      </button>

      <!-- Edit Button (if user can edit) -->
      <button v-if="canEdit" type="button"
        class="p-1.5 rounded text-gray-500 hover:text-green-600 hover:bg-green-50/80 transition-all duration-100 focus:outline-none"
        @click="handleEdit" title="Edit">
        <PencilIcon class="h-3.5 w-3.5" />
      </button>

      <!-- Delete Button (if user can delete) -->
      <button v-if="canDelete" type="button"
        class="p-1.5 rounded text-gray-500 hover:text-red-600 hover:bg-red-50/80 transition-all duration-100 focus:outline-none"
        @click="handleDelete" title="Delete">
        <TrashIcon class="h-3.5 w-3.5" />
      </button>

      <!-- More Options Button -->
      <button type="button"
        class="p-1.5 rounded text-gray-500 hover:text-gray-700 hover:bg-gray-50/80 transition-all duration-100 focus:outline-none"
        @click="handleMoreOptions" title="More">
        <EllipsisHorizontalIcon class="h-3.5 w-3.5" />
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import {
  ArrowUturnLeftIcon,
  PencilIcon,
  TrashIcon,
  EllipsisHorizontalIcon,
  LanguageIcon,
} from '@heroicons/vue/24/outline'

// Props
const props = defineProps({
  message: {
    type: Object,
    required: true
  },
  isVisible: {
    type: Boolean,
    default: false
  },
  canEdit: {
    type: Boolean,
    default: false
  },
  canDelete: {
    type: Boolean,
    default: false
  }
})

// Emits
const emit = defineEmits([
  'reply',
  'edit',
  'delete',
  'translate',
  'more-options',
  'hide',
  'keep-visible'
])

// Reactive data
const isHovered = ref(false)

// Methods - 完善版本，集成mention和翻译功能
const handleReply = () => {
  // Reply功能: 集成mention到输入栏
  const replyData = {
    messageId: props.message.id,
    senderId: props.message.sender_id,
    senderName: props.message.sender?.fullname || props.message.sender_name || 'Unknown User',
    content: props.message.content,
    originalMessage: props.message
  }

  console.log('🔄 Reply with mention integration:', replyData)
  emit('reply', replyData)
  emit('hide')
}

const handleTranslate = () => {
  // 🌐 翻译功能: 对接翻译面板
  const translateData = {
    messageId: props.message.id,
    content: props.message.content,
    senderName: props.message.sender?.fullname || props.message.sender_name || 'Unknown User',
    originalMessage: props.message
  }

  console.log('🌐 Translate message:', translateData)
  emit('translate', translateData)
  emit('hide')
}

const handleEdit = () => {
  emit('edit', props.message)
  emit('hide')
}

const handleDelete = () => {
  emit('delete', props.message)
  emit('hide')
}

const handleMoreOptions = () => {
  emit('more-options', props.message)
  emit('hide')
}

const keepVisible = () => {
  isHovered.value = true
  emit('keep-visible')
}

const hideToolbar = () => {
  isHovered.value = false
  setTimeout(() => {
    if (!isHovered.value) {
      emit('hide')
    }
  }, 100) // 小延迟以防止闪烁
}

// 简化: 不再需要复杂的updatePosition方法
// 直接暴露一个空的updatePosition以保持兼容性
defineExpose({
  updatePosition: () => {
    console.log('原生定位模式: 无需位置计算')
  }
})
</script>

<style scoped>
.message-toolbar-optimized {
  pointer-events: auto;
  /* 优化: 减少闪烁，降低认知负担 */
  will-change: opacity;
  transform-origin: top right;
  /* 简化动画，减少闪烁 */
  animation: gentleFadeIn 0.15s ease-out;
  /* 减少视觉干扰 */
  opacity: 0.9;
}

.message-toolbar-optimized:hover {
  opacity: 1;
}

@keyframes gentleFadeIn {
  from {
    opacity: 0;
    transform: translateY(-2px) scale(0.98);
  }

  to {
    opacity: 0.9;
    transform: translateY(0) scale(1);
  }
}

/* 工具栏按钮容器优化 - 更紧凑 */
.message-toolbar-optimized .flex {
  min-height: 28px;
  align-items: center;
}

/* 响应式优化 */
@media (max-width: 768px) {
  .message-toolbar-optimized {
    animation-duration: 0.2s;
    /* 移动端调整位置 */
    right: 4px;
    top: -6px;
  }
}
</style>