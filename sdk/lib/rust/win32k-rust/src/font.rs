use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ExAllocatePoolWithQuotaTag, KeEnterCriticalRegion, KeLeaveCriticalRegion};

// ExAllocatePoolZero is a FORCEINLINE (alloc + zero) with no exported symbol — inline it.
#[inline(always)]
unsafe fn ExAllocatePoolZero(pool: POOL_TYPE, n: usize, tag: u32) -> PVOID {
    let p = ntkrnl_sys::ddk::ExAllocatePoolWithTag(pool, n as _, tag);
    if !p.is_null() {
        ntkrnl_sys::rtl::RtlZeroMemory(p, n as _);
    }
    p
}

// _ExInitializePushLock is a FORCEINLINE (`PushLock->Ptr = 0`) — no exported symbol,
// so inline it here instead of using the (undefined) ddk extern binding. Generic over
// the pointee so call sites need no cast.
#[inline(always)]
unsafe fn ExInitializePushLock<T>(push_lock: *mut T) {
    *(push_lock as *mut usize) = 0;
}
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS};
use core::ffi::c_void;
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;

// usize-accepting wrappers over ddk/rtl primitives (i386 SIZE_T = u32) — same as path.rs.
#[inline(always)]
unsafe fn ExAllocatePoolWithTag(pool: POOL_TYPE, n: usize, tag: u32) -> PVOID {
    ntkrnl_sys::ddk::ExAllocatePoolWithTag(pool, n as _, tag)
}
#[inline(always)]
unsafe fn ExFreePoolWithTag(p: PVOID, tag: u32) {
    ntkrnl_sys::ddk::ExFreePoolWithTag(p, tag)
}
#[inline(always)]
unsafe fn RtlZeroMemory(d: *mut c_void, n: usize) {
    ntkrnl_sys::rtl::RtlZeroMemory(d, n as _)
}
#[inline(always)]
unsafe fn RtlCopyMemory(d: *mut c_void, s: *const c_void, n: usize) {
    ntkrnl_sys::rtl::RtlCopyMemory(d, s, n as _)
}

// FASTCALL kernel helpers (freetype.c / text.c): i386 decorates @Name@N, cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ftGdiGetTextCharsetInfo(Dc: PDC, lpSig: *mut FONTSIGNATURE, dwFlags: u32) -> u32;
    fn TextIntRealizeFont(hFont: HFONT, pTextObj: PTEXTOBJ) -> NTSTATUS;
    fn ftGdiGetGlyphOutline(dc: PDC, wch: u16, iFormat: u32, pgm: LPGLYPHMETRICS, cjBuf: u32, pvBuf: *mut c_void, pmat2: *const MAT2, bIgnoreRotation: i32) -> u32;
    fn ftGdiGetKerningPairs(FontGDI: PFONTGDI, NumPairs: ULONG, krnpair: LPKERNINGPAIR) -> DWORD;
    fn TextIntGetTextExtentPoint(pdc: PDC, TextObj: PTEXTOBJ, String: *const u16, Count: core::ffi::c_int, MaxExtent: ULONG, Fit: *mut core::ffi::c_int, Dx: *mut core::ffi::c_int, Size: *mut SIZE, flOpts: FLONG) -> BOOL;
    fn ftGdiGetTextMetricsW(hDC: HDC, ptmwi: *mut TMW_INTERNAL) -> BOOL;
    fn SetLastNtError(Status: NTSTATUS);
    fn ftGetFontUnicodeRanges(Font: PFONTGDI, glyphset: PGLYPHSET) -> DWORD;
    fn ftGdiRealizationInfo(Font: PFONTGDI, Info: PREALIZATION_INFO) -> BOOL;
    fn ftGdiGetFontData(FontGdi: PFONTGDI, Table: DWORD, Offset: DWORD, Buffer: PVOID, Size: DWORD) -> DWORD;
    fn TextIntUpdateSize(dc: PDC, TextObj: PTEXTOBJ, FontGDI: PFONTGDI, bDoLock: BOOL) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ftGdiGetTextCharsetInfo(Dc: PDC, lpSig: *mut FONTSIGNATURE, dwFlags: u32) -> u32;
    fn TextIntRealizeFont(hFont: HFONT, pTextObj: PTEXTOBJ) -> NTSTATUS;
    fn ftGdiGetGlyphOutline(dc: PDC, wch: u16, iFormat: u32, pgm: LPGLYPHMETRICS, cjBuf: u32, pvBuf: *mut c_void, pmat2: *const MAT2, bIgnoreRotation: i32) -> u32;
    fn ftGdiGetKerningPairs(FontGDI: PFONTGDI, NumPairs: ULONG, krnpair: LPKERNINGPAIR) -> DWORD;
    fn TextIntGetTextExtentPoint(pdc: PDC, TextObj: PTEXTOBJ, String: *const u16, Count: core::ffi::c_int, MaxExtent: ULONG, Fit: *mut core::ffi::c_int, Dx: *mut core::ffi::c_int, Size: *mut SIZE, flOpts: FLONG) -> BOOL;
    fn ftGdiGetTextMetricsW(hDC: HDC, ptmwi: *mut TMW_INTERNAL) -> BOOL;
    fn SetLastNtError(Status: NTSTATUS);
    fn ftGetFontUnicodeRanges(Font: PFONTGDI, glyphset: PGLYPHSET) -> DWORD;
    fn ftGdiRealizationInfo(Font: PFONTGDI, Info: PREALIZATION_INFO) -> BOOL;
    fn ftGdiGetFontData(FontGdi: PFONTGDI, Table: DWORD, Offset: DWORD, Buffer: PVOID, Size: DWORD) -> DWORD;
    fn TextIntUpdateSize(dc: PDC, TextObj: PTEXTOBJ, FontGDI: PFONTGDI, bDoLock: BOOL) -> BOOL;
}

// NTAPI (STDCALL) helpers: _Name@N on i386, cdecl elsewhere → extern "system".
// SafeProbe* take SIZE_T length (usize), confirmed _SafeProbeForRead@12 on i386.
extern "system" {
    fn SafeProbeForRead(Address: PVOID, Length: usize, Alignment: ULONG) -> NTSTATUS;
    fn SafeProbeForWrite(Address: PVOID, Length: usize, Alignment: ULONG) -> NTSTATUS;
    fn GDIOBJ_AllocObjWithHandle(ObjectType: ULONG, cjSize: ULONG) -> POBJ;
    fn GDIOBJ_vUnlockObject(pobj: POBJ);
}

// Global GDI handle-table pointer; we deref through it but never reassign it.
extern "C" {
    static mut GdiHandleTable: PGDI_HANDLE_TABLE;
}

// cdecl shims (font_rust_shims.c) over the FORCEINLINE TEXTOBJ_LockText/UnlockText.
extern "C" {
    fn w32r_TEXTOBJ_LockText(hfont: HFONT) -> PTEXTOBJ;
    fn w32r_TEXTOBJ_UnlockText(pto: PTEXTOBJ);
}

// C macro: ObjToGDI(ClipObj, Type) = (Type##GDI *)(ClipObj). Only used with FONT here,
// so the type token is ignored and the value is a plain cast to PFONTGDI.
macro_rules! ObjToGDI {
    ($obj:expr, $ty:ident) => { ($obj) as PFONTGDI };
    ($obj:expr) => { ($obj) as PFONTGDI };
}

const GDI_OBJECT_TYPE_FONT: ULONG = 0x000a0000;
const GDI_HANDLE_COUNT: usize = 0x10000;
const GDI_HANDLE_INDEX_MASK: ULONG_PTR = (GDI_HANDLE_COUNT - 1) as ULONG_PTR;

// LFONT_* and GDI_HANDLE_GET_INDEX are C macros over GDIOBJ_* / the handle table.
// Expressed as inline fns (no bare-token args) so call sites need no `!`.
#[inline]
unsafe fn LFONT_AllocFontWithHandle() -> PLFONT {
    GDIOBJ_AllocObjWithHandle(GDI_OBJECT_TYPE_FONT, core::mem::size_of::<TEXTOBJ>() as ULONG) as PLFONT
}
#[inline]
unsafe fn LFONT_UnlockFont(p: PLFONT) {
    GDIOBJ_vUnlockObject(p as POBJ)
}
#[inline]
unsafe fn GDI_HANDLE_GET_INDEX(h: HGDIOBJ) -> ULONG_PTR {
    (h as ULONG_PTR) & GDI_HANDLE_INDEX_MASK
}

// ---- file-scope preamble (translated) ----
pub const UNICODE_NULL: u16 = 0;

// Pool tags — C multi-char literals are big-endian (e.g. 'ETNF' -> from_be_bytes).
pub const TAG_STRING: u32 = u32::from_be_bytes(*b" RTS"); // nolint: const-contradiction (' RTS' -- win32ss/user/ntuser/tags.h:3; freeldr's TAG_STRING is ' rtS')
pub const TAG_USTR: u32 = u32::from_be_bytes(*b"RTSU");
pub const TAG_FONT: u32 = u32::from_be_bytes(*b"ETNF");
pub const TAG_FINF: u32 = u32::from_be_bytes(*b"FNIF");
pub const GDITAG_TEXT: u32 = u32::from_be_bytes(*b"txtG");

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GreCreateFontIndirectW(lplf: *const LOGFONTW) -> HFONT {
    GreCreateFontIndirectW_impl(lplf)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GreCreateFontIndirectW(lplf: *const LOGFONTW) -> HFONT {
    GreCreateFontIndirectW_impl(lplf)
}

unsafe fn GreCreateFontIndirectW_impl(lplf: *const LOGFONTW) -> HFONT {
    if !lplf.is_null() {
        let mut Logfont: ENUMLOGFONTEXDVW = core::mem::zeroed();

        RtlCopyMemory(
            core::ptr::addr_of_mut!(Logfont.elfEnumLogfontEx.elfLogFont) as PVOID,
            lplf as PVOID,
            core::mem::size_of::<LOGFONTW>() as ULONG as _ ,
        );
        RtlZeroMemory(
            core::ptr::addr_of_mut!(Logfont.elfEnumLogfontEx.elfFullName) as PVOID,
            core::mem::size_of_val(&Logfont.elfEnumLogfontEx.elfFullName) as ULONG as _ ,
        );
        RtlZeroMemory(
            core::ptr::addr_of_mut!(Logfont.elfEnumLogfontEx.elfStyle) as PVOID,
            core::mem::size_of_val(&Logfont.elfEnumLogfontEx.elfStyle) as ULONG as _ ,
        );
        RtlZeroMemory(
            core::ptr::addr_of_mut!(Logfont.elfEnumLogfontEx.elfScript) as PVOID,
            core::mem::size_of_val(&Logfont.elfEnumLogfontEx.elfScript) as ULONG as _ ,
        );

        Logfont.elfDesignVector.dvNumAxes = 0;

        RtlZeroMemory(
            core::ptr::addr_of_mut!(Logfont.elfDesignVector) as PVOID,
            core::mem::size_of::<DESIGNVECTOR>() as ULONG as _ ,
        );

        HfontCreate(&Logfont, 0, 0, 0, core::ptr::null_mut())
    } else {
        core::ptr::null_mut()
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GreGetKerningPairs(
    hDC: HDC,
    NumPairs: ULONG,
    krnpair: LPKERNINGPAIR,
) -> DWORD {
    GreGetKerningPairs_impl(hDC, NumPairs, krnpair)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GreGetKerningPairs(
    hDC: HDC,
    NumPairs: ULONG,
    krnpair: LPKERNINGPAIR,
) -> DWORD {
    GreGetKerningPairs_impl(hDC, NumPairs, krnpair)
}

unsafe fn GreGetKerningPairs_impl(
    hDC: HDC,
    NumPairs: ULONG,
    krnpair: LPKERNINGPAIR,
) -> DWORD {
    let dc: PDC;
    let pdcattr: PDC_ATTR;
    let TextObj: PTEXTOBJ;
    let FontGDI: PFONTGDI;
    let Count: DWORD;
    let pKP: *mut KERNINGPAIR;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    pdcattr = (*dc).pdcattr;
    TextObj = RealizeFontInit((*pdcattr).hlfntNew as _ );
    DC_UnlockDc(dc);

    if TextObj.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    FontGDI = ObjToGDI!((*TextObj).Font as *mut core::ffi::c_void, FONT);
    w32r_TEXTOBJ_UnlockText(TextObj);

    Count = ftGdiGetKerningPairs(FontGDI, 0, core::ptr::null_mut());

    if Count != 0 && !krnpair.is_null() {
        if Count > NumPairs {
            EngSetLastError(ERROR_INSUFFICIENT_BUFFER);
            return 0;
        }
        pKP = ExAllocatePoolWithTag(
            PagedPool,
            Count as usize * core::mem::size_of::<KERNINGPAIR>(),
            GDITAG_TEXT,
        ) as *mut KERNINGPAIR;
        if pKP.is_null() {
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
        ftGdiGetKerningPairs(FontGDI, Count, pKP);

        RtlCopyMemory(
            krnpair as *mut core::ffi::c_void,
            pKP as *const core::ffi::c_void,
            Count as usize * core::mem::size_of::<KERNINGPAIR>(),
        );

        ExFreePoolWithTag(pKP as *mut core::ffi::c_void, GDITAG_TEXT);
    }
    Count
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn FontGetObject(
    plfont: PTEXTOBJ,
    mut cjBuffer: ULONG,
    pvBuffer: *mut core::ffi::c_void,
) -> ULONG {
    FontGetObject_impl(plfont, cjBuffer, pvBuffer)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn FontGetObject(
    plfont: PTEXTOBJ,
    cjBuffer: ULONG,
    pvBuffer: *mut core::ffi::c_void,
) -> ULONG {
    FontGetObject_impl(plfont, cjBuffer, pvBuffer)
}

unsafe fn FontGetObject_impl(
    plfont: PTEXTOBJ,
    mut cjBuffer: ULONG,
    pvBuffer: *mut core::ffi::c_void,
) -> ULONG {
    let plf: *mut ENUMLOGFONTEXDVW = core::ptr::addr_of_mut!((*plfont).logfont);

    if ((*plfont).fl & TEXTOBJECT_INIT) == 0 {
        let _Status = TextIntRealizeFont((*plfont).BaseObject.hHmgr as _ , plfont);
    }

    if pvBuffer.is_null() {
        return core::mem::size_of::<LOGFONTW>() as ULONG;
    }

    let base = plf as usize;
    let field = core::ptr::addr_of!(
        (*plf).elfDesignVector.dvValues[(*plf).elfDesignVector.dvNumAxes as usize]
    ) as usize;
    let cjMaxSize = (field - base) as ULONG;

    if cjBuffer > cjMaxSize {
        cjBuffer = cjMaxSize;
    }

    RtlCopyMemory(pvBuffer, plf as *const core::ffi::c_void, cjBuffer as usize);

    cjBuffer
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetCharDimensions(
    hdc: HDC,
    ptm: PTEXTMETRICW,
    height: PDWORD,
) -> DWORD {
    IntGetCharDimensions_impl(hdc, ptm, height)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetCharDimensions(
    hdc: HDC,
    ptm: PTEXTMETRICW,
    height: PDWORD,
) -> DWORD {
    IntGetCharDimensions_impl(hdc, ptm, height)
}

unsafe fn IntGetCharDimensions_impl(
    hdc: HDC,
    ptm: PTEXTMETRICW,
    height: PDWORD,
) -> DWORD {
    let pdc: PDC;
    let pdcattr: PDC_ATTR;
    let TextObj: PTEXTOBJ;
    let mut sz: SIZE = core::mem::zeroed();
    let mut tmwi: TMW_INTERNAL = core::mem::zeroed();
    let Good: BOOL;

    static alphabet: [u16; 53] = [
        'a' as u16, 'b' as u16, 'c' as u16, 'd' as u16, 'e' as u16, 'f' as u16,
        'g' as u16, 'h' as u16, 'i' as u16, 'j' as u16, 'k' as u16, 'l' as u16,
        'm' as u16, 'n' as u16, 'o' as u16, 'p' as u16, 'q' as u16, 'r' as u16,
        's' as u16, 't' as u16, 'u' as u16, 'v' as u16, 'w' as u16, 'x' as u16,
        'y' as u16, 'z' as u16, 'A' as u16, 'B' as u16, 'C' as u16, 'D' as u16,
        'E' as u16, 'F' as u16, 'G' as u16, 'H' as u16, 'I' as u16, 'J' as u16,
        'K' as u16, 'L' as u16, 'M' as u16, 'N' as u16, 'O' as u16, 'P' as u16,
        'Q' as u16, 'R' as u16, 'S' as u16, 'T' as u16, 'U' as u16, 'V' as u16,
        'W' as u16, 'X' as u16, 'Y' as u16, 'Z' as u16, 0,
    ];

    if ftGdiGetTextMetricsW(hdc, &mut tmwi) == 0 {
        return 0;
    }

    pdc = DC_LockDc(hdc);

    if pdc.is_null() {
        return 0;
    }

    pdcattr = (*pdc).pdcattr;

    TextObj = RealizeFontInit((*pdcattr).hlfntNew as _ );
    if TextObj.is_null() {
        DC_UnlockDc(pdc);
        return 0;
    }

    Good = TextIntGetTextExtentPoint(
        pdc,
        TextObj,
        alphabet.as_ptr(),
        52,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut sz,
        0,
    );
    w32r_TEXTOBJ_UnlockText(TextObj);
    DC_UnlockDc(pdc);

    if Good == 0 {
        return 0;
    }
    if !ptm.is_null() {
        *ptm = tmwi.TextMetric;
    }
    if !height.is_null() {
        *height = tmwi.TextMetric.tmHeight as DWORD;
    }

    ((sz.cx / 26 + 1) / 2) as DWORD
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetFontLanguageInfo(Dc: PDC) -> u32 {
    IntGetFontLanguageInfo_impl(Dc)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetFontLanguageInfo(Dc: PDC) -> u32 {
    IntGetFontLanguageInfo_impl(Dc)
}

unsafe fn IntGetFontLanguageInfo_impl(Dc: PDC) -> u32 {

    const GCP_DBCS_MASK: u32 = 0x003F0000;
    const GCP_DIACRITIC_MASK: u32 = 0x00000000;
    const FLI_GLYPHS_MASK: u32 = 0x00000000;
    const GCP_GLYPHSHAPE_MASK: u32 = 0x00000040;
    const GCP_KASHIDA_MASK: u32 = 0x00000000;
    const GCP_LIGATE_MASK: u32 = 0x00000000;
    const GCP_USEKERNING_MASK: u32 = 0x00000000;
    const GCP_REORDER_MASK: u32 = 0x00000060;

    let pdcattr: PDC_ATTR;
    let mut fontsig: FONTSIGNATURE = core::mem::zeroed();
    let mut result: u32 = 0;

    ftGdiGetTextCharsetInfo(Dc, &mut fontsig, 0);

    if (fontsig.fsCsb[0] & GCP_DBCS_MASK) != 0 {
        result |= GCP_DBCS;
    }

    if (fontsig.fsCsb[0] & GCP_DIACRITIC_MASK) != 0 {
        result |= GCP_DIACRITIC;
    }

    if (fontsig.fsCsb[0] & FLI_GLYPHS_MASK) != 0 {
        result |= FLI_GLYPHS;
    }

    if (fontsig.fsCsb[0] & GCP_GLYPHSHAPE_MASK) != 0 {
        result |= GCP_GLYPHSHAPE;
    }

    if (fontsig.fsCsb[0] & GCP_KASHIDA_MASK) != 0 {
        result |= GCP_KASHIDA;
    }

    if (fontsig.fsCsb[0] & GCP_LIGATE_MASK) != 0 {
        result |= GCP_LIGATE;
    }

    if (fontsig.fsCsb[0] & GCP_USEKERNING_MASK) != 0 {
        result |= GCP_USEKERNING;
    }

    pdcattr = (*Dc).pdcattr;

    if (*pdcattr).flTextAlign & TA_RTLREADING != 0 {
        if (fontsig.fsCsb[0] & GCP_REORDER_MASK) != 0 {
            result |= GCP_REORDER;
        }
    }

    result
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn RealizeFontInit(hFont: HFONT) -> PTEXTOBJ {
    RealizeFontInit_impl(hFont)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn RealizeFontInit(hFont: HFONT) -> PTEXTOBJ {
    RealizeFontInit_impl(hFont)
}

unsafe fn RealizeFontInit_impl(hFont: HFONT) -> PTEXTOBJ {
    let mut Status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;

    let pTextObj: PTEXTOBJ = w32r_TEXTOBJ_LockText(hFont);

    if !pTextObj.is_null() && ((*pTextObj).fl & TEXTOBJECT_INIT) == 0 {
        Status = TextIntRealizeFont(hFont, pTextObj);
        if !NT_SUCCESS(Status) {
            w32r_TEXTOBJ_UnlockText(pTextObj);
            return core::ptr::null_mut();
        }
    }
    pTextObj
}

unsafe fn IntCheckFontPathNames(
    pwcFiles: PCWCH,
    cFiles: ULONG,
    cwc: ULONG,
) -> BOOL {
    let mut ich: ULONG;
    let mut cRealFiles: ULONG;

    if *pwcFiles.add((cwc - 1) as usize) != UNICODE_NULL as u16 {
        return 0;
    }

    ich = 0;
    cRealFiles = 0;
    while ich < cwc {
        if *pwcFiles.add(ich as usize) == 0 {
            cRealFiles += 1;
        }
        ich += 1;
    }

    (cRealFiles >= cFiles) as BOOL
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiAddFontResourceW(
    pwcFiles: PCWCH,
    cwc: ULONG,
    cFiles: ULONG,
    fl: FLONG,
    dwPidTid: DWORD,
    pdv: *const DESIGNVECTOR,
) -> INT {
    let mut SafeFileName: _UNICODE_STRING = core::mem::zeroed();
    let Ret: INT;

    if (cwc <= 1) || (cwc > UNICODE_STRING_MAX_CHARS) {
        return 0;
    }

    SafeFileName.Length = ((cwc - 1) * core::mem::size_of::<u16>() as ULONG) as USHORT;
    SafeFileName.MaximumLength =
        SafeFileName.Length + core::mem::size_of::<u16>() as USHORT;
    SafeFileName.Buffer = ExAllocatePoolWithTag(
        PagedPool,
        (SafeFileName.MaximumLength as SIZE_T as _),
        TAG_STRING,
    ) as *mut u16;
    if SafeFileName.Buffer.is_null() {
        return 0;
    }

    // The C probes pwcFiles here — ProbeForRead(pwcFiles, cwc * sizeof(WCHAR),
    // sizeof(WCHAR)) — inside a _SEH2_TRY whose handler frees the buffer and returns 0.
    // That probe was dropped, so both IntCheckFontPathNames (which WALKS the string)
    // and the RtlCopyMemory below ran straight off the caller's raw pointer. Under
    // DUMMY_PSEH nothing catches the resulting fault, so validate first; alignment 2
    // matches the C's sizeof(WCHAR).
    //
    // IntCheckFontPathNames was also being called TWICE here, the first result thrown
    // away into an empty `if` whose condition (`!f(..) != 0`) could not have done
    // anything either way — a translation artifact, not a second check. One call, as
    // the C has, so the user buffer is walked once.
    //
    // Both failure exits now free SafeFileName.Buffer. The C leaks it on the
    // IntCheckFontPathNames path (a bare `return` out of _SEH2_TRY, so the handler
    // that frees never runs) — reachable from user mode with an attacker-chosen cwc,
    // which makes it a paged-pool exhaustion primitive rather than a stray leak.
    if !NT_SUCCESS(SafeProbeForRead(
        pwcFiles as PVOID,
        (cwc as usize) * core::mem::size_of::<u16>(),
        core::mem::size_of::<u16>() as ULONG,
    )) {
        ExFreePoolWithTag(SafeFileName.Buffer as PVOID, TAG_STRING);
        return 0;
    }

    // CAPTURE first, then validate the KERNEL copy. Upstream has the probe,
    // IntCheckFontPathNames and the copy in ONE _SEH2_TRY; our probe above is
    // validate-only, so the walk and the copy both had to stop touching user
    // memory. Validating the captured bytes also closes a double fetch the C
    // has: it walks the caller's buffer to approve the path, then copies that
    // buffer again, so another thread can swap the path in between.
    if !crate::copy::capture_from_user(
        SafeFileName.Buffer as *mut c_void,
        pwcFiles as *const c_void,
        SafeFileName.Length as usize,
    ) {
        ExFreePoolWithTag(SafeFileName.Buffer as PVOID, TAG_STRING);
        return 0;
    }

    *SafeFileName
        .Buffer
        .add((SafeFileName.Length as usize) / core::mem::size_of::<u16>()) = UNICODE_NULL as u16;

    if IntCheckFontPathNames(SafeFileName.Buffer as PCWCH, cFiles, cwc) == 0 {
        ExFreePoolWithTag(SafeFileName.Buffer as PVOID, TAG_STRING);
        return 0;
    }
    Ret = IntGdiAddFontResourceEx(&mut SafeFileName, cFiles, fl, 0);
    ExFreePoolWithTag(SafeFileName.Buffer as PVOID, TAG_STRING);
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiAddFontResourceW(
    pwcFiles: PCWCH,
    cwc: ULONG,
    cFiles: ULONG,
    fl: FLONG,
    dwPidTid: DWORD,
    pdv: *const DESIGNVECTOR,
) -> INT {
    let mut SafeFileName: _UNICODE_STRING = core::mem::zeroed();
    let Ret: INT;

    if (cwc <= 1) || (cwc > UNICODE_STRING_MAX_CHARS) {
        return 0;
    }

    SafeFileName.Length = ((cwc - 1) * core::mem::size_of::<u16>() as ULONG) as USHORT;
    SafeFileName.MaximumLength =
        SafeFileName.Length + core::mem::size_of::<u16>() as USHORT;
    SafeFileName.Buffer = ExAllocatePoolWithTag(
        PagedPool,
        SafeFileName.MaximumLength as SIZE_T as _ ,
        TAG_STRING,
    ) as *mut u16;
    if SafeFileName.Buffer.is_null() {
        return 0;
    }

    // The C probes pwcFiles here — ProbeForRead(pwcFiles, cwc * sizeof(WCHAR),
    // sizeof(WCHAR)) — inside a _SEH2_TRY whose handler frees the buffer and returns 0.
    // That probe was dropped, so both IntCheckFontPathNames (which WALKS the string)
    // and the RtlCopyMemory below ran straight off the caller's raw pointer. Under
    // DUMMY_PSEH nothing catches the resulting fault, so validate first; alignment 2
    // matches the C's sizeof(WCHAR).
    //
    // IntCheckFontPathNames was also being called TWICE here, the first result thrown
    // away into an empty `if` whose condition (`!f(..) != 0`) could not have done
    // anything either way — a translation artifact, not a second check. One call, as
    // the C has, so the user buffer is walked once.
    //
    // Both failure exits now free SafeFileName.Buffer. The C leaks it on the
    // IntCheckFontPathNames path (a bare `return` out of _SEH2_TRY, so the handler
    // that frees never runs) — reachable from user mode with an attacker-chosen cwc,
    // which makes it a paged-pool exhaustion primitive rather than a stray leak.
    if !NT_SUCCESS(SafeProbeForRead(
        pwcFiles as PVOID,
        (cwc as usize) * core::mem::size_of::<u16>(),
        core::mem::size_of::<u16>() as ULONG,
    )) {
        ExFreePoolWithTag(SafeFileName.Buffer as PVOID, TAG_STRING);
        return 0;
    }

    // CAPTURE first, then validate the KERNEL copy. Upstream has the probe,
    // IntCheckFontPathNames and the copy in ONE _SEH2_TRY; our probe above is
    // validate-only, so the walk and the copy both had to stop touching user
    // memory. Validating the captured bytes also closes a double fetch the C
    // has: it walks the caller's buffer to approve the path, then copies that
    // buffer again, so another thread can swap the path in between.
    if !crate::copy::capture_from_user(
        SafeFileName.Buffer as *mut c_void,
        pwcFiles as *const c_void,
        SafeFileName.Length as usize,
    ) {
        ExFreePoolWithTag(SafeFileName.Buffer as PVOID, TAG_STRING);
        return 0;
    }

    *SafeFileName
        .Buffer
        .add((SafeFileName.Length as usize) / core::mem::size_of::<u16>()) = UNICODE_NULL as u16;

    if IntCheckFontPathNames(SafeFileName.Buffer as PCWCH, cFiles, cwc) == 0 {
        ExFreePoolWithTag(SafeFileName.Buffer as PVOID, TAG_STRING);
        return 0;
    }
    Ret = IntGdiAddFontResourceEx(&mut SafeFileName, cFiles, fl, 0);
    ExFreePoolWithTag(SafeFileName.Buffer as PVOID, TAG_STRING);
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiRemoveFontResourceW(
    pwszFiles: PCWCH,
    cwc: ULONG,
    cFiles: ULONG,
    fl: ULONG,
    dwPidTid: DWORD,
    pdv: *const DESIGNVECTOR,
) -> BOOL {
    let mut SafeFileName: _UNICODE_STRING = core::mem::zeroed();
    let Ret: BOOL;

    let _ = dwPidTid;
    let _ = pdv;

    /* cwc = Length + trailing zero. */
    if (cwc <= 1) || (cwc > UNICODE_STRING_MAX_CHARS) {
        return 0;
    }

    SafeFileName.Length = ((cwc - 1) * core::mem::size_of::<u16>() as ULONG) as USHORT;
    SafeFileName.MaximumLength =
        SafeFileName.Length + core::mem::size_of::<u16>() as USHORT;
    SafeFileName.Buffer = ExAllocatePoolWithTag(
        PagedPool,
        SafeFileName.MaximumLength as usize,
        TAG_STRING,
    ) as *mut u16;
    if SafeFileName.Buffer.is_null() {
        return 0;
    }

    // Same dropped probe as NtGdiAddFontResourceW above, same shape: the C wraps this
    // in _SEH2_TRY after ProbeForRead(pwszFiles, cwc * sizeof(WCHAR), sizeof(WCHAR)),
    // and both IntCheckFontPathNames and the RtlCopyMemory below read the caller's
    // buffer. Validate first, and free on the failure path the C leaks.
    if !NT_SUCCESS(SafeProbeForRead(
        pwszFiles as PVOID,
        (cwc as usize) * core::mem::size_of::<u16>(),
        core::mem::size_of::<u16>() as ULONG,
    )) {
        ExFreePoolWithTag(SafeFileName.Buffer as PVOID, TAG_STRING);
        return 0;
    }

    // CAPTURE first, then validate the KERNEL copy. Upstream has the probe,
    // IntCheckFontPathNames and the copy in ONE _SEH2_TRY; our probe above is
    // validate-only, so the walk and the copy both had to stop touching user
    // memory. Validating the captured bytes also closes a double fetch the C
    // has: it walks the caller's buffer to approve the path, then copies that
    // buffer again, so another thread can swap the path in between.
    if !crate::copy::capture_from_user(
        SafeFileName.Buffer as *mut c_void,
        pwszFiles as *const c_void,
        SafeFileName.Length as usize,
    ) {
        ExFreePoolWithTag(SafeFileName.Buffer as PVOID, TAG_STRING);
        return 0;
    }

    *SafeFileName
        .Buffer
        .add((SafeFileName.Length as usize) / core::mem::size_of::<u16>()) = UNICODE_NULL as u16;

    if IntCheckFontPathNames(SafeFileName.Buffer as PCWCH, cFiles, cwc) == 0 {
        ExFreePoolWithTag(SafeFileName.Buffer as PVOID, TAG_STRING);
        return 0;
    }

    Ret = IntGdiRemoveFontResource(&mut SafeFileName, cFiles, fl);

    ExFreePoolWithTag(SafeFileName.Buffer as PVOID, TAG_STRING);

    Ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiAddFontMemResourceEx(
    pvBuffer: *const core::ffi::c_void,
    cjBuffer: DWORD,
    pdv: *const DESIGNVECTOR,
    cjDV: ULONG,
    pNumFonts: PDWORD,
) -> HANDLE {
    let mut Buffer: PVOID = core::ptr::null_mut();
    let Ret: HANDLE;
    let mut NumFonts: DWORD = 0;

    let _ = pdv;
    let _ = cjDV;

    if pvBuffer.is_null() || cjBuffer == 0 {
        return core::ptr::null_mut();
    }

    // C: _SEH2_TRY { ProbeForRead(pvBuffer, cjBuffer, 1); Buffer = ExAllocatePoolWithQuotaTag(..);
    // RtlCopyMemory(Buffer, pvBuffer, cjBuffer); } _SEH2_EXCEPT { free Buffer; return NULL; }
    // The quota allocation RAISES on failure in the C (caught by that handler);
    // without SEH ask it to fail instead, and copy fault-tolerantly.
    Buffer = ExAllocatePoolWithQuotaTag(
        (PagedPool as u32 | POOL_QUOTA_FAIL_INSTEAD_OF_RAISE) as _,
        cjBuffer as usize as _,
        TAG_FONT,
    );
    if Buffer.is_null() {
        return core::ptr::null_mut();
    }
    if crate::safe_user_io::copy_from_user(Buffer, pvBuffer, cjBuffer as usize, 1).is_err() {
        ExFreePoolWithTag(Buffer, TAG_FONT);
        return core::ptr::null_mut();
    }

    Ret = IntGdiAddFontMemResource(Buffer, cjBuffer, &mut NumFonts);
    ExFreePoolWithTag(Buffer, TAG_FONT);

    {
        let __SehStatus = SafeProbeForWrite(
            pNumFonts as PVOID,
            core::mem::size_of::<DWORD>(),
            1,
        );
        if !NT_SUCCESS(__SehStatus) {
            return core::ptr::null_mut();
        }
    }
    // Probe is validate-only; the C had probe AND store in one _SEH2_TRY.
    if !crate::copy::publish_to_user(
        pNumFonts as *mut core::ffi::c_void,
        core::ptr::addr_of!(NumFonts) as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>(),
    ) {
        return core::ptr::null_mut();
    }
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiAddFontMemResourceEx(
    pvBuffer: *const core::ffi::c_void,
    cjBuffer: DWORD,
    pdv: *const DESIGNVECTOR,
    cjDV: ULONG,
    pNumFonts: PDWORD,
) -> HANDLE {
    let mut Buffer: PVOID = core::ptr::null_mut();
    let Ret: HANDLE;
    let mut NumFonts: DWORD = 0;

    let _ = pdv;
    let _ = cjDV;

    if pvBuffer.is_null() || cjBuffer == 0 {
        return core::ptr::null_mut();
    }

    // C: _SEH2_TRY { ProbeForRead(pvBuffer, cjBuffer, 1); Buffer = ExAllocatePoolWithQuotaTag(..);
    // RtlCopyMemory(Buffer, pvBuffer, cjBuffer); } _SEH2_EXCEPT { free Buffer; return NULL; }
    // The quota allocation RAISES on failure in the C (caught by that handler);
    // without SEH ask it to fail instead, and copy fault-tolerantly.
    Buffer = ExAllocatePoolWithQuotaTag(
        (PagedPool as u32 | POOL_QUOTA_FAIL_INSTEAD_OF_RAISE) as _,
        cjBuffer as usize as _,
        TAG_FONT,
    );
    if Buffer.is_null() {
        return core::ptr::null_mut();
    }
    if crate::safe_user_io::copy_from_user(Buffer, pvBuffer, cjBuffer as usize, 1).is_err() {
        ExFreePoolWithTag(Buffer, TAG_FONT);
        return core::ptr::null_mut();
    }

    Ret = IntGdiAddFontMemResource(Buffer, cjBuffer, &mut NumFonts);
    ExFreePoolWithTag(Buffer, TAG_FONT);

    {
        let __SehStatus = SafeProbeForWrite(
            pNumFonts as PVOID,
            core::mem::size_of::<DWORD>(),
            1,
        );
        if !NT_SUCCESS(__SehStatus) {
            return core::ptr::null_mut();
        }
    }
    // Probe is validate-only; the C had probe AND store in one _SEH2_TRY.
    if !crate::copy::publish_to_user(
        pNumFonts as *mut core::ffi::c_void,
        core::ptr::addr_of!(NumFonts) as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>(),
    ) {
        return core::ptr::null_mut();
    }
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiRemoveFontMemResourceEx(hMMFont: HANDLE) -> BOOL {
    IntGdiRemoveFontMemResource(hMMFont)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetCharacterPlacementW(
    hdc: HDC,
    pwsz: PCWCH,
    nCount: INT,
    nMaxExtent: INT,
    pgcpw: LPGCP_RESULTSW,
    dwFlags: DWORD,
) -> DWORD {
    0
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetFontData(
    hDC: HDC,
    Table: DWORD,
    Offset: DWORD,
    Buffer: PVOID,
    Size: DWORD,
) -> DWORD {
    let Dc: PDC;
    let pdcattr: PDC_ATTR;
    let hFont: HFONT;
    let TextObj: PTEXTOBJ;
    let FontGdi: PFONTGDI;
    let mut Result: DWORD = GDI_ERROR;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if !Buffer.is_null() && Size != 0 {
        Status = SafeProbeForRead(Buffer, Size as usize, 1);
    }

    if !NT_SUCCESS(Status) {
        return Result;
    }

    Dc = DC_LockDc(hDC);
    if Dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return GDI_ERROR;
    }
    pdcattr = (*Dc).pdcattr;

    hFont = ((*pdcattr).hlfntNew as _);
    TextObj = RealizeFontInit(hFont);
    DC_UnlockDc(Dc);

    if TextObj.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return GDI_ERROR;
    }

    FontGdi = ObjToGDI!((*TextObj).Font);

    Result = ftGdiGetFontData(FontGdi, Table, Offset, Buffer, Size);

    w32r_TEXTOBJ_UnlockText(TextObj);

    Result
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetFontData(
    hDC: HDC,
    Table: DWORD,
    Offset: DWORD,
    Buffer: PVOID,
    Size: DWORD,
) -> DWORD {
    let Dc: PDC;
    let pdcattr: PDC_ATTR;
    let hFont: HFONT;
    let TextObj: PTEXTOBJ;
    let FontGdi: PFONTGDI;
    let mut Result: DWORD = GDI_ERROR;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if !Buffer.is_null() && Size != 0 {
        Status = SafeProbeForRead(Buffer, Size as usize, 1);
    }

    if !NT_SUCCESS(Status) {
        return Result;
    }

    Dc = DC_LockDc(hDC);
    if Dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return GDI_ERROR;
    }
    pdcattr = (*Dc).pdcattr;

    hFont = (*pdcattr).hlfntNew as _ ;
    TextObj = RealizeFontInit(hFont);
    DC_UnlockDc(Dc);

    if TextObj.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return GDI_ERROR;
    }

    FontGdi = ObjToGDI!((*TextObj).Font);

    Result = ftGdiGetFontData(FontGdi, Table, Offset, Buffer, Size);

    w32r_TEXTOBJ_UnlockText(TextObj);

    Result
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetFontUnicodeRanges(hdc: HDC, pgs: LPGLYPHSET) -> DWORD {
    let pDc: PDC;
    let pdcattr: PDC_ATTR;
    let hFont: HFONT;
    let mut TextObj: PTEXTOBJ;
    let FontGdi: PFONTGDI;
    let mut Size: DWORD = 0;
    let pgsSafe: PGLYPHSET;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    pDc = DC_LockDc(hdc);
    if pDc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    pdcattr = (*pDc).pdcattr;

    hFont = ((*pdcattr).hlfntNew as _);
    TextObj = RealizeFontInit(hFont);

    'exit: {
        if TextObj.is_null() {
            EngSetLastError(ERROR_INVALID_HANDLE);
            break 'exit;
        }
        FontGdi = ObjToGDI!((*TextObj).Font, FONT);

        Size = ftGetFontUnicodeRanges(FontGdi, core::ptr::null_mut());

        if Size != 0 && !pgs.is_null() {
            pgsSafe = ExAllocatePoolWithTag(PagedPool, Size as usize, GDITAG_TEXT) as PGLYPHSET;
            if pgsSafe.is_null() {
                EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                Size = 0;
                break 'exit;
            }

            Size = ftGetFontUnicodeRanges(FontGdi, pgsSafe);

            if Size != 0 {
                Status = SafeProbeForWrite(pgs as *mut core::ffi::c_void, (Size as _), 1);
                // Validate-only probe; the copy is the access. A copy fault lands in Status,
                // which is read below -- upstream's handler does exactly this.
                if NT_SUCCESS(Status)
                    && !crate::copy::publish_to_user(pgs as *mut core::ffi::c_void, pgsSafe as *mut core::ffi::c_void, Size as usize)
                {
                    Status = STATUS_ACCESS_VIOLATION;
                }

                if !NT_SUCCESS(Status) {
                    Size = 0;
                }
            }
            ExFreePoolWithTag(pgsSafe as *mut core::ffi::c_void, GDITAG_TEXT);
        }
    }

    w32r_TEXTOBJ_UnlockText(TextObj);
    DC_UnlockDc(pDc);
    Size
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetFontUnicodeRanges(hdc: HDC, pgs: LPGLYPHSET) -> DWORD {
    let pDc: PDC;
    let pdcattr: PDC_ATTR;
    let hFont: HFONT;
    let mut TextObj: PTEXTOBJ;
    let FontGdi: PFONTGDI;
    let mut Size: DWORD = 0;
    let pgsSafe: PGLYPHSET;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    pDc = DC_LockDc(hdc);
    if pDc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    pdcattr = (*pDc).pdcattr;

    hFont = (*pdcattr).hlfntNew as _ ;
    TextObj = RealizeFontInit(hFont);

    'exit: {
        if TextObj.is_null() {
            EngSetLastError(ERROR_INVALID_HANDLE);
            break 'exit;
        }
        FontGdi = ObjToGDI!((*TextObj).Font, FONT);

        Size = ftGetFontUnicodeRanges(FontGdi, core::ptr::null_mut());

        if Size != 0 && !pgs.is_null() {
            pgsSafe = ExAllocatePoolWithTag(PagedPool, Size as usize, GDITAG_TEXT) as PGLYPHSET;
            if pgsSafe.is_null() {
                EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                Size = 0;
                break 'exit;
            }

            Size = ftGetFontUnicodeRanges(FontGdi, pgsSafe);

            if Size != 0 {
                Status = SafeProbeForWrite(pgs as *mut core::ffi::c_void, Size as _ , 1);
                // Validate-only probe; the copy is the access. A copy fault lands in Status,
                // which is read below -- upstream's handler does exactly this.
                if NT_SUCCESS(Status)
                    && !crate::copy::publish_to_user(pgs as *mut core::ffi::c_void, pgsSafe as *mut core::ffi::c_void, Size as usize)
                {
                    Status = STATUS_ACCESS_VIOLATION;
                }

                if !NT_SUCCESS(Status) {
                    Size = 0;
                }
            }
            ExFreePoolWithTag(pgsSafe as *mut core::ffi::c_void, GDITAG_TEXT);
        }
    }

    w32r_TEXTOBJ_UnlockText(TextObj);
    DC_UnlockDc(pDc);
    Size
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetGlyphOutline(
    hdc: HDC,
    wch: u16,
    iFormat: u32,
    pgm: LPGLYPHMETRICS,
    cjBuf: u32,
    UnsafeBuf: *mut core::ffi::c_void,
    pmat2: *const MAT2,
    bIgnoreRotation: i32,
) -> u32 {


    let mut Ret: u32 = GDI_ERROR;
    let dc: PDC;
    let mut pvBuf: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut gm: GLYPHMETRICS = core::mem::zeroed();
    let mut Status: i32 = STATUS_SUCCESS;

    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return GDI_ERROR;
    }

    'exit: {
        if !UnsafeBuf.is_null() && cjBuf != 0 {
            pvBuf = ExAllocatePoolZero(PagedPool, (cjBuf as u64 as _), GDITAG_TEXT);
            if pvBuf.is_null() {
                EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'exit;
            }
        }

        Ret = ftGdiGetGlyphOutline(
            dc,
            wch,
            iFormat,
            if !pgm.is_null() {
                core::ptr::addr_of_mut!(gm)
            } else {
                core::ptr::null_mut()
            },
            cjBuf,
            pvBuf,
            pmat2,
            bIgnoreRotation,
        );

        if !pvBuf.is_null() {
            Status = SafeProbeForWrite(UnsafeBuf, (cjBuf as _), 1);
            // Validate-only probe; the copy is the access. A copy fault lands in Status,
            // which is read below -- upstream's handler does exactly this.
            if NT_SUCCESS(Status)
                && !crate::copy::publish_to_user(UnsafeBuf, pvBuf, cjBuf as usize)
            {
                Status = STATUS_ACCESS_VIOLATION;
            }

            ExFreePoolWithTag(pvBuf, GDITAG_TEXT);
        }

        if !pgm.is_null() {
            Status = SafeProbeForWrite(
                pgm as *mut core::ffi::c_void,
                (core::mem::size_of::<GLYPHMETRICS>() as u32 as _),
                1,
            );
            // Validate-only probe; the copy is the access. A copy fault lands in Status,
            // which is read below -- upstream's handler does exactly this.
            if NT_SUCCESS(Status)
                && !crate::copy::publish_to_user(pgm as *mut core::ffi::c_void, core::ptr::addr_of!(gm) as *const core::ffi::c_void, core::mem::size_of::<GLYPHMETRICS>())
            {
                Status = STATUS_ACCESS_VIOLATION;
            }
        }

        if !NT_SUCCESS(Status) {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            Ret = GDI_ERROR;
        }
    }

    DC_UnlockDc(dc);
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetGlyphOutline(
    hdc: HDC,
    wch: u16,
    iFormat: u32,
    pgm: LPGLYPHMETRICS,
    cjBuf: u32,
    UnsafeBuf: *mut core::ffi::c_void,
    pmat2: *const MAT2,
    bIgnoreRotation: i32,
) -> u32 {


    let mut Ret: u32 = GDI_ERROR;
    let dc: PDC;
    let mut pvBuf: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut gm: GLYPHMETRICS = core::mem::zeroed();
    let mut Status: i32 = STATUS_SUCCESS;

    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return GDI_ERROR;
    }

    'exit: {
        if !UnsafeBuf.is_null() && cjBuf != 0 {
            pvBuf = ExAllocatePoolZero(PagedPool, cjBuf as usize, GDITAG_TEXT);
            if pvBuf.is_null() {
                EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'exit;
            }
        }

        Ret = ftGdiGetGlyphOutline(
            dc,
            wch,
            iFormat,
            if !pgm.is_null() {
                core::ptr::addr_of_mut!(gm)
            } else {
                core::ptr::null_mut()
            },
            cjBuf,
            pvBuf,
            pmat2,
            bIgnoreRotation,
        );

        if !pvBuf.is_null() {
            Status = SafeProbeForWrite(UnsafeBuf, cjBuf as _ , 1);
            // Validate-only probe; the copy is the access. A copy fault lands in Status,
            // which is read below -- upstream's handler does exactly this.
            if NT_SUCCESS(Status)
                && !crate::copy::publish_to_user(UnsafeBuf, pvBuf, cjBuf as usize)
            {
                Status = STATUS_ACCESS_VIOLATION;
            }

            ExFreePoolWithTag(pvBuf, GDITAG_TEXT);
        }

        if !pgm.is_null() {
            Status = SafeProbeForWrite(
                pgm as *mut core::ffi::c_void,
                core::mem::size_of::<GLYPHMETRICS>() as u32 as _ ,
                1,
            );
            // Validate-only probe; the copy is the access. A copy fault lands in Status,
            // which is read below -- upstream's handler does exactly this.
            if NT_SUCCESS(Status)
                && !crate::copy::publish_to_user(pgm as *mut core::ffi::c_void, core::ptr::addr_of!(gm) as *const core::ffi::c_void, core::mem::size_of::<GLYPHMETRICS>())
            {
                Status = STATUS_ACCESS_VIOLATION;
            }
        }

        if !NT_SUCCESS(Status) {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            Ret = GDI_ERROR;
        }
    }

    DC_UnlockDc(dc);
    Ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetKerningPairs(
    hDC: HDC,
    NumPairs: ULONG,
    krnpair: LPKERNINGPAIR,
) -> DWORD {
    let dc: PDC;
    let pdcattr: PDC_ATTR;
    let TextObj: PTEXTOBJ;
    let FontGDI: PFONTGDI;
    let mut Count: DWORD;
    let pKP: *mut KERNINGPAIR;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    pdcattr = (*dc).pdcattr;
    TextObj = RealizeFontInit(((*pdcattr).hlfntNew as _));
    DC_UnlockDc(dc);

    if TextObj.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    FontGDI = ObjToGDI!((*TextObj).Font) as PFONTGDI;
    w32r_TEXTOBJ_UnlockText(TextObj);

    Count = ftGdiGetKerningPairs(FontGDI, 0, core::ptr::null_mut());

    if Count != 0 && !krnpair.is_null() {
        if Count > NumPairs {
            EngSetLastError(ERROR_INSUFFICIENT_BUFFER);
            return 0;
        }
        pKP = ExAllocatePoolWithTag(
            PagedPool,
            (Count as usize) * core::mem::size_of::<KERNINGPAIR>(),
            GDITAG_TEXT,
        ) as *mut KERNINGPAIR;
        if pKP.is_null() {
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
        ftGdiGetKerningPairs(FontGDI, Count, pKP);
        Status = SafeProbeForWrite(
            krnpair as *mut core::ffi::c_void,
            (Count as usize) * core::mem::size_of::<KERNINGPAIR>(),
            1,
        );
        // Validate-only probe; the copy is the access. A copy fault lands in Status,
        // which is read below -- upstream's handler does exactly this.
        if NT_SUCCESS(Status)
            && !crate::copy::publish_to_user(krnpair as *mut core::ffi::c_void, pKP as *const core::ffi::c_void, (Count as usize) * core::mem::size_of::<KERNINGPAIR>())
        {
            Status = STATUS_ACCESS_VIOLATION;
        }
        if !NT_SUCCESS(Status) {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            Count = 0;
        }
        ExFreePoolWithTag(pKP as *mut core::ffi::c_void, GDITAG_TEXT);
    }
    Count
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetKerningPairs(
    hDC: HDC,
    NumPairs: ULONG,
    krnpair: LPKERNINGPAIR,
) -> DWORD {
    let dc: PDC;
    let pdcattr: PDC_ATTR;
    let TextObj: PTEXTOBJ;
    let FontGDI: PFONTGDI;
    let mut Count: DWORD;
    let pKP: *mut KERNINGPAIR;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    pdcattr = (*dc).pdcattr;
    TextObj = RealizeFontInit((*pdcattr).hlfntNew as _ );
    DC_UnlockDc(dc);

    if TextObj.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    FontGDI = ObjToGDI!((*TextObj).Font) as PFONTGDI;
    w32r_TEXTOBJ_UnlockText(TextObj);

    Count = ftGdiGetKerningPairs(FontGDI, 0, core::ptr::null_mut());

    if Count != 0 && !krnpair.is_null() {
        if Count > NumPairs {
            EngSetLastError(ERROR_INSUFFICIENT_BUFFER);
            return 0;
        }
        pKP = ExAllocatePoolWithTag(
            PagedPool,
            (Count as usize) * core::mem::size_of::<KERNINGPAIR>(),
            GDITAG_TEXT,
        ) as *mut KERNINGPAIR;
        if pKP.is_null() {
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
        ftGdiGetKerningPairs(FontGDI, Count, pKP);
        Status = SafeProbeForWrite(
            krnpair as *mut core::ffi::c_void,
            (Count as usize) * core::mem::size_of::<KERNINGPAIR>(),
            1,
        );
        // Validate-only probe; the copy is the access. A copy fault lands in Status,
        // which is read below -- upstream's handler does exactly this.
        if NT_SUCCESS(Status)
            && !crate::copy::publish_to_user(krnpair as *mut core::ffi::c_void, pKP as *const core::ffi::c_void, (Count as usize) * core::mem::size_of::<KERNINGPAIR>())
        {
            Status = STATUS_ACCESS_VIOLATION;
        }
        if !NT_SUCCESS(Status) {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            Count = 0;
        }
        ExFreePoolWithTag(pKP as *mut core::ffi::c_void, GDITAG_TEXT);
    }
    Count
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetOutlineTextMetricsInternalW(
    hDC: HDC,
    Data: ULONG,
    otm: POUTLINETEXTMETRICW,
    Tmd: PTMDIFF,
) -> ULONG {
    let dc: PDC;
    let pdcattr: PDC_ATTR;
    let TextObj: PTEXTOBJ;
    let FontGDI: PFONTGDI;
    let mut hFont: HFONT = 0 as HFONT;
    let mut Size: ULONG;
    let potm: POUTLINETEXTMETRICW;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    pdcattr = (*dc).pdcattr;
    hFont = ((*pdcattr).hlfntNew as _);
    TextObj = RealizeFontInit(hFont);
    DC_UnlockDc(dc);
    if TextObj.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    FontGDI = ObjToGDI!((*TextObj).Font);
    if ((*FontGDI).flType & FO_TYPE_TRUETYPE as ULONG) == 0 {
        w32r_TEXTOBJ_UnlockText(TextObj);
        return 0;
    }
    TextIntUpdateSize(dc, TextObj, FontGDI, (1u8 as _));
    w32r_TEXTOBJ_UnlockText(TextObj);
    Size = ((IntGetOutlineTextMetrics(FontGDI, 0, core::ptr::null_mut(), 0 as BOOL)) as _);
    if otm.is_null() {
        return Size;
    }
    if Size > Data {
        EngSetLastError(ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }
    potm = ExAllocatePoolWithTag(PagedPool, Size as usize, GDITAG_TEXT) as POUTLINETEXTMETRICW;
    if potm.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }
    RtlZeroMemory(potm as *mut _, Size as usize);
    IntGetOutlineTextMetrics(FontGDI, Size, potm, (0u8 as _));

    Status = SafeProbeForWrite(otm as *mut _, Size as usize, 1);
    // Validate-only probe; the copy is the access. A copy fault lands in Status,
    // which is read below -- upstream's handler does exactly this.
    if NT_SUCCESS(Status)
        && !crate::copy::publish_to_user(otm as *mut _, potm as *const _, Size as usize)
    {
        Status = STATUS_ACCESS_VIOLATION;
    }

    if !NT_SUCCESS(Status) {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        Size = 0;
    }

    ExFreePoolWithTag(potm as *mut _, GDITAG_TEXT);
    Size
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetOutlineTextMetricsInternalW(
    hDC: HDC,
    Data: ULONG,
    otm: POUTLINETEXTMETRICW,
    Tmd: PTMDIFF,
) -> ULONG {
    let dc: PDC;
    let pdcattr: PDC_ATTR;
    let TextObj: PTEXTOBJ;
    let FontGDI: PFONTGDI;
    let mut hFont: HFONT = 0 as HFONT;
    let mut Size: ULONG;
    let potm: POUTLINETEXTMETRICW;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    pdcattr = (*dc).pdcattr;
    hFont = (*pdcattr).hlfntNew as _ ;
    TextObj = RealizeFontInit(hFont);
    DC_UnlockDc(dc);
    if TextObj.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    FontGDI = ObjToGDI!((*TextObj).Font);
    if ((*FontGDI).flType & FO_TYPE_TRUETYPE as ULONG) == 0 {
        w32r_TEXTOBJ_UnlockText(TextObj);
        return 0;
    }
    TextIntUpdateSize(dc, TextObj, FontGDI, 1u8 as _ );
    w32r_TEXTOBJ_UnlockText(TextObj);
    Size = (IntGetOutlineTextMetrics(FontGDI, 0, core::ptr::null_mut(), 0 as BOOL)) as _ ;
    if otm.is_null() {
        return Size;
    }
    if Size > Data {
        EngSetLastError(ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }
    potm = ExAllocatePoolWithTag(PagedPool, Size as usize, GDITAG_TEXT) as POUTLINETEXTMETRICW;
    if potm.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }
    RtlZeroMemory(potm as *mut _, Size as usize);
    IntGetOutlineTextMetrics(FontGDI, Size, potm, 0u8 as _ );

    Status = SafeProbeForWrite(otm as *mut _, Size as usize, 1);
    // Validate-only probe; the copy is the access. A copy fault lands in Status,
    // which is read below -- upstream's handler does exactly this.
    if NT_SUCCESS(Status)
        && !crate::copy::publish_to_user(otm as *mut _, potm as *const _, Size as usize)
    {
        Status = STATUS_ACCESS_VIOLATION;
    }

    if !NT_SUCCESS(Status) {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        Size = 0;
    }

    ExFreePoolWithTag(potm as *mut _, GDITAG_TEXT);
    Size
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetFontResourceInfoInternalW(
    pwszFiles: PCWCH,
    cwc: ULONG,
    cFiles: ULONG,
    cjIn: UINT,
    pdwBytes: PDWORD,
    pvBuf: PVOID,
    dwType: DWORD,
) -> BOOL {
    let mut Status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;
    let mut dwBytes: DWORD = 0;
    let dwBytesRequested: DWORD;
    let mut SafeFileNames: _UNICODE_STRING = core::mem::zeroed();
    let mut bRet: BOOL = 0;
    let cbStringSize: ULONG;
    let Buffer: LPVOID;

    /* Check for valid dwType values */
    if dwType > 5 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    /* Allocate a safe unicode string buffer */
    cbStringSize = cwc * core::mem::size_of::<u16>() as ULONG;
    SafeFileNames.Length = (cbStringSize as u16).wrapping_sub(core::mem::size_of::<u16>() as u16);
    SafeFileNames.MaximumLength = SafeFileNames.Length;
    SafeFileNames.Buffer = ExAllocatePoolWithTag(PagedPool, cbStringSize as usize, TAG_USTR) as *mut u16;
    if SafeFileNames.Buffer.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }
    RtlZeroMemory(SafeFileNames.Buffer as *mut core::ffi::c_void, SafeFileNames.MaximumLength as usize);

    /* Check buffers and copy pwszFiles to safe unicode string */
    // The C probes all three user pointers inside a _SEH2_TRY and stashes the exception
    // code in Status; ALL THREE were dropped. That did two things. It left the
    // `if (!NT_SUCCESS(Status))` below checking a status nothing could ever set — the
    // same dead-check shape as NtSetValueKey (#79) — and it let `*pdwBytes` be read and
    // `pvBuf` be written at whatever addresses the caller passed, from an unprivileged
    // syscall. Under DUMMY_PSEH a fault here is not an access violation, it is the box.
    //
    // Validate-first, so the existing status check now sits BEFORE the dereferences it
    // was supposed to guard rather than after them. pvBuf is probed only when non-null,
    // matching the C's `if (pvBuf)`.
    Status = SafeProbeForRead(pwszFiles as PVOID, cbStringSize as usize, 1);
    if NT_SUCCESS(Status) {
        Status = SafeProbeForWrite(pdwBytes as PVOID, core::mem::size_of::<DWORD>(), 1);
    }
    if NT_SUCCESS(Status) && !pvBuf.is_null() {
        Status = SafeProbeForWrite(pvBuf as PVOID, cjIn as usize, 1);
    }

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        ExFreePoolWithTag(SafeFileNames.Buffer as LPVOID, TAG_USTR);
        return 0;
    }

    // Probe is validate-only (USE_DUMMY_PSEH); the C wrapped the probes AND both of
    // these accesses in one _SEH2_TRY. pwszFiles is a user buffer, so the copy out of
    // it is as exposed as the DWORD load.
    if !crate::copy::capture_from_user(
        core::ptr::addr_of_mut!(dwBytes) as *mut core::ffi::c_void,
        pdwBytes as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>(),
    ) || !crate::copy::capture_from_user(
        SafeFileNames.Buffer as *mut core::ffi::c_void,
        pwszFiles as *const core::ffi::c_void,
        cbStringSize as usize,
    ) {
        SetLastNtError(STATUS_ACCESS_VIOLATION);
        ExFreePoolWithTag(SafeFileNames.Buffer as LPVOID, TAG_USTR);
        return 0;
    }
    dwBytesRequested = dwBytes;
    if dwBytes > 0 {
        Buffer = ExAllocatePoolWithTag(PagedPool, dwBytes as usize, TAG_FINF);
    } else {
        Buffer = ExAllocatePoolWithTag(PagedPool, core::mem::size_of::<DWORD>(), TAG_FINF);
    }

    /* Do the actual call */
    bRet = IntGdiGetFontResourceInfo(
        core::ptr::addr_of_mut!(SafeFileNames),
        if !pvBuf.is_null() { Buffer } else { core::ptr::null_mut() },
        core::ptr::addr_of_mut!(dwBytes),
        dwType,
    );

    /* Check if succeeded */
    if bRet != 0 {
        /* Copy the data back to caller */
        // Probes are validate-only (USE_DUMMY_PSEH); the C had them AND these stores
        // inside one _SEH2_TRY. RtlCopyMemory into a user buffer is a plain memcpy.
        let mut ok = true;
        if !pvBuf.is_null() && dwBytesRequested > 0 {
            let count = if dwBytesRequested < dwBytes { dwBytesRequested } else { dwBytes };
            ok = crate::copy::publish_to_user(pvBuf, Buffer, count as usize);
        }
        if !ok
            || !crate::copy::publish_to_user(
                pdwBytes as *mut core::ffi::c_void,
                core::ptr::addr_of!(dwBytes) as *const core::ffi::c_void,
                core::mem::size_of::<DWORD>(),
            )
        {
            Status = STATUS_ACCESS_VIOLATION;
        }

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            bRet = 0;
        }
    }

    ExFreePoolWithTag(Buffer, TAG_FINF);
    ExFreePoolWithTag(SafeFileNames.Buffer as LPVOID, TAG_USTR);

    bRet
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetRealizationInfo(
    hdc: HDC,
    pri: PREALIZATION_INFO,
    hf: HFONT,
) -> BOOL {
    let pDc: PDC;
    let pTextObj: PTEXTOBJ;
    let pFontGdi: PFONTGDI;
    let pdcattr: PDC_ATTR;
    let mut Ret: BOOL;
    let mut i: INT = 0;
    let mut ri: REALIZATION_INFO = core::mem::zeroed();

    pDc = DC_LockDc(hdc);
    if pDc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    pdcattr = (*pDc).pdcattr;
    pTextObj = RealizeFontInit(((*pdcattr).hlfntNew as _));
    // Null-guard: RealizeFontInit provably returns null on failure (inlined
    // body), and the (*pTextObj).Font load sinks below UnlockText in release —
    // the unguarded deref folded the tail into `ud2` (§9.103).
    if pTextObj.is_null() {
        DC_UnlockDc(pDc);
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    pFontGdi = ObjToGDI!((*pTextObj).Font);
    w32r_TEXTOBJ_UnlockText(pTextObj);
    DC_UnlockDc(pDc);

    Ret = ftGdiRealizationInfo(pFontGdi, &mut ri);
    if Ret != 0 {
        if !pri.is_null() {
            let mut Status: NTSTATUS = SafeProbeForWrite(
                pri as *mut core::ffi::c_void,
                (core::mem::size_of::<REALIZATION_INFO>() as ULONG as _),
                1,
            );
            // Validate-only probe; the copy is the access. A copy fault lands in Status,
            // which is read below -- upstream's handler does exactly this.
            if NT_SUCCESS(Status)
                && !crate::copy::publish_to_user(pri as *mut core::ffi::c_void, &mut ri as *mut REALIZATION_INFO as *mut core::ffi::c_void, (core::mem::size_of::<REALIZATION_INFO>() as ULONG as _))
            {
                Status = STATUS_ACCESS_VIOLATION;
            }

            if !NT_SUCCESS(Status) {
                SetLastNtError(Status);
                return 0;
            }
        }
        loop {
            if (*GdiHandleTable).cfPublic[i as usize].hf == hf {
                (*GdiHandleTable).cfPublic[i as usize].iTechnology = ri.iTechnology;
                (*GdiHandleTable).cfPublic[i as usize].iUniq = ri.iUniq;
                (*GdiHandleTable).cfPublic[i as usize].dwUnknown = ri.dwUnknown;
                (*GdiHandleTable).cfPublic[i as usize].dwCFCount = (*GdiHandleTable).dwCFCount;
                (*GdiHandleTable).cfPublic[i as usize].fl |= CFONT_REALIZATION;
            }
            i += 1;
            if i >= GDI_CFONT_MAX as INT {
                break;
            }
        }
    }
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetRealizationInfo(
    hdc: HDC,
    pri: PREALIZATION_INFO,
    hf: HFONT,
) -> BOOL {
    let pDc: PDC;
    let pTextObj: PTEXTOBJ;
    let pFontGdi: PFONTGDI;
    let pdcattr: PDC_ATTR;
    let mut Ret: BOOL;
    let mut i: INT = 0;
    let mut ri: REALIZATION_INFO = core::mem::zeroed();

    pDc = DC_LockDc(hdc);
    if pDc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    pdcattr = (*pDc).pdcattr;
    pTextObj = RealizeFontInit((*pdcattr).hlfntNew as _ );
    // Null-guard — see the x86 twin above (§9.103).
    if pTextObj.is_null() {
        DC_UnlockDc(pDc);
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    pFontGdi = ObjToGDI!((*pTextObj).Font);
    w32r_TEXTOBJ_UnlockText(pTextObj);
    DC_UnlockDc(pDc);

    Ret = ftGdiRealizationInfo(pFontGdi, &mut ri);
    if Ret != 0 {
        if !pri.is_null() {
            let mut Status: NTSTATUS = SafeProbeForWrite(
                pri as *mut core::ffi::c_void,
                core::mem::size_of::<REALIZATION_INFO>() as ULONG as _ ,
                1,
            );
            // Validate-only probe; the copy is the access. A copy fault lands in Status,
            // which is read below -- upstream's handler does exactly this.
            if NT_SUCCESS(Status)
                && !crate::copy::publish_to_user(pri as *mut core::ffi::c_void, &mut ri as *mut REALIZATION_INFO as *mut core::ffi::c_void, core::mem::size_of::<REALIZATION_INFO>() as ULONG as _)
            {
                Status = STATUS_ACCESS_VIOLATION;
            }

            if !NT_SUCCESS(Status) {
                SetLastNtError(Status);
                return 0;
            }
        }
        loop {
            if (*GdiHandleTable).cfPublic[i as usize].hf == hf {
                (*GdiHandleTable).cfPublic[i as usize].iTechnology = ri.iTechnology;
                (*GdiHandleTable).cfPublic[i as usize].iUniq = ri.iUniq;
                (*GdiHandleTable).cfPublic[i as usize].dwUnknown = ri.dwUnknown;
                (*GdiHandleTable).cfPublic[i as usize].dwCFCount = (*GdiHandleTable).dwCFCount;
                (*GdiHandleTable).cfPublic[i as usize].fl |= CFONT_REALIZATION;
            }
            i += 1;
            if i >= GDI_CFONT_MAX as INT {
                break;
            }
        }
    }
    Ret
}

// The three NT6 font-file syscalls dwrite's GDI interop uses (upstream ab1aefbabb3
// appended them to w32ksvc32/64.h as stubs). Semantics are Wine win32u's
// (NtGdiGetRealizationInfo, NtGdiGetFontFileInfo, NtGdiGetFontFileData); the face lookup
// lives in freetype.rs beside the font lists it walks.

// FONT_REALIZATION_INFO has two sizes: 16 (through instance_id) and 24.
const FONT_REALIZATION_INFO_V0: usize = 16;

/// The bytes of the selected font's file, for dwrite's custom loader when there is no path.
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetFontFileData(
    uFileCollectionID: UINT,
    uFileIndex: UINT,
    pullFileOffset: PULONGLONG,
    pvBuf: PVOID,
    cjBuf: SIZE_T,
) -> DWORD {
    let mut Offset: u64 = 0;
    if uFileIndex != 0 || pullFileOffset.is_null()
        || !NT_SUCCESS(SafeProbeForRead(pullFileOffset as PVOID, core::mem::size_of::<u64>(), 1))
        || !crate::copy::capture_from_user(&mut Offset as *mut u64 as _, pullFileOffset as _, core::mem::size_of::<u64>())
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Some((SharedFace, Name)) = crate::freetype::IntFindFontInstance(uFileCollectionID) else {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    };
    let (Buffer, Size) = crate::freetype::IntFontInstanceMemory(SharedFace);
    let mut Ret: DWORD = 0;
    match Offset.checked_add(cjBuf as u64) {
        Some(End) if !Buffer.is_null() && End <= Size as u64 => {
            if cjBuf == 0
                || (NT_SUCCESS(SafeProbeForWrite(pvBuf, cjBuf as usize, 1))
                    && crate::copy::publish_to_user(pvBuf, Buffer.add(Offset as usize) as _, cjBuf as usize))
            {
                Ret = 1;
            } else {
                EngSetLastError(ERROR_NOACCESS);
            }
        }
        _ => EngSetLastError(ERROR_INVALID_PARAMETER),
    }
    crate::freetype::IntReleaseFontInstance(SharedFace, Name);
    Ret
}

/// `\SystemRoot\x` -> `<NtSystemRoot>\x`, `\??\C:\x` -> `C:\x`; anything else has no DOS
/// spelling and yields an empty path, which sends dwrite to GetFontFileData instead.
unsafe fn FontDosPath(NtName: *const u16, Out: &mut [u16; 520]) -> usize {
    let mut n = 0usize;
    if NtName.is_null() {
        return 0;
    }
    let len = { let mut l = 0; while *NtName.add(l) != 0 { l += 1; } l };
    let name = core::slice::from_raw_parts(NtName, len);
    let starts = |p: &str| name.len() >= p.len() && p.encode_utf16().zip(name).all(|(a, &b)| {
        let up = |c: u16| if (b'a' as u16..=b'z' as u16).contains(&c) { c - 32 } else { c };
        up(a) == up(b)
    });
    let rest: &[u16] = if starts("\\SystemRoot\\") {
        let sud = ntkrnl_sys::mm::KI_USER_SHARED_DATA as usize as *const ntkrnl_sys::ddk::_KUSER_SHARED_DATA;
        let root = &(*sud).NtSystemRoot;
        for &c in root.iter().take_while(|&&c| c != 0) {
            if n + 1 >= Out.len() { return 0; }
            Out[n] = c;
            n += 1;
        }
        &name[11..] // keeps the backslash after \SystemRoot
    } else if starts("\\??\\") {
        &name[4..]
    } else {
        return 0;
    };
    for &c in rest {
        if n + 1 >= Out.len() { return 0; }
        Out[n] = c;
        n += 1;
    }
    Out[n] = 0;
    n
}

/// Write time, size and DOS path of the selected font's file.
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetFontFileInfo(
    uFileCollectionID: UINT,
    uFileIndex: UINT,
    pffi: PFONT_FILE_INFO,
    cjSize: SIZE_T,
    pcjActualSize: PSIZE_T,
) -> DWORD {
    let mut Required: SIZE_T = 0;
    let mut Ret: DWORD = 0;
    let found = if uFileIndex == 0 { crate::freetype::IntFindFontInstance(uFileCollectionID) } else { None };
    if let Some((SharedFace, Name)) = found {
        let mut Path = [0u16; 520];
        let PathLen = FontDosPath(Name, &mut Path);
        Required = (core::mem::size_of::<FONT_FILE_INFO>() + PathLen * core::mem::size_of::<u16>()) as SIZE_T;
        if Required <= cjSize && !pffi.is_null() {
            let mut WriteTime: i64 = 0;
            let mut FileSize: i64 = crate::freetype::IntFontInstanceMemory(SharedFace).1 as i64;
            if !Name.is_null() {
                if let Some((w, s)) = FontFileTimes(Name) {
                    WriteTime = w;
                    FileSize = s;
                }
            }
            let mut Head: FONT_FILE_INFO = core::mem::zeroed();
            Head.writetime.dwLowDateTime = WriteTime as u32;
            Head.writetime.dwHighDateTime = (WriteTime >> 32) as u32;
            *(core::ptr::addr_of_mut!(Head.size) as *mut i64) = FileSize;
            let PathOff = core::mem::offset_of!(FONT_FILE_INFO, path);
            if NT_SUCCESS(SafeProbeForWrite(pffi as PVOID, Required as usize, 1))
                && crate::copy::publish_to_user(pffi as _, &Head as *const FONT_FILE_INFO as _, PathOff)
                && crate::copy::publish_to_user(
                    (pffi as *mut u8).add(PathOff) as _,
                    Path.as_ptr() as _,
                    (PathLen + 1) * core::mem::size_of::<u16>(),
                )
            {
                Ret = 1;
            } else {
                EngSetLastError(ERROR_NOACCESS);
            }
        } else {
            EngSetLastError(ERROR_INSUFFICIENT_BUFFER);
        }
        crate::freetype::IntReleaseFontInstance(SharedFace, Name);
    } else {
        EngSetLastError(ERROR_INVALID_PARAMETER);
    }
    if !pcjActualSize.is_null()
        && NT_SUCCESS(SafeProbeForWrite(pcjActualSize as PVOID, core::mem::size_of::<SIZE_T>(), 1))
    {
        let _ = crate::copy::publish_to_user(pcjActualSize as _, &Required as *const SIZE_T as _, core::mem::size_of::<SIZE_T>());
    }
    Ret
}

extern "system" {
    #[link_name = "ZwQueryInformationFile"]
    fn ZwQueryInformationFileRaw(FileHandle: HANDLE, IoStatusBlock: *mut IO_STATUS_BLOCK, FileInformation: PVOID, Length: ULONG, FileInformationClass: ULONG) -> NTSTATUS;
    #[link_name = "ZwOpenFile"]
    fn ZwOpenFileRaw(FileHandle: *mut HANDLE, DesiredAccess: ACCESS_MASK, ObjectAttributes: *mut OBJECT_ATTRIBUTES, IoStatusBlock: *mut IO_STATUS_BLOCK, ShareAccess: ULONG, OpenOptions: ULONG) -> NTSTATUS;
    #[link_name = "ZwClose"]
    fn ZwCloseRaw(Handle: HANDLE) -> NTSTATUS;
}

/// (LastWriteTime, EndOfFile) of an NT-named file.
unsafe fn FontFileTimes(NtName: *const u16) -> Option<(i64, i64)> {
    let mut Name: UNICODE_STRING = core::mem::zeroed();
    let len = { let mut l = 0; while *NtName.add(l) != 0 { l += 1; } l };
    Name.Buffer = NtName as *mut u16;
    Name.Length = (len * 2) as u16;
    Name.MaximumLength = Name.Length;
    let mut Oa: OBJECT_ATTRIBUTES = core::mem::zeroed();
    Oa.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
    Oa.Attributes = OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE;
    Oa.ObjectName = &mut Name;
    let mut Iosb: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Handle: HANDLE = core::ptr::null_mut();
    if !NT_SUCCESS(ZwOpenFileRaw(&mut Handle, FILE_READ_ATTRIBUTES | SYNCHRONIZE, &mut Oa, &mut Iosb,
                                 FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, FILE_SYNCHRONOUS_IO_NONALERT)) {
        return None;
    }
    let mut Basic: FILE_BASIC_INFORMATION = core::mem::zeroed();
    let mut Std: FILE_STANDARD_INFORMATION = core::mem::zeroed();
    let ok = NT_SUCCESS(ZwQueryInformationFileRaw(Handle, &mut Iosb, &mut Basic as *mut _ as PVOID,
                                                  core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
                                                  _FILE_INFORMATION_CLASS_FileBasicInformation as ULONG))
        && NT_SUCCESS(ZwQueryInformationFileRaw(Handle, &mut Iosb, &mut Std as *mut _ as PVOID,
                                                core::mem::size_of::<FILE_STANDARD_INFORMATION>() as ULONG,
                                                _FILE_INFORMATION_CLASS_FileStandardInformation as ULONG));
    ZwCloseRaw(Handle);
    ok.then(|| (*(core::ptr::addr_of!(Basic.LastWriteTime) as *const i64), *(core::ptr::addr_of!(Std.EndOfFile) as *const i64)))
}

/// Which loaded face the DC's font realized to, and the simulations GDI applies to it.
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetFontRealizationInfo(hdc: HDC, info: PFONT_REALIZATION_INFO) -> BOOL {
    let mut Size: DWORD = 0;
    if info.is_null()
        || !NT_SUCCESS(SafeProbeForRead(info as PVOID, core::mem::size_of::<DWORD>(), 1))
        || !crate::copy::capture_from_user(&mut Size as *mut DWORD as _, info as _, core::mem::size_of::<DWORD>())
    {
        return 0;
    }
    if Size as usize != core::mem::size_of::<FONT_REALIZATION_INFO>() && Size as usize != FONT_REALIZATION_INFO_V0 {
        return 0;
    }
    let Dc: PDC = DC_LockDc(hdc);
    if Dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    let TextObj: PTEXTOBJ = RealizeFontInit((*(*Dc).pdcattr).hlfntNew as _);
    if TextObj.is_null() {
        DC_UnlockDc(Dc);
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    let FontGdi: PFONTGDI = ObjToGDI!((*TextObj).Font);
    let (flags, id, face_index, simulations) = crate::freetype::IntFontRealization(
        FontGdi,
        core::ptr::addr_of!((*TextObj).logfont.elfEnumLogfontEx.elfLogFont),
    );
    w32r_TEXTOBJ_UnlockText(TextObj);
    DC_UnlockDc(Dc);

    let mut Out: FONT_REALIZATION_INFO = core::mem::zeroed();
    Out.size = Size;
    Out.flags = flags;
    Out.cache_num = id;
    Out.instance_id = id;
    Out.file_count = 1;
    Out.face_index = face_index;
    Out.simulations = simulations;
    if !NT_SUCCESS(SafeProbeForWrite(info as PVOID, Size as usize, 1))
        || !crate::copy::publish_to_user(info as _, &Out as *const FONT_REALIZATION_INFO as _, Size as usize)
    {
        return 0;
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn HfontCreate(
    pelfw: *const ENUMLOGFONTEXDVW,
    cjElfw: ULONG,
    lft: LFTYPE,
    fl: FLONG,
    pvCliData: PVOID,
) -> HFONT {
    let hNewFont: HFONT;
    let plfont: PLFONT;

    if pelfw.is_null() {
        return core::ptr::null_mut();
    }

    plfont = LFONT_AllocFontWithHandle();
    if plfont.is_null() {
        return core::ptr::null_mut();
    }
    hNewFont = (*plfont).BaseObject.hHmgr as HFONT;

    (*plfont).lft = lft;
    (*plfont).fl = fl;
    RtlCopyMemory(
        core::ptr::addr_of_mut!((*plfont).logfont) as PVOID,
        pelfw as PVOID,
        core::mem::size_of::<ENUMLOGFONTEXDVW>() as ULONG as _ ,
    );
    ExInitializePushLock(core::ptr::addr_of_mut!((*plfont).lock) as _ );

    if (*pelfw).elfEnumLogfontEx.elfLogFont.lfEscapement
        != (*pelfw).elfEnumLogfontEx.elfLogFont.lfOrientation
    {
        (*plfont).logfont.elfEnumLogfontEx.elfLogFont.lfOrientation =
            (*plfont).logfont.elfEnumLogfontEx.elfLogFont.lfEscapement;
    }
    LFONT_UnlockFont(plfont);

    if !pvCliData.is_null() && !hNewFont.is_null() {
        KeEnterCriticalRegion();
        {
            let Index = GDI_HANDLE_GET_INDEX(hNewFont as HGDIOBJ);
            let Entry: PGDI_TABLE_ENTRY =
                core::ptr::addr_of_mut!((*GdiHandleTable).Entries[Index as usize]);
            (*Entry).UserData = pvCliData;
        }
        KeLeaveCriticalRegion();
    }

    hNewFont
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiHfontCreate(
    pelfw: *const ENUMLOGFONTEXDVW,
    cjElfw: ULONG,
    lft: LFTYPE,
    fl: FLONG,
    pvCliData: PVOID,
) -> HFONT {
    let mut SafeLogfont: ENUMLOGFONTEXDVW = core::mem::zeroed();
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    SafeLogfont.elfEnumLogfontEx.elfLogFont.lfEscapement = 0;
    SafeLogfont.elfEnumLogfontEx.elfLogFont.lfOrientation = 0;

    if pelfw.is_null() {
        return core::ptr::null_mut();
    }

    Status = SafeProbeForRead(
        pelfw as PVOID,
        (core::mem::size_of::<ENUMLOGFONTEXDVW>() as ULONG as _),
        1,
    );
    // Validate-only probe; the copy is the access. A copy fault lands in Status,
    // which is read below -- upstream's handler does exactly this.
    if NT_SUCCESS(Status)
        && !crate::copy::publish_to_user(core::ptr::addr_of_mut!(SafeLogfont) as PVOID, pelfw as PVOID, (core::mem::size_of::<ENUMLOGFONTEXDVW>() as ULONG as _))
    {
        Status = STATUS_ACCESS_VIOLATION;
    }

    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut();
    }

    HfontCreate(
        core::ptr::addr_of!(SafeLogfont),
        cjElfw,
        lft,
        fl,
        pvCliData,
    )
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiHfontCreate(
    pelfw: *const ENUMLOGFONTEXDVW,
    cjElfw: ULONG,
    lft: LFTYPE,
    fl: FLONG,
    pvCliData: PVOID,
) -> HFONT {
    let mut SafeLogfont: ENUMLOGFONTEXDVW = core::mem::zeroed();
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    SafeLogfont.elfEnumLogfontEx.elfLogFont.lfEscapement = 0;
    SafeLogfont.elfEnumLogfontEx.elfLogFont.lfOrientation = 0;

    if pelfw.is_null() {
        return core::ptr::null_mut();
    }

    Status = SafeProbeForRead(
        pelfw as PVOID,
        core::mem::size_of::<ENUMLOGFONTEXDVW>() as ULONG as _ ,
        1,
    );
    // Validate-only probe; the copy is the access. A copy fault lands in Status,
    // which is read below -- upstream's handler does exactly this.
    if NT_SUCCESS(Status)
        && !crate::copy::publish_to_user(core::ptr::addr_of_mut!(SafeLogfont) as PVOID, pelfw as PVOID, core::mem::size_of::<ENUMLOGFONTEXDVW>() as ULONG as _)
    {
        Status = STATUS_ACCESS_VIOLATION;
    }

    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut();
    }

    HfontCreate(
        core::ptr::addr_of!(SafeLogfont),
        cjElfw,
        lft,
        fl,
        pvCliData,
    )
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS win32 kernel mode subsystem
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Font
//  * COPYRIGHT:   Copyright (C) James Tabor <james.tabor@reactos.org>
//  *              Copyright (C) Timo Kreuzer <timo.kreuzer@reactos.org>
//  *              Copyright (C) Katayama Hirofumi MZ <katayama.hirofumi.mz@gmail.com>
//  */
//
// /** Includes ******************************************************************/
//
// #include <win32k.h>
// #include <safe_user_io.h>
//
// #define NDEBUG
// #include <debug.h>
//
// HFONT __stdcall
// HfontCreate(
//      const ENUMLOGFONTEXDVW *pelfw,
//      ULONG cjElfw,
//      LFTYPE lft,
//      FLONG fl,
//      PVOID pvCliData);
//
// /** Internal ******************************************************************/
//
// HFONT FASTCALL
// GreCreateFontIndirectW( const LOGFONTW *lplf)
// {
//     if (lplf)
//     {
//         ENUMLOGFONTEXDVW Logfont;
//
//         RtlCopyMemory( &Logfont.elfEnumLogfontEx.elfLogFont, lplf, sizeof(LOGFONTW));
//         RtlZeroMemory( &Logfont.elfEnumLogfontEx.elfFullName,
//                        sizeof(Logfont.elfEnumLogfontEx.elfFullName));
//         RtlZeroMemory( &Logfont.elfEnumLogfontEx.elfStyle,
//                        sizeof(Logfont.elfEnumLogfontEx.elfStyle));
//         RtlZeroMemory( &Logfont.elfEnumLogfontEx.elfScript,
//                        sizeof(Logfont.elfEnumLogfontEx.elfScript));
//
//         Logfont.elfDesignVector.dvNumAxes = 0;
//
//         RtlZeroMemory( &Logfont.elfDesignVector, sizeof(DESIGNVECTOR));
//
//         return HfontCreate(&Logfont, 0, 0, 0, NULL);
//     }
//     else return NULL;
// }
//
// DWORD
// FASTCALL
// GreGetKerningPairs(
//      HDC hDC,
//      ULONG NumPairs,
//      LPKERNINGPAIR krnpair)
// {
//   PDC dc;
//   PDC_ATTR pdcattr;
//   PTEXTOBJ TextObj;
//   PFONTGDI FontGDI;
//   DWORD Count;
//   KERNINGPAIR *pKP;
//
//   dc = DC_LockDc(hDC);
//   if (!dc)
//   {
//      EngSetLastError(ERROR_INVALID_HANDLE);
//      return 0;
//   }
//
//   pdcattr = dc->pdcattr;
//   TextObj = RealizeFontInit(pdcattr->hlfntNew);
//   DC_UnlockDc(dc);
//
//   if (!TextObj)
//   {
//      EngSetLastError(ERROR_INVALID_HANDLE);
//      return 0;
//   }
//
//   FontGDI = ObjToGDI!(TextObj->Font, FONT);
//   w32r_TEXTOBJ_UnlockText(TextObj);
//
//   Count = ftGdiGetKerningPairs(FontGDI,0,NULL);
//
//   if ( Count && krnpair )
//   {
//      if (Count > NumPairs)
//      {
//         EngSetLastError(ERROR_INSUFFICIENT_BUFFER);
//         return 0;
//      }
//      pKP = ExAllocatePoolWithTag(PagedPool, Count * sizeof(KERNINGPAIR), GDITAG_TEXT);
//      if (!pKP)
//      {
//         EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return 0;
//      }
//      ftGdiGetKerningPairs(FontGDI,Count,pKP);
//
//      RtlCopyMemory(krnpair, pKP, Count * sizeof(KERNINGPAIR));
//
//      ExFreePoolWithTag(pKP,GDITAG_TEXT);
//   }
//   return Count;
// }
//
// /*
//   It is recommended that an application use the GetFontLanguageInfo function
//   to determine whether the GCP_DIACRITIC, GCP_DBCS, GCP_USEKERNING, GCP_LIGATE,
//   GCP_REORDER, GCP_GLYPHSHAPE, and GCP_KASHIDA values are valid for the
//   currently selected font. If not valid, GetCharacterPlacement ignores the
//   value.
//
//   MS must use a preset "compiled in" support for each language based releases.
//   ReactOS uses FreeType, this will need to be supported. ATM this is hard coded
//   for GCPCLASS_LATIN!
//  */
//
// ULONG
// FASTCALL
// FontGetObject(
//      PTEXTOBJ plfont,
//      ULONG cjBuffer,
//      PVOID pvBuffer)
// {
//     ULONG cjMaxSize;
//     ENUMLOGFONTEXDVW *plf;
//
//     ASSERT(plfont);
//     plf = &plfont->logfont;
//
//     if (!(plfont->fl & TEXTOBJECT_INIT))
//     {
//         NTSTATUS Status;
//         DPRINT("FontGetObject font not initialized!\n");
//
//         Status = TextIntRealizeFont(plfont->BaseObject.hHmgr, plfont);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("FontGetObject(TextIntRealizeFont) Status = 0x%lx\n", Status);
//         }
//     }
//
//     /* If buffer is NULL, only the size is requested */
//     if (pvBuffer == NULL) return sizeof(LOGFONTW);
//
//     /* Calculate the maximum size according to number of axes */
//     cjMaxSize = FIELD_OFFSET(ENUMLOGFONTEXDVW,
//                     elfDesignVector.dvValues[plf->elfDesignVector.dvNumAxes]);
//
//     if (cjBuffer > cjMaxSize) cjBuffer = cjMaxSize;
//
//     RtlCopyMemory(pvBuffer, plf, cjBuffer);
//
//     return cjBuffer;
// }
//
// DWORD
// FASTCALL
// IntGetCharDimensions(
//      HDC hdc,
//      PTEXTMETRICW ptm,
//      PDWORD height)
// {
//   PDC pdc;
//   PDC_ATTR pdcattr;
//   PTEXTOBJ TextObj;
//   SIZE sz;
//   TMW_INTERNAL tmwi;
//   BOOL Good;
//
//   static const WCHAR alphabet[] = {
//         'a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q',
//         'r','s','t','u','v','w','x','y','z','A','B','C','D','E','F','G','H',
//         'I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z',0};
//
//   if(!ftGdiGetTextMetricsW(hdc, &tmwi)) return 0;
//
//   pdc = DC_LockDc(hdc);
//
//   if (!pdc) return 0;
//
//   pdcattr = pdc->pdcattr;
//
//   TextObj = RealizeFontInit(pdcattr->hlfntNew);
//   if ( !TextObj )
//   {
//      DC_UnlockDc(pdc);
//      return 0;
//   }
//   Good = TextIntGetTextExtentPoint(pdc, TextObj, alphabet, 52, 0, NULL, NULL, &sz, 0);
//   w32r_TEXTOBJ_UnlockText(TextObj);
//   DC_UnlockDc(pdc);
//
//   if (!Good) return 0;
//   if (ptm) *ptm = tmwi.TextMetric;
//   if (height) *height = tmwi.TextMetric.tmHeight;
//
//   return (sz.cx / 26 + 1) / 2;
// }
//
//
// DWORD
// FASTCALL
// IntGetFontLanguageInfo( PDC Dc)
// {
//   PDC_ATTR pdcattr;
//   FONTSIGNATURE fontsig;
//   static const DWORD GCP_DBCS_MASK=0x003F0000,
// 		GCP_DIACRITIC_MASK=0x00000000,
// 		FLI_GLYPHS_MASK=0x00000000,
// 		GCP_GLYPHSHAPE_MASK=0x00000040,
// 		GCP_KASHIDA_MASK=0x00000000,
// 		GCP_LIGATE_MASK=0x00000000,
// 		GCP_USEKERNING_MASK=0x00000000,
// 		GCP_REORDER_MASK=0x00000060;
//
//   DWORD result=0;
//
//   ftGdiGetTextCharsetInfo( Dc, &fontsig, 0 );
//
//  /* We detect each flag we return using a bitmask on the Codepage Bitfields */
//   if( (fontsig.fsCsb[0]&GCP_DBCS_MASK)!=0 )
// 		result|=GCP_DBCS;
//
//   if( (fontsig.fsCsb[0]&GCP_DIACRITIC_MASK)!=0 )
// 		result|=GCP_DIACRITIC;
//
//   if( (fontsig.fsCsb[0]&FLI_GLYPHS_MASK)!=0 )
// 		result|=FLI_GLYPHS;
//
//   if( (fontsig.fsCsb[0]&GCP_GLYPHSHAPE_MASK)!=0 )
// 		result|=GCP_GLYPHSHAPE;
//
//   if( (fontsig.fsCsb[0]&GCP_KASHIDA_MASK)!=0 )
// 		result|=GCP_KASHIDA;
//
//   if( (fontsig.fsCsb[0]&GCP_LIGATE_MASK)!=0 )
// 		result|=GCP_LIGATE;
//
//   if( (fontsig.fsCsb[0]&GCP_USEKERNING_MASK)!=0 )
// 		result|=GCP_USEKERNING;
//
//   pdcattr = Dc->pdcattr;
//
//   /* This might need a test for a HEBREW- or ARABIC_CHARSET as well */
//   if ( pdcattr->flTextAlign & TA_RTLREADING )
//      if( (fontsig.fsCsb[0]&GCP_REORDER_MASK)!=0 )
//                     result|=GCP_REORDER;
//
//   return result;
// }
//
// PTEXTOBJ
// FASTCALL
// RealizeFontInit( HFONT hFont)
// {
//   NTSTATUS Status = STATUS_SUCCESS;
//   PTEXTOBJ pTextObj;
//
//   pTextObj = w32r_TEXTOBJ_LockText(hFont);
//
//   if ( pTextObj && !(pTextObj->fl & TEXTOBJECT_INIT))
//   {
//      Status = TextIntRealizeFont(hFont, pTextObj);
//      if (!NT_SUCCESS(Status))
//      {
//         w32r_TEXTOBJ_UnlockText(pTextObj);
//         return NULL;
//      }
//   }
//   return pTextObj;
// }
//
// static BOOL
// IntCheckFontPathNames(
//      PCWCH pwcFiles,
//      ULONG cFiles,
//      ULONG cwc)
// {
//     ULONG ich, cRealFiles;
//
//     if (pwcFiles[cwc - 1] != UNICODE_NULL)
//         return FALSE;
//
//     for (ich = cRealFiles = 0; ich < cwc; ++ich)
//     {
//         if (!pwcFiles[ich])
//             ++cRealFiles;
//     }
//
//     return cRealFiles >= cFiles;
// }
//
// /** Functions ******************************************************************/
//
// INT
// __stdcall
// NtGdiAddFontResourceW(
//      PCWCH pwcFiles,
//      ULONG cwc,
//      ULONG cFiles,
//      FLONG fl,
//      DWORD dwPidTid,
//      const DESIGNVECTOR *pdv)
// {
//     UNICODE_STRING SafeFileName;
//     INT Ret;
//
//     DBG_UNREFERENCED_PARAMETER(dwPidTid);
//     DBG_UNREFERENCED_PARAMETER(pdv);
//
//     DPRINT("NtGdiAddFontResourceW\n");
//
//     /* cwc = Length + trailing zero. */
//     if ((cwc <= 1) || (cwc > UNICODE_STRING_MAX_CHARS))
//         return 0;
//
//     SafeFileName.Length = (USHORT)((cwc - 1) * sizeof(WCHAR));
//     SafeFileName.MaximumLength = SafeFileName.Length + sizeof(UNICODE_NULL);
//     SafeFileName.Buffer = ExAllocatePoolWithTag(PagedPool,
//                                                 SafeFileName.MaximumLength,
//                                                 TAG_STRING);
//     if (!SafeFileName.Buffer)
//         return 0;
//
//     _SEH2_TRY
//     {
//         ProbeForRead(pwcFiles, cwc * sizeof(WCHAR), sizeof(WCHAR));
//
//         if (!IntCheckFontPathNames(pwcFiles, cFiles, cwc))
//             return 0;
//  
//         RtlCopyMemory(SafeFileName.Buffer, pwcFiles, SafeFileName.Length);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         ExFreePoolWithTag(SafeFileName.Buffer, TAG_STRING);
//         _SEH2_YIELD(return 0);
//     }
//     _SEH2_END;
//
//     SafeFileName.Buffer[SafeFileName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     Ret = IntGdiAddFontResourceEx(&SafeFileName, cFiles, fl, 0);
//
//     ExFreePoolWithTag(SafeFileName.Buffer, TAG_STRING);
//     return Ret;
// }
//
// BOOL
// __stdcall
// NtGdiRemoveFontResourceW(
//      PCWCH pwszFiles,
//      ULONG cwc,
//      ULONG cFiles,
//      ULONG fl,
//      DWORD dwPidTid,
//      const DESIGNVECTOR *pdv)
// {
//     UNICODE_STRING SafeFileName;
//     BOOL Ret;
//
//     DBG_UNREFERENCED_PARAMETER(dwPidTid);
//     DBG_UNREFERENCED_PARAMETER(pdv);
//
//     DPRINT("NtGdiRemoveFontResourceW\n");
//
//     /* cwc = Length + trailing zero. */
//     if ((cwc <= 1) || (cwc > UNICODE_STRING_MAX_CHARS))
//         return FALSE;
//
//     SafeFileName.Length = (USHORT)((cwc - 1) * sizeof(WCHAR));
//     SafeFileName.MaximumLength = SafeFileName.Length + sizeof(UNICODE_NULL);
//     SafeFileName.Buffer = ExAllocatePoolWithTag(PagedPool,
//                                                 SafeFileName.MaximumLength,
//                                                 TAG_STRING);
//     if (!SafeFileName.Buffer)
//         return FALSE;
//
//     _SEH2_TRY
//     {
//         ProbeForRead(pwszFiles, cwc * sizeof(WCHAR), sizeof(WCHAR));
//
//         if (!IntCheckFontPathNames(pwszFiles, cFiles, cwc))
//             return FALSE;
//
//         RtlCopyMemory(SafeFileName.Buffer, pwszFiles, SafeFileName.Length);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         ExFreePoolWithTag(SafeFileName.Buffer, TAG_STRING);
//         _SEH2_YIELD(return FALSE);
//     }
//     _SEH2_END;
//
//     SafeFileName.Buffer[SafeFileName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     Ret = IntGdiRemoveFontResource(&SafeFileName, cFiles, fl);
//
//     ExFreePoolWithTag(SafeFileName.Buffer, TAG_STRING);
//     return Ret;
// }
//
// HANDLE
// __stdcall
// NtGdiAddFontMemResourceEx(
//      const VOID *pvBuffer,
//      DWORD cjBuffer,
//      const DESIGNVECTOR *pdv,
//      ULONG cjDV,
//      PDWORD pNumFonts)
// {
//     _SEH2_VOLATILE PVOID Buffer = NULL;
//     HANDLE Ret;
//     DWORD NumFonts = 0;
//
//     DPRINT("NtGdiAddFontMemResourceEx\n");
//     DBG_UNREFERENCED_PARAMETER(pdv);
//     DBG_UNREFERENCED_PARAMETER(cjDV);
//
//     if (!pvBuffer || !cjBuffer)
//         return NULL;
//
//     _SEH2_TRY
//     {
//         ProbeForRead(pvBuffer, cjBuffer, sizeof(BYTE));
//         Buffer = ExAllocatePoolWithQuotaTag(PagedPool, cjBuffer, TAG_FONT);
//         RtlCopyMemory(Buffer, pvBuffer, cjBuffer);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         if (Buffer != NULL)
//         {
//             ExFreePoolWithTag(Buffer, TAG_FONT);
//         }
//         _SEH2_YIELD(return NULL);
//     }
//     _SEH2_END;
//
//     Ret = IntGdiAddFontMemResource(Buffer, cjBuffer, &NumFonts);
//     ExFreePoolWithTag(Buffer, TAG_FONT);
//
//     {
//         NTSTATUS __SehStatus = SafeProbeForWrite(pNumFonts, sizeof(NumFonts), 1);
//         if (!NT_SUCCESS(__SehStatus))
//         {
//             /* Leak it? */
//             return NULL;
//         }
//     }
//     *pNumFonts = NumFonts;
//
//
//     return Ret;
// }
//
//
// BOOL
// __stdcall
// NtGdiRemoveFontMemResourceEx(
//      HANDLE hMMFont)
// {
//     return IntGdiRemoveFontMemResource(hMMFont);
// }
//
//
//  /*
//  * @unimplemented
//  */
// DWORD
// __stdcall
// NtGdiGetCharacterPlacementW(
//      HDC hdc,
//      PCWCH pwsz,
//      INT nCount,
//      INT nMaxExtent,
//      LPGCP_RESULTSW pgcpw,
//      DWORD dwFlags)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// DWORD
// __stdcall
// NtGdiGetFontData(
//      HDC hDC,
//      DWORD Table,
//      DWORD Offset,
//      PVOID Buffer,
//      DWORD Size)
// {
//   PDC Dc;
//   PDC_ATTR pdcattr;
//   HFONT hFont;
//   PTEXTOBJ TextObj;
//   PFONTGDI FontGdi;
//   DWORD Result = GDI_ERROR;
//   NTSTATUS Status = STATUS_SUCCESS;
//
//   if (Buffer && Size)
//   {
//      Status = SafeProbeForRead(Buffer, Size, 1);
//   }
//
//   if (!NT_SUCCESS(Status)) return Result;
//
//   Dc = DC_LockDc(hDC);
//   if (Dc == NULL)
//   {
//      EngSetLastError(ERROR_INVALID_HANDLE);
//      return GDI_ERROR;
//   }
//   pdcattr = Dc->pdcattr;
//
//   hFont = pdcattr->hlfntNew;
//   TextObj = RealizeFontInit(hFont);
//   DC_UnlockDc(Dc);
//
//   if (TextObj == NULL)
//   {
//      EngSetLastError(ERROR_INVALID_HANDLE);
//      return GDI_ERROR;
//   }
//
//   FontGdi = ObjToGDI!(TextObj->Font, FONT);
//
//   Result = ftGdiGetFontData(FontGdi, Table, Offset, Buffer, Size);
//
//   w32r_TEXTOBJ_UnlockText(TextObj);
//
//   return Result;
// }
//
// /* @implemented */
// DWORD
// __stdcall
// NtGdiGetFontUnicodeRanges(
//      HDC hdc,
//      LPGLYPHSET pgs)
// {
//   PDC pDc;
//   PDC_ATTR pdcattr;
//   HFONT hFont;
//   PTEXTOBJ TextObj;
//   PFONTGDI FontGdi;
//   DWORD Size = 0;
//   PGLYPHSET pgsSafe;
//   NTSTATUS Status = STATUS_SUCCESS;
//
//   pDc = DC_LockDc(hdc);
//   if (!pDc)
//   {
//      EngSetLastError(ERROR_INVALID_HANDLE);
//      return 0;
//   }
//
//   pdcattr = pDc->pdcattr;
//
//   hFont = pdcattr->hlfntNew;
//   TextObj = RealizeFontInit(hFont);
//
//   if ( TextObj == NULL)
//   {
//      EngSetLastError(ERROR_INVALID_HANDLE);
//      goto Exit;
//   }
//   FontGdi = ObjToGDI!(TextObj->Font, FONT);
//
//   Size = ftGetFontUnicodeRanges( FontGdi, NULL);
//
//   if (Size && pgs)
//   {
//      pgsSafe = ExAllocatePoolWithTag(PagedPool, Size, GDITAG_TEXT);
//      if (!pgsSafe)
//      {
//         EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         Size = 0;
//         goto Exit;
//      }
//
//      Size = ftGetFontUnicodeRanges( FontGdi, pgsSafe);
//
//      if (Size)
//      {
//         Status = SafeProbeForWrite(pgs, Size, 1);
//         if (NT_SUCCESS(Status))
//         {
//             RtlCopyMemory(pgs, pgsSafe, Size);
//         }
//
//         if (!NT_SUCCESS(Status)) Size = 0;
//      }
//      ExFreePoolWithTag(pgsSafe, GDITAG_TEXT);
//   }
// Exit:
//   w32r_TEXTOBJ_UnlockText(TextObj);
//   DC_UnlockDc(pDc);
//   return Size;
// }
//
// ULONG
// __stdcall
// NtGdiGetGlyphOutline(
//      HDC hdc,
//      WCHAR wch,
//      UINT iFormat,
//      LPGLYPHMETRICS pgm,
//      ULONG cjBuf,
//      PVOID UnsafeBuf,
//      const MAT2 *pmat2,
//      BOOL bIgnoreRotation)
// {
//   ULONG Ret = GDI_ERROR;
//   PDC dc;
//   PVOID pvBuf = NULL;
//   GLYPHMETRICS gm;
//   NTSTATUS Status = STATUS_SUCCESS;
//
//   dc = DC_LockDc(hdc);
//   if (!dc)
//   {
//      EngSetLastError(ERROR_INVALID_HANDLE);
//      return GDI_ERROR;
//   }
//
//   if (UnsafeBuf && cjBuf)
//   {
//      pvBuf = ExAllocatePoolZero(PagedPool, cjBuf, GDITAG_TEXT);
//      if (!pvBuf)
//      {
//         EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         goto Exit;
//      }
//   }
//
//   Ret = ftGdiGetGlyphOutline( dc,
//                              wch,
//                          iFormat,
//                              pgm ? &gm : NULL,
//                            cjBuf,
//                            pvBuf,
//                            pmat2,
//                  bIgnoreRotation);
//
//   if (pvBuf)
//   {
//      Status = SafeProbeForWrite(UnsafeBuf, cjBuf, 1);
//      if (NT_SUCCESS(Status))
//      {
//          RtlCopyMemory(UnsafeBuf, pvBuf, cjBuf);
//      }
//
//      ExFreePoolWithTag(pvBuf, GDITAG_TEXT);
//   }
//
//   if (pgm)
//   {
//      Status = SafeProbeForWrite(pgm, sizeof(GLYPHMETRICS), 1);
//      if (NT_SUCCESS(Status))
//      {
//          RtlCopyMemory(pgm, &gm, sizeof(GLYPHMETRICS));
//      }
//   }
//
//   if (! NT_SUCCESS(Status))
//   {
//      EngSetLastError(ERROR_INVALID_PARAMETER);
//      Ret = GDI_ERROR;
//   }
//
// Exit:
//   DC_UnlockDc(dc);
//   return Ret;
// }
//
// DWORD
// __stdcall
// NtGdiGetKerningPairs(
//      HDC  hDC,
//      ULONG NumPairs,
//      LPKERNINGPAIR krnpair)
// {
//   PDC dc;
//   PDC_ATTR pdcattr;
//   PTEXTOBJ TextObj;
//   PFONTGDI FontGDI;
//   DWORD Count;
//   KERNINGPAIR *pKP;
//   NTSTATUS Status = STATUS_SUCCESS;
//
//   dc = DC_LockDc(hDC);
//   if (!dc)
//   {
//      EngSetLastError(ERROR_INVALID_HANDLE);
//      return 0;
//   }
//
//   pdcattr = dc->pdcattr;
//   TextObj = RealizeFontInit(pdcattr->hlfntNew);
//   DC_UnlockDc(dc);
//
//   if (!TextObj)
//   {
//      EngSetLastError(ERROR_INVALID_HANDLE);
//      return 0;
//   }
//
//   FontGDI = ObjToGDI!(TextObj->Font, FONT);
//   w32r_TEXTOBJ_UnlockText(TextObj);
//
//   Count = ftGdiGetKerningPairs(FontGDI,0,NULL);
//
//   if ( Count && krnpair )
//   {
//      if (Count > NumPairs)
//      {
//         EngSetLastError(ERROR_INSUFFICIENT_BUFFER);
//         return 0;
//      }
//      pKP = ExAllocatePoolWithTag(PagedPool, Count * sizeof(KERNINGPAIR), GDITAG_TEXT);
//      if (!pKP)
//      {
//         EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return 0;
//      }
//      ftGdiGetKerningPairs(FontGDI,Count,pKP);
//      Status = SafeProbeForWrite(krnpair, Count * sizeof(KERNINGPAIR), 1);
//      if (NT_SUCCESS(Status))
//      {
//          RtlCopyMemory(krnpair, pKP, Count * sizeof(KERNINGPAIR));
//      }
//      if (!NT_SUCCESS(Status))
//      {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         Count = 0;
//      }
//      ExFreePoolWithTag(pKP,GDITAG_TEXT);
//   }
//   return Count;
// }
//
// /*
//  From "Undocumented Windows 2000 Secrets" Appendix B, Table B-2, page
//  472, this is NtGdiGetOutlineTextMetricsInternalW.
//  */
// ULONG
// __stdcall
// NtGdiGetOutlineTextMetricsInternalW(
//      HDC hDC,
//      ULONG Data,
//      POUTLINETEXTMETRICW otm,
//      PTMDIFF Tmd)
// {
//   PDC dc;
//   PDC_ATTR pdcattr;
//   PTEXTOBJ TextObj;
//   PFONTGDI FontGDI;
//   HFONT hFont = 0;
//   ULONG Size;
//   OUTLINETEXTMETRICW *potm;
//   NTSTATUS Status = STATUS_SUCCESS;
//
//   dc = DC_LockDc(hDC);
//   if (!dc)
//   {
//      EngSetLastError(ERROR_INVALID_HANDLE);
//      return 0;
//   }
//   pdcattr = dc->pdcattr;
//   hFont = pdcattr->hlfntNew;
//   TextObj = RealizeFontInit(hFont);
//   DC_UnlockDc(dc);
//   if (!TextObj)
//   {
//      EngSetLastError(ERROR_INVALID_HANDLE);
//      return 0;
//   }
//   FontGDI = ObjToGDI!(TextObj->Font, FONT);
//   if (!(FontGDI->flType & FO_TYPE_TRUETYPE))
//   {
//      w32r_TEXTOBJ_UnlockText(TextObj);
//      return 0;
//   }
//   TextIntUpdateSize(dc, TextObj, FontGDI, TRUE);
//   w32r_TEXTOBJ_UnlockText(TextObj);
//   Size = IntGetOutlineTextMetrics(FontGDI, 0, NULL, FALSE);
//   if (!otm) return Size;
//   if (Size > Data)
//   {
//       EngSetLastError(ERROR_INSUFFICIENT_BUFFER);
//       return 0;
//   }
//   potm = ExAllocatePoolWithTag(PagedPool, Size, GDITAG_TEXT);
//   if (!potm)
//   {
//       EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//       return 0;
//   }
//   RtlZeroMemory(potm, Size);
//   IntGetOutlineTextMetrics(FontGDI, Size, potm, FALSE);
//
//   Status = SafeProbeForWrite(otm, Size, 1);
//   if (NT_SUCCESS(Status))
//   {
//       RtlCopyMemory(otm, potm, Size);
//   }
//
//   if (!NT_SUCCESS(Status))
//   {
//      EngSetLastError(ERROR_INVALID_PARAMETER);
//      Size = 0;
//   }
//
//   ExFreePoolWithTag(potm,GDITAG_TEXT);
//   return Size;
// }
//
// W32KAPI
// BOOL
// __stdcall
// NtGdiGetFontResourceInfoInternalW(
//      PCWCH pwszFiles,
//      ULONG cwc,
//      ULONG cFiles,
//      UINT cjIn,
//      PDWORD pdwBytes,
//      PVOID pvBuf,
//      DWORD dwType)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     DWORD dwBytes, dwBytesRequested;
//     UNICODE_STRING SafeFileNames;
//     BOOL bRet = FALSE;
//     ULONG cbStringSize;
//     LPVOID Buffer;
//
//     /* FIXME: Handle cFiles > 0 */
//
//     /* Check for valid dwType values */
//     if (dwType > 5)
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Allocate a safe unicode string buffer */
//     cbStringSize = cwc * sizeof(WCHAR);
//     SafeFileNames.MaximumLength = SafeFileNames.Length = (USHORT)cbStringSize - sizeof(WCHAR);
//     SafeFileNames.Buffer = ExAllocatePoolWithTag(PagedPool,
//                                                  cbStringSize,
//                                                  TAG_USTR);
//     if (!SafeFileNames.Buffer)
//     {
//         EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//     RtlZeroMemory(SafeFileNames.Buffer, SafeFileNames.MaximumLength);
//
//     /* Check buffers and copy pwszFiles to safe unicode string */
//     _SEH2_TRY
//     {
//         ProbeForRead(pwszFiles, cbStringSize, 1);
//         ProbeForWrite(pdwBytes, sizeof(DWORD), 1);
//         if (pvBuf)
//             ProbeForWrite(pvBuf, cjIn, 1);
//
//         dwBytes = *pdwBytes;
//         dwBytesRequested = dwBytes;
//
//         RtlCopyMemory(SafeFileNames.Buffer, pwszFiles, cbStringSize);
//         if (dwBytes > 0)
//         {
//             Buffer = ExAllocatePoolWithTag(PagedPool, dwBytes, TAG_FINF);
//         }
//         else
//         {
//             Buffer = ExAllocatePoolWithTag(PagedPool, sizeof(DWORD), TAG_FINF);
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END
//
//     if(!NT_SUCCESS(Status))
//     {
//         SetLastNtError(Status);
//         /* Free the string buffer for the safe filename */
//         ExFreePoolWithTag(SafeFileNames.Buffer, TAG_USTR);
//         return FALSE;
//     }
//
//     /* Do the actual call */
//     bRet = IntGdiGetFontResourceInfo(&SafeFileNames,
//                                      (pvBuf ? Buffer : NULL),
//                                      &dwBytes, dwType);
//
//     /* Check if succeeded */
//     if (bRet)
//     {
//         /* Copy the data back to caller */
//         _SEH2_TRY
//         {
//             /* Buffers are already probed */
//             if (pvBuf && dwBytesRequested > 0)
//                 RtlCopyMemory(pvBuf, Buffer, min(dwBytesRequested, dwBytes));
//             *pdwBytes = dwBytes;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END
//
//         if(!NT_SUCCESS(Status))
//         {
//             SetLastNtError(Status);
//             bRet = FALSE;
//         }
//     }
//
//     ExFreePoolWithTag(Buffer, TAG_FINF);
//     /* Free the string for the safe filenames */
//     ExFreePoolWithTag(SafeFileNames.Buffer, TAG_USTR);
//
//     return bRet;
// }
//
// /* @unimplemented */
// BOOL
// __stdcall
// NtGdiGetRealizationInfo(
//      HDC hdc,
//      PREALIZATION_INFO pri,
//      HFONT hf)
// {
//   PDC pDc;
//   PTEXTOBJ pTextObj;
//   PFONTGDI pFontGdi;
//   PDC_ATTR pdcattr;
//   BOOL Ret = FALSE;
//   INT i = 0;
//   REALIZATION_INFO ri;
//
//   pDc = DC_LockDc(hdc);
//   if (!pDc)
//   {
//      EngSetLastError(ERROR_INVALID_HANDLE);
//      return 0;
//   }
//   pdcattr = pDc->pdcattr;
//   pTextObj = RealizeFontInit(pdcattr->hlfntNew);
//   ASSERT(pTextObj != NULL);
//   pFontGdi = ObjToGDI!(pTextObj->Font, FONT);
//   w32r_TEXTOBJ_UnlockText(pTextObj);
//   DC_UnlockDc(pDc);
//
//   Ret = ftGdiRealizationInfo(pFontGdi, &ri);
//   if (Ret)
//   {
//      if (pri)
//      {
//         NTSTATUS Status = STATUS_SUCCESS;
//         Status = SafeProbeForWrite(pri, sizeof(REALIZATION_INFO), 1);
//         if (NT_SUCCESS(Status))
//         {
//             RtlCopyMemory(pri, &ri, sizeof(REALIZATION_INFO));
//         }
//
//         if(!NT_SUCCESS(Status))
//         {
//             SetLastNtError(Status);
//             return FALSE;
//         }
//      }
//      do
//      {
//         if (GdiHandleTable->cfPublic[i].hf == hf)
//         {
//            GdiHandleTable->cfPublic[i].iTechnology = ri.iTechnology;
//            GdiHandleTable->cfPublic[i].iUniq = ri.iUniq;
//            GdiHandleTable->cfPublic[i].dwUnknown = ri.dwUnknown;
//            GdiHandleTable->cfPublic[i].dwCFCount = GdiHandleTable->dwCFCount;
//            GdiHandleTable->cfPublic[i].fl |= CFONT_REALIZATION;
//         }
//         i++;
//      }
//      while ( i < GDI_CFONT_MAX );
//   }
//   return Ret;
// }
//
// HFONT
// __stdcall
// HfontCreate(
//     const ENUMLOGFONTEXDVW *pelfw,
//      ULONG cjElfw,
//      LFTYPE lft,
//      FLONG  fl,
//      PVOID pvCliData)
// {
//   HFONT hNewFont;
//   PLFONT plfont;
//
//   if (!pelfw)
//   {
//       return NULL;
//   }
//
//   plfont = LFONT_AllocFontWithHandle();
//   if (!plfont)
//   {
//       return NULL;
//   }
//   hNewFont = plfont->BaseObject.hHmgr;
//
//   plfont->lft = lft;
//   plfont->fl  = fl;
//   RtlCopyMemory (&plfont->logfont, pelfw, sizeof(ENUMLOGFONTEXDVW));
//   ExInitializePushLock(&plfont->lock);
//
//   if (pelfw->elfEnumLogfontEx.elfLogFont.lfEscapement !=
//       pelfw->elfEnumLogfontEx.elfLogFont.lfOrientation)
//   {
//     /* This should really depend on whether GM_ADVANCED is set */
//     plfont->logfont.elfEnumLogfontEx.elfLogFont.lfOrientation =
//     plfont->logfont.elfEnumLogfontEx.elfLogFont.lfEscapement;
//   }
//   LFONT_UnlockFont(plfont);
//
//   if (pvCliData && hNewFont)
//   {
//     // FIXME: Use GDIOBJ_InsertUserData
//     KeEnterCriticalRegion();
//     {
//        INT Index = GDI_HANDLE_GET_INDEX((HGDIOBJ)hNewFont);
//        PGDI_TABLE_ENTRY Entry = &GdiHandleTable->Entries[Index];
//        Entry->UserData = pvCliData;
//     }
//     KeLeaveCriticalRegion();
//   }
//
//   return hNewFont;
// }
//
//
// HFONT
// __stdcall
// NtGdiHfontCreate(
//      const ENUMLOGFONTEXDVW *pelfw,
//      ULONG cjElfw,
//      LFTYPE lft,
//      FLONG  fl,
//      PVOID pvCliData)
// {
//   ENUMLOGFONTEXDVW SafeLogfont;
//   NTSTATUS Status = STATUS_SUCCESS;
//
//   /* Silence GCC warnings */
//   SafeLogfont.elfEnumLogfontEx.elfLogFont.lfEscapement = 0;
//   SafeLogfont.elfEnumLogfontEx.elfLogFont.lfOrientation = 0;
//
//   if (!pelfw)
//   {
//       return NULL;
//   }
//
//   Status = SafeProbeForRead(pelfw, sizeof(ENUMLOGFONTEXDVW), 1);
//   if (NT_SUCCESS(Status))
//   {
//       RtlCopyMemory(&SafeLogfont, pelfw, sizeof(ENUMLOGFONTEXDVW));
//   }
//
//   if (!NT_SUCCESS(Status))
//   {
//       return NULL;
//   }
//
//   return HfontCreate(&SafeLogfont, cjElfw, lft, fl, pvCliData);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
