//! compat.rs — centralized shim layer for names the per-module translations expect
//! but which the fan-out contract (`use ntkrnl_sys::ddk::*`) doesn't surface, plus the
//! handful of uniata header-inlines / macros (atapi.h, bsmaster.h) that the agents
//! couldn't translate because they live in .h, not a .cpp. Re-exported at crate root
//! (`pub use compat::*`) so every module sees these via `use crate::*;`.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused_unsafe)]

use crate::types::*;
use crate::regidx::*;
use crate::*;
use ntkrnl_sys::ddk::*;
use crate::io_port::AtapiReadPort1;

// --- kernel names that live outside ntkrnl_sys::ddk -------------------------
pub use ntkrnl_sys::base::{TRUE, FALSE, STATUS_SUCCESS};
pub use ntkrnl_sys::constants::{STATUS_UNSUCCESSFUL, REG_NONE, REG_BINARY};
pub use ntkrnl_sys::mm::{NonPagedPool, PagedPool};
pub use ntkrnl_sys::rtl::{RtlZeroMemory, RtlCopyMemory};
pub use ntkrnl_sys::driver_ddk::{
    FILE_DEVICE_UNKNOWN, RTL_REGISTRY_SERVICES, RTL_QUERY_REGISTRY_REQUIRED,
};

// PCI_MAX_FUNCTION (wdm.h) — hal bindgen module is private, so define directly.
pub const PCI_MAX_FUNCTION: ULONG = 8;

// KeQuerySystemTime: a real exported fn on i386 (modules get it from ddk::*), but a
// SharedUserData inline on amd64/arm64 — re-export the inline ONLY there (re-exporting
// on i386 too would collide with the ddk glob → E0659).
#[cfg(not(target_arch = "x86"))]
pub use ntkrnl_sys::inline::ke::KeQuerySystemTime;

// amd64/arm64 cohab link trap: RtlUshort/UlongByteSwap are exported by ntoskrnl on i386
// but are FORCEINLINE-only on amd64/arm64 — provide the trivial implementations.
// These are NON-pub on purpose: #[no_mangle] emits the linker symbol (resolving the
// ddk extern decls the modules already call) while keeping the names OUT of the crate
// glob — making them pub would collide with ntkrnl_sys::ddk::* (E0659).

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
extern "C" fn RtlUshortByteSwap(s: USHORT) -> USHORT {
    s.swap_bytes()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
extern "C" fn RtlUlongByteSwap(s: ULONG) -> ULONG {
    s.swap_bytes()
}

// --- SMART / IDE command codes (ntdddisk.h #defines bindgen dropped) ---------
pub const ENABLE_DISABLE_AUTOSAVE: UCHAR = 0xD2;
pub const SAVE_ATTRIBUTE_VALUES: UCHAR = 0xD3;
pub const EXECUTE_OFFLINE_DIAGS: UCHAR = 0xD4;
pub const ENABLE_SMART: UCHAR = 0xD8;
pub const DISABLE_SMART: UCHAR = 0xD9;

// --- atapi.h type aliases (#define IDENTIFY_DATA2 IDENTIFY_DATA) --------------
pub type IDENTIFY_DATA2 = IDENTIFY_DATA;
pub type PIDENTIFY_DATA2 = PIDENTIFY_DATA;

// --- InterlockedExchange: not in ntkrnl_sys::interlocked --------------------
// Atomic swap of a 32-bit location; returns the old value.
#[inline]
pub unsafe extern "system" fn InterlockedExchange(Target: *mut LONG, Value: LONG) -> LONG {
    let a = &*(Target as *const core::sync::atomic::AtomicI32);
    a.swap(Value, core::sync::atomic::Ordering::SeqCst)
}

// --- GetDmaStatus(de, c) macro (bsmaster.h:1711) ----------------------------
//   (((de)->BusMaster == DMA_MODE_BM) ? AtapiReadPort1(&de->chan[c], IDX_BM_Status) : 0)
#[inline]
pub unsafe extern "system" fn GetDmaStatus(de: PHW_DEVICE_EXTENSION, c: ULONG) -> UCHAR {
    if (*de).BusMaster as ULONG == DMA_MODE_BM {
        AtapiReadPort1((*de).chan.add(c as usize), IDX_BM_Status)
    } else {
        0
    }
}

// --- UniAtaClearAtaReq(AtaReq) macro (atapi.h:1617) -------------------------
//   RtlZeroMemory(AtaReq, FIELD_OFFSET(ATA_REQ, ata));
// FIELD_OFFSET(ATA_REQ, ata) == offset of the ata/ahci union within the header
// struct, computed correct-by-construction from the bindgen layout.
#[inline]
pub unsafe extern "system" fn UniAtaClearAtaReq(AtaReq: PVOID) {
    let n = core::mem::offset_of!(_ATA_REQ__bindgen_ty_1__bindgen_ty_1, __bindgen_anon_1);
    core::ptr::write_bytes(AtaReq as *mut u8, 0, n);
}

// --- ata_cur_mode_from_ident (atapi.h:1650 __inline) ------------------------
// Decodes the current/maximum transfer mode from an IDENTIFY page.
pub const IDENT_MODE_MAX: BOOLEAN = FALSE;
pub const IDENT_MODE_ACTIVE: BOOLEAN = TRUE;

pub unsafe extern "system" fn ata_cur_mode_from_ident(ident: PIDENTIFY_DATA, Active: BOOLEAN) -> LONG {
    let mut mode: USHORT;
    if ata_is_sata(ident) != 0 {
        if ((*ident).SataCapabilities as ULONG & ATA_SATA_GEN3) != 0 {
            return ATA_SA600 as LONG;
        } else if ((*ident).SataCapabilities as ULONG & ATA_SATA_GEN2) != 0 {
            return ATA_SA300 as LONG;
        } else if ((*ident).SataCapabilities as ULONG & ATA_SATA_GEN1) != 0 {
            return ATA_SA150 as LONG;
        }
        return ATA_SA150 as LONG;
    }

    if (*ident).UdmaModesValid() != 0 {
        mode = if Active != 0 { (*ident).UltraDMAActive() } else { (*ident).UltraDMASupport() };
        if (mode & 0x40) != 0 { return (ATA_UDMA0 + 6) as LONG; }
        if (mode & 0x20) != 0 { return (ATA_UDMA0 + 5) as LONG; }
        if (mode & 0x10) != 0 { return (ATA_UDMA0 + 4) as LONG; }
        if (mode & 0x08) != 0 { return (ATA_UDMA0 + 3) as LONG; }
        if (mode & 0x04) != 0 { return (ATA_UDMA0 + 2) as LONG; }
        if (mode & 0x02) != 0 { return (ATA_UDMA0 + 1) as LONG; }
        if (mode & 0x01) != 0 { return (ATA_UDMA0 + 0) as LONG; }
    }

    // MultiWordDMASupport + SingleWordDMA* are bindgen-nested in the word-62/63 union.
    let dma1 = (*ident).__bindgen_anon_1.__bindgen_anon_1.as_ref();
    let dma2 = (*ident).__bindgen_anon_1.AtapiDMA.as_ref();
    let _ = if Active != 0 { (*ident).MultiWordDMAActive() } else { dma2.MultiWordDMASupport() };
    if ((*ident).MultiWordDMAActive() & 0x04) != 0 { return (ATA_WDMA0 + 2) as LONG; }
    if ((*ident).MultiWordDMAActive() & 0x02) != 0 { return (ATA_WDMA0 + 1) as LONG; }
    if ((*ident).MultiWordDMAActive() & 0x01) != 0 { return (ATA_WDMA0 + 0) as LONG; }

    let _ = if Active != 0 { dma1.SingleWordDMAActive() } else { dma1.SingleWordDMASupport() };
    if (dma1.SingleWordDMAActive() & 0x04) != 0 { return (ATA_SDMA0 + 2) as LONG; }
    if (dma1.SingleWordDMAActive() & 0x02) != 0 { return (ATA_SDMA0 + 1) as LONG; }
    if (dma1.SingleWordDMAActive() & 0x01) != 0 { return (ATA_SDMA0 + 0) as LONG; }

    if (*ident).PioTimingsValid() != 0 {
        mode = (*ident).AdvancedPIOModes();
        if (mode & AdvancedPIOModes_5 as USHORT) != 0 { return (ATA_PIO0 + 5) as LONG; }
        if (mode & AdvancedPIOModes_4 as USHORT) != 0 { return (ATA_PIO0 + 4) as LONG; }
        if (mode & AdvancedPIOModes_3 as USHORT) != 0 { return (ATA_PIO0 + 3) as LONG; }
    }
    if (*ident).PioCycleTimingMode == 2 { return (ATA_PIO0 + 2) as LONG; }
    if (*ident).PioCycleTimingMode == 1 { return (ATA_PIO0 + 1) as LONG; }
    if (*ident).PioCycleTimingMode == 0 { return (ATA_PIO0 + 0) as LONG; }

    ATA_PIO as LONG
}
