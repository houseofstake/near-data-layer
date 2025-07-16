# Use the official Rust image as a builder
FROM rust:latest as builder

# Set working directory
WORKDIR /usr/src/app

# Copy source code and build files
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY schema.sql config.toml ./

# Build the application
RUN cargo build --release

# Create a new stage with a minimal runtime image
FROM debian:bookworm-slim

# Install runtime dependencies and create non-root user
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /bin/false app

# Set working directory and copy files
WORKDIR /app
COPY --from=builder /usr/src/app/target/release/near-indexer ./
COPY --from=builder /usr/src/app/schema.sql ./
COPY --from=builder /usr/src/app/config.toml ./

# Change ownership and switch to app user
RUN chown -R app:app /app
USER app

# Set environment and default command
ENV RUST_LOG=info
CMD ["./near-indexer", "start"] 