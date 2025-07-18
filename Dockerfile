# Use Rust slim image
FROM rust:1.86-slim

# Install all dependencies (build + runtime + gcloud CLI)
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    git \
    ca-certificates \
    libpq5 \
    gnupg \
    && curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | gpg --dearmor -o /usr/share/keyrings/cloud.google.gpg \
    && echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] https://packages.cloud.google.com/apt cloud-sdk main" > /etc/apt/sources.list.d/google-cloud-sdk.list \
    && apt-get update && apt-get install -y google-cloud-cli \
    && rm -rf /var/lib/apt/lists/*

# Add wasm32-unknown-unknown target
RUN rustup target add wasm32-unknown-unknown

# Set working directory
WORKDIR /app

# Copy dependency files and create dummy main.rs for dependency caching
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only (this will be cached unless Cargo.toml/Cargo.lock changes)
RUN cargo build --release && rm -rf src target/release/deps/near_indexer*

# Copy real source files
COPY src/ ./src/

# Copy remaining files
COPY schema.sql config.toml ./
COPY scripts/ ./scripts/

# Copy .env file if it exists (wildcard pattern won't fail if missing)
COPY .env* ./

# Build the actual application (fast since dependencies are cached)
RUN cargo build --release

# Make scripts executable and create entrypoint
RUN chmod +x ./scripts/*.sh && \
    printf '#!/bin/bash\nset -e\n\necho "Starting NEAR Indexer container..."\n\necho "Fetching configuration..."\nsource ./scripts/fetch_secrets.sh\n\necho "Initializing database..."\n./target/release/near-indexer init || echo "Database already initialized or init failed - continuing"\n\necho "Starting indexer..."\nexec ./target/release/near-indexer start --start-block 199560000\n' > entrypoint.sh && \
    chmod +x entrypoint.sh

# Set environment and default command
ENV RUST_LOG=info
ENTRYPOINT ["./entrypoint.sh"] 