<!--
  ActionButton.vue
  Jobs-Inspired Premium Action Button
  
  Philosophy:
  - Perfect: Every pixel matters, premium materials
  - Humanize: Natural touch feedback, breathing animations
  - Simplify: One clear purpose per button
  - Focus: Visual hierarchy through state and variant
-->
<template>
  <div class="action-button-container">
    <!-- Primary Button -->
    <button ref="buttonRef" :class="buttonClasses" @click="handleClick" @mouseenter="handleMouseEnter"
      @mouseleave="handleMouseLeave" @touchstart="handleTouchStart" @touchend="handleTouchEnd" :aria-label="ariaLabel"
      :title="tooltip" :disabled="disabled">
      <!-- Icon Container -->
      <div class="icon-container">
        <Icon :name="icon" :class="iconClasses" :size="iconSize" />

        <!-- Badge - Smart positioning -->
        <div v-if="badge && badge > 0" :class="badgeClasses" :data-count="badge > 99 ? '99+' : badge">
          {{ badge > 99 ? '99+' : badge }}
        </div>
      </div>

      <!-- Label - Context-aware visibility -->
      <span v-if="showLabel" :class="labelClasses">
        {{ label }}
      </span>

      <!-- Touch Ripple Effect - Jobs loved tactile feedback -->
      <div v-if="showRipple" ref="rippleRef" :class="rippleClasses" :style="rippleStyle"></div>
    </button>

    <!-- Tooltip - Premium positioning -->
    <div v-if="showTooltip && tooltip" ref="tooltipRef" :class="tooltipClasses">
      {{ tooltip }}
      <div class="tooltip-arrow"></div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue';
import Icon from '@/components/ui/Icon.vue';

// Props - Clean, focused API
const props = defineProps({
  icon: {
    type: String,
    required: true
  },
  label: {
    type: String,
    default: ''
  },
  tooltip: {
    type: String,
    default: ''
  },
  variant: {
    type: String,
    default: 'default',
    validator: (value) => ['default', 'search', 'members', 'settings', 'primary', 'danger'].includes(value)
  },
  active: {
    type: Boolean,
    default: false
  },
  disabled: {
    type: Boolean,
    default: false
  },
  badge: {
    type: Number,
    default: 0
  },
  size: {
    type: String,
    default: 'medium',
    validator: (value) => ['small', 'medium', 'large'].includes(value)
  },
  showLabel: {
    type: Boolean,
    default: false
  }
});

// Emits
const emit = defineEmits(['click']);

// Reactive state
const buttonRef = ref(null);
const tooltipRef = ref(null);
const rippleRef = ref(null);
const isHovered = ref(false);
const isPressed = ref(false);
const showTooltip = ref(false);
const showRipple = ref(false);
const rippleStyle = ref({});

// Computed classes - Jobs believed in systematic design
const buttonClasses = computed(() => [
  'action-button',
  `action-button--${props.variant}`,
  `action-button--${props.size}`,
  {
    'action-button--active': props.active,
    'action-button--disabled': props.disabled,
    'action-button--hovered': isHovered.value,
    'action-button--pressed': isPressed.value,
    'action-button--with-label': props.showLabel && props.label,
    'action-button--with-badge': props.badge > 0
  }
]);

const iconClasses = computed(() => [
  'action-icon',
  {
    'action-icon--active': props.active,
    'action-icon--pressed': isPressed.value
  }
]);

const labelClasses = computed(() => [
  'action-label',
  {
    'action-label--active': props.active
  }
]);

const badgeClasses = computed(() => [
  'action-badge',
  `action-badge--${props.variant}`,
  {
    'action-badge--large': props.badge > 9
  }
]);

const tooltipClasses = computed(() => [
  'action-tooltip',
  {
    'action-tooltip--visible': showTooltip.value
  }
]);

const rippleClasses = computed(() => [
  'action-ripple',
  {
    'action-ripple--active': showRipple.value
  }
]);

const iconSize = computed(() => {
  switch (props.size) {
    case 'small': return 16;
    case 'large': return 24;
    default: return 20;
  }
});

const ariaLabel = computed(() => {
  let label = props.label || props.tooltip || `${props.icon} button`;
  if (props.badge > 0) {
    label += ` (${props.badge} ${props.badge === 1 ? 'notification' : 'notifications'})`;
  }
  return label;
});

// Event handlers - Jobs emphasized immediate feedback
const handleClick = (event) => {
  if (props.disabled) return;

  createRippleEffect(event);
  emit('click', event);
};

const handleMouseEnter = () => {
  if (props.disabled) return;

  isHovered.value = true;
  showTooltipDelayed();
};

const handleMouseLeave = () => {
  isHovered.value = false;
  hideTooltip();
};

const handleTouchStart = () => {
  if (props.disabled) return;

  isPressed.value = true;
  hideTooltip(); // Hide tooltip on touch
};

const handleTouchEnd = () => {
  isPressed.value = false;
};

// Premium interactions - Jobs loved delightful details
const createRippleEffect = (event) => {
  if (!buttonRef.value) return;

  const button = buttonRef.value;
  const rect = button.getBoundingClientRect();
  const size = Math.max(rect.width, rect.height);
  const x = event.clientX - rect.left - size / 2;
  const y = event.clientY - rect.top - size / 2;

  rippleStyle.value = {
    width: `${size}px`,
    height: `${size}px`,
    left: `${x}px`,
    top: `${y}px`,
  };

  showRipple.value = true;

  setTimeout(() => {
    showRipple.value = false;
  }, 600);
};

const showTooltipDelayed = () => {
  if (!props.tooltip || props.disabled) return;

  setTimeout(() => {
    if (isHovered.value) {
      showTooltip.value = true;
    }
  }, 500); // Jobs believed in purposeful delays
};

const hideTooltip = () => {
  showTooltip.value = false;
};

// Cleanup
let tooltipHideTimeout;

onUnmounted(() => {
  if (tooltipHideTimeout) {
    clearTimeout(tooltipHideTimeout);
  }
});
</script>

<style scoped>
/*
  Jobs-Inspired Design System
  - Premium materials (glass, subtle shadows)
  - Perfect spacing and proportions
  - Smooth, purposeful animations
  - Accessible color contrast
*/

.action-button-container {
  position: relative;
  display: inline-flex;
}

.action-button {
  /* Base structure */
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(20px);
  cursor: pointer;
  overflow: hidden;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);

  /* Typography */
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
  font-weight: 500;
  text-decoration: none;
  user-select: none;

  /* Accessibility */
  outline: none;
  -webkit-tap-highlight-color: transparent;
}

/* Size variants - Perfect proportions */
.action-button--small {
  padding: 6px;
  gap: 4px;
  font-size: 12px;
  min-height: 32px;
}

.action-button--medium {
  padding: 8px;
  gap: 6px;
  font-size: 14px;
  min-height: 36px;
}

.action-button--large {
  padding: 12px;
  gap: 8px;
  font-size: 16px;
  min-height: 44px;
}

.action-button--with-label {
  padding-left: 12px;
  padding-right: 16px;
}

/* Color variants - Semantic and beautiful */
.action-button--default {
  color: rgba(255, 255, 255, 0.8);
}

.action-button--search {
  color: #64B5F6;
}

.action-button--members {
  color: #81C784;
}

.action-button--settings {
  color: rgba(255, 255, 255, 0.7);
}

.action-button--primary {
  background: linear-gradient(135deg, #007AFF, #5856D6);
  color: white;
}

.action-button--danger {
  color: #FF6B6B;
}

/* Interactive states - Jobs loved immediate feedback */
.action-button:hover:not(.action-button--disabled) {
  background: rgba(255, 255, 255, 0.12);
  transform: translateY(-1px);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
}

.action-button--active {
  background: rgba(255, 255, 255, 0.16);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
}

.action-button--pressed {
  transform: translateY(0px) scale(0.96);
}

.action-button--disabled {
  opacity: 0.4;
  cursor: not-allowed;
  transform: none !important;
}

/* Icon styling */
.icon-container {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.action-icon {
  transition: transform 0.2s ease;
}

.action-icon--active {
  transform: scale(1.1);
}

.action-icon--pressed {
  transform: scale(0.9);
}

/* Label styling */
.action-label {
  color: inherit;
  font-weight: 500;
  transition: opacity 0.2s ease;
  white-space: nowrap;
}

/* Badge - Smart notification system */
.action-badge {
  position: absolute;
  top: -6px;
  right: -6px;
  min-width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 9px;
  font-size: 11px;
  font-weight: 600;
  line-height: 1;
  border: 2px solid rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(20px);
}

.action-badge--default,
.action-badge--settings {
  background: linear-gradient(135deg, #FF6B6B, #FF8E85);
  color: white;
}

.action-badge--search {
  background: linear-gradient(135deg, #64B5F6, #90CAF9);
  color: white;
}

.action-badge--members {
  background: linear-gradient(135deg, #81C784, #A5D6A7);
  color: white;
}

.action-badge--large {
  min-width: 22px;
  height: 22px;
  font-size: 10px;
  top: -8px;
  right: -8px;
}

/* Ripple effect - Jobs loved tactile feedback */
.action-ripple {
  position: absolute;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.2);
  pointer-events: none;
  transform: scale(0);
  animation: ripple 0.6s linear;
}

@keyframes ripple {
  to {
    transform: scale(2);
    opacity: 0;
  }
}

/* Tooltip - Premium positioning */
.action-tooltip {
  position: absolute;
  bottom: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.9);
  backdrop-filter: blur(20px);
  color: white;
  padding: 6px 10px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
  opacity: 0;
  visibility: hidden;
  transition: all 0.2s ease;
  z-index: 1000;
  pointer-events: none;
}

.action-tooltip--visible {
  opacity: 1;
  visibility: visible;
}

.tooltip-arrow {
  position: absolute;
  top: 100%;
  left: 50%;
  transform: translateX(-50%);
  width: 0;
  height: 0;
  border-left: 4px solid transparent;
  border-right: 4px solid transparent;
  border-top: 4px solid rgba(0, 0, 0, 0.9);
}

/* Dark mode - Seamless adaptation */
@media (prefers-color-scheme: dark) {
  .action-button {
    background: rgba(255, 255, 255, 0.06);
  }

  .action-button:hover:not(.action-button--disabled) {
    background: rgba(255, 255, 255, 0.1);
  }

  .action-button--active {
    background: rgba(255, 255, 255, 0.14);
  }
}

/* Accessibility */
.action-button:focus-visible {
  outline: 2px solid #007AFF;
  outline-offset: 2px;
}

/* High contrast mode */
@media (prefers-contrast: high) {
  .action-button {
    border: 1px solid currentColor;
  }
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {

  .action-button,
  .action-icon,
  .action-tooltip,
  .action-ripple {
    transition: none;
    animation: none;
  }
}
</style>