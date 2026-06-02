//! Constants and FFI struct layout for `sdk/lib/rtl/resource.c`.
//!
//! Wave RES is pure plumbing — every fn chains `Rtl*CriticalSection`
//! and `Nt*Semaphore` / `NtClose` calls. There's no algorithmic
//! content to host in `core_rs`; this module just centralises the
//! `RTL_RESOURCE` layout and the few constants the cabi shims need.
//!
//! Faithful to ReactOS `sdk/lib/rtl/resource.c` (Wine-derived; Marcus
//! Meissner / Alex Korobka).

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

/// `SEMAPHORE_ALL_ACCESS` (winnt.h) — `STANDARD_RIGHTS_REQUIRED
/// (0xF0000) | SYNCHRONIZE (0x100000) | 0x3` = `0x1F0003`.
pub const SEMAPHORE_ALL_ACCESS: u32 = 0x1F_0003;

/// `STATUS_WAIT_0` — `NtWaitForSingleObject` success sentinel used by
/// `RtlAcquireResourceShared`'s "already woken by `RtlReleaseResource`"
/// branch (set when waking out of the shared-semaphore queue).
pub const STATUS_WAIT_0: i32 = 0;

/// `RTL_CRITICAL_SECTION` layout-match (ndk/rtltypes.h, NDK baseline).
/// `Lock` is embedded by value in `RTL_RESOURCE`; we just need the
/// right footprint + alignment for the per-arch packing of the
/// containing struct. ReactOS treats this as opaque from C too —
/// callers only pass `&Lock` to `RtlInitializeCriticalSection` etc.
#[repr(C)]
pub struct RTL_CRITICAL_SECTION_OPAQUE {
    pub DebugInfo: *mut u8,
    pub LockCount: i32,
    pub RecursionCount: i32,
    pub OwningThread: *mut u8,
    pub LockSemaphore: *mut u8,
    pub SpinCount: usize,
}

/// `RTL_RESOURCE` layout-match (ndk/rtltypes.h:1535). Lock at offset 0
/// is critical — `&Resource->Lock` is passed to the critsec APIs.
#[repr(C)]
pub struct RTL_RESOURCE {
    pub Lock: RTL_CRITICAL_SECTION_OPAQUE,
    pub SharedSemaphore: *mut u8,
    pub SharedWaiters: u32,
    pub ExclusiveSemaphore: *mut u8,
    pub ExclusiveWaiters: u32,
    pub NumberActive: i32,
    pub OwningThread: *mut u8,
    pub TimeoutBoost: u32,
    pub DebugInfo: *mut u8,
}

// --- TEB.ClientId.UniqueThread offset — per-arch. ClientId at
// TEB+0x20 (i386) / TEB+0x40 (amd64+arm64); UniqueThread is the
// second HANDLE in CLIENT_ID (+ pointer-width into ClientId).
#[cfg(target_arch = "x86")]
pub const TEB_UNIQUE_THREAD_OFFSET: usize = 0x20 + 4;
#[cfg(target_arch = "x86_64")]
pub const TEB_UNIQUE_THREAD_OFFSET: usize = 0x40 + 8;
#[cfg(target_arch = "aarch64")]
pub const TEB_UNIQUE_THREAD_OFFSET: usize = 0x40 + 8;

// --- pure decision kernels --------------------------------------------------
//
// The RTL_RESOURCE acquire/release state machines are testable as pure logic:
// they describe transitions of (NumberActive, SharedWaiters, ExclusiveWaiters,
// OwningThread) given the lock state, the caller's thread id, and whether
// `Wait==TRUE`. The cabi shim chains these decisions with semaphore wait /
// release / critsec acquire syscalls — the SYSCALL coupling is what the smoke
// covers; the DECISIONS are what these tests cover.
//
// Faithful to ReactOS `sdk/lib/rtl/resource.c` (Wine derived).

/// Snapshot of the mutable fields of `RTL_RESOURCE` that any of the
/// state-machine transitions read or write.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResourceState {
    pub number_active: i32,
    pub shared_waiters: u32,
    pub exclusive_waiters: u32,
    /// Thread-id token — `0` is the C `NULL`/"no owner" sentinel.
    pub owning_thread: usize,
}

/// What `RtlAcquireResourceExclusive` decides to do at the *first*
/// check (before any wait). The cabi shim either:
///   - returns the boolean directly, OR
///   - performs the wait if `WaitForSemaphore`, then re-enters the state.
#[derive(Debug, PartialEq, Eq)]
pub enum AcquireExclusiveDecision {
    /// Acquired (or recursive-acquired): set state, return TRUE.
    AcquiredSet(ResourceState),
    /// Blocked: increment ExclusiveWaiters, release the critsec, wait
    /// on the exclusive semaphore, restart.
    WaitForSemaphore(ResourceState),
    /// Not blocked, not acquired (Wait==FALSE): return FALSE.
    Refused,
}

/// Pure: decide what `RtlAcquireResourceExclusive` does *at this entry*.
/// Faithful to lines 92-127 of `sdk/lib/rtl/resource.c`.
pub fn acquire_exclusive_decision(
    state: ResourceState,
    current_thread: usize,
    wait: bool,
) -> AcquireExclusiveDecision {
    if state.number_active == 0 {
        // Lock is free — acquire it.
        return AcquireExclusiveDecision::AcquiredSet(ResourceState {
            number_active: -1,
            owning_thread: current_thread,
            ..state
        });
    }
    if state.number_active < 0 {
        // Exclusive lock in progress.
        if state.owning_thread == current_thread {
            // Recursive: decrement (more negative), keep owner.
            return AcquireExclusiveDecision::AcquiredSet(ResourceState {
                number_active: state.number_active - 1,
                ..state
            });
        }
        // Someone else owns it.
        if wait {
            return AcquireExclusiveDecision::WaitForSemaphore(ResourceState {
                exclusive_waiters: state.exclusive_waiters + 1,
                ..state
            });
        }
        return AcquireExclusiveDecision::Refused;
    }
    // Shared locks active.
    if wait {
        return AcquireExclusiveDecision::WaitForSemaphore(ResourceState {
            exclusive_waiters: state.exclusive_waiters + 1,
            ..state
        });
    }
    AcquireExclusiveDecision::Refused
}

/// What `RtlAcquireResourceShared` decides at first entry.
#[derive(Debug, PartialEq, Eq)]
pub enum AcquireSharedDecision {
    /// Acquired (incl. recursive on the writer-thread): set state, return TRUE.
    AcquiredSet(ResourceState),
    /// Blocked: bump SharedWaiters, wait on shared semaphore.
    WaitForSemaphore(ResourceState),
    /// Refused (Wait==FALSE, writer-locked, not by us).
    Refused,
}

/// Pure: decide what `RtlAcquireResourceShared` does *at this entry*.
/// `previous_status_is_wait_0 == true` if we arrived here as a
/// restart after `NtWaitForSingleObject` woke us via the shared
/// semaphore — in which case `RtlReleaseResource` already accounted
/// for our slot and we must not double-count.
///
/// Faithful to lines 137-178 of `sdk/lib/rtl/resource.c`.
pub fn acquire_shared_decision(
    state: ResourceState,
    current_thread: usize,
    wait: bool,
    previous_status_is_wait_0: bool,
) -> AcquireSharedDecision {
    if state.number_active < 0 {
        if state.owning_thread == current_thread {
            // Recursive acquire as shared on the writer thread.
            return AcquireSharedDecision::AcquiredSet(ResourceState {
                number_active: state.number_active - 1,
                ..state
            });
        }
        if wait {
            return AcquireSharedDecision::WaitForSemaphore(ResourceState {
                shared_waiters: state.shared_waiters + 1,
                ..state
            });
        }
        return AcquireSharedDecision::Refused;
    }
    // number_active >= 0 — already in shared mode (or free).
    let new_n = if previous_status_is_wait_0 {
        state.number_active
    } else {
        state.number_active + 1
    };
    AcquireSharedDecision::AcquiredSet(ResourceState {
        number_active: new_n,
        ..state
    })
}

/// What `RtlReleaseResource` does. The C body branches on
/// number_active sign and the waiter counts. Pure.
#[derive(Debug, PartialEq, Eq)]
pub enum ReleaseDecision {
    /// Just update state; no semaphore release.
    UpdateOnly(ResourceState),
    /// Wake one exclusive waiter (semaphore release count = 1).
    WakeOneExclusive(ResourceState),
    /// Wake `n` shared waiters (semaphore release count = n).
    WakeNShared { state: ResourceState, n: u32 },
}

/// Pure: decide what `RtlReleaseResource` does.
/// Faithful to lines 256-302 of `sdk/lib/rtl/resource.c`.
pub fn release_decision(state: ResourceState) -> ReleaseDecision {
    if state.number_active > 0 {
        // Reader leaving.
        let after = state.number_active - 1;
        if after == 0 && state.exclusive_waiters > 0 {
            return ReleaseDecision::WakeOneExclusive(ResourceState {
                number_active: 0,
                exclusive_waiters: state.exclusive_waiters - 1,
                ..state
            });
        }
        return ReleaseDecision::UpdateOnly(ResourceState {
            number_active: after,
            ..state
        });
    }
    if state.number_active < 0 {
        // Writer leaving (possibly recursive).
        let after = state.number_active + 1;
        if after != 0 {
            // Still recursive-held by us.
            return ReleaseDecision::UpdateOnly(ResourceState {
                number_active: after,
                ..state
            });
        }
        // Final release of the writer.
        let no_owner = ResourceState {
            number_active: 0,
            owning_thread: 0,
            ..state
        };
        if state.exclusive_waiters > 0 {
            return ReleaseDecision::WakeOneExclusive(ResourceState {
                exclusive_waiters: state.exclusive_waiters - 1,
                ..no_owner
            });
        }
        if state.shared_waiters > 0 {
            let n = state.shared_waiters;
            return ReleaseDecision::WakeNShared {
                state: ResourceState {
                    number_active: n as i32,
                    shared_waiters: 0,
                    ..no_owner
                },
                n,
            };
        }
        return ReleaseDecision::UpdateOnly(no_owner);
    }
    // number_active == 0 — release on unheld lock; no-op.
    ReleaseDecision::UpdateOnly(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    const T_A: usize = 0xA;
    const T_B: usize = 0xB;
    const ST_FREE: ResourceState = ResourceState {
        number_active: 0,
        shared_waiters: 0,
        exclusive_waiters: 0,
        owning_thread: 0,
    };

    // --- AcquireExclusive ---

    #[test]
    fn acq_excl_free_acquires() {
        let r = acquire_exclusive_decision(ST_FREE, T_A, true);
        match r {
            AcquireExclusiveDecision::AcquiredSet(s) => {
                assert_eq!(s.number_active, -1);
                assert_eq!(s.owning_thread, T_A);
            }
            _ => panic!("expected AcquiredSet"),
        }
    }

    #[test]
    fn acq_excl_recursive_self_no_wait_needed() {
        let st = ResourceState {
            number_active: -1,
            owning_thread: T_A,
            ..ST_FREE
        };
        let r = acquire_exclusive_decision(st, T_A, false);
        assert_eq!(
            r,
            AcquireExclusiveDecision::AcquiredSet(ResourceState {
                number_active: -2,
                owning_thread: T_A,
                ..ST_FREE
            })
        );
    }

    #[test]
    fn acq_excl_other_writer_with_wait_queues() {
        let st = ResourceState {
            number_active: -1,
            owning_thread: T_B,
            ..ST_FREE
        };
        let r = acquire_exclusive_decision(st, T_A, true);
        match r {
            AcquireExclusiveDecision::WaitForSemaphore(s) => {
                assert_eq!(s.exclusive_waiters, 1);
                assert_eq!(s.owning_thread, T_B);
            }
            _ => panic!("expected WaitForSemaphore"),
        }
    }

    #[test]
    fn acq_excl_other_writer_no_wait_refuses() {
        let st = ResourceState {
            number_active: -1,
            owning_thread: T_B,
            ..ST_FREE
        };
        let r = acquire_exclusive_decision(st, T_A, false);
        assert_eq!(r, AcquireExclusiveDecision::Refused);
    }

    #[test]
    fn acq_excl_shared_in_progress_waits() {
        let st = ResourceState {
            number_active: 3,
            ..ST_FREE
        };
        let r = acquire_exclusive_decision(st, T_A, true);
        match r {
            AcquireExclusiveDecision::WaitForSemaphore(s) => {
                assert_eq!(s.exclusive_waiters, 1);
            }
            _ => panic!("expected WaitForSemaphore"),
        }
    }

    // --- AcquireShared ---

    #[test]
    fn acq_shared_free_first_reader() {
        let r = acquire_shared_decision(ST_FREE, T_A, true, false);
        assert_eq!(
            r,
            AcquireSharedDecision::AcquiredSet(ResourceState {
                number_active: 1,
                ..ST_FREE
            })
        );
    }

    #[test]
    fn acq_shared_existing_readers_increments() {
        let st = ResourceState {
            number_active: 5,
            ..ST_FREE
        };
        let r = acquire_shared_decision(st, T_A, true, false);
        assert_eq!(
            r,
            AcquireSharedDecision::AcquiredSet(ResourceState {
                number_active: 6,
                ..ST_FREE
            })
        );
    }

    #[test]
    fn acq_shared_woken_from_release_does_not_double_count() {
        let st = ResourceState {
            number_active: 3,
            ..ST_FREE
        };
        let r = acquire_shared_decision(st, T_A, true, /*wait_0*/ true);
        assert_eq!(
            r,
            AcquireSharedDecision::AcquiredSet(ResourceState {
                number_active: 3, // unchanged — already counted in by release
                ..ST_FREE
            })
        );
    }

    #[test]
    fn acq_shared_writer_holds_other_thread_waits() {
        let st = ResourceState {
            number_active: -1,
            owning_thread: T_B,
            ..ST_FREE
        };
        let r = acquire_shared_decision(st, T_A, true, false);
        match r {
            AcquireSharedDecision::WaitForSemaphore(s) => {
                assert_eq!(s.shared_waiters, 1);
            }
            _ => panic!("expected WaitForSemaphore"),
        }
    }

    #[test]
    fn acq_shared_writer_holds_self_recurses() {
        let st = ResourceState {
            number_active: -1,
            owning_thread: T_A,
            ..ST_FREE
        };
        let r = acquire_shared_decision(st, T_A, false, false);
        assert_eq!(
            r,
            AcquireSharedDecision::AcquiredSet(ResourceState {
                number_active: -2,
                owning_thread: T_A,
                ..ST_FREE
            })
        );
    }

    // --- Release ---

    #[test]
    fn release_one_reader_no_waiter_just_decrements() {
        let st = ResourceState {
            number_active: 2,
            ..ST_FREE
        };
        let r = release_decision(st);
        assert_eq!(
            r,
            ReleaseDecision::UpdateOnly(ResourceState {
                number_active: 1,
                ..ST_FREE
            })
        );
    }

    #[test]
    fn release_last_reader_wakes_exclusive_waiter() {
        let st = ResourceState {
            number_active: 1,
            exclusive_waiters: 1,
            ..ST_FREE
        };
        let r = release_decision(st);
        assert_eq!(
            r,
            ReleaseDecision::WakeOneExclusive(ResourceState {
                number_active: 0,
                exclusive_waiters: 0,
                ..ST_FREE
            })
        );
    }

    #[test]
    fn release_writer_recursive_just_decrements() {
        let st = ResourceState {
            number_active: -2,
            owning_thread: T_A,
            ..ST_FREE
        };
        let r = release_decision(st);
        assert_eq!(
            r,
            ReleaseDecision::UpdateOnly(ResourceState {
                number_active: -1,
                owning_thread: T_A,
                ..ST_FREE
            })
        );
    }

    #[test]
    fn release_writer_final_wakes_exclusive_when_pending() {
        let st = ResourceState {
            number_active: -1,
            owning_thread: T_A,
            exclusive_waiters: 2,
            ..ST_FREE
        };
        let r = release_decision(st);
        assert_eq!(
            r,
            ReleaseDecision::WakeOneExclusive(ResourceState {
                number_active: 0,
                owning_thread: 0,
                exclusive_waiters: 1,
                ..ST_FREE
            })
        );
    }

    #[test]
    fn release_writer_final_wakes_n_shared_when_no_excl_pending() {
        let st = ResourceState {
            number_active: -1,
            owning_thread: T_A,
            shared_waiters: 3,
            ..ST_FREE
        };
        let r = release_decision(st);
        assert_eq!(
            r,
            ReleaseDecision::WakeNShared {
                state: ResourceState {
                    number_active: 3,
                    owning_thread: 0,
                    shared_waiters: 0,
                    ..ST_FREE
                },
                n: 3,
            }
        );
    }
}
