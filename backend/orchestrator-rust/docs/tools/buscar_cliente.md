# buscar_cliente

Busca um cliente no CRM pelo nome, e-mail ou ID. Retorna dados cadastrais e status.

**Módulo:** Sales  
**Roles:** `sales`, `admin`  
**Arquivo:** `src/tools/sales.rs`

---

## Parâmetros

| Parâmetro | Tipo   | Obrigatório | Descrição                              |
|-----------|--------|:-----------:|----------------------------------------|
| `query`   | string | ✅          | Nome, e-mail ou ID do cliente          |

---

## Exemplo — curl

```bash
curl -X POST http://localhost:8016/api/tools/buscar_cliente/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"query": "Empresa Acme"}}'
```

### Resposta
```json
{
  "result": {
    "clientes": [
      {
        "id": "001",
        "nome": "Empresa Acme",
        "status": "ativo",
        "contato": "contato@acme.com"
      }
    ],
    "total": 1,
    "query": "Empresa Acme"
  }
}
```

---

## Exemplo — protocolo MCP (SSE)

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "buscar_cliente",
    "arguments": {
      "query": "joao@empresa.com"
    }
  }
}
```

---

## Exemplo — busca por ID

```bash
curl -X POST http://localhost:8016/api/tools/buscar_cliente/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"query": "001"}}'
```

---

## Erro — parâmetro obrigatório ausente

```bash
curl -X POST http://localhost:8016/api/tools/buscar_cliente/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {}}'
```
```json
{ "error": "Missing required parameter: query" }
```

---

## Erro — sem permissão (role `dev` não tem acesso)

```bash
curl -X POST http://localhost:8016/api/tools/buscar_cliente/call \
  -H "Authorization: Bearer key_dev_01" \
  ...
```
```json
{ "error": "Not Found", "status": 404 }
```

---

## Como adaptar para produção

Edite `src/tools/sales.rs`, função `buscar_cliente`, e substitua o stub pelo acesso real ao seu CRM:

```rust
// Exemplo com SQLx (PostgreSQL)
// let rows = sqlx::query!("SELECT id, nome, status, email FROM clientes WHERE nome ILIKE $1", format!("%{}%", query))
//     .fetch_all(&pool)
//     .await?;
```
