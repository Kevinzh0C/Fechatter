// Fechatter API Types
// Auto-generated based on fechatter_server/lib.rs route analysis
// Generated at: 2025-06-10T18:00:00.000Z

// ========================================
// Common Response Types
// ========================================
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: ApiError;
  meta?: {
    request_id: string;
    timestamp: string;
    version: string;
    duration_ms?: number;
  };
}

export interface ApiError {
  code: string;
  message: string;
  details?: string;
  field?: string;
  stack?: string[];
  suggestion?: string;
  help_url?: string;
}

// ========================================
// Authentication Related Types
// ========================================
export interface SigninRequest {
  email: string;
  password: string;
  device_type?: string;
}

export interface SignupRequest {
  email: string;
  password: string;
  fullname: string;
  workspace_name?: string;
}

export interface RefreshTokenRequest {
  refresh_token: string;
}

export interface ChangePasswordRequest {
  current_password: string;
  new_password: string;
}

export interface AuthTokens {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
}

export interface AuthUser {
  id: number;
  fullname: string;
  email: string;
  status: 'Active' | 'Inactive' | 'Suspended';
  created_at: string;
  workspace_id: number;
  phone?: string;
  title?: string;
  department?: string;
  avatar_url?: string;
  bio?: string;
  timezone?: string;
  language?: string;
  last_active_at?: string;
}

export interface AuthWorkspace {
  id: number;
  name: string;
  owner_id: number;
  created_at: string;
}

export interface SigninResponse {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
  user: AuthUser;
  workspace: AuthWorkspace;
  login_time: string;
}

export interface SignupResponse {
  user: AuthUser;
  workspace: AuthWorkspace;
  message: string;
  email_verification_required: boolean;
  created_at: string;
}

export interface ChangePasswordResponse {
  message: string;
  success: boolean;
  changed_at: string;
  logout_other_sessions: boolean;
}

// ========================================
// User Profile Related Types
// ========================================
export interface UserSettings {
  email_notifications: boolean;
  push_notifications: boolean;
  desktop_notifications: boolean;
  notification_sound: string;
  show_online_status: boolean;
  auto_away: boolean;
  auto_away_minutes: number;
  theme: string;
  message_display: string;
}

export interface UserProfileResponse {
  id: number;
  fullname: string;
  email: string;
  status: string;
  created_at: string;
  workspace_id: number;
  phone?: string;
  title?: string;
  department?: string;
  avatar_url?: string;
  bio?: string;
  timezone?: string;
  language?: string;
  last_active_at?: string;
  settings?: UserSettings;
}

export interface UpdateUserProfileRequest {
  fullname?: string;
  email?: string;
  phone?: string;
  title?: string;
  department?: string;
  avatar_url?: string;
  bio?: string;
  timezone?: string;
  language?: string;
}

export interface ProfileUpdateResponse {
  success: boolean;
  message: string;
  updated_fields: string[];
  profile: UserProfileResponse;
}

// ========================================
// Chat Related Types
// ========================================
export interface Chat {
  id: number;
  chat_name: string; // Database field name is chat_name
  type: string; // No strict type constraints in database
  description?: string;
  created_by: number;
  created_at: string;
  updated_at: string;
  workspace_id: number;
  max_members?: number;
  is_public?: boolean;
  invite_code?: string;
  settings?: Record<string, any>; // JSONB type in database
  chat_members: number[]; // BIGINT[] type in database

  // Compatibility fields (for backwards compatibility, frontend components may still use these)
  name?: string; // Maps to chat_name
  chat_type?: 'direct' | 'group' | 'channel'; // Maps to type

  // Associated data (included in JOIN queries)
  last_message?: ChatMessage;
  unread_count?: number;
  member_count?: number;
  creator?: {
    id: number;
    fullname: string;
    email: string;
    avatar_url?: string;
  };
}

export interface ChatMember {
  id: number;
  chat_id: number;
  user_id: number;
  joined_at: string;
  left_at?: string;
  role: 'admin' | 'member';
  user: {
    id: number;
    fullname: string;
    email: string;
    avatar_url?: string;
    status: string;
  };
}

export interface CreateChatRequest {
  name: string;
  chat_type: 'Single' | 'Group' | 'PrivateChannel' | 'PublicChannel'; // Match server ChatType enum
  members?: number[]; // Optional members list (matches server 'members' field)
  description?: string; // Optional description field
}

export interface UpdateChatRequest {
  name?: string;
  description?: string;
  is_public?: boolean;
}

export interface AddChatMembersRequest {
  user_ids: number[];
}

// ========================================
// Message Related Types
// ========================================
export interface ChatMessage {
  id: number | string; // Allow string for temp IDs
  temp_id?: string;
  chat_id: number;
  sender_id: number;
  sender?: User;
  content: string;
  files: UploadedFile[];
  mentions: User[];
  created_at: string;
  updated_at: string;
  priority: 'low' | 'normal' | 'high' | 'urgent';
  is_important: boolean;
  is_scheduled: boolean;
  scheduled_at?: string;
  message_type: 'text' | 'file' | 'system';
  reply_to?: ChatMessage;
  reactions?: any[]; // Replace with a strong type later
  status?: 'sending' | 'sent' | 'failed' | 'delivered' | 'read';
}

export interface UploadedFile {
  id: number | string; // Allow string for temp IDs
  filename: string;
  url: string;
  mime_type: string;
  size: number;
  created_at: string;
}

export interface MessageFile {
  id: number;
  filename: string;
  path: string;
  size: number;
  mime_type: string;
  url?: string;
}

export interface MessageReaction {
  id: number;
  message_id: number;
  emoji: string;
  created_at: string;

  // Aggregated data (derived from database queries)
  count?: number;
  users?: number[];
  user_names?: string[];
}

export interface SendMessageRequest {
  content?: string;
  files?: (File | string | number)[]; // Allow file objects, paths, or IDs
  reply_to?: number; // Database field name is reply_to
  mentions?: number[];
  idempotency_key?: string;
  priority?: 'low' | 'normal' | 'high' | 'urgent';
  is_important?: boolean;
  scheduled_for?: string; // ISO date string
}

export interface EditMessageRequest {
  content: string;
}

export interface MarkReadRequest {
  message_ids: number[];
}

export interface MessageSearchOptions {
  limit?: number;
  offset?: number;
  sort?: 'relevance' | 'date_desc' | 'date_asc';
}

export interface MessageSearchResult {
  hits: ChatMessage[];
  total: number;
  took_ms: number;
  page?: {
    offset: number;
    limit: number;
    has_more: boolean;
  };
}

export interface UnreadCountData {
  chat_id: number;
  unread_count: number;
}

export interface MessageReceipt {
  id: number;
  message_id: number;
  user_id: number;
  read_at: string;
  user: {
    id: number;
    fullname: string;
  };
}

// ========================================
// Real-time Feature Types
// ========================================
export interface PresenceUpdate {
  status: 'online' | 'away' | 'busy' | 'offline';
  last_seen?: string;
}

export interface TypingUser {
  user_id: number;
  fullname: string;
  started_at: string;
}

// ========================================
// Search Related Types
// ========================================
export interface SearchQuery {
  q: string;
  limit?: number;
  offset?: number;
  sort?: 'relevance' | 'date_desc' | 'date_asc';
}

export interface MessageSearchResult {
  message: ChatMessage;
  chat: {
    id: number;
    name: string;
  };
  score: number;
  highlights: string[];
}

export interface SearchResponse {
  hits: MessageSearchResult[];
  total: number;
  took_ms: number;
  query: string;
  page: {
    offset: number;
    limit: number;
    has_more: boolean;
  };
}

export interface SearchSuggestion {
  text: string;
  type: 'user' | 'channel' | 'keyword';
  count?: number;
}

// ========================================
// File Related Types
// ========================================
export interface FileUploadResponse {
  id: number;
  filename: string;
  path: string;
  size: number;
  mime_type: string;
  url: string;
  workspace_id: number;
  uploaded_by: number;
  uploaded_at: string;
}

export interface FileDownloadInfo {
  url: string;
  filename: string;
  size: number;
  mime_type: string;
  expires_at?: string;
}

// ========================================
// Cache and System Management Types
// ========================================
export interface CacheStats {
  redis_connected: boolean;
  total_keys: number;
  memory_used: string;
  hit_rate: number;
  operations_per_second: number;
  uptime: string;
}

export interface CacheConfig {
  enabled: boolean;
  ttl_seconds: number;
  max_memory: string;
  eviction_policy: string;
}

// ========================================
// Pagination Types
// ========================================
export interface PaginationParams {
  page?: number;
  limit?: number;
  offset?: number;
  before?: number; // For cursor-based pagination
}

export interface PaginatedResponse<T> {
  data: T[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    total_pages: number;
    has_next: boolean;
    has_prev: boolean;
    has_more?: boolean; // For cursor-based pagination
  };
}

// ========================================
// WebSocket/SSE Event Types
// ========================================
export interface RealtimeEvent {
  type: 'message' | 'typing' | 'presence' | 'chat_update' | 'member_update';
  data: any;
  timestamp: string;
  chat_id?: number;
  user_id?: number;
}

export interface MessageEvent extends RealtimeEvent {
  type: 'message';
  data: ChatMessage;
}

export interface TypingEvent extends RealtimeEvent {
  type: 'typing';
  data: {
    user_id: number;
    chat_id: number;
    is_typing: boolean;
  };
}

export interface PresenceEvent extends RealtimeEvent {
  type: 'presence';
  data: {
    user_id: number;
    status: 'online' | 'away' | 'busy' | 'offline';
    last_seen?: string;
  };
}

// ========================================
// Error Types
// ========================================
export interface ValidationError {
  field: string;
  message: string;
  code: string;
}

export interface ApiErrorResponse {
  success: false;
  error: {
    code: string;
    message: string;
    details?: string;
    field?: string;
    suggestion?: string;
    help_url?: string;
    validation_errors?: ValidationError[];
  };
  meta: {
    request_id: string;
    timestamp: string;
    version: string;
  };
}

// ========================================
// Model Interfaces
// ========================================

export interface User {
  id: number;
  fullname: string;
  email: string;
  avatar_url?: string;
  status: string;
  title?: string;
  department?: string;
  last_seen_at?: string;
}

// ========================================
// Export All Types
// ========================================
export type {
  // Keep an empty export to avoid syntax errors
};