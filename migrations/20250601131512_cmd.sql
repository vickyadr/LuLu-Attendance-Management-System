-- Add migration script here
CREATE TABLE commands (
    command_id varchar(8) NOT NULL,
    command_name varchar(50) NULL,
    command_destination varchar(50) NULL,
    command_params TEXT NOT NULL,
    create_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);