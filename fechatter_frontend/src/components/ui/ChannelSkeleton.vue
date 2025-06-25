<template>
  <div class="channel-skeleton" role="status" aria-label="Loading channels">
    <div 
      v-for="i in count" 
      :key="i" 
      class="skeleton-item"
      :style="{ animationDelay: `${i * 100}ms` }"
    >
      <div class="skeleton-prefix"></div>
      <div class="skeleton-text">
        <div class="skeleton-line" :style="{ width: `${60 + Math.random() * 30}%` }"></div>
      </div>
    </div>
    <span class="sr-only">Loading channels...</span>
  </div>
</template>

<script setup>
defineProps({
  count: {
    type: Number,
    default: 3
  }
});
</script>

<style scoped>
.channel-skeleton {
  padding: 0 12px;
}

.skeleton-item {
  display: flex;
  align-items: center;
  padding: 6px 12px;
  margin-bottom: 2px;
  border-radius: 6px;
  animation: pulse 1.5s ease-in-out infinite;
}

.skeleton-prefix {
  width: 20px;
  height: 20px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  margin-right: 8px;
  flex-shrink: 0;
}

.skeleton-text {
  flex: 1;
}

.skeleton-line {
  height: 12px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 6px;
}

@keyframes pulse {
  0%, 100% {
    opacity: 0.4;
  }
  50% {
    opacity: 0.8;
  }
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border-width: 0;
}
</style>