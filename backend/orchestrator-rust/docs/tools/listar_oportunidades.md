# listar_oportunidades

Lista oportunidades de venda em aberto para um cliente específico ou para toda a carteira.

**Módulo:** Sales  
**Roles:** `sales`, `admin`  
**Arquivo:** `src/tools/sales.rs`

---

## Parâmetros

| Parâmetro    | Tipo   | Obrigatório | Padrão   | Descrição                                          |
|--------------|--------|:-----------:|----------|----------------------------------------------------|
| `cliente_id` | string | ❌          | `""`     | ID do cliente (omita para ver toda a carteira)     |
| `status`     | string | ❌          | `aberta` | Filtro: `aberta` \| `ganha` \| `perdida`           |

---

## Exemplo — listar todas as abertas

```bash
curl -X POST http://localhost:8016/api/tools/listar_oportunidades/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"status": "aberta"}}'
```

```json
{
  "result": {
    "oportunidades": [
      {
        "id": "op_42",
        "cliente": "Empresa Exemplo",
        "valor": 15000,
        "etapa": "Proposta enviada",
        "status": "aberta"
      }
    ],
    "total": 1
  }
}
```

---

## Exemplo — filtrar por cliente

```bash
curl -X POST http://localhost:8016/api/tools/listar_oportunidades/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"cliente_id": "001", "status": "aberta"}}'
```

---

## Exemplo — ver oportunidades ganhas

```bash
curl -X POST http://localhost:8016/api/tools/listar_oportunidades/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"status": "ganha"}}'
```

---

## Exemplo — sem argumentos (usa padrões)

```bash
curl -X POST http://localhost:8016/api/tools/listar_oportunidades/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {}}'
```

Retorna todas as oportunidades com `status=aberta`.

---

## Como adaptar para produção

Edite `src/tools/sales.rs`, função `listar_oportunidades`:

```rust
// Substitua o stub por uma query real ao seu CRM/banco:
// SELECT id, cliente_id, valor, etapa, status
// FROM oportunidades
// WHERE ($1 = '' OR cliente_id = $1)
//   AND status = $2
```
