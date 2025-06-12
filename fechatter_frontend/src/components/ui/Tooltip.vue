<template>
  <div class="tooltip-container" @mouseenter="showTooltip" @mouseleave="hideTooltip" @focus="showTooltip"
    @blur="hideTooltip">
    <!-- Trigger element -->
    <slot />

    <!-- Tooltip -->
    <Transition name="tooltip">
      <div v-if="isVisible" :class="[
        'tooltip',
        `tooltip-${position}`,
        { 'tooltip-dark': theme === 'dark' }
      ]" role="tooltip" :aria-hidden="!isVisible">
        <div class="tooltip-content">
          {{ text }}
        </div>
        <div class="tooltip-arrow"></div>
      </div>
    </Transition>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'

// Props
const props = defineProps({
  text: {
    type: String,
    required: true
  },
  position: {
    type: String,
    default: 'top',
    validator: (value) => ['top', 'bottom', 'left', 'right'].includes(value)
  },
  theme: {
    type: String,
    default: 'dark',
    validator: (value) => ['dark', 'light'].includes(value)
  },
  delay: {
    type: Number,
    default: 100
  },
  disabled: {
    type: Boolean,
    default: false
  }
})

// State
const isVisible = ref(false)
let showTimer = null
let hideTimer = null

// Methods
const showTooltip = () => {
  if (props.disabled) return

  // Clear any existing hide timer
  if (hideTimer) {
    clearTimeout(hideTimer)
    hideTimer = null
  }

  // Set show timer
  showTimer = setTimeout(() => {
    isVisible.value = true
  }, props.delay)
}

const hideTooltip = () => {
  // Clear any existing show timer
  if (showTimer) {
    clearTimeout(showTimer)
    showTimer = null
  }

  // Set hide timer for smooth UX
  hideTimer = setTimeout(() => {
    isVisible.value = false
  }, 50)
}

// Cleanup
onUnmounted(() => {
  if (showTimer) clearTimeout(showTimer)
  if (hideTimer) clearTimeout(hideTimer)
})
</script>

<style scoped>
/* Tooltip Container */
.tooltip-container {
  position: relative;
  display: inline-block;
}

/* Tooltip Base */
.tooltip {
  position: absolute;
  z-index: 9999;
  pointer-events: none;
  font-size: 12px;
  font-weight: 500;
  line-height: 1.3;
  white-space: nowrap;
  border-radius: 6px;
  padding: 0;
  max-width: 200px;
}

/* Tooltip Content */
.tooltip-content {
  padding: 8px 12px;
  border-radius: 6px;
  backdrop-filter: blur(8px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

/* Tooltip Arrow */
.tooltip-arrow {
  position: absolute;
  width: 8px;
  height: 8px;
  transform: rotate(45deg);
}

/* Dark Theme (Default) */
.tooltip-dark .tooltip-content {
  background: rgba(30, 30, 30, 0.95);
  color: rgba(255, 255, 255, 0.9);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.tooltip-dark .tooltip-arrow {
  background: rgba(30, 30, 30, 0.95);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

/* Light Theme */
.tooltip:not(.tooltip-dark) .tooltip-content {
  background: rgba(255, 255, 255, 0.95);
  color: rgba(30, 30, 30, 0.9);
  border: 1px solid rgba(0, 0, 0, 0.1);
}

.tooltip:not(.tooltip-dark) .tooltip-arrow {
  background: rgba(255, 255, 255, 0.95);
  border: 1px solid rgba(0, 0, 0, 0.1);
}

/* Position: Top */
.tooltip-top {
  bottom: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%);
}

.tooltip-top .tooltip-arrow {
  top: calc(100% - 4px);
  left: 50%;
  transform: translateX(-50%) rotate(45deg);
  border-top: none;
  border-left: none;
}

/* Position: Bottom */
.tooltip-bottom {
  top: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%);
}

.tooltip-bottom .tooltip-arrow {
  bottom: calc(100% - 4px);
  left: 50%;
  transform: translateX(-50%) rotate(45deg);
  border-bottom: none;
  border-right: none;
}

/* Position: Left */
.tooltip-left {
  right: calc(100% + 8px);
  top: 50%;
  transform: translateY(-50%);
}

.tooltip-left .tooltip-arrow {
  left: calc(100% - 4px);
  top: 50%;
  transform: translateY(-50%) rotate(45deg);
  border-left: none;
  border-bottom: none;
}

/* Position: Right */
.tooltip-right {
  left: calc(100% + 8px);
  top: 50%;
  transform: translateY(-50%);
}

.tooltip-right .tooltip-arrow {
  right: calc(100% - 4px);
  top: 50%;
  transform: translateY(-50%) rotate(45deg);
  border-right: none;
  border-top: none;
}

/* Transitions */
.tooltip-enter-active,
.tooltip-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.tooltip-enter-from,
.tooltip-leave-to {
  opacity: 0;
  transform: scale(0.9);
}

.tooltip-enter-to,
.tooltip-leave-from {
  opacity: 1;
  transform: scale(1);
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .tooltip {
    font-size: 13px;
    max-width: 180px;
  }

  .tooltip-content {
    padding: 10px 14px;
  }
}

/* Reduced motion support */
@media (prefers-reduced-motion: reduce) {

  .tooltip-enter-active,
  .tooltip-leave-active {
    transition: none;
  }
}

/* High contrast mode support */
@media (prefers-contrast: high) {
  .tooltip-dark .tooltip-content {
    background: #000;
    border-color: #fff;
  }

  .tooltip-dark .tooltip-arrow {
    background: #000;
    border-color: #fff;
  }

  .tooltip:not(.tooltip-dark) .tooltip-content {
    background: #fff;
    border-color: #000;
  }

  .tooltip:not(.tooltip-dark) .tooltip-arrow {
    background: #fff;
    border-color: #000;
  }
}
</style>