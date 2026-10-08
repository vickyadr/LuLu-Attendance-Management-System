-- User page permissions: allow/deny per user per page
CREATE TABLE IF NOT EXISTS user_page_permissions (
    id SERIAL PRIMARY KEY,
    user_nickname VARCHAR(30) NOT NULL REFERENCES users(user_nickname) ON DELETE CASCADE,
    page_key VARCHAR(100) NOT NULL,
    allowed BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(user_nickname, page_key)
);

-- Index for faster lookups
CREATE INDEX IF NOT EXISTS idx_user_page_permissions_user_nickname ON user_page_permissions(user_nickname);
CREATE INDEX IF NOT EXISTS idx_user_page_permissions_page_key ON user_page_permissions(page_key);

ALTER TABLE user_page_permissions
    DROP CONSTRAINT IF EXISTS user_page_permissions_user_nickname_fkey;
ALTER TABLE user_page_permissions
    ADD CONSTRAINT user_page_permissions_user_nickname_fkey
    FOREIGN KEY (user_nickname) REFERENCES users(user_nickname)
    ON DELETE CASCADE ON UPDATE CASCADE;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = current_schema()
          AND table_name = 'users'
          AND column_name = 'user_level'
    )
    AND EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = current_schema()
          AND table_name = 'user_page_permissions'
          AND column_name = 'allowed'
    ) THEN
        INSERT INTO user_page_permissions (user_nickname, page_key, allowed)
        SELECT user_nickname, p.page_key, true
        FROM users
        CROSS JOIN (VALUES ('dashboard'), ('employee'), ('shift'), ('report'),
                           ('controller'), ('settings'), ('users')) AS p(page_key)
        WHERE user_level = 100
        ON CONFLICT DO NOTHING;
    END IF;
END
$$;
