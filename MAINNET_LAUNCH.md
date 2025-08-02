# Mainnet Launch Guide

## Overview

This guide outlines the process for launching the NEAR indexer to production mainnet. The launch involves transitioning from testnet to mainnet contract indexing.

## Prerequisites

- [ ] Mainnet contract address confirmed and deployed
- [ ] Mainnet contract deployment block number identified
- [ ] Access to production database

---

## Launch Process

### Step 1: Create Mainnet Configuration

#### Create `config-mainnet.toml`

Create a separate configuration file for mainnet to avoid modifying the existing testnet config:

```toml
# config-mainnet.toml - Production mainnet configuration

# Database Configuration
db_host = "postgres"
db_port = 5432
db_database = "postgres"
db_username = "postgres"
db_password = "postgres"
db_schema = "fastnear"
db_max_connections = 10

# Indexer Configuration
start_block = 999999999  # <- SET TO ACTUAL MAINNET CONTRACT DEPLOYMENT BLOCK
poll_interval = 1
retry_delay = 5
num_threads = 64

# NEAR API Configuration
api_chain_id = "mainnet"  # <- MAINNET
api_finality = "final"

# Other
log_level = "info"
app_version = "v2.0.0"  # <- Major version bump for mainnet

# DataDog Configuration
datadog_enabled = true
environment = 'production'
dd_environment = 'production'

# Contract Configuration
hos_contracts = [
    'your-mainnet-contract.near'  # <- ADD ACTUAL MAINNET CONTRACT
]
```

#### Keep `config.toml` for Testnet/Dev

Rename or keep the existing config for development:

```bash
# Optionally rename existing config for clarity
mv config.toml config-testnet.toml
```

#### Set Environment for Mainnet

The application automatically selects the mainnet configuration when the chain ID is set:

**Set Chain ID Environment Variable**
```bash
# This will automatically use configs/mainnet.toml
export INDEXER_API_CHAIN_ID="mainnet"
```

**For Docker/Production Deployment**
```dockerfile
# Set in Dockerfile or deployment environment
ENV INDEXER_API_CHAIN_ID=mainnet
```

**Terraform Variables (if using environment-specific deployments)**
```hcl
# In near-tf/variables/prod.tfvars
api_chain_id = "mainnet"

# In near-tf/variables/dev.tfvars  
api_chain_id = "testnet"
```

### Step 2: Deploy to Production

```bash
# 1. Commit mainnet configuration files (if not already done)
git add configs/mainnet.toml
git add configs/testnet.toml
git add src/config.rs         # updated config loading logic
git add near-tf/variables/    # if using terraform approach
git commit -m "feat: mainnet launch - add environment-based config selection"

# 2. Set production environment variables
# Either via GitHub repository secrets or Terraform:
# INDEXER_API_CHAIN_ID=mainnet

# 3. Push to main branch (triggers production deployment)
git push origin main

# 4. Monitor deployment in GitHub Actions
# Navigate to: GitHub repo → Actions → Latest workflow

# 5. Verify deployment in GCP
# Check that mainnet configuration is active and chain_id=mainnet
```

### Step 3: Clean Existing Testnet Data

⚠️ **CRITICAL**: Choose ONE of the following options to remove testnet data:

#### Option 1: Truncate Above Testnet Block (Recommended)

```sql
-- Connect to production database
-- This removes all data from blocks 183500000 and above (all testnet data)

-- Truncate main data tables
TRUNCATE TABLE fastnear.blocks CASCADE;
TRUNCATE TABLE fastnear.transactions CASCADE; 
TRUNCATE TABLE fastnear.receipts CASCADE;
-- Add other data tables as needed

-- Reset cursors to force restart from configured start_block
DELETE FROM fastnear.cursors WHERE block_num >= 183500000;

-- Verify cleanup
SELECT COUNT(*) FROM fastnear.blocks;  -- Should be 0 or very low
SELECT * FROM fastnear.cursors;       -- Should show no recent cursors
```

#### Option 2: Drop All Tables (Brute Force)

```sql
-- Connect to production database
-- This will cause indexer to crash and restart, recreating tables

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
-- Add other tables as needed

-- Indexer will crash and restart, recreating everything from scratch
```

### Step 4: Monitor Indexer Startup

```bash
# 1. Check GCP logs for indexer restart
# Look for these key log messages:

# "App version: v2.0.0" 
# "Using chain ID: mainnet"
# "HOS contracts: [\"your-mainnet-contract.near\"]"
# "No cursor found for app version 'v2.0.0', using config start block: XXXXXX"
# "Starting from block: XXXXXX"

# 2. Monitor DataDog metrics
# Verify indexer is processing blocks and performance is healthy

# 3. Check database for new data
psql -d production_db -c "
SELECT 
    MAX(block_height) as latest_block,
    COUNT(*) as total_blocks,
    MAX(block_timestamp) as latest_timestamp
FROM fastnear.blocks;
"
```

### Step 5: Verify Mainnet Indexing

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

-- Check transactions are related to mainnet contracts
SELECT 
    transaction_hash,
    receiver_account_id,
    signer_account_id
FROM fastnear.transactions 
WHERE receiver_account_id LIKE '%your-mainnet-contract.near%'
ORDER BY included_in_block_height DESC
LIMIT 10;

-- Verify cursors are advancing
SELECT * FROM fastnear.cursors ORDER BY block_num DESC;
```

#### Performance Monitoring

- **DataDog**: Monitor block processing rate, system resources
- **Logs**: Verify no error messages, steady progress
- **Database**: Confirm steady growth in block/transaction counts

### Step 6: Wait for Catch-Up

Monitor indexer progress until it reaches the current mainnet block height using DataDog metrics and dashboards to track:

- **Block processing rate** (blocks per second)
- **Current block height** vs target mainnet height
- **Processing lag** and catch-up progress
- **System performance** during indexing


---

## Launch Verification Checklist

- [ ] **Configuration**: Mainnet contracts and chain_id configured
- [ ] **Deployment**: GitHub Actions deployment completed successfully
- [ ] **Data Cleanup**: Testnet data removed from production database
- [ ] **Indexer Startup**: New app version started from correct mainnet block
- [ ] **Data Flow**: Mainnet transactions being processed and stored
- [ ] **Performance**: DataDog metrics showing healthy processing
- [ ] **Views**: Analytical SQL views recreated and functioning
- [ ] **Catch-Up**: Indexer reached current mainnet block height

---

## Post-Launch

### Monitoring Setup

- **DataDog Alerts**: Set up production alerting for processing delays
- **Database Monitoring**: Monitor storage growth and performance  
- **Log Aggregation**: Ensure production logs are properly collected

### Documentation Updates

- Update README with mainnet contract addresses
- Update any API documentation pointing to testnet
- Notify frontend teams of mainnet availability

---

## Mainnet Launch Complete! 🚀

The NEAR indexer is now successfully running on mainnet and processing real governance data. 