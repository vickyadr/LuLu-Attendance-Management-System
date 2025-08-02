CREATE TABLE IF NOT EXISTS "fingers" (
	"finger_id" VARCHAR(2) NOT NULL,
	"finger_code" TEXT NOT NULL,
	"finger_employee_id" INTEGER NOT NULL,
	"create_at" BIGINT NOT NULL DEFAULT (date_part('epoch'::text, CURRENT_TIMESTAMP))::integer,
	"update_at" BIGINT NOT NULL DEFAULT (date_part('epoch'::text, CURRENT_TIMESTAMP))::integer,
	PRIMARY KEY ("finger_employee_id", "finger_id")
);