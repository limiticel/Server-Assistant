# emitir_nota_fiscal

Solicita a emissão de nota fiscal para um pedido aprovado. Suporta NF-e, NFS-e e NFC-e.

**Módulo:** Internal Systems  
**Roles:** `admin` (apenas administradores)  
**Arquivo:** `src/tools/internal_systems.rs`

---

## Parâmetros

| Parâmetro       | Tipo   | Obrigatório | Padrão | Descrição                              |
|-----------------|--------|:-----------:|--------|----------------------------------------|
| `numero_pedido` | string | ✅          | —      | Número do pedido no ERP                |
| `tipo`          | string | ❌          | `NFe`  | `NFe` \| `NFSe` \| `NFCe`             |

---

## Exemplo — NF-e (padrão)

```bash
curl -X POST http://localhost:8016/api/tools/emitir_nota_fiscal/call \
  -H "Authorization: Bearer key_admin_00" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"numero_pedido": "PED-2024-001"}}'
```

```json
{
  "result": {
    "sucesso": true,
    "numero_pedido": "PED-2024-001",
    "tipo": "NFe",
    "mensagem": "NFe solicitada para pedido PED-2024-001. Aguarde processamento.",
    "protocolo": "STUB-12345"
  }
}
```

---

## Exemplo — NFS-e (serviços)

```bash
curl -X POST http://localhost:8016/api/tools/emitir_nota_fiscal/call \
  -H "Authorization: Bearer key_admin_00" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"numero_pedido": "SRV-2024-007", "tipo": "NFSe"}}'
```

---

## Exemplo — NFC-e (consumidor final)

```bash
curl -X POST http://localhost:8016/api/tools/emitir_nota_fiscal/call \
  -H "Authorization: Bearer key_admin_00" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"numero_pedido": "PDV-2024-123", "tipo": "NFCe"}}'
```

---

## Tentativa com role insuficiente (sales)

```bash
curl -X POST http://localhost:8016/api/tools/emitir_nota_fiscal/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"numero_pedido": "PED-2024-001"}}'
```
```json
{ "error": "Not Found", "status": 404 }
```

A tool simplesmente não aparece no registry para role `sales` — nem é possível descobrir que existe.

---

## Tipos de nota fiscal

| Tipo   | Uso                                      | Emissor típico          |
|--------|------------------------------------------|-------------------------|
| `NFe`  | Venda de produtos (ICMS)                 | SAP, TOTVS, Focus NFe   |
| `NFSe` | Prestação de serviços (ISS)              | Prefeitura, Enotas      |
| `NFCe` | Venda ao consumidor final (substitui ECF)| PDV, Bling              |

---

## Como adaptar para produção

```rust
// Integração com emissor fiscal:
// let protocolo = focus_nfe_client
//     .emitir(numero_pedido, tipo)
//     .await?;
// Ok(json!({
//     "sucesso": true,
//     "protocolo": protocolo.numero,
//     "chave_acesso": protocolo.chave,
//     ...
// }))
```
