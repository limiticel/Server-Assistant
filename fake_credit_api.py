from hashlib import sha256
from typing import Literal
from uuid import uuid4

from fastapi import FastAPI, HTTPException, Query
from pydantic import BaseModel


app = FastAPI(title="Fake Credit API", version="0.1.0")
PROPOSALS: dict[str, dict] = {}


class CreditSimulation(BaseModel):
    cpf: str
    status: Literal["approved", "manual_review", "denied"]
    score: int
    credit_limit: float
    interest_rate_monthly: float
    term_months: int
    message: str


class CreditSimulationRequest(BaseModel):
    cpf: str


class ProposalCreateRequest(BaseModel):
    cpf: str
    customer_name: str
    requested_amount: float
    term_months: int


class ProposalCreateResponse(BaseModel):
    proposal_id: str
    cpf: str
    customer_name: str
    requested_amount: float
    term_months: int
    status: Literal["created", "manual_review", "rejected"]
    message: str


class ProposalStatusResponse(BaseModel):
    proposal_id: str
    cpf: str
    customer_name: str
    requested_amount: float
    term_months: int
    status: Literal["created", "manual_review", "approved", "rejected", "cancelled"]
    score: int
    credit_limit: float
    message: str


def only_digits(value: str) -> str:
    return "".join(char for char in value if char.isdigit())


def fake_number(seed: str, minimum: int, maximum: int) -> int:
    digest = sha256(seed.encode("utf-8")).hexdigest()
    number = int(digest[:12], 16)
    return minimum + (number % (maximum - minimum + 1))


def build_credit_simulation(cpf: str) -> CreditSimulation:
    clean_cpf = only_digits(cpf)

    if len(clean_cpf) != 11:
        raise HTTPException(status_code=400, detail="CPF deve ter 11 digitos.")

    score = fake_number(clean_cpf, 250, 950)
    term_months = fake_number(f"{clean_cpf}:term", 6, 48)

    if score >= 720:
        status: Literal["approved", "manual_review", "denied"] = "approved"
        credit_limit = float(fake_number(f"{clean_cpf}:limit", 5000, 50000))
        interest_rate = round(fake_number(f"{clean_cpf}:rate", 149, 329) / 100, 2)
        message = "Credito pre-aprovado na simulacao fake."
    elif score >= 520:
        status = "manual_review"
        credit_limit = float(fake_number(f"{clean_cpf}:limit", 1000, 12000))
        interest_rate = round(fake_number(f"{clean_cpf}:rate", 299, 599) / 100, 2)
        message = "Simulacao encaminhada para analise manual fake."
    else:
        status = "denied"
        credit_limit = 0.0
        interest_rate = 0.0
        message = "Credito recusado na simulacao fake."

    return CreditSimulation(
        cpf=clean_cpf,
        status=status,
        score=score,
        credit_limit=credit_limit,
        interest_rate_monthly=interest_rate,
        term_months=term_months,
        message=message,
    )


@app.get("/health")
def health():
    return {"status": "ok", "proposals_count": len(PROPOSALS)}


@app.get("/credit/simulate", response_model=CreditSimulation)
def simulate_credit(cpf: str = Query(..., description="CPF com ou sem pontuacao")):
    return build_credit_simulation(cpf)


@app.post("/credit/simulate", response_model=CreditSimulation)
def simulate_credit_post(payload: CreditSimulationRequest):
    return build_credit_simulation(payload.cpf)


@app.post("/proposals", response_model=ProposalCreateResponse)
def create_proposal(payload: ProposalCreateRequest):
    simulation = build_credit_simulation(payload.cpf)

    if payload.requested_amount <= 0:
        raise HTTPException(status_code=400, detail="Valor solicitado deve ser maior que zero.")
    if payload.term_months < 1 or payload.term_months > 84:
        raise HTTPException(status_code=400, detail="Prazo deve ficar entre 1 e 84 meses.")

    proposal_id = f"PROP-{uuid4().hex[:10].upper()}"

    if simulation.status == "denied" or payload.requested_amount > simulation.credit_limit:
        status: Literal["created", "manual_review", "rejected"] = "rejected"
        message = "Proposta recusada na API fake."
    elif simulation.status == "manual_review":
        status = "manual_review"
        message = "Proposta criada e enviada para analise manual fake."
    else:
        status = "created"
        message = "Proposta criada com sucesso na API fake."

    PROPOSALS[proposal_id] = {
        "proposal_id": proposal_id,
        "cpf": simulation.cpf,
        "customer_name": payload.customer_name,
        "requested_amount": payload.requested_amount,
        "term_months": payload.term_months,
        "status": status,
        "score": simulation.score,
        "credit_limit": simulation.credit_limit,
        "status_checks": 0,
    }

    return ProposalCreateResponse(
        proposal_id=proposal_id,
        cpf=simulation.cpf,
        customer_name=payload.customer_name,
        requested_amount=payload.requested_amount,
        term_months=payload.term_months,
        status=status,
        message=message,
    )


@app.get("/proposals/{proposal_id}/status", response_model=ProposalStatusResponse)
def proposal_status(proposal_id: str):
    proposal = PROPOSALS.get(proposal_id)
    if not proposal:
        raise HTTPException(status_code=404, detail="Proposta nao encontrada.")

    proposal["status_checks"] += 1

    if proposal["status"] == "created" and proposal["status_checks"] >= 2:
        proposal["status"] = "approved"

    status = proposal["status"]
    if status == "approved":
        message = "Proposta aprovada na verificacao fake."
    elif status == "manual_review":
        message = "Proposta ainda em analise manual fake."
    elif status == "rejected":
        message = "Proposta recusada na verificacao fake."
    else:
        message = "Proposta criada e aguardando processamento fake."

    return ProposalStatusResponse(
        proposal_id=proposal["proposal_id"],
        cpf=proposal["cpf"],
        customer_name=proposal["customer_name"],
        requested_amount=proposal["requested_amount"],
        term_months=proposal["term_months"],
        status=status,
        score=proposal["score"],
        credit_limit=proposal["credit_limit"],
        message=message,
    )
