# syntax=docker/dockerfile:1

# Stage 1: Builder
FROM debian:bookworm-slim AS builder

# Install system build dependencies and tools
RUN apt-get update && apt-get install -y --no-install-recommends \
		curl \
		ca-certificates \
		build-essential \
		pkg-config \
		libssl-dev \
		libpq-dev \
		git \
	&& rm -rf /var/lib/apt/lists/*

# Install rustup and a nightly toolchain that supports edition=2024
ENV RUSTUP_HOME=/usr/local/rustup \
		CARGO_HOME=/usr/local/cargo \
		PATH=/usr/local/cargo/bin:$PATH

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly && \
		rustup default nightly

WORKDIR /app

# Copy manifest first to leverage Docker cache for dependencies
COPY Cargo.toml Cargo.lock ./

# Copy backend crate and workspace files (binary at backend/)
COPY backend/ ./backend/

# Fetch dependencies to populate cargo cache
RUN cargo fetch --locked --manifest-path /app/Cargo.toml || true

# Build release binary
RUN cargo build --release --manifest-path /app/Cargo.toml
 
# Install diesel_cli in the builder so we can copy it to the runtime image
RUN /usr/local/cargo/bin/cargo install diesel_cli --no-default-features --features postgres || true

FROM debian:bookworm-slim AS runtime

# Runtime libs required by the binary (libpq for diesel/postgres, ca-certificates, psql client)
RUN apt-get update && apt-get install -y --no-install-recommends \
	libpq5 \
	postgresql-client \
	netcat-openbsd \
	ca-certificates \
	&& rm -rf /var/lib/apt/lists/*

# Create a non-privileged user
RUN useradd -m -u 10001 -s /bin/bash appuser
USER appuser

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/starknake ./starknake

# Copy wait script and migrations
COPY --chown=appuser:appuser scripts/wait-for-db.sh /app/wait-for-db.sh
RUN chmod +x /app/wait-for-db.sh
COPY --chown=appuser:appuser migrations /app/migrations

# Copy diesel CLI from builder into runtime so migrations can be run via diesel
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel

# We will run SQL migration files directly using psql in the runtime image

EXPOSE 8080

# Entrypoint: wait for DB, run migrations, then exec the web binary
# Pass empty HOST and PORT so the script will parse them from the DATABASE_URL env on platforms like Render
ENTRYPOINT ["/app/wait-for-db.sh", "", "", "", "./starknake"]