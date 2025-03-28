use std::sync::Arc;

use scc::{HashMap, hash_map::OccupiedEntry};

#[derive(Clone, Debug)]
pub struct Cache {
    inner: Arc<HashMap<u64, u64>>,
    max_size: usize,
}

impl Cache {
    pub fn new(max_size: usize) -> Self {
        Cache { inner: Arc::new(HashMap::new()), max_size }
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

    pub async fn keys(&self) -> Vec<u64> {
        let mut keys = Vec::new();

        let _ = async {
            let mut iter = self.inner.first_entry_async().await;
            while let Some(entry) = iter {
                keys.push(*entry.key());
                iter = entry.next_async().await;
            }
        }
        .await;

        keys
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cache() {
        let cache = Cache::new(2);
        assert_eq!(cache.len().await, 0);

        cache.insert(1, 10).await.unwrap();
        assert!(cache.contains(&1).await);

        cache.insert(2, 20).await.unwrap();
        assert!(cache.contains(&2).await);

        // eviction
        cache.insert(3, 30).await.unwrap();
        assert!(cache.contains(&2).await);
        assert!(cache.contains(&3).await);
        assert!(!cache.contains(&1).await);
    }

    #[tokio::test]
    async fn test_keys() {
        let cache = Cache::new(3);
        cache.insert(1, 10).await.unwrap();
        cache.insert(2, 20).await.unwrap();
        cache.insert(3, 30).await.unwrap();

        let keys = cache.keys().await;
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&1));
        assert!(keys.contains(&2));
        assert!(keys.contains(&3));
    }
}
