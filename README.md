# Self Hosted AI Platform

Plataforma de IA auto-hospedada inspirada no ChatGPT, com backend Rust/Axum, frontend Vue 3 e integração planejada com `limiticel/orchestrator-rust` para MCP, contexto e function calling.

## Estrutura

- `backend/`: API Rust com arquitetura limpa, JWT, providers de IA, SSE e gateway compatível com OpenAI.
- `frontend/`: Vue 3 + Vite + TypeScript + Pinia + Vue Router + Axios + TailwindCSS.
- `deploy/`: Nginx, systemd e Docker Compose.
- `scripts/`: instalação, backup, restore e vendorização do Orchestrator.

## Desenvolvimento

```bash
cp .env.example .env
docker compose up -d postgres redis
cd backend && cargo run
cd frontend && npm install && npm run dev
```

## Orchestrator-Rust

O projeto externo fica em `backend/orchestrator-rust`. O backend possui apenas um adapter HTTP em `backend/src/infrastructure/mcp/orchestrator.rs`; a criacao, registro e execucao de ferramentas MCP acontece no Orchestrator.

```bash
./scripts/vendor_orchestrator.sh
```

Fonte consultada: https://github.com/limiticel/orchestrator-rust

## Rotas principais

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
- `GET /api/mcp/tools`
- `POST /api/mcp/tools/:name/call`
- `GET /v1/models`
- `POST /v1/chat/completions`
- `POST /v1/embeddings`

## Testar Um Provider No Chat

1. Abra `http://localhost:5173/admin/providers`.
2. Escolha um preset, por exemplo OpenAI, DeepSeek, Ollama ou Compatível.
3. Preencha `Base URL`, `API Key` e `Modelo padrão`.
4. Salve o provider.
5. Abra `http://localhost:5173/`, selecione o provider/modelo no topo do chat e envie uma mensagem.

Exemplos de Base URL:

- OpenAI: `https://api.openai.com/v1`
- DeepSeek: `https://api.deepseek.com/v1`
- Ollama: `http://localhost:11434/v1`
- API compatível com OpenAI: use a URL `/v1` do serviço.

Nesta fase local, a chave é gravada no campo `api_key_cipher` para permitir teste rápido. Para produção, substitua por criptografia/secret manager.

## Ferramentas MCP Internas

### `ubuntu_server_ssh`

Registrada em `backend/orchestrator-rust/src/tools/server_admin.rs`. Executa comandos no Ubuntu Server por SSH usando `127.0.0.1:2222`.

Payload:

```json
{
  "arguments": {
    "username": "usuario_ssh",
    "command": "hostname",
    "timeout_seconds": 30
  }
}
```

Rota:

```bash
POST /api/mcp/tools/ubuntu_server_ssh/call
```

Para automação, configure autenticação por chave SSH ou defina `UBUNTU_SSH_DEFAULT_USER` no `.env`.

## Produção

```bash
cp .env.example .env
docker compose up -d --build
sudo cp deploy/systemd/server-assistant.service /etc/systemd/system/
sudo systemctl enable --now server-assistant
```
