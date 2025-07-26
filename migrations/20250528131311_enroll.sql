-- Add migration script here
CREATE TABLE enrolls (
    enroll_device_sn varchar(50) NULL,
    enroll_employee_id INTEGER NULL,
    enroll_type int NOT NULL DEFAULT 0,
    enroll_status int NOT NULL DEFAULT 0,
    enroll_time BIGINT NOT NULL,
    create_at BIGINT NOT NULL DEFAULT DATE_PART('epoch'::text, CURRENT_TIMESTAMP)::integer
);