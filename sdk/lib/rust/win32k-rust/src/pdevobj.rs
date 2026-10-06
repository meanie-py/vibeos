use win32k_sys::win32k::*;
use ntkrnl_sys::base::{NT_SUCCESS as DdkNT_SUCCESS};
use ntkrnl_sys::ddk::{
    RtlEqualUnicodeString as DdkRtlEqualUnicodeString,
    RtlInitUnicodeString as DdkRtlInitUnicodeString,
    _UNICODE_STRING as DdkUNICODE_STRING,
};
use ntkrnl_sys::interlocked::{InterlockedDecrement as DdkInterlockedDecrement};

// NTSTATUS constants — ntkrnl-sys uses i32, win32k-sys NTSTATUS = LONG = i32.
#[allow(non_upper_case_globals)]
const STATUS_SUCCESS: NTSTATUS = 0;
#[allow(non_upper_case_globals)]
const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000009Au32 as NTSTATUS;
#[allow(non_upper_case_globals)]
const ERROR_INVALID_HANDLE: ULONG = 6;

// ExAllocatePoolWithTag / ExFreePoolWithTag are NTAPI exports
// (stdcall on i386, cdecl elsewhere). `extern "system"` resolves correctly.
// Mirrors mdevobj.rs / pandisp.rs.
extern "system" {
    fn ExAllocatePoolWithTag(PoolType: POOL_TYPE, NumberOfBytes: SIZE_T, Tag: ULONG) -> PVOID;
    fn ExFreePoolWithTag(P: PVOID, Tag: ULONG);
}

// Pool tags from win32ss/user/ntuser/tags.h
//   GDITAG_PDEV    'veDG' = 0x76654447 → LE bytes "GDev"
//   GDITAG_DEVMODE 'vedG' = 0x76656447 → LE bytes "Gdev"
//   GDITAG_GDEVICE 'vdgG' = 0x76646747 → LE bytes "Ggdv"
#[allow(non_upper_case_globals)]
const GDITAG_PDEV: u32 = u32::from_le_bytes(*b"GDev");
#[allow(non_upper_case_globals)]
const GDITAG_DEVMODE: u32 = u32::from_le_bytes(*b"Gdev");
#[allow(non_upper_case_globals)]
const GDITAG_GDEVICE: u32 = u32::from_le_bytes(*b"Ggdv");

/// InterlockedDecrement adapter — bridges win32k-sys `LONG` to ntkrnl-sys `LONG`.
/// Returns the new value (matches Windows semantics).
#[inline(always)]
unsafe fn InterlockedDecrement(p: *mut LONG) -> LONG {
    DdkInterlockedDecrement(p as *mut ntkrnl_sys::base::LONG) as LONG
}

/// RtlCopyMemory adapter — memcpy. Takes win32k-sys PVOID/SIZE_T.
#[inline(always)]
unsafe fn RtlCopyMemory(dst: PVOID, src: *const core::ffi::c_void, len: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len);
}

/// NT_SUCCESS adapter — bridges win32k-sys NTSTATUS to ntkrnl-sys NTSTATUS.
#[inline(always)]
fn NT_SUCCESS(status: NTSTATUS) -> bool {
    DdkNT_SUCCESS(status as ntkrnl_sys::base::NTSTATUS)
}

/// RtlInitUnicodeString adapter — bridges win32k-sys UNICODE_STRING to ntkrnl-sys.
#[inline(always)]
unsafe fn RtlInitUnicodeString(s: *mut _UNICODE_STRING, src: *const u16) {
    DdkRtlInitUnicodeString(s as *mut DdkUNICODE_STRING, src)
}

/// RtlEqualUnicodeString adapter — bridges win32k-sys UNICODE_STRING to ntkrnl-sys.
#[inline(always)]
unsafe fn RtlEqualUnicodeString(
    s1: *const _UNICODE_STRING,
    s2: *const _UNICODE_STRING,
    ci: u8,
) -> u8 {
    DdkRtlEqualUnicodeString(
        s1 as *const DdkUNICODE_STRING,
        s2 as *const DdkUNICODE_STRING,
        ci,
    )
}

// LDEVOBJ_bBuildDevmodeList lives in ldevobj.c (or ldevobj.rs when enabled).
// C decl in ldevobj.h is plain `BOOL LDEVOBJ_bBuildDevmodeList(...)` with no
// calling convention attribute — that's cdecl (i386: `_LDEVOBJ_bBuildDevmodeList`,
// NOT `@4`-decorated). `extern "C"` matches.
// gPanDispDrvFn / gPanDispDrvCount live in pandisp.rs as `#[no_mangle] pub static`.
extern "C" {
    fn LDEVOBJ_bBuildDevmodeList(pGraphicsDevice: PGRAPHICS_DEVICE) -> u8;
}

extern "C" {
    static gPanDispDrvFn: [DRVFN; 13];
    static gPanDispDrvCount: ULONG;
}

// MmSystemRangeStart is a cross-DLL data export (ntoskrnl.spec:821).
// Per-arch ABI (same pattern as engevent.rs's ExEventObjectType — see
// feedback_win32k_raw_dylib_freeldr_oom hazard 2):
// - i386: needs `raw-dylib +verbatim` for the `__imp_` thunk under
//   --disable-auto-import; plain extern fails link.
// - amd64: auto-resolves via regular implib; adding raw-dylib here risks
//   BUGCHECK 0x6B regressions (cgu=256 dup-descriptor interaction).
// - arm64: USED to auto-resolve but the wave-2-after-device.rs cgu shift
//   moved the reference into a partition that no longer resolves (link
//   error on `MmSystemRangeStart`). Gate raw-dylib in for arm64 too.
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static MmSystemRangeStart: *mut core::ffi::c_void;
}

#[cfg(target_arch = "x86_64")]
extern "C" {
    static MmSystemRangeStart: *mut core::ffi::c_void;
}

// MultiEnableDriver is the multidisp.rs DrvEnableDriver entry point. Declared
// here as extern (real export); lld resolves to the multidisp.rs definition.
extern "system" {
    fn MultiEnableDriver(iEngineVersion: ULONG, cj: ULONG, pded: PDRVENABLEDATA) -> BOOL;
}

// SafeProbeForWrite — win32ss/include/safe_user_io.h shim (NTAPI).
extern "system" {
    fn SafeProbeForWrite(Address: PVOID, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
}

// SetLastNtError lives in error.rs as FASTCALL on i386, cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
}

// EngpFindGraphicsDevice is NTAPI = stdcall on i386, cdecl elsewhere.
extern "system" {
    fn EngpFindGraphicsDevice(pustrDeviceName: PUNICODE_STRING, iDevNum: u32) -> PGRAPHICS_DEVICE;
}

// LDEVOBJ_pLoadDriver is APIENTRY (= stdcall on i386, cdecl elsewhere) per ldevobj.h:56.
// LDEVOBJ_pLoadInternal is plain cdecl per ldevobj.h:50.
extern "system" {
    fn LDEVOBJ_pLoadDriver(pszDriverName: *mut u16, ldevtype: u32) -> PLDEVOBJ;
}
extern "C" {
    fn LDEVOBJ_pLoadInternal(pfnEnableDriver: *mut core::ffi::c_void, ldevtype: u32) -> PLDEVOBJ;
    fn LDEVOBJ_bProbeAndCaptureDevmode(pGraphicsDevice: PGRAPHICS_DEVICE, RequestedMode: PDEVMODEW, pSelectedMode: *mut PDEVMODEW, bSearchClosestMode: BOOL,) -> BOOL;
    fn MDEVOBJ_Create(pustrDeviceName: PUNICODE_STRING, pdm: PDEVMODEW) -> PMDEVOBJ;
    fn MDEVOBJ_bDisable(pmdev: PMDEVOBJ) -> BOOL;
    fn MDEVOBJ_vEnable(pmdev: PMDEVOBJ);
    fn MDEVOBJ_vDestroy(pmdev: PMDEVOBJ);
}

// C macros — silenced via macro_rules! (variadic no-op).
macro_rules! NT_ASSERT { ($($t:tt)*) => { () }; }
// RtlEqualMemory is `#define RtlEqualMemory(a,b,n) (!memcmp(a,b,n))` (winnt.h).
#[inline(always)]
unsafe fn RtlEqualMemory(s1: *const core::ffi::c_void, s2: *const core::ffi::c_void, n: usize) -> u32 {
    if core::slice::from_raw_parts(s1 as *const u8, n)
        == core::slice::from_raw_parts(s2 as *const u8, n) { 1 } else { 0 }
}

// `mdev_get` accessor for the flexible-array `dev` field on `_MDEVOBJ`.
// Mirrors mdevobj.rs's accessor (cannot share since it's pub(crate) there).
#[inline(always)]
unsafe fn mdev_get(pmdev: PMDEVOBJ, i: usize) -> *mut MDEVDISPLAY {
    (*pmdev).dev.as_mut_ptr().add(i)
}
// drop_gdiobj_macro_externs: dropped 3 bogus extern decl(s) for GDI-object macro(s): SURFACE_ShareLockSurface, SURFACE_ShareUnlockSurface, PALETTE_ShareUnlockPalette — these are macros in gdi/eng/<type>.h or gdi/ntgdi/<type>.h, not real fns. Inline-expand via GDIOBJ_* primitives per win32k_translation_prompt.md §6.
// ---- file-scope preamble (translated) ----
static mut gppdevList: PPDEVOBJ = core::ptr::null_mut();
static mut ghsemPDEV: HSEMAPHORE = core::ptr::null_mut();

#[inline(always)]

#[no_mangle]
pub unsafe extern "system" fn InitPDEVImpl() -> NTSTATUS {
    ghsemPDEV = EngCreateSemaphore();
    if ghsemPDEV.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    STATUS_SUCCESS
}

pub unsafe extern "system" fn DbgLookupDHPDEV(dhpdev: DHPDEV) -> PPDEVOBJ {
    let mut ppdev: PPDEVOBJ;

    EngAcquireSemaphoreShared(ghsemPDEV);

    ppdev = gppdevList;
    while !ppdev.is_null() {
        if (*ppdev).dhpdev == dhpdev {
            break;
        }
        ppdev = (*ppdev).ppdevNext;
    }

    EngReleaseSemaphore(ghsemPDEV);

    ppdev
}

unsafe fn PDEVOBJ_AllocPDEV() -> PPDEVOBJ {
    let ppdev: PPDEVOBJ = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<PDEVOBJ>() as SIZE_T,
        GDITAG_PDEV,
    ) as PPDEVOBJ;
    if ppdev.is_null() {
        return core::ptr::null_mut();
    }

    core::ptr::write_bytes(ppdev, 0, 1);

    (*ppdev).hsemDevLock = EngCreateSemaphore();
    if (*ppdev).hsemDevLock.is_null() {
        ExFreePoolWithTag(ppdev as *mut core::ffi::c_void, GDITAG_PDEV);
        return core::ptr::null_mut();
    }

    (*ppdev).pEDDgpl = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<EDD_DIRECTDRAW_GLOBAL>() as SIZE_T,
        GDITAG_PDEV,
    ) as *mut EDD_DIRECTDRAW_GLOBAL;
    if !(*ppdev).pEDDgpl.is_null() {
        core::ptr::write_bytes((*ppdev).pEDDgpl, 0, 1);
    }

    (*ppdev).cPdevRefs = 1;

    ppdev
}

unsafe fn PDEVOBJ_vDeletePDEV(ppdev: PPDEVOBJ) {
    EngDeleteSemaphore((*ppdev).hsemDevLock);
    if !(*ppdev).pdmwDev.is_null() {
        ExFreePoolWithTag((*ppdev).pdmwDev as *mut core::ffi::c_void, GDITAG_DEVMODE);
    }
    if !(*ppdev).pEDDgpl.is_null() {
        ExFreePoolWithTag((*ppdev).pEDDgpl as *mut core::ffi::c_void, GDITAG_PDEV);
    }
    ExFreePoolWithTag(ppdev as *mut core::ffi::c_void, GDITAG_PDEV);
}
#[no_mangle]
pub unsafe extern "system" fn PDEVOBJ_vRelease(
    ppdev: PPDEVOBJ) {

    EngAcquireSemaphore(ghsemPDEV);

    InterlockedDecrement(&mut (*ppdev).cPdevRefs);

    if (*ppdev).cPdevRefs == 0 {
        if !(*ppdev).pSurface.is_null() {
            // SURFACE_ShareUnlockSurface(pBMObj) macro from gdi/eng/surface.h:102
            //   => GDIOBJ_vDereferenceObject((POBJ)pBMObj)
            GDIOBJ_vDereferenceObject((*ppdev).pSurface as POBJ);
            TRACE!(b"DrvDisableSurface(dhpdev %p)\n\0".as_ptr() as *const i8, (*ppdev).dhpdev);
            ((*ppdev).__bindgen_anon_1.pfn.DisableSurface.unwrap())((*ppdev).dhpdev);
        }

        if !(*ppdev).ppalSurf.is_null() {
            // PALETTE_ShareUnlockPalette(ppal) macro from gdi/ntgdi/palette.h:59
            //   => GDIOBJ_vDereferenceObject(&ppal->BaseObject)
            GDIOBJ_vDereferenceObject(core::ptr::addr_of_mut!((*(*ppdev).ppalSurf).BaseObject));
        }

        if !(*ppdev).dhpdev.is_null() {
            TRACE!(b"DrvDisablePDEV(dhpdev %p)\n\0".as_ptr() as *const i8, (*ppdev).dhpdev);
            ((*ppdev).__bindgen_anon_1.pfn.DisablePDEV.unwrap())((*ppdev).dhpdev);
        }

        if ppdev == gppdevList {
            gppdevList = (*ppdev).ppdevNext;
        } else if !gppdevList.is_null() {
            let mut ppdevCurrent = gppdevList;
            let mut found = 0u8;
            while found == 0 && !(*ppdevCurrent).ppdevNext.is_null() {
                if (*ppdevCurrent).ppdevNext == ppdev {
                    found = 1u8;
                } else {
                    ppdevCurrent = (*ppdevCurrent).ppdevNext;
                }
            }
            if found != 0 {
                (*ppdevCurrent).ppdevNext = (*ppdev).ppdevNext;
            }
        }

        EngUnloadImage((*ppdev).pldev as HANDLE);

        PDEVOBJ_vDeletePDEV(ppdev);
    }

    EngReleaseSemaphore(ghsemPDEV);
}

unsafe fn PDEVOBJ_bEnablePDEV(
    ppdev: PPDEVOBJ,
    pdevmode: PDEVMODEW,
    pwszLogAddress: PWSTR,
) -> u8 {
    let pfnEnablePDEV: PFN_DrvEnablePDEV;
    let mut i: u32;

    pfnEnablePDEV = (*ppdev).__bindgen_anon_1.pfn.EnablePDEV;

    TRACE!(
        b"DrvEnablePDEV(pdevmode %p (%dx%dx%d %d Hz) hdev %p (%S))\n\0".as_ptr() as PCHAR,
        pdevmode,
        if !(*ppdev).pGraphicsDevice.is_null() { (*pdevmode).dmPelsWidth } else { 0 },
        if !(*ppdev).pGraphicsDevice.is_null() { (*pdevmode).dmPelsHeight } else { 0 },
        if !(*ppdev).pGraphicsDevice.is_null() { (*pdevmode).dmBitsPerPel } else { 0 },
        if !(*ppdev).pGraphicsDevice.is_null() { (*pdevmode).dmDisplayFrequency } else { 0 },
        ppdev,
        if !(*ppdev).pGraphicsDevice.is_null() { (*(*ppdev).pGraphicsDevice).szNtDeviceName.as_ptr() as *const u16 } else { [0u16].as_ptr() },
    );

    (*ppdev).dhpdev = pfnEnablePDEV.unwrap()(
        pdevmode,
        pwszLogAddress,
        HS_DDI_MAX,
        (*ppdev).ahsurf.as_mut_ptr(),
        core::mem::size_of::<GDIINFO>() as u32,
        core::ptr::addr_of_mut!((*ppdev).gdiinfo) as *mut u32,
        core::mem::size_of::<DEVINFO>() as u32,
        core::ptr::addr_of_mut!((*ppdev).devinfo),
        ppdev as HDEV,
        if !(*ppdev).pGraphicsDevice.is_null() { (*(*ppdev).pGraphicsDevice).pwszDescription } else { core::ptr::null_mut() },
        if !(*ppdev).pGraphicsDevice.is_null() { (*(*ppdev).pGraphicsDevice).DeviceObject as HANDLE } else { core::ptr::null_mut() },
    );

    TRACE!(
        b"DrvEnablePDEV(pdevmode %p hdev %p) => dhpdev %p\n\0".as_ptr() as PCHAR,
        pdevmode,
        ppdev,
        (*ppdev).dhpdev,
    );

    if (*ppdev).dhpdev.is_null() {
        ERR!("Failed to enable PDEV\n");
        return 0u8;
    }

    if (*ppdev).gdiinfo.ulLogPixelsX == 0 {
        (*ppdev).gdiinfo.ulLogPixelsX = 96;
    }

    if (*ppdev).gdiinfo.ulLogPixelsY == 0 {
        (*ppdev).gdiinfo.ulLogPixelsY = 96;
    }

    (*ppdev).gdiinfo.flRaster = RC_OP_DX_OUTPUT | RC_GDI20_OUTPUT | RC_BIGFONT;
    if ((*ppdev).gdiinfo.ulTechnology != DT_PLOTTER) && ((*ppdev).gdiinfo.ulTechnology != DT_CHARSTREAM) {
        (*ppdev).gdiinfo.flRaster |= RC_STRETCHDIB | RC_STRETCHBLT | RC_DIBTODEV | RC_DI_BITMAP | RC_BITMAP64 | RC_BITBLT;
    }
    if (*ppdev).gdiinfo.ulTechnology == DT_RASDISPLAY {
        (*ppdev).gdiinfo.flRaster |= RC_FLOODFILL;
    }
    if ((*ppdev).devinfo.flGraphicsCaps & GCAPS_PALMANAGED) != 0 {
        (*ppdev).gdiinfo.flRaster |= RC_PALETTE;
    }

    // PALETTE_ShareLockPalette(hpal) macro from gdi/ntgdi/palette.h:57
    //   => (PPALETTE)GDIOBJ_ShareLockObj((HGDIOBJ)hpal, GDI_OBJECT_TYPE_PALETTE)
    (*ppdev).ppalSurf = GDIOBJ_ShareLockObj(
        (*ppdev).devinfo.hpalDefault as HGDIOBJ,
        GDI_OBJECT_TYPE_PALETTE,
    ) as PPALETTE;

    i = 0;
    while i < HS_DDI_MAX {
        if (*ppdev).ahsurf[i as usize].is_null() {
            (*ppdev).ahsurf[i as usize] = gahsurfHatch[i as usize];
        }
        i += 1;
    }

    TRACE!(
        b"PDEVOBJ_bEnablePDEV - dhpdev = %p\n\0".as_ptr() as PCHAR,
        (*ppdev).dhpdev,
    );

    1u8
}

pub unsafe extern "system" fn PDEVOBJ_vCompletePDEV(
    ppdev: PPDEVOBJ)
{
    TRACE!(b"DrvCompletePDEV(dhpdev %p hdev %p)\n\0".as_ptr() as PCHAR, (*ppdev).dhpdev, ppdev);
    ((*ppdev).__bindgen_anon_1.pfn.CompletePDEV.unwrap())((*ppdev).dhpdev, ppdev as HDEV);
}

unsafe fn PDEVOBJ_vFilterDriverHooks(
    ppdev: PPDEVOBJ)
{
    let pldev: PLDEVOBJ = (*ppdev).pldev;
    let dwAccelerationLevel: u32 = (*ppdev).dwAccelerationLevel;

    if (*pldev).pGdiDriverInfo.is_null() {
        return;
    }
    if (*pldev).ldevtype != LDEV_DEVICE_DISPLAY {
        return;
    }

    if dwAccelerationLevel >= 1 {
        (*ppdev).__bindgen_anon_1.apfn[INDEX_DrvSetPointerShape as usize] = core::ptr::null_mut();
        (*ppdev).__bindgen_anon_1.apfn[INDEX_DrvCreateDeviceBitmap as usize] = core::ptr::null_mut();
    }

    if dwAccelerationLevel >= 2 {
        (*(*ppdev).pSurface).flags &= !(HOOK_STRETCHBLT |
                                        HOOK_FILLPATH |
                                        HOOK_GRADIENTFILL |
                                        HOOK_LINETO |
                                        HOOK_ALPHABLEND |
                                        HOOK_TRANSPARENTBLT);
    }

    if dwAccelerationLevel >= 3 {
    }

    if dwAccelerationLevel >= 4 {
        (*(*ppdev).pSurface).flags &= !HOOK_FLAGS |
                                       HOOK_BITBLT |
                                       HOOK_COPYBITS |
                                       HOOK_TEXTOUT |
                                       HOOK_STROKEPATH |
                                       HOOK_SYNCHRONIZE;
    }

    if dwAccelerationLevel >= 5 {
    }
}
#[no_mangle]
pub unsafe extern "system" fn PDEVOBJ_pSurface(
    ppdev: PPDEVOBJ,
) -> PSURFACE {

    let mut hsurf: HSURF;

    if (*ppdev).pSurface == core::ptr::null_mut() {
        hsurf = ((*ppdev).__bindgen_anon_1.pfn.EnableSurface.unwrap())((*ppdev).dhpdev);
        if hsurf == core::ptr::null_mut() {
            return core::ptr::null_mut();
        }

        // SURFACE_ShareLockSurface(hBMObj) macro from gdi/eng/surface.h:91
        //   => (PSURFACE)GDIOBJ_ShareLockObj((HGDIOBJ)hBMObj, GDI_OBJECT_TYPE_BITMAP)
        (*ppdev).pSurface = GDIOBJ_ShareLockObj(
            hsurf as HGDIOBJ,
            GDI_OBJECT_TYPE_BITMAP,
        ) as PSURFACE;
        NT_ASSERT!(((*ppdev).pSurface != core::ptr::null_mut()) as u8);
    }

    GDIOBJ_vReferenceObjectByPointer(core::ptr::addr_of_mut!((*(*ppdev).pSurface).BaseObject));

    (*ppdev).pSurface
}

// gpDxFuncs is declared in win32k-sys as `[DRVFN; 0]` (a real zero-length
// array, not bindgen `__IncompleteArrayField`). Direct indexing returns
// out-of-bounds element; use pointer arithmetic to read element i.
#[inline(always)]
unsafe fn dx_pfn(idx: u32) -> PFN {
    (*core::ptr::addr_of!(gpDxFuncs).cast::<DRVFN>().add(idx as usize)).pfn
}

#[no_mangle]
pub unsafe extern "system" fn PDEVOBJ_bEnableDirectDraw(
    ppdev: PPDEVOBJ
) -> BOOL {
    // PFN = Option<unsafe extern "stdcall" fn() -> LONG_PTR>; the slot can be None
    // if dxg.sys hasn't loaded (e.g. ARM64). transmute to the typed signature.
    let pfnDdEnableDirectDraw: PGD_DXDDENABLEDIRECTDRAW =
        core::mem::transmute(dx_pfn(DXG_INDEX_DxDdEnableDirectDraw));
    let Success: BOOL;

    if pfnDdEnableDirectDraw.is_none() {
        return 1;
    }

    TRACE!(b"DxDdEnableDirectDraw(ppdev %p)\n\0".as_ptr() as PCHAR, ppdev);
    Success = pfnDdEnableDirectDraw.unwrap()(ppdev as PVOID, 1u8) as BOOL;
    TRACE!(b"DxDdEnableDirectDraw(ppdev %p) => %d\n\0".as_ptr() as PCHAR, ppdev, Success);

    return Success;
}

pub unsafe fn PDEVOBJ_vResumeDirectDraw(ppdev: PPDEVOBJ) {
    let pfnDdResumeDirectDraw: PGD_DXDDRESUMEDIRECTDRAW =
        core::mem::transmute(dx_pfn(DXG_INDEX_DxDdResumeDirectDraw));

    pfnDdResumeDirectDraw.unwrap()(ppdev as HDEV, 0);
}

pub unsafe fn PDEVOBJ_vSuspendDirectDraw(
    ppdev: PPDEVOBJ)
{
    let pfnDdSuspendDirectDraw: PGD_DXDDSUSPENDDIRECTDRAW =
        core::mem::transmute(dx_pfn(DXG_INDEX_DxDdSuspendDirectDraw));

    TRACE!(b"DxDdSuspendDirectDraw(ppdev %p)\n\0".as_ptr() as PCHAR, ppdev);
    pfnDdSuspendDirectDraw.unwrap()(ppdev as HDEV, 0);
}

unsafe fn PDEVOBJ_vSwitchDirectDraw(
    ppdev: PPDEVOBJ,
    ppdev2: PPDEVOBJ,
) {
    let pfnDdDynamicModeChange: PGD_DXDDDYNAMICMODECHANGE =
        core::mem::transmute(dx_pfn(DXG_INDEX_DxDdDynamicModeChange));

    pfnDdDynamicModeChange.unwrap()(ppdev as HDEV, ppdev2 as HDEV, 0);
}

// OWNER: real export, declared `extern "C"` (no NTAPI in pdevobj.h:215-217)
// in win32k-sys bindgen for all 3 arches (PDEVOBJ_vEnableDisplay).
#[no_mangle]
pub unsafe extern "C" fn PDEVOBJ_vEnableDisplay(ppdev: PPDEVOBJ) {
    let mut assertVal: BOOL;

    if ((*ppdev).flFlags & PDEV_DISABLED) == 0 {
        return;
    }

    loop {
        assertVal = ((*ppdev).__bindgen_anon_1.pfn.AssertMode.unwrap())((*ppdev).dhpdev, 1);
        if assertVal != 0 {
            break;
        }
    }

    (*ppdev).flFlags &= !PDEV_DISABLED;
}

// OWNER: real export, declared `extern "C"` (no NTAPI in pdevobj.h:219-221)
// in win32k-sys bindgen for all 3 arches (PDEVOBJ_bDisableDisplay).
#[no_mangle]
pub unsafe extern "C" fn PDEVOBJ_bDisableDisplay(
    ppdev: PPDEVOBJ) -> BOOL
{
    let mut assertVal: BOOL;

    if ((*ppdev).flFlags & PDEV_DISABLED) != 0 {
        return 1;
    }

    PDEVOBJ_vSuspendDirectDraw(ppdev);

    TRACE!(b"DrvAssertMode(dhpdev %p, FALSE)\n\0".as_ptr() as PCHAR, (*ppdev).dhpdev);
    assertVal = ((*ppdev).__bindgen_anon_1.pfn.AssertMode.unwrap())((*ppdev).dhpdev, 0);
    TRACE!(b"DrvAssertMode(dhpdev %p, FALSE) => %d\n\0".as_ptr() as PCHAR, (*ppdev).dhpdev, assertVal);

    if assertVal != 0 {
        (*ppdev).flFlags |= PDEV_DISABLED;
    }

    assertVal
}

// NTAPI in C (device.h:6) — stdcall on i386, cdecl elsewhere.
#[no_mangle]
pub unsafe extern "system" fn PDEVOBJ_vRefreshModeList(ppdev: PPDEVOBJ) {
    PDEVOBJ_vRefreshModeList_impl(ppdev)
}

unsafe fn PDEVOBJ_vRefreshModeList_impl(
    ppdev: PPDEVOBJ)
{
    let pGraphicsDevice: PGRAPHICS_DEVICE;
    let mut pdminfo: PDEVMODEINFO;
    let mut pdmiNext: PDEVMODEINFO;

    EngAcquireSemaphore((*ppdev).hsemDevLock);

    pGraphicsDevice = (*ppdev).pGraphicsDevice;

    pdminfo = (*pGraphicsDevice).pdevmodeInfo;
    while !pdminfo.is_null() {
        pdmiNext = (*pdminfo).pdmiNext;
        ExFreePoolWithTag(pdminfo as *mut core::ffi::c_void, GDITAG_DEVMODE);
        pdminfo = pdmiNext;
    }
    (*pGraphicsDevice).pdevmodeInfo = core::ptr::null_mut();
    ExFreePoolWithTag((*pGraphicsDevice).pDevModeList as *mut core::ffi::c_void, GDITAG_GDEVICE);
    (*pGraphicsDevice).pDevModeList = core::ptr::null_mut();

    LDEVOBJ_bBuildDevmodeList(pGraphicsDevice);

    EngReleaseSemaphore((*ppdev).hsemDevLock);
}

// OWNER: real export, declared `extern "C"` (no NTAPI in pdevobj.h:223-228)
// in win32k-sys bindgen for all 3 arches (PDEVOBJ_Create).
#[no_mangle]
pub unsafe extern "C" fn PDEVOBJ_Create(
    pGraphicsDevice: PGRAPHICS_DEVICE,
    pdm: PDEVMODEW,
    dwAccelerationLevel: u32,
    ldevtype: u32,
) -> PPDEVOBJ {

    let mut ppdev: PPDEVOBJ;
    let mut ppdevMatch: PPDEVOBJ = core::ptr::null_mut();
    let pldev: PLDEVOBJ;
    let pSurface: PSURFACE;

    TRACE!(b"PDEVOBJ_Create(%p %p %d)\n\0".as_ptr() as *const i8, pGraphicsDevice, pdm, ldevtype);

    if ldevtype != LDEV_DEVICE_META {
        ppdev = gppdevList;
        while !ppdev.is_null() {
            if (*ppdev).pGraphicsDevice == pGraphicsDevice {
                PDEVOBJ_vReference(ppdev);

                if RtlEqualMemory(pdm as *const core::ffi::c_void, (*ppdev).pdmwDev as *const core::ffi::c_void, core::mem::size_of::<DEVMODEW>()) != 0 &&
                   (*ppdev).dwAccelerationLevel == dwAccelerationLevel {
                    PDEVOBJ_vReference(ppdev);
                    ppdevMatch = ppdev;
                } else {
                    PDEVOBJ_bDisableDisplay(ppdev);
                }

                PDEVOBJ_vRelease(ppdev);
            }
            ppdev = (*ppdev).ppdevNext;
        }

        if !ppdevMatch.is_null() {
            PDEVOBJ_vEnableDisplay(ppdevMatch);
            return ppdevMatch;
        }
    }

    if ldevtype == LDEV_DEVICE_META {
        pldev = LDEVOBJ_pLoadInternal(MultiEnableDriver as *mut core::ffi::c_void, ldevtype);
    } else {
        pldev = LDEVOBJ_pLoadDriver((*pdm).dmDeviceName.as_mut_ptr(), ldevtype);
    }
    
    if pldev.is_null() {
        ERR!("Could not load display driver '%S'\n",
            if ldevtype == LDEV_DEVICE_META { [0u16].as_ptr() } else { (*pdm).dmDeviceName.as_ptr() });
        return core::ptr::null_mut();
    }

    ppdev = PDEVOBJ_AllocPDEV();
    if ppdev.is_null() {
        ERR!("failed to allocate a PDEV\n");
        return core::ptr::null_mut();
    }

    if ldevtype != LDEV_DEVICE_META {
        (*ppdev).pGraphicsDevice = pGraphicsDevice;
        (*ppdev).hSpooler = (*(*ppdev).pGraphicsDevice).DeviceObject as HANDLE;

        if !(*ppdev).pdmwDev.is_null() {
            ExFreePoolWithTag((*ppdev).pdmwDev as *mut core::ffi::c_void, GDITAG_DEVMODE);
        }
        (*ppdev).pdmwDev = ExAllocatePoolWithTag(PagedPool, ((*pdm).dmSize + (*pdm).dmDriverExtra) as SIZE_T, GDITAG_DEVMODE) as PDEVMODEW;
        if !(*ppdev).pdmwDev.is_null() {
            RtlCopyMemory((*ppdev).pdmwDev as *mut core::ffi::c_void, pdm as *const core::ffi::c_void, ((*pdm).dmSize + (*pdm).dmDriverExtra) as usize);
            (*pGraphicsDevice).StateFlags |= DISPLAY_DEVICE_ATTACHED_TO_DESKTOP;
        }
    }

    (*ppdev).flFlags = PDEV_DISPLAY;
    (*ppdev).Pointer.Exclude.right = -1;

    (*ppdev).pldev = pldev;
    (*ppdev).dwAccelerationLevel = dwAccelerationLevel;

    if ldevtype == LDEV_DEVICE_DISPLAY && (dwAccelerationLevel >= 5 ||
        ((*pdm).dmFields & (DM_PANNINGWIDTH | DM_PANNINGHEIGHT)) != 0) {
        let mut i: u32;

        if ((*pdm).dmFields & DM_PANNINGWIDTH) == 0 {
            (*pdm).dmPanningWidth = (*pdm).dmPelsWidth;
        }
        if ((*pdm).dmFields & DM_PANNINGHEIGHT) == 0 {
            (*pdm).dmPanningHeight = (*pdm).dmPelsHeight;
        }

        i = 0;
        while i < gPanDispDrvCount {
            (*ppdev).__bindgen_anon_1.apfn[(*gPanDispDrvFn.as_ptr().add(i as usize)).iFunc as usize] =
                core::mem::transmute::<PFN, PVOID>((*gPanDispDrvFn.as_ptr().add(i as usize)).pfn);
            i += 1;
        }
    } else {
        (*ppdev).__bindgen_anon_1.pfn = (*(*ppdev).pldev).__bindgen_anon_1.pfn;
    }

    if PDEVOBJ_bEnablePDEV(ppdev, pdm, core::ptr::null_mut()) == 0 {
        PDEVOBJ_vRelease(ppdev);
        EngUnloadImage(pldev as HANDLE);
        return core::ptr::null_mut();
    }

    PDEVOBJ_vCompletePDEV(ppdev);

    pSurface = PDEVOBJ_pSurface(ppdev);
    if pSurface.is_null() {
        PDEVOBJ_vRelease(ppdev);
        EngUnloadImage(pldev as HANDLE);
        return core::ptr::null_mut();
    }

    if PDEVOBJ_bEnableDirectDraw(ppdev) == 0 {
        PDEVOBJ_vRelease(ppdev);
        EngUnloadImage(pldev as HANDLE);
        return core::ptr::null_mut();
    }

    PDEVOBJ_vFilterDriverHooks(ppdev);

    (*ppdev).pfnMovePointer = (*ppdev).__bindgen_anon_1.pfn.MovePointer;
    if (*ppdev).pfnMovePointer.is_none() {
        (*ppdev).pfnMovePointer = Some(EngMovePointer);
    }

    (*ppdev).ppdevNext = gppdevList;
    gppdevList = ppdev;

    ppdev
}

unsafe fn SwitchPointer(
    pvPointer1: PVOID,
    pvPointer2: PVOID)
{
    let ppvPointer1 = pvPointer1 as *mut PVOID;
    let ppvPointer2 = pvPointer2 as *mut PVOID;
    let pvTemp: PVOID;

    pvTemp = *ppvPointer1;
    *ppvPointer1 = *ppvPointer2;
    *ppvPointer2 = pvTemp;
}
#[no_mangle]
pub unsafe extern "system" fn PDEVOBJ_bDynamicModeChange(
    ppdev: PPDEVOBJ,
    ppdev2: PPDEVOBJ,
) -> BOOL {
    let mut temp_pfn: DRIVER_FUNCTIONS;
    let mut temp_gdiinfo: GDIINFO;
    let mut temp_devinfo: DEVINFO;
    let mut temp_StateFlags: DWORD;

    temp_pfn = (*ppdev).__bindgen_anon_1.pfn;
    (*ppdev).__bindgen_anon_1.pfn = (*ppdev2).__bindgen_anon_1.pfn;
    (*ppdev2).__bindgen_anon_1.pfn = temp_pfn;

    SwitchPointer(
        core::ptr::addr_of_mut!((*ppdev).pldev) as PVOID,
        core::ptr::addr_of_mut!((*ppdev2).pldev) as PVOID,
    );

    SwitchPointer(
        core::ptr::addr_of_mut!((*ppdev).dhpdev) as PVOID,
        core::ptr::addr_of_mut!((*ppdev2).dhpdev) as PVOID,
    );

    SwitchPointer(
        core::ptr::addr_of_mut!((*ppdev).pSurface) as PVOID,
        core::ptr::addr_of_mut!((*ppdev2).pSurface) as PVOID,
    );
    (*(*ppdev).pSurface).SurfObj.hdev = ppdev as HDEV;
    (*(*ppdev2).pSurface).SurfObj.hdev = ppdev2 as HDEV;

    temp_devinfo = (*ppdev).devinfo;
    (*ppdev).devinfo = (*ppdev2).devinfo;
    (*ppdev2).devinfo = temp_devinfo;

    temp_gdiinfo = (*ppdev).gdiinfo;
    (*ppdev).gdiinfo = (*ppdev2).gdiinfo;
    (*ppdev2).gdiinfo = temp_gdiinfo;

    SwitchPointer(
        core::ptr::addr_of_mut!((*ppdev).pdmwDev) as PVOID,
        core::ptr::addr_of_mut!((*ppdev2).pdmwDev) as PVOID,
    );

    temp_StateFlags = (*(*ppdev).pGraphicsDevice).StateFlags;
    (*(*ppdev).pGraphicsDevice).StateFlags = (*(*ppdev2).pGraphicsDevice).StateFlags;
    (*(*ppdev2).pGraphicsDevice).StateFlags = temp_StateFlags;

    ((*ppdev).__bindgen_anon_1.pfn.CompletePDEV.unwrap())((*ppdev).dhpdev, ppdev as HDEV);
    ((*ppdev2).__bindgen_anon_1.pfn.CompletePDEV.unwrap())((*ppdev2).dhpdev, ppdev2 as HDEV);

    PDEVOBJ_vSwitchDirectDraw(ppdev, ppdev2);

    1
}
#[no_mangle]
pub unsafe extern "system" fn PDEVOBJ_bSwitchMode(
    ppdev: PPDEVOBJ,
    pdm: PDEVMODEW,
) -> BOOL {

    let mut ppdevTmp: PPDEVOBJ;
    let mut pSurface: PSURFACE;
    let mut retval: BOOL = 0;

    EngAcquireSemaphore((*ppdev).hsemDevLock);

    EngAcquireSemaphore(ghsemPDEV);

    // DPRINT removed

    if PDEVOBJ_bDisableDisplay(ppdev) == 0 {
        // DPRINT removed
        PDEVOBJ_vResumeDirectDraw(ppdev);
        EngReleaseSemaphore(ghsemPDEV);
        EngReleaseSemaphore((*ppdev).hsemDevLock);
        // DPRINT removed
        return retval;
    }

    ppdevTmp = PDEVOBJ_Create((*ppdev).pGraphicsDevice, pdm, 0, LDEV_DEVICE_DISPLAY);
    if ppdevTmp == core::ptr::null_mut() {
        // DPRINT removed
        PDEVOBJ_vEnableDisplay(ppdev);
        EngReleaseSemaphore(ghsemPDEV);
        EngReleaseSemaphore((*ppdev).hsemDevLock);
        // DPRINT removed
        return retval;
    }

    pSurface = PDEVOBJ_pSurface(ppdevTmp);
    if pSurface == core::ptr::null_mut() {
        // DPRINT removed
        PDEVOBJ_vRelease(ppdevTmp);
        PDEVOBJ_vEnableDisplay(ppdev);
        EngReleaseSemaphore(ghsemPDEV);
        EngReleaseSemaphore((*ppdev).hsemDevLock);
        // DPRINT removed
        return retval;
    }

    PDEVOBJ_vSuspendDirectDraw(ppdev);
    PDEVOBJ_vSuspendDirectDraw(ppdevTmp);

    if PDEVOBJ_bDynamicModeChange(ppdev, ppdevTmp) == 0 {
        // DPRINT removed
        PDEVOBJ_vRelease(ppdevTmp);
        PDEVOBJ_vEnableDisplay(ppdev);
        EngReleaseSemaphore(ghsemPDEV);
        EngReleaseSemaphore((*ppdev).hsemDevLock);
        // DPRINT removed
        return retval;
    }

    PDEVOBJ_vResumeDirectDraw(ppdev);
    PDEVOBJ_vResumeDirectDraw(ppdevTmp);

    PDEVOBJ_vRelease(ppdevTmp);

    (*(*ppdev).pEDDgpl).hDev = ppdev as HDEV;
    (*(*ppdev).pEDDgpl).dhpdev = (*ppdev).dhpdev as PVOID;

    if ppdev == (*gpmdev).ppdevGlobal {
        PDEVOBJ_vGetDeviceCaps(ppdev, core::ptr::addr_of_mut!((*GdiHandleTable).DevCaps));
    }

    retval = 1;

    PDEVOBJ_vEnableDisplay(ppdev);

    EngReleaseSemaphore(ghsemPDEV);
    EngReleaseSemaphore((*ppdev).hsemDevLock);

    // DPRINT removed

    return retval;
}

#[no_mangle]
pub unsafe extern "system" fn EngpGetPDEV(
    pustrDeviceName: PUNICODE_STRING,
) -> PPDEVOBJ {
    let mut ustrCurrent: _UNICODE_STRING = core::mem::zeroed();
    let mut ppdev: PPDEVOBJ = core::ptr::null_mut();
    let mut pGraphicsDevice: PGRAPHICS_DEVICE;
    let mut i: ULONG;

    EngAcquireSemaphore(ghsemPDEV);

    if !pustrDeviceName.is_null() {
        i = 0;
        while i < (*gpmdev).cDev {
            pGraphicsDevice = (*(*mdev_get(gpmdev, i as usize)).ppdev).pGraphicsDevice;

            RtlInitUnicodeString(
                core::ptr::addr_of_mut!(ustrCurrent),
                (*pGraphicsDevice).szWinDeviceName.as_ptr(),
            );
            if RtlEqualUnicodeString(pustrDeviceName, core::ptr::addr_of!(ustrCurrent), 0u8) != 0 {
                ppdev = (*mdev_get(gpmdev, i as usize)).ppdev;
                break;
            }
            i += 1;
        }
    } else if !gpmdev.is_null() {
        ppdev = (*gpmdev).ppdevGlobal;
    }

    if !ppdev.is_null() {
        PDEVOBJ_vReference(ppdev);
    }

    EngReleaseSemaphore(ghsemPDEV);

    ppdev
}

// Plain cdecl in C (no NTAPI/APIENTRY — pdevobj.h:239) — `extern "C"` on all arches.
#[no_mangle]
pub unsafe extern "C" fn PDEVOBJ_lChangeDisplaySettings(
    pustrDeviceName: PUNICODE_STRING,
    RequestedMode: PDEVMODEW,
    pmdevOld: PMDEVOBJ,
    ppmdevNew: *mut PMDEVOBJ,
    bSearchClosestMode: BOOL,
) -> i32 {

    let mut pGraphicsDevice: PGRAPHICS_DEVICE = core::ptr::null_mut();
    let mut pmdev: PMDEVOBJ = core::ptr::null_mut();
    let mut pdm: PDEVMODEW = core::ptr::null_mut();
    let mut lRet: i32 = DISP_CHANGE_SUCCESSFUL as i32;
    let mut i: u32;
    let mut j: u32;

    TRACE!(
        b"PDEVOBJ_lChangeDisplaySettings('%wZ' '%dx%dx%d (%d Hz)' %p %p)\n\0".as_ptr() as *const i8,
        pustrDeviceName,
        if !RequestedMode.is_null() { (*RequestedMode).dmPelsWidth } else { 0 },
        if !RequestedMode.is_null() { (*RequestedMode).dmPelsHeight } else { 0 },
        if !RequestedMode.is_null() { (*RequestedMode).dmBitsPerPel } else { 0 },
        if !RequestedMode.is_null() { (*RequestedMode).dmDisplayFrequency } else { 0 },
        pmdevOld,
        ppmdevNew,
    );

    'cleanup: loop {
        if !pustrDeviceName.is_null() {
            pGraphicsDevice = EngpFindGraphicsDevice(pustrDeviceName, 0);
            if pGraphicsDevice.is_null() {
                ERR!("Wrong device name provided: '%wZ'\n", pustrDeviceName);
                lRet = DISP_CHANGE_BADPARAM as i32;
                break 'cleanup;
            }
        } else if !RequestedMode.is_null() {
            pGraphicsDevice = EngpFindGraphicsDevice(core::ptr::null_mut(), 0);
            if pGraphicsDevice.is_null() {
                ERR!("Wrong device'\n");
                lRet = DISP_CHANGE_BADPARAM as i32;
                break 'cleanup;
            }
        }

        if !pGraphicsDevice.is_null() {
            if LDEVOBJ_bProbeAndCaptureDevmode(pGraphicsDevice, RequestedMode, core::ptr::addr_of_mut!(pdm), bSearchClosestMode) == 0 {
                ERR!("DrvProbeAndCaptureDevmode() failed\n");
                lRet = DISP_CHANGE_BADMODE as i32;
                break 'cleanup;
            }
        }

        if !pmdevOld.is_null() {
            if MDEVOBJ_bDisable(pmdevOld) != 0 {
                pmdev = MDEVOBJ_Create(pustrDeviceName, pdm);
                if pmdev.is_null() {
                    MDEVOBJ_vEnable(pmdevOld);
                }
            }
        } else {
            pmdev = MDEVOBJ_Create(pustrDeviceName, pdm);
        }

        if pmdev.is_null() {
            ERR!("Failed to create new MDEV\n");
            lRet = DISP_CHANGE_FAILED as i32;
            break 'cleanup;
        }

        lRet = DISP_CHANGE_SUCCESSFUL as i32;
        *ppmdevNew = pmdev;

        if !pustrDeviceName.is_null() && !pmdevOld.is_null() {
            i = 0;
            while i < (*pmdevOld).cDev {
                j = 0;
                while j < (*pmdev).cDev {
                    if (*(*pmdev).dev.as_ptr().add(j as usize)).ppdev != core::ptr::null_mut()
                        && (*(*pmdevOld).dev.as_ptr().add(i as usize)).ppdev != core::ptr::null_mut()
                        && (*(*(*pmdev).dev.as_ptr().add(j as usize)).ppdev).pGraphicsDevice
                            == (*(*(*pmdevOld).dev.as_ptr().add(i as usize)).ppdev).pGraphicsDevice
                    {
                        if PDEVOBJ_bDynamicModeChange(
                            (*(*pmdevOld).dev.as_ptr().add(i as usize)).ppdev,
                            (*(*pmdev).dev.as_ptr().add(j as usize)).ppdev,
                        ) != 0
                        {
                            let tmp = (*mdev_get(pmdevOld, i as usize)).ppdev;
                            (*mdev_get(pmdevOld, i as usize)).ppdev = (*mdev_get(pmdev, j as usize)).ppdev;
                            (*mdev_get(pmdev, j as usize)).ppdev = tmp;
                        } else {
                            ERR!("Failed to apply new settings\n");
                        }
                        break;
                    }
                    j += 1;
                }
                if j == (*pmdev).cDev {
                    PDEVOBJ_vReference((*(*pmdevOld).dev.as_ptr().add(i as usize)).ppdev);
                    (*mdev_get(pmdev, (*pmdev).cDev as usize)).ppdev = (*mdev_get(pmdevOld, i as usize)).ppdev;
                    (*pmdev).cDev += 1;
                }
                i += 1;
            }
        }

        if (*pmdev).cDev == 1 {
            (*pmdev).ppdevGlobal = (*(*pmdev).dev.as_ptr().add(0)).ppdev;
        } else {
            (*pmdev).ppdevGlobal = PDEVOBJ_Create(core::ptr::null_mut(), pmdev as PDEVMODEW, 0, LDEV_DEVICE_META);
            if (*pmdev).ppdevGlobal.is_null() {
                WARN!(b"Failed to create meta-device. Using only first display\n\0".as_ptr() as *const i8);
                PDEVOBJ_vReference((*(*pmdev).dev.as_ptr().add(0)).ppdev);
                (*pmdev).ppdevGlobal = (*(*pmdev).dev.as_ptr().add(0)).ppdev;
            }
        }

        if !pmdevOld.is_null() {
            i = 0;
            while i < (*pmdevOld).cDev {
                j = 0;
                while j < (*pmdev).cDev {
                    if (*(*pmdev).dev.as_ptr().add(j as usize)).ppdev != core::ptr::null_mut()
                        && (*(*pmdevOld).dev.as_ptr().add(i as usize)).ppdev != core::ptr::null_mut()
                        && (*(*(*pmdev).dev.as_ptr().add(j as usize)).ppdev).pGraphicsDevice
                            == (*(*(*pmdevOld).dev.as_ptr().add(i as usize)).ppdev).pGraphicsDevice
                    {
                        break;
                    }
                    j += 1;
                }
                if j == (*pmdev).cDev {
                    PDEVOBJ_bDisableDisplay((*(*pmdevOld).dev.as_ptr().add(i as usize)).ppdev);
                }
                i += 1;
            }
        }

        break 'cleanup;
    }

    if lRet != DISP_CHANGE_SUCCESSFUL as i32 {
        *ppmdevNew = core::ptr::null_mut();
        if !pmdev.is_null() {
            MDEVOBJ_vDestroy(pmdev);
        }
        if !pdm.is_null() && pdm != RequestedMode {
            ExFreePoolWithTag(pdm as *mut core::ffi::c_void, 0x444D4447);
        }
    }

    lRet as i32
}

unsafe fn PDEVOBJ_iGetColorManagementCaps(ppdev: PPDEVOBJ) -> INT {
    let mut ret: INT = CM_NONE as INT;

    if (*ppdev).flFlags & PDEV_DISPLAY != 0 {
        if (*ppdev).devinfo.iDitherFormat == BMF_8BPP ||
           (*ppdev).devinfo.flGraphicsCaps2 & GCAPS2_CHANGEGAMMARAMP != 0 {
            ret = CM_GAMMA_RAMP as INT;
        }
    }

    if (*ppdev).devinfo.flGraphicsCaps & GCAPS_CMYKCOLOR != 0 {
        ret |= CM_CMYK_COLOR as INT;
    }
    if (*ppdev).devinfo.flGraphicsCaps & GCAPS_ICM != 0 {
        ret |= CM_DEVICE_ICM as INT;
    }

    ret
}

// NTAPI in C (pdevobj.h:186) — stdcall on i386, cdecl elsewhere.
#[no_mangle]
pub unsafe extern "system" fn PDEVOBJ_vGetDeviceCaps(ppdev: PPDEVOBJ, pDevCaps: PDEVCAPS) {
    PDEVOBJ_vGetDeviceCaps_impl(ppdev, pDevCaps)
}

unsafe fn PDEVOBJ_vGetDeviceCaps_impl(
    ppdev: PPDEVOBJ,
    pDevCaps: PDEVCAPS)
{
    let pGdiInfo: PGDIINFO = core::ptr::addr_of_mut!((*ppdev).gdiinfo);

    (*pDevCaps).ulVersion = (*pGdiInfo).ulVersion;
    (*pDevCaps).ulTechnology = (*pGdiInfo).ulTechnology;
    (*pDevCaps).ulHorzSizeM = ((*pGdiInfo).ulHorzSize + 500) / 1000;
    (*pDevCaps).ulVertSizeM = ((*pGdiInfo).ulVertSize + 500) / 1000;
    (*pDevCaps).ulHorzSize = (*pGdiInfo).ulHorzSize;
    (*pDevCaps).ulVertSize = (*pGdiInfo).ulVertSize;
    (*pDevCaps).ulHorzRes = (*pGdiInfo).ulHorzRes;
    (*pDevCaps).ulVertRes = (*pGdiInfo).ulVertRes;
    (*pDevCaps).ulBitsPixel = (*pGdiInfo).cBitsPixel;
    if (*pDevCaps).ulBitsPixel == 15 { (*pDevCaps).ulBitsPixel = 16; }
    (*pDevCaps).ulPlanes = (*pGdiInfo).cPlanes;
    (*pDevCaps).ulNumPens = (*pGdiInfo).ulNumColors;
    if (*pDevCaps).ulNumPens != 0xFFFFFFFF { (*pDevCaps).ulNumPens *= 5; }
    (*pDevCaps).ulNumFonts = 0;
    (*pDevCaps).ulNumColors = (*pGdiInfo).ulNumColors;
    (*pDevCaps).ulRasterCaps = (*pGdiInfo).flRaster;
    (*pDevCaps).ulAspectX = (*pGdiInfo).ulAspectX;
    (*pDevCaps).ulAspectY = (*pGdiInfo).ulAspectY;
    (*pDevCaps).ulAspectXY = (*pGdiInfo).ulAspectXY;
    (*pDevCaps).ulLogPixelsX = (*pGdiInfo).ulLogPixelsX;
    (*pDevCaps).ulLogPixelsY = (*pGdiInfo).ulLogPixelsY;
    (*pDevCaps).ulSizePalette = (*pGdiInfo).ulNumPalReg;
    (*pDevCaps).ulColorRes = (*pGdiInfo).ulDACRed +
                             (*pGdiInfo).ulDACGreen +
                             (*pGdiInfo).ulDACBlue;
    (*pDevCaps).ulPhysicalWidth = (*pGdiInfo).szlPhysSize.cx as u32;
    (*pDevCaps).ulPhysicalHeight = (*pGdiInfo).szlPhysSize.cy as u32;
    (*pDevCaps).ulPhysicalOffsetX = (*pGdiInfo).ptlPhysOffset.x as u32;
    (*pDevCaps).ulPhysicalOffsetY = (*pGdiInfo).ptlPhysOffset.y as u32;
    (*pDevCaps).ulTextCaps = (*pGdiInfo).flTextCaps;
    (*pDevCaps).ulTextCaps |= TC_SO_ABLE | TC_UA_ABLE | TC_CP_STROKE | TC_OP_STROKE | TC_OP_CHARACTER;
    if (*pGdiInfo).ulTechnology != DT_PLOTTER {
        (*pDevCaps).ulTextCaps |= TC_VA_ABLE;
    }
    (*pDevCaps).ulVRefresh = (*pGdiInfo).ulVRefresh;
    (*pDevCaps).ulDesktopHorzRes = (*pGdiInfo).ulHorzRes;
    (*pDevCaps).ulDesktopVertRes = (*pGdiInfo).ulVertRes;
    (*pDevCaps).ulBltAlignment = (*pGdiInfo).ulBltAlignment;
    (*pDevCaps).ulPanningHorzRes = (*pGdiInfo).ulPanningHorzRes;
    (*pDevCaps).ulPanningVertRes = (*pGdiInfo).ulPanningVertRes;
    (*pDevCaps).xPanningAlignment = (*pGdiInfo).xPanningAlignment;
    (*pDevCaps).yPanningAlignment = (*pGdiInfo).yPanningAlignment;
    (*pDevCaps).ulShadeBlend = (*pGdiInfo).flShadeBlend;
    (*pDevCaps).ulColorMgmtCaps = PDEVOBJ_iGetColorManagementCaps(ppdev) as u32;
}
#[no_mangle]
pub unsafe extern "system" fn EngQueryDeviceAttribute(
    hdev: HDEV,
    devAttr: ENG_DEVICE_ATTRIBUTE,
    pvIn: PVOID,
    cjInSize: ULONG,
    pvOut: PVOID,
    cjOutSize: ULONG,
) -> BOOL {
    let ppdev = hdev as PPDEVOBJ;

    if devAttr != QDA_ACCELERATION_LEVEL {
        return 0;
    }

    if cjOutSize >= core::mem::size_of::<DWORD>() as ULONG {
        *(pvOut as *mut DWORD) = (*ppdev).dwAccelerationLevel;
        return 1;
    }

    0
}
#[no_mangle]
pub unsafe extern "system" fn EngGetDriverName(hdev: HDEV) -> LPWSTR {
    let ppdev = hdev as PPDEVOBJ;
    
    (*(*(*ppdev).pldev).pGdiDriverInfo).DriverName.Buffer
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetDeviceCaps(
    hdc: HDC,
    Index: i32) -> i32 {
    let pdc: PDC;
    let mut devcaps: DEVCAPS = core::mem::zeroed();

    pdc = DC_LockDc(hdc);
    if pdc.is_null()
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    PDEVOBJ_vGetDeviceCaps((*pdc).ppdev, core::ptr::addr_of_mut!(devcaps));

    DC_UnlockDc(pdc);

    match Index as u32
    {
        DRIVERVERSION =>
            return devcaps.ulVersion as i32,

        TECHNOLOGY =>
            return devcaps.ulTechnology as i32,

        HORZSIZE =>
            return devcaps.ulHorzSize as i32,

        VERTSIZE =>
            return devcaps.ulVertSize as i32,

        HORZRES =>
            return devcaps.ulHorzRes as i32,

        VERTRES =>
            return devcaps.ulVertRes as i32,

        LOGPIXELSX =>
            return devcaps.ulLogPixelsX as i32,

        LOGPIXELSY =>
            return devcaps.ulLogPixelsY as i32,

        BITSPIXEL =>
            return devcaps.ulBitsPixel as i32,

        PLANES =>
            return devcaps.ulPlanes as i32,

        NUMBRUSHES =>
            return -1,

        NUMPENS =>
            return devcaps.ulNumPens as i32,

        NUMFONTS =>
            return devcaps.ulNumFonts as i32,

        NUMCOLORS =>
            return devcaps.ulNumColors as i32,

        ASPECTX =>
            return devcaps.ulAspectX as i32,

        ASPECTY =>
            return devcaps.ulAspectY as i32,

        ASPECTXY =>
            return devcaps.ulAspectXY as i32,

        CLIPCAPS =>
            return CP_RECTANGLE as i32,

        SIZEPALETTE =>
            return devcaps.ulSizePalette as i32,

        NUMRESERVED =>
            return 20,

        COLORRES =>
            return devcaps.ulColorRes as i32,

        DESKTOPVERTRES =>
            return devcaps.ulVertRes as i32,

        DESKTOPHORZRES =>
            return devcaps.ulHorzRes as i32,

        BLTALIGNMENT =>
            return devcaps.ulBltAlignment as i32,

        SHADEBLENDCAPS =>
            return devcaps.ulShadeBlend as i32,

        COLORMGMTCAPS =>
            return devcaps.ulColorMgmtCaps as i32,

        PHYSICALWIDTH =>
            return devcaps.ulPhysicalWidth as i32,

        PHYSICALHEIGHT =>
            return devcaps.ulPhysicalHeight as i32,

        PHYSICALOFFSETX =>
            return devcaps.ulPhysicalOffsetX as i32,

        PHYSICALOFFSETY =>
            return devcaps.ulPhysicalOffsetY as i32,

        VREFRESH =>
            return devcaps.ulVRefresh as i32,

        RASTERCAPS =>
            return devcaps.ulRasterCaps as i32,

        CURVECAPS =>
            return (CC_CIRCLES | CC_PIE | CC_CHORD | CC_ELLIPSES | CC_WIDE |
                    CC_STYLED | CC_WIDESTYLED | CC_INTERIORS | CC_ROUNDRECT) as i32,

        LINECAPS =>
            return (LC_POLYLINE | LC_MARKER | LC_POLYMARKER | LC_WIDE |
                    LC_STYLED | LC_WIDESTYLED | LC_INTERIORS) as i32,

        POLYGONALCAPS =>
            return (PC_POLYGON | PC_RECTANGLE | PC_WINDPOLYGON | PC_SCANLINE |
                    PC_WIDE | PC_STYLED | PC_WIDESTYLED | PC_INTERIORS) as i32,

        TEXTCAPS =>
            return devcaps.ulTextCaps as i32,

        CAPS1 | PDEVICESIZE | SCALINGFACTORX | SCALINGFACTORY | _ =>
            return 0,
    }
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetDeviceCapsAll(
    hDC: HDC,
    pDevCaps: PDEVCAPS) -> BOOL {
    let mut pdc: PDC;
    let mut devcaps: DEVCAPS = core::mem::zeroed();
    let mut bResult: BOOL = 1;
    let mut Status: NTSTATUS;

    pdc = DC_LockDc(hDC);
    if pdc.is_null()
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    PDEVOBJ_vGetDeviceCaps((*pdc).ppdev, core::ptr::addr_of_mut!(devcaps));

    DC_UnlockDc(pdc);

    // C: _SEH2_TRY { ProbeForWrite(pDevCaps, sizeof(DEVCAPS), 1); RtlCopyMemory(..); }
    // _SEH2_EXCEPT { SetLastNtError(code); return FALSE; } -- probe + fault-tolerant copy.
    if let Err(st) = crate::safe_user_io::copy_to_user(
        pDevCaps as *mut core::ffi::c_void,
        core::ptr::addr_of!(devcaps) as *const core::ffi::c_void,
        core::mem::size_of::<DEVCAPS>(),
        1,
    ) {
        SetLastNtError(st);
        bResult = 0;
    }

    return bResult;
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetDhpdev(
    hdev: HDEV,
) -> DHPDEV {
    let mut ppdev: PPDEVOBJ;
    let mut dhpdev: DHPDEV = core::ptr::null_mut();

    if hdev.is_null() || (hdev as *mut u8) < MmSystemRangeStart as *mut u8 {
        return core::ptr::null_mut();
    }

    EngAcquireSemaphoreShared(ghsemPDEV);

    ppdev = gppdevList;
    while !ppdev.is_null() {
        if ppdev == hdev as PPDEVOBJ {
            dhpdev = (*ppdev).dhpdev;
            break;
        }
        ppdev = (*ppdev).ppdevNext;
    }

    EngReleaseSemaphore(ghsemPDEV);

    dhpdev
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PDEVOBJ_sizl(ppdev: PPDEVOBJ, psizl: PSIZEL) -> PSIZEL {
    if (*ppdev).flFlags & PDEV_META_DEVICE != 0 {
        *psizl = (*ppdev).szlMetaRes;
    } else {
        (*psizl).cx = (*ppdev).gdiinfo.ulHorzRes as LONG;
        (*psizl).cy = (*ppdev).gdiinfo.ulVertRes as LONG;
    }
    psizl
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PDEVOBJ_sizl(ppdev: PPDEVOBJ, psizl: PSIZEL) -> PSIZEL {
    if (*ppdev).flFlags & PDEV_META_DEVICE != 0 {
        *psizl = (*ppdev).szlMetaRes;
    } else {
        (*psizl).cx = (*ppdev).gdiinfo.ulHorzRes as LONG;
        (*psizl).cy = (*ppdev).gdiinfo.ulVertRes as LONG;
    }
    psizl
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * PURPOSE:          Support for physical devices
//  * FILE:             win32ss/gdi/eng/pdevobj.c
//  * PROGRAMERS:       Timo Kreuzer (timo.kreuzer@reactos.org)
//  *                   Oleg Dubinskiy (oleg.dubinskij30@gmail.com)
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
// #define NDEBUG
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(EngPDev);
//
// static PPDEVOBJ gppdevList = NULL;
// static HSEMAPHORE ghsemPDEV;
//
// BOOL
// APIENTRY
// MultiEnableDriver(
//     _In_ ULONG iEngineVersion,
//     _In_ ULONG cj,
//     _Inout_bytecount_(cj) PDRVENABLEDATA pded);
//
// extern DRVFN gPanDispDrvFn[];
// extern ULONG gPanDispDrvCount;
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// InitPDEVImpl(VOID)
// {
//     ghsemPDEV = EngCreateSemaphore();
//     if (!ghsemPDEV) return STATUS_INSUFFICIENT_RESOURCES;
//     return STATUS_SUCCESS;
// }
//
// #if DBG
// PPDEVOBJ
// NTAPI
// DbgLookupDHPDEV(DHPDEV dhpdev)
// {
//     PPDEVOBJ ppdev;
//
//     /* Lock PDEV list */
//     EngAcquireSemaphoreShared(ghsemPDEV);
//
//     /* Walk through the list of PDEVs */
//     for (ppdev = gppdevList;  ppdev; ppdev = ppdev->ppdevNext)
//     {
//         /* Compare with the given DHPDEV */
//         if (ppdev->dhpdev == dhpdev) break;
//     }
//
//     /* Unlock PDEV list */
//     EngReleaseSemaphore(ghsemPDEV);
//
//     return ppdev;
// }
// #endif
//
// PPDEVOBJ
// PDEVOBJ_AllocPDEV(VOID)
// {
//     PPDEVOBJ ppdev;
//
//     ppdev = ExAllocatePoolWithTag(PagedPool, sizeof(PDEVOBJ), GDITAG_PDEV);
//     if (!ppdev)
//         return NULL;
//
//     RtlZeroMemory(ppdev, sizeof(PDEVOBJ));
//
//     ppdev->hsemDevLock = EngCreateSemaphore();
//     if (ppdev->hsemDevLock == NULL)
//     {
//         ExFreePoolWithTag(ppdev, GDITAG_PDEV);
//         return NULL;
//     }
//
//     /* Allocate EDD_DIRECTDRAW_GLOBAL for our ReactX driver */
//     ppdev->pEDDgpl = ExAllocatePoolWithTag(PagedPool, sizeof(EDD_DIRECTDRAW_GLOBAL), GDITAG_PDEV);
//     if (ppdev->pEDDgpl)
//         RtlZeroMemory(ppdev->pEDDgpl, sizeof(EDD_DIRECTDRAW_GLOBAL));
//
//     ppdev->cPdevRefs = 1;
//
//     return ppdev;
// }
//
// static
// VOID
// PDEVOBJ_vDeletePDEV(
//     PPDEVOBJ ppdev)
// {
//     EngDeleteSemaphore(ppdev->hsemDevLock);
//     if (ppdev->pdmwDev)
//         ExFreePoolWithTag(ppdev->pdmwDev, GDITAG_DEVMODE);
//     if (ppdev->pEDDgpl)
//         ExFreePoolWithTag(ppdev->pEDDgpl, GDITAG_PDEV);
//     ExFreePoolWithTag(ppdev, GDITAG_PDEV);
// }
//
// VOID
// NTAPI
// PDEVOBJ_vRelease(
//     _Inout_ PPDEVOBJ ppdev)
// {
//     /* Lock loader */
//     EngAcquireSemaphore(ghsemPDEV);
//
//     /* Decrease reference count */
//     InterlockedDecrement(&ppdev->cPdevRefs);
//     ASSERT(ppdev->cPdevRefs >= 0);
//
//     /* Check if references are left */
//     if (ppdev->cPdevRefs == 0)
//     {
//         /* Do we have a surface? */
//         if (ppdev->pSurface)
//         {
//             /* Release the surface and let the driver free it */
//             SURFACE_ShareUnlockSurface(ppdev->pSurface);
//             TRACE("DrvDisableSurface(dhpdev %p)\n", ppdev->dhpdev);
//             ppdev->pfn.DisableSurface(ppdev->dhpdev);
//         }
//
//         /* Do we have a palette? */
//         if (ppdev->ppalSurf)
//         {
//             PALETTE_ShareUnlockPalette(ppdev->ppalSurf);
//         }
//
//         /* Check if the PDEV was enabled */
//         if (ppdev->dhpdev != NULL)
//         {
//             /* Disable the PDEV */
//             TRACE("DrvDisablePDEV(dhpdev %p)\n", ppdev->dhpdev);
//             ppdev->pfn.DisablePDEV(ppdev->dhpdev);
//         }
//
//         /* Remove it from list */
//         if (ppdev == gppdevList)
//         {
//             gppdevList = ppdev->ppdevNext;
//         }
//         else if (gppdevList)
//         {
//             PPDEVOBJ ppdevCurrent = gppdevList;
//             BOOL found = FALSE;
//             while (!found && ppdevCurrent->ppdevNext)
//             {
//                 if (ppdevCurrent->ppdevNext == ppdev)
//                     found = TRUE;
//                 else
//                     ppdevCurrent = ppdevCurrent->ppdevNext;
//             }
//             if (found)
//                 ppdevCurrent->ppdevNext = ppdev->ppdevNext;
//         }
//
//         /* Unload display driver */
//         EngUnloadImage(ppdev->pldev);
//
//         /* Free it */
//         PDEVOBJ_vDeletePDEV(ppdev);
//     }
//
//     /* Unlock loader */
//     EngReleaseSemaphore(ghsemPDEV);
// }
//
// BOOL
// NTAPI
// PDEVOBJ_bEnablePDEV(
//     _In_ PPDEVOBJ ppdev,
//     _In_ PDEVMODEW pdevmode,
//     _In_ PWSTR pwszLogAddress)
// {
//     PFN_DrvEnablePDEV pfnEnablePDEV;
//     ULONG i;
//
//     /* Get the DrvEnablePDEV function */
//     pfnEnablePDEV = ppdev->pfn.EnablePDEV;
//
//     /* Call the drivers DrvEnablePDEV function */
//     TRACE("DrvEnablePDEV(pdevmode %p (%dx%dx%d %d Hz) hdev %p (%S))\n",
//         pdevmode,
//         ppdev->pGraphicsDevice ? pdevmode->dmPelsWidth : 0,
//         ppdev->pGraphicsDevice ? pdevmode->dmPelsHeight : 0,
//         ppdev->pGraphicsDevice ? pdevmode->dmBitsPerPel : 0,
//         ppdev->pGraphicsDevice ? pdevmode->dmDisplayFrequency : 0,
//         ppdev,
//         ppdev->pGraphicsDevice ? ppdev->pGraphicsDevice->szNtDeviceName : L"");
//     ppdev->dhpdev = pfnEnablePDEV(pdevmode,
//                                   pwszLogAddress,
//                                   HS_DDI_MAX,
//                                   ppdev->ahsurf,
//                                   sizeof(GDIINFO),
//                                   (PULONG)&ppdev->gdiinfo,
//                                   sizeof(DEVINFO),
//                                   &ppdev->devinfo,
//                                   (HDEV)ppdev,
//                                   ppdev->pGraphicsDevice ? ppdev->pGraphicsDevice->pwszDescription : NULL,
//                                   ppdev->pGraphicsDevice ? ppdev->pGraphicsDevice->DeviceObject : NULL);
//     TRACE("DrvEnablePDEV(pdevmode %p hdev %p) => dhpdev %p\n", pdevmode, ppdev, ppdev->dhpdev);
//     if (ppdev->dhpdev == NULL)
//     {
//         ERR("Failed to enable PDEV\n");
//         return FALSE;
//     }
//
//     /* Fix up some values */
//     if (ppdev->gdiinfo.ulLogPixelsX == 0)
//         ppdev->gdiinfo.ulLogPixelsX = 96;
//
//     if (ppdev->gdiinfo.ulLogPixelsY == 0)
//         ppdev->gdiinfo.ulLogPixelsY = 96;
//
//     /* Set raster caps */
//     ppdev->gdiinfo.flRaster = RC_OP_DX_OUTPUT | RC_GDI20_OUTPUT | RC_BIGFONT;
//     if ((ppdev->gdiinfo.ulTechnology != DT_PLOTTER) && (ppdev->gdiinfo.ulTechnology != DT_CHARSTREAM))
//         ppdev->gdiinfo.flRaster |= RC_STRETCHDIB | RC_STRETCHBLT | RC_DIBTODEV | RC_DI_BITMAP | RC_BITMAP64 | RC_BITBLT;
//     if (ppdev->gdiinfo.ulTechnology == DT_RASDISPLAY)
//         ppdev->gdiinfo.flRaster |= RC_FLOODFILL;
//     if (ppdev->devinfo.flGraphicsCaps & GCAPS_PALMANAGED)
//         ppdev->gdiinfo.flRaster |= RC_PALETTE;
//
//     /* Setup Palette */
//     ppdev->ppalSurf = PALETTE_ShareLockPalette(ppdev->devinfo.hpalDefault);
//
//     /* Setup hatch brushes */
//     for (i = 0; i < HS_DDI_MAX; i++)
//     {
//         if (ppdev->ahsurf[i] == NULL)
//             ppdev->ahsurf[i] = gahsurfHatch[i];
//     }
//
//     TRACE("PDEVOBJ_bEnablePDEV - dhpdev = %p\n", ppdev->dhpdev);
//
//     return TRUE;
// }
//
// VOID
// NTAPI
// PDEVOBJ_vCompletePDEV(
//     PPDEVOBJ ppdev)
// {
//     /* Call the drivers DrvCompletePDEV function */
//     TRACE("DrvCompletePDEV(dhpdev %p hdev %p)\n", ppdev->dhpdev, ppdev);
//     ppdev->pfn.CompletePDEV(ppdev->dhpdev, (HDEV)ppdev);
// }
//
// static
// VOID
// PDEVOBJ_vFilterDriverHooks(
//     _In_ PPDEVOBJ ppdev)
// {
//     PLDEVOBJ pldev = ppdev->pldev;
//     ULONG dwAccelerationLevel = ppdev->dwAccelerationLevel;
//
//     if (!pldev->pGdiDriverInfo)
//         return;
//     if (pldev->ldevtype != LDEV_DEVICE_DISPLAY)
//         return;
//
//     if (dwAccelerationLevel >= 1)
//     {
//         ppdev->apfn[INDEX_DrvSetPointerShape] = NULL;
//         ppdev->apfn[INDEX_DrvCreateDeviceBitmap] = NULL;
//     }
//
//     if (dwAccelerationLevel >= 2)
//     {
//         /* Remove sophisticated display accelerations */
//         ppdev->pSurface->flags &= ~(HOOK_STRETCHBLT |
//                                     HOOK_FILLPATH |
//                                     HOOK_GRADIENTFILL |
//                                     HOOK_LINETO |
//                                     HOOK_ALPHABLEND |
//                                     HOOK_TRANSPARENTBLT);
//     }
//
//     if (dwAccelerationLevel >= 3)
//     {
//         /* Disable DirectDraw and Direct3D accelerations */
//         /* FIXME: need to call DxDdSetAccelLevel */
//         UNIMPLEMENTED;
//     }
//
//     if (dwAccelerationLevel >= 4)
//     {
//         /* Remove almost all display accelerations */
//         ppdev->pSurface->flags &= ~HOOK_FLAGS |
//                                    HOOK_BITBLT |
//                                    HOOK_COPYBITS |
//                                    HOOK_TEXTOUT |
//                                    HOOK_STROKEPATH |
//                                    HOOK_SYNCHRONIZE;
//
//     }
//
//     if (dwAccelerationLevel >= 5)
//     {
//         /* Disable all display accelerations */
//         /* (nothing to do. Already handled in PDEVOBJ_Create) */
//     }
// }
//
// PSURFACE
// NTAPI
// PDEVOBJ_pSurface(
//     PPDEVOBJ ppdev)
// {
//     HSURF hsurf;
//
//     /* Check if there is no surface for this PDEV yet */
//     if (ppdev->pSurface == NULL)
//     {
//         /* Call the drivers DrvEnableSurface */
//         hsurf = ppdev->pfn.EnableSurface(ppdev->dhpdev);
//         if (hsurf == NULL)
//         {
//             return NULL;
//         }
//
//         /* Get a reference to the surface */
//         ppdev->pSurface = SURFACE_ShareLockSurface(hsurf);
//         NT_ASSERT(ppdev->pSurface != NULL);
//     }
//
//     /* Increment reference count */
//     GDIOBJ_vReferenceObjectByPointer(&ppdev->pSurface->BaseObject);
//
//     return ppdev->pSurface;
// }
//
// BOOL
// PDEVOBJ_bEnableDirectDraw(
//     _Inout_ PPDEVOBJ ppdev)
// {
//     PGD_DXDDENABLEDIRECTDRAW pfnDdEnableDirectDraw = (PGD_DXDDENABLEDIRECTDRAW)gpDxFuncs[DXG_INDEX_DxDdEnableDirectDraw].pfn;
//     BOOL Success;
//
//     /* Check if DirectDraw is available (dxg.sys may not be loaded, e.g. ARM64) */
//     if (!pfnDdEnableDirectDraw)
//     {
//         return TRUE;
//     }
//
//     /* Enable DirectDraw */
//     TRACE("DxDdEnableDirectDraw(ppdev %p)\n", ppdev);
//     Success = pfnDdEnableDirectDraw((HDEV)ppdev, TRUE);
//     TRACE("DxDdEnableDirectDraw(ppdev %p) => %d\n", ppdev, Success);
//
//     return Success;
// }
//
// VOID
// PDEVOBJ_vResumeDirectDraw(
//     _Inout_ PPDEVOBJ ppdev)
// {
//     PGD_DXDDRESUMEDIRECTDRAW pfnDdResumeDirectDraw = (PGD_DXDDRESUMEDIRECTDRAW)gpDxFuncs[DXG_INDEX_DxDdResumeDirectDraw].pfn;
//
//     /* Resume DirectDraw after mode change */
//     TRACE("DxDdResumeDirectDraw(ppdev %p)\n", ppdev);
//     pfnDdResumeDirectDraw((HDEV)ppdev, 0);
// }
//
// VOID
// PDEVOBJ_vSuspendDirectDraw(
//     _Inout_ PPDEVOBJ ppdev)
// {
//     PGD_DXDDSUSPENDDIRECTDRAW pfnDdSuspendDirectDraw = (PGD_DXDDSUSPENDDIRECTDRAW)gpDxFuncs[DXG_INDEX_DxDdSuspendDirectDraw].pfn;
//
//     /* Suspend DirectDraw for mode change */
//     TRACE("DxDdSuspendDirectDraw(ppdev %p)\n", ppdev);
//     pfnDdSuspendDirectDraw((HDEV)ppdev, 0);
// }
//
// VOID
// PDEVOBJ_vSwitchDirectDraw(
//     _Inout_ PPDEVOBJ ppdev,
//     _Inout_ PPDEVOBJ ppdev2)
// {
//     PGD_DXDDDYNAMICMODECHANGE pfnDdDynamicModeChange = (PGD_DXDDDYNAMICMODECHANGE)gpDxFuncs[DXG_INDEX_DxDdDynamicModeChange].pfn;
//
//     /* Switch DirectDraw instances between the PDEVs */
//     TRACE("DxDdDynamicModeChange(ppdev %p, ppdev2 %p)\n", ppdev, ppdev2);
//     pfnDdDynamicModeChange((HDEV)ppdev, (HDEV)ppdev2, 0);
// }
//
// VOID
// PDEVOBJ_vEnableDisplay(
//     _Inout_ PPDEVOBJ ppdev)
// {
//     BOOL assertVal;
//
//     if (!(ppdev->flFlags & PDEV_DISABLED))
//         return;
//
//     /* Try to enable display until success */
//     do
//     {
//         TRACE("DrvAssertMode(dhpdev %p, TRUE)\n", ppdev->dhpdev);
//         assertVal = ppdev->pfn.AssertMode(ppdev->dhpdev, TRUE);
//         TRACE("DrvAssertMode(dhpdev %p, TRUE) => %d\n", ppdev->dhpdev, assertVal);
//     } while (!assertVal);
//
//     ppdev->flFlags &= ~PDEV_DISABLED;
// }
//
// BOOL
// PDEVOBJ_bDisableDisplay(
//     _Inout_ PPDEVOBJ ppdev)
// {
//     BOOL assertVal;
//
//     if (ppdev->flFlags & PDEV_DISABLED)
//         return TRUE;
//
//     PDEVOBJ_vSuspendDirectDraw(ppdev);
//
//     TRACE("DrvAssertMode(dhpdev %p, FALSE)\n", ppdev->dhpdev);
//     assertVal = ppdev->pfn.AssertMode(ppdev->dhpdev, FALSE);
//     TRACE("DrvAssertMode(dhpdev %p, FALSE) => %d\n", ppdev->dhpdev, assertVal);
//
//     if (assertVal)
//         ppdev->flFlags |= PDEV_DISABLED;
//
//     return assertVal;
// }
//
// VOID
// NTAPI
// PDEVOBJ_vRefreshModeList(
//     PPDEVOBJ ppdev)
// {
//     PGRAPHICS_DEVICE pGraphicsDevice;
//     PDEVMODEINFO pdminfo, pdmiNext;
//
//     /* Lock the PDEV */
//     EngAcquireSemaphore(ppdev->hsemDevLock);
//
//     pGraphicsDevice = ppdev->pGraphicsDevice;
//
//     /* Clear out the modes */
//     for (pdminfo = pGraphicsDevice->pdevmodeInfo;
//          pdminfo;
//          pdminfo = pdmiNext)
//     {
//         pdmiNext = pdminfo->pdmiNext;
//         ExFreePoolWithTag(pdminfo, GDITAG_DEVMODE);
//     }
//     pGraphicsDevice->pdevmodeInfo = NULL;
//     ExFreePoolWithTag(pGraphicsDevice->pDevModeList, GDITAG_GDEVICE);
//     pGraphicsDevice->pDevModeList = NULL;
//
//     /* Update available display mode list */
//     LDEVOBJ_bBuildDevmodeList(pGraphicsDevice);
//
//     /* Unlock PDEV */
//     EngReleaseSemaphore(ppdev->hsemDevLock);
// }
//
// PPDEVOBJ
// PDEVOBJ_Create(
//     _In_opt_ PGRAPHICS_DEVICE pGraphicsDevice,
//     _In_opt_ PDEVMODEW pdm,
//     _In_ ULONG dwAccelerationLevel,
//     _In_ ULONG ldevtype)
// {
//     PPDEVOBJ ppdev, ppdevMatch = NULL;
//     PLDEVOBJ pldev;
//     PSURFACE pSurface;
//
//     TRACE("PDEVOBJ_Create(%p %p %d)\n", pGraphicsDevice, pdm, ldevtype);
//
//     if (ldevtype != LDEV_DEVICE_META)
//     {
//         ASSERT(pGraphicsDevice);
//         ASSERT(pdm);
//         /* Search if we already have a PPDEV with the required characteristics.
//          * We will compare the graphics device, the devmode and the desktop
//          */
//         for (ppdev = gppdevList; ppdev; ppdev = ppdev->ppdevNext)
//         {
//             if (ppdev->pGraphicsDevice == pGraphicsDevice)
//             {
//                 PDEVOBJ_vReference(ppdev);
//
//                 if (RtlEqualMemory(pdm, ppdev->pdmwDev, sizeof(DEVMODEW)) &&
//                     ppdev->dwAccelerationLevel == dwAccelerationLevel)
//                 {
//                     PDEVOBJ_vReference(ppdev);
//                     ppdevMatch = ppdev;
//                 }
//                 else
//                 {
//                     PDEVOBJ_bDisableDisplay(ppdev);
//                 }
//
//                 PDEVOBJ_vRelease(ppdev);
//             }
//         }
//
//         if (ppdevMatch)
//         {
//             PDEVOBJ_vEnableDisplay(ppdevMatch);
//
//             return ppdevMatch;
//         }
//     }
//
//     /* Try to get a display driver */
//     if (ldevtype == LDEV_DEVICE_META)
//         pldev = LDEVOBJ_pLoadInternal(MultiEnableDriver, ldevtype);
//     else
//         pldev = LDEVOBJ_pLoadDriver(pdm->dmDeviceName, ldevtype);
//     if (!pldev)
//     {
//         ERR("Could not load display driver '%S'\n",
//              (ldevtype == LDEV_DEVICE_META) ? L"" : pdm->dmDeviceName);
//         return NULL;
//     }
//
//     /* Allocate a new PDEVOBJ */
//     ppdev = PDEVOBJ_AllocPDEV();
//     if (!ppdev)
//     {
//         ERR("failed to allocate a PDEV\n");
//         return NULL;
//     }
//
//     if (ldevtype != LDEV_DEVICE_META)
//     {
//         ppdev->pGraphicsDevice = pGraphicsDevice;
//
//         // DxEngGetHdevData asks for Graphics DeviceObject in hSpooler field
//         ppdev->hSpooler = ppdev->pGraphicsDevice->DeviceObject;
//
//         /* Keep selected resolution */
//         if (ppdev->pdmwDev)
//             ExFreePoolWithTag(ppdev->pdmwDev, GDITAG_DEVMODE);
//         ppdev->pdmwDev = ExAllocatePoolWithTag(PagedPool, pdm->dmSize + pdm->dmDriverExtra, GDITAG_DEVMODE);
//         if (ppdev->pdmwDev)
//         {
//             RtlCopyMemory(ppdev->pdmwDev, pdm, pdm->dmSize + pdm->dmDriverExtra);
//             /* FIXME: this must be done in a better way */
//             pGraphicsDevice->StateFlags |= DISPLAY_DEVICE_ATTACHED_TO_DESKTOP;
//         }
//     }
//
//     /* FIXME! */
//     ppdev->flFlags = PDEV_DISPLAY;
//
//     /* HACK: Don't use the pointer */
//     ppdev->Pointer.Exclude.right = -1;
//
//     /* Initialize PDEV */
//     ppdev->pldev = pldev;
//     ppdev->dwAccelerationLevel = dwAccelerationLevel;
//
//     /* Copy the function table */
//     if (ldevtype == LDEV_DEVICE_DISPLAY && (dwAccelerationLevel >= 5 ||
//         pdm->dmFields & (DM_PANNINGWIDTH | DM_PANNINGHEIGHT)))
//     {
//         ULONG i;
//
//         /* Initialize missing fields */
//         if (!(pdm->dmFields & DM_PANNINGWIDTH))
//             pdm->dmPanningWidth = pdm->dmPelsWidth;
//         if (!(pdm->dmFields & DM_PANNINGHEIGHT))
//             pdm->dmPanningHeight = pdm->dmPelsHeight;
//
//         /* Replace vtable by panning vtable */
//         for (i = 0; i < gPanDispDrvCount; i++)
//             ppdev->apfn[gPanDispDrvFn[i].iFunc] = gPanDispDrvFn[i].pfn;
//     }
//     else
//     {
//         ppdev->pfn = ppdev->pldev->pfn;
//     }
//
//     /* Call the driver to enable the PDEV */
//     if (!PDEVOBJ_bEnablePDEV(ppdev, pdm, NULL))
//     {
//         PDEVOBJ_vRelease(ppdev);
//         EngUnloadImage(pldev as HANDLE);
//         return NULL;
//     }
//
//     /* Tell the driver that the PDEV is ready */
//     PDEVOBJ_vCompletePDEV(ppdev);
//
//     /* Create the initial surface */
//     pSurface = PDEVOBJ_pSurface(ppdev);
//     if (!pSurface)
//     {
//         PDEVOBJ_vRelease(ppdev);
//         EngUnloadImage(pldev as HANDLE);
//         return NULL;
//     }
//
//     /* Enable DirectDraw */
//     if (!PDEVOBJ_bEnableDirectDraw(ppdev))
//     {
//         PDEVOBJ_vRelease(ppdev);
//         EngUnloadImage(pldev as HANDLE);
//         return NULL;
//     }
//
//     /* Remove some acceleration capabilities from driver */
//     PDEVOBJ_vFilterDriverHooks(ppdev);
//
//     /* Set MovePointer function */
//     ppdev->pfnMovePointer = ppdev->pfn.MovePointer;
//     if (!ppdev->pfnMovePointer)
//         ppdev->pfnMovePointer = EngMovePointer;
//
//     /* Insert the PDEV into the list */
//     ppdev->ppdevNext = gppdevList;
//     gppdevList = ppdev;
//
//     /* Return the PDEV */
//     return ppdev;
// }
//
// FORCEINLINE
// VOID
// SwitchPointer(
//     _Inout_ PVOID pvPointer1,
//     _Inout_ PVOID pvPointer2)
// {
//     PVOID *ppvPointer1 = pvPointer1;
//     PVOID *ppvPointer2 = pvPointer2;
//     PVOID pvTemp;
//
//     pvTemp = *ppvPointer1;
//     *ppvPointer1 = *ppvPointer2;
//     *ppvPointer2 = pvTemp;
// }
//
// BOOL
// NTAPI
// PDEVOBJ_bDynamicModeChange(
//     PPDEVOBJ ppdev,
//     PPDEVOBJ ppdev2)
// {
//     union
//     {
//         DRIVER_FUNCTIONS pfn;
//         GDIINFO gdiinfo;
//         DEVINFO devinfo;
//         DWORD StateFlags;
//     } temp;
//
//     /* Exchange driver functions */
//     temp.pfn = ppdev->pfn;
//     ppdev->pfn = ppdev2->pfn;
//     ppdev2->pfn = temp.pfn;
//
//     /* Exchange LDEVs */
//     SwitchPointer(&ppdev->pldev, &ppdev2->pldev);
//
//     /* Exchange DHPDEV */
//     SwitchPointer(&ppdev->dhpdev, &ppdev2->dhpdev);
//
//     /* Exchange surfaces and associate them with their new PDEV */
//     SwitchPointer(&ppdev->pSurface, &ppdev2->pSurface);
//     ppdev->pSurface->SurfObj.hdev = (HDEV)ppdev;
//     ppdev2->pSurface->SurfObj.hdev = (HDEV)ppdev2;
//
//     /* Exchange devinfo */
//     temp.devinfo = ppdev->devinfo;
//     ppdev->devinfo = ppdev2->devinfo;
//     ppdev2->devinfo = temp.devinfo;
//
//     /* Exchange gdiinfo */
//     temp.gdiinfo = ppdev->gdiinfo;
//     ppdev->gdiinfo = ppdev2->gdiinfo;
//     ppdev2->gdiinfo = temp.gdiinfo;
//
//     /* Exchange DEVMODE */
//     SwitchPointer(&ppdev->pdmwDev, &ppdev2->pdmwDev);
//
//     /* Exchange state flags */
//     temp.StateFlags = ppdev->pGraphicsDevice->StateFlags;
//     ppdev->pGraphicsDevice->StateFlags = ppdev2->pGraphicsDevice->StateFlags;
//     ppdev2->pGraphicsDevice->StateFlags = temp.StateFlags;
//
//     /* Notify each driver instance of its new HDEV association */
//     ppdev->pfn.CompletePDEV(ppdev->dhpdev, (HDEV)ppdev);
//     ppdev2->pfn.CompletePDEV(ppdev2->dhpdev, (HDEV)ppdev2);
//
//     /* Switch DirectDraw mode */
//     PDEVOBJ_vSwitchDirectDraw(ppdev, ppdev2);
//
//     return TRUE;
// }
//
//
// BOOL
// NTAPI
// PDEVOBJ_bSwitchMode(
//     PPDEVOBJ ppdev,
//     PDEVMODEW pdm)
// {
//     PPDEVOBJ ppdevTmp;
//     PSURFACE pSurface;
//     BOOL retval = FALSE;
//
//     /* Lock the PDEV */
//     EngAcquireSemaphore(ppdev->hsemDevLock);
//
//     /* And everything else */
//     EngAcquireSemaphore(ghsemPDEV);
//
//     DPRINT1("PDEVOBJ_bSwitchMode, ppdev = %p, pSurface = %p\n", ppdev, ppdev->pSurface);
//
//     // Lookup the GraphicsDevice + select DEVMODE
//     // pdm = LDEVOBJ_bProbeAndCaptureDevmode(ppdev, pdm);
//
//     /* 1. Temporarily disable the current PDEV and reset video to its default mode */
//     if (!PDEVOBJ_bDisableDisplay(ppdev))
//     {
//         DPRINT1("PDEVOBJ_bDisableDisplay() failed\n");
//         /* Resume DirectDraw in case of failure */
//         PDEVOBJ_vResumeDirectDraw(ppdev);
//         goto leave;
//     }
//
//     /* 2. Create new PDEV */
//     ppdevTmp = PDEVOBJ_Create(ppdev->pGraphicsDevice, pdm, 0, LDEV_DEVICE_DISPLAY);
//     if (!ppdevTmp)
//     {
//         DPRINT1("Failed to create a new PDEV\n");
//         goto leave2;
//     }
//
//     /* 3. Create a new surface */
//     pSurface = PDEVOBJ_pSurface(ppdevTmp);
//     if (!pSurface)
//     {
//         DPRINT1("PDEVOBJ_pSurface failed\n");
//         PDEVOBJ_vRelease(ppdevTmp);
//         goto leave2;
//     }
//
//     /* 4. Temporarily suspend DirectDraw for mode change */
//     PDEVOBJ_vSuspendDirectDraw(ppdev);
//     PDEVOBJ_vSuspendDirectDraw(ppdevTmp);
//
//     /* 5. Switch the PDEVs */
//     if (!PDEVOBJ_bDynamicModeChange(ppdev, ppdevTmp))
//     {
//         DPRINT1("PDEVOBJ_bDynamicModeChange() failed\n");
//         PDEVOBJ_vRelease(ppdevTmp);
//         goto leave2;
//     }
//
//     /* 6. Resume DirectDraw */
//     PDEVOBJ_vResumeDirectDraw(ppdev);
//     PDEVOBJ_vResumeDirectDraw(ppdevTmp);
//
//     /* Release temp PDEV */
//     PDEVOBJ_vRelease(ppdevTmp);
//
//     /* Re-initialize DirectDraw data */
//     ppdev->pEDDgpl->hDev = (HDEV)ppdev;
//     ppdev->pEDDgpl->dhpdev = ppdev->dhpdev;
//
//     /* Update primary display capabilities */
//     if (ppdev == gpmdev->ppdevGlobal)
//     {
//         PDEVOBJ_vGetDeviceCaps(ppdev, &GdiHandleTable->DevCaps);
//     }
//
//     /* Success! */
//     retval = TRUE;
//
// leave2:
//     /* Set the new video mode, or restore the original one in case of failure */
//     PDEVOBJ_vEnableDisplay(ppdev);
//
// leave:
//     /* Unlock everything else */
//     EngReleaseSemaphore(ghsemPDEV);
//     /* Unlock the PDEV */
//     EngReleaseSemaphore(ppdev->hsemDevLock);
//
//     DPRINT1("leave, ppdev = %p, pSurface = %p\n", ppdev, ppdev->pSurface);
//
//     return retval;
// }
//
//
// PPDEVOBJ
// NTAPI
// EngpGetPDEV(
//     _In_opt_ PUNICODE_STRING pustrDeviceName)
// {
//     UNICODE_STRING ustrCurrent;
//     PPDEVOBJ ppdev = NULL;
//     PGRAPHICS_DEVICE pGraphicsDevice;
//     ULONG i;
//
//     /* Acquire PDEV lock */
//     EngAcquireSemaphore(ghsemPDEV);
//
//     /* Did the caller pass a device name? */
//     if (pustrDeviceName)
//     {
//         /* Loop all present PDEVs */
//         for (i = 0; i < gpmdev->cDev; i++)
//         {
//             /* Get a pointer to the GRAPHICS_DEVICE */
//             pGraphicsDevice = gpmdev->dev[i].ppdev->pGraphicsDevice;
//
//             /* Compare the name */
//             RtlInitUnicodeString(&ustrCurrent, pGraphicsDevice->szWinDeviceName);
//             if (RtlEqualUnicodeString(pustrDeviceName, &ustrCurrent, FALSE))
//             {
//                 /* Found! */
//                 ppdev = gpmdev->dev[i].ppdev;
//                 break;
//             }
//         }
//     }
//     else if (gpmdev)
//     {
//         /* Otherwise use the primary PDEV */
//         ppdev = gpmdev->ppdevGlobal;
//     }
//
//     /* Did we find one? */
//     if (ppdev)
//     {
//         /* Yes, reference the PDEV */
//         PDEVOBJ_vReference(ppdev);
//     }
//
//     /* Release PDEV lock */
//     EngReleaseSemaphore(ghsemPDEV);
//
//     return ppdev;
// }
//
// LONG
// PDEVOBJ_lChangeDisplaySettings(
//     _In_opt_ PUNICODE_STRING pustrDeviceName,
//     _In_opt_ PDEVMODEW RequestedMode,
//     _In_opt_ PMDEVOBJ pmdevOld,
//     _Out_ PMDEVOBJ *ppmdevNew,
//     _In_ BOOL bSearchClosestMode)
// {
//     PGRAPHICS_DEVICE pGraphicsDevice = NULL;
//     PMDEVOBJ pmdev = NULL;
//     PDEVMODEW pdm = NULL;
//     ULONG lRet = DISP_CHANGE_SUCCESSFUL;
//     ULONG i, j;
//
//     TRACE("PDEVOBJ_lChangeDisplaySettings('%wZ' '%dx%dx%d (%d Hz)' %p %p)\n",
//         pustrDeviceName,
//         RequestedMode ? RequestedMode->dmPelsWidth : 0,
//         RequestedMode ? RequestedMode->dmPelsHeight : 0,
//         RequestedMode ? RequestedMode->dmBitsPerPel : 0,
//         RequestedMode ? RequestedMode->dmDisplayFrequency : 0,
//         pmdevOld, ppmdevNew);
//
//     if (pustrDeviceName)
//     {
//         pGraphicsDevice = EngpFindGraphicsDevice(pustrDeviceName, 0);
//         if (!pGraphicsDevice)
//         {
//             ERR("Wrong device name provided: '%wZ'\n", pustrDeviceName);
//             lRet = DISP_CHANGE_BADPARAM;
//             goto cleanup;
//         }
//     }
//     else if (RequestedMode)
//     {
//         pGraphicsDevice = EngpFindGraphicsDevice(NULL, 0);
//         if (!pGraphicsDevice)
//         {
//             ERR("Wrong device'\n");
//             lRet = DISP_CHANGE_BADPARAM;
//             goto cleanup;
//         }
//     }
//
//     if (pGraphicsDevice)
//     {
//         if (!LDEVOBJ_bProbeAndCaptureDevmode(pGraphicsDevice, RequestedMode, &pdm, bSearchClosestMode))
//         {
//             ERR("DrvProbeAndCaptureDevmode() failed\n");
//             lRet = DISP_CHANGE_BADMODE;
//             goto cleanup;
//         }
//     }
//
//     /* Here, we know that input parameters were correct */
//
//     {
//         /* Create new MDEV. Note that if we provide a device name,
//          * MDEV will only contain one device.
//          * */
//
//         if (pmdevOld)
//         {
//             /* Disable old MDEV */
//             if (MDEVOBJ_bDisable(pmdevOld))
//             {
//                 /* Create new MDEV. On failure, reenable old MDEV */
//                 pmdev = MDEVOBJ_Create(pustrDeviceName, pdm);
//                 if (!pmdev)
//                     MDEVOBJ_vEnable(pmdevOld);
//             }
//         }
//         else
//         {
//             pmdev = MDEVOBJ_Create(pustrDeviceName, pdm);
//         }
//
//         if (!pmdev)
//         {
//             ERR("Failed to create new MDEV\n");
//             lRet = DISP_CHANGE_FAILED;
//             goto cleanup;
//         }
//
//         lRet = DISP_CHANGE_SUCCESSFUL;
//         *ppmdevNew = pmdev;
//
//         /* We now have to do the mode switch */
//
//         if (pustrDeviceName && pmdevOld)
//         {
//             /* We changed settings of one device. Add other devices which were already present */
//             for (i = 0; i < pmdevOld->cDev; i++)
//             {
//                 for (j = 0; j < pmdev->cDev; j++)
//                 {
//                     if (pmdev->dev[j].ppdev->pGraphicsDevice == pmdevOld->dev[i].ppdev->pGraphicsDevice)
//                     {
//                         if (PDEVOBJ_bDynamicModeChange(pmdevOld->dev[i].ppdev, pmdev->dev[j].ppdev))
//                         {
//                             PPDEVOBJ tmp = pmdevOld->dev[i].ppdev;
//                             pmdevOld->dev[i].ppdev = pmdev->dev[j].ppdev;
//                             pmdev->dev[j].ppdev = tmp;
//                         }
//                         else
//                         {
//                             ERR("Failed to apply new settings\n");
//                             UNIMPLEMENTED;
//                             ASSERT(FALSE);
//                         }
//                         break;
//                     }
//                 }
//                 if (j == pmdev->cDev)
//                 {
//                     PDEVOBJ_vReference(pmdevOld->dev[i].ppdev);
//                     pmdev->dev[pmdev->cDev].ppdev = pmdevOld->dev[i].ppdev;
//                     pmdev->cDev++;
//                 }
//             }
//         }
//
//         if (pmdev->cDev == 1)
//         {
//             pmdev->ppdevGlobal = pmdev->dev[0].ppdev;
//         }
//         else
//         {
//             /* Enable MultiDriver */
//             pmdev->ppdevGlobal = PDEVOBJ_Create(NULL, (PDEVMODEW)pmdev, 0, LDEV_DEVICE_META);
//             if (!pmdev->ppdevGlobal)
//             {
//                 WARN("Failed to create meta-device. Using only first display\n");
//                 PDEVOBJ_vReference(pmdev->dev[0].ppdev);
//                 pmdev->ppdevGlobal = pmdev->dev[0].ppdev;
//             }
//         }
//
//         if (pmdevOld)
//         {
//             /* Search PDEVs which were in pmdevOld, but are not anymore in pmdev, and disable them */
//             for (i = 0; i < pmdevOld->cDev; i++)
//             {
//                 for (j = 0; j < pmdev->cDev; j++)
//                 {
//                     if (pmdev->dev[j].ppdev->pGraphicsDevice == pmdevOld->dev[i].ppdev->pGraphicsDevice)
//                         break;
//                 }
//                 if (j == pmdev->cDev)
//                     PDEVOBJ_bDisableDisplay(pmdevOld->dev[i].ppdev);
//             }
//         }
//     }
//
// cleanup:
//     if (lRet != DISP_CHANGE_SUCCESSFUL)
//     {
//         *ppmdevNew = NULL;
//         if (pmdev)
//             MDEVOBJ_vDestroy(pmdev);
//         if (pdm && pdm != RequestedMode)
//             ExFreePoolWithTag(pdm, GDITAG_DEVMODE);
//     }
//
//     return lRet;
// }
//
// INT
// NTAPI
// PDEVOBJ_iGetColorManagementCaps(PPDEVOBJ ppdev)
// {
//     INT ret = CM_NONE;
//
//     if (ppdev->flFlags & PDEV_DISPLAY)
//     {
//         if (ppdev->devinfo.iDitherFormat == BMF_8BPP ||
//             ppdev->devinfo.flGraphicsCaps2 & GCAPS2_CHANGEGAMMARAMP)
//             ret = CM_GAMMA_RAMP;
//     }
//
//     if (ppdev->devinfo.flGraphicsCaps & GCAPS_CMYKCOLOR)
//         ret |= CM_CMYK_COLOR;
//     if (ppdev->devinfo.flGraphicsCaps & GCAPS_ICM)
//         ret |= CM_DEVICE_ICM;
//
//     return ret;
// }
//
// VOID
// NTAPI
// PDEVOBJ_vGetDeviceCaps(
//     IN PPDEVOBJ ppdev,
//     OUT PDEVCAPS pDevCaps)
// {
//     PGDIINFO pGdiInfo = &ppdev->gdiinfo;
//
//     pDevCaps->ulVersion = pGdiInfo->ulVersion;
//     pDevCaps->ulTechnology = pGdiInfo->ulTechnology;
//     pDevCaps->ulHorzSizeM = (pGdiInfo->ulHorzSize + 500) / 1000;
//     pDevCaps->ulVertSizeM = (pGdiInfo->ulVertSize + 500) / 1000;
//     pDevCaps->ulHorzSize = pGdiInfo->ulHorzSize;
//     pDevCaps->ulVertSize = pGdiInfo->ulVertSize;
//     pDevCaps->ulHorzRes = pGdiInfo->ulHorzRes;
//     pDevCaps->ulVertRes = pGdiInfo->ulVertRes;
//     pDevCaps->ulBitsPixel = pGdiInfo->cBitsPixel;
//     if (pDevCaps->ulBitsPixel == 15) pDevCaps->ulBitsPixel = 16;
//     pDevCaps->ulPlanes = pGdiInfo->cPlanes;
//     pDevCaps->ulNumPens = pGdiInfo->ulNumColors;
//     if (pDevCaps->ulNumPens != -1) pDevCaps->ulNumPens *= 5;
//     pDevCaps->ulNumFonts = 0; // PDEVOBJ_cFonts(ppdev);
//     pDevCaps->ulNumColors = pGdiInfo->ulNumColors;
//     pDevCaps->ulRasterCaps = pGdiInfo->flRaster;
//     pDevCaps->ulAspectX = pGdiInfo->ulAspectX;
//     pDevCaps->ulAspectY = pGdiInfo->ulAspectY;
//     pDevCaps->ulAspectXY = pGdiInfo->ulAspectXY;
//     pDevCaps->ulLogPixelsX = pGdiInfo->ulLogPixelsX;
//     pDevCaps->ulLogPixelsY = pGdiInfo->ulLogPixelsY;
//     pDevCaps->ulSizePalette = pGdiInfo->ulNumPalReg;
//     pDevCaps->ulColorRes = pGdiInfo->ulDACRed +
//                            pGdiInfo->ulDACGreen +
//                            pGdiInfo->ulDACBlue;
//     pDevCaps->ulPhysicalWidth = pGdiInfo->szlPhysSize.cx;
//     pDevCaps->ulPhysicalHeight = pGdiInfo->szlPhysSize.cy;
//     pDevCaps->ulPhysicalOffsetX = pGdiInfo->ptlPhysOffset.x;
//     pDevCaps->ulPhysicalOffsetY = pGdiInfo->ptlPhysOffset.y;
//     pDevCaps->ulTextCaps = pGdiInfo->flTextCaps;
//     pDevCaps->ulTextCaps |= (TC_SO_ABLE|TC_UA_ABLE|TC_CP_STROKE|TC_OP_STROKE|TC_OP_CHARACTER);
//     if (pGdiInfo->ulTechnology != DT_PLOTTER)
//         pDevCaps->ulTextCaps |= TC_VA_ABLE;
//     pDevCaps->ulVRefresh = pGdiInfo->ulVRefresh;
//     pDevCaps->ulDesktopHorzRes = pGdiInfo->ulHorzRes;
//     pDevCaps->ulDesktopVertRes = pGdiInfo->ulVertRes;
//     pDevCaps->ulBltAlignment = pGdiInfo->ulBltAlignment;
//     pDevCaps->ulPanningHorzRes = pGdiInfo->ulPanningHorzRes;
//     pDevCaps->ulPanningVertRes = pGdiInfo->ulPanningVertRes;
//     pDevCaps->xPanningAlignment = pGdiInfo->xPanningAlignment;
//     pDevCaps->yPanningAlignment = pGdiInfo->yPanningAlignment;
//     pDevCaps->ulShadeBlend = pGdiInfo->flShadeBlend;
//     pDevCaps->ulColorMgmtCaps = PDEVOBJ_iGetColorManagementCaps(ppdev);
// }
//
//
// /** Exported functions ********************************************************/
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// EngQueryDeviceAttribute(
//     _In_ HDEV hdev,
//     _In_ ENG_DEVICE_ATTRIBUTE devAttr,
//     _In_reads_bytes_(cjInSize) PVOID pvIn,
//     _In_ ULONG cjInSize,
//     _Out_writes_bytes_(cjOutSize) PVOID pvOut,
//     _In_ ULONG cjOutSize)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)hdev;
//
//     if (devAttr != QDA_ACCELERATION_LEVEL)
//         return FALSE;
//
//     if (cjOutSize >= sizeof(DWORD))
//     {
//         /* Set all Accelerations Level Key to enabled Full 0 to 5 turned off. */
//         *(DWORD*)pvOut = ppdev->dwAccelerationLevel;
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// _Must_inspect_result_ _Ret_z_
// LPWSTR
// APIENTRY
// EngGetDriverName(_In_ HDEV hdev)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)hdev;
//
//     ASSERT(ppdev);
//     ASSERT(ppdev->pldev);
//     ASSERT(ppdev->pldev->pGdiDriverInfo);
//     ASSERT(ppdev->pldev->pGdiDriverInfo->DriverName.Buffer);
//
//     return ppdev->pldev->pGdiDriverInfo->DriverName.Buffer;
// }
//
//
// INT
// APIENTRY
// NtGdiGetDeviceCaps(
//     HDC hdc,
//     INT Index)
// {
//     PDC pdc;
//     DEVCAPS devcaps;
//
//     /* Lock the given DC */
//     pdc = DC_LockDc(hdc);
//     if (!pdc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return 0;
//     }
//
//     /* Get the data */
//     PDEVOBJ_vGetDeviceCaps(pdc->ppdev, &devcaps);
//
//     /* Unlock the DC */
//     DC_UnlockDc(pdc);
//
//     /* Return capability */
//     switch (Index)
//     {
//         case DRIVERVERSION:
//             return devcaps.ulVersion;
//
//         case TECHNOLOGY:
//             return devcaps.ulTechnology;
//
//         case HORZSIZE:
//             return devcaps.ulHorzSize;
//
//         case VERTSIZE:
//             return devcaps.ulVertSize;
//
//         case HORZRES:
//             return devcaps.ulHorzRes;
//
//         case VERTRES:
//             return devcaps.ulVertRes;
//
//         case LOGPIXELSX:
//             return devcaps.ulLogPixelsX;
//
//         case LOGPIXELSY:
//             return devcaps.ulLogPixelsY;
//
//         case BITSPIXEL:
//             return devcaps.ulBitsPixel;
//
//         case PLANES:
//             return devcaps.ulPlanes;
//
//         case NUMBRUSHES:
//             return -1;
//
//         case NUMPENS:
//             return devcaps.ulNumPens;
//
//         case NUMFONTS:
//             return devcaps.ulNumFonts;
//
//         case NUMCOLORS:
//             return devcaps.ulNumColors;
//
//         case ASPECTX:
//             return devcaps.ulAspectX;
//
//         case ASPECTY:
//             return devcaps.ulAspectY;
//
//         case ASPECTXY:
//             return devcaps.ulAspectXY;
//
//         case CLIPCAPS:
//             return CP_RECTANGLE;
//
//         case SIZEPALETTE:
//             return devcaps.ulSizePalette;
//
//         case NUMRESERVED:
//             return 20;
//
//         case COLORRES:
//             return devcaps.ulColorRes;
//
//         case DESKTOPVERTRES:
//             return devcaps.ulVertRes;
//
//         case DESKTOPHORZRES:
//             return devcaps.ulHorzRes;
//
//         case BLTALIGNMENT:
//             return devcaps.ulBltAlignment;
//
//         case SHADEBLENDCAPS:
//             return devcaps.ulShadeBlend;
//
//         case COLORMGMTCAPS:
//             return devcaps.ulColorMgmtCaps;
//
//         case PHYSICALWIDTH:
//             return devcaps.ulPhysicalWidth;
//
//         case PHYSICALHEIGHT:
//             return devcaps.ulPhysicalHeight;
//
//         case PHYSICALOFFSETX:
//             return devcaps.ulPhysicalOffsetX;
//
//         case PHYSICALOFFSETY:
//             return devcaps.ulPhysicalOffsetY;
//
//         case VREFRESH:
//             return devcaps.ulVRefresh;
//
//         case RASTERCAPS:
//             return devcaps.ulRasterCaps;
//
//         case CURVECAPS:
//             return (CC_CIRCLES | CC_PIE | CC_CHORD | CC_ELLIPSES | CC_WIDE |
//                     CC_STYLED | CC_WIDESTYLED | CC_INTERIORS | CC_ROUNDRECT);
//
//         case LINECAPS:
//             return (LC_POLYLINE | LC_MARKER | LC_POLYMARKER | LC_WIDE |
//                     LC_STYLED | LC_WIDESTYLED | LC_INTERIORS);
//
//         case POLYGONALCAPS:
//             return (PC_POLYGON | PC_RECTANGLE | PC_WINDPOLYGON | PC_SCANLINE |
//                     PC_WIDE | PC_STYLED | PC_WIDESTYLED | PC_INTERIORS);
//
//         case TEXTCAPS:
//             return devcaps.ulTextCaps;
//
//         case CAPS1:
//         case PDEVICESIZE:
//         case SCALINGFACTORX:
//         case SCALINGFACTORY:
//         default:
//             return 0;
//     }
//
//     return 0;
// }
//
// _Success_(return!=FALSE)
// BOOL
// APIENTRY
// NtGdiGetDeviceCapsAll(
//     IN HDC hDC,
//     OUT PDEVCAPS pDevCaps)
// {
//     PDC pdc;
//     DEVCAPS devcaps;
//     BOOL bResult = TRUE;
//     NTSTATUS Status;
//
//     /* Lock the given DC */
//     pdc = DC_LockDc(hDC);
//     if (!pdc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     /* Get the data */
//     PDEVOBJ_vGetDeviceCaps(pdc->ppdev, &devcaps);
//
//     /* Unlock the DC */
//     DC_UnlockDc(pdc);
//
//     /* Copy data to caller */
//     Status = SafeProbeForWrite(pDevCaps, sizeof(DEVCAPS), 1);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastNtError(Status);
//         bResult = FALSE;
//     }
//     else
//     {
//         RtlCopyMemory(pDevCaps, &devcaps, sizeof(DEVCAPS));
//     }
//
//     return bResult;
// }
//
//
// DHPDEV
// APIENTRY
// NtGdiGetDhpdev(
//     IN HDEV hdev)
// {
//     PPDEVOBJ ppdev;
//     DHPDEV dhpdev = NULL;
//
//     /* Check parameter */
//     if (!hdev || (PCHAR)hdev < (PCHAR)MmSystemRangeStart)
//         return NULL;
//
//     /* Lock PDEV list */
//     EngAcquireSemaphoreShared(ghsemPDEV);
//
//     /* Walk through the list of PDEVs */
//     for (ppdev = gppdevList;  ppdev; ppdev = ppdev->ppdevNext)
//     {
//         /* Compare with the given HDEV */
//         if (ppdev == (PPDEVOBJ)hdev)
//         {
//             /* Found the PDEV! Get it's dhpdev and break */
//             dhpdev = ppdev->dhpdev;
//             break;
//         }
//     }
//
//     /* Unlock PDEV list */
//     EngReleaseSemaphore(ghsemPDEV);
//
//     return dhpdev;
// }
//
// PSIZEL
// FASTCALL
// PDEVOBJ_sizl(PPDEVOBJ ppdev, PSIZEL psizl)
// {
//     if (ppdev->flFlags & PDEV_META_DEVICE)
//     {
//         *psizl = ppdev->szlMetaRes;
//     }
//     else
//     {
//         psizl->cx = ppdev->gdiinfo.ulHorzRes;
//         psizl->cy = ppdev->gdiinfo.ulVertRes;
//     }
//     return psizl;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
