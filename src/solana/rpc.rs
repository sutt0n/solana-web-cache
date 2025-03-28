use super::error::SolanaError;
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_sdk::clock::Slot;

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
        self.get_slot()
            .map_err(|e| SolanaError::RpcError(e.to_string()))
    }

    async fn get_blocks(
        &self,
        start_slot: Slot,
        end_slot: Option<Slot>,
    ) -> Result<Vec<Slot>, SolanaError> {
        self.get_blocks(start_slot, end_slot)
            .map_err(|e| SolanaError::RpcError(e.to_string()))
    }
}
