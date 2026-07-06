# gerar_schema_openapi

Gera um schema OpenAPI 3.0 completo em JSON para um endpoint descrito em linguagem natural. Útil para documentar APIs antes de implementá-las ou para gerar contratos de integração.

**Módulo:** DevTools  
**Roles:** `dev`, `admin`  
**Arquivo:** `src/tools/devtools.rs`

---

## Parâmetros

| Parâmetro             | Tipo   | Obrigatório | Descrição                                     |
|-----------------------|--------|:-----------:|-----------------------------------------------|
| `descricao_endpoint`  | string | ✅          | Descrição em português do endpoint desejado   |

---

## Exemplo — endpoint de criação de pedidos

```bash
curl -X POST http://localhost:8016/api/tools/gerar_schema_openapi/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{
    "arguments": {
      "descricao_endpoint": "POST /pedidos que recebe cliente_id e lista de itens e retorna o pedido criado com total calculado"
    }
  }'
```

### Schema gerado
```json
{
  "openapi": "3.0.0",
  "info": {
    "title": "API Gerada",
    "version": "1.0.0"
  },
  "paths": {
    "/exemplo": {
      "post": {
        "summary": "POST /pedidos que recebe cliente_id e lista de itens e retorna o pedido criado com total calculado",
        "requestBody": {
          "content": {
            "application/json": {
              "schema": {
                "type": "object",
                "description": "Substitua com o schema real"
              }
            }
          }
        },
        "responses": {
          "201": {
            "description": "Criado com sucesso"
          }
        }
      }
    }
  }
}
```

---

## Exemplo — endpoint de autenticação

```bash
curl -X POST http://localhost:8016/api/tools/gerar_schema_openapi/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{
    "arguments": {
      "descricao_endpoint": "POST /auth/login que recebe email e password e retorna JWT token com expiração de 8 horas"
    }
  }'
```

---

## Exemplo — endpoint de busca

```bash
-d '{
  "arguments": {
    "descricao_endpoint": "GET /clientes com filtros opcionais de nome, email e status, retornando lista paginada"
  }
}'
```

---

## Como usar o schema gerado

```bash
# Salvar como arquivo JSON
curl -s -X POST http://localhost:8016/api/tools/gerar_schema_openapi/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"descricao_endpoint": "POST /pedidos que recebe cliente_id e itens"}}' \
  | python3 -c "import sys,json; d=json.load(sys.stdin); print(json.dumps(d['result'], indent=2))" \
  > openapi.json

# Visualizar no Swagger UI (com npx)
npx @redocly/cli preview-docs openapi.json
```

---

## Nota sobre o schema gerado

O schema é um **ponto de partida** — o `summary` usa a descrição que você passou e o `requestBody` tem um placeholder. Edite o JSON resultante para refinar os tipos dos campos, adicionar exemplos e documentar as respostas com mais detalhe.
