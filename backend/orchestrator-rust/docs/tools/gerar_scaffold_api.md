# gerar_scaffold_api

Gera código scaffold completo de uma API REST (FastAPI + Pydantic) a partir do nome do recurso e seus campos. O código gerado inclui modelo, CRUD completo e roteador pronto para uso.

**Módulo:** DevTools  
**Roles:** `dev`, `admin`  
**Arquivo:** `src/tools/devtools.rs`

---

## Parâmetros

| Parâmetro | Tipo   | Obrigatório | Descrição                                                            |
|-----------|--------|:-----------:|----------------------------------------------------------------------|
| `recurso` | string | ✅          | Nome do recurso. Ex: `Produto`, `Pedido`, `Cliente`                  |
| `campos`  | string | ✅          | Campos separados por vírgula. Formato: `nome:tipo`. Ex: `nome:str, preco:float` |

### Tipos suportados em `campos`
`str`, `int`, `float`, `bool`, `list`, `dict`

---

## Exemplo — recurso Produto

```bash
curl -X POST http://localhost:8016/api/tools/gerar_scaffold_api/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{
    "arguments": {
      "recurso": "Produto",
      "campos": "nome:str, preco:float, ativo:bool, categoria:str"
    }
  }'
```

### Código gerado
```python
# Scaffold gerado para: Produto
from fastapi import APIRouter, HTTPException
from pydantic import BaseModel
from typing import List

router = APIRouter(prefix="/produtos", tags=["Produto"])


class Produto(BaseModel):
    nome: str
    preco: float
    ativo: bool
    categoria: str


class ProdutoCreate(Produto):
    pass


# Banco em memória (substitua pelo seu repositório real)
_db: dict[int, Produto] = {}
_seq = 0


@router.get("/", response_model=List[Produto])
async def listar():
    return list(_db.values())


@router.get("/{id}", response_model=Produto)
async def obter(id: int):
    if id not in _db:
        raise HTTPException(404, "Produto nao encontrado")
    return _db[id]


@router.post("/", response_model=Produto, status_code=201)
async def criar(payload: ProdutoCreate):
    global _seq
    _seq += 1
    _db[_seq] = payload
    return payload


@router.delete("/{id}", status_code=204)
async def deletar(id: int):
    if id not in _db:
        raise HTTPException(404, "Produto nao encontrado")
    del _db[id]
```

---

## Exemplo — recurso Pedido

```bash
curl -X POST http://localhost:8016/api/tools/gerar_scaffold_api/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{
    "arguments": {
      "recurso": "Pedido",
      "campos": "cliente_id:str, valor_total:float, status:str, itens:list"
    }
  }'
```

---

## Exemplo — recurso simples (só nome)

```bash
-d '{"arguments": {"recurso": "Tag", "campos": "nome:str"}}'
```

---

## Salvar o código gerado em arquivo

```bash
curl -s -X POST http://localhost:8016/api/tools/gerar_scaffold_api/call \
  -H "Authorization: Bearer key_dev_01" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"recurso": "Produto", "campos": "nome:str, preco:float, ativo:bool"}}' \
  | python3 -c "import sys,json; print(json.load(sys.stdin)['result'])" \
  > produto_router.py

# Verificar:
cat produto_router.py
```

---

## Como incluir na sua aplicação FastAPI

```python
# main.py
from fastapi import FastAPI
from produto_router import router as produto_router

app = FastAPI()
app.include_router(produto_router)
```
