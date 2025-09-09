use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json::{json, Value};
use tracing::{error, info};

#[derive(Clone)]
pub struct DataDogMetrics {
    client: Client,
    api_key: Option<String>,
    enabled: bool,
    dd_environment: String,
}

impl DataDogMetrics {
    pub fn new(api_key: Option<String>, enabled: bool, dd_environment: String) -> Self {
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
            dd_environment,
        }
    }

    /// Check if metrics are enabled
    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Helper method to validate if metrics should be sent
    fn should_send_metrics(&self) -> Option<&String> {
        if !self.enabled {
            return None;
        }
        self.api_key.as_ref()
    }

    /// Helper method to create a single metric series
    fn create_metric_series(&self, metric_name: &str, value: f64, timestamp: f64) -> Value {
        json!({
            "metric": metric_name,
            "points": [[timestamp, value]],
            "tags": [
                "service:near-indexer",
                format!("env:{}", self.dd_environment)
            ]
        })
    }

    /// Helper method to create a payload with multiple metrics
    fn create_payload(&self, metrics: Vec<Value>) -> Value {
        json!({
            "series": metrics
        })
    }

    /// Helper method to get current timestamp
    fn get_current_timestamp(&self) -> f64 {
        Utc::now().timestamp() as f64
    }

    pub async fn send_block_metrics(&self, _block_height: u64, block_timestamp: DateTime<Utc>) {
        let Some(api_key) = self.should_send_metrics() else {
            return;
        };

        let now = self.get_current_timestamp();
        let block_timestamp_unix = block_timestamp.timestamp() as f64;
        let block_age_seconds = now - block_timestamp_unix;
        
        let metric = self.create_metric_series("near.indexer.block_age_seconds", block_age_seconds, now);
        let payload = self.create_payload(vec![metric]);

        // Use the existing send_metrics_payload for consistency
        self.send_metrics_payload(api_key, payload, "block metrics").await;
    }

    /// Send indexing speed metrics (blocks processed per second)
    pub async fn send_indexing_speed_metrics(&self, blocks_per_second: f64) {
        let Some(api_key) = self.should_send_metrics() else {
            return;
        };

        let now = self.get_current_timestamp();
        let metric = self.create_metric_series("near.indexer.blocks_per_second", blocks_per_second, now);
        let payload = self.create_payload(vec![metric]);

        self.send_metrics_payload(api_key, payload, "indexing speed").await;
    }

    /// Send database performance metrics
    pub async fn send_database_metrics(
        &self,
        query_execution_time_ms: f64,
        connection_pool_size: u32,
        active_connections: u32,
        _idle_connections: u32,
    ) {
        let Some(api_key) = self.should_send_metrics() else {
            return;
        };

        let now = self.get_current_timestamp();
        
        // Calculate connection pool utilization percentage
        let pool_utilization = if connection_pool_size > 0 {
            (active_connections as f64 / connection_pool_size as f64) * 100.0
        } else {
            0.0
        };

        let metrics = vec![
            self.create_metric_series("near.indexer.db.query_execution_time_ms", query_execution_time_ms, now),
            self.create_metric_series("near.indexer.db.pool_utilization_percent", pool_utilization, now),
        ];
        
        let payload = self.create_payload(metrics);
        self.send_metrics_payload(api_key, payload, "database performance").await;
    }

    /// Helper method to send metrics payload to DataDog
    async fn send_metrics_payload(&self, api_key: &str, payload: Value, metric_type: &str) {
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
                    info!("Successfully sent DataDog {} metrics", metric_type);
                } else {
                    error!(
                        "DataDog API returned error for {} metrics: {} - {}",
                        metric_type,
                        response.status(),
                        response.text().await.unwrap_or_else(|_| "Unknown error".to_string())
                    );
                }
            }
            Err(e) => {
                error!(
                    "Failed to send DataDog {} metrics: {}",
                    metric_type, e
                );
            }
        }
    }
} 