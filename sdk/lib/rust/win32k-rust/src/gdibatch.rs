// gdibatch.rs — GDI batch-flush processor (Phase 2c wave 24)
// Hand-translation of win32ss/gdi/ntgdi/gdibatch.c.
// Every switch case faithfully translated; no stubs.

use win32k_sys::win32k::*;
use ntkrnl_sys::inline::ke::NtCurrentTeb;
use ntkrnl_sys::base::STATUS_SUCCESS;

// ── Local constants not present in win32k_sys bindgen ─────────────────────

// pdevobj.h:21  PDEV_DRIVER_PUNTED_CALL = 0x00040000
const PDEV_DRIVER_PUNTED_CALL: FLONG = 0x00040000;

// GDIBATCHBUFSIZE = 0x136*4 (ntgdityp.h:200)
const GDIBATCHBUFSIZE: ULONG = 0x136 * 4;

// ── FORCEINLINE / macro helpers ────────────────────────────────────────────

// MAKEROP4(f,b) = (DWORD)((((b)<<8)&0xFF000000)|(f))  [wingdi.h:3392]
#[inline(always)]
fn MAKEROP4(f: DWORD, b: DWORD) -> DWORD {
    ((b << 8) & 0xFF000000) | f
}

// WIN32_ROP4_USES_SOURCE(Rop) [intgdi.h:7]
// = ((((Rop) & 0xCCCC0000) >> 2) != ((Rop) & 0x33330000))
#[inline(always)]
fn WIN32_ROP4_USES_SOURCE(rop: DWORD) -> bool {
    ((rop & 0xCCCC0000) >> 2) != (rop & 0x33330000)
}

// GDI_HANDLE_GET_TYPE(h) = ((ULONG_PTR)(h)) & GDI_HANDLE_TYPE_MASK  [ntgdihdl.h:93-94]
#[inline(always)]
fn GDI_HANDLE_GET_TYPE(h: HANDLE) -> ULONG_PTR {
    (h as ULONG_PTR) & (GDI_HANDLE_TYPE_MASK as ULONG_PTR)
}

// BRUSH_ShareLockBrush(h) = GDIOBJ_ShareLockObj(h, GDI_OBJECT_TYPE_BRUSH) [brush.h:117]
#[inline(always)]
unsafe fn BRUSH_ShareLockBrush(hbrush: HBRUSH) -> PBRUSH {
    GDIOBJ_ShareLockObj(hbrush as HGDIOBJ, GDI_OBJECT_TYPE_BRUSH as DWORD) as PBRUSH
}

// BRUSH_ShareUnlockBrush(p) = GDIOBJ_vDereferenceObject(p) [brush.h:118]
#[inline(always)]
unsafe fn BRUSH_ShareUnlockBrush(pbrush: PBRUSH) {
    GDIOBJ_vDereferenceObject(pbrush as POBJ);
}

// ── extern declarations for helpers NOT in win32k_sys bindgen ─────────────

// IntPatBlt: FASTCALL (@IntPatBlt@28, 7 args × 4 bytes).
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" {
    fn IntPatBlt(
        dc: PDC,
        nXLeft: ::core::ffi::c_int,
        nYLeft: ::core::ffi::c_int,
        nWidth: ::core::ffi::c_int,
        nHeight: ::core::ffi::c_int,
        dwRop: DWORD,
        pebo: PEBRUSHOBJ,
    ) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" {
    fn IntPatBlt(
        dc: PDC,
        nXLeft: ::core::ffi::c_int,
        nYLeft: ::core::ffi::c_int,
        nWidth: ::core::ffi::c_int,
        nHeight: ::core::ffi::c_int,
        dwRop: DWORD,
        pebo: PEBRUSHOBJ,
    ) -> BOOL;
}

// IntExtTextOutW: APIENTRY/stdcall (_IntExtTextOutW@36, 9 args × 4 bytes).
extern "system" {
    fn IntExtTextOutW(
        dc: PDC,
        x: ::core::ffi::c_int,
        y: ::core::ffi::c_int,
        fuOptions: UINT,
        lprc: PRECTL,
        lpString: *const WCHAR,
        cchString: ::core::ffi::c_int,
        lpDx: *const ::core::ffi::c_int,
        iCS_CP: DWORD,
    ) -> BOOL;
}

// EBRUSHOBJ_vInitFromDC: stdcall (_EBRUSHOBJ_vInitFromDC@12, 3 args).
extern "system" {
    fn EBRUSHOBJ_vInitFromDC(pebo: PEBRUSHOBJ, pbrush: PBRUSH, pdc: PDC);
}

// EBRUSHOBJ_vCleanup: stdcall (_EBRUSHOBJ_vCleanup@4, 1 arg).
extern "system" {
    fn EBRUSHOBJ_vCleanup(pebo: PEBRUSHOBJ);
}

// ── DoDeviceSync ───────────────────────────────────────────────────────────
// Based on IntEngEnter from eng/engmisc.c
// FASTCALL: @DoDeviceSync@12 — dual-cfg.

#[inline(always)]
unsafe fn DoDeviceSync_impl(Surface: *mut SURFOBJ, Rect: PRECTL, fl: FLONG) {
    let Device = (*Surface).hdev as *mut _PDEVOBJ;
    // No punting and "Handle to a surface, provided that the surface is device-managed.
    // Otherwise, dhsurf is zero".
    if ((*Device).flFlags & PDEV_DRIVER_PUNTED_CALL) == 0
        && !(*Surface).dhsurf.is_null()
    {
        let fns = &(*Device).__bindgen_anon_1.DriverFunctions;
        if let Some(sync_surf) = fns.SynchronizeSurface {
            sync_surf(Surface, Rect, fl);
        } else if let Some(sync) = fns.Synchronize {
            sync((*Surface).dhpdev, Rect);
        }
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DoDeviceSync(
    Surface: *mut SURFOBJ,
    Rect: PRECTL,
    fl: FLONG,
) {
    DoDeviceSync_impl(Surface, Rect, fl)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DoDeviceSync(
    Surface: *mut SURFOBJ,
    Rect: PRECTL,
    fl: FLONG,
) {
    DoDeviceSync_impl(Surface, Rect, fl)
}

// ── SynchronizeDriver ──────────────────────────────────────────────────────
// FASTCALL: @SynchronizeDriver@4 — dual-cfg.

#[inline(always)]
unsafe fn SynchronizeDriver_impl(mut Flags: FLONG) {
    if Flags & GCAPS2_SYNCFLUSH != 0 {
        Flags = DSS_FLUSH_EVENT;
    }
    if Flags & GCAPS2_SYNCTIMER != 0 {
        Flags = DSS_TIMER_EVENT;
    }

    // IntEnumHDev() is UNIMPLEMENTED — SurfObj is always NULL here.
    let SurfObj: *mut SURFOBJ = core::ptr::null_mut();
    if SurfObj.is_null() {
        return;
    }
    DoDeviceSync_impl(SurfObj, core::ptr::null_mut(), Flags);
    EngUnlockSurface(SurfObj);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn SynchronizeDriver(Flags: FLONG) {
    SynchronizeDriver_impl(Flags)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn SynchronizeDriver(Flags: FLONG) {
    SynchronizeDriver_impl(Flags)
}

// ── GdiFlushUserBatch ──────────────────────────────────────────────────────
// Process the batch. FASTCALL: @GdiFlushUserBatch@8 — dual-cfg.

#[inline(always)]
unsafe fn GdiFlushUserBatch_impl(dc: PDC, pHdr: PGDIBATCHHDR) -> ULONG {
    let mut Cmd: ULONG = 0;
    let mut Size: ULONG = 0;
    let mut pdcattr: PDC_ATTR = core::ptr::null_mut();

    if !dc.is_null() {
        pdcattr = (*dc).pdcattr;
    }

    // _SEH2_TRY: under DUMMY_PSEH the _EXCEPT is dead; plain reads.
    Cmd = (*pHdr).Cmd as ULONG;
    Size = (*pHdr).Size as ULONG; // Return the full size of the structure.

    match Cmd {
        // ── GdiBCPatBlt ────────────────────────────────────────────────────
        _GDIBATCHCMD_GdiBCPatBlt => {
            if dc.is_null() {
                return Size;
            }
            let pgDPB = pHdr as PGDIBSPATBLT;

            // Convert the ROP3 to a ROP4
            let mut dwRop = (*pgDPB).dwRop;
            dwRop = MAKEROP4(dwRop & 0xFF0000, dwRop);

            // Check if the rop uses a source
            if WIN32_ROP4_USES_SOURCE(dwRop) {
                return Size;
            }

            // Check if the DC has no surface (empty mem or info DC)
            if (*dc).dclevel.pSurface.is_null() {
                return Size;
            }

            // Save current attributes and flags
            let crColor         = (*(*dc).pdcattr).crForegroundClr;
            let crBkColor       = (*(*dc).pdcattr).ulBackgroundClr;
            let crBrushClr      = (*(*dc).pdcattr).crBrushClr;
            let ulForegroundClr = (*(*dc).pdcattr).ulForegroundClr;
            let ulBackgroundClr = (*(*dc).pdcattr).ulBackgroundClr;
            let ulBrushClr      = (*(*dc).pdcattr).ulBrushClr;
            let hOrgBrush       = (*(*dc).pdcattr).hbrush;
            let flags = (*(*dc).pdcattr).ulDirty_
                & (DIRTY_BACKGROUND | DIRTY_TEXT | DIRTY_FILL | DC_BRUSH_DIRTY);

            // Set the attribute snapshot
            (*(*dc).pdcattr).hbrush          = (*pgDPB).hbrush;
            (*(*dc).pdcattr).crForegroundClr = (*pgDPB).crForegroundClr;
            (*(*dc).pdcattr).crBackgroundClr = (*pgDPB).crBackgroundClr;
            (*(*dc).pdcattr).crBrushClr      = (*pgDPB).crBrushClr;
            (*(*dc).pdcattr).ulForegroundClr = (*pgDPB).ulForegroundClr;
            (*(*dc).pdcattr).ulBackgroundClr = (*pgDPB).ulBackgroundClr;
            (*(*dc).pdcattr).ulBrushClr      = (*pgDPB).ulBrushClr;

            // Process dirty attributes if any.
            if (*(*dc).pdcattr).ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY) != 0 {
                DC_vUpdateFillBrush(dc);
            }
            if (*(*dc).pdcattr).ulDirty_ & DIRTY_TEXT != 0 {
                DC_vUpdateTextBrush(dc);
            }
            if (*pdcattr).ulDirty_ & DIRTY_BACKGROUND != 0 {
                DC_vUpdateBackgroundBrush(dc);
            }

            // Call the internal function
            IntPatBlt(
                dc,
                (*pgDPB).nXLeft,
                (*pgDPB).nYLeft,
                (*pgDPB).nWidth,
                (*pgDPB).nHeight,
                dwRop,
                core::ptr::addr_of_mut!((*dc).eboFill),
            );

            // Restore attributes and flags
            (*(*dc).pdcattr).hbrush          = hOrgBrush;
            (*(*dc).pdcattr).crForegroundClr = crColor;
            (*(*dc).pdcattr).crBackgroundClr = crBkColor;
            (*(*dc).pdcattr).crBrushClr      = crBrushClr;
            (*(*dc).pdcattr).ulForegroundClr = ulForegroundClr;
            (*(*dc).pdcattr).ulBackgroundClr = ulBackgroundClr;
            (*(*dc).pdcattr).ulBrushClr      = ulBrushClr;
            (*(*dc).pdcattr).ulDirty_ |= flags;
        }

        // ── GdiBCPolyPatBlt ────────────────────────────────────────────────
        _GDIBATCHCMD_GdiBCPolyPatBlt => {
            if dc.is_null() {
                return Size;
            }
            let pgDPB = pHdr as PGDIBSPPATBLT;

            // Convert the ROP3 to a ROP4
            let mut dwRop = (*pgDPB).rop4;
            dwRop = MAKEROP4(dwRop & 0xFF0000, dwRop);

            // Check if the rop uses a source
            if WIN32_ROP4_USES_SOURCE(dwRop) {
                return Size;
            }

            // Check if the DC has no surface (empty mem or info DC)
            if (*dc).dclevel.pSurface.is_null() {
                return Size;
            }

            // Save current attributes and flags
            let crColor         = (*(*dc).pdcattr).crForegroundClr;
            let crBkColor: COLORREF = (*(*dc).pdcattr).ulBackgroundClr;
            let crBrushClr      = (*(*dc).pdcattr).crBrushClr;
            let ulForegroundClr = (*(*dc).pdcattr).ulForegroundClr;
            let ulBackgroundClr = (*(*dc).pdcattr).ulBackgroundClr;
            let ulBrushClr      = (*(*dc).pdcattr).ulBrushClr;
            let flags = (*(*dc).pdcattr).ulDirty_
                & (DIRTY_BACKGROUND | DIRTY_TEXT | DIRTY_FILL | DC_BRUSH_DIRTY);

            // Set the attribute snapshot
            (*(*dc).pdcattr).crForegroundClr = (*pgDPB).crForegroundClr;
            (*(*dc).pdcattr).crBackgroundClr = (*pgDPB).crBackgroundClr;
            (*(*dc).pdcattr).crBrushClr      = (*pgDPB).crBrushClr;
            (*(*dc).pdcattr).ulForegroundClr = (*pgDPB).ulForegroundClr;
            (*(*dc).pdcattr).ulBackgroundClr = (*pgDPB).ulBackgroundClr;
            (*(*dc).pdcattr).ulBrushClr      = (*pgDPB).ulBrushClr;

            // Process dirty attributes if any
            if (*(*dc).pdcattr).ulDirty_ & DIRTY_TEXT != 0 {
                DC_vUpdateTextBrush(dc);
            }
            if (*pdcattr).ulDirty_ & DIRTY_BACKGROUND != 0 {
                DC_vUpdateBackgroundBrush(dc);
            }

            // Iterate over the PATRECT array (flexible array at end of struct)
            let pRects = core::ptr::addr_of_mut!((*pgDPB).pRect[0]);
            let count = (*pgDPB).Count as isize;
            for i in 0..count {
                let prect = pRects.offset(i);
                let pbrush = BRUSH_ShareLockBrush((*prect).hBrush);

                if !pbrush.is_null() {
                    // Initialize a brush object on the stack
                    let mut eboFill: EBRUSHOBJ = core::mem::zeroed();
                    EBRUSHOBJ_vInitFromDC(
                        core::ptr::addr_of_mut!(eboFill),
                        pbrush,
                        dc,
                    );

                    IntPatBlt(
                        dc,
                        (*prect).r.left,
                        (*prect).r.top,
                        (*prect).r.right,
                        (*prect).r.bottom,
                        dwRop,
                        core::ptr::addr_of_mut!(eboFill),
                    );

                    // Cleanup the brush object and unlock the brush
                    EBRUSHOBJ_vCleanup(core::ptr::addr_of_mut!(eboFill));
                    BRUSH_ShareUnlockBrush(pbrush);
                }
            }

            // Restore attributes and flags
            (*(*dc).pdcattr).crForegroundClr = crColor;
            (*(*dc).pdcattr).crBackgroundClr = crBkColor;
            (*(*dc).pdcattr).crBrushClr      = crBrushClr;
            (*(*dc).pdcattr).ulForegroundClr = ulForegroundClr;
            (*(*dc).pdcattr).ulBackgroundClr = ulBackgroundClr;
            (*(*dc).pdcattr).ulBrushClr      = ulBrushClr;
            (*(*dc).pdcattr).ulDirty_ |= flags;
        }

        // ── GdiBCTextOut ───────────────────────────────────────────────────
        _GDIBATCHCMD_GdiBCTextOut => {
            if dc.is_null() {
                return Size;
            }
            let pgO = pHdr as PGDIBSTEXTOUT;

            // crColor = -1 sentinel: use 0xFFFFFFFF (COLORREF is u32)
            let mut crColor: COLORREF = 0xFFFFFFFF_u32;
            let mut crBkColor: COLORREF = 0;
            let mut ulForegroundClr: ULONG = 0;
            let mut ulBackgroundClr: ULONG = 0;
            let mut flags: DWORD = 0;
            let mut flXform: FLONG = 0;
            let mut saveflXform: FLONG = 0;
            // flTextAlign = -1 sentinel
            let mut flTextAlign: FLONG = 0xFFFFFFFF_u32;
            let mut hlfntNew: HANDLE = core::ptr::null_mut();
            let mut ptlViewportOrg: POINTL = unsafe { core::mem::zeroed() };

            // Save current attributes, flags
            let saveflags = (*(*dc).pdcattr).ulDirty_
                & (DIRTY_BACKGROUND | DIRTY_LINE | DIRTY_TEXT | DIRTY_FILL
                   | DC_BRUSH_DIRTY | DIRTY_CHARSET);

            // In this instance check for differences and set the appropriate dirty flags.
            if (*(*dc).pdcattr).crForegroundClr != (*pgO).crForegroundClr {
                crColor = (*(*dc).pdcattr).crForegroundClr;
                (*(*dc).pdcattr).crForegroundClr = (*pgO).crForegroundClr;
                ulForegroundClr = (*(*dc).pdcattr).ulForegroundClr;
                (*(*dc).pdcattr).ulForegroundClr = (*pgO).ulForegroundClr;
                flags |= DIRTY_FILL | DIRTY_LINE | DIRTY_TEXT;
            }
            if (*(*dc).pdcattr).crBackgroundClr != (*pgO).crBackgroundClr {
                crBkColor = (*(*dc).pdcattr).ulBackgroundClr;
                (*(*dc).pdcattr).crBackgroundClr = (*pgO).crBackgroundClr;
                ulBackgroundClr = (*(*dc).pdcattr).ulBackgroundClr;
                (*(*dc).pdcattr).ulBackgroundClr = (*pgO).ulBackgroundClr;
                flags |= DIRTY_FILL | DIRTY_LINE | DIRTY_TEXT | DIRTY_BACKGROUND;
            }
            if (*(*dc).pdcattr).flTextAlign != (*pgO).flTextAlign {
                flTextAlign = (*(*dc).pdcattr).flTextAlign;
                (*(*dc).pdcattr).flTextAlign = (*pgO).flTextAlign;
            }
            if (*(*dc).pdcattr).hlfntNew != (*pgO).hlfntNew {
                hlfntNew = (*(*dc).pdcattr).hlfntNew;
                (*(*dc).pdcattr).hlfntNew = (*pgO).hlfntNew;
                (*(*dc).pdcattr).ulDirty_ &= !SLOW_WIDTHS;
                flags |= DIRTY_CHARSET;
            }

            if (*(*dc).pdcattr).ptlViewportOrg.x != (*pgO).ptlViewportOrg.x
                || (*(*dc).pdcattr).ptlViewportOrg.y != (*pgO).ptlViewportOrg.y
            {
                saveflXform = (*(*dc).pdcattr).flXform
                    & (PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID);
                ptlViewportOrg = (*(*dc).pdcattr).ptlViewportOrg;
                (*(*dc).pdcattr).ptlViewportOrg = (*pgO).ptlViewportOrg;
                flXform = PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID;
            }

            (*(*dc).pdcattr).flXform  |= flXform;
            (*(*dc).pdcattr).ulDirty_ |= flags;

            let jBkMode = (*(*dc).pdcattr).jBkMode;
            (*(*dc).pdcattr).jBkMode = (*pgO).lBkMode as BYTE;
            let lBkMode = (*(*dc).pdcattr).lBkMode;
            (*(*dc).pdcattr).lBkMode = (*pgO).lBkMode;

            // lprc = (pgO->Options & GDIBS_NORECT) ? NULL : &pgO->Rect
            let lprc: PRECTL = if (*pgO).Options & GDIBS_NORECT != 0 {
                core::ptr::null_mut()
            } else {
                core::ptr::addr_of_mut!((*pgO).Rect)
            };
            (*pgO).Options &= !GDIBS_NORECT;

            // String pointer: &pgO->String[pgO->Size/sizeof(WCHAR)]
            // Buffer pointer: pgO->Size ? &pgO->Buffer : NULL
            let string_ptr = (*pgO).__bindgen_anon_1.String
                .as_ptr()
                .add(((*pgO).Size as usize) / core::mem::size_of::<WCHAR>())
                as *const WCHAR;
            let dx_ptr: *const ::core::ffi::c_int = if (*pgO).Size != 0 {
                (*pgO).__bindgen_anon_1.Buffer.as_ptr() as *const ::core::ffi::c_int
            } else {
                core::ptr::null()
            };

            IntExtTextOutW(
                dc,
                (*pgO).x,
                (*pgO).y,
                (*pgO).Options,
                lprc,
                string_ptr,
                (*pgO).cbCount as ::core::ffi::c_int,
                dx_ptr,
                (*pgO).iCS_CP,
            );

            // Restore attributes and flags
            (*(*dc).pdcattr).jBkMode = jBkMode;
            (*(*dc).pdcattr).lBkMode = lBkMode;

            if saveflXform != 0 {
                (*(*dc).pdcattr).ptlViewportOrg = ptlViewportOrg;
                (*(*dc).pdcattr).flXform |= saveflXform | flXform;
            }

            // crColor != -1 check: compare against 0xFFFFFFFF sentinel
            if flags & DIRTY_TEXT != 0 && crColor != 0xFFFFFFFF_u32 {
                (*(*dc).pdcattr).crForegroundClr = crColor;
                (*(*dc).pdcattr).ulForegroundClr = ulForegroundClr;
            }
            if flags & DIRTY_BACKGROUND != 0 {
                (*(*dc).pdcattr).crBackgroundClr = crBkColor;
                (*(*dc).pdcattr).ulBackgroundClr = ulBackgroundClr;
            }
            // flTextAlign != -1 check: compare against 0xFFFFFFFF sentinel
            if flTextAlign != 0xFFFFFFFF_u32 {
                (*(*dc).pdcattr).flTextAlign = flTextAlign;
            }

            if flags & DIRTY_CHARSET != 0 {
                (*(*dc).pdcattr).hlfntNew = hlfntNew;
                (*(*dc).pdcattr).ulDirty_ &= !SLOW_WIDTHS;
            }
            (*(*dc).pdcattr).ulDirty_ |= saveflags | flags;
            (*(*dc).pdcattr).flXform  |= saveflXform | flXform;
        }

        // ── GdiBCExtTextOut ────────────────────────────────────────────────
        _GDIBATCHCMD_GdiBCExtTextOut => {
            if dc.is_null() {
                return Size;
            }
            let pgO = pHdr as PGDIBSEXTTEXTOUT;

            let mut crBkColor: COLORREF = 0;
            let mut ulBackgroundClr: ULONG = 0;
            let mut ptlViewportOrg: POINTL = unsafe { core::mem::zeroed() };
            let mut flags: DWORD = 0;
            let mut flXform: FLONG = 0;
            let mut saveflXform: FLONG = 0;

            let saveflags = (*(*dc).pdcattr).ulDirty_
                & (DIRTY_BACKGROUND | DIRTY_TEXT | DIRTY_FILL | DC_BRUSH_DIRTY | DIRTY_CHARSET);

            if (*(*dc).pdcattr).crBackgroundClr != (*pgO).ulBackgroundClr {
                crBkColor = (*(*dc).pdcattr).crBackgroundClr;
                ulBackgroundClr = (*(*dc).pdcattr).ulBackgroundClr;
                (*(*dc).pdcattr).crBackgroundClr = (*pgO).ulBackgroundClr;
                (*(*dc).pdcattr).ulBackgroundClr = (*pgO).ulBackgroundClr;
                flags |= DIRTY_BACKGROUND | DIRTY_LINE | DIRTY_FILL;
            }

            if (*(*dc).pdcattr).ptlViewportOrg.x != (*pgO).ptlViewportOrg.x
                || (*(*dc).pdcattr).ptlViewportOrg.y != (*pgO).ptlViewportOrg.y
            {
                saveflXform = (*(*dc).pdcattr).flXform
                    & (PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID);
                ptlViewportOrg = (*(*dc).pdcattr).ptlViewportOrg;
                (*(*dc).pdcattr).ptlViewportOrg = (*pgO).ptlViewportOrg;
                flXform = PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID;
            }

            (*(*dc).pdcattr).flXform  |= flXform;
            (*(*dc).pdcattr).ulDirty_ |= flags;

            IntExtTextOutW(
                dc,
                0,
                0,
                (*pgO).Options,
                core::ptr::addr_of_mut!((*pgO).Rect),
                core::ptr::null(),
                (*pgO).Count as ::core::ffi::c_int,
                core::ptr::null(),
                0,
            );

            if saveflXform != 0 {
                (*(*dc).pdcattr).ptlViewportOrg = ptlViewportOrg;
                (*(*dc).pdcattr).flXform |= saveflXform | flXform;
            }

            if flags & DIRTY_BACKGROUND != 0 {
                (*(*dc).pdcattr).crBackgroundClr = crBkColor;
                (*(*dc).pdcattr).ulBackgroundClr = ulBackgroundClr;
            }
            (*(*dc).pdcattr).ulDirty_ |= saveflags | flags;
            (*(*dc).pdcattr).flXform  |= saveflXform | flXform;
        }

        // ── GdiBCSetBrushOrg ───────────────────────────────────────────────
        _GDIBATCHCMD_GdiBCSetBrushOrg => {
            if dc.is_null() {
                return Size;
            }
            let pgSBO = pHdr as PGDIBSSETBRHORG;
            (*pdcattr).ptlBrushOrigin = (*pgSBO).ptlBrushOrigin;
            DC_vSetBrushOrigin(
                dc,
                (*pgSBO).ptlBrushOrigin.x,
                (*pgSBO).ptlBrushOrigin.y,
            );
        }

        // ── GdiBCExtSelClipRgn ─────────────────────────────────────────────
        _GDIBATCHCMD_GdiBCExtSelClipRgn => {
            if dc.is_null() {
                return Size;
            }
            let pgO = pHdr as PGDIBSEXTSELCLPRGN;
            IntGdiExtSelectClipRect(dc, core::ptr::addr_of_mut!((*pgO).rcl), (*pgO).fnMode);
        }

        // ── GdiBCSelObj ────────────────────────────────────────────────────
        _GDIBATCHCMD_GdiBCSelObj => {
            if dc.is_null() {
                return Size;
            }
            let pgO = pHdr as PGDIBSOBJECT;
            DC_hSelectFont(dc, (*pgO).hgdiobj as HFONT);
        }

        // ── GdiBCDelRgn (falls through to GdiBCDelObj) ────────────────────
        _GDIBATCHCMD_GdiBCDelRgn | _GDIBATCHCMD_GdiBCDelObj => {
            let pgO = pHdr as PGDIBSOBJECT;
            GreDeleteObject((*pgO).hgdiobj);
        }

        _ => {}
    }

    Size
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GdiFlushUserBatch(dc: PDC, pHdr: PGDIBATCHHDR) -> ULONG {
    GdiFlushUserBatch_impl(dc, pHdr)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GdiFlushUserBatch(dc: PDC, pHdr: PGDIBATCHHDR) -> ULONG {
    GdiFlushUserBatch_impl(dc, pHdr)
}

// ── WoW64 guest batch ─────────────────────────────────────────────────────
//
// A 32-bit guest's gdi32 writes i386 records into TEB32. They were reset and
// DROPPED until this translation existed, and that was not only lost drawing:
// gdi32's DeleteObject on a region or brush with a user attribute is a batched
// GdiBCDelRgn/GdiBCDelObj, so every such delete leaked its handle. osk.exe
// repaints continuously and filled all 64K GDI handles with regions in a few
// minutes, after which every Create* returned NULL and co_WinPosSetWindowPos
// bugchecked 0x1E offsetting one.
//
// The i386 layouts differ from these only by HANDLE width (and the alignment it
// forces): GDIBSPATBLT.hbrush, PATRECT.hBrush, GDIBSTEXTOUT.hlfntNew and
// GDIBSOBJECT.hgdiobj. ExtTextOut, SetBrushOrg and ExtSelClipRgn carry no
// pointer-sized field and copy through unchanged. Each record is widened into a
// kernel scratch buffer — which also stops the guest rewriting it mid-flush —
// and handed to the same executor as a native batch. Handles are sign-extended
// like every generated win32k thunk's.

#[cfg(not(target_arch = "x86"))]
mod guest32 {
    use win32k_sys::win32k::{GDIBATCHHDR, POINTL, RECTL};

    #[repr(C)]
    pub struct PatBlt {
        pub gbHdr: GDIBATCHHDR,
        pub nXLeft: i32,
        pub nYLeft: i32,
        pub nWidth: i32,
        pub nHeight: i32,
        pub hbrush: u32,
        pub dwRop: u32,
        pub crForegroundClr: u32,
        pub crBackgroundClr: u32,
        pub crBrushClr: u32,
        pub IcmBrushColor: i32,
        pub ptlViewportOrg: POINTL,
        pub ulForegroundClr: u32,
        pub ulBackgroundClr: u32,
        pub ulBrushClr: u32,
    }

    #[repr(C)]
    pub struct PatRect {
        pub r: RECTL,
        pub hBrush: u32,
    }

    #[repr(C)]
    pub struct PolyPatBlt {
        pub gbHdr: GDIBATCHHDR,
        pub rop4: u32,
        pub Mode: u32,
        pub Count: u32,
        pub crForegroundClr: u32,
        pub crBackgroundClr: u32,
        pub crBrushClr: u32,
        pub ulForegroundClr: u32,
        pub ulBackgroundClr: u32,
        pub ulBrushClr: u32,
        pub ptlViewportOrg: POINTL,
        pub pRect: [PatRect; 1],
    }

    #[repr(C)]
    pub struct TextOut {
        pub gbHdr: GDIBATCHHDR,
        pub crForegroundClr: u32,
        pub crBackgroundClr: u32,
        pub lBkMode: i32,
        pub ulForegroundClr: u32,
        pub ulBackgroundClr: u32,
        pub x: i32,
        pub y: i32,
        pub Options: u32,
        pub Rect: RECTL,
        pub iCS_CP: u32,
        pub cbCount: u32,
        pub Size: u32,
        pub hlfntNew: u32,
        pub flTextAlign: u32,
        pub ptlViewportOrg: POINTL,
        pub Buffer: [u32; 1],
    }

    #[repr(C)]
    pub struct Object {
        pub gbHdr: GDIBATCHHDR,
        pub hgdiobj: u32,
    }

    // sizeof() as the i386 compiler lays them out, which is what the guest wrote.
    const _: () = assert!(core::mem::size_of::<PatBlt>() == 64);
    const _: () = assert!(core::mem::size_of::<PatRect>() == 20);
    const _: () = assert!(core::mem::offset_of!(PolyPatBlt, pRect) == 48);
    const _: () = assert!(core::mem::offset_of!(TextOut, Buffer) == 80);
    const _: () = assert!(core::mem::size_of::<Object>() == 8);
}

#[cfg(not(target_arch = "x86"))]
#[inline]
fn guest_handle(h: u32) -> HANDLE {
    h as i32 as isize as HANDLE
}

/// Widen one i386 batch record at `src` (its header `Size` bytes, already
/// bounds-checked against the batch buffer) into `dst`. Returns false for a
/// record that cannot be translated, which ends the flush.
#[cfg(not(target_arch = "x86"))]
unsafe fn widen_guest_record(src: *const u8, src_size: usize, dst: *mut u8, dst_cap: usize) -> bool {
    use core::mem::{offset_of, size_of};
    let hdr = src as *const GDIBATCHHDR;
    let cmd = (*hdr).Cmd as u16 as ULONG;
    let out_size: usize;

    match cmd {
        _GDIBATCHCMD_GdiBCPatBlt => {
            if src_size < size_of::<guest32::PatBlt>() || size_of::<GDIBSPATBLT>() > dst_cap {
                return false;
            }
            let g = src as *const guest32::PatBlt;
            let n = dst as *mut GDIBSPATBLT;
            out_size = size_of::<GDIBSPATBLT>();
            (*n).nXLeft = (*g).nXLeft;
            (*n).nYLeft = (*g).nYLeft;
            (*n).nWidth = (*g).nWidth;
            (*n).nHeight = (*g).nHeight;
            (*n).hbrush = guest_handle((*g).hbrush);
            (*n).dwRop = (*g).dwRop;
            (*n).crForegroundClr = (*g).crForegroundClr;
            (*n).crBackgroundClr = (*g).crBackgroundClr;
            (*n).crBrushClr = (*g).crBrushClr;
            (*n).IcmBrushColor = (*g).IcmBrushColor;
            (*n).ptlViewportOrg = (*g).ptlViewportOrg;
            (*n).ulForegroundClr = (*g).ulForegroundClr;
            (*n).ulBackgroundClr = (*g).ulBackgroundClr;
            (*n).ulBrushClr = (*g).ulBrushClr;
        }
        _GDIBATCHCMD_GdiBCPolyPatBlt => {
            let head = offset_of!(guest32::PolyPatBlt, pRect);
            if src_size < head {
                return false;
            }
            let g = src as *const guest32::PolyPatBlt;
            let count = (*g).Count as usize;
            if head + count * size_of::<guest32::PatRect>() > src_size {
                return false;
            }
            out_size = offset_of!(GDIBSPPATBLT, pRect) + count * size_of::<PATRECT>();
            if out_size > dst_cap {
                return false;
            }
            let n = dst as *mut GDIBSPPATBLT;
            (*n).rop4 = (*g).rop4;
            (*n).Mode = (*g).Mode;
            (*n).Count = (*g).Count;
            (*n).crForegroundClr = (*g).crForegroundClr;
            (*n).crBackgroundClr = (*g).crBackgroundClr;
            (*n).crBrushClr = (*g).crBrushClr;
            (*n).ulForegroundClr = (*g).ulForegroundClr;
            (*n).ulBackgroundClr = (*g).ulBackgroundClr;
            (*n).ulBrushClr = (*g).ulBrushClr;
            (*n).ptlViewportOrg = (*g).ptlViewportOrg;
            let gr = (*g).pRect.as_ptr();
            let nr = (*n).pRect.as_mut_ptr();
            for i in 0..count {
                (*nr.add(i)).r = (*gr.add(i)).r;
                (*nr.add(i)).hBrush = guest_handle((*gr.add(i)).hBrush) as HBRUSH;
            }
        }
        _GDIBATCHCMD_GdiBCTextOut => {
            let head = offset_of!(guest32::TextOut, Buffer);
            if src_size < head {
                return false;
            }
            // The Dx array and the string follow the fixed part; both are
            // 4-byte units with no pointers, so they move as one block.
            let tail = src_size - head;
            out_size = offset_of!(GDIBSTEXTOUT, __bindgen_anon_1) + tail;
            if out_size > dst_cap {
                return false;
            }
            let g = src as *const guest32::TextOut;
            let n = dst as *mut GDIBSTEXTOUT;
            (*n).crForegroundClr = (*g).crForegroundClr;
            (*n).crBackgroundClr = (*g).crBackgroundClr;
            (*n).lBkMode = (*g).lBkMode;
            (*n).ulForegroundClr = (*g).ulForegroundClr;
            (*n).ulBackgroundClr = (*g).ulBackgroundClr;
            (*n).x = (*g).x;
            (*n).y = (*g).y;
            (*n).Options = (*g).Options;
            (*n).Rect = (*g).Rect;
            (*n).iCS_CP = (*g).iCS_CP;
            (*n).cbCount = (*g).cbCount;
            (*n).Size = (*g).Size;
            (*n).hlfntNew = guest_handle((*g).hlfntNew);
            (*n).flTextAlign = (*g).flTextAlign;
            (*n).ptlViewportOrg = (*g).ptlViewportOrg;
            core::ptr::copy_nonoverlapping(
                src.add(head),
                dst.add(offset_of!(GDIBSTEXTOUT, __bindgen_anon_1)),
                tail,
            );
        }
        _GDIBATCHCMD_GdiBCSelObj | _GDIBATCHCMD_GdiBCDelRgn | _GDIBATCHCMD_GdiBCDelObj => {
            if src_size < size_of::<guest32::Object>() || size_of::<GDIBSOBJECT>() > dst_cap {
                return false;
            }
            let g = src as *const guest32::Object;
            let n = dst as *mut GDIBSOBJECT;
            out_size = size_of::<GDIBSOBJECT>();
            (*n).hgdiobj = guest_handle((*g).hgdiobj);
        }
        _GDIBATCHCMD_GdiBCExtTextOut
        | _GDIBATCHCMD_GdiBCSetBrushOrg
        | _GDIBATCHCMD_GdiBCExtSelClipRgn => {
            if src_size > dst_cap {
                return false;
            }
            out_size = src_size;
            core::ptr::copy_nonoverlapping(src, dst, src_size);
        }
        _ => return false,
    }

    let nhdr = dst as *mut GDIBATCHHDR;
    (*nhdr).Cmd = (*hdr).Cmd;
    (*nhdr).Size = out_size as SHORT;
    true
}

/// NtGdiFlushUserBatch for a WoW64 guest: execute its TEB32 batch, then reset it.
#[cfg(not(target_arch = "x86"))]
unsafe fn flush_guest_batch() {
    let teb32 = crate::wow64::Wow64CurrentTeb32();
    if teb32.is_null() {
        return;
    }
    let batch = core::ptr::addr_of_mut!((*teb32).GdiTebBatch);
    let count = (*teb32).GdiBatchCount;
    let buf = (*batch).Buffer.as_ptr() as *const u8;
    let buf_len = core::mem::size_of_val(&(*batch).Buffer);

    if count > 0 && count <= (GDIBATCHBUFSIZE / 4) {
        let hDC = guest_handle((*batch).HDC) as HDC;
        let mut pDC: PDC = core::ptr::null_mut();
        if GDI_HANDLE_GET_TYPE(hDC as HANDLE) == GDILoObjType_GDILoObjType_LO_DC_TYPE as ULONG_PTR
            && GreIsHandleValid(hDC as HGDIOBJ) != 0
        {
            pDC = DC_LockDc(hDC);
        }

        // Widest record: a batch-filling PolyPatBlt, whose 20-byte PATRECTs grow to 24.
        let mut scratch = [0u64; 256];
        let dst = scratch.as_mut_ptr() as *mut u8;
        let mut off = 0usize;
        for _ in 0..count {
            if off + core::mem::size_of::<GDIBATCHHDR>() > buf_len {
                break;
            }
            let size = (*(buf.add(off) as *const GDIBATCHHDR)).Size as u16 as usize;
            if size < core::mem::size_of::<GDIBATCHHDR>() || off + size > buf_len {
                break;
            }
            if !widen_guest_record(buf.add(off), size, dst, core::mem::size_of_val(&scratch)) {
                break;
            }
            GdiFlushUserBatch_impl(pDC, dst as PGDIBATCHHDR);
            off += size;
        }

        if !pDC.is_null() {
            DC_UnlockDc(pDC);
        }
    }

    (*batch).Offset = 0;
    (*batch).HDC = 0;
    (*teb32).GdiBatchCount = 0;
}

// ── NtGdiFlush ─────────────────────────────────────────────────────────────
// Flushes the calling thread's current batch.
// stdcall: _NtGdiFlush@0

#[no_mangle]
pub unsafe extern "system" fn NtGdiFlush() -> NTSTATUS {
    SynchronizeDriver_impl(GCAPS2_SYNCFLUSH);
    STATUS_SUCCESS
}

// ── NtGdiFlushUserBatch ────────────────────────────────────────────────────
// Callback for thread batch flush routine. Think small & fast!
// stdcall: _NtGdiFlushUserBatch@0

#[no_mangle]
pub unsafe extern "system" fn NtGdiFlushUserBatch() -> NTSTATUS {
    let pTeb = NtCurrentTeb() as *mut ntkrnl_sys::ddk::_TEB;

    // A WoW64 guest batches into its TEB32, never this one.
    #[cfg(not(target_arch = "x86"))]
    if crate::wow64::IsWow64Process() {
        flush_guest_batch();
        return STATUS_SUCCESS;
    }

    let GdiBatchCount = (*pTeb).GdiBatchCount;

    if GdiBatchCount > 0 && GdiBatchCount <= (GDIBATCHBUFSIZE / 4) {
        let hDC = (*pTeb).GdiTebBatch.HDC as HDC;

        // If hDC is zero and the buffer fills up with delete objects we need
        // to run anyway.
        if !hDC.is_null() || GdiBatchCount > 0 {
            let mut pHdr = (*pTeb).GdiTebBatch.Buffer.as_mut_ptr() as *mut u8;
            let mut pDC: PDC = core::ptr::null_mut();

            if GDI_HANDLE_GET_TYPE(hDC as HANDLE)
                == GDILoObjType_GDILoObjType_LO_DC_TYPE as ULONG_PTR
                && GreIsHandleValid(hDC as HGDIOBJ) != 0
            {
                pDC = DC_LockDc(hDC);
            }

            // No need to init anything, just go!
            let mut count = GdiBatchCount;
            while count > 0 {
                let sz = GdiFlushUserBatch_impl(pDC, pHdr as PGDIBATCHHDR);
                if sz == 0 {
                    break;
                }
                pHdr = pHdr.add(sz as usize);
                count -= 1;
            }

            if !pDC.is_null() {
                DC_UnlockDc(pDC);
            }

            // Exit and clear out for the next round.
            (*pTeb).GdiTebBatch.Offset = 0;
            (*pTeb).GdiBatchCount = 0;
            (*pTeb).GdiTebBatch.HDC = core::ptr::null_mut();
        }
    }

    // FIXME: On Windows XP the function returns &pTeb->RealClientId, maybe VOID?
    STATUS_SUCCESS
}
