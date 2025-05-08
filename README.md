# NEAR Blockchain SQL Sink

A substreams-based indexer for the NEAR blockchain. Data is sunk into a PostgreSQL database.

## The Data Model
![](https://github.com/user-attachments/assets/244ae41f-f40f-45ef-8f5c-c385fe01860c)

## Prerequisites

- Docker and Docker Compose

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

## Local Development

If you want to run the project locally without Docker, you'll need to install:

- Rust and Cargo: Install from [rustup.rs](https://rustup.rs)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- `buf` for protobuf generation: Install from [buf.build](https://buf.build/docs/installation)
- `substreams` CLI tool: Download with `brew install streamingfast/tap/substreams` or from [GitHub releases](https://github.com/streamingfast/substreams/releases)
- `substreams-sink-sql` tool: Download from [GitHub releases](https://github.com/streamingfast/substreams-sink-sql/releases)
- PostgreSQL server: Install from [postgresql.org](https://www.postgresql.org/download/)
