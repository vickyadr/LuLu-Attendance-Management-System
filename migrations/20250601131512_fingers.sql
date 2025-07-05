-- Add migration script here
CREATE TABLE fingers (
    finger_employee_id INTEGER NOT NULL,
    finger_id varchar(2) NULL,
    finger_code TEXT NOT NULL,
    update_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    create_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);