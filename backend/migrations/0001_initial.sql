create schema if not exists server_assistant;
create extension if not exists "uuid-ossp" with schema public;
create extension if not exists pgcrypto with schema public;
set search_path to server_assistant, public;

create table roles (
  id uuid primary key default uuid_generate_v4(),
  name text unique not null,
  created_at timestamptz not null default now()
);

create table users (
  id uuid primary key default uuid_generate_v4(),
  email text unique not null,
  name text not null,
  password_hash text not null,
  role text not null default 'user',
  active boolean not null default true,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table providers (
  id uuid primary key default uuid_generate_v4(),
  name text not null,
  base_url text not null,
  api_key_cipher text,
  default_model text,
  provider_type text not null,
  openai_compatible boolean not null default false,
  active boolean not null default true,
  health_status text not null default 'unknown',
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table models (
  id uuid primary key default uuid_generate_v4(),
  provider_id uuid references providers(id) on delete cascade,
  name text not null,
  context_window integer,
  input_price numeric(12,6) default 0,
  output_price numeric(12,6) default 0,
  active boolean not null default true,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table personalities (
  id uuid primary key default uuid_generate_v4(),
  name text not null,
  description text not null,
  system_prompt text not null,
  temperature real not null default 0.7,
  top_p real not null default 1,
  max_tokens integer not null default 2048,
  default_provider_id uuid references providers(id),
  default_model_id uuid references models(id),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table conversations (
  id uuid primary key default uuid_generate_v4(),
  user_id uuid references users(id) on delete set null,
  title text not null default 'Novo chat',
  provider_id uuid references providers(id),
  model_id uuid references models(id),
  personality_id uuid references personalities(id),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table messages (
  id uuid primary key default uuid_generate_v4(),
  conversation_id uuid references conversations(id) on delete cascade,
  role text not null,
  content text not null,
  provider text,
  model text,
  metadata jsonb not null default '{}',
  created_at timestamptz not null default now()
);

create table token_usage (
  id uuid primary key default uuid_generate_v4(),
  user_id uuid references users(id) on delete set null,
  provider text not null,
  model text not null,
  prompt_tokens bigint not null default 0,
  completion_tokens bigint not null default 0,
  total_tokens bigint not null default 0,
  estimated_cost numeric(12,6) not null default 0,
  created_at timestamptz not null default now()
);

create table settings (
  id uuid primary key default uuid_generate_v4(),
  key text unique not null,
  value jsonb not null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table api_keys (
  id uuid primary key default uuid_generate_v4(),
  user_id uuid references users(id) on delete cascade,
  name text not null,
  key_hash text not null,
  active boolean not null default true,
  created_at timestamptz not null default now(),
  last_used_at timestamptz
);

create table audit_logs (
  id uuid primary key default uuid_generate_v4(),
  user_id uuid references users(id) on delete set null,
  action text not null,
  entity text not null,
  entity_id uuid,
  metadata jsonb not null default '{}',
  created_at timestamptz not null default now()
);

create table sessions (
  id uuid primary key default uuid_generate_v4(),
  user_id uuid references users(id) on delete cascade,
  refresh_token_hash text not null,
  expires_at timestamptz not null,
  created_at timestamptz not null default now()
);

create table mcp_tools (
  id uuid primary key default uuid_generate_v4(),
  name text unique not null,
  description text not null,
  tool_type text not null default 'physical',
  input_schema jsonb not null default '{}',
  config jsonb not null default '{}',
  response_schema jsonb not null default '{}',
  enabled boolean not null default true,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table mcp_permissions (
  id uuid primary key default uuid_generate_v4(),
  tool_id uuid references mcp_tools(id) on delete cascade,
  role text not null,
  readonly boolean not null default true,
  created_at timestamptz not null default now()
);

insert into roles (name) values ('admin'), ('user'), ('readonly') on conflict do nothing;
insert into personalities (name, description, system_prompt) values
('Programador Senior', 'Ajuda com arquitetura, Rust, Vue e DevOps.', 'Voce e um programador senior pragmatico.'),
('Professor', 'Explica conceitos passo a passo.', 'Voce e um professor claro e paciente.'),
('Tradutor', 'Traduz preservando sentido e tom.', 'Voce e um tradutor profissional.'),
('Assistente Financeiro', 'Ajuda com analise financeira geral.', 'Voce e um assistente financeiro cuidadoso.')
on conflict do nothing;

insert into mcp_tools (name, description, input_schema, enabled)
values (
  'ubuntu_server_ssh',
  'Executa comandos no Ubuntu Server via SSH em 127.0.0.1:2222.',
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
  true
)
on conflict (name) do nothing;

insert into mcp_permissions (tool_id, role, readonly)
select id, 'admin', false
from mcp_tools
where name = 'ubuntu_server_ssh';
