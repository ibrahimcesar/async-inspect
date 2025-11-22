# Multi-stage build for async-inspect CLI
FROM rust:1.70-slim as builder

WORKDIR /usr/src/async-inspect

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY async-inspect-macros ./async-inspect-macros/

# Copy source code
COPY src ./src
COPY examples ./examples

# Build release binary with CLI features
RUN cargo build --release --bin async-inspect --features cli

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /usr/src/async-inspect/target/release/async-inspect /usr/local/bin/async-inspect

# Create non-root user
RUN useradd -m -u 1000 inspector && \
    mkdir -p /home/inspector/.async-inspect && \
    chown -R inspector:inspector /home/inspector

USER inspector
WORKDIR /home/inspector

# Expose default ports (if web dashboard is added)
# EXPOSE 8080

ENTRYPOINT ["async-inspect"]
CMD ["--help"]
