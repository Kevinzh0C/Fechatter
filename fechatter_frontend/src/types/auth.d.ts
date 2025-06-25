/**
 * Auth Types
 * 认证相关类型定义
 */

export interface User {
  id: number;
  email: string;
  fullname: string;
  status: 'Active' | 'Inactive' | 'Suspended';
  created_at: string;
  updated_at?: string;
  workspace_id: number;
  avatar_url?: string;
  bio?: string;
  phone?: string;
  timezone?: string;
  language?: string;
}

export interface Workspace {
  id: number;
  name: string;
  owner_id: number;
  created_at: string;
  updated_at?: string;
  description?: string;
  logo_url?: string;
  settings?: Record<string, any>;
}

export interface AuthTokens {
  accessToken: string;
  refreshToken: string;
  expiresAt: number;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface LoginResponse {
  user: User;
  tokens: {
    accessToken: string;
    refreshToken: string;
    expiresIn: number;
  };
  workspace: Workspace;
  loginTime: string;
}

export interface RegisterRequest {
  fullname: string;
  email: string;
  password: string;
  confirmPassword: string;
}

export interface AuthState {
  // 用户信息
  user: User | null;
  
  // 认证令牌
  tokens: AuthTokens;
  
  // 工作空间信息
  workspace: Workspace | null;
  
  // 状态标志
  isAuthenticated: boolean;
  isInitialized: boolean;
  isRefreshing: boolean;
  
  // UI状态
  loading: boolean;
  error: string | null;
  
  // 设置
  rememberMe: boolean;
  sessionTimeout: number | null;
  
  // 统计信息
  loginTime: string | null;
  lastActivity: string | null;
}

export interface AuthError extends Error {
  code: string;
  userMessage: string;
  validationErrors?: Record<string, string[]>;
}