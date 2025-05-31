<template>
  <div class="w-64 bg-gray-800 text-white flex flex-col h-screen p-4 text-sm">
    <div class="flex items-center justify-between mb-6">
      <div class="font-bold text-base truncate cursor-pointer" @click="toggleDropdown">
        <span>{{ workspaceName }}</span>
        <button class="text-gray-400 ml-1">&nbsp;▼</button>
      </div>
      <div v-if="dropdownVisible" class="absolute top-12 left-0 w-48 bg-gray-800 border border-gray-700 rounded-md shadow-lg z-10">
        <ul class="py-1">
          <li @click="handleLogout" class="px-4 py-2 hover:bg-gray-700 cursor-pointer">Logout</li>
        </ul>
      </div>
      <button @click="addChannel" class="text-gray-400 text-xl hover:text-white">+</button>
    </div>

    <div class="mb-6">
      <h2 class="text-xs uppercase text-gray-400 mb-2">Channels</h2>
      <ul>
        <li v-for="channel in channels" :key="channel.id" @click="selectChannel(channel.id)"
            :class="['px-2 py-1 rounded cursor-pointer', { 'bg-blue-600': channel.id === activeChannelId }]">
          # {{ channel.name }}
        </li>
      </ul>
    </div>

    <div>
      <h2 class="text-xs uppercase text-gray-400 mb-2">Direct Messages</h2>
      <ul>
        <li v-for="dm in directMessages" :key="dm.id" @click="selectChannel(dm.id)"
            :class="['flex items-center px-2 py-1 rounded cursor-pointer', { 'bg-blue-600': dm.id === activeChannelId }]">
          <div class="w-2 h-2 bg-green-500 rounded-full mr-2"></div>
          {{ dm.name }}
        </li>
      </ul>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';

const props = defineProps({
  workspaceName: {
    type: String,
    default: 'My Workspace'
  },
  channels: {
    type: Array,
    default: () => []
  },
  directMessages: {
    type: Array,
    default: () => []
  },
  activeChannelId: {
    type: String,
    default: null
  }
});

const emit = defineEmits(['logout', 'add-channel', 'select-channel']);

const dropdownVisible = ref(false);

function toggleDropdown() {
  dropdownVisible.value = !dropdownVisible.value;
}

function handleLogout() {
  emit('logout');
}

function addChannel() {
  emit('add-channel');
}

function selectChannel(channelId) {
  emit('select-channel', channelId);
}

onMounted(() => {
  document.addEventListener('click', handleOutsideClick);
});

function handleOutsideClick(event) {
  if (!event.target.closest('.dropdown-trigger')) {
    dropdownVisible.value = false;
  }
}
</script>