# Self Hosted AI Platform

Plataforma de IA auto-hospedada inspirada no ChatGPT, com backend Rust/Axum, frontend Vue 3, function calling e ferramentas dinamicas criadas pelo painel.

## Estrutura

- `backend/`: API Rust com arquitetura limpa, JWT, providers de IA, SSE, gateway compativel com OpenAI e executor local de tools.
- `frontend/`: Vue 3 + Vite + TypeScript + Pinia + Vue Router + Axios + TailwindCSS.
- `deploy/`: Nginx, systemd e Docker Compose.
- `scripts/`: instalacao, backup e restore.

## Desenvolvimento

```bash
cp .env.example .env
docker compose up -d postgres redis
cd backend && cargo run
cd frontend && npm install && npm run dev
```

## Ferramentas Dinamicas

As ferramentas sao criadas no painel em `/admin/mcp-tools`, salvas na tabela `mcp_tools` e executadas pelo proprio backend. O chat carrega as ferramentas atribuidas ao modelo em `model_mcp_tools`, envia os schemas para o provider como function calling e executa a chamada localmente quando o modelo escolhe uma tool.

Tipos suportados no executor local:

- `kind: "api"` ou `"http"`: chama uma API HTTP/HTTPS usando `method`, `url`, `headers`, `query`, `body` e `timeout_seconds`.
- `kind: "infra"`: executa acoes de infraestrutura com acesso total. Os parametros esperados definem o runtime e os dados necessarios, por exemplo `runtime: "local"` ou `runtime: "ssh"`.
- `kind: "ssh"` ou `"infra_ssh"`: executa comandos SSH nao interativos em um servidor autorizado.
- `kind: "abstract"`, `"static"` ou `"text"`: retorna texto/instrucoes com interpolacao simples de argumentos.

Templates podem usar `{{nome_do_parametro}}` em `url`, `headers`, `query`, `body` e respostas estaticas.

## Rotas Principais

- `POST /api/auth/login`
- `POST /api/auth/register`
- `GET /api/conversations`
- `POST /api/conversations`
- `POST /api/chat/:conversation_id/messages`
- `GET /api/chat/:conversation_id/stream`
- `GET /api/admin/dashboard`
- `GET /api/admin/providers`
- `GET /api/admin/models`
- `GET /api/admin/personalities`
- `GET /api/admin/mcp-tools`
- `POST /api/admin/mcp-tools`
- `GET /api/mcp/tools`
- `POST /api/mcp/tools/:name/call`
- `GET /v1/models`
- `POST /v1/chat/completions`
- `POST /v1/embeddings`

## Testar Um Provider No Chat

1. Abra `http://localhost:48117/admin/providers`.
2. Escolha um preset, por exemplo OpenAI, DeepSeek, Ollama ou Compativel.
3. Preencha `Base URL`, `API Key` e `Modelo padrao`.
4. Salve o provider.
5. Abra `http://localhost:48117/`, selecione o provider/modelo no topo do chat e envie uma mensagem.

Exemplos de Base URL:

- OpenAI: `https://api.openai.com/v1`
- DeepSeek: `https://api.deepseek.com/v1`
- Ollama: `http://localhost:11434/v1`
- API compativel com OpenAI: use a URL `/v1` do servico.

Nesta fase local, a chave e gravada no campo `api_key_cipher` para permitir teste rapido. Para producao, substitua por criptografia/secret manager.

## Exemplo De Tool API

Config:

```json
{
  "kind": "api",
  "method": "POST",
  "url": "https://api.exemplo.com/clientes",
  "headers": {
    "Authorization": "Bearer {{api_key}}",
    "Content-Type": "application/json"
  },
  "body": {
    "nome": "{{nome}}",
    "email": "{{email}}"
  },
  "timeout_seconds": 30
}
```

Payload de teste:

```json
{
  "arguments": {
    "api_key": "token",
    "nome": "Acme",
    "email": "financeiro@acme.com"
  }
}
```

Rota:

```bash
POST /api/mcp/tools/minha_tool/call
```

## Producao

```bash
cp .env.example .env
docker compose up -d --build
sudo cp deploy/systemd/server-assistant.service /etc/systemd/system/
sudo systemctl enable --now server-assistant
```
