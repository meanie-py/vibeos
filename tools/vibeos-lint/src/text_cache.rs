//! Bound cache ownership of masked source text. Callers keep independent Arcs;
//! eviction releases only our reference, never a view still in use by a rule.
//! Use Arc<String>, not Arc<str>: weak slots must not pin the text allocation
//! after the last strong reference is dropped.
use lru_slab::LruSlab;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub(crate) const DEFAULT_BYTE_LIMIT: usize = 128 * 1024 * 1024;

struct State {
    entries: LruSlab<Arc<String>>,
    slots: HashMap<usize, u32>,
    bytes: usize,
    limit: usize,
}

pub(crate) struct Cache(Mutex<State>);

impl Cache {
    pub(crate) fn new(limit: usize) -> Self {
        Self(Mutex::new(State {
            entries: LruSlab::new(),
            slots: HashMap::new(),
            bytes: 0,
            limit,
        }))
    }

    pub(crate) fn retain(&self, text: &Arc<String>) {
        let mut discarded = Vec::new();
        {
            let mut state = self.0.lock().expect("text cache poisoned");
            // Allocation addresses are unique while retained. Weak per-view
            // slots retrieve the text; this index only tracks LRU ownership.
            let key = Arc::as_ptr(text) as *const () as usize;
            if let Some(&slot) = state.slots.get(&key) {
                state.entries.get_mut(slot);
                return;
            }
            if text.capacity() > state.limit || state.limit == 0 {
                return;
            }
            while text.capacity() > state.limit - state.bytes {
                let slot = state.entries.lru().expect("nonempty text cache");
                let old = state.entries.remove(slot);
                state
                    .slots
                    .remove(&(Arc::as_ptr(&old) as *const () as usize));
                state.bytes -= old.capacity();
                discarded.push(old);
            }
            let slot = state.entries.insert(text.clone());
            state.slots.insert(key, slot);
            state.bytes += text.capacity();
        }
        drop(discarded);
    }

    pub(crate) fn stats(&self) -> serde_json::Value {
        let state = self.0.lock().expect("text cache poisoned");
        serde_json::json!({"byte_limit": state.limit, "retained_bytes": state.bytes,
            "retained_views": state.entries.len()})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eviction_preserves_live_views_and_refreshes_recency() {
        let cache = Cache::new(8);
        let a: Arc<String> = Arc::new("aaaa".into());
        let b: Arc<String> = Arc::new("bbbb".into());
        let weak_b = Arc::downgrade(&b);
        cache.retain(&a);
        cache.retain(&b);
        drop(b);
        cache.retain(&a);
        let c: Arc<String> = Arc::new("cccc".into());
        cache.retain(&c);
        assert!(weak_b.upgrade().is_none());
        assert_eq!(&*a, "aaaa");
        assert_eq!(cache.stats()["retained_bytes"], 8);
        let oversized: Arc<String> = Arc::new("123456789".into());
        cache.retain(&oversized);
        assert_eq!(cache.stats()["retained_bytes"], 8);
        let weak_c = Arc::downgrade(&c);
        drop(c);
        drop(cache);
        assert!(weak_c.upgrade().is_none());
        assert_eq!(&*a, "aaaa");
    }
}
