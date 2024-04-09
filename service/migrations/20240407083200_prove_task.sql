-- Add migration script here
CREATE TABLE IF NOT EXISTS prove_task
(
    id                  varchar(255) not null primary key,
    proof_id            varchar(255) not null default '',
    itype               int          not null default 0,
    status              int          not null default 0,
    time_cost           int          not null default 0,
    node_info           varchar(255) not null default '',
    content             text         ,
    check_at            bigint       not null default 0,
    created_at          timestamp    not null default now(),
    updated_at          timestamp    not null default now()
);