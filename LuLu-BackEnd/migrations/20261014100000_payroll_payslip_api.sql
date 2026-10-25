CREATE TABLE IF NOT EXISTS payslip_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payslip_id UUID NOT NULL REFERENCES payslips(id) ON DELETE CASCADE,
    work_date DATE NOT NULL,
    days_credited REAL NOT NULL DEFAULT 0 CHECK (days_credited >= 0),
    ot_hours REAL NOT NULL DEFAULT 0 CHECK (ot_hours >= 0),
    daily_pay BIGINT NOT NULL DEFAULT 0 CHECK (daily_pay >= 0),
    ot_pay BIGINT NOT NULL DEFAULT 0 CHECK (ot_pay >= 0),
    holiday_premium BIGINT NOT NULL DEFAULT 0 CHECK (holiday_premium >= 0),
    UNIQUE (payslip_id, work_date)
);

ALTER TABLE payslips
    ADD COLUMN IF NOT EXISTS created_by_user_nickname VARCHAR(30) REFERENCES users(user_nickname) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS locked_by_user_nickname VARCHAR(30) REFERENCES users(user_nickname) ON DELETE SET NULL;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'payslips_calendar_month_check'
    ) THEN
        ALTER TABLE payslips DROP CONSTRAINT payslips_calendar_month_check;
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'payslips_status_check'
    ) THEN
        ALTER TABLE payslips ADD CONSTRAINT payslips_status_check
            CHECK (status IN ('draft', 'locked', 'exported'));
    END IF;
END
$$;

CREATE INDEX IF NOT EXISTS idx_payslip_lines_payslip_id ON payslip_lines(payslip_id);

-- Payroll exposes compensation and tax data, so grant it only to settings editors.
-- Demo is included by the settings permission seed on fresh installs; report-only accounts stay denied.
INSERT INTO user_page_permissions
    (user_nickname, page_key, can_view, can_add, can_edit, can_delete)
SELECT user_nickname, 'payroll', true, true, true, true
FROM user_page_permissions
WHERE page_key = 'settings' AND can_edit
GROUP BY user_nickname
ON CONFLICT (user_nickname, page_key) DO NOTHING;
