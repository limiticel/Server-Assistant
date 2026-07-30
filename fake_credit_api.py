from hashlib import sha256
import json
from pathlib import Path
from typing import Literal
from uuid import uuid4

from fastapi import FastAPI, HTTPException, Query
from pydantic import BaseModel


app = FastAPI(title="Fake Credit API", version="0.1.0")
PROPOSALS_FILE = Path(__file__).with_name(".fake_credit_proposals.json")


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


class ProposalStatusQuery(BaseModel):
    proposal_id: str | None = None
    cpf: str | None = None


def only_digits(value: str) -> str:
    return "".join(char for char in value if char.isdigit())


def load_proposals() -> dict[str, dict]:
    if not PROPOSALS_FILE.exists():
        return {}

    try:
        data = json.loads(PROPOSALS_FILE.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}

    if not isinstance(data, dict):
        return {}

    return {
        str(key): value
        for key, value in data.items()
        if isinstance(value, dict)
    }


def save_proposals(proposals: dict[str, dict]) -> None:
    PROPOSALS_FILE.write_text(
        json.dumps(proposals, ensure_ascii=True, indent=2),
        encoding="utf-8",
    )


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


def normalize_proposal_id(proposal_id: str) -> str:
    return proposal_id.strip().upper()


def find_proposal(proposal_id: str | None = None, cpf: str | None = None) -> dict:
    proposals = load_proposals()

    if proposal_id:
        proposal = proposals.get(normalize_proposal_id(proposal_id))
        if proposal:
            return proposal

    if cpf:
        clean_cpf = only_digits(cpf)
        matches = [proposal for proposal in proposals.values() if proposal["cpf"] == clean_cpf]
        if matches:
            return matches[-1]

    raise HTTPException(
        status_code=404,
        detail="Proposta nao encontrada. Informe proposal_id retornado no cadastro ou o CPF usado na proposta.",
    )


def build_proposal_status_response(proposal: dict) -> ProposalStatusResponse:
    proposals = load_proposals()

    proposal["status_checks"] += 1

    if proposal["status"] == "created" and proposal["status_checks"] >= 2:
        proposal["status"] = "approved"

    proposals[proposal["proposal_id"]] = proposal
    save_proposals(proposals)

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


@app.get("/health")
def health():
    return {"status": "ok", "proposals_count": len(load_proposals())}


@app.get("/credit/simulate", response_model=CreditSimulation)
def simulate_credit(cpf: str = Query(..., description="CPF com ou sem pontuacao")):
    return build_credit_simulation(cpf)


@app.post("/credit/simulate", response_model=CreditSimulation)
def simulate_credit_post(payload: CreditSimulationRequest):
    return build_credit_simulation(payload.cpf)


@app.post("/proposals", response_model=ProposalCreateResponse)
def create_proposal(payload: ProposalCreateRequest):
    simulation = build_credit_simulation(payload.cpf)
    proposals = load_proposals()

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

    proposals[proposal_id] = {
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
    save_proposals(proposals)

    return ProposalCreateResponse(
        proposal_id=proposal_id,
        cpf=simulation.cpf,
        customer_name=payload.customer_name,
        requested_amount=payload.requested_amount,
        term_months=payload.term_months,
        status=status,
        message=message,
    )


@app.post("/proposal", response_model=ProposalCreateResponse)
def create_proposal_alias(payload: ProposalCreateRequest):
    return create_proposal(payload)


@app.get("/proposals/{proposal_id}/status", response_model=ProposalStatusResponse)
def proposal_status(proposal_id: str):
    proposal = find_proposal(proposal_id=proposal_id)
    return build_proposal_status_response(proposal)


@app.get("/proposal/{proposal_id}/status", response_model=ProposalStatusResponse)
def proposal_status_alias(proposal_id: str):
    return proposal_status(proposal_id)


@app.get("/proposals/status", response_model=ProposalStatusResponse)
def proposal_status_query(proposal_id: str | None = None, cpf: str | None = None):
    proposal = find_proposal(proposal_id=proposal_id, cpf=cpf)
    return build_proposal_status_response(proposal)


@app.get("/proposal/status", response_model=ProposalStatusResponse)
def proposal_status_query_alias(proposal_id: str | None = None, cpf: str | None = None):
    return proposal_status_query(proposal_id=proposal_id, cpf=cpf)


@app.post("/proposals/status", response_model=ProposalStatusResponse)
def proposal_status_post(payload: ProposalStatusQuery):
    proposal = find_proposal(proposal_id=payload.proposal_id, cpf=payload.cpf)
    return build_proposal_status_response(proposal)


@app.post("/proposal/status", response_model=ProposalStatusResponse)
def proposal_status_post_alias(payload: ProposalStatusQuery):
    return proposal_status_post(payload)
