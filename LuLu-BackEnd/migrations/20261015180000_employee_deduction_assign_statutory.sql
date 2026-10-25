-- Employee deduction assignment + per-rule override + statutory (BPJS/tax) per-employee rates.
-- Depends on deduction_templates (sibling task t_cdbc0ab8). Additive only.
CREATE TABLE IF NOT EXISTS employee_deductions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id INT NOT NULL REFERENCES employee(employee_id) ON DELETE CASCADE,
    template_id UUID REFERENCES deduction_templates(id) ON DELETE SET NULL,
    kind VARCHAR(20) NOT NULL CHECK (kind IN ('permit', 'sick', 'absent', 'late')),
    grace_minutes INT CHECK (grace_minutes IS NULL OR grace_minutes >= 0),
    fixed_amount BIGINT CHECK (fixed_amount IS NULL OR fixed_amount >= 0),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(employee_id, kind)
);

-- Per-employee statutory overrides. NULL percent = fall back to payroll_config company rate.
CREATE TABLE IF NOT EXISTS employee_statutory (
    employee_id INT PRIMARY KEY REFERENCES employee(employee_id) ON DELETE CASCADE,
    bpjs_jht_employee NUMERIC(5,3) CHECK (bpjs_jht_employee IS NULL OR (bpjs_jht_employee >= 0 AND bpjs_jht_employee <= 100)),
    bpjs_jp_employee NUMERIC(5,3) CHECK (bpjs_jp_employee IS NULL OR (bpjs_jp_employee >= 0 AND bpjs_jp_employee <= 100)),
    bpjs_kes_employee NUMERIC(5,3) CHECK (bpjs_kes_employee IS NULL OR (bpjs_kes_employee >= 0 AND bpjs_kes_employee <= 100)),
    pph21_borne_by_company BOOLEAN NOT NULL DEFAULT FALSE,
    ptkp_status VARCHAR(4) NOT NULL DEFAULT 'TK' CHECK (ptkp_status IN ('TK', 'K', 'K1', 'K2', 'K3', 'I0', 'I1')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
