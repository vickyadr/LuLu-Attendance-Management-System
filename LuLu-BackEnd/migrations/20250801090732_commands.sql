CREATE TABLE IF NOT EXISTS "commands" (
	"command_id" VARCHAR(8) NOT NULL,
	"command_name" VARCHAR(50) NOT NULL DEFAULT NULL::character varying,
	"command_params" TEXT NOT NULL,
	"command_destination" VARCHAR(50) NOT NULL,
	"create_at" BIGINT NOT NULL DEFAULT (date_part('epoch'::text, CURRENT_TIMESTAMP))::integer,
	PRIMARY KEY ("command_name")
);