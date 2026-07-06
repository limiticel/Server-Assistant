// Para uso local/dev
// Transport stdio — para uso local, desenvolvimento e Claude Desktop.
// 1 cliente por processo. Não usar em produção multi-usuário.
use std::time::SystemTime;

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{info, warn};

use crate::auth::UserContext;
use crate::registry::Registry;
use crate::sse_transport::handle_json_rpc;

pub async fn run(registry: &Registry, server_name: &str, server_version: &str) -> Result<()> {
    info!("Iniciando servidor MCP via stdio...");

    let ctx = UserContext {
        user_id: "stdio".to_string(),
        role: "admin".to_string(),
        api_key: "stdio".to_string(),
        connected_at: SystemTime::now(),
    };

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut writer = tokio::io::BufWriter::new(stdout);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break; // EOF — cliente desconectou
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let payload = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                warn!("JSON inválido no stdio: {e}");
                continue;
            }
        };

        if let Some(response) =
            handle_json_rpc(payload, registry, &ctx, server_name, server_version)
        {
            let data = serde_json::to_string(&response)?;
            writer.write_all(data.as_bytes()).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;
        }
    }

    Ok(())
}
