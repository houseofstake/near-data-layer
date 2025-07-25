# CI/CD Workflow for View Management

This document describes the comprehensive CI/CD workflow for managing database views in the NEAR Data Layer project.

## Overview

The CI/CD workflow ensures that view changes are:
- ✅ **Validated** before deployment
- ✅ **Tested** for dependencies and syntax
- ✅ **Safely deployed** with zero downtime
- ✅ **Monitored** for issues

## Components

### 1. GitHub Actions Workflow (`.github/workflows/view-validation.yml`)

**Triggers:**
- Pull requests with changes to `sql_files/views/**`
- Changes to `src/database.rs`
- Pushes to `main` or `develop` branches

**What it does:**
- Sets up a test PostgreSQL database
- Validates view creation in dependency order
- Tests view queries and dependencies
- Verifies view recreation (CREATE OR REPLACE)
- Generates deployment documentation

### 2. Local Testing Script (`scripts/test-views.sh`)

**Purpose:** Test view changes locally before pushing to CI/CD

**Usage:**
```bash
./scripts/test-views.sh
```

**Features:**
- ✅ PostgreSQL connection validation
- ✅ Data availability checking
- ✅ View creation testing
- ✅ Query validation
- ✅ View recreation testing
- ✅ Colored output for easy reading

### 3. Production Deployment Script (`scripts/deploy-views.sh`)

**Purpose:** Safely deploy view changes to production

**Usage:**
```bash
# Development mode (with confirmation)
./scripts/deploy-views.sh

# Production mode (set environment variables)
SCHEMA_NAME=fastnear DB_HOST=prod-db ./scripts/deploy-views.sh
```

**Features:**
- 🔒 Safety checks for production
- 💾 Automatic backup of current views
- 🚀 Zero-downtime deployment
- ✅ Deployment verification
- 📊 Health checks

## View Dependency Order

Views are created and deployed in this specific order to handle dependencies:

1. `delegation_events.sql` - Base delegation data
2. `proposal_voting_history.sql` - Voting history
3. `proposals.sql` - Proposal metadata
4. `approved_proposals.sql` - Approved proposals
5. `registered_voters.sql` - Voter registrations
6. `proposal_non_voters.sql` - Non-voting analysis
7. `user_activities.sql` - User activity summary

## Workflow Process

### For Developers

1. **Make view changes** in `sql_files/views/`
2. **Test locally:**
   ```bash
   ./scripts/test-views.sh
   ```
3. **Create pull request**
4. **CI/CD automatically validates** the changes
5. **Review CI/CD results** in GitHub Actions
6. **Merge to main** when validation passes

### For Production Deployment

1. **Automatic deployment** when merged to `main`
2. **Zero-downtime view updates** using `CREATE OR REPLACE`
3. **Automatic verification** of all views
4. **Health checks** to ensure data integrity

## Safety Features

### Zero Downtime
- Uses `CREATE OR REPLACE VIEW` instead of `DROP` + `CREATE`
- Views remain accessible during updates
- No interruption to front-end applications

### Automatic Backups
- Creates timestamped backups before deployment
- Backup files include view definitions and metadata
- Easy rollback if needed

### Validation Checks
- Syntax validation for all SQL
- Dependency order verification
- Query accessibility testing
- Data integrity checks

## Troubleshooting

### Common Issues

1. **Views are empty**
   - Check if source tables (`execution_outcomes`, `receipt_actions`) have data
   - Verify veNEAR contract interactions exist in the block range

2. **Dependency errors**
   - Ensure views are created in the correct order
   - Check for circular dependencies

3. **CI/CD failures**
   - Review GitHub Actions logs
   - Test locally with `./scripts/test-views.sh`
   - Check PostgreSQL connection and permissions

### Getting Data for Testing

To test views with actual data:

1. **Update start block** in `config.toml`:
   ```toml
   start_block = 199579301  # First proposal block
   ```

2. **Restart the indexer:**
   ```bash
   docker-compose down && docker-compose up -d
   ```

3. **Wait for data ingestion** and then test views

## Environment Variables

### For Local Testing
```bash
# Default values (development)
SCHEMA_NAME=fastnear
DB_HOST=localhost
DB_PORT=5432
DB_NAME=dev-node
DB_USER=dev-node
DB_PASSWORD=insecure-change-me-in-prod
```

### For Production
```bash
# Set these in your production environment
SCHEMA_NAME=fastnear
DB_HOST=your-prod-db-host
DB_PORT=5432
DB_NAME=your-prod-db
DB_USER=your-prod-user
DB_PASSWORD=your-secure-password
```

## Best Practices

1. **Always test locally** before creating pull requests
2. **Follow the dependency order** when creating new views
3. **Use `CREATE OR REPLACE VIEW`** for all view definitions
4. **Include proper comments** in SQL files
5. **Test with actual data** when possible
6. **Monitor view performance** after deployment

## Monitoring

### View Health Checks
- All views should be queryable
- Views should return data (even if empty)
- No syntax errors in view definitions

### Performance Monitoring
- Monitor query performance on large datasets
- Check for missing indexes on source tables
- Verify view refresh times

## Rollback Procedure

If deployment fails:

1. **Check the backup file** created during deployment
2. **Restore views** from backup if needed
3. **Investigate the issue** using CI/CD logs
4. **Fix the problem** and redeploy

## Support

For issues with the CI/CD workflow:

1. Check the GitHub Actions logs
2. Review this documentation
3. Test locally with the provided scripts
4. Create an issue with detailed error information 