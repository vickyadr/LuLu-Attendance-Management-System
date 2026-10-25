-- Salary assign: dynamic rules + department snapshot support.
ALTER TABLE salaries ADD COLUMN IF NOT EXISTS department VARCHAR(100);
ALTER TABLE salaries ADD COLUMN IF NOT EXISTS rules_json JSONB NOT NULL DEFAULT '[]';
