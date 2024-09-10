-- Add migration script here
ALTER TABLE stage_task ADD COLUMN `step` int not null default 0 AFTER `status`;