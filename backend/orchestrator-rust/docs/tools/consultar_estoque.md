# consultar_estoque

Consulta o estoque atual de um produto no ERP pelo código ou nome. Retorna quantidade disponível, reservada e unidade de medida.

**Módulo:** Internal Systems  
**Roles:** `sales`, `admin`  
**Arquivo:** `src/tools/internal_systems.rs`

---

## Parâmetros

| Parâmetro | Tipo   | Obrigatório | Descrição                            |
|-----------|--------|:-----------:|--------------------------------------|
| `produto` | string | ✅          | Código SKU ou nome do produto        |

---

## Exemplo — consulta por código SKU

```bash
curl -X POST http://localhost:8016/api/tools/consultar_estoque/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"produto": "SKU-001"}}'
```

```json
{
  "result": {
    "produto": "SKU-001",
    "estoque_disponivel": 42,
    "estoque_reservado": 8,
    "unidade": "un",
    "ultima_atualizacao": "2024-01-15T10:30:00"
  }
}
```

---

## Exemplo — consulta por nome do produto

```bash
curl -X POST http://localhost:8016/api/tools/consultar_estoque/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"produto": "Parafuso Sextavado M8"}}'
```

---

## Exemplo — via protocolo MCP

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "consultar_estoque",
    "arguments": { "produto": "SKU-001" }
  }
}
```

---

## Campos da resposta

| Campo                 | Tipo    | Descrição                                   |
|-----------------------|---------|---------------------------------------------|
| `produto`             | string  | Código/nome consultado                      |
| `estoque_disponivel`  | número  | Quantidade disponível para venda            |
| `estoque_reservado`   | número  | Quantidade reservada por pedidos pendentes  |
| `unidade`             | string  | Unidade de medida (`un`, `kg`, `cx`, etc.)  |
| `ultima_atualizacao`  | string  | Timestamp da última sincronização com o ERP |

> **Estoque real disponível** = `estoque_disponivel` - `estoque_reservado`

---

## Erro — parâmetro ausente

```bash
-d '{"arguments": {}}'
# { "error": "Missing required parameter: produto" }
```

---

## Como adaptar para produção

Edite `src/tools/internal_systems.rs`, função `consultar_estoque`:

```rust
// Integração com SAP, TOTVS, Omie, Bling, etc.
// let saldo = erp_client.get("/estoque/{produto}").await?;
// Ok(json!({
//     "produto": produto,
//     "estoque_disponivel": saldo.disponivel,
//     "estoque_reservado": saldo.reservado,
//     ...
// }))
```
