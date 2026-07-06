# rag_ingest

Ingesta um arquivo ou diretório de documentos no índice RAG (Retrieval-Augmented Generation) para busca semântica posterior.

**Módulo:** RAG (requer `RAG_ACTIVATE=true`)  
**Roles:** `admin`  
**Arquivo:** `src/tools/rag.rs`

---

## Ativação

Esta tool só aparece quando `RAG_ACTIVATE=true` está no `.env` ou na variável de ambiente:

```bash
RAG_ACTIVATE=true cargo run
```

Verifique com:
```bash
curl http://localhost:8016/health
# tools_count deve ser 16 (13 + 3 do RAG)
```

---

## Parâmetros

| Parâmetro | Tipo   | Obrigatório | Descrição                                  |
|-----------|--------|:-----------:|--------------------------------------------|
| `path`    | string | ✅          | Caminho do arquivo ou diretório a ingestar |

---

## Exemplo — ingestar um documento

```bash
curl -X POST http://localhost:8016/api/tools/rag_ingest/call \
  -H "Authorization: Bearer key_admin_00" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"path": "/documentos/manual_erp.pdf"}}'
```

```json
{
  "result": {
    "status": "ok",
    "path": "/documentos/manual_erp.pdf",
    "message": "RAG: implementar pipeline de ingestão com Chroma/FAISS/Weaviate."
  }
}
```

---

## Exemplo — ingestar diretório inteiro

```bash
curl -X POST http://localhost:8016/api/tools/rag_ingest/call \
  -H "Authorization: Bearer key_admin_00" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"path": "/documentos/politicas/"}}'
```

---

## Como implementar o pipeline real

Edite `src/tools/rag.rs`, função `rag_ingest`:

```rust
// Pipeline completo de ingestão:
// 1. Ler arquivo (PDF, DOCX, TXT, MD)
// 2. Dividir em chunks (ex: 512 tokens com overlap de 50)
// 3. Gerar embeddings via API (OpenAI, Ollama, etc.)
// 4. Salvar no vector store (Chroma, Qdrant, FAISS, Weaviate)

// Exemplo conceitual com Qdrant:
// let text = read_document(path).await?;
// let chunks = split_into_chunks(&text, 512, 50);
// for chunk in chunks {
//     let embedding = embed_client.embed(&chunk).await?;
//     qdrant.upsert(collection, embedding, chunk).await?;
// }
```

---

## Formatos de documento sugeridos

| Formato | Biblioteca Rust sugerida         |
|---------|----------------------------------|
| PDF     | `pdf-extract`, `lopdf`           |
| DOCX    | `docx-rs`                        |
| TXT/MD  | Leitura nativa com `tokio::fs`   |
| HTML    | `scraper`                        |

---

## Fluxo completo: ingest → search → ask

```bash
# 1. Ingestar documentos
curl -X POST .../rag_ingest/call -d '{"arguments": {"path": "/docs/"}}'

# 2. Buscar passagens relevantes
curl -X POST .../rag_search/call -d '{"arguments": {"query": "política de devolução", "top_k": 3}}'

# 3. Perguntar com contexto
curl -X POST .../rag_ask/call -d '{"arguments": {"query": "Qual é o prazo para devolução?"}}'
```
