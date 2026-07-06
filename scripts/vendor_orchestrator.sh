#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="$ROOT/backend/orchestrator-rust"

if [ -d "$TARGET/.git" ]; then
  git -C "$TARGET" pull --ff-only
else
  git clone https://github.com/limiticel/orchestrator-rust "$TARGET"
fi

echo "Orchestrator instalado em $TARGET"

