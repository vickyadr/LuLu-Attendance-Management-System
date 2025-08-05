-- Add migration script here
CREATE TABLE logs (
    log_type varchar(50) NOT NULL,
    log_name varchar(50) NOT NULL,
    log_trace varchar(50) NULL,
    log_detail varchar(255) NULL,
    create_at BIGINT NOT NULL DEFAULT DATE_PART('epoch'::text, CURRENT_TIMESTAMP)::integer,
    update_at BIGINT NOT NULL DEFAULT DATE_PART('epoch'::text, CURRENT_TIMESTAMP)::integer
);