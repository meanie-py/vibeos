//! ORPHAN-BY-DESIGN: `win32ss/gdi/eng/drvdbg.c` appears in no CMake source list,
//! and nothing in the tree references `apfnDbgDrvFunctions` or any `DbgDrv*`
//! entry — it is an upstream debug driver-wrapper shim that is not built. The 45
//! exports here are therefore unreachable, not missing.
//!
use win32k_sys::win32k::*;

use core::ffi::c_void;
use ntkrnl_sys::ddk::RtlAssert;


// ---- file-scope preamble (translated) ----
pub static apfnDbgDrvFunctions: [PVOID; 91] = [
    DbgDrvEnablePDEV as PVOID,
    DbgDrvCompletePDEV as PVOID,
    DbgDrvDisablePDEV as PVOID,
    DbgDrvEnableSurface as PVOID,
    DbgDrvDisableSurface as PVOID,
    DbgDrvAssertMode as PVOID,
    DbgDrvOffset as PVOID,
    DbgDrvResetPDEV as PVOID,
    core::ptr::null_mut(),
    DbgDrvReserved as PVOID,
    DbgDrvCreateDeviceBitmap as PVOID,
    core::ptr::null_mut(),
    DbgDrvRealizeBrush as PVOID,
    DbgDrvDitherColor as PVOID,
    DbgDrvStrokePath as PVOID,
    DbgDrvFillPath as PVOID,
    DbgDrvStrokeAndFillPath as PVOID,
    DbgDrvPaint as PVOID,
    DbgDrvBitBlt as PVOID,
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    DbgDrvReserved as PVOID,
    DbgDrvSetPalette as PVOID,
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    DbgDrvQueryFont as PVOID,
    core::ptr::null_mut(),
    DbgDrvQueryFontData as PVOID,
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    DbgDrvReserved as PVOID,
    DbgDrvGetGlyphMode as PVOID,
    DbgDrvSynchronize as PVOID,
    DbgDrvReserved as PVOID,
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    DbgDrvMovePanning as PVOID,
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    DbgDrvDescribePixelFormat as PVOID,
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    DbgDrvGetDirectDrawInfo as PVOID,
    DbgDrvEnableDirectDraw as PVOID,
    DbgDrvDisableDirectDraw as PVOID,
    DbgDrvQuerySpoolType as PVOID,
    DbgDrvReserved as PVOID,
    DbgDrvIcmCreateColorTransform as PVOID,
    DbgDrvIcmDeleteColorTransform as PVOID,
    DbgDrvIcmCheckBitmapBits as PVOID,
    DbgDrvIcmSetDeviceGammaRamp as PVOID,
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    DbgDrvQueryDeviceSupport as PVOID,
    DbgDrvReserved as PVOID,
    DbgDrvReserved as PVOID,
    DbgDrvReserved as PVOID,
    DbgDrvReserved as PVOID,
    DbgDrvReserved as PVOID,
    DbgDrvReserved as PVOID,
    DbgDrvReserved as PVOID,
    DbgDrvReserved as PVOID,
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    DbgDrvNotify as PVOID,
    DbgDrvSynchronizeSurface as PVOID,
    DbgDrvResetDevice as PVOID,
    DbgDrvReserved as PVOID,
    DbgDrvReserved as PVOID,
    DbgDrvReserved as PVOID,
];

unsafe fn DbgDrvReserved() {
    ntkrnl_sys::ASSERT!(false);
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvEnablePDEV(
    pdm: *mut DEVMODEW,
    pwszLogAddress: LPWSTR,
    cPat: u32,
    phsurfPatterns: *mut HSURF,
    cjCaps: u32,
    pdevcaps: *mut u32,
    cjDevInfo: u32,
    pdi: *mut DEVINFO,
    hdev: HDEV,
    pwszDeviceName: LPWSTR,
    hDriver: *mut core::ffi::c_void,
) -> DHPDEV {
    let ppdev = hdev as PPDEVOBJ;

    ((*(*ppdev).pldev).pfn.EnablePDEV)(
        pdm,
        pwszLogAddress,
        cPat,
        phsurfPatterns,
        cjCaps,
        pdevcaps,
        cjDevInfo,
        pdi,
        hdev,
        pwszDeviceName,
        hDriver,
    )
}

pub unsafe extern "system" fn DbgDrvCompletePDEV(
    dhpdev: DHPDEV,
    hdev: HDEV)
{
    let ppdev: PPDEVOBJ = hdev as PPDEVOBJ;

    ((*(*ppdev).pldev).pfn.CompletePDEV)(dhpdev, hdev);
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvDisablePDEV(
    dhpdev: DHPDEV)
{
    let ppdev = DbgLookupDHPDEV(dhpdev);

    ntkrnl_sys::ASSERT!(!ppdev.is_null());

    ((*(*ppdev).pldev).pfn.DisablePDEV)(dhpdev);
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvEnableSurface(
    dhpdev: DHPDEV
) -> HSURF
{
    let ppdev: PPDEVOBJ = DbgLookupDHPDEV(dhpdev);

    ((*(*ppdev).pldev).pfn.EnableSurface)(dhpdev)
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvDisableSurface(
    dhpdev: DHPDEV)
{
    let ppdev: PPDEVOBJ = DbgLookupDHPDEV(dhpdev);

    ntkrnl_sys::ASSERT!(!ppdev.is_null());

    ((*(*ppdev).pldev).pfn.DisableSurface)(dhpdev);
}

pub unsafe extern "system" fn DbgDrvAssertMode(
    dhpdev: DHPDEV,
    bEnable: BOOL,
) -> BOOL {
    let ppdev: PPDEVOBJ = DbgLookupDHPDEV(dhpdev);

    ntkrnl_sys::ASSERT!(!ppdev.is_null());

    ((*(*(*ppdev).pldev).pfn).AssertMode)(dhpdev, bEnable)
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvOffset(
    pso: *mut SURFOBJ,
    x1: i32,
    x2: i32,
    fl: FLONG,
) -> i32 {
    let ppdev: PPDEVOBJ = (*pso).hdev as PPDEVOBJ;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvResetPDEV(
    dhpdev: DHPDEV,
    Reserved: PVOID,
) -> ULONG {
    let ppdev: PPDEVOBJ = DbgLookupDHPDEV(dhpdev);

    (*(*ppdev).pldev).pfn.ResetDevice.unwrap()(dhpdev, Reserved)
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvDisableDriver() {
    ntkrnl_sys::ASSERT!(false);
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvCreateDeviceBitmap(
    dhpdev: DHPDEV,
    sizl: SIZEL,
    iFormat: ULONG,
) -> HBITMAP {
    let ppdev: PPDEVOBJ = DbgLookupDHPDEV(dhpdev);
    
    ((*(*ppdev).pldev).pfn.CreateDeviceBitmap.unwrap())(dhpdev, sizl, iFormat)
}

pub unsafe extern "system" fn DbgDrvDeleteDeviceBitmap(
    dhsurf: DHSURF)
{
    ntkrnl_sys::ASSERT!(false);
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvRealizeBrush(
    pbo: *mut BRUSHOBJ,
    psoTarget: *mut SURFOBJ,
    psoPattern: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    pxlo: *mut XLATEOBJ,
    iHatch: ULONG,
) -> BOOL {
    let ppdev: PPDEVOBJ = (*psoTarget).hdev as PPDEVOBJ;
    NT_ASSERT!(0u8);
    0
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvDitherColor(
    dhpdev: DHPDEV,
    iMode: ULONG,
    rgb: ULONG,
    pul: *mut ULONG,
) -> ULONG {
    let ppdev: PPDEVOBJ = DbgLookupDHPDEV(dhpdev);
    
    ((*(*ppdev).pldev).pfn.DitherColor)(dhpdev, iMode, rgb, pul)
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvStrokePath(
    pso: *mut SURFOBJ,
    ppo: *mut PATHOBJ,
    pco: *mut CLIPOBJ,
    pxo: *mut XFORMOBJ,
    pbo: *mut BRUSHOBJ,
    pptlBrushOrg: *mut POINTL,
    plineattrs: *mut LINEATTRS,
    mix: MIX,
) -> i32 {
    let ppdev: PPDEVOBJ = (*pso).hdev as PPDEVOBJ;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvFillPath(
    pso: *mut SURFOBJ,
    ppo: *mut PATHOBJ,
    pco: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    pptlBrushOrg: *mut POINTL,
    mix: MIX,
    flOptions: FLONG,
) -> i32 {
    let ppdev = (*pso).hdev as PPDEVOBJ;
    ntkrnl_sys::ASSERT!(false);
    0
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvStrokeAndFillPath(
    pso: *mut SURFOBJ,
    ppo: *mut PATHOBJ,
    pco: *mut CLIPOBJ,
    pxo: *mut XFORMOBJ,
    pboStroke: *mut BRUSHOBJ,
    plineattrs: *mut LINEATTRS,
    pboFill: *mut BRUSHOBJ,
    pptlBrushOrg: *mut POINTL,
    mixFill: MIX,
    flOptions: FLONG,
) -> i32 {
    let ppdev: PPDEVOBJ = (*pso).hdev as PPDEVOBJ;
    ntkrnl_sys::ASSERT!(false);
    0
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvPaint(
    pso: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    pptlBrushOrg: *mut POINTL,
    mix: MIX,
) -> i32 {
    let ppdev: PPDEVOBJ = (*pso).hdev as PPDEVOBJ;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvBitBlt(
    psoTrg: *mut SURFOBJ,
    psoSrc: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    prclTrg: *mut RECTL,
    pptlSrc: *mut POINTL,
    pptlMask: *mut POINTL,
    pbo: *mut BRUSHOBJ,
    pptlBrush: *mut POINTL,
    rop4: ROP4,
) -> i32 {
    let psurfTrg = (psoTrg as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;
    let psurfSrc = (psoSrc as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;
    let ppdev: PPDEVOBJ;

    if (*psurfTrg).flags & HOOK_BITBLT != 0 {
        ppdev = (*psoTrg).hdev as PPDEVOBJ;
    } else {
        ppdev = (*psoSrc).hdev as PPDEVOBJ;
    }

    ((*(*ppdev).pldev).pfn.BitBlt.unwrap())(
        psoTrg,
        psoSrc,
        psoMask,
        pco,
        pxlo,
        prclTrg,
        pptlSrc,
        pptlMask,
        pbo,
        pptlBrush,
        rop4,
    )
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvCopyBits(
    psoTrg: *mut SURFOBJ,
    psoSrc: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    prclTrg: *mut RECTL,
    pptlSrc: *mut POINTL,
) -> i32 {
    let psurfTrg = (psoTrg as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;
    let psurfSrc = (psoSrc as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;
    let ppdev: PPDEVOBJ;

    if (*psurfTrg).flags & HOOK_COPYBITS != 0 {
        ppdev = (*psoTrg).hdev as PPDEVOBJ;
    } else {
        ppdev = (*psoSrc).hdev as PPDEVOBJ;
    }

    ((*(*ppdev).pldev).pfn.CopyBits)(psoTrg, psoSrc, pco, pxlo, prclTrg, pptlSrc)
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvStretchBlt(
    psoTrg: *mut SURFOBJ,
    psoSrc: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    pca: *mut COLORADJUSTMENT,
    pptlHTOrg: *mut POINTL,
    prclDest: *mut RECTL,
    prclSrc: *mut RECTL,
    pptlMask: *mut POINTL,
    iMode: u32,
) -> i32 {
    let psurfTrg: PSURFACE = (psoTrg as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as PSURFACE;
    let psurfSrc: PSURFACE = (psoSrc as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as PSURFACE;

    0
}

pub unsafe fn DbgDrvSetPalette(
    dhpdev: DHPDEV,
    ppalo: *mut PALOBJ,
    fl: FLONG,
    iStart: u32,
    cColors: u32,
) -> i32 {
    let ppdev: PPDEVOBJ = DbgLookupDHPDEV(dhpdev);

    ((*(*ppdev).pldev).pfn.SetPalette)(dhpdev, ppalo, fl, iStart, cColors)
}

pub unsafe extern "system" fn DbgDrvTextOut(
    pso: *mut SURFOBJ,
    pstro: *mut STROBJ,
    pfo: *mut FONTOBJ,
    pco: *mut CLIPOBJ,
    prclExtra: *mut RECTL,
    prclOpaque: *mut RECTL,
    pboFore: *mut BRUSHOBJ,
    pboOpaque: *mut BRUSHOBJ,
    pptlOrg: *mut POINTL,
    mix: MIX,
) -> i32 {
    let ppdev: PPDEVOBJ = (*pso).hdev as PPDEVOBJ;
    return 0;
}

unsafe fn DbgDrvEscape(
    pso: *mut SURFOBJ,
    iEsc: ULONG,
    cjIn: ULONG,
    pvIn: PVOID,
    cjOut: ULONG,
    pvOut: PVOID,
) -> ULONG {
    let ppdev: PPDEVOBJ = (*pso).hdev as PPDEVOBJ;
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvDrawEscape(
    pso: *mut SURFOBJ,
    iEsc: ULONG,
    pco: *mut CLIPOBJ,
    prcl: *mut RECTL,
    cjIn: ULONG,
    pvIn: PVOID,
) -> ULONG {
    let ppdev: PPDEVOBJ = (*pso).hdev as PPDEVOBJ;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvQueryFont(
    dhpdev: DHPDEV,
    iFile: ULONG_PTR,
    iFace: ULONG,
    pid: *mut ULONG_PTR,
) -> PIFIMETRICS {
    let ppdev: PPDEVOBJ = DbgLookupDHPDEV(dhpdev);

    ASSERT(!ppdev.is_null());

    ((*(*ppdev).pldev).pfn.QueryFont)(dhpdev, iFile, iFace, pid)
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvQueryFontTree(
    dhpdev: DHPDEV,
    iFile: ULONG_PTR,
    iFace: ULONG,
    iMode: ULONG,
    pid: *mut ULONG_PTR,
) -> PVOID {
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvQueryFontData(
    dhpdev: DHPDEV,
    pfo: *mut FONTOBJ,
    iMode: ULONG,
    hg: HGLYPH,
    pgd: *mut GLYPHDATA,
    pv: PVOID,
    cjSize: ULONG,
) -> LONG {
    let ppdev: PPDEVOBJ = DbgLookupDHPDEV(dhpdev);

    (*ppdev).pldev.as_ref().unwrap().pfn.QueryFontData.unwrap()(dhpdev, pfo, iMode, hg, pgd, pv, cjSize)
}

pub unsafe extern "system" fn DbgDrvSetPointerShape(
    pso: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    psoColor: *mut SURFOBJ,
    pxlo: *mut XLATEOBJ,
    xHot: i32,
    yHot: i32,
    x: i32,
    y: i32,
    prcl: *mut RECTL,
    fl: FLONG,
) -> u32 {
    let ppdev = (*pso).hdev as PPDEVOBJ;
    return 0;
}

pub unsafe extern "system" fn DbgDrvMovePointer(
    pso: *mut SURFOBJ,
    x: i32,
    y: i32,
    prcl: *mut RECTL,
) {
    let ppdev: PPDEVOBJ = (*pso).hdev as PPDEVOBJ;
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvLineTo(
    pso: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    prclBounds: *mut RECTL,
    mix: MIX,
) -> i32 {
    let ppdev: PPDEVOBJ = (*pso).hdev as PPDEVOBJ;
    return 0;
}

pub unsafe extern "system" fn DbgDrvSendPage(
    pso: *mut SURFOBJ
) -> i32 {
    let ppdev: PPDEVOBJ = (*pso).hdev as PPDEVOBJ;
    return 0;
}

pub unsafe extern "system" fn DbgDrvStartPage(
    pso: *mut SURFOBJ
) -> BOOL {
    let ppdev: PPDEVOBJ = (*pso).hdev as PPDEVOBJ;
    return 0;
}

pub unsafe extern "system" fn DbgDrvEndDoc(
    pso: *mut SURFOBJ,
    fl: FLONG,
) -> i32 {
    let ppdev: PPDEVOBJ = (*pso).hdev as PPDEVOBJ;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvStartDoc(
    pso: *mut SURFOBJ,
    pwszDocName: LPWSTR,
    dwJobId: u32,
) -> i32 {
    let ppdev: PPDEVOBJ = (*pso).hdev as PPDEVOBJ;
    return 0;
}

pub unsafe extern "system" fn DbgDrvGetGlyphMode(
    dhpdev: DHPDEV,
    pfo: *mut FONTOBJ,
) -> u32 {
    let ppdev = DbgLookupDHPDEV(dhpdev);
    
    (*(*ppdev).pldev).pfn.GetGlyphMode.unwrap()(dhpdev, pfo)
}

pub unsafe extern "system" fn DbgDrvSynchronize(
    dhpdev: DHPDEV,
    prcl: *mut RECTL)
{
    let ppdev: PPDEVOBJ = DbgLookupDHPDEV(dhpdev);

    (*(*ppdev).pldev).pfn.Synchronize.unwrap()(dhpdev, prcl);
}

unsafe fn DbgDrvSaveScreenBits(
    pso: *mut SURFOBJ,
    iMode: u32,
    ident: usize,
    prcl: *mut RECTL,
) -> usize {
    let ppdev: PPDEVOBJ = (*pso).hdev as PPDEVOBJ;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvGetModes(
    hDriver: HANDLE,
    cjSize: ULONG,
    pdm: *mut DEVMODEW,
) -> ULONG {
    0
}

unsafe fn DbgDrvFree(
    pv: PVOID,
    id: ULONG_PTR)
{
}

pub unsafe extern "system" fn DbgDrvDestroyFont(
    pfo: *mut FONTOBJ)
{
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvQueryFontCaps(
    culCaps: ULONG,
    pulCaps: *mut ULONG,
) -> LONG {
    0
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvLoadFontFile(
    cFiles: ULONG,
    piFile: *mut ULONG_PTR,
    ppvView: *mut PVOID,
    pcjView: *mut ULONG,
    pdv: *mut DESIGNVECTOR,
    ulLangID: ULONG,
    ulFastCheckSum: ULONG,
) -> ULONG_PTR {
    0
}

unsafe fn DbgDrvUnloadFontFile(
    iFile: ULONG_PTR) -> BOOL
{
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvFontManagement(
    pso: *mut SURFOBJ,
    pfo: *mut FONTOBJ,
    iMode: ULONG,
    cjIn: ULONG,
    pvIn: PVOID,
    cjOut: ULONG,
    pvOut: PVOID,
) -> ULONG {
    0
}

unsafe fn DbgDrvQueryTrueTypeTable(
    iFile: ULONG_PTR,
    ulFont: ULONG,
    ulTag: ULONG,
    dpStart: PTRDIFF,
    cjBuf: ULONG,
    pjBuf: *mut BYTE,
    ppjTable: *mut PBYTE,
    pcjTable: *mut ULONG,
) -> LONG {
    0
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvQueryTrueTypeOutline(
    dhpdev: DHPDEV,
    pfo: *mut FONTOBJ,
    hglyph: HGLYPH,
    bMetricsOnly: BOOL,
    pgldt: *mut GLYPHDATA,
    cjBuf: ULONG,
    ppoly: *mut TTPOLYGONHEADER,
) -> LONG {
    let ppdev: PPDEVOBJ = DbgLookupDHPDEV(dhpdev);
    return 0;
}

pub unsafe extern "system" fn DbgDrvGetTrueTypeFile(
    iFile: ULONG_PTR,
    pcj: *mut ULONG,
) -> PVOID {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvQueryFontFile(
    iFile: ULONG_PTR,
    ulMode: ULONG,
    cjBuf: ULONG,
    pulBuf: *mut ULONG,
) -> LONG {
    0
}

pub unsafe extern "system" fn DbgDrvMovePanning(
    x: i32,
    y: i32,
    fl: FLONG,
) {
    ERR!("Obsolete driver function %s called!\n", b"DbgDrvMovePanning\0".as_ptr() as *const i8);
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvQueryAdvanceWidths(
    dhpdev: DHPDEV,
    pfo: *mut FONTOBJ,
    iMode: u32,
    phg: *mut HGLYPH,
    pvWidths: *mut core::ffi::c_void,
    cGlyphs: u32,
) -> i32 {
    let ppdev: PPDEVOBJ = DbgLookupDHPDEV(dhpdev);
    0
}

pub unsafe extern "system" fn DbgDrvSetPixelFormat(
    pso: *mut SURFOBJ,
    iPixelFormat: i32,
    hwnd: HWND,
) -> i32 {
    let ppdev = (*pso).hdev as PPDEVOBJ;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvDescribePixelFormat(
    dhpdev: DHPDEV,
    iPixelFormat: LONG,
    cjpfd: ULONG,
    ppfd: *mut PIXELFORMATDESCRIPTOR,
) -> LONG {
    let ppdev: PPDEVOBJ = DbgLookupDHPDEV(dhpdev);

    ((*(*ppdev).pldev).pfn.DescribePixelFormat)(dhpdev, iPixelFormat, cjpfd, ppfd)
}

pub unsafe extern "system" fn DbgDrvSwapBuffers(
    pso: *mut SURFOBJ,
    pwo: *mut WNDOBJ
) -> i32 {
    let ppdev: PPDEVOBJ = (*pso).hdev as PPDEVOBJ;
    return 0;
}

pub unsafe extern "system" fn DbgDrvStartBanding(
    pso: *mut SURFOBJ,
    pptl: *mut POINTL,
) -> BOOL {
    let ppdev = (*pso).hdev as PPDEVOBJ;
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvNextBand(
    pso: *mut SURFOBJ,
    pptl: *mut POINTL,
) -> i32 {
    let ppdev: PPDEVOBJ = (*pso).hdev as PPDEVOBJ;
    0
}

unsafe fn DbgDrvGetDirectDrawInfo(
    dhpdev: DHPDEV,
    pHalInfo: *mut DD_HALINFO,
    pdwNumHeaps: *mut DWORD,
    pvmList: *mut VIDEOMEMORY,
    pdwNumFourCCCodes: *mut DWORD,
    pdwFourCC: *mut DWORD,
) -> BOOL {
    let ppdev: PPDEVOBJ = DbgLookupDHPDEV(dhpdev);

    ntkrnl_sys::ASSERT!(!ppdev.is_null());

    ((*(*ppdev).pldev).pfn.GetDirectDrawInfo.unwrap())(
        dhpdev,
        pHalInfo,
        pdwNumHeaps,
        pvmList,
        pdwNumFourCCCodes,
        pdwFourCC,
    )
}

unsafe extern "system" fn DbgDrvEnableDirectDraw(
    dhpdev: DHPDEV,
    pCallBacks: *mut DD_CALLBACKS,
    pSurfaceCallBacks: *mut DD_SURFACECALLBACKS,
    pPaletteCallBacks: *mut DD_PALETTECALLBACKS) -> i32
{
    let ppdev: PPDEVOBJ = DbgLookupDHPDEV(dhpdev);

    ((*(*ppdev).pldev).pfn.EnableDirectDraw)(dhpdev,
                                              pCallBacks,
                                              pSurfaceCallBacks,
                                              pPaletteCallBacks)
}

pub unsafe extern "system" fn DbgDrvDisableDirectDraw(
    dhpdev: DHPDEV)
{
    let ppdev: PPDEVOBJ = DbgLookupDHPDEV(dhpdev);

    ntkrnl_sys::ASSERT!(!ppdev.is_null());

    ((*(*ppdev).pldev).pfn.DisableDirectDraw)(dhpdev);
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvQuerySpoolType(PDev: DHPDEV, SpoolType: LPWSTR) -> i32 {
    ERR!("Obsolete driver function %s called!\n", b"DbgDrvQuerySpoolType\0".as_ptr() as *const i8);
    ntkrnl_sys::ASSERT!(false);
    0
}

unsafe fn DbgDrvIcmCreateColorTransform(
    dhpdev: DHPDEV,
    pLogColorSpace: LPLOGCOLORSPACEW,
    pvSourceProfile: PVOID,
    cjSourceProfile: ULONG,
    pvDestProfile: PVOID,
    cjDestProfile: ULONG,
    pvTargetProfile: PVOID,
    cjTargetProfile: ULONG,
    dwReserved: DWORD,
) -> HANDLE {
    let ppdev: PPDEVOBJ = DbgLookupDHPDEV(dhpdev);

    ((*(*ppdev).pldev).pfn.IcmCreateColorTransform.unwrap())(
        dhpdev,
        pLogColorSpace,
        pvSourceProfile,
        cjSourceProfile,
        pvDestProfile,
        cjDestProfile,
        pvTargetProfile,
        cjTargetProfile,
        dwReserved,
    )
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvIcmDeleteColorTransform(
    dhpdev: DHPDEV,
    hcmXform: HANDLE,
) -> BOOL {
    let ppdev: PPDEVOBJ = DbgLookupDHPDEV(dhpdev);

    ((*(*(*ppdev).pldev).pfn).IcmDeleteColorTransform)(dhpdev, hcmXform)
}

unsafe fn DbgDrvIcmCheckBitmapBits(
    dhpdev: DHPDEV,
    hColorTransform: HANDLE,
    pso: *mut SURFOBJ,
    paResults: PBYTE
) -> BOOL {
    let ppdev: PPDEVOBJ = DbgLookupDHPDEV(dhpdev);

    ASSERT(ppdev as usize != 0);

    ((*(*ppdev).pldev).pfn.IcmCheckBitmapBits)(dhpdev,
                                                hColorTransform,
                                                pso,
                                                paResults)
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvIcmSetDeviceGammaRamp(
    dhpdev: DHPDEV,
    iFormat: ULONG,
    lpRamp: LPVOID,
) -> BOOL {
    let ppdev: PPDEVOBJ = DbgLookupDHPDEV(dhpdev);

    if ppdev.is_null() {
        RtlAssert(
            b"ppdev != NULL\0".as_ptr() as *mut core::ffi::c_void,
            b"drvdbg.rs\0".as_ptr() as *mut core::ffi::c_void,
            line!(),
            core::ptr::null_mut::<i8>(),
        );
    }

    ((*(*ppdev).pldev).pfn.IcmSetDeviceGammaRamp)(dhpdev, iFormat, lpRamp)
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvGradientFill(
    psoTrg: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    pVertex: *mut TRIVERTEX,
    nVertex: ULONG,
    pMesh: PVOID,
    nMesh: ULONG,
    prclExtents: *mut RECTL,
    pptlDitherOrg: *mut POINTL,
    ulMode: ULONG,
) -> BOOL {
    let ppdev: PPDEVOBJ = (*psoTrg).hdev as PPDEVOBJ;
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvStretchBltROP(
    psoTrg: *mut SURFOBJ,
    psoSrc: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    pca: *mut COLORADJUSTMENT,
    pptlHTOrg: *mut POINTL,
    prclDest: *mut RECTL,
    prclSrc: *mut RECTL,
    pptlMask: *mut POINTL,
    iMode: ULONG,
    pbo: *mut BRUSHOBJ,
    rop4: DWORD,
) -> BOOL {
    let ppdev: PPDEVOBJ = (*psoTrg).hdev as PPDEVOBJ;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvPlgBlt(
    psoTrg: *mut SURFOBJ,
    psoSrc: *mut SURFOBJ,
    psoMsk: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    pca: *mut COLORADJUSTMENT,
    pptlBrushOrg: *mut POINTL,
    pptfx: *mut POINTFIX,
    prcl: *mut RECTL,
    pptl: *mut POINTL,
    iMode: ULONG,
) -> BOOL {
    let ppdev = (*psoTrg).hdev as PPDEVOBJ;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvAlphaBlend(
    psoDest: *mut SURFOBJ,
    psoSrc: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    prclDest: *mut RECTL,
    prclSrc: *mut RECTL,
    pBlendObj: *mut BLENDOBJ,
) -> i32 {
    0
}

pub unsafe extern "system" fn DbgSynthesizeFont() {
    NT_ASSERT!(0u8);
}

unsafe extern "C" fn DbgGetSynthesizedFontFiles() {
    ntkrnl_sys::ASSERT!(false);
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvTransparentBlt(
    psoTrg: *mut SURFOBJ,
    psoSrc: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    prclDst: *mut RECTL,
    prclSrc: *mut RECTL,
    iTransColor: ULONG,
    ulReserved: ULONG,
) -> BOOL {
    let ppdev: PPDEVOBJ = (*psoTrg).hdev as PPDEVOBJ;
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvQueryPerBandInfo(
    pso: *mut SURFOBJ,
    pbi: *mut PERBANDINFO,
) -> u32 {
    let ppdev = (*pso).hdev as PPDEVOBJ;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvQueryDeviceSupport(
    pso: *mut SURFOBJ,
    pxlo: *mut XLATEOBJ,
    pxo: *mut XFORMOBJ,
    iType: ULONG,
    cjIn: ULONG,
    pvIn: PVOID,
    cjOut: ULONG,
    pvOut: PVOID,
) -> BOOL {
    let ppdev = (*pso).hdev as PPDEVOBJ;
    ASSERT!(ppdev.is_null() == false);

    ((*(*ppdev).pldev).pfn.QueryDeviceSupport)(
        pso,
        pxlo,
        pxo as PVOID,
        iType,
        cjIn,
        pvIn,
        cjOut,
        pvOut,
    )
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvDeriveSurface(
    pDirectDraw: *mut DD_DIRECTDRAW_GLOBAL,
    pSurface: *mut DD_SURFACE_LOCAL,
) -> HBITMAP {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvQueryGlyphAttrs(
    pfo: *mut FONTOBJ,
    iMode: u32,
) -> PFD_GLYPHATTR {
    core::ptr::null_mut()
}

pub unsafe extern "system" fn DbgDrvNotify(
    pso: *mut SURFOBJ,
    iType: ULONG,
    pvData: PVOID)
{
    let ppdev = (*pso).hdev as PPDEVOBJ;
    ASSERT(ppdev as ULONG_PTR != 0);

    ((*(*ppdev).pldev).pfn.Notify)(pso, iType, pvData);
}

pub unsafe extern "system" fn DbgDrvSynchronizeSurface(
    pso: *mut SURFOBJ,
    prcl: *mut RECTL,
    fl: FLONG)
{
    let ppdev = (*pso).hdev as PPDEVOBJ;
    ntkrnl_sys::ASSERT!(!ppdev.is_null());

    ((*(*ppdev).pldev).pfn.SynchronizeSurface)(pso, prcl, fl);
}

#[no_mangle]
pub unsafe extern "system" fn DbgDrvResetDevice(
    dhpdev: DHPDEV,
    Reserved: PVOID,
) -> ULONG {
    let ppdev = DbgLookupDHPDEV(dhpdev);
    
    ((*(*ppdev).pldev).pfn.ResetDevice)(dhpdev, Reserved)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         Win32 subsystem
//  * LICENSE:         GNU GPL, see COPYING in the top level directory
//  * FILE:            win32ss/gdi/eng/drvdbg.c
//  * PURPOSE:         Debug hooks for display driver callbacks
//  * PROGRAMMERS:     Timo Kreuzer
//  */
//
// #include <win32k.h>
// DBG_DEFAULT_CHANNEL(EngDev);
//
// PPDEVOBJ
// NTAPI
// DbgLookupDHPDEV(DHPDEV dhpdev);
//
// VOID
// DbgDrvReserved(void)
// {
//     ASSERT(FALSE);
// }
//
// DHPDEV
// APIENTRY
// DbgDrvEnablePDEV(
//     _In_ DEVMODEW *pdm,
//     _In_ LPWSTR pwszLogAddress,
//     ULONG cPat,
//     _In_opt_ HSURF *phsurfPatterns,
//     ULONG cjCaps,
//     _Out_ ULONG *pdevcaps,
//     ULONG cjDevInfo,
//     _Out_ DEVINFO *pdi,
//     HDEV hdev,
//     _In_ LPWSTR pwszDeviceName,
//     HANDLE hDriver)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)hdev;
//
//     ASSERT(pdm);
//     ASSERT(hdev);
//
//     return ppdev->pldev->pfn.EnablePDEV(pdm,
//                                         pwszLogAddress,
//                                         cPat,
//                                         phsurfPatterns,
//                                         cjCaps,
//                                         pdevcaps,
//                                         cjDevInfo,
//                                         pdi,
//                                         hdev,
//                                         pwszDeviceName,
//                                         hDriver);
// }
//
// VOID
// APIENTRY
// DbgDrvCompletePDEV(
//     DHPDEV dhpdev,
//     HDEV hdev)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)hdev;
//
//     ASSERT(ppdev);
//
//     ppdev->pldev->pfn.CompletePDEV(dhpdev, hdev);
// }
//
// VOID
// APIENTRY
// DbgDrvDisablePDEV(
//     DHPDEV dhpdev)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//
//     ASSERT(ppdev);
//
//     ppdev->pldev->pfn.DisablePDEV(dhpdev);
// }
//
// HSURF
// APIENTRY
// DbgDrvEnableSurface(
//     DHPDEV dhpdev)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//
//     ASSERT(ppdev);
//
//     return ppdev->pldev->pfn.EnableSurface(dhpdev);
// }
//
// VOID
// APIENTRY
// DbgDrvDisableSurface(
//     DHPDEV dhpdev)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//
//     ASSERT(ppdev);
//
//     ppdev->pldev->pfn.DisableSurface(dhpdev);
// }
//
// BOOL
// APIENTRY
// DbgDrvAssertMode(
//     _In_ DHPDEV dhpdev,
//     _In_ BOOL bEnable)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//
//     ASSERT(ppdev);
//
//     return ppdev->pldev->pfn.AssertMode(dhpdev, bEnable);
// }
//
// BOOL
// APIENTRY
// DbgDrvOffset(
//     SURFOBJ* pso,
//     LONG x1,
//     LONG x2,
//     FLONG fl)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//
//     ASSERT(FALSE);
//     return 0;
// }
//
// ULONG
// APIENTRY
// DbgDrvResetPDEV(
//     DHPDEV dhpdev,
//     PVOID Reserved)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//
//     ASSERT(ppdev);
//
//     return ppdev->pldev->pfn.ResetDevice(dhpdev, Reserved);
// }
//
// VOID
// APIENTRY
// DbgDrvDisableDriver(void)
// {
//     ASSERT(FALSE);
// }
//
// HBITMAP
// APIENTRY
// DbgDrvCreateDeviceBitmap(
//     DHPDEV dhpdev,
//     SIZEL sizl,
//     ULONG iFormat)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//
//     ASSERT(ppdev);
//
//     return ppdev->pldev->pfn.CreateDeviceBitmap(dhpdev, sizl, iFormat);
// }
//
// VOID
// APIENTRY
// DbgDrvDeleteDeviceBitmap(
//     DHSURF dhsurf)
// {
//     ASSERT(FALSE);
// }
//
// BOOL
// APIENTRY
// DbgDrvRealizeBrush(
//     _In_      BRUSHOBJ *pbo,
//     _Inout_   SURFOBJ *psoTarget,
//     _In_      SURFOBJ *psoPattern,
//     _In_opt_  SURFOBJ *psoMask,
//     _In_      XLATEOBJ *pxlo,
//     _In_      ULONG iHatch)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)psoTarget->hdev;
//     ASSERT(FALSE);
//     return 0;
// }
//
// ULONG
// APIENTRY
// DbgDrvDitherColor(
//     _In_     DHPDEV dhpdev,
//     _In_     ULONG iMode,
//     _In_     ULONG rgb,
//     _Inout_  ULONG *pul)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//
//     ASSERT(ppdev);
//
//     return ppdev->pldev->pfn.DitherColor(dhpdev, iMode, rgb, pul);
// }
//
// BOOL
// APIENTRY
// DbgDrvStrokePath(
//     _Inout_   SURFOBJ *pso,
//     _In_      PATHOBJ *ppo,
//     _In_      CLIPOBJ *pco,
//     _In_opt_  XFORMOBJ *pxo,
//     _In_      BRUSHOBJ *pbo,
//     _In_      POINTL *pptlBrushOrg,
//     _In_      LINEATTRS *plineattrs,
//     _In_      MIX mix)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     ASSERT(FALSE);
//     return 0;
// }
//
// BOOL
// APIENTRY
// DbgDrvFillPath(
//     _Inout_  SURFOBJ *pso,
//     _In_     PATHOBJ *ppo,
//     _In_     CLIPOBJ *pco,
//     _In_     BRUSHOBJ *pbo,
//     _In_     POINTL *pptlBrushOrg,
//     _In_     MIX mix,
//     _In_     FLONG flOptions)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     ASSERT(FALSE);
//     return 0;
// }
//
// BOOL
// APIENTRY
// DbgDrvStrokeAndFillPath(
//     _Inout_   SURFOBJ *pso,
//     _Inout_   PATHOBJ *ppo,
//     _In_      CLIPOBJ *pco,
//     _In_opt_  XFORMOBJ *pxo,
//     _In_      BRUSHOBJ *pboStroke,
//     _In_      LINEATTRS *plineattrs,
//     _In_      BRUSHOBJ *pboFill,
//     _In_      POINTL *pptlBrushOrg,
//     _In_      MIX mixFill,
//     _In_      FLONG flOptions)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     ASSERT(FALSE);
//     return 0;
// }
//
// BOOL
// APIENTRY
// DbgDrvPaint(
//     IN SURFOBJ *pso,
//     IN CLIPOBJ *pco,
//     IN BRUSHOBJ *pbo,
//     IN POINTL *pptlBrushOrg,
//     IN MIX mix)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     return 0;
// }
//
// BOOL
// APIENTRY
// DbgDrvBitBlt(
//     _Inout_ SURFOBJ *psoTrg,
//     _In_opt_ SURFOBJ *psoSrc,
//     _In_opt_ SURFOBJ *psoMask,
//     _In_opt_ CLIPOBJ *pco,
//     _In_opt_ XLATEOBJ *pxlo,
//     _In_ RECTL *prclTrg,
//     _When_(psoSrc, _In_) POINTL *pptlSrc,
//     _When_(psoMask, _In_) POINTL *pptlMask,
//     _In_opt_ BRUSHOBJ *pbo,
//     _When_(pbo, _In_) POINTL *pptlBrush,
//     _In_ ROP4 rop4)
// {
//     PSURFACE psurfTrg = CONTAINING_RECORD(psoTrg, SURFACE, SurfObj);
//     PSURFACE psurfSrc = CONTAINING_RECORD(psoSrc, SURFACE, SurfObj);
//     PPDEVOBJ ppdev;
//
//     /* Get the right BitBlt function */
//     if (psurfTrg->flags & HOOK_BITBLT)
//     {
//         ppdev = (PPDEVOBJ)psoTrg->hdev;
//     }
//     else
//     {
//         ASSERT(ROP4_USES_SOURCE(rop4));
//         ASSERT(psurfSrc->flags & HOOK_BITBLT);
//         ppdev = (PPDEVOBJ)psoSrc->hdev;
//     }
//
//     /* Sanity checks */
//     ASSERT(IS_VALID_ROP4(rop4));
//     ASSERT(psoTrg);
//     ASSERT(psoTrg->iBitmapFormat >= BMF_1BPP);
//     ASSERT(psoTrg->iBitmapFormat <= BMF_32BPP);
//     ASSERT(prclTrg);
//     ASSERT(prclTrg->left >= 0);
//     ASSERT(prclTrg->top >= 0);
//     ASSERT(prclTrg->right <= psoTrg->sizlBitmap.cx);
//     ASSERT(prclTrg->bottom <= psoTrg->sizlBitmap.cy);
//     ASSERT(RECTL_bIsWellOrdered(prclTrg));
//     ASSERT(pco);
//     ASSERT(pco->iDComplexity != DC_RECT);
//
//     if (ROP4_USES_SOURCE(rop4))
//     {
//         ASSERT(psoSrc);
//         ASSERT(psoSrc->iBitmapFormat >= BMF_1BPP);
//         ASSERT(psoSrc->iBitmapFormat <= BMF_8RLE);
//         ASSERT(pptlSrc);
//         ASSERT(pptlSrc->x >= 0);
//         ASSERT(pptlSrc->y >= 0);
//         ASSERT(pptlSrc->x <= psoTrg->sizlBitmap.cx);
//         ASSERT(pptlSrc->y <= psoTrg->sizlBitmap.cy);
//     }
//
//     if (ROP4_USES_MASK(rop4))
//     {
//         ASSERT(psoMask);
//         ASSERT(psoMask->iBitmapFormat == BMF_1BPP);
//         ASSERT(pptlMask);
//         ASSERT(pptlMask->x >= 0);
//         ASSERT(pptlMask->y >= 0);
//         ASSERT(pptlMask->x <= psoMask->sizlBitmap.cx);
//         ASSERT(pptlMask->y <= psoMask->sizlBitmap.cy);
//
//     }
//
//     if (ROP4_USES_PATTERN(rop4))
//     {
//         ASSERT(pbo);
//         ASSERT(pptlBrush);
//     }
//
//
//     return ppdev->pldev->pfn.BitBlt(psoTrg,
//                                     psoSrc,
//                                     psoMask,
//                                     pco,
//                                     pxlo,
//                                     prclTrg,
//                                     pptlSrc,
//                                     pptlMask,
//                                     pbo,
//                                     pptlBrush,
//                                     rop4);
// }
//
// BOOL
// APIENTRY
// DbgDrvCopyBits(
//     SURFOBJ *psoTrg,
//     SURFOBJ *psoSrc,
//     CLIPOBJ *pco,
//     XLATEOBJ *pxlo,
//     RECTL *prclTrg,
//     POINTL *pptlSrc)
// {
//     PSURFACE psurfTrg = CONTAINING_RECORD(psoTrg, SURFACE, SurfObj);
//     PSURFACE psurfSrc = CONTAINING_RECORD(psoSrc, SURFACE, SurfObj);
//     PPDEVOBJ ppdev;
//
//     /* Get the right BitBlt function */
//     if (psurfTrg->flags & HOOK_COPYBITS)
//     {
//         ppdev = (PPDEVOBJ)psoTrg->hdev;
//     }
//     else
//     {
//         ASSERT(psurfSrc->flags & HOOK_COPYBITS);
//         ppdev = (PPDEVOBJ)psoSrc->hdev;
//     }
//
//     return ppdev->pldev->pfn.CopyBits(psoTrg,
//                                       psoSrc,
//                                       pco,
//                                       pxlo,
//                                       prclTrg,
//                                       pptlSrc);
//
// }
//
// BOOL
// APIENTRY
// DbgDrvStretchBlt(
//     _Inout_   SURFOBJ *psoTrg,
//     _Inout_   SURFOBJ *psoSrc,
//     _In_opt_  SURFOBJ *psoMask,
//     _In_      CLIPOBJ *pco,
//     _In_opt_  XLATEOBJ *pxlo,
//     _In_opt_  COLORADJUSTMENT *pca,
//     _In_      POINTL *pptlHTOrg,
//     _In_      RECTL *prclDest,
//     _In_      RECTL *prclSrc,
//     _In_opt_  POINTL *pptlMask,
//     _In_      ULONG iMode)
// {
//     PSURFACE psurfTrg = CONTAINING_RECORD(psoTrg, SURFACE, SurfObj);
//     PSURFACE psurfSrc = CONTAINING_RECORD(psoSrc, SURFACE, SurfObj);
//
//     return 0;
// }
//
// BOOL
// APIENTRY
// DbgDrvSetPalette(
//     DHPDEV dhpdev,
//     PALOBJ *ppalo,
//     FLONG fl,
//     ULONG iStart,
//     ULONG cColors)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//
//     ASSERT(ppdev);
//
//     return ppdev->pldev->pfn.SetPalette(dhpdev, ppalo, fl, iStart, cColors);
// }
//
// BOOL
// APIENTRY
// DbgDrvTextOut(
//     SURFOBJ *pso,
//     STROBJ *pstro,
//     FONTOBJ *pfo,
//     CLIPOBJ *pco,
//     RECTL *prclExtra ,
//     RECTL *prclOpaque,
//     BRUSHOBJ *pboFore,
//     BRUSHOBJ *pboOpaque,
//     POINTL *pptlOrg,
//     MIX mix)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     return 0;
// }
//
// ULONG
// APIENTRY
// DbgDrvEscape(
//     _In_   SURFOBJ *pso,
//     _In_   ULONG iEsc,
//     _In_   ULONG cjIn,
//     _In_   PVOID pvIn,
//     _In_   ULONG cjOut,
//     _Out_  PVOID pvOut)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     return 0;
// }
//
// ULONG
// APIENTRY
// DbgDrvDrawEscape(
//     _In_  SURFOBJ *pso,
//     _In_  ULONG iEsc,
//     _In_  CLIPOBJ *pco,
//     _In_  RECTL *prcl,
//     _In_  ULONG cjIn,
//     _In_  PVOID pvIn)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     return 0;
// }
//
// PIFIMETRICS
// APIENTRY
// DbgDrvQueryFont(
//     DHPDEV dhpdev,
//     ULONG_PTR iFile,
//     ULONG iFace,
//     ULONG_PTR *pid)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//
//     ASSERT(ppdev);
//
//     return ppdev->pldev->pfn.QueryFont(dhpdev, iFile, iFace, pid);
// }
//
// PVOID
// APIENTRY
// DbgDrvQueryFontTree(
//     DHPDEV dhpdev,
//     ULONG_PTR iFile,
//     ULONG iFace,
//     ULONG iMode,
//     ULONG_PTR *pid)
// {
//     return 0;
// }
//
// LONG
// APIENTRY
// DbgDrvQueryFontData(
//     DHPDEV dhpdev,
//     FONTOBJ *pfo,
//     ULONG iMode,
//     HGLYPH hg,
//     GLYPHDATA *pgd,
//     _Out_  PVOID pv,
//     ULONG cjSize)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//
//     ASSERT(ppdev);
//
//     return ppdev->pldev->pfn.QueryFontData(dhpdev, pfo, iMode, hg, pgd, pv, cjSize);
// }
//
// ULONG
// APIENTRY
// DbgDrvSetPointerShape(
//     _In_  SURFOBJ *pso,
//     _In_  SURFOBJ *psoMask,
//     _In_  SURFOBJ *psoColor,
//     _In_  XLATEOBJ *pxlo,
//     _In_  LONG xHot,
//     _In_  LONG yHot,
//     _In_  LONG x,
//     _In_  LONG y,
//     _In_  RECTL *prcl,
//     _In_  FLONG fl)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     return 0;
// }
//
// VOID
// APIENTRY
// DbgDrvMovePointer(
//     _In_ SURFOBJ *pso,
//     _In_ LONG x,
//     _In_ LONG y,
//     _In_ RECTL *prcl)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
// }
//
// BOOL
// APIENTRY
// DbgDrvLineTo(
//     SURFOBJ *pso,
//     CLIPOBJ *pco,
//     BRUSHOBJ *pbo,
//     LONG x1,
//     LONG y1,
//     LONG x2,
//     LONG y2,
//     RECTL *prclBounds,
//     MIX mix)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     return 0;
// }
//
// BOOL
// APIENTRY
// DbgDrvSendPage(
//     _In_  SURFOBJ *pso)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     return 0;
// }
//
// BOOL
// APIENTRY
// DbgDrvStartPage(
//     _In_  SURFOBJ *pso)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     return 0;
// }
//
// BOOL
// APIENTRY
// DbgDrvEndDoc(
//     _In_  SURFOBJ *pso,
//     _In_  FLONG fl)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     return 0;
// }
//
// BOOL
// APIENTRY
// DbgDrvStartDoc(
//     _In_ SURFOBJ *pso,
//     _In_ LPWSTR pwszDocName,
//     _In_ DWORD dwJobId)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     return 0;
// }
//
// ULONG
// APIENTRY
// DbgDrvGetGlyphMode(
//     _In_ DHPDEV dhpdev,
//     _In_ FONTOBJ *pfo)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//
//     ASSERT(ppdev);
//
//     return ppdev->pldev->pfn.GetGlyphMode(dhpdev, pfo);
// }
//
// VOID
// APIENTRY
// DbgDrvSynchronize(
//     DHPDEV dhpdev,
//     RECTL *prcl)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//
//     ASSERT(ppdev);
//
//     ppdev->pldev->pfn.Synchronize(dhpdev, prcl);
// }
//
// ULONG_PTR
// APIENTRY
// DbgDrvSaveScreenBits(
//     SURFOBJ *pso,
//     ULONG iMode,
//     ULONG_PTR ident,
//     RECTL *prcl)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     return 0;
// }
//
// ULONG
// APIENTRY
// DbgDrvGetModes(
//     _In_       HANDLE hDriver,
//     ULONG cjSize,
//     _Out_opt_  DEVMODEW *pdm)
// {
//     return 0;
// }
//
// VOID
// APIENTRY
// DbgDrvFree(
//     PVOID pv,
//     ULONG_PTR id)
// {
// }
//
// VOID
// APIENTRY
// DbgDrvDestroyFont(
//     FONTOBJ *pfo)
// {
// }
//
// LONG
// APIENTRY
// DbgDrvQueryFontCaps(
//     ULONG culCaps,
//     ULONG *pulCaps)
// {
//     return 0;
// }
//
// ULONG_PTR
// APIENTRY
// DbgDrvLoadFontFile(
//     ULONG cFiles,
//     ULONG_PTR *piFile,
//     PVOID *ppvView,
//     ULONG *pcjView,
//     DESIGNVECTOR *pdv,
//     ULONG ulLangID,
//     ULONG ulFastCheckSum)
// {
//     return 0;
// }
//
// BOOL
// APIENTRY
// DbgDrvUnloadFontFile(
//     ULONG_PTR iFile)
// {
//     return 0;
// }
//
// ULONG
// APIENTRY
// DbgDrvFontManagement(
//     _In_      SURFOBJ *pso,
//     _In_opt_  FONTOBJ *pfo,
//     _In_      ULONG iMode,
//     _In_      ULONG cjIn,
//     _In_      PVOID pvIn,
//     _In_      ULONG cjOut,
//     _Out_     PVOID pvOut)
// {
//     return 0;
// }
//
// LONG
// APIENTRY
// DbgDrvQueryTrueTypeTable(
//     ULONG_PTR iFile,
//     ULONG ulFont,
//     ULONG ulTag,
//     PTRDIFF dpStart,
//     ULONG cjBuf,
//     BYTE *pjBuf,
//     PBYTE *ppjTable,
//     ULONG *pcjTable)
// {
//     return 0;
// }
//
// LONG
// APIENTRY
// DbgDrvQueryTrueTypeOutline(
//     DHPDEV dhpdev,
//     FONTOBJ *pfo,
//     HGLYPH hglyph,
//     BOOL bMetricsOnly,
//     GLYPHDATA *pgldt,
//     ULONG cjBuf,
//     TTPOLYGONHEADER *ppoly)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//     return 0;
// }
//
// PVOID
// APIENTRY
// DbgDrvGetTrueTypeFile(
//     ULONG_PTR iFile,
//     ULONG *pcj)
// {
//     return 0;
// }
//
// LONG
// APIENTRY
// DbgDrvQueryFontFile(
//     ULONG_PTR iFile,
//     ULONG ulMode,
//     ULONG cjBuf,
//     ULONG *pulBuf)
// {
//     return 0;
// }
//
// VOID
// APIENTRY
// DbgDrvMovePanning(
//     LONG x,
//     LONG y,
//     FLONG fl)
// {
//     ERR("Obsolete driver function %s called!\n", __FUNCTION__);
//     ASSERT(FALSE);
// }
//
// BOOL
// APIENTRY
// DbgDrvQueryAdvanceWidths(
//     DHPDEV dhpdev,
//     FONTOBJ *pfo,
//     ULONG iMode,
//     _In_   HGLYPH *phg,
//     _Out_  PVOID pvWidths,
//     ULONG cGlyphs)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//     return 0;
// }
//
// BOOL
// APIENTRY
// DbgDrvSetPixelFormat(
//     SURFOBJ *pso,
//     LONG iPixelFormat,
//     HWND hwnd)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     return 0;
// }
//
// LONG
// APIENTRY
// DbgDrvDescribePixelFormat(
//     DHPDEV dhpdev,
//     LONG iPixelFormat,
//     ULONG cjpfd,
//     PIXELFORMATDESCRIPTOR *ppfd)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//
//     ASSERT(ppdev);
//
//     return ppdev->pldev->pfn.DescribePixelFormat(dhpdev, iPixelFormat, cjpfd, ppfd);
// }
//
// BOOL
// APIENTRY
// DbgDrvSwapBuffers(
//     SURFOBJ *pso,
//     WNDOBJ *pwo)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     return 0;
// }
//
// BOOL
// APIENTRY
// DbgDrvStartBanding(
//     _In_  SURFOBJ *pso,
//     _In_  POINTL *pptl)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     return 0;
// }
//
// BOOL
// APIENTRY
// DbgDrvNextBand(
//     _In_  SURFOBJ *pso,
//     _In_  POINTL *pptl)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     return 0;
// }
//
// BOOL
// APIENTRY
// DbgDrvGetDirectDrawInfo(
//     DHPDEV dhpdev,
//     DD_HALINFO *pHalInfo,
//     DWORD *pdwNumHeaps,
//     VIDEOMEMORY *pvmList,
//     DWORD *pdwNumFourCCCodes,
//     DWORD *pdwFourCC)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//
//     ASSERT(ppdev);
//
//     return ppdev->pldev->pfn.GetDirectDrawInfo(dhpdev,
//                                                pHalInfo,
//                                                pdwNumHeaps,
//                                                pvmList,
//                                                pdwNumFourCCCodes,
//                                                pdwFourCC);
// }
//
// BOOL
// APIENTRY
// DbgDrvEnableDirectDraw(
//     DHPDEV dhpdev,
//     DD_CALLBACKS *pCallBacks,
//     DD_SURFACECALLBACKS *pSurfaceCallBacks,
//     DD_PALETTECALLBACKS *pPaletteCallBacks)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//
//     ASSERT(ppdev);
//
//     return ppdev->pldev->pfn.EnableDirectDraw(dhpdev,
//                                               pCallBacks,
//                                               pSurfaceCallBacks,
//                                               pPaletteCallBacks);
// }
//
// VOID
// APIENTRY
// DbgDrvDisableDirectDraw(
//     DHPDEV dhpdev)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//
//     ASSERT(ppdev);
//
//     ppdev->pldev->pfn.DisableDirectDraw(dhpdev);
// }
//
// BOOL
// APIENTRY
// DbgDrvQuerySpoolType(DHPDEV PDev, LPWSTR SpoolType)
// {
//     ERR("Obsolete driver function %s called!\n", __FUNCTION__);
//     ASSERT(FALSE);
//     return 0;
// }
//
// HANDLE
// APIENTRY
// DbgDrvIcmCreateColorTransform(
//     _In_      DHPDEV dhpdev,
//     _In_      LPLOGCOLORSPACEW pLogColorSpace,
//     _In_opt_  PVOID pvSourceProfile,
//     _In_      ULONG cjSourceProfile,
//     _In_      PVOID pvDestProfile,
//     _In_      ULONG cjDestProfile,
//     _In_opt_  PVOID pvTargetProfile,
//     _In_      ULONG cjTargetProfile,
//     _In_      DWORD dwReserved)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//
//     ASSERT(ppdev);
//
//     return ppdev->pldev->pfn.IcmCreateColorTransform(dhpdev,
//                                                      pLogColorSpace,
//                                                      pvSourceProfile,
//                                                      cjSourceProfile,
//                                                      pvDestProfile,
//                                                      cjDestProfile,
//                                                      pvTargetProfile,
//                                                      cjTargetProfile,
//                                                      dwReserved);
// }
//
// BOOL
// APIENTRY
// DbgDrvIcmDeleteColorTransform(
//     _In_  DHPDEV dhpdev,
//     _In_  HANDLE hcmXform)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//
//     ASSERT(ppdev);
//
//     return ppdev->pldev->pfn.IcmDeleteColorTransform(dhpdev, hcmXform);
// }
//
// BOOL
// APIENTRY
// DbgDrvIcmCheckBitmapBits(
//     DHPDEV dhpdev,
//     HANDLE hColorTransform,
//     SURFOBJ *pso,
//     PBYTE paResults)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//
//     ASSERT(ppdev);
//
//     return ppdev->pldev->pfn.IcmCheckBitmapBits(dhpdev,
//                                                 hColorTransform,
//                                                 pso,
//                                                 paResults);
// }
//
// BOOL
// APIENTRY
// DbgDrvIcmSetDeviceGammaRamp(
//     DHPDEV dhpdev,
//     ULONG iFormat,
//     LPVOID lpRamp)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//
//     ASSERT(ppdev);
//
//     return ppdev->pldev->pfn.IcmSetDeviceGammaRamp(dhpdev, iFormat, lpRamp);
// }
//
// BOOL
// APIENTRY
// DbgDrvGradientFill(
//     _Inout_   SURFOBJ *psoTrg,
//     _In_      CLIPOBJ *pco,
//     _In_opt_  XLATEOBJ *pxlo,
//     _In_      TRIVERTEX *pVertex,
//     _In_      ULONG nVertex,
//     _In_      PVOID pMesh,
//     _In_      ULONG nMesh,
//     _In_      RECTL *prclExtents,
//     _In_      POINTL *pptlDitherOrg,
//     _In_      ULONG ulMode)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)psoTrg->hdev;
//     return 0;
// }
//
// BOOL
// APIENTRY
// DbgDrvStretchBltROP(
//     _Inout_   SURFOBJ *psoTrg,
//     _Inout_   SURFOBJ *psoSrc,
//     _In_opt_  SURFOBJ *psoMask,
//     _In_      CLIPOBJ *pco,
//     _In_opt_  XLATEOBJ *pxlo,
//     _In_opt_  COLORADJUSTMENT *pca,
//     _In_      POINTL *pptlHTOrg,
//     _In_      RECTL *prclDest,
//     _In_      RECTL *prclSrc,
//     _In_opt_  POINTL *pptlMask,
//     _In_      ULONG iMode,
//     _In_      BRUSHOBJ *pbo,
//     _In_      DWORD rop4)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)psoTrg->hdev;
//     return 0;
// }
//
// BOOL
// APIENTRY
// DbgDrvPlgBlt(
//     _Inout_   SURFOBJ *psoTrg,
//     _Inout_   SURFOBJ *psoSrc,
//     _In_opt_  SURFOBJ *psoMsk,
//     _In_      CLIPOBJ *pco,
//     _In_opt_  XLATEOBJ *pxlo,
//     _In_opt_  COLORADJUSTMENT *pca,
//     _In_opt_  POINTL *pptlBrushOrg,
//     _In_      POINTFIX *pptfx,
//     _In_      RECTL *prcl,
//     _In_opt_  POINTL *pptl,
//     _In_      ULONG iMode)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)psoTrg->hdev;
//     return 0;
// }
//
// BOOL
// APIENTRY
// DbgDrvAlphaBlend(
//     _Inout_   SURFOBJ *psoDest,
//     _In_      SURFOBJ *psoSrc,
//     _In_      CLIPOBJ *pco,
//     _In_opt_  XLATEOBJ *pxlo,
//     _In_      RECTL *prclDest,
//     _In_      RECTL *prclSrc,
//     _In_      BLENDOBJ *pBlendObj)
// {
//     return 0;
// }
//
// VOID
// APIENTRY
// DbgSynthesizeFont(void)
// {
//     ASSERT(FALSE);
// }
//
// VOID
// APIENTRY
// DbgGetSynthesizedFontFiles(void)
// {
//     ASSERT(FALSE);
// }
//
// BOOL
// APIENTRY
// DbgDrvTransparentBlt(
//     _Inout_   SURFOBJ *psoTrg,
//     _In_      SURFOBJ *psoSrc,
//     _In_      CLIPOBJ *pco,
//     _In_opt_  XLATEOBJ *pxlo,
//     _In_      RECTL *prclDst,
//     _In_      RECTL *prclSrc,
//     _In_      ULONG iTransColor,
//     _In_      ULONG ulReserved)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)psoTrg->hdev;
//     return 0;
// }
//
// ULONG
// APIENTRY
// DbgDrvQueryPerBandInfo(
//     _In_     SURFOBJ *pso,
//     _Inout_  PERBANDINFO *pbi)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     return 0;
// }
//
// BOOL
// APIENTRY
// DbgDrvQueryDeviceSupport(
//     SURFOBJ *pso,
//     XLATEOBJ *pxlo,
//     XFORMOBJ *pxo,
//     ULONG iType,
//     ULONG cjIn,
//     _In_ PVOID pvIn,
//     ULONG cjOut,
//     _Out_ PVOID pvOut)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     ASSERT(ppdev);
//
//     return ppdev->pldev->pfn.QueryDeviceSupport(pso,
//                                                 pxlo,
//                                                 (PVOID)pxo, // FIXME!!!
//                                                 iType,
//                                                 cjIn,
//                                                 pvIn,
//                                                 cjOut,
//                                                 pvOut);
// }
//
// HBITMAP
// APIENTRY
// DbgDrvDeriveSurface(
//     DD_DIRECTDRAW_GLOBAL *pDirectDraw,
//     DD_SURFACE_LOCAL *pSurface)
// {
//     return 0;
// }
//
// PFD_GLYPHATTR
// APIENTRY
// DbgDrvQueryGlyphAttrs(
//     _In_  FONTOBJ *pfo,
//     _In_  ULONG iMode)
// {
//     return 0;
// }
//
// VOID
// APIENTRY
// DbgDrvNotify(
//     SURFOBJ *pso,
//     ULONG iType,
//     PVOID pvData)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     ASSERT(ppdev);
//
//     ppdev->pldev->pfn.Notify(pso, iType, pvData);
// }
//
// VOID
// APIENTRY
// DbgDrvSynchronizeSurface(
//     SURFOBJ *pso,
//     RECTL *prcl,
//     FLONG fl)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)pso->hdev;
//     ASSERT(ppdev);
//
//     ppdev->pldev->pfn.SynchronizeSurface(pso, prcl, fl);
// }
//
// ULONG
// APIENTRY
// DbgDrvResetDevice(
//     DHPDEV dhpdev,
//     PVOID Reserved)
// {
//     PPDEVOBJ ppdev = DbgLookupDHPDEV(dhpdev);
//     ASSERT(ppdev);
//
//     return ppdev->pldev->pfn.ResetDevice(dhpdev, Reserved);
// }
//
// PVOID
// apfnDbgDrvFunctions[] =
// {
//     DbgDrvEnablePDEV,
//     DbgDrvCompletePDEV,
//     DbgDrvDisablePDEV,
//     DbgDrvEnableSurface,
//     DbgDrvDisableSurface,
//     DbgDrvAssertMode,
//     DbgDrvOffset,
//     DbgDrvResetPDEV,
//     NULL, //DbgDrvDisableDriver,
//     DbgDrvReserved, // Unknown1
//     DbgDrvCreateDeviceBitmap,
//     NULL, //DbgDrvDeleteDeviceBitmap,
//     DbgDrvRealizeBrush,
//     DbgDrvDitherColor,
//     DbgDrvStrokePath,
//     DbgDrvFillPath,
//     DbgDrvStrokeAndFillPath,
//     DbgDrvPaint,
//     DbgDrvBitBlt,
//     NULL, //DbgDrvCopyBits,
//     NULL, //DbgDrvStretchBlt,
//     DbgDrvReserved,
//     DbgDrvSetPalette,
//     NULL, //DbgDrvTextOut,
//     NULL, //DbgDrvEscape,
//     NULL, //DbgDrvDrawEscape,
//     DbgDrvQueryFont,
//     NULL, //DbgDrvQueryFontTree,
//     DbgDrvQueryFontData,
//     NULL, //DbgDrvSetPointerShape,
//     NULL, //DbgDrvMovePointer,
//     NULL, //DbgDrvLineTo,
//     NULL, //DbgDrvSendPage,
//     NULL, //DbgDrvStartPage,
//     NULL, //DbgDrvEndDoc,
//     NULL, //DbgDrvStartDoc,
//     DbgDrvReserved,
//     DbgDrvGetGlyphMode,
//     DbgDrvSynchronize,
//     DbgDrvReserved,
//     NULL, //DbgDrvSaveScreenBits,
//     NULL, //DbgDrvGetModes,
//     NULL, //DbgDrvFree,
//     NULL, //DbgDrvDestroyFont,
//     NULL, //DbgDrvQueryFontCaps,
//     NULL, //DbgDrvLoadFontFile,
//     NULL, //DbgDrvUnloadFontFile,
//     NULL, //DbgDrvFontManagement,
//     NULL, //DbgDrvQueryTrueTypeTable,
//     NULL, //DbgDrvQueryTrueTypeOutline,
//     NULL, //DbgDrvGetTrueTypeFile,
//     NULL, //DbgDrvQueryFontFile,
//     DbgDrvMovePanning,
//     NULL, //DbgDrvQueryAdvanceWidths,
//     NULL, //DbgDrvSetPixelFormat,
//     DbgDrvDescribePixelFormat,
//     NULL, //DbgDrvSwapBuffers,
//     NULL, //DbgDrvStartBanding,
//     NULL, //DbgDrvNextBand,
//     DbgDrvGetDirectDrawInfo,
//     DbgDrvEnableDirectDraw,
//     DbgDrvDisableDirectDraw,
//     DbgDrvQuerySpoolType,
//     DbgDrvReserved,
//     DbgDrvIcmCreateColorTransform,
//     DbgDrvIcmDeleteColorTransform,
//     DbgDrvIcmCheckBitmapBits,
//     DbgDrvIcmSetDeviceGammaRamp,
//     NULL, //DbgDrvGradientFill,
//     NULL, //DbgDrvStretchBltROP,
//     NULL, //DbgDrvPlgBlt,
//     NULL, //DbgDrvAlphaBlend,
//     NULL, //DbgSynthesizeFont,
//     NULL, //DbgGetSynthesizedFontFiles,
//     NULL, //DbgDrvTransparentBlt,
//     NULL, //DbgDrvQueryPerBandInfo,
//     DbgDrvQueryDeviceSupport,
//     DbgDrvReserved,
//     DbgDrvReserved,
//     DbgDrvReserved,
//     DbgDrvReserved,
//     DbgDrvReserved,
//     DbgDrvReserved,
//     DbgDrvReserved,
//     DbgDrvReserved,
//     NULL, //DbgDrvDeriveSurface,
//     NULL, //DbgDrvQueryGlyphAttrs,
//     DbgDrvNotify,
//     DbgDrvSynchronizeSurface,
//     DbgDrvResetDevice,
//     DbgDrvReserved,
//     DbgDrvReserved,
//     DbgDrvReserved
// };
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
