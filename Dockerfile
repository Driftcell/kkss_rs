#### Multi-stage build for kkss-backend (Actix-web + SQLite)
## Build stage
FROM rust:1.88 AS builder

ARG APP_NAME=kkss-backend
WORKDIR /build

## Install system build dependencies (OpenSSL headers, pkg-config, build tools)
RUN apt-get update && apt-get install -y --no-install-recommends \
	pkg-config libssl-dev ca-certificates build-essential && \
	rm -rf /var/lib/apt/lists/*

# 1. Pre-copy manifests for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create a stub src to prime dependency cache
RUN mkdir -p src && echo "fn main(){}" > src/main.rs && \
	cargo fetch && cargo build --release || true

# 2. Copy real source & migrations
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx

# 3. Enable offline mode & perform final release build using generated sqlx-data.json
ENV SQLX_OFFLINE=1
RUN cargo build --release --locked --bin ${APP_NAME}

## Runtime stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install required runtime libs (OpenSSL for reqwest default-tls, CA certs for HTTPS)
RUN apt-get update && apt-get install -y --no-install-recommends \
	ca-certificates libssl3 && \
	rm -rf /var/lib/apt/lists/*

# Copy binary & sqlx-data.json (kept for reference / future diagnostics)
COPY --from=builder /build/target/release/kkss-backend ./kkss-backend

# Copy entrypoint
COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

# Default environment variables (can be overridden at run time)
ENV DATABASE_URL=sqlite://./kkss.db \
	RUST_LOG=debug \
	CONFIG_PATH=/app/config.toml \
	RUST_BACKTRACE=1

# Create non-root user
RUN useradd -u 10001 -ms /bin/bash appuser && chown -R appuser:appuser /app
USER appuser

# Expose typical Actix port (adjust if your config.toml uses another)
EXPOSE 8080

# Working directory will have config.toml & kkss.db mounted at runtime:
#   docker run -v $(pwd)/config.toml:/app/config.toml -v $(pwd)/kkss.db:/app/kkss.db IMAGE

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["./kkss-backend"]
