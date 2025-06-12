<template>
  <svg :width="size" :height="size" :viewBox="viewBox" xmlns="http://www.w3.org/2000/svg" :class="['icon', className]"
    :style="computedStyle" fill="currentColor" aria-hidden="true" role="img">
    <!-- Hash Icon -->
    <path v-if="name === 'hash'"
      d="M5.41 21L6.12 17H2.12L2.47 15H6.47L7.53 9H3.53L3.88 7H7.88L8.59 3H10.59L9.88 7H15.88L16.59 3H18.59L17.88 7H21.88L21.53 9H17.53L16.47 15H20.47L20.12 17H16.12L15.41 21H13.41L14.12 17H8.12L7.41 21H5.41ZM10.12 15H16.12L17.18 9H11.18L10.12 15Z" />

    <!-- Lock Icon -->
    <path v-if="name === 'lock'"
      d="M18,8A2,2 0 0,1 20,10V20A2,2 0 0,1 18,22H6A2,2 0 0,1 4,20V10A2,2 0 0,1 6,8H7V6A5,5 0 0,1 12,1A5,5 0 0,1 17,6V8H18M12,3A3,3 0 0,0 9,6V8H15V6A3,3 0 0,0 12,3Z" />

    <!-- Megaphone Icon -->
    <path v-if="name === 'megaphone'"
      d="M12,8H4A2,2 0 0,0 2,10V14A2,2 0 0,0 4,16H5V20A1,1 0 0,0 6,21H8A1,1 0 0,0 9,20V16H12L17,20V4L12,8M21.5,12C21.5,13.71 20.54,15.26 19,16V8C20.53,8.75 21.5,10.3 21.5,12Z" />

    <!-- Chevron Down Icon -->
    <path v-if="name === 'chevron-down'" d="M7.41,8.58L12,13.17L16.59,8.58L18,10L12,16L6,10L7.41,8.58Z" />

    <!-- Refresh Icon -->
    <path v-if="name === 'refresh'"
      d="M17.65,6.35C16.2,4.9 14.21,4 12,4C7.58,4 4.01,7.58 4.01,12C4.01,16.42 7.58,20 12,20C15.73,20 18.84,17.45 19.73,14H17.65C16.83,16.33 14.61,18 12,18C8.69,18 6,15.31 6,12C6,8.69 8.69,6 12,6C13.66,6 15.14,6.69 16.22,7.78L13,11H20V4L17.65,6.35Z" />

    <!-- Plus Icon -->
    <path v-if="name === 'plus'" d="M19,13H13V19H11V13H5V11H11V5H13V11H19V13Z" />

    <!-- User Icon -->
    <path v-if="name === 'user'"
      d="M12,4A4,4 0 0,1 16,8A4,4 0 0,1 12,12A4,4 0 0,1 8,8A4,4 0 0,1 12,4M12,14C16.42,14 20,15.79 20,18V20H4V18C4,15.79 7.58,14 12,14Z" />

    <!-- Bell Off Icon -->
    <path v-if="name === 'bell-off'"
      d="M20,18.69L7.84,6.14L5.27,3.49L4,4.76L6.8,7.56V8A6,6 0 0,0 12.8,14A1.7,1.7 0 0,0 14.5,15.7V16.5L17.5,19.5C17.55,19.66 17.6,19.82 17.6,20A1.9,1.9 0 0,1 15.7,21.9H8.3A1.9,1.9 0 0,1 6.4,20A1.9,1.9 0 0,1 8.3,18.1H14.7L20,23.31L21.27,22L20,18.69M12,2A6,6 0 0,1 18,8V15.17L16,13.17V8A4,4 0 0,0 12,4A4,4 0 0,0 8,8V9.17L6.83,8H6.8V8A6,6 0 0,1 12,2Z" />

    <!-- Arrow Right Icon -->
    <path v-if="name === 'arrow-right'" d="M4,11V13H16L10.5,18.5L11.92,19.92L19.84,12L11.92,4.08L10.5,5.5L16,11H4Z" />

    <!-- Edit Icon -->
    <path v-if="name === 'edit'"
      d="M20.71,7.04C21.1,6.65 21.1,6 20.71,5.63L18.37,3.29C18,2.9 17.35,2.9 16.96,3.29L15.12,5.12L18.87,8.87M3,17.25V21H6.75L17.81,9.93L14.06,6.18L3,17.25Z" />

    <!-- Eye Icon -->
    <path v-if="name === 'eye'"
      d="M12,9A3,3 0 0,0 9,12A3,3 0 0,0 12,15A3,3 0 0,0 15,12A3,3 0 0,0 12,9M12,17A5,5 0 0,1 7,12A5,5 0 0,1 12,7A5,5 0 0,1 17,12A5,5 0 0,1 12,17M12,4.5C7,4.5 2.73,7.61 1,12C2.73,16.39 7,19.5 12,19.5C17,19.5 21.27,16.39 23,12C21.27,7.61 17,4.5 12,4.5Z" />

    <!-- Users Icon -->
    <path v-if="name === 'users'"
      d="M16,4C18.21,4 20,5.79 20,8C20,10.21 18.21,12 16,12C13.79,12 12,10.21 12,8C12,5.79 13.79,4 16,4M16,14C19.31,14 22,15.79 22,18V20H10V18C10,15.79 12.69,14 16,14M8,4C10.21,4 12,5.79 12,8C12,10.21 10.21,12 8,12C5.79,12 4,10.21 4,8C4,5.79 5.79,4 8,4M8,14C11.31,14 14,15.79 14,18V20H2V18C2,15.79 4.69,14 8,14Z" />

    <!-- Log Out Icon -->
    <path v-if="name === 'log-out'"
      d="M16,17V14H9V10H16V7L21,12L16,17M14,2A2,2 0 0,1 16,4V6H14V4H5V20H14V18H16V20A2,2 0 0,1 14,22H5A2,2 0 0,1 3,20V4A2,2 0 0,1 5,2H14Z" />

    <!-- Fallback: Question Mark for unknown icons -->
    <path v-if="!knownIcons.includes(name)"
      d="M11,18H13V16H11V18M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2M12,20C7.59,20 4,16.41 4,12C4,7.59 7.59,4 12,4C16.41,4 20,7.59 20,12C20,16.41 16.41,20 12,20M12,6A4,4 0 0,0 8,10H10A2,2 0 0,1 12,8A2,2 0 0,1 14,10C14,12 11,11.75 11,15H13C13,12.75 16,12.5 16,10A4,4 0 0,0 12,6Z" />
  </svg>
</template>

<script setup>
import { computed } from 'vue'

// Props
const props = defineProps({
  name: {
    type: String,
    required: true
  },
  size: {
    type: [Number, String],
    default: 16
  },
  className: {
    type: String,
    default: ''
  }
})

// Known icon names
const knownIcons = [
  'hash', 'lock', 'megaphone', 'chevron-down', 'refresh', 'plus',
  'user', 'bell-off', 'arrow-right', 'edit', 'eye', 'users', 'log-out'
]

// Computed
const viewBox = computed(() => '0 0 24 24')

const computedStyle = computed(() => ({
  width: typeof props.size === 'number' ? `${props.size}px` : props.size,
  height: typeof props.size === 'number' ? `${props.size}px` : props.size,
  minWidth: typeof props.size === 'number' ? `${props.size}px` : props.size,
  minHeight: typeof props.size === 'number' ? `${props.size}px` : props.size
}))
</script>

<style scoped>
.icon {
  display: inline-block;
  vertical-align: middle;
  flex-shrink: 0;
  transition: all 0.2s ease;
}
</style>