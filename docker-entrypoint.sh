#!/usr/bin/env bash
set -euo pipefail

cd /app

# Ensure config is present (user should mount it); warn if missing.
if [ ! -f "config.toml" ]; then
  echo "[WARN] /app/config.toml 不存在，请通过 -v 挂载 config.toml (CONFIG_PATH=$CONFIG_PATH)" >&2
fi

echo "[INFO] 使用数据库: $DATABASE_URL"

exec "$@"
