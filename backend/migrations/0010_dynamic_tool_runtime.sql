set search_path to server_assistant, public;

update mcp_tools
set
  description = 'Executa consultas SELECT no PostgreSQL principal do Server Assistant.',
  config = '{"kind": "postgres_query"}'::jsonb,
  enabled = true,
  updated_at = now()
where name = 'postgres_query';

update mcp_tools
set
  description = 'Executa comandos via SSH usando o executor dinamico local.',
  config = jsonb_set(
    coalesce(config, '{}'::jsonb),
    '{kind}',
    '"ssh"'::jsonb,
    true
  ),
  enabled = true,
  updated_at = now()
where name = 'ubuntu_server_ssh';
