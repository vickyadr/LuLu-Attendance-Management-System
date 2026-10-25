-- Employee PWA accounts are rows in `users` like any admin, because that is
-- where credentials live. This flag separates the two: the admin login refuses
-- a field-only account, and session checks refuse its token on admin pages.
-- Existing field accounts are exactly the ones an employee row points at.
ALTER TABLE users ADD COLUMN IF NOT EXISTS user_field_only BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE users u
SET user_field_only = TRUE
WHERE EXISTS (
    SELECT 1 FROM employee e WHERE e.employee_user_nickname = u.user_nickname
);
