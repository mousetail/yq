use std::{hash::Hash, sync::Arc};

use dashmap::DashMap;
use tokio::sync::OnceCell;

// todo: Evaluate replacing with `OnceMap` from the UV project:
// https://github.com/astral-sh/uv/blob/main/crates/once-map/src/lib.rs
pub struct CacheMap<K: Hash + Eq, V> {
    inner: DashMap<K, Arc<OnceCell<V>>>,
}

impl<K: Hash + Eq, V> CacheMap<K, V> {
    pub fn new() -> Self {
        CacheMap {
            inner: DashMap::new(),
        }
    }

    pub fn get(&self, key: K) -> Arc<OnceCell<V>> {
        let entry = self.inner.entry(key).or_default().clone();
        entry
    }
}

impl<K: Hash + Eq, V> FromIterator<(K, V)> for CacheMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        CacheMap {
            inner: iter
                .into_iter()
                .map(|(k, v)| (k, Arc::new(OnceCell::new_with(Some(v)))))
                .collect(),
        }
    }
}