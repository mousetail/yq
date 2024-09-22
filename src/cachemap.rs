use std::{hash::Hash, sync::Arc};

use dashmap::DashMap;
use tokio::sync::OnceCell;

pub struct CacheMap<K: Hash + Eq, V> {
    inner: DashMap<K, Arc<OnceCell<V>>>,
}

impl<K: Hash + Eq, V> CacheMap<K, V> {
    pub fn new() -> Self {
        return CacheMap {
            inner: DashMap::new(),
        };
    }

    pub fn get(&self, key: K) -> Arc<OnceCell<V>> {
        let entry = self.inner.entry(key).or_default().clone();
        return entry;
    }
}

impl<K: Hash + Eq, V> FromIterator<(K, V)> for CacheMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        return CacheMap {
            inner: iter
                .into_iter()
                .map(|(k, v)| (k, Arc::new(OnceCell::new_with(Some(v)))))
                .collect(),
        };
    }
}
