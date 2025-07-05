-- Add migration script here
CREATE TABLE devices (
    device_name SERIAL PRIMARY KEY,
    device_sn varchar(160) NOT NULL,
    device_brand int NOT NULL DEFAULT 0,
    device_location varchar(160) NULL,
    device_timezone int NOT NULL,
    device_state int NOT NULL DEFAULT 0,
    device_online TIMESTAMP NULL,
    create_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    update_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);