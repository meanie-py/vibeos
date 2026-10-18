//! Weighted LRU retention of immutable parsed trees. Live callers hold their own
//! Arcs, so eviction never invalidates nodes or forces duplicate concurrent parses.
use lru_slab::LruSlab;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tree_sitter::Tree;

pub(crate) const DEFAULT_NODE_LIMIT: usize = 4_000_000;

struct Entry {
    file: usize,
    nodes: usize,
    tree: Arc<Tree>,
}

struct State {
    entries: LruSlab<Entry>,
    files: HashMap<usize, u32>,
    limit: usize,
    nodes: usize,
    parses: usize,
    shared_hits: usize,
    seen: HashMap<usize, bool>,
}

pub(crate) struct Cache(Mutex<State>);

impl Cache {
    pub(crate) fn new(limit: usize) -> Self {
        Self(Mutex::new(State {
            entries: LruSlab::new(),
            files: HashMap::new(),
            limit,
            nodes: 0,
            parses: 0,
            shared_hits: 0,
            seen: HashMap::new(),
        }))
    }

    pub(crate) fn retain(
        &self,
        file: usize,
        tree: &Arc<Tree>,
        parsed: bool,
        retention_allowed: bool,
    ) {
        let mut discarded = Vec::new();
        {
            let mut state = self.0.lock().expect("syntax retention cache poisoned");
            if parsed {
                state.parses += 1;
            } else {
                state.shared_hits += 1;
            }
            // One-off syntax consumers should not displace trees shared by
            // several rules. Record first use without retaining its allocation.
            match state.seen.entry(file) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(false);
                    return;
                }
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    *entry.get_mut() = true;
                }
            }
            if !retention_allowed {
                return;
            }
            if let Some(&slot) = state.files.get(&file) {
                state.entries.get_mut(slot);
                return;
            }
            let nodes = tree.root_node().descendant_count();
            if nodes > state.limit {
                return;
            }
            while state.nodes + nodes > state.limit {
                let slot = state.entries.lru().expect("nonempty weighted cache");
                let entry = state.entries.remove(slot);
                state.files.remove(&entry.file);
                state.nodes -= entry.nodes;
                discarded.push(entry.tree);
            }
            let slot = state.entries.insert(Entry {
                file,
                nodes,
                tree: tree.clone(),
            });
            state.files.insert(file, slot);
            state.nodes += nodes;
        }
        // Tree destructors can be expensive. Never free them while holding the
        // global cache lock, which would stall unrelated syntax consumers.
        drop(discarded);
    }

    pub(crate) fn set_limit(&self, limit: usize) {
        let mut discarded = Vec::new();
        {
            let mut state = self.0.lock().expect("syntax retention cache poisoned");
            state.limit = limit;
            while state.nodes > limit {
                let slot = state.entries.lru().expect("nonempty weighted cache");
                let entry = state.entries.remove(slot);
                state.files.remove(&entry.file);
                state.nodes -= entry.nodes;
                discarded.push(entry.tree);
            }
        }
        drop(discarded);
    }

    pub(crate) fn stats(&self) -> serde_json::Value {
        let state = self.0.lock().expect("syntax retention cache poisoned");
        serde_json::json!({
            "node_limit": state.limit, "retained_nodes": state.nodes,
            "retained_trees": state.entries.len(), "successful_parses": state.parses,
            "shared_tree_hits": state.shared_hits,
            "unique_tree_files": state.seen.len(),
            "reused_tree_files": state.seen.values().filter(|reused| **reused).count(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> Arc<Tree> {
        Arc::new(crate::syntax::parse("fn item() {}").unwrap())
    }

    #[test]
    fn weighted_eviction_keeps_recent_trees_and_active_consumers_valid() {
        let a = tree();
        let nodes = a.root_node().descendant_count();
        let cache = Cache::new(nodes * 2);
        let b = tree();
        let weak_b = Arc::downgrade(&b);
        cache.retain(0, &a, true, true);
        cache.retain(0, &a, false, true);
        cache.retain(1, &b, true, true);
        cache.retain(1, &b, false, true);
        drop(b);
        cache.retain(0, &a, false, true);
        let c = tree();
        cache.retain(2, &c, true, true);
        cache.retain(2, &c, false, true);
        assert!(weak_b.upgrade().is_none());
        assert_eq!(a.root_node().kind(), "source_file");
        assert_eq!(cache.stats()["retained_nodes"], nodes * 2);
        cache.set_limit(0);
        assert_eq!(cache.stats()["retained_nodes"], 0);
        assert_eq!(a.root_node().descendant_count(), nodes);
        assert_eq!(c.root_node().descendant_count(), nodes);
    }

    #[test]
    fn oversized_trees_do_not_flush_useful_smaller_entries() {
        let small = tree();
        let nodes = small.root_node().descendant_count();
        let cache = Cache::new(nodes);
        let weak = Arc::downgrade(&small);
        cache.retain(0, &small, true, true);
        cache.retain(0, &small, false, true);
        drop(small);
        let large = Arc::new(crate::syntax::parse("fn a() {} fn b() {}").unwrap());
        assert!(large.root_node().descendant_count() > nodes);
        cache.retain(1, &large, true, true);
        cache.retain(1, &large, false, true);
        assert!(weak.upgrade().is_some());
        assert_eq!(cache.stats()["retained_trees"], 1);
    }

    #[test]
    fn one_use_trees_are_not_retained_but_second_use_is_admitted() {
        let cache = Cache::new(1000);
        let first = tree();
        let weak = Arc::downgrade(&first);
        cache.retain(0, &first, true, true);
        drop(first);
        assert!(weak.upgrade().is_none());
        let second = tree();
        let weak = Arc::downgrade(&second);
        cache.retain(0, &second, true, true);
        drop(second);
        assert!(weak.upgrade().is_some());
        assert_eq!(cache.stats()["reused_tree_files"], 1);
    }
}
