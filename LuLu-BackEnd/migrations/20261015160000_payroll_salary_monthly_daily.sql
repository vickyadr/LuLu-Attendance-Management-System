-- Payroll reconcept: salary schema monthly/daily with manual rates.
-- Adds salary_type (monthly|daily), drops division formulas (old /20, /173
-- code removed in handler). monthly_salary kept for monthly type reference.
-- daily_salary kept as canonical per-day rate column (used by generate
-- fallback joins) and is now written from manual daily_rate input.
-- hourly_rate now fully manual input (used by OT below-threshold calc).
-- Effective_date versioning unchanged: UNIQUE(employee_id, effective_date).

ALTER TABLE salaries ADD COLUMN IF NOT EXISTS salary_type VARCHAR(20) NOT NULL DEFAULT 'monthly';
ALTER TABLE salaries ADD CONSTRAINT salaries_salary_type_check
    CHECK (salary_type IN ('monthly', 'daily'));

-- Backfill: existing rows are monthly by construction (old code derived
-- daily/hourly from monthly_salary via /20 and /173).
UPDATE salaries SET salary_type = 'monthly' WHERE salary_type IS NULL OR salary_type NOT IN ('monthly', 'daily');
