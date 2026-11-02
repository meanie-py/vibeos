//! disk-rust — drivers/storage/class/disk → Rust (DeepSeek mechanical + c2rust).
//! SCSI disk class driver built on classpnp's Class* exports.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
#![allow(unused_imports, unused_variables, unused_assignments, unused_mut)]


// cdrom.h / classp.h `FREE_POOL(p)`: `if (p != NULL) { ExFreePool(p); p = NULL; }`. A MACRO, because
// it NULLS ITS ARGUMENT; the `fn FREE_POOL(PVOID)` the translation used freed and left the place
// dangling (cdrom's DeviceInitializeGesn left Gesn.Buffer pointing at freed pool, caught by its
// NT_ASSERT once the crate was armed). Frees through the crate's FREE_POOL fn, then nulls.
#[allow(unused_macros)]
macro_rules! FREE_POOL {
    ($p:expr) => {{
        let __freed = ($p) as *mut core::ffi::c_void;
        if !__freed.is_null() {
            FREE_POOL(__freed);
            $p = core::ptr::null_mut();
        }
    }};
}

// NT_ASSERT is a checked-build assertion (wdm.h): its operand is never evaluated in a free
// build. It was an extern `fn`, so every call evaluated its argument out of line (cachesup.rs
// documents a fault that caused). `debug_assert!` type-checks it and evaluates nothing here.
#[macro_export]
macro_rules! NT_ASSERT { ($($t:tt)*) => { ntkrnl_sys::nt_assert!($($t)*) }; }

// PAGED_CODE() is the DDK's "pageable code" IRQL assertion (xdk/kefuncs.h): NOP_FUNCTION in
// a free build, and a no-op here. It is a macro, never a fn -- an out-of-line no-op call is an
// optimization barrier, and an `extern` one resolved by whichever crate happened to export it.
macro_rules! PAGED_CODE {
    () => {};
}

// Fold-proof trace shims (TracePrint/StorDebugPrint) — see drvtrace-shim/src/lib.rs.
extern crate drvtrace_shim;

// crate-type = staticlib (cohab) → this PE module needs its own panic handler /
// eh personality (each .sys links independently of ntoskrnl). Matches classpnp-rust.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

pub mod types;
pub mod data;
pub mod glue;
pub mod ddk_shims;

pub mod disk;
pub mod diskwmi;
pub mod geometry;
pub mod pnp;
