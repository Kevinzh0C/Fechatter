<template>
  <button 
    @click="runTest"
    :disabled="test.status === 'running'"
    class="w-full p-4 text-left border rounded-lg transition-colors"
    :class="buttonClass">
    <div class="flex items-center justify-between">
      <div class="flex items-center">
        <div class="flex-shrink-0 mr-3">
          <div class="w-3 h-3 rounded-full" :class="statusColor"></div>
        </div>
        <div>
          <h4 class="font-medium text-gray-900">{{ test.name }}</h4>
          <p v-if="test.description" class="text-sm text-gray-500">{{ test.description }}</p>
        </div>
      </div>
      <div class="flex items-center">
        <span v-if="test.status === 'running'" class="text-sm text-blue-600">Running...</span>
        <span v-else-if="test.status === 'passed'" class="text-sm text-green-600">Passed</span>
        <span v-else-if="test.status === 'failed'" class="text-sm text-red-600">Failed</span>
        <span v-else class="text-sm text-gray-500">Ready</span>
      </div>
    </div>
  </button>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  test: {
    type: Object,
    required: true
  }
});

const emit = defineEmits(['run']);

const buttonClass = computed(() => {
  switch (props.test.status) {
    case 'running':
      return 'border-blue-300 bg-blue-50 hover:bg-blue-100';
    case 'passed':
      return 'border-green-300 bg-green-50 hover:bg-green-100';
    case 'failed':
      return 'border-red-300 bg-red-50 hover:bg-red-100';
    default:
      return 'border-gray-300 bg-white hover:bg-gray-50';
  }
});

const statusColor = computed(() => {
  switch (props.test.status) {
    case 'running':
      return 'bg-blue-500 animate-pulse';
    case 'passed':
      return 'bg-green-500';
    case 'failed':
      return 'bg-red-500';
    default:
      return 'bg-gray-300';
  }
});

function runTest() {
  if (props.test.status !== 'running') {
    emit('run', props.test);
  }
}
</script> 