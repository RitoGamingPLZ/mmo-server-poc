# Build stage
FROM rust:1.88-slim AS builder

# Set target architecture variables
ARG TARGETPLATFORM

WORKDIR /app

# Install minimal system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create empty project for dependency caching
RUN cargo init --name mmo_game_server

# Copy the full project (Cargo.toml, Cargo.lock, src/, etc.)
COPY . .

# Build the application with optimizations
ENV CARGO_TARGET_DIR=/app/target

# Set architecture-specific optimizations
RUN if [ "$TARGETPLATFORM" = "linux/amd64" ]; then \
        export RUSTFLAGS="-C target-cpu=x86-64-v3 -C opt-level=3 -C codegen-units=1 -C panic=abort"; \
    elif [ "$TARGETPLATFORM" = "linux/arm64" ]; then \
        export RUSTFLAGS="-C target-cpu=cortex-a72 -C opt-level=3 -C codegen-units=1 -C panic=abort"; \
    else \
        export RUSTFLAGS="-C opt-level=3 -C codegen-units=1 -C panic=abort"; \
    fi && \
    cargo build --release

# Runtime stage - Use distroless for security and size
FROM gcr.io/distroless/cc-debian12:latest

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/mmo_game_server /app/mmo_game_server

# Create non-root user for security
USER 1000:1000

# Expose the WebSocket port
EXPOSE 5000

# Run the server
ENTRYPOINT ["./mmo_game_server"]