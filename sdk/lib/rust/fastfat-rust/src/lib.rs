//! fastfat-rust — drivers/filesystems/fastfat → Rust (DeepSeek mechanical + c2rust).
//! The FAT filesystem (MS-architecture, 34 .c / 66.8K LOC). WDM kernelmode FS
//! driver, DriverEntry-only cohab. arm64 is the boot-exercised arch (ramdisk.img
//! is FAT32, the live system volume).
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
#![allow(unused_imports, unused_variables, unused_assignments, unused_mut, unused_unsafe)]

// NT_ASSERT is a checked-build assertion (wdm.h): its operand is never evaluated in a free
// build. It was an extern `fn`, so every call evaluated its argument out of line (cachesup.rs
// documents a fault that caused). `debug_assert!` type-checks it and evaluates nothing here.
#[macro_export]
macro_rules! NT_ASSERT { ($($t:tt)*) => { ntkrnl_sys::nt_assert!($($t)*) }; }

// ASSERT is an alias for NT_ASSERT, available from translated code that expected the name.
#[macro_export]
macro_rules! ASSERT { ($($t:tt)*) => { ntkrnl_sys::nt_assert!($($t)*) }; }

// UNREFERENCED_PARAMETER is a macro in the DDK headers; an out-of-line no-op fn is an
// optimization barrier that can uncloak an ABI mismatch.
#[allow(unused_macros)]
macro_rules! UNREFERENCED_PARAMETER {
    ($($p:expr),* $(,)?) => {{ $( let _ = $p; )* }};
}

// PAGED_CODE() is the DDK's "pageable code" IRQL assertion (xdk/kefuncs.h): NOP_FUNCTION in
// a free build, and a no-op here. It is a macro, never a fn -- an out-of-line no-op call is an
// optimization barrier, and an `extern` one resolved by whichever crate happened to export it.
macro_rules! PAGED_CODE {
    () => {};
}

// Debug timing macros — discarded in free builds. Macros, never out-of-line functions.
#[macro_export]
macro_rules! TimerStart {
    ($($arg:tt)*) => {};
}
#[macro_export]
macro_rules! TimerStop {
    ($($arg:tt)*) => {};
}

// KdPrintEx — debug print macro discarded in free builds. A macro, never an out-of-line fn.
#[macro_export]
macro_rules! KdPrintEx {
    ($($arg:tt)*) => {};
}

// crate-type = staticlib (cohab) → this PE module needs its own panic handler /
// eh personality (each .sys links independently of ntoskrnl). Matches disk-rust.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

// arm64: KeGetCurrentPrcb is not exported (it's a register-reading FORCEINLINE).
// fastfat reaches it via the ntkrnl_sys inline KeGetCurrentThread path; forward the
// symbol to the register-reading inline, as mup-rust / classpnp-rust do.
#[cfg(all(not(test), target_arch = "aarch64"))]
#[no_mangle]
pub unsafe extern "C" fn KeGetCurrentPrcb() -> ntkrnl_sys::ddk::PKPRCB {
    ntkrnl_sys::inline::ke::KeGetCurrentPrcb()
}

pub mod types;
pub mod glue;       // harvested #define consts
pub mod ddk_shims;  // macro/FORCEINLINE shims
pub mod seh;        // real __try/__except/__finally (ntkrnl_sys::seh)

// CD-ROM TOC structures (from ntddcdrm.h), used by FatScanForDataTrack.
#[repr(C)]
pub struct CDROM_TOC_TRACK_DATA {
    pub Reserved: u8,
    /// Packed byte: bits 3:0 = Control, bits 7:4 = Adr
    pub Control: u8,
    pub TrackNumber: u8,
    pub Reserved1: u8,
    pub Address: [u8; 4],
}
#[repr(C)]
pub struct CDROM_TOC {
    pub Length: [u8; 2],
    pub FirstTrack: u8,
    pub LastTrack: u8,
    pub TrackData: [CDROM_TOC_TRACK_DATA; 100],
}

// Translated modules (one per C source).
pub mod acchksup;
pub mod allocsup;
pub mod cachesup;
pub mod cleanup;
pub mod close;
pub mod create;
pub mod devctrl;
pub mod deviosup;
pub mod dirctrl;
pub mod dirsup;
pub mod fileid;    // fatprocs.h FatGenerateFileId* macros (were bugcheck stubs)
pub mod dumpsup;
pub mod ea;
pub mod easup;
pub mod fatdata;
pub mod fatinit;
pub mod fileinfo;
pub mod filobsup;
pub mod flush;
pub mod fsctrl;
pub mod fspdisp;
pub mod lockctrl;
pub mod namesup;
pub mod pnp;
pub mod read;
pub mod resrcsup;
pub mod shutdown;
pub mod splaysup;
pub mod strucsup;
pub mod timesup;
pub mod verfysup;
pub mod volinfo;
pub mod workque;
pub mod write;
