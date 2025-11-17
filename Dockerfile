# =============================================================================
# Eryndor MMO - Multi-stage Docker Build
# =============================================================================
# This creates a minimal production image (~100MB) with just the server binary
#
# Usage:
#   docker build -t eryndor-server .
#   docker run -p 5003:5003 -v /var/lib/eryndor:/data eryndor-server
# =============================================================================

# Stage 1: Build the server
# Using nightly because Bevy 0.17.2 requires edition2024
FROM rustlang/rust:nightly-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libasound2-dev \
    libudev-dev \
    libwayland-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy dependency manifests first (for layer caching)
COPY Cargo.toml Cargo.lock ./
COPY crates/eryndor_server/Cargo.toml ./crates/eryndor_server/
COPY crates/eryndor_shared/Cargo.toml ./crates/eryndor_shared/
COPY crates/eryndor_client/Cargo.toml ./crates/eryndor_client/

# Create dummy source files to build dependencies
RUN mkdir -p crates/eryndor_server/src crates/eryndor_shared/src crates/eryndor_client/src && \
    echo "fn main() {}" > crates/eryndor_server/src/main.rs && \
    echo "" > crates/eryndor_shared/src/lib.rs && \
    echo "fn main() {}" > crates/eryndor_client/src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release --bin server

# Remove dummy files
RUN rm -rf crates/eryndor_server/src crates/eryndor_shared/src crates/eryndor_client/src

# Copy actual source code
COPY crates ./crates

# Build the actual server
RUN cargo build --release --bin server

# Stage 2: Create minimal runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libasound2 \
    libudev1 \
    libwayland-client0 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 eryndor

# Create directories
RUN mkdir -p /opt/eryndor /data && \
    chown -R eryndor:eryndor /opt/eryndor /data

# Copy binary from builder
COPY --from=builder /build/target/release/server /opt/eryndor/server

# Copy config files
# Use example config as the base config (override OAuth via env vars)
COPY config.example.toml /opt/eryndor/config.toml
COPY config.example.toml /opt/eryndor/config.example.toml

USER eryndor
WORKDIR /opt/eryndor

# Environment variables (can be overridden at runtime)
ENV SERVER_ADDR=0.0.0.0
ENV SERVER_PORT=5001
ENV SERVER_PORT_WEBSOCKET=5003
ENV DATABASE_PATH=/data/eryndor.db
ENV RUST_LOG=info

# Expose ports
EXPOSE 5001/udp
EXPOSE 5003

# Run the server
CMD ["./server"]
