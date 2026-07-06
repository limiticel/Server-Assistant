# consultar_pedido

Retorna o status, itens e previsão de entrega de um pedido no ERP pelo número.

**Módulo:** Internal Systems  
**Roles:** `sales`, `admin`  
**Arquivo:** `src/tools/internal_systems.rs`

---

## Parâmetros

| Parâmetro       | Tipo   | Obrigatório | Descrição                      |
|-----------------|--------|:-----------:|--------------------------------|
| `numero_pedido` | string | ✅          | Número do pedido no ERP        |

---

## Exemplo

```bash
curl -X POST http://localhost:8016/api/tools/consultar_pedido/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"numero_pedido": "PED-2024-001"}}'
```

```json
{
  "result": {
    "numero": "PED-2024-001",
    "status": "Em separacao",
    "cliente": "Empresa Exemplo Ltda",
    "valor_total": 4580.0,
    "previsao_entrega": "2024-01-20",
    "itens": [
      {
        "produto": "SKU-001",
        "quantidade": 10,
        "valor_unitario": 458.0
      }
    ]
  }
}
```

---

## Campos da resposta

| Campo              | Tipo    | Descrição                              |
|--------------------|---------|----------------------------------------|
| `numero`           | string  | Número do pedido                       |
| `status`           | string  | Status atual no ERP                    |
| `cliente`          | string  | Nome do cliente                        |
| `valor_total`      | número  | Valor total do pedido                  |
| `previsao_entrega` | string  | Data prevista de entrega (YYYY-MM-DD)  |
| `itens`            | array   | Lista de produtos com qtd e valor      |

---

## Status comuns

| Status              | Significado                            |
|---------------------|----------------------------------------|
| `Aguardando pagamento` | Pedido criado, pagamento pendente   |
| `Em separacao`      | Pagamento confirmado, separando itens  |
| `Em transporte`     | Enviado para transportadora            |
| `Entregue`          | Entregue ao destinatário               |
| `Cancelado`         | Pedido cancelado                       |

---

## Exemplo — via protocolo MCP

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "consultar_pedido",
    "arguments": { "numero_pedido": "PED-2024-001" }
  }
}
```

---

## Como adaptar para produção

```rust
// Consulta ao ERP real:
// let pedido = erp_client.get_pedido(&numero_pedido).await?;
// Ok(json!({
//     "numero": pedido.numero,
//     "status": pedido.status,
//     "itens": pedido.itens,
//     ...
// }))
```
