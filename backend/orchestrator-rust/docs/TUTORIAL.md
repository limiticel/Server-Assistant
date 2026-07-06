# Tutorial — mcp-orchestrator-rust

Tutorial passo a passo para levantar o servidor, configurar e usar cada tool.
Execute os comandos na ordem. Cada passo tem verificação do resultado esperado.

---

## Passo 1 — Pré-requisitos

Certifique-se de ter o Rust instalado:

```bash
rustup --version
# Esperado: rustup 1.27+ ou superior
```

Se não tiver:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

No Windows (PowerShell):
```powershell
winget install Rustlang.Rustup
# ou baixe em https://rustup.rs
```

---

## Passo 2 — Clonar e entrar na pasta

```bash
cd mcp-orchestrator-rust
```

Verifique que os arquivos estão lá:
```bash
ls
# Esperado: Cargo.toml  Cargo.lock  README.md  src/  docs/
```

---

## Passo 3 — Compilar o projeto

```bash
cargo build
```

Na primeira vez vai baixar dependências (~30s). Esperado ao final:
```
Finished `dev` profile [unoptimized + debuginfo] target(s) in Xs
```

Para produção (binário ~5x menor e mais rápido):
```bash
cargo build --release
```

---

## Passo 4 — Criar o arquivo de configuração `.env`

Crie o arquivo `.env` na raiz do projeto:

```bash
# Linux/macOS
cat > .env << 'EOF'
HOST=0.0.0.0
PORT=8016
DEBUG=true
SERVER_NAME=corporate-mcp-server
SERVER_VERSION=1.0.0
SECRET_KEY=minha-chave-secreta-local
API_KEYS=joao:sales:key_sales_01,maria:dev:key_dev_01,root:admin:key_admin_00
RATE_LIMIT_PER_MINUTE=60
RATE_LIMIT_BURST=10
LOG_LEVEL=info
LOG_FORMAT=text
ENABLE_SALES=true
ENABLE_DEVTOOLS=true
ENABLE_INTERNAL_SYSTEMS=true
RAG_ACTIVATE=false
TRANSPORT=http
EOF
```

No Windows (PowerShell):
```powershell
@"
HOST=0.0.0.0
PORT=8016
DEBUG=true
SERVER_NAME=corporate-mcp-server
SERVER_VERSION=1.0.0
SECRET_KEY=minha-chave-secreta-local
API_KEYS=joao:sales:key_sales_01,maria:dev:key_dev_01,root:admin:key_admin_00
RATE_LIMIT_PER_MINUTE=60
RATE_LIMIT_BURST=10
LOG_LEVEL=info
LOG_FORMAT=text
ENABLE_SALES=true
ENABLE_DEVTOOLS=true
ENABLE_INTERNAL_SYSTEMS=true
RAG_ACTIVATE=false
TRANSPORT=http
"@ | Out-File -Encoding utf8 .env
```

> **Atenção:** Em produção, mude `DEBUG=false`, use `LOG_FORMAT=json` e coloque chaves reais em `API_KEYS`.

---

## Passo 5 — Subir o servidor

```bash
cargo run
```

Você deve ver no terminal:
```
INFO  mcp_orchestrator_rust: starting mcp-orchestrator-rust (HTTP+SSE) addr=0.0.0.0:8016
```

Deixe este terminal aberto. Abra **um novo terminal** para os próximos passos.

---

## Passo 6 — Verificar saúde do servidor

```bash
curl http://localhost:8016/health
```

Resposta esperada:
```json
{
  "status": "ok",
  "server": "corporate-mcp-server",
  "version": "1.0.0",
  "tools_count": 13
}
```

Se `tools_count` for 13, está tudo certo (4 devtools + 5 sales + 4 ERP).

---

## Passo 7 — Listar todas as rotas

```bash
curl http://localhost:8016/api/all-routes
```

Você verá a lista de todos os endpoints registrados no servidor.

---

## Passo 8 — Listar tools (modo debug)

Este endpoint só funciona com `DEBUG=true`:

```bash
curl http://localhost:8016/tools
```

Retorna todas as tools com nome e descrição, sem autenticação.

---

## Passo 9 — Autenticar e ver tools disponíveis

Usando a chave do vendedor (role `sales`):
```bash
curl http://localhost:8016/api/tools \
  -H "Authorization: Bearer key_sales_01"
```

Usando a chave do dev (role `dev`):
```bash
curl http://localhost:8016/api/tools \
  -H "Authorization: Bearer key_dev_01"
```

Usando a chave de admin:
```bash
curl http://localhost:8016/api/tools \
  -H "Authorization: Bearer key_admin_00"
```

Cada role vê um subconjunto diferente de tools. Compare os resultados.

---

## Passo 10 — Ver detalhes de uma tool específica

```bash
curl http://localhost:8016/api/tools/buscar_cliente \
  -H "Authorization: Bearer key_sales_01"
```

```json
{
  "name": "buscar_cliente",
  "description": "Busca um cliente no CRM pelo nome, e-mail ou ID. Retorna dados cadastrais e status.",
  "input_schema": {
    "type": "object",
    "properties": {
      "query": { "type": "string", "description": "Nome, e-mail ou ID do cliente" }
    },
    "required": ["query"]
  },
  "route": "/api/tools/buscar_cliente/call"
}
```

---

## Passo 11 — Executar tools de Sales

### buscar_cliente
```bash
curl -X POST http://localhost:8016/api/tools/buscar_cliente/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"query": "Empresa Acme"}}'
```

### listar_oportunidades
```bash
curl -X POST http://localhost:8016/api/tools/listar_oportunidades/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"status": "aberta"}}'
```

### registrar_interacao
```bash
curl -X POST http://localhost:8016/api/tools/registrar_interacao/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"cliente_id": "001", "tipo": "reuniao", "descricao": "Demo do produto realizada com sucesso"}}'
```

### buscar_documentacao
```bash
curl -X POST http://localhost:8016/api/tools/buscar_documentacao/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"pergunta": "como cadastrar um cliente", "sistema": "crm"}}'
```

### consultar_politica_comercial
```bash
curl -X POST http://localhost:8016/api/tools/consultar_politica_comercial/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"topico": "desconto maximo"}}'
```
Resposta: `"Desconto maximo permitido sem aprovacao: 10%. Acima, requer aval do gerente."`

---

## Passo 12 — Executar tools de ERP / Internal Systems

### tools_health (sem autenticação necessária)
```bash
curl -X POST http://localhost:8016/api/tools/tools_health/call \
  -H "Content-Type: application/json" \
  -d '{"arguments": {}}'
```

> **Dica:** Como `tools_health` tem role `*`, qualquer usuário (ou nenhum em DEBUG) pode chamá-la.

### consultar_estoque
```bash
curl -X POST http://localhost:8016/api/tools/consultar_estoque/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"produto": "Produto Premium XL"}}'
```

### consultar_pedido
```bash
curl -X POST http://localhost:8016/api/tools/consultar_pedido/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"numero_pedido": "PED-2024-042"}}'
```

### emitir_nota_fiscal (apenas admin)
```bash
curl -X POST http://localhost:8016/api/tools/emitir_nota_fiscal/call \
  -H "Authorization: Bearer key_admin_00" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"numero_pedido": "PED-2024-042", "tipo": "NFe"}}'
```

Tente com a chave `key_sales_01` — você deve receber `404 Not Found` (sem permissão).

---

## Passo 13 — Executar tools de DevTools

### gerar_scaffold_api
```bash
curl -X POST http://localhost:8016/api/tools/gerar_scaffold_api/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"recurso": "Produto", "campos": "nome:str, preco:float, ativo:bool, categoria:str"}}'
```

O resultado é código Python completo. Para salvar em arquivo:
```bash
curl -s -X POST http://localhost:8016/api/tools/gerar_scaffold_api/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"recurso": "Produto", "campos": "nome:str, preco:float, ativo:bool"}}' \
  | python3 -c "import sys,json; print(json.load(sys.stdin)['result'])" \
  > produto_router.py
```

### gerar_schema_openapi
```bash
curl -X POST http://localhost:8016/api/tools/gerar_schema_openapi/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"descricao_endpoint": "POST /pedidos que recebe cliente_id e lista de itens e retorna o pedido criado com total calculado"}}'
```

### gerar_bot_whatsapp
```bash
curl -X POST http://localhost:8016/api/tools/gerar_bot_whatsapp/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"nome_bot": "BotAtendimento", "fluxos": "boas-vindas, consulta de pedido, segunda via boleto, falar com humano"}}'
```

### gerar_webhook_handler
```bash
curl -X POST http://localhost:8016/api/tools/gerar_webhook_handler/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"servico": "Stripe", "eventos": "payment.success, payment.failed, customer.created, refund.created"}}'
```

---

## Passo 14 — Testar o rate limit

Execute a mesma chamada rapidamente mais de 10 vezes (burst limit):

```bash
for i in $(seq 1 15); do
  curl -s -o /dev/null -w "%{http_code}\n" \
    -X POST http://localhost:8016/api/tools/tools_health/call \
    -H "Authorization: Bearer key_sales_01" \
    -H "Content-Type: application/json" \
    -d '{"arguments": {}}'
done
```

Você verá `200` nas primeiras chamadas e `429` quando o burst for atingido.

---

## Passo 15 — Testar erro de autenticação

```bash
# Sem API key (fora de DEBUG ou host não-local)
curl http://localhost:8016/api/tools \
  -H "Authorization: Bearer chave-invalida"
```
Resposta: `401 Unauthorized`

```bash
# Tentando emitir NF com role de sales
curl -X POST http://localhost:8016/api/tools/emitir_nota_fiscal/call \
  -H "Authorization: Bearer key_sales_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"numero_pedido": "001"}}'
```
Resposta: `404 Not Found` (tool não visível para este role)

---

## Passo 16 — Habilitar e usar o módulo RAG

Pare o servidor (`Ctrl+C`) e adicione ao `.env`:
```env
RAG_ACTIVATE=true
```

Reinicie:
```bash
cargo run
```

Verifique que as tools RAG foram carregadas:
```bash
curl http://localhost:8016/health
# tools_count deve ser 16 agora
```

Teste `rag_ingest`:
```bash
curl -X POST http://localhost:8016/api/tools/rag_ingest/call \
  -H "Authorization: Bearer key_admin_00" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"path": "/docs/manual.pdf"}}'
```

Teste `rag_search`:
```bash
curl -X POST http://localhost:8016/api/tools/rag_search/call \
  -H "Authorization: Bearer key_admin_00" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"query": "política de devolução", "top_k": 5}}'
```

---

## Passo 17 — Usar via protocolo MCP (SSE)

O protocolo MCP real usa SSE. Veja como conectar manualmente:

**Terminal 1 — Abrir sessão SSE:**
```bash
curl -N -H "Authorization: Bearer key_sales_01" \
  http://localhost:8016/sse
```

Você receberá algo como:
```
event: endpoint
data: /messages?session_id=1
```

**Terminal 2 — Inicializar a sessão MCP:**
```bash
curl -X POST "http://localhost:8016/messages?session_id=1" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}'
```

**Terminal 2 — Listar tools via protocolo MCP:**
```bash
curl -X POST "http://localhost:8016/messages?session_id=1" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}'
```

**Terminal 2 — Chamar uma tool via protocolo MCP:**
```bash
curl -X POST "http://localhost:8016/messages?session_id=1" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "buscar_cliente",
      "arguments": { "query": "Acme" }
    }
  }'
```

Todas as respostas chegam via SSE no Terminal 1.

---

## Passo 18 — Usar com Claude Desktop (stdio)

**Build release:**
```bash
cargo build --release
```

**Configure o Claude Desktop.** Localize o arquivo:
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`

Adicione ao JSON:
```json
{
  "mcpServers": {
    "corporate-mcp": {
      "command": "C:\\caminho\\completo\\para\\mcp-orchestrator-rust.exe",
      "args": ["--stdio"],
      "env": {
        "ENABLE_SALES": "true",
        "ENABLE_DEVTOOLS": "true",
        "ENABLE_INTERNAL_SYSTEMS": "true",
        "LOG_LEVEL": "warn"
      }
    }
  }
}
```

**Reinicie o Claude Desktop.**

Agora você pode perguntar ao Claude:
- *"Busque o cliente Empresa Acme no CRM"*
- *"Qual é o estoque do produto SKU-001?"*
- *"Gere um scaffold FastAPI para o recurso Pedido"*
- *"Crie um bot WhatsApp com os fluxos: boas-vindas, rastreio de pedido"*

---

## Passo 19 — Build para produção

```bash
# Build otimizado
cargo build --release

# Defina variáveis de produção
export DEBUG=false
export LOG_FORMAT=json
export API_KEYS=usuario:role:chave_segura_real
export SECRET_KEY=chave-super-secreta-256-bits
export RATE_LIMIT_PER_MINUTE=30

# Execute
./target/release/mcp-orchestrator-rust
```

No Windows:
```powershell
$env:DEBUG = "false"
$env:LOG_FORMAT = "json"
$env:API_KEYS = "usuario:role:chave_segura_real"
.\target\release\mcp-orchestrator-rust.exe
```

---

## Resumo dos endpoints usados neste tutorial

| Endpoint                              | O que faz                          |
|---------------------------------------|------------------------------------|
| `GET /health`                         | Verifica se o servidor está vivo   |
| `GET /tools`                          | Lista tools (DEBUG)                |
| `GET /api/tools`                      | Tools do usuário logado            |
| `GET /api/tools/:name`                | Detalhes de uma tool               |
| `POST /api/tools/:name/call`          | Executa uma tool                   |
| `GET /sse`                            | Abre stream SSE (protocolo MCP)    |
| `POST /messages?session_id=<id>`      | Envia mensagem MCP                 |

---

## Próximos passos

- Leia a documentação individual de cada tool em [docs/tools/](tools/)
- Adapte os handlers em `src/tools/` para suas integrações reais (ERP, CRM, banco de dados)
- Configure `DATABASE_URL` e `REDIS_URL` para persistência real
- Implemente o pipeline RAG completo em `src/tools/rag.rs`
