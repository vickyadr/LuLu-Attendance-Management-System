-- Add migration script here
-- session_token default using md5 hash 32 characters 
CREATE TABLE IF NOT EXISTS "users" (
	"user_id" SERIAL NOT NULL,
	"user_password" VARCHAR(80) NOT NULL,
	"user_level" INTEGER NOT NULL DEFAULT 100,
	"user_nickname" VARCHAR(30) NOT NULL,
	"user_fname" VARCHAR(50) NOT NULL,
	"user_lname" VARCHAR(50) NULL DEFAULT NULL,
	"create_at" BIGINT NOT NULL DEFAULT (date_part('epoch'::text, CURRENT_TIMESTAMP))::integer,
	"update_at" BIGINT NOT NULL DEFAULT (date_part('epoch'::text, CURRENT_TIMESTAMP))::integer,
	PRIMARY KEY ("user_nickname")
);