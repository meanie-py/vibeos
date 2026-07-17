// C ABI scalar typedefs (Windows semantics: DWORD = u32, BOOL = int, PVOID =
// void*). Defined locally rather than imported from ntdll-sys so this
// pure-algorithm crate carries no Windows-only dependency and is host-testable
// (`cargo test`). ABI-identical to the prior `ntdll_sys::nt` aliases on every
// Windows target — there `DWORD = c_ulong` is 32-bit (Windows long), which is
// exactly `u32`; on a linux host `c_ulong` would (wrongly) be 64-bit, which is
// why importing them blocked host compilation.
use core::ffi::c_void;
pub type PVOID = *mut c_void;
pub type DWORD = u32;
pub type BOOL = i32;

// sdk/lib/skiplist/CMakeLists.txt builds skiplist16 with -DSKIPLIST_LEVELS=16.
pub const SKIPLIST_LEVELS: usize = 16;

// --- Type definitions (must match sdk/lib/skiplist/skiplist.h exactly) ---

pub type PSKIPLIST_ALLOCATE_ROUTINE = unsafe extern "system" fn(DWORD) -> PVOID;
pub type PSKIPLIST_COMPARE_ROUTINE = unsafe extern "system" fn(PVOID, PVOID) -> i32;
pub type PSKIPLIST_FREE_ROUTINE = unsafe extern "system" fn(PVOID);

#[repr(C)]
pub struct SKIPLIST_NODE {
    pub Distance: [DWORD; SKIPLIST_LEVELS],
    pub Next: [*mut SKIPLIST_NODE; SKIPLIST_LEVELS],
    pub Element: PVOID,
}

#[repr(C)]
pub struct SKIPLIST {
    pub Head: SKIPLIST_NODE,
    pub MaximumLevel: i8,       // CHAR (signed)
    pub NodeCount: DWORD,
    pub AllocateRoutine: PSKIPLIST_ALLOCATE_ROUTINE,
    pub CompareRoutine: PSKIPLIST_COMPARE_ROUTINE,
    pub FreeRoutine: PSKIPLIST_FREE_ROUTINE,
}

// ============================================================================
// Internal helpers
// ============================================================================

unsafe fn _GetRandomLevel() -> i8 {
    static mut dwRandom: u32 = 1;
    let mut dwLevel: u32 = 0;
    let dwShifted: u32;

    dwRandom = ((dwRandom as u64 * 48271u64) % 2147483647u64) as u32;
    dwShifted = dwRandom >> (31 - SKIPLIST_LEVELS as u32);

    if dwShifted != 0 {
        // BitScanForward sets dwLevel to the zero-based position of the
        // first set bit (from LSB to MSB).
        dwLevel = dwShifted.trailing_zeros();
    }

    return dwLevel as i8;
}

unsafe fn _InsertElementSkiplistWithInformation(
    Skiplist: *mut SKIPLIST,
    Element: PVOID,
    pUpdate: *mut *mut SKIPLIST_NODE,
    dwDistance: *mut DWORD,
) -> BOOL {
    let chNewLevel: i8;
    let _i: i8;
    let pNode: *mut SKIPLIST_NODE;

    chNewLevel = _GetRandomLevel();

    if chNewLevel > (*Skiplist).MaximumLevel {
        let mut i = (*Skiplist).MaximumLevel + 1;
        while i <= chNewLevel {
            *pUpdate.add(i as usize) = core::ptr::addr_of_mut!((*Skiplist).Head);
            (*(*pUpdate.add(i as usize))).Distance[i as usize] = (*Skiplist).NodeCount + 1;
            i += 1;
        }
        (*Skiplist).MaximumLevel = chNewLevel;
    }

    pNode = ((*Skiplist).AllocateRoutine)(
        core::mem::size_of::<SKIPLIST_NODE>() as DWORD,
    ) as *mut SKIPLIST_NODE;
    if pNode.is_null() {
        return 0; // FALSE
    }

    (*pNode).Element = Element;

    let mut i = 0i8;
    while i <= chNewLevel {
        let node_ptr = *pUpdate.add(i as usize);
        (*pNode).Next[i as usize] = (*node_ptr).Next[i as usize];
        (*node_ptr).Next[i as usize] = pNode;
        // C unsigned arithmetic: the inner `Distance[i] - dwDistance[0]` can
        // underflow (wrap) while the full expression stays non-negative — the
        // wrap cancels mod 2^32. Spell it out so debug overflow-checks don't
        // trip; disasm-identical to the wrapping release build.
        (*pNode).Distance[i as usize] = (*dwDistance.add(i as usize)).wrapping_add(
            (*node_ptr).Distance[i as usize].wrapping_sub(*dwDistance.add(0)),
        );
        (*node_ptr).Distance[i as usize] =
            *dwDistance.add(0) + 1 - *dwDistance.add(i as usize);
        i += 1;
    }

    let mut i = chNewLevel + 1;
    while i <= (*Skiplist).MaximumLevel {
        let node_ptr = *pUpdate.add(i as usize);
        (*node_ptr).Distance[i as usize] += 1;
        i += 1;
    }

    (*Skiplist).NodeCount += 1;
    return 1; // TRUE
}

// ============================================================================
// Public API (WINAPI / stdcall — i386 needs _Name@N decoration)
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn DeleteElementSkiplist(
    Skiplist: *mut SKIPLIST,
    Element: PVOID,
) -> PVOID {
    let mut i: i8;
    let mut pLastComparedNode: *mut SKIPLIST_NODE = core::ptr::null_mut();
    let mut pNode: *mut SKIPLIST_NODE = core::ptr::addr_of_mut!((*Skiplist).Head);
    let mut pUpdate: [*mut SKIPLIST_NODE; SKIPLIST_LEVELS] =
        [core::ptr::null_mut(); SKIPLIST_LEVELS];
    let pReturnValue: PVOID;

    i = ((*Skiplist).MaximumLevel + 1) as i8;
    loop {
        i -= 1;
        if i < 0 {
            break;
        }
        while !(*pNode).Next[i as usize].is_null()
            && (*pNode).Next[i as usize] != pLastComparedNode
            && ((*Skiplist).CompareRoutine)((*(*pNode).Next[i as usize]).Element, Element) < 0
        {
            pNode = (*pNode).Next[i as usize];
        }
        pLastComparedNode = (*pNode).Next[i as usize];
        pUpdate[i as usize] = pNode;
    }
    pNode = (*pNode).Next[0];
    if pNode.is_null() || ((*Skiplist).CompareRoutine)((*pNode).Element, Element) != 0 {
        return core::ptr::null_mut();
    }
    i = 0;
    while i <= (*Skiplist).MaximumLevel as i8 && (*pUpdate[i as usize]).Next[i as usize] == pNode {
        // `Distance[i] - 1` underflows when the deleted node is the last at
        // level i (span 0); the merged `+=` result is correct mod 2^32 (C
        // unsigned). Explicit wrapping keeps debug overflow-checks happy.
        (*pUpdate[i as usize]).Distance[i as usize] = (*pUpdate[i as usize]).Distance
            [i as usize]
            .wrapping_add((*pNode).Distance[i as usize])
            .wrapping_sub(1);
        (*pUpdate[i as usize]).Next[i as usize] = (*pNode).Next[i as usize];
        i += 1;
    }
    while i <= (*Skiplist).MaximumLevel as i8 {
        (*pUpdate[i as usize]).Distance[i as usize] -= 1;
        i += 1;
    }
    pReturnValue = (*pNode).Element;
    ((*Skiplist).FreeRoutine)(pNode as PVOID);
    while (*Skiplist).MaximumLevel > 0
        && (*Skiplist).Head.Next[(*Skiplist).MaximumLevel as usize].is_null()
    {
        (*Skiplist).MaximumLevel -= 1;
    }
    (*Skiplist).NodeCount -= 1;
    pReturnValue
}

#[no_mangle]
pub unsafe extern "C" fn InitializeSkiplist(
    Skiplist: *mut SKIPLIST,
    AllocateRoutine: PSKIPLIST_ALLOCATE_ROUTINE,
    CompareRoutine: PSKIPLIST_COMPARE_ROUTINE,
    FreeRoutine: PSKIPLIST_FREE_ROUTINE,
) {
    // Store the routines.
    (*Skiplist).AllocateRoutine = AllocateRoutine;
    (*Skiplist).CompareRoutine = CompareRoutine;
    (*Skiplist).FreeRoutine = FreeRoutine;

    // Initialize the members and pointers.
    // The Distance array is only used when a node is non-NULL, so it
    // doesn't need initialization.
    (*Skiplist).MaximumLevel = 0;
    (*Skiplist).NodeCount = 0;
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*Skiplist).Head.Next).cast::<*mut SKIPLIST_NODE>(),
        0u8,
        SKIPLIST_LEVELS,
    );
}

#[no_mangle]
pub unsafe extern "C" fn InsertElementSkiplist(
    Skiplist: *mut SKIPLIST,
    Element: PVOID,
) -> BOOL {
    let mut i: i8;
    let mut dwDistance: [DWORD; SKIPLIST_LEVELS + 1] = [0; SKIPLIST_LEVELS + 1];
    let mut pLastComparedNode: *mut SKIPLIST_NODE = core::ptr::null_mut();
    let mut pNode: *mut SKIPLIST_NODE = core::ptr::addr_of_mut!((*Skiplist).Head);
    let mut pUpdate: [*mut SKIPLIST_NODE; SKIPLIST_LEVELS] =
        [core::ptr::null_mut(); SKIPLIST_LEVELS];

    i = ((*Skiplist).MaximumLevel as i8) + 1;
    while {
        i -= 1;
        i >= 0
    } {
        dwDistance[i as usize] = dwDistance[(i + 1) as usize];
        while !((*pNode).Next[i as usize]).is_null()
            && (*pNode).Next[i as usize] != pLastComparedNode
            && ((*Skiplist).CompareRoutine)((*(*pNode).Next[i as usize]).Element, Element) < 0
        {
            dwDistance[i as usize] += (*pNode).Distance[i as usize];
            pNode = (*pNode).Next[i as usize];
        }
        pLastComparedNode = (*pNode).Next[i as usize];
        pUpdate[i as usize] = pNode;
    }

    pNode = (*pNode).Next[0];
    if !pNode.is_null() && ((*Skiplist).CompareRoutine)((*pNode).Element, Element) == 0 {
        return 0; // FALSE
    }

    _InsertElementSkiplistWithInformation(
        Skiplist,
        Element,
        pUpdate.as_mut_ptr(),
        dwDistance.as_mut_ptr(),
    )
}

#[no_mangle]
pub unsafe extern "C" fn InsertTailElementSkiplist(
    Skiplist: *mut SKIPLIST,
    Element: PVOID,
) -> BOOL {
    let mut i: i8;
    let mut dwDistance: [DWORD; SKIPLIST_LEVELS + 1] = [0; SKIPLIST_LEVELS + 1];
    let mut pNode: *mut SKIPLIST_NODE = core::ptr::addr_of_mut!((*Skiplist).Head);
    let mut pUpdate: [*mut SKIPLIST_NODE; SKIPLIST_LEVELS] =
        [core::ptr::null_mut(); SKIPLIST_LEVELS];

    i = ((*Skiplist).MaximumLevel as i8) + 1;
    loop {
        i -= 1;
        if i < 0 {
            break;
        }
        dwDistance[i as usize] = dwDistance[(i + 1) as usize];
        while !(*pNode).Next[i as usize].is_null() {
            dwDistance[i as usize] = dwDistance[i as usize]
                .wrapping_add((*pNode).Distance[i as usize]);
            pNode = (*pNode).Next[i as usize];
        }
        pUpdate[i as usize] = pNode;
    }

    _InsertElementSkiplistWithInformation(
        Skiplist,
        Element,
        pUpdate.as_mut_ptr(),
        dwDistance.as_mut_ptr(),
    )
}

#[no_mangle]
pub unsafe extern "C" fn LookupElementSkiplist(
    Skiplist: *mut SKIPLIST,
    Element: PVOID,
    ElementIndex: *mut DWORD,
) -> PVOID {
    let mut i: i8;
    let mut dwIndex: DWORD = 0;
    let mut pLastComparedNode: *mut SKIPLIST_NODE = core::ptr::null_mut();
    let mut pNode: *mut SKIPLIST_NODE = core::ptr::addr_of_mut!((*Skiplist).Head);

    i = ((*Skiplist).MaximumLevel as i8) + 1;
    while {
        i -= 1;
        i >= 0
    } {
        loop {
            let next = (*pNode).Next[i as usize];
            if next.is_null() {
                break;
            }
            if next == pLastComparedNode {
                break;
            }
            if ((*Skiplist).CompareRoutine)((*next).Element, Element) >= 0 {
                break;
            }
            dwIndex = dwIndex.wrapping_add((*pNode).Distance[i as usize]);
            pNode = next;
        }
        pLastComparedNode = (*pNode).Next[i as usize];
    }

    pNode = (*pNode).Next[0];
    if pNode.is_null() || ((*Skiplist).CompareRoutine)((*pNode).Element, Element) != 0 {
        return core::ptr::null_mut();
    }

    if !ElementIndex.is_null() {
        core::ptr::write_volatile(ElementIndex, dwIndex);
    }

    (*pNode).Element
}

#[no_mangle]
pub unsafe extern "C" fn LookupNodeByIndexSkiplist(
    Skiplist: *mut SKIPLIST,
    ElementIndex: DWORD,
) -> *mut SKIPLIST_NODE {
    let mut i: i8;
    let mut dwIndex: DWORD = 0;
    let mut pNode: *mut SKIPLIST_NODE = core::ptr::addr_of_mut!((*Skiplist).Head);

    if ElementIndex >= (*Skiplist).NodeCount {
        return core::ptr::null_mut();
    }

    i = ((*Skiplist).MaximumLevel as i8) + 1;
    while {
        i -= 1;
        i >= 0
    } {
        while !((*pNode).Next[i as usize]).is_null()
            && dwIndex + (*pNode).Distance[i as usize] <= ElementIndex
        {
            dwIndex += (*pNode).Distance[i as usize];
            pNode = (*pNode).Next[i as usize];
        }
    }

    return (*pNode).Next[0];
}

// ============================================================================
// Tests — model-based verification against a sorted reference (BTreeSet).
//
// The skiplist is an *order-statistic* (indexable) structure: every node holds
// a `Distance` array so `LookupElementSkiplist` can report an element's rank and
// `LookupNodeByIndexSkiplist` can fetch by rank. That Distance bookkeeping is
// the subtle, translation-bug-prone part, so every check below asserts the full
// rank/index consistency, not just membership.
//
// Elements are encoded as their (non-zero) integer key cast directly to PVOID —
// no separate key storage. Key 0 is avoided because a stored null Element is
// indistinguishable from the "not found" sentinel the API returns.
//
// Note on `_GetRandomLevel`'s process-wide `static mut`: cargo runs tests in
// parallel, but the only thing that static influences is the random *level*
// assigned to a node — and a skiplist is correct for ANY level assignment. Each
// test asserts structural invariants of its OWN independent list, which hold
// regardless of the levels chosen, so the shared RNG is benign here.
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use std::alloc::{alloc_zeroed, dealloc, Layout};
    use std::collections::BTreeSet;

    fn node_layout() -> Layout {
        Layout::new::<SKIPLIST_NODE>()
    }

    unsafe extern "system" fn test_alloc(size: DWORD) -> PVOID {
        assert_eq!(size as usize, core::mem::size_of::<SKIPLIST_NODE>());
        alloc_zeroed(node_layout()) as PVOID
    }

    unsafe extern "system" fn test_free(p: PVOID) {
        dealloc(p as *mut u8, node_layout());
    }

    unsafe extern "system" fn test_compare(a: PVOID, b: PVOID) -> i32 {
        let (ka, kb) = (a as usize, b as usize);
        match ka.cmp(&kb) {
            core::cmp::Ordering::Less => -1,
            core::cmp::Ordering::Equal => 0,
            core::cmp::Ordering::Greater => 1,
        }
    }

    fn elem(k: u32) -> PVOID {
        assert_ne!(k, 0, "key 0 is reserved (collides with the null sentinel)");
        k as usize as PVOID
    }

    // Build a fully-initialized SKIPLIST (avoids `mem::zeroed` UB on the
    // non-nullable fn-pointer fields), then run InitializeSkiplist over it.
    fn make() -> Box<SKIPLIST> {
        let mut sl = Box::new(SKIPLIST {
            Head: SKIPLIST_NODE {
                Distance: [0; SKIPLIST_LEVELS],
                Next: [core::ptr::null_mut(); SKIPLIST_LEVELS],
                Element: core::ptr::null_mut(),
            },
            MaximumLevel: 0,
            NodeCount: 0,
            AllocateRoutine: test_alloc,
            CompareRoutine: test_compare,
            FreeRoutine: test_free,
        });
        unsafe {
            InitializeSkiplist(&mut *sl, test_alloc, test_compare, test_free);
        }
        sl
    }

    // Assert the list exactly mirrors `model`, including every rank/index query.
    unsafe fn check(sl: *mut SKIPLIST, model: &BTreeSet<u32>) {
        assert_eq!((*sl).NodeCount as usize, model.len(), "NodeCount");

        for (rank, &k) in model.iter().enumerate() {
            // forward: key -> element + rank
            let mut idx: DWORD = DWORD::MAX;
            let got = LookupElementSkiplist(sl, elem(k), &mut idx);
            assert_eq!(got, elem(k), "lookup of present key {k}");
            assert_eq!(idx as usize, rank, "reported rank for key {k}");

            // reverse: rank -> node holding that element
            let node = LookupNodeByIndexSkiplist(sl, rank as DWORD);
            assert!(!node.is_null(), "byindex({rank}) null");
            assert_eq!((*node).Element, elem(k), "byindex({rank}) element");
        }

        // out-of-range index queries return null
        assert!(
            LookupNodeByIndexSkiplist(sl, model.len() as DWORD).is_null(),
            "byindex(len) should be null"
        );
    }

    // Delete everything (also exercises delete + frees all nodes).
    unsafe fn drain(sl: *mut SKIPLIST, model: &mut BTreeSet<u32>) {
        let keys: Vec<u32> = model.iter().copied().collect();
        for k in keys {
            let removed = DeleteElementSkiplist(sl, elem(k));
            assert_eq!(removed, elem(k), "delete returns removed element {k}");
            model.remove(&k);
        }
        assert_eq!((*sl).NodeCount, 0, "drained NodeCount");
        check(sl, model);
    }

    // SplitMix64 — deterministic, dependency-free PRNG.
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }
        fn below(&mut self, n: u32) -> u32 {
            (self.next() % n as u64) as u32
        }
    }

    #[test]
    fn empty_table() {
        unsafe {
            let mut sl = make();
            let p = &mut *sl as *mut SKIPLIST;
            assert_eq!((*p).NodeCount, 0);
            let mut idx: DWORD = 7;
            assert!(LookupElementSkiplist(p, elem(5), &mut idx).is_null());
            assert!(LookupNodeByIndexSkiplist(p, 0).is_null());
            assert!(DeleteElementSkiplist(p, elem(5)).is_null());
            check(p, &BTreeSet::new());
        }
    }

    #[test]
    fn single_insert_lookup_delete() {
        unsafe {
            let mut sl = make();
            let p = &mut *sl as *mut SKIPLIST;
            assert_eq!(InsertElementSkiplist(p, elem(42)), 1);
            // duplicate is rejected
            assert_eq!(InsertElementSkiplist(p, elem(42)), 0);

            let mut model = BTreeSet::new();
            model.insert(42);
            check(p, &model);

            assert_eq!(DeleteElementSkiplist(p, elem(42)), elem(42));
            model.remove(&42);
            check(p, &model);
            // second delete finds nothing
            assert!(DeleteElementSkiplist(p, elem(42)).is_null());
        }
    }

    #[test]
    fn ascending_then_descending_insert() {
        unsafe {
            for order in 0..2 {
                let mut sl = make();
                let p = &mut *sl as *mut SKIPLIST;
                let mut model = BTreeSet::new();
                let seq: Vec<u32> = if order == 0 {
                    (1..=200).collect()
                } else {
                    (1..=200).rev().collect()
                };
                for k in seq {
                    assert_eq!(InsertElementSkiplist(p, elem(k)), 1);
                    model.insert(k);
                }
                check(p, &model);
                drain(p, &mut model);
            }
        }
    }

    #[test]
    fn insert_tail_ascending() {
        // InsertTailElementSkiplist appends at the tail (no compare); it is the
        // fast path for strictly-ascending insertion. The resulting list must
        // still be a correct indexable structure.
        unsafe {
            let mut sl = make();
            let p = &mut *sl as *mut SKIPLIST;
            let mut model = BTreeSet::new();
            for k in 1..=300u32 {
                assert_eq!(InsertTailElementSkiplist(p, elem(k)), 1);
                model.insert(k);
            }
            check(p, &model);
            drain(p, &mut model);
        }
    }

    #[test]
    fn lookup_absent_keys() {
        unsafe {
            let mut sl = make();
            let p = &mut *sl as *mut SKIPLIST;
            let mut model = BTreeSet::new();
            // insert evens 2..=200
            for k in (2..=200u32).step_by(2) {
                assert_eq!(InsertElementSkiplist(p, elem(k)), 1);
                model.insert(k);
            }
            check(p, &model);
            // odds are absent
            for k in (1..=199u32).step_by(2) {
                let mut idx: DWORD = DWORD::MAX;
                assert!(
                    LookupElementSkiplist(p, elem(k), &mut idx).is_null(),
                    "odd key {k} should be absent"
                );
                assert!(DeleteElementSkiplist(p, elem(k)).is_null());
            }
            drain(p, &mut model);
        }
    }

    #[test]
    fn random_insert_unique() {
        unsafe {
            let mut sl = make();
            let p = &mut *sl as *mut SKIPLIST;
            let mut model = BTreeSet::new();
            let mut rng = Rng(0xDEAD_BEEF);
            for _ in 0..2000 {
                let k = rng.below(1000) + 1; // 1..=1000
                let expected: BOOL = (!model.contains(&k)) as BOOL;
                assert_eq!(
                    InsertElementSkiplist(p, elem(k)),
                    expected,
                    "insert {k} new={expected}"
                );
                model.insert(k);
            }
            check(p, &model);
            drain(p, &mut model);
        }
    }

    #[test]
    fn random_insert_delete_stress() {
        unsafe {
            let mut sl = make();
            let p = &mut *sl as *mut SKIPLIST;
            let mut model = BTreeSet::new();
            let mut rng = Rng(0x0123_4567_89AB_CDEF);
            for step in 0..8000 {
                let k = rng.below(400) + 1; // 1..=400
                if rng.below(2) == 0 {
                    let expected: BOOL = (!model.contains(&k)) as BOOL;
                    assert_eq!(InsertElementSkiplist(p, elem(k)), expected);
                    model.insert(k);
                } else {
                    let present = model.contains(&k);
                    let removed = DeleteElementSkiplist(p, elem(k));
                    if present {
                        assert_eq!(removed, elem(k), "delete present {k}");
                        model.remove(&k);
                    } else {
                        assert!(removed.is_null(), "delete absent {k}");
                    }
                }
                // Full consistency sweep periodically (cheap relative to the run).
                if step % 500 == 0 {
                    check(p, &model);
                }
            }
            check(p, &model);
            drain(p, &mut model);
        }
    }
}
