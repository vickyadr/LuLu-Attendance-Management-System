-- Add migration script here
CREATE TABLE IF NOT EXISTS "shifts" (
	"shift_start_time" BIGINT NOT NULL,
	"shift_end_time" BIGINT NOT NULL,
	"shift_start_enroll" BIGINT NOT NULL,
	"shift_end_enroll" BIGINT NOT NULL,
	"shift_passday" SMALLINT NOT NULL DEFAULT 0,
	"create_at" BIGINT NOT NULL DEFAULT (date_part('epoch'::text, CURRENT_TIMESTAMP))::integer,
	"update_at" BIGINT NOT NULL DEFAULT (date_part('epoch'::text, CURRENT_TIMESTAMP))::integer,
	"shift_name" VARCHAR(50) NOT NULL,
	"shift_id" SERIAL NOT NULL,
	"shift_prevday" SMALLINT NOT NULL DEFAULT 0,
	PRIMARY KEY ("shift_name")
);