#!/usr/bin/env bash
set -euo pipefail

# Usage: wait-for-db.sh HOST PORT DATABASE_URL CMD...
# Example: wait-for-db.sh db 5432 "postgres://user:pass@db:5432/dbname" ./starknake

HOST="${1:-db}"
PORT="${2:-5432}"
ARG_DATABASE_URL="${3:-}"
shift 3 || true

# Prefer passed DATABASE_URL; if empty, fall back to env DATABASE_URL
if [ -n "$ARG_DATABASE_URL" ]; then
  DATABASE_URL="$ARG_DATABASE_URL"
else
  DATABASE_URL="${DATABASE_URL:-}"
fi

# If HOST or PORT are empty, try to extract from DATABASE_URL
if [ -z "$HOST" ] || [ -z "$PORT" ]; then
  if [ -n "$DATABASE_URL" ]; then
    # extract host and port from postgres://user:pass@host:port/db
    # host
    HOST_EXTRACT=$(echo "$DATABASE_URL" | sed -E 's#.*@([^:/]+).*#\1#')
    # port (optional)
    PORT_EXTRACT=$(echo "$DATABASE_URL" | sed -E 's#.*@[^:/]+:([0-9]+).*#\1#')
    if [ -n "$HOST_EXTRACT" ]; then
      HOST="$HOST_EXTRACT"
    fi
    if [ -n "$PORT_EXTRACT" ]; then
      PORT="$PORT_EXTRACT"
    fi
  fi
fi

# Wait for TCP
echo "Waiting for $HOST:$PORT..."
while ! nc -z "$HOST" "$PORT"; do
  sleep 0.5
done

# Optionally run migrations if DATABASE_URL provided
if [ -n "${DATABASE_URL}" ]; then
  echo "Attempting to run migrations against $DATABASE_URL"
  if command -v diesel >/dev/null 2>&1; then
    echo "Found diesel CLI, running diesel migration run"
    # diesel reads DATABASE_URL from env when not provided explicitly
    if [ -n "$DATABASE_URL" ]; then
      export DATABASE_URL="$DATABASE_URL"
    fi
    diesel migration run || true
  else
    echo "diesel CLI not found; applying SQL migration files with psql"
    # iterate over migrations directories and apply up.sql in lexicographic order
    for dir in /app/migrations/*; do
      if [ -d "$dir" ] && [ -f "$dir/up.sql" ]; then
        echo "Applying $dir/up.sql"
        PGPASSWORD="${POSTGRES_PASSWORD:-}" psql "$DATABASE_URL" -f "$dir/up.sql" || true
      fi
    done
  fi
fi

# Exec the provided command (start the web binary)
if [ "$#" -eq 0 ]; then
  echo "No command provided to exec; exiting"
  exit 1
fi

exec "$@"
