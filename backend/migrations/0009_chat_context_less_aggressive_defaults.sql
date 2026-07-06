set search_path to server_assistant, public;

update settings
set
  value = jsonb_build_object(
    'compaction_enabled', coalesce((value->>'compaction_enabled')::boolean, true),
    'max_messages', greatest(coalesce((value->>'max_messages')::int, 80), 80),
    'keep_last_messages', greatest(coalesce((value->>'keep_last_messages')::int, 24), 24),
    'max_summary_chars', greatest(coalesce((value->>'max_summary_chars')::int, 8000), 8000)
  ),
  updated_at = now()
where key = 'chat_context'
  and (
    coalesce((value->>'max_messages')::int, 0) < 80
    or coalesce((value->>'keep_last_messages')::int, 0) < 24
    or coalesce((value->>'max_summary_chars')::int, 0) < 8000
  );
