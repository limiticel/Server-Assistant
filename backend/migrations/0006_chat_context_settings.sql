set search_path to server_assistant, public;

insert into settings (key, value)
values (
  'chat_context',
  '{
    "compaction_enabled": true,
    "max_messages": 80,
    "keep_last_messages": 24,
    "max_summary_chars": 8000
  }'::jsonb
)
on conflict (key) do nothing;
