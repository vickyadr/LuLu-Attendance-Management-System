-- The dev database renamed salaries.monthly_salary to monthly_rate by hand, and no
-- migration recorded it. A database built from these migrations still has the old
-- name, so every INSERT/SELECT on salaries.monthly_rate fails there. Rename only
-- when the old column exists and the new one does not, so both a fresh install and
-- the already-renamed dev database end in the same shape.
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = 'salaries' AND column_name = 'monthly_salary'
    ) AND NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = 'salaries' AND column_name = 'monthly_rate'
    ) THEN
        ALTER TABLE salaries RENAME COLUMN monthly_salary TO monthly_rate;
    END IF;
END $$;
