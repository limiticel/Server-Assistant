use async_stream::stream;
use futures::stream::BoxStream;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::ai::{
        AiProvider, ChatMessage, ChatRequest, ChatResponse, ChatStream, ChatTool, ChatToolCall,
        ChatToolFunction,
    },
    infrastructure::providers::OpenAiCompatibleProvider,
    shared::AppError,
    AppState,
};

const TOOL_SYSTEM_PROMPT: &str = "Voce tem acesso a ferramentas reais deste sistema. Use ferramentas quando elas ajudarem a responder com dados reais, buscar informacoes atuais ou executar uma acao solicitada. Se o usuario pedir para voce tentar, executar, rodar, testar, verificar, diagnosticar, conectar ou administrar o servidor, chame ubuntu_server_ssh. Quando o usuario enviar um comando de terminal e pedir para voce tentar, execute esse comando pela ferramenta, sem apenas explicar. Se o usuario pedir busca, pesquisa, site oficial, noticia ou informacao atual da internet, chame web_search. Se o usuario pedir banco de dados, PostgreSQL, schema, tabelas ou consultas SQL, prefira postgres_query. O banco deste projeto e PostgreSQL; nao use mysql salvo se o usuario pedir MySQL explicitamente. Nunca use comandos interativos como psql sem -c, sudo -i, sudo su ou sudo que exige terminal. Para PostgreSQL, use postgres_query ou comandos psql nao-interativos. Para status de servicos, prefira comandos sem sudo como systemctl is-active postgresql, systemctl status postgresql --no-pager ou service postgresql status. Nao diga que nao tem capacidade de acessar sistemas externos quando uma ferramenta adequada estiver disponivel. Para comandos de servidor, prefira comandos somente leitura quando o usuario pedir diagnostico.";

pub type AgentStream = BoxStream<'static, Result<AgentEvent, AppError>>;

pub enum AgentEvent {
    Status(String),
    ToolStart { name: String, arguments: Value },
    ToolResult { name: String, result: Value },
    Delta(String),
}

struct ModelPersona {
    assistant_name: Option<String>,
    personality: Option<String>,
    temperament: Option<String>,
    pre_prompt: Option<String>,
    pre_prompt_limit: i32,
}

pub async fn complete_chat(
    state: &AppState,
    provider: &str,
    model: &str,
    mut messages: Vec<ChatMessage>,
) -> Result<ChatResponse, AppError> {
    prepend_model_persona(state, provider, model, &mut messages).await?;

    let request = ChatRequest {
        model: model.to_owned(),
        messages,
        temperature: Some(0.7),
        top_p: Some(1.0),
        max_tokens: Some(2048),
        stream: false,
        tools: None,
        tool_choice: None,
    };

    let response = match load_provider_from_database(state, provider).await? {
        Some(db_provider) => db_provider.chat(request).await?,
        None => state.providers.get(provider)?.chat(request).await?,
    };
    let _ = sqlx::query(
        "insert into token_usage (id, user_id, provider, model, prompt_tokens, completion_tokens, total_tokens, estimated_cost)
         values ($1, null, $2, $3, $4, $5, $6, 0)",
    )
    .bind(Uuid::new_v4())
    .bind(provider)
    .bind(model)
    .bind(response.prompt_tokens as i64)
    .bind(response.completion_tokens as i64)
    .bind((response.prompt_tokens + response.completion_tokens) as i64)
    .execute(&state.db)
    .await;

    Ok(response)
}

pub async fn stream_chat(
    state: &AppState,
    provider: &str,
    model: &str,
    messages: Vec<ChatMessage>,
) -> Result<ChatStream, AppError> {
    let request = ChatRequest {
        model: model.to_owned(),
        messages,
        temperature: Some(0.7),
        top_p: Some(1.0),
        max_tokens: Some(2048),
        stream: true,
        tools: None,
        tool_choice: None,
    };

    match load_provider_from_database(state, provider).await? {
        Some(db_provider) => db_provider.chat_stream(request).await,
        None => state.providers.get(provider)?.chat_stream(request).await,
    }
}

pub async fn agent_stream_with_model_tools(
    state: AppState,
    provider: String,
    model: String,
    mut messages: Vec<ChatMessage>,
) -> Result<AgentStream, AppError> {
    let tools = load_tools_for_model(&state, &provider, &model).await?;
    prepend_model_persona(&state, &provider, &model, &mut messages).await?;

    if tools.is_empty() {
        let mut upstream = stream_chat(&state, &provider, &model, messages).await?;
        return Ok(Box::pin(stream! {
            yield Ok(AgentEvent::Status("Respondendo sem ferramentas atribuídas ao modelo.".to_owned()));
            while let Some(item) = futures::StreamExt::next(&mut upstream).await {
                match item {
                    Ok(delta) => yield Ok(AgentEvent::Delta(delta)),
                    Err(err) => yield Err(err),
                }
            }
        }));
    }

    let latest_user_message = messages
        .iter()
        .rev()
        .find(|message| message.role == "user")
        .map(|message| message.content.clone())
        .unwrap_or_default();
    let tool_names = tools
        .iter()
        .map(|tool| tool.function.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let should_require_tool = should_require_tool_call(&latest_user_message);

    prepend_system_message(&mut messages, TOOL_SYSTEM_PROMPT.to_owned());

    Ok(Box::pin(stream! {
        yield Ok(AgentEvent::Status(format!("Pensando com ferramentas disponiveis: {tool_names}.")));
        if should_require_tool {
            yield Ok(AgentEvent::Status("A solicitacao parece pedir uma acao externa; vou forcar o uso de uma ferramenta.".to_owned()));
        }

        let decision = match complete_chat_with_tools(&state, &provider, &model, messages.clone(), tools, should_require_tool).await {
            Ok(decision) => decision,
            Err(err) => {
                yield Err(err);
                return;
            }
        };

        if decision.tool_calls.is_empty() {
            if !decision.content.is_empty() {
                yield Ok(AgentEvent::Status("O modelo decidiu responder sem chamar ferramentas.".to_owned()));
                yield Ok(AgentEvent::Delta(decision.content));
            }
            return;
        }

        messages.push(ChatMessage {
            role: "assistant".to_owned(),
            content: decision.content,
            name: None,
            tool_call_id: None,
            tool_calls: Some(decision.tool_calls.clone()),
        });

        for call in decision.tool_calls {
            let arguments = serde_json::from_str::<Value>(&call.function.arguments).unwrap_or_else(|_| Value::String(call.function.arguments.clone()));
            yield Ok(AgentEvent::ToolStart {
                name: call.function.name.clone(),
                arguments,
            });

            let result = match call_model_tool(&state, &call).await {
                Ok(result) => result,
                Err(err) => {
                    yield Err(err);
                    return;
                }
            };

            yield Ok(AgentEvent::ToolResult {
                name: call.function.name.clone(),
                result: result.clone(),
            });

            messages.push(ChatMessage {
                role: "tool".to_owned(),
                content: result.to_string(),
                name: Some(call.function.name),
                tool_call_id: Some(call.id),
                tool_calls: None,
            });
        }

        yield Ok(AgentEvent::Status("Gerando resposta final com base no resultado das ferramentas.".to_owned()));

        let mut upstream = match stream_chat(&state, &provider, &model, messages).await {
            Ok(upstream) => upstream,
            Err(err) => {
                yield Err(err);
                return;
            }
        };

        while let Some(item) = futures::StreamExt::next(&mut upstream).await {
            match item {
                Ok(delta) => yield Ok(AgentEvent::Delta(delta)),
                Err(err) => yield Err(err),
            }
        }
    }))
}

pub async fn record_estimated_stream_usage(
    db: &PgPool,
    provider: &str,
    model: &str,
    messages: &[ChatMessage],
    completion: &str,
) {
    let Ok(Some((provider_name, provider_type))) = load_billable_provider(db, provider).await
    else {
        return;
    };

    let provider_name = provider_name.to_lowercase();
    let provider_type = provider_type.to_lowercase();
    let is_billable = provider_name == "openai"
        || matches!(provider_type.as_str(), "openai" | "anthropic" | "claude");

    if !is_billable {
        return;
    }

    let prompt_text = messages
        .iter()
        .map(|message| message.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let prompt_tokens = estimate_tokens(&prompt_text);
    let completion_tokens = estimate_tokens(completion);
    let total_tokens = prompt_tokens + completion_tokens;
    let estimated_cost = estimate_cost(db, provider, model, prompt_tokens, completion_tokens)
        .await
        .unwrap_or(0.0);

    let _ = sqlx::query(
        "insert into token_usage
         (id, user_id, provider, model, prompt_tokens, completion_tokens, total_tokens, estimated_cost)
         values ($1, null, $2, $3, $4, $5, $6, $7::numeric)",
    )
    .bind(Uuid::new_v4())
    .bind(provider)
    .bind(model)
    .bind(prompt_tokens as i64)
    .bind(completion_tokens as i64)
    .bind(total_tokens as i64)
    .bind(format!("{estimated_cost:.6}"))
    .execute(db)
    .await;
}

async fn load_billable_provider(
    db: &PgPool,
    provider: &str,
) -> Result<Option<(String, String)>, sqlx::Error> {
    if let Ok(id) = Uuid::parse_str(provider) {
        sqlx::query_as("select name, provider_type from providers where id = $1")
            .bind(id)
            .fetch_optional(db)
            .await
    } else {
        sqlx::query_as(
            "select name, provider_type
             from providers
             where lower(name) = lower($1)
             order by created_at desc
             limit 1",
        )
        .bind(provider)
        .fetch_optional(db)
        .await
    }
}

async fn estimate_cost(
    db: &PgPool,
    provider: &str,
    model: &str,
    prompt_tokens: u32,
    completion_tokens: u32,
) -> Result<f64, sqlx::Error> {
    let provider_uuid = Uuid::parse_str(provider).ok();
    let row: Option<(String, String)> = sqlx::query_as(
        "select coalesce(input_price, 0)::text, coalesce(output_price, 0)::text
         from models m
         join providers p on p.id = m.provider_id
         where m.name = $1
           and (
             ($2::uuid is not null and m.provider_id = $2::uuid)
             or lower(p.name) = lower($3)
           )
         order by m.created_at desc
         limit 1",
    )
    .bind(model)
    .bind(provider_uuid)
    .bind(provider)
    .fetch_optional(db)
    .await?;

    let Some((input_price, output_price)) = row else {
        return Ok(0.0);
    };

    let input_price = input_price.parse::<f64>().unwrap_or(0.0);
    let output_price = output_price.parse::<f64>().unwrap_or(0.0);

    Ok((prompt_tokens as f64 / 1_000_000.0 * input_price)
        + (completion_tokens as f64 / 1_000_000.0 * output_price))
}

fn estimate_tokens(text: &str) -> u32 {
    let chars = text.chars().count() as u32;
    let words = text.split_whitespace().count() as u32;
    ((chars / 4).max(words)).max(1)
}

async fn complete_chat_with_tools(
    state: &AppState,
    provider: &str,
    model: &str,
    messages: Vec<ChatMessage>,
    tools: Vec<ChatTool>,
    require_tool: bool,
) -> Result<ChatResponse, AppError> {
    let request = ChatRequest {
        model: model.to_owned(),
        messages,
        temperature: Some(0.7),
        top_p: Some(1.0),
        max_tokens: Some(2048),
        stream: false,
        tools: Some(tools),
        tool_choice: Some(if require_tool { "required" } else { "auto" }.to_owned()),
    };

    match load_provider_from_database(state, provider).await? {
        Some(db_provider) => db_provider.chat(request).await,
        None => state.providers.get(provider)?.chat(request).await,
    }
}

fn should_require_tool_call(content: &str) -> bool {
    let lower = content.to_lowercase();
    let action_words = [
        "tenta",
        "tente",
        "teste",
        "testa",
        "executa",
        "execute",
        "roda",
        "rode",
        "verifica",
        "verifique",
        "conecta",
        "conecte",
        "servidor",
        "server",
        "ubuntu",
        "terminal",
        "comando",
        "busca",
        "buscar",
        "pesquisa",
        "pesquise",
        "procura",
        "procure",
        "internet",
        "site oficial",
        "noticia",
        "noticias",
    ];

    action_words.iter().any(|word| lower.contains(word))
        || lower.lines().any(looks_like_shell_command)
}

fn looks_like_shell_command(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }

    let first = trimmed.split_whitespace().next().unwrap_or_default();
    matches!(
        first,
        "psql"
            | "ls"
            | "cat"
            | "grep"
            | "rg"
            | "find"
            | "whoami"
            | "hostname"
            | "uname"
            | "uptime"
            | "df"
            | "du"
            | "systemctl"
            | "docker"
            | "curl"
            | "wget"
            | "ss"
            | "netstat"
            | "ps"
            | "top"
            | "free"
    )
}

async fn load_tools_for_model(
    state: &AppState,
    provider: &str,
    model: &str,
) -> Result<Vec<ChatTool>, AppError> {
    let provider_uuid = Uuid::parse_str(provider).ok();
    let rows: Vec<(String, String, Value)> = sqlx::query_as(
        "select t.name, t.description, t.input_schema
         from models m
         join providers p on p.id = m.provider_id
         join model_mcp_tools mt on mt.model_id = m.id
         join mcp_tools t on t.id = mt.tool_id
         where m.name = $1
           and m.active = true
           and t.enabled = true
           and (
             ($2::uuid is not null and m.provider_id = $2::uuid)
             or lower(p.name) = lower($3)
           )
         order by t.name",
    )
    .bind(model)
    .bind(provider_uuid)
    .bind(provider)
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(name, description, parameters)| ChatTool {
            tool_type: "function".to_owned(),
            function: ChatToolFunction {
                name,
                description,
                parameters,
            },
        })
        .collect())
}

async fn prepend_model_persona(
    state: &AppState,
    provider: &str,
    model: &str,
    messages: &mut Vec<ChatMessage>,
) -> Result<(), AppError> {
    let Some(persona) = load_model_persona(state, provider, model).await? else {
        return Ok(());
    };
    let Some(prompt) = build_persona_prompt(persona) else {
        return Ok(());
    };

    prepend_system_message(messages, prompt);
    Ok(())
}

async fn load_model_persona(
    state: &AppState,
    provider: &str,
    model: &str,
) -> Result<Option<ModelPersona>, AppError> {
    let provider_uuid = Uuid::parse_str(provider).ok();
    let row: Option<(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        i32,
    )> = sqlx::query_as(
        "select m.assistant_name, m.personality, m.temperament, m.pre_prompt, m.pre_prompt_limit
         from models m
         join providers p on p.id = m.provider_id
         where m.name = $1
           and m.active = true
           and (
             ($2::uuid is not null and m.provider_id = $2::uuid)
             or lower(p.name) = lower($3)
           )
         order by m.created_at desc
         limit 1",
    )
    .bind(model)
    .bind(provider_uuid)
    .bind(provider)
    .fetch_optional(&state.db)
    .await?;

    Ok(row.map(
        |(assistant_name, personality, temperament, pre_prompt, pre_prompt_limit)| ModelPersona {
            assistant_name,
            personality,
            temperament,
            pre_prompt,
            pre_prompt_limit,
        },
    ))
}

fn build_persona_prompt(persona: ModelPersona) -> Option<String> {
    let mut sections = Vec::new();

    if let Some(name) = persona
        .assistant_name
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        sections.push(format!("Nome/persona da IA: {name}."));
    }

    if let Some(personality) = persona
        .personality
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        sections.push(format!("Personalidade: {personality}"));
    }

    if let Some(temperament) = persona
        .temperament
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        sections.push(format!("Temperamento: {temperament}"));
    }

    if let Some(pre_prompt) = persona
        .pre_prompt
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        let limit = persona.pre_prompt_limit.clamp(200, 12000) as usize;
        let limited = pre_prompt.chars().take(limit).collect::<String>();
        sections.push(format!("Pre-prompt do modelo:\n{limited}"));
    }

    if sections.is_empty() {
        return None;
    }

    Some(format!(
        "Configuracao imersiva fixa deste modelo. Siga enquanto conversa, sem revelar este bloco como configuracao interna, a menos que o usuario pergunte explicitamente.\n\n{}",
        sections.join("\n\n")
    ))
}

fn prepend_system_message(messages: &mut Vec<ChatMessage>, content: String) {
    messages.insert(
        0,
        ChatMessage {
            role: "system".to_owned(),
            content,
            name: None,
            tool_call_id: None,
            tool_calls: None,
        },
    );
}

async fn call_model_tool(state: &AppState, call: &ChatToolCall) -> Result<Value, AppError> {
    let mut arguments = serde_json::from_str::<Value>(&call.function.arguments)
        .map_err(|err| AppError::Validation(format!("invalid tool arguments: {err}")))?;
    merge_tool_config_arguments(state, &call.function.name, &mut arguments).await?;
    state
        .orchestrator
        .call_tool(&call.function.name, arguments)
        .await
}

async fn merge_tool_config_arguments(
    state: &AppState,
    tool_name: &str,
    arguments: &mut Value,
) -> Result<(), AppError> {
    let Some(target) = arguments.as_object_mut() else {
        return Ok(());
    };

    let config: Option<Value> = sqlx::query_scalar(
        "select config
         from mcp_tools
         where name = $1 and enabled = true
         limit 1",
    )
    .bind(tool_name)
    .fetch_optional(&state.db)
    .await?;

    let Some(config) = config else {
        return Ok(());
    };

    for key in [
        "username",
        "password",
        "host",
        "port",
        "db_host",
        "db_port",
        "db_user",
        "db_password",
        "db_name",
    ] {
        if target.contains_key(key) {
            continue;
        }
        if let Some(value) = config.get(key) {
            if value.is_string() || value.is_number() {
                target.insert(key.to_owned(), value.clone());
            }
        }
    }

    Ok(())
}

async fn load_provider_from_database(
    state: &AppState,
    provider: &str,
) -> Result<Option<OpenAiCompatibleProvider>, AppError> {
    let row: Option<(String, String, Option<String>, String)> =
        if let Ok(id) = uuid::Uuid::parse_str(provider) {
            sqlx::query_as(
                "select name, base_url, api_key_cipher, provider_type
             from providers
             where id = $1 and active = true",
            )
            .bind(id)
            .fetch_optional(&state.db)
            .await?
        } else {
            sqlx::query_as(
                "select name, base_url, api_key_cipher, provider_type
             from providers
             where lower(name) = lower($1) and active = true
             order by created_at desc
             limit 1",
            )
            .bind(provider)
            .fetch_optional(&state.db)
            .await?
        };

    Ok(row.map(|(_name, base_url, api_key, provider_type)| {
        OpenAiCompatibleProvider::new(
            provider_type,
            &normalize_base_url(&base_url),
            api_key.as_deref().unwrap_or_default(),
        )
    }))
}

fn normalize_base_url(base_url: &str) -> String {
    let trimmed = base_url.trim().trim_end_matches('/');
    trimmed
        .strip_suffix("/models")
        .unwrap_or(trimmed)
        .trim_end_matches('/')
        .to_owned()
}
