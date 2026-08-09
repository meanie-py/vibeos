use win32k_sys::win32k::*;
use core::ffi::c_char;
#[allow(unused_imports)]
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// `UNIMPLEMENTED` is the C macro:
//   #define UNIMPLEMENTED DbgPrint("(%s:%i) WIN32K: %s UNIMPLEMENTED\n",
//                                  __FILE__, __LINE__, __FUNCTION__ )
// Each translated stub expands it inline as a `DbgPrint(...)` call (see below).
// The `pub const UNIMPLEMENTED: () = ();` token below preserves one
// LLM-emitted bare `UNIMPLEMENTED;` statement (NtGdiEngDeletePath) without
// needing a textual rewrite.
pub const UNIMPLEMENTED: () = ();

// DbgPrint is cdecl variadic (ntddk @ varargs DbgPrint). win32k-sys does
// not re-export it (bindgen scope), so declare it directly here.
extern "C" {
    fn DbgPrint(Format: *const c_char, ...) -> u32;
}

// STATUS_NOT_IMPLEMENTED is not in win32k-sys bindgen (lives in ntstatus.h);
// hardcode the canonical value. NTSTATUS is `LONG` (signed), so the bit
// pattern 0xC0000002 (STATUS_NOT_IMPLEMENTED, severity=ERROR) is cast.
const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC0000002u32 as NTSTATUS;
#[no_mangle]
pub unsafe extern "system" fn NtGdiUMPDEngFreeUserMem(
    ppv: *mut KERNEL_PVOID,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        file!().as_ptr() as *const i8,
        line!() as i32,
        b"NtGdiUMPDEngFreeUserMem\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetPUMPDOBJ(
    humpd: HUMPD,
    bStoreID: BOOL,
    phumpd: *mut HUMPD,
    pbWOW64: *mut BOOL,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"umpdstubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiSetPUMPDOBJ\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiBRUSHOBJ_hGetColorTransform(
    pbo: *mut BRUSHOBJ
) -> HANDLE {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR, 
             b"umpdstubs.c\0".as_ptr() as PCHAR, 
             line!() as i32, 
             b"NtGdiBRUSHOBJ_hGetColorTransform\0".as_ptr() as PCHAR);
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiBRUSHOBJ_pvAllocRbrush(
    pbo: *mut BRUSHOBJ,
    cj: ULONG,
) -> PVOID {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"umpdstubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiBRUSHOBJ_pvAllocRbrush\0".as_ptr() as PCHAR,
    );
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiBRUSHOBJ_pvGetRbrush(
    pbo: *mut BRUSHOBJ
) -> PVOID {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR, file!().as_ptr() as PCHAR, line!() as i32, b"NtGdiBRUSHOBJ_pvGetRbrush\0".as_ptr() as PCHAR);
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiBRUSHOBJ_ulGetBrushColor(
    pbo: *mut BRUSHOBJ
) -> ULONG {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR, 
             b"umpdstubs.c\0".as_ptr() as PCHAR, 
             line!() as i32, 
             b"NtGdiBRUSHOBJ_ulGetBrushColor\0".as_ptr() as PCHAR);
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiBRUSHOBJ_DeleteRbrush(
    pbo: *mut BRUSHOBJ,
    pboB: *mut BRUSHOBJ,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"umpdstubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiBRUSHOBJ_DeleteRbrush\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCLIPOBJ_bEnum(
    pco: *mut CLIPOBJ,
    cj: ULONG,
    pul: *mut ULONG,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiCLIPOBJ_bEnum\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCLIPOBJ_cEnumStart(
    _pco: *mut CLIPOBJ,
    _bAll: BOOL,
    _iType: ULONG,
    _iDirection: ULONG,
    _cLimit: ULONG,
) -> ULONG {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiCLIPOBJ_cEnumStart\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCLIPOBJ_ppoGetPath(
    pco: *mut CLIPOBJ
) -> *mut PATHOBJ {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"umpdstubs.c\0".as_ptr() as PCHAR,
        0i32,
        b"NtGdiCLIPOBJ_ppoGetPath\0".as_ptr() as PCHAR
    );
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngAssociateSurface(
    hsurf: HSURF,
    hdev: HDEV,
    flHooks: FLONG,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        file!().as_ptr() as *const i8,
        line!() as i32,
        b"NtGdiEngAssociateSurface\0".as_ptr() as *const i8,
    );
    0i32
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngCheckAbort(
    _pso: *mut SURFOBJ
) -> i32 {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR, 
             file!().as_ptr() as PCHAR, 
             line!() as i32, 
             b"NtGdiEngCheckAbort\0".as_ptr() as PCHAR);
    0i32
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngCopyBits(
    psoDst: *mut SURFOBJ,
    psoSrc: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    prclDst: *mut RECTL,
    pptlSrc: *mut POINTL,
) -> u32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"umpdstubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiEngCopyBits\0".as_ptr() as PCHAR,
    );
    0u32
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngCreateBitmap(
    sizl: SIZEL,
    lWidth: LONG,
    iFormat: ULONG,
    fl: FLONG,
    pvBits: PVOID,
) -> HBITMAP {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"umpdstubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiEngCreateBitmap\0".as_ptr() as PCHAR,
    );
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngCreateClip() -> *mut CLIPOBJ {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"umpdstubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiEngCreateClip\0".as_ptr() as PCHAR,
    );
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngCreateDeviceBitmap(
    dhsurf: DHSURF,
    sizl: SIZEL,
    iFormatCompat: ULONG,
) -> HBITMAP {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiEngCreateDeviceBitmap\0".as_ptr() as PCHAR,
    );
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngCreateDeviceSurface(
    dhsurf: DHSURF,
    sizl: SIZEL,
    iFormatCompat: ULONG,
) -> HANDLE {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiEngCreateDeviceSurface\0".as_ptr() as PCHAR,
    );
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngDeleteClip(
    pco: *mut CLIPOBJ
) -> NTSTATUS {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR, file!().as_ptr() as PCHAR, line!() as i32, b"NtGdiEngDeleteClip\0".as_ptr() as PCHAR);
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngDeletePath(
    _ppo: *mut PATHOBJ
) -> NTSTATUS {
    UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngDeleteSurface(
    hsurf: HSURF
) -> BOOL {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR, file!().as_ptr() as PCHAR, line!() as i32, b"NtGdiEngDeleteSurface\0".as_ptr() as PCHAR);
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngEraseSurface(
    _pso: *mut SURFOBJ,
    _prcl: *mut RECTL,
    _iColor: u32,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"umpdstubs.c\0".as_ptr() as *const i8,
        line!() as i32,
        b"NtGdiEngEraseSurface\0".as_ptr() as *const i8,
    );
    0i32
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngFillPath(
    pso: *mut SURFOBJ,
    ppo: *mut PATHOBJ,
    pco: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    pptlBrushOrg: *mut POINTL,
    mix: MIX,
    flOptions: FLONG,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"umpdstubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiEngFillPath\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngGradientFill(
    psoDest: *mut SURFOBJ,
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
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiEngGradientFill\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngLineTo(
    _pso: *mut SURFOBJ,
    _pco: *mut CLIPOBJ,
    _pbo: *mut BRUSHOBJ,
    _x1: i32,
    _y1: i32,
    _x2: i32,
    _y2: i32,
    _prclBounds: *mut RECTL,
    _mix: MIX,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"umpdstubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiEngLineTo\0".as_ptr() as PCHAR,
    );
    0i32
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngMarkBandingSurface(
    hsurf: HSURF
) -> BOOL {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR, file!().as_ptr() as PCHAR, line!() as i32, b"NtGdiEngMarkBandingSurface\0".as_ptr() as PCHAR);
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngPaint(
    _pso: *mut SURFOBJ,
    _pco: *mut CLIPOBJ,
    _pbo: *mut BRUSHOBJ,
    _pptlBrushOrg: *mut POINTL,
    _mix: MIX,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"umpdstubs.c\0".as_ptr() as *const i8,
        line!() as i32,
        b"NtGdiEngPaint\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngPlgBlt(
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
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"umpdstubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiEngPlgBlt\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngStretchBltROP(
    psoTrg: *mut SURFOBJ,
    psoSrc: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    pca: *mut COLORADJUSTMENT,
    pptlBrushOrg: *mut POINTL,
    prclTrg: *mut RECTL,
    prclSrc: *mut RECTL,
    pptlMask: *mut POINTL,
    iMode: ULONG,
    pbo: *mut BRUSHOBJ,
    rop4: ROP4,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"umpdstubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiEngStretchBltROP\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngStrokePath(
    pso: *mut SURFOBJ,
    ppo: *mut PATHOBJ,
    pco: *mut CLIPOBJ,
    pxo: *mut XFORMOBJ,
    pbo: *mut BRUSHOBJ,
    pptlBrushOrg: *mut POINTL,
    plineattrs: *mut LINEATTRS,
    mix: MIX,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"umpdstubs.c\0".as_ptr() as *const i8,
        line!() as i32,
        b"NtGdiEngStrokePath\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngStrokeAndFillPath(
    pso: *mut SURFOBJ,
    ppo: *mut PATHOBJ,
    pco: *mut CLIPOBJ,
    pxo: *mut XFORMOBJ,
    pboStroke: *mut BRUSHOBJ,
    plineattrs: *mut LINEATTRS,
    pboFill: *mut BRUSHOBJ,
    pptlBrushOrg: *mut POINTL,
    mix: MIX,
    flOptions: FLONG,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"umpdstubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiEngStrokeAndFillPath\0".as_ptr() as PCHAR,
    );
    0i32
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngTextOut(
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
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"umpdstubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiEngTextOut\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngTransparentBlt(
    psoDst: *mut SURFOBJ,
    psoSrc: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    prclDst: *mut RECTL,
    prclSrc: *mut RECTL,
    iTransColor: u32,
    ulReserved: u32,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        file!().as_ptr() as *const i8,
        line!() as i32,
        b"NtGdiEngTransparentBlt\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiFONTOBJ_vGetInfo(
    pfo: *mut FONTOBJ,
    cjSize: u32,
    pfi: *mut FONTINFO,
) -> NTSTATUS {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"umpdstubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiFONTOBJ_vGetInfo\0".as_ptr() as PCHAR,
    );
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiFONTOBJ_pxoGetXform(
    pfo: *mut FONTOBJ
) -> *mut XFORMOBJ {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR, file!().as_ptr() as PCHAR, line!() as i32, b"NtGdiFONTOBJ_pxoGetXform\0".as_ptr() as PCHAR);
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiFONTOBJ_cGetGlyphs(
    pfo: *mut FONTOBJ,
    iMode: u32,
    cGlyph: u32,
    phg: *mut HGLYPH,
    ppvGlyph: *mut *mut core::ffi::c_void,
) -> u32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        file!().as_ptr() as *const i8,
        line!() as i32,
        b"NtGdiFONTOBJ_cGetGlyphs\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiFONTOBJ_pQueryGlyphAttrs(
    pfo: *mut FONTOBJ,
    iMode: ULONG
) -> PFD_GLYPHATTR {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR, file!().as_ptr() as PCHAR, line!() as i32, b"NtGdiFONTOBJ_pQueryGlyphAttrs\0".as_ptr() as PCHAR);
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiFONTOBJ_pvTrueTypeFontFile(
    pfo: *mut FONTOBJ,
    pcjFile: *mut ULONG,
) -> PVOID {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"umpdstubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiFONTOBJ_pvTrueTypeFontFile\0".as_ptr() as PCHAR,
    );
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiFONTOBJ_cGetAllGlyphHandles(
    pfo: *mut FONTOBJ,
    phg: *mut HGLYPH,
) -> ULONG {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiFONTOBJ_cGetAllGlyphHandles\0".as_ptr() as PCHAR,
    );
    0
}
// NtGdiEngComputeGlyphSet — APIENTRY (stdcall i386, cdecl elsewhere).
// @unimplemented in umpdstubs.c; NULL-returning stub.
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngComputeGlyphSet(
    _nCodePage: INT,
    _nFirstChar: INT,
    _cChars: INT,
) -> *mut FD_GLYPHSET {
    core::ptr::null_mut()
}

// NtGdiFONTOBJ_pifi — APIENTRY. @unimplemented.
#[no_mangle]
pub unsafe extern "system" fn NtGdiFONTOBJ_pifi(_pfo: *mut FONTOBJ) -> *mut IFIMETRICS {
    core::ptr::null_mut()
}

// NtGdiFONTOBJ_pfdg — APIENTRY. @unimplemented. Returns FD_GLYPHSET*.
#[no_mangle]
pub unsafe extern "system" fn NtGdiFONTOBJ_pfdg(_pfo: *mut FONTOBJ) -> *mut FD_GLYPHSET {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiHT_Get8BPPMaskPalette(
    pPaletteEntry: LPPALETTEENTRY,
    Use8BPPMaskPal: BOOL,
    CMYMask: BYTE,
    RedGamma: USHORT,
    GreenGamma: USHORT,
    BlueGamma: USHORT,
) -> ULONG {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiHT_Get8BPPMaskPalette\0".as_ptr() as PCHAR,
    );
    0u32
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiHT_Get8BPPFormatPalette(
    pPaletteEntry: LPPALETTEENTRY,
    RedGamma: u16,
    GreenGamma: u16,
    BlueGamma: u16,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"umpdstubs.c\0".as_ptr() as *const i8,
        line!() as i32,
        b"NtGdiHT_Get8BPPFormatPalette\0".as_ptr() as *const i8,
    );
    0i32
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiPATHOBJ_vGetBounds(
    ppo: *mut PATHOBJ,
    prectfx: PRECTFX,
) -> NTSTATUS {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiPATHOBJ_vGetBounds\0".as_ptr() as PCHAR,
    );
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiPATHOBJ_bEnum(
    ppo: *mut PATHOBJ,
    ppd: *mut PATHDATA,
) -> u32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiPATHOBJ_bEnum\0".as_ptr() as PCHAR,
    );
    0u32
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiPATHOBJ_vEnumStart(
    ppo: *mut PATHOBJ
) -> NTSTATUS {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR, file!().as_ptr() as PCHAR, line!() as i32, b"NtGdiPATHOBJ_vEnumStart\0".as_ptr() as PCHAR);
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiPATHOBJ_vEnumStartClipLines(
    ppo: *mut PATHOBJ,
    pco: *mut CLIPOBJ,
    pso: *mut SURFOBJ,
    pla: *mut LINEATTRS,
) -> NTSTATUS {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiPATHOBJ_vEnumStartClipLines\0".as_ptr() as PCHAR,
    );
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiPATHOBJ_bEnumClipLines(
    ppo: *mut PATHOBJ,
    cb: ULONG,
    pcl: *mut CLIPLINE,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"umpdstubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiPATHOBJ_bEnumClipLines\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSTROBJ_bEnum(
    pstro: *mut STROBJ,
    pc: *mut ULONG,
    ppgpos: *mut PGLYPHPOS,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"umpdstubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiSTROBJ_bEnum\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSTROBJ_bEnumPositionsOnly(
    pstro: *mut STROBJ,
    pc: *mut ULONG,
    ppgpos: *mut PGLYPHPOS,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"umpdstubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiSTROBJ_bEnumPositionsOnly\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSTROBJ_bGetAdvanceWidths(
    pstro: *mut STROBJ,
    iFirst: ULONG,
    c: ULONG,
    pptqD: *mut POINTQF,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiSTROBJ_bGetAdvanceWidths\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSTROBJ_vEnumStart(
    pstro: *mut STROBJ
) -> NTSTATUS {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR, 
             file!().as_ptr() as PCHAR, 
             line!() as i32, 
             b"NtGdiSTROBJ_vEnumStart\0".as_ptr() as PCHAR);
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSTROBJ_dwGetCodePage(
    pstro: *mut STROBJ
) -> u32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiSTROBJ_dwGetCodePage\0".as_ptr() as PCHAR
    );
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiXFORMOBJ_bApplyXform(
    pxo: *mut XFORMOBJ,
    iMode: ULONG,
    cPoints: ULONG,
    pptIn: PPOINTL,
    pptOut: PPOINTL,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"umpdstubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiXFORMOBJ_bApplyXform\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiXFORMOBJ_iGetXform(
    pxo: *mut XFORMOBJ,
    pxform: *mut XFORML,
) -> ULONG {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiXFORMOBJ_iGetXform\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiXLATEOBJ_cGetPalette(
    pxlo: *mut XLATEOBJ,
    iPal: ULONG,
    cPal: ULONG,
    pPal: *mut ULONG,
) -> ULONG {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"umpdstubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiXLATEOBJ_cGetPalette\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiXLATEOBJ_iXlate(
    pxlo: *mut XLATEOBJ,
    iColor: ULONG,
) -> ULONG {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiXLATEOBJ_iXlate\0".as_ptr() as PCHAR,
    );
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiXLATEOBJ_hGetColorTransform(
    pxlo: *mut XLATEOBJ
) -> HANDLE {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR, file!().as_ptr() as PCHAR, line!() as i32, b"NtGdiXLATEOBJ_hGetColorTransform\0".as_ptr() as PCHAR);
    0 as HANDLE
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include <win32k.h>
// #undef XFORMOBJ
//
// #define UNIMPLEMENTED DbgPrint("(%s:%i) WIN32K: %s UNIMPLEMENTED\n", __FILE__, __LINE__, __FUNCTION__ )
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiUMPDEngFreeUserMem(
//     _In_ KERNEL_PVOID *ppv)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiSetPUMPDOBJ(
//     _In_opt_ HUMPD humpd,
//     _In_ BOOL bStoreID,
//     _Inout_opt_ HUMPD *phumpd,
//     _Out_opt_ BOOL *pbWOW64)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// HANDLE
// APIENTRY
// NtGdiBRUSHOBJ_hGetColorTransform(
//     _In_ BRUSHOBJ *pbo)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// __kernel_entry
// PVOID
// APIENTRY
// NtGdiBRUSHOBJ_pvAllocRbrush(
//     _In_ BRUSHOBJ *pbo,
//     _In_ ULONG cj)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// __kernel_entry
// PVOID
// APIENTRY
// NtGdiBRUSHOBJ_pvGetRbrush(
//     _In_ BRUSHOBJ *pbo)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// __kernel_entry
// ULONG
// APIENTRY
// NtGdiBRUSHOBJ_ulGetBrushColor(
//     _In_ BRUSHOBJ *pbo)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiBRUSHOBJ_DeleteRbrush(
//     _In_opt_ BRUSHOBJ *pbo,
//     _In_opt_ BRUSHOBJ *pboB)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiCLIPOBJ_bEnum(
//     _In_ CLIPOBJ *pco,
//     _In_ ULONG cj,
//     _Out_writes_bytes_(cj) ULONG *pul)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// ULONG
// APIENTRY
// NtGdiCLIPOBJ_cEnumStart(
//     _In_ CLIPOBJ *pco,
//     _In_ BOOL bAll,
//     _In_ ULONG iType,
//     _In_ ULONG iDirection,
//     _In_ ULONG cLimit)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// __kernel_entry
// PATHOBJ*
// APIENTRY
// NtGdiCLIPOBJ_ppoGetPath(
//     _In_ CLIPOBJ *pco)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiEngAssociateSurface(
//     _In_ HSURF hsurf,
//     _In_ HDEV hdev,
//     _In_ FLONG flHooks)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiEngCheckAbort(
//     _In_ SURFOBJ *pso)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// FD_GLYPHSET*
// APIENTRY
// NtGdiEngComputeGlyphSet(
//     _In_ INT nCodePage,
//     _In_ INT nFirstChar,
//     _In_ INT cChars)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiEngCopyBits(
//     _In_ SURFOBJ *psoDst,
//     _In_ SURFOBJ *psoSrc,
//     _In_opt_ CLIPOBJ *pco,
//     _In_opt_ XLATEOBJ *pxlo,
//     _In_ RECTL *prclDst,
//     _In_ POINTL *pptlSrc)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// HBITMAP
// APIENTRY
// NtGdiEngCreateBitmap(
//     _In_ SIZEL sizl,
//     _In_ LONG lWidth,
//     _In_ ULONG iFormat,
//     _In_ FLONG fl,
//     _In_opt_ PVOID pvBits)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// __kernel_entry
// CLIPOBJ*
// APIENTRY
// NtGdiEngCreateClip(
//     VOID)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// __kernel_entry
// HBITMAP
// APIENTRY
// NtGdiEngCreateDeviceBitmap(
//     _In_ DHSURF dhsurf,
//     _In_ SIZEL sizl,
//     _In_ ULONG iFormatCompat)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// __kernel_entry
// HSURF
// APIENTRY
// NtGdiEngCreateDeviceSurface(
//     _In_ DHSURF dhsurf,
//     _In_ SIZEL sizl,
//     _In_ ULONG iFormatCompat)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// __kernel_entry
// NTSTATUS
// APIENTRY
// NtGdiEngDeleteClip(
//     _In_ CLIPOBJ*pco)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// __kernel_entry
// NTSTATUS
// APIENTRY
// NtGdiEngDeletePath(
//     _In_ PATHOBJ *ppo)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiEngDeleteSurface(
//     _In_ HSURF hsurf)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiEngEraseSurface(
//     _In_ SURFOBJ *pso,
//     _In_ RECTL *prcl,
//     _In_ ULONG iColor)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiEngFillPath(
//     _In_ SURFOBJ *pso,
//     _In_ PATHOBJ *ppo,
//     _In_ CLIPOBJ *pco,
//     _In_ BRUSHOBJ *pbo,
//     _In_ POINTL *pptlBrushOrg,
//     _In_ MIX mix,
//     _In_ FLONG flOptions)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiEngGradientFill(
//     _In_ SURFOBJ *psoDest,
//     _In_ CLIPOBJ *pco,
//     _In_ XLATEOBJ *pxlo,
//     _In_reads_(nVertex) TRIVERTEX *pVertex,
//     _In_ ULONG nVertex,
//     _In_ /* _In_reads_(nMesh) */ PVOID pMesh,
//     _In_ ULONG nMesh,
//     _In_ RECTL *prclExtents,
//     _In_ POINTL *pptlDitherOrg,
//     _In_ ULONG ulMode)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiEngLineTo(
//     _In_ SURFOBJ *pso,
//     _In_ CLIPOBJ *pco,
//     _In_ BRUSHOBJ *pbo,
//     _In_ LONG x1,
//     _In_ LONG y1,
//     _In_ LONG x2,
//     _In_ LONG y2,
//     _In_ RECTL *prclBounds,
//     _In_ MIX mix)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiEngMarkBandingSurface(
//     _In_ HSURF hsurf)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiEngPaint(
//     _In_ SURFOBJ *pso,
//     _In_ CLIPOBJ *pco,
//     _In_ BRUSHOBJ *pbo,
//     _In_ POINTL *pptlBrushOrg,
//     _In_ MIX mix)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiEngPlgBlt(
//     _In_ SURFOBJ *psoTrg,
//     _In_ SURFOBJ *psoSrc,
//     _In_opt_ SURFOBJ *psoMsk,
//     _In_ CLIPOBJ *pco,
//     _In_opt_ XLATEOBJ *pxlo,
//     _In_ COLORADJUSTMENT *pca,
//     _In_ POINTL *pptlBrushOrg,
//     _In_ POINTFIX *pptfx,
//     _In_ RECTL *prcl,
//     _In_opt_ POINTL *pptl,
//     _In_ ULONG iMode)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiEngStretchBltROP(
//     _In_ SURFOBJ *psoTrg,
//     _In_ SURFOBJ *psoSrc,
//     _In_ SURFOBJ *psoMask,
//     _In_ CLIPOBJ *pco,
//     _In_ XLATEOBJ *pxlo,
//     _In_ COLORADJUSTMENT *pca,
//     _In_ POINTL *pptlBrushOrg,
//     _In_ RECTL *prclTrg,
//     _In_ RECTL *prclSrc,
//     _In_ POINTL *pptlMask,
//     _In_ ULONG iMode,
//     _In_ BRUSHOBJ *pbo,
//     _In_ ROP4 rop4)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiEngStrokePath(
//     _In_ SURFOBJ *pso,
//     _In_ PATHOBJ *ppo,
//     _In_ CLIPOBJ *pco,
//     _In_ XFORMOBJ *pxo,
//     _In_ BRUSHOBJ *pbo,
//     _In_ POINTL *pptlBrushOrg,
//     _In_ LINEATTRS *plineattrs,
//     _In_ MIX mix)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiEngStrokeAndFillPath(
//     _In_ SURFOBJ *pso,
//     _In_ PATHOBJ *ppo,
//     _In_ CLIPOBJ *pco,IN XFORMOBJ *pxo,
//     _In_ BRUSHOBJ *pboStroke,
//     _In_ LINEATTRS *plineattrs,
//     _In_ BRUSHOBJ *pboFill,
//     _In_ POINTL *pptlBrushOrg,
//     _In_ MIX mix,
//     _In_ FLONG flOptions)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiEngTextOut(
//     _In_ SURFOBJ *pso,
//     _In_ STROBJ *pstro,
//     _In_ FONTOBJ *pfo,
//     _In_ CLIPOBJ *pco,
//     _In_ RECTL *prclExtra,
//     _In_ RECTL *prclOpaque,
//     _In_ BRUSHOBJ *pboFore,
//     _In_ BRUSHOBJ *pboOpaque,
//     _In_ POINTL *pptlOrg,
//     _In_ MIX mix)
// {
//      UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiEngTransparentBlt(
//     _In_ SURFOBJ *psoDst,
//     _In_ SURFOBJ *psoSrc,
//     _In_ CLIPOBJ *pco,
//     _In_ XLATEOBJ *pxlo,
//     _In_ RECTL *prclDst,
//     _In_ RECTL *prclSrc,
//     _In_ ULONG iTransColor,
//     _In_ ULONG ulReserved)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// NTSTATUS
// APIENTRY
// NtGdiFONTOBJ_vGetInfo(
//     _In_ FONTOBJ *pfo,
//     _In_ ULONG cjSize,
//     _Out_writes_bytes_(cjSize) FONTINFO *pfi)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// __kernel_entry
// XFORMOBJ*
// APIENTRY
// NtGdiFONTOBJ_pxoGetXform(
//     _In_ FONTOBJ *pfo)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// __kernel_entry
// ULONG
// APIENTRY
// NtGdiFONTOBJ_cGetGlyphs(
//     _In_ FONTOBJ *pfo,
//     _In_ ULONG iMode,
//     _In_ ULONG cGlyph,
//     _In_ HGLYPH *phg,
//     _At_((GLYPHDATA**)ppvGlyph, _Outptr_) PVOID *ppvGlyph)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// __kernel_entry
// IFIMETRICS*
// APIENTRY
// NtGdiFONTOBJ_pifi(
//     _In_ FONTOBJ *pfo)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// __kernel_entry
// FD_GLYPHSET*
// APIENTRY
// NtGdiFONTOBJ_pfdg(
//     _In_ FONTOBJ *pfo)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// __kernel_entry
// PFD_GLYPHATTR
// APIENTRY
// NtGdiFONTOBJ_pQueryGlyphAttrs(
//     _In_ FONTOBJ *pfo,
//     _In_ ULONG iMode)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// __kernel_entry
// PVOID
// APIENTRY
// NtGdiFONTOBJ_pvTrueTypeFontFile(
//     _In_ FONTOBJ *pfo,
//     _Out_ ULONG *pcjFile)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// __kernel_entry
// ULONG
// APIENTRY
// NtGdiFONTOBJ_cGetAllGlyphHandles(
//     _In_ FONTOBJ *pfo,
//     _Out_opt_ _Post_count_(return) HGLYPH *phg)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// __kernel_entry
// LONG
// APIENTRY
// NtGdiHT_Get8BPPMaskPalette(
//     _Out_opt_ _Post_count_(return) LPPALETTEENTRY pPaletteEntry,
//     _In_ BOOL Use8BPPMaskPal,
//     _In_ BYTE CMYMask,
//     _In_ USHORT RedGamma,
//     _In_ USHORT GreenGamma,
//     _In_ USHORT BlueGamma)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// LONG
// APIENTRY
// NtGdiHT_Get8BPPFormatPalette(
//     _Out_opt_ _Post_count_(return) LPPALETTEENTRY pPaletteEntry,
//     _In_ USHORT RedGamma,
//     _In_ USHORT GreenGamma,
//     _In_ USHORT BlueGamma)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// NTSTATUS
// APIENTRY
// NtGdiPATHOBJ_vGetBounds(
//     _In_ PATHOBJ *ppo,
//     _Out_ PRECTFX prectfx)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiPATHOBJ_bEnum(
//     _In_ PATHOBJ *ppo,
//     _Out_ PATHDATA *ppd)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// NTSTATUS
// APIENTRY
// NtGdiPATHOBJ_vEnumStart(
//     _In_ PATHOBJ *ppo)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// __kernel_entry
// NTSTATUS
// APIENTRY
// NtGdiPATHOBJ_vEnumStartClipLines(
//     _In_ PATHOBJ *ppo,
//     _In_ CLIPOBJ *pco,
//     _In_ SURFOBJ *pso,
//     _In_ LINEATTRS *pla)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiPATHOBJ_bEnumClipLines(
//     _In_ PATHOBJ *ppo,
//     _In_ ULONG cb,
//     _Out_writes_bytes_(cb) CLIPLINE *pcl)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiSTROBJ_bEnum(
//     _In_ STROBJ *pstro,
//     _Out_ ULONG *pc,
//     _Outptr_result_buffer_(*pc) PGLYPHPOS *ppgpos)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiSTROBJ_bEnumPositionsOnly(
//     _In_ STROBJ *pstro,
//     _Out_ ULONG *pc,
//     _Outptr_result_buffer_(*pc) PGLYPHPOS *ppgpos)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiSTROBJ_bGetAdvanceWidths(
//     _In_ STROBJ*pstro,
//     _In_ ULONG iFirst,
//     _In_ ULONG c,
//     _Out_writes_(c) POINTQF*pptqD)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// NTSTATUS
// APIENTRY
// NtGdiSTROBJ_vEnumStart(
//     _Inout_ STROBJ *pstro)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// __kernel_entry
// DWORD
// APIENTRY
// NtGdiSTROBJ_dwGetCodePage(
//     _In_ STROBJ *pstro)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiXFORMOBJ_bApplyXform(
//     _In_ XFORMOBJ *pxo,
//     _In_ ULONG iMode,
//     _In_ ULONG cPoints,
//     _In_reads_(cPoints) PPOINTL pptIn,
//     _Out_writes_(cPoints) PPOINTL pptOut)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// __kernel_entry
// ULONG
// APIENTRY
// NtGdiXFORMOBJ_iGetXform(
//     _In_ XFORMOBJ *pxo,
//     _Out_opt_ XFORML *pxform)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// __kernel_entry
// ULONG
// APIENTRY
// NtGdiXLATEOBJ_cGetPalette(
//     _In_ XLATEOBJ *pxlo,
//     _In_ ULONG iPal,
//     _In_ ULONG cPal,
//     _Out_writes_(cPal) ULONG *pPal)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// __kernel_entry
// ULONG
// APIENTRY
// NtGdiXLATEOBJ_iXlate(
//     _In_ XLATEOBJ *pxlo,
//     _In_ ULONG iColor)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// __kernel_entry
// HANDLE
// APIENTRY
// NtGdiXLATEOBJ_hGetColorTransform(
//     _In_ XLATEOBJ *pxlo)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// //NtGdiEngAlphaBlend
// //NtGdiEngUnlockSurface
// //NtGdiEngLockSurface
// //NtGdiEngBitBlt
// //NtGdiEngStretchBlt
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
