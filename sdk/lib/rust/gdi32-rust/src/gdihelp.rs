//! gdi32p.h (+ ntgdihdl.h handle macros) → Rust. The shared header-inline GDI substrate:
//! the GDI handle-table accessor macros and the three FORCEINLINE helpers
//! (`GdiAllocBatchCommand` / `GdiGetDcAttr` / `GdiGetRgnAttr`) that every translated
//! gdi32-rust body relies on. Central integration places this as `src/gdihelp.rs`.
//!
//! NEEDS-TYPE: none — all GDI types come from `win32k_sys::win32k`.
//! NEEDS-IMPORT: `GdiGetHandleUserData` and `GDI_BatchLimit` are owned by `misc.rs`
//!   (`#[no_mangle]` cdecl fn / `#[no_mangle] static mut`); referenced here via `extern "C"`.
//!
//! Faithfulness note: the ntgdihdl.h handle macros are header `#define`s on `(ULONG_PTR)(h)`.
//! Siblings (gdiobj.rs) call them with an `HGDIOBJ` pointer argument, so they are modelled
//! as `#[inline] pub(crate) fn(h: HGDIOBJ)` rather than the `const fn(usize)` the contract
//! sketched — a `const fn` cannot perform a pointer→integer cast in Rust, and an `HGDIOBJ`
//! parameter is what the sibling call sites require. The bit-math is byte-for-byte identical.

use win32k_sys::win32k::*;

// ── NtCurrentTeb() — per-thread TEB self pointer (NtTib.Self). FORCEINLINE, not an export. ──
// Exported pub(crate) so sibling bodies (gdiobj.rs etc.) reach the one TEB substrate.
#[inline]
pub(crate) unsafe fn NtCurrentTeb() -> *mut _TEB {
    let teb: *mut _TEB;
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!("mov {}, gs:[0x30]", out(reg) teb,
        options(nostack, pure, readonly, preserves_flags));
    #[cfg(target_arch = "x86")]
    core::arch::asm!("mov {}, fs:[0x18]", out(reg) teb,
        options(nostack, pure, readonly, preserves_flags));
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("mov {}, x18", out(reg) teb,
        options(nostack, pure, nomem, preserves_flags));
    teb
}

// ── Handle macros (ntgdihdl.h) ────────────────────────────────────────────────
// GDI_HANDLE_INDEX_MASK = GDI_HANDLE_COUNT-1 = 0xFFFF; TYPE 0x007f0000; UPPER 0x00ff0000;
// STOCK 0x00800000. Reproduced as literals (self-contained; identical to win32k_sys consts).

/// `GDI_HANDLE_GET_INDEX(h) = ((ULONG_PTR)h) & GDI_HANDLE_INDEX_MASK`.
#[inline]
pub(crate) fn GDI_HANDLE_GET_INDEX(h: HGDIOBJ) -> usize {
    (h as usize) & 0xFFFF
}

/// `GDI_HANDLE_GET_TYPE(h) = ((ULONG_PTR)h) & GDI_HANDLE_TYPE_MASK`.
#[inline]
pub(crate) fn GDI_HANDLE_GET_TYPE(h: HGDIOBJ) -> usize {
    (h as usize) & 0x007f0000
}

/// `GDI_HANDLE_GET_UPPER(h) = ((ULONG_PTR)h) & GDI_HANDLE_UPPER_MASK`.
#[inline]
pub(crate) fn GDI_HANDLE_GET_UPPER(h: HGDIOBJ) -> usize {
    (h as usize) & 0x00ff0000
}

/// `GDI_HANDLE_IS_STOCKOBJ(h) = 0 != (((ULONG_PTR)h) & GDI_HANDLE_STOCK_MASK)`.
#[inline]
pub(crate) fn GDI_HANDLE_IS_STOCKOBJ(h: HGDIOBJ) -> bool {
    ((h as usize) & 0x00800000) != 0
}

// ── GDIBATCHCMD short-name aliases (gdi32p.h uses GdiBCPatBlt etc.) ────────────
// win32k_sys names them `_GDIBATCHCMD_GdiBC*`; re-export the bare names the bodies use.
pub(crate) const GdiBCPatBlt: USHORT = _GDIBATCHCMD_GdiBCPatBlt as USHORT;
pub(crate) const GdiBCPolyPatBlt: USHORT = _GDIBATCHCMD_GdiBCPolyPatBlt as USHORT;
pub(crate) const GdiBCTextOut: USHORT = _GDIBATCHCMD_GdiBCTextOut as USHORT;
pub(crate) const GdiBCExtTextOut: USHORT = _GDIBATCHCMD_GdiBCExtTextOut as USHORT;
pub(crate) const GdiBCSetBrushOrg: USHORT = _GDIBATCHCMD_GdiBCSetBrushOrg as USHORT;
pub(crate) const GdiBCExtSelClipRgn: USHORT = _GDIBATCHCMD_GdiBCExtSelClipRgn as USHORT;
pub(crate) const GdiBCSelObj: USHORT = _GDIBATCHCMD_GdiBCSelObj as USHORT;
pub(crate) const GdiBCDelObj: USHORT = _GDIBATCHCMD_GdiBCDelObj as USHORT;
pub(crate) const GdiBCDelRgn: USHORT = _GDIBATCHCMD_GdiBCDelRgn as USHORT;

// ── Substrate owned by misc.rs, referenced here ───────────────────────────────
extern "C" {
    static mut GDI_BatchLimit: DWORD;
    fn GdiGetHandleUserData(hGdiObj: HGDIOBJ, ObjectType: DWORD, UserData: *mut PVOID) -> BOOL;
}

/// `GdiAllocBatchCommand` (gdi32p.h FORCEINLINE) — reserve `cjSize` bytes in the TEB's
/// GdiTebBatch ring for batch command `Cmd`, flushing via `NtGdiFlush()` when the ring is
/// full or the batch-count limit is reached. Returns a `PGDIBATCHHDR` (as `PVOID`) with the
/// `Cmd`/`Size` core fields filled, or NULL on no-environment / unsupported / wrong-DC.
pub(crate) unsafe fn GdiAllocBatchCommand(hdc: HDC, Cmd: USHORT) -> PVOID {
    /* Get a pointer to the TEB */
    let pTeb: *mut _TEB = NtCurrentTeb();

    /* Check if we have a valid environment */
    if pTeb.is_null() || (*pTeb).Win32ThreadInfo.is_null() {
        return core::ptr::null_mut();
    }

    /* Get the size of the entry */
    let cjSize: USHORT = if Cmd == GdiBCPatBlt {
        core::mem::size_of::<GDIBSPATBLT>() as USHORT
    } else if Cmd == GdiBCPolyPatBlt {
        core::mem::size_of::<GDIBSPPATBLT>() as USHORT
    } else if Cmd == GdiBCTextOut {
        core::mem::size_of::<GDIBSTEXTOUT>() as USHORT
    } else if Cmd == GdiBCExtTextOut {
        core::mem::size_of::<GDIBSEXTTEXTOUT>() as USHORT
    } else if Cmd == GdiBCSetBrushOrg {
        core::mem::size_of::<GDIBSSETBRHORG>() as USHORT
    } else if Cmd == GdiBCExtSelClipRgn {
        core::mem::size_of::<GDIBSEXTSELCLPRGN>() as USHORT
    } else if Cmd == GdiBCSelObj {
        core::mem::size_of::<GDIBSOBJECT>() as USHORT
    } else if Cmd == GdiBCDelRgn {
        core::mem::size_of::<GDIBSOBJECT>() as USHORT
    } else if Cmd == GdiBCDelObj {
        core::mem::size_of::<GDIBSOBJECT>() as USHORT
    } else {
        0
    };

    /* Unsupported operation */
    if cjSize == 0 {
        return core::ptr::null_mut();
    }

    /* Do we use a DC? */
    if !hdc.is_null() {
        /* If the batch DC is NULL, we set this one as the new one */
        if (*pTeb).GdiTebBatch.HDC.is_null() {
            (*pTeb).GdiTebBatch.HDC = hdc as HANDLE;
        }
        /* If not, check if the batch DC equal to our DC */
        else if (*pTeb).GdiTebBatch.HDC != hdc as HANDLE {
            return core::ptr::null_mut();
        }
    }

    /* Check if the buffer is full */
    if ((*pTeb).GdiBatchCount >= GDI_BatchLimit)
        || (((*pTeb).GdiTebBatch.Offset + cjSize as ULONG) > GDIBATCHBUFSIZE)
    {
        /* Call win32k, the kernel will call NtGdiFlushUserBatch to flush the current batch */
        NtGdiFlush();

        // If Flushed, lose the hDC for this batch job! See CORE-15839.
        if !hdc.is_null() {
            if (*pTeb).GdiTebBatch.HDC.is_null() {
                (*pTeb).GdiTebBatch.HDC = hdc as HANDLE;
            }
        }
    }

    /* Get the head of the entry */
    let pHdr: PGDIBATCHHDR = (core::ptr::addr_of_mut!((*pTeb).GdiTebBatch.Buffer) as *mut u8)
        .add((*pTeb).GdiTebBatch.Offset as usize) as PGDIBATCHHDR;

    /* Update Offset and batch count */
    (*pTeb).GdiTebBatch.Offset += cjSize as ULONG;
    (*pTeb).GdiBatchCount += 1;

    /* Fill in the core fields */
    (*pHdr).Cmd = Cmd as SHORT;
    (*pHdr).Size = cjSize as SHORT;

    pHdr as PVOID
}

/// `GdiGetDcAttr` (gdi32p.h FORCEINLINE) — return the DC_ATTR for a (LO_DC/LO_ALTDC) DC
/// handle via the handle-table user-data, or NULL if `hdc` is not a DC.
pub(crate) unsafe fn GdiGetDcAttr(hdc: HDC) -> PDC_ATTR {
    /* Check DC object type */
    let eDcObjType: usize = GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ);
    if (eDcObjType != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize)
        && (eDcObjType != GDILoObjType_GDILoObjType_LO_ALTDC_TYPE as usize)
    {
        return core::ptr::null_mut();
    }

    /* Get the DC attribute */
    let mut pdcattr: PDC_ATTR = core::ptr::null_mut();
    if GdiGetHandleUserData(
        hdc as HGDIOBJ,
        eDcObjType as DWORD,
        &mut pdcattr as *mut PDC_ATTR as *mut PVOID,
    ) == 0
    {
        return core::ptr::null_mut();
    }

    pdcattr
}

/// `GdiGetRgnAttr` (gdi32p.h FORCEINLINE) — return the RGN_ATTR for a region handle via the
/// handle-table user-data, or NULL.
pub(crate) unsafe fn GdiGetRgnAttr(hrgn: HRGN) -> PRGN_ATTR {
    /* Get the region attribute */
    let mut prgnattr: PRGN_ATTR = core::ptr::null_mut();
    if GdiGetHandleUserData(
        hrgn as HGDIOBJ,
        GDILoObjType_GDILoObjType_LO_REGION_TYPE as DWORD,
        &mut prgnattr as *mut PRGN_ATTR as *mut PVOID,
    ) == 0
    {
        return core::ptr::null_mut();
    }

    prgnattr
}
