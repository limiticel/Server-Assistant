use serde_json::{json, Value};

use crate::registry::{object_schema, Registry, Tool, ToolResult};

pub fn register(registry: &mut Registry) {
    registry.register(Tool {
        name: "tools_health".to_string(),
        description: "Retorna status simples sobre as tools registradas".to_string(),
        roles: vec!["*".to_string()],
        input_schema: object_schema(json!({}), vec![]),
        handler: tools_health,
    });

    registry.register(Tool {
        name: "consultar_estoque".to_string(),
        description: "Consulta o estoque atual de um produto no ERP pelo codigo ou nome."
            .to_string(),
        roles: vec!["sales".to_string(), "admin".to_string()],
        input_schema: object_schema(
            json!({
                "produto": { "type": "string", "description": "Codigo ou nome do produto" }
            }),
            vec!["produto"],
        ),
        handler: consultar_estoque,
    });

    registry.register(Tool {
        name: "consultar_pedido".to_string(),
        description: "Retorna status e detalhes de um pedido pelo numero.".to_string(),
        roles: vec!["sales".to_string(), "admin".to_string()],
        input_schema: object_schema(
            json!({
                "numero_pedido": { "type": "string", "description": "Numero do pedido no ERP" }
            }),
            vec!["numero_pedido"],
        ),
        handler: consultar_pedido,
    });

    registry.register(Tool {
        name: "emitir_nota_fiscal".to_string(),
        description: "Solicita a emissao de nota fiscal para um pedido aprovado.".to_string(),
        roles: vec!["admin".to_string()],
        input_schema: object_schema(
            json!({
                "numero_pedido": { "type": "string" },
                "tipo": { "type": "string", "description": "'NFe' | 'NFSe' | 'NFCe'", "default": "NFe" }
            }),
            vec!["numero_pedido"],
        ),
        handler: emitir_nota_fiscal,
    });
}

fn tools_health(_args: &Value, registry: &Registry) -> ToolResult {
    let tools = registry.list_registered_tools();
    Ok(json!({
        "status": "ok",
        "tools_count": tools.len(),
        "tools": tools,
    }))
}

fn consultar_estoque(args: &Value, _registry: &Registry) -> ToolResult {
    let produto = required_string(args, "produto")?;
    Ok(json!({
        "produto": produto,
        "estoque_disponivel": 42,
        "estoque_reservado": 8,
        "unidade": "un",
        "ultima_atualizacao": "2024-01-15T10:30:00",
    }))
}

fn consultar_pedido(args: &Value, _registry: &Registry) -> ToolResult {
    let numero_pedido = required_string(args, "numero_pedido")?;
    Ok(json!({
        "numero": numero_pedido,
        "status": "Em separacao",
        "cliente": "Empresa Exemplo Ltda",
        "valor_total": 4580.00,
        "previsao_entrega": "2024-01-20",
        "itens": [
            { "produto": "SKU-001", "quantidade": 10, "valor_unitario": 458.00 }
        ],
    }))
}

fn emitir_nota_fiscal(args: &Value, _registry: &Registry) -> ToolResult {
    let numero_pedido = required_string(args, "numero_pedido")?;
    let tipo = optional_string(args, "tipo", "NFe");
    Ok(json!({
        "sucesso": true,
        "numero_pedido": numero_pedido,
        "tipo": tipo,
        "mensagem": format!("{tipo} solicitada para pedido {numero_pedido}. Aguarde processamento."),
        "protocolo": "STUB-12345",
    }))
}

fn required_string<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Invalid or missing parameter: {key}"))
}

fn optional_string<'a>(args: &'a Value, key: &str, default: &'a str) -> &'a str {
    args.get(key).and_then(Value::as_str).unwrap_or(default)
}
