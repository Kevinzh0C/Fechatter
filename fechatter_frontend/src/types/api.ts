// Fechatter API Types
// 自动生成，请勿手动修改
// Generated at: 2025-05-27T23:34:16.270Z

// AUTH Types
export interface _signin_Request {
  email: string;
  password: string;
}

export interface _signin_Response {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  user: {
    id: number;
    email: string;
    fullname: string;
    workspace_id: number;
    status: string;
    created_at: string;
  };
}

export interface _signup_Request {
  fullname: string;
  email: string;
  password: string;
  workspace?: string;
}

export interface _signup_Response {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  user: {
    id: number;
    email: string;
    fullname: string;
    workspace_id: number;
    status: string;
    created_at: string;
  };
}


// CHAT Types
export type _chat_GET_Response = Array<{
    id: number;
    name: string;
    chat_type: string;
    is_public: boolean;
    created_by: number;
    workspace_id: number;
    member_count: number;
    last_message: {
    id: number;
    content: string;
    created_at: string;
    sender: {
    id: number;
    fullname: string;
  };
  };
  }>;

export interface _chat_POST_Request {
  name: string;
  chat_type: string;
  is_public: boolean;
  workspace_id: number;
}

export interface _chat_POST_Response {
  id: number;
  name: string;
  chat_type: string;
  is_public: boolean;
  created_by: number;
  workspace_id: number;
  created_at: string;
}

export type _chat_id_messages_GET_Response = Array<{
    id: number;
    content: string;
    message_type: string;
    chat_id: number;
    sender_id: number;
    created_at: string;
    sender: {
    id: number;
    fullname: string;
  };
    files: Array<{
    path: string;
    filename: string;
    size: number;
    mime_type: string;
  }>;
  }>;

export interface _chat_id_messages_POST_Request {
  content: string;
  message_type: string;
  files?: any[];
}

export interface _chat_id_messages_POST_Response {
  id: number;
  content: string;
  message_type: string;
  chat_id: number;
  sender_id: number;
  created_at: string;
}


