# registrar_interacao

Registra uma interação (ligação, e-mail, reunião, etc.) no histórico do cliente no CRM.

**Módulo:** Sales  
**Roles:** `sales`, `admin`  
**Arquivo:** `src/tools/sales.rs`

---

## Parâmetros

| Parâmetro    | Tipo   | Obrigatório | Descrição                                           |
|--------------|--------|:-----------:|-----------------------------------------------------|
| `cliente_id` | string | ✅          | ID do cliente no CRM                                |
| `tipo`       | string | ✅          | `ligacao` \| `email` \| `reuniao` \| `outro`        |
| `descricao`  | string | ✅          | Descrição do que ocorreu na interação               |

---

## Exemplo — registrar ligação

```bash
curl -X POST http://localhost:8016/api/tools/registrar_interacao/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{
    "arguments": {
      "cliente_id": "001",
      "tipo": "ligacao",
      "descricao": "Ligação de prospecção. Cliente demonstrou interesse no plano Premium. Retorno agendado para sexta-feira."
    }
  }'
```

```json
{
  "result": {
    "sucesso": true,
    "mensagem": "Interacao 'ligacao' registrada para cliente 001."
  }
}
```

---

## Exemplo — registrar reunião

```bash
curl -X POST http://localhost:8016/api/tools/registrar_interacao/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{
    "arguments": {
      "cliente_id": "042",
      "tipo": "reuniao",
      "descricao": "Reunião de apresentação da proposta comercial. Presentes: João (vendedor) e Maria (diretora financeira). Proposta aceita com desconto de 5%."
    }
  }'
```

---

## Exemplo — registrar envio de e-mail

```bash
curl -X POST http://localhost:8016/api/tools/registrar_interacao/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{
    "arguments": {
      "cliente_id": "001",
      "tipo": "email",
      "descricao": "Enviado e-mail com proposta atualizada após negociação de preço."
    }
  }'
```

---

## Erros comuns

### Parâmetro obrigatório ausente
```bash
-d '{"arguments": {"cliente_id": "001", "tipo": "email"}}'
# erro: { "error": "Missing required parameter: descricao" }
```

### Sem permissão (role `dev`)
```json
{ "error": "Not Found", "status": 404 }
```

---

## Como adaptar para produção

Edite `src/tools/sales.rs`, função `registrar_interacao`, e salve no banco:

```rust
// INSERT INTO interacoes (cliente_id, tipo, descricao, criado_em)
// VALUES ($1, $2, $3, NOW())
```
