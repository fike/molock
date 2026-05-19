// SPDX-FileCopyrightText: 2026 Molock Team
// SPDX-License-Identifier: Apache-2.0

use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct StateManager {
    counters: Arc<DashMap<String, CounterState>>,
    ttl: Duration,
}

#[derive(Debug, Clone)]
struct CounterState {
    count: u64,
    last_updated: Instant,
}

impl StateManager {
    /// Creates a new `StateManager` with a 1-hour TTL.
    #[must_use]
    pub fn new() -> Self {
        Self::with_ttl(Duration::from_hours(1))
    }

    /// Creates a new `StateManager` with a custom TTL.
    #[must_use]
    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            counters: Arc::new(DashMap::new()),
            ttl,
        }
    }

    /// Increments the request count for a given key.
    #[must_use]
    pub fn increment_count(&self, key: &str) -> u64 {
        self.cleanup_expired();

        let mut entry = self
            .counters
            .entry(key.to_string())
            .or_insert_with(|| CounterState {
                count: 0,
                last_updated: Instant::now(),
            });

        entry.count += 1;
        entry.last_updated = Instant::now();
        entry.count
    }

    /// Returns the request count for a given key.
    #[must_use]
    pub fn get_count(&self, key: &str) -> u64 {
        self.cleanup_expired();

        self.counters.get(key).map_or(0, |entry| entry.count)
    }

    /// Removes expired entries from the state.
    pub fn cleanup_expired(&self) {
        let now = Instant::now();
        let expired_keys: Vec<String> = self
            .counters
            .iter()
            .filter(|entry| now.duration_since(entry.last_updated) > self.ttl)
            .map(|entry| entry.key().clone())
            .collect();

        for key in expired_keys {
            self.counters.remove(&key);
        }
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_increment_and_get_count() {
        let manager = StateManager::new();

        assert_eq!(manager.get_count("test"), 0);
        assert_eq!(manager.increment_count("test"), 1);
        assert_eq!(manager.increment_count("test"), 2);
        assert_eq!(manager.get_count("test"), 2);
    }

    #[test]
    fn test_cleanup_expired() {
        let manager = StateManager::with_ttl(Duration::from_millis(100));

        let _ = manager.increment_count("test1");
        let _ = manager.increment_count("test2");

        assert_eq!(manager.get_count("test1"), 1);
        assert_eq!(manager.get_count("test2"), 1);

        thread::sleep(Duration::from_millis(150));

        manager.cleanup_expired();

        assert_eq!(manager.get_count("test1"), 0);
        assert_eq!(manager.get_count("test2"), 0);
    }

    #[test]
    fn test_multiple_keys() {
        let manager = StateManager::new();

        let _ = manager.increment_count("key1");
        let _ = manager.increment_count("key2");
        let _ = manager.increment_count("key1");

        assert_eq!(manager.get_count("key1"), 2);
        assert_eq!(manager.get_count("key2"), 1);
    }

    #[test]
    fn test_concurrent_access() {
        let manager = StateManager::new();
        let mut handles = Vec::new();

        for i in 0..10 {
            let manager = manager.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    let _ = manager.increment_count(&format!("key{}", i % 3));
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert!(
            manager.get_count("key0") > 0
                || manager.get_count("key1") > 0
                || manager.get_count("key2") > 0
        );
    }
}
