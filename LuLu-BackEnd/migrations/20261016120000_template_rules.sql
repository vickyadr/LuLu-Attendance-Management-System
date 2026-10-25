-- Template rules (opsi A): template holds rules_json only, amounts stay per salary.
ALTER TABLE salary_templates ADD COLUMN IF NOT EXISTS rules_json JSONB NOT NULL DEFAULT '[]';
