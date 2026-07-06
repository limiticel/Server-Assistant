set search_path to server_assistant, public;

alter table mcp_tools
  add column if not exists tool_type text not null default 'physical',
  add column if not exists config jsonb not null default '{}',
  add column if not exists response_schema jsonb not null default '{}';

update mcp_tools
set
  tool_type = 'physical',
  config = jsonb_build_object(
    'kind', 'ssh',
    'host', '127.0.0.1',
    'port', 2222
  )
where name = 'ubuntu_server_ssh'
  and (config = '{}'::jsonb or config is null);

