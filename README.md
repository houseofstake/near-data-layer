# NEAR Data Layer

A high-performance, real-time blockchain data indexing infrastructure that powers the Agora web application. This data layer provides sub-second latency access to NEAR blockchain data through a simple streaming architecture that leverages StreamingFast Substreams and PostgreSQL.


## Architecture Overview

The NEAR data layer is built on a modern streaming architecture that processes blockchain data in real-time:

```
NEAR Blockchain → Pinax Substreams → substreams-sink-sql → PostgreSQL → Agora BE/FE
```

### Key Components

1. **Pinax-hosted Substreams**: Hosted deployment of custom NEAR substreams packages
2. **StreamingFast Infrastructure**: Powers the real-time data streaming using `substreams-sink-sql`
3. **PostgreSQL Database**: Shared data store serving both the indexer and Agora backend
4. **GCP Infrastructure**: Cloud-based deployment with Compute Engine VM and CloudSQL

### Performance Characteristics

- **Blockchain to PostgreSQL latency**: Tens of milliseconds
- **End-to-end query latency**: Hundreds of milliseconds
- **Data processing**: Real-time streaming using Rust and gRPC
- **Cost efficiency**: ~$40/month for Pinax hosting
- **Data optimization**: Preemptive filtering keeps indexed tables lightweight

## Data Model

![Data Model](https://github.com/user-attachments/assets/244ae41f-f40f-45ef-8f5c-c385fe01860c)

### Core Tables

- **`blocks`**: NEAR blockchain blocks with metadata
- **`receipt_actions`**: Function call actions and their parameters
- **`execution_outcomes`**: Results of action execution including gas usage and logs

### Views

The data layer includes several optimized views for the Agora application:

- **`proposals`**: Governance proposals with voting metadata
- **`registered_voters`**: veNEAR token holders eligible to vote
- **`proposal_voting_history`**: Individual vote records
- **`delegation_events`**: Voting power delegation transactions
- **`approved_proposals`**: Proposals approved for public voting

## Infrastructure & Deployment

### Cloud Architecture

- **Substreams**: Hosted on Pinax infrastructure
- **Compute**: GCP Compute Engine VM running the indexer service
- **Database**: CloudSQL PostgreSQL instance (shared with Agora backend)
- **Infrastructure as Code**: Fully terraformed deployment (separate repo)
- **CI/CD**: Automated redeployment of indexer service
- **CI/CD**: Automated deployment of view definition updates (in progress)

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Google Cloud SDK
- PINAX API key

### Local Development Setup

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd near-data-layer
   ```

2. **Set up GCP credentials** (for production secrets)
   ```bash
   # Install Google Cloud SDK
   brew install google-cloud-sdk

   # Authenticate
   gcloud auth application-default login
   ```
   This will create a credentials file at `~/.config/gcloud/application_default_credentials.json` which will be automatically mounted into the container.

3. **Configure environment variables**

   Create a `.env` file:
   ```env
   DSN=postgres://dev-node:insecure-change-me-in-prod@postgres:5432/dev-node?sslmode=disable
   ENDPOINT=near.substreams.pinax.network:443
   SUBSTREAMS_API_KEY=your_pinax_api_key_here
   ```

   The application will:
   - First check for a local `.env` file in the project root
   - If no `.env` file is found, it will attempt to fetch secrets from GCP Secret Manager
   - Make sure you have GCP credentials set up (see GCP Credentials Setup section) if you plan to use GCP Secret Manager

4. **Start the services**
   ```bash
   docker-compose up --build
   ```

   This starts:
   - PostgreSQL database (port 5432)
   - pgweb interface (http://localhost:8081)
   - NEAR sink container with real-time indexing

### Development Tools

#### Requirements for Local Development

**Install via Homebrew (recommended):**
```bash
# Core development tools
brew install rust
brew install buf
brew install streamingfast/tap/substreams
brew install streamingfast/tap/substreams-sink-sql
```

**Additional setup:**
```bash
# Add WebAssembly target for Rust
rustup target add wasm32-unknown-unknown
```

**Alternative installations (if Homebrew not available):**
- **Rust**: Install from [rustup.rs](https://rustup.rs)
- **buf**: Download from [buf.build](https://buf.build/docs/installation)
- **Substreams CLI**: Download from [GitHub releases](https://github.com/streamingfast/substreams/releases)
- **Sink SQL**: Download from [substreams-sink-sql releases](https://github.com/streamingfast/substreams-sink-sql/releases)

#### Building the Substreams Package

```bash
# Generate protobuf files
make protogen

# Build the WASM module
make build

# Package the substreams
substreams pack

# Optional: Set up the database sink
make setup_sink

# Optional: Run the sink locally
make run_sink
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DSN` | PostgreSQL connection string | Local docker postgres |
| `ENDPOINT` | NEAR Substreams endpoint | `near.substreams.pinax.network:443` |
| `SUBSTREAMS_API_KEY` | PINAX API key for access | Required |
| `GOOGLE_APPLICATION_CREDENTIALS` | GCP service account path | Auto-mounted in Docker |

### Substreams Configuration

The substreams package is configured in `substreams.yaml`:

- **Initial TestNet Block**: 162542069 (May 2024)
- **Network**: NEAR testnet
- **Filtering**: Optimized for Agora-specific data requirements, configurable in `config/`

### Database Schema

The PostgreSQL schema (`schema.sql`) includes:

- Core blockchain data tables
- Optimized indexes for query performance

## Data Processing Pipeline

### 1. Blockchain Data Ingestion
- Real-time streaming from NEAR blockchain
- Substreams filters and processes relevant transactions
- Focus on governance-related actions (proposals, voting, delegation)

### 2. Data Transformation
- Receipt actions are decoded and structured
- Execution outcomes provide success/failure status
- Base64-encoded arguments are parsed into JSON

### 3. Database Updates
- Streaming inserts into PostgreSQL tables
- Optimized for both write performance and query speed

### 4. API Layer
- Shared PostgreSQL instance serves Agora backend
- Non-materialized views provide real-time data access

## Monitoring & Operations

### Health Checks

WIP

### Logs & Debugging

WIP

## Development Workflow

### Making Schema Changes

**Most schema changes will trigger a backfill unless it the indexer is explicitly set to ignore the mismatch.**

1. Update `schema.sql` with new table/index definitions
2. Update substreams modules in `src/` if needed
3. Rebuild and redeploy via CI/CD

## Contributing

### Development Setup

1. Fork the repository
2. Set up local development environment
3. Make changes and test locally
4. Submit pull request with comprehensive testing

### Code Style

- Rust code follows standard formatting (`cargo fmt`)
- SQL follows consistent naming conventions

## Troubleshooting

### Common Issues

1. **Substreams connection failures**
   - Verify PINAX API key is valid
   - Check endpoint connectivity
   - Review authentication setup

2. **Database connection issues**
   - Ensure PostgreSQL container is running
   - Verify DSN connection string
   - Check port availability

3. **Performance issues**
   - Monitor PostgreSQL query performance
   - Check materialized view refresh schedules
   - Verify index usage

### Support

For technical issues:
1. Check container logs: `docker-compose logs`
2. Review PostgreSQL logs for query issues
3. Verify substreams processing status
4. Contact team for infrastructure access

