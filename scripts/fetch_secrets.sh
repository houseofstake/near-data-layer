#!/bin/bash

# Check if we're running in Docker or if .env exists
if [ -f "/.dockerenv" ]; then
    ENV_PATH="/app/.env"
else
    ENV_PATH="$(dirname "$0")/../.env"
fi

# Check for .env file
if [ -f "$ENV_PATH" ]; then
    echo "Using .env file at: $ENV_PATH"
    set -o allexport
    source "$ENV_PATH"
    set +o allexport
else
    echo "No local .env file found, fetching secrets from GCP Secret Manager"
    # Fetch secrets from Secret Manager
    export INDEXER_DB_HOST=$(gcloud secrets versions access latest --secret=DATABASE_HOST)
    export INDEXER_DB_USERNAME=$(gcloud secrets versions access latest --secret=DATABASE_USER)
    export INDEXER_DB_PASSWORD=$(gcloud secrets versions access latest --secret=DATABASE_PASSWORD)
    export INDEXER_API_AUTH_TOKEN=$(gcloud secrets versions access latest --secret=FASTNEAR_API_KEY)
    export INDEXER_DD_API_KEY=$(gcloud secrets versions access latest --secret=DD_API_KEY)
fi

# Set environment variables from Terraform variables  
export INDEXER_API_CHAIN_ID=${INDEXER_API_CHAIN_ID:-${API_CHAIN_ID}}
export INDEXER_ENVIRONMENT=${INDEXER_ENVIRONMENT:-${ENVIRONMENT:-development}}
export INDEXER_DD_ENVIRONMENT=${INDEXER_DD_ENVIRONMENT:-${DD_ENVIRONMENT:-development}}

# Set RUST_LOG if not already set
export RUST_LOG=${RUST_LOG:-${INDEXER_LOG_LEVEL:-info}}

echo "Environment variables configured for NEAR indexer (chain: $INDEXER_API_CHAIN_ID, env: $INDEXER_ENVIRONMENT)" 