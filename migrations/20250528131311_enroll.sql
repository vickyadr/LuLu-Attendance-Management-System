-- Add migration script here
CREATE TABLE enrolls (
    enroll_device_sn varchar(50) NULL,
    enroll_employee_id INTEGER NULL,
    enroll_type int NOT NULL DEFAULT 0,
    enroll_time TIMESTAMP NOT NULL,
    create_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);