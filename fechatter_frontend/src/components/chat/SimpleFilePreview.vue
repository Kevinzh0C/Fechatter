<template>
  <div class="simple-file-preview">
    <!-- 文件列表 -->
    <div v-if="hasFiles" class="files-container">
      <div class="files-header">
        <span>{{ fileCount }} file{{ fileCount > 1 ? 's' : '' }}</span>
        <button @click="clearAll" class="clear-btn">Clear All</button>
      </div>

      <div class="files-list">
        <div v-for="file in files" :key="file.id" class="file-item">
          <!-- 图片缩略图 -->
          <div v-if="isImage(file)" class="image-thumb">
            <img v-if="file.preview" :src="file.preview" :alt="file.name" />
            <div v-else class="loading">📷</div>
          </div>

          <!-- 其他文件图标 -->
          <div v-else class="file-icon">📄</div>

          <!-- 文件信息 -->
          <div class="file-info">
            <div class="file-name">{{ file.name }}</div>
          </div>

          <!-- 删除按钮 -->
          <button @click="removeFile(file.id)" class="remove-btn">×</button>
        </div>
      </div>
    </div>

    <!-- 拖拽上传区域 -->
    <div v-if="isDragOver" class="drag-overlay">
      <div class="drag-content">
        <div class="drag-icon">📁</div>
        <div>Drop files here to upload</div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, watch, ref, onMounted, onUnmounted } from 'vue';
import { useFileUploadStore } from '@/stores/fileUploadStore';

const store = useFileUploadStore();
const isDragOver = ref(false);

const hasFiles = computed(() => store.pendingFiles?.length > 0);
const fileCount = computed(() => store.pendingFiles?.length || 0);
const files = computed(() => store.pendingFiles || []);

const isImage = (file) => file?.type?.startsWith('image/');

const removeFile = (id) => store.removeFile?.(id);
const clearAll = () => store.clearAll?.();

// 生成图片预览
watch(files, (newFiles) => {
  newFiles?.forEach(file => {
    if (isImage(file) && !file.preview && !file.loading) {
      file.loading = true;
      const reader = new FileReader();
      reader.onload = e => {
        file.preview = e.target.result;
        file.loading = false;
      };
      reader.onerror = () => {
        file.loading = false;
      };
      reader.readAsDataURL(file);
    }
  });
}, { deep: true, immediate: true });

// 拖拽上传功能
onMounted(() => {
  const handleDragEnter = (e) => {
    e.preventDefault();
    isDragOver.value = true;
  };

  const handleDragLeave = (e) => {
    if (e.relatedTarget === null || e.relatedTarget === document.documentElement) {
      isDragOver.value = false;
    }
  };

  const handleDragOver = (e) => {
    e.preventDefault();
  };

  const handleDrop = (e) => {
    e.preventDefault();
    isDragOver.value = false;

    const files = Array.from(e.dataTransfer.files);
    if (files.length > 0 && store.addFiles) {
      store.addFiles(files);
    }
  };

  // 添加全局拖拽监听
  document.addEventListener('dragenter', handleDragEnter);
  document.addEventListener('dragleave', handleDragLeave);
  document.addEventListener('dragover', handleDragOver);
  document.addEventListener('drop', handleDrop);

  // 清理函数
  onUnmounted(() => {
    document.removeEventListener('dragenter', handleDragEnter);
    document.removeEventListener('dragleave', handleDragLeave);
    document.removeEventListener('dragover', handleDragOver);
    document.removeEventListener('drop', handleDrop);
  });
});
</script>

<style scoped>
.simple-file-preview {
  margin-bottom: 8px;
}

.files-container {
  background: #f8f9fa;
  border-radius: 6px;
  padding: 12px;
  border: 1px solid #e9ecef;
}

.files-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  font-size: 14px;
}

.clear-btn {
  background: #dc3545;
  color: white;
  border: none;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
}

.files-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 8px;
}

.file-item {
  position: relative;
  background: white;
  border-radius: 4px;
  padding: 8px;
  text-align: center;
  border: 1px solid #dee2e6;
}

.image-thumb {
  width: 100%;
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 4px;
}

.image-thumb img {
  max-width: 100%;
  max-height: 100%;
  object-fit: cover;
  border-radius: 3px;
}

.file-icon {
  font-size: 32px;
  margin-bottom: 4px;
}

.file-name {
  font-size: 11px;
  word-break: break-word;
}

.remove-btn {
  position: absolute;
  top: 4px;
  right: 4px;
  background: rgba(220, 53, 69, 0.8);
  color: white;
  border: none;
  border-radius: 50%;
  width: 20px;
  height: 20px;
  cursor: pointer;
  font-size: 14px;
}

.drag-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
}

.drag-content {
  background: white;
  padding: 20px;
  border-radius: 8px;
  text-align: center;
}

.drag-icon {
  font-size: 48px;
  margin-bottom: 12px;
}
</style>