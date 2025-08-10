#!/usr/bin/env bash
set -euo pipefail

cd /app

# Ensure config is present (user should mount it); warn if missing.
if [ ! -f "config.toml" ]; then
  echo "[WARN] /app/config.toml 不存在，请通过 -v 挂载 config.toml (CONFIG_PATH=$CONFIG_PATH)" >&2
fi

# DATABASE_URL expected like sqlite://./kkss.db
DB_PATH="${DATABASE_URL#sqlite://}"  # remove scheme
# normalize leading ./
DB_PATH="${DB_PATH#./}"

if [[ "$DATABASE_URL" == sqlite:* ]]; then
  if [ ! -f "$DB_PATH" ]; then
    echo "[INFO] 未找到数据库文件 $DB_PATH，创建..."
    # touch to create; migrations will run inside the app on start
    mkdir -p "$(dirname "$DB_PATH")"
    touch "$DB_PATH"
  fi
else
  echo "[INFO] 非 sqlite 数据库 URL: $DATABASE_URL" >&2
fi

exec "$@"
