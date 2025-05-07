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
    gnupg \
    && rm -rf /var/lib/apt/lists/*

# Install Google Cloud SDK
RUN echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] https://packages.cloud.google.com/apt cloud-sdk main" | tee -a /etc/apt/sources.list.d/google-cloud-sdk.list && \
    curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | apt-key --keyring /usr/share/keyrings/cloud.google.gpg add - && \
    apt-get update && apt-get install -y google-cloud-sdk && \
    rm -rf /var/lib/apt/lists/*

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

# Copy application files
COPY . .

# Generate protobuf files and build the project
RUN substreams protogen ./substreams.yaml --exclude-paths="sf/substreams,google/" && \
    cargo build --target wasm32-unknown-unknown --release

# Make the copy env script executable
RUN chmod +x /app/scripts/copy_env.sh

# Set the entrypoint to our startup script
ENTRYPOINT ["/app/scripts/copy_env.sh"]

# Set the default command to run the sink
CMD ["make", "setup_sink", "run_sink"]

# Required environment variables (provided at runtime)
# DSN - PostgreSQL connection string 
# ENDPOINT - PINAX endpoint
# SUBSTREAMS_API_KEY - PINAX API KEY 