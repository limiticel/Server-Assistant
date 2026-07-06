# gerar_bot_whatsapp

Gera código base completo de um bot WhatsApp via Evolution API (ou Baileys) com os fluxos de conversa descritos. O código gerado inclui conexão com a API, roteador de mensagens e handlers individuais por fluxo.

**Módulo:** DevTools  
**Roles:** `dev`, `admin`  
**Arquivo:** `src/tools/devtools.rs`

---

## Parâmetros

| Parâmetro  | Tipo   | Obrigatório | Descrição                                                              |
|------------|--------|:-----------:|------------------------------------------------------------------------|
| `nome_bot` | string | ✅          | Nome do bot                                                            |
| `fluxos`   | string | ✅          | Fluxos separados por vírgula. Ex: `boas-vindas, consulta de pedido`   |

---

## Exemplo — bot de atendimento completo

```bash
curl -X POST http://localhost:8016/api/tools/gerar_bot_whatsapp/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{
    "arguments": {
      "nome_bot": "BotAtendimento",
      "fluxos": "boas-vindas, consulta de pedido, segunda via boleto, falar com humano"
    }
  }'
```

### Código gerado
```python
# Bot WhatsApp — BotAtendimento
# Gerado automaticamente pelo MCP DevTools
# Integração: Evolution API (substitua pelo seu provider)

import httpx

EVOLUTION_URL = "https://sua-evolution-api.com"
INSTANCE = "sua-instancia"
API_KEY = "sua-api-key"


async def send_message(to: str, text: str):
    async with httpx.AsyncClient() as client:
        await client.post(
            f"{EVOLUTION_URL}/message/sendText/{INSTANCE}",
            json={"number": to, "text": text},
            headers={"apikey": API_KEY},
        )


async def handle_message(message: dict):
    text = message.get("body", "").strip().lower()

    # Roteador de fluxos
    if "boas-vindas" in text:
        await fluxo_boas_vindas(message)
    if "consulta de pedido" in text:
        await fluxo_consulta_de_pedido(message)
    if "segunda via boleto" in text:
        await fluxo_segunda_via_boleto(message)
    if "falar com humano" in text:
        await fluxo_falar_com_humano(message)
    else:
        await send_message(message["from"], "Olá! Como posso ajudar?")


# ─── Fluxos ───────────────────────────────────────────────────────────────────
async def fluxo_boas_vindas(message):
    # TODO: implementar fluxo "boas-vindas"
    await send_message(message["from"], "Fluxo boas-vindas em construção.")

async def fluxo_consulta_de_pedido(message):
    # TODO: implementar fluxo "consulta de pedido"
    await send_message(message["from"], "Fluxo consulta de pedido em construção.")

async def fluxo_segunda_via_boleto(message):
    # TODO: implementar fluxo "segunda via boleto"
    await send_message(message["from"], "Fluxo segunda via boleto em construção.")

async def fluxo_falar_com_humano(message):
    # TODO: implementar fluxo "falar com humano"
    await send_message(message["from"], "Fluxo falar com humano em construção.")
```

---

## Exemplo — bot de vendas simples

```bash
curl -X POST http://localhost:8016/api/tools/gerar_bot_whatsapp/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{
    "arguments": {
      "nome_bot": "BotVendas",
      "fluxos": "catalogo de produtos, fazer pedido, rastrear entrega"
    }
  }'
```

---

## Como implementar os fluxos

Depois de gerar o código, implemente cada função `fluxo_*`:

```python
async def fluxo_consulta_de_pedido(message):
    numero = extrair_numero_pedido(message["body"])
    if not numero:
        await send_message(message["from"], "Por favor, informe o número do pedido.")
        return

    # Consulta ao ERP (ou chame a tool consultar_pedido via MCP)
    pedido = await buscar_pedido(numero)
    resposta = f"Pedido {numero}: {pedido['status']}\nPrevisão: {pedido['previsao_entrega']}"
    await send_message(message["from"], resposta)
```

---

## Salvar e executar o bot

```bash
# Salvar código gerado
curl -s -X POST http://localhost:8016/api/tools/gerar_bot_whatsapp/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"nome_bot": "BotVendas", "fluxos": "boas-vindas, catalogo"}}' \
  | python3 -c "import sys,json; print(json.load(sys.stdin)['result'])" \
  > bot_vendas.py

# Instalar dependências
pip install httpx fastapi uvicorn

# Integrar com Evolution API via webhook (adicione o endpoint de recebimento):
# POST /webhook → chama handle_message(payload)
```
