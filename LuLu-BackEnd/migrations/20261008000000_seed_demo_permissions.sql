-- Grant full access to the seed demo account on a fresh install.
--
-- Migration order problem this fixes: the 20260925130000 backfill grants
-- full permissions to every user_level = 100 account, but on a fresh
-- install the users table is still EMPTY at that point (the seed inserts
-- demo later, in 20260927192103). Without this file, a clean install
-- creates a demo user with zero permission rows, and the default-deny gate
-- in check_action_access() locks that user out of every page.
--
-- On the dev database this file is a no-op: demo already has full rows
-- (granted live on 2026-10-08), so ON CONFLICT DO NOTHING fires on all
-- seven pages. Safe to run anywhere, safe to re-run.
INSERT INTO user_page_permissions
    (user_nickname, page_key, can_view, can_add, can_edit, can_delete)
SELECT 'demo', p.page_key, true, true, true, true
FROM (VALUES ('dashboard'), ('employee'), ('shift'), ('report'),
             ('controller'), ('settings'), ('users')) AS p(page_key)
WHERE EXISTS (SELECT 1 FROM users WHERE user_nickname = 'demo')
ON CONFLICT (user_nickname, page_key) DO NOTHING;
