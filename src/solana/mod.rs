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

            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }

    pub async fn contiguously_get_confirmed_blocks(&mut self) -> anyhow::Result<(), SolanaError> {
        let original_last_slot = {
            let guard = self.last_confirmed_slot.lock().await;
            *guard
        };
        let mut last_slot = original_last_slot;

        loop {
            println!("Cache size: {}", self.confirmed_blocks.len().await);
            // Check if the shared last_confirmed_slot is still None.
            if self.last_confirmed_slot.lock().await.is_none() {
                println!("Is none!");
                continue;
            }

            let slot = self.last_confirmed_slot.lock().await.unwrap();

            let start_slot = if slot < 10 { 0 } else { slot - 10 };
            let end_slot = slot - 1;

            // get the next confirmed block
            let confirmed_blocks = self
                .inner
                .get_blocks(start_slot, Some(end_slot))
                .map_err(|e| SolanaError::RpcError(format!("Failed to get blocks: {}", e)))?;

            println!("Confirmed blocks: {:?}", confirmed_blocks);

            last_slot = Some(slot - 10);

            {
                let mut guard = self.last_confirmed_slot.lock().await;
                *guard = last_slot;
            }

            // insert the confirmed block into the cache
            for slot in confirmed_blocks.iter() {
                if !self.confirmed_blocks.contains(slot).await {
                    println!("Inserting slot {} into cache.", slot);

                    self.confirmed_blocks
                        .insert(*slot, *slot)
                        .await
                        .unwrap_or_else(|_| {
                            println!("Failed to insert {} into the cache.", slot);
                        });
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        }
    }

    pub async fn is_slot_confirmed(&self, slot: Slot) -> bool {
        self.inner.get_blocks(slot, Some(slot)).is_ok()
            && self.confirmed_blocks.contains(&slot).await
    }
}
