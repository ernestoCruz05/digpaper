# DigPaper Backend - Multi-stage Dockerfile
# Produces a small (~50MB) final image

# ============================================
# Stage 1: Build the Rust application
# ============================================
FROM rust:1.83-slim-bookworm AS builder

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
# Stage 2: Create minimal runtime image
# ============================================
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies (SSL for HTTPS, ca-certs for TLS)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd -m -u 1000 digpaper
USER digpaper

# Copy the compiled binary from builder
COPY --from=builder /app/target/release/digpaper /app/digpaper

# Create directories for uploads and database
RUN mkdir -p /app/uploads /app/data

# Expose port (Axum default)
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:/app/data/digpaper.db

# Run the application
CMD ["/app/digpaper"]
