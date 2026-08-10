use win32k_sys::win32k::*;
use core::ffi::c_char;
#[allow(unused_imports)]
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// `UNIMPLEMENTED` is the C macro:
//   #define UNIMPLEMENTED DbgPrint("(%s:%i) WIN32K: %s UNIMPLEMENTED\n",
//                                  __FILE__, __LINE__, __FUNCTION__ )
// Each translated stub expands it inline as a `DbgPrint(...)` call (see
// umpdstubs.rs for the same pattern). The bare `pub const UNIMPLEMENTED: () = ();`
// keeps any stray bare `UNIMPLEMENTED;` statements compilable.
pub const UNIMPLEMENTED: () = ();

// DbgPrint is cdecl variadic. win32k-sys does not re-export it (bindgen
// scope), so declare it directly here.
extern "C" {
    fn DbgPrint(Format: *const c_char, ...) -> u32;
}

// STATUS_NOT_IMPLEMENTED is not in win32k-sys bindgen (lives in ntstatus.h);
// hardcode the canonical value. NTSTATUS is `LONG` (signed), so the bit
// pattern 0xC0000002 (STATUS_NOT_IMPLEMENTED, severity=ERROR) is cast.
const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC0000002u32 as NTSTATUS;
#[no_mangle]
pub unsafe extern "system" fn EngTextOut(
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
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"EngTextOut\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn EngCheckAbort(pso: *mut SURFOBJ) -> i32 {
    0
}
#[no_mangle]
pub unsafe extern "system" fn EngEnumForms(
    hPrinter: HANDLE,
    Level: DWORD,
    pForm: LPBYTE,
    cbBuf: DWORD,
    pcbNeeded: LPDWORD,
    pcReturned: LPDWORD,
) -> BOOL {
    EngSetLastError(ERROR_NOT_SUPPORTED);
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn EngFillPath(
    pso: *mut SURFOBJ,
    ppo: *mut PATHOBJ,
    pco: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    pptlBrushOrg: *mut POINTL,
    mix: MIX,
    flOptions: FLONG,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        0i32,
        b"EngFillPath\0".as_ptr() as PCHAR,
    );
    0i32
}
#[no_mangle]
pub unsafe extern "system" fn EngFindResource(
    h: HANDLE,
    iName: i32,
    iType: i32,
    pulSize: PULONG,
) -> PVOID {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"EngFindResource\0".as_ptr() as PCHAR,
    );
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn EngGetFileChangeTime(
    h: HANDLE,
    pChangeTime: *mut LARGE_INTEGER,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"EngGetFileChangeTime\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn EngGetFilePath(
    h: HANDLE,
    pDest: *mut [WCHAR; MAX_PATH as usize + 1],
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"EngGetFilePath\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn EngGetForm(
    hPrinter: HANDLE,
    pFormName: LPWSTR,
    Level: DWORD,
    pForm: LPBYTE,
    cbBuf: DWORD,
    pcbNeeded: LPDWORD,
) -> BOOL {
    EngSetLastError(ERROR_NOT_SUPPORTED);
    return FALSE as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn EngGetPrinter(
    hPrinter: HANDLE,
    dwLevel: DWORD,
    pPrinter: LPBYTE,
    cbBuf: DWORD,
    pcbNeeded: LPDWORD,
) -> BOOL {
    EngSetLastError(ERROR_NOT_SUPPORTED);
    0
}
#[no_mangle]
pub unsafe extern "system" fn EngGetPrinterData(
    hPrinter: HANDLE,
    pValueName: LPWSTR,
    pType: LPDWORD,
    pData: LPBYTE,
    nSize: DWORD,
    pcbNeeded: LPDWORD,
) -> DWORD {
    EngSetLastError(ERROR_NOT_SUPPORTED);
    0
}
#[no_mangle]
pub unsafe extern "system" fn EngGetPrinterDataFileName(hdev: HDEV) -> LPWSTR {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"EngGetPrinterDataFileName\0".as_ptr() as PCHAR,
    );
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn EngGetType1FontList(
    hdev: HDEV,
    pType1Buffer: *mut TYPE1_FONT,
    cjType1Buffer: u32,
    pulLocalFonts: *mut u32,
    pulRemoteFonts: *mut u32,
    pLastModified: *mut LARGE_INTEGER,
) -> i32 {
    EngSetLastError(ERROR_NOT_SUPPORTED);
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn EngMarkBandingSurface(hsurf: HSURF) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"stubs.c\0".as_ptr() as *const i8,
        line!() as i32,
        b"EngMarkBandingSurface\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn EngMultiByteToWideChar(
    CodePage: UINT,
    WideCharString: LPWSTR,
    BytesInWideCharString: INT,
    MultiByteString: LPSTR,
    BytesInMultiByteString: INT,
) -> INT {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCCH,
        file!().as_ptr() as PCCH,
        line!() as i32,
        b"EngMultiByteToWideChar\0".as_ptr() as PCCH,
    );
    0
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn EngQueryLocalTime(ptf: PENG_TIME_FIELDS) {
    // §5: kernel APIs require the ntkrnl_sys version of LARGE_INTEGER —
    // win32k_sys re-bindgens a distinct (but layout-identical) type.
    let mut liSystemTime: ntkrnl_sys::ddk::_LARGE_INTEGER = core::mem::zeroed();
    let mut liLocalTime: ntkrnl_sys::ddk::_LARGE_INTEGER = core::mem::zeroed();

    /* Query the system time */
    // i386: real ntoskrnl export (KeQuerySystemTime). amd64/arm64: read from
    // SharedUserData via the fixed VA cast (NOT the inline::ke wrapper,
    // which uses `extern { static SharedUserData; }` that doesn't link in
    // win32k.sys — SharedUserData is not in ntoskrnl.spec, it's the fixed
    // KI_USER_SHARED_DATA macro page).
    #[cfg(target_arch = "x86")]
    ntkrnl_sys::ddk::KeQuerySystemTime(&mut liSystemTime);
    #[cfg(not(target_arch = "x86"))]
    {
        let sud = ntkrnl_sys::mm::KI_USER_SHARED_DATA as usize as *const ntkrnl_sys::ddk::_KUSER_SHARED_DATA;
        loop {
            let high1 = core::ptr::read_volatile(core::ptr::addr_of!((*sud).SystemTime.High1Time));
            let low   = core::ptr::read_volatile(core::ptr::addr_of!((*sud).SystemTime.LowPart));
            let high2 = core::ptr::read_volatile(core::ptr::addr_of!((*sud).SystemTime.High2Time));
            if high1 == high2 { liSystemTime.QuadPart = ((high1 as i64) << 32) | (low as u32 as i64); break; }
        }
    }

    /* Convert it to local time */
    ntkrnl_sys::ddk::ExSystemTimeToLocalTime(&mut liSystemTime, &mut liLocalTime);

    /* Convert the local time into time fields
       (note that ENG_TIME_FIELDS is identical to TIME_FIELDS) */
    ntkrnl_sys::ddk::RtlTimeToTimeFields(
        &mut liLocalTime,
        ptf as *mut _ as *mut ntkrnl_sys::ddk::_TIME_FIELDS,
    );
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn EngQueryLocalTime(ptf: PENG_TIME_FIELDS) {
    let mut liSystemTime: ntkrnl_sys::ddk::_LARGE_INTEGER = core::mem::zeroed();
    let mut liLocalTime: ntkrnl_sys::ddk::_LARGE_INTEGER = core::mem::zeroed();

    // amd64/arm64: read SharedUserData->SystemTime via fixed-VA cast.
    // See the x86 cfg branch above for the rationale.
    {
        let sud = ntkrnl_sys::mm::KI_USER_SHARED_DATA as usize as *const ntkrnl_sys::ddk::_KUSER_SHARED_DATA;
        loop {
            let high1 = core::ptr::read_volatile(core::ptr::addr_of!((*sud).SystemTime.High1Time));
            let low   = core::ptr::read_volatile(core::ptr::addr_of!((*sud).SystemTime.LowPart));
            let high2 = core::ptr::read_volatile(core::ptr::addr_of!((*sud).SystemTime.High2Time));
            if high1 == high2 { liSystemTime.QuadPart = ((high1 as i64) << 32) | (low as u32 as i64); break; }
        }
    }

    ntkrnl_sys::ddk::ExSystemTimeToLocalTime(&mut liSystemTime, &mut liLocalTime);

    ntkrnl_sys::ddk::RtlTimeToTimeFields(
        &mut liLocalTime,
        ptf as *mut _ as *mut ntkrnl_sys::ddk::_TIME_FIELDS,
    );
}
#[no_mangle]
pub unsafe extern "system" fn EngQueryPalette(
    hPal: HPALETTE,
    piMode: *mut ULONG,
    cColors: ULONG,
    pulColors: *mut ULONG,
) -> ULONG {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"EngQueryPalette\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn EngSetPrinterData(
    hPrinter: *mut core::ffi::c_void,
    pType: LPWSTR,
    dwType: u32,
    lpbPrinterData: LPBYTE,
    cjPrinterData: u32,
) -> u32 {
    EngSetLastError(ERROR_NOT_SUPPORTED);
    0
}
#[no_mangle]
pub unsafe extern "system" fn EngStrokeAndFillPath(
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
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"EngStrokeAndFillPath\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn EngStrokePath(
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
        b"stubs.c\0".as_ptr() as *const i8,
        line!() as i32,
        b"EngStrokePath\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn EngWideCharToMultiByte(
    CodePage: UINT,
    WideCharString: LPWSTR,
    BytesInWideCharString: INT,
    MultiByteString: LPSTR,
    BytesInMultiByteString: INT,
) -> INT {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        0 as INT,
        b"EngWideCharToMultiByte\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn EngWritePrinter(
    hPrinter: HANDLE,
    pBuf: LPVOID,
    cbBuf: DWORD,
    pcWritten: LPDWORD,
) -> BOOL {
    EngSetLastError(ERROR_NOT_SUPPORTED);
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn FONTOBJ_cGetAllGlyphHandles(
    FontObj: *mut FONTOBJ,
    Glyphs: *mut HGLYPH,
) -> ULONG {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"FONTOBJ_cGetAllGlyphHandles\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn FONTOBJ_cGetGlyphs(
    FontObj: *mut FONTOBJ,
    Mode: u32,
    NumGlyphs: u32,
    GlyphHandles: *mut HGLYPH,
    OutGlyphs: *mut *mut core::ffi::c_void,
) -> u32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"FONTOBJ_cGetGlyphs\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn FONTOBJ_pvTrueTypeFontFile(
    pfo: *mut FONTOBJ,
    pcjFile: *mut ULONG,
) -> PVOID {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"FONTOBJ_pvTrueTypeFontFile\0".as_ptr() as PCHAR,
    );
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn FONTOBJ_vGetInfo(
    FontObj: *mut FONTOBJ,
    InfoSize: u32,
    FontInfo: PFONTINFO,
) {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"stubs.c\0".as_ptr() as *const i8,
        line!() as i32,
        b"FONTOBJ_vGetInfo\0".as_ptr() as *const i8,
    );
}
#[no_mangle]
pub unsafe extern "system" fn HT_ComputeRGBGammaTable(
    GammaTableEntries: USHORT,
    GammaTableType: USHORT,
    RedGamma: USHORT,
    GreenGamma: USHORT,
    BlueGamma: USHORT,
    pGammaTable: LPBYTE,
) -> LONG {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        0 as INT,
        b"HT_ComputeRGBGammaTable\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn HT_Get8BPPFormatPalette(
    pPaletteEntry: LPPALETTEENTRY,
    RedGamma: u16,
    GreenGamma: u16,
    BlueGamma: u16,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"stubs.c\0".as_ptr() as *const i8,
        line!() as i32,
        b"HT_Get8BPPFormatPalette\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn EngDitherColor(
    hdev: HDEV,
    iMode: ULONG,
    rgb: ULONG,
    pul: *mut ULONG,
) -> ULONG {
    *pul = 0;
    return DCR_SOLID;
}
#[no_mangle]
pub unsafe extern "system" fn BRUSHOBJ_hGetColorTransform(
    Brush: *mut BRUSHOBJ
) -> HANDLE {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"BRUSHOBJ_hGetColorTransform\0".as_ptr() as PCHAR
    );
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn EngDeleteFile(
    FileName: LPWSTR
) -> i32 {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8, file!().as_ptr(), line!(), b"EngDeleteFile\0".as_ptr());
    0
}
#[no_mangle]
pub unsafe extern "system" fn EngGetPrinterDriver(
    Printer: HANDLE,
    Environment: LPWSTR,
    Level: DWORD,
    DrvInfo: *mut BYTE,
    Buf: DWORD,
    Needed: *mut DWORD,
) -> BOOL {
    EngSetLastError(ERROR_NOT_SUPPORTED);
    0
}
#[no_mangle]
pub unsafe extern "system" fn EngHangNotification(
    Dev: HDEV,
    Reserved: PVOID,
) -> ULONG {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"EngHangNotification\0".as_ptr() as PCHAR,
    );
    EHN_ERROR
}
#[no_mangle]
pub unsafe extern "system" fn EngLpkInstalled() -> BOOL {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR, 
             file!().as_ptr() as PCHAR, 
             line!() as i32, 
             b"EngLpkInstalled\0".as_ptr() as PCHAR);
    0
}
#[no_mangle]
pub unsafe extern "system" fn EngPlgBlt(
    Dest: *mut SURFOBJ,
    Source: *mut SURFOBJ,
    Mask: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    Xlate: *mut XLATEOBJ,
    ColorAdjustment: *mut COLORADJUSTMENT,
    BrusOrigin: *mut POINTL,
    DestPoints: *mut POINTFIX,
    SourceRect: *mut RECTL,
    MaskPoint: *mut POINTL,
    Mode: u32,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"stubs.c\0".as_ptr() as *const i8,
        line!() as i32,
        b"EngPlgBlt\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn EngQueryFileTimeStamp(FileName: LPWSTR) -> LARGE_INTEGER {
    let mut FileTime: LARGE_INTEGER = core::mem::zeroed();
    FileTime.QuadPart = 0;
    return FileTime;
}
#[no_mangle]
pub unsafe extern "system" fn FONTOBJ_pjOpenTypeTablePointer(
    pfo: *mut FONTOBJ,
    ulTag: u32,
    pcjTable: *mut u32,
) -> *mut u8 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"FONTOBJ_pjOpenTypeTablePointer\0".as_ptr() as PCHAR,
    );
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn FONTOBJ_pQueryGlyphAttrs(
    FontObj: *mut FONTOBJ,
    Mode: u32,
) -> PFD_GLYPHATTR {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"FONTOBJ_pQueryGlyphAttrs\0".as_ptr() as PCHAR,
    );
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn FONTOBJ_pwszFontFilePaths(
    FontObj: *mut FONTOBJ,
    PathLength: *mut u32,
) -> LPWSTR {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"stubs.c\0".as_ptr() as *const i8,
        line!() as i32,
        b"FONTOBJ_pwszFontFilePaths\0".as_ptr() as *const i8,
    );
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn HT_Get8BPPMaskPalette(
    PaletteEntry: LPPALETTEENTRY,
    Use8BPPMaskPal: i32,
    CMYMask: u8,
    RedGamma: u16,
    GreenGamma: u16,
    BlueGamma: u16,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"stubs.c\0".as_ptr() as *const i8,
        line!() as i32,
        b"HT_Get8BPPMaskPalette\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiAnyLinkedFonts() -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiAnyLinkedFonts\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEnableEudc(enable: BOOL) -> BOOL {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR, 
             file!().as_ptr() as PCHAR, 
             line!() as i32, 
             b"NtGdiEnableEudc\0".as_ptr() as PCHAR);
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiAddRemoteFontToDC(
    hdc: HDC,
    pvBuffer: PVOID,
    cjBuffer: ULONG,
    pufi: PUNIVERSAL_FONT_ID,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiAddRemoteFontToDC\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiRemoveMergeFont(
    hdc: HDC,
    pufi: *mut UNIVERSAL_FONT_ID,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiRemoveMergeFont\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiAddRemoteMMInstanceToDC(
    hdc: HDC,
    pddv: *mut DOWNLOADDESIGNVECTOR,
    cjDDV: u32,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"stubs.c\0".as_ptr() as *const i8,
        line!() as i32,
        b"NtGdiAddRemoteMMInstanceToDC\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiFontIsLinked(hdc: HDC) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiFontIsLinked\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCheckBitmapBits(
    hdc: HDC,
    hColorTransform: HANDLE,
    pvBits: PVOID,
    bmFormat: ULONG,
    dwWidth: DWORD,
    dwHeight: DWORD,
    dwStride: DWORD,
    paResults: PBYTE,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiCheckBitmapBits\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiClearBitmapAttributes(
    hbm: HBITMAP,
    dwFlags: u32,
) -> HBITMAP {
    let mut hbm_mut = hbm;
    if dwFlags & SC_BB_STOCKOBJ != 0 {
        if GDIOBJ_ConvertFromStockObj(&mut hbm_mut as *mut HBITMAP as *mut HGDIOBJ) != 0 {
            return hbm_mut;
        }
    }
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetUFI(
    hdc: HDC,
    pufi: PUNIVERSAL_FONT_ID,
    pdv: *mut DESIGNVECTOR,
    pcjDV: *mut ULONG,
    pulBaseCheckSum: *mut ULONG,
    pfl: *mut FLONG,
) -> BOOLEAN {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiGetUFI\0".as_ptr() as PCHAR,
    );
    0u8
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiColorCorrectPalette(
    hdc: HDC,
    hpal: HPALETTE,
    FirstEntry: u32,
    NumberOfEntries: u32,
    ppalEntry: *mut PALETTEENTRY,
    Command: u32,
) -> u32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        file!().as_ptr() as *const i8,
        line!() as i32,
        b"NtGdiColorCorrectPalette\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCreateColorTransform(
    hdc: HDC,
    pLogColorSpaceW: LPLOGCOLORSPACEW,
    pvSrcProfile: PVOID,
    cjSrcProfile: ULONG,
    pvDestProfile: PVOID,
    cjDestProfile: ULONG,
    pvTargetProfile: PVOID,
    cjTargetProfile: ULONG,
) -> HANDLE {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiCreateColorTransform\0".as_ptr() as PCHAR,
    );
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiComputeXformCoefficients(hdc: HDC) -> BOOL {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR, file!().as_ptr() as PCHAR, line!() as i32, b"NtGdiComputeXformCoefficients\0".as_ptr() as PCHAR);
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiConsoleTextOut(
    hdc: HDC,
    lpto: *mut POLYTEXTW,
    nStrings: UINT,
    prclBounds: *mut RECTL,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiConsoleTextOut\0".as_ptr() as PCHAR,
    );
    0i32
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDeleteColorTransform(
    hdc: HDC,
    hColorTransform: HANDLE,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiDeleteColorTransform\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetPerBandInfo(
    hdc: HDC,
    ppbi: *mut PERBANDINFO,
) -> ULONG {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiGetPerBandInfo\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDoBanding(
    hdc: HDC,
    bStart: BOOL,
    pptl: *mut POINTL,
    pSize: PSIZE,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiDoBanding\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEnumFontChunk(
    hdc: HDC,
    idEnum: ULONG_PTR,
    cjEfdw: ULONG,
    pcjEfdw: *mut ULONG,
    pefdw: PENUMFONTDATAW,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiEnumFontChunk\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEnumFontClose(
    idEnum: ULONG_PTR,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiEnumFontClose\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEnumFontOpen(
    hdc: HDC,
    iEnumType: u32,
    flWin31Compat: FLONG,
    cwchMax: u32,
    pwszFaceName: LPWSTR,
    lfCharSet: u32,
    pulCount: *mut u32,
) -> usize {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiEnumFontOpen\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEudcLoadUnloadLink(
    pBaseFaceName: LPCWSTR,
    cwcBaseFaceName: u32,
    pEudcFontPath: LPCWSTR,
    cwcEudcFontPath: u32,
    iPriority: i32,
    iFontLinkType: i32,
    bLoadLin: i32,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"stubs.c\0".as_ptr() as *const i8,
        line!() as i32,
        b"NtGdiEudcLoadUnloadLink\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiForceUFIMapping(
    hdc: HDC,
    pufi: PUNIVERSAL_FONT_ID,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiForceUFIMapping\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiFullscreenControl(
    FullscreenCommand: FULLSCREENCONTROL,
    FullscreenInput: PVOID,
    FullscreenInputLength: DWORD,
    FullscreenOutput: PVOID,
    FullscreenOutputLength: PULONG,
) -> NTSTATUS {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        0i32,
        b"NtGdiFullscreenControl\0".as_ptr() as PCHAR,
    );
    0u8 as NTSTATUS
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetCharWidthInfo(
    hdc: HDC,
    pChWidthInfo: PCHWIDTHINFO,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiGetCharWidthInfo\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetColorSpaceforBitmap(
    hsurf: HBITMAP
) -> ULONG_PTR {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR, file!().as_ptr() as PCHAR, line!() as i32, b"NtGdiGetColorSpaceforBitmap\0".as_ptr() as PCHAR);
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetETM(
    hdc: HDC,
    petm: *mut EXTTEXTMETRIC,
) -> i32 {
    UNIMPLEMENTED;
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetEudcTimeStampEx(
    lpBaseFaceName: LPWSTR,
    cwcBaseFaceName: u32,
    bSystemTimeStamp: u8,
) -> u32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        file!().as_ptr() as *const i8,
        line!() as i32,
        b"NtGdiGetEudcTimeStampEx\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiInitSpool() -> BOOL {
    EngSetLastError(ERROR_NOT_SUPPORTED);
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiQueryFonts(
    pufiFontList: PUNIVERSAL_FONT_ID,
    nBufferSize: ULONG,
    pTimeStamp: PLARGE_INTEGER,
) -> INT {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiQueryFonts\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetSpoolMessage(
    u1: DWORD,
    u2: DWORD,
    u3: DWORD,
    u4: DWORD,
) -> INT {
    EngSetLastError(ERROR_NOT_SUPPORTED);
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetGlyphIndicesWInternal(
    hdc: HDC,
    pwc: PCWCH,
    cwc: i32,
    pgi: PWORD,
    iMode: u32,
    bSubset: i32,
) -> u32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"stubs.c\0".as_ptr() as *const i8,
        line!() as i32,
        b"NtGdiGetGlyphIndicesWInternal\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetLinkedUFIs(
    hdc: HDC,
    pufiLinkedUFIs: PUNIVERSAL_FONT_ID,
    BufferSize: i32,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        file!().as_ptr() as *const i8,
        line!() as i32,
        b"NtGdiGetLinkedUFIs\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetMonitorID(
    hdc: HDC,
    dwSize: u32,
    pszMonitorID: LPWSTR,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        file!().as_ptr() as *const i8,
        line!() as i32,
        b"NtGdiGetMonitorID\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDrawStream(
    hdcDst: HDC,
    cjIn: u32,
    pvIn: *mut core::ffi::c_void,
) -> i32 {
    UNIMPLEMENTED;
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiUpdateTransform(hdc: HDC) -> BOOL {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR, file!().as_ptr() as PCHAR, line!() as i32, b"NtGdiUpdateTransform\0".as_ptr() as PCHAR);
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiQueryFontAssocInfo(
    hdc: HDC
) -> ULONG {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR, file!().as_ptr() as PCHAR, line!() as i32, b"NtGdiQueryFontAssocInfo\0".as_ptr() as PCHAR);
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetStringBitmapW(
    hdc: HDC,
    pwsz: LPWSTR,
    cwc: u32,
    lpSB: *mut u8,
    cj: u32,
) -> u32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiGetStringBitmapW\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiPolyTextOutW(
    hdc: HDC,
    pptw: *mut POLYTEXTW,
    cStr: UINT,
    dwCodePage: DWORD,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiPolyTextOutW\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetStats(
    hProcess: HANDLE,
    iIndex: INT,
    iPidType: INT,
    pResults: PVOID,
    cjResultSize: UINT,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetupPublicCFONT(
    hdc: HDC,
    hf: HFONT,
    ulAve: ULONG,
) -> INT {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiSetupPublicCFONT\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetEmbUFI(
    hdc: HDC,
    pufi: PUNIVERSAL_FONT_ID,
    pdv: *mut DESIGNVECTOR,
    pcjDV: *mut u32,
    pulBaseCheckSum: *mut u32,
    pfl: *mut FLONG,
    embFontID: *mut KERNEL_PVOID,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"stubs.c\0".as_ptr() as *const i8,
        line!() as i32,
        b"NtGdiGetEmbUFI\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetUFIPathname(
    pufi: PUNIVERSAL_FONT_ID,
    pcwc: *mut u32,
    pwszPathname: LPWSTR,
    pcNumFiles: *mut u32,
    fl: FLONG,
    pbMemFont: *mut u8,
    pcjView: *mut u32,
    pvView: *mut core::ffi::c_void,
    pbTTC: *mut u8,
    piTTC: *mut u32,
) -> u8 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiGetUFIPathname\0".as_ptr() as PCHAR,
    );
    0u8
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetEmbedFonts() -> ULONG {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiGetEmbedFonts\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiChangeGhostFont(
    pfontID: *mut KERNEL_PVOID,
    bLoad: BOOL,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiChangeGhostFont\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiAddEmbFontToDC(
    hdc: HDC,
    pFontID: *mut *mut core::ffi::c_void,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        file!().as_ptr() as *const i8,
        line!() as i32,
        b"NtGdiAddEmbFontToDC\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetWidthTable(
    hdc: HDC,
    cSpecial: ULONG,
    pwc: *mut WCHAR,
    cwc: ULONG,
    psWidth: *mut USHORT,
    pwd: *mut WIDTHDATA,
    pflInfo: *mut FLONG,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiGetWidthTable\0".as_ptr() as PCHAR,
    );
    0i32
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiIcmBrushInfo(
    hdc: HDC,
    hbrush: HBRUSH,
    pbmiDIB: PBITMAPINFO,
    pvBits: PVOID,
    pulBits: *mut ULONG,
    piUsage: *mut DWORD,
    pbAlreadyTran: *mut BOOL,
    Command: ULONG,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiIcmBrushInfo\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiMakeFontDir(
    flEmbed: FLONG,
    pjFontDir: PBYTE,
    cjFontDir: u32,
    pwszPathname: LPWSTR,
    cjPathname: u32,
) -> u32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        file!().as_ptr() as *const i8,
        line!() as i32,
        b"NtGdiMakeFontDir\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiMonoBitmap(
    hbm: HBITMAP,
) -> i32 {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR, file!().as_ptr() as PCHAR, line!() as i32, b"NtGdiMonoBitmap\0".as_ptr() as PCHAR);
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetBitmapAttributes(
    mut hbm: HBITMAP,
    dwFlags: DWORD,
) -> HBITMAP {
    if dwFlags & SC_BB_STOCKOBJ != 0 {
        if GDIOBJ_ConvertToStockObj(&mut hbm as *mut HBITMAP as *mut HGDIOBJ) != 0 {
            return hbm;
        }
    }
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetFontEnumeration(
    ulType: ULONG,
) -> ULONG {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiSetFontEnumeration\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetFontXform(
    hdc: HDC,
    dwxScale: DWORD,
    dwyScale: DWORD,
) -> BOOL {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"NtGdiSetFontXform\0".as_ptr() as PCHAR,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetLinkedUFIs(
    hdc: HDC,
    pufiLinks: PUNIVERSAL_FONT_ID,
    uNumUFIs: u32,
) -> i32 {
    DbgPrint(b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8, file!().as_ptr(), line!(), b"NtGdiSetLinkedUFIs\0".as_ptr());
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetMagicColors(
    hdc: HDC,
    peMagic: PALETTEENTRY,
    Index: u32,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        file!().as_ptr() as *const i8,
        line!() as i32,
        b"NtGdiSetMagicColors\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiUnloadPrinterDriver(
    pDriverName: LPWSTR,
    cbDriverName: u32,
) -> i32 {
    EngSetLastError(ERROR_NOT_SUPPORTED);
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiUnmapMemFont(
    pvView: PVOID) -> BOOL {
    return 1i32;
}
#[no_mangle]
pub unsafe extern "system" fn EngControlSprites(
    pwo: *mut WNDOBJ,
    fl: FLONG,
) -> i32 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"stubs.c\0".as_ptr() as *const i8,
        line!() as i32,
        b"EngControlSprites\0".as_ptr() as *const i8,
    );
    0
}
#[no_mangle]
pub unsafe extern "system" fn EngFntCacheAlloc(
    ulFastCheckSum: ULONG,
    cjSize: ULONG,
) -> PVOID {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"EngFntCacheAlloc\0".as_ptr() as PCHAR,
    );
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn EngFntCacheFault(
    ulFastCheckSum: ULONG,
    iFaultMode: ULONG,
) {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"EngFntCacheFault\0".as_ptr() as PCHAR,
    );
}
#[no_mangle]
pub unsafe extern "system" fn EngFntCacheLookUp(
    FastCheckSum: ULONG,
    pulSize: PULONG,
) -> PVOID {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        file!().as_ptr() as PCHAR,
        line!() as i32,
        b"EngFntCacheLookUp\0".as_ptr() as PCHAR,
    );
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn EngNineGrid(
    pDestSurfaceObj: *mut SURFOBJ,
    pSourceSurfaceObj: *mut SURFOBJ,
    pClipObj: *mut CLIPOBJ,
    pXlateObj: *mut XLATEOBJ,
    prclSource: *mut RECTL,
    prclDest: *mut RECTL,
    pvUnknown1: *mut core::ffi::c_void,
    pvUnknown2: *mut core::ffi::c_void,
    dwReserved: u32,
) -> u8 {
    DbgPrint(
        b"(%s:%i) WIN32K: %s UNIMPLEMENTED\n\0".as_ptr() as PCHAR,
        b"stubs.c\0".as_ptr() as PCHAR,
        line!() as i32,
        b"EngNineGrid\0".as_ptr() as PCHAR,
    );
    0u8
}

// EngComputeGlyphSet — APIENTRY (stdcall on i386, cdecl elsewhere).
// @unimplemented in stubs.c; NULL-returning stub.
#[no_mangle]
pub unsafe extern "system" fn EngComputeGlyphSet(
    _nCodePage: INT,
    _nFirstChar: INT,
    _cChars: INT,
) -> *mut FD_GLYPHSET {
    core::ptr::null_mut()
}

// FONTOBJ_pifi — APIENTRY (stdcall i386, cdecl elsewhere). @unimplemented.
#[no_mangle]
pub unsafe extern "system" fn FONTOBJ_pifi(_FontObj: *mut FONTOBJ) -> *mut IFIMETRICS {
    core::ptr::null_mut()
}

// FONTOBJ_pxoGetXform — APIENTRY. @unimplemented.
#[no_mangle]
pub unsafe extern "system" fn FONTOBJ_pxoGetXform(_FontObj: *mut FONTOBJ) -> *mut XFORMOBJ {
    core::ptr::null_mut()
}

// FONTOBJ_pfdg — APIENTRY. @unimplemented. Returns FD_GLYPHSET*.
#[no_mangle]
pub unsafe extern "system" fn FONTOBJ_pfdg(_FontObj: *mut FONTOBJ) -> *mut FD_GLYPHSET {
    core::ptr::null_mut()
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Stubs for unimplemented WIN32K.SYS exports
//  */
//
// #include <win32k.h>
// #undef XFORMOBJ
//
// #define UNIMPLEMENTED DbgPrint("(%s:%i) WIN32K: %s UNIMPLEMENTED\n", __FILE__, __LINE__, __FUNCTION__ )
//
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// EngTextOut(
//     SURFOBJ  *pso,
//     STROBJ   *pstro,
//     FONTOBJ  *pfo,
//     CLIPOBJ  *pco,
//     RECTL    *prclExtra,
//     RECTL    *prclOpaque,
//     BRUSHOBJ *pboFore,
//     BRUSHOBJ *pboOpaque,
//     POINTL   *pptlOrg,
//     MIX       mix)
// {
//     // www.osr.com/ddk/graphics/gdifncs_4tgn.htm
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// PATHOBJ*
// APIENTRY
// CLIPOBJ_ppoGetPath(IN CLIPOBJ *pco)
// {
//     // www.osr.com/ddk/graphics/gdifncs_6hbb.htm
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// EngCheckAbort(IN SURFOBJ *pso)
// {
//     // www.osr.com/ddk/graphics/gdifncs_3u7b.htm
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// FD_GLYPHSET*
// APIENTRY
// EngComputeGlyphSet(
//     IN INT nCodePage,
//     IN INT nFirstChar,
//     IN INT cChars)
// {
//     // www.osr.com/ddk/graphics/gdifncs_9607.htm
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// EngEnumForms(
//     IN  HANDLE   hPrinter,
//     IN  DWORD    Level,
//     OUT LPBYTE   pForm,
//     IN  DWORD    cbBuf,
//     OUT LPDWORD  pcbNeeded,
//     OUT LPDWORD  pcReturned)
// {
//     // www.osr.com/ddk/graphics/gdifncs_5e07.htm
//     EngSetLastError(ERROR_NOT_SUPPORTED);
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// EngFillPath(
//     IN SURFOBJ   *pso,
//     IN PATHOBJ   *ppo,
//     IN CLIPOBJ   *pco,
//     IN BRUSHOBJ  *pbo,
//     IN POINTL    *pptlBrushOrg,
//     IN MIX        mix,
//     IN FLONG      flOptions)
// {
//     // www.osr.com/ddk/graphics/gdifncs_9pyf.htm
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// PVOID
// APIENTRY
// EngFindResource(
//     IN  HANDLE  h,
//     IN  int     iName,
//     IN  int     iType,
//     OUT PULONG  pulSize)
// {
//     // www.osr.com/ddk/graphics/gdifncs_7rjb.htm
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// EngGetFileChangeTime(
//     IN  HANDLE h,
//     OUT LARGE_INTEGER *pChangeTime)
// {
//     // www.osr.com/ddk/graphics/gdifncs_1i1z.htm
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// EngGetFilePath(
//     IN  HANDLE h,
//     OUT WCHAR (*pDest)[MAX_PATH + 1])
// {
//     // www.osr.com/ddk/graphics/gdifncs_5g2v.htm
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// EngGetForm(
//     IN  HANDLE   hPrinter,
//     IN  LPWSTR   pFormName,
//     IN  DWORD    Level,
//     OUT LPBYTE   pForm,
//     IN  DWORD    cbBuf,
//     OUT LPDWORD  pcbNeeded)
// {
//     // www.osr.com/ddk/graphics/gdifncs_5vvr.htm
//     EngSetLastError(ERROR_NOT_SUPPORTED);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// EngGetPrinter(
//     IN  HANDLE   hPrinter,
//     IN  DWORD    dwLevel,
//     OUT LPBYTE   pPrinter,
//     IN  DWORD    cbBuf,
//     OUT LPDWORD  pcbNeeded)
// {
//     // www.osr.com/ddk/graphics/gdifncs_50h3.htm
//     EngSetLastError(ERROR_NOT_SUPPORTED);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// APIENTRY
// EngGetPrinterData(
//     IN  HANDLE   hPrinter,
//     IN  LPWSTR   pValueName,
//     OUT LPDWORD  pType,
//     OUT LPBYTE   pData,
//     IN  DWORD    nSize,
//     OUT LPDWORD  pcbNeeded)
// {
//     // www.osr.com/ddk/graphics/gdifncs_8t5z.htm
//     EngSetLastError(ERROR_NOT_SUPPORTED);
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// LPWSTR
// APIENTRY
// EngGetPrinterDataFileName(IN HDEV hdev)
// {
//     // www.osr.com/ddk/graphics/gdifncs_2giv.htm
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// EngGetType1FontList(
//     IN  HDEV            hdev,
//     OUT TYPE1_FONT     *pType1Buffer,
//     IN  ULONG           cjType1Buffer,
//     OUT PULONG          pulLocalFonts,
//     OUT PULONG          pulRemoteFonts,
//     OUT LARGE_INTEGER  *pLastModified)
// {
//     // www.osr.com/ddk/graphics/gdifncs_6e5j.htm
//     EngSetLastError(ERROR_NOT_SUPPORTED);
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// EngMarkBandingSurface(IN HSURF hsurf)
// {
//     // www.osr.com/ddk/graphics/gdifncs_2jon.htm
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// INT
// APIENTRY
// EngMultiByteToWideChar(
//     IN UINT  CodePage,
//     OUT LPWSTR  WideCharString,
//     IN INT  BytesInWideCharString,
//     IN LPSTR  MultiByteString,
//     IN INT  BytesInMultiByteString)
// {
//     // www.osr.com/ddk/graphics/gdifncs_32cn.htm
//     UNIMPLEMENTED;
//     return 0;
// }
//
// VOID
// APIENTRY
// EngQueryLocalTime(
//     _Out_ PENG_TIME_FIELDS ptf)
// {
//     LARGE_INTEGER liSystemTime, liLocalTime;
//     NT_ASSERT(ptf != NULL);
//
//     /* Query the system time */
//     KeQuerySystemTime(&liSystemTime);
//
//     /* Convert it to local time */
//     ExSystemTimeToLocalTime(&liSystemTime, &liLocalTime);
//
//     /* Convert the local time into time fields
//        (note that ENG_TIME_FIELDS is identical to TIME_FIELDS) */
//     RtlTimeToTimeFields(&liLocalTime, (PTIME_FIELDS)ptf);
// }
//
// ULONG
// APIENTRY
// EngQueryPalette(
//     IN HPALETTE  hPal,
//     OUT ULONG  *piMode,
//     IN ULONG  cColors,
//     OUT ULONG  *pulColors)
// {
//     // www.osr.com/ddk/graphics/gdifncs_21t3.htm
//     UNIMPLEMENTED;
//     return 0;
// }
//
// DWORD
// APIENTRY
// EngSetPrinterData(
//     IN HANDLE  hPrinter,
//     IN LPWSTR  pType,
//     IN DWORD  dwType,
//     IN LPBYTE  lpbPrinterData,
//     IN DWORD  cjPrinterData)
// {
//     // www.osr.com/ddk/graphics/gdifncs_8drb.htm
//     EngSetLastError(ERROR_NOT_SUPPORTED);
//     return 0;
// }
//
// BOOL
// APIENTRY
// EngStrokeAndFillPath(
//     IN SURFOBJ  *pso,
//     IN PATHOBJ  *ppo,
//     IN CLIPOBJ  *pco,
//     IN XFORMOBJ  *pxo,
//     IN BRUSHOBJ  *pboStroke,
//     IN LINEATTRS  *plineattrs,
//     IN BRUSHOBJ  *pboFill,
//     IN POINTL  *pptlBrushOrg,
//     IN MIX  mixFill,
//     IN FLONG  flOptions)
// {
//     // www.osr.com/ddk/graphics/gdifncs_2xwn.htm
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// BOOL
// APIENTRY
// EngStrokePath(
//     IN SURFOBJ  *pso,
//     IN PATHOBJ  *ppo,
//     IN CLIPOBJ  *pco,
//     IN XFORMOBJ  *pxo,
//     IN BRUSHOBJ  *pbo,
//     IN POINTL  *pptlBrushOrg,
//     IN LINEATTRS  *plineattrs,
//     IN MIX  mix)
// {
//     // www.osr.com/ddk/graphics/gdifncs_4yaw.htm
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// INT
// APIENTRY
// EngWideCharToMultiByte(
//     IN UINT  CodePage,
//     IN LPWSTR  WideCharString,
//     IN INT  BytesInWideCharString,
//     OUT LPSTR  MultiByteString,
//     IN INT  BytesInMultiByteString)
// {
//     // www.osr.com/ddk/graphics/gdifncs_35wn.htm
//     UNIMPLEMENTED;
//     return 0;
// }
//
// BOOL
// APIENTRY
// EngWritePrinter(
//     IN HANDLE    hPrinter,
//     IN LPVOID    pBuf,
//     IN DWORD     cbBuf,
//     OUT LPDWORD  pcWritten)
// {
//     // www.osr.com/ddk/graphics/gdifncs_9v6v.htm
//     EngSetLastError(ERROR_NOT_SUPPORTED);
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// ULONG
// APIENTRY
// FONTOBJ_cGetAllGlyphHandles(
//     IN FONTOBJ  *FontObj,
//     IN HGLYPH   *Glyphs)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// ULONG
// APIENTRY
// FONTOBJ_cGetGlyphs(
//     IN FONTOBJ *FontObj,
//     IN ULONG    Mode,
//     IN ULONG    NumGlyphs,
//     IN HGLYPH  *GlyphHandles,
//     IN PVOID   *OutGlyphs)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// IFIMETRICS*
// APIENTRY
// FONTOBJ_pifi(IN FONTOBJ *FontObj)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// /*
//  * @unimplemented
//  */
// _Ret_opt_bytecount_(*pcjFile)
// ENGAPI
// PVOID
// APIENTRY
// FONTOBJ_pvTrueTypeFontFile(
//     _In_ FONTOBJ *pfo,
//     _Out_ ULONG *pcjFile)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// /*
//  * @unimplemented
//  */
// XFORMOBJ*
// APIENTRY
// FONTOBJ_pxoGetXform(IN FONTOBJ *FontObj)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// APIENTRY
// FONTOBJ_vGetInfo(
//     IN  FONTOBJ *FontObj,
//     IN  ULONG InfoSize,
//     OUT PFONTINFO FontInfo)
// {
//     UNIMPLEMENTED;
// }
//
// LONG
// APIENTRY
// HT_ComputeRGBGammaTable(
//     IN USHORT  GammaTableEntries,
//     IN USHORT  GammaTableType,
//     IN USHORT  RedGamma,
//     IN USHORT  GreenGamma,
//     IN USHORT  BlueGamma,
//     OUT LPBYTE  pGammaTable)
// {
//     // www.osr.com/ddk/graphics/gdifncs_9dpj.htm
//     UNIMPLEMENTED;
//     return 0;
// }
//
// LONG
// APIENTRY
// HT_Get8BPPFormatPalette(
//     OUT LPPALETTEENTRY  pPaletteEntry,
//     IN USHORT  RedGamma,
//     IN USHORT  GreenGamma,
//     IN USHORT  BlueGamma)
// {
//     // www.osr.com/ddk/graphics/gdifncs_8kvb.htm
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// ULONG
// APIENTRY
// EngDitherColor(
//     IN HDEV hdev,
//     IN ULONG iMode,
//     IN ULONG rgb,
//     OUT ULONG *pul)
// {
//     *pul = 0;
//     return DCR_SOLID;
// }
//
// /*
//  * @unimplemented
//  */
// HANDLE
// APIENTRY
// BRUSHOBJ_hGetColorTransform(
//     IN BRUSHOBJ *Brush)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// EngDeleteFile(
//     IN LPWSTR FileName)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// EngGetPrinterDriver(
//     IN HANDLE Printer,
//     IN LPWSTR Environment,
//     IN DWORD Level,
//     OUT BYTE *DrvInfo,
//     IN DWORD Buf,
//     OUT DWORD *Needed)
// {
//     EngSetLastError(ERROR_NOT_SUPPORTED);
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// ULONG
// APIENTRY
// EngHangNotification(
//     IN HDEV Dev,
//     IN PVOID Reserved)
// {
//     UNIMPLEMENTED;
//     return EHN_ERROR;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// EngLpkInstalled(VOID)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// EngPlgBlt(
//     IN SURFOBJ *Dest,
//     IN SURFOBJ *Source,
//     IN SURFOBJ *Mask,
//     IN CLIPOBJ *Clip,
//     IN XLATEOBJ *Xlate,
//     IN COLORADJUSTMENT *ColorAdjustment,
//     IN POINTL *BrusOrigin,
//     IN POINTFIX *DestPoints,
//     IN RECTL *SourceRect,
//     IN POINTL *MaskPoint,
//     IN ULONG Mode)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// LARGE_INTEGER
// APIENTRY
// EngQueryFileTimeStamp(IN LPWSTR FileName)
// {
//     LARGE_INTEGER FileTime;
//     FileTime.QuadPart = 0;
//     UNIMPLEMENTED;
//     return FileTime;
// }
//
// /*
//  * @unimplemented
//  */
// FD_GLYPHSET *
// APIENTRY
// FONTOBJ_pfdg(
//     IN FONTOBJ *FontObj)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// /*
//  * @unimplemented
//  */
// _Ret_opt_bytecount_(*pcjTable)
// ENGAPI
// PBYTE
// APIENTRY
// FONTOBJ_pjOpenTypeTablePointer(
//     _In_ FONTOBJ *pfo,
//     _In_ ULONG ulTag,
//     _Out_ ULONG *pcjTable)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// /*
//  * @unimplemented
//  */
// PFD_GLYPHATTR
// APIENTRY
// FONTOBJ_pQueryGlyphAttrs(
//     IN FONTOBJ *FontObj,
//     IN ULONG Mode)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// /*
//  * @unimplemented
//  */
// LPWSTR
// APIENTRY
// FONTOBJ_pwszFontFilePaths(
//     IN FONTOBJ *FontObj,
//     OUT ULONG *PathLength)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// /*
//  * @unimplemented
//  */
// LONG
// APIENTRY
// HT_Get8BPPMaskPalette(
//     IN OUT LPPALETTEENTRY PaletteEntry,
//     IN BOOL Use8BPPMaskPal,
//     IN BYTE CMYMask,
//     IN USHORT RedGamma,
//     IN USHORT GreenGamma,
//     IN USHORT BlueGamma)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiAnyLinkedFonts(VOID)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiEnableEudc(BOOL enable)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiAddRemoteFontToDC(
//     IN HDC hdc,
//     IN PVOID pvBuffer,
//     IN ULONG cjBuffer,
//     IN OPTIONAL PUNIVERSAL_FONT_ID pufi)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiRemoveMergeFont(
//     IN HDC hdc,
//     IN UNIVERSAL_FONT_ID *pufi)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiAddRemoteMMInstanceToDC(
//     IN HDC hdc,
//     IN DOWNLOADDESIGNVECTOR *pddv,
//     IN ULONG cjDDV)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiFontIsLinked(IN HDC hdc)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiCheckBitmapBits(
//     IN HDC hdc,
//     IN HANDLE hColorTransform,
//     IN PVOID pvBits,
//     IN ULONG bmFormat,
//     IN DWORD dwWidth,
//     IN DWORD dwHeight,
//     IN DWORD dwStride,
//     OUT PBYTE paResults)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// HBITMAP
// APIENTRY
// NtGdiClearBitmapAttributes(
//     IN HBITMAP hbm,
//     IN DWORD dwFlags)
// {
//     if ( dwFlags & SC_BB_STOCKOBJ )
//     {
//         if (GDIOBJ_ConvertFromStockObj((HGDIOBJ*)&hbm))
//         {
//             return hbm;
//         }
//     }
//     return NULL;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiGetUFI(
//     IN  HDC hdc,
//     OUT PUNIVERSAL_FONT_ID pufi,
//     OUT OPTIONAL DESIGNVECTOR *pdv,
//     OUT ULONG *pcjDV,
//     OUT ULONG *pulBaseCheckSum,
//     OUT FLONG *pfl)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// ULONG
// APIENTRY
// NtGdiColorCorrectPalette(
//     IN HDC hdc,
//     IN HPALETTE hpal,
//     IN ULONG FirstEntry,
//     IN ULONG NumberOfEntries,
//     IN OUT PALETTEENTRY *ppalEntry,
//     IN ULONG Command)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// HANDLE
// APIENTRY
// NtGdiCreateColorTransform(
//     IN HDC hdc,
//     IN LPLOGCOLORSPACEW pLogColorSpaceW,
//     IN OPTIONAL PVOID pvSrcProfile,
//     IN ULONG cjSrcProfile,
//     IN OPTIONAL PVOID pvDestProfile,
//     IN ULONG cjDestProfile,
//     IN OPTIONAL PVOID pvTargetProfile,
//     IN ULONG cjTargetProfile)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiComputeXformCoefficients(IN HDC hdc)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiConsoleTextOut(
//     IN HDC hdc,
//     IN POLYTEXTW *lpto,
//     IN UINT nStrings,
//     IN RECTL *prclBounds)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiDeleteColorTransform(
//     IN HDC hdc,
//     IN HANDLE hColorTransform)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// ULONG
// APIENTRY
// NtGdiGetPerBandInfo(
//     IN HDC hdc,
//     IN OUT PERBANDINFO *ppbi)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiDoBanding(
//     IN HDC hdc,
//     IN BOOL bStart,
//     OUT POINTL *pptl,
//     OUT PSIZE pSize)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiEnumFontChunk(
//     IN HDC hdc,
//     IN ULONG_PTR idEnum,
//     IN ULONG cjEfdw,
//     OUT ULONG *pcjEfdw,
//     OUT PENUMFONTDATAW pefdw)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiEnumFontClose(
//     IN ULONG_PTR idEnum)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// ULONG_PTR
// APIENTRY
// NtGdiEnumFontOpen(
//     IN HDC hdc,
//     IN ULONG iEnumType,
//     IN FLONG flWin31Compat,
//     IN ULONG cwchMax,
//     IN OPTIONAL LPWSTR pwszFaceName,
//     IN ULONG lfCharSet,
//     OUT ULONG *pulCount)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiEudcLoadUnloadLink(
//     IN OPTIONAL LPCWSTR pBaseFaceName,
//     IN UINT cwcBaseFaceName,
//     IN LPCWSTR pEudcFontPath,
//     IN UINT cwcEudcFontPath,
//     IN INT iPriority,
//     IN INT iFontLinkType,
//     IN BOOL bLoadLin)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiForceUFIMapping(
//     IN HDC hdc,
//     IN PUNIVERSAL_FONT_ID pufi)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// APIENTRY
// NtGdiFullscreenControl(
//     IN FULLSCREENCONTROL FullscreenCommand,
//     IN PVOID FullscreenInput,
//     IN DWORD FullscreenInputLength,
//     OUT PVOID FullscreenOutput,
//     IN OUT PULONG FullscreenOutputLength)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiGetCharWidthInfo(
//     IN HDC hdc,
//     OUT PCHWIDTHINFO pChWidthInfo)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// ULONG_PTR
// APIENTRY
// NtGdiGetColorSpaceforBitmap(
//     IN HBITMAP hsurf)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiGetETM(
//     IN HDC hdc,
//     OUT EXTTEXTMETRIC *petm)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// ULONG
// APIENTRY
// NtGdiGetEudcTimeStampEx(
//     IN OPTIONAL LPWSTR lpBaseFaceName,
//     IN ULONG cwcBaseFaceName,
//     IN BOOL bSystemTimeStamp)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// NtGdiInitSpool(VOID)
// {
//     EngSetLastError(ERROR_NOT_SUPPORTED);
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// APIENTRY
// NtGdiQueryFonts(
//     OUT PUNIVERSAL_FONT_ID pufiFontList,
//     IN ULONG nBufferSize,
//     OUT PLARGE_INTEGER pTimeStamp)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @implemented
//  */
// INT
// APIENTRY
// NtGdiGetSpoolMessage(
//     DWORD u1,
//     DWORD u2,
//     DWORD u3,
//     DWORD u4)
// {
//     /* FIXME: The prototypes */
//     EngSetLastError(ERROR_NOT_SUPPORTED);
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// DWORD
// APIENTRY
// NtGdiGetGlyphIndicesWInternal(
//     _In_ HDC hdc,
//     _In_reads_opt_(cwc) PCWCH pwc,
//     _In_ INT cwc,
//     _Out_writes_opt_(cwc) PWORD pgi,
//     _In_ DWORD iMode,
//     _In_ BOOL bSubset)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// APIENTRY
// NtGdiGetLinkedUFIs(
//     IN HDC hdc,
//     OUT OPTIONAL PUNIVERSAL_FONT_ID pufiLinkedUFIs,
//     IN INT BufferSize)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiGetMonitorID(
//     IN  HDC hdc,
//     IN  DWORD dwSize,
//     OUT LPWSTR pszMonitorID)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiDrawStream(
//     IN HDC hdcDst,
//     IN ULONG cjIn,
//     IN VOID *pvIn)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiUpdateTransform(IN HDC hdc)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// ULONG
// APIENTRY
// NtGdiQueryFontAssocInfo(
//     IN HDC hdc)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// UINT
// APIENTRY
// NtGdiGetStringBitmapW(
//     IN HDC hdc,
//     IN LPWSTR pwsz,
//     IN UINT cwc,
//     OUT BYTE *lpSB,
//     IN UINT cj)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiPolyTextOutW(
//     IN HDC hdc,
//     IN POLYTEXTW *pptw,
//     IN UINT cStr,
//     IN DWORD dwCodePage)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// APIENTRY
// NtGdiGetStats(
//     IN HANDLE hProcess,
//     IN INT iIndex,
//     IN INT iPidType,
//     OUT PVOID pResults,
//     IN UINT cjResultSize)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// APIENTRY
// NtGdiSetupPublicCFONT(
//     IN HDC hdc,
//     IN OPTIONAL HFONT hf,
//     IN ULONG ulAve)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiGetEmbUFI(
//     IN HDC hdc,
//     OUT PUNIVERSAL_FONT_ID pufi,
//     OUT OPTIONAL DESIGNVECTOR *pdv,
//     OUT ULONG *pcjDV,
//     OUT ULONG *pulBaseCheckSum,
//     OUT FLONG  *pfl,
//     OUT KERNEL_PVOID *embFontID)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiGetUFIPathname(
//     IN PUNIVERSAL_FONT_ID pufi,
//     OUT OPTIONAL ULONG* pcwc,
//     OUT OPTIONAL LPWSTR pwszPathname,
//     OUT OPTIONAL ULONG* pcNumFiles,
//     IN FLONG fl,
//     OUT OPTIONAL BOOL *pbMemFont,
//     OUT OPTIONAL ULONG *pcjView,
//     OUT OPTIONAL PVOID pvView,
//     OUT OPTIONAL BOOL *pbTTC,
//     OUT OPTIONAL ULONG *piTTC)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// ULONG
// APIENTRY
// NtGdiGetEmbedFonts(
//     VOID)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiChangeGhostFont(
//     IN KERNEL_PVOID *pfontID,
//     IN BOOL bLoad)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiAddEmbFontToDC(
//     IN HDC hdc,
//     IN VOID **pFontID)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiGetWidthTable(
//     IN HDC hdc,
//     IN ULONG cSpecial,
//     IN WCHAR *pwc,
//     IN ULONG cwc,
//     OUT USHORT *psWidth,
//     OUT OPTIONAL WIDTHDATA *pwd,
//     OUT FLONG *pflInfo)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiIcmBrushInfo(
//     IN HDC hdc,
//     IN HBRUSH hbrush,
//     IN OUT PBITMAPINFO pbmiDIB,
//     IN OUT PVOID pvBits,
//     IN OUT ULONG *pulBits,
//     OUT OPTIONAL DWORD *piUsage,
//     OUT OPTIONAL BOOL *pbAlreadyTran,
//     IN ULONG Command)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// ULONG
// APIENTRY
// NtGdiMakeFontDir(
//     IN FLONG flEmbed,
//     OUT PBYTE pjFontDir,
//     IN unsigned cjFontDir,
//     IN LPWSTR pwszPathname,
//     IN unsigned cjPathname)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiMonoBitmap(
//     IN HBITMAP hbm)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// HBITMAP
// APIENTRY
// NtGdiSetBitmapAttributes(
//     IN HBITMAP hbm,
//     IN DWORD dwFlags)
// {
//     if ( dwFlags & SC_BB_STOCKOBJ )
//     {
//         if (GDIOBJ_ConvertToStockObj((HGDIOBJ*)&hbm))
//         {
//             return hbm;
//         }
//     }
//     return NULL;
// }
//
// /*
//  * @unimplemented
//  */
// ULONG
// APIENTRY
// NtGdiSetFontEnumeration(
//     IN ULONG ulType)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiSetFontXform(
//     IN HDC hdc,
//     IN DWORD dwxScale,
//     IN DWORD dwyScale)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiSetLinkedUFIs(
//     IN HDC hdc,
//     IN PUNIVERSAL_FONT_ID pufiLinks,
//     IN ULONG uNumUFIs)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// NtGdiSetMagicColors(
//     IN HDC hdc,
//     IN PALETTEENTRY peMagic,
//     IN ULONG Index)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// NtGdiUnloadPrinterDriver(
//     IN LPWSTR pDriverName,
//     IN ULONG cbDriverName)
// {
//     EngSetLastError(ERROR_NOT_SUPPORTED);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// NtGdiUnmapMemFont(
//     IN PVOID pvView)
// {
//     return TRUE;
// }
//
// BOOL
// APIENTRY
// EngControlSprites(
//     IN WNDOBJ  *pwo,
//     IN FLONG  fl)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// _Must_inspect_result_
// _Ret_opt_bytecap_(cjSize)
// ENGAPI
// PVOID
// APIENTRY
// EngFntCacheAlloc(
//     _In_ ULONG ulFastCheckSum,
//     _In_ ULONG cjSize)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// VOID
// APIENTRY
// EngFntCacheFault(
//     IN ULONG ulFastCheckSum,
//     IN ULONG iFaultMode)
// {
//     UNIMPLEMENTED;
// }
//
// PVOID
// APIENTRY
// EngFntCacheLookUp(
//     IN ULONG FastCheckSum,
//     OUT PULONG pulSize)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// BOOLEAN
// APIENTRY
// EngNineGrid(
//     IN SURFOBJ* pDestSurfaceObj,
//     IN SURFOBJ* pSourceSurfaceObj,
//     IN CLIPOBJ* pClipObj,
//     IN XLATEOBJ* pXlateObj,
//     IN RECTL* prclSource,
//     IN RECTL* prclDest,
//     PVOID pvUnknown1,
//     PVOID pvUnknown2,
//     DWORD dwReserved)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
