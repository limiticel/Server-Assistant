set search_path to server_assistant, public;

insert into users (email, name, password_hash, role, active)
values (
  'richard.vaz@grupo3rn.com.br',
  'Richard Vaz',
  '$2a$06$JVxU1a/0mgaMA6mKHWPiO.vs7lT6zUfvNTu9ZtqDyM.duhyrW/VUi',
  'admin',
  true
)
on conflict (email) do update
set
  name = excluded.name,
  password_hash = excluded.password_hash,
  role = excluded.role,
  active = true,
  updated_at = now();
