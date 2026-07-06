# rag_ask

Faz uma pergunta em linguagem natural sobre documentos previamente indexados via `rag_ingest`. Recupera contexto relevante e gera uma resposta com base nos documentos.

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

| Parâmetro | Tipo   | Obrigatório | Descrição                                   |
|-----------|--------|:-----------:|---------------------------------------------|
| `query`   | string | ✅          | Pergunta em linguagem natural               |

---

## Exemplo

```bash
curl -X POST http://localhost:8016/api/tools/rag_ask/call \
  -H "Authorization: Bearer key_admin_00" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"query": "Qual é o prazo máximo para devolução de produtos?"}}'
```

```json
{
  "result": {
    "status": "ok",
    "query": "Qual é o prazo máximo para devolução de produtos?",
    "answer": "IMPLEMENT_ME",
    "sources": []
  }
}
```

> O `answer` retorna `IMPLEMENT_ME` até que o pipeline real seja implementado.

---

## Campos da resposta

| Campo     | Tipo   | Descrição                                             |
|-----------|--------|-------------------------------------------------------|
| `status`  | string | `ok` se a chamada foi aceita                          |
| `query`   | string | A pergunta feita                                      |
| `answer`  | string | Resposta gerada (IMPLEMENT_ME no stub atual)          |
| `sources` | array  | Documentos fonte usados na resposta (vazio no stub)   |

---

## Como implementar o pipeline real

Edite `src/tools/rag.rs`, função `rag_ask`:

```rust
// Fluxo RAG completo:
// 1. Gerar embedding da query
// 2. Buscar top-k passagens similares no vector store
// 3. Montar prompt: "Com base nos documentos abaixo, responda: {query}\n\n{passagens}"
// 4. Chamar LLM (Claude, GPT-4, Llama, etc.) via API HTTP
// 5. Retornar resposta + fontes

// Exemplo conceitual:
// let query_embedding = embed_client.embed(query).await?;
// let passages = qdrant.search(query_embedding, top_k=5).await?;
// let context = passages.iter().map(|p| p.text).collect::<Vec<_>>().join("\n\n");
// let prompt = format!("Documentos:\n{context}\n\nPergunta: {query}\n\nResposta:");
// let answer = llm_client.complete(&prompt).await?;
// Ok(json!({ "status": "ok", "query": query, "answer": answer, "sources": passages }))
```

---

## Diferença entre `rag_ask` e `rag_search`

| Tool         | Retorna                                         | Quando usar                                  |
|--------------|-------------------------------------------------|----------------------------------------------|
| `rag_ask`    | Uma resposta em linguagem natural               | Quando quer uma resposta direta              |
| `rag_search` | Lista de passagens relevantes com pontuação     | Quando quer os trechos crus para processar   |

---

## Exemplo de resposta implementada

Quando o pipeline estiver implementado, a resposta seria:

```json
{
  "result": {
    "status": "ok",
    "query": "Qual é o prazo máximo para devolução?",
    "answer": "De acordo com a Política de Devolução (seção 3.2), o prazo máximo para solicitação de devolução é de 30 dias corridos após o recebimento do produto, desde que ele esteja em perfeitas condições e na embalagem original.",
    "sources": [
      {
        "documento": "politica_devolucao_v2.pdf",
        "pagina": 4,
        "trecho": "...o prazo máximo para solicitação de devolução é de 30 dias corridos...",
        "score": 0.94
      }
    ]
  }
}
```
