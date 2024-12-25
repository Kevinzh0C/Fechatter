CREATE TABLE IF NOT EXISTS refresh_tokens (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    replaced_by VARCHAR(64),
    user_agent TEXT,
    ip_address VARCHAR(45),
    absolute_expires_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS refresh_token_hash_index ON refresh_tokens(token_hash);

CREATE INDEX IF NOT EXISTS refresh_token_user_id_index ON refresh_tokens(user_id);
