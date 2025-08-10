#!/usr/bin/env bash
set -euo pipefail

DB_URL="${DATABASE_URL:-postgres://postgres:postgres@localhost:5432/kkss}"

if ! command -v sqlx >/dev/null 2>&1; then
	cargo install sqlx-cli --no-default-features --features rustls,postgres
fi

echo "[INIT] Using DATABASE_URL=$DB_URL"
DATABASE_URL="$DB_URL" sqlx database create || true
DATABASE_URL="$DB_URL" sqlx migrate run --source migrations