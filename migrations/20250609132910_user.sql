-- Add migration script here
-- session_token default using md5 hash 32 characters 
CREATE TABLE users (
    user_id SERIAL PRIMARY KEY,
    user_password varchar(80) NOT NULL,
    user_level INTEGER NOT NULL DEFAULT 100,
    user_nickname varchar(30) NOT NULL,
    user_fname varchar(50) NOT NULL,
    user_lname varchar(50) NULL,
    create_at BIGINT NOT NULL DEFAULT DATE_PART('epoch'::text, CURRENT_TIMESTAMP)::integer,
    update_at BIGINT NOT NULL DEFAULT DATE_PART('epoch'::text, CURRENT_TIMESTAMP)::integer
);