<template>
  <svg class="golden-search-icon" :width="size" :height="size" viewBox="0 0 100 100" fill="none"
    xmlns="http://www.w3.org/2000/svg">
    <!-- 🔍 主搜索圆环 - 黄金分割比例设计 -->
    <circle cx="38.2" cy="38.2" r="23.6" stroke="currentColor" :stroke-width="strokeWidth" fill="none"
      stroke-linecap="round" />

    <!-- 🔍 搜索手柄 - 黄金分割角度 -->
    <line x1="55.9" y1="55.9" x2="76.4" y2="76.4" stroke="currentColor" :stroke-width="strokeWidth"
      stroke-linecap="round" />

    <!-- ✨ 精致内部高光效果 -->
    <circle v-if="showHighlight" cx="32.4" cy="32.4" r="4.8" fill="currentColor" opacity="0.3" />
  </svg>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  size: {
    type: [Number, String],
    default: 20
  },
  strokeWidth: {
    type: [Number, String],
    default: 6.18 // 黄金分割比例相关的线宽
  },
  showHighlight: {
    type: Boolean,
    default: true
  }
})

// 🎨 黄金分割比例计算
// 主圆直径：47.2 (100 * 0.472, 接近黄金分割比例)
// 中心点：38.2 (100 * 0.382, 黄金分割点)
// 半径：23.6 (47.2 / 2)
// 搜索柄起点：55.9 (38.2 + 23.6 * 0.75)
// 搜索柄终点：76.4 (38.2 + 38.2，黄金比例延伸)
// 高光位置：32.4 (38.2 * 0.85)
// 高光半径：4.8 (23.6 * 0.203)
</script>

<style scoped>
.golden-search-icon {
  transition: all 0.2s ease;
  color: inherit;
}

.golden-search-icon:hover {
  transform: scale(1.05);
}
</style>