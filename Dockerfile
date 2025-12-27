# DigPaper - Multi-stage Dockerfile
# Builds both the Rust backend and Flutter web app
# Produces a single container that serves everything

# ============================================
# Stage 1: Build the Flutter Web App
# ============================================
FROM ghcr.io/cirruslabs/flutter:stable AS flutter-builder

WORKDIR /app/mobile

# Copy Flutter project
COPY mobile/pubspec.yaml mobile/pubspec.lock ./
RUN flutter pub get

COPY mobile/ ./

# Generate app icons and build web
RUN dart run flutter_launcher_icons || true
RUN flutter build web --release

# ============================================
# Stage 2: Build the Rust Backend
# ============================================
FROM rust:1.85-slim-bookworm AS rust-builder

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
RUN useradd -m -u 1000 digpaper

# Create directories
RUN mkdir -p /app/uploads /app/data /app/web && chown -R digpaper:digpaper /app

USER digpaper

# Copy the compiled binary from Rust builder
COPY --from=rust-builder --chown=digpaper:digpaper /app/target/release/digpaper /app/digpaper

# Copy the Flutter web build
COPY --from=flutter-builder --chown=digpaper:digpaper /app/mobile/build/web /app/web

# Expose port
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:/app/data/digpaper.db

# Run the application
CMD ["/app/digpaper"]
