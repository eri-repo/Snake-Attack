Deploying Starknake to Render

This file documents the minimal steps and recommended settings to deploy the backend web service and Postgres on Render.

1) Create a Postgres database on Render
- Go to Render dashboard -> New -> Database -> PostgreSQL
- Choose a name (e.g. `starknake-db`) and a region.
- After provisioning, open the database page and copy the "Connection URL".

2) Create the Web Service
- Go to Render dashboard -> New -> Web Service
- Connect the Git repo that contains this project.
- Environment: Docker
- Build Command: leave empty (Render will build via Dockerfile)
- Start Command: leave empty OR set explicitly to:
  /app/wait-for-db.sh "" "" "$DATABASE_URL" ./starknake
  The explicit Start Command forces the startup script to use the `DATABASE_URL` env variable.

3) Environment variables / Secrets
- On the Web Service page, add the following environment variable (as a secret):
  - DATABASE_URL = <the postgres connection URL copied from the DB dashboard>

- Optionally set:
  - PGSSLMODE = require  # ensures libpq uses SSL

4) How the container starts
- The image's ENTRYPOINT runs `/app/wait-for-db.sh` which:
  - Parses `DATABASE_URL` to discover DB host and port
  - Waits for the DB host:port to become reachable
  - Runs `diesel migration run` (diesel CLI is included in the image)
  - Starts the web server binary (`./starknake`)

5) Verify deployment
- Watch the Render deploy logs. You should see lines like:
  Waiting for <db-host>:<port>...
  Found diesel CLI, running diesel migration run
  Server running on http://0.0.0.0:8080

- Once deployed, call the API's public endpoint (Render will provide a URL) to exercise `/create_user`.

6) Security and production tips
- Do not commit your `.env` with credentials. Use Render's secrets instead.
- Consider running migrations separately as a one-off job if you prefer not to run migrations in the web process.
- Add a health endpoint and configure readiness/liveness checks in Render.

If you want, I can also add a small `render.yml` (Render Blueprint) or prepare a `migrate` job in `compose.yaml` for local testing.
