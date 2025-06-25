/**
 * User Service
 * 处理用户相关的API调用
 */
import api from './api';
import type {
  UserProfileResponse,
  UpdateUserProfileRequest,
  ProfileUpdateResponse,
  ChangePasswordRequest,
  ChangePasswordResponse,
  ApiResponse
} from '@/types/api';

class UserService {
  /**
   * 获取当前用户档案
   */
  async getCurrentUserProfile(): Promise<UserProfileResponse> {
    try {
      const response = await api.get<ApiResponse<UserProfileResponse>>('/users/profile');
      
      if (response.data.success && response.data.data) {
        return response.data.data;
      }
      
      throw new Error(response.data.error?.message || 'Failed to get user profile');
    } catch (error) {
      console.error('❌ [UserService] Get current user profile failed:', error);
      throw this.handleError(error);
    }
  }

  /**
   * 获取指定用户档案
   */
  async getUserProfile(userId: number): Promise<UserProfileResponse> {
    try {
      const response = await api.get<ApiResponse<UserProfileResponse>>(`/users/${userId}/profile`);
      
      if (response.data.success && response.data.data) {
        return response.data.data;
      }
      
      throw new Error(response.data.error?.message || 'Failed to get user profile');
    } catch (error) {
      console.error(`❌ [UserService] Get user profile failed for user ${userId}:`, error);
      throw this.handleError(error);
    }
  }

  /**
   * 更新当前用户档案
   */
  async updateCurrentUserProfile(updates: UpdateUserProfileRequest): Promise<ProfileUpdateResponse> {
    try {
      const response = await api.put<ApiResponse<ProfileUpdateResponse>>('/users/profile', updates);
      
      if (response.data.success && response.data.data) {
        return response.data.data;
      }
      
      throw new Error(response.data.error?.message || 'Failed to update user profile');
    } catch (error) {
      console.error('❌ [UserService] Update current user profile failed:', error);
      throw this.handleError(error);
    }
  }

  /**
   * 更新指定用户档案 (管理员功能)
   */
  async updateUserProfile(userId: number, updates: UpdateUserProfileRequest): Promise<ProfileUpdateResponse> {
    try {
      const response = await api.put<ApiResponse<ProfileUpdateResponse>>(`/users/${userId}/profile`, updates);
      
      if (response.data.success && response.data.data) {
        return response.data.data;
      }
      
      throw new Error(response.data.error?.message || 'Failed to update user profile');
    } catch (error) {
      console.error(`❌ [UserService] Update user profile failed for user ${userId}:`, error);
      throw this.handleError(error);
    }
  }

  /**
   * 修改密码
   */
  async changePassword(currentPassword: string, newPassword: string): Promise<ChangePasswordResponse> {
    try {
      const request: ChangePasswordRequest = {
        current_password: currentPassword,
        new_password: newPassword
      };

      const response = await api.post<ApiResponse<ChangePasswordResponse>>('/users/change-password', request);
      
      if (response.data.success && response.data.data) {
        return response.data.data;
      }
      
      throw new Error(response.data.error?.message || 'Failed to change password');
    } catch (error) {
      console.error('❌ [UserService] Change password failed:', error);
      throw this.handleError(error);
    }
  }

  /**
   * 上传用户头像
   */
  async uploadAvatar(file: File): Promise<string> {
    try {
      const formData = new FormData();
      formData.append('file', file);

      const response = await api.post<ApiResponse<{ url: string }>>('/files/single', formData, {
      });
      
      if (response.data.success && response.data.data) {
        return response.data.data.url;
      }
      
      throw new Error(response.data.error?.message || 'Failed to upload avatar');
    } catch (error) {
      console.error('❌ [UserService] Upload avatar failed:', error);
      throw this.handleError(error);
    }
  }

  /**
   * 验证用户名可用性
   */
  async checkUsernameAvailability(username: string): Promise<boolean> {
    try {
      // 注意：这个API端点需要在后端实现
      const response = await api.get<ApiResponse<{ available: boolean }>>(`/users/check-username?username=${encodeURIComponent(username)}`);
      
      if (response.data.success && response.data.data) {
        return response.data.data.available;
      }
      
      return false;
    } catch (error) {
      console.warn('⚠️ [UserService] Check username availability failed:', error);
      // 如果检查失败，假设用户名可用
      return true;
    }
  }

  /**
   * 验证邮箱可用性
   */
  async checkEmailAvailability(email: string): Promise<boolean> {
    try {
      // 注意：这个API端点需要在后端实现
      const response = await api.get<ApiResponse<{ available: boolean }>>(`/users/check-email?email=${encodeURIComponent(email)}`);
      
      if (response.data.success && response.data.data) {
        return response.data.data.available;
      }
      
      return false;
    } catch (error) {
      console.warn('⚠️ [UserService] Check email availability failed:', error);
      // 如果检查失败，假设邮箱可用
      return true;
    }
  }

  /**
   * 搜索用户
   */
  async searchUsers(query: string, limit: number = 10): Promise<UserProfileResponse[]> {
    try {
      // 注意：这个API端点需要在后端实现
      const response = await api.get<ApiResponse<UserProfileResponse[]>>(`/users/search?q=${encodeURIComponent(query)}&limit=${limit}`);
      
      if (response.data.success && response.data.data) {
        return response.data.data;
      }
      
      return [];
    } catch (error) {
      console.warn('⚠️ [UserService] Search users failed:', error);
      return [];
    }
  }

  /**
   * 错误处理
   */
  private handleError(error: any): Error {
    // 网络错误
    if (!error.response) {
      const networkError = new Error('Network error. Please check your connection.');
      (networkError as any).code = 'NETWORK_ERROR';
      return networkError;
    }

    const status = error.response?.status;
    const data = error.response?.data;

    // 创建统一的错误对象
    const userError = new Error();
    (userError as any).code = data?.error?.code || 'UNKNOWN_ERROR';
    (userError as any).status = status;
    (userError as any).originalError = error;

    // 根据状态码设置用户友好的错误信息
    switch (status) {
      case 400:
        userError.message = data?.error?.message || 'Invalid request. Please check your input.';
        break;

      case 401:
        userError.message = 'You are not authorized to perform this action. Please log in again.';
        break;

      case 403:
        userError.message = 'You do not have permission to perform this action.';
        break;

      case 404:
        userError.message = 'The requested user was not found.';
        break;

      case 422:
        userError.message = data?.error?.message || 'Validation failed. Please check your input.';
        (userError as any).validationErrors = data?.error?.validation_errors || [];
        break;

      case 429:
        userError.message = 'Too many requests. Please wait a moment and try again.';
        break;

      case 500:
        userError.message = 'Internal server error. Please try again later.';
        break;

      default:
        userError.message = data?.error?.message || 'An unexpected error occurred.';
    }

    return userError;
  }
}

// 导出单例
export default new UserService();