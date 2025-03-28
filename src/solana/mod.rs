pub mod error;
pub mod rpc;

use crate::cache::Cache;
use anyhow::Result;
use async_trait::async_trait;
use error::SolanaError;
use rpc::SolanaRpc;
use solana_client::rpc_client::RpcClient;
use solana_sdk::clock::Slot;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SolanaClientTrait: Send + Sync {
    async fn poll_for_latest_slot(&mut self) -> Result<(), SolanaError>;
    async fn contiguously_get_confirmed_blocks(
        &mut self,
        chunk_size: usize,
    ) -> Result<(), SolanaError>;
    async fn is_slot_confirmed(&self, slot: Slot) -> bool;
}

#[derive(Clone)]
pub struct SolanaClient {
    inner: Arc<dyn SolanaRpc + Send + Sync>,
    confirmed_blocks: Arc<Cache>,
    last_confirmed_slot: Arc<Mutex<Option<Slot>>>,
}

static SOLANA_DEVNET: &str = "https://solana-mainnet.api.syndica.io/api-key/232m5n6PA1xpfTEwbqiGiBhrWzUCr1Jaj6vdD6cfp3PWyEQ5jnPGdxijJmHKUYLUKP4T4WV\
NM95kw157PsfyyBbqRDyrxtwykpG";

static SOLANA_GET_SLOT_THROTTLE_MS: u64 = 450;
static SOLANA_GET_BLOCKS_THROTTLE_MS: u64 = 150;

impl SolanaClient {
    pub async fn new(inner: Arc<dyn SolanaRpc + Send + Sync>, cache: Arc<Cache>) -> Self {
        Self {
            inner,
            confirmed_blocks: cache,
            last_confirmed_slot: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn init(cache: Arc<Cache>) -> Self {
        let rpc = Arc::new(RpcClient::new(SOLANA_DEVNET.to_string()));
        Self::new(rpc, cache).await
    }
}

#[async_trait]
impl SolanaClientTrait for SolanaClient {
    async fn poll_for_latest_slot(&mut self) -> Result<(), SolanaError> {
        loop {
            let latest_slot = self.inner.get_slot().await?;
            println!("Latest slot: {}", latest_slot);
            {
                let mut guard = self.last_confirmed_slot.lock().await;
                if let Some(current) = *guard {
                    if latest_slot > current {
                        *guard = Some(latest_slot);
                        println!("Updated latest confirmed slot to {}", latest_slot);
                    }
                } else {
                    *guard = Some(latest_slot);
                    println!("Initialized latest confirmed slot to {}", latest_slot);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(
                SOLANA_GET_SLOT_THROTTLE_MS,
            ))
            .await;
        }
    }

    async fn contiguously_get_confirmed_blocks(
        &mut self,
        chunk_size: usize,
    ) -> Result<(), SolanaError> {
        loop {
            println!("Cache size: {}", self.confirmed_blocks.len().await);
            if self.last_confirmed_slot.lock().await.is_none() {
                println!("No confirmed slot yet!");
                tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
                continue;
            }
            let slot = self.last_confirmed_slot.lock().await.unwrap();
            let start_slot = if slot < chunk_size as u64 {
                0
            } else {
                slot - chunk_size as u64
            };
            let end_slot = if slot == 0 { 0 } else { slot - 1 };

            let confirmed_blocks = self.inner.get_blocks(start_slot, Some(end_slot)).await?;

            println!("Confirmed blocks: {:?}", confirmed_blocks);

            {
                let mut guard = self.last_confirmed_slot.lock().await;
                *guard = Some(slot.saturating_sub(chunk_size as u64));
            }

            for slot in confirmed_blocks.iter() {
                if !self.confirmed_blocks.contains(slot).await {
                    println!("Inserting slot {} into cache.", slot);
                    if let Err(err) = self.confirmed_blocks.insert(*slot, *slot).await {
                        println!("Failed to insert {}: {:?}", slot, err);
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(
                SOLANA_GET_BLOCKS_THROTTLE_MS,
            ))
            .await;
        }
    }

    async fn is_slot_confirmed(&self, slot: Slot) -> bool {
        if self.confirmed_blocks.contains(&slot).await {
            return true;
        }

        self.inner
            .get_blocks(slot, Some(slot))
            .await.is_ok_and(|blocks| blocks.contains(&slot))
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::timeout;

    use super::*;
    use crate::{cache::Cache, solana::rpc::MockSolanaRpc};
    use std::{sync::Arc, time::Duration};

    #[tokio::test(flavor = "multi_thread")]
    async fn test_contiguously_get_confirmed_blocks() {
        let mut mock_rpc = MockSolanaRpc::new();

        mock_rpc.expect_get_slot().returning(|| Ok(10));

        mock_rpc
            .expect_get_blocks()
            .returning(|start_slot, end_slot| {
                let blocks = (start_slot..=end_slot.unwrap_or(start_slot)).collect();
                Ok(blocks)
            });

        let cache = Arc::new(Cache::new(1000));
        let rpc_arc: Arc<dyn SolanaRpc + Send + Sync> = Arc::new(mock_rpc);

        let mut solana_client = SolanaClient::new(rpc_arc, Arc::clone(&cache)).await;

        let _ = timeout(
            Duration::from_millis(500),
            solana_client.poll_for_latest_slot(),
        )
        .await;

        let result = timeout(
            Duration::from_millis(1000),
            solana_client.contiguously_get_confirmed_blocks(5),
        )
        .await;

        assert!(result.is_err());

        assert_eq!(cache.len().await, 10);
        assert!(cache.contains(&5).await);
        assert!(cache.contains(&6).await);
        assert!(cache.contains(&7).await);
        assert!(cache.contains(&8).await);
        assert!(cache.contains(&9).await);
    }
}
