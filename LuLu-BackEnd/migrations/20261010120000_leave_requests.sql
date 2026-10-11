CREATE TABLE IF NOT EXISTS leave_requests (
    leave_id SERIAL PRIMARY KEY,
    leave_employee_id INTEGER NOT NULL REFERENCES employee(employee_id) ON DELETE CASCADE,
    leave_date DATE NOT NULL,
    leave_kind TEXT NOT NULL DEFAULT 'izin',
    leave_reason TEXT NOT NULL DEFAULT '',
    leave_status TEXT NOT NULL DEFAULT 'pending',
    leave_source TEXT NOT NULL DEFAULT 'manual',
    request_by VARCHAR(30) NOT NULL DEFAULT '',
    decided_by VARCHAR(30) NOT NULL DEFAULT '',
    create_at BIGINT NOT NULL DEFAULT date_part('epoch'::text, CURRENT_TIMESTAMP)::integer,
    update_at BIGINT NOT NULL DEFAULT date_part('epoch'::text, CURRENT_TIMESTAMP)::integer,
    -- One request per employee per day: a second row would silently double the
    -- deduction and make "who was absent" ambiguous.
    UNIQUE (leave_employee_id, leave_date)
);

-- Report windows are always a date range, so index the date first.
CREATE INDEX IF NOT EXISTS idx_leave_requests_date ON leave_requests(leave_date);
-- The report joins per employee while iterating a day range.
CREATE INDEX IF NOT EXISTS idx_leave_requests_employee_date
    ON leave_requests(leave_employee_id, leave_date);

-- Machine values stay English; the UI maps them to labels. CHECK rather than a
-- Postgres ENUM so adding a kind is a migration, not a type rewrite.
ALTER TABLE leave_requests DROP CONSTRAINT IF EXISTS leave_requests_kind_check;
ALTER TABLE leave_requests
    ADD CONSTRAINT leave_requests_kind_check
    CHECK (leave_kind IN ('izin', 'cuti', 'sakit'));

ALTER TABLE leave_requests DROP CONSTRAINT IF EXISTS leave_requests_status_check;
ALTER TABLE leave_requests
    ADD CONSTRAINT leave_requests_status_check
    CHECK (leave_status IN ('pending', 'approved', 'rejected'));

ALTER TABLE leave_requests DROP CONSTRAINT IF EXISTS leave_requests_source_check;
ALTER TABLE leave_requests
    ADD CONSTRAINT leave_requests_source_check
    CHECK (leave_source IN ('manual', 'import'));
-- Grant the new 'leave' page to the seed demo account, mirroring
-- 20261008000000_seed_demo_permissions.sql. Without this row the default-deny
-- gate in check_action_access() answers 403 and the demo account cannot reach
-- the page it ships with. Safe on a fresh install (guarded on the user
-- existing) and safe to re-run (ON CONFLICT DO NOTHING).
INSERT INTO user_page_permissions
    (user_nickname, page_key, can_view, can_add, can_edit, can_delete)
SELECT 'demo', p.page_key, true, true, true, true
FROM (VALUES ('leave')) AS p(page_key)
WHERE EXISTS (SELECT 1 FROM users WHERE user_nickname = 'demo')
ON CONFLICT (user_nickname, page_key) DO NOTHING;

INSERT INTO user_page_permissions
    (user_nickname, page_key, can_view, can_add, can_edit, can_delete)
SELECT user_nickname, 'leave', true, true, true, true
FROM user_page_permissions
WHERE page_key = 'settings' AND can_view
GROUP BY user_nickname
ON CONFLICT (user_nickname, page_key) DO NOTHING;
