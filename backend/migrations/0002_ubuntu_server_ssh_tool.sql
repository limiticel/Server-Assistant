set search_path to server_assistant, public;

insert into mcp_tools (name, description, tool_type, input_schema, config, response_schema, enabled)
values (
  'ubuntu_server_ssh',
  'Executa comandos no Ubuntu Server via SSH em 127.0.0.1:2222.',
  'physical',
  '{
    "type": "object",
    "properties": {
      "username": {
        "type": "string",
        "description": "Usuario SSH. Opcional se MCP_UBUNTU_SSH_DEFAULT_USER estiver configurado."
      },
      "command": {
        "type": "string",
        "description": "Comando para executar no servidor remoto."
      },
      "timeout_seconds": {
        "type": "integer",
        "minimum": 1,
        "maximum": 120,
        "default": 30
      }
    },
    "required": ["command"]
  }'::jsonb,
  '{"kind": "ssh", "host": "127.0.0.1", "port": 2222}'::jsonb,
  '{"type": "object", "properties": {"success": {"type": "boolean"}, "stdout": {"type": "string"}, "stderr": {"type": "string"}, "exit_code": {"type": "integer"}}}'::jsonb,
  true
)
on conflict (name) do update
set
  description = excluded.description,
  tool_type = excluded.tool_type,
  input_schema = excluded.input_schema,
  config = excluded.config,
  response_schema = excluded.response_schema,
  enabled = true,
  updated_at = now();

insert into mcp_permissions (tool_id, role, readonly)
select id, 'admin', false
from mcp_tools
where name = 'ubuntu_server_ssh'
and not exists (
  select 1
  from mcp_permissions
  where tool_id = mcp_tools.id and role = 'admin'
);
