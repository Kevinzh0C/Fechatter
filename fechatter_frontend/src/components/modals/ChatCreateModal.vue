<template>
  <div v-if="isOpen" class="modal-overlay" @click="handleOverlayClick">
    <div class="modal-container" @click.stop>
      <!-- 模态框头部 -->
      <div class="modal-header">
        <h3 class="modal-title">创建新聊天</h3>
        <button @click="close" class="modal-close-btn">
          <svg class="close-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- 模态框内容 -->
      <div class="modal-content">
        <!-- 聊天类型选择 -->
        <div class="chat-type-tabs">
          <button 
            v-for="type in chatTypes" 
            :key="type.key"
            @click="selectedType = type.key"
            class="chat-type-tab"
            :class="{ 'active': selectedType === type.key }"
          >
            <svg class="tab-icon" fill="currentColor" viewBox="0 0 20 20">
              <path :d="type.icon" />
            </svg>
            {{ type.label }}
          </button>
        </div>

        <!-- 错误提示 -->
        <div v-if="error" class="error-alert">
          <svg class="error-icon" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
          </svg>
          <span>{{ error }}</span>
        </div>

        <!-- 创建表单 -->
        <form @submit.prevent="handleSubmit" class="create-form">
          <!-- 聊天名称 -->
          <div v-if="selectedType !== 'direct'" class="form-group">
            <label for="chatName" class="form-label">
              {{ selectedType === 'channel' ? '频道名称' : '群组名称' }} *
            </label>
            <input
              id="chatName"
              v-model="formData.name"
              type="text"
              class="form-input"
              :class="{ 'form-input-error': validationErrors.name }"
              :placeholder="selectedType === 'channel' ? '例如：general' : '例如：项目讨论组'"
              required
            />
            <span v-if="validationErrors.name" class="form-error">
              {{ validationErrors.name }}
            </span>
          </div>

          <!-- 聊天描述 -->
          <div v-if="selectedType !== 'direct'" class="form-group">
            <label for="chatDescription" class="form-label">描述</label>
            <textarea
              id="chatDescription"
              v-model="formData.description"
              class="form-textarea"
              rows="3"
              :placeholder="selectedType === 'channel' ? '这个频道是用来...' : '这个群组用于...'"
            />
          </div>

          <!-- 私密性设置 -->
          <div v-if="selectedType === 'channel'" class="form-group">
            <div class="checkbox-group">
              <input
                id="isPrivate"
                v-model="formData.is_public"
                type="checkbox"
                class="form-checkbox"
              />
              <label for="isPrivate" class="checkbox-label">
                <span class="checkbox-text">公开频道</span>
                <span class="checkbox-description">
                  所有工作区成员都可以找到并加入这个频道
                </span>
              </label>
            </div>
          </div>

          <!-- 成员选择 -->
          <div v-if="selectedType !== 'channel' || !formData.is_public" class="form-group">
            <label class="form-label">
              {{ selectedType === 'direct' ? '选择聊天对象' : '添加成员' }}
              {{ selectedType === 'direct' ? '*' : '' }}
            </label>
            
            <!-- 用户搜索 -->
            <div class="user-search">
              <input
                v-model="userSearchQuery"
                type="text"
                class="form-input"
                placeholder="搜索用户..."
                @input="searchUsers"
              />
              <svg class="search-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
            </div>

            <!-- 搜索结果 -->
            <div v-if="filteredUsers.length > 0" class="user-search-results">
              <div 
                v-for="user in filteredUsers" 
                :key="user.id"
                @click="toggleUserSelection(user)"
                class="user-search-item"
                :class="{ 'selected': isUserSelected(user.id) }"
              >
                <div class="user-avatar">
                  <img 
                    v-if="user.avatar_url" 
                    :src="user.avatar_url" 
                    :alt="user.fullname"
                    class="avatar-image"
                  />
                  <span v-else class="avatar-initials">
                    {{ getUserInitials(user.fullname) }}
                  </span>
                </div>
                <div class="user-info">
                  <div class="user-name">{{ user.fullname }}</div>
                  <div class="user-email">{{ user.email }}</div>
                </div>
                <div v-if="isUserSelected(user.id)" class="selection-indicator">
                  <svg class="check-icon" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                  </svg>
                </div>
              </div>
            </div>

            <!-- 已选择的用户 -->
            <div v-if="selectedUsers.length > 0" class="selected-users">
              <div class="selected-users-label">已选择的用户:</div>
              <div class="selected-users-list">
                <div 
                  v-for="user in selectedUsers" 
                  :key="user.id"
                  class="selected-user-tag"
                >
                  {{ user.fullname }}
                  <button 
                    @click="removeUserSelection(user.id)"
                    class="remove-user-btn"
                  >
                    <svg class="remove-icon" fill="currentColor" viewBox="0 0 20 20">
                      <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                    </svg>
                  </button>
                </div>
              </div>
            </div>

            <span v-if="validationErrors.members" class="form-error">
              {{ validationErrors.members }}
            </span>
          </div>
        </form>
      </div>

      <!-- 模态框底部 -->
      <div class="modal-footer">
        <button
          type="button"
          @click="close"
          class="btn btn-secondary"
          :disabled="loading"
        >
          取消
        </button>
        
        <button
          @click="handleSubmit"
          class="btn btn-primary"
          :disabled="loading || !isFormValid"
        >
          <span v-if="loading" class="loading-spinner" />
          {{ loading ? '创建中...' : getCreateButtonText() }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted } from 'vue';
import { useToast } from '@/composables/useToast';
import { useAuthStore } from '@/stores/auth';
import ChatService from '@/services/ChatService';
import UserService from '@/services/UserService';
import type { CreateChatRequest, Chat, UserProfileResponse } from '@/types/api';

// Props
interface Props {
  isOpen: boolean;
}

const props = defineProps<Props>();

// Emits
const emit = defineEmits<{
  close: [];
  created: [chat: Chat];
}>();

// Composables
const { notifySuccess, notifyError } = useToast();
const authStore = useAuthStore();

// 状态
const loading = ref(false);
const error = ref('');
const selectedType = ref<'channel' | 'group' | 'direct'>('channel');
const userSearchQuery = ref('');
const availableUsers = ref<UserProfileResponse[]>([]);
const selectedUsers = ref<UserProfileResponse[]>([]);

// 表单数据
const formData = reactive({
  name: '',
  description: '',
  is_public: true
});

// 验证错误
const validationErrors = reactive<Record<string, string>>({});

// 聊天类型配置
const chatTypes = [
  {
    key: 'channel' as const,
    label: '频道',
    icon: 'M7 4V2c0-1.1.9-2 2-2h2c1.1 0 2 .9 2 2v2h4c1.1 0 2 .9 2 2v2h-2V8h-4v12c0 1.1-.9 2-2 2H9c-1.1 0-2-.9-2-2V8H3v2H1V6c0-1.1.9-2 2-2h4z'
  },
  {
    key: 'group' as const,
    label: '群组',
    icon: 'M13 6a3 3 0 11-6 0 3 3 0 016 0zM18 8a2 2 0 11-4 0 2 2 0 014 0zM14 15a4 4 0 00-8 0v3h8v-3zM6 8a2 2 0 11-4 0 2 2 0 014 0zM16 18v-3a5.972 5.972 0 00-.75-2.906A3.005 3.005 0 0119 15v3h-3zM4.75 12.094A5.973 5.973 0 004 15v3H1v-3a3 3 0 013.75-2.906z'
  },
  {
    key: 'direct' as const,
    label: '私信',
    icon: 'M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z'
  }
];

// 计算属性
const filteredUsers = computed(() => {
  if (!userSearchQuery.value.trim()) {
    return availableUsers.value.slice(0, 10); // 显示前10个用户
  }
  
  const query = userSearchQuery.value.toLowerCase();
  return availableUsers.value.filter(user => 
    user.fullname.toLowerCase().includes(query) ||
    user.email.toLowerCase().includes(query)
  ).slice(0, 10);
});

const isFormValid = computed(() => {
  if (selectedType.value === 'direct') {
    return selectedUsers.value.length === 1;
  }
  
  if (selectedType.value === 'channel' || selectedType.value === 'group') {
    return formData.name.trim().length > 0;
  }
  
  return false;
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

const getCreateButtonText = (): string => {
  switch (selectedType.value) {
    case 'channel':
      return '创建频道';
    case 'group':
      return '创建群组';
    case 'direct':
      return '开始聊天';
    default:
      return '创建';
  }
};

const isUserSelected = (userId: number): boolean => {
  return selectedUsers.value.some(user => user.id === userId);
};

// 用户操作
const toggleUserSelection = (user: UserProfileResponse) => {
  const index = selectedUsers.value.findIndex(u => u.id === user.id);
  
  if (index >= 0) {
    selectedUsers.value.splice(index, 1);
  } else {
    if (selectedType.value === 'direct') {
      // 私信只能选择一个用户
      selectedUsers.value = [user];
    } else {
      selectedUsers.value.push(user);
    }
};

const removeUserSelection = (userId: number) => {
  const index = selectedUsers.value.findIndex(u => u.id === userId);
  if (index >= 0) {
    selectedUsers.value.splice(index, 1);
  }
};

// 搜索用户
const searchUsers = async () => {
  if (!userSearchQuery.value.trim()) return;
  
  try {
    const results = await UserService.searchUsers(userSearchQuery.value, 10);
    // 过滤掉当前用户
    availableUsers.value = results.filter(user => user.id !== authStore.user?.id);
  } catch (err: any) {
    console.warn('搜索用户失败:', err);
  }
};

// 表单验证
const validateForm = (): boolean => {
  // 清除之前的验证错误
  Object.keys(validationErrors).forEach(key => {
    delete validationErrors[key];
  });

  let isValid = true;

  // 验证聊天名称
  if (selectedType.value !== 'direct' && !formData.name.trim()) {
    validationErrors.name = '请输入聊天名称';
    isValid = false;
  } else if (selectedType.value !== 'direct' && formData.name.length < 2) {
    validationErrors.name = '聊天名称至少需要2个字符';
    isValid = false;
  }

  // 验证成员选择
  if (selectedType.value === 'direct' && selectedUsers.value.length !== 1) {
    validationErrors.members = '请选择一个聊天对象';
    isValid = false;
  }

  return isValid;
};

// 处理表单提交
const handleSubmit = async () => {
  if (!validateForm()) {
    return;
  }

  try {
    loading.value = true;
    error.value = '';

    let chatData: CreateChatRequest;

    if (selectedType.value === 'direct') {
      // 创建私信
      const chat = await ChatService.createDirectMessage(selectedUsers.value[0].id);
      notifySuccess('私信创建成功');
      emit('created', chat);
      close();
      return;
    }

    // 创建频道或群组
    chatData = {
      name: formData.name.trim(),
      chat_type: selectedType.value,
      is_public: selectedType.value === 'channel' ? formData.is_public : false,
      description: formData.description.trim() || undefined,
      member_ids: selectedUsers.value.map(user => user.id)
    };

    const chat = await ChatService.createChat(chatData);
    
    notifySuccess(
      `${selectedType.value === 'channel' ? '频道' : '群组'}创建成功`,
      `"${chat.name}" 已创建`
    );
    
    emit('created', chat);
    close();

  } catch (err: any) {
    error.value = err.message || '创建失败';
    notifyError('创建失败', err.message);
  } finally {
    loading.value = false;
  }
};

// 关闭模态框
const close = () => {
  // 重置表单
  formData.name = '';
  formData.description = '';
  formData.is_public = true;
  selectedUsers.value = [];
  userSearchQuery.value = '';
  error.value = '';
  
  // 清除验证错误
  Object.keys(validationErrors).forEach(key => {
    delete validationErrors[key];
  });

  emit('close');
};

// 处理覆盖层点击
const handleOverlayClick = (event: Event) => {
  if (event.target === event.currentTarget) {
    close();
  }
};

// 监听聊天类型变化
watch(selectedType, () => {
  // 重置表单和选择
  formData.name = '';
  formData.description = '';
  formData.is_public = true;
  selectedUsers.value = [];
  error.value = '';
  
  // 清除验证错误
  Object.keys(validationErrors).forEach(key => {
    delete validationErrors[key];
  });
});

// 加载用户列表
const loadUsers = async () => {
  try {
    const users = await UserService.searchUsers('', 50); // 获取前50个用户
    availableUsers.value = users.filter(user => user.id !== authStore.user?.id);
  } catch (err: any) {
    console.warn('加载用户列表失败:', err);
  }
};

// 组件挂载时加载用户列表
onMounted(() => {
  loadUsers();
});
</script>

<style scoped>
.modal-overlay {
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

.modal-container {
  background: white;
  border-radius: 12px;
  box-shadow: 0 25px 50px rgba(0, 0, 0, 0.25);
  width: 100%;
  max-width: 600px;
  max-height: 90vh;
  overflow-y: auto;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 24px 24px 0 24px;
  margin-bottom: 24px;
}

.modal-title {
  font-size: 20px;
  font-weight: 700;
  color: #1f2937;
}

.modal-close-btn {
  padding: 8px;
  border: none;
  background: none;
  color: #6b7280;
  cursor: pointer;
  border-radius: 6px;
  transition: background-color 0.2s;
}

.modal-close-btn:hover {
  background-color: #f3f4f6;
}

.close-icon {
  width: 20px;
  height: 20px;
}

.modal-content {
  padding: 0 24px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 24px;
  border-top: 1px solid #f3f4f6;
  margin-top: 24px;
}

/* 聊天类型选择 */
.chat-type-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 24px;
  border-radius: 8px;
  background-color: #f3f4f6;
  padding: 4px;
}

.chat-type-tab {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px 16px;
  border: none;
  background: none;
  color: #6b7280;
  font-size: 14px;
  font-weight: 500;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
}

.chat-type-tab.active {
  background-color: white;
  color: #6366f1;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.tab-icon {
  width: 16px;
  height: 16px;
}

/* 错误警告样式 */
.error-alert {
  display: flex;
  align-items: center;
  padding: 12px;
  margin-bottom: 20px;
  background-color: #fef2f2;
  border: 1px solid #fecaca;
  border-radius: 8px;
  color: #dc2626;
}

.error-icon {
  width: 20px;
  height: 20px;
  margin-right: 8px;
  flex-shrink: 0;
}

/* 表单样式 */
.create-form {
  space-y: 20px;
}

.form-group {
  margin-bottom: 20px;
}

.form-label {
  display: block;
  font-size: 14px;
  font-weight: 600;
  color: #374151;
  margin-bottom: 6px;
}

.form-input, .form-textarea {
  width: 100%;
  padding: 12px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-size: 14px;
  transition: border-color 0.2s, box-shadow 0.2s;
}

.form-input:focus, .form-textarea:focus {
  outline: none;
  border-color: #6366f1;
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
}

.form-input-error {
  border-color: #dc2626;
}

.form-input-error:focus {
  border-color: #dc2626;
  box-shadow: 0 0 0 3px rgba(220, 38, 38, 0.1);
}

.form-error {
  font-size: 12px;
  color: #dc2626;
  margin-top: 4px;
}

.form-textarea {
  resize: vertical;
  min-height: 80px;
}

/* 复选框样式 */
.checkbox-group {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.form-checkbox {
  margin-top: 2px;
}

.checkbox-label {
  flex: 1;
}

.checkbox-text {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: #374151;
  margin-bottom: 2px;
}

.checkbox-description {
  display: block;
  font-size: 12px;
  color: #6b7280;
}

/* 用户搜索样式 */
.user-search {
  position: relative;
  margin-bottom: 16px;
}

.search-icon {
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
  width: 16px;
  height: 16px;
  color: #6b7280;
}

.user-search-results {
  max-height: 200px;
  overflow-y: auto;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  margin-bottom: 16px;
}

.user-search-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.user-search-item:hover {
  background-color: #f9fafb;
}

.user-search-item.selected {
  background-color: #eef2ff;
}

.user-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  flex-shrink: 0;
}

.avatar-image {
  width: 100%;
  height: 100%;
  border-radius: 50%;
  object-fit: cover;
}

.avatar-initials {
  width: 100%;
  height: 100%;
  border-radius: 50%;
  background: linear-gradient(45deg, #6366f1, #8b5cf6);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
}

.user-info {
  flex: 1;
  min-width: 0;
}

.user-name {
  font-size: 14px;
  font-weight: 500;
  color: #374151;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.user-email {
  font-size: 12px;
  color: #6b7280;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.selection-indicator {
  color: #10b981;
}

.check-icon {
  width: 16px;
  height: 16px;
}

/* 已选择用户样式 */
.selected-users {
  margin-bottom: 16px;
}

.selected-users-label {
  font-size: 12px;
  color: #6b7280;
  margin-bottom: 8px;
}

.selected-users-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.selected-user-tag {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  background-color: #eef2ff;
  color: #6366f1;
  border-radius: 16px;
  font-size: 12px;
  font-weight: 500;
}

.remove-user-btn {
  padding: 2px;
  border: none;
  background: none;
  color: #6366f1;
  cursor: pointer;
  border-radius: 50%;
  transition: background-color 0.2s;
}

.remove-user-btn:hover {
  background-color: rgba(99, 102, 241, 0.1);
}

.remove-icon {
  width: 12px;
  height: 12px;
}

/* 按钮样式 */
.btn {
  padding: 12px 20px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 100px;
}

.btn-primary {
  background-color: #6366f1;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background-color: #5856eb;
}

.btn-secondary {
  background-color: #f3f4f6;
  color: #374151;
}

.btn-secondary:hover:not(:disabled) {
  background-color: #e5e7eb;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.loading-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid transparent;
  border-top: 2px solid currentColor;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-right: 8px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }

/* 响应式设计 */
@media (max-width: 768px) {
  .modal-container {
    margin: 0;
    max-width: none;
    border-radius: 0;
    max-height: 100vh;
  }
  
  .modal-header,
  .modal-content,
  .modal-footer {
    padding-left: 16px;
    padding-right: 16px;
  }
  
  .chat-type-tabs {
    flex-direction: column;
  }
  
  .modal-footer {
    flex-direction: column;
  }
  
  .btn {
    width: 100%;
  }
</style>