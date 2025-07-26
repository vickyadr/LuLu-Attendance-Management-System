-- Add migration script here
CREATE TABLE fingers (
    finger_employee_id INTEGER NOT NULL,
    finger_id varchar(2) NULL,
    finger_code TEXT NOT NULL,
    create_at BIGINT NOT NULL DEFAULT DATE_PART('epoch'::text, CURRENT_TIMESTAMP)::integer,
    update_at BIGINT NOT NULL DEFAULT DATE_PART('epoch'::text, CURRENT_TIMESTAMP)::integer
);