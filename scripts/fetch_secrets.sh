#!/bin/bash

# Set environment (default to dev if not specified)
ENV=${ENVIRONMENT:-DEV}

# Check if we're running in Docker
if [ -f "/.dockerenv" ]; then
    ENV_PATH="/app/.env"
else
    ENV_PATH="$(dirname "$0")/../.env"
fi

# Check for .env file
if [ -f "$ENV_PATH" ]; then
    echo "Using .env file at: $ENV_PATH"
    source "$ENV_PATH"
else
    echo "No local .env file found, fetching secrets from GCP Secret Manager"
    # Fetch secrets from Secret Manager
    export DATABASE_USER=$(gcloud secrets versions access latest --secret=DATABASE_USER)
    export DATABASE_PASSWORD=$(gcloud secrets versions access latest --secret=DATABASE_PASSWORD)
    export DATABASE_HOST=$(gcloud secrets versions access latest --secret=DATABASE_HOST_${ENV})
    export SUBSTREAMS_API_KEY=$(gcloud secrets versions access latest --secret=PINAX_API_KEY)
    export ENDPOINT=$(gcloud secrets versions access latest --secret=PINAX_ENDPOINT_${ENV})

    # Construct DSN
    export DSN="postgres://${DATABASE_USER}:${DATABASE_PASSWORD}@${DATABASE_HOST}:5432/postgres?sslmode=disable"
fi
