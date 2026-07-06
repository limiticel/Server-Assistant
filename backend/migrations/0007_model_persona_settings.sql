set search_path to server_assistant, public;

alter table models
  add column if not exists assistant_name text,
  add column if not exists personality text,
  add column if not exists temperament text,
  add column if not exists pre_prompt text,
  add column if not exists pre_prompt_limit integer not null default 2000;
