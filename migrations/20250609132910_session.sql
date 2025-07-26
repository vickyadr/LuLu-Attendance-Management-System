-- Add migration script here
-- session_token default using md5 hash 32 characters 
CREATE TABLE sessions (
    session_id SERIAL PRIMARY KEY,
    session_user_id SERIAL NOT NULL,
    session_token varchar(80) NOT NULL,
    session_ip varchar(16) NOT NULL,
    session_ua varchar(160) NOT NULL,
    create_at BIGINT NOT NULL DEFAULT DATE_PART('epoch'::text, CURRENT_TIMESTAMP)::integer,
    update_at BIGINT NOT NULL DEFAULT DATE_PART('epoch'::text, CURRENT_TIMESTAMP)::integer
);