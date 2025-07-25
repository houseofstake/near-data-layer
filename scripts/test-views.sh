#!/bin/bash
set -e

# View Testing Script
# This script helps test view changes locally before pushing to CI/CD

echo "🔍 View Testing and Validation Script"
echo "====================================="

# Configuration
SCHEMA_NAME="fastnear"
DB_HOST="localhost"
DB_PORT="5432"
DB_NAME="dev-node"
DB_USER="dev-node"
DB_PASSWORD="insecure-change-me-in-prod"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
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
            echo -e "ℹ️  $message"
            ;;
    esac
}

# Check if PostgreSQL is running
check_postgres() {
    print_status "info" "Checking PostgreSQL connection..."
    
    if ! pg_isready -h $DB_HOST -p $DB_PORT -U $DB_USER > /dev/null 2>&1; then
        print_status "error" "PostgreSQL is not running or not accessible"
        print_status "info" "Please start PostgreSQL or check your connection settings"
        exit 1
    fi
    
    print_status "success" "PostgreSQL connection established"
}

# Test view creation
test_view_creation() {
    print_status "info" "Testing view creation in dependency order..."
    
    # Define the correct order for view creation (dependencies first)
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
        
        print_status "info" "Testing view: $view_file"
        
        # Replace schema placeholder and test view creation
        if sed "s/{SCHEMA_NAME}/$SCHEMA_NAME/g" "$file_path" | \
           PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -f - > /dev/null 2>&1; then
            print_status "success" "Successfully created view: $view_file"
        else
            print_status "error" "Failed to create view: $view_file"
            exit 1
        fi
    done
}

# Test view queries
test_view_queries() {
    print_status "info" "Testing view queries..."
    
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
        print_status "info" "Testing query on view: $view"
        
        if PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "SELECT COUNT(*) FROM $SCHEMA_NAME.$view LIMIT 1;" > /dev/null 2>&1; then
            print_status "success" "Successfully queried view: $view"
        else
            print_status "error" "Failed to query view: $view"
            exit 1
        fi
    done
}

# Test view recreation
test_view_recreation() {
    print_status "info" "Testing view recreation (CREATE OR REPLACE)..."
    
    for view_file in sql_files/views/*.sql; do
        local filename=$(basename "$view_file")
        print_status "info" "Testing recreation of: $filename"
        
        if sed "s/{SCHEMA_NAME}/$SCHEMA_NAME/g" "$view_file" | \
           PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -f - > /dev/null 2>&1; then
            print_status "success" "Successfully recreated view: $filename"
        else
            print_status "error" "Failed to recreate view: $filename"
            exit 1
        fi
    done
}

# Check for data in tables
check_data_availability() {
    print_status "info" "Checking data availability in source tables..."
    
    local tables=("execution_outcomes" "receipt_actions")
    
    for table in "${tables[@]}"; do
        local count=$(PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -t -c "SELECT COUNT(*) FROM $SCHEMA_NAME.$table;" | tr -d ' ')
        
        if [ "$count" -gt 0 ]; then
            print_status "success" "Table $table has $count rows"
        else
            print_status "warning" "Table $table is empty - views will also be empty"
        fi
    done
}

# Main execution
main() {
    echo ""
    print_status "info" "Starting view validation..."
    echo ""
    
    check_postgres
    check_data_availability
    test_view_creation
    test_view_queries
    test_view_recreation
    
    echo ""
    print_status "success" "All view tests passed! 🎉"
    print_status "info" "Your views are ready for production deployment."
    echo ""
}

# Run main function
main "$@" 