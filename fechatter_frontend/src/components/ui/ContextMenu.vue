<template>
  <teleport to="body">
    <div v-if="visible" class="context-menu-overlay" @click="handleOverlayClick" @contextmenu.prevent>
      <div ref="menuRef" class="context-menu" :style="menuStyle" @click.stop>
        <template v-for="(item, index) in items" :key="index">
          <!-- Divider -->
          <div v-if="item.type === 'divider'" class="context-menu-divider"></div>

          <!-- Menu Item -->
          <button v-else class="context-menu-item" :class="{
            'danger': item.danger,
            'disabled': item.disabled
          }" :disabled="item.disabled" @click="handleItemClick(item)">
            <!-- Icon -->
            <Icon v-if="item.icon" :name="item.icon" :size="16" class="menu-item-icon" />

            <!-- Label -->
            <span class="menu-item-label">{{ item.label }}</span>

            <!-- Shortcut -->
            <span v-if="item.shortcut" class="menu-item-shortcut">
              {{ item.shortcut }}
            </span>
          </button>
        </template>
      </div>
    </div>
  </teleport>
</template>

<script setup>
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue'
import Icon from './Icon.vue'

// Props
const props = defineProps({
  x: {
    type: Number,
    required: true
  },
  y: {
    type: Number,
    required: true
  },
  items: {
    type: Array,
    required: true
  },
  visible: {
    type: Boolean,
    default: true
  }
})

// Emits
const emit = defineEmits(['close', 'action'])

// Refs
const menuRef = ref(null)

// Computed
const menuStyle = computed(() => {
  const viewportWidth = window.innerWidth
  const viewportHeight = window.innerHeight
  const menuWidth = 200 // Estimated menu width
  const menuHeight = props.items.length * 32 + 8 // Estimated menu height

  let left = props.x
  let top = props.y

  // Adjust position if menu would go off-screen
  if (left + menuWidth > viewportWidth) {
    left = props.x - menuWidth
  }

  if (top + menuHeight > viewportHeight) {
    top = props.y - menuHeight
  }

  // Ensure menu doesn't go above or to the left of viewport
  left = Math.max(8, left)
  top = Math.max(8, top)

  return {
    left: `${left}px`,
    top: `${top}px`
  }
})

// Methods
const handleOverlayClick = () => {
  emit('close')
}

const handleItemClick = (item) => {
  if (item.disabled) return

  emit('action', item)
  emit('close')
}

const handleKeydown = (event) => {
  if (event.key === 'Escape') {
    emit('close')
  }
}

const handleClickOutside = (event) => {
  if (menuRef.value && !menuRef.value.contains(event.target)) {
    emit('close')
  }
}

// Lifecycle
onMounted(() => {
  nextTick(() => {
    // Focus the menu for keyboard navigation
    if (menuRef.value) {
      menuRef.value.focus()
    }
  })

  document.addEventListener('keydown', handleKeydown)
  document.addEventListener('click', handleClickOutside, true)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  document.removeEventListener('click', handleClickOutside, true)
})
</script>

<style scoped>
.context-menu-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: var(--z-context-menu, 3000);
  /* Transparent overlay to capture clicks */
}

.context-menu {
  position: absolute;
  background: var(--bg-primary);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  padding: 4px;
  min-width: 160px;
  max-width: 250px;
  outline: none;
  font-size: 14px;
  z-index: var(--z-context-menu, 3000);
  /* Ensure proper stacking context */
  isolation: isolate;
}

.context-menu-divider {
  height: 1px;
  background: var(--border-primary);
  margin: 4px 0;
}

.context-menu-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border: none;
  background: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
  font-size: 14px;
  text-align: left;
  color: var(--text-primary);
}

.context-menu-item:hover {
  background: var(--bg-tertiary);
}

.context-menu-item:active {
  background: var(--bg-quaternary);
}

.context-menu-item.danger {
  color: var(--error-color);
}

.context-menu-item.danger:hover {
  background: var(--error-color-alpha);
}

.context-menu-item.disabled {
  color: var(--text-muted);
  cursor: not-allowed;
  opacity: 0.5;
}

.context-menu-item.disabled:hover {
  background: none;
}

.menu-item-icon {
  flex-shrink: 0;
  color: inherit;
}

.menu-item-label {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.menu-item-shortcut {
  flex-shrink: 0;
  font-size: 12px;
  color: var(--text-muted);
  font-family: var(--font-family-mono);
}

/* Dark theme adjustments */
[data-theme="dark"] .context-menu {
  background: var(--bg-secondary);
  border-color: var(--border-secondary);
}

/* High contrast mode */
@media (prefers-contrast: high) {
  .context-menu {
    border-width: 2px;
  }

  .context-menu-item {
    border: 1px solid transparent;
  }

  .context-menu-item:hover {
    border-color: var(--border-focus);
  }
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {

  .context-menu,
  .context-menu-item {
    transition: none;
  }
}

/* Animation */
.context-menu {
  animation: contextMenuFadeIn 0.15s ease-out;
}

@keyframes contextMenuFadeIn {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(-5px);
  }

  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}
</style>