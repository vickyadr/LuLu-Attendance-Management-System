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

UPDATE employee SET employee_device_uid = employee_id WHERE employee_device_uid IS NULL;