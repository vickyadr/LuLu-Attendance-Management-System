-- Store the raw device op/verify code alongside the mapped enroll type.
--
-- The ADMS protocol assigns numeric codes to each operation (OPERLOG) and each
-- verification modality (ATTLOG). LuLu maps a documented subset to `enroll_type`
-- for display, but the protocol also defines codes it does not enumerate here
-- (the vendor's own server recognises at least OPERLOG 9 and 13). Dropping such
-- a row loses the punch entirely; keeping the raw code lets an unknown code be
-- identified from data instead of guessed at.
--
-- Nullable so existing rows are untouched and the column stays optional.
ALTER TABLE "enrolls"
    ADD COLUMN IF NOT EXISTS "enroll_op_code" INTEGER NULL DEFAULT NULL;

COMMENT ON COLUMN "enrolls"."enroll_op_code" IS
    'Raw ADMS wire code (OPERLOG operation type or ATTLOG VerifyMode); NULL when the row did not carry one.';