// Fechatter API Types
// 基于 fechatter_server/lib.rs 路由分析自动生成
// Generated at: 2025-06-10T18:00:00.000Z

// ========================================
// 通用响应类型
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
// 认证相关类型
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
// 用户档案相关类型
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
// 聊天相关类型
// ========================================
export interface Chat {
  id: number;
  chat_name: string; // 数据库字段名是 chat_name
  type: string; // 数据库中没有严格限制类型
  description?: string;
  created_by: number;
  created_at: string;
  updated_at: string;
  workspace_id: number;
  max_members?: number;
  is_public?: boolean;
  invite_code?: string;
  settings?: Record<string, any>; // 数据库中是 jsonb 类型
  chat_members: number[]; // 数据库中是 bigint[] 类型
  
  // 兼容字段（为了向后兼容，前端组件可能仍在使用）
  name?: string; // 映射到 chat_name
  chat_type?: 'direct' | 'group' | 'channel'; // 映射到 type
  
  // 关联数据（JOIN查询时包含）
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
  chat_type: 'direct' | 'group' | 'channel';
  is_public: boolean;
  description?: string;
  member_ids?: number[];
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
// 消息相关类型
// ========================================
export interface ChatMessage {
  id: number;
  chat_id: number;
  sender_id: number;
  content?: string;
  files: string[]; // 数据库中是 text[] 类型
  created_at: string;
  updated_at: string;
  idempotency_key?: string;
  status: 'sent' | 'delivered' | 'read' | 'failed';
  delivered_at?: string;
  read_at?: string;
  reply_to?: number; // 数据库中字段名是 reply_to，不是 reply_to_id
  thread_id?: number;
  mentions: number[]; // 数据库中是 bigint[] 类型
  priority: 'low' | 'normal' | 'high' | 'urgent';
  is_important: boolean;
  scheduled_for?: string;
  is_scheduled: boolean;
  is_edited: boolean;
  edit_count: number;
  read_by_users: number[]; // 数据库中是 bigint[] 类型
  sequence_number: number;
  
  // 关联数据（JOIN查询时包含）
  sender?: {
    id: number;
    fullname: string;
    email: string;
    avatar_url?: string;
    status: string;
    title?: string;
    department?: string;
  };
  reactions?: MessageReaction[];
  reply_to_message?: ChatMessage; // 被回复的消息
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
  
  // 聚合数据（从数据库查询聚合得出）
  count?: number;
  users?: number[];
  user_names?: string[];
}

export interface SendMessageRequest {
  content?: string;
  files?: File[] | string[]; // 可以是文件对象或文件路径
  reply_to?: number; // 数据库字段名是 reply_to
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
// 实时功能类型
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
// 搜索相关类型
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
// 文件相关类型
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
// 缓存和系统管理类型
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
// 分页类型
// ========================================
export interface PaginationParams {
  page?: number;
  limit?: number;
  offset?: number;
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
  };
}

// ========================================
// WebSocket/SSE 事件类型
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
// 错误类型
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
// 导出所有类型
// ========================================
export type {
  // 向后兼容的旧类型名称
  _signin_Request as SigninRequestLegacy,
  _signin_Response as SigninResponseLegacy,
  _signup_Request as SignupRequestLegacy,
  _signup_Response as SignupResponseLegacy,
  _chat_GET_Response as ChatListResponseLegacy,
  _chat_POST_Request as CreateChatRequestLegacy,
  _chat_POST_Response as CreateChatResponseLegacy,
  _chat_id_messages_GET_Response as MessageListResponseLegacy,
  _chat_id_messages_POST_Request as SendMessageRequestLegacy,
  _chat_id_messages_POST_Response as SendMessageResponseLegacy,
};