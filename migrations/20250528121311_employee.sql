-- Add migration script here
CREATE TABLE employee (
    employee_id SERIAL PRIMARY KEY,
    employee_first_name varchar(160) NOT NULL,
    employee_last_name varchar(160) NOT NULL,
    employee_address varchar(160) NULL,
    employee_departement varchar(160) NULL,
    employee_status int NOT NULL DEFAULT 0,
    create_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);