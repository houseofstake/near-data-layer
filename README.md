# NEAR Blockchain SQL Sink

A substreams-based indexer for the NEAR blockchain. Data is sunk into a PostgreSQL database.

This project processes NEAR blockchain data in real-time using [Substreams](https://substreams.streamingfast.io/) technology and stores structured data in PostgreSQL for efficient querying and analysis.

## Architecture

The system consists of several key components:

1. **Substreams Module** (`src/lib.rs`): The core WASM module that processes NEAR blocks
2. **Data Processors** (`src/processors/`): Individual processors for different blockchain entities
3. **Database Schema** (`schema.sql`): PostgreSQL tables for storing indexed data
4. **Sink Configuration** (`sink/`): Configuration for the SQL sink
5. **Docker Environment**: Containerized setup with PostgreSQL and pgweb

### Data Processing Pipeline

The indexer processes the following NEAR blockchain entities:

- **Blocks**: Block metadata including height, hash, author, timestamp, and gas information
- **Receipts**: Transaction receipts with predecessor/receiver information
- **Receipt Actions**: Detailed action data including method calls, gas usage, and arguments
- **Execution Outcomes**: Results of executed receipts including gas consumption and status

## The Data Model
![](https://github.com/user-attachments/assets/244ae41f-f40f-45ef-8f5c-c385fe01860c)

## Database Schema

The system creates the following main tables:

- `blocks`: Core block information (height, hash, author, timestamp, gas data)
- `receipts`: Receipt metadata (IDs, predecessor/receiver accounts, block references)
- `receipt_actions`: Detailed action data (method calls, gas, deposits, arguments)
- `execution_outcomes`: Execution results (gas consumption, status, logs)
- `cursors`: Substreams cursor tracking for resuming from interruptions

## Prerequisites

- Docker and Docker Compose
- For local development: Rust, `substreams` CLI, `substreams-sink-sql` CLI

## GCP Credentials Setup

To use Google Cloud services within the Docker container, you need to set up application default credentials:

1. Install the Google Cloud SDK if you haven't already:
   ```bash
   # For macOS
   brew install google-cloud-sdk
   ```

2. Authenticate with Google Cloud:
   ```bash
   gcloud auth login
   ```

3. Generate application default credentials:
   ```bash
   gcloud auth application-default login
   ```
   This will create a credentials file at `~/.config/gcloud/application_default_credentials.json` which will be automatically mounted into the container.

## Setup

1. Create a `.env` file with your configuration:
   ```
   DSN=postgres://dev-node:insecure-change-me-in-prod@postgres:5432/dev-node?sslmode=disable
   ENDPOINT=near.substreams.pinax.network:443
   SUBSTREAMS_API_KEY=your_api_key_here
   ```

   The application will:
   - First check for a local `.env` file in the project root
   - If no `.env` file is found, it will attempt to fetch secrets from GCP Secret Manager
   - Make sure you have GCP credentials set up (see GCP Credentials Setup section) if you plan to use GCP Secret Manager

2. Start the services:
   ```bash
   docker-compose up --build
   ```

   This will:
   - Start a PostgreSQL database
   - Start pgweb (PostgreSQL web interface) at http://localhost:8081
   - Build and run the NEAR sink container with environment variables from either the `.env` file or GCP Secret Manager

## Configuration

### Environment Variables

The following environment variables are required in your `.env` file:

- `DSN`: PostgreSQL connection string
  - Format: `postgres://username:password@host:port/database?sslmode=disable`
  - Default in Docker: `postgres://dev-node:insecure-change-me-in-prod@postgres:5432/dev-node?sslmode=disable`

- `ENDPOINT`: NEAR Substreams endpoint
  - Default: `near.substreams.pinax.network:443`

- `SUBSTREAMS_API_KEY`: Your PINAX API key
  - Required for accessing the NEAR Substreams endpoint

### Substreams Configuration

The `substreams.yaml` file configures:
- **Initial Block**: Currently set to block `198570538` (Receipt Actions Example Block)
- **Network**: `testnet` (can be changed to `mainnet`)
- **Output Module**: `db_out` - processes blocks and outputs database changes

## Development

### Building the Project

```bash
# Build the WASM module
make build

# Generate protobuf files
make protogen

# Clean build artifacts
make clean
```

### Testing the Substreams

```bash
# Stream the last 10 blocks (requires ENDPOINT env var)
make stream_db_out
```

### Database Operations

```bash
# Set up database schema (requires DSN env var)
make setup_sink

# Run the SQL sink (requires DSN and ENDPOINT env vars)
make run_sink
```

### Accessing the Database

- **pgweb Interface**: http://localhost:8081 (when using Docker Compose)
- **Direct Connection**: Use the DSN connection string with any PostgreSQL client

## Local Development

If you want to run the project locally without Docker, you'll need to install:

- Rust and Cargo: Install from [rustup.rs](https://rustup.rs)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- `buf` for protobuf generation: Install from [buf.build](https://buf.build/docs/installation)
- `substreams` CLI tool: Download with `brew install streamingfast/tap/substreams` or from [GitHub releases](https://github.com/streamingfast/substreams/releases)
- `substreams-sink-sql` tool: Download from [GitHub releases](https://github.com/streamingfast/substreams-sink-sql/releases)
- PostgreSQL server: Install from [postgresql.org](https://www.postgresql.org/download/)

### Local Development Workflow

1. **Start a local PostgreSQL instance**
2. **Update your `.env` file** with local database credentials
3. **Build and run the substreams**:
   ```bash
   make build
   make setup_sink
   make run_sink
   ```

## API and Data Access

Once running, you can query the PostgreSQL database directly or through pgweb to:

- Analyze block production patterns
- Track account activity and transactions
- Monitor gas usage and costs
- Examine smart contract interactions
- Build custom analytics dashboards

### Example Queries

```sql
-- Get recent blocks
SELECT height, hash, author, timestamp FROM blocks ORDER BY height DESC LIMIT 10;

-- Find function calls to a specific contract
SELECT * FROM receipt_actions 
WHERE receiver_id = 'your-contract.near' 
AND method_name = 'your_method' 
ORDER BY block_timestamp DESC;

-- Analyze gas usage patterns
SELECT DATE(block_timestamp) as date, 
       AVG(gas) as avg_gas, 
       COUNT(*) as action_count
FROM receipt_actions 
GROUP BY DATE(block_timestamp) 
ORDER BY date DESC;
```

## Troubleshooting

### Common Issues

1. **API Key Issues**: Ensure your `SUBSTREAMS_API_KEY` is valid and has sufficient quota
2. **Database Connection**: Verify your `DSN` connection string is correct
3. **GCP Credentials**: Make sure you're authenticated with `gcloud auth application-default login`
4. **Block Range**: The initial block is set to `162542069` - adjust in `substreams.yaml` if needed

### Debugging

- Enable debug logging: `RUST_LOG=debug` in your environment
- Check container logs: `docker-compose logs -f near-sink`
- Monitor database tables for data ingestion progress
- Use pgweb interface to inspect data structure and content

### Performance Considerations

- The indexer processes blocks sequentially from the configured starting block
- Database performance depends on PostgreSQL configuration and available resources
- Consider adding indexes on frequently queried columns for better performance
- Monitor disk space as blockchain data grows continuously
