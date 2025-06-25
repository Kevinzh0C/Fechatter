export type ChatType = 'Single' | 'Group' | 'PrivateChannel' | 'PublicChannel';

export interface Chat {
  id: string;
  name: string;
  chat_type: ChatType;
  description?: string;
  created_at: string;
  updated_at: string;
  unread_count?: number;
  last_message?: {
    content: string;
    created_at: string;
    sender_name: string;
  };
}

export interface User {
  id: string;
  email: string;
  fullname: string;
  created_at: string;
}

export interface Workspace {
  id: string;
  name: string;
  owner_id: string;
  created_at: string;
}