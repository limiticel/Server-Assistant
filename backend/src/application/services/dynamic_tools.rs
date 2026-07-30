use std::env;
use std::fs;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::Duration;

use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, Method,
};
use serde_json::{json, Map, Value};
use sqlx::PgPool;

use crate::shared::AppError;

#[derive(Clone, Debug)]
struct DynamicTool {
    name: String,
    description: String,
    input_schema: Value,
    config: Value,
}

pub async fn list_tools(db: &PgPool) -> Result<Value, AppError> {
    let rows: Vec<(String, String, Value)> = sqlx::query_as(
        "select name, description, input_schema
         from mcp_tools
         where enabled = true
         order by name",
    )
    .fetch_all(db)
    .await?;

    Ok(Value::Array(
        rows.into_iter()
            .map(|(name, description, input_schema)| {
                json!({
                    "name": name,
                    "description": description,
                    "input_schema": schema_for_model(&input_schema),
                    "route": format!("/api/mcp/tools/{name}/call"),
                })
            })
            .collect(),
    ))
}

pub async fn call_tool(db: &PgPool, name: &str, arguments: Value) -> Result<Value, AppError> {
    let tool = load_tool(db, name).await?;
    let mut arguments = normalize_arguments(arguments);
    apply_schema_defaults(&tool, &mut arguments);
    apply_infra_connection_test_defaults(&tool, &mut arguments);
    validate_required_arguments(&tool, &arguments)?;

    let kind =
        tool.config
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or(match tool.config.get("url") {
                Some(_) => "api",
                None => "static",
            });

    let result = match kind {
        "api" | "http" => call_api_tool(&tool, &arguments).await?,
        "web_search" => call_web_search_tool(&tool, &arguments).await?,
        "postgres_query" => call_postgres_query_tool(db, &tool, &arguments).await?,
        "infra" => call_infra_tool(&tool, &arguments)?,
        "ssh" | "infra_ssh" => call_ssh_tool(&tool, &arguments)?,
        "abstract" | "static" | "text" => call_static_tool(&tool, &arguments),
        other => {
            return Err(AppError::Validation(format!(
                "unsupported dynamic tool kind: {other}"
            )))
        }
    };

    Ok(json!({ "result": result }))
}

fn normalize_arguments(arguments: Value) -> Value {
    if arguments.is_object() {
        arguments
    } else {
        json!({})
    }
}

fn apply_schema_defaults(tool: &DynamicTool, arguments: &mut Value) {
    let Some(target) = arguments.as_object_mut() else {
        return;
    };

    if is_simple_defaults_object(&tool.input_schema) {
        if let Some(defaults) = tool.input_schema.as_object() {
            for (name, value) in defaults {
                if !target.contains_key(name) {
                    target.insert(name.clone(), value.clone());
                }
            }
        }
        return;
    }

    let Some(properties) = tool
        .input_schema
        .get("properties")
        .and_then(Value::as_object)
    else {
        return;
    };

    for (name, schema) in properties {
        if target.contains_key(name) {
            continue;
        }
        if let Some(default) = schema.get("default") {
            target.insert(name.clone(), default.clone());
        }
    }
}

pub fn schema_for_model(input_schema: &Value) -> Value {
    if !is_simple_defaults_object(input_schema) {
        return input_schema.clone();
    }

    let properties = input_schema
        .as_object()
        .map(|defaults| {
            defaults
                .iter()
                .map(|(name, value)| {
                    (
                        name.clone(),
                        json!({
                            "type": json_schema_type(name, value),
                            "description": simple_parameter_description(name)
                        }),
                    )
                })
                .collect::<Map<String, Value>>()
        })
        .unwrap_or_default();

    json!({
        "type": "object",
        "properties": properties,
        "required": []
    })
}

fn is_simple_defaults_object(input_schema: &Value) -> bool {
    input_schema.is_object()
        && input_schema.get("type").is_none()
        && input_schema.get("properties").is_none()
}

fn json_schema_type(name: &str, value: &Value) -> &'static str {
    if matches!(name, "port" | "timeout_seconds") {
        return "integer";
    }

    match value {
        Value::Bool(_) => "boolean",
        Value::Number(number) if number.is_i64() || number.is_u64() => "integer",
        Value::Number(_) => "number",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
        Value::Null | Value::String(_) => "string",
    }
}

fn simple_parameter_description(name: &str) -> String {
    match name {
        "runtime" => {
            "Tipo de execucao. Use ssh para servidor remoto ou local para maquina local.".to_owned()
        }
        "host" => "Endereco do servidor, sem porta. Exemplo: 127.0.0.1.".to_owned(),
        "port" => "Porta do servidor. Envie neste campo, nunca junto com host.".to_owned(),
        "username" => "Usuario configurado para acesso.".to_owned(),
        "password" => {
            "Senha configurada para acesso. Omita se ja estiver salva como default.".to_owned()
        }
        "private_key_path" => "Caminho da chave privada, se usado.".to_owned(),
        "command" => "Comando de linha unica para executar.".to_owned(),
        "timeout_seconds" => "Tempo maximo de execucao em segundos.".to_owned(),
        other => format!("Parametro {other} configurado para esta ferramenta."),
    }
}

fn apply_infra_connection_test_defaults(tool: &DynamicTool, arguments: &mut Value) {
    if !is_infra_tool(tool) {
        return;
    }

    let Some(target) = arguments.as_object_mut() else {
        return;
    };

    if !target.contains_key("runtime") {
        target.insert("runtime".to_owned(), Value::String("ssh".to_owned()));
    }
    if !target.contains_key("command") {
        target.insert("command".to_owned(), Value::String("hostname".to_owned()));
    }
}

fn is_infra_tool(tool: &DynamicTool) -> bool {
    matches!(
        tool.config.get("kind").and_then(Value::as_str),
        Some("infra" | "ssh" | "infra_ssh")
    )
}

async fn load_tool(db: &PgPool, name: &str) -> Result<DynamicTool, AppError> {
    let row: Option<(String, String, Value, Value)> = sqlx::query_as(
        "select name, description, input_schema, config
         from mcp_tools
         where name = $1 and enabled = true
         limit 1",
    )
    .bind(name)
    .fetch_optional(db)
    .await?;

    row.map(|(name, description, input_schema, config)| DynamicTool {
        name,
        description,
        input_schema,
        config,
    })
    .ok_or(AppError::NotFound)
}

fn validate_required_arguments(tool: &DynamicTool, arguments: &Value) -> Result<(), AppError> {
    let Some(required) = tool.input_schema.get("required").and_then(Value::as_array) else {
        return Ok(());
    };

    for item in required {
        let Some(name) = item.as_str() else {
            continue;
        };

        if arguments.get(name).is_none() {
            return Err(AppError::Validation(format!(
                "missing required parameter for {}: {name}",
                tool.name
            )));
        }
    }

    Ok(())
}

fn call_infra_tool(tool: &DynamicTool, arguments: &Value) -> Result<Value, AppError> {
    let runtime = arguments
        .get("runtime")
        .or_else(|| arguments.get("driver"))
        .or_else(|| arguments.get("protocol"))
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default();

    match runtime {
        "local" | "shell" | "powershell" | "cmd" => call_local_command_tool(tool, arguments),
        "ssh" | "infra_ssh" => call_ssh_tool(tool, arguments),
        "" if arguments.get("host").is_some() || arguments.get("username").is_some() => {
            call_ssh_tool(tool, arguments)
        }
        "" => Err(AppError::Validation(
            "infra tool requires runtime. Send runtime local/ssh or configure a runtime default in Parametros esperados.".to_owned(),
        )),
        other => Err(AppError::Validation(format!(
            "unsupported infra runtime: {other}"
        ))),
    }
}

fn call_local_command_tool(tool: &DynamicTool, arguments: &Value) -> Result<Value, AppError> {
    let command = required_command(arguments)?;
    let timeout_seconds = timeout_seconds(&tool.config, arguments);
    let output =
        run_local_command(command, timeout_seconds).map_err(|err| AppError::Upstream(err))?;

    Ok(json!({
        "tool": tool.name,
        "runtime": "local",
        "success": output.status.success(),
        "exit_code": output.status.code(),
        "stdout": String::from_utf8_lossy(&output.stdout),
        "stderr": String::from_utf8_lossy(&output.stderr),
        "timeout_seconds": timeout_seconds,
    }))
}

fn call_ssh_tool(tool: &DynamicTool, arguments: &Value) -> Result<Value, AppError> {
    let command = required_command(arguments)?;

    let username = optional_string(arguments, "username")
        .or_else(|| optional_string(&tool.config, "username"))
        .ok_or_else(|| AppError::Validation("ssh tool requires username. Send username or configure a username default in Parametros esperados.".to_owned()))?;
    let host = optional_string(arguments, "host")
        .or_else(|| optional_string(&tool.config, "host"))
        .ok_or_else(|| AppError::Validation("ssh tool requires host. Send host or configure a host default in Parametros esperados.".to_owned()))?;
    let explicit_port = optional_u16(arguments, "port")
        .or_else(|| optional_u16(&tool.config, "port"))
        .map(|value| value.to_string());
    let (host, port) = split_host_port(&host, explicit_port.as_deref().unwrap_or("22"));
    let password = optional_string(arguments, "password")
        .or_else(|| optional_string(&tool.config, "password"));
    let private_key_path = optional_string(arguments, "private_key_path")
        .or_else(|| optional_string(&tool.config, "private_key_path"));
    let timeout_seconds = timeout_seconds(&tool.config, arguments);

    let output = run_ssh_command(
        &username,
        &host,
        port,
        command,
        password.as_deref(),
        private_key_path.as_deref(),
        timeout_seconds,
    )
    .map_err(AppError::Upstream)?;

    Ok(json!({
        "tool": tool.name,
        "runtime": "ssh",
        "success": output.status.success(),
        "exit_code": output.status.code(),
        "stdout": String::from_utf8_lossy(&output.stdout),
        "stderr": String::from_utf8_lossy(&output.stderr),
        "host": host,
        "port": port,
        "timeout_seconds": timeout_seconds,
    }))
}

fn required_command(arguments: &Value) -> Result<&str, AppError> {
    let command = arguments
        .get("command")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::Validation("infra tool requires command".to_owned()))?;

    if command.contains('\n') || command.contains('\r') {
        return Err(AppError::Validation(
            "infra command must be a single line".to_owned(),
        ));
    }

    Ok(command)
}

fn timeout_seconds(config: &Value, arguments: &Value) -> u64 {
    arguments
        .get("timeout_seconds")
        .or_else(|| config.get("timeout_seconds"))
        .and_then(Value::as_u64)
        .unwrap_or(30)
        .clamp(1, 120)
}

async fn call_web_search_tool(tool: &DynamicTool, arguments: &Value) -> Result<Value, AppError> {
    let query = arguments
        .get("query")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::Validation("web_search requires query".to_owned()))?;
    let max_results = arguments
        .get("max_results")
        .and_then(Value::as_u64)
        .unwrap_or(5)
        .clamp(1, 8);

    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|err| AppError::Internal(err.to_string()))?;

    let response = client
        .get("https://api.duckduckgo.com/")
        .query(&[
            ("q", query),
            ("format", "json"),
            ("no_html", "1"),
            ("skip_disambig", "1"),
        ])
        .send()
        .await
        .map_err(|err| AppError::Upstream(format!("web_search failed: {err}")))?;

    let status = response.status();
    let body = response
        .json::<Value>()
        .await
        .map_err(|err| AppError::Upstream(format!("invalid web_search response: {err}")))?;
    let mut results = Vec::new();
    collect_duckduckgo_topics(
        body.get("RelatedTopics"),
        &mut results,
        max_results as usize,
    );

    Ok(json!({
        "tool": tool.name,
        "query": query,
        "status": status.as_u16(),
        "source": "duckduckgo_instant_answer",
        "heading": body.get("Heading").and_then(Value::as_str).unwrap_or_default(),
        "abstract": body.get("AbstractText").and_then(Value::as_str).unwrap_or_default(),
        "abstract_url": body.get("AbstractURL").and_then(Value::as_str).unwrap_or_default(),
        "official_website": body.get("OfficialWebsite").and_then(Value::as_str).unwrap_or_default(),
        "results": results,
    }))
}

fn collect_duckduckgo_topics(topics: Option<&Value>, results: &mut Vec<Value>, max_results: usize) {
    if results.len() >= max_results {
        return;
    }

    let Some(items) = topics.and_then(Value::as_array) else {
        return;
    };

    for item in items {
        if results.len() >= max_results {
            return;
        }

        if let Some(nested) = item.get("Topics") {
            collect_duckduckgo_topics(Some(nested), results, max_results);
            continue;
        }

        let text = item.get("Text").and_then(Value::as_str).unwrap_or_default();
        let url = item
            .get("FirstURL")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if !text.is_empty() || !url.is_empty() {
            results.push(json!({ "text": text, "url": url }));
        }
    }
}

async fn call_postgres_query_tool(
    db: &PgPool,
    tool: &DynamicTool,
    arguments: &Value,
) -> Result<Value, AppError> {
    let query = arguments
        .get("query")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::Validation("postgres_query requires query".to_owned()))?;
    let trimmed = query.trim().trim_end_matches(';').trim();
    let lower = trimmed.to_lowercase();

    if !lower.starts_with("select ") {
        return Err(AppError::Validation(
            "postgres_query currently allows SELECT statements only".to_owned(),
        ));
    }
    if trimmed.contains(';') {
        return Err(AppError::Validation(
            "postgres_query does not allow multiple statements".to_owned(),
        ));
    }

    let wrapped = format!(
        "select coalesce(json_agg(row_to_json(q)), '[]'::json) from (select * from ({trimmed}) source limit 100) q"
    );
    let rows: Value = sqlx::query_scalar(&wrapped).fetch_one(db).await?;

    Ok(json!({
        "tool": tool.name,
        "query": trimmed,
        "rows": rows,
    }))
}

async fn call_api_tool(tool: &DynamicTool, arguments: &Value) -> Result<Value, AppError> {
    let method = tool
        .config
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or("GET")
        .parse::<Method>()
        .map_err(|err| AppError::Validation(format!("invalid API method: {err}")))?;

    let url_template = tool
        .config
        .get("url")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::Validation("API tool config.url is required".to_owned()))?;
    let url = render_template(url_template, arguments);

    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(AppError::Validation(
            "API tool URL must start with http:// or https://".to_owned(),
        ));
    }

    let timeout_seconds = tool
        .config
        .get("timeout_seconds")
        .and_then(Value::as_u64)
        .unwrap_or(30)
        .clamp(1, 120);
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout_seconds))
        .build()
        .map_err(|err| AppError::Internal(err.to_string()))?;

    let mut request = client
        .request(method.clone(), url)
        .headers(build_headers(tool.config.get("headers"), arguments)?);

    if let Some(query) = tool.config.get("query") {
        request = request.query(&json_to_string_pairs(&render_value(query, arguments))?);
    } else if method == Method::GET {
        request = request.query(&json_to_string_pairs(arguments)?);
    }

    if let Some(body) = tool.config.get("body") {
        request = request.json(&render_value(body, arguments));
    } else if method != Method::GET {
        request = request.json(arguments);
    }

    let response = request
        .send()
        .await
        .map_err(|err| AppError::Upstream(format!("API tool {} failed: {err}", tool.name)))?;

    let status = response.status();
    let headers = response
        .headers()
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.to_string(), Value::String(value.to_owned())))
        })
        .collect::<Map<String, Value>>();
    let text = response
        .text()
        .await
        .map_err(|err| AppError::Upstream(err.to_string()))?;
    let body = serde_json::from_str::<Value>(&text).unwrap_or(Value::String(text));

    Ok(json!({
        "tool": tool.name,
        "status": status.as_u16(),
        "success": status.is_success(),
        "headers": headers,
        "body": body,
    }))
}

fn run_ssh_command(
    username: &str,
    host: &str,
    port: &str,
    command: &str,
    password: Option<&str>,
    private_key_path: Option<&str>,
    timeout_seconds: u64,
) -> Result<Output, String> {
    let destination = format!("{username}@{host}");

    if let Some(password) = password {
        if command_exists("plink") {
            let mut process = Command::new("plink");
            process
                .arg("-ssh")
                .arg("-P")
                .arg(port)
                .arg("-batch")
                .arg("-pw")
                .arg(password);
            if let Some(path) = private_key_path {
                process.arg("-i").arg(path);
            }
            process.arg(destination).arg(command);
            return run_command_with_timeout(process, timeout_seconds)
                .map_err(|err| format!("failed to execute plink: {err}"));
        }

        if command_exists("sshpass") {
            let mut process = Command::new("sshpass");
            process
                .arg("-p")
                .arg(password)
                .arg("ssh")
                .arg("-p")
                .arg(port)
                .arg("-o")
                .arg("StrictHostKeyChecking=accept-new")
                .arg("-o")
                .arg(format!("ConnectTimeout={}", timeout_seconds.min(10)));
            if let Some(path) = private_key_path {
                process.arg("-i").arg(path);
            }
            process.arg(destination).arg(command);
            return run_command_with_timeout(process, timeout_seconds)
                .map_err(|err| format!("failed to execute sshpass: {err}"));
        }

        return run_ssh_with_askpass(
            &destination,
            port,
            command,
            password,
            private_key_path,
            timeout_seconds,
        );
    }

    let mut process = Command::new("ssh");
    process
        .arg("-p")
        .arg(port)
        .arg("-o")
        .arg("BatchMode=yes")
        .arg("-o")
        .arg("StrictHostKeyChecking=accept-new")
        .arg("-o")
        .arg(format!("ConnectTimeout={}", timeout_seconds.min(10)));
    if let Some(path) = private_key_path {
        process.arg("-i").arg(path);
    }
    process.arg(destination).arg(command);

    run_command_with_timeout(process, timeout_seconds)
        .map_err(|err| format!("failed to execute ssh: {err}"))
}

fn run_local_command(command: &str, timeout_seconds: u64) -> Result<Output, String> {
    let process = if cfg!(windows) {
        let mut process = Command::new("powershell");
        process.arg("-NoProfile").arg("-Command").arg(command);
        process
    } else {
        let mut process = Command::new("sh");
        process.arg("-lc").arg(command);
        process
    };

    run_command_with_timeout(process, timeout_seconds)
        .map_err(|err| format!("failed to execute local command: {err}"))
}

fn run_ssh_with_askpass(
    destination: &str,
    port: &str,
    command: &str,
    password: &str,
    private_key_path: Option<&str>,
    timeout_seconds: u64,
) -> Result<Output, String> {
    let askpass_path = write_askpass_script()?;
    let mut process = Command::new("ssh");
    process
        .arg("-p")
        .arg(port)
        .arg("-o")
        .arg("StrictHostKeyChecking=accept-new")
        .arg("-o")
        .arg("PreferredAuthentications=password")
        .arg("-o")
        .arg("PubkeyAuthentication=no")
        .arg("-o")
        .arg("NumberOfPasswordPrompts=1")
        .arg("-o")
        .arg(format!("ConnectTimeout={}", timeout_seconds.min(10)));
    if let Some(path) = private_key_path {
        process.arg("-i").arg(path);
    }
    process
        .arg(destination)
        .arg(command)
        .env("SSH_ASKPASS", &askpass_path)
        .env("SSH_ASKPASS_REQUIRE", "force")
        .env("SSH_PASSWORD", password)
        .env("DISPLAY", "server-assistant")
        .stdin(Stdio::null());

    let output = run_command_with_timeout(process, timeout_seconds)
        .map_err(|err| format!("failed to execute ssh with askpass: {err}"));

    let _ = fs::remove_file(&askpass_path);
    output
}

fn run_command_with_timeout(mut command: Command, timeout_seconds: u64) -> Result<Output, String> {
    let timeout = Duration::from_secs(timeout_seconds.max(1));
    let start = std::time::Instant::now();
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| err.to_string())?;

    loop {
        match child.try_wait() {
            Ok(Some(_)) => return child.wait_with_output().map_err(|err| err.to_string()),
            Ok(None) if start.elapsed() >= timeout => {
                let _ = child.kill();
                let output = child.wait_with_output().map_err(|err| err.to_string())?;
                return Err(format!(
                    "command timed out after {timeout_seconds}s. stdout: {} stderr: {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            Ok(None) => thread::sleep(Duration::from_millis(100)),
            Err(err) => return Err(err.to_string()),
        }
    }
}

fn write_askpass_script() -> Result<std::path::PathBuf, String> {
    let extension = if cfg!(windows) { "cmd" } else { "sh" };
    let path = env::temp_dir().join(format!(
        "server-assistant-ssh-askpass-{}.{}",
        std::process::id(),
        extension
    ));

    let content = if cfg!(windows) {
        "@echo off\r\necho %SSH_PASSWORD%\r\n"
    } else {
        "#!/bin/sh\nprintf '%s\\n' \"$SSH_PASSWORD\"\n"
    };

    fs::write(&path, content).map_err(|err| format!("failed to create askpass script: {err}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&path)
            .map_err(|err| format!("failed to read askpass permissions: {err}"))?
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&path, permissions)
            .map_err(|err| format!("failed to set askpass permissions: {err}"))?;
    }

    Ok(path)
}

fn command_exists(command: &str) -> bool {
    let probe = if cfg!(windows) { "where" } else { "which" };
    let mut process = Command::new(probe);
    if cfg!(windows) {
        process.arg(command);
    } else {
        process.arg(command);
    }

    process
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn optional_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn split_host_port<'a>(host: &'a str, fallback_port: &'a str) -> (String, &'a str) {
    let Some((raw_host, raw_port)) = host.rsplit_once(':') else {
        return (host.to_owned(), fallback_port);
    };

    if raw_host.is_empty() || raw_host.contains(':') || raw_port.parse::<u16>().is_err() {
        return (host.to_owned(), fallback_port);
    }

    (raw_host.to_owned(), raw_port)
}

fn optional_u16(value: &Value, key: &str) -> Option<u16> {
    let value = value.get(key)?;
    if let Some(number) = value.as_u64() {
        return u16::try_from(number).ok();
    }

    value
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<u16>().ok())
}

fn build_headers(headers: Option<&Value>, arguments: &Value) -> Result<HeaderMap, AppError> {
    let mut map = HeaderMap::new();
    let Some(headers) = headers.and_then(Value::as_object) else {
        return Ok(map);
    };

    for (name, value) in headers {
        let Some(value) = value.as_str() else {
            continue;
        };
        let header_name = HeaderName::from_bytes(name.as_bytes())
            .map_err(|err| AppError::Validation(format!("invalid header name {name}: {err}")))?;
        let rendered = render_template(value, arguments);
        let header_value = HeaderValue::from_str(&rendered).map_err(|err| {
            AppError::Validation(format!("invalid header value for {name}: {err}"))
        })?;
        map.insert(header_name, header_value);
    }

    Ok(map)
}

fn json_to_string_pairs(value: &Value) -> Result<Vec<(String, String)>, AppError> {
    let Some(object) = value.as_object() else {
        return Ok(Vec::new());
    };

    Ok(object
        .iter()
        .filter_map(|(key, value)| {
            if value.is_null() {
                None
            } else {
                Some((key.clone(), value_to_string(value)))
            }
        })
        .collect())
}

fn call_static_tool(tool: &DynamicTool, arguments: &Value) -> Value {
    let template = tool
        .config
        .get("static_response")
        .or_else(|| tool.config.get("response"))
        .or_else(|| tool.config.get("instructions"))
        .and_then(Value::as_str)
        .unwrap_or(&tool.description);

    json!({
        "tool": tool.name,
        "text": render_template(template, arguments),
    })
}

fn render_value(value: &Value, arguments: &Value) -> Value {
    match value {
        Value::String(text) => Value::String(render_template(text, arguments)),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| render_value(item, arguments))
                .collect(),
        ),
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, value)| (key.clone(), render_value(value, arguments)))
                .collect(),
        ),
        value => value.clone(),
    }
}

fn render_template(template: &str, arguments: &Value) -> String {
    let mut rendered = template.to_owned();
    let Some(object) = arguments.as_object() else {
        return rendered;
    };

    for (key, value) in object {
        rendered = rendered.replace(&format!("{{{{{key}}}}}"), &value_to_string(value));
    }

    rendered
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}
