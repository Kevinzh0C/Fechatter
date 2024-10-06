-- Add migration script here
-- Create enum for user status
CREATE TYPE user_status AS ENUM (
    'suspended',
    'active'
);

-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    fullname VARCHAR(50) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    -- hashed argon2 password, length is 97
    password_hash VARCHAR(97) NOT NULL,
    status user_status NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index for users for email
CREATE UNIQUE INDEX IF NOT EXISTS email_index ON users(email);

-- Create enum for chat types
CREATE TYPE chat_type AS ENUM (
    'single',
    'group',
    'private_channel',
    'public_channel'
);

-- Create chats table for chat chats
CREATE TABLE IF NOT EXISTS chats (
    id BIGSERIAL PRIMARY KEY,
    chat_name VARCHAR(128) NOT NULL UNIQUE,
    type chat_type NOT NULL,
    -- chat_members is a list of user ids
    chat_members BIGINT[] NOT NULL,
    description TEXT,
    created_by BIGINT REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create messages table for chat history
CREATE TABLE IF NOT EXISTS messages (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGSERIAL NOT NULL REFERENCES chats(id),
    sender_id BIGINT NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    image_url TEXT[],
    created_at timestamp DEFAULT CURRENT_TIMESTAMP
);

-- Create chat_members table for chat members
CREATE TABLE IF NOT EXISTS  chat_members_relation (
    chat_id BIGSERIAL NOT NULL REFERENCES chats(id),
    user_id BIGINT NOT NULL REFERENCES users(id),
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (chat_id, user_id)
);

-- Create index for messages for chat_id and created_at order by created_at desc
CREATE INDEX IF NOT EXISTS chat_id_created_at_index ON messages(chat_id, created_at DESC);

-- Create index for messages for send_id and created_at order by created_at desc
CREATE INDEX IF NOT EXISTS sender_id_index ON messages(sender_id, created_at DESC);


