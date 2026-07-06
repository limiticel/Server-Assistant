#!/usr/bin/env bash
set -euo pipefail

BACKUP_FILE="${1:?informe o arquivo .sql}"
psql "${DATABASE_URL:?DATABASE_URL obrigatoria}" < "$BACKUP_FILE"

