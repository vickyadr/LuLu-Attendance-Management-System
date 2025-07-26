-- Add migration script here
CREATE TABLE employee (
    employee_id SERIAL PRIMARY KEY,
    employee_fname varchar(160) NOT NULL,
    employee_lname varchar(160) NOT NULL,
    employee_address varchar(160) NULL,
    employee_departement varchar(160) NULL,
    employee_hire_date BIGINT NOT NULL DEFAULT DATE_PART('epoch'::text, CURRENT_TIMESTAMP)::integer,
    employee_phone varchar(20) NULL,
    employee_status int NOT NULL DEFAULT 0,
    create_at BIGINT NOT NULL DEFAULT DATE_PART('epoch'::text, CURRENT_TIMESTAMP)::integer,
    update_at BIGINT NOT NULL DEFAULT DATE_PART('epoch'::text, CURRENT_TIMESTAMP)::integer
);