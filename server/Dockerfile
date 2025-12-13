# Multi-stage build for w9-tools backend
# Stage 1: Build
FROM rust:1.75-slim as builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock* ./
COPY server/ ./server/

# Build the binary
WORKDIR /build/server
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies (SQLite, wget for healthcheck)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /build/server/target/release/w9 /app/w9

# Create data directory
RUN mkdir -p /app/data /app/uploads && \
    chmod 755 /app/data /app/uploads

# Run as non-root user
RUN useradd -r -s /bin/false w9 && \
    chown -R w9:w9 /app

USER w9

EXPOSE 8080

ENTRYPOINT ["/app/w9"]

