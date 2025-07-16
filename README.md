# NEAR Blockchain Indexer

A Rust-based blockchain indexer for NEAR protocol, focusing on veNEAR contract interactions. Extracts and stores receipt actions and execution outcomes in PostgreSQL with configurable schema support.

## Features

- **Configurable Schema**: Set PostgreSQL schema via environment variables
- **veNEAR Contract Focus**: Filters and indexes veNEAR contract interactions
- **Comprehensive Data Extraction**: Captures receipt actions, execution outcomes, and block data
- **JSON Processing**: Converts base64 args to JSON with validation
- **Docker Support**: Full containerized deployment
- **Resumable Indexing**: Cursor-based progress tracking

## Quick Start

### Prerequisites
- Rust
- Docker & Docker Compose
- PostgreSQL

### Run with Docker
```bash
# Start the full stack
docker-compose up -d

# View logs
docker-compose logs -f indexer-run
```

### Run Locally
```bash
# Build
cargo build --release

# Initialize database tables
cargo run -- init

# Start indexer
cargo run -- start
```

## Configuration

### Environment Variables

Set these environment variables or use a `.env` file (all prefixed with `INDEXER_`):

```bash
# Database Configuration
INDEXER_DB_HOST=localhost
INDEXER_DB_PORT=5432
INDEXER_DB_DATABASE=near_indexer
INDEXER_DB_USERNAME=postgres
INDEXER_DB_PASSWORD=password
INDEXER_DB_MAX_CONNECTIONS=10
INDEXER_DB_SCHEMA=fastnear

# NEAR API Configuration
INDEXER_API_URL=https://api.fastnear.com
INDEXER_API_AUTH_TOKEN=your_api_token_here
INDEXER_API_CHAIN_ID=testnet
INDEXER_API_FINALITY=final

# Indexer Settings
INDEXER_START_BLOCK=183500000
INDEXER_BATCH_SIZE=10
INDEXER_POLL_INTERVAL=1
INDEXER_MAX_RETRIES=3
INDEXER_RETRY_DELAY=5
INDEXER_NUM_THREADS=64
INDEXER_LOG_LEVEL=info
```

### Configuration File

Alternatively, modify `config.toml`:

```toml
# Database Configuration
db_host = "localhost"
db_port = 5432
db_database = "near_indexer"
db_username = "postgres"
db_password = "password"
db_max_connections = 10
db_schema = "fastnear"

# Indexer Configuration
start_block = 183500000
batch_size = 10
poll_interval = 1
max_retries = 3
retry_delay = 5
num_threads = 64

# veNEAR Contracts
venear_contracts = [
    "r-1745564650.testnet",
    "r-1746683627.testnet", 
    "r-1748895584.testnet"
]
log_level = "info"
```

## Database Schema

The indexer creates the following tables in the configured schema:

- **`blocks`**: Block headers and metadata
- **`receipt_actions`**: Function call actions with decoded arguments
- **`execution_outcomes`**: Transaction execution results
- **`cursors`**: Indexing progress tracking

## Commands

- `cargo run -- init` - Initialize database tables from schema.sql
- `cargo run -- start [--start-block <block_num>] [--num-threads <threads>]` - Start the indexer from configured start block (or optional start block) with configured threads (or optional thread count)

## Development

### Testing
```bash
# Run tests
cargo test

# Check with clippy
cargo clippy

# Format code
cargo fmt
```

### Docker Build
```bash
# Build image
docker build -t near-indexer .

# Run container
docker run -e INDEXER_DB_HOST=host.docker.internal near-indexer
```