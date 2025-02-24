<template>
  <div class="message-skeletons">
    <div v-for="i in count" :key="i" class="message-skeleton" :class="{ animated }">
      <div class="skeleton-avatar"></div>
      <div class="skeleton-content">
        <div class="skeleton-line skeleton-name"></div>
        <div class="skeleton-line skeleton-text"></div>
        <div class="skeleton-line skeleton-short" v-if="variant === 'normal'"></div>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  count: {
    type: Number,
    default: 3
  },
  variant: {
    type: String,
    default: 'normal' // 'normal' | 'compact'
  },
  animated: {
    type: Boolean,
    default: true
  }
});
</script>

<style scoped>
.message-skeletons {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.message-skeleton {
  display: flex;
  gap: 0.75rem;
  padding: 0.5rem 0;
}

.skeleton-avatar {
  width: 32px;
  height: 32px;
  border-radius: 0.375rem;
  background: linear-gradient(90deg, #f5f4f2 25%, #ebe9e6 50%, #f5f4f2 75%);
  background-size: 200% 100%;
  flex-shrink: 0;
}

.skeleton-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  min-width: 0;
}

.skeleton-line {
  height: 12px;
  border-radius: 6px;
  background: linear-gradient(90deg, #f5f4f2 25%, #ebe9e6 50%, #f5f4f2 75%);
  background-size: 200% 100%;
}

.skeleton-name {
  width: 30%;
  max-width: 120px;
}

.skeleton-text {
  width: 80%;
}

.skeleton-short {
  width: 60%;
}

.message-skeleton.animated .skeleton-avatar,
.message-skeleton.animated .skeleton-line {
  animation: shimmer 2s infinite linear;
}

@keyframes shimmer {
  0% {
    background-position: -200% 0;
  }

  100% {
    background-position: 200% 0;
  }
}
</style>