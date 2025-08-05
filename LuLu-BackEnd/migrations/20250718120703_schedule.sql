-- Add migration script here
CREATE TABLE IF NOT EXISTS "schedules" (
	"schedule_id" SERIAL NOT NULL,
	"schedule_name" VARCHAR(50) NOT NULL,
	"schedule_shift_id" INTEGER NOT NULL,
	"schedule_dom" INTEGER NOT NULL,
	"schedule_parrent" INTEGER NOT NULL,
	"create_at" BIGINT NOT NULL DEFAULT (date_part('epoch'::text, CURRENT_TIMESTAMP))::integer,
	"update_at" BIGINT NOT NULL DEFAULT (date_part('epoch'::text, CURRENT_TIMESTAMP))::integer,
	"schedule_type" INTEGER NOT NULL DEFAULT 1,
	"schedule_hols" SMALLINT NOT NULL DEFAULT 0,
	PRIMARY KEY ("schedule_name", "schedule_dom")
);