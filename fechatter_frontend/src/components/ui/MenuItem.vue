<template>
  <button 
    class="menu-item"
    :class="{
      'danger': danger,
      'disabled': disabled
    }"
    :disabled="disabled"
    @click="handleClick"
  >
    <Icon v-if="icon" :name="icon" class="menu-item-icon" />
    <span class="menu-item-text">
      <slot>{{ label }}</slot>
    </span>
    <span v-if="shortcut" class="menu-item-shortcut">{{ shortcut }}</span>
  </button>
</template>

<script setup>
import Icon from './Icon.vue'

// Props
const props = defineProps({
  label: {
    type: String,
    default: ''
  },
  icon: {
    type: String,
    default: null
  },
  shortcut: {
    type: String,
    default: null
  },
  danger: {
    type: Boolean,
    default: false
  },
  disabled: {
    type: Boolean,
    default: false
  }
})

// Emits
const emit = defineEmits(['click'])

// Methods
const handleClick = () => {
  if (!props.disabled) {
    emit('click')
  }
}
</script>

<style scoped>
.menu-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border: none;
  background: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s ease;
  font-size: 14px;
  text-align: left;
  color: var(--text-primary, #dcddde);
}

.menu-item:hover:not(.disabled) {
  background-color: var(--bg-tertiary, #40444b);
}

.menu-item:active:not(.disabled) {
  background-color: var(--bg-quaternary, #36393f);
}

.menu-item.danger {
  color: var(--danger-color, #ed4245);
}

.menu-item.danger:hover:not(.disabled) {
  background-color: var(--danger-bg-hover, rgba(237, 66, 69, 0.1));
}

.menu-item.disabled {
  color: var(--text-muted, #72767d);
  cursor: not-allowed;
  opacity: 0.5;
}

.menu-item-icon {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.menu-item-text {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.menu-item-shortcut {
  font-size: 12px;
  color: var(--text-muted, #72767d);
  font-family: monospace;
}
</style> 