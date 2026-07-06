use serde_json::{json, Value};

use crate::registry::{object_schema, Registry, Tool, ToolResult};

pub fn register(registry: &mut Registry) {
    registry.register(Tool {
        name: "buscar_cliente".to_string(),
        description:
            "Busca um cliente no CRM pelo nome, e-mail ou ID. Retorna dados cadastrais e status."
                .to_string(),
        roles: vec!["sales".to_string(), "admin".to_string()],
        input_schema: object_schema(
            json!({
                "query": { "type": "string", "description": "Nome, e-mail ou ID do cliente" }
            }),
            vec!["query"],
        ),
        handler: buscar_cliente,
    });

    registry.register(Tool {
        name: "listar_oportunidades".to_string(),
        description: "Lista oportunidades de venda em aberto para um cliente ou para toda a carteira.".to_string(),
        roles: vec!["sales".to_string(), "admin".to_string()],
        input_schema: object_schema(
            json!({
                "cliente_id": { "type": "string", "description": "ID do cliente opcional" },
                "status": { "type": "string", "description": "Filtro: 'aberta' | 'ganha' | 'perdida'", "default": "aberta" }
            }),
            vec![],
        ),
        handler: listar_oportunidades,
    });

    registry.register(Tool {
        name: "registrar_interacao".to_string(),
        description: "Registra uma interacao no historico do cliente.".to_string(),
        roles: vec!["sales".to_string(), "admin".to_string()],
        input_schema: object_schema(
            json!({
                "cliente_id": { "type": "string" },
                "tipo": { "type": "string", "description": "'ligacao' | 'email' | 'reuniao' | 'outro'" },
                "descricao": { "type": "string" }
            }),
            vec!["cliente_id", "tipo", "descricao"],
        ),
        handler: registrar_interacao,
    });

    registry.register(Tool {
        name: "buscar_documentacao".to_string(),
        description: "Busca na documentacao interna dos sistemas da empresa.".to_string(),
        roles: vec!["sales".to_string(), "dev".to_string(), "admin".to_string()],
        input_schema: object_schema(
            json!({
                "pergunta": { "type": "string", "description": "Duvida ou termo a buscar na documentacao" },
                "sistema": { "type": "string", "description": "Sistema especifico opcional: 'crm' | 'erp' | 'portal'" }
            }),
            vec!["pergunta"],
        ),
        handler: buscar_documentacao,
    });

    registry.register(Tool {
        name: "consultar_politica_comercial".to_string(),
        description: "Retorna regras de desconto, condicoes de pagamento e politicas de venda vigentes.".to_string(),
        roles: vec!["sales".to_string(), "admin".to_string()],
        input_schema: object_schema(
            json!({
                "topico": { "type": "string", "description": "Ex: 'desconto maximo', 'prazo de pagamento', 'comissao'" }
            }),
            vec!["topico"],
        ),
        handler: consultar_politica_comercial,
    });
}

fn buscar_cliente(args: &Value, _registry: &Registry) -> ToolResult {
    let query = required_string(args, "query")?;
    Ok(json!({
        "clientes": [
            { "id": "001", "nome": "Empresa Exemplo", "status": "ativo", "contato": "joao@exemplo.com" }
        ],
        "total": 1,
        "query": query,
    }))
}

fn listar_oportunidades(args: &Value, _registry: &Registry) -> ToolResult {
    let status = optional_string(args, "status", "aberta");
    Ok(json!({
        "oportunidades": [
            {
                "id": "op_42",
                "cliente": "Empresa Exemplo",
                "valor": 15000,
                "etapa": "Proposta enviada",
                "status": status,
            }
        ],
        "total": 1,
    }))
}

fn registrar_interacao(args: &Value, _registry: &Registry) -> ToolResult {
    let cliente_id = required_string(args, "cliente_id")?;
    let tipo = required_string(args, "tipo")?;
    let _descricao = required_string(args, "descricao")?;

    Ok(json!({
        "sucesso": true,
        "mensagem": format!("Interacao '{tipo}' registrada para cliente {cliente_id}."),
    }))
}

fn buscar_documentacao(args: &Value, _registry: &Registry) -> ToolResult {
    let pergunta = required_string(args, "pergunta")?;
    let sistema = optional_string(args, "sistema", "geral");

    Ok(json!({
        "resultados": [
            {
                "titulo": format!("Como usar: {pergunta}"),
                "conteudo": "Documentacao de exemplo - substitua pela integracao real.",
                "sistema": sistema,
                "url": "https://docs.suaempresa.com/exemplo",
            }
        ],
        "total": 1,
    }))
}

fn consultar_politica_comercial(args: &Value, _registry: &Registry) -> ToolResult {
    let topico = required_string(args, "topico")?;
    let normalized = topico.to_lowercase();
    let politica = match normalized.as_str() {
        "desconto maximo" | "desconto maximo permitido" | "desconto máximo" => {
            "Desconto maximo permitido sem aprovacao: 10%. Acima, requer aval do gerente."
                .to_string()
        }
        "prazo de pagamento" => {
            "Padrao: 30/60/90 dias. Condicoes especiais mediante aprovacao financeira.".to_string()
        }
        "comissao" | "comissão" => {
            "Comissao base: 3% sobre valor liquido. Bonus por meta: ate 5% adicional.".to_string()
        }
        _ => format!("Politica sobre '{topico}' nao encontrada. Consulte o RH ou gerencia."),
    };

    Ok(json!({ "topico": topico, "politica": politica }))
}

fn required_string<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Invalid or missing parameter: {key}"))
}

fn optional_string<'a>(args: &'a Value, key: &str, default: &'a str) -> &'a str {
    args.get(key).and_then(Value::as_str).unwrap_or(default)
}
