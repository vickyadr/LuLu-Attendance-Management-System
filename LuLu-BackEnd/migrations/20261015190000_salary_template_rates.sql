-- Template rate defaults for assign prefill (monthly/daily/hourly).
ALTER TABLE salary_templates ADD COLUMN IF NOT EXISTS monthly_rate BIGINT;
ALTER TABLE salary_templates ADD COLUMN IF NOT EXISTS daily_rate BIGINT;
ALTER TABLE salary_templates ADD COLUMN IF NOT EXISTS hourly_rate BIGINT;
