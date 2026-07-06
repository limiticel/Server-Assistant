use std::process::Command;
use std::time::Duration;
use std::{env, fs};

use serde_json::{json, Value};

use crate::registry::{object_schema, Registry, Tool, ToolResult};

pub fn register(registry: &mut Registry) {
    registry.register(Tool {
        name: "ubuntu_server_ssh".to_string(),
        description: "Executa comandos no Ubuntu Server via SSH em 127.0.0.1:2222.".to_string(),
        roles: vec!["admin".to_string()],
        input_schema: object_schema(
            json!({
                "username": {
                    "type": "string",
                    "description": "Usuario SSH. Opcional se UBUNTU_SSH_DEFAULT_USER estiver configurado."
                },
                "password": {
                    "type": "string",
                    "description": "Senha SSH. Opcional; requer plink no Windows ou sshpass no Linux."
                },
                "host": {
                    "type": "string",
                    "description": "Host SSH. Padrao: 127.0.0.1."
                },
                "port": {
                    "type": "integer",
                    "description": "Porta SSH. Padrao: 2222."
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
            }),
            vec!["command"],
        ),
        handler: ubuntu_server_ssh,
    });

    registry.register(Tool {
        name: "postgres_query".to_string(),
        description:
            "Executa uma consulta SQL no PostgreSQL configurado via SSH, sem terminal interativo."
                .to_string(),
        roles: vec!["admin".to_string()],
        input_schema: object_schema(
            json!({
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
            }),
            vec!["query"],
        ),
        handler: postgres_query,
    });
}

fn ubuntu_server_ssh(args: &Value, _registry: &Registry) -> ToolResult {
    let command = required_string(args, "command")?;
    let username = optional_string(args, "username")
        .or_else(|| env::var("UBUNTU_SSH_DEFAULT_USER").ok())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            "Invalid or missing parameter: username. Configure UBUNTU_SSH_DEFAULT_USER or send username.".to_string()
        })?;

    let host = env::var("UBUNTU_SSH_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let host = optional_string(args, "host").unwrap_or(host);
    let port = optional_u16(args, "port")
        .map(|value| value.to_string())
        .or_else(|| optional_string(args, "port"))
        .or_else(|| env::var("UBUNTU_SSH_PORT").ok())
        .unwrap_or_else(|| "2222".to_string());
    let password = optional_string(args, "password")
        .or_else(|| env::var("UBUNTU_SSH_PASSWORD").ok())
        .filter(|value| !value.trim().is_empty());
    let timeout_seconds = args
        .get("timeout_seconds")
        .and_then(Value::as_u64)
        .unwrap_or(30)
        .clamp(1, 120);

    let output = run_ssh_command(
        username.trim(),
        host.trim(),
        port.trim(),
        command,
        password.as_deref(),
        timeout_seconds,
    )?;

    Ok(json!({
        "success": output.status.success(),
        "exit_code": output.status.code(),
        "stdout": String::from_utf8_lossy(&output.stdout),
        "stderr": String::from_utf8_lossy(&output.stderr),
        "host": host,
        "port": port,
        "timeout_seconds": Duration::from_secs(timeout_seconds).as_secs()
    }))
}

fn postgres_query(args: &Value, _registry: &Registry) -> ToolResult {
    let query = required_string(args, "query")?;
    let username = optional_string(args, "username")
        .or_else(|| env::var("UBUNTU_SSH_DEFAULT_USER").ok())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            "Invalid or missing parameter: username. Configure tool username.".to_string()
        })?;
    let password = optional_string(args, "password")
        .or_else(|| env::var("UBUNTU_SSH_PASSWORD").ok())
        .filter(|value| !value.trim().is_empty());
    let ssh_host = optional_string(args, "host")
        .or_else(|| env::var("UBUNTU_SSH_HOST").ok())
        .unwrap_or_else(|| "127.0.0.1".to_string());
    let ssh_port = optional_u16(args, "port")
        .map(|value| value.to_string())
        .or_else(|| optional_string(args, "port"))
        .or_else(|| env::var("UBUNTU_SSH_PORT").ok())
        .unwrap_or_else(|| "2222".to_string());

    let db_host = optional_string(args, "db_host")
        .or_else(|| env::var("POSTGRES_HOST").ok())
        .unwrap_or_else(|| "127.0.0.1".to_string());
    let db_port = optional_u16(args, "db_port")
        .map(|value| value.to_string())
        .or_else(|| optional_string(args, "db_port"))
        .or_else(|| env::var("POSTGRES_PORT").ok())
        .unwrap_or_else(|| "5432".to_string());
    let db_user = optional_string(args, "db_user")
        .or_else(|| env::var("POSTGRES_USER").ok())
        .ok_or_else(|| {
            "Invalid or missing parameter: db_user. Configure tool db_user.".to_string()
        })?;
    let db_password = optional_string(args, "db_password")
        .or_else(|| env::var("POSTGRES_PASSWORD").ok())
        .ok_or_else(|| {
            "Invalid or missing parameter: db_password. Configure tool db_password.".to_string()
        })?;
    let db_name = optional_string(args, "db_name")
        .or_else(|| env::var("POSTGRES_DB").ok())
        .ok_or_else(|| {
            "Invalid or missing parameter: db_name. Configure tool db_name.".to_string()
        })?;
    let timeout_seconds = args
        .get("timeout_seconds")
        .and_then(Value::as_u64)
        .unwrap_or(30)
        .clamp(1, 120);

    let command = format!(
        "PGPASSWORD={} psql -h {} -p {} -U {} -d {} -v ON_ERROR_STOP=1 -c {}",
        shell_quote(&db_password),
        shell_quote(&db_host),
        shell_quote(&db_port),
        shell_quote(&db_user),
        shell_quote(&db_name),
        shell_quote(query)
    );

    let output = run_ssh_command(
        username.trim(),
        ssh_host.trim(),
        ssh_port.trim(),
        &command,
        password.as_deref(),
        timeout_seconds,
    )?;

    Ok(json!({
        "success": output.status.success(),
        "exit_code": output.status.code(),
        "stdout": String::from_utf8_lossy(&output.stdout),
        "stderr": String::from_utf8_lossy(&output.stderr),
        "database": db_name,
        "db_host": db_host,
        "db_port": db_port,
        "timeout_seconds": Duration::from_secs(timeout_seconds).as_secs()
    }))
}

fn run_ssh_command(
    username: &str,
    host: &str,
    port: &str,
    command: &str,
    password: Option<&str>,
    timeout_seconds: u64,
) -> Result<std::process::Output, String> {
    let destination = format!("{username}@{host}");
    let output = if let Some(password) = password {
        if command_exists("plink") {
            Command::new("plink")
                .arg("-ssh")
                .arg("-P")
                .arg(port)
                .arg("-batch")
                .arg("-pw")
                .arg(password)
                .arg(destination)
                .arg(command)
                .output()
                .map_err(|err| format!("Failed to execute plink: {err}"))?
        } else if command_exists("sshpass") {
            Command::new("sshpass")
                .arg("-p")
                .arg(password)
                .arg("ssh")
                .arg("-p")
                .arg(port)
                .arg("-o")
                .arg("StrictHostKeyChecking=accept-new")
                .arg("-o")
                .arg(format!("ConnectTimeout={}", timeout_seconds.min(10)))
                .arg(destination)
                .arg(command)
                .output()
                .map_err(|err| format!("Failed to execute sshpass: {err}"))?
        } else {
            run_ssh_with_askpass(&destination, port, command, password, timeout_seconds)?
        }
    } else {
        Command::new("ssh")
            .arg("-p")
            .arg(port)
            .arg("-o")
            .arg("BatchMode=yes")
            .arg("-o")
            .arg("StrictHostKeyChecking=accept-new")
            .arg("-o")
            .arg(format!("ConnectTimeout={}", timeout_seconds.min(10)))
            .arg(destination)
            .arg(command)
            .output()
            .map_err(|err| format!("Failed to execute ssh: {err}"))?
    };

    Ok(output)
}

fn run_ssh_with_askpass(
    destination: &str,
    port: &str,
    command: &str,
    password: &str,
    timeout_seconds: u64,
) -> Result<std::process::Output, String> {
    let askpass_path = write_askpass_script()?;
    let output = Command::new("ssh")
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
        .arg(format!("ConnectTimeout={}", timeout_seconds.min(10)))
        .arg(destination)
        .arg(command)
        .env("SSH_ASKPASS", &askpass_path)
        .env("SSH_ASKPASS_REQUIRE", "force")
        .env("SSH_PASSWORD", password)
        .env("DISPLAY", "server-assistant")
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|err| format!("Failed to execute ssh with askpass: {err}"));

    let _ = fs::remove_file(&askpass_path);
    output
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

    fs::write(&path, content).map_err(|err| format!("Failed to create askpass script: {err}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&path)
            .map_err(|err| format!("Failed to read askpass permissions: {err}"))?
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&path, permissions)
            .map_err(|err| format!("Failed to set askpass permissions: {err}"))?;
    }

    Ok(path)
}

fn required_string<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Invalid or missing parameter: {key}"))
}

fn optional_string(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn optional_u16(args: &Value, key: &str) -> Option<u16> {
    args.get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn command_exists(command: &str) -> bool {
    let probe = if cfg!(windows) { "where" } else { "which" };
    Command::new(probe)
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
