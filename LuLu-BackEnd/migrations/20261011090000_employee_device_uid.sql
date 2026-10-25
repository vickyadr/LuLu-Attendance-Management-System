-- Device-side user id an employee punches with.
--
-- Punches arrive keyed by the number stored in the machine (enrolls.enroll_employee_id).
-- Until now every report and live-table query joined that number straight to
-- employee_id, which only works by coincidence: it assumes the machine was
-- provisioned with the same ids the app assigned. A machine that starts at uid 1
-- for a different person then punches into nobody, and the punch is invisible.
--
-- This column is the explicit link. After it exists, joins go through it only.
--
-- NULL = not linked to any machine. UNIQUE so two employees can never claim the
-- same machine id, which would make attendance rows ambiguous.
ALTER TABLE employee ADD COLUMN IF NOT EXISTS employee_device_uid INTEGER;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_indexes WHERE indexname = 'employee_device_uid_key'
    ) THEN
        CREATE UNIQUE INDEX employee_device_uid_key
            ON employee (employee_device_uid)
            WHERE employee_device_uid IS NOT NULL;
    END IF;
END$$;

-- Backfill with the id the old join used, so switching the join key does not
-- change which employee a historical punch resolves to. Employee_id is unique,
-- so the ids cannot collide with each other; the unique index above still holds.
-- An operator can later point an employee at the real machine number, and can
-- clear it back to NULL from the employee form.
UPDATE employee SET employee_device_uid = employee_id WHERE employee_device_uid IS NULL;