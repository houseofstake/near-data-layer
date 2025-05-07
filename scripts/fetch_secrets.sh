#!/bin/bash

# Set environment (default to dev if not specified)
ENV=${ENVIRONMENT:-DEV}

# Fetch secrets from Secret Manager
export DATABASE_USER=$(gcloud secrets versions access latest --secret=DATABASE_USER)
export DATABASE_PASSWORD=$(gcloud secrets versions access latest --secret=DATABASE_PASSWORD)
export SUBSTREAMS_API_KEY=$(gcloud secrets versions access latest --secret=PINAX_API_KEY)
export ENDPOINT=$(gcloud secrets versions access latest --secret=PINAX_ENDPOINT_${ENV})

# Construct DSN
export DSN="postgres://${DATABASE_USER}:${DATABASE_PASSWORD}@10.20.179.3:5432/postgres?sslmode=disable"
