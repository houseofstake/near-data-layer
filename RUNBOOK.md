# NEAR Indexer Runbook

## Overview

This runbook provides operational guidance for managing the NEAR blockchain indexer application. The indexer monitors specific NEAR contracts and stores their interaction data in PostgreSQL.

## 1. Defining Contracts to Monitor

The indexer only processes transactions that interact with contracts listed in the configuration. This filtering significantly reduces processing overhead and storage requirements. Performance metrics are automatically sent to DataDog for monitoring.

### Configuration Structure

Contracts are defined in the `config.toml` file under the `[venear_contracts]` section, organized by network:

```toml
[venear_contracts]
testnet = [
    'hos-07.testnet',
    'another-contract.testnet'
]
mainnet = [
    'contract1.near',
    'contract2.near'
]
```

### Adding New Contracts

1. **Edit `config.toml`**:
   ```toml
   [venear_contracts]
   testnet = [
       'hos-07.testnet',
       'new-contract.testnet',  # <- Add new contract here
       'another-new.testnet'
   ]
   ```

2. **Deploy the changes**:
   ```bash
   # Commit and push changes to trigger GitHub Actions deployment
   git add config.toml
   git commit -m "Add new contract: new-contract.testnet"
   git push origin main
   
   # Local testing
   cargo run -- start
   ```

### Network Selection

The indexer automatically selects contracts based on the `api_chain_id` setting:
- `testnet` → uses contracts from `venear_contracts.testnet`
- `mainnet` → uses contracts from `venear_contracts.mainnet`

### Important Notes

- **Contract matching**: The indexer uses substring matching (`account_id.contains(contract_id)`)
- **Case sensitivity**: Contract names are case-sensitive
- **Performance**: Adding more contracts increases processing time and storage usage

---

## 2. Triggering a Backfill

Backfills allow you to reprocess historical blockchain data with updated processing logic or configuration.

### When You DON'T Need a Backfill

Some changes can be deployed without incrementing `app_version` or triggering a backfill:

**Safe changes (no app_version increment needed):**
- **Logging updates**: Adding new log statements, changing log levels
- **Documentation**: README updates, code comments, runbooks
- **New metrics**: Adding DataDog metrics that don't change data processing
- **Performance optimizations**: Database connection pooling, caching improvements
- **Code refactoring**: Internal restructuring that doesn't change output
- **Configuration changes**: Non-processing settings like `poll_interval`, `retry_delay`
- **SQL views**: Updates to analytical views (proposals, voting history, etc.) since they query existing data
- **Infrastructure**: Docker, CI/CD, deployment script changes

**For these changes**: Simply commit and push - the indexer will continue from its current cursor position with the new code.

### When You DO Need a Backfill

Changes that require incrementing `app_version` and potentially backfilling:
- Adding new contracts that existed in historical blocks
- Fixing bugs in data processing logic  
- Adding new data extraction features
- Changing business logic that affects stored data
- Modifying database schema or data formats

### The Cursor System

The indexer uses a cursor system to track processing progress:
- Each `app_version` maintains its own cursor (last processed block)
- Cursors are stored in the database `cursors` table
- When restarting, the indexer resumes from the cursor position

### Backfill Process

#### Step 1: Update App Version

**CRITICAL**: You MUST increment the `app_version` to trigger a backfill.

Edit `config.toml`:
```toml
# Change from:
app_version = "v1.0.1"

# To:
app_version = "v1.0.2"  # <- Increment version
```

Or use environment variable:
```bash
INDEXER_APP_VERSION="v1.0.2"
```

#### Step 2: Set Starting Block

Update the `start_block` value based on your needs:

**For backfilling historical data:**
```toml
start_block = 183500000  # <- Set historical backfill start
```

**For changes going forward only (no backfill):**
```toml
start_block = 210000000  # <- Set to current block height to avoid reprocessing
```

**IMPORTANT**: Always increment `start_block` to the current block height when you only want changes to apply going forward. This prevents the indexer from reprocessing historical blocks with your new configuration.

#### Step 3: Deploy and Start

```bash
# Commit and push to trigger GitHub Actions deployment
git add config.toml
git commit -m "Backfill: Update app_version to v1.0.2, start_block to 183500000"
git push origin main

# Local testing
cargo run -- start
```

### Backfill Verification

Monitor logs to confirm backfill started correctly:
```bash
# Check deployment logs in GitHub Actions
# Navigate to: GitHub repo → Actions → Latest deployment workflow

# Or check application logs in your deployment environment
# (exact command depends on your infrastructure - GCP Cloud Run, etc.)

# Look for these log messages:
# "App version: v1.0.2"
# "No cursor found for app version 'v1.0.2', using config start block: 183500000"
# "Starting from block: 183500000"
```

### Important Backfill Considerations

1. **Data Overwriting**: While each app version has its own cursor, all versions write to the same database tables. Backfilling will overwrite existing data for those blocks
2. **Resource Usage**: Backfills consume significant CPU, memory, and network resources
3. **Duration**: Historical processing can take hours to days depending on block range
4. **Database Growth**: Ensure adequate storage space for additional historical data

---

## 3. Assigning Starting Blocks for Backfills

The indexer determines starting blocks using this precedence order:

### Precedence Order (Highest to Lowest)

1. **CLI Argument** (Highest Priority)
2. **Database Cursor** (for current app_version)
3. **Configuration File** (Lowest Priority)

### Method 1: CLI Argument (Temporary Override)

Use for one-time overrides or testing:

```bash
# Start from specific block (ignores config and cursor)
cargo run -- start --start-block 184000000

# For production: set environment variable and redeploy
INDEXER_START_BLOCK=184000000
```

**Use Cases**:
- Testing specific block ranges
- Emergency recovery from specific points
- One-time historical analysis

### Method 2: Configuration File (Persistent Setting)

Edit `config.toml` for permanent starting block changes:

```toml
start_block = 184000000  # First proposal block, adjust as needed
```

**Use Cases**:
- New deployments
- Permanent backfill operations
- Setting default starting points

### Method 3: Environment Variables (Deployment Override)

Override configuration without modifying files:

```bash
# Set in .env file
INDEXER_START_BLOCK=184000000

# Or export directly
export INDEXER_START_BLOCK=184000000
cargo run -- start
```

**Use Cases**:
- Environment-specific overrides (dev/staging/prod)
- GitHub Actions deployments with different starting points
- CI/CD pipeline configurations

### Block Number Guidelines

#### Common Starting Points for Testnet

| Block Number | Date (Approximate) | Description |
|--------------|-------------------|-------------|
| `183500000` | January 1st | General historical start |
| `199579301` | First proposal | First governance proposal |
| `207797689` | Current default | Recent starting point |

#### Finding Appropriate Blocks

1. **By Date**: Use NEAR explorers to find blocks by timestamp
2. **By Event**: Search for specific contract deployments or first interactions
3. **By Performance**: Balance historical completeness vs. processing time

### Example Configurations

#### Complete Historical Backfill
```toml
app_version = "v1.1.0"
start_block = 183500000  # From January 1st
```

#### Recent Data Only
```toml
app_version = "v1.1.0"
start_block = 207797689  # Last few months
```

#### Contract-Specific Backfill
```toml
app_version = "v1.1.0"
start_block = 199579301  # From first governance proposal
```

---

## GitHub Actions Deployment

This application is automatically deployed via GitHub Actions. Understanding the deployment workflow is crucial for operational procedures.

### Deployment Trigger

The GitHub Actions workflow automatically deploys when:
1. **Code changes** are pushed to the main branch
2. **Configuration changes** in `config.toml` are committed
3. **Environment variables** are updated in the repository secrets

### Environment Variables in Production

For production deployments, sensitive or environment-specific values should be set as GitHub repository secrets or environment variables, not in `config.toml`:

1. **Navigate to**: GitHub repo → Settings → Secrets and variables → Actions
2. **Add/Update secrets** such as:
   - `INDEXER_DB_PASSWORD`
   - `INDEXER_API_AUTH_TOKEN` 
   - `INDEXER_DD_API_KEY` (required for DataDog metrics)
3. **Update environment variables** in your deployment configuration

### Deployment Workflow

```bash
# 1. Make configuration changes
vim config.toml

# 2. Commit and push (triggers deployment)
git add config.toml
git commit -m "feat: trigger backfill v1.0.2 from block 183500000"
git push origin main

# 3. Monitor deployment
# GitHub repo → Actions → Latest workflow

# 4. Verify deployment
# Check application logs in GCP

# 5. Monitor DataDog metrics
# Verify indexer performance and block processing in DataDog dashboards 
```

### Rollback Procedures

If a deployment needs to be rolled back:

1. **Quick rollback**: Revert the commit and push
   ```bash
   git revert HEAD
   git push origin main
   ```

2. **Configuration rollback**: Reset specific config values
   ```bash
   # Reset to previous app version
   git checkout HEAD~1 -- config.toml
   git commit -m "rollback: revert to previous configuration"
   git push origin main
   ```

**IMPORTANT**: Rolling back to a previous `app_version` will resume processing from where that version last stopped (its cursor position). It will NOT overwrite or restore data that was processed by newer versions. If you need to restore previous data formats, you must trigger a new backfill with an incremented version.

---

## Operational Procedures

### Regular Operations

1. **Normal Restart**: Indexer automatically resumes from last cursor position
2. **Configuration Changes**: Commit and push to trigger GitHub Actions deployment
3. **Contract Updates**: Commit and push config changes to redeploy with new contracts

### Monitoring

1. **DataDog Metrics**: Indexer performance metrics are automatically sent to DataDog for monitoring processing speed, block heights, and system health
2. **Log Monitoring**: Watch for processing errors and cursor updates  
3. **Database Growth**: Monitor storage usage during backfills
4. **Performance**: Track processing speed (blocks per second) via DataDog dashboards

### Troubleshooting

#### Backfill Not Starting
- Verify `app_version` was actually changed
- Check that new version doesn't already have a cursor
- Confirm `start_block` configuration

#### Processing Errors
- Check contract addresses are correct
- Verify network connectivity to NEAR API
- Review database connection and permissions

#### Performance Issues
- Adjust `batch_size` and `num_threads` settings
- Monitor database connection pool usage
- Consider network latency to NEAR API endpoints

---

## Quick Reference Commands

```bash
# Initialize database tables (local)
cargo run -- init

# Start with default settings (local)
cargo run -- start

# Start from specific block (local)
cargo run -- start --start-block 184000000

# Start with custom thread count (local)
cargo run -- start --num-threads 32

# Check configuration
cat config.toml

# Deploy configuration changes
git add config.toml
git commit -m "Update indexer configuration"
git push origin main

# Monitor deployment in GitHub Actions
# Navigate to: GitHub repo → Actions → Latest workflow run
``` 