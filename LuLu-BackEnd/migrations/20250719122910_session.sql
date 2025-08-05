-- Add migration script here
-- session_token default using md5 hash 32 characters 
CREATE TABLE IF NOT EXISTS "sessions" (
	"session_id" SERIAL NOT NULL,
	"session_token" VARCHAR(80) NOT NULL,
	"session_ip" VARCHAR(16) NOT NULL,
	"session_ua" VARCHAR(160) NOT NULL,
	"session_user_id" INTEGER NOT NULL,
	"create_at" BIGINT NOT NULL DEFAULT (date_part('epoch'::text, CURRENT_TIMESTAMP))::integer,
	"update_at" BIGINT NOT NULL DEFAULT (date_part('epoch'::text, CURRENT_TIMESTAMP))::integer,
	PRIMARY KEY ("session_token")
);