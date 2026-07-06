set search_path to server_assistant, public;

create table if not exists model_mcp_tools (
  model_id uuid references models(id) on delete cascade,
  tool_id uuid references mcp_tools(id) on delete cascade,
  created_at timestamptz not null default now(),
  primary key (model_id, tool_id)
);
