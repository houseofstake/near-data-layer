#!/usr/bin/env python3
import os
import glob
import time
import subprocess
import signal
import sys
import socket
from jinja2 import Environment, FileSystemLoader
from google.cloud import secretmanager

# Configuration
class Config:
    def __init__(self):
        self.project_id = os.getenv('GCP_PROJECT_ID', 'agora-near')
        self.instance_name = os.getenv('INSTANCE_NAME', 'sql-access-vm')
        self.zone = os.getenv('ZONE', 'us-west1-a')
        self.db_user = os.getenv('DB_USER', 'near_user')
        self.db_name = os.getenv('DB_NAME', 'postgres')
        
        # Contract IDs for templating
        self.contract_ids = {
            'venear_contract': os.getenv('VENEAR_CONTRACT_ID', 'r-1748895584.testnet'),
            'voting_contract': os.getenv('VOTING_CONTRACT_ID', 'r-1748895584.testnet')
        }

def get_secret(secret_id):
    """Retrieve secret from Google Cloud Secret Manager."""
    client = secretmanager.SecretManagerServiceClient()
    name = f"projects/{Config().project_id}/secrets/{secret_id}/versions/latest"
    response = client.access_secret_version(request={"name": name})
    return response.payload.data.decode("UTF-8")

def wait_for_port(port, host='127.0.0.1', timeout=30):
    """Wait for a port to be ready."""
    start_time = time.time()
    while True:
        try:
            with socket.create_connection((host, port), timeout=1):
                return True
        except OSError:
            time.sleep(1)
            if time.time() - start_time >= timeout:
                return False

def setup_tunnel():
    """Set up IAP tunnel to the SQL instance."""
    config = Config()
    
    # Kill any existing processes using port 5432
    try:
        subprocess.run(['lsof', '-ti', ':5432', '-sTCP:LISTEN'], capture_output=True)
        subprocess.run(['pkill', '-f', f'ssh.*{config.instance_name}.*5432'], capture_output=True)
    except Exception:
        pass  # Ignore errors if no processes found
    
    # First, ensure we can SSH to the instance
    print("Testing SSH connection...")
    test_cmd = [
        'gcloud', 'compute', 'ssh',
        config.instance_name,
        '--tunnel-through-iap',
        f'--project={config.project_id}',
        f'--zone={config.zone}',
        '--command', 'echo "SSH connection successful"'
    ]
    
    result = subprocess.run(test_cmd, capture_output=True, text=True)
    if result.returncode != 0:
        print("SSH test output:", result.stdout)
        print("SSH test error:", result.stderr)
        raise Exception("Failed to establish SSH connection")
    
    # Now set up the tunnel
    print("Setting up IAP tunnel...")
    tunnel_cmd = [
        'gcloud', 'compute', 'ssh',
        config.instance_name,
        '--tunnel-through-iap',
        f'--project={config.project_id}',
        f'--zone={config.zone}',
        '--',
        '-L', '5432:localhost:5432',
        '-N',  # Don't execute remote command
        '-o', 'ExitOnForwardFailure=yes'  # Exit if port forwarding fails
    ]
    
    tunnel_process = subprocess.Popen(tunnel_cmd, 
                                    stdout=subprocess.PIPE,
                                    stderr=subprocess.PIPE)
    
    # Wait for tunnel to be established
    if not wait_for_port(5432):
        tunnel_process.terminate()
        raise Exception("Failed to establish tunnel connection")
    
    print("Tunnel established successfully")
    return tunnel_process

def process_sql_file(file_path, jinja_vars):
    """Process SQL file with Jinja templating."""
    env = Environment(loader=FileSystemLoader('.'))
    template = env.get_template(file_path)
    return template.render(**jinja_vars)

def test_db_connection():
    """Test the database connection."""
    config = Config()
    db_password = get_secret('DATABASE_PASSWORD')
    
    # Create a temporary password file
    pass_file = '.pgpass_temp'
    with open(pass_file, 'w') as f:
        f.write(f'127.0.0.1:5432:*:{config.db_user}:{db_password}')
    os.chmod(pass_file, 0o600)
    
    try:
        # Test connection
        env = os.environ.copy()
        env['PGPASSFILE'] = os.path.abspath(pass_file)
        
        test_cmd = [
            'psql',
            '-h', '127.0.0.1',
            '-U', config.db_user,
            '-d', config.db_name,
            '-c', 'SELECT 1'
        ]
        
        result = subprocess.run(test_cmd, 
                              env=env,
                              capture_output=True,
                              text=True)
        
        if result.returncode != 0:
            raise Exception(f"Database connection test failed: {result.stderr}")
        
        print("Database connection test successful")
        
    finally:
        if os.path.exists(pass_file):
            os.remove(pass_file)

def execute_sql(sql_content):
    """Execute SQL using psql."""
    config = Config()
    db_password = get_secret('DATABASE_PASSWORD')
    
    # Create a temporary password file
    pass_file = '.pgpass_temp'
    with open(pass_file, 'w') as f:
        f.write(f'127.0.0.1:5432:*:{config.db_user}:{db_password}')
    os.chmod(pass_file, 0o600)
    
    try:
        # Create environment with PGPASSFILE set
        env = os.environ.copy()
        env['PGPASSFILE'] = os.path.abspath(pass_file)
        
        # Write SQL to temporary file
        sql_file = '.temp.sql'
        with open(sql_file, 'w') as f:
            f.write(sql_content)
        
        # Execute SQL
        psql_cmd = [
            'psql',
            '-h', '127.0.0.1',
            '-U', config.db_user,
            '-d', config.db_name,
            '-f', sql_file,
            '--set', 'ON_ERROR_STOP=1'  # Stop on first error
        ]
        
        result = subprocess.run(psql_cmd, 
                              env=env,
                              capture_output=True,
                              text=True)
        
        if result.returncode != 0:
            print("Error executing SQL:")
            print(result.stderr)
            raise Exception("SQL execution failed")
        
        print(result.stdout)
        
    finally:
        # Clean up temporary files
        if os.path.exists(pass_file):
            os.remove(pass_file)
        if os.path.exists(sql_file):
            os.remove(sql_file)

def generate_sql():
    """Generate SQL commands to refresh all materialized views."""
    config = Config()
    
    # Get all SQL files in materialized_views directory
    sql_files = glob.glob('materialized_views/*.sql')
    
    # Sort files to handle dependencies
    view_order = [
        'delegation_events.sql',
        'proposal_voting_history.sql',
        'proposals.sql',
        'approved_proposals.sql',
        'registered_voters.sql',
        'proposal_non_voters.sql'
    ]
    
    # Filter and sort files based on the order
    sql_files = [f for f in sql_files if os.path.basename(f) in view_order]
    sql_files.sort(key=lambda x: view_order.index(os.path.basename(x)))
    
    if not sql_files:
        raise Exception("No SQL files found in materialized_views directory")
    
    print(f"Processing views in order: {[os.path.basename(f) for f in sql_files]}")
    
    # Generate SQL
    sql_content = ["BEGIN;"]
    
    for sql_file in sql_files:
        view_name = os.path.basename(sql_file).replace('.sql', '')
        sql_content.append(f"\n-- Processing {view_name}")
        
        # Save existing indexes
        sql_content.append(f"""
DO $$
DECLARE
    index_names text;
BEGIN
    SELECT string_agg(indexname, ', ')
    INTO index_names
    FROM pg_indexes
    WHERE tablename = '{view_name}';
    
    IF index_names IS NOT NULL THEN
        RAISE NOTICE 'Existing indexes for {view_name}: %', index_names;
    END IF;
END $$;
""")
        
        # Drop existing view
        sql_content.append(f"DROP MATERIALIZED VIEW IF EXISTS {view_name} CASCADE;")
        
        # Add the SQL file contents
        sql_content.append(process_sql_file(sql_file, config.contract_ids))
        
        # Verify the view was created
        sql_content.append(f"""
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT FROM pg_matviews
        WHERE matviewname = '{view_name}'
    ) THEN
        RAISE EXCEPTION 'Failed to create materialized view {view_name}';
    END IF;
    
    -- List new indexes
    RAISE NOTICE 'New indexes for {view_name}: %', (
        SELECT string_agg(indexname, ', ')
        FROM pg_indexes
        WHERE tablename = '{view_name}'
    );
END $$;
""")
    
    sql_content.append("\nCOMMIT;")
    return '\n'.join(sql_content)

def cleanup(tunnel_process):
    """Clean up resources."""
    if tunnel_process:
        tunnel_process.terminate()
        tunnel_process.wait()

def main():
    tunnel_process = None
    try:
        # Set up tunnel
        tunnel_process = setup_tunnel()
        
        # Test database connection
        test_db_connection()
        
        # Generate and execute SQL
        sql_content = generate_sql()
        execute_sql(sql_content)
        
    except Exception as e:
        print(f"Error: {str(e)}")
        sys.exit(1)
    finally:
        cleanup(tunnel_process)

if __name__ == "__main__":
    main() 