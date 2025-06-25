<template>
  <div v-if="isVisible" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="bg-white rounded-lg shadow-xl w-full max-w-md mx-4">
      <div class="px-6 py-4 border-b border-gray-200">
        <div class="flex items-center justify-between">
          <h3 class="text-lg font-medium text-gray-900">Create Group Chat</h3>
          <button @click="closeModal" class="text-gray-400 hover:text-gray-600">
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
            </svg>
          </button>
        </div>
      </div>

      <div class="px-6 py-4">
        <!-- Group Name -->
        <div class="mb-4">
          <label for="groupName" class="block text-sm font-medium text-gray-700 mb-2">
            Group Name (Optional)
          </label>
          <input
            id="groupName"
            v-model="groupName"
            type="text"
            placeholder="Enter group name..."
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
          <p class="text-xs text-gray-500 mt-1">
            If left empty, the group will be named after its members
          </p>
        </div>

        <!-- Member Search -->
        <div class="mb-4">
          <label for="memberSearch" class="block text-sm font-medium text-gray-700 mb-2">
            Add Members
          </label>
          <div class="relative">
            <input
              id="memberSearch"
              v-model="searchQuery"
              type="text"
              placeholder="Search users..."
              @input="searchUsers"
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <div v-if="searchResults.length > 0" class="absolute z-10 w-full mt-1 bg-white border border-gray-300 rounded-md shadow-lg max-h-40 overflow-y-auto">
              <div
                v-for="user in searchResults"
                :key="user.id"
                @click="addMember(user)"
                class="px-3 py-2 hover:bg-gray-100 cursor-pointer flex items-center"
              >
                <div class="w-8 h-8 bg-gray-300 rounded-full flex items-center justify-center text-sm font-medium mr-3">
                  {{ user.name.charAt(0).toUpperCase() }}
                </div>
                <div>
                  <div class="font-medium text-gray-900">{{ user.name }}</div>
                  <div class="text-sm text-gray-500">{{ user.email }}</div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Selected Members -->
        <div v-if="selectedMembers.length > 0" class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-2">
            Selected Members ({{ selectedMembers.length }})
          </label>
          <div class="flex flex-wrap gap-2">
            <div
              v-for="member in selectedMembers"
              :key="member.id"
              class="inline-flex items-center bg-blue-100 text-blue-800 text-sm px-3 py-1 rounded-full"
            >
              <span>{{ member.name }}</span>
              <button
                @click="removeMember(member.id)"
                class="ml-2 text-blue-600 hover:text-blue-800"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                </svg>
              </button>
            </div>
          </div>
        </div>

        <!-- Group Type -->
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-2">
            Group Type
          </label>
          <div class="space-y-2">
            <label class="flex items-center">
              <input
                v-model="groupType"
                type="radio"
                value="public"
                class="text-blue-600 border-gray-300 focus:ring-blue-500"
              />
              <span class="ml-2 text-sm text-gray-700">
                <strong>Public</strong> - Anyone in the workspace can join
              </span>
            </label>
            <label class="flex items-center">
              <input
                v-model="groupType"
                type="radio"
                value="private"
                class="text-blue-600 border-gray-300 focus:ring-blue-500"
              />
              <span class="ml-2 text-sm text-gray-700">
                <strong>Private</strong> - Only invited members can join
              </span>
            </label>
          </div>
        </div>

        <!-- Description (Optional) -->
        <div class="mb-4">
          <label for="description" class="block text-sm font-medium text-gray-700 mb-2">
            Description (Optional)
          </label>
          <textarea
            id="description"
            v-model="description"
            rows="3"
            placeholder="What's this group about?"
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
          ></textarea>
        </div>
      </div>

      <div class="px-6 py-4 border-t border-gray-200 flex justify-end space-x-3">
        <button
          @click="closeModal"
          class="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 border border-gray-300 rounded-md hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-gray-500"
        >
          Cancel
        </button>
        <button
          @click="createGroupChat"
          :disabled="selectedMembers.length === 0 || isCreating"
          class="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <span v-if="isCreating">Creating...</span>
          <span v-else>Create Group</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue';

const props = defineProps({
  isVisible: {
    type: Boolean,
    default: false
  },
  availableUsers: {
    type: Array,
    default: () => []
  }
});

const emit = defineEmits(['close', 'create-group']);

// Form data
const groupName = ref('');
const description = ref('');
const groupType = ref('private');
const searchQuery = ref('');
const selectedMembers = ref([]);
const searchResults = ref([]);
const isCreating = ref(false);

// Watch for visibility changes to reset form
watch(() => props.isVisible, (newValue) => {
  if (newValue) {
    resetForm();
  }
});

function resetForm() {
  groupName.value = '';
  description.value = '';
  groupType.value = 'private';
  searchQuery.value = '';
  selectedMembers.value = [];
  searchResults.value = [];
  isCreating.value = false;
}

function searchUsers() {
  if (searchQuery.value.length < 2) {
    searchResults.value = [];
    return;
  }

  const query = searchQuery.value.toLowerCase();
  searchResults.value = props.availableUsers.filter(user => {
    const isAlreadySelected = selectedMembers.value.some(member => member.id === user.id);
    const matchesQuery = user.name.toLowerCase().includes(query) || 
                        user.email.toLowerCase().includes(query);
    return !isAlreadySelected && matchesQuery;
  });
}

function addMember(user) {
  selectedMembers.value.push(user);
  searchQuery.value = '';
  searchResults.value = [];
}

function removeMember(userId) {
  selectedMembers.value = selectedMembers.value.filter(member => member.id !== userId);
}

function closeModal() {
  emit('close');
}

async function createGroupChat() {
  if (selectedMembers.value.length === 0) return;

  isCreating.value = true;

  try {
    const groupData = {
      name: groupName.value.trim() || null,
      description: description.value.trim() || null,
      type: groupType.value,
      members: selectedMembers.value.map(member => member.id),
      created_at: new Date().toISOString()
    };

    emit('create-group', groupData);
    closeModal();
  } catch (error) {
    console.error('Failed to create group chat:', error);
  } finally {
    isCreating.value = false;
  }
}
</script>