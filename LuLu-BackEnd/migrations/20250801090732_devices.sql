-- Add migration script here
CREATE TABLE IF NOT EXISTS "devices" (
	"device_id" SERIAL NOT NULL,
	"device_name" VARCHAR(50) NOT NULL,
	"device_sn" VARCHAR(50) NOT NULL,
	"device_handler_id" INTEGER NOT NULL DEFAULT 0,
	"device_location" VARCHAR(160) NULL DEFAULT NULL,
	"device_timezone" INTEGER NOT NULL,
	"device_state" INTEGER NOT NULL DEFAULT 0,
	"create_at" BIGINT NOT NULL DEFAULT (date_part('epoch'::text, CURRENT_TIMESTAMP))::integer,
	"update_at" BIGINT NOT NULL DEFAULT (date_part('epoch'::text, CURRENT_TIMESTAMP))::integer,
	"device_online" BIGINT NOT NULL DEFAULT (date_part('epoch'::text, CURRENT_TIMESTAMP))::integer,
	PRIMARY KEY ("device_sn", "device_handler_id")
);