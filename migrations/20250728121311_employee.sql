-- Add migration script here
CREATE TABLE IF NOT EXISTS "employee" (
	"employee_id" SERIAL NOT NULL,
	"employee_fname" VARCHAR(160) NOT NULL,
	"employee_lname" VARCHAR(160) NOT NULL,
	"employee_address" VARCHAR(160) NULL DEFAULT NULL,
	"employee_departement" VARCHAR(160) NULL DEFAULT NULL,
	"employee_status" INTEGER NOT NULL DEFAULT 0,
	"create_at" BIGINT NOT NULL DEFAULT (date_part('epoch'::text, CURRENT_TIMESTAMP))::integer,
	"update_at" BIGINT NOT NULL DEFAULT (date_part('epoch'::text, CURRENT_TIMESTAMP))::integer,
	"employee_schedule_id" INTEGER NOT NULL DEFAULT 0,
	PRIMARY KEY ("employee_id")
);