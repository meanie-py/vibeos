//! Pure state-transition functions for SList operations.
//!
//! These functions take the old header state and return the new header state
//! plus any result. They contain no unsafe code, no inline assembly, and no
//! side effects — making them ideal targets for Kani model checking.

use crate::header::{self, SListFields};

/// Result of a pop transition.
pub struct PopResult {
    /// New (alignment, region) to store back.
    pub new_alignment: u64,
    pub new_region: u64,
    /// The popped entry pointer (0 if list was empty).
    pub popped: u64,
}

/// Compute the pop transition.
///
/// `old_alignment`, `old_region`: current SLIST_HEADER read via LDXP.
/// `next_of_head`: the `->Next` field of the current head entry (read from memory).
///                 Only used when the list is non-empty.
///
/// Returns `None` if the list is empty (caller should CLREX and return NULL).
/// Returns `Some(PopResult)` with the new header state and popped pointer.
pub fn pop_transition(old_alignment: u64, old_region: u64, next_of_head: u64) -> Option<PopResult> {
    let old = header::unpack(old_alignment, old_region);
    let head_ptr = header::region_to_ptr(old.next_entry);

    if head_ptr == 0 {
        return None; // Empty list
    }

    let new = SListFields {
        depth: old.depth.wrapping_sub(1),
        sequence: old.sequence.wrapping_add(1) & 0xFFFF_FFFF_FFFF, // 48-bit wrap
        next_entry: next_of_head, // New head is old head's ->Next
    };
    let (new_a, new_r) = header::pack(&new);

    Some(PopResult {
        new_alignment: new_a,
        new_region: new_r,
        popped: head_ptr,
    })
}

/// Result of a push transition.
pub struct PushResult {
    /// New (alignment, region) to store back.
    pub new_alignment: u64,
    pub new_region: u64,
    /// The old head pointer (to return to caller).
    pub old_head: u64,
}

/// Compute the push transition.
///
/// `old_alignment`, `old_region`: current SLIST_HEADER.
/// `new_entry`: pointer to the new entry being pushed.
///
/// The caller must also write `old_head` into `new_entry->Next` before the STXP.
pub fn push_transition(old_alignment: u64, old_region: u64, new_entry: u64) -> PushResult {
    let old = header::unpack(old_alignment, old_region);
    let old_head = header::region_to_ptr(old.next_entry);

    let new = SListFields {
        depth: old.depth.wrapping_add(1),
        sequence: old.sequence.wrapping_add(1) & 0xFFFF_FFFF_FFFF,
        next_entry: new_entry, // New head is the pushed entry
    };
    let (new_a, new_r) = header::pack(&new);

    PushResult {
        new_alignment: new_a,
        new_region: new_r,
        old_head,
    }
}

/// Result of a push-list (bulk push) transition.
pub struct PushListResult {
    /// New (alignment, region) to store back.
    pub new_alignment: u64,
    pub new_region: u64,
    /// The old head pointer — the caller writes it into `ListEnd->Next` AND
    /// returns it (RtlInterlockedPushListSList returns the previous first entry).
    pub old_head: u64,
}

/// Compute the push-list transition (RtlInterlockedPushListSList).
///
/// Pushes a pre-linked chain whose head is `list` and whose tail is `ListEnd`
/// (caller-owned) onto the front of the list. `list` becomes the new head; the
/// caller must write `old_head` into `ListEnd->Next` before the store. `count`
/// is the number of entries in the chain (added to Depth, truncated to 16 bits
/// exactly as the C `Header.Depth += Count`).
pub fn push_list_transition(
    old_alignment: u64,
    old_region: u64,
    list: u64,
    count: u16,
) -> PushListResult {
    let old = header::unpack(old_alignment, old_region);
    let old_head = header::region_to_ptr(old.next_entry);

    let new = SListFields {
        depth: old.depth.wrapping_add(count),
        sequence: old.sequence.wrapping_add(1) & 0xFFFF_FFFF_FFFF,
        next_entry: list, // New head is the chain head
    };
    let (new_a, new_r) = header::pack(&new);

    PushListResult {
        new_alignment: new_a,
        new_region: new_r,
        old_head,
    }
}

/// Result of a flush transition.
pub struct FlushResult {
    /// New (alignment, region) — always zeroed.
    pub new_alignment: u64,
    pub new_region: u64,
    /// The old head pointer (entire chain returned to caller).
    pub old_head: u64,
}

/// Compute the flush transition.
///
/// Returns `None` if the list is already empty.
pub fn flush_transition(old_alignment: u64, old_region: u64) -> Option<FlushResult> {
    let old = header::unpack(old_alignment, old_region);
    let old_head = header::region_to_ptr(old.next_entry);

    if old_head == 0 {
        return None;
    }

    Some(FlushResult {
        new_alignment: 0,
        new_region: 0,
        old_head,
    })
}

// ============================================================================
// Unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_header(depth: u16, seq: u64, next: u64) -> (u64, u64) {
        header::pack(&SListFields {
            depth,
            sequence: seq,
            next_entry: next,
        })
    }

    #[test]
    fn pop_empty_returns_none() {
        let (a, r) = make_header(0, 0, 0);
        assert!(pop_transition(a, r, 0).is_none());
    }

    #[test]
    fn pop_single_element() {
        let entry_addr: u64 = 0x1000; // 16-byte aligned
        let (a, r) = make_header(1, 5, entry_addr);
        let result = pop_transition(a, r, 0).unwrap(); // entry->Next = NULL
        assert_eq!(result.popped, entry_addr);
        let new = header::unpack(result.new_alignment, result.new_region);
        assert_eq!(new.depth, 0);
        assert_eq!(new.sequence, 6);
        assert_eq!(header::region_to_ptr(new.next_entry), 0);
    }

    #[test]
    fn push_onto_empty() {
        let (a, r) = make_header(0, 0, 0);
        let new_entry: u64 = 0x2000;
        let result = push_transition(a, r, new_entry);
        assert_eq!(result.old_head, 0);
        let new = header::unpack(result.new_alignment, result.new_region);
        assert_eq!(new.depth, 1);
        assert_eq!(new.sequence, 1);
        assert_eq!(header::region_to_ptr(new.next_entry), new_entry);
    }

    #[test]
    fn push_pop_roundtrip() {
        let (a, r) = make_header(0, 0, 0);
        let entry: u64 = 0x3000;

        // Push
        let push = push_transition(a, r, entry);
        let after_push = header::unpack(push.new_alignment, push.new_region);
        assert_eq!(after_push.depth, 1);

        // Pop (entry->Next would be the old head = 0)
        let pop = pop_transition(push.new_alignment, push.new_region, push.old_head).unwrap();
        assert_eq!(pop.popped, entry);
        let after_pop = header::unpack(pop.new_alignment, pop.new_region);
        assert_eq!(after_pop.depth, 0);
    }

    #[test]
    fn lifo_order() {
        let (a, r) = make_header(0, 0, 0);
        let entry_a: u64 = 0x1000;
        let entry_b: u64 = 0x2000;

        // Push A
        let p1 = push_transition(a, r, entry_a);
        // Push B (B->Next = A)
        let p2 = push_transition(p1.new_alignment, p1.new_region, entry_b);

        // Pop should return B first
        let pop1 = pop_transition(p2.new_alignment, p2.new_region, entry_a).unwrap();
        assert_eq!(pop1.popped, entry_b);

        // Pop should return A next
        let pop2 = pop_transition(pop1.new_alignment, pop1.new_region, 0).unwrap();
        assert_eq!(pop2.popped, entry_a);
    }

    #[test]
    fn push_list_onto_empty() {
        let (a, r) = make_header(0, 0, 0);
        let list: u64 = 0x2000; // chain head, 16-byte aligned
        let result = push_list_transition(a, r, list, 3);
        assert_eq!(result.old_head, 0); // ListEnd->Next becomes NULL
        let new = header::unpack(result.new_alignment, result.new_region);
        assert_eq!(new.depth, 3); // count added
        assert_eq!(new.sequence, 1);
        assert_eq!(header::region_to_ptr(new.next_entry), list);
    }

    #[test]
    fn push_list_onto_nonempty_links_tail() {
        // Existing single element at 0x1000, depth 1
        let (a, r) = make_header(1, 5, 0x1000);
        let list: u64 = 0x4000; // new chain of 2
        let result = push_list_transition(a, r, list, 2);
        // Old head returned so caller can splice ListEnd->Next = 0x1000
        assert_eq!(result.old_head, 0x1000);
        let new = header::unpack(result.new_alignment, result.new_region);
        assert_eq!(new.depth, 3); // 1 + 2
        assert_eq!(new.sequence, 6);
        assert_eq!(header::region_to_ptr(new.next_entry), list);
    }

    #[test]
    fn push_list_then_pop_returns_chain_head() {
        let (a, r) = make_header(0, 0, 0);
        let list: u64 = 0x5000;
        let push = push_list_transition(a, r, list, 4);
        // Pop: chain head's ->Next was set to old_head (0) by the caller
        let pop = pop_transition(push.new_alignment, push.new_region, push.old_head).unwrap();
        assert_eq!(pop.popped, list);
        let after = header::unpack(pop.new_alignment, pop.new_region);
        assert_eq!(after.depth, 3); // 4 - 1
    }

    #[test]
    fn flush_empty() {
        let (a, r) = make_header(0, 0, 0);
        assert!(flush_transition(a, r).is_none());
    }

    #[test]
    fn flush_nonempty() {
        let entry: u64 = 0x4000;
        let (a, r) = make_header(3, 10, entry);
        let result = flush_transition(a, r).unwrap();
        assert_eq!(result.old_head, entry);
        assert_eq!(result.new_alignment, 0);
        assert_eq!(result.new_region, 0);
    }

    #[test]
    fn sequence_increments() {
        let (a, r) = make_header(0, 100, 0);
        let entry: u64 = 0x5000;
        let push = push_transition(a, r, entry);
        let after = header::unpack(push.new_alignment, push.new_region);
        assert_eq!(after.sequence, 101);
    }

    #[test]
    fn depth_tracking_multiple() {
        let mut a: u64 = 0;
        let mut r: u64 = 0;
        let entries: [u64; 5] = [0x1000, 0x2000, 0x3000, 0x4000, 0x5000];
        for &e in &entries {
            let p = push_transition(a, r, e);
            a = p.new_alignment;
            r = p.new_region;
        }
        let h = header::unpack(a, r);
        assert_eq!(h.depth, 5);
    }
}

// ============================================================================
// Kani verification proofs
// ============================================================================

#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    fn pack_unpack_roundtrip() {
        let depth: u16 = kani::any();
        let sequence: u64 = kani::any::<u64>() & 0xFFFF_FFFF_FFFF; // 48 bits
        let next_entry: u64 = kani::any::<u64>() & !0xF; // 16-byte aligned

        let f = SListFields { depth, sequence, next_entry };
        let (a, r) = header::pack(&f);
        let f2 = header::unpack(a, r);

        assert_eq!(f.depth, f2.depth);
        assert_eq!(f.sequence, f2.sequence);
        assert_eq!(f.next_entry, f2.next_entry);
    }

    #[kani::proof]
    fn push_increments_depth() {
        let depth: u16 = kani::any();
        kani::assume(depth < u16::MAX); // No overflow
        let seq: u64 = kani::any::<u64>() & 0xFFFF_FFFF_FFFF;
        let next: u64 = kani::any::<u64>() & !0xF;
        let (a, r) = header::pack(&SListFields { depth, sequence: seq, next_entry: next });

        let new_entry: u64 = kani::any::<u64>() & !0xF;
        let result = push_transition(a, r, new_entry);
        let new = header::unpack(result.new_alignment, result.new_region);

        assert_eq!(new.depth, depth + 1);
    }

    #[kani::proof]
    fn push_increments_sequence() {
        let depth: u16 = kani::any();
        let seq: u64 = kani::any::<u64>() & 0xFFFF_FFFF_FFFF;
        let next: u64 = kani::any::<u64>() & !0xF;
        let (a, r) = header::pack(&SListFields { depth, sequence: seq, next_entry: next });

        let new_entry: u64 = kani::any::<u64>() & !0xF;
        let result = push_transition(a, r, new_entry);
        let new = header::unpack(result.new_alignment, result.new_region);

        assert_eq!(new.sequence, (seq + 1) & 0xFFFF_FFFF_FFFF);
    }

    #[kani::proof]
    fn pop_empty_is_none() {
        let seq: u64 = kani::any::<u64>() & 0xFFFF_FFFF_FFFF;
        let depth: u16 = kani::any();
        // Region = 0 means empty (pointer is zero after masking)
        let (a, r) = header::pack(&SListFields { depth, sequence: seq, next_entry: 0 });

        assert!(pop_transition(a, r, 0).is_none());
    }

    #[kani::proof]
    fn pop_decrements_depth() {
        let depth: u16 = kani::any();
        kani::assume(depth > 0);
        let seq: u64 = kani::any::<u64>() & 0xFFFF_FFFF_FFFF;
        let head: u64 = kani::any::<u64>() & !0xF;
        kani::assume(head != 0);
        let (a, r) = header::pack(&SListFields { depth, sequence: seq, next_entry: head });

        let next_of_head: u64 = kani::any::<u64>() & !0xF;
        let result = pop_transition(a, r, next_of_head).unwrap();
        let new = header::unpack(result.new_alignment, result.new_region);

        assert_eq!(new.depth, depth - 1);
    }

    #[kani::proof]
    fn push_pop_roundtrip() {
        // Start with empty list
        let (a, r) = header::pack(&SListFields { depth: 0, sequence: 0, next_entry: 0 });
        let entry: u64 = kani::any::<u64>() & !0xF;
        kani::assume(entry != 0);

        // Push
        let push = push_transition(a, r, entry);
        // entry->Next = old_head = 0

        // Pop
        let pop = pop_transition(push.new_alignment, push.new_region, push.old_head).unwrap();

        // Should get back the same entry
        assert_eq!(pop.popped, entry);
        // Depth back to 0
        let final_h = header::unpack(pop.new_alignment, pop.new_region);
        assert_eq!(final_h.depth, 0);
    }

    #[kani::proof]
    fn push_list_adds_count_to_depth() {
        let depth: u16 = kani::any();
        let count: u16 = kani::any();
        let seq: u64 = kani::any::<u64>() & 0xFFFF_FFFF_FFFF;
        let next: u64 = kani::any::<u64>() & !0xF;
        let (a, r) = header::pack(&SListFields { depth, sequence: seq, next_entry: next });

        let list: u64 = kani::any::<u64>() & !0xF;
        let result = push_list_transition(a, r, list, count);
        let new = header::unpack(result.new_alignment, result.new_region);

        // Depth wraps at 16 bits exactly like the C `Header.Depth += Count`.
        assert_eq!(new.depth, depth.wrapping_add(count));
        // Sequence advances by one.
        assert_eq!(new.sequence, (seq + 1) & 0xFFFF_FFFF_FFFF);
        // Old head is returned for the caller to splice into ListEnd->Next.
        assert_eq!(result.old_head, header::region_to_ptr(next));
        // New head is the chain head.
        assert_eq!(header::region_to_ptr(new.next_entry), list);
    }

    #[kani::proof]
    fn flush_empty_is_none() {
        let seq: u64 = kani::any::<u64>() & 0xFFFF_FFFF_FFFF;
        let depth: u16 = kani::any();
        let (a, r) = header::pack(&SListFields { depth, sequence: seq, next_entry: 0 });
        assert!(flush_transition(a, r).is_none());
    }

    #[kani::proof]
    fn flush_returns_head_and_zeros() {
        let depth: u16 = kani::any();
        let seq: u64 = kani::any::<u64>() & 0xFFFF_FFFF_FFFF;
        let head: u64 = kani::any::<u64>() & !0xF;
        kani::assume(head != 0);
        let (a, r) = header::pack(&SListFields { depth, sequence: seq, next_entry: head });

        let result = flush_transition(a, r).unwrap();
        assert_eq!(result.old_head, head);
        assert_eq!(result.new_alignment, 0);
        assert_eq!(result.new_region, 0);
    }

    #[kani::proof]
    fn lifo_order() {
        let entry_a: u64 = kani::any::<u64>() & !0xF;
        let entry_b: u64 = kani::any::<u64>() & !0xF;
        kani::assume(entry_a != 0 && entry_b != 0 && entry_a != entry_b);

        let (a, r) = header::pack(&SListFields { depth: 0, sequence: 0, next_entry: 0 });

        // Push A
        let p1 = push_transition(a, r, entry_a);
        // Push B (B->Next will be A)
        let p2 = push_transition(p1.new_alignment, p1.new_region, entry_b);

        // Pop should return B
        let pop1 = pop_transition(p2.new_alignment, p2.new_region, entry_a).unwrap();
        assert_eq!(pop1.popped, entry_b);

        // Pop should return A
        let pop2 = pop_transition(pop1.new_alignment, pop1.new_region, 0).unwrap();
        assert_eq!(pop2.popped, entry_a);
    }
}
