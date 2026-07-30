set search_path to server_assistant, public;

insert into users (email, name, password_hash, role, active)
values (
  'richard.vaz@grupo3rn.com.br',
  'Richard Vaz',
  '$1$saadmin2$5snl34AHLAPYgnZRFaquO.',
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
