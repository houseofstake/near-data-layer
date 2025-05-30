import os
import yaml
import logging
from typing import Dict, List
import psycopg2
from google.cloud import scheduler_v1
from dotenv import load_dotenv

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class MaterializedViewManager:
    def __init__(self):
        load_dotenv()
        self.db_params = {
            'dbname': os.getenv('DB_NAME'),
            'user': os.getenv('DB_USER'),
            'password': os.getenv('DB_PASSWORD'),
            'host': os.getenv('DB_HOST'),
            'port': os.getenv('DB_PORT', '5432')
        }
        self.project_id = os.getenv('GCP_PROJECT_ID')
        self.location_id = os.getenv('GCP_LOCATION_ID', 'us-central1')
        
    def get_db_connection(self):
        """Establish database connection"""
        try:
            return psycopg2.connect(**self.db_params)
        except Exception as e:
            logger.error(f"Failed to connect to database: {e}")
            raise

    def load_view_config(self, config_path: str) -> Dict:
        """Load view configuration from YAML file"""
        try:
            with open(config_path, 'r') as f:
                return yaml.safe_load(f)
        except Exception as e:
            logger.error(f"Failed to load config from {config_path}: {e}")
            raise

    def drop_materialized_view(self, view_name: str, conn) -> None:
        """Drop the materialized view if it exists"""
        try:
            with conn.cursor() as cur:
                cur.execute(f"DROP MATERIALIZED VIEW IF EXISTS {view_name} CASCADE")
                conn.commit()
                logger.info(f"Dropped materialized view: {view_name}")
        except Exception as e:
            logger.error(f"Failed to drop view {view_name}: {e}")
            conn.rollback()
            raise

    def create_materialized_view(self, view_name: str, query: str, conn) -> None:
        """Create the materialized view"""
        try:
            with conn.cursor() as cur:
                create_view_sql = f"CREATE MATERIALIZED VIEW {view_name} AS {query}"
                cur.execute(create_view_sql)
                conn.commit()
                logger.info(f"Created materialized view: {view_name}")
        except Exception as e:
            logger.error(f"Failed to create view {view_name}: {e}")
            conn.rollback()
            raise

    def create_indices(self, view_name: str, index_fields: List[Dict], conn) -> None:
        """Create indices on the materialized view"""
        try:
            with conn.cursor() as cur:
                for idx in index_fields:
                    index_name = f"{view_name}_{idx['name']}_idx"
                    index_type = idx.get('type', '').upper()
                    
                    if index_type == 'HASH':
                        index_sql = f"CREATE INDEX {index_name} ON {view_name} USING hash({idx['name']})"
                    else:
                        index_sql = f"CREATE INDEX {index_name} ON {view_name} ({idx['name']} {index_type})"
                    
                    cur.execute(index_sql)
                    conn.commit()
                    logger.info(f"Created index {index_name} on {view_name}")
        except Exception as e:
            logger.error(f"Failed to create indices for {view_name}: {e}")
            conn.rollback()
            raise

    def update_cron_schedule(self, view_config: Dict) -> None:
        """Update the Cloud Scheduler cron job"""
        if 'cron_schedule' not in view_config:
            return

        try:
            client = scheduler_v1.CloudSchedulerClient()
            parent = client.location_path(self.project_id, self.location_id)
            
            schedule_config = view_config['cron_schedule']
            job_name = f"{parent}/jobs/{schedule_config['name']}"
            
            # Create or update the job
            job = {
                'name': job_name,
                'schedule': schedule_config['schedule'],
                'time_zone': schedule_config.get('timezone', 'UTC'),
                'http_target': {
                    'uri': f"https://your-refresh-endpoint/{view_config['name']}",
                    'http_method': scheduler_v1.HttpMethod.POST,
                }
            }

            try:
                client.update_job(job=job)
                logger.info(f"Updated cron job: {job_name}")
            except Exception:
                client.create_job(parent=parent, job=job)
                logger.info(f"Created cron job: {job_name}")

        except Exception as e:
            logger.error(f"Failed to update cron schedule: {e}")
            raise

    def process_view(self, config_path: str) -> None:
        """Process a single view configuration"""
        view_config = self.load_view_config(config_path)
        view_name = view_config['name']
        
        conn = self.get_db_connection()
        try:
            # Execute all operations in order
            self.drop_materialized_view(view_name, conn)
            self.create_materialized_view(view_name, view_config['query'], conn)
            self.create_indices(view_name, view_config.get('index_fields', []), conn)
            self.update_cron_schedule(view_config)
            
            logger.info(f"Successfully processed view: {view_name}")
        finally:
            conn.close()

def main():
    """Main function to process all view configurations"""
    view_manager = MaterializedViewManager()
    config_dir = os.path.join(os.path.dirname(__file__), '..', 'config', 'views')
    
    for config_file in os.listdir(config_dir):
        if config_file.endswith('.yaml'):
            config_path = os.path.join(config_dir, config_file)
            try:
                view_manager.process_view(config_path)
            except Exception as e:
                logger.error(f"Failed to process {config_file}: {e}")

if __name__ == "__main__":
    main() 