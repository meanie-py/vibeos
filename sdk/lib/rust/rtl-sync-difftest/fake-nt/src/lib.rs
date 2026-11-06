//! Just the `ntkrnl_sys` surface that rtl-sync's `srw.rs` and `atomics.rs` reach, with the same
//! names and Windows widths (`LONG` is 32-bit here too), so the shipping files compile unchanged.
#![allow(non_snake_case, non_camel_case_types, clippy::missing_safety_doc)]

pub mod base {
    pub use core::ffi::c_void;
    pub type NTSTATUS = i32;
    pub type PVOID = *mut c_void;
    pub type HANDLE = PVOID;
    pub type LONG = i32;
    pub type BOOLEAN = u8;
    pub type LONG_PTR = isize;
}

pub mod interlocked {
    use super::base::PVOID;
    use core::sync::atomic::{AtomicPtr, Ordering};

    pub unsafe fn InterlockedExchangePointer(Target: *mut PVOID, Value: PVOID) -> PVOID {
        AtomicPtr::from_ptr(Target).swap(Value, Ordering::SeqCst)
    }

    pub unsafe fn InterlockedCompareExchangePointer(
        Destination: *mut PVOID,
        Exchange: PVOID,
        Comparand: PVOID,
    ) -> PVOID {
        match AtomicPtr::from_ptr(Destination).compare_exchange(
            Comparand,
            Exchange,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(v) | Err(v) => v,
        }
    }
}

pub mod rtl {
    pub unsafe fn RtlRaiseStatus(Status: super::base::NTSTATUS) -> ! {
        panic!("RtlRaiseStatus({:#x})", Status as u32);
    }
}

pub mod inline {
    pub mod ke {
        pub unsafe fn YieldProcessor() {
            core::hint::spin_loop();
        }
    }
}

pub mod ddk {
    use super::base::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Condvar, Mutex};

    #[repr(C)]
    pub struct _RTL_SRWLOCK {
        pub Ptr: PVOID,
    }
    pub type RTL_SRWLOCK = _RTL_SRWLOCK;
    pub type PRTL_SRWLOCK = *mut _RTL_SRWLOCK;

    #[derive(Clone, Copy)]
    pub struct LARGE_INTEGER {
        pub QuadPart: i64,
    }

    pub unsafe fn NtYieldExecution() -> NTSTATUS {
        std::thread::yield_now();
        0
    }

    pub unsafe fn NtDelayExecution(_Alertable: BOOLEAN, Interval: *mut LARGE_INTEGER) -> NTSTATUS {
        let ticks = (-(*Interval).QuadPart).max(0) as u64; // relative, 100 ns units
        std::thread::sleep(std::time::Duration::from_nanos(ticks * 100));
        0
    }

    /// Per key: waiters parked, releases parked, and grants handed to each side. A wait meets a
    /// parked release or parks; a release meets a parked wait or parks until one arrives, which
    /// is ex-rust keyedevt.rs `ExpReleaseOrWaitForKeyedEvent` (one process, so no process key).
    #[derive(Default)]
    struct Slot {
        waiting: u64,
        releasing: u64,
        wait_grants: u64,
        release_grants: u64,
    }

    static TABLE: Mutex<Option<HashMap<usize, Slot>>> = Mutex::new(None);
    static CHANGED: Condvar = Condvar::new();
    /// How many waits blocked in the kernel, for the harness to prove the path ran.
    pub static KEYED_WAITS: AtomicU64 = AtomicU64::new(0);
    pub static KEYED_RELEASES: AtomicU64 = AtomicU64::new(0);

    fn rendezvous(key: usize, release: bool) {
        let mut g = TABLE.lock().unwrap();
        let slot = g.get_or_insert_with(HashMap::new).entry(key).or_default();
        let (mine_parked, theirs_parked) = if release {
            (slot.releasing, slot.waiting)
        } else {
            (slot.waiting, slot.releasing)
        };
        let _ = mine_parked;
        if theirs_parked > 0 {
            if release {
                slot.waiting -= 1;
                slot.wait_grants += 1;
            } else {
                slot.releasing -= 1;
                slot.release_grants += 1;
            }
            CHANGED.notify_all();
            return;
        }
        if release {
            slot.releasing += 1;
        } else {
            slot.waiting += 1;
        }
        loop {
            g = CHANGED.wait(g).unwrap();
            let slot = g.as_mut().unwrap().get_mut(&key).unwrap();
            let grants = if release {
                &mut slot.release_grants
            } else {
                &mut slot.wait_grants
            };
            if *grants > 0 {
                *grants -= 1;
                return;
            }
        }
    }

    pub unsafe fn NtWaitForKeyedEvent(
        _Handle: HANDLE,
        Key: PVOID,
        _Alertable: BOOLEAN,
        _Timeout: *mut LARGE_INTEGER,
    ) -> NTSTATUS {
        assert!(Key as usize & 1 == 0, "keyed-event keys must be even");
        KEYED_WAITS.fetch_add(1, Ordering::Relaxed);
        rendezvous(Key as usize, false);
        0
    }

    pub unsafe fn NtReleaseKeyedEvent(
        _Handle: HANDLE,
        Key: PVOID,
        _Alertable: BOOLEAN,
        _Timeout: *mut LARGE_INTEGER,
    ) -> NTSTATUS {
        assert!(Key as usize & 1 == 0, "keyed-event keys must be even");
        KEYED_RELEASES.fetch_add(1, Ordering::Relaxed);
        rendezvous(Key as usize, true);
        0
    }
}
