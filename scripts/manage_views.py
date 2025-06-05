#!/usr/bin/env python3
import os
import glob
import psycopg2
from jinja2 import Environment, FileSystemLoader
from google.cloud import secretmanager
from google.oauth2 import service_account

# Configuration
class Config:
    def __init__(self):
        self.project_id = os.getenv('GCP_PROJECT_ID')
        self.instance_connection_name = os.getenv('INSTANCE_CONNECTION_NAME')
        self.db_user = os.getenv('DB_USER')
        self.db_name = os.getenv('DB_NAME')
        
        # Contract IDs for templating
        self.contract_ids = {
            'venear_contract': 'r-1745564650.testnet',
            'voting_contract': 'r-1745564650.testnet'
        }

def get_secret(secret_id):
    """Retrieve secret from Google Cloud Secret Manager."""
    client = secretmanager.SecretManagerServiceClient()
    name = f"projects/{Config().project_id}/secrets/{secret_id}/versions/latest"
    response = client.access_secret_version(request={"name": name})
    return response.payload.data.decode("UTF-8")

def get_db_connection():
    """Create database connection using Cloud SQL Proxy."""
    config = Config()
    
    # Get DB password from Secret Manager
    db_password = get_secret('postgres-password')
    
    conn = psycopg2.connect(
        host='localhost',  # Using Cloud SQL Proxy
        port=5432,
        dbname=config.db_name,
        user=config.db_user,
        password=db_password
    )
    return conn

def process_sql_file(file_path, jinja_vars):
    """Process SQL file with Jinja templating."""
    env = Environment(loader=FileSystemLoader('.'))
    template = env.get_template(file_path)
    return template.render(**jinja_vars)

def refresh_materialized_views():
    """Main function to refresh all materialized views."""
    config = Config()
    conn = get_db_connection()
    cursor = conn.cursor()
    
    try:
        # Get all SQL files in materialized_views directory
        sql_files = glob.glob('materialized_views/*.sql')
        
        # Sort files to handle dependencies
        # Add your dependency order here if needed
        view_order = [
            'delegation_events.sql',
            'registered_voters.sql',
            'proposal_voting_history.sql',
            'approved_proposals.sql',
            'proposals.sql',
            'proposal_non_voters.sql'
        ]
        
        # Sort files based on the order
        sql_files.sort(key=lambda x: view_order.index(os.path.basename(x)) if os.path.basename(x) in view_order else len(view_order))
        
        for sql_file in sql_files:
            view_name = os.path.basename(sql_file).replace('.sql', '')
            print(f"Processing {view_name}...")
            
            # Drop existing view if exists
            cursor.execute(f"DROP MATERIALIZED VIEW IF EXISTS {view_name} CASCADE;")
            
            # Process the SQL file with Jinja
            sql_content = process_sql_file(sql_file, config.contract_ids)
            
            # Execute the processed SQL
            cursor.execute(sql_content)
            conn.commit()
            
            print(f"Successfully refreshed {view_name}")
            
    except Exception as e:
        conn.rollback()
        print(f"Error: {str(e)}")
        raise
    finally:
        cursor.close()
        conn.close()

if __name__ == "__main__":
    refresh_materialized_views() 