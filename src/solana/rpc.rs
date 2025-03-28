use super::error::SolanaError;
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_sdk::clock::Slot;
use tokio_retry::Retry;
use tokio_retry::strategy::{ExponentialBackoff, jitter};
use tracing::error;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SolanaRpc {
    async fn get_slot(&self) -> Result<Slot, SolanaError>;
    async fn get_blocks(
        &self,
        start_slot: Slot,
        end_slot: Option<Slot>,
    ) -> Result<Vec<Slot>, SolanaError>;
}

#[async_trait]
impl SolanaRpc for RpcClient {
    async fn get_slot(&self) -> Result<Slot, SolanaError> {
        let retry_strategy = ExponentialBackoff::from_millis(100)
            .max_delay(std::time::Duration::from_secs(10))
            .map(jitter);

        let slot = Retry::spawn(retry_strategy, || async {
            self.get_slot().map_err(|err| {
                error!("RPC Error encountered: {}. Retrying...", err);
                err
            })
        })
        .await?;

        Ok(slot)
    }

    async fn get_blocks(
        &self,
        start_slot: Slot,
        end_slot: Option<Slot>,
    ) -> Result<Vec<Slot>, SolanaError> {
        let retry_strategy = ExponentialBackoff::from_millis(100)
            .max_delay(std::time::Duration::from_secs(10))
            .map(jitter);

        let blocks = Retry::spawn(retry_strategy, || async {
            self.get_blocks(start_slot, end_slot).map_err(|err| {
                error!("RPC Error encountered: {}. Retrying...", err);
                err
            })
        })
        .await?;

        Ok(blocks)
    }
}
