<template>
  <div class="avatar-container"
    :style="{ '--avatar-size': `${size}px`, width: `${size}px`, height: `${size}px`, fontSize: `${size * 0.45}px` }">
    <img v-if="!error && src" :src="src" :alt="alt" class="avatar-image" @error="handleImageError" />
    <div v-else-if="initials && initials !== '?'" class="avatar-initials" :style="{ backgroundColor: avatarColor }">
      {{ initials }}
    </div>
    <div v-else class="avatar-fallback-icon" :style="{ backgroundColor: avatarColor }">
      <svg fill="currentColor" viewBox="0 0 20 20">
        <path d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" />
      </svg>
    </div>
    <span v-if="status" class="status-indicator" :class="`status-${status}`"
      :style="{ borderWidth: `${Math.max(1, size * 0.08)}px` }"></span>
  </div>
</template>

<script setup>
import { ref, computed, toRefs } from 'vue';
import { getUserInitials, generateAvatarColor } from '@/utils/userAvatarHelper';

const props = defineProps({
  src: {
    type: String,
    default: null,
  },
  alt: {
    type: String,
    default: 'User Avatar',
  },
  userId: {
    type: [String, Number],
    default: 0,
  },
  size: {
    type: Number,
    default: 40,
  },
  status: {
    type: String,
    default: null, // 'online', 'away', 'busy', 'offline'
  },
});

const { src, alt, userId, size, status } = toRefs(props);
const error = ref(false);

const initials = computed(() => getUserInitials(alt.value));
const avatarColor = computed(() => generateAvatarColor(userId.value));

const handleImageError = () => {
  error.value = true;
};
</script>

<style scoped>
.avatar-container {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  overflow: hidden;
  flex-shrink: 0;
  box-shadow:
    inset 0 1px 1px rgba(0, 0, 0, 0.08),
    0 0 0 1.5px rgba(255, 255, 255, 0.4),
    0 3px 8px rgba(0, 0, 0, 0.12);
  transition: all 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

.avatar-container:hover {
  transform: scale(1.08);
  box-shadow:
    inset 0 1px 2px rgba(0, 0, 0, 0.1),
    0 0 0 2px rgba(255, 255, 255, 0.5),
    0 5px 12px rgba(0, 0, 0, 0.2);
}

.avatar-container:active {
  transform: scale(1.02);
  transition: transform 0.1s ease;
}

.avatar-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  color: white;
  text-transform: uppercase;
  user-select: none;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

.avatar-initials,
.avatar-fallback-icon {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  color: white;
  text-transform: uppercase;
  user-select: none;
}

.avatar-fallback-icon {
  padding: 15%;
  box-sizing: border-box;
  bottom: 5%;
  right: 5%;
  width: 25%;
  height: 25%;
  border-radius: 50%;
  background-color: #64748b;
  border-style: solid;
  border-color: #ffffff;
  box-sizing: border-box;
  animation: pulse-status 2s infinite ease-in-out;
}

@keyframes pulse-status {

  0%,
  100% {
    transform: scale(1);
    box-shadow: 0 0 0 0 rgba(255, 255, 255, 0.1);
  }

  50% {
    transform: scale(1.1);
    box-shadow: 0 0 0 3px rgba(255, 255, 255, 0);
  }
}

.status-indicator {
  position: absolute;
  width: calc(var(--avatar-size, 40px) * 0.236);
  height: calc(var(--avatar-size, 40px) * 0.236);
  right: calc(var(--avatar-size, 40px) * 0.09);
  bottom: calc(var(--avatar-size, 40px) * 0.09);
  border-radius: 50%;
  border-style: solid;
  border-color: #ffffff;
  box-sizing: border-box;
  animation: none;
}

@keyframes pulse-online {

  0%,
  100% {
    transform: scale(1);
    opacity: 1;
  }

  50% {
    transform: scale(1.1);
    opacity: 0.8;
  }
}

.status-online {
  background-color: #22c55e;
  animation: pulse-online 2.5s infinite ease-in-out;
}

.status-away {
  background-color: transparent;
  border: calc(var(--avatar-size, 40px) * 0.07) solid #f59e0b;
}

.status-busy {
  background-color: #ef4444;
  display: flex;
  align-items: center;
  justify-content: center;
}

.status-busy::before {
  content: '';
  display: block;
  width: 60%;
  height: 15%;
  background-color: white;
  border-radius: 1px;
}

.status-offline {
  background-color: transparent;
  border: calc(var(--avatar-size, 40px) * 0.05) solid #64748b;
}
</style>