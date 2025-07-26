-- Add migration script here
CREATE TABLE devices (
    device_id SERIAL,
    device_name varchar(50) NOT NULL,
    device_sn varchar(50) NOT NULL PRIMARY KEY,
    device_brand int NOT NULL DEFAULT 0,
    device_location varchar(160) NULL,
    device_timezone int NOT NULL,
    device_state int NOT NULL DEFAULT 0,
    device_online BIGINT NULL,
    create_at BIGINT NOT NULL DEFAULT DATE_PART('epoch'::text, CURRENT_TIMESTAMP)::integer,
    update_at BIGINT NOT NULL DEFAULT DATE_PART('epoch'::text, CURRENT_TIMESTAMP)::integer
);