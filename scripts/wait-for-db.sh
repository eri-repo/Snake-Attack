#!/usr/bin/env bash
set -euo pipefail

# Usage: wait-for-db.sh HOST PORT DATABASE_URL CMD...
# Example: wait-for-db.sh db 5432 "postgres://user:pass@db:5432/dbname" ./starknake

HOST="${1:-db}"
PORT="${2:-5432}"
ARG_DATABASE_URL="${3:-}"
shift 3 || true

# If the user accidentally passed the DATABASE_URL as the first positional
# argument (common when calling the script with a single arg), detect that
# and move it into ARG_DATABASE_URL so we parse it correctly.
if [[ "$HOST" == postgresql:* || "$HOST" == postgresql://* || "$HOST" == postgres:* || "$HOST" == postgres://* ]]; then
  ARG_DATABASE_URL="$HOST"
  HOST="db"
  PORT="5432"
fi

# Prefer passed DATABASE_URL; if empty, fall back to env DATABASE_URL
if [ -n "${ARG_DATABASE_URL:-}" ]; then
  DATABASE_URL="$ARG_DATABASE_URL"
else
  DATABASE_URL="${DATABASE_URL:-}"
fi

# If we have a DATABASE_URL, parse host and port robustly. Strip any query
# string like `?sslmode=require` and handle URLs with or without an explicit
# port. Fall back to default port 5432 when none is present.
if [ -n "${DATABASE_URL:-}" ]; then
  # remove scheme (postgres:// or postgresql://)
  tmp="${DATABASE_URL#*://}"
  # remove userinfo if present (user:pass@)
  if [[ "$tmp" == *"@"* ]]; then
    tmp="${tmp#*@}"
  fi
  # extract host[:port] (before first /)
  hostport="${tmp%%/*}"

  # host and port split
  if [[ "$hostport" == *":"* ]]; then
    HOST_EXTRACT="${hostport%%:*}"
    PORT_EXTRACT="${hostport##*:}"
  else
    HOST_EXTRACT="$hostport"
    PORT_EXTRACT=""
  fi

  # strip query from port if present (e.g. 5432?sslmode=require)
  PORT_EXTRACT="${PORT_EXTRACT%%\?*}"

  if [ -n "$HOST_EXTRACT" ]; then
    HOST="$HOST_EXTRACT"
  fi
  if [ -n "$PORT_EXTRACT" ]; then
    PORT="$PORT_EXTRACT"
  fi
fi

# ensure PORT is set to a numeric default if empty
if [ -z "${PORT:-}" ]; then
  PORT=5432
fi

# Wait for TCP
# Ensure PORT is numeric; if not, try parsing it again from DATABASE_URL or fall
# back to default 5432. This prevents `nc` from being called with the whole
# DATABASE_URL string as the port (Render can sometimes pass things oddly).
if ! [[ "$PORT" =~ ^[0-9]+$ ]]; then
  # If PORT mistakenly contains the URL, try to re-parse from DATABASE_URL
  if [[ "${DATABASE_URL:-}" == *"://"* ]]; then
    tmp="${DATABASE_URL#*://}"
    if [[ "$tmp" == *"@"* ]]; then
      tmp="${tmp#*@}"
    fi
    hostport="${tmp%%/*}"
    if [[ "$hostport" == *":"* ]]; then
      PORT="${hostport##*:}"
    else
      PORT=""
    fi
    # strip query string if any
    PORT="${PORT%%\?*}"
  fi
fi

if ! [[ "$PORT" =~ ^[0-9]+$ ]]; then
  echo "Warning: parsed port was not numeric; defaulting to 5432" >&2
  PORT=5432
fi

echo "Waiting for $HOST:$PORT... (DATABASE_URL=${DATABASE_URL:-<none>})"
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
