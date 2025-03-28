use std::sync::Arc;

use scc::{HashMap, hash_map::OccupiedEntry};

#[derive(Clone, Debug)]
pub struct Cache {
    inner: Arc<HashMap<u64, u64>>,
    max_size: usize,
}

impl Cache {
    pub fn new(max_size: usize) -> Self {
        Cache {
            inner: Arc::new(HashMap::new()),
            max_size,
        }
    }

    pub async fn len(&self) -> usize {
        self.inner.len()
    }

    pub async fn is_max_size(&self) -> bool {
        self.inner.len() >= self.max_size
    }

    pub async fn contains(&self, key: &u64) -> bool {
        self.inner.contains_async(key).await
    }

    pub async fn insert(&self, key: u64, value: u64) -> anyhow::Result<(), (u64, u64)> {
        if self.inner.len() >= self.max_size {
            if let Some(first_entry) = self.inner.first_entry_async().await {
                let oldest_key = *first_entry.key();
                drop(first_entry);
                let _ = self.inner.remove_async(&oldest_key).await;
            }
        }

        self.inner.insert_async(key, value).await.map(|_| ())
    }

    pub async fn get(&self, key: &u64) -> Option<OccupiedEntry<'_, u64, u64>> {
        self.inner.get_async(key).await
    }
}
