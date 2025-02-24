<template>
  <div class="group relative flex items-start px-4 py-2 transition-all duration-200 hover:bg-gray-50/50 rounded-lg mx-2"
    :class="messageClasses" :data-message-id="message.id || message.temp_id" @contextmenu="handleRightClick"
    @click.right="handleRightClick" @mouseenter="handleShowFloatingToolbar" @mouseleave="handleHideFloatingToolbar"
    ref="messageElement">
    <!-- Debug Data Panel (Development Only) -->
    <div v-if="showDebugData && isDevelopment"
      class="absolute top-0 right-0 z-40 bg-yellow-50 border border-yellow-200 rounded-lg p-3 text-xs shadow-lg max-w-md"
      style="transform: translateY(-100%);">
      <div class="font-bold text-yellow-800 mb-2">🔍 数据传输断点诊断</div>
      <div class="space-y-1 text-yellow-700">
        <div><strong>Message ID:</strong> {{ message.id || message.temp_id }}</div>
        <div><strong>Sender ID:</strong> {{ message.sender_id }}</div>
        <div><strong>Raw sender_name:</strong> {{ message.sender_name || 'null' }}</div>
        <div><strong>Raw sender.fullname:</strong> {{ message.sender?.fullname || 'null' }}</div>
        <div><strong>Raw sender.username:</strong> {{ message.sender?.username || 'null' }}</div>
        <div><strong>Computed senderName:</strong> {{ senderName }}</div>
        <div><strong>Avatar URL:</strong> {{ senderAvatar || 'null' }}</div>
        <div class="mt-2 pt-2 border-t border-yellow-300">
          <strong>Raw Message Object:</strong>
          <pre
            class="text-xs mt-1 p-1 bg-yellow-100 rounded overflow-auto max-h-20">{{ JSON.stringify(message, null, 2) }}</pre>
        </div>
      </div>
    </div>

    <!-- Avatar -->
    <div class="relative mr-4 mt-1 flex-shrink-0">
      <button type="button"
        class="flex h-10 w-10 items-center justify-center rounded-full text-white font-semibold text-sm shadow-lg ring-2 ring-white transition-all duration-200 hover:shadow-xl focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
        :style="{ background: avatarGradient }" @click="handleAvatarClick" @dblclick="toggleDebugData">
        <img v-if="senderAvatar" :src="senderAvatar" :alt="senderName" class="h-full w-full rounded-full object-cover"
          @error="onAvatarError" />
        <span v-else class="text-white font-bold text-sm select-none">
          {{ senderInitials }}
        </span>
      </button>

      <!-- Debug Indicator -->
      <div v-if="isDevelopment"
        class="absolute -top-1 -right-1 bg-yellow-400 text-yellow-800 rounded-full w-4 h-4 flex items-center justify-center text-xs font-bold cursor-help"
        :title="`调试模式 - 用户名源: ${userNameSource}`">
        🔍
      </div>

      <!-- Online Status Indicator -->
      <div v-if="senderOnlineStatus === 'online'"
        class="absolute -bottom-0.5 -right-0.5 bg-green-400 rounded-full w-3 h-3 border-2 border-white">
      </div>
    </div>

    <!-- Message Content -->
    <div class="min-w-0 flex-1">
      <!-- Message Header -->
      <div class="flex items-baseline space-x-2 mb-1">
        <button type="button"
          class="font-semibold text-gray-900 hover:text-blue-600 transition-colors duration-150 focus:outline-none focus:text-blue-600 text-base leading-5"
          @click="handleUsernameClick" :title="`原始数据: ${JSON.stringify({
            sender_name: message.sender_name,
            fullname: message.sender?.fullname,
            username: message.sender?.username
          })}`">
          {{ senderName }}
          <span v-if="isDevelopment && userNameSource" class="text-xs text-gray-500 ml-1">({{ userNameSource }})</span>
        </button>

        <time
          class="text-xs text-gray-500 opacity-0 group-hover:opacity-100 transition-opacity duration-200 font-medium"
          :datetime="message.created_at" :title="fullTimestamp">
          {{ formatTimestamp(message.created_at) }}
        </time>

        <span v-if="isEdited" class="text-xs text-gray-400 italic" title="This message has been edited">
          (edited)
        </span>

        <!-- Message Status -->
        <div v-if="isCurrentUserMessage" class="flex items-center ml-auto">
          <CheckIcon v-if="message.status === 'sent'" class="h-4 w-4 text-green-500" title="Sent" />
          <ClockIcon v-else-if="message.status === 'sending'" class="h-4 w-4 text-gray-400 animate-spin"
            title="Sending..." />
          <ExclamationTriangleIcon v-else-if="message.status === 'failed'" class="h-4 w-4 text-red-500"
            title="Failed to send" />
        </div>
      </div>

      <!-- Reply Reference -->
      <div v-if="message.reply_to"
        class="mb-2 flex items-center space-x-2 rounded-lg bg-gray-50 p-2 text-sm cursor-pointer hover:bg-gray-100 transition-colors duration-150"
        @click="scrollToReplyMessage">
        <ArrowUturnLeftIcon class="h-4 w-4 text-gray-400" />
        <img v-if="replyToAvatar" :src="replyToAvatar" :alt="replyToUsername" class="h-4 w-4 rounded-full" />
        <span class="font-medium text-gray-700">{{ replyToUsername }}</span>
        <span class="text-gray-500 truncate">{{ truncatedReplyContent }}</span>
      </div>

      <!-- Message Body -->
      <div class="space-y-2">
        <!-- Text Content -->
        <div v-if="message.content"
          class="prose prose-sm max-w-none text-gray-900 prose-p:mb-2 prose-p:leading-relaxed prose-code:bg-gray-100 prose-code:rounded prose-code:px-1 prose-code:py-0.5 prose-pre:bg-gray-100 prose-pre:rounded-lg prose-pre:p-3 prose-pre:overflow-x-auto prose-headings:text-gray-900 prose-strong:text-gray-900"
          v-html="renderedContent"></div>

        <!-- File Attachments -->
        <div v-if="message.files && message.files.length > 0" class="space-y-2">
          <div v-for="file in message.files" :key="file.id || file.url"
            class="rounded-lg border border-gray-200 bg-white shadow-sm transition-all duration-200 hover:shadow-md">
            <!-- Image Attachment -->
            <div v-if="isImageFile(file)" class="relative">
              <div v-if="!imageLoaded[file.id]" class="flex h-48 items-center justify-center bg-gray-100 rounded-lg">
                <div class="animate-pulse">
                  <PhotoIcon class="h-12 w-12 text-gray-400" />
                </div>
              </div>
              <img v-show="imageLoaded[file.id]" :src="file.url || file.file_url" :alt="file.filename || file.file_name"
                class="max-h-80 w-full rounded-lg object-cover cursor-pointer transition-transform duration-200"
                @load="handleImageLoad(file.id)" @click="openImagePreview(file)" @error="handleImageError(file.id)" />
              <div class="absolute bottom-2 left-2 rounded bg-black/70 px-2 py-1 text-xs text-white backdrop-blur-sm">
                {{ file.filename || file.file_name }}
              </div>
            </div>

            <!-- Other File Types -->
            <div v-else class="flex items-center space-x-3 p-3">
              <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-blue-100">
                <DocumentIcon class="h-6 w-6 text-blue-600" />
              </div>
              <div class="flex-1 min-w-0">
                <p class="font-medium text-gray-900 truncate">
                  {{ file.filename || file.file_name }}
                </p>
                <p class="text-sm text-gray-500">
                  {{ formatFileSize(file.size || file.file_size) }}
                </p>
              </div>
              <button type="button"
                class="flex items-center justify-center h-8 w-8 rounded-lg bg-gray-100 text-gray-600 hover:bg-gray-200 transition-colors duration-150 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                @click="downloadFile(file)" title="Download file">
                <ArrowDownTrayIcon class="h-4 w-4" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Floating Message Toolbar - 完善版本 -->
    <FloatingMessageToolbar :message="message" :is-visible="showFloatingToolbar" :can-edit="canEdit"
      :can-delete="canDelete" @reply="handleReplyToMessage" @translate="handleTranslateMessage" @edit="startEdit"
      @delete="deleteMessage" @more-options="handleRightClick" @hide="handleToolbarHide"
      @keep-visible="keepFloatingToolbar" />

    <!-- Context Menu - 悬浮菜单，不影响消息布局 -->
    <Teleport to="body">
      <Menu v-if="showContextMenu" as="div" class="fixed z-[9999] context-menu" :style="contextMenuStyle">
        <MenuButton class="sr-only">Options</MenuButton>
        <transition enter-active-class="transition duration-150 ease-out"
          enter-from-class="transform scale-95 opacity-0" enter-to-class="transform scale-100 opacity-100"
          leave-active-class="transition duration-100 ease-in" leave-from-class="transform scale-100 opacity-100"
          leave-to-class="transform scale-95 opacity-0">
          <MenuItems
            class="origin-top-left rounded-lg bg-white py-2 shadow-xl ring-1 ring-black ring-opacity-5 focus:outline-none min-w-48 border border-gray-200 backdrop-blur-sm transform-gpu"
            style="transform-origin: top left;">
            <MenuItem v-if="canEdit" v-slot="{ active }">
            <button type="button"
              class="flex w-full items-center px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 transition-colors duration-150"
              @click="startEdit">
              <PencilIcon class="mr-3 h-4 w-4" />
              Edit message
            </button>
            </MenuItem>

            <MenuItem v-if="canDelete" v-slot="{ active }">
            <button type="button"
              class="flex w-full items-center px-4 py-2 text-sm text-red-700 hover:bg-red-50 transition-colors duration-150"
              @click="deleteMessage">
              <TrashIcon class="mr-3 h-4 w-4" />
              Delete message
            </button>
            </MenuItem>

            <MenuItem v-slot="{ active }">
            <button type="button"
              class="flex w-full items-center px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 transition-colors duration-150"
              @click="handleReplyToMessage">
              <ArrowUturnLeftIcon class="mr-3 h-4 w-4" />
              Reply
            </button>
            </MenuItem>

            <MenuItem v-slot="{ active }">
            <button type="button"
              class="flex w-full items-center px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 transition-colors duration-150"
              @click="translateMessage">
              <LanguageIcon class="mr-3 h-4 w-4" />
              Translate
            </button>
            </MenuItem>

            <MenuItem v-slot="{ active }">
            <button type="button"
              class="flex w-full items-center px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 transition-colors duration-150"
              @click="copyMessage">
              <ClipboardDocumentIcon class="mr-3 h-4 w-4" />
              Copy message
            </button>
            </MenuItem>

            <!-- Debug Menu Item -->
            <MenuItem v-if="isDevelopment" v-slot="{ active }">
            <button type="button"
              class="flex w-full items-center px-4 py-2 text-sm text-yellow-700 hover:bg-yellow-50 transition-colors duration-150"
              @click="logMessageData">
              <svg class="mr-3 h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
                <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              Debug Data
            </button>
            </MenuItem>
          </MenuItems>
        </transition>
      </Menu>
    </Teleport>

    <!-- Image Preview Modal -->
    <TransitionRoot appear :show="showImagePreview" as="template">
      <Dialog as="div" class="relative z-50" @close="closeImagePreview">
        <TransitionChild as="template" enter="duration-300 ease-out" enter-from="opacity-0" enter-to="opacity-100"
          leave="duration-200 ease-in" leave-from="opacity-100" leave-to="opacity-0">
          <div class="fixed inset-0 bg-black/75 backdrop-blur-sm" />
        </TransitionChild>

        <div class="fixed inset-0 overflow-y-auto">
          <div class="flex min-h-full items-center justify-center p-4 text-center">
            <TransitionChild as="template" enter="duration-300 ease-out" enter-from="opacity-0 scale-95"
              enter-to="opacity-100 scale-100" leave="duration-200 ease-in" leave-from="opacity-100 scale-100"
              leave-to="opacity-0 scale-95">
              <DialogPanel
                class="w-full max-w-4xl transform overflow-hidden rounded-2xl bg-white p-6 text-left align-middle shadow-xl transition-all">
                <div class="flex items-center justify-between mb-4">
                  <DialogTitle as="h3" class="text-lg font-medium leading-6 text-gray-900">
                    {{ previewImageAlt }}
                  </DialogTitle>
                  <button type="button"
                    class="rounded-md bg-white text-gray-400 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                    @click="closeImagePreview">
                    <XMarkIcon class="h-6 w-6" />
                  </button>
                </div>

                <div class="aspect-w-16 aspect-h-9">
                  <img :src="previewImageSrc" :alt="previewImageAlt"
                    class="max-h-[70vh] w-full object-contain rounded-lg" />
                </div>
              </DialogPanel>
            </TransitionChild>
          </div>
        </div>
      </Dialog>
    </TransitionRoot>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { useChatStore } from '@/stores/chat'
import { useVirtualList } from '@vueuse/core'
import {
  Menu,
  MenuButton,
  MenuItems,
  MenuItem,
  Dialog,
  DialogPanel,
  DialogTitle,
  TransitionRoot,
  TransitionChild,
} from '@headlessui/vue'
import {
  CheckIcon,
  ClockIcon,
  ExclamationTriangleIcon,
  ArrowUturnLeftIcon,
  PhotoIcon,
  DocumentIcon,
  ArrowDownTrayIcon,
  PencilIcon,
  TrashIcon,
  LanguageIcon,
  ClipboardDocumentIcon,
  XMarkIcon,
} from '@heroicons/vue/24/outline'
import { formatTimestamp, formatFileSize } from '@/utils/formatters'
import { renderMarkdown } from '@/utils/markdown'
import { highlightCodeAsync } from '@/utils/codeHighlight'
import FloatingMessageToolbar from '@/components/chat/FloatingMessageToolbar.vue'

// Props
const props = defineProps({
  message: {
    type: Object,
    required: true
  },
  currentUserId: {
    type: Number,
    required: false,
    default: 0
  },
  chatId: {
    type: [Number, String],
    default: null
  },
  isGrouped: {
    type: Boolean,
    default: false
  },
  isConsecutive: {
    type: Boolean,
    default: false
  },
  previousMessage: {
    type: Object,
    default: null
  },
  showDebugData: {
    type: Boolean,
    default: false
  },
  isDevelopment: {
    type: Boolean,
    default: false
  },
  userNameSource: {
    type: String,
    default: null
  }
})

// Emits
const emit = defineEmits([
  'user-profile-opened',
  'dm-created',
  'reply-to',
  'edit-message',
  'delete-message',
  'scroll-to-message'
])

// Stores
const authStore = useAuthStore()
const chatStore = useChatStore()

// Reactive data
const imageLoaded = ref({})
const showContextMenu = ref(false)
const contextMenuPosition = ref({ x: 0, y: 0 })
const showActions = ref(false)
const showImagePreview = ref(false)
const previewImageSrc = ref('')
const previewImageAlt = ref('')
const avatarError = ref(false)
const showDebugData = ref(false)
const messageElement = ref(null)
const showFloatingToolbar = ref(false)
const floatingToolbar = ref(null)
const toolbarHovered = ref(false)

// ✨ Enhanced Code Highlighting State
const highlightedContent = ref('')
const isHighlightingCode = ref(false)
const highlightError = ref(null)

// Development mode detection
const isDevelopment = computed(() => {
  return import.meta.env.DEV || import.meta.env.MODE === 'development'
})

// Modern professional color palette
const AVATAR_COLORS = [
  ['#3B82F6', '#8B5CF6'], // Blue to Purple  
  ['#10B981', '#06B6D4'], // Green to Teal
  ['#8B5CF6', '#EC4899'], // Purple to Pink
  ['#EF4444', '#F97316'], // Red to Orange
  ['#F59E0B', '#EAB308'], // Orange to Yellow
  ['#06B6D4', '#0EA5E9'], // Teal to Sky
  ['#EC4899', '#F43F5E'], // Pink to Rose
  ['#6366F1', '#3B82F6'], // Indigo to Blue
]

// Computed properties
const isCurrentUserMessage = computed(() => {
  return props.message.sender_id === props.currentUserId ||
    props.message.sender_id === authStore.user?.id
})

const messageClasses = computed(() => ({
  'bg-blue-50/30': isCurrentUserMessage.value,
  'border-l-2 border-blue-400': isCurrentUserMessage.value,
}))

// Enhanced username detection with source tracking
const senderName = computed(() => {
  let name = 'Unknown User'
  let source = 'fallback'

  if (props.message.sender?.fullname) {
    name = props.message.sender.fullname
    source = 'sender.fullname'
  } else if (props.message.sender_name) {
    name = props.message.sender_name
    source = 'sender_name'
  } else if (props.message.sender?.username) {
    name = props.message.sender.username
    source = 'sender.username'
  } else if (props.message.sender?.name) {
    name = props.message.sender.name
    source = 'sender.name'
  }

  // Log data transmission analysis in development
  if (isDevelopment.value) {
    console.log(`🔍 [${props.message.id}] Username源追踪:`, {
      source,
      value: name,
      rawData: {
        'sender.fullname': props.message.sender?.fullname,
        'sender_name': props.message.sender_name,
        'sender.username': props.message.sender?.username,
        'sender.name': props.message.sender?.name,
      },
      fullMessage: props.message
    })
  }

  return name
})

// Track username source for debugging
const userNameSource = computed(() => {
  if (!isDevelopment.value) return null

  if (props.message.sender?.fullname) return 'fullname'
  if (props.message.sender_name) return 'sender_name'
  if (props.message.sender?.username) return 'username'
  if (props.message.sender?.name) return 'name'
  return 'unknown'
})

const senderInitials = computed(() => {
  const name = senderName.value
  if (name === 'Unknown User') return 'U'

  const words = name.trim().split(/\s+/)
  if (words.length >= 2) {
    return (words[0].charAt(0) + words[words.length - 1].charAt(0)).toUpperCase()
  }
  return name.substring(0, 2).toUpperCase()
})

const senderAvatar = computed(() => {
  if (avatarError.value) return null
  return props.message.sender?.avatar_url ||
    props.message.sender_avatar
})

const avatarGradient = computed(() => {
  const userId = props.message.sender_id || senderName.value
  const hash = String(userId).split('').reduce((a, b) => {
    a = ((a << 5) - a) + b.charCodeAt(0)
    return a & a
  }, 0)
  const index = Math.abs(hash) % AVATAR_COLORS.length
  const colors = AVATAR_COLORS[index]
  return `linear-gradient(135deg, ${colors[0]}, ${colors[1]})`
})

const senderOnlineStatus = computed(() => {
  // TODO: Implement real online status
  return props.message.sender?.is_online ? 'online' : 'offline'
})

const fullTimestamp = computed(() => {
  if (!props.message.created_at) return 'Invalid date'

  const date = new Date(props.message.created_at)
  const now = new Date()
  const diffInMinutes = Math.floor((now - date) / (1000 * 60))

  // 基础时间格式：精确到分钟
  const dateString = date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    weekday: 'short'
  })

  const timeString = date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false
  })

  // 相对时间信息
  let relativeTime = ''
  if (diffInMinutes < 1) {
    relativeTime = '刚刚'
  } else if (diffInMinutes < 60) {
    relativeTime = `${diffInMinutes}分钟前`
  } else if (diffInMinutes < 1440) {
    const hours = Math.floor(diffInMinutes / 60)
    const remainingMinutes = diffInMinutes % 60
    relativeTime = remainingMinutes > 0
      ? `${hours}小时${remainingMinutes}分钟前`
      : `${hours}小时前`
  } else {
    const days = Math.floor(diffInMinutes / 1440)
    relativeTime = `${days}天前`
  }

  return `${dateString} ${timeString} (${relativeTime})`
})

const isEdited = computed(() => {
  return props.message.updated_at &&
    props.message.updated_at !== props.message.created_at
})

const canEdit = computed(() => {
  return isCurrentUserMessage.value &&
    props.message.status !== 'failed'
})

const canDelete = computed(() => {
  return isCurrentUserMessage.value
})

const renderedContent = computed(() => {
  if (!props.message.content) return ''

  // 🎯 Use cached highlighted content if available
  if (highlightedContent.value) {
    return highlightedContent.value
  }

  // Fallback to basic markdown rendering
  return renderMarkdown(props.message.content)
})

const contextMenuStyle = computed(() => ({
  left: `${contextMenuPosition.value.x}px`,
  top: `${contextMenuPosition.value.y}px`,
}))

// Reply-related computed properties
const replyToMessage = computed(() => {
  if (!props.message.reply_to) return null
  return chatStore.getMessageById(props.message.reply_to)
})

const replyToUsername = computed(() => {
  if (!replyToMessage.value) return ''
  return replyToMessage.value.sender?.fullname ||
    replyToMessage.value.sender_name ||
    'Unknown User'
})

const replyToAvatar = computed(() => {
  if (!replyToMessage.value) return null
  return replyToMessage.value.sender?.avatar_url ||
    replyToMessage.value.sender_avatar ||
    null
})

const truncatedReplyContent = computed(() => {
  if (!replyToMessage.value?.content) return 'Click to see message'
  const content = replyToMessage.value.content
  return content.length > 50 ? content.substring(0, 50) + '...' : content
})

// Methods
const onAvatarError = () => {
  avatarError.value = true
}

const handleAvatarClick = () => {
  if (props.message.sender) {
    emit('user-profile-opened', props.message.sender)
  }
}

const handleUsernameClick = () => {
  handleAvatarClick()
}

// Floating toolbar methods - 简化版本
const handleShowFloatingToolbar = () => {
  showFloatingToolbar.value = true
}

const handleHideFloatingToolbar = () => {
  // 延迟隐藏，给用户时间移动到工具栏上
  setTimeout(() => {
    if (!toolbarHovered.value) {
      showFloatingToolbar.value = false
    }
  }, 150)
}

const keepFloatingToolbar = () => {
  toolbarHovered.value = true
}

const handleToolbarHide = () => {
  toolbarHovered.value = false
  showFloatingToolbar.value = false
}

const handleRightClick = (event) => {
  event.preventDefault()
  event.stopPropagation()

  console.log('🔍 右键菜单调试信息:', {
    clientX: event.clientX,
    clientY: event.clientY,
    pageX: event.pageX,
    pageY: event.pageY,
    viewportWidth: window.innerWidth,
    viewportHeight: window.innerHeight,
    scrollX: window.scrollX,
    scrollY: window.scrollY
  })

  // 获取菜单预估尺寸
  const menuWidth = 200
  const menuHeight = 280 // 增加高度以适应更多菜单项

  // 获取视口尺寸
  const viewportWidth = window.innerWidth
  const viewportHeight = window.innerHeight

  // 计算基础位置 - 使用鼠标点击位置
  let x = event.clientX
  let y = event.clientY

  // 智能水平定位：优先显示在右侧，空间不足时显示在左侧
  if (x + menuWidth > viewportWidth - 20) {
    x = Math.max(20, x - menuWidth) // 显示在鼠标左侧
  } else {
    x = x + 5 // 显示在鼠标右侧，稍微偏移避免遮挡
  }

  // 智能垂直定位：优先显示在下方，空间不足时显示在上方
  if (y + menuHeight > viewportHeight - 20) {
    y = Math.max(20, y - menuHeight) // 显示在鼠标上方
  } else {
    y = y + 5 // 显示在鼠标下方，稍微偏移
  }

  // 最终边界检查
  x = Math.max(20, Math.min(x, viewportWidth - menuWidth - 20))
  y = Math.max(20, Math.min(y, viewportHeight - menuHeight - 20))

  console.log('📍 菜单最终位置:', { x, y })

  contextMenuPosition.value = { x, y }
  showContextMenu.value = true

  // 添加一个小延迟来确保菜单正确显示
  setTimeout(() => {
    const menuElement = document.querySelector('.context-menu')
    if (menuElement) {
      console.log('✅ 菜单元素状态:', {
        position: getComputedStyle(menuElement).position,
        left: getComputedStyle(menuElement).left,
        top: getComputedStyle(menuElement).top,
        zIndex: getComputedStyle(menuElement).zIndex,
        display: getComputedStyle(menuElement).display
      })
    }
  }, 50)
}

const closeContextMenu = () => {
  showContextMenu.value = false
}

// 处理点击外部关闭菜单
const handleClickOutside = (event) => {
  if (showContextMenu.value) {
    // 检查点击是否在菜单内部
    const menuElement = document.querySelector('.context-menu')
    if (menuElement && !menuElement.contains(event.target)) {
      closeContextMenu()
    }
  }
}

// 处理ESC键关闭菜单
const handleEscapeKey = (event) => {
  if (event.key === 'Escape' && showContextMenu.value) {
    closeContextMenu()
  }
}

const handleImageLoad = (fileId) => {
  imageLoaded.value[fileId] = true
}

const handleImageError = (fileId) => {
  imageLoaded.value[fileId] = false
}

const openImagePreview = (file) => {
  previewImageSrc.value = file.url || file.file_url
  previewImageAlt.value = file.filename || file.file_name
  showImagePreview.value = true
}

const closeImagePreview = () => {
  showImagePreview.value = false
}

const isImageFile = (file) => {
  const filename = file.filename || file.file_name || ''
  const imageExts = ['.jpg', '.jpeg', '.png', '.gif', '.webp', '.svg']
  return imageExts.some(ext => filename.toLowerCase().endsWith(ext))
}

const downloadFile = (file) => {
  const url = file.url || file.file_url
  const filename = file.filename || file.file_name

  const link = document.createElement('a')
  link.href = url
  link.download = filename
  link.click()
}

const scrollToReplyMessage = () => {
  if (props.message.reply_to) {
    emit('scroll-to-message', props.message.reply_to)
  }
}

const startEdit = () => {
  emit('edit-message', props.message)
  closeContextMenu()
}

const deleteMessage = () => {
  emit('delete-message', props.message)
  closeContextMenu()
}

const handleReplyToMessage = (replyData) => {
  // 🔄 Reply with Mention Integration
  const enhancedReplyData = {
    messageId: props.message.id,
    senderId: props.message.sender_id,
    senderName: props.message.sender?.fullname || props.message.sender_name || 'Unknown User',
    content: props.message.content,
    originalMessage: props.message,
    replyType: 'mention', // 标识这是一个mention回复
    timestamp: new Date().toISOString()
  }

  console.log('🔄 Enhanced Reply with mention integration:', enhancedReplyData)

  // Emit to parent for input field integration
  emit('reply-to', enhancedReplyData)

  // 触发输入栏focus并设置mention
  setTimeout(() => {
    const messageInput = document.querySelector('textarea[placeholder*="message"], input[placeholder*="message"]')
    if (messageInput) {
      const mentionText = `@${enhancedReplyData.senderName} `
      messageInput.value = mentionText
      messageInput.focus()
      // 设置光标位置到末尾
      messageInput.setSelectionRange(mentionText.length, mentionText.length)
      console.log('✅ Message input focused with mention:', mentionText)
    }
  }, 100)

  closeContextMenu()
}

const handleTranslateMessage = async (translateData) => {
  // 🌐 Translation Panel Integration - FIXED: Use proper Vue state management
  const translationRequest = {
    messageId: props.message.id,
    content: props.message.content,
    senderName: props.message.sender?.fullname || props.message.sender_name || 'Unknown User',
    originalMessage: props.message,
    timestamp: new Date().toISOString()
  }

  console.log('🌐 Translation request:', translationRequest)

  // 🔧 FIXED: Use messageUIStore state management with optimal positioning
  try {
    // Import messageUIStore
    const { useMessageUIStore } = await import('@/stores/messageUI')
    const messageUIStore = useMessageUIStore()

    // 🎯 优化：让Chat.vue的getOptimalTranslationPanelPosition处理位置计算
    // 移除position参数，使用最优定位算法
    messageUIStore.openTranslationPanel(props.message.id, {
      showAdvanced: false,
      preserveFormatting: true,
      showConfidence: true
    })

    console.log('✅ Translation panel opened via state management with optimal positioning')
  } catch (error) {
    console.error('🚨 Failed to open translation panel:', error)

    // Fallback to temporary dialog
    showTranslationDialog(translationRequest)
  }

  closeContextMenu()
}

// 临时翻译对话框（如果没有翻译面板时使用）
const showTranslationDialog = (request) => {
  // 创建简单的翻译提示
  const dialog = document.createElement('div')
  dialog.className = 'fixed top-4 right-4 bg-white rounded-lg shadow-lg border p-4 z-50 max-w-sm'
  dialog.innerHTML = `
    <div class="flex items-center justify-between mb-2">
      <h3 class="font-semibold text-gray-900">🌐 Translation</h3>
      <button onclick="this.parentElement.parentElement.remove()" class="text-gray-400 hover:text-gray-600">✕</button>
    </div>
    <p class="text-sm text-gray-600 mb-2">Original: "${request.content.substring(0, 100)}${request.content.length > 100 ? '...' : ''}"</p>
    <p class="text-sm text-blue-600">Translation feature will be integrated with bot service...</p>
  `
  document.body.appendChild(dialog)

  // 3秒后自动移除
  setTimeout(() => {
    if (dialog.parentElement) {
      dialog.remove()
    }
  }, 3000)
}

const translateMessage = () => {
  // 🔄 Redirect to enhanced translate method
  handleTranslateMessage()
}

const copyMessage = () => {
  if (props.message.content) {
    navigator.clipboard.writeText(props.message.content)
  }
  closeContextMenu()
}

// Debug methods
const toggleDebugData = () => {
  showDebugData.value = !showDebugData.value
  if (showDebugData.value) {
    logMessageData()
  }
}

const logMessageData = () => {
  console.group(`🔍 数据传输断点分析 - Message ${props.message.id}`)

  console.log('📋 原始消息对象:', props.message)

  console.log('👤 用户名数据源分析:', {
    'sender.fullname': props.message.sender?.fullname || '❌ null',
    'sender_name': props.message.sender_name || '❌ null',
    'sender.username': props.message.sender?.username || '❌ null',
    'sender.name': props.message.sender?.name || '❌ null',
    '最终显示': senderName.value,
    '数据源': userNameSource.value
  })

  console.log('🎨 头像数据源分析:', {
    'sender.avatar_url': props.message.sender?.avatar_url || '❌ null',
    'sender_avatar': props.message.sender_avatar || '❌ null',
    '最终显示': senderAvatar.value || '❌ 使用fallback',
    'fallback初始字母': senderInitials.value
  })

  console.log('🔗 数据传输链路检查:', {
    '消息ID': props.message.id || props.message.temp_id,
    '发送者ID': props.message.sender_id,
    '是否有sender对象': !!props.message.sender,
    'sender对象内容': props.message.sender || '❌ null',
    '创建时间': props.message.created_at,
    '消息内容': props.message.content
  })

  // Check for potential data loss points
  const dataLossIndicators = []
  if (!props.message.sender && !props.message.sender_name) {
    dataLossIndicators.push('❌ 缺少所有用户名数据源')
  }
  if (!props.message.sender?.fullname && !props.message.sender_name) {
    dataLossIndicators.push('⚠️ 只有fallback用户名数据')
  }
  if (!props.message.sender?.avatar_url && !props.message.sender_avatar) {
    dataLossIndicators.push('⚠️ 缺少头像数据，使用生成头像')
  }

  if (dataLossIndicators.length > 0) {
    console.warn('🚨 发现数据传输断点:', dataLossIndicators)
  } else {
    console.log('✅ 数据传输完整')
  }

  console.groupEnd()
  closeContextMenu()
}

// ✨ Enhanced Code Highlighting Methods
const highlightCodeInContent = async () => {
  if (!props.message.content || isHighlightingCode.value) return

  try {
    isHighlightingCode.value = true
    highlightError.value = null

    // First render basic markdown
    let content = renderMarkdown(props.message.content)

    // Check if content contains code blocks
    const hasCodeBlocks = /```[\s\S]*?```/g.test(props.message.content)

    if (hasCodeBlocks) {
      // Apply async code highlighting
      const { highlightMarkdownCode } = await import('@/utils/codeHighlight')

      content = await highlightMarkdownCode(props.message.content, {
        theme: 'dark', // TODO: Get from theme store
        lineNumbers: true,
        cache: true
      })
    }

    highlightedContent.value = content

    if (import.meta.env.DEV) {
      console.log(`✨ [${props.message.id}] Code highlighting completed`)
    }
  } catch (error) {
    highlightError.value = error
    console.error('💥 Code highlighting failed:', error)

    // Fallback to basic markdown
    highlightedContent.value = renderMarkdown(props.message.content)
  } finally {
    isHighlightingCode.value = false
  }
}

// Lifecycle
onMounted(async () => {
  // 🔧 CRITICAL FIX: Mark message as displayed for MessageDisplayGuarantee
  const messageId = props.message.id || props.message.temp_id
  if (messageId && window.messageDisplayGuarantee) {
    // Get the actual DOM element
    const messageElement = document.querySelector(`[data-message-id="${messageId}"]`)
    window.messageDisplayGuarantee.markMessageDisplayed(messageId, messageElement, props.chatId)

    if (isDevelopment.value) {
      console.log(`✅ [DiscordMessageItem] Marked message ${messageId} as displayed`)
    }
  }

  // ✨ Initialize Code Highlighting
  await highlightCodeInContent()

  // 监听点击外部关闭菜单
  document.addEventListener('click', closeContextMenu)

  // 监听ESC键关闭菜单
  document.addEventListener('keydown', (event) => {
    if (event.key === 'Escape' && showContextMenu.value) {
      closeContextMenu()
    }
  })

  // Show actions on hover
  const element = document.querySelector(`[data-message-id="${props.message.id || props.message.temp_id}"]`)
  if (element) {
    element.addEventListener('mouseenter', () => { showActions.value = true })
    element.addEventListener('mouseleave', () => { showActions.value = false })
  }

  // Auto-analyze data in development on mount
  if (isDevelopment.value && (!props.message.sender?.fullname && !props.message.sender_name)) {
    console.warn(`🚨 [${props.message.id}] 检测到数据传输断点 - 缺少用户名数据`)
    setTimeout(() => logMessageData(), 100)
  }
})

onUnmounted(() => {
  document.removeEventListener('click', closeContextMenu)
  document.removeEventListener('keydown', (event) => {
    if (event.key === 'Escape' && showContextMenu.value) {
      closeContextMenu()
    }
  })
})
</script>