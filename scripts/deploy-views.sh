#!/bin/bash
set -e

# Production View Deployment Script
# This script safely deploys view changes to production with zero downtime

echo "🚀 Production View Deployment Script"
echo "===================================="

# Configuration - These should be set via environment variables in production
SCHEMA_NAME="${SCHEMA_NAME:-fastnear}"
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-dev-node}"
DB_USER="${DB_USER:-dev-node}"
DB_PASSWORD="${DB_PASSWORD:-insecure-change-me-in-prod}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    case $status in
        "success")
            echo -e "${GREEN}✅ $message${NC}"
            ;;
        "error")
            echo -e "${RED}❌ $message${NC}"
            ;;
        "warning")
            echo -e "${YELLOW}⚠️  $message${NC}"
            ;;
        "info")
            echo -e "${BLUE}ℹ️  $message${NC}"
            ;;
    esac
}

# Safety check for production
safety_check() {
    print_status "info" "Performing production safety checks..."
    
    # Check if we're in a production environment
    if [ "$DB_HOST" = "localhost" ] && [ "$DB_NAME" = "dev-node" ]; then
        print_status "warning" "Running in development mode"
        read -p "Continue with deployment? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_status "info" "Deployment cancelled"
            exit 0
        fi
    fi
    
    # Check PostgreSQL connection
    if ! pg_isready -h $DB_HOST -p $DB_PORT -U $DB_USER > /dev/null 2>&1; then
        print_status "error" "Cannot connect to PostgreSQL"
        exit 1
    fi
    
    print_status "success" "Safety checks passed"
}

# Backup current views
backup_views() {
    print_status "info" "Creating backup of current views..."
    
    local backup_file="view_backup_$(date +%Y%m%d_%H%M%S).sql"
    
    # Get list of views
    local views=$(PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -t -c "
        SELECT viewname FROM pg_views WHERE schemaname = '$SCHEMA_NAME' ORDER BY viewname;
    " | tr -d ' ')
    
    # Create backup file
    echo "-- View Backup created on $(date)" > "$backup_file"
    echo "-- Schema: $SCHEMA_NAME" >> "$backup_file"
    echo "" >> "$backup_file"
    
    for view in $views; do
        echo "-- Backup of view: $view" >> "$backup_file"
        PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "\d+ $SCHEMA_NAME.$view" >> "$backup_file" 2>/dev/null || true
        echo "" >> "$backup_file"
    done
    
    print_status "success" "Backup created: $backup_file"
}

# Deploy views in correct order
deploy_views() {
    print_status "info" "Deploying views in dependency order..."
    
    # Define the correct order for view deployment (dependencies first)
    local view_order=(
        "delegation_events.sql"
        "proposal_voting_history.sql"
        "proposals.sql"
        "approved_proposals.sql"
        "registered_voters.sql"
        "proposal_non_voters.sql"
        "user_activities.sql"
    )
    
    for view_file in "${view_order[@]}"; do
        local file_path="sql_files/views/$view_file"
        
        if [ ! -f "$file_path" ]; then
            print_status "error" "View file not found: $file_path"
            exit 1
        fi
        
        print_status "info" "Deploying view: $view_file"
        
        # Replace schema placeholder and deploy view
        if sed "s/{SCHEMA_NAME}/$SCHEMA_NAME/g" "$file_path" | \
           PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -f - > /dev/null 2>&1; then
            print_status "success" "Successfully deployed view: $view_file"
        else
            print_status "error" "Failed to deploy view: $view_file"
            exit 1
        fi
    done
}

# Verify deployment
verify_deployment() {
    print_status "info" "Verifying deployment..."
    
    local views=(
        "delegation_events"
        "proposal_voting_history"
        "proposals"
        "approved_proposals"
        "registered_voters"
        "proposal_non_voters"
        "user_activities"
    )
    
    local failed_views=()
    
    for view in "${views[@]}"; do
        print_status "info" "Verifying view: $view"
        
        if PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "SELECT COUNT(*) FROM $SCHEMA_NAME.$view LIMIT 1;" > /dev/null 2>&1; then
            print_status "success" "View $view is accessible"
        else
            print_status "error" "View $view is not accessible"
            failed_views+=("$view")
        fi
    done
    
    if [ ${#failed_views[@]} -gt 0 ]; then
        print_status "error" "Deployment verification failed for views: ${failed_views[*]}"
        exit 1
    fi
    
    print_status "success" "All views verified successfully"
}

# Health check
health_check() {
    print_status "info" "Performing health check..."
    
    # Check if views return data (even if empty)
    local views=(
        "delegation_events"
        "proposal_voting_history"
        "proposals"
        "approved_proposals"
        "registered_voters"
        "proposal_non_voters"
        "user_activities"
    )
    
    for view in "${views[@]}"; do
        local count=$(PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -t -c "SELECT COUNT(*) FROM $SCHEMA_NAME.$view;" | tr -d ' ')
        print_status "info" "View $view has $count rows"
    done
    
    print_status "success" "Health check completed"
}

# Main execution
main() {
    echo ""
    print_status "info" "Starting production view deployment..."
    echo ""
    
    safety_check
    backup_views
    deploy_views
    verify_deployment
    health_check
    
    echo ""
    print_status "success" "Production deployment completed successfully! 🎉"
    print_status "info" "All views have been updated with zero downtime."
    echo ""
}

# Run main function
main "$@" 