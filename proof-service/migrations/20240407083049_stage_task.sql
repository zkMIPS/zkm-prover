-- Add migration script here
CREATE TABLE IF NOT EXISTS stage_task
(
    id                  varchar(255) primary key,
    status              int          not null default 0,
    context             text         ,
    result              text         ,
    check_at            bigint       not null default 0,
    created_at          timestamp    not null default now(),
    updated_at          timestamp    not null default now()
);