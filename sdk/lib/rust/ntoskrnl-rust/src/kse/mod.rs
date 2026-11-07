//! Kernel Shim Engine (upstream ntoskrnl/kse/, 2026-10): driver-database matching
//! (drvmain.sdb from the loader block) plus the DriverScope, IgnoreFreePoolTag and
//! KmWin2k3VersionLie shims. Translated with rustconv (Haiku); ksedb and the
//! functions c2rust could not lower (`_ReturnAddress` users, two call-through
//! wrappers) are hand-written. Every entry point is a `#[no_mangle]` link symbol:
//! iomgr-rust and the loader reach it across the kernel link, as upstream's C does.
/// debug.h DPRINT1 (the KSE C defines NDEBUG, which silences DPRINT but never
/// DPRINT1). The translated bodies pass the format as a `*const` pointer, so
/// this forwards straight to DbgPrint. A macro, never a no-op fn.
macro_rules! DPRINT1 {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {{
        ntkrnl_sys::driver_ddk::DbgPrint($fmt $(, $arg)*);
    }};
}

/// debug.h DPRINT under NDEBUG (which the KSE C defines): prints nothing, but its
/// arguments stay type-checked so a broken one cannot hide in a silenced trace.
/// Same body as win32macros_rust::DBG_TYPECHECK! (not a dependency of this kernel crate).
macro_rules! DPRINT {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {{
        let _ = || {
            #[allow(unused_unsafe)]
            unsafe {
                let _ = $fmt;
                $(let _ = $arg;)*
            }
        };
    }};
}

pub mod defs;
pub mod kse;
pub mod ksedb;
pub mod driverscope;
pub mod ignorepooltag;
pub mod versionlie;
