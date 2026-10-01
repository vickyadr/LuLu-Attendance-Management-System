-- Attendance rule settings (global, single row).
-- late_tolerance_sec    : IN must exceed shift start by more than this to count as late
-- overtime_tolerance_sec: OUT must exceed shift end by more than this to count as overtime
CREATE TABLE IF NOT EXISTS settings (
    setting_key TEXT PRIMARY KEY,
    setting_value BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO settings (setting_key, setting_value) VALUES
    ('late_tolerance_sec', 60),
    ('overtime_tolerance_sec', 60)
ON CONFLICT (setting_key) DO NOTHING;
