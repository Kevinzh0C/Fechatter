<template>
  <div class="layout-container">
    <aside class="layout-sidebar">
      <slot name="sidebar" />
    </aside>
    <main class="layout-main">
      <slot />
    </main>
  </div>
</template>

<script>
export default {
  name: 'Layout'
};
</script>

<style scoped>
.layout-container {
  display: flex;
  height: 100vh;
  background: #f8f9fa;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
}

.layout-sidebar {
  width: 260px;
  background: #3f0f40;
  color: white;
  display: flex;
  flex-direction: column;
  position: relative;
}

.layout-main {
  flex: 1;
  overflow: auto;
  position: relative;
}
</style>