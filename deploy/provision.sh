#!/usr/bin/env bash
set -euo pipefail
cd /opt/kyb
site="$1"

if [ ! -f .env ]; then
  printf 'POSTGRES_PASSWORD=%s\nOPENAI_API_KEY=\nOPENAI_MODEL=gpt-5.4-mini\nSITE_ADDRESS=%s\n' "$(openssl rand -hex 16)" "$site" > .env
  chmod 600 .env
fi

if [ ! -f deploy/auth.caddy ]; then
  password="$(openssl rand -base64 15 | tr -d '/+=')"
  hash="$(docker run --rm caddy:2-alpine caddy hash-password --plaintext "$password")"
  printf 'basic_auth {\n\tdemo %s\n}\n' "$hash" > deploy/auth.caddy
  printf 'user: demo\npassword: %s\n' "$password" > .demo-credentials
  chmod 600 deploy/auth.caddy .demo-credentials
fi
