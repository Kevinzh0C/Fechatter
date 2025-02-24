<template>
  <div class="bg-white rounded-lg shadow-lg max-w-4xl w-full max-h-[90vh] overflow-hidden">
    <!-- Header -->
    <div class="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
      <h2 class="text-xl font-semibold text-gray-900">Workspace Settings</h2>
      <button @click="$emit('close')" class="text-gray-400 hover:text-gray-600">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
        </svg>
      </button>
    </div>

    <!-- Content -->
    <div class="flex h-full">
      <!-- Sidebar Navigation -->
      <div class="w-64 bg-gray-50 border-r border-gray-200">
        <nav class="p-4 space-y-2">
          <button v-for="tab in tabs" :key="tab.id" @click="activeTab = tab.id"
            class="w-full text-left px-3 py-2 rounded-md text-sm font-medium transition-colors" :class="activeTab === tab.id
              ? 'bg-purple-100 text-purple-700'
              : 'text-gray-600 hover:text-gray-900 hover:bg-gray-100'">
            <div class="flex items-center">
              <component :is="tab.icon" class="w-5 h-5 mr-3" />
              {{ tab.name }}
            </div>
          </button>
        </nav>
      </div>

      <!-- Main Content -->
      <div class="flex-1 overflow-y-auto">
        <!-- General Settings -->
        <div v-if="activeTab === 'general'" class="p-6">
          <h3 class="text-lg font-medium text-gray-900 mb-4">General Settings</h3>

          <div class="space-y-6">
            <!-- Workspace Name -->
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                Workspace Name
              </label>
              <input v-model="workspaceName" type="text"
                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500"
                placeholder="Enter workspace name">
            </div>

            <!-- Workspace Description -->
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                Description
              </label>
              <textarea v-model="workspaceDescription" rows="3"
                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500"
                placeholder="Describe your workspace..."></textarea>
            </div>

            <!-- Workspace Info -->
            <div class="bg-gray-50 p-4 rounded-lg">
              <h4 class="text-sm font-medium text-gray-900 mb-3">Workspace Information</h4>
              <div class="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span class="text-gray-500">Created:</span>
                  <span class="ml-2 text-gray-900">{{ formatDate(workspaceStore.currentWorkspace?.created_at) }}</span>
                </div>
                <div>
                  <span class="text-gray-500">Owner:</span>
                  <span class="ml-2 text-gray-900">{{ workspaceStore.workspaceOwner?.fullname || 'Unknown' }}</span>
                </div>
                <div>
                  <span class="text-gray-500">Members:</span>
                  <span class="ml-2 text-gray-900">{{ workspaceStore.memberCount }}</span>
                </div>
                <div>
                  <span class="text-gray-500">Channels:</span>
                  <span class="ml-2 text-gray-900">{{ workspaceStore.chatStats.length }}</span>
                </div>
              </div>
            </div>

            <!-- Save Button -->
            <div class="flex justify-end">
              <button @click="saveWorkspaceSettings" :disabled="workspaceStore.loading"
                class="px-4 py-2 bg-purple-600 text-white rounded-md hover:bg-purple-700 disabled:opacity-50">
                {{ workspaceStore.loading ? 'Saving...' : 'Save Changes' }}
              </button>
            </div>
          </div>
        </div>

        <!-- Members Management -->
        <div v-if="activeTab === 'members'" class="p-6">
          <div class="flex items-center justify-between mb-6">
            <h3 class="text-lg font-medium text-gray-900">Members</h3>
            <button @click="showInviteModal = true"
              class="px-4 py-2 bg-purple-600 text-white rounded-md hover:bg-purple-700">
              Invite Member
            </button>
          </div>

          <!-- Members List -->
          <div class="space-y-3">
            <div v-for="member in workspaceStore.workspaceUsers" :key="member.id"
              class="flex items-center justify-between p-4 bg-gray-50 rounded-lg">
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
                    <span v-if="workspaceStore.isWorkspaceOwner(member.id)"
                      class="px-2 py-1 bg-yellow-100 text-yellow-800 text-xs rounded-full">
                      Owner
                    </span>
                    <span class="px-2 py-1 bg-blue-100 text-blue-800 text-xs rounded-full">
                      Member
                    </span>
                    <span class="text-xs text-gray-500">
                      Joined {{ formatDate(member.created_at) }}
                    </span>
                  </div>
                </div>
              </div>

              <!-- Actions -->
              <div class="flex items-center space-x-2">
                <button v-if="canTransferOwnership(member)" @click="confirmTransferOwnership(member)"
                  class="px-3 py-1 bg-yellow-100 text-yellow-800 text-sm rounded hover:bg-yellow-200">
                  Make Owner
                </button>

                <button v-if="canRemoveMember(member)" @click="confirmRemoveMember(member)"
                  class="px-3 py-1 bg-red-100 text-red-800 text-sm rounded hover:bg-red-200">
                  Remove
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- Channels Overview -->
        <div v-if="activeTab === 'channels'" class="p-6">
          <div class="flex items-center justify-between mb-6">
            <h3 class="text-lg font-medium text-gray-900">Channels Overview</h3>
            <button @click="refreshStats" :disabled="workspaceStore.loading"
              class="px-3 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 disabled:opacity-50">
              {{ workspaceStore.loading ? 'Refreshing...' : 'Refresh Stats' }}
            </button>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div v-for="chat in workspaceStore.chatStats" :key="chat.id"
              class="p-4 border border-gray-200 rounded-lg relative">
              <!-- 归档状态指示器 -->
              <div v-if="chat.is_archived"
                class="absolute top-2 right-2 px-2 py-1 bg-gray-100 text-gray-500 text-xs rounded-full">
                Archived
              </div>

              <div class="flex items-center justify-between mb-2">
                <h4 class="font-medium text-gray-900"># {{ chat.name }}</h4>
                <div class="flex items-center gap-2">
                  <!-- 活跃度指示器 -->
                  <span :class="{
                    'bg-green-100 text-green-600': chat.activity_level === 'high',
                    'bg-yellow-100 text-yellow-600': chat.activity_level === 'medium',
                    'bg-gray-100 text-gray-600': chat.activity_level === 'low'
                  }" class="px-2 py-1 text-xs rounded-full">
                    {{ chat.activity_level }} activity
                  </span>
                  <span class="px-2 py-1 bg-gray-100 text-gray-600 text-xs rounded-full">
                    {{ chat.chat_type }}
                  </span>
                </div>
              </div>

              <p class="text-sm text-gray-600 mb-3">{{ chat.description || 'No description' }}</p>

              <!-- 详细统计信息 -->
              <div class="grid grid-cols-2 gap-4 text-xs text-gray-500 mb-3">
                <div>
                  <span class="font-medium">{{ chat.member_count }}</span> members
                </div>
                <div>
                  <span class="font-medium">{{ chat.message_count }}</span> messages
                </div>
                <div>
                  <span class="font-medium">{{ chat.file_count }}</span> files
                </div>
                <div>
                  Score: <span class="font-medium">{{ Math.round(chat.activity_score) }}</span>
                </div>
              </div>

              <div class="flex items-center justify-between text-xs text-gray-500">
                <span>Creator: {{ chat.creator_name }}</span>
                <span>Created {{ formatDate(chat.created_at) }}</span>
              </div>

              <div v-if="chat.last_activity" class="text-xs text-gray-500 mt-1">
                Last activity: {{ formatDate(chat.last_activity) }}
              </div>

              <!-- 管理员操作按钮 -->
              <div v-if="workspaceStore.isWorkspaceOwner(currentUserId)"
                class="flex gap-2 mt-3 pt-3 border-t border-gray-100">
                <button @click="editChat(chat)"
                  class="px-3 py-1 text-xs bg-blue-50 text-blue-600 rounded hover:bg-blue-100">
                  Edit
                </button>
                <button @click="toggleArchiveChat(chat)" :class="{
                  'bg-gray-50 text-gray-600 hover:bg-gray-100': chat.is_archived,
                  'bg-orange-50 text-orange-600 hover:bg-orange-100': !chat.is_archived
                }" class="px-3 py-1 text-xs rounded">
                  {{ chat.is_archived ? 'Unarchive' : 'Archive' }}
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- Permissions -->
        <div v-if="activeTab === 'permissions'" class="p-6">
          <h3 class="text-lg font-medium text-gray-900 mb-6">Permissions & Security</h3>

          <div class="space-y-6">
            <!-- Workspace Permissions -->
            <div>
              <h4 class="text-sm font-medium text-gray-900 mb-3">Workspace Permissions</h4>
              <div class="space-y-3">
                <label class="flex items-center">
                  <input type="checkbox" v-model="permissions.allowMemberInvites"
                    class="rounded border-gray-300 text-purple-600 focus:ring-purple-500">
                  <span class="ml-2 text-sm text-gray-700">Allow members to invite new users</span>
                </label>
                <label class="flex items-center">
                  <input type="checkbox" v-model="permissions.allowChannelCreation"
                    class="rounded border-gray-300 text-purple-600 focus:ring-purple-500">
                  <span class="ml-2 text-sm text-gray-700">Allow members to create channels</span>
                </label>
                <label class="flex items-center">
                  <input type="checkbox" v-model="permissions.allowFileUploads"
                    class="rounded border-gray-300 text-purple-600 focus:ring-purple-500">
                  <span class="ml-2 text-sm text-gray-700">Allow file uploads</span>
                </label>
              </div>
            </div>

            <!-- Security Settings -->
            <div>
              <h4 class="text-sm font-medium text-gray-900 mb-3">Security Settings</h4>
              <div class="space-y-3">
                <label class="flex items-center">
                  <input type="checkbox" v-model="security.requireEmailVerification"
                    class="rounded border-gray-300 text-purple-600 focus:ring-purple-500">
                  <span class="ml-2 text-sm text-gray-700">Require email verification for new members</span>
                </label>
                <label class="flex items-center">
                  <input type="checkbox" v-model="security.enableTwoFactor"
                    class="rounded border-gray-300 text-purple-600 focus:ring-purple-500">
                  <span class="ml-2 text-sm text-gray-700">Enable two-factor authentication</span>
                </label>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Invite Member Modal -->
    <div v-if="showInviteModal" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white rounded-lg p-6 w-96">
        <h3 class="text-lg font-medium text-gray-900 mb-4">Invite Member</h3>

        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Email Address</label>
            <input v-model="inviteEmail" type="email"
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500"
              placeholder="Enter email address">
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Role</label>
            <select v-model="inviteRole"
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500">
              <option value="member">Member</option>
              <option value="admin">Admin</option>
            </select>
          </div>
        </div>

        <div class="flex justify-end space-x-3 mt-6">
          <button @click="showInviteModal = false" class="px-4 py-2 text-gray-600 hover:text-gray-800">
            Cancel
          </button>
          <button @click="inviteMember" :disabled="!inviteEmail || workspaceStore.loading"
            class="px-4 py-2 bg-purple-600 text-white rounded hover:bg-purple-700 disabled:opacity-50">
            {{ workspaceStore.loading ? 'Inviting...' : 'Send Invite' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Error Message -->
    <div v-if="workspaceStore.error"
      class="absolute bottom-4 right-4 bg-red-50 border border-red-200 rounded-lg p-4 max-w-sm">
      <p class="text-sm text-red-600">{{ workspaceStore.error }}</p>
      <button @click="workspaceStore.clearError()" class="mt-2 text-xs text-red-500 hover:text-red-700">
        Dismiss
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
import { useWorkspaceStore } from '../../stores/workspace';
import { useChatStore } from '../../stores/chat';
import { useAuthStore } from '../../stores/auth';

// Icons (you can replace these with actual icon components)
const SettingsIcon = 'svg';
const UsersIcon = 'svg';
const HashIcon = 'svg';
const ShieldIcon = 'svg';

const emit = defineEmits(['close']);

const workspaceStore = useWorkspaceStore();
const chatStore = useChatStore();
const authStore = useAuthStore();

const activeTab = ref('general');
const workspaceName = ref('');
const workspaceDescription = ref('');
const showInviteModal = ref(false);
const inviteEmail = ref('');
const inviteRole = ref('member');

const permissions = ref({
  allowMemberInvites: true,
  allowChannelCreation: true,
  allowFileUploads: true
});

const security = ref({
  requireEmailVerification: false,
  enableTwoFactor: false
});

const tabs = [
  { id: 'general', name: 'General', icon: SettingsIcon },
  { id: 'members', name: 'Members', icon: UsersIcon },
  { id: 'channels', name: 'Channels', icon: HashIcon },
  { id: 'permissions', name: 'Permissions', icon: ShieldIcon }
];

const currentUserId = computed(() => authStore.user?.id);

const canRemoveMember = (member) => {
  return member.id !== currentUserId.value &&
    workspaceStore.isWorkspaceOwner(currentUserId.value) &&
    !workspaceStore.isWorkspaceOwner(member.id);
};

const canTransferOwnership = (member) => {
  return member.id !== currentUserId.value &&
    workspaceStore.isWorkspaceOwner(currentUserId.value);
};

function formatDate(dateString) {
  if (!dateString) return 'Unknown';
  const date = new Date(dateString);
  return date.toLocaleDateString([], {
    year: 'numeric',
    month: 'short',
    day: 'numeric'
  });
}

async function saveWorkspaceSettings() {
  try {
    await workspaceStore.updateWorkspace(workspaceName.value, workspaceDescription.value);
  } catch (error) {
    console.error('Failed to save workspace settings:', error);
  }
}

async function inviteMember() {
  try {
    await workspaceStore.inviteUserToWorkspace(inviteEmail.value, inviteRole.value);
    showInviteModal.value = false;
    inviteEmail.value = '';
    inviteRole.value = 'member';
  } catch (error) {
    console.error('Failed to invite member:', error);
  }
}

async function confirmRemoveMember(member) {
  const confirmed = confirm(`Are you sure you want to remove ${member.fullname} from the workspace?`);
  if (!confirmed) return;

  try {
    await workspaceStore.removeUserFromWorkspace(member.id);
  } catch (error) {
    console.error('Failed to remove member:', error);
  }
}

async function confirmTransferOwnership(member) {
  const confirmed = confirm(`Are you sure you want to transfer ownership to ${member.fullname}? You will no longer be the workspace owner.`);
  if (!confirmed) return;

  try {
    await workspaceStore.transferWorkspaceOwnership(member.id);
  } catch (error) {
    console.error('Failed to transfer ownership:', error);
  }
}

// 新增：编辑聊天方法
async function editChat(chat) {
  const newName = prompt('Enter new chat name:', chat.name);
  if (!newName || newName === chat.name) return;

  try {
    await workspaceStore.updateWorkspaceChat(chat.id, { name: newName });
  } catch (error) {
    console.error('Failed to update chat:', error);
  }
}

// 新增：归档/取消归档聊天方法
async function toggleArchiveChat(chat) {
  const action = chat.is_archived ? 'unarchive' : 'archive';
  const confirmed = confirm(`Are you sure you want to ${action} "${chat.name}"?`);
  if (!confirmed) return;

  try {
    await workspaceStore.archiveWorkspaceChat(chat.id, !chat.is_archived);
  } catch (error) {
    console.error(`Failed to ${action} chat:`, error);
  }
}

async function refreshStats() {
  try {
    await workspaceStore.refreshStats();
  } catch (error) {
    console.error('Failed to refresh workspace stats:', error);
  }
}

onMounted(() => {
  // Initialize form with current workspace data
  if (workspaceStore.currentWorkspace) {
    workspaceName.value = workspaceStore.currentWorkspace.name || '';
    workspaceDescription.value = workspaceStore.currentWorkspace.description || '';
  }
});
</script>