CREATE TABLE IF NOT EXISTS "handlers" (
	"handler_id" SERIAL NOT NULL,
	"handler_name" VARCHAR(25) NOT NULL,
	"create_at" BIGINT NOT NULL DEFAULT (date_part('epoch'::text, CURRENT_TIMESTAMP))::integer,
	PRIMARY KEY ("handler_name")
);