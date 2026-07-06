# consultar_politica_comercial

Retorna as regras de desconto, condições de pagamento e políticas de venda vigentes da empresa.

**Módulo:** Sales  
**Roles:** `sales`, `admin`  
**Arquivo:** `src/tools/sales.rs`

---

## Parâmetros

| Parâmetro | Tipo   | Obrigatório | Descrição                                                        |
|-----------|--------|:-----------:|------------------------------------------------------------------|
| `topico`  | string | ✅          | Ex: `desconto maximo`, `prazo de pagamento`, `comissao`          |

---

## Tópicos reconhecidos

| Tópico                  | Política retornada                                                |
|-------------------------|-------------------------------------------------------------------|
| `desconto maximo`       | Desconto máximo sem aprovação: 10%. Acima requer gerente.         |
| `prazo de pagamento`    | Padrão: 30/60/90 dias. Especiais com aprovação financeira.        |
| `comissao`              | Base: 3% sobre valor líquido. Bônus por meta: até 5% adicional.  |
| Qualquer outro tópico   | Mensagem orientando a consultar RH ou gerência.                   |

---

## Exemplo — desconto máximo

```bash
curl -X POST http://localhost:8016/api/tools/consultar_politica_comercial/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"topico": "desconto maximo"}}'
```

```json
{
  "result": {
    "topico": "desconto maximo",
    "politica": "Desconto maximo permitido sem aprovacao: 10%. Acima, requer aval do gerente."
  }
}
```

---

## Exemplo — prazo de pagamento

```bash
curl -X POST http://localhost:8016/api/tools/consultar_politica_comercial/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"topico": "prazo de pagamento"}}'
```

```json
{
  "result": {
    "topico": "prazo de pagamento",
    "politica": "Padrao: 30/60/90 dias. Condicoes especiais mediante aprovacao financeira."
  }
}
```

---

## Exemplo — comissão

```bash
curl -X POST http://localhost:8016/api/tools/consultar_politica_comercial/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"topico": "comissao"}}'
```

```json
{
  "result": {
    "topico": "comissao",
    "politica": "Comissao base: 3% sobre valor liquido. Bonus por meta: ate 5% adicional."
  }
}
```

---

## Exemplo — tópico não mapeado

```bash
-d '{"arguments": {"topico": "politica de devolucao"}}'
```
```json
{
  "result": {
    "topico": "politica de devolucao",
    "politica": "Politica sobre 'politica de devolucao' nao encontrada. Consulte o RH ou gerencia."
  }
}
```

---

## Como adaptar para produção

Edite `src/tools/sales.rs`, função `consultar_politica_comercial`, e carregue do banco real:

```rust
// SELECT politica FROM politicas_comerciais WHERE topico ILIKE $1 AND vigente = true
```
