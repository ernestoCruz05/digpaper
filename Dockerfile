# Charta - Multi-stage Dockerfile
# Builds the React frontend and Rust backend, producing a single container

# ============================================
# Stage 1: Build the React Frontend
# ============================================
FROM node:20-bookworm-slim AS frontend-builder

WORKDIR /app/web-react

# Install dependencies first for layer caching
COPY web-react/package*.json ./
RUN npm install

# Build the app (outputs to /app/web via vite.config.js)
COPY web-react ./
RUN npm run build

# ============================================
# Stage 2: Build the Rust Backend
# ============================================
FROM rust:1.85-slim-bookworm AS backend-builder

WORKDIR /app

# Install build dependencies for SQLx
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies first
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy actual source code
COPY src ./src

# Touch main.rs to ensure it gets rebuilt
RUN touch src/main.rs

# Build the real application
RUN cargo build --release

# ============================================
# Stage 3: Create minimal runtime image
# ============================================
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd -m -u 1000 charta

# Create directories
RUN mkdir -p /app/uploads /app/data /app/web && chown -R charta:charta /app

USER charta

# Copy the compiled binary from Rust builder
COPY --from=backend-builder --chown=charta:charta /app/target/release/charta /app/charta

# Copy the built React web app from Node builder
COPY --from=frontend-builder --chown=charta:charta /app/web /app/web

# Expose port
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:/app/data/charta.db

# Run the application
CMD ["/app/charta"]
