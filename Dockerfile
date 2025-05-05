FROM rust:1.86-slim

# Install required dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    git \
    pkg-config \
    libssl-dev \
    unzip \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Add wasm32-unknown-unknown target
RUN rustup target add wasm32-unknown-unknown

# Install buf for protobuf
RUN curl -sSL \
    "https://github.com/bufbuild/buf/releases/download/v1.28.1/buf-$(uname -s)-$(uname -m)" \
    -o /usr/local/bin/buf && \
    chmod +x /usr/local/bin/buf

# Install substreams 
RUN wget "https://github.com/streamingfast/substreams/releases/download/v1.15.3/substreams_linux_x86_64.tar.gz" && \
    tar -xzf "substreams_linux_x86_64.tar.gz" && \
    mv substreams /usr/local/bin/ && \
    rm "substreams_linux_x86_64.tar.gz"

# Install substreams-sink-sql
RUN wget "https://github.com/streamingfast/substreams-sink-sql/releases/download/v4.5.0/substreams-sink-sql_linux_x86_64.tar.gz" && \
    tar -xzf "substreams-sink-sql_linux_x86_64.tar.gz" && \
    mv substreams-sink-sql /usr/local/bin/ && \
    rm "substreams-sink-sql_linux_x86_64.tar.gz"

# Copy just the .env file first
COPY .env /app/.env

# Create entrypoint script
RUN echo '#!/bin/bash\n\
# Load environment variables from .env file\n\
export $(grep -v "^#" /app/.env | xargs)\n\
\n\
# Run the command\n\
exec "$@"' > /app/entrypoint.sh && \
    chmod +x /app/entrypoint.sh

# Copy application files
COPY . .

# Generate protobuf files and build the project
RUN substreams protogen ./substreams.yaml --exclude-paths="sf/substreams,google/" && \
    cargo build --target wasm32-unknown-unknown --release

# Set the entrypoint
ENTRYPOINT ["/app/entrypoint.sh"]

# Setup and run the sink
CMD ["make", "setup_sink", "run_sink"]


# Required environment variables (now loaded from .env file at runtime)
# DSN - PostgreSQL connection string 
# ENDPOINT - PINAX endpoint
# SUBSTREAMS_API_KEY - PINAX API KEY 