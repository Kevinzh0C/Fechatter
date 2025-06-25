<template>
  <div v-if="isOpen" class="modal-overlay" @click="handleOverlayClick">
    <div class="modal-container" @click.stop>
      <!-- 模态框头部 -->
      <div class="modal-header">
        <h3 class="modal-title">修改密码</h3>
        <button @click="close" class="modal-close-btn">
          <svg class="close-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- 模态框内容 -->
      <div class="modal-content">
        <!-- 错误提示 -->
        <div v-if="error" class="error-alert">
          <svg class="error-icon" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
          </svg>
          <span>{{ error }}</span>
        </div>

        <!-- 密码修改表单 -->
        <form @submit.prevent="handleSubmit" class="password-form">
          <!-- 当前密码 -->
          <div class="form-group">
            <label for="currentPassword" class="form-label">当前密码 *</label>
            <div class="password-input-container">
              <input
                id="currentPassword"
                v-model="formData.currentPassword"
                :type="showCurrentPassword ? 'text' : 'password'"
                class="form-input"
                :class="{ 'form-input-error': validationErrors.currentPassword }"
                placeholder="请输入当前密码"
                autocomplete="current-password"
                required
              />
              <button
                type="button"
                @click="showCurrentPassword = !showCurrentPassword"
                class="password-toggle-btn"
              >
                <svg v-if="showCurrentPassword" class="eye-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.878 9.878L8.464 8.464M9.878 9.878l4.242 4.242m0 0L18.536 15.536M14.12 14.12l4.242 4.242" />
                </svg>
                <svg v-else class="eye-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                </svg>
              </button>
            </div>
            <span v-if="validationErrors.currentPassword" class="form-error">
              {{ validationErrors.currentPassword }}
            </span>
          </div>

          <!-- 新密码 -->
          <div class="form-group">
            <label for="newPassword" class="form-label">新密码 *</label>
            <div class="password-input-container">
              <input
                id="newPassword"
                v-model="formData.newPassword"
                :type="showNewPassword ? 'text' : 'password'"
                class="form-input"
                :class="{ 'form-input-error': validationErrors.newPassword }"
                placeholder="请输入新密码"
                autocomplete="new-password"
                required
              />
              <button
                type="button"
                @click="showNewPassword = !showNewPassword"
                class="password-toggle-btn"
              >
                <svg v-if="showNewPassword" class="eye-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.878 9.878L8.464 8.464M9.878 9.878l4.242 4.242m0 0L18.536 15.536M14.12 14.12l4.242 4.242" />
                </svg>
                <svg v-else class="eye-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                </svg>
              </button>
            </div>
            <span v-if="validationErrors.newPassword" class="form-error">
              {{ validationErrors.newPassword }}
            </span>
            
            <!-- 密码强度指示器 -->
            <div v-if="formData.newPassword" class="password-strength">
              <div class="strength-label">密码强度:</div>
              <div class="strength-bar">
                <div 
                  class="strength-fill"
                  :class="passwordStrengthClass"
                  :style="{ width: passwordStrengthPercent + '%' }"
                />
              </div>
              <div class="strength-text" :class="passwordStrengthClass">
                {{ passwordStrengthText }}
              </div>
            </div>
          </div>

          <!-- 确认新密码 -->
          <div class="form-group">
            <label for="confirmPassword" class="form-label">确认新密码 *</label>
            <div class="password-input-container">
              <input
                id="confirmPassword"
                v-model="formData.confirmPassword"
                :type="showConfirmPassword ? 'text' : 'password'"
                class="form-input"
                :class="{ 'form-input-error': validationErrors.confirmPassword }"
                placeholder="请再次输入新密码"
                autocomplete="new-password"
                required
              />
              <button
                type="button"
                @click="showConfirmPassword = !showConfirmPassword"
                class="password-toggle-btn"
              >
                <svg v-if="showConfirmPassword" class="eye-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.878 9.878L8.464 8.464M9.878 9.878l4.242 4.242m0 0L18.536 15.536M14.12 14.12l4.242 4.242" />
                </svg>
                <svg v-else class="eye-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                </svg>
              </button>
            </div>
            <span v-if="validationErrors.confirmPassword" class="form-error">
              {{ validationErrors.confirmPassword }}
            </span>
          </div>

          <!-- 密码要求提示 -->
          <div class="password-requirements">
            <h4 class="requirements-title">密码要求:</h4>
            <ul class="requirements-list">
              <li :class="{ 'requirement-met': hasMinLength }">
                <svg class="requirement-icon" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
                至少8个字符
              </li>
              <li :class="{ 'requirement-met': hasUppercase }">
                <svg class="requirement-icon" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
                包含大写字母
              </li>
              <li :class="{ 'requirement-met': hasLowercase }">
                <svg class="requirement-icon" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
                包含小写字母
              </li>
              <li :class="{ 'requirement-met': hasNumber }">
                <svg class="requirement-icon" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
                包含数字
              </li>
              <li :class="{ 'requirement-met': hasSpecialChar }">
                <svg class="requirement-icon" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
                包含特殊字符
              </li>
            </ul>
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
          {{ loading ? '修改中...' : '修改密码' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue';
import { useToast } from '@/composables/useToast';
import UserService from '@/services/UserService';

// Props
interface Props {
  isOpen: boolean;
}

const props = defineProps<Props>();

// Emits
const emit = defineEmits<{
  close: [];
  success: [];
}>();

// Composables
const { notifySuccess, notifyError } = useToast();

// 状态
const loading = ref(false);
const error = ref('');

// 密码显示状态
const showCurrentPassword = ref(false);
const showNewPassword = ref(false);
const showConfirmPassword = ref(false);

// 表单数据
const formData = reactive({
  currentPassword: '',
  newPassword: '',
  confirmPassword: ''
});

// 验证错误
const validationErrors = reactive<Record<string, string>>({});

// 密码强度检查
const hasMinLength = computed(() => formData.newPassword.length >= 8);
const hasUppercase = computed(() => /[A-Z]/.test(formData.newPassword));
const hasLowercase = computed(() => /[a-z]/.test(formData.newPassword));
const hasNumber = computed(() => /\d/.test(formData.newPassword));
const hasSpecialChar = computed(() => /[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>\/?~`]/.test(formData.newPassword));

const passwordStrength = computed(() => {
  const checks = [hasMinLength.value, hasUppercase.value, hasLowercase.value, hasNumber.value, hasSpecialChar.value];
  return checks.filter(Boolean).length;
});

const passwordStrengthPercent = computed(() => (passwordStrength.value / 5) * 100);

const passwordStrengthClass = computed(() => {
  if (passwordStrength.value <= 2) return 'strength-weak';
  if (passwordStrength.value <= 3) return 'strength-medium';
  if (passwordStrength.value <= 4) return 'strength-good';
  return 'strength-strong';
});

const passwordStrengthText = computed(() => {
  if (passwordStrength.value <= 2) return '弱';
  if (passwordStrength.value <= 3) return '中等';
  if (passwordStrength.value <= 4) return '良好';
  return '强';
});

// 表单验证
const isFormValid = computed(() => {
  return formData.currentPassword.trim() !== '' &&
         formData.newPassword.trim() !== '' &&
         formData.confirmPassword.trim() !== '' &&
         formData.newPassword === formData.confirmPassword &&
         passwordStrength.value >= 3; // 至少中等强度
});

// 验证表单
const validateForm = (): boolean => {
  // 清除之前的验证错误
  Object.keys(validationErrors).forEach(key => {
    delete validationErrors[key];
  });

  let isValid = true;

  // 验证当前密码
  if (!formData.currentPassword.trim()) {
    validationErrors.currentPassword = '请输入当前密码';
    isValid = false;
  }

  // 验证新密码
  if (!formData.newPassword.trim()) {
    validationErrors.newPassword = '请输入新密码';
    isValid = false;
  } else if (formData.newPassword.length < 8) {
    validationErrors.newPassword = '新密码至少需要8个字符';
    isValid = false;
  } else if (passwordStrength.value < 3) {
    validationErrors.newPassword = '新密码强度太低，请满足更多要求';
    isValid = false;
  } else if (formData.newPassword === formData.currentPassword) {
    validationErrors.newPassword = '新密码不能与当前密码相同';
    isValid = false;
  }

  // 验证确认密码
  if (!formData.confirmPassword.trim()) {
    validationErrors.confirmPassword = '请确认新密码';
    isValid = false;
  } else if (formData.newPassword !== formData.confirmPassword) {
    validationErrors.confirmPassword = '两次输入的密码不一致';
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

    await UserService.changePassword(formData.currentPassword, formData.newPassword);

    notifySuccess('密码修改成功', '您的密码已经成功修改');
    emit('success');
    close();

  } catch (err: any) {
    error.value = err.message || '修改密码失败';
    notifyError('修改密码失败', err.message);
  } finally {
    loading.value = false;
  }
};

// 关闭模态框
const close = () => {
  // 重置表单
  formData.currentPassword = '';
  formData.newPassword = '';
  formData.confirmPassword = '';
  error.value = '';
  
  // 重置密码显示状态
  showCurrentPassword.value = false;
  showNewPassword.value = false;
  showConfirmPassword.value = false;
  
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

// 清除错误消息当用户输入时
watch([() => formData.currentPassword, () => formData.newPassword, () => formData.confirmPassword], () => {
  if (error.value) {
    error.value = '';
  }
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
  max-width: 500px;
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
.password-form {
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

.password-input-container {
  position: relative;
  display: flex;
  align-items: center;
}

.form-input {
  width: 100%;
  padding: 12px 44px 12px 12px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-size: 14px;
  transition: border-color 0.2s, box-shadow 0.2s;
}

.form-input:focus {
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

.password-toggle-btn {
  position: absolute;
  right: 12px;
  padding: 4px;
  border: none;
  background: none;
  color: #6b7280;
  cursor: pointer;
  border-radius: 4px;
  transition: color 0.2s;
}

.password-toggle-btn:hover {
  color: #374151;
}

.eye-icon {
  width: 18px;
  height: 18px;
}

.form-error {
  display: block;
  font-size: 12px;
  color: #dc2626;
  margin-top: 4px;
}

/* 密码强度样式 */
.password-strength {
  margin-top: 12px;
}

.strength-label {
  font-size: 12px;
  color: #6b7280;
  margin-bottom: 6px;
}

.strength-bar {
  height: 6px;
  background-color: #f3f4f6;
  border-radius: 3px;
  overflow: hidden;
  margin-bottom: 4px;
}

.strength-fill {
  height: 100%;
  transition: width 0.3s ease, background-color 0.3s ease;
  border-radius: 3px;
}

.strength-text {
  font-size: 12px;
  font-weight: 600;
}

.strength-weak .strength-fill,
.strength-weak {
  background-color: #dc2626;
  color: #dc2626;
}

.strength-medium .strength-fill,
.strength-medium {
  background-color: #f59e0b;
  color: #f59e0b;
}

.strength-good .strength-fill,
.strength-good {
  background-color: #3b82f6;
  color: #3b82f6;
}

.strength-strong .strength-fill,
.strength-strong {
  background-color: #10b981;
  color: #10b981;
}

/* 密码要求样式 */
.password-requirements {
  margin-top: 16px;
  padding: 16px;
  background-color: #f9fafb;
  border-radius: 8px;
}

.requirements-title {
  font-size: 14px;
  font-weight: 600;
  color: #374151;
  margin-bottom: 8px;
}

.requirements-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.requirements-list li {
  display: flex;
  align-items: center;
  font-size: 12px;
  color: #6b7280;
  margin-bottom: 4px;
  transition: color 0.2s;
}

.requirements-list li.requirement-met {
  color: #10b981;
}

.requirement-icon {
  width: 14px;
  height: 14px;
  margin-right: 6px;
  opacity: 0.3;
  transition: opacity 0.2s;
}

.requirement-met .requirement-icon {
  opacity: 1;
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
  
  .modal-footer {
    flex-direction: column;
  }
  
  .btn {
    width: 100%;
  }
}
</style>