/*
 * Copyright 2026 Molock Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct StateManager {
    counters: Arc<DashMap<String, CounterState>>,
    ttl: Duration,
}

struct CounterState {
    count: u64,
    last_updated: Instant,
}

impl StateManager {
    pub fn new() -> Self {
        Self::with_ttl(Duration::from_secs(3600)) // 1 hour default TTL
    }

    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            counters: Arc::new(DashMap::new()),
            ttl,
        }
    }

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

    pub fn get_count(&self, key: &str) -> u64 {
        self.cleanup_expired();

        self.counters.get(key).map(|entry| entry.count).unwrap_or(0)
    }

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
        assert_eq!(manager.get_count("test"), 1);

        assert_eq!(manager.increment_count("test"), 2);
        assert_eq!(manager.get_count("test"), 2);
    }

    #[test]
    fn test_multiple_keys() {
        let manager = StateManager::new();

        assert_eq!(manager.increment_count("key1"), 1);
        assert_eq!(manager.increment_count("key2"), 1);
        assert_eq!(manager.increment_count("key1"), 2);

        assert_eq!(manager.get_count("key1"), 2);
        assert_eq!(manager.get_count("key2"), 1);
    }

    #[test]
    fn test_cleanup_expired() {
        let manager = StateManager::with_ttl(Duration::from_millis(100));

        manager.increment_count("test1");
        manager.increment_count("test2");

        assert_eq!(manager.get_count("test1"), 1);
        assert_eq!(manager.get_count("test2"), 1);

        thread::sleep(Duration::from_millis(150));

        manager.cleanup_expired();

        assert_eq!(manager.get_count("test1"), 0);
        assert_eq!(manager.get_count("test2"), 0);
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let manager = Arc::new(StateManager::new());
        let mut handles = vec![];

        for i in 0..10 {
            let manager = manager.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    manager.increment_count(&format!("key{}", i % 3));
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
