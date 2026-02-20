#!/bin/bash

# Check if we're running in Docker or if .env exists
if [ -f "/.dockerenv" ]; then
    ENV_PATH="/app/.env"
else
    ENV_PATH="$(dirname "$0")/../.env"
fi

# Check for .env file
if [ -f "$ENV_PATH" ]; then
    echo "Using .env file"
    set -o allexport
    source "$ENV_PATH"
    set +o allexport
else
    echo "Fetching secrets from GCP Secret Manager..."
    export INDEXER_DB_HOST=$(gcloud secrets versions access latest --secret="$SM_DATABASE_HOST")
    export INDEXER_DB_USERNAME=$(gcloud secrets versions access latest --secret="$SM_DATABASE_USER")
    export INDEXER_DB_PASSWORD=$(gcloud secrets versions access latest --secret="$SM_DATABASE_PASSWORD")
    export INDEXER_API_AUTH_TOKEN=$(gcloud secrets versions access latest --secret="$SM_API_AUTH_TOKEN")
    export INDEXER_DD_API_KEY=$(gcloud secrets versions access latest --secret="$SM_DD_API_KEY")
fi

# Set environment variables from Terraform variables  
export INDEXER_API_CHAIN_ID=${INDEXER_API_CHAIN_ID:-${API_CHAIN_ID}}
export INDEXER_ENVIRONMENT=${INDEXER_ENVIRONMENT:-${ENVIRONMENT:-development}}
export INDEXER_DD_ENVIRONMENT=${INDEXER_DD_ENVIRONMENT:-${DD_ENVIRONMENT:-development}}

# Set RUST_LOG if not already set
export RUST_LOG=${RUST_LOG:-${INDEXER_LOG_LEVEL:-info}}

echo "Environment configured (chain: $INDEXER_API_CHAIN_ID, env: $INDEXER_ENVIRONMENT)"
echo "Secrets loaded: $([ -n "$INDEXER_DB_HOST" ] && echo 'OK' || echo 'MISSING')"