ALTER TABLE user_page_permissions
    ADD COLUMN IF NOT EXISTS can_view BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS can_add BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS can_edit BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS can_delete BOOLEAN NOT NULL DEFAULT false;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = current_schema()
          AND table_name = 'user_page_permissions'
          AND column_name = 'allowed'
    ) THEN
        UPDATE user_page_permissions
        SET can_view = allowed,
            can_add = allowed,
            can_edit = allowed,
            can_delete = allowed;
    END IF;
END
$$;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = current_schema()
          AND table_name = 'users'
          AND column_name = 'user_level'
    ) THEN
        INSERT INTO user_page_permissions (user_nickname, page_key, can_view, can_add, can_edit, can_delete)
        SELECT user_nickname, p.page_key, true, true, true, true
        FROM users
        CROSS JOIN (VALUES ('dashboard'), ('employee'), ('shift'), ('report'),
                           ('controller'), ('settings'), ('users')) AS p(page_key)
        WHERE user_level = 100
        ON CONFLICT (user_nickname, page_key) DO NOTHING;
    END IF;
END
$$;

ALTER TABLE user_page_permissions
    DROP COLUMN IF EXISTS allowed;
