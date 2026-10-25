-- Public holiday calendar.
-- The attendance report classifies a day as normal working time only when the
-- date is absent from this table, so an entry here means "nobody works".
--
-- holiday_date : the calendar day, stored as a plain DATE (no time, no zone).
-- holiday_kind : 'national'   decree holiday (Hari Libur Nasional)
--                'collective' agreed leave day (Cuti Bersama)
--                'office'     company-specific closure (Libur Kantor)
-- holiday_source: 'import'  fetched from the public calendar and confirmed by an operator
--                 'manual'   typed in by an operator
--
-- The source column is provenance for the operator, not a sync cursor: nothing
-- re-reads it automatically, because a remote calendar is an unofficial
-- endpoint that can change or disappear without warning.
CREATE TABLE IF NOT EXISTS holiday (
    holiday_date DATE PRIMARY KEY,
    holiday_name TEXT NOT NULL,
    holiday_kind TEXT NOT NULL DEFAULT 'national',
    holiday_source TEXT NOT NULL DEFAULT 'manual',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- The report reads a date range, so the covering index keeps that scan off a
-- sequential table read once the calendar spans several years.
CREATE INDEX IF NOT EXISTS holiday_date_idx ON holiday (holiday_date);

-- Reject unknown kinds at the storage layer as well as in the handler: a typo
-- from a script or a hand-edited row must not become a category the UI cannot
-- render or the report cannot filter.
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
