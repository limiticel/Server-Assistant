# mcp-orchestrator-rust

Servidor MCP (Model Context Protocol) de alta performance escrito em Rust.
Equivalente funcional ao `mcp-orchestrator-` Python — mesmos módulos, tools e endpoints — com desempenho nativo e zero overhead de runtime.

---

## Índice

- [O que é o MCP](#o-que-é-o-mcp)
- [Arquitetura](#arquitetura)
- [Pré-requisitos](#pré-requisitos)
- [Build e instalação](#build-e-instalação)
- [Configuração](#configuração)
- [Iniciando o servidor](#iniciando-o-servidor)
- [Autenticação](#autenticação)
- [Endpoints HTTP](#endpoints-http)
- [Tools disponíveis](#tools-disponíveis)
- [Exemplos de uso (curl)](#exemplos-de-uso-curl)
- [Uso com Claude Desktop (stdio)](#uso-com-claude-desktop-stdio)
- [Variáveis de ambiente](#variáveis-de-ambiente)
- [Documentação por tool](#documentação-por-tool)

---

## O que é o MCP

O MCP (Model Context Protocol) é o protocolo aberto da Anthropic que permite que modelos de linguagem (como o Claude) chamem **ferramentas externas** de forma padronizada. Este servidor expõe tools de negócio que qualquer cliente MCP pode invocar — seja o Claude Desktop, o Claude API ou integrações customizadas.

```
┌─────────────────┐    MCP (SSE / stdio / JSON-RPC)    ┌──────────────────────┐
│  Claude Desktop │ ──────────────────────────────────► │  mcp-orchestrator-   │
│  Claude API     │                                      │       rust           │
│  Seu cliente    │ ◄────────────────────────────────── │  (este servidor)     │
└─────────────────┘         respostas JSON-RPC           └──────────────────────┘
                                                                   │
                                               ┌───────────────────┼──────────────────┐
                                               ▼                   ▼                  ▼
                                           ERP/CRM             APIs internas      Documentação
```

---

## Arquitetura

```
src/
├── main.rs                  # Entrypoint — detecta HTTP ou stdio
├── config.rs                # Settings por variáveis de ambiente
├── auth.rs                  # Autenticação por API Key + log_tool_call
├── registry.rs              # Registry de tools (registro, lookup por role)
├── rate_limit.rs            # Token bucket (rate limiter in-memory)
├── sse_transport.rs         # Protocolo MCP via HTTP+SSE (JSON-RPC)
├── stdio_transport.rs       # Protocolo MCP via stdio (Claude Desktop)
├── http/
│   └── mod.rs               # Rotas REST: /health, /api/tools/*, /sse, /messages
└── tools/
    ├── mod.rs               # build_registry() — habilita módulos por feature flag
    ├── sales.rs             # CRM: buscar_cliente, listar_oportunidades, ...
    ├── internal_systems.rs  # ERP: consultar_estoque, consultar_pedido, ...
    ├── devtools.rs          # Dev: scaffold, openapi, bot whatsapp, webhook
    └── rag.rs               # RAG: ingest, ask, search (requer RAG_ACTIVATE=true)
```

### Fluxo de uma chamada de tool

```
Cliente                        mcp-orchestrator-rust
   │                                    │
   │  POST /api/tools/buscar_cliente/call
   │  Authorization: Bearer <api_key>   │
   │  {"arguments": {"query": "Acme"}}  │
   │ ──────────────────────────────────►│
   │                                    │ 1. Valida API key
   │                                    │ 2. Verifica role do usuário
   │                                    │ 3. Verifica rate limit (token bucket)
   │                                    │ 4. Valida argumentos obrigatórios
   │                                    │ 5. Executa o handler Rust (< 1ms)
   │                                    │ 6. Loga: user, tool, duração, sucesso
   │  {"result": {"clientes": [...]}}   │
   │ ◄──────────────────────────────────│
```

---

## Pré-requisitos

- **Rust 1.75+** → [instalar via rustup](https://rustup.rs)
- Nenhuma dependência externa de runtime (sem Python, sem Node, sem JVM)

---

## Build e instalação

```bash
# Build para produção (binário otimizado, ~5MB)
cargo build --release

# Binário gerado em:
./target/release/mcp-orchestrator-rust

# Para desenvolvimento com recompilação automática:
cargo install cargo-watch
cargo watch -x run
```

---

## Configuração

Crie `.env` na raiz do projeto:

```env
# ── Servidor ─────────────────────────────────────────────────
HOST=0.0.0.0
PORT=8016
DEBUG=true
SERVER_NAME=corporate-mcp-server
SERVER_VERSION=1.0.0

# ── Autenticação ──────────────────────────────────────────────
SECRET_KEY=minha-chave-super-secreta-troque-em-producao
# Formato: user_id:role:api_key  (múltiplas chaves separadas por vírgula)
API_KEYS=joao:sales:key_vendedor_01,maria:dev:key_dev_01,root:admin:key_admin_00

# ── Rate Limiting ─────────────────────────────────────────────
RATE_LIMIT_PER_MINUTE=60
RATE_LIMIT_BURST=10

# ── Logging ───────────────────────────────────────────────────
LOG_LEVEL=info
LOG_FORMAT=json

# ── Módulos (feature flags) ───────────────────────────────────
ENABLE_SALES=true
ENABLE_DEVTOOLS=true
ENABLE_INTERNAL_SYSTEMS=true
RAG_ACTIVATE=false

# ── Database / Redis ──────────────────────────────────────────
DATABASE_HOST=localhost
DATABASE_PORT=5432
DATABASE_USER=user
DATABASE_PASSWORD=senha
DATABASE_NAME=mcp_database
REDIS_URL=redis://localhost:6379/0

# ── Transporte ────────────────────────────────────────────────
TRANSPORT=http   # "http" (padrão) ou "stdio" (Claude Desktop)
```

---

## Iniciando o servidor

```bash
# Modo HTTP+SSE (produção, multi-cliente)
cargo run
# ou
./target/release/mcp-orchestrator-rust

# Modo stdio (Claude Desktop, uso local, 1 cliente)
TRANSPORT=stdio ./target/release/mcp-orchestrator-rust
# ou
./target/release/mcp-orchestrator-rust --stdio
```

Saída esperada no modo HTTP:
```
2024-01-15T10:30:00Z INFO  starting mcp-orchestrator-rust (HTTP+SSE) addr=0.0.0.0:8016
```

---

## Autenticação

Toda requisição protegida deve enviar a API key em um destes headers:

```bash
# Bearer token (padrão)
Authorization: Bearer key_vendedor_01

# Header customizado (alternativa)
X-API-Key: key_vendedor_01
```

**Roles e permissões:**

| Role    | Acesso                                                          |
|---------|-----------------------------------------------------------------|
| `*`     | Tools públicas (ex: `tools_health`)                             |
| `sales` | Tools de CRM: busca, oportunidades, interações, políticas       |
| `dev`   | Tools de DevTools: scaffold, OpenAPI, bots, webhooks            |
| `admin` | Acesso total — inclui tools de todas as roles + emissão de NF  |

> Em `DEBUG=true`, requisições do localhost sem API key recebem `role=admin` automaticamente.

---

## Endpoints HTTP

| Método | Endpoint                        | Autenticação | Descrição                              |
|--------|---------------------------------|:------------:|----------------------------------------|
| GET    | `/health`                       | ❌           | Status do servidor + contagem de tools |
| GET    | `/tools`                        | ❌           | Lista tools (apenas `DEBUG=true`)      |
| GET    | `/api/all-routes`               | ❌           | Lista todas as rotas registradas       |
| GET    | `/api/tools`                    | ✅           | Tools visíveis para o usuário logado   |
| GET    | `/api/tools/:name`              | ✅           | Detalhes + schema de uma tool          |
| POST   | `/api/tools/:name/call`         | ✅           | Executa uma tool com argumentos        |
| GET    | `/sse`                          | ✅           | Abre stream SSE (protocolo MCP)        |
| POST   | `/messages?session_id=<id>`     | ✅           | Envia mensagem JSON-RPC na sessão SSE  |

---

## Tools disponíveis

### Módulo Sales (`ENABLE_SALES=true`)

| Tool                           | Roles         | Descrição                              |
|--------------------------------|---------------|----------------------------------------|
| `buscar_cliente`               | sales, admin  | Busca cliente no CRM por nome/email/ID |
| `listar_oportunidades`         | sales, admin  | Lista oportunidades com filtro         |
| `registrar_interacao`          | sales, admin  | Registra ligação, e-mail ou reunião    |
| `buscar_documentacao`          | sales, dev, admin | Busca na documentação interna     |
| `consultar_politica_comercial` | sales, admin  | Descontos, prazos e comissões          |

### Módulo Internal Systems (`ENABLE_INTERNAL_SYSTEMS=true`)

| Tool                   | Roles         | Descrição                              |
|------------------------|---------------|----------------------------------------|
| `tools_health`         | * (todos)     | Status e lista das tools registradas   |
| `consultar_estoque`    | sales, admin  | Estoque disponível e reservado no ERP  |
| `consultar_pedido`     | sales, admin  | Status, itens e previsão de entrega    |
| `emitir_nota_fiscal`   | admin         | Solicita emissão de NF-e/NFS-e/NFC-e  |

### Módulo DevTools (`ENABLE_DEVTOOLS=true`)

| Tool                    | Roles       | Descrição                               |
|-------------------------|-------------|-----------------------------------------|
| `gerar_scaffold_api`    | dev, admin  | Gera código FastAPI completo do zero    |
| `gerar_schema_openapi`  | dev, admin  | Gera schema OpenAPI 3.0 em JSON         |
| `gerar_bot_whatsapp`    | dev, admin  | Gera bot WhatsApp via Evolution API     |
| `gerar_webhook_handler` | dev, admin  | Gera handler FastAPI para webhooks      |

### Módulo RAG (`RAG_ACTIVATE=true`)

| Tool          | Roles          | Descrição                               |
|---------------|----------------|-----------------------------------------|
| `rag_ingest`  | admin          | Ingesta documentos no índice RAG        |
| `rag_ask`     | admin, dev     | Pergunta sobre documentos indexados     |
| `rag_search`  | admin, dev     | Busca por similaridade (top-k)          |

---

## Exemplos de uso (curl)

### Healthcheck (sem autenticação)
```bash
curl http://localhost:8016/health
```
```json
{
  "status": "ok",
  "server": "corporate-mcp-server",
  "version": "1.0.0",
  "tools_count": 13
}
```

### Listar tools disponíveis para o usuário
```bash
curl http://localhost:8016/api/tools \
  -H "Authorization: Bearer key_vendedor_01"
```
```json
[
  {
    "name": "buscar_cliente",
    "description": "Busca um cliente no CRM pelo nome, e-mail ou ID.",
    "input_schema": { "type": "object", "properties": { "query": { "type": "string" } }, "required": ["query"] },
    "route": "/api/tools/buscar_cliente/call"
  }
]
```

### Buscar cliente no CRM
```bash
curl -X POST http://localhost:8016/api/tools/buscar_cliente/call \
  -H "Authorization: Bearer key_vendedor_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"query": "Empresa Acme"}}'
```
```json
{
  "result": {
    "clientes": [
      { "id": "001", "nome": "Empresa Acme", "status": "ativo", "contato": "contato@acme.com" }
    ],
    "total": 1,
    "query": "Empresa Acme"
  }
}
```

### Listar oportunidades com filtro
```bash
curl -X POST http://localhost:8016/api/tools/listar_oportunidades/call \
  -H "Authorization: Bearer key_vendedor_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"cliente_id": "001", "status": "aberta"}}'
```
```json
{
  "result": {
    "oportunidades": [
      { "id": "op_42", "cliente": "Empresa Acme", "valor": 15000, "etapa": "Proposta enviada", "status": "aberta" }
    ],
    "total": 1
  }
}
```

### Registrar interação com cliente
```bash
curl -X POST http://localhost:8016/api/tools/registrar_interacao/call \
  -H "Authorization: Bearer key_vendedor_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"cliente_id": "001", "tipo": "ligacao", "descricao": "Apresentação do novo produto linha premium"}}'
```
```json
{ "result": { "sucesso": true, "mensagem": "Interacao 'ligacao' registrada para cliente 001." } }
```

### Consultar estoque no ERP
```bash
curl -X POST http://localhost:8016/api/tools/consultar_estoque/call \
  -H "Authorization: Bearer key_vendedor_01" \
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

### Consultar pedido
```bash
curl -X POST http://localhost:8016/api/tools/consultar_pedido/call \
  -H "Authorization: Bearer key_vendedor_01" \
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
    "itens": [{ "produto": "SKU-001", "quantidade": 10, "valor_unitario": 458.0 }]
  }
}
```

### Emitir nota fiscal (admin)
```bash
curl -X POST http://localhost:8016/api/tools/emitir_nota_fiscal/call \
  -H "Authorization: Bearer key_admin_00" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"numero_pedido": "PED-2024-001", "tipo": "NFe"}}'
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

### Gerar scaffold de API FastAPI
```bash
curl -X POST http://localhost:8016/api/tools/gerar_scaffold_api/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"recurso": "Produto", "campos": "nome:str, preco:float, ativo:bool"}}'
```
O resultado é código Python completo com CRUD, modelos Pydantic e roteador FastAPI.

### Gerar bot WhatsApp
```bash
curl -X POST http://localhost:8016/api/tools/gerar_bot_whatsapp/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"nome_bot": "BotVendas", "fluxos": "boas-vindas, consulta de pedido, falar com humano"}}'
```
O resultado é código Python completo com a integração Evolution API e handlers por fluxo.

### Gerar webhook handler
```bash
curl -X POST http://localhost:8016/api/tools/gerar_webhook_handler/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"servico": "Stripe", "eventos": "payment.success, payment.failed, refund.created"}}'
```
O resultado é código Python com roteador FastAPI e handlers individuais para cada evento.

---

## Uso com Claude Desktop (stdio)

Adicione ao arquivo de configuração do Claude Desktop (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "corporate-mcp": {
      "command": "C:\\caminho\\para\\mcp-orchestrator-rust.exe",
      "args": ["--stdio"],
      "env": {
        "LOG_LEVEL": "warn",
        "ENABLE_SALES": "true",
        "ENABLE_DEVTOOLS": "true",
        "ENABLE_INTERNAL_SYSTEMS": "true"
      }
    }
  }
}
```

No modo stdio:
- O servidor usa `role=admin` (acesso total a todas as tools)
- Não é necessário API key
- Um processo por sessão do Claude Desktop
- Comunica via stdin/stdout com JSON-RPC linha a linha

---

## Variáveis de ambiente

| Variável                  | Padrão                     | Descrição                                  |
|---------------------------|----------------------------|--------------------------------------------|
| `HOST`                    | `0.0.0.0`                  | Interface de escuta                        |
| `PORT`                    | `8016`                     | Porta HTTP                                 |
| `DEBUG`                   | `true`                     | Fallback auth para localhost               |
| `SERVER_NAME`             | `corporate-mcp-server`     | Nome no handshake MCP                      |
| `SERVER_VERSION`          | `1.0.0`                    | Versão no handshake MCP                    |
| `SECRET_KEY`              | `change-me-in-production`  | Chave secreta para uso futuro              |
| `API_KEYS`                | `""`                       | `user:role:key` separados por vírgula      |
| `JWT_ALGORITHM`           | `HS256`                    | Algoritmo JWT                              |
| `JWT_EXPIRE_MINUTES`      | `480`                      | Expiração de token em minutos (8h)         |
| `CORS_ORIGINS`            | `*`                        | Origens CORS permitidas                    |
| `RATE_LIMIT_PER_MINUTE`   | `60`                       | Requisições por minuto por usuário         |
| `RATE_LIMIT_BURST`        | `10`                       | Burst máximo (token bucket)                |
| `DATABASE_HOST`           | `localhost`                | Host do banco de dados                     |
| `DATABASE_PORT`           | `5432`                     | Porta do banco                             |
| `DATABASE_USER`           | `user`                     | Usuário do banco                           |
| `DATABASE_PASSWORD`       | `password`                 | Senha do banco                             |
| `DATABASE_NAME`           | `mcp_database`             | Nome do banco                              |
| `DATABASE_URL`            | `None`                     | URL completa (sobrepõe os campos acima)    |
| `REDIS_URL`               | `redis://localhost:6379/0` | URL do Redis                               |
| `LOG_LEVEL`               | `info`                     | `trace`, `debug`, `info`, `warn`, `error`  |
| `LOG_FORMAT`              | `text`                     | `json` ou `text`                           |
| `ENABLE_SALES`            | `true`                     | Habilita módulo de vendas                  |
| `ENABLE_DEVTOOLS`         | `true`                     | Habilita módulo de DevTools               |
| `ENABLE_INTERNAL_SYSTEMS` | `true`                     | Habilita módulo de sistemas internos       |
| `RAG_ACTIVATE`            | `false`                    | Habilita módulo RAG                        |
| `TRANSPORT`               | `http`                     | `http` (SSE) ou `stdio` (Claude Desktop)   |

---

## Documentação por tool

Cada tool tem sua própria documentação em `docs/tools/`:

**Sales:** [buscar_cliente](docs/tools/buscar_cliente.md) · [listar_oportunidades](docs/tools/listar_oportunidades.md) · [registrar_interacao](docs/tools/registrar_interacao.md) · [buscar_documentacao](docs/tools/buscar_documentacao.md) · [consultar_politica_comercial](docs/tools/consultar_politica_comercial.md)

**ERP:** [tools_health](docs/tools/tools_health.md) · [consultar_estoque](docs/tools/consultar_estoque.md) · [consultar_pedido](docs/tools/consultar_pedido.md) · [emitir_nota_fiscal](docs/tools/emitir_nota_fiscal.md)

**DevTools:** [gerar_scaffold_api](docs/tools/gerar_scaffold_api.md) · [gerar_schema_openapi](docs/tools/gerar_schema_openapi.md) · [gerar_bot_whatsapp](docs/tools/gerar_bot_whatsapp.md) · [gerar_webhook_handler](docs/tools/gerar_webhook_handler.md)

**RAG:** [rag_ingest](docs/tools/rag_ingest.md) · [rag_ask](docs/tools/rag_ask.md) · [rag_search](docs/tools/rag_search.md)

**Tutorial passo a passo:** [docs/TUTORIAL.md](docs/TUTORIAL.md)
