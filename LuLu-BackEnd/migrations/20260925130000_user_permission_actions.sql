-- Replace the single per-page `allowed` flag with four explicit actions.
--
-- Level used to mean one of two things only: ">= 100 bypasses everything" and
-- "the Users page is admin-only". Page permissions cover the first job, and the
-- second falls out of having the 'users' page key, so the numeric level is gone.
-- Every action is stored separately and defaults to deny: a row that only has
-- can_view set must not silently grant write access.
ALTER TABLE user_page_permissions
    ADD COLUMN IF NOT EXISTS can_view BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS can_add BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS can_edit BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS can_delete BOOLEAN NOT NULL DEFAULT false;

-- Backfill: the old boolean was all-or-nothing, so an allowed page used to be
-- fully usable. The write actions follow it only when the row really was allowed,
-- so a denied page stays denied on every action. Guarded because this same file
-- drops the column it reads, so a re-run must not mention it unconditionally.
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

-- Level 100 users had no permission rows at all (they bypassed the table), so
-- they would silently lose every page once the bypass is gone. Give them full
-- access on every known page before the column disappears.
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

-- The old flag has carried its meaning into the four columns above, so it goes now.
ALTER TABLE user_page_permissions
    DROP COLUMN IF EXISTS allowed;
