-- Deduction Templates (per-kind absence/late penalty rules)
-- kinds: permit | sick | absent | late
-- policy: standard | strict | lenient | custom
-- grace_minutes: late tolerance window (only meaningful for kind='late')
-- fixed_amount: flat deduction in IDR applied per incident
CREATE TABLE IF NOT EXISTS deduction_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    kind VARCHAR(20) NOT NULL CHECK (kind IN ('permit', 'sick', 'absent', 'late')),
    policy VARCHAR(20) NOT NULL DEFAULT 'standard' CHECK (policy IN ('standard', 'strict', 'lenient', 'custom')),
    grace_minutes INT NOT NULL DEFAULT 0 CHECK (grace_minutes >= 0),
    fixed_amount BIGINT NOT NULL DEFAULT 0 CHECK (fixed_amount >= 0),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
