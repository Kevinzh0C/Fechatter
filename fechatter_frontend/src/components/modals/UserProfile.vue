/**
* User Profile Modal - Production Stub
* Displays user profile information in a modal
*/

<template>
  <div class="user-profile-overlay" @click="handleOverlayClick">
    <div class="user-profile-modal" @click.stop>
      <!-- Loading State -->
      <div v-if="loading" class="profile-loading">
        <div class="loading-spinner" />
        <p>Loading user profile...</p>
      </div>

      <!-- Error State -->
      <div v-else-if="error" class="profile-error">
        <svg class="error-icon" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd"
            d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
            clip-rule="evenodd" />
        </svg>
        <p>{{ error }}</p>
        <button @click="loadUserProfile" class="retry-btn">Retry</button>
      </div>

      <!-- Profile Content -->
      <div v-else-if="profile" class="profile-content">
        <!-- Close Button -->
        <button @click="$emit('close')" class="profile-close-btn">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <path
              d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
          </svg>
        </button>

        <!-- Profile Header -->
        <div class="profile-header">
          <div class="profile-avatar">
            <div class="avatar-initials">{{ getUserInitials(profile.fullname) }}</div>
          </div>
          <div class="profile-basic-info">
            <h2 class="profile-name">{{ profile.fullname }}</h2>
            <p class="profile-email">{{ profile.email }}</p>
            <div class="profile-status">
              <span class="status-dot" :class="getStatusClass(profile.status)"></span>
              <span class="status-text">{{ getStatusText(profile.status) }}</span>
            </div>
          </div>
        </div>

        <!-- Quick Actions -->
        <div class="profile-actions">
          <button v-if="!isCurrentUser" @click="startDirectMessage" :disabled="startingDM"
            class="profile-action-btn primary">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path
                d="M20 2H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h4v3c0 .6.4 1 1 1 .2 0 .4-.1.6-.2l4.9-3.2H20c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2z" />
            </svg>
            {{ startingDM ? 'Creating...' : 'Send Message' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import { useToast } from '@/composables/useToast';
import UserService from '@/services/UserService';
import ChangePasswordModal from '@/components/user/ChangePasswordModal.vue';
import type { UserProfileResponse } from '@/types/api';
import { useChatStore } from '@/stores/chat';
import ChatService from '@/services/ChatService';
import Avatar from '@/components/common/Avatar.vue';

// Props
interface Props {
  userId?: number; // 用户ID，如果不提供则显示当前用户
  user?: any; // 向后兼容的用户对象
}

const props = withDefaults(defineProps<Props>(), {
  userId: undefined,
  user: undefined
});

// Emits
const emit = defineEmits<{
  close: [];
  dmCreated: [chat: any];
}>();

// Composables
const router = useRouter();
const authStore = useAuthStore();
const { success: notifySuccess, error: notifyError } = useToast();
const chatStore = useChatStore();

// 状态
const loading = ref(false);
const error = ref('');
const profile = ref<UserProfileResponse | null>(null);
const startingDM = ref(false);
const showChangePassword = ref(false);

// 计算属性
const isCurrentUser = computed(() => {
  const currentUserId = authStore.user?.id;
  const targetUserId = props.userId || profile.value?.id;
  return currentUserId === targetUserId;
});

// 工具函数
const getUserInitials = (name: string): string => {
  if (!name) return '?';
  return name
    .split(' ')
    .map(n => n.charAt(0))
    .join('')
    .toUpperCase()
    .slice(0, 2);
};

const getStatusClass = (status: string): string => {
  switch (status?.toLowerCase()) {
    case 'active':
    case 'online':
      return 'status-online';
    case 'away':
      return 'status-away';
    case 'busy':
      return 'status-busy';
    case 'inactive':
    case 'offline':
    default:
      return 'status-offline';
  }
};

const getStatusText = (status: string): string => {
  switch (status?.toLowerCase()) {
    case 'active':
      return 'Active';
    case 'online':
      return 'Online';
    case 'away':
      return 'Away';
    case 'busy':
      return 'Busy';
    case 'inactive':
      return 'Offline';
    case 'offline':
      return 'Offline';
    default:
      return 'Offline';
  }
};

const getTimezoneDisplay = (timezone: string): string => {
  const timezoneMap: Record<string, string> = {
    'Asia/Shanghai': '北京时间 (UTC+8)',
    'Asia/Tokyo': '东京时间 (UTC+9)',
    'America/New_York': '纽约时间 (UTC-5)',
    'America/Los_Angeles': '洛杉矶时间 (UTC-8)',
    'Europe/London': '伦敦时间 (UTC+0)',
    'Europe/Paris': '巴黎时间 (UTC+1)',
  };
  return timezoneMap[timezone] || timezone;
};

const getLanguageDisplay = (language: string): string => {
  const languageMap: Record<string, string> = {
    'zh-CN': '简体中文',
    'zh-TW': '繁體中文',
    'en': 'English',
    'ja': '日本語',
    'ko': '한국어',
  };
  return languageMap[language] || language;
};

const formatLastActive = (dateString: string): string => {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / (1000 * 60));
  const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  if (diffMins < 1) return 'Just now';
  if (diffMins < 60) return `${diffMins} min ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;

  return date.toLocaleDateString();
};

const formatJoinDate = (dateString: string): string => {
  const date = new Date(dateString);
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  });
};

// 数据加载
const loadUserProfile = async () => {
  try {
    loading.value = true;
    error.value = '';

    let targetUserId: number;

    // 确定要加载的用户ID
    if (props.userId) {
      targetUserId = props.userId;
    } else if (props.user?.id) {
      targetUserId = props.user.id;
    } else if (authStore.user?.id) {
      targetUserId = authStore.user.id;
    } else {
      throw new Error('无法确定用户ID');
    }

    // 加载用户档案
    if (targetUserId === authStore.user?.id) {
      // 加载当前用户档案
      profile.value = await UserService.getCurrentUserProfile();
    } else {
      // 加载指定用户档案
      profile.value = await UserService.getUserProfile(targetUserId);
    }

  } catch (err: any) {
    error.value = err.message || '加载用户档案失败';
    console.error('Load user profile error:', err);
  } finally {
    loading.value = false;
  }
};

// 操作处理
const handleOverlayClick = (event: Event) => {
  if (event.target === event.currentTarget) {
    emit('close');
  }
};

const handleAvatarError = (event: Event) => {
  const img = event.target as HTMLImageElement;
  img.style.display = 'none';
};

const startDirectMessage = async () => {
  if (!profile.value) return;

  startingDM.value = true;
  console.log(`[UserProfile] Attempting to find or create DM for user ID: ${(profile.value as any).id}`);

  try {
    const chat = await chatStore.findOrCreateDM((profile.value as any).id);

    if (chat && (chat as any).id) {
      console.log(`[UserProfile] Successfully got chat ID: ${(chat as any).id}. Navigating...`);
      notifySuccess("Opening conversation...");
      emit('dmCreated', chat);
      emit('close');
    } else {
      // This case should ideally not be reached if the action is robust.
      throw new Error("Could not retrieve a valid chat.");
    }
  } catch (err: any) {
    console.error('[UserProfile] Failed to start DM:', err);
    notifyError(err.message || 'An unexpected error occurred.');
  } finally {
    startingDM.value = false;
  }
};

const editProfile = () => {
  emit('close');
  router.push('/profile/edit');
};

const changePassword = () => {
  showChangePassword.value = true;
};

const handlePasswordChanged = () => {
  notifySuccess('密码修改成功');
  showChangePassword.value = false;
};

// 监听props变化
watch([() => props.userId, () => props.user], () => {
  loadUserProfile();
}, { immediate: false });

// 组件挂载时加载数据
onMounted(() => {
  // 如果有传入的用户对象且包含完整信息，直接使用
  if (props.user && props.user.fullname && props.user.email) {
    profile.value = {
      id: props.user.id,
      fullname: props.user.fullname,
      email: props.user.email,
      status: props.user.status || 'Active',
      created_at: props.user.created_at || new Date().toISOString(),
      workspace_id: props.user.workspace_id || authStore.user?.workspace_id || 0,
      phone: props.user.phone,
      title: props.user.title,
      department: props.user.department,
      avatar_url: props.user.avatar_url,
      bio: props.user.bio,
      timezone: props.user.timezone,
      language: props.user.language,
      last_active_at: props.user.last_active_at,
    };
  } else {
    // 否则从API加载完整档案
    loadUserProfile();
  }
});
</script>

<style scoped>
.user-profile-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 16px;
}

.user-profile-modal {
  background: white;
  border-radius: 16px;
  box-shadow: 0 25px 50px rgba(0, 0, 0, 0.25);
  width: 100%;
  max-width: 480px;
  max-height: 90vh;
  overflow-y: auto;
  position: relative;
}

/* Loading & Error States */
.profile-loading,
.profile-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
  text-align: center;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #f3f4f6;
  border-top: 3px solid #6366f1;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 16px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.error-icon {
  width: 48px;
  height: 48px;
  color: #dc2626;
  margin-bottom: 16px;
}

.retry-btn {
  margin-top: 16px;
  padding: 8px 16px;
  background-color: #6366f1;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.retry-btn:hover {
  background-color: #5856eb;
}

/* Profile Content */
.profile-content {
  padding: 24px;
}

.profile-close-btn {
  position: absolute;
  top: 16px;
  right: 16px;
  padding: 8px;
  border: none;
  background: none;
  color: #6b7280;
  cursor: pointer;
  border-radius: 6px;
  transition: background-color 0.2s;
  z-index: 10;
}

.profile-close-btn:hover {
  background-color: #f3f4f6;
}

/* Profile Header */
.profile-header {
  display: flex;
  align-items: flex-start;
  gap: 20px;
  margin-bottom: 24px;
  padding-top: 16px;
}

.profile-avatar {
  position: relative;
  flex-shrink: 0;
}

.avatar-initials {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 28px;
  font-weight: 600;
  color: white;
  background: linear-gradient(45deg, #6366f1, #8b5cf6);
}

.profile-basic-info {
  flex: 1;
  min-width: 0;
}

.profile-name {
  font-size: 24px;
  font-weight: 700;
  color: #1f2937;
  margin: 0 0 4px 0;
  word-wrap: break-word;
}

.profile-email {
  font-size: 16px;
  color: #6b7280;
  margin: 0 0 8px 0;
}

.profile-status {
  display: flex;
  align-items: center;
  font-size: 14px;
  color: #6b7280;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-right: 8px;
}

.status-online {
  background-color: #10b981;
}

.status-away {
  background-color: #f59e0b;
}

.status-busy {
  background-color: #ef4444;
}

.status-offline {
  background-color: #6b7280;
}

.status-text {
  font-size: 14px;
  color: #6b7280;
}

/* Profile Details */
.profile-details {
  margin-bottom: 24px;
}

.detail-section {
  margin-bottom: 20px;
}

.detail-title {
  font-size: 16px;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 12px;
}

.detail-item {
  display: flex;
  align-items: center;
  margin-bottom: 8px;
  font-size: 14px;
  color: #4b5563;
}

.detail-icon {
  width: 16px;
  height: 16px;
  margin-right: 12px;
  color: #6b7280;
  flex-shrink: 0;
}

.bio-text {
  font-size: 14px;
  color: #4b5563;
  line-height: 1.5;
  white-space: pre-wrap;
  word-wrap: break-word;
  margin: 0;
}

/* Actions */
.profile-actions {
  display: flex;
  gap: 12px;
  padding-top: 20px;
  border-top: 1px solid #f3f4f6;
}

.profile-action-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px 16px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
}

.profile-action-btn.primary {
  background-color: #6366f1;
  color: white;
}

.profile-action-btn.primary:hover:not(:disabled) {
  background-color: #5856eb;
}

.profile-action-btn.secondary {
  background-color: #f3f4f6;
  color: #374151;
}

.profile-action-btn.secondary:hover:not(:disabled) {
  background-color: #e5e7eb;
}

.profile-action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Responsive Design */
@media (max-width: 768px) {
  .user-profile-overlay {
    padding: 0;
  }

  .user-profile-modal {
    border-radius: 0;
    max-width: none;
    max-height: 100vh;
    height: 100vh;
  }

  .profile-header {
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .profile-actions {
    flex-direction: column;
  }

  .profile-action-btn {
    width: 100%;
  }
}
</style>