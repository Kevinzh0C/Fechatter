<template>
  <div class="message-skeleton-container" role="status" aria-label="Loading messages">
    <!-- M4: 固定高度的骨架屏，避免布局跳动 -->
    <div 
      v-for="i in count" 
      :key="`skeleton-${i}`" 
      class="message-skeleton"
      :style="{ 
        animationDelay: `${i * 50}ms`,
        height: `${getSkeletonHeight(i)}px`
      }"
    >
      <!-- Message bubble -->
      <div class="skeleton-message" :class="{ 'own-message': i % 3 === 0 }">
        <div class="skeleton-avatar"></div>
        <div class="skeleton-content">
          <div class="skeleton-header">
            <div class="skeleton-name" :style="{ width: `${getNameWidth(i)}px` }"></div>
            <div class="skeleton-time"></div>
          </div>
          <div class="skeleton-text">
            <div 
              v-for="(width, index) in getLineWidths(i)" 
              :key="`line-${i}-${index}`"
              class="skeleton-line" 
              :style="{ width: `${width}%` }"
            ></div>
          </div>
          <!-- 附件占位符 -->
          <div v-if="hasAttachment(i)" class="skeleton-attachment">
            <div class="skeleton-attachment-icon"></div>
            <div class="skeleton-attachment-text"></div>
          </div>
        </div>
      </div>
    </div>
    <span class="sr-only">Loading messages...</span>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';

const props = defineProps({
  count: {
    type: Number,
    default: 5
  },
  minHeight: {
    type: Number,
    default: 60
  },
  maxHeight: {
    type: Number,
    default: 120
  },
  animated: {
    type: Boolean,
    default: true
  }
});

// M4: 预定义高度，避免随机性导致的布局跳动
const heightPatterns = ref([80, 95, 70, 110, 85]);
const nameWidthPatterns = ref([90, 70, 110, 80, 95]);
const linePatterns = ref([
  [95, 80],
  [100],
  [90, 85, 60],
  [95, 70],
  [85]
]);

// 获取固定的骨架高度
const getSkeletonHeight = (index) => {
  const pattern = heightPatterns.value[index % heightPatterns.value.length];
  return Math.max(props.minHeight, Math.min(props.maxHeight, pattern));
};

// 获取固定的名称宽度
const getNameWidth = (index) => {
  return nameWidthPatterns.value[index % nameWidthPatterns.value.length];
};

// 获取固定的行宽度
const getLineWidths = (index) => {
  return linePatterns.value[index % linePatterns.value.length];
};

// 是否显示附件
const hasAttachment = (index) => {
  return index % 4 === 0; // 每4个消息有一个带附件
};

// 生成更多样化的模式
onMounted(() => {
  // 可以根据实际消息统计生成更准确的模式
  const generatePatterns = () => {
    // 高度模式（基于实际消息统计）
    heightPatterns.value = [
      75, 85, 65, 95, 80, // 短消息
      110, 95, 120, 100, 90, // 中等消息
      85, 75, 90, 80, 70 // 混合
    ];
    
    // 名称宽度（基于用户名长度）
    nameWidthPatterns.value = [
      80, 95, 70, 110, 85,
      90, 75, 100, 80, 95
    ];
    
    // 文本行模式
    linePatterns.value = [
      [95, 80], // 两行
      [100], // 单行
      [90, 85, 60], // 三行
      [95, 70], // 两行
      [85], // 单行
      [100, 95, 80], // 三行
      [90, 75], // 两行
      [95], // 单行
      [85, 80, 70, 50], // 四行
      [90, 85] // 两行
    ];
  };
  
  generatePatterns();
});
</script>

<style scoped>
.message-skeleton-container {
  padding: 16px;
  space-y: 12px;
}

.message-skeleton {
  animation: fadeInUp 0.6s ease-out both;
}

.skeleton-message {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 16px;
  padding: 12px;
  border-radius: 8px;
}

.skeleton-message.own-message {
  flex-direction: row-reverse;
  justify-content: flex-start;
}

.skeleton-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
  background-size: 200% 100%;
  animation: shimmer 2s infinite;
  flex-shrink: 0;
}

.skeleton-content {
  flex: 1;
  max-width: 70%;
}

.skeleton-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.skeleton-name {
  height: 14px;
  background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
  background-size: 200% 100%;
  animation: shimmer 2s infinite;
  border-radius: 4px;
}

.skeleton-time {
  width: 40px;
  height: 12px;
  background: linear-gradient(90deg, #f8f8f8 25%, #f0f0f0 50%, #f8f8f8 75%);
  background-size: 200% 100%;
  animation: shimmer 2s infinite;
  border-radius: 4px;
}

.skeleton-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.skeleton-line {
  height: 16px;
  background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
  background-size: 200% 100%;
  animation: shimmer 2s infinite;
  border-radius: 4px;
}

/* M4: 附件骨架样式 */
.skeleton-attachment {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
  padding: 8px;
  background: rgba(124, 58, 237, 0.03);
  border-radius: 6px;
  border: 1px solid rgba(124, 58, 237, 0.08);
}

.skeleton-attachment-icon {
  width: 24px;
  height: 24px;
  background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
  background-size: 200% 100%;
  animation: shimmer 2s infinite;
  border-radius: 4px;
  flex-shrink: 0;
}

.skeleton-attachment-text {
  width: 120px;
  height: 14px;
  background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
  background-size: 200% 100%;
  animation: shimmer 2s infinite;
  border-radius: 4px;
}

.own-message .skeleton-content {
  text-align: right;
}

.own-message .skeleton-header {
  justify-content: flex-end;
}

@keyframes shimmer {
  0% {
    background-position: -200% 0;
  }
  100% {
    background-position: 200% 0;
  }
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
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