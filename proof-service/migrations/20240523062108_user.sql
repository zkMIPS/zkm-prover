-- Add migration script here
CREATE TABLE IF NOT EXISTS user
(
    address             varchar(64) primary key,
    created_at          timestamp    not null default now(),
    updated_at          timestamp    not null default now()
);