use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{RtlCopyUnicodeString, RtlEqualUnicodeString};
use core::ffi::c_void;
type VOID = core::ffi::c_void;

// ── DC-lifecycle shared preamble (hand-added; macros/consts rustconv can't see) ──
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::base::STATUS_SUCCESS;
const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC0000001u32 as i32;
// GDITAG_HMGR_LOOKASIDE_DC_TYPE = '1alG' (big-endian multichar)

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}

// EXLATEOBJ_* real ENGAPI (xlateobj.rs) — declare extern, same-ABI (mirror fillshap).
extern "system" {
    fn EXLATEOBJ_vInitialize(pexlo: PEXLATEOBJ, ppalSrc: *mut PALETTE, ppalDst: *mut PALETTE, c1: COLORREF, c2: COLORREF, c3: COLORREF);
    fn EXLATEOBJ_vCleanup(exlo: *mut EXLATEOBJ);
}
#[cfg(target_arch = "x86")] extern "fastcall" { fn PEN_ShareLockPen(hpen: HPEN) -> PBRUSH; }
#[cfg(not(target_arch = "x86"))] extern "C" { fn PEN_ShareLockPen(hpen: HPEN) -> PBRUSH; }
#[inline(always)] fn UlongToHandle(u: ULONG) -> HANDLE { u as usize as HANDLE }
#[inline(always)] unsafe fn InterlockedCompareExchangePointer(dst: *mut PVOID, xchg: PVOID, cmp: PVOID) -> PVOID {
    let a = &*(dst as *const core::sync::atomic::AtomicPtr<core::ffi::c_void>);
    match a.compare_exchange(cmp, xchg, core::sync::atomic::Ordering::SeqCst, core::sync::atomic::Ordering::SeqCst) { Ok(v) => v, Err(v) => v }
}
const TAG_DC: u32 = 0x31616c47;
#[inline(always)] fn NT_SUCCESS(s: NTSTATUS) -> bool { s >= 0 }
#[inline(always)] unsafe fn RtlCopyMemory(d: *mut c_void, s: *const c_void, n: usize) {
    core::ptr::copy_nonoverlapping(s as *const u8, d as *mut u8, n);
}
// GDI object share/lock macros (surface.h/palette.h/path.h/text.h) — inline.
#[inline(always)] unsafe fn LFONT_ShareUnlockFont(p: PLFONT) { GDIOBJ_vDereferenceObject(p as POBJ); }
#[inline(always)] unsafe fn LFONT_ShareLockFont(h: HGDIOBJ) -> PLFONT { GDIOBJ_ReferenceObjectByHandle(h, GDIObjType_LFONT_TYPE as u8) as PLFONT }
#[inline(always)] unsafe fn SURFACE_ShareLockSurface(h: HGDIOBJ) -> PSURFACE { GDIOBJ_ShareLockObj(h, GDI_OBJECT_TYPE_BITMAP as DWORD) as PSURFACE }
#[inline(always)] unsafe fn SURFACE_ShareUnlockSurface(p: PSURFACE) { GDIOBJ_vDereferenceObject(p as POBJ); }
#[inline(always)] unsafe fn PALETTE_ShareLockPalette(h: HGDIOBJ) -> PPALETTE { GDIOBJ_ShareLockObj(h, GDI_OBJECT_TYPE_PALETTE as DWORD) as PPALETTE }
#[inline(always)] unsafe fn PALETTE_ShareUnlockPalette(p: PPALETTE) { GDIOBJ_vDereferenceObject(core::ptr::addr_of_mut!((*p).BaseObject)); }
#[inline(always)] unsafe fn PATH_LockPath(h: HGDIOBJ) -> PPATH { GDIOBJ_ShareLockObj(h, GDI_OBJECT_TYPE_PATH as DWORD) as PPATH }
#[inline(always)] unsafe fn PATH_UnlockPath(p: PPATH) { GDIOBJ_vDereferenceObject(p as POBJ); }
// FLOATOBJ constants (engobjects.h) — per-arch (i386 struct / non-x86 f32).
#[cfg(target_arch = "x86")] const FLOATOBJ_0: FLOATOBJ = FLOATOBJ { ul1: 0x00000000, ul2: 0x00000000 };
#[cfg(target_arch = "x86")] const FLOATOBJ_1: FLOATOBJ = FLOATOBJ { ul1: 0x40000000, ul2: 0x00000002 };
#[cfg(target_arch = "x86")] const FLOATOBJ_16: FLOATOBJ = FLOATOBJ { ul1: 0x40000000, ul2: 0x00000006 };
#[cfg(target_arch = "x86")] const FLOATOBJ_1_16: FLOATOBJ = FLOATOBJ { ul1: 0x40000000, ul2: 0xfffffffe };
#[cfg(not(target_arch = "x86"))] const FLOATOBJ_0: FLOATOBJ = 0.;
#[cfg(not(target_arch = "x86"))] const FLOATOBJ_1: FLOATOBJ = 1.;
#[cfg(not(target_arch = "x86"))] const FLOATOBJ_16: FLOATOBJ = 16.;
#[cfg(not(target_arch = "x86"))] const FLOATOBJ_1_16: FLOATOBJ = 1. / 16.;
// ── end shared preamble ──
#[cfg(target_arch = "x86")] extern "fastcall" { fn TextIntRealizeFont(hfont: HFONT, pto: PTEXTOBJ) -> NTSTATUS; fn ftGdiGetTextCharsetInfo(pdc: PDC, pfs: *mut FONTSIGNATURE, fl: DWORD) -> INT; }
#[cfg(not(target_arch = "x86"))] extern "C" { fn TextIntRealizeFont(hfont: HFONT, pto: PTEXTOBJ) -> NTSTATUS; fn ftGdiGetTextCharsetInfo(pdc: PDC, pfs: *mut FONTSIGNATURE, fl: DWORD) -> INT; }
#[inline(always)] fn GDI_HANDLE_GET_TYPE(h: HANDLE) -> ULONG { ((h as usize) & 0x007f0000) as ULONG }
#[inline(always)] fn GDI_HANDLE_IS_STOCKOBJ(h: HANDLE) -> BOOL { (((h as usize) & 0x00800000) != 0) as BOOL }
#[inline(always)] unsafe fn RtlInitEmptyUnicodeString(dst: *mut _UNICODE_STRING, buf: *mut u16, len: USHORT) { (*dst).Length = 0; (*dst).MaximumLength = len; (*dst).Buffer = buf; }
#[inline(always)] unsafe fn RTL_CONSTANT_STRING(s: *const u16) -> _UNICODE_STRING { let mut n = 0usize; while core::ptr::read_unaligned(s.add(n)) != 0 { n += 1; } _UNICODE_STRING { Length: (n * 2) as USHORT, MaximumLength: ((n + 1) * 2) as USHORT, Buffer: s as *mut u16 } }
#[cfg(target_arch = "x86")] extern "fastcall" { fn CLIPPING_UpdateGCRegion(pdc: PDC); }
#[cfg(not(target_arch = "x86"))] extern "C" { fn CLIPPING_UpdateGCRegion(pdc: PDC); }
extern "system" { fn MouseSafetyOnDrawEnd(ppdev: *mut core::ffi::c_void) -> u32; }
#[inline(always)] fn RGB(r: u32, g: u32, b: u32) -> COLORREF { (r & 0xff) | ((g & 0xff) << 8) | ((b & 0xff) << 16) }
#[cfg(target_arch = "x86")] extern "fastcall" { fn UpdateVisRgn(pdc: PDC); }
#[cfg(not(target_arch = "x86"))] extern "C" { fn UpdateVisRgn(pdc: PDC); }
#[inline(always)] unsafe fn BRUSH_ShareLockBrush(h: HGDIOBJ) -> PBRUSH { GDIOBJ_ShareLockObj(h, GDI_OBJECT_TYPE_BRUSH as DWORD) as PBRUSH }
#[inline(always)] unsafe fn BRUSH_ShareUnlockBrush(pbr: PBRUSH) { GDIOBJ_vDereferenceObject(pbr as POBJ); }
// DC_vSelect*/BRUSH_ShareUnlockBrush are FORCEINLINE (dc.h/brush.h) — inline locally.
#[inline(always)] unsafe fn DC_vSelectSurface(pdc: PDC, psurfNew: PSURFACE) {
    let psurfOld = (*pdc).dclevel.pSurface;
    if !psurfOld.is_null() { (*psurfOld).hdc = core::ptr::null_mut(); SURFACE_ShareUnlockSurface(psurfOld); }
    if !psurfNew.is_null() { GDIOBJ_vReferenceObjectByPointer(psurfNew as POBJ); }
    (*pdc).dclevel.pSurface = psurfNew;
}
#[inline(always)] unsafe fn DC_vSelectFillBrush(pdc: PDC, pbrFill: PBRUSH) {
    let o = (*pdc).dclevel.pbrFill;
    if !o.is_null() { BRUSH_ShareUnlockBrush(o); }
    if !pbrFill.is_null() { GDIOBJ_vReferenceObjectByPointer(pbrFill as POBJ); }
    (*pdc).dclevel.pbrFill = pbrFill;
}
#[inline(always)] unsafe fn DC_vSelectLineBrush(pdc: PDC, pbrLine: PBRUSH) {
    let o = (*pdc).dclevel.pbrLine;
    if !o.is_null() { BRUSH_ShareUnlockBrush(o); }
    if !pbrLine.is_null() { GDIOBJ_vReferenceObjectByPointer(pbrLine as POBJ); }
    (*pdc).dclevel.pbrLine = pbrLine;
}
#[inline(always)] unsafe fn DC_vSelectPalette(pdc: PDC, ppal: PPALETTE) {
    let o = (*pdc).dclevel.ppal;
    if !o.is_null() { PALETTE_ShareUnlockPalette(o); }
    if !ppal.is_null() { GDIOBJ_vReferenceObjectByPointer(ppal as POBJ); }
    (*pdc).dclevel.ppal = ppal;
}

// ABI: EBRUSHOBJ_vCleanup=NTAPI(system), MouseSafetyOnDrawStart=stdcall(≡system on i386).
// rustconv put these in extern "C" → on i386 the cdecl decoration mismatches the
// stdcall definition → crate-wide CC collision folds callers to ud2. (feedback_dup_abi_name_collision)
extern "system" {
    fn EBRUSHOBJ_vCleanup(pebo: *mut EBRUSHOBJ);
    fn MouseSafetyOnDrawStart(ppdev: *mut core::ffi::c_void, left: i32, top: i32, right: i32, bottom: i32) -> u32;
}
// drop_gdiobj_macro_externs: dropped 2 bogus extern decl(s) for GDI-object macro(s): LFONT_ShareUnlockFont, SURFACE_ShareUnlockSurface — these are macros in gdi/eng/<type>.h or gdi/ntgdi/<type>.h, not real fns. Inline-expand via GDIOBJ_* primitives per win32k_translation_prompt.md §6.
// ABI: EBRUSHOBJ_vInitFromDC=stdcall, EngpGetPDEV=NTAPI(system) — both ≡ system on i386.
// EngpGetPDEV is the GreOpenDCW killer: defined extern "system" in pdevobj.rs, declared
// extern "C" here → CC mismatch made LLVM fold GreOpenDCW's whole body to ud2.
extern "system" {
    fn EBRUSHOBJ_vInitFromDC(pebo: PEBRUSHOBJ, pbrush: PBRUSH, pdc: PDC);
    fn EngpGetPDEV(pustrDevice: PUNICODE_STRING) -> PPDEVOBJ;
}
// SetLastNtError is FASTCALL (@SetLastNtError@4 on i386) — needs dual-cfg.
#[cfg(target_arch = "x86")] extern "fastcall" { fn SetLastNtError(Status: NTSTATUS); }
#[cfg(not(target_arch = "x86"))] extern "C" { fn SetLastNtError(Status: NTSTATUS); }
// drop_gdiobj_macro_externs: dropped 2 bogus extern decl(s) for GDI-object macro(s): LFONT_ShareLockFont, PALETTE_ShareLockPalette — these are macros in gdi/eng/<type>.h or gdi/ntgdi/<type>.h, not real fns. Inline-expand via GDIOBJ_* primitives per win32k_translation_prompt.md §6.
// ---- file-scope preamble (translated) ----
pub const DIRTY_DEFAULT: u32 = DIRTY_CHARSET | DIRTY_BACKGROUND | DIRTY_TEXT | DIRTY_LINE | DIRTY_FILL;

#[no_mangle]
pub static mut psurfDefaultBitmap: PSURFACE = core::ptr::null_mut();
#[no_mangle]
pub static mut pbrDefaultBrush: PBRUSH = core::ptr::null_mut();

#[no_mangle]
pub static gmxWorldToDeviceDefault: MATRIX = MATRIX {
    efM11: FLOATOBJ_16,
    efM12: FLOATOBJ_0,
    efM21: FLOATOBJ_0,
    efM22: FLOATOBJ_16,
    efDx: FLOATOBJ_0,
    efDy: FLOATOBJ_0,
    fxDx: 0,
    fxDy: 0,
    flAccel: 0x4b,
};

pub static gmxDeviceToWorldDefault: MATRIX = MATRIX {
    efM11: FLOATOBJ_1_16,
    efM12: FLOATOBJ_0,
    efM21: FLOATOBJ_0,
    efM22: FLOATOBJ_1_16,
    efDx: FLOATOBJ_0,
    efDy: FLOATOBJ_0,
    fxDx: 0,
    fxDy: 0,
    flAccel: 0x53,
};

#[no_mangle]
pub static gmxWorldToPageDefault: MATRIX = MATRIX {
    efM11: FLOATOBJ_1,
    efM12: FLOATOBJ_0,
    efM21: FLOATOBJ_0,
    efM22: FLOATOBJ_1,
    efDx: FLOATOBJ_0,
    efDy: FLOATOBJ_0,
    fxDx: 0,
    fxDy: 0,
    flAccel: 0x63,
};

#[no_mangle]
pub unsafe extern "system" fn InitDcImpl() -> NTSTATUS {
    psurfDefaultBitmap = SURFACE_ShareLockSurface((*(*(&raw const StockObjects)).as_ptr().add(DEFAULT_BITMAP as usize)) as _);
    if psurfDefaultBitmap.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    pbrDefaultBrush = BRUSH_ShareLockBrush((*(*(&raw const StockObjects)).as_ptr().add(BLACK_BRUSH as usize)) as _);
    if pbrDefaultBrush.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    return STATUS_SUCCESS;
}
#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn DC_AllocDcWithHandle(eDcObjType: GDILOOBJTYPE) -> PDC {
    let pdc: PDC;

    pdc = GDIOBJ_AllocateObject(
        GDIObjType_DC_TYPE as u8,
        core::mem::size_of::<DC>() as u32,
        BASEFLAG_LOOKASIDE,
    ) as PDC;
    if pdc.is_null() {
        return core::ptr::null_mut();
    }

    (*pdc).BaseObject.hHmgr = UlongToHandle(eDcObjType);

    (*pdc).pdcattr = core::ptr::addr_of_mut!((*pdc).dcattr);

    if GDIOBJ_hInsertObject(
        core::ptr::addr_of_mut!((*pdc).BaseObject),
        GDI_OBJ_HMGR_POWNED,
    ).is_null()
    {
        GDIOBJ_vFreeObject(core::ptr::addr_of_mut!((*pdc).BaseObject));
        return core::ptr::null_mut();
    }

    pdc
}

unsafe fn DC_InitHack(pdc: PDC) {
    if defaultDCstate.is_null() {
        defaultDCstate = ExAllocatePoolWithTag(PagedPool, core::mem::size_of::<DC>() as SIZE_T, TAG_DC) as PDC;
        core::ptr::write_bytes(defaultDCstate, 0, 1);
        (*defaultDCstate).pdcattr = core::ptr::addr_of_mut!((*defaultDCstate).dcattr);
        DC_vCopyState(pdc, defaultDCstate, 1);
    }

    if prgnDefault.is_null() {
        prgnDefault = IntSysCreateRectpRgn(0, 0, 0, 0);
    }

    #[cfg(target_arch = "aarch64")]
    {
        let fontStatus = TextIntRealizeFont((*(*pdc).pdcattr).hlfntNew as _, core::ptr::null_mut());
        if NT_SUCCESS(fontStatus) {
            (*(*pdc).pdcattr).iCS_CP = ftGdiGetTextCharsetInfo(pdc, core::ptr::null_mut(), 0) as u32;
        } else {
            (*(*pdc).pdcattr).iCS_CP = 0;
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        TextIntRealizeFont((*(*pdc).pdcattr).hlfntNew as _, core::ptr::null_mut());
        (*(*pdc).pdcattr).iCS_CP = ftGdiGetTextCharsetInfo(pdc, core::ptr::null_mut(), 0) as u32;
    }
}
#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn DC_vInitDc(
    pdc: PDC,
    dctype: DCTYPE,
    ppdev: PPDEVOBJ,
) {

    (*pdc).dctype = dctype;
    (*pdc).ppdev = ppdev;
    (*pdc).dhpdev = (*ppdev).dhpdev;
    (*pdc).hsem = (*ppdev).hsemDevLock as _;
    (*pdc).flGraphicsCaps = (*ppdev).devinfo.flGraphicsCaps;
    (*pdc).flGraphicsCaps2 = (*ppdev).devinfo.flGraphicsCaps2;
    (*pdc).fs = (DC_DIRTY_RAO as i32);

    (*pdc).pdcattr = core::ptr::addr_of_mut!((*pdc).dcattr);
    (*pdc).dcattr.pvLDC = core::ptr::null_mut();
    (*pdc).dcattr.ulDirty_ = DIRTY_DEFAULT;
    if ppdev == (*gpmdev).ppdevGlobal {
        (*pdc).dcattr.ulDirty_ |= DC_PRIMARY_DISPLAY;
    }

    if dctype == DCTYPE_MEMORY {
        (*pdc).dclevel.sizl.cx = 1;
        (*pdc).dclevel.sizl.cy = 1;
    } else {
        (*pdc).dclevel.sizl.cx = (*ppdev).gdiinfo.ulHorzRes as i32;
        (*pdc).dclevel.sizl.cy = (*ppdev).gdiinfo.ulVertRes as i32;
    }

    (*pdc).erclWindow.left = 0;
    (*pdc).erclWindow.top = 0;
    (*pdc).erclWindow.right = (*pdc).dclevel.sizl.cx;
    (*pdc).erclWindow.bottom = (*pdc).dclevel.sizl.cy;

    if dctype == DCTYPE_DIRECT {
        (*pdc).dclevel.pSurface = PDEVOBJ_pSurface(ppdev);

        (*pdc).erclBounds.left = 0x7fffffff;
        (*pdc).erclBounds.top = 0x7fffffff;
        (*pdc).erclBounds.right = 0x80000000u32 as i32;
        (*pdc).erclBounds.bottom = 0x80000000u32 as i32;
        (*pdc).erclBoundsApp.left = 0xffffffffu32 as i32;
        (*pdc).erclBoundsApp.top = 0xfffffffcu32 as i32;
        (*pdc).erclBoundsApp.right = 0x00007ffc;
        (*pdc).erclBoundsApp.bottom = 0x00000333;
        (*pdc).erclClip = (*pdc).erclBounds;
        (*pdc).co = gxcoTrivial;
    } else {
        (*pdc).dclevel.pSurface = core::ptr::null_mut();

        (*pdc).erclBounds.left = 0;
        (*pdc).erclBounds.top = 0;
        (*pdc).erclBounds.right = 0;
        (*pdc).erclBounds.bottom = 0;
        (*pdc).erclBoundsApp = (*pdc).erclBounds;
        (*pdc).erclClip = (*pdc).erclWindow;
        (*pdc).co = gxcoTrivial;
    }

    // dclife.c HACK (`#define gmxWorldToDeviceDefault gmxWorldToPageDefault` /
    // `gmxDeviceToWorldDefault gmxWorldToPageDefault`): the real 16:1 / 1:16
    // defaults are NOT used — all three init to the 1:1 identity page matrix
    // until XFORMOBJ is fixed. (Dropping this #define left DCs with a 16x
    // mxWorldToDevice → DPtoLP/clip box came out screen/16 → blank desktop.)
    (*pdc).dclevel.mxWorldToDevice = gmxWorldToPageDefault;
    (*pdc).dclevel.mxDeviceToWorld = gmxWorldToPageDefault;
    (*pdc).dclevel.mxWorldToPage = gmxWorldToPageDefault;
    (*pdc).dclevel.efM11PtoD = gef16;
    (*pdc).dclevel.efM22PtoD = gef16;
    (*pdc).dclevel.efDxPtoD = gef0;
    (*pdc).dclevel.efDyPtoD = gef0;
    (*pdc).dclevel.efM11_TWIPS = gef0;
    (*pdc).dclevel.efM22_TWIPS = gef0;
    (*pdc).dclevel.efPr11 = gef0;
    (*pdc).dclevel.efPr22 = gef0;
    (*pdc).dcattr.mxWorldToDevice = (*pdc).dclevel.mxWorldToDevice;
    (*pdc).dcattr.mxDeviceToWorld = (*pdc).dclevel.mxDeviceToWorld;
    (*pdc).dcattr.mxWorldToPage = (*pdc).dclevel.mxWorldToPage;
    (*pdc).dcattr.efM11PtoD = (*pdc).dclevel.efM11PtoD;
    (*pdc).dcattr.efM22PtoD = (*pdc).dclevel.efM22PtoD;
    (*pdc).dcattr.efDxPtoD = (*pdc).dclevel.efDxPtoD;
    (*pdc).dcattr.efDyPtoD = (*pdc).dclevel.efDyPtoD;
    (*pdc).dcattr.iMapMode = MM_TEXT as i32;
    (*pdc).dcattr.dwLayout = 0;
    (*pdc).dcattr.flXform = PAGE_TO_DEVICE_SCALE_IDENTITY |
                            PAGE_TO_DEVICE_IDENTITY |
                            WORLD_TO_PAGE_IDENTITY;

    (*pdc).ptlDCOrig.x = 0;
    (*pdc).ptlDCOrig.y = 0;
    (*pdc).dcattr.lWindowOrgx = 0;
    (*pdc).dcattr.ptlWindowOrg.x = 0;
    (*pdc).dcattr.ptlWindowOrg.y = 0;
    (*pdc).dcattr.szlWindowExt.cx = 1;
    (*pdc).dcattr.szlWindowExt.cy = 1;
    (*pdc).dcattr.ptlViewportOrg.x = 0;
    (*pdc).dcattr.ptlViewportOrg.y = 0;
    (*pdc).dcattr.szlViewportExt.cx = 1;
    (*pdc).dcattr.szlViewportExt.cy = 1;
    (*pdc).dcattr.szlVirtualDevicePixel.cx = (*ppdev).gdiinfo.ulHorzRes as i32;
    (*pdc).dcattr.szlVirtualDevicePixel.cy = (*ppdev).gdiinfo.ulVertRes as i32;
    (*pdc).dcattr.szlVirtualDeviceMm.cx = (*ppdev).gdiinfo.ulHorzSize as i32;
    (*pdc).dcattr.szlVirtualDeviceMm.cy = (*ppdev).gdiinfo.ulVertSize as i32;
    (*pdc).dcattr.szlVirtualDeviceSize.cx = 0;
    (*pdc).dcattr.szlVirtualDeviceSize.cy = 0;

    (*pdc).prgnAPI = core::ptr::null_mut();
    (*pdc).prgnRao = core::ptr::null_mut();
    (*pdc).dclevel.prgnClip = core::ptr::null_mut();
    (*pdc).dclevel.prgnMeta = core::ptr::null_mut();
    (*pdc).prgnVis = IntSysCreateRectpRgn(0, 0, (*pdc).dclevel.sizl.cx, (*pdc).dclevel.sizl.cy);

    UpdateVisRgn(pdc);

    IntEngInitClipObj(core::ptr::addr_of_mut!((*pdc).co));

    (*pdc).dclevel.hpal = (*(*(&raw const StockObjects)).as_ptr().add(DEFAULT_PALETTE as usize)) as _;
    (*pdc).dclevel.ppal = PALETTE_ShareLockPalette((*pdc).dclevel.hpal as _);

    (*pdc).dclevel.hPath = core::ptr::null_mut();
    (*pdc).dclevel.flPath = 0;

    (*pdc).dcattr.crBackgroundClr = RGB(0xff, 0xff, 0xff);
    (*pdc).dcattr.ulBackgroundClr = RGB(0xff, 0xff, 0xff);
    (*pdc).dcattr.crForegroundClr = RGB(0, 0, 0);
    (*pdc).dcattr.ulForegroundClr = RGB(0, 0, 0);
    (*pdc).dcattr.crBrushClr = RGB(0xff, 0xff, 0xff);
    (*pdc).dcattr.ulBrushClr = RGB(0xff, 0xff, 0xff);
    (*pdc).dcattr.crPenClr = RGB(0, 0, 0);
    (*pdc).dcattr.ulPenClr = RGB(0, 0, 0);

    (*pdc).dcattr.hbrush = (*(*(&raw const StockObjects)).as_ptr().add(WHITE_BRUSH as usize)) as _;
    (*pdc).dcattr.hpen = (*(*(&raw const StockObjects)).as_ptr().add(BLACK_PEN as usize)) as _;
    (*pdc).dclevel.pbrFill = BRUSH_ShareLockBrush((*(*pdc).pdcattr).hbrush as _);
    (*pdc).dclevel.pbrLine = PEN_ShareLockPen((*(*pdc).pdcattr).hpen as _);
    (*pdc).dclevel.ptlBrushOrigin.x = 0;
    (*pdc).dclevel.ptlBrushOrigin.y = 0;
    (*pdc).dcattr.ptlBrushOrigin = (*pdc).dclevel.ptlBrushOrigin;

    EBRUSHOBJ_vInitFromDC(core::ptr::addr_of_mut!((*pdc).eboFill), (*pdc).dclevel.pbrFill, pdc);
    EBRUSHOBJ_vInitFromDC(core::ptr::addr_of_mut!((*pdc).eboLine), (*pdc).dclevel.pbrLine, pdc);
    EBRUSHOBJ_vInitFromDC(core::ptr::addr_of_mut!((*pdc).eboText), pbrDefaultBrush, pdc);
    EBRUSHOBJ_vInitFromDC(core::ptr::addr_of_mut!((*pdc).eboBackground), pbrDefaultBrush, pdc);

    (*pdc).dcattr.jROP2 = R2_COPYPEN as u8;
    (*pdc).dcattr.jBkMode = 2;
    (*pdc).dcattr.lBkMode = 2;
    (*pdc).dcattr.jFillMode = ALTERNATE as u8;
    (*pdc).dcattr.lFillMode = 1;
    (*pdc).dcattr.jStretchBltMode = 1;
    (*pdc).dcattr.lStretchBltMode = 1;
    (*pdc).ptlFillOrigin.x = 0;
    (*pdc).ptlFillOrigin.y = 0;

    (*pdc).dcattr.ptlCurrent.x = 0;
    (*pdc).dcattr.ptlCurrent.y = 0;
    (*pdc).dcattr.ptfxCurrent.x = 0;
    (*pdc).dcattr.ptfxCurrent.y = 0;

    (*pdc).dclevel.lIcmMode = 0;
    (*pdc).dcattr.lIcmMode = 0;
    (*pdc).dcattr.hcmXform = core::ptr::null_mut();
    (*pdc).dcattr.flIcmFlags = 0;
    (*pdc).dcattr.IcmBrushColor = CLR_INVALID as i32;
    (*pdc).dcattr.IcmPenColor = CLR_INVALID as i32;
    (*pdc).dcattr.pvLIcm = core::ptr::null_mut();
    (*pdc).dcattr.hColorSpace = core::ptr::null_mut();
    (*pdc).dclevel.pColorSpace = core::ptr::null_mut();
    (*pdc).pClrxFormLnk = core::ptr::null_mut();

    (*pdc).hlfntCur = core::ptr::null_mut();
    (*pdc).pPFFList = core::ptr::null_mut();
    (*pdc).flSimulationFlags = 0;
    (*pdc).lEscapement = 0;
    (*pdc).prfnt = core::ptr::null_mut();
    (*pdc).dcattr.flFontMapper = 0;
    (*pdc).dcattr.flTextAlign = 0;
    (*pdc).dcattr.lTextAlign = 0;
    (*pdc).dcattr.lTextExtra = 0;
    (*pdc).dcattr.lRelAbs = 1;
    (*pdc).dcattr.lBreakExtra = 0;
    (*pdc).dcattr.cBreak = 0;
    (*pdc).dcattr.hlfntNew = (*(*(&raw const StockObjects)).as_ptr().add(SYSTEM_FONT as usize)) as _;
    (*pdc).dclevel.plfnt = LFONT_ShareLockFont((*pdc).dcattr.hlfntNew as _);

    (*pdc).hdcNext = core::ptr::null_mut();
    (*pdc).hdcPrev = core::ptr::null_mut();
    (*pdc).ipfdDevMax = 0;
    (*pdc).ulCopyCount = 0xffffffffu32;
    (*pdc).ptlDoBanding.x = 0;
    (*pdc).ptlDoBanding.y = 0;
    (*pdc).dclevel.lSaveDepth = 1;
    (*pdc).dclevel.hdcSave = core::ptr::null_mut();
    (*pdc).dcattr.iGraphicsMode = GM_COMPATIBLE as i32;
    (*pdc).dcattr.iCS_CP = 0;
    (*pdc).pSurfInfo = core::ptr::null_mut();
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn DC_vInitDc(
    pdc: PDC,
    dctype: DCTYPE,
    ppdev: PPDEVOBJ,
) {

    (*pdc).dctype = dctype;
    (*pdc).ppdev = ppdev;
    (*pdc).dhpdev = (*ppdev).dhpdev;
    (*pdc).hsem = (*ppdev).hsemDevLock as _;
    (*pdc).flGraphicsCaps = (*ppdev).devinfo.flGraphicsCaps;
    (*pdc).flGraphicsCaps2 = (*ppdev).devinfo.flGraphicsCaps2;
    (*pdc).fs = DC_DIRTY_RAO as i32 ;

    (*pdc).pdcattr = core::ptr::addr_of_mut!((*pdc).dcattr);
    (*pdc).dcattr.pvLDC = core::ptr::null_mut();
    (*pdc).dcattr.ulDirty_ = DIRTY_DEFAULT;
    if ppdev == (*gpmdev).ppdevGlobal {
        (*pdc).dcattr.ulDirty_ |= DC_PRIMARY_DISPLAY;
    }

    if dctype == DCTYPE_MEMORY {
        (*pdc).dclevel.sizl.cx = 1;
        (*pdc).dclevel.sizl.cy = 1;
    } else {
        (*pdc).dclevel.sizl.cx = (*ppdev).gdiinfo.ulHorzRes as i32;
        (*pdc).dclevel.sizl.cy = (*ppdev).gdiinfo.ulVertRes as i32;
    }

    (*pdc).erclWindow.left = 0;
    (*pdc).erclWindow.top = 0;
    (*pdc).erclWindow.right = (*pdc).dclevel.sizl.cx;
    (*pdc).erclWindow.bottom = (*pdc).dclevel.sizl.cy;

    if dctype == DCTYPE_DIRECT {
        (*pdc).dclevel.pSurface = PDEVOBJ_pSurface(ppdev);

        (*pdc).erclBounds.left = 0x7fffffff;
        (*pdc).erclBounds.top = 0x7fffffff;
        (*pdc).erclBounds.right = 0x80000000u32 as i32;
        (*pdc).erclBounds.bottom = 0x80000000u32 as i32;
        (*pdc).erclBoundsApp.left = 0xffffffffu32 as i32;
        (*pdc).erclBoundsApp.top = 0xfffffffcu32 as i32;
        (*pdc).erclBoundsApp.right = 0x00007ffc;
        (*pdc).erclBoundsApp.bottom = 0x00000333;
        (*pdc).erclClip = (*pdc).erclBounds;
        (*pdc).co = gxcoTrivial;
    } else {
        (*pdc).dclevel.pSurface = core::ptr::null_mut();

        (*pdc).erclBounds.left = 0;
        (*pdc).erclBounds.top = 0;
        (*pdc).erclBounds.right = 0;
        (*pdc).erclBounds.bottom = 0;
        (*pdc).erclBoundsApp = (*pdc).erclBounds;
        (*pdc).erclClip = (*pdc).erclWindow;
        (*pdc).co = gxcoTrivial;
    }

    // dclife.c HACK (`#define gmxWorldToDeviceDefault gmxWorldToPageDefault` /
    // `gmxDeviceToWorldDefault gmxWorldToPageDefault`): the real 16:1 / 1:16
    // defaults are NOT used — all three init to the 1:1 identity page matrix
    // until XFORMOBJ is fixed. (Dropping this #define left DCs with a 16x
    // mxWorldToDevice → DPtoLP/clip box came out screen/16 → blank desktop.)
    (*pdc).dclevel.mxWorldToDevice = gmxWorldToPageDefault;
    (*pdc).dclevel.mxDeviceToWorld = gmxWorldToPageDefault;
    (*pdc).dclevel.mxWorldToPage = gmxWorldToPageDefault;
    (*pdc).dclevel.efM11PtoD = gef16;
    (*pdc).dclevel.efM22PtoD = gef16;
    (*pdc).dclevel.efDxPtoD = gef0;
    (*pdc).dclevel.efDyPtoD = gef0;
    (*pdc).dclevel.efM11_TWIPS = gef0;
    (*pdc).dclevel.efM22_TWIPS = gef0;
    (*pdc).dclevel.efPr11 = gef0;
    (*pdc).dclevel.efPr22 = gef0;
    (*pdc).dcattr.mxWorldToDevice = (*pdc).dclevel.mxWorldToDevice;
    (*pdc).dcattr.mxDeviceToWorld = (*pdc).dclevel.mxDeviceToWorld;
    (*pdc).dcattr.mxWorldToPage = (*pdc).dclevel.mxWorldToPage;
    (*pdc).dcattr.efM11PtoD = (*pdc).dclevel.efM11PtoD;
    (*pdc).dcattr.efM22PtoD = (*pdc).dclevel.efM22PtoD;
    (*pdc).dcattr.efDxPtoD = (*pdc).dclevel.efDxPtoD;
    (*pdc).dcattr.efDyPtoD = (*pdc).dclevel.efDyPtoD;
    (*pdc).dcattr.iMapMode = MM_TEXT as i32;
    (*pdc).dcattr.dwLayout = 0;
    (*pdc).dcattr.flXform = PAGE_TO_DEVICE_SCALE_IDENTITY |
                            PAGE_TO_DEVICE_IDENTITY |
                            WORLD_TO_PAGE_IDENTITY;

    (*pdc).ptlDCOrig.x = 0;
    (*pdc).ptlDCOrig.y = 0;
    (*pdc).dcattr.lWindowOrgx = 0;
    (*pdc).dcattr.ptlWindowOrg.x = 0;
    (*pdc).dcattr.ptlWindowOrg.y = 0;
    (*pdc).dcattr.szlWindowExt.cx = 1;
    (*pdc).dcattr.szlWindowExt.cy = 1;
    (*pdc).dcattr.ptlViewportOrg.x = 0;
    (*pdc).dcattr.ptlViewportOrg.y = 0;
    (*pdc).dcattr.szlViewportExt.cx = 1;
    (*pdc).dcattr.szlViewportExt.cy = 1;
    (*pdc).dcattr.szlVirtualDevicePixel.cx = (*ppdev).gdiinfo.ulHorzRes as i32;
    (*pdc).dcattr.szlVirtualDevicePixel.cy = (*ppdev).gdiinfo.ulVertRes as i32;
    (*pdc).dcattr.szlVirtualDeviceMm.cx = (*ppdev).gdiinfo.ulHorzSize as i32;
    (*pdc).dcattr.szlVirtualDeviceMm.cy = (*ppdev).gdiinfo.ulVertSize as i32;
    (*pdc).dcattr.szlVirtualDeviceSize.cx = 0;
    (*pdc).dcattr.szlVirtualDeviceSize.cy = 0;

    (*pdc).prgnAPI = core::ptr::null_mut();
    (*pdc).prgnRao = core::ptr::null_mut();
    (*pdc).dclevel.prgnClip = core::ptr::null_mut();
    (*pdc).dclevel.prgnMeta = core::ptr::null_mut();
    (*pdc).prgnVis = IntSysCreateRectpRgn(0, 0, (*pdc).dclevel.sizl.cx, (*pdc).dclevel.sizl.cy);

    UpdateVisRgn(pdc);

    IntEngInitClipObj(core::ptr::addr_of_mut!((*pdc).co));

    (*pdc).dclevel.hpal = (*(*(&raw const StockObjects)).as_ptr().add(DEFAULT_PALETTE as usize)) as _;
    (*pdc).dclevel.ppal = PALETTE_ShareLockPalette((*pdc).dclevel.hpal as _);

    (*pdc).dclevel.hPath = core::ptr::null_mut();
    (*pdc).dclevel.flPath = 0;

    (*pdc).dcattr.crBackgroundClr = RGB(0xff, 0xff, 0xff);
    (*pdc).dcattr.ulBackgroundClr = RGB(0xff, 0xff, 0xff);
    (*pdc).dcattr.crForegroundClr = RGB(0, 0, 0);
    (*pdc).dcattr.ulForegroundClr = RGB(0, 0, 0);
    (*pdc).dcattr.crBrushClr = RGB(0xff, 0xff, 0xff);
    (*pdc).dcattr.ulBrushClr = RGB(0xff, 0xff, 0xff);
    (*pdc).dcattr.crPenClr = RGB(0, 0, 0);
    (*pdc).dcattr.ulPenClr = RGB(0, 0, 0);

    (*pdc).dcattr.hbrush = (*(*(&raw const StockObjects)).as_ptr().add(WHITE_BRUSH as usize)) as _;
    (*pdc).dcattr.hpen = (*(*(&raw const StockObjects)).as_ptr().add(BLACK_PEN as usize)) as _;
    (*pdc).dclevel.pbrFill = BRUSH_ShareLockBrush((*(*pdc).pdcattr).hbrush as _);
    (*pdc).dclevel.pbrLine = PEN_ShareLockPen((*(*pdc).pdcattr).hpen as _);
    (*pdc).dclevel.ptlBrushOrigin.x = 0;
    (*pdc).dclevel.ptlBrushOrigin.y = 0;
    (*pdc).dcattr.ptlBrushOrigin = (*pdc).dclevel.ptlBrushOrigin;

    EBRUSHOBJ_vInitFromDC(core::ptr::addr_of_mut!((*pdc).eboFill), (*pdc).dclevel.pbrFill, pdc);
    EBRUSHOBJ_vInitFromDC(core::ptr::addr_of_mut!((*pdc).eboLine), (*pdc).dclevel.pbrLine, pdc);
    EBRUSHOBJ_vInitFromDC(core::ptr::addr_of_mut!((*pdc).eboText), pbrDefaultBrush, pdc);
    EBRUSHOBJ_vInitFromDC(core::ptr::addr_of_mut!((*pdc).eboBackground), pbrDefaultBrush, pdc);

    (*pdc).dcattr.jROP2 = R2_COPYPEN as u8;
    (*pdc).dcattr.jBkMode = 2;
    (*pdc).dcattr.lBkMode = 2;
    (*pdc).dcattr.jFillMode = ALTERNATE as u8;
    (*pdc).dcattr.lFillMode = 1;
    (*pdc).dcattr.jStretchBltMode = 1;
    (*pdc).dcattr.lStretchBltMode = 1;
    (*pdc).ptlFillOrigin.x = 0;
    (*pdc).ptlFillOrigin.y = 0;

    (*pdc).dcattr.ptlCurrent.x = 0;
    (*pdc).dcattr.ptlCurrent.y = 0;
    (*pdc).dcattr.ptfxCurrent.x = 0;
    (*pdc).dcattr.ptfxCurrent.y = 0;

    (*pdc).dclevel.lIcmMode = 0;
    (*pdc).dcattr.lIcmMode = 0;
    (*pdc).dcattr.hcmXform = core::ptr::null_mut();
    (*pdc).dcattr.flIcmFlags = 0;
    (*pdc).dcattr.IcmBrushColor = CLR_INVALID as i32;
    (*pdc).dcattr.IcmPenColor = CLR_INVALID as i32;
    (*pdc).dcattr.pvLIcm = core::ptr::null_mut();
    (*pdc).dcattr.hColorSpace = core::ptr::null_mut();
    (*pdc).dclevel.pColorSpace = core::ptr::null_mut();
    (*pdc).pClrxFormLnk = core::ptr::null_mut();

    (*pdc).hlfntCur = core::ptr::null_mut();
    (*pdc).pPFFList = core::ptr::null_mut();
    (*pdc).flSimulationFlags = 0;
    (*pdc).lEscapement = 0;
    (*pdc).prfnt = core::ptr::null_mut();
    (*pdc).dcattr.flFontMapper = 0;
    (*pdc).dcattr.flTextAlign = 0;
    (*pdc).dcattr.lTextAlign = 0;
    (*pdc).dcattr.lTextExtra = 0;
    (*pdc).dcattr.lRelAbs = 1;
    (*pdc).dcattr.lBreakExtra = 0;
    (*pdc).dcattr.cBreak = 0;
    (*pdc).dcattr.hlfntNew = (*(*(&raw const StockObjects)).as_ptr().add(SYSTEM_FONT as usize)) as _;
    (*pdc).dclevel.plfnt = LFONT_ShareLockFont((*pdc).dcattr.hlfntNew as _);

    (*pdc).hdcNext = core::ptr::null_mut();
    (*pdc).hdcPrev = core::ptr::null_mut();
    (*pdc).ipfdDevMax = 0;
    (*pdc).ulCopyCount = 0xffffffffu32;
    (*pdc).ptlDoBanding.x = 0;
    (*pdc).ptlDoBanding.y = 0;
    (*pdc).dclevel.lSaveDepth = 1;
    (*pdc).dclevel.hdcSave = core::ptr::null_mut();
    (*pdc).dcattr.iGraphicsMode = GM_COMPATIBLE as i32;
    (*pdc).dcattr.iCS_CP = 0;
    (*pdc).pSurfInfo = core::ptr::null_mut();
}
#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn DC_vCleanup(ObjectBody: PVOID) {
    let pdc = ObjectBody as PDC;

    DC_vFreeDcAttr(pdc);

    DC_vRestoreDC(pdc, 1);

    DC_vSelectSurface(pdc, core::ptr::null_mut());
    DC_vSelectFillBrush(pdc, core::ptr::null_mut());
    DC_vSelectLineBrush(pdc, core::ptr::null_mut());
    DC_vSelectPalette(pdc, core::ptr::null_mut());

    EBRUSHOBJ_vCleanup(core::ptr::addr_of_mut!((*pdc).eboFill));
    EBRUSHOBJ_vCleanup(core::ptr::addr_of_mut!((*pdc).eboLine));
    EBRUSHOBJ_vCleanup(core::ptr::addr_of_mut!((*pdc).eboText));
    EBRUSHOBJ_vCleanup(core::ptr::addr_of_mut!((*pdc).eboBackground));

    if !(*pdc).dclevel.plfnt.is_null() {
        LFONT_ShareUnlockFont((*pdc).dclevel.plfnt);
    }

    if !(*pdc).dclevel.prgnClip.is_null() {
        REGION_Delete((*pdc).dclevel.prgnClip);
    }
    if !(*pdc).dclevel.prgnMeta.is_null() {
        REGION_Delete((*pdc).dclevel.prgnMeta);
    }
    if !(*pdc).prgnVis.is_null() {
        REGION_Delete((*pdc).prgnVis);
    }
    if !(*pdc).prgnRao.is_null() {
        REGION_Delete((*pdc).prgnRao);
    }
    if !(*pdc).prgnAPI.is_null() {
        REGION_Delete((*pdc).prgnAPI);
    }

    IntEngFreeClipResources(core::ptr::addr_of_mut!((*pdc).co));

    if !(*pdc).dclevel.hPath.is_null() {
        PATH_Delete((*pdc).dclevel.hPath);
        (*pdc).dclevel.hPath = core::ptr::null_mut();
        (*pdc).dclevel.flPath = 0;
    }
    if !(*pdc).dclevel.pSurface.is_null() {
        SURFACE_ShareUnlockSurface((*pdc).dclevel.pSurface);
    }

    if !(*pdc).ppdev.is_null() {
        PDEVOBJ_vRelease((*pdc).ppdev);
    }
}

unsafe fn DC_vSetOwner(pdc: PDC, ulOwner: ULONG) {
    DC_vRestoreDC(pdc, 1);

    if (*pdc).dclevel.hPath != core::ptr::null_mut() {
        GreSetObjectOwner((*pdc).dclevel.hPath, ulOwner);
    }

    BRUSH_ShareUnlockBrush((*pdc).dclevel.pbrFill);
    BRUSH_ShareUnlockBrush((*pdc).dclevel.pbrLine);

    (*pdc).dcattr.hbrush = (*(*(&raw const StockObjects)).as_ptr().add(WHITE_BRUSH as usize)) as _;
    (*pdc).dcattr.hpen = (*(*(&raw const StockObjects)).as_ptr().add(BLACK_PEN as usize)) as _;
    (*pdc).dclevel.pbrFill = BRUSH_ShareLockBrush((*(*pdc).pdcattr).hbrush as _);
    (*pdc).dclevel.pbrLine = PEN_ShareLockPen((*(*pdc).pdcattr).hpen as _);

    (*(*pdc).pdcattr).ulDirty_ |= DIRTY_FILL | DIRTY_LINE;

    if ulOwner == GDI_OBJ_HMGR_PUBLIC || ulOwner == GDI_OBJ_HMGR_NONE {
        if (*pdc).pdcattr != core::ptr::addr_of_mut!((*pdc).dcattr) {
            DC_vFreeDcAttr(pdc);
        }
    } else if ulOwner == GDI_OBJ_HMGR_POWNED {
        if (*pdc).pdcattr == core::ptr::addr_of_mut!((*pdc).dcattr) {
            DC_bAllocDcAttr(pdc);
        }
    }

    GDIOBJ_vSetObjectOwner(core::ptr::addr_of_mut!((*pdc).BaseObject), ulOwner);
}
#[no_mangle]
pub unsafe extern "system" fn GreSetDCOwner(hdc: HDC, ulOwner: ULONG) -> BOOL {
    let pdc: PDC;

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        return 0;
    }

    DC_vSetOwner(pdc, ulOwner);

    DC_UnlockDc(pdc);
    return 1;
}

unsafe fn DC_vUpdateDC(pdc: PDC) {
    let ppdev: PPDEVOBJ = (*pdc).ppdev;

    (*pdc).dhpdev = (*ppdev).dhpdev;

    SURFACE_ShareUnlockSurface((*pdc).dclevel.pSurface);
    (*pdc).dclevel.pSurface = PDEVOBJ_pSurface(ppdev);

    PDEVOBJ_sizl((*pdc).ppdev, core::ptr::addr_of_mut!((*pdc).dclevel.sizl));

    (*pdc).flGraphicsCaps = (*ppdev).devinfo.flGraphicsCaps;
    (*pdc).flGraphicsCaps2 = (*ppdev).devinfo.flGraphicsCaps2;

    (*(*pdc).pdcattr).ulDirty_ |= 0x00010000 | 0x00000004 | 0x00000008 | 0x00000010 | 0x00000020 ;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DC_vPrepareDCsForBlit(
    pdcDest: PDC,
    rcDest: *const RECT,
    pdcSrc: PDC,
    rcSrc: *const RECT)
{

    let mut pdcFirst: PDC;
    let mut pdcSecond: PDC;
    let mut prcFirst: *const RECT;
    let mut prcSecond: *const RECT;

    if (*(*pdcDest).pdcattr).ulDirty_ & (0x00000020 | 0x00000002) != 0 {
        DC_vUpdateFillBrush(pdcDest);
    }
    if (*(*pdcDest).pdcattr).ulDirty_ & (0x00000010 | 0x00000001) != 0 {
        DC_vUpdateLineBrush(pdcDest);
    }
    if (*(*pdcDest).pdcattr).ulDirty_ & 0x00000008 != 0 {
        DC_vUpdateTextBrush(pdcDest);
    }

    if !pdcSrc.is_null() {
        if (*(*pdcDest).ppdev).hsemDevLock as usize >= (*(*pdcSrc).ppdev).hsemDevLock as usize {
            pdcFirst = pdcDest;
            prcFirst = rcDest;
            pdcSecond = pdcSrc;
            prcSecond = rcSrc;
        } else {
            pdcFirst = pdcSrc;
            prcFirst = rcSrc;
            pdcSecond = pdcDest;
            prcSecond = rcDest;
        }
    } else {
        pdcFirst = pdcDest;
        prcFirst = rcDest;
        pdcSecond = core::ptr::null_mut();
        prcSecond = core::ptr::null();
    }

    if (*pdcDest).fs & (DC_DIRTY_RAO as i32) != 0 {
        CLIPPING_UpdateGCRegion(pdcDest);
    }

    if (*pdcFirst).dctype == 0 {
        EngAcquireSemaphore((*(*pdcFirst).ppdev).hsemDevLock);

        if (*(*pdcFirst).ppdev).pSurface != (*pdcFirst).dclevel.pSurface {
            DC_vUpdateDC(pdcFirst);
        }
    }

    if (*pdcFirst).dctype == 0 {
        if prcFirst.is_null() {
            prcFirst = core::ptr::addr_of!((*pdcFirst).erclClip) as *const tagRECT;
        }

        MouseSafetyOnDrawStart(
            (*pdcFirst).ppdev as *mut core::ffi::c_void,
            (*prcFirst).left,
            (*prcFirst).top,
            (*prcFirst).right,
            (*prcFirst).bottom);
    }

    #[cfg(debug_assertions)]
    {
        (*pdcFirst).fs |= DC_PREPARED as i32;
    }

    if pdcSecond.is_null() {
        return;
    }

    if (*pdcSecond).dctype == 0 {
        EngAcquireSemaphore((*(*pdcSecond).ppdev).hsemDevLock);

        if (*(*pdcSecond).ppdev).pSurface != (*pdcSecond).dclevel.pSurface {
            DC_vUpdateDC(pdcSecond);
        }
    }

    if (*pdcSecond).dctype == 0 {
        if prcSecond.is_null() {
            prcSecond = core::ptr::addr_of!((*pdcSecond).erclClip) as *const tagRECT;
        }
        MouseSafetyOnDrawStart(
            (*pdcSecond).ppdev as *mut core::ffi::c_void,
            (*prcSecond).left,
            (*prcSecond).top,
            (*prcSecond).right,
            (*prcSecond).bottom);
    }

    #[cfg(debug_assertions)]
    {
        (*pdcSecond).fs |= DC_PREPARED as i32;
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DC_vPrepareDCsForBlit(
    pdcDest: PDC,
    rcDest: *const RECT,
    pdcSrc: PDC,
    rcSrc: *const RECT)
{

    let mut pdcFirst: PDC;
    let mut pdcSecond: PDC;
    let mut prcFirst: *const RECT;
    let mut prcSecond: *const RECT;

    if (*(*pdcDest).pdcattr).ulDirty_ & (0x00000020 | 0x00000002) != 0 {
        DC_vUpdateFillBrush(pdcDest);
    }
    if (*(*pdcDest).pdcattr).ulDirty_ & (0x00000010 | 0x00000001) != 0 {
        DC_vUpdateLineBrush(pdcDest);
    }
    if (*(*pdcDest).pdcattr).ulDirty_ & 0x00000008 != 0 {
        DC_vUpdateTextBrush(pdcDest);
    }

    if !pdcSrc.is_null() {
        if (*(*pdcDest).ppdev).hsemDevLock as usize >= (*(*pdcSrc).ppdev).hsemDevLock as usize {
            pdcFirst = pdcDest;
            prcFirst = rcDest;
            pdcSecond = pdcSrc;
            prcSecond = rcSrc;
        } else {
            pdcFirst = pdcSrc;
            prcFirst = rcSrc;
            pdcSecond = pdcDest;
            prcSecond = rcDest;
        }
    } else {
        pdcFirst = pdcDest;
        prcFirst = rcDest;
        pdcSecond = core::ptr::null_mut();
        prcSecond = core::ptr::null();
    }

    if (*pdcDest).fs & (DC_DIRTY_RAO as i32) != 0 {
        CLIPPING_UpdateGCRegion(pdcDest);
    }

    if (*pdcFirst).dctype == 0 {
        EngAcquireSemaphore((*(*pdcFirst).ppdev).hsemDevLock);

        if (*(*pdcFirst).ppdev).pSurface != (*pdcFirst).dclevel.pSurface {
            DC_vUpdateDC(pdcFirst);
        }
    }

    if (*pdcFirst).dctype == 0 {
        if prcFirst.is_null() {
            prcFirst = core::ptr::addr_of!((*pdcFirst).erclClip) as *const tagRECT;
        }

        MouseSafetyOnDrawStart(
            (*pdcFirst).ppdev as *mut core::ffi::c_void,
            (*prcFirst).left,
            (*prcFirst).top,
            (*prcFirst).right,
            (*prcFirst).bottom);
    }

    #[cfg(debug_assertions)]
    {
        (*pdcFirst).fs |= DC_PREPARED as i32;
    }

    if pdcSecond.is_null() {
        return;
    }

    if (*pdcSecond).dctype == 0 {
        EngAcquireSemaphore((*(*pdcSecond).ppdev).hsemDevLock);

        if (*(*pdcSecond).ppdev).pSurface != (*pdcSecond).dclevel.pSurface {
            DC_vUpdateDC(pdcSecond);
        }
    }

    if (*pdcSecond).dctype == 0 {
        if prcSecond.is_null() {
            prcSecond = core::ptr::addr_of!((*pdcSecond).erclClip) as *const tagRECT;
        }
        MouseSafetyOnDrawStart(
            (*pdcSecond).ppdev as *mut core::ffi::c_void,
            (*prcSecond).left,
            (*prcSecond).top,
            (*prcSecond).right,
            (*prcSecond).bottom);
    }

    #[cfg(debug_assertions)]
    {
        (*pdcSecond).fs |= DC_PREPARED as i32;
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn DC_vFinishBlit(pdc1: PDC, pdc2: PDC) {
    if (*pdc1).dctype == DCTYPE_DIRECT {
        MouseSafetyOnDrawEnd((*pdc1).ppdev as _);
        EngReleaseSemaphore((*(*pdc1).ppdev).hsemDevLock);
    }
    #[cfg(debug_assertions)]
    {
        (*pdc1).fs &= !(DC_PREPARED as i32);
    }

    if !pdc2.is_null() {
        if (*pdc2).dctype == DCTYPE_DIRECT {
            MouseSafetyOnDrawEnd((*pdc2).ppdev as _);
            EngReleaseSemaphore((*(*pdc2).ppdev).hsemDevLock);
        }
        #[cfg(debug_assertions)]
        {
            (*pdc2).fs &= !(DC_PREPARED as i32);
        }
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn DC_vFinishBlit(pdc1: PDC, pdc2: PDC) {
    if (*pdc1).dctype == DCTYPE_DIRECT {
        MouseSafetyOnDrawEnd((*pdc1).ppdev as _);
        EngReleaseSemaphore((*(*pdc1).ppdev).hsemDevLock);
    }
    #[cfg(debug_assertions)]
    {
        (*pdc1).fs &= !(DC_PREPARED as i32);
    }

    if !pdc2.is_null() {
        if (*pdc2).dctype == DCTYPE_DIRECT {
            MouseSafetyOnDrawEnd((*pdc2).ppdev as _);
            EngReleaseSemaphore((*(*pdc2).ppdev).hsemDevLock);
        }
        #[cfg(debug_assertions)]
        {
            (*pdc2).fs &= !(DC_PREPARED as i32);
        }
    }
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn GreOpenDCW(
    pustrDevice: PUNICODE_STRING,
    pdmInit: *mut DEVMODEW,
    pustrLogAddr: PUNICODE_STRING,
    iType: ULONG,
    bDisplay: BOOL,
    hspool: HANDLE,
    pDriverInfo2: *mut VOID,
    pUMdhpdev: *mut PVOID,
) -> HDC {

    let ppdev: PPDEVOBJ;
    let pdc: PDC;
    let hdc: HDC;

    ppdev = EngpGetPDEV(pustrDevice);
    if ppdev.is_null() {
        return core::ptr::null_mut();
    }

    pdc = DC_AllocDcWithHandle(GDILoObjType_LO_DC_TYPE);
    if pdc.is_null() {
        PDEVOBJ_vRelease(ppdev);
        return core::ptr::null_mut();
    }
    hdc = (*pdc).BaseObject.hHmgr as _;

    DC_vInitDc(pdc, iType, ppdev);
    if !pUMdhpdev.is_null() {
        *pUMdhpdev = (*ppdev).dhpdev as _;
    }
    DC_InitHack(pdc);

    DC_bAllocDcAttr(pdc);

    DC_UnlockDc(pdc);

    hdc
}
#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn NtGdiOpenDCW(
    mut pustrDevice: PUNICODE_STRING,
    mut pdmInit: *mut DEVMODEW,
    pustrLogAddr: PUNICODE_STRING,
    iType: ULONG,
    bDisplay: BOOL,
    hspool: HANDLE,
    pUMdhpdev: PVOID,
) -> HDC {

    let mut ustrDevice: _UNICODE_STRING = core::mem::zeroed();
    let mut awcDevice: [WCHAR; CCHDEVICENAME as usize] = [0; CCHDEVICENAME as usize];
    let mut dhpdev: PVOID = core::ptr::null_mut();
    let mut hdc: HDC;
    let mut dmSize: WORD;
    let mut dmDriverExtra: WORD;
    let mut Size: DWORD;
    let mut pdmAllocated: *mut DEVMODEW = core::ptr::null_mut();

    if !pustrDevice.is_null() {
        RtlInitEmptyUnicodeString(
            core::ptr::addr_of_mut!(ustrDevice) as *mut _,
            awcDevice.as_mut_ptr(),
            core::mem::size_of_val(&awcDevice) as USHORT,
        );

        RtlCopyUnicodeString(core::ptr::addr_of_mut!(ustrDevice) as *mut _, pustrDevice as *mut _);

        if !pdmInit.is_null() {
            dmSize = (*pdmInit).dmSize;
            dmDriverExtra = (*pdmInit).dmDriverExtra;
            Size = dmSize as DWORD + dmDriverExtra as DWORD;

            pdmAllocated = ExAllocatePoolWithTag(
                PagedPool | POOL_RAISE_IF_ALLOCATION_FAILURE,
                Size as SIZE_T,
                TAG_DC,
            ) as *mut DEVMODEW;
            RtlCopyMemory(
                pdmAllocated as *mut VOID,
                pdmInit as *const VOID,
                Size as usize,
            );
            (*pdmAllocated).dmSize = dmSize;
            (*pdmAllocated).dmDriverExtra = dmDriverExtra;
        }
    } else {
        pdmInit = core::ptr::null_mut();
    }

    if !pustrDevice.is_null() {
        let ustrDISPLAY = RTL_CONSTANT_STRING(w!("DISPLAY").as_ptr() as *const u16);
        if RtlEqualUnicodeString(core::ptr::addr_of!(ustrDevice) as *const _, core::ptr::addr_of!(ustrDISPLAY) as *const _, 1) != 0 {
            pustrDevice = core::ptr::null_mut();
        }
    }

    hdc = GreOpenDCW(
        if !pustrDevice.is_null() { core::ptr::addr_of_mut!(ustrDevice) } else { core::ptr::null_mut() },
        pdmAllocated,
        core::ptr::null_mut(),
        iType,
        bDisplay,
        hspool,
        core::ptr::null_mut(),
        if !pUMdhpdev.is_null() { core::ptr::addr_of_mut!(dhpdev) } else { core::ptr::null_mut() },
    );

    if !hdc.is_null() && !pUMdhpdev.is_null() {
        *(pUMdhpdev as *mut HANDLE) = dhpdev as HANDLE;
    }

    if !pdmAllocated.is_null() {
        ExFreePoolWithTag(pdmAllocated as PVOID, TAG_DC);
    }

    hdc
}
#[no_mangle]
pub unsafe extern "system" fn GreCreateCompatibleDC(hdc: HDC, bAltDc: i32) -> HDC {

    let eDcObjType: u32;
    let hdcNew: HDC;
    let ppdev: PPDEVOBJ;
    let pdc: PDC;
    let pdcNew: PDC;

    if !hdc.is_null() {
        pdc = DC_LockDc(hdc);
        if pdc.is_null() {
            return core::ptr::null_mut();
        }

        ppdev = (*pdc).ppdev;
        PDEVOBJ_vReference(ppdev);

        DC_UnlockDc(pdc);
    } else {
        ppdev = EngpGetPDEV(core::ptr::null_mut());
    }

    if ppdev.is_null() {
        return core::ptr::null_mut();
    }

    eDcObjType = if bAltDc != 0 { GDILoObjType_LO_ALTDC_TYPE } else { GDILoObjType_LO_DC_TYPE };
    pdcNew = DC_AllocDcWithHandle(eDcObjType);
    if pdcNew.is_null() {
        PDEVOBJ_vRelease(ppdev);
        return core::ptr::null_mut();
    }
    hdcNew = (*pdcNew).BaseObject.hHmgr as _;

    DC_vInitDc(pdcNew, if bAltDc != 0 { DCTYPE_INFO } else { DCTYPE_MEMORY }, ppdev);
    DC_InitHack(pdcNew);

    DC_bAllocDcAttr(pdcNew);

    DC_UnlockDc(pdcNew);

    hdcNew
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCreateCompatibleDC(hdc: HDC) -> HDC {
    GreCreateCompatibleDC(hdc, 0)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntGdiDeleteDC(hDC: HDC, Force: BOOL) -> BOOL {
    let DCToDelete: PDC = DC_LockDc(hDC);

    if DCToDelete.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if Force == 0 {
        if ((*DCToDelete).fs & (DC_PERMANANT as i32)) != 0 {
            DC_UnlockDc(DCToDelete);
            if UserReleaseDC(core::ptr::null_mut(), hDC, 0) != 0 {
                return 1;
            } else {
                return IntGdiDeleteDC(hDC, 1);
            }
        }
    }

    DC_UnlockDc(DCToDelete);

    if GreIsHandleValid(hDC as _) != 0 {
        if GreDeleteObject(hDC as _) == 0 {
            return 0;
        }
    } else {
        return 0;
    }

    1
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntGdiDeleteDC(hDC: HDC, Force: BOOL) -> BOOL {
    let DCToDelete: PDC = DC_LockDc(hDC);

    if DCToDelete.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if Force == 0 {
        if ((*DCToDelete).fs & (DC_PERMANANT as i32)) != 0 {
            DC_UnlockDc(DCToDelete);
            if UserReleaseDC(core::ptr::null_mut(), hDC, 0) != 0 {
                return 1;
            } else {
                return IntGdiDeleteDC(hDC, 1);
            }
        }
    }

    DC_UnlockDc(DCToDelete);

    if GreIsHandleValid(hDC as _) != 0 {
        if GreDeleteObject(hDC as _) == 0 {
            return 0;
        }
    } else {
        return 0;
    }

    1
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDeleteObjectApp(hobj: HANDLE) -> BOOL {

    if GDI_HANDLE_IS_STOCKOBJ(hobj) != 0 {
        return 1;
    }

    if GreGetObjectOwner(hobj) != GDI_OBJ_HMGR_POWNED {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if GDI_HANDLE_GET_TYPE(hobj) != GDI_OBJECT_TYPE_DC {
        return GreDeleteObject(hobj);
    }

    {
        IntGdiDeleteDC(hobj as _, 0)
    }
}

#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn MakeInfoDC(pdc: PDC, bSet: BOOL) -> BOOL {
    let mut pSurface: PSURFACE;
    let mut sizl: SIZEL = core::mem::zeroed();

    if ((*pdc).fs & (DC_DISPLAY as i32)) != 0 {
        return 0;
    }
    
    if bSet != 0 {
        if ((*pdc).fs & (DC_TEMPINFODC as i32)) != 0 || (*pdc).dctype == DCTYPE_DIRECT {
            return 0;
        }

        pSurface = (*pdc).dclevel.pSurface;
        (*pdc).fs |= (DC_TEMPINFODC as i32);
        (*pdc).pSurfInfo = pSurface as _;
        (*pdc).dctype = DCTYPE_INFO;
        (*pdc).dclevel.pSurface = core::ptr::null_mut();

        PDEVOBJ_sizl((*pdc).ppdev, &mut sizl);

        if sizl.cx == (*pdc).dclevel.sizl.cx && sizl.cy == (*pdc).dclevel.sizl.cy {
            return 1;
        }

        (*pdc).dclevel.sizl.cx = sizl.cx;
        (*pdc).dclevel.sizl.cy = sizl.cy;
    } else {
        if ((*pdc).fs & (DC_TEMPINFODC as i32)) == 0 || (*pdc).dctype != DCTYPE_INFO {
            return 0;
        }

        pSurface = (*pdc).pSurfInfo as _;
        (*pdc).fs &= !(DC_TEMPINFODC as i32);
        (*pdc).dclevel.pSurface = pSurface;
        (*pdc).dctype = DCTYPE_DIRECT;
        (*pdc).pSurfInfo = core::ptr::null_mut();

        if pSurface.is_null() ||
           ((*pSurface).SurfObj.sizlBitmap.cx == (*pdc).dclevel.sizl.cx &&
            (*pSurface).SurfObj.sizlBitmap.cy == (*pdc).dclevel.sizl.cy) {
            return 1;
        }

        (*pdc).dclevel.sizl.cx = (*pSurface).SurfObj.sizlBitmap.cx;
        (*pdc).dclevel.sizl.cy = (*pSurface).SurfObj.sizlBitmap.cy;
    }
    
    IntSetDefaultRegion(pdc)
}

#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn MakeInfoDC(pdc: PDC, bSet: BOOL) -> BOOL {
    let mut pSurface: PSURFACE;
    let mut sizl: SIZEL = core::mem::zeroed();

    if ((*pdc).fs & (DC_DISPLAY as i32)) != 0 {
        return 0;
    }
    
    if bSet != 0 {
        if ((*pdc).fs & (DC_TEMPINFODC as i32)) != 0 || (*pdc).dctype == DCTYPE_DIRECT {
            return 0;
        }

        pSurface = (*pdc).dclevel.pSurface;
        (*pdc).fs |= DC_TEMPINFODC as i32 ;
        (*pdc).pSurfInfo = pSurface as _;
        (*pdc).dctype = DCTYPE_INFO;
        (*pdc).dclevel.pSurface = core::ptr::null_mut();

        PDEVOBJ_sizl((*pdc).ppdev, &mut sizl);

        if sizl.cx == (*pdc).dclevel.sizl.cx && sizl.cy == (*pdc).dclevel.sizl.cy {
            return 1;
        }

        (*pdc).dclevel.sizl.cx = sizl.cx;
        (*pdc).dclevel.sizl.cy = sizl.cy;
    } else {
        if ((*pdc).fs & (DC_TEMPINFODC as i32)) == 0 || (*pdc).dctype != DCTYPE_INFO {
            return 0;
        }

        pSurface = (*pdc).pSurfInfo as _;
        (*pdc).fs &= !(DC_TEMPINFODC as i32);
        (*pdc).dclevel.pSurface = pSurface;
        (*pdc).dctype = DCTYPE_DIRECT;
        (*pdc).pSurfInfo = core::ptr::null_mut();

        if pSurface.is_null() ||
           ((*pSurface).SurfObj.sizlBitmap.cx == (*pdc).dclevel.sizl.cx &&
            (*pSurface).SurfObj.sizlBitmap.cy == (*pdc).dclevel.sizl.cy) {
            return 1;
        }

        (*pdc).dclevel.sizl.cx = (*pSurface).SurfObj.sizlBitmap.cx;
        (*pdc).dclevel.sizl.cy = (*pSurface).SurfObj.sizlBitmap.cy;
    }
    
    IntSetDefaultRegion(pdc)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiMakeInfoDC(
    hdc: HDC,
    bSet: BOOL,
) -> BOOL {
    let pdc: PDC = DC_LockDc(hdc);
    if !pdc.is_null() {
        let Ret = MakeInfoDC(pdc, bSet);
        DC_UnlockDc(pdc);
        return Ret;
    }
    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "fastcall" fn IntGdiCreateDC(
    Driver: PUNICODE_STRING,
    mut pustrDevice: PUNICODE_STRING,
    pUMdhpdev: PVOID,
    mut pdmInit: *const DEVMODEW,
    CreateAsIC: BOOL,
) -> HDC {
    let hdc: HDC;

    hdc = GreOpenDCW(
        pustrDevice,
        pdmInit as PDEVMODEW,
        core::ptr::null_mut(),
        if CreateAsIC != 0 {
            DCTYPE_INFO
        } else {
            DCTYPE_DIRECT
        },
        1,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        pUMdhpdev as *mut *mut core::ffi::c_void,
    );

    hdc
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn IntGdiCreateDC(
    Driver: PUNICODE_STRING,
    mut pustrDevice: PUNICODE_STRING,
    pUMdhpdev: PVOID,
    mut pdmInit: *const DEVMODEW,
    CreateAsIC: BOOL,
) -> HDC {
    let hdc: HDC;

    hdc = GreOpenDCW(
        pustrDevice,
        pdmInit as PDEVMODEW,
        core::ptr::null_mut(),
        if CreateAsIC != 0 {
            DCTYPE_INFO
        } else {
            DCTYPE_DIRECT
        },
        1,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        pUMdhpdev as *mut *mut core::ffi::c_void,
    );

    hdc
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiCreateDisplayDC(hDev: HDEV, DcType: ULONG, EmptyDC: BOOL) -> HDC {
    let hDC: HDC;

    if DcType == DCTYPE_MEMORY {
        hDC = NtGdiCreateCompatibleDC(core::ptr::null_mut());
    } else {
        hDC = IntGdiCreateDC(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            if DcType == DCTYPE_INFO { 1 } else { 0 },
        );
    }

    hDC
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiCreateDisplayDC(hDev: HDEV, DcType: ULONG, EmptyDC: BOOL) -> HDC {
    let hDC: HDC;

    if DcType == DCTYPE_MEMORY {
        hDC = NtGdiCreateCompatibleDC(core::ptr::null_mut());
    } else {
        hDC = IntGdiCreateDC(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            if DcType == DCTYPE_INFO { 1 } else { 0 },
        );
    }

    hDC
}
