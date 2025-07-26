-- Add migration script here
CREATE TABLE commands (
    command_id varchar(8) NOT NULL,
    command_name varchar(50) NULL,
    command_destination varchar(50) NULL,
    command_params TEXT NOT NULL,
    create_at BIGINT NOT NULL DEFAULT DATE_PART('epoch'::text, CURRENT_TIMESTAMP)::integer
);