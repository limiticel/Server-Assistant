set search_path to server_assistant, public;

insert into mcp_tools (name, description, tool_type, input_schema, config, response_schema, enabled)
values (
  'web_search',
  'Busca informacoes na internet usando DuckDuckGo Instant Answer.',
  'physical',
  '{
    "type": "object",
    "properties": {
      "query": {
        "type": "string",
        "description": "Termo ou pergunta para buscar na internet."
      },
      "max_results": {
        "type": "integer",
        "minimum": 1,
        "maximum": 8,
        "default": 5
      }
    },
    "required": ["query"]
  }'::jsonb,
  '{"kind": "web_search", "provider": "duckduckgo_instant_answer"}'::jsonb,
  '{
    "type": "object",
    "properties": {
      "query": {"type": "string"},
      "source": {"type": "string"},
      "heading": {"type": "string"},
      "abstract": {"type": "string"},
      "abstract_url": {"type": "string"},
      "official_website": {"type": "string"},
      "results": {"type": "array"}
    }
  }'::jsonb,
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
select id, 'admin', true
from mcp_tools
where name = 'web_search'
and not exists (
  select 1
  from mcp_permissions
  where tool_id = mcp_tools.id and role = 'admin'
);
