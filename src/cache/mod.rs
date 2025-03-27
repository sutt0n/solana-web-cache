use std::sync::Arc;

use scc::{HashMap, hash_map::OccupiedEntry};
use solana_sdk::clock::Slot;

#[derive(Clone, Debug)]
pub struct Cache {
    inner: Arc<HashMap<Slot, Slot>>,
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

    pub async fn contains(&self, key: &Slot) -> bool {
        self.inner.contains_async(key).await
    }

    pub async fn insert(&self, key: Slot, value: Slot) -> anyhow::Result<()> {
        // evict smallest Slot (u64) key if we exceed max_size
        if self.inner.len() >= self.max_size {
            let first_entry = self.inner.first_entry_async().await.unwrap();

            println!("Removing oldest entry: {:?}", first_entry.key());

            self.inner
                .remove_async(first_entry.key())
                .await
                .unwrap_or_else(|| {
                    println!("Failed to remove entry: {:?}", first_entry.key());
                    (first_entry.key().clone(), first_entry.get().clone())
                });
        }

        let r = self
            .inner
            .insert_async(key, value)
            .await
            .unwrap_or_else(|e| {
                println!("Failed to insert entry: {:?}", e);
                ()
            });

        r
    }

    pub async fn get(&self, key: &Slot) -> Option<OccupiedEntry<'_, Slot, Slot>> {
        self.inner.get_async(key).await
    }
}
