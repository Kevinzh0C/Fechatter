<!--
  MemberAvatar.vue
  Simple Member Avatar Component
-->
<template>
  <div class="member-avatar" :style="{ width: `${size}px`, height: `${size}px` }">
    <img v-if="member.avatar_url && !imageError" :src="member.avatar_url" :alt="member.fullname" class="avatar-image"
      @error="handleImageError" />
    <div v-else class="avatar-initials">
      {{ getInitials(member.fullname) }}
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';

const props = defineProps({
  member: {
    type: Object,
    required: true
  },
  size: {
    type: Number,
    default: 32
  }
});

const imageError = ref(false);

const handleImageError = () => {
  imageError.value = true;
};

const getInitials = (name) => {
  if (!name) return '?';
  return name
    .split(' ')
    .map(word => word.charAt(0))
    .join('')
    .toUpperCase()
    .slice(0, 2);
};
</script>

<style scoped>
.member-avatar {
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  overflow: hidden;
  flex-shrink: 0;
}

.avatar-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.avatar-initials {
  font-weight: 600;
  color: white;
  font-size: 12px;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
}
</style>