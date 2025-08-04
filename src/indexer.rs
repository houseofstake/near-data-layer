use crate::config::Settings;
use crate::database::Database;
use crate::metrics::DataDogMetrics;
use crate::processor::Processor;
use anyhow::Result;
use fastnear_neardata_fetcher::fetcher;
use fastnear_primitives::block_with_tx_hash::BlockWithTxHashes;
use fastnear_primitives::types::ChainId;
use fastnear_primitives::near_primitives::types::{BlockHeight, Finality};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};

pub struct Indexer {
    settings: Settings,
    processor: Processor,
    is_running: Arc<AtomicBool>,
    datadog_metrics: DataDogMetrics,
}

impl Indexer {
    pub async fn new(settings: Settings) -> Result<Self> {
        let database = Database::new(settings.clone()).await?;
        let processor = Processor::new(database, settings.clone());
        let is_running = Arc::new(AtomicBool::new(true));
        let datadog_metrics = DataDogMetrics::new(
            settings.dd_api_key.clone(),
            settings.datadog_enabled,
            settings.dd_environment.clone(),
        );

        Ok(Self {
            settings,
            processor,
            is_running,
            datadog_metrics,
        })
    }

    pub async fn initialize(&self) -> Result<()> {
        self.processor.initialize_tables().await?;
        info!("Indexer initialized successfully");
        Ok(())
    }

    pub async fn start(&mut self, cli_start_block: Option<u64>) -> Result<()> {
        info!("Starting NEAR blockchain indexer");
        
        // Determine the starting block height
        let start_block_height = self.determine_start_block(cli_start_block).await?;
        info!("App version: {}", self.settings.app_version);
        info!("Starting from block: {}", start_block_height);
        info!("HOS contracts: {:?}", self.settings.get_hos_contracts());

        // Set up signal handling
        let ctrl_c_running = self.is_running.clone();
        ctrlc::set_handler(move || {
            ctrl_c_running.store(false, Ordering::SeqCst);
            info!("Received Ctrl+C, starting shutdown...");
        })
        .expect("Error setting Ctrl+C handler");

        // Build fetcher configuration
        let chain_id = ChainId::try_from(self.settings.api_chain_id.clone())
            .map_err(|e| anyhow::anyhow!("Invalid chain ID: {}", e))?;

        // Log configuration being used
        info!("API key length: {}", self.settings.api_auth_token.as_ref().map(|k| k.len()).unwrap_or(0));
        info!("Using chain ID: {}", self.settings.api_chain_id);
        info!("Using poll interval: {}s", self.settings.poll_interval);
        info!("Using retry delay: {}s", self.settings.retry_delay);
        info!("Using num_threads: {}", self.settings.num_threads);
        info!("Using environment: {}", self.settings.environment);
        info!("Using dd_environment: {}", self.settings.dd_environment);

        let fetcher_config = fetcher::FetcherConfig {
            num_threads: self.settings.num_threads,
            start_block_height: Some(BlockHeight::try_from(start_block_height).unwrap()),
            end_block_height: None,
            chain_id,
            timeout_duration: Some(std::time::Duration::from_secs(self.settings.poll_interval)),
            retry_duration: Some(std::time::Duration::from_secs(self.settings.retry_delay)),
            disable_archive_sync: false,
            auth_bearer_token: self.settings.api_auth_token.clone(),
            finality: Finality::Final,
            enable_r2_archive_sync: false,
            user_agent: None,
        };

        // Create channel for receiving blocks
        let (sender, receiver) = mpsc::channel(100);

        // Start the fetcher in a separate task
        let fetcher_task = tokio::spawn(fetcher::start_fetcher(
            fetcher_config,
            sender,
            self.is_running.clone(),
        ));

        // Process blocks
        self.process_blocks(receiver).await?;

        // Wait for fetcher to complete
        if let Err(e) = fetcher_task.await {
            error!("Fetcher task failed: {}", e);
        }

        info!("Indexer stopped");
        Ok(())
    }

    /// Determine the starting block height based on priority:
    /// 1. CLI argument (highest precedence)
    /// 2. Cursor from database for this app version
    /// 3. Config file (lowest precedence)
    async fn determine_start_block(&self, cli_start_block: Option<u64>) -> Result<u64> {
        // Priority 1: CLI argument (if provided)
        if let Some(start_block) = cli_start_block {
            info!(
                "CLI start block provided: {}, using CLI argument",
                start_block
            );
            return Ok(start_block);
        }
        
        // Priority 2: Check cursor in database for this app version
        if let Some(last_processed_block) = self.processor.get_cursor_for_app_version().await? {
            let resume_block = last_processed_block + 1;
            info!(
                "Resuming from block {} for app version '{}' (last processed: {})",
                resume_block, self.settings.app_version, last_processed_block
            );
            Ok(resume_block)
        } else {
            // Priority 3: No cursor found for this version, use config start block
            info!(
                "No cursor found for app version '{}', using config start block: {}",
                self.settings.app_version, self.settings.start_block
            );
            Ok(self.settings.start_block)
        }
    }

    async fn process_blocks(&self, mut receiver: mpsc::Receiver<BlockWithTxHashes>) -> Result<()> {
        let mut prev_block_hash = None;
        let mut processed_blocks = 0;
        let mut batch_start_block: Option<u64> = None;
        let mut last_block_height: Option<u64> = None;

        while let Some(block) = receiver.recv().await {
            if !self.is_running.load(Ordering::SeqCst) {
                break;
            }

            let block_height = block.block.header.height;
            processed_blocks += 1;
            if batch_start_block.is_none() {
                batch_start_block = Some(block_height);
            }
            last_block_height = Some(block_height);

            // Validate block chain
            let block_hash = block.block.header.hash.clone();
            if let Some(prev_hash) = &prev_block_hash {
                if prev_hash != &block.block.header.prev_hash.to_string() {
                    error!(
                        "Block hash mismatch at height {}: expected {}, got {}",
                        block_height, prev_hash, block.block.header.prev_hash.to_string()
                    );
                    continue;
                }
            }
            prev_block_hash = Some(block_hash.to_string());

            // Process the block
            if let Err(e) = self.processor.process_block(&block).await {
                error!("Failed to process block {}: {}", block_height, e);
                continue;
            }

            // Update cursor
            if let Err(e) = self
                .processor
                .update_cursor(&self.settings.app_version, block_height, &block_hash.to_string())
                .await
            {
                error!("Failed to update cursor for block {}: {}", block_height, e);
            }

            // Send DataDog metrics every 10 blocks
            if processed_blocks % 10 == 0 {
                let block_timestamp = {
                    let secs = (block.block.header.timestamp_nanosec / 1_000_000_000) as i64;
                    let nsecs = (block.block.header.timestamp_nanosec % 1_000_000_000) as u32;
                    chrono::DateTime::<chrono::Utc>::from_timestamp(secs, nsecs)
                        .unwrap_or_else(|| chrono::Utc::now())
                };
                
                self.datadog_metrics
                    .send_block_metrics(block_height, block_timestamp)
                    .await;
            }

            // Log every 1000 blocks
            if processed_blocks % 1000 == 0 {
                if let (Some(start), Some(end)) = (batch_start_block, last_block_height) {
                    info!("Processed and stored blocks {}-{}", start, end);
                }
                batch_start_block = None;
            }
        }
        // Log any remaining blocks at the end
        if let (Some(start), Some(end)) = (batch_start_block, last_block_height) {
            if processed_blocks % 1000 != 0 {
                info!("Processed and stored blocks {}-{}", start, end);
            }
        }
        info!("Block processing stopped");
        Ok(())
    }
} 