# buscar_documentacao

Busca na documentação interna da empresa. Use para tirar dúvidas sobre como usar qualquer sistema, processo ou política.

**Módulo:** Sales  
**Roles:** `sales`, `dev`, `admin`  
**Arquivo:** `src/tools/sales.rs`

---

## Parâmetros

| Parâmetro | Tipo   | Obrigatório | Descrição                                                      |
|-----------|--------|:-----------:|----------------------------------------------------------------|
| `pergunta`| string | ✅          | Dúvida ou termo a buscar                                       |
| `sistema` | string | ❌          | Sistema específico: `crm` \| `erp` \| `portal` (opcional)     |

---

## Exemplo — busca geral

```bash
curl -X POST http://localhost:8016/api/tools/buscar_documentacao/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"pergunta": "como emitir segunda via de boleto"}}'
```

```json
{
  "result": {
    "resultados": [
      {
        "titulo": "Como usar: como emitir segunda via de boleto",
        "conteudo": "Documentacao de exemplo — substitua pela integracao real.",
        "sistema": "geral",
        "url": "https://docs.suaempresa.com/exemplo"
      }
    ],
    "total": 1
  }
}
```

---

## Exemplo — busca em sistema específico

```bash
curl -X POST http://localhost:8016/api/tools/buscar_documentacao/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"pergunta": "como cadastrar pedido", "sistema": "erp"}}'
```

---

## Exemplo — uso pelo dev

```bash
curl -X POST http://localhost:8016/api/tools/buscar_documentacao/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"pergunta": "documentacao da API de estoque", "sistema": "erp"}}'
```

---

## Como adaptar para produção

Integre com sua base de conhecimento real — Notion, Confluence, banco vetorial, Elasticsearch:

```rust
// Exemplo: busca via Elasticsearch
// let response = es_client.search(index="docs", query=pergunta).await?;
// let hits = response.hits.iter().map(|h| doc! { "titulo": h.title, ... });
```

Ou use o módulo RAG (`rag_search`) para busca semântica em documentos indexados.
