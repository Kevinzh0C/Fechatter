<template>
  <div class="bg-white rounded-lg shadow-lg max-w-2xl w-full max-h-[80vh] overflow-hidden">
    <!-- Header -->
    <div class="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
      <h3 class="text-lg font-semibold text-gray-900">Manage Members</h3>
      <button @click="$emit('close')" class="text-gray-400 hover:text-gray-600">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
        </svg>
      </button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto">
      <!-- Current Members -->
      <div class="px-6 py-4">
        <div class="flex items-center justify-between mb-4">
          <h4 class="text-md font-medium text-gray-900">
            Current Members ({{ members.length }})
          </h4>
          <button @click="showAddMembers = true" 
                  class="px-3 py-1 bg-purple-600 text-white text-sm rounded hover:bg-purple-700">
            Add Members
          </button>
        </div>

        <!-- Loading State -->
        <div v-if="loading" class="flex justify-center py-8">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-purple-600"></div>
        </div>

        <!-- Members List -->
        <div v-else class="space-y-3">
          <div v-for="member in members" :key="member.id" 
               class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
            <div class="flex items-center space-x-3">
              <!-- Avatar -->
              <div class="w-10 h-10 bg-purple-600 rounded-full flex items-center justify-center">
                <span class="text-white font-medium">
                  {{ member.fullname?.charAt(0).toUpperCase() || 'U' }}
                </span>
              </div>
              
              <!-- Member Info -->
              <div>
                <p class="font-medium text-gray-900">{{ member.fullname }}</p>
                <p class="text-sm text-gray-500">{{ member.email }}</p>
                <div class="flex items-center space-x-2 mt-1">
                  <span v-if="member.id === chat.owner_id" 
                        class="px-2 py-1 bg-yellow-100 text-yellow-800 text-xs rounded-full">
                    Owner
                  </span>
                  <span v-if="member.role" 
                        class="px-2 py-1 bg-blue-100 text-blue-800 text-xs rounded-full">
                    {{ member.role }}
                  </span>
                </div>
              </div>
            </div>

            <!-- Actions -->
            <div class="flex items-center space-x-2">
              <!-- Transfer Ownership -->
              <button v-if="canTransferOwnership(member)" 
                      @click="transferOwnership(member)"
                      class="px-3 py-1 bg-yellow-100 text-yellow-800 text-sm rounded hover:bg-yellow-200"
                      :disabled="transferring">
                Make Owner
              </button>
              
              <!-- Remove Member -->
              <button v-if="canRemoveMember(member)" 
                      @click="removeMember(member)"
                      class="px-3 py-1 bg-red-100 text-red-800 text-sm rounded hover:bg-red-200"
                      :disabled="removing">
                Remove
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Add Members Modal -->
      <div v-if="showAddMembers" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
        <div class="bg-white rounded-lg p-6 w-full max-w-md mx-4">
          <div class="flex items-center justify-between mb-4">
            <h4 class="text-lg font-medium text-gray-900">Add Members</h4>
            <button @click="showAddMembers = false" class="text-gray-400 hover:text-gray-600">
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
              </svg>
            </button>
          </div>

          <!-- Search Users -->
          <div class="mb-4">
            <input v-model="searchQuery" 
                   @input="searchUsers"
                   placeholder="Search users by name or email..."
                   class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500">
          </div>

          <!-- Available Users -->
          <div class="max-h-60 overflow-y-auto mb-4">
            <div v-if="searchingUsers" class="text-center py-4 text-gray-500">
              Searching...
            </div>
            <div v-else-if="availableUsers.length === 0" class="text-center py-4 text-gray-500">
              No users found
            </div>
            <div v-else class="space-y-2">
              <div v-for="user in availableUsers" :key="user.id" 
                   class="flex items-center justify-between p-2 hover:bg-gray-50 rounded">
                <div class="flex items-center space-x-3">
                  <div class="w-8 h-8 bg-purple-600 rounded-full flex items-center justify-center">
                    <span class="text-white text-sm font-medium">
                      {{ user.fullname?.charAt(0).toUpperCase() || 'U' }}
                    </span>
                  </div>
                  <div>
                    <p class="font-medium text-gray-900">{{ user.fullname }}</p>
                    <p class="text-sm text-gray-500">{{ user.email }}</p>
                  </div>
                </div>
                <button @click="toggleUserSelection(user)"
                        class="px-3 py-1 text-sm rounded"
                        :class="selectedUsers.includes(user.id) 
                          ? 'bg-purple-600 text-white' 
                          : 'bg-gray-200 text-gray-700 hover:bg-gray-300'">
                  {{ selectedUsers.includes(user.id) ? 'Selected' : 'Select' }}
                </button>
              </div>
            </div>
          </div>

          <!-- Selected Users -->
          <div v-if="selectedUsers.length > 0" class="mb-4">
            <p class="text-sm font-medium text-gray-700 mb-2">
              Selected Users ({{ selectedUsers.length }})
            </p>
            <div class="flex flex-wrap gap-2">
              <span v-for="userId in selectedUsers" :key="userId" 
                    class="px-2 py-1 bg-purple-100 text-purple-800 text-sm rounded-full flex items-center">
                {{ getUserName(userId) }}
                <button @click="selectedUsers = selectedUsers.filter(id => id !== userId)"
                        class="ml-1 text-purple-600 hover:text-purple-800">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                  </svg>
                </button>
              </span>
            </div>
          </div>

          <!-- Actions -->
          <div class="flex justify-end space-x-3">
            <button @click="showAddMembers = false" 
                    class="px-4 py-2 text-gray-600 hover:text-gray-800">
              Cancel
            </button>
            <button @click="addSelectedMembers" 
                    :disabled="selectedUsers.length === 0 || adding"
                    class="px-4 py-2 bg-purple-600 text-white rounded hover:bg-purple-700 disabled:opacity-50">
              {{ adding ? 'Adding...' : `Add ${selectedUsers.length} Member${selectedUsers.length !== 1 ? 's' : ''}` }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Error Message -->
    <div v-if="error" class="px-6 py-3 bg-red-50 border-t border-red-200">
      <p class="text-sm text-red-600">{{ error }}</p>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
import { useChatStore } from '../../stores/chat';
import { useUserStore } from '../../stores/user';
import { useAuthStore } from '../../stores/auth';

const props = defineProps({
  chat: {
    type: Object,
    required: true
  }
});

const emit = defineEmits(['close', 'updated']);

const chatStore = useChatStore();
const userStore = useUserStore();
const authStore = useAuthStore();

const members = ref([]);
const loading = ref(false);
const error = ref('');
const showAddMembers = ref(false);
const searchQuery = ref('');
const availableUsers = ref([]);
const selectedUsers = ref([]);
const searchingUsers = ref(false);
const adding = ref(false);
const removing = ref(false);
const transferring = ref(false);

const currentUserId = computed(() => authStore.user?.id);

const canRemoveMember = (member) => {
  return member.id !== props.chat.owner_id && 
         member.id !== currentUserId.value &&
         (currentUserId.value === props.chat.owner_id);
};

const canTransferOwnership = (member) => {
  return member.id !== props.chat.owner_id && 
         currentUserId.value === props.chat.owner_id;
};

async function loadMembers() {
  try {
    loading.value = true;
    error.value = '';
    members.value = await chatStore.fetchChatMembers(props.chat.id);
  } catch (err) {
    error.value = err.message || 'Failed to load members';
  } finally {
    loading.value = false;
  }
}

async function searchUsers() {
  if (!searchQuery.value.trim()) {
    availableUsers.value = [];
    return;
  }

  try {
    searchingUsers.value = true;
    const allUsers = await userStore.fetchWorkspaceUsers();
    const memberIds = members.value.map(m => m.id);
    
    availableUsers.value = allUsers.filter(user => 
      !memberIds.includes(user.id) &&
      (user.fullname?.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
       user.email?.toLowerCase().includes(searchQuery.value.toLowerCase()))
    );
  } catch (err) {
    error.value = err.message || 'Failed to search users';
  } finally {
    searchingUsers.value = false;
  }
}

function toggleUserSelection(user) {
  const index = selectedUsers.value.indexOf(user.id);
  if (index > -1) {
    selectedUsers.value.splice(index, 1);
  } else {
    selectedUsers.value.push(user.id);
  }
}

function getUserName(userId) {
  const user = availableUsers.value.find(u => u.id === userId);
  return user?.fullname || user?.email || 'Unknown User';
}

async function addSelectedMembers() {
  if (selectedUsers.value.length === 0) return;

  try {
    adding.value = true;
    error.value = '';
    
    await chatStore.addChatMembers(props.chat.id, selectedUsers.value);
    
    // Refresh members list
    await loadMembers();
    
    // Reset form
    selectedUsers.value = [];
    searchQuery.value = '';
    availableUsers.value = [];
    showAddMembers.value = false;
    
    emit('updated');
  } catch (err) {
    error.value = err.message || 'Failed to add members';
  } finally {
    adding.value = false;
  }
}

async function removeMember(member) {
  if (!confirm(`Are you sure you want to remove ${member.fullname} from this chat?`)) {
    return;
  }

  try {
    removing.value = true;
    error.value = '';
    
    await chatStore.removeChatMembers(props.chat.id, [member.id]);
    
    // Refresh members list
    await loadMembers();
    emit('updated');
  } catch (err) {
    error.value = err.message || 'Failed to remove member';
  } finally {
    removing.value = false;
  }
}

async function transferOwnership(member) {
  if (!confirm(`Are you sure you want to transfer ownership to ${member.fullname}? You will no longer be the owner of this chat.`)) {
    return;
  }

  try {
    transferring.value = true;
    error.value = '';
    
    await chatStore.transferChatOwnership(props.chat.id, member.id);
    
    // Refresh members list
    await loadMembers();
    emit('updated');
  } catch (err) {
    error.value = err.message || 'Failed to transfer ownership';
  } finally {
    transferring.value = false;
  }
}

onMounted(() => {
  loadMembers();
});
</script> 