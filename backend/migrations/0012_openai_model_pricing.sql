set search_path to server_assistant, public;

update models m
set
  input_price = 0.15,
  output_price = 0.60,
  updated_at = now()
from providers p
where p.id = m.provider_id
  and lower(m.name) in ('gpt-4o-mini', 'gpt-4o-mini-2024-07-18')
  and (
    lower(p.name) = 'openai'
    or lower(p.provider_type) in ('openai', 'openai_compatible')
  )
  and coalesce(m.input_price, 0) = 0
  and coalesce(m.output_price, 0) = 0;

update token_usage t
set estimated_cost =
  ((t.prompt_tokens::numeric / 1000000) * 0.15)
  + ((t.completion_tokens::numeric / 1000000) * 0.60)
from providers p
where p.id::text = t.provider
  and lower(t.model) in ('gpt-4o-mini', 'gpt-4o-mini-2024-07-18')
  and (
    lower(p.name) = 'openai'
    or lower(p.provider_type) in ('openai', 'openai_compatible')
  )
  and coalesce(t.estimated_cost, 0) = 0;

update token_usage t
set estimated_cost =
  ((t.prompt_tokens::numeric / 1000000) * 0.15)
  + ((t.completion_tokens::numeric / 1000000) * 0.60)
where lower(t.provider) = 'openai'
  and lower(t.model) in ('gpt-4o-mini', 'gpt-4o-mini-2024-07-18')
  and coalesce(t.estimated_cost, 0) = 0;
