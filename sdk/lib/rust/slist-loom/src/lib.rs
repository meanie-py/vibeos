//! Loom-based concurrency tests for the SList algorithm.
//!
//! This crate models the SList header state with `loom::sync::Mutex` to
//! allow Loom's model checker to explore all possible thread interleavings.
//! The actual LDXP/STXP atomics provide sequential consistency on ARM64,
//! so Mutex is a faithful abstraction.

#[cfg(test)]
mod tests {
    use loom::sync::{Arc, Mutex};
    use loom::thread;

    /// Simulated SLIST_HEADER: (alignment, region).
    /// We use a Mutex to model the atomicity of LDXP/STXP.
    type SListHeader = Arc<Mutex<(u64, u64)>>;

    /// Simulated SLIST_ENTRY: just holds a "next" pointer (u64).
    /// In the real kernel these are heap-allocated; here we use sentinel values.
    type EntryStore = Arc<Mutex<Vec<(u64, u64)>>>; // (entry_addr, next_ptr)

    fn region_to_ptr(region: u64) -> u64 {
        region & !0xF
    }

    fn unpack(alignment: u64) -> (u16, u64) {
        let depth = (alignment & 0xFFFF) as u16;
        let seq = alignment >> 16;
        (depth, seq)
    }

    fn pack(depth: u16, seq: u64, next: u64) -> (u64, u64) {
        let alignment = (depth as u64) | (seq << 16);
        (alignment, next)
    }

    /// Simulated push: CAS loop with Mutex.
    fn push(header: &SListHeader, entries: &EntryStore, entry_addr: u64) -> u64 {
        loop {
            let (old_a, old_r) = {
                let h = header.lock().unwrap();
                (*h).clone()
            };
            let (depth, seq) = unpack(old_a);
            let old_head = region_to_ptr(old_r);

            // entry->Next = old_head
            {
                let mut e = entries.lock().unwrap();
                e.push((entry_addr, old_head));
            }

            let (new_a, new_r) = pack(depth.wrapping_add(1), seq.wrapping_add(1), entry_addr);

            // CAS
            let mut h = header.lock().unwrap();
            if *h == (old_a, old_r) {
                *h = (new_a, new_r);
                return old_head;
            }
            // CAS failed — remove the entry we added and retry
            {
                let mut e = entries.lock().unwrap();
                if let Some(pos) = e.iter().rposition(|&(a, _)| a == entry_addr) {
                    e.remove(pos);
                }
            }
        }
    }

    /// Simulated pop: CAS loop with Mutex.
    fn pop(header: &SListHeader, entries: &EntryStore) -> u64 {
        loop {
            let (old_a, old_r) = {
                let h = header.lock().unwrap();
                (*h).clone()
            };
            let head_ptr = region_to_ptr(old_r);
            if head_ptr == 0 {
                return 0; // Empty
            }

            // Find entry->Next
            let next_of_head = {
                let e = entries.lock().unwrap();
                e.iter()
                    .rev()
                    .find(|&&(addr, _)| addr == head_ptr)
                    .map(|&(_, next)| next)
                    .unwrap_or(0)
            };

            let (depth, seq) = unpack(old_a);
            let (new_a, new_r) = pack(depth.wrapping_sub(1), seq.wrapping_add(1), next_of_head);

            let mut h = header.lock().unwrap();
            if *h == (old_a, old_r) {
                *h = (new_a, new_r);
                return head_ptr;
            }
        }
    }

    /// Simulated flush: CAS loop with Mutex.
    fn flush(header: &SListHeader) -> u64 {
        loop {
            let (old_a, old_r) = {
                let h = header.lock().unwrap();
                (*h).clone()
            };
            let head_ptr = region_to_ptr(old_r);
            if head_ptr == 0 {
                return 0;
            }

            let mut h = header.lock().unwrap();
            if *h == (old_a, old_r) {
                *h = (0, 0);
                return head_ptr;
            }
        }
    }

    #[test]
    fn concurrent_push_pop() {
        loom::model(|| {
            let header: SListHeader = Arc::new(Mutex::new((0, 0)));
            let entries: EntryStore = Arc::new(Mutex::new(Vec::new()));

            let h1 = header.clone();
            let e1 = entries.clone();
            let h2 = header.clone();
            let e2 = entries.clone();

            // Thread 1: push entry 0x1000
            let t1 = thread::spawn(move || {
                push(&h1, &e1, 0x1000);
            });

            // Thread 2: push entry 0x2000
            let t2 = thread::spawn(move || {
                push(&h2, &e2, 0x2000);
            });

            t1.join().unwrap();
            t2.join().unwrap();

            // Both entries should be in the list (depth = 2)
            let (a, _r) = *header.lock().unwrap();
            let (depth, _) = unpack(a);
            assert_eq!(depth, 2, "Both pushes must succeed");

            // Pop both — should get both entries back
            let p1 = pop(&header, &entries);
            let p2 = pop(&header, &entries);
            let p3 = pop(&header, &entries);

            assert_ne!(p1, 0);
            assert_ne!(p2, 0);
            assert_eq!(p3, 0); // List should be empty now

            // Both entries accounted for
            let mut got = [p1, p2];
            got.sort();
            assert_eq!(got, [0x1000, 0x2000]);
        });
    }

    #[test]
    fn concurrent_push_flush() {
        loom::model(|| {
            let header: SListHeader = Arc::new(Mutex::new((0, 0)));
            let entries: EntryStore = Arc::new(Mutex::new(Vec::new()));

            let h1 = header.clone();
            let e1 = entries.clone();
            let h2 = header.clone();

            // Thread 1: push entry 0x1000
            let t1 = thread::spawn(move || {
                push(&h1, &e1, 0x1000);
            });

            // Thread 2: flush
            let t2 = thread::spawn(move || -> u64 {
                flush(&h2)
            });

            t1.join().unwrap();
            let flushed = t2.join().unwrap();

            // Either the flush got the entry, or the list still has it
            let (a, r) = *header.lock().unwrap();
            let (depth, _) = unpack(a);
            let head = region_to_ptr(r);

            if flushed != 0 {
                // Flush got something — list should be empty
                assert_eq!(depth, 0);
                assert_eq!(head, 0);
            } else {
                // Flush got nothing — entry should still be in list
                assert_eq!(depth, 1);
                assert_eq!(head, 0x1000);
            }
        });
    }

    #[test]
    fn depth_consistency() {
        loom::model(|| {
            let header: SListHeader = Arc::new(Mutex::new((0, 0)));
            let entries: EntryStore = Arc::new(Mutex::new(Vec::new()));

            // Push 2 entries sequentially from one thread
            push(&header, &entries, 0x1000);
            push(&header, &entries, 0x2000);

            let h1 = header.clone();
            let e1 = entries.clone();
            let h2 = header.clone();
            let e2 = entries.clone();

            // Thread 1: pop
            let t1 = thread::spawn(move || -> u64 {
                pop(&h1, &e1)
            });

            // Thread 2: pop
            let t2 = thread::spawn(move || -> u64 {
                pop(&h2, &e2)
            });

            let r1 = t1.join().unwrap();
            let r2 = t2.join().unwrap();

            // Both should succeed (2 entries)
            assert_ne!(r1, 0);
            assert_ne!(r2, 0);
            assert_ne!(r1, r2); // Different entries

            // List should be empty
            let (a, _) = *header.lock().unwrap();
            let (depth, _) = unpack(a);
            assert_eq!(depth, 0);
        });
    }
}
