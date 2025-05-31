<template>
  <div :class="containerClass">
    <div class="flex flex-col items-center justify-center space-y-4">
      <!-- Spinner -->
      <div :class="spinnerClass">
        <svg class="animate-spin h-full w-full" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
      </div>
      
      <!-- Loading Text -->
      <div v-if="text" :class="textClass">
        {{ text }}
      </div>
      
      <!-- Progress Bar (optional) -->
      <div v-if="showProgress && progress !== null" class="w-full max-w-xs">
        <div class="bg-gray-200 rounded-full h-2">
          <div 
            class="bg-purple-600 h-2 rounded-full transition-all duration-300 ease-out"
            :style="{ width: `${Math.min(100, Math.max(0, progress))}%` }">
          </div>
        </div>
        <div class="text-xs text-gray-500 mt-1 text-center">
          {{ Math.round(progress) }}%
        </div>
      </div>
      
      <!-- Additional Content Slot -->
      <slot />
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  size: {
    type: String,
    default: 'medium',
    validator: (value) => ['small', 'medium', 'large', 'xlarge'].includes(value)
  },
  text: {
    type: String,
    default: ''
  },
  overlay: {
    type: Boolean,
    default: false
  },
  fullscreen: {
    type: Boolean,
    default: false
  },
  color: {
    type: String,
    default: 'purple',
    validator: (value) => ['purple', 'blue', 'green', 'gray', 'white'].includes(value)
  },
  showProgress: {
    type: Boolean,
    default: false
  },
  progress: {
    type: Number,
    default: null
  }
});

const containerClass = computed(() => {
  const classes = [];
  
  if (props.fullscreen) {
    classes.push('fixed inset-0 z-50 flex items-center justify-center');
    if (props.overlay) {
      classes.push('bg-black bg-opacity-50');
    } else {
      classes.push('bg-white');
    }
  } else if (props.overlay) {
    classes.push('absolute inset-0 z-10 flex items-center justify-center bg-white bg-opacity-90');
  } else {
    classes.push('flex items-center justify-center py-8');
  }
  
  return classes.join(' ');
});

const spinnerClass = computed(() => {
  const sizeClasses = {
    small: 'h-6 w-6',
    medium: 'h-8 w-8',
    large: 'h-12 w-12',
    xlarge: 'h-16 w-16'
  };
  
  const colorClasses = {
    purple: 'text-purple-600',
    blue: 'text-blue-600',
    green: 'text-green-600',
    gray: 'text-gray-600',
    white: 'text-white'
  };
  
  return `${sizeClasses[props.size]} ${colorClasses[props.color]}`;
});

const textClass = computed(() => {
  const sizeClasses = {
    small: 'text-sm',
    medium: 'text-base',
    large: 'text-lg',
    xlarge: 'text-xl'
  };
  
  const colorClasses = {
    purple: 'text-purple-600',
    blue: 'text-blue-600',
    green: 'text-green-600',
    gray: 'text-gray-600',
    white: 'text-white'
  };
  
  return `${sizeClasses[props.size]} ${colorClasses[props.color]} font-medium`;
});
</script>

<style scoped>
/* Additional animations for enhanced loading experience */
@keyframes pulse-slow {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.animate-pulse-slow {
  animation: pulse-slow 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
</style> 