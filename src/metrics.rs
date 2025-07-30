use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json::json;
use tracing::{error, info};

pub struct DataDogMetrics {
    client: Client,
    api_key: Option<String>,
    enabled: bool,
    environment: String,
}

impl DataDogMetrics {
    pub fn new(api_key: Option<String>, enabled: bool, environment: String) -> Self {
        let client = Client::new();
        
        let has_api_key = api_key.is_some();
        
        if enabled && has_api_key {
            info!("DataDog metrics client initialized successfully");
        } else if enabled {
            error!("DataDog enabled but no API key provided");
        }

        Self {
            client,
            api_key,
            enabled: enabled && has_api_key,
            environment,
        }
    }

    pub async fn send_block_metrics(&self, block_height: u64, block_timestamp: DateTime<Utc>) {
        if !self.enabled {
            return;
        }

        let Some(api_key) = &self.api_key else {
            return;
        };

        let now = Utc::now().timestamp() as f64;
        
        // Convert block timestamp to Unix timestamp
        let block_timestamp_unix = block_timestamp.timestamp() as f64;

        // Calculate how old the block is (indexer lag)
        let block_age_seconds = now - block_timestamp_unix;
        
        // Create metrics payload for DataDog API - single metric for indexer health monitoring
        let payload = json!({
            "series": [
                {
                    "metric": "near.indexer.block_age_seconds",
                    "points": [[now, block_age_seconds]],
                    "tags": [
                        "service:near-indexer",
                        format!("env:{}", self.environment)
                    ]
                }
            ]
        });

        let url = "https://api.datadoghq.com/api/v1/series";
        
        match self
            .client
            .post(url)
            .header("DD-API-KEY", api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    info!(
                        "Successfully sent DataDog metrics for block {} (timestamp: {})",
                        block_height, block_timestamp
                    );
                } else {
                    error!(
                        "DataDog API returned error for block {}: {} - {}",
                        block_height,
                        response.status(),
                        response.text().await.unwrap_or_else(|_| "Unknown error".to_string())
                    );
                }
            }
            Err(e) => {
                error!(
                    "Failed to send DataDog metrics for block {}: {}",
                    block_height, e
                );
            }
        }
    }
} 