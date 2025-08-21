# Build stage
FROM rust:1.89.0-slim as builder

# Install specific versions of trunk and wasm-pack that are compatible
RUN cargo install --version 0.21.14 trunk
RUN cargo install wasm-pack

# Add wasm32 target for WebAssembly compilation
RUN rustup target add wasm32-unknown-unknown

# Set working directory
WORKDIR /app

# Copy frontend source
COPY . .

# Build the frontend
RUN trunk build --release

# Runtime stage with nginx
FROM nginx:alpine

# Copy custom nginx config
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Copy built frontend from builder stage
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy environment variable script with the correct name
COPY env_config_generator.sh /docker-entrypoint.d/30-env-config.sh

# Make script executable
RUN chmod +x /docker-entrypoint.d/30-env-config.sh

# Expose port
EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]