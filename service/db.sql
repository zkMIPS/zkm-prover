CREATE DATABASE IF NOT EXISTS stage;

use stage;

CREATE TABLE IF NOT EXISTS stage_task
(
    id                  varchar(255) primary key,
    status              int          not null default 0,
    context             text         ,
    result              text         ,
    check_at            timestamp    not null default now(),
    created_at          timestamp    not null default now(),
    updated_at          timestamp    not null default now()
);

CREATE TABLE IF NOT EXISTS prove_task
(
    id                  varchar(255) not null primary key,
    proof_id            varchar(255) not null default '',
    type                int          not null default 0,
    status              int          not null default 0,
    time_cost           int          not null default 0,
    node_info           varchar(255) not null default '',
    request             text         ,
    response            text         ,
    check_at            timestamp    not null default now(),
    created_at          timestamp    not null default now(),
    updated_at          timestamp    not null default now()
);