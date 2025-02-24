<!--
  ContextualMenu.vue
  Jobs-Inspired Contextual Menu Component
  
  Philosophy:
  - Simplify: Clean, focused menu design
  - Anticipate: Smart positioning and behavior
  - Perfect: Pixel-perfect alignment and animations
  - Focus: Essential actions only
-->
<template>
  <div class="contextual-menu-container" ref="containerRef">
    <!-- Trigger Slot -->
    <div ref="triggerRef" @click="handleTriggerClick" class="menu-trigger">
      <slot />
    </div>

    <!-- Menu Backdrop -->
    <div v-if="modelValue" class="menu-backdrop" @click="handleBackdropClick"></div>

    <!-- Menu Panel -->
    <Transition name="menu-slide">
      <div v-if="modelValue" ref="menuRef" :class="menuClasses" :style="menuStyle" @click.stop>
        <!-- Menu Items -->
        <template v-for="(item, index) in items" :key="item.id || index">

          <!-- Divider -->
          <div v-if="item.type === 'divider'" class="menu-divider"></div>

          <!-- Menu Item -->
          <button v-else :class="menuItemClasses(item)" @click="handleItemClick(item)" :disabled="item.disabled">
            <!-- Icon -->
            <Icon v-if="item.icon" :name="item.icon" :size="16" class="menu-item-icon" />

            <!-- Label -->
            <span class="menu-item-label">{{ item.label }}</span>

            <!-- Shortcut -->
            <span v-if="item.shortcut" class="menu-item-shortcut">
              {{ item.shortcut }}
            </span>

            <!-- Submenu Indicator -->
            <Icon v-if="item.submenu" name="chevron-right" :size="14" class="menu-item-arrow" />
          </button>
        </template>
      </div>
    </Transition>
  </div>
</template>

<script setup>
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import Icon from '@/components/ui/Icon.vue';

// Props
const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false
  },
  items: {
    type: Array,
    default: () => []
  },
  placement: {
    type: String,
    default: 'bottom-start',
    validator: (value) => [
      'top', 'top-start', 'top-end',
      'bottom', 'bottom-start', 'bottom-end',
      'left', 'left-start', 'left-end',
      'right', 'right-start', 'right-end'
    ].includes(value)
  },
  offset: {
    type: Number,
    default: 4
  },
  closeOnClick: {
    type: Boolean,
    default: true
  }
});

// Emits
const emit = defineEmits([
  'update:modelValue',
  'action',
  'open',
  'close'
]);

// Reactive state
const containerRef = ref(null);
const triggerRef = ref(null);
const menuRef = ref(null);
const menuPosition = ref({ x: 0, y: 0 });
const menuDirection = ref('bottom');

// Computed properties
const menuClasses = computed(() => [
  'contextual-menu',
  `contextual-menu--${menuDirection.value}`,
  {
    'contextual-menu--visible': props.modelValue
  }
]);

const menuStyle = computed(() => ({
  left: `${menuPosition.value.x}px`,
  top: `${menuPosition.value.y}px`,
}));

const menuItemClasses = (item) => [
  'menu-item',
  {
    'menu-item--danger': item.variant === 'danger',
    'menu-item--disabled': item.disabled,
    'menu-item--with-icon': item.icon,
    'menu-item--with-submenu': item.submenu
  }
];

// Event handlers
const handleTriggerClick = () => {
  if (props.modelValue) {
    closeMenu();
  } else {
    openMenu();
  }
};

const handleBackdropClick = () => {
  closeMenu();
};

const handleItemClick = (item) => {
  if (item.disabled) return;

  emit('action', item);

  if (props.closeOnClick && !item.submenu) {
    closeMenu();
  }
};

const openMenu = async () => {
  emit('update:modelValue', true);
  emit('open');

  await nextTick();
  calculatePosition();
};

const closeMenu = () => {
  emit('update:modelValue', false);
  emit('close');
};

// Position calculation - Jobs loved smart positioning
const calculatePosition = () => {
  if (!triggerRef.value || !menuRef.value) return;

  const trigger = triggerRef.value;
  const menu = menuRef.value;
  const viewport = {
    width: window.innerWidth,
    height: window.innerHeight,
    scrollX: window.scrollX,
    scrollY: window.scrollY
  };

  const triggerRect = trigger.getBoundingClientRect();
  const menuRect = menu.getBoundingClientRect();

  let x = 0;
  let y = 0;
  let direction = 'bottom';

  // Calculate position based on placement
  switch (props.placement) {
    case 'bottom':
    case 'bottom-start':
      x = triggerRect.left;
      y = triggerRect.bottom + props.offset;
      direction = 'bottom';
      break;

    case 'bottom-end':
      x = triggerRect.right - menuRect.width;
      y = triggerRect.bottom + props.offset;
      direction = 'bottom';
      break;

    case 'top':
    case 'top-start':
      x = triggerRect.left;
      y = triggerRect.top - menuRect.height - props.offset;
      direction = 'top';
      break;

    case 'top-end':
      x = triggerRect.right - menuRect.width;
      y = triggerRect.top - menuRect.height - props.offset;
      direction = 'top';
      break;

    case 'right':
    case 'right-start':
      x = triggerRect.right + props.offset;
      y = triggerRect.top;
      direction = 'right';
      break;

    case 'left':
    case 'left-start':
      x = triggerRect.left - menuRect.width - props.offset;
      y = triggerRect.top;
      direction = 'left';
      break;
  }

  // Viewport boundary checks and adjustments
  if (x + menuRect.width > viewport.width) {
    x = viewport.width - menuRect.width - 8;
  }
  if (x < 8) {
    x = 8;
  }

  if (y + menuRect.height > viewport.height) {
    // Flip to opposite side
    if (direction === 'bottom') {
      y = triggerRect.top - menuRect.height - props.offset;
      direction = 'top';
    }
  }
  if (y < 8) {
    // Flip to opposite side
    if (direction === 'top') {
      y = triggerRect.bottom + props.offset;
      direction = 'bottom';
    }
  }

  menuPosition.value = { x, y };
  menuDirection.value = direction;
};

// Keyboard handling
const handleKeydown = (event) => {
  if (!props.modelValue) return;

  switch (event.key) {
    case 'Escape':
      event.preventDefault();
      closeMenu();
      break;

    case 'ArrowDown':
      event.preventDefault();
      // Focus next item
      break;

    case 'ArrowUp':
      event.preventDefault();
      // Focus previous item
      break;

    case 'Enter':
    case ' ':
      event.preventDefault();
      // Activate focused item
      break;
  }
};

// Watchers
watch(() => props.modelValue, (newValue) => {
  if (newValue) {
    nextTick(() => calculatePosition());
  }
});

// Lifecycle
onMounted(() => {
  document.addEventListener('keydown', handleKeydown);
  window.addEventListener('resize', calculatePosition);
  window.addEventListener('scroll', calculatePosition);
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown);
  window.removeEventListener('resize', calculatePosition);
  window.removeEventListener('scroll', calculatePosition);
});
</script>

<style scoped>
/*
  Jobs-Inspired Contextual Menu Design
  - Clean, minimal appearance
  - Smart positioning and animations
  - Premium glass effects
  - Perfect spacing and typography
*/

.contextual-menu-container {
  position: relative;
  display: inline-block;
}

.menu-trigger {
  display: contents;
}

.menu-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: var(--z-modal-backdrop, 9000);
  background: transparent;
}

.contextual-menu {
  position: fixed;
  min-width: 200px;
  max-width: 320px;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(40px);
  border: 1px solid rgba(0, 0, 0, 0.08);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
  padding: 6px;
  z-index: var(--z-modal, 9500);
  overflow: hidden;

  /* Typography */
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
  font-size: 14px;
  line-height: 1.4;
}

/* Menu positioning variants */
.contextual-menu--bottom {
  transform-origin: top center;
}

.contextual-menu--top {
  transform-origin: bottom center;
}

.contextual-menu--left {
  transform-origin: center right;
}

.contextual-menu--right {
  transform-origin: center left;
}

/* Menu item */
.menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  border-radius: 8px;
  background: none;
  color: #1a1a1a;
  font-size: 14px;
  font-weight: 500;
  text-align: left;
  cursor: pointer;
  transition: all 0.15s ease;
  position: relative;
  outline: none;
}

.menu-item:hover:not(.menu-item--disabled) {
  background: rgba(0, 122, 255, 0.08);
  color: #007AFF;
}

.menu-item:active:not(.menu-item--disabled) {
  background: rgba(0, 122, 255, 0.12);
  transform: scale(0.98);
}

.menu-item--danger {
  color: #FF3B30;
}

.menu-item--danger:hover:not(.menu-item--disabled) {
  background: rgba(255, 59, 48, 0.08);
  color: #FF3B30;
}

.menu-item--disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.menu-item--with-icon {
  padding-left: 12px;
}

/* Menu item elements */
.menu-item-icon {
  flex-shrink: 0;
  color: inherit;
}

.menu-item-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.menu-item-shortcut {
  flex-shrink: 0;
  font-size: 12px;
  color: rgba(0, 0, 0, 0.5);
  font-family: -apple-system, BlinkMacSystemFont, 'SF Mono', 'Monaco', monospace;
}

.menu-item-arrow {
  flex-shrink: 0;
  color: rgba(0, 0, 0, 0.4);
}

/* Divider */
.menu-divider {
  height: 1px;
  background: rgba(0, 0, 0, 0.08);
  margin: 4px 8px;
}

/* Menu transitions */
.menu-slide-enter-active,
.menu-slide-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.menu-slide-enter-from {
  opacity: 0;
  transform: scale(0.95) translateY(-4px);
}

.menu-slide-leave-to {
  opacity: 0;
  transform: scale(0.95) translateY(-4px);
}

/* Dark mode */
@media (prefers-color-scheme: dark) {
  .contextual-menu {
    background: rgba(32, 32, 32, 0.95);
    border-color: rgba(255, 255, 255, 0.12);
  }

  .menu-item {
    color: #ffffff;
  }

  .menu-item:hover:not(.menu-item--disabled) {
    background: rgba(0, 122, 255, 0.12);
    color: #64B5F6;
  }

  .menu-item--danger {
    color: #FF6B6B;
  }

  .menu-item--danger:hover:not(.menu-item--disabled) {
    background: rgba(255, 107, 107, 0.12);
  }

  .menu-item-shortcut {
    color: rgba(255, 255, 255, 0.5);
  }

  .menu-item-arrow {
    color: rgba(255, 255, 255, 0.4);
  }

  .menu-divider {
    background: rgba(255, 255, 255, 0.12);
  }
}

/* High contrast mode */
@media (prefers-contrast: high) {
  .contextual-menu {
    border-width: 2px;
    border-color: currentColor;
  }

  .menu-item {
    border: 1px solid transparent;
  }

  .menu-item:hover:not(.menu-item--disabled) {
    border-color: currentColor;
  }
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {

  .menu-item,
  .menu-slide-enter-active,
  .menu-slide-leave-active {
    transition: none;
  }
}

/* Focus management */
.menu-item:focus-visible {
  outline: 2px solid #007AFF;
  outline-offset: -2px;
}

/* Touch-friendly sizing */
@media (hover: none) and (pointer: coarse) {
  .menu-item {
    min-height: 44px;
    padding: 12px;
  }
}
</style>