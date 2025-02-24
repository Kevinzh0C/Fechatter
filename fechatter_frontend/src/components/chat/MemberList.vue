<template>
  <div class="member-list-pane" :class="{ 'visible': visible }">
    <div class="member-list-header">
      <h4>Members</h4>
      <button @click="$emit('close')" class="close-btn">&times;</button>
    </div>
    <div class="member-list-content">
      <input type="text" placeholder="Search members..." class="search-input" v-model="searchTerm" />
      <ul>
        <li v-for="member in filteredMembers" :key="member.id" class="member-item">
          <Avatar :user="member" :size="32" class="member-avatar" />
          <span class="member-name">{{ member.fullname }}</span>
        </li>
      </ul>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';
import Avatar from '@/components/ui/Avatar.vue';

const props = defineProps({
  members: { type: Array, required: true },
  visible: { type: Boolean, default: false },
});
defineEmits(['close']);

const searchTerm = ref('');

const filteredMembers = computed(() => {
  if (!searchTerm.value) {
    return props.members;
  }
  return props.members.filter(member =>
    member.fullname.toLowerCase().includes(searchTerm.value.toLowerCase())
  );
});
</script>

<style scoped>
.member-list-pane {
  /* styles for the slide-out panel */
  position: fixed;
  right: -300px;
  /* Initially hidden */
  top: 0;
  width: 300px;
  height: 100%;
  background: white;
  border-left: 1px solid #e1e5e9;
  transition: right 0.3s ease;
  z-index: 1001;
}

.member-list-pane.visible {
  right: 0;
  /* Slide in */
}

.member-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 16px;
  border-radius: 8px;
  transition: background-color 0.2s ease;
}

.member-item:hover {
  background-color: #f3f4f6;
}

.member-avatar {
  flex-shrink: 0;
}

.member-name {
  font-weight: 500;
  color: #1f2937;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Additional styles for header, list, items etc. */
</style>