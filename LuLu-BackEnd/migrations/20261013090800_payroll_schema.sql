-- Payroll Configuration (per company/tenant, single row)
CREATE TABLE IF NOT EXISTS payroll_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_week_days INT NOT NULL DEFAULT 5, -- 5 or 6
    daily_work_hours INT NOT NULL DEFAULT 8, -- 7 or 8
    ot_daily_cap_hours INT NOT NULL DEFAULT 4, -- PP 35/2021 fixed: 4
    ot_weekly_cap_hours INT NOT NULL DEFAULT 18, -- PP 35/2021 fixed: 18
    ot_rate_first_hour DECIMAL(3,2) NOT NULL DEFAULT 1.5, -- 1.5x hourly
    ot_rate_subsequent DECIMAL(3,2) NOT NULL DEFAULT 2.0, -- 2.0x hourly
    rest_day_multiplier DECIMAL(3,2) NOT NULL DEFAULT 1.5, -- 1.5x day credit
    bpjs_jkk_rate_min DECIMAL(5,3) NOT NULL DEFAULT 0.24, -- Work Accident (min)
    bpjs_jkk_rate_max DECIMAL(5,3) NOT NULL DEFAULT 1.74, -- Work Accident (max)
    bpjs_jkm_rate DECIMAL(5,3) NOT NULL DEFAULT 0.30, -- Death Benefit
    bpjs_jht_employer DECIMAL(5,3) NOT NULL DEFAULT 3.70, -- Old-age Employer
    bpjs_jht_employee DECIMAL(5,3) NOT NULL DEFAULT 2.00, -- Old-age Employee
    bpjs_jp_employer DECIMAL(5,3) NOT NULL DEFAULT 1.0, -- Pension Employer
    bpjs_jp_employee DECIMAL(5,3) NOT NULL DEFAULT 1.0, -- Pension Employee
    bpjs_kes_employer DECIMAL(5,3) NOT NULL DEFAULT 4.0, -- Health Employer
    bpjs_kes_employee DECIMAL(5,3) NOT NULL DEFAULT 1.0, -- Health Employee
    bpjs_kes_wage_cap BIGINT NOT NULL DEFAULT 12000000, -- Wage cap for health
    meal_ot_threshold_hours INT NOT NULL DEFAULT 3, -- Provide meal if OT > 3 hrs
    meal_calories INT NOT NULL DEFAULT 1400, -- Minimum 1400 kcal
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO payroll_config (work_week_days, daily_work_hours) 
VALUES (5, 8)
ON CONFLICT DO NOTHING;

-- Salary Templates (reusable structures)
CREATE TABLE IF NOT EXISTS salary_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    basic_salary_type VARCHAR(20) NOT NULL DEFAULT 'monthly', -- monthly | hourly | daily
    hra_percent DECIMAL(5,2), -- % of basic (if fixed)
    hra_fixed BIGINT, -- Fixed amount (if not %)
    transport_fixed BIGINT,
    allowances_json JSONB DEFAULT '{}', -- {name: amount or percent}
    deductions_json JSONB DEFAULT '{}', -- {name: amount or percent}
    ot_eligible BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Employee Salary (assigns template + base salary to employee)
CREATE TABLE IF NOT EXISTS salaries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id INT NOT NULL REFERENCES employee(employee_id) ON DELETE CASCADE,
    template_id UUID REFERENCES salary_templates(id) ON DELETE SET NULL,
    monthly_salary BIGINT NOT NULL,
    daily_salary BIGINT NOT NULL, -- monthly / 20 (typical)
    hourly_rate BIGINT NOT NULL, -- monthly / 173
    basic_salary BIGINT NOT NULL,
    fixed_allowances_json JSONB DEFAULT '{}', -- {name: amount}
    non_fixed_allowances_json JSONB DEFAULT '{}', -- {name: amount or percent}
    ot_calculation_basis INT NOT NULL DEFAULT 100, -- 100 or 75 (wage component rule)
    ot_eligible BOOLEAN NOT NULL DEFAULT TRUE,
    wage_cap BIGINT, -- Optional override
    effective_date DATE NOT NULL DEFAULT CURRENT_DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(employee_id, effective_date)
);

-- Public Holidays & Bridge Holidays
CREATE TABLE IF NOT EXISTS public_holidays (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    holiday_date DATE NOT NULL UNIQUE,
    holiday_name VARCHAR(100) NOT NULL,
    holiday_type VARCHAR(20) NOT NULL DEFAULT 'official', -- official | bridge | custom
    ot_rate_multiplier_min DECIMAL(3,2) NOT NULL DEFAULT 2.0,
    ot_rate_multiplier_max DECIMAL(3,2) NOT NULL DEFAULT 4.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Daily Attendance Calculations (denormalized from attendance + salary)
-- Materialized view or calculated on-the-fly
CREATE TABLE IF NOT EXISTS attendance_daily (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id INT NOT NULL REFERENCES employee(employee_id) ON DELETE CASCADE,
    work_date DATE NOT NULL,
    punch_in TIMESTAMPTZ,
    punch_out TIMESTAMPTZ,
    total_hours DECIMAL(5,2), -- total work hours
    is_sunday BOOLEAN NOT NULL DEFAULT FALSE,
    is_public_holiday BOOLEAN NOT NULL DEFAULT FALSE,
    is_paid_leave BOOLEAN NOT NULL DEFAULT FALSE,
    leave_type VARCHAR(50), -- 'casual', 'sick', 'personal', NULL if no leave
    
    -- Calculated fields
    days_credited DECIMAL(5,2), -- 1.0, 1.5, 2.0, etc
    full_days INT, -- floor(total_hours / daily_work_hours)
    remainder_hours DECIMAL(5,2), -- total_hours - (full_days * daily_work_hours)
    ot_hours DECIMAL(5,2),
    daily_pay BIGINT, -- days_credited * daily_salary
    ot_pay BIGINT, -- ot_hours * ot_rate
    holiday_premium BIGINT, -- bonus for rest day / holiday
    total_day_pay BIGINT, -- daily_pay + ot_pay + holiday_premium
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(employee_id, work_date)
);

-- Monthly Payslip
CREATE TABLE IF NOT EXISTS payslips (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id INT NOT NULL REFERENCES employee(employee_id) ON DELETE CASCADE,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    
    -- Earnings
    regular_days_pay BIGINT NOT NULL DEFAULT 0,
    ot_pay BIGINT NOT NULL DEFAULT 0,
    holiday_premium_pay BIGINT NOT NULL DEFAULT 0,
    paid_leave_pay BIGINT NOT NULL DEFAULT 0,
    thr_amount BIGINT NOT NULL DEFAULT 0, -- Tunjangan Hari Raya
    other_earnings_json JSONB DEFAULT '{}',
    gross_salary BIGINT NOT NULL,
    
    -- Deductions
    bpjs_kes_employee BIGINT NOT NULL DEFAULT 0,
    bpjs_jht_employee BIGINT NOT NULL DEFAULT 0,
    bpjs_jp_employee BIGINT NOT NULL DEFAULT 0,
    pph_21_withholding BIGINT NOT NULL DEFAULT 0,
    other_deductions_json JSONB DEFAULT '{}',
    total_deductions BIGINT NOT NULL,
    
    -- Summary
    net_salary BIGINT NOT NULL,
    
    -- Metadata
    status VARCHAR(20) NOT NULL DEFAULT 'draft', -- draft | locked | exported
    exported_at TIMESTAMPTZ,
    export_notes TEXT,
    created_by UUID, -- user_id
    locked_by UUID, -- user_id
    locked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(employee_id, period_start, period_end)
);

-- Yearly Attendance Summary (for reconciliation & tax)
CREATE TABLE IF NOT EXISTS attendance_summary_yearly (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id INT NOT NULL REFERENCES employee(employee_id) ON DELETE CASCADE,
    year INT NOT NULL,
    total_working_days INT,
    total_ot_hours DECIMAL(7,2),
    total_rest_day_ot_hours DECIMAL(7,2),
    total_holiday_premium_hours DECIMAL(7,2),
    total_paid_leave_hours DECIMAL(7,2),
    total_unpaid_leave_hours DECIMAL(7,2),
    total_gross_salary BIGINT,
    total_deductions BIGINT,
    thr_paid BIGINT,
    pph_21_total_withheld BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(employee_id, year)
);

-- Indexes for performance
CREATE INDEX idx_salaries_employee_id ON salaries(employee_id);
CREATE INDEX idx_salaries_effective_date ON salaries(effective_date);
CREATE INDEX idx_attendance_daily_employee_id ON attendance_daily(employee_id);
CREATE INDEX idx_attendance_daily_work_date ON attendance_daily(work_date);
CREATE INDEX idx_payslips_employee_id ON payslips(employee_id);
CREATE INDEX idx_payslips_period ON payslips(period_start, period_end);
CREATE INDEX idx_payslips_status ON payslips(status);
CREATE INDEX idx_public_holidays_date ON public_holidays(holiday_date);
