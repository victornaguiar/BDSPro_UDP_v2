# Multi-stage build for BDSPro Victor

# Build stage
FROM rust:1.70-bullseye as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./
COPY bridge-of-bridges/Cargo.toml ./bridge-of-bridges/
COPY network/Cargo.toml ./network/
COPY test-node/Cargo.toml ./test-node/
COPY external/BDSPro_UDP_Channel/Cargo.toml ./external/BDSPro_UDP_Channel/
COPY bridge-of-bridges/network/Cargo.toml ./bridge-of-bridges/network/
COPY bridge-of-bridges/spdlog/Cargo.toml ./bridge-of-bridges/spdlog/

# Create dummy source files for dependency caching
RUN mkdir -p bridge-of-bridges/src bridge-of-bridges/network/src bridge-of-bridges/spdlog/src \
    network/src test-node/src external/BDSPro_UDP_Channel/src \
    && echo "fn main() {}" > test-node/src/main.rs \
    && echo "fn main() {}" > external/BDSPro_UDP_Channel/src/main.rs \
    && echo "// dummy" > bridge-of-bridges/src/lib.rs \
    && echo "// dummy" > bridge-of-bridges/network/src/lib.rs \
    && echo "// dummy" > bridge-of-bridges/spdlog/src/lib.rs \
    && echo "// dummy" > network/src/lib.rs

# Build dependencies
RUN cargo build --release

# Remove dummy files
RUN rm -rf bridge-of-bridges/src bridge-of-bridges/network/src bridge-of-bridges/spdlog/src \
    network/src test-node/src external/BDSPro_UDP_Channel/src

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false -m -d /app app

# Set working directory
WORKDIR /app

# Copy binaries from builder stage
COPY --from=builder /app/target/release/test-node ./bin/
COPY --from=builder /app/target/release/sender ./bin/
COPY --from=builder /app/target/release/receiver ./bin/
COPY --from=builder /app/target/release/libbridge_of_bridges.a ./lib/

# Copy configuration files
COPY --from=builder /app/test-node/tests/ ./configs/

# Set permissions
RUN chown -R app:app /app && chmod +x ./bin/*

# Switch to app user
USER app

# Expose default ports
EXPOSE 8000 8001 8002 8003 9090

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Default command
CMD ["./bin/test-node", "configs/test.yml", "0"]

# Labels
LABEL maintainer="Victor Naguiar"
LABEL version="1.0"
LABEL description="BDSPro Victor - Distributed Data Processing System"
LABEL org.opencontainers.image.source="https://github.com/victornaguiar/BDSPro_victor"