<template>
  <div class="notification-container">
    <!-- 🔔 Global notifications will be displayed here -->
  </div>
</template>

<script setup>
// Simple notification container placeholder
</script>

<style scoped>
.notification-container {
  position: fixed;
  top: 1rem;
  left: 50%;
  transform: translateX(-50%);
  z-index: 9999;
  pointer-events: none;
}
</style>