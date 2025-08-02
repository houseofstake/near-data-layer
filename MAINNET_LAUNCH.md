# Mainnet Launch Guide

## TL;DR

1. **Populate PR** with mainnet contract address
2. **Merge near-tf PR** (`near-api-chain-param-v2`) and deploy to prod  
3. **Merge near-data-layer PR** (`data-runbook`) with mainnet config and deploy to prod
4. ⚠️ **WAIT for step 3 to complete and confirm via VM logs, THEN drop production tables** to remove testnet data
5. **Monitor** indexer startup and mainnet indexing

## Overview

This guide outlines the process for launching the NEAR indexer to production mainnet. The launch involves transitioning from testnet to mainnet contract indexing using Terraform-managed infrastructure.

## Prerequisites

- [ ] **Terraform PR merged**: The `near-api-chain-param-v2` branch with `api_chain_id` variable must be merged to main
- [ ] Mainnet contract address confirmed and deployed
- [ ] Mainnet contract deployment block number identified
- [ ] Access to production database and GCP environment

---

## Launch Process

### Step 1: Merge Terraform Infrastructure Changes

**CRITICAL**: The Terraform changes that add `api_chain_id` variable support must be deployed first.

### Step 2: Update Mainnet Configuration

#### Update `configs/mainnet.toml`

Ensure the mainnet config file has the correct settings:

```toml
# configs/mainnet.toml - Production mainnet configuration
...
# Indexer Configuration
start_block = 999999999  # <- SET TO ACTUAL MAINNET CONTRACT DEPLOYMENT BLOCK
...

# NEAR API Configuration  
api_chain_id = "mainnet"  # <- This will be overridden by INDEXER_API_CHAIN_ID env var from Terraform
...
app_version = "v2.0.0"  # <- Major version bump for mainnet


# Contract Configuration
hos_contracts = [
    "hos-123.near"  # <- ADD ACTUAL MAINNET CONTRACT ADDRESS
]
```

### Step 3: Merge and Deploy Terraform PR

Merge the `near-api-chain-param-v2` branch into main and deploy to production using near-tf GitHub Actions.

### Step 4: Deploy Mainnet Configuration

Deploy the near-data-layer PR with mainnet configuration changes to production.

### Step 5: Clean Existing Testnet Data

⚠️ **CRITICAL**: WAIT for Step 4 deployment to complete and confirm via VM logs, THEN remove testnet data from production database.

**Confirm in VM logs before proceeding to drop tables:**
- "Using chain ID: mainnet"
- "HOS contracts: [\"hos-123.near\"]"
- Indexer running without errors

```bash
# Check GCP logs for indexer restart
# Look for these key log messages:

# "App version: v2.0.0" 
# "Using chain ID: mainnet"
# "HOS contracts: [\"hos-123.near\"]"
# "No cursor found for app version 'v2.0.0', using config start block: XXXXXX"
# "Starting from block: XXXXXX"
```

#### Recommended: Drop Tables and Restart Indexer

```sql
-- Connect to production database
-- This will cause indexer to crash and restart, recreating everything from scratch

-- Drop all views first (they depend on tables)
DROP VIEW IF EXISTS fastnear.proposals CASCADE;
DROP VIEW IF EXISTS fastnear.registered_voters CASCADE;
DROP VIEW IF EXISTS fastnear.proposal_voting_history CASCADE;
DROP VIEW IF EXISTS fastnear.user_activities CASCADE;
DROP VIEW IF EXISTS fastnear.delegation_events CASCADE;
DROP VIEW IF EXISTS fastnear.approved_proposals CASCADE;
DROP VIEW IF EXISTS fastnear.proposal_non_voters CASCADE;

-- Drop all tables
DROP TABLE IF EXISTS fastnear.cursors CASCADE;
DROP TABLE IF EXISTS fastnear.blocks CASCADE;
DROP TABLE IF EXISTS fastnear.transactions CASCADE;
DROP TABLE IF EXISTS fastnear.receipts CASCADE;

-- Indexer will crash and restart, recreating everything from scratch
```

### Step 6: Verify Mainnet Indexing

#### Database Verification

```sql
-- Verify mainnet data is being indexed
SELECT 
    block_height,
    block_hash,
    block_timestamp,
    total_transactions
FROM fastnear.blocks 
ORDER BY block_height DESC 
LIMIT 10;

-- Verify cursors are advancing
SELECT * FROM fastnear.cursors ORDER BY block_num DESC;
```

#### Performance Monitoring

- **DataDog**: Monitor block processing rate, system resources
- **GCP Logs**: Verify no error messages, steady progress
- **Database**: Confirm steady growth in block/transaction counts


## Launch Verification Checklist

- [ ] **near-tf PR**: `api_chain_id` variable changes merged and deployed
- [ ] **data-layer PR**: Mainnet contract config merged and deployed  
- [ ] **Data Cleanup**: Production tables dropped and testnet data removed
- [ ] **Indexer Startup**: Mainnet indexing started successfully
- [ ] **Monitoring**: DataDog metrics showing healthy processing
