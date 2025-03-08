-- Add migration script here 
ALTER TABLE stage_task ADD COLUMN `address` varchar(64) AFTER id;