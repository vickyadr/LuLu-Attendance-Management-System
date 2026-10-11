CREATE TABLE IF NOT EXISTS holiday (
    holiday_date DATE PRIMARY KEY,
    holiday_name TEXT NOT NULL,
    holiday_kind TEXT NOT NULL DEFAULT 'national',
    holiday_source TEXT NOT NULL DEFAULT 'manual',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS holiday_date_idx ON holiday (holiday_date);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'holiday_kind_check') THEN
        ALTER TABLE holiday ADD CONSTRAINT holiday_kind_check
            CHECK (holiday_kind IN ('national', 'collective', 'office'));
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'holiday_source_check') THEN
        ALTER TABLE holiday ADD CONSTRAINT holiday_source_check
            CHECK (holiday_source IN ('import', 'manual'));
    END IF;
END
$$;
