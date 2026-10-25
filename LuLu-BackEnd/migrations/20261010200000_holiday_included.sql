-- Let a holiday still be counted in the report.
--
-- A holiday row previously meant one thing only: nobody works, so the day drops
-- out of the report entirely. That is wrong wherever the pay rate differs on a
-- holiday, because the worked hours on that day are real and feed payroll, they
-- just must not be judged against a normal roster.
--
-- holiday_included is per DATE, not global: a company closure nobody works can
-- stay excluded while a national holiday where the office stays open gets
-- counted. One extra flag per row keeps that decision with the operator who set
-- the date, instead of a single setting that forces the same answer for every
-- holiday in the year.
--
-- Default is false, which reproduces the existing behaviour exactly: a migration
-- that turned counting on for rows nobody had looked at would silently change
-- every historical payroll run.
ALTER TABLE public.holiday
    ADD COLUMN IF NOT EXISTS holiday_included BOOLEAN NOT NULL DEFAULT FALSE;
