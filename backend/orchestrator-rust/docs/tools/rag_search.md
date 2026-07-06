# rag_search

Busca por similaridade semântica no índice RAG. Retorna os `top_k` trechos de documentos mais relevantes para a query, com pontuação de similaridade.

**Módulo:** RAG (requer `RAG_ACTIVATE=true`)  
**Roles:** `admin`, `dev`  
**Arquivo:** `src/tools/rag.rs`

---

## Ativação

```bash
RAG_ACTIVATE=true cargo run
```

---

## Parâmetros

| Parâmetro | Tipo    | Obrigatório | Padrão | Descrição                                  |
|-----------|---------|:-----------:|--------|--------------------------------------------|
| `query`   | string  | ✅          | —      | Texto a buscar semanticamente              |
| `top_k`   | integer | ❌          | `5`    | Número máximo de resultados a retornar     |

---

## Exemplo — busca padrão (top 5)

```bash
curl -X POST http://localhost:8016/api/tools/rag_search/call \
  -H "Authorization: Bearer key_admin_00" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"query": "política de devolução e reembolso"}}'
```

```json
{
  "result": {
    "status": "ok",
    "query": "política de devolução e reembolso",
    "results": [],
    "top_k": 5
  }
}
```

> `results` retorna vazio até que o pipeline real seja implementado.

---

## Exemplo — busca com top_k customizado

```bash
curl -X POST http://localhost:8016/api/tools/rag_search/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"query": "desconto máximo para vendas corporativas", "top_k": 3}}'
```

---

## Exemplo — busca técnica

```bash
curl -X POST http://localhost:8016/api/tools/rag_search/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"query": "como configurar autenticação JWT na API", "top_k": 10}}'
```

---

## Campos da resposta

| Campo     | Tipo    | Descrição                                              |
|-----------|---------|--------------------------------------------------------|
| `status`  | string  | `ok` se a chamada foi aceita                           |
| `query`   | string  | A query de busca                                       |
| `results` | array   | Lista de passagens relevantes (vazia no stub atual)    |
| `top_k`   | integer | Número de resultados solicitados                       |

---

## Como implementar o pipeline real

Edite `src/tools/rag.rs`, função `rag_search`:

```rust
// 1. Gerar embedding da query
// 2. Buscar top_k vetores mais próximos no vector store
// 3. Retornar passagens com score de similaridade

// Exemplo com Qdrant:
// let query_embedding = embed_client.embed(query).await?;
// let results = qdrant
//     .search(collection="documentos", vector=query_embedding, top=top_k)
//     .await?;
// let passages = results.iter().map(|r| json!({
//     "id": r.id,
//     "texto": r.payload["texto"],
//     "fonte": r.payload["fonte"],
//     "score": r.score,
// })).collect::<Vec<_>>();
// Ok(json!({ "status": "ok", "query": query, "results": passages, "top_k": top_k }))
```

---

## Resposta esperada após implementação

```json
{
  "result": {
    "status": "ok",
    "query": "política de devolução",
    "results": [
      {
        "id": "chunk_042",
        "texto": "O prazo para devolução é de 30 dias corridos a partir do recebimento do produto.",
        "fonte": "politica_comercial_2024.pdf",
        "pagina": 12,
        "score": 0.97
      },
      {
        "id": "chunk_043",
        "texto": "Para reembolsos em cartão de crédito, o prazo de estorno é de até 2 faturas.",
        "fonte": "politica_comercial_2024.pdf",
        "pagina": 13,
        "score": 0.89
      }
    ],
    "top_k": 5
  }
}
```

---

## Diferença entre `rag_search` e `rag_ask`

| Tool         | Retorna                                              | Quando usar                                         |
|--------------|------------------------------------------------------|-----------------------------------------------------|
| `rag_search` | Lista de trechos crus com score de similaridade      | Para inspeção, debugging ou montagem manual de prompt |
| `rag_ask`    | Resposta em linguagem natural gerada por LLM         | Quando quer a resposta final direto                  |

---

## Vector stores compatíveis

| Vector Store | Observação                              |
|--------------|-----------------------------------------|
| Qdrant       | Recomendado — Rust-native, alta performance |
| ChromaDB     | Simples, bom para PoC                   |
| FAISS        | In-memory, sem persistência             |
| Weaviate     | GraphQL API, bom para dados estruturados |
| Pinecone     | Managed cloud, sem infra local          |
