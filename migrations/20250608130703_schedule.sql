-- Add migration script here
CREATE TABLE schedules (
    schedule_id SERIAL PRIMARY KEY,
    schedule_name varchar(50) NOT NULL,
    schedule_start INTEGER NULL,
    schedule_end int NOT NULL DEFAULT 0,
    schedule_sub_id SERIAL NULL,
    schedule_shift_id SERIAL NOT NULL,
    schedule_passmonth int NOT NULL DEFAULT 0,
    schedule_use_week int NOT NULL DEFAULT 0,
    create_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);