-- Daily absences: track pending lupa_absen (missed punch) approvals per employee per day
-- Operator marks a day as "lupa_absen" during payslip generation to signal:
--   "This person forgot to punch. I am recording it as a zero-credit day pending verification."
-- The row holds zero credit until approved/rejected via separate action.

CREATE TABLE IF NOT EXISTS daily_absences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id INT NOT NULL REFERENCES employee(employee_id) ON DELETE CASCADE,
    absence_date DATE NOT NULL,
    absence_reason VARCHAR(50) NOT NULL DEFAULT 'lupa_absen', -- lupa_absen | other
    status VARCHAR(30) NOT NULL DEFAULT 'pending_verification', -- pending_verification | approved | rejected
    recorded_by VARCHAR(30) NOT NULL, -- user who recorded it
    approved_by VARCHAR(30), -- user who approved/rejected
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(employee_id, absence_date)
);

CREATE INDEX IF NOT EXISTS idx_daily_absences_employee_id ON daily_absences(employee_id);
CREATE INDEX IF NOT EXISTS idx_daily_absences_date ON daily_absences(absence_date);
CREATE INDEX IF NOT EXISTS idx_daily_absences_status ON daily_absences(status);
