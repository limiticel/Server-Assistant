#!/usr/bin/env bash
set -euo pipefail

BACKUP_DIR="${BACKUP_DIR:-./backups}"
mkdir -p "$BACKUP_DIR"
pg_dump "${DATABASE_URL:?DATABASE_URL obrigatoria}" > "$BACKUP_DIR/server_assistant_$(date +%Y%m%d_%H%M%S).sql"

