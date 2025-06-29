<template>
  <div class="user-profile-edit">
    <!-- Page Header -->
    <div class="profile-header">
      <h2 class="profile-title">Edit Profile</h2>
      <p class="profile-subtitle">Update your personal information and preferences</p>
    </div>

    <!-- Error Alert -->
    <div v-if="error" class="error-alert">
      <svg class="error-icon" fill="currentColor" viewBox="0 0 20 20">
        <path fill-rule="evenodd"
          d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
          clip-rule="evenodd" />
      </svg>
      <span>{{ error }}</span>
    </div>

    <!-- Success Alert -->
    <div v-if="successMessage" class="success-alert">
      <svg class="success-icon" fill="currentColor" viewBox="0 0 20 20">
        <path fill-rule="evenodd"
          d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
          clip-rule="evenodd" />
      </svg>
      <span>{{ successMessage }}</span>
    </div>

    <!-- Avatar Upload -->
    <div class="avatar-section">
      <div class="avatar-container">
        <img :src="avatarUrl || defaultAvatarUrl" :alt="formData.fullname || 'User Avatar'" class="avatar" />
        <button @click="triggerAvatarUpload" class="avatar-upload-btn" :disabled="loading">
          <svg class="upload-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M15 13a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
          Upload Avatar
        </button>
        <input ref="avatarInput" type="file" accept="image/*" @change="handleAvatarUpload" class="hidden-file-input" />
      </div>
    </div>

    <!-- Profile Form -->
    <form @submit.prevent="handleSubmit" class="profile-form">
      <!-- Basic Information -->
      <div class="form-section">
        <h3 class="section-title">Basic Information</h3>

        <div class="form-grid">
          <!-- Full Name -->
          <div class="form-group">
            <label for="fullname" class="form-label">Full Name *</label>
            <input id="fullname" v-model="formData.fullname" type="text" class="form-input"
              :class="{ 'form-input-error': validationErrors.fullname }" placeholder="Enter your full name" required />
            <span v-if="validationErrors.fullname" class="form-error">
              {{ validationErrors.fullname }}
            </span>
          </div>

          <!-- Email -->
          <div class="form-group">
            <label for="email" class="form-label">Email *</label>
            <input id="email" v-model="formData.email" type="email" class="form-input"
              :class="{ 'form-input-error': validationErrors.email }" placeholder="Enter your email address" required />
            <span v-if="validationErrors.email" class="form-error">
              {{ validationErrors.email }}
            </span>
          </div>

          <!-- Phone -->
          <div class="form-group">
            <label for="phone" class="form-label">Phone</label>
            <input id="phone" v-model="formData.phone" type="tel" class="form-input" placeholder="Enter your phone number" />
          </div>

          <!-- Job Title -->
          <div class="form-group">
            <label for="title" class="form-label">Job Title</label>
            <input id="title" v-model="formData.title" type="text" class="form-input" placeholder="Enter your job title" />
          </div>

          <!-- Department -->
          <div class="form-group">
            <label for="department" class="form-label">Department</label>
            <input id="department" v-model="formData.department" type="text" class="form-input" placeholder="Enter your department" />
          </div>

          <!-- Timezone -->
          <div class="form-group">
            <label for="timezone" class="form-label">Timezone</label>
            <select id="timezone" v-model="formData.timezone" class="form-select">
              <option value="">Select timezone</option>
              <option value="Asia/Shanghai">Beijing Time (UTC+8)</option>
              <option value="Asia/Tokyo">Tokyo Time (UTC+9)</option>
              <option value="America/New_York">New York Time (UTC-5)</option>
              <option value="America/Los_Angeles">Los Angeles Time (UTC-8)</option>
              <option value="Europe/London">London Time (UTC+0)</option>
              <option value="Europe/Paris">Paris Time (UTC+1)</option>
            </select>
          </div>

          <!-- Language -->
          <div class="form-group">
            <label for="language" class="form-label">Language</label>
            <select id="language" v-model="formData.language" class="form-select">
              <option value="">Select language</option>
              <option value="zh-CN">Simplified Chinese</option>
              <option value="zh-TW">Traditional Chinese</option>
              <option value="en">English</option>
              <option value="ja">日本語</option>
              <option value="ko">한국어</option>
            </select>
          </div>
        </div>
      </div>

      <!-- Bio Section -->
      <div class="form-section">
        <h3 class="section-title">Bio</h3>

        <div class="form-group">
          <label for="bio" class="form-label">Bio</label>
          <textarea id="bio" v-model="formData.bio" class="form-textarea" rows="4" maxlength="500"
            placeholder="Tell us about yourself..." />
          <div class="character-count">
            {{ (formData.bio || '').length }}/500
          </div>
        </div>
      </div>

      <!-- Form Actions -->
      <div class="form-actions">
        <button type="button" @click="handleCancel" class="btn btn-secondary" :disabled="loading">
          Cancel
        </button>

        <button type="submit" class="btn btn-primary" :disabled="loading || !hasChanges">
          <span v-if="loading" class="loading-spinner" />
          {{ loading ? 'Saving...' : 'Save' }}
        </button>
      </div>
    </form>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import { useToast } from '@/composables/useToast';
import UserService from '@/services/UserService';
import type { UserProfileResponse, UpdateUserProfileRequest } from '@/types/api';

// Props
interface Props {
  userId?: number; // If provided, edit specified user; otherwise edit current user
}

const props = withDefaults(defineProps<Props>(), {
  userId: undefined
});

// Composables
const router = useRouter();
const authStore = useAuthStore();
const { notifySuccess, notifyError } = useToast();

// State
const loading = ref(false);
const error = ref('');
const successMessage = ref('');
const originalData = ref<UserProfileResponse | null>(null);
const avatarInput = ref<HTMLInputElement>();

// Form data
const formData = reactive<UpdateUserProfileRequest & { email: string }>({
  fullname: '',
  email: '',
  phone: '',
  title: '',
  department: '',
  avatar_url: '',
  bio: '',
  timezone: '',
  language: ''
});

// Validation errors
const validationErrors = reactive<Record<string, string>>({});

// Avatar URL
const avatarUrl = computed(() => formData.avatar_url);
const defaultAvatarUrl = computed(() => {
  const name = formData.fullname || formData.email || '?';
  const initials = name.split(' ').map(n => n.charAt(0)).join('').toUpperCase().slice(0, 2);
  return `https://ui-avatars.com/api/?name=${encodeURIComponent(initials)}&background=6366f1&color=fff&size=128`;
});

// Check for changes
const hasChanges = computed(() => {
  if (!originalData.value) return false;

  return Object.keys(formData).some(key => {
    const formValue = (formData as any)[key] || '';
    const originalValue = (originalData.value as any)[key] || '';
    return formValue !== originalValue;
  });
});

// Form validation
const validateForm = (): boolean => {
  // Clear previous validation errors
  Object.keys(validationErrors).forEach(key => {
    delete validationErrors[key];
  });

  let isValid = true;

  // Validate name
  if (!formData.fullname?.trim()) {
    validationErrors.fullname = 'Name cannot be empty';
    isValid = false;
  } else if (formData.fullname.length < 2) {
    validationErrors.fullname = 'Name must be at least 2 characters';
    isValid = false;
  }

  // Validate email
  if (!formData.email?.trim()) {
    validationErrors.email = 'Email cannot be empty';
    isValid = false;
  } else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(formData.email)) {
    validationErrors.email = 'Email format is invalid';
    isValid = false;
  }

  // Validate phone (if provided)
  if (formData.phone && !/^[+]?[\d\s\-()]+$/.test(formData.phone)) {
    validationErrors.phone = 'Phone number format is invalid';
    isValid = false;
  }

  return isValid;
};

// Load user data
const loadUserProfile = async () => {
  try {
    loading.value = true;
    error.value = '';

    let profile: UserProfileResponse;

    if (props.userId) {
      // Load specified user profile
      profile = await UserService.getUserProfile(props.userId);
    } else {
      // Load current user profile
      profile = await UserService.getCurrentUserProfile();
    }

    // Save original data
    originalData.value = profile;

    // Populate form
    Object.assign(formData, {
      fullname: profile.fullname || '',
      email: profile.email || '',
      phone: profile.phone || '',
      title: profile.title || '',
      department: profile.department || '',
      avatar_url: profile.avatar_url || '',
      bio: profile.bio || '',
      timezone: profile.timezone || '',
      language: profile.language || ''
    });

  } catch (err: any) {
    error.value = err.message || 'Failed to load user profile';
    notifyError('Failed to load user profile', err.message);
  } finally {
    loading.value = false;
  }
};

// Handle avatar upload
const triggerAvatarUpload = () => {
  avatarInput.value?.click();
};

const handleAvatarUpload = async (event: Event) => {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];

  if (!file) return;

  // Validate file type
  if (!file.type.startsWith('image/')) {
    notifyError('Upload failed: Please select an image file');
    return;
  }

  // Validate file size (5MB)
  if (file.size > 5 * 1024 * 1024) {
    notifyError('Upload failed: Image file size cannot exceed 5MB');
    return;
  }

  try {
    loading.value = true;
    const avatarUrl = await UserService.uploadAvatar(file);
    formData.avatar_url = avatarUrl;
    notifySuccess('Avatar uploaded successfully');
  } catch (err: any) {
    notifyError('Avatar upload failed', err.message);
  } finally {
    loading.value = false;
  }
};

// Handle form submission
const handleSubmit = async () => {
  if (!validateForm()) {
    return;
  }

  try {
    loading.value = true;
    error.value = '';
    successMessage.value = '';

    // Build update request (exclude email, cannot be modified through this interface)
    const updateRequest: UpdateUserProfileRequest = {
      fullname: formData.fullname,
      phone: formData.phone || undefined,
      title: formData.title || undefined,
      department: formData.department || undefined,
      avatar_url: formData.avatar_url || undefined,
      bio: formData.bio || undefined,
      timezone: formData.timezone || undefined,
      language: formData.language || undefined
    };

    let result;
    if (props.userId) {
      // Update specified user profile
      result = await UserService.updateUserProfile(props.userId, updateRequest);
    } else {
      // Update current user profile
      result = await UserService.updateCurrentUserProfile(updateRequest);
    }

    // Update successful
    successMessage.value = result.message;
    notifySuccess(`Profile updated successfully - Updated: ${result.updated_fields.join(', ')}`);

    // If current user, update auth state
    if (!props.userId && authStore.user) {
      // authStore.user = { ...authStore.user, ...result.profile }; // TODO: Use setter method
    }

    // Update original data
    originalData.value = result.profile;

    // Clear success message after 3 seconds
    setTimeout(() => {
      successMessage.value = '';
    }, 3000);

  } catch (err: any) {
    error.value = err.message || 'Failed to update profile';
    notifyError('Update failed', err.message);
  } finally {
    loading.value = false;
  }
};

// Handle cancel
const handleCancel = () => {
  if (hasChanges.value) {
    if (confirm('You have unsaved changes. Are you sure you want to leave?')) {
      router.back();
    }
  } else {
    router.back();
  }
};

// Clear error messages when user types
watch([() => formData.fullname, () => formData.email], () => {
  if (error.value) {
    error.value = '';
  }
  if (successMessage.value) {
    successMessage.value = '';
  }
});

// Load data on component mount
onMounted(() => {
  loadUserProfile();
});
</script>

<style scoped>
.user-profile-edit {
  max-width: 800px;
  margin: 0 auto;
  padding: 24px;
  background: white;
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.profile-header {
  margin-bottom: 32px;
  text-align: center;
}

.profile-title {
  font-size: 28px;
  font-weight: 700;
  color: #1f2937;
  margin-bottom: 8px;
}

.profile-subtitle {
  font-size: 16px;
  color: #6b7280;
}

/* Alert styles */
.error-alert {
  display: flex;
  align-items: center;
  padding: 12px;
  margin-bottom: 24px;
  background-color: #fef2f2;
  border: 1px solid #fecaca;
  border-radius: 8px;
  color: #dc2626;
}

.success-alert {
  display: flex;
  align-items: center;
  padding: 12px;
  margin-bottom: 24px;
  background-color: #f0fdf4;
  border: 1px solid #bbf7d0;
  border-radius: 8px;
  color: #059669;
}

.error-icon,
.success-icon {
  width: 20px;
  height: 20px;
  margin-right: 8px;
  flex-shrink: 0;
}

/* Avatar styles */
.avatar-section {
  margin-bottom: 32px;
  text-align: center;
}

.avatar-container {
  display: inline-block;
  position: relative;
}

.avatar {
  width: 120px;
  height: 120px;
  border-radius: 50%;
  object-fit: cover;
  border: 4px solid #f3f4f6;
  margin-bottom: 16px;
}

.avatar-upload-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px 16px;
  background-color: #6366f1;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.avatar-upload-btn:hover:not(:disabled) {
  background-color: #5856eb;
}

.avatar-upload-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.upload-icon {
  width: 16px;
  height: 16px;
  margin-right: 8px;
}

.hidden-file-input {
  display: none;
}

/* Form styles */
.profile-form {
  space-y: 32px;
}

.form-section {
  margin-bottom: 32px;
}

.section-title {
  font-size: 18px;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 2px solid #f3f4f6;
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 20px;
}

.form-group {
  display: flex;
  flex-direction: column;
}

.form-label {
  font-size: 14px;
  font-weight: 600;
  color: #374151;
  margin-bottom: 6px;
}

.form-input,
.form-select,
.form-textarea {
  padding: 12px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-size: 14px;
  transition: border-color 0.2s, box-shadow 0.2s;
}

.form-input:focus,
.form-select:focus,
.form-textarea:focus {
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
  min-height: 100px;
}

.character-count {
  font-size: 12px;
  color: #6b7280;
  text-align: right;
  margin-top: 4px;
}

/* Button styles */
.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding-top: 24px;
  border-top: 1px solid #f3f4f6;
}

.btn {
  padding: 12px 24px;
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
}

/* Responsive design */
@media (max-width: 768px) {
  .user-profile-edit {
    padding: 16px;
  }

  .form-grid {
    grid-template-columns: 1fr;
  }

  .form-actions {
    flex-direction: column;
  }

  .btn {
    width: 100%;
  }
}
</style>