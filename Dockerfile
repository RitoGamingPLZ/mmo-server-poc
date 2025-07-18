# Build stage
FROM rust:1.88-slim AS builder

WORKDIR /app

# Install minimal system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create empty project for dependency caching
RUN cargo init --name mmo_game_server

# Copy manifest files first for better caching
COPY Cargo.toml Cargo.lock ./

# Build dependencies (cached layer)
RUN cargo build --release && rm src/*.rs target/release/deps/mmo_game_server*

# Copy source code
COPY src ./src

# Build the application with optimizations
ENV CARGO_TARGET_DIR=/app/target
ENV RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C codegen-units=1 -C panic=abort"
RUN cargo build --release

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