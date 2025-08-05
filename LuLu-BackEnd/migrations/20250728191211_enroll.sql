-- Add migration script here
CREATE TABLE IF NOT EXISTS "enrolls" (
	"enroll_device_sn" VARCHAR(50) NOT NULL DEFAULT NULL::character varying,
	"enroll_type" INTEGER NULL DEFAULT 0,
	"enroll_employee_id" INTEGER NOT NULL,
	"enroll_status" INTEGER NULL DEFAULT 0,
	"enroll_id" SERIAL NOT NULL,
	"enroll_time" BIGINT NOT NULL DEFAULT (date_part('epoch'::text, CURRENT_TIMESTAMP))::integer,
	"create_at" BIGINT NOT NULL DEFAULT (date_part('epoch'::text, CURRENT_TIMESTAMP))::integer,
	PRIMARY KEY ("enroll_device_sn", "enroll_employee_id", "enroll_time")
);