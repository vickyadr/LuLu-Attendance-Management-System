-- Add migration script here
-- session_token default using md5 hash 32 characters 
CREATE TABLE sessions (
    session_id SERIAL PRIMARY KEY,
    session_user_id SERIAL NOT NULL,
    session_token varchar(40) NOT NULL,
    session_ip varchar(16) NOT NULL,
    session_ua varchar(160) NOT NULL,
    update_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    create_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);