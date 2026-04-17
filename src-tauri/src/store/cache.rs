use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

pub struct TtlCache<V: Clone> {
    store: RwLock<HashMap<String, (V, Instant)>>,
}

impl<V: Clone> Default for TtlCache<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: Clone> TtlCache<V> {
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<V> {
        // Fast path: read-only check.
        {
            let guard = self.store.read().ok()?;
            if let Some((value, expires_at)) = guard.get(key) {
                if Instant::now() <= *expires_at {
                    return Some(value.clone());
                }
            } else {
                return None;
            }
        }
        // Expired — upgrade to write lock and evict.
        if let Ok(mut guard) = self.store.write() {
            if let Some((_, expires_at)) = guard.get(key) {
                if Instant::now() > *expires_at {
                    guard.remove(key);
                }
            }
        }
        None
    }

    pub fn set(&self, key: &str, value: V, ttl: Duration) {
        if let Ok(mut guard) = self.store.write() {
            guard.insert(key.to_string(), (value, Instant::now() + ttl));
        }
    }

    pub fn invalidate(&self, key: &str) {
        if let Ok(mut guard) = self.store.write() {
            guard.remove(key);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut guard) = self.store.write() {
            guard.clear();
        }
    }
}
