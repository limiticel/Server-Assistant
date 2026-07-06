set search_path to server_assistant, public;

insert into mcp_tools (name, description, tool_type, input_schema, config, response_schema, enabled)
values (
  'postgres_query',
  'Executa consultas SQL no PostgreSQL configurado via SSH sem terminal interativo.',
  'physical',
  '{
    "type": "object",
    "properties": {
      "query": {
        "type": "string",
        "description": "Consulta SQL para executar. Use SELECT para diagnosticos."
      },
      "timeout_seconds": {
        "type": "integer",
        "minimum": 1,
        "maximum": 120,
        "default": 30
      }
    },
    "required": ["query"]
  }'::jsonb,
  '{
    "kind": "postgres_query"
  }'::jsonb,
  '{}'::jsonb,
  true
)
on conflict (name) do update set
  description = excluded.description,
  tool_type = excluded.tool_type,
  input_schema = excluded.input_schema,
  response_schema = excluded.response_schema,
  enabled = true,
  updated_at = now();
