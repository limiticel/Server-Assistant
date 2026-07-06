use serde_json::{json, Value};

use crate::registry::{object_schema, Registry, Tool, ToolResult};

pub fn register(registry: &mut Registry) {
    registry.register(Tool {
        name: "gerar_scaffold_api".to_string(),
        description: "Gera codigo scaffold de uma API REST (FastAPI) a partir de uma descricao do recurso.".to_string(),
        roles: vec!["dev".to_string(), "admin".to_string()],
        input_schema: object_schema(
            json!({
                "recurso": { "type": "string", "description": "Nome do recurso. Ex: 'Produto', 'Pedido', 'Cliente'" },
                "campos": { "type": "string", "description": "Campos separados por virgula. Ex: 'nome:str, preco:float, ativo:bool'" }
            }),
            vec!["recurso", "campos"],
        ),
        handler: gerar_scaffold_api,
    });

    registry.register(Tool {
        name: "gerar_schema_openapi".to_string(),
        description: "Gera um schema OpenAPI 3.0 (JSON) para um endpoint descrito em linguagem natural.".to_string(),
        roles: vec!["dev".to_string(), "admin".to_string()],
        input_schema: object_schema(
            json!({
                "descricao_endpoint": {
                    "type": "string",
                    "description": "Descreva o endpoint em portugues. Ex: 'POST /pedidos que recebe cliente_id e lista de itens e retorna o pedido criado com total'"
                }
            }),
            vec!["descricao_endpoint"],
        ),
        handler: gerar_schema_openapi,
    });

    registry.register(Tool {
        name: "gerar_bot_whatsapp".to_string(),
        description: "Gera codigo base de um bot WhatsApp (via Evolution API ou Baileys) com os fluxos descritos.".to_string(),
        roles: vec!["dev".to_string(), "admin".to_string()],
        input_schema: object_schema(
            json!({
                "nome_bot": { "type": "string", "description": "Nome do bot" },
                "fluxos": { "type": "string", "description": "Descreva os fluxos. Ex: 'boas-vindas, consulta de pedido, falar com humano'" }
            }),
            vec!["nome_bot", "fluxos"],
        ),
        handler: gerar_bot_whatsapp,
    });

    registry.register(Tool {
        name: "gerar_webhook_handler".to_string(),
        description: "Gera um handler FastAPI para receber webhooks de um servico externo.".to_string(),
        roles: vec!["dev".to_string(), "admin".to_string()],
        input_schema: object_schema(
            json!({
                "servico": { "type": "string", "description": "Nome do servico. Ex: 'Stripe', 'GitHub', 'Evolution API'" },
                "eventos": { "type": "string", "description": "Eventos esperados separados por virgula. Ex: 'payment.success, payment.failed'" }
            }),
            vec!["servico", "eventos"],
        ),
        handler: gerar_webhook_handler,
    });
}

fn gerar_scaffold_api(args: &Value, _registry: &Registry) -> ToolResult {
    let recurso = required_string(args, "recurso")?;
    let campos = required_string(args, "campos")?;
    let recurso_lower = recurso.to_lowercase();

    let parsed: Vec<(String, String)> = campos
        .split(',')
        .map(|campo| {
            let mut parts = campo.trim().split(':');
            let nome = parts.next().unwrap_or_default().trim().to_string();
            let tipo = parts.next().unwrap_or("str").trim().to_string();
            (nome, tipo)
        })
        .filter(|(nome, _)| !nome.is_empty())
        .collect();

    let campos_model = parsed
        .iter()
        .map(|(nome, tipo)| format!("    {nome}: {tipo}"))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(json!(format!(
        r#"# Scaffold gerado para: {recurso}
from fastapi import APIRouter, HTTPException
from pydantic import BaseModel
from typing import List

router = APIRouter(prefix="/{recurso_lower}s", tags=["{recurso}"])


class {recurso}(BaseModel):
{campos_model}


class {recurso}Create({recurso}):
    pass


_db: dict[int, {recurso}] = {{}}
_seq = 0


@router.get("/", response_model=List[{recurso}])
async def listar():
    return list(_db.values())


@router.get("/{{id}}", response_model={recurso})
async def obter(id: int):
    if id not in _db:
        raise HTTPException(404, "{recurso} nao encontrado")
    return _db[id]


@router.post("/", response_model={recurso}, status_code=201)
async def criar(payload: {recurso}Create):
    global _seq
    _seq += 1
    _db[_seq] = payload
    return payload


@router.delete("/{{id}}", status_code=204)
async def deletar(id: int):
    if id not in _db:
        raise HTTPException(404, "{recurso} nao encontrado")
    del _db[id]
"#
    )))
}

fn gerar_schema_openapi(args: &Value, _registry: &Registry) -> ToolResult {
    let descricao_endpoint = required_string(args, "descricao_endpoint")?;
    Ok(json!({
        "openapi": "3.0.0",
        "info": { "title": "API Gerada", "version": "1.0.0" },
        "paths": {
            "/exemplo": {
                "post": {
                    "summary": descricao_endpoint,
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "description": "Substitua com o schema real"
                                }
                            }
                        }
                    },
                    "responses": { "201": { "description": "Criado com sucesso" } }
                }
            }
        }
    }))
}

fn gerar_bot_whatsapp(args: &Value, _registry: &Registry) -> ToolResult {
    let nome_bot = required_string(args, "nome_bot")?;
    let fluxos = required_string(args, "fluxos")?;

    let fluxo_list: Vec<&str> = fluxos.split(',').map(str::trim).collect();

    let handlers = fluxo_list
        .iter()
        .map(|f| {
            let fn_name = f.replace(' ', "_").to_lowercase();
            format!(
                "async def fluxo_{fn_name}(message):\n    # TODO: implementar fluxo \"{f}\"\n    await send_message(message[\"from\"], \"Fluxo {f} em construção.\")"
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let routing = fluxo_list
        .iter()
        .map(|f| {
            let fn_name = f.replace(' ', "_").to_lowercase();
            let f_lower = f.to_lowercase();
            format!("    if \"{f_lower}\" in text:\n        await fluxo_{fn_name}(message)")
        })
        .collect::<Vec<_>>()
        .join("\n");

    Ok(json!(format!(
        r#"# Bot WhatsApp — {nome_bot}
# Gerado automaticamente pelo MCP DevTools
# Integração: Evolution API (substitua pelo seu provider)

import httpx

EVOLUTION_URL = "https://sua-evolution-api.com"
INSTANCE = "sua-instancia"
API_KEY = "sua-api-key"


async def send_message(to: str, text: str):
    async with httpx.AsyncClient() as client:
        await client.post(
            f"{{EVOLUTION_URL}}/message/sendText/{{INSTANCE}}",
            json={{"number": to, "text": text}},
            headers={{"apikey": API_KEY}},
        )


async def handle_message(message: dict):
    text = message.get("body", "").strip().lower()

    # Roteador de fluxos
{routing}
    else:
        await send_message(message["from"], "Olá! Como posso ajudar?")


# ─── Fluxos ───────────────────────────────────────────────────────────────────
{handlers}
"#
    )))
}

fn gerar_webhook_handler(args: &Value, _registry: &Registry) -> ToolResult {
    let servico = required_string(args, "servico")?;
    let eventos = required_string(args, "eventos")?;

    let evento_list: Vec<&str> = eventos.split(',').map(str::trim).collect();

    let handlers = evento_list
        .iter()
        .map(|e| {
            let fn_name = e.replace('.', "_").replace('-', "_");
            format!(
                "async def handle_{fn_name}(payload: dict):\n    # TODO: processar evento \"{e}\"\n    pass"
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let routing = evento_list
        .iter()
        .map(|e| {
            let fn_name = e.replace('.', "_").replace('-', "_");
            format!("elif event_type == \"{e}\":\n        await handle_{fn_name}(payload)")
        })
        .collect::<Vec<_>>()
        .join("\n    ");

    let servico_lower = servico.to_lowercase();

    Ok(json!(format!(
        r#"# Webhook Handler — {servico}
from fastapi import APIRouter, HTTPException, Request

router = APIRouter(prefix="/webhooks/{servico_lower}", tags=["webhooks"])


@router.post("/")
async def receive_webhook(request: Request):
    payload = await request.json()
    event_type = payload.get("event") or payload.get("type", "unknown")

    if event_type == "unknown":
        raise HTTPException(400, "Evento não identificado")
    {routing}
    else:
        # Evento não mapeado — logue e ignore
        pass

    return {{"received": True}}


# ─── Handlers de eventos ──────────────────────────────────────────────────────
{handlers}
"#
    )))
}

fn required_string<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Invalid or missing parameter: {key}"))
}
