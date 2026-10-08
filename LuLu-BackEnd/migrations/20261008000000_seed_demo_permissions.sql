INSERT INTO user_page_permissions
    (user_nickname, page_key, can_view, can_add, can_edit, can_delete)
SELECT 'demo', p.page_key, true, true, true, true
FROM (VALUES ('dashboard'), ('employee'), ('shift'), ('report'),
             ('controller'), ('settings'), ('users')) AS p(page_key)
WHERE EXISTS (SELECT 1 FROM users WHERE user_nickname = 'demo')
ON CONFLICT (user_nickname, page_key) DO NOTHING;
