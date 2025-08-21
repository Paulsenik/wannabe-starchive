# Build stage
FROM rust:1.89.0-slim as builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
# Copy Rocket configuration file
COPY Rocket.toml ./

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/backend /app/backend
# Copy Rocket configuration to runtime stage
COPY --from=builder /app/Rocket.toml /app/Rocket.toml

# Expose the port your Rocket app runs on (typically 8000)
EXPOSE 8000

# Run the binary
CMD ["./backend"]