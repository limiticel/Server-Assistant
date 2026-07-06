use serde_json::{json, Value};

use crate::registry::{object_schema, Registry, Tool, ToolResult};

pub fn register(registry: &mut Registry) {
    registry.register(Tool {
        name: "rag_ingest".to_string(),
        description: "(POC) Ingesta básica de documentos para RAG. Requer dependências RAG.".to_string(),
        roles: vec!["admin".to_string()],
        input_schema: object_schema(
            json!({
                "path": { "type": "string", "description": "Caminho do arquivo ou diretório a ingestar" }
            }),
            vec!["path"],
        ),
        handler: rag_ingest,
    });

    registry.register(Tool {
        name: "rag_ask".to_string(),
        description: "(POC) Pergunte sobre documentos indexados (requer deps e um model client)."
            .to_string(),
        roles: vec!["admin".to_string(), "dev".to_string()],
        input_schema: object_schema(
            json!({
                "query": { "type": "string" }
            }),
            vec!["query"],
        ),
        handler: rag_ask,
    });

    registry.register(Tool {
        name: "rag_search".to_string(),
        description: "(POC) Busca por similaridade em índice RAG (requer deps).".to_string(),
        roles: vec!["admin".to_string(), "dev".to_string()],
        input_schema: object_schema(
            json!({
                "query": { "type": "string" },
                "top_k": { "type": "integer" }
            }),
            vec!["query"],
        ),
        handler: rag_search,
    });
}

fn rag_ingest(args: &Value, _registry: &Registry) -> ToolResult {
    let path = required_string(args, "path")?;
    Ok(json!({
        "status": "ok",
        "path": path,
        "message": "RAG: implementar pipeline de ingestão com Chroma/FAISS/Weaviate."
    }))
}

fn rag_ask(args: &Value, _registry: &Registry) -> ToolResult {
    let query = required_string(args, "query")?;
    Ok(json!({
        "status": "ok",
        "query": query,
        "answer": "IMPLEMENT_ME",
        "sources": []
    }))
}

fn rag_search(args: &Value, _registry: &Registry) -> ToolResult {
    let query = required_string(args, "query")?;
    let top_k = args.get("top_k").and_then(Value::as_u64).unwrap_or(5);
    Ok(json!({
        "status": "ok",
        "query": query,
        "results": [],
        "top_k": top_k
    }))
}

fn required_string<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Invalid or missing parameter: {key}"))
}
