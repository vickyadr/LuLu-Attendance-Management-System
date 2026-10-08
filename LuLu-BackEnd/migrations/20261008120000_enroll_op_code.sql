ALTER TABLE "enrolls"
    ADD COLUMN IF NOT EXISTS "enroll_op_code" INTEGER NULL DEFAULT NULL;

COMMENT ON COLUMN "enrolls"."enroll_op_code" IS
    'Raw ADMS wire code (OPERLOG operation type or ATTLOG VerifyMode); NULL when the row did not carry one.';