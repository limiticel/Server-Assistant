# tools_health

Retorna o status das tools registradas no servidor — útil para verificar quais ferramentas estão ativas e funcionando.

**Módulo:** Internal Systems  
**Roles:** `*` (todos — não requer autenticação específica)  
**Arquivo:** `src/tools/internal_systems.rs`

---

## Parâmetros

Nenhum. Chame com `"arguments": {}`.

---

## Exemplo — sem autenticação (em DEBUG=true local)

```bash
curl -X POST http://localhost:8016/api/tools/tools_health/call \
  -H "Content-Type: application/json" \
  -d '{"arguments": {}}'
```

```json
{
  "result": {
    "status": "ok",
    "tools_count": 13,
    "tools": [
      "tools_health",
      "consultar_estoque",
      "consultar_pedido",
      "emitir_nota_fiscal",
      "buscar_cliente",
      "listar_oportunidades",
      "registrar_interacao",
      "buscar_documentacao",
      "consultar_politica_comercial",
      "gerar_scaffold_api",
      "gerar_schema_openapi",
      "gerar_bot_whatsapp",
      "gerar_webhook_handler"
    ]
  }
}
```

---

## Exemplo — com autenticação

```bash
curl -X POST http://localhost:8016/api/tools/tools_health/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {}}'
```

---

## Exemplo — via protocolo MCP

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "tools_health",
    "arguments": {}
  }
}
```

---

## Diferença entre `/health` e `tools_health`

| Endpoint/Tool       | Quem usa             | O que retorna                              |
|---------------------|----------------------|--------------------------------------------|
| `GET /health`       | Monitoramento, k8s   | Status HTTP + total de tools               |
| `tools_health`      | Clientes MCP, Claude | Lista com **nomes** de todas as tools ativas |

Use `tools_health` quando precisar que o modelo de linguagem saiba quais ferramentas estão disponíveis em tempo de execução.
