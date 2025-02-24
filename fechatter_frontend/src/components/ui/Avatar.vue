<template>
  <div class="avatar-container" :style="{ width: size + 'px', height: size + 'px' }">
    <!-- Avatar Image -->
    <img v-if="showImage" :src="avatarUrl" :alt="altText" class="avatar-image" @error="handleImageError"
      @load="handleImageLoad" />

    <!-- Fallback Avatar -->
    <div v-else class="avatar-fallback" :style="fallbackStyle">
      <span class="avatar-initials">{{ initials }}</span>
    </div>

    <!-- Status Indicator -->
    <div v-if="showStatus && status" class="status-indicator" :class="status" :title="statusText"></div>
  </div>
</template>

<script setup>
import { computed, ref, watch } from 'vue';
import { createAvatarData } from '@/utils/userAvatarHelper.js';

const props = defineProps({
  user: {
    type: Object,
    default: () => ({})
  },
  avatarUrl: {
    type: String,
    default: null
  },
  size: {
    type: Number,
    default: 40
  },
  showStatus: {
    type: Boolean,
    default: false
  },
  status: {
    type: String,
    default: null // 'online', 'away', 'busy', 'offline'
  }
});

// State
const showImage = ref(true);
const imageLoaded = ref(false);

// Computed
const avatarData = computed(() => createAvatarData(props.user));

const avatarUrl = computed(() => {
  return props.avatarUrl || props.user?.avatar_url || '/default-avatar.svg';
});

const altText = computed(() => {
  return props.user?.fullname || props.user?.name || 'User Avatar';
});

const initials = computed(() => avatarData.value.initials);

const fallbackStyle = computed(() => ({
  background: avatarData.value.gradient,
  color: avatarData.value.textColor,
  fontSize: Math.floor(props.size * 0.4) + 'px'
}));

const statusText = computed(() => {
  const statusMap = {
    online: 'Online',
    away: 'Away',
    busy: 'Busy',
    offline: 'Offline'
  };
  return statusMap[props.status] || '';
});

// Methods
const handleImageError = () => {
  showImage.value = false;
};

const handleImageLoad = () => {
  imageLoaded.value = true;
};

// Reset when avatar URL changes
watch(() => props.avatarUrl, () => {
  showImage.value = true;
  imageLoaded.value = false;
});
</script>

<style scoped>
.avatar-container {
  position: relative;
  display: inline-block;
  flex-shrink: 0;
}

.avatar-image {
  width: 100%;
  height: 100%;
  border-radius: 50%;
  object-fit: cover;
  background: rgba(255, 255, 255, 0.1);
  border: 2px solid rgba(255, 255, 255, 0.1);
  transition: all 0.15s ease;
}

.avatar-fallback {
  width: 100%;
  height: 100%;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  border: 2px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  transition: all 0.15s ease;
}

.avatar-initials {
  user-select: none;
  text-transform: uppercase;
}

.status-indicator {
  position: absolute;
  bottom: -1px;
  right: -1px;
  width: 25%;
  height: 25%;
  min-width: 8px;
  min-height: 8px;
  border-radius: 50%;
  border: 2px solid var(--bg-color, #fff);
  transition: all 0.2s ease;
}

.status-indicator.online {
  background: #2eb67d;
  box-shadow: 0 0 6px rgba(46, 182, 125, 0.4);
}

.status-indicator.away {
  background: #ecb22e;
  box-shadow: 0 0 6px rgba(236, 178, 46, 0.4);
}

.status-indicator.busy {
  background: #e01e5a;
  box-shadow: 0 0 6px rgba(224, 30, 90, 0.4);
}

.status-indicator.offline {
  background: rgba(255, 255, 255, 0.3);
}

/* Hover effects */
.avatar-container:hover .avatar-image,
.avatar-container:hover .avatar-fallback {
  transform: scale(1.05);
  border-color: rgba(255, 255, 255, 0.2);
}

.avatar-container:hover .avatar-fallback {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}
</style>