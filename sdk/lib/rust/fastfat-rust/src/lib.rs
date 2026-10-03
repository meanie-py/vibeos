//! fastfat-rust — drivers/filesystems/fastfat → Rust (DeepSeek mechanical + c2rust).
//! The FAT filesystem (MS-architecture, 34 .c / 66.8K LOC). WDM kernelmode FS
//! driver, DriverEntry-only cohab. arm64 is the boot-exercised arch (ramdisk.img
//! is FAT32, the live system volume).
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
#![allow(unused_imports, unused_variables, unused_assignments, unused_mut, unused_unsafe)]

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

/// `#if (NTDDI_VERSION >= NTDDI_WIN8)` in the C. fastfat builds at `_WIN32_WINNT=0x502`, so every
/// such block is compiled OUT, and rust_cohab_fat.c's `FatGetFcbOplock` is the pre-Win8
/// `&Fcb->Specific.Fcb.Oplock` -- which on a DCB is the DIRECTORY half of the union. A Win8 block
/// run unconditionally hands the kernel directory fields as an oplock: `FsRtlCheckOplock` faulted
/// at `*Oplock`'s 0x33 on the parent-directory check of every create (bugcheck 0x23), and the DCB
/// oplock initializers wrote a NULL over live directory state. Gate Win8 blocks on this. (Older
/// gates in cleanup.rs/deviosup.rs spell it as the never-set `#[cfg(ntddi_ge_*win8)]`.)
pub(crate) const NTDDI_WIN8: bool = false;
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
