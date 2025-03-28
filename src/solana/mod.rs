pub mod error;

use error::SolanaError;
use solana_sdk::clock::Slot;
use std::sync::Arc;
use tokio::sync::Mutex;

use solana_client::rpc_client::RpcClient;

use crate::cache::Cache;

#[derive(Clone)]
pub struct SolanaClient {
    inner: Arc<RpcClient>,
    confirmed_blocks: Arc<Cache>,
    last_confirmed_slot: Arc<Mutex<Option<Slot>>>,
}

static SOLANA_DEVNET: &str = "https://solana-mainnet.api.syndica.io/api-key/232m5n6PA1xpfTEwbqiGiBhrWzUCr1Jaj6vdD6cfp3PWyEQ5jnPGdxijJmHKUYLUKP4T4WV
NM95kw157PsfyyBbqRDyrxtwykpG";

static SOLANA_BLOCK_THROTTLE_MS: u64 = 450;
static SOLANA_CACHE_THROTTLE_MS: u64 = 150;

impl SolanaClient {
    pub async fn init(cache: &Arc<Cache>) -> anyhow::Result<Self, SolanaError> {
        let inner = RpcClient::new(SOLANA_DEVNET.to_string());

        let inner = Arc::new(inner);

        Ok(Self {
            inner,
            confirmed_blocks: Arc::clone(cache),
            last_confirmed_slot: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn poll_for_latest_slot(&mut self) -> anyhow::Result<(), SolanaError> {
        loop {
            let latest_slot = self
                .inner
                .get_slot()
                .map_err(|e| SolanaError::RpcError(format!("Failed to get latest slot: {}", e)))?;

            println!("Latest slot: {}", latest_slot);

            // force scope to avoid deadlock
            {
                let mut last_slot_guard = self.last_confirmed_slot.lock().await;
                if let Some(last_slot) = *last_slot_guard {
                    if latest_slot > last_slot {
                        *last_slot_guard = Some(latest_slot);
                        println!("Latest confirmed slot updated to {}", latest_slot);
                    }
                } else {
                    *last_slot_guard = Some(latest_slot);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(SOLANA_BLOCK_THROTTLE_MS)).await;
        }
    }

    pub async fn get_slot(&self) -> anyhow::Result<Slot, SolanaError> {
        self.inner
            .get_slot()
            .map_err(|e| SolanaError::RpcError(format!("Failed to get slot: {}", e)))
    }

    pub async fn get_blocks(
        &self,
        start_slot: Slot,
        end_slot: Option<Slot>,
    ) -> anyhow::Result<Vec<Slot>, SolanaError> {
        self.inner
            .get_blocks(start_slot, end_slot)
            .map_err(|e| SolanaError::RpcError(format!("Failed to get blocks: {}", e)))
    }

    pub async fn contiguously_get_confirmed_blocks(
        &mut self,
        chunk_size: u64,
    ) -> anyhow::Result<(), SolanaError> {
        loop {
            println!("Cache size: {}", self.confirmed_blocks.len().await);
            if self.last_confirmed_slot.lock().await.is_none() {
                println!("Is none!");
                continue;
            }

            let slot = self.last_confirmed_slot.lock().await.unwrap();

            let start_slot = if slot < chunk_size {
                0
            } else {
                slot - chunk_size
            };
            let end_slot = slot - 1;

            let confirmed_blocks = self
                .inner
                .get_blocks(start_slot, Some(end_slot))
                .map_err(|e| SolanaError::RpcError(format!("Failed to get blocks: {}", e)))?;

            println!("Confirmed blocks: {:?}", confirmed_blocks);

            {
                let mut guard = self.last_confirmed_slot.lock().await;
                *guard = Some(slot - chunk_size);
            }

            // insert the confirmed block into the cache
            for slot in confirmed_blocks.iter() {
                if !self.confirmed_blocks.contains(slot).await {
                    println!("Inserting slot {} into cache.", slot);

                    if let Err(err) = self.confirmed_blocks.insert(*slot, *slot).await {
                        println!("Failed to insert {} into the cache: {:?}", slot, err);
                    }
                }
            }

            // 150ms for throttling
            tokio::time::sleep(tokio::time::Duration::from_millis(SOLANA_CACHE_THROTTLE_MS)).await;
        }
    }

    pub async fn is_slot_confirmed(&self, slot: Slot) -> bool {
        self.inner
            .get_blocks(slot, Some(slot))
            .map(|blocks| !blocks.is_empty() && blocks[0] == slot)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::Cache;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_solana_client_poll_for_latest_slot() {
        let cache = Arc::new(Cache::new(10));
        let mut client = SolanaClient::init(&cache).await.unwrap();

        // Mock the poll_for_latest_slot method to simulate behavior.
        client.poll_for_latest_slot().await.unwrap();
    }

    #[tokio::test]
    async fn test_solana_client_contiguously_get_confirmed_blocks() {
        let cache = Arc::new(Cache::new(10));
        let mut client = SolanaClient::init(&cache).await.unwrap();

        // Mock the contiguously_get_confirmed_blocks method to simulate behavior.
        client.contiguously_get_confirmed_blocks(5).await.unwrap();
    }
}
