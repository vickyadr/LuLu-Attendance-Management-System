-- Payroll reconcept: company config OT holiday carry
-- Adds OT threshold/mode/tiers, holiday pay basis, unapproved-carry policy.
ALTER TABLE payroll_config
    ADD COLUMN IF NOT EXISTS ot_day_threshold_hours INT NOT NULL DEFAULT 8,
    ADD COLUMN IF NOT EXISTS ot_below_mode VARCHAR(10) NOT NULL DEFAULT 'flat',
    ADD COLUMN IF NOT EXISTS ot_tiers_json JSONB NOT NULL DEFAULT '[{"up_to_hours":4,"multiplier":1.5},{"up_to_hours":null,"multiplier":2.0}]',
    ADD COLUMN IF NOT EXISTS holiday_multiplier NUMERIC(3,2) NOT NULL DEFAULT 2.0,
    ADD COLUMN IF NOT EXISTS holiday_basis VARCHAR(10) NOT NULL DEFAULT 'hourly',
    ADD COLUMN IF NOT EXISTS carry_unapproved VARCHAR(10) NOT NULL DEFAULT 'next';

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'payroll_config_ot_below_mode_check') THEN
        ALTER TABLE payroll_config ADD CONSTRAINT payroll_config_ot_below_mode_check
            CHECK (ot_below_mode IN ('flat', 'tiered'));
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'payroll_config_holiday_basis_check') THEN
        ALTER TABLE payroll_config ADD CONSTRAINT payroll_config_holiday_basis_check
            CHECK (holiday_basis IN ('hourly', 'daily'));
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'payroll_config_carry_unapproved_check') THEN
        ALTER TABLE payroll_config ADD CONSTRAINT payroll_config_carry_unapproved_check
            CHECK (carry_unapproved IN ('never', 'next', 'backpay'));
    END IF;
END
$$;
