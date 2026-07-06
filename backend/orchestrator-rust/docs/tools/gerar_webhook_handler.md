# gerar_webhook_handler

Gera um handler FastAPI completo para receber webhooks de serviços externos como Stripe, GitHub, Evolution API, etc. O código inclui roteador, dispatcher de eventos e handlers individuais por tipo de evento.

**Módulo:** DevTools  
**Roles:** `dev`, `admin`  
**Arquivo:** `src/tools/devtools.rs`

---

## Parâmetros

| Parâmetro | Tipo   | Obrigatório | Descrição                                                              |
|-----------|--------|:-----------:|------------------------------------------------------------------------|
| `servico` | string | ✅          | Nome do serviço externo. Ex: `Stripe`, `GitHub`, `Evolution API`       |
| `eventos` | string | ✅          | Eventos esperados separados por vírgula. Ex: `payment.success, refund` |

---

## Exemplo — webhooks do Stripe

```bash
curl -X POST http://localhost:8016/api/tools/gerar_webhook_handler/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{
    "arguments": {
      "servico": "Stripe",
      "eventos": "payment.success, payment.failed, customer.created, refund.created"
    }
  }'
```

### Código gerado
```python
# Webhook Handler — Stripe
from fastapi import APIRouter, HTTPException, Request

router = APIRouter(prefix="/webhooks/stripe", tags=["webhooks"])


@router.post("/")
async def receive_webhook(request: Request):
    payload = await request.json()
    event_type = payload.get("event") or payload.get("type", "unknown")

    if event_type == "unknown":
        raise HTTPException(400, "Evento não identificado")
    elif event_type == "payment.success":
        await handle_payment_success(payload)
    elif event_type == "payment.failed":
        await handle_payment_failed(payload)
    elif event_type == "customer.created":
        await handle_customer_created(payload)
    elif event_type == "refund.created":
        await handle_refund_created(payload)
    else:
        # Evento não mapeado — logue e ignore
        pass

    return {"received": True}


# ─── Handlers de eventos ──────────────────────────────────────────────────────
async def handle_payment_success(payload: dict):
    # TODO: processar evento "payment.success"
    pass

async def handle_payment_failed(payload: dict):
    # TODO: processar evento "payment.failed"
    pass

async def handle_customer_created(payload: dict):
    # TODO: processar evento "customer.created"
    pass

async def handle_refund_created(payload: dict):
    # TODO: processar evento "refund.created"
    pass
```

---

## Exemplo — webhooks do GitHub

```bash
curl -X POST http://localhost:8016/api/tools/gerar_webhook_handler/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{
    "arguments": {
      "servico": "GitHub",
      "eventos": "push, pull_request, issues, release"
    }
  }'
```

---

## Exemplo — webhooks da Evolution API (WhatsApp)

```bash
curl -X POST http://localhost:8016/api/tools/gerar_webhook_handler/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{
    "arguments": {
      "servico": "EvolutionAPI",
      "eventos": "messages.upsert, connection.update, qrcode.updated"
    }
  }'
```

---

## Como implementar os handlers

```python
async def handle_payment_success(payload: dict):
    amount = payload.get("data", {}).get("amount", 0)
    customer_id = payload.get("data", {}).get("customer")

    # 1. Registrar pagamento no banco
    await db.execute(
        "UPDATE pedidos SET status='pago' WHERE stripe_payment_id=$1",
        payload["id"]
    )

    # 2. Enviar confirmação por e-mail
    await email_client.send(
        to=customer_id,
        subject="Pagamento confirmado!",
        body=f"Recebemos seu pagamento de R$ {amount/100:.2f}."
    )
```

---

## Incluir na aplicação FastAPI

```python
# main.py
from fastapi import FastAPI
from stripe_webhook import router as stripe_router

app = FastAPI()
app.include_router(stripe_router)
```

---

## Salvar código gerado

```bash
curl -s -X POST http://localhost:8016/api/tools/gerar_webhook_handler/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"servico": "Stripe", "eventos": "payment.success, payment.failed"}}' \
  | python3 -c "import sys,json; print(json.load(sys.stdin)['result'])" \
  > stripe_webhook.py
```
