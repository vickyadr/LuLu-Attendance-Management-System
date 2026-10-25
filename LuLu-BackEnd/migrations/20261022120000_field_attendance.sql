-- Field attendance: employees who spend the day outside the office send a
-- photo + GPS + timestamp per event; an admin approves the day before it
-- counts. No assignment/trip table on purpose: the employee declares, the
-- admin decides.

ALTER TABLE employee
    ADD COLUMN IF NOT EXISTS employee_user_nickname VARCHAR(30)
        REFERENCES users(user_nickname) ON UPDATE CASCADE ON DELETE SET NULL;

CREATE UNIQUE INDEX IF NOT EXISTS employee_user_nickname_key
    ON employee(employee_user_nickname)
    WHERE employee_user_nickname IS NOT NULL;

-- Retention is a knob, not code: photos older than this are removed.
INSERT INTO settings (setting_key, setting_value)
VALUES ('field_photo_retention_days', 60)
ON CONFLICT (setting_key) DO NOTHING;

CREATE TABLE IF NOT EXISTS field_attendance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id INT NOT NULL REFERENCES employee(employee_id) ON DELETE CASCADE,
    work_date DATE NOT NULL, -- office timezone, never the device's
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    note TEXT,
    reviewed_by VARCHAR(30) REFERENCES users(user_nickname) ON DELETE SET NULL,
    reviewed_at TIMESTAMPTZ,
    review_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT field_attendance_status CHECK (status IN ('pending', 'approved', 'rejected')),
    CONSTRAINT field_attendance_day UNIQUE (employee_id, work_date)
);

CREATE INDEX IF NOT EXISTS idx_field_attendance_status ON field_attendance(status, work_date);

CREATE TABLE IF NOT EXISTS field_photo (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    attendance_id UUID NOT NULL REFERENCES field_attendance(id) ON DELETE CASCADE,
    kind VARCHAR(10) NOT NULL,
    file_path TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    bytes INT NOT NULL,
    lat DOUBLE PRECISION,
    lng DOUBLE PRECISION,
    accuracy_m REAL,
    tz_offset_minutes INT,
    device_at TIMESTAMPTZ,
    server_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT field_photo_kind CHECK (kind IN ('in', 'out'))
);

CREATE INDEX IF NOT EXISTS idx_field_photo_attendance ON field_photo(attendance_id, server_at);
CREATE INDEX IF NOT EXISTS idx_field_photo_server_at ON field_photo(server_at);
