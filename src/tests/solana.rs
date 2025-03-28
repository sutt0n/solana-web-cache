use anyhow::Result;
use async_trait::async_trait;
use solana_sdk::clock::Slot;

use crate::solana::{SolanaClient, error::SolanaError};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SolanaClientTrait: Send + Sync {
    async fn poll_for_latest_slot(&mut self) -> Result<(), SolanaError>;
    async fn contiguously_get_confirmed_blocks(
        &mut self,
        chunk_size: u64,
    ) -> Result<(), SolanaError>;
    async fn is_slot_confirmed(&self, slot: Slot) -> bool;
}

#[async_trait]
impl SolanaClientTrait for SolanaClient {
    async fn poll_for_latest_slot(&mut self) -> Result<(), SolanaError> {
        Self::poll_for_latest_slot(self).await
    }

    async fn contiguously_get_confirmed_blocks(
        &mut self,
        chunk_size: u64,
    ) -> Result<(), SolanaError> {
        Self::contiguously_get_confirmed_blocks(self, chunk_size).await
    }

    async fn is_slot_confirmed(&self, slot: Slot) -> bool {
        Self::is_slot_confirmed(self, slot).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_mock_is_slot_confirmed() {
        let mut mock_client = MockSolanaClientTrait::new();

        mock_client
            .expect_is_slot_confirmed()
            .with(eq(69))
            .times(1)
            .returning(|_| true);

        let confirmed = mock_client.is_slot_confirmed(69).await;
        assert!(confirmed);
    }

    #[tokio::test]
    async fn test_solana_client_poll_for_latest_slot() {
        let mut mock_client = MockSolanaClientTrait::new();

        mock_client
            .expect_poll_for_latest_slot()
            .times(1)
            .returning(|| Ok(()));

        let result = mock_client.poll_for_latest_slot().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_solana_client_contiguously_get_confirmed_blocks() {
        let mut mock_client = MockSolanaClientTrait::new();

        // mock inner rpc client
        mock_client
            .expect_contiguously_get_confirmed_blocks()
            .with(eq(10))
            .times(1)
            .returning(|_| Ok(()));

        let result = mock_client.contiguously_get_confirmed_blocks(10).await;
        assert!(result.is_ok());
    }
}
