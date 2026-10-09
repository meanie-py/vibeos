//! `RtlWaitOnAddress` / `RtlWakeAddressAll` / `RtlWakeAddressSingle` — the
//! address-wait primitive family that backs Win32 `WaitOnAddress` /
//! `WakeByAddressSingle` / `WakeByAddressAll` (exported from kernelbase.dll,
//! which forwards here). No ReactOS C ancestor — this is a fresh Rust
//! implementation modelled on the keyed-event scheme the sibling `condvar`
//! module uses, but corrected for the stricter WaitOnAddress contract:
//! a wake must NEVER be lost (condvar tolerates lost wakes because its callers
//! re-test a predicate under a lock; WaitOnAddress callers only re-test the
//! *value*, so a genuinely lost wake is a hang).
//!
//! Substrate is anonymous NT keyed events (`NtWaitForKeyedEvent` /
//! `NtReleaseKeyedEvent`), the same primitive condvar/critsec/SRW block on.
//! Each waiter parks on a UNIQUE key (its own stack entry address), so every
//! wait/release pairs exactly — no key aliasing, no spurious cross-wakes.
//!
//! Correctness of the timeout race (the subtle part): the parked waiter and the
//! waker synchronise through the per-bucket spin lock plus the entry's
//! `Signaled` flag. Whoever takes the bucket lock first after the wait ends
//! decides ownership of the entry:
//!   * waiter first  -> `Signaled==0`, waiter unlinks itself, no release ever
//!                      issued for it (exact: 0 waits consumed, 0 releases).
//!   * waker first   -> waker unlinks, sets `Signaled=1`, releases OUTSIDE the
//!                      lock. The waiter, seeing `Signaled==1`, is obligated to
//!                      consume that release with an infinite `NtWaitForKeyedEvent`
//!                      even though it already timed out — keeping the pairing
//!                      exact. The entry address is used only as an opaque key
//!                      value (keyed events never dereference it), so the
//!                      waiter's stack frame outliving the release is not required.
//! Releases are always issued outside the bucket lock, so the lock is only ever
//! held for short intrusive-list edits.

use core::ffi::c_void;
use core::sync::atomic::{AtomicPtr, AtomicU32, Ordering};
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    LARGE_INTEGER, NtClose, NtCreateKeyedEvent, NtReleaseKeyedEvent, NtWaitForKeyedEvent,
};
use ntkrnl_sys::mm::HANDLE;

use crate::atomics::YieldProcessor;
use crate::shared::EVENT_ALL_ACCESS;

// ntstatus.h — the three codes this primitive can return.
const STATUS_SUCCESS: NTSTATUS = 0x0000_0000u32 as NTSTATUS;
const STATUS_TIMEOUT: NTSTATUS = 0x0000_0102u32 as NTSTATUS;
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000_000Du32 as NTSTATUS;

/// Intrusive wait record. Lives on the waiter's stack; linked into a bucket
/// only for the duration of the park. All fields except the park itself are
/// touched exclusively under the owning bucket's spin lock.
#[repr(C)]
struct AddrWaitEntry {
    Next: *mut AddrWaitEntry,
    /// The address this waiter is blocked on (wake filters on it).
    Address: *const c_void,
    /// 0 = still owned by the waiter; 1 = a waker unlinked us and will release.
    Signaled: u32,
}

/// One hash bucket: a spin lock guarding a singly linked list of waiters.
struct Bucket {
    Lock: AtomicU32,
    Head: AtomicPtr<AddrWaitEntry>,
}

// 256 buckets, hashed from the address. All-zero initial state (lock free, list
// empty) so the table needs no runtime construction.
const BUCKET_COUNT: usize = 256;
static BUCKETS: [Bucket; BUCKET_COUNT] = [const {
    Bucket {
        Lock: AtomicU32::new(0),
        Head: AtomicPtr::new(core::ptr::null_mut()),
    }
}; BUCKET_COUNT];

/// Process-wide anonymous keyed event, created on first use.
static KEYED_EVENT: AtomicPtr<c_void> = AtomicPtr::new(core::ptr::null_mut());

#[inline]
fn bucket_for(addr: *const c_void) -> &'static Bucket {
    // Drop the low 4 bits (sub-cache-line noise) before folding.
    let idx = ((addr as usize) >> 4) % BUCKET_COUNT;
    &BUCKETS[idx]
}

#[inline]
unsafe fn spin_lock(b: &Bucket) {
    while b
        .Lock
        .compare_exchange_weak(0, 1, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        YieldProcessor();
    }
}

#[inline]
unsafe fn spin_unlock(b: &Bucket) {
    b.Lock.store(0, Ordering::Release);
}

/// Lazily create (once) the anonymous keyed event every waiter/waker parks on.
unsafe fn ensure_keyed_event() -> HANDLE {
    let existing = KEYED_EVENT.load(Ordering::Acquire);
    if !existing.is_null() {
        return existing as HANDLE;
    }

    let mut fresh: HANDLE = core::ptr::null_mut();
    NtCreateKeyedEvent(
        core::ptr::addr_of_mut!(fresh),
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        0,
    );

    // Publish; if another thread beat us to it, drop ours and use theirs.
    match KEYED_EVENT.compare_exchange(
        core::ptr::null_mut(),
        fresh as *mut c_void,
        Ordering::AcqRel,
        Ordering::Acquire,
    ) {
        Ok(_) => fresh,
        Err(winner) => {
            // Another thread beat us to publishing; drop our redundant handle.
            NtClose(fresh);
            winner as HANDLE
        }
    }
}

/// Byte-compare `addr` against `cmp` for `size` in {1,2,4,8}. Any other size is
/// rejected by the caller before we get here.
#[inline]
unsafe fn addr_equals(addr: *const c_void, cmp: *const c_void, size: usize) -> bool {
    match size {
        1 => *(addr as *const u8) == *(cmp as *const u8),
        2 => *(addr as *const u16) == *(cmp as *const u16),
        4 => *(addr as *const u32) == *(cmp as *const u32),
        8 => *(addr as *const u64) == *(cmp as *const u64),
        _ => false,
    }
}

/// Unlink `entry` from `b`'s list if present. Returns true if it was found and
/// removed. Caller must hold `b`'s spin lock.
#[inline]
unsafe fn unlink_if_present(b: &Bucket, entry: *mut AddrWaitEntry) -> bool {
    let mut cur = b.Head.load(Ordering::Relaxed);
    let mut prev: *mut AddrWaitEntry = core::ptr::null_mut();
    while !cur.is_null() {
        if cur == entry {
            let next = (*cur).Next;
            if prev.is_null() {
                b.Head.store(next, Ordering::Relaxed);
            } else {
                (*prev).Next = next;
            }
            return true;
        }
        prev = cur;
        cur = (*cur).Next;
    }
    false
}

/// `RtlWaitOnAddress` — block until the value at `Address` differs from the
/// value at `CompareAddress` (checked under the bucket lock so a wake can't slip
/// between the compare and the park), a wake targets `Address`, or `Timeout`
/// elapses. `Size` must be 1, 2, 4, or 8.
#[no_mangle]
pub unsafe extern "system" fn RtlWaitOnAddress(
    Address: *const c_void,
    CompareAddress: *const c_void,
    Size: usize,
    Timeout: *const LARGE_INTEGER,
) -> NTSTATUS {
    if Size != 1 && Size != 2 && Size != 4 && Size != 8 {
        return STATUS_INVALID_PARAMETER;
    }

    let keyed_event = ensure_keyed_event();
    let b = bucket_for(Address);

    let mut entry = AddrWaitEntry {
        Next: core::ptr::null_mut(),
        Address,
        Signaled: 0,
    };
    let entry_ptr = core::ptr::addr_of_mut!(entry);

    // Insert + compare under the lock: if the value already changed we never
    // park (and never touch the keyed event), so pairing stays trivially exact.
    spin_lock(b);
    if !addr_equals(Address, CompareAddress, Size) {
        spin_unlock(b);
        return STATUS_SUCCESS;
    }
    // Through entry_ptr, not `entry`: wakers read the entry via that pointer, and a
    // write through the parent place would invalidate it (and rustc then calls the
    // store dead).
    (*entry_ptr).Next = b.Head.load(Ordering::Relaxed);
    b.Head.store(entry_ptr, Ordering::Relaxed);
    spin_unlock(b);

    // Park. The key is our own entry address — unique to this waiter.
    let status = NtWaitForKeyedEvent(
        keyed_event,
        entry_ptr as *mut c_void,
        0u8,
        Timeout as *mut LARGE_INTEGER,
    );

    // Resolve ownership of the entry under the lock.
    spin_lock(b);
    if unlink_if_present(b, entry_ptr) {
        // We got the lock before any waker claimed us: no release was or will be
        // issued for this entry. Return whatever the park reported.
        spin_unlock(b);
        status
    } else {
        // A waker already unlinked us (Signaled==1) and has released, or is about
        // to release, the keyed event for our key. We MUST consume exactly one
        // release to keep the pairing exact.
        spin_unlock(b);
        if status != STATUS_SUCCESS {
            // We timed out / were alerted before the wake landed; drain the
            // guaranteed in-flight release with an infinite wait (it cannot
            // block indefinitely — the waker is committed to releasing).
            NtWaitForKeyedEvent(
                keyed_event,
                entry_ptr as *mut c_void,
                0u8,
                core::ptr::null_mut(),
            );
        }
        // A wake really did target us, so report success.
        STATUS_SUCCESS
    }
}

/// Shared wake path: unlink up to (all ? every : one) waiter(s) on `Address`
/// from its bucket, marking each `Signaled`, then release the keyed event for
/// each OUTSIDE the bucket lock (release can block until the waiter parks, which
/// must not stall other threads spinning on the bucket).
#[inline]
unsafe fn wake_address(Address: *const c_void, all: bool) {
    let keyed_event = KEYED_EVENT.load(Ordering::Acquire);
    if keyed_event.is_null() {
        // Nobody has ever waited (the event is created on first wait), so there
        // is nothing to wake.
        return;
    }
    let b = bucket_for(Address);

    // Collect the entries to wake under the lock; a small fixed batch avoids any
    // allocation. When waking all, loop until the bucket yields no more matches.
    loop {
        let mut batch: [*mut AddrWaitEntry; 16] = [core::ptr::null_mut(); 16];
        let mut n = 0usize;

        spin_lock(b);
        let mut cur = b.Head.load(Ordering::Relaxed);
        let mut prev: *mut AddrWaitEntry = core::ptr::null_mut();
        while !cur.is_null() && n < batch.len() {
            let next = (*cur).Next;
            if (*cur).Address == Address {
                // Unlink and claim.
                if prev.is_null() {
                    b.Head.store(next, Ordering::Relaxed);
                } else {
                    (*prev).Next = next;
                }
                (*cur).Signaled = 1;
                batch[n] = cur;
                n += 1;
                if !all {
                    break;
                }
                // prev stays; advance cur without moving prev past a removed node.
                cur = next;
            } else {
                prev = cur;
                cur = next;
            }
        }
        spin_unlock(b);

        // Release outside the lock.
        let mut i = 0;
        while i < n {
            NtReleaseKeyedEvent(
                keyed_event as HANDLE,
                batch[i] as *mut c_void,
                0u8,
                core::ptr::null_mut(),
            );
            i += 1;
        }

        // Single-wake, or a batch that didn't fill (list drained) -> done.
        if !all || n < batch.len() {
            break;
        }
    }
}

/// `RtlWakeAddressSingle` — wake at most one waiter blocked on `Address`.
#[no_mangle]
pub unsafe extern "system" fn RtlWakeAddressSingle(Address: *const c_void) {
    wake_address(Address, false);
}

/// `RtlWakeAddressAll` — wake every waiter blocked on `Address`.
#[no_mangle]
pub unsafe extern "system" fn RtlWakeAddressAll(Address: *const c_void) {
    wake_address(Address, true);
}
