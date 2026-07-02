//! win32ss/gdi/gdi32/objects/text.c → Rust. GDI32 text-drawing client API:
//! TextOut{A,W} / ExtTextOut{A,W} (batch ring) / PolyTextOut{A,W} / GetTextExtent*
//! / GetText{Metrics,Face,Align,Color,Charset} / SetText{Align,Color,Justification,
//! CharacterExtra} / GdiGetCodePage / GetFontResourceInfoW / GetStringBitmap{A,W} / GetETM.
//
// NEEDS-TYPE: none — all scalar aliases + GDI types come from `win32k_sys::win32k`
//   (BOOL/DWORD/UINT/INT/USHORT/ULONG/LONG/NTSTATUS/COLORREF/FLONG/WORD/WCHAR/HDC/HANDLE/
//   PVOID/PWSTR/LPWSTR/PCSTR/LPCSTR/LPCWSTR/PCHAR/PWCH/PCWCH/PULONG/LPINT/LPWORD/LPDWORD/
//   LPSIZE/PSIZE/LPCRECTL/RECT/RECTL/POINTL/TEXTMETRICW/LPTEXTMETRICA/LPTEXTMETRICW/
//   TMW_INTERNAL/EXTTEXTMETRIC/POLYTEXTA/POLYTEXTW/STRING(ANSI_STRING)/UNICODE_STRING/
//   PUNICODE_STRING/EXTTEXTMETRIC/PLDC/PGDIBSTEXTOUT/PGDIBSEXTTEXTOUT/PGDIBATCHHDR).
// NEEDS-IMPORT: CP_ACP (0), LPK_ETO (2), LPK_GTEP (4) are not in win32k_sys bindgen —
//   reproduced as local consts (psdk values, identical to winnls.h / gdi32p.h).
// NEEDS-IMPORT: `HEAP_free` (gdi32p.h static-inline FASTCALL) is reproduced locally as
//   `RtlFreeHeap(RtlGetProcessHeap(), 0, mem)` (same as the C inline; brush.rs precedent).
// NEEDS-IMPORT: the LPK hook globals `LpkExtTextOut`/`LpkGetTextExtentExPoint` (gdi32p.h
//   `extern LPKETO/LPKGTEP`) are data fn-pointer symbols → `extern "C" { static mut .. }`.
// NEEDS-IMPORT: `LoadLPK`/`GdiConvertAndCheckDC`/`GdiSetLastError`/`GetKerningPairsA` are
//   gdi32 WINAPI exports (other objects); `METADC_*`/`EMFDC_*` plain cdecl (DECLSPEC_HIDDEN);
//   `GdiGetLDC`/`FONT_TextMetricWToA`/`HEAP_strdupA2W` FASTCALL; Rtl*/kernel32 via import libs.

use win32k_sys::win32k::*;
use crate::gdihelp::*;

// win32k_sys's FALSE/TRUE are bindgen u32 (#define), but BOOL is i32 — shadow with
// BOOL-typed locals so `== FALSE`/`return TRUE` type-check against the i32 BOOL returns.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// Not in win32k_sys bindgen — psdk constants (winnls.h CP_ACP, gdi32p.h LPK_* IDs).
const CP_ACP: UINT = 0;
const LPK_ETO: INT = 2;
const LPK_GTEP: INT = 4;

// ── LPK hook function-pointer types (gdi32p.h LPKETO / LPKGTEP, WINAPI) ─────────
type LPKETO = unsafe extern "system" fn(
    hdc: HDC,
    x: INT,
    y: INT,
    fuOptions: UINT,
    lprc: *const RECT,
    lpString: LPCWSTR,
    uCount: UINT,
    lpDx: *const INT,
    unknown: INT,
) -> BOOL;

type LPKGTEP = unsafe extern "system" fn(
    hdc: HDC,
    lpString: LPCWSTR,
    cString: INT,
    nMaxExtent: INT,
    lpnFit: LPINT,
    lpnDx: LPINT,
    lpSize: LPSIZE,
    dwUnused: DWORD,
    unknown: INT,
) -> BOOL;

// ── Cross-file gdi32 / kernel32 / ntdll fns (WINAPI / NTAPI → stdcall on i386) ──
extern "system" {
    fn SetLastError(dwErrCode: DWORD);
    fn GdiSetLastError(dwErrCode: DWORD); // misc.rs (WINAPI)
    fn GdiConvertAndCheckDC(hdc: HDC) -> HDC;
    fn LoadLPK(LpkFunctionID: INT) -> BOOL;
    fn GetKerningPairsA(hdc: HDC, nPairs: DWORD, lpKernPair: PVOID) -> DWORD;
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn GetProcessHeap() -> HANDLE;
    fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: LPCWSTR,
        cchWideChar: INT,
        lpMultiByteStr: LPSTR,
        cbMultiByte: INT,
        lpDefaultChar: LPCSTR,
        lpUsedDefaultChar: *mut BOOL,
    ) -> INT;
    // ntdll NTAPI
    fn RtlAnsiStringToUnicodeString(
        DestinationString: PUNICODE_STRING,
        SourceString: *const STRING,
        AllocateDestinationString: BOOLEAN,
    ) -> NTSTATUS;
    fn RtlInitAnsiString(DestinationString: *mut STRING, SourceString: PCSTR);
    fn RtlFreeUnicodeString(UnicodeString: PUNICODE_STRING);
    fn RtlNtStatusToDosError(Status: NTSTATUS) -> ULONG;
    fn RtlDosPathNameToNtPathName_U(
        DosFileName: LPCWSTR,
        NtFileName: PUNICODE_STRING,
        FilePart: *mut PWSTR,
        Reserved: PVOID,
    ) -> BOOLEAN;
    fn RtlFreeHeap(HeapHandle: HANDLE, Flags: ULONG, P: PVOID) -> BOOLEAN;
}

// ── Plain cdecl gdi32-private metafile glue (DECLSPEC_HIDDEN, no WINAPI) ────────
extern "C" {
    fn METADC_ExtTextOut(
        hdc: HDC,
        x: INT,
        y: INT,
        flags: UINT,
        rect: *const RECT,
        str_: LPCWSTR,
        count: UINT,
        dx: *const INT,
    ) -> BOOL;
    fn METADC_SetTextAlign(hdc: HDC, align: UINT) -> BOOL;
    fn METADC_SetTextColor(hdc: HDC, color: COLORREF) -> BOOL;
    fn METADC_SetTextCharacterExtra(hdc: HDC, extra: INT) -> BOOL;
    fn METADC_SetTextJustification(hdc: HDC, extra: INT, breaks: INT) -> BOOL;
    fn EMFDC_ExtTextOut(
        dc_attr: PLDC,
        x: INT,
        y: INT,
        flags: UINT,
        rect: *const RECT,
        str_: *const WCHAR,
        count: UINT,
        dx: *const INT,
    ) -> BOOL;
    fn EMFDC_SetTextAlign(dc_attr: PLDC, align: UINT) -> BOOL;
    fn EMFDC_SetTextColor(dc_attr: PLDC, color: COLORREF) -> BOOL;
    fn strlen(s: *const core::ffi::c_char) -> usize;
}

// ── FASTCALL gdi32-private helpers ─────────────────────────────────────────────
extern "fastcall" {
    fn GdiGetLDC(hdc: HDC) -> PLDC; // misc.rs
    fn FONT_TextMetricWToA(ptmW: *const TEXTMETRICW, ptmA: LPTEXTMETRICA);
    fn HEAP_strdupA2W(ppszW: *mut PWSTR, lpszA: PCSTR) -> NTSTATUS;
}

// ── LPK hook globals (gdi32p.h `extern LPKETO/LPKGTEP`, data symbols) ───────────
extern "C" {
    static mut LpkExtTextOut: LPKETO;
    static mut LpkGetTextExtentExPoint: LPKGTEP;
}

/// `RtlGetProcessHeap()` — FORCEINLINE macro `NtCurrentPeb()->ProcessHeap` (rtlfuncs.h),
/// reproduced over the win32k_sys _TEB/_PEB (NtCurrentTeb lives in crate::gdihelp).
#[inline]
unsafe fn RtlGetProcessHeap() -> PVOID {
    (*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessHeap
}

/// `HEAP_free` (gdi32p.h static-inline FASTCALL) — `RtlFreeHeap(hProcessHeap, 0, memory)`.
#[inline]
unsafe fn HEAP_free(memory: PVOID) {
    RtlFreeHeap(RtlGetProcessHeap(), 0, memory);
}

/// `NT_SUCCESS(Status)` — `Status >= 0` (NTSTATUS is signed LONG).
#[inline]
fn NT_SUCCESS(Status: NTSTATUS) -> bool {
    Status >= 0
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn TextOutA(
    hdc: HDC,
    nXStart: INT,
    nYStart: INT,
    lpString: LPCSTR,
    mut cchString: INT,
) -> BOOL {
    let mut StringA: STRING = core::mem::zeroed();
    let mut StringU: UNICODE_STRING = core::mem::zeroed();
    let bResult: BOOL;
    let Status: NTSTATUS;

    if !lpString.is_null() && cchString > 0 {
        if cchString > MAXUSHORT as INT {
            cchString = MAXUSHORT as INT;
        }

        StringA.Length = cchString as USHORT;
        StringA.MaximumLength = cchString as USHORT;
        StringA.Buffer = lpString as PCHAR;

        Status = RtlAnsiStringToUnicodeString(&mut StringU, &StringA, TRUE as BOOLEAN);
        if !NT_SUCCESS(Status) {
            StringU.Buffer = core::ptr::null_mut();
            StringU.Length = 0;
        }
    } else {
        StringU.Buffer = core::ptr::null_mut();
        StringU.Length = 0;
    }

    bResult = TextOutW(
        hdc,
        nXStart,
        nYStart,
        StringU.Buffer,
        (StringU.Length / core::mem::size_of::<WCHAR>() as USHORT) as INT,
    );

    RtlFreeUnicodeString(&mut StringU);
    bResult
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn TextOutW(
    hdc: HDC,
    nXStart: INT,
    nYStart: INT,
    lpString: LPCWSTR,
    cchString: INT,
) -> BOOL {
    ExtTextOutW(
        hdc,
        nXStart,
        nYStart,
        0,
        core::ptr::null(),
        lpString,
        cchString as UINT,
        core::ptr::null(),
    )
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn PolyTextOutA(
    hdc: HDC,
    mut pptxt: *const POLYTEXTA,
    mut cStrings: INT,
) -> BOOL {
    while cStrings > 0 {
        if ExtTextOutA(
            hdc,
            (*pptxt).x,
            (*pptxt).y,
            (*pptxt).uiFlags,
            &(*pptxt).rcl as *const RECTL as *const RECT,
            (*pptxt).lpstr,
            (*pptxt).n,
            (*pptxt).pdx as *const INT,
        ) == FALSE
        {
            return FALSE;
        }
        cStrings -= 1;
        pptxt = pptxt.add(1);
    }

    TRUE
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn PolyTextOutW(
    hdc: HDC,
    mut pptxt: *const POLYTEXTW,
    mut cStrings: INT,
) -> BOOL {
    while cStrings > 0 {
        if ExtTextOutW(
            hdc,
            (*pptxt).x,
            (*pptxt).y,
            (*pptxt).uiFlags,
            &(*pptxt).rcl as *const RECTL as *const RECT,
            (*pptxt).lpstr,
            (*pptxt).n,
            (*pptxt).pdx as *const INT,
        ) == FALSE
        {
            return FALSE;
        }
        cStrings -= 1;
        pptxt = pptxt.add(1);
    }

    TRUE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiGetCodePage(hdc: HDC) -> DWORD {
    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if (*pdcattr).ulDirty_ & DIRTY_CHARSET != 0 {
        return NtGdiGetCharSet(hdc) & 0xffff;
    }

    (*pdcattr).iCS_CP & 0xffff
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetTextCharacterExtra(hdc: HDC) -> INT {
    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Do not set LastError here! */
        return 0x8000000;
    }

    (*pdcattr).lTextExtra
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetTextCharset(hdc: HDC) -> INT {
    /* MSDN docs say this is equivalent */
    NtGdiGetTextCharsetInfo(hdc, core::ptr::null_mut(), 0)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetTextMetricsA(hdc: HDC, lptm: LPTEXTMETRICA) -> BOOL {
    let mut tmwi: TMW_INTERNAL = core::mem::zeroed();

    if NtGdiGetTextMetricsW(hdc, &mut tmwi, core::mem::size_of::<TMW_INTERNAL>() as ULONG)
        == FALSE
    {
        return FALSE;
    }

    FONT_TextMetricWToA(&tmwi.TextMetric, lptm);
    TRUE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetTextMetricsW(hdc: HDC, lptm: LPTEXTMETRICW) -> BOOL {
    let mut tmwi: TMW_INTERNAL = core::mem::zeroed();

    if NtGdiGetTextMetricsW(hdc, &mut tmwi, core::mem::size_of::<TMW_INTERNAL>() as ULONG)
        == FALSE
    {
        return FALSE;
    }

    *lptm = tmwi.TextMetric;
    TRUE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetTextExtentPointA(
    hdc: HDC,
    lpString: LPCSTR,
    cchString: INT,
    lpsz: LPSIZE,
) -> BOOL {
    let mut StringA: STRING = core::mem::zeroed();
    let mut StringU: UNICODE_STRING = core::mem::zeroed();
    let ret: BOOL;

    /* FIXME: lpString can be non-NUL-terminated */
    RtlInitAnsiString(&mut StringA, lpString);
    RtlAnsiStringToUnicodeString(&mut StringU, &StringA, TRUE as BOOLEAN);

    ret = GetTextExtentPointW(hdc, StringU.Buffer, cchString, lpsz);

    RtlFreeUnicodeString(&mut StringU);

    ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetTextExtentPointW(
    hdc: HDC,
    lpString: LPCWSTR,
    cchString: INT,
    lpsz: LPSIZE,
) -> BOOL {
    NtGdiGetTextExtent(hdc, lpString as PCWCH, cchString, lpsz as PSIZE, 0)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetTextExtentExPointW(
    hdc: HDC,
    lpszString: LPCWSTR,
    cchString: INT,
    nMaxExtent: INT,
    lpnFit: LPINT,
    lpnDx: LPINT,
    lpSize: LPSIZE,
) -> BOOL {
    /* Windows doesn't check nMaxExtent validity in unicode version */
    if nMaxExtent < -1 {
        // DPRINT("nMaxExtent is invalid: %d\n", nMaxExtent);
    }

    if LoadLPK(LPK_GTEP) != FALSE {
        return LpkGetTextExtentExPoint(
            hdc, lpszString, cchString, nMaxExtent, lpnFit, lpnDx, lpSize, 0, 0,
        );
    }

    NtGdiGetTextExtentExW(
        hdc,
        lpszString as PCWCH,
        cchString as ULONG,
        nMaxExtent as ULONG,
        lpnFit as PULONG,
        lpnDx as PULONG,
        lpSize as PSIZE,
        0,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetTextExtentExPointWPri(
    hdc: HDC,
    lpwsz: PCWCH,
    cwc: INT,
    dxMax: INT,
    pcCh: LPINT,
    pdxOut: LPINT,
    psize: LPSIZE,
) -> BOOL {
    NtGdiGetTextExtentExW(
        hdc,
        lpwsz,
        cwc as ULONG,
        dxMax as ULONG,
        pcCh as PULONG,
        pdxOut as PULONG,
        psize as PSIZE,
        0,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetTextExtentExPointA(
    hdc: HDC,
    lpszStr: LPCSTR,
    cchString: INT,
    nMaxExtent: INT,
    lpnFit: LPINT,
    lpnDx: LPINT,
    lpSize: LPSIZE,
) -> BOOL {
    let Status: NTSTATUS;
    let mut lpszStrW: LPWSTR = core::ptr::null_mut();
    let bResult: BOOL;

    if nMaxExtent < -1 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    Status = HEAP_strdupA2W(&mut lpszStrW, lpszStr);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return FALSE;
    }

    bResult = NtGdiGetTextExtentExW(
        hdc,
        lpszStrW as PCWCH,
        cchString as ULONG,
        nMaxExtent as ULONG,
        lpnFit as PULONG,
        lpnDx as PULONG,
        lpSize as PSIZE,
        0,
    );

    HEAP_free(lpszStrW as PVOID);

    bResult
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetTextExtentPoint32A(
    hdc: HDC,
    lpString: LPCSTR,
    cchString: INT,
    lpSize: LPSIZE,
) -> BOOL {
    let mut StringA: STRING = core::mem::zeroed();
    let mut StringU: UNICODE_STRING = core::mem::zeroed();
    let ret: BOOL;

    StringA.Buffer = lpString as PCHAR;
    StringA.Length = cchString as USHORT;
    RtlAnsiStringToUnicodeString(&mut StringU, &StringA, TRUE as BOOLEAN);

    ret = GetTextExtentPoint32W(hdc, StringU.Buffer, cchString, lpSize);

    RtlFreeUnicodeString(&mut StringU);

    ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetTextExtentPoint32W(
    hdc: HDC,
    lpString: LPCWSTR,
    cchString: INT,
    lpSize: LPSIZE,
) -> BOOL {
    NtGdiGetTextExtent(hdc, lpString as PCWCH, cchString, lpSize as PSIZE, 0)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetTextExtentExPointI(
    hdc: HDC,
    pgiIn: LPWORD,
    cgi: INT,
    nMaxExtent: INT,
    lpnFit: LPINT,
    lpnDx: LPINT,
    lpSize: LPSIZE,
) -> BOOL {
    NtGdiGetTextExtentExW(
        hdc,
        pgiIn as PCWCH,
        cgi as ULONG,
        nMaxExtent as ULONG,
        lpnFit as PULONG,
        lpnDx as PULONG,
        lpSize as PSIZE,
        GTEF_INDICES as FLONG,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetTextExtentPointI(
    hdc: HDC,
    pgiIn: LPWORD,
    cgi: INT,
    lpSize: LPSIZE,
) -> BOOL {
    NtGdiGetTextExtent(hdc, pgiIn as PCWCH, cgi, lpSize as PSIZE, GTEF_INDICES)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn ExtTextOutA(
    hdc: HDC,
    x: INT,
    y: INT,
    fuOptions: UINT,
    lprc: *const RECT,
    lpString: LPCSTR,
    cch: UINT,
    lpDx: *const INT,
) -> BOOL {
    let mut StringA: STRING = core::mem::zeroed();
    let mut StringU: UNICODE_STRING = core::mem::zeroed();
    let ret: BOOL;

    if fuOptions & ETO_GLYPH_INDEX != 0 {
        return ExtTextOutW(hdc, x, y, fuOptions, lprc, lpString as LPCWSTR, cch, lpDx);
    }

    StringA.Buffer = lpString as PCHAR;
    StringA.Length = cch as USHORT;
    StringA.MaximumLength = cch as USHORT;
    RtlAnsiStringToUnicodeString(&mut StringU, &StringA, TRUE as BOOLEAN);

    // C promotes both USHORTs to int before the multiply/compare (avoid u16 overflow).
    if StringU.Length as u32 != StringA.Length as u32 * core::mem::size_of::<WCHAR>() as u32 {
        // DPRINT1("ERROR: Should convert lpDx properly!\n");
    }

    ret = ExtTextOutW(hdc, x, y, fuOptions, lprc, StringU.Buffer, cch, lpDx);

    RtlFreeUnicodeString(&mut StringU);

    ret
}

static mut bBypassETOWMF: BOOL = FALSE;

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn ExtTextOutW(
    hdc: HDC,
    x: INT,
    y: INT,
    mut fuOptions: UINT,
    mut lprc: *const RECT,
    lpString: LPCWSTR,
    cwc: UINT,
    lpDx: *const INT,
) -> BOOL {
    let pdcattr: PDC_ATTR;

    // Need both, should return a parameter error? No they don't!
    if lpDx.is_null() && (fuOptions & ETO_PDY != 0) {
        return FALSE;
    }

    // Now sorting out rectangle.

    // Here again, need both.
    if !lprc.is_null() && (fuOptions & (ETO_CLIPPED | ETO_OPAQUE) == 0) {
        lprc = core::ptr::null(); // No flags, no rectangle.
    } else if lprc.is_null() {
        // No rectangle, force clear flags if set and continue.
        fuOptions &= !(ETO_CLIPPED | ETO_OPAQUE);
    }

    if bBypassETOWMF == FALSE {
        // HANDLE_METADC(BOOL, ExtTextOut, FALSE, hdc, x, y, fuOptions, lprc, lpString, cwc, lpDx)
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
            != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize
        {
            if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
                == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
            {
                return METADC_ExtTextOut(hdc, x, y, fuOptions, lprc, lpString, cwc, lpDx);
            } else {
                let pLDC: PLDC = GdiGetLDC(hdc);
                let _Ret: BOOL = FALSE;
                if pLDC.is_null() {
                    SetLastError(ERROR_INVALID_HANDLE);
                    return _Ret;
                }
                if (*pLDC).iType == LDC_EMFLDC as INT
                    && EMFDC_ExtTextOut(pLDC, x, y, fuOptions, lprc, lpString, cwc, lpDx)
                        == FALSE
                {
                    return _Ret;
                }
                /*  Fall through to support information DC's.*/
            }
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    if fuOptions & (ETO_GLYPH_INDEX | ETO_IGNORELANGUAGE) == 0 {
        bBypassETOWMF = TRUE;

        if LoadLPK(LPK_ETO) != FALSE {
            return LpkExtTextOut(hdc, x, y, fuOptions, lprc, lpString, cwc, lpDx, 0);
        }
    } else {
        bBypassETOWMF = FALSE;
    }

    /* Get the DC attribute */
    pdcattr = GdiGetDcAttr(hdc);
    if !pdcattr.is_null()
        && (*pdcattr).ulDirty_ & DC_DIBSECTION == 0
        && (*pdcattr).lTextAlign & TA_UPDATECP as LONG == 0
    {
        if !lprc.is_null() && cwc == 0 {
            if fuOptions & ETO_OPAQUE != 0 {
                let pgO: PGDIBSEXTTEXTOUT =
                    GdiAllocBatchCommand(hdc, crate::gdihelp::GdiBCExtTextOut) as PGDIBSEXTTEXTOUT;
                if !pgO.is_null() {
                    (*pdcattr).ulDirty_ |= DC_MODE_DIRTY;
                    (*pgO).Count = cwc;
                    (*pgO).Rect = *(lprc as *const RECTL);
                    (*pgO).Options = fuOptions;
                    /* Snapshot attribute */
                    (*pgO).ulBackgroundClr = (*pdcattr).ulBackgroundClr;
                    (*pgO).ptlViewportOrg = (*pdcattr).ptlViewportOrg;
                    return TRUE;
                }
            } else {
                // Do nothing, old explorer pops this off.
                // DPRINT("GdiBCExtTextOut nothing\n");
                return TRUE;
            }
        }
        // Max 580 wchars, if offset 0
        else if cwc as ULONG
            <= ((GDIBATCHBUFSIZE - core::mem::size_of::<GDIBSTEXTOUT>() as u32)
                / core::mem::size_of::<WCHAR>() as u32) as ULONG
        {
            let pgO: PGDIBSTEXTOUT =
                GdiAllocBatchCommand(hdc, crate::gdihelp::GdiBCTextOut) as PGDIBSTEXTOUT;
            let pTeb: PTEB = NtCurrentTeb();

            if !pgO.is_null() {
                let mut cjSize: USHORT = 0;
                let mut DxSize: ULONG = 0;

                if cwc > 2 {
                    cjSize = (cwc * core::mem::size_of::<WCHAR>() as UINT
                        - core::mem::size_of::<[WCHAR; 2]>() as UINT)
                        as USHORT;
                }

                /* Calculate buffer size for string and Dx values */
                if !lpDx.is_null() {
                    /* If ETO_PDY is specified, we have pairs of INTs */
                    DxSize = (cwc * core::mem::size_of::<INT>() as UINT)
                        * (if fuOptions & ETO_PDY != 0 { 2 } else { 1 });
                    cjSize = cjSize.wrapping_add(DxSize as USHORT);
                    // The structure buffer holds 4 bytes. Store Dx data then string.
                    // Result one wchar -> Buf[ Dx ]Str[wC], [4][2][X] one extra unused wchar
                    // to assure alignment of 4.
                }

                if ((*pTeb).GdiTebBatch.Offset + cjSize as ULONG) <= GDIBATCHBUFSIZE {
                    (*pdcattr).ulDirty_ |= DC_MODE_DIRTY | DC_FONTTEXT_DIRTY;
                    (*pgO).cbCount = cwc;
                    (*pgO).x = x;
                    (*pgO).y = y;
                    (*pgO).Options = fuOptions;
                    (*pgO).iCS_CP = 0;

                    if !lprc.is_null() {
                        (*pgO).Rect = *(lprc as *const RECTL);
                    } else {
                        (*pgO).Options |= GDIBS_NORECT; // Tell the other side lprc is nill.
                    }

                    /* Snapshot attributes */
                    (*pgO).crForegroundClr = (*pdcattr).crForegroundClr;
                    (*pgO).crBackgroundClr = (*pdcattr).crBackgroundClr;
                    (*pgO).ulForegroundClr = (*pdcattr).ulForegroundClr;
                    (*pgO).ulBackgroundClr = (*pdcattr).ulBackgroundClr;
                    (*pgO).lBkMode = if (*pdcattr).lBkMode == OPAQUE as LONG {
                        OPAQUE as LONG
                    } else {
                        TRANSPARENT as LONG
                    };
                    (*pgO).hlfntNew = (*pdcattr).hlfntNew;
                    (*pgO).flTextAlign = (*pdcattr).flTextAlign;
                    (*pgO).ptlViewportOrg = (*pdcattr).ptlViewportOrg;

                    (*pgO).Size = DxSize; // of lpDx then string after.
                    /* Put the Dx before the String to assure alignment of 4 */
                    if !lpDx.is_null() {
                        core::ptr::copy_nonoverlapping(
                            lpDx as *const u8,
                            core::ptr::addr_of_mut!((*pgO).__bindgen_anon_1.Buffer) as *mut u8,
                            DxSize as usize,
                        );
                    }

                    if cwc != 0 {
                        let strbase =
                            core::ptr::addr_of_mut!((*pgO).__bindgen_anon_1.String) as *mut WCHAR;
                        core::ptr::copy_nonoverlapping(
                            lpString as *const u8,
                            strbase.add((DxSize / core::mem::size_of::<WCHAR>() as ULONG) as usize)
                                as *mut u8,
                            (cwc * core::mem::size_of::<WCHAR>() as UINT) as usize,
                        );
                    }

                    // Recompute offset and return size
                    (*pTeb).GdiTebBatch.Offset += cjSize as ULONG;
                    (*(pgO as PGDIBATCHHDR)).Size =
                        (*(pgO as PGDIBATCHHDR)).Size.wrapping_add(cjSize as SHORT);
                    return TRUE;
                }
                // Reset offset and count then fall through
                (*pTeb).GdiTebBatch.Offset -= core::mem::size_of::<GDIBSTEXTOUT>() as ULONG;
                (*pTeb).GdiBatchCount -= 1;
            }
        }
    }
    NtGdiExtTextOutW(
        hdc,
        x,
        y,
        fuOptions,
        lprc as LPCRECTL,
        lpString as PCWCH,
        cwc,
        lpDx,
        0,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetTextFaceW(
    hdc: HDC,
    cwcMax: INT,
    pFaceName: LPWSTR,
) -> INT {
    /* Validate parameters */
    if !pFaceName.is_null() && cwcMax <= 0 {
        /* Set last error and return failure */
        GdiSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    /* Forward to kernel */
    NtGdiGetTextFaceW(hdc, cwcMax, pFaceName, FALSE)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetTextFaceA(hdc: HDC, cchMax: INT, lpName: LPSTR) -> INT {
    let mut res: INT;
    let nameW: LPWSTR;

    /* Validate parameters */
    if !lpName.is_null() && cchMax <= 0 {
        /* Set last error and return failure */
        GdiSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    res = GetTextFaceW(hdc, 0, core::ptr::null_mut());
    nameW = HeapAlloc(GetProcessHeap(), 0, (res * 2) as SIZE_T) as LPWSTR;
    if nameW.is_null() {
        return 0;
    }

    GetTextFaceW(hdc, res, nameW);

    if !lpName.is_null() {
        if cchMax != 0
            && WideCharToMultiByte(
                CP_ACP,
                0,
                nameW,
                -1,
                lpName,
                cchMax,
                core::ptr::null(),
                core::ptr::null_mut(),
            ) == 0
        {
            *lpName.offset((cchMax - 1) as isize) = 0;
        }
        res = strlen(lpName) as INT;
    } else {
        res = WideCharToMultiByte(
            CP_ACP,
            0,
            nameW,
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
    }

    HeapFree(GetProcessHeap(), 0, nameW as LPVOID);
    res
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetTextFaceAliasW(
    hdc: HDC,
    cwcMax: INT,
    pszOut: LPWSTR,
) -> INT {
    if !pszOut.is_null() && cwcMax == 0 {
        GdiSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    NtGdiGetTextFaceW(hdc, cwcMax, pszOut, TRUE)
}

#[no_mangle]
pub unsafe extern "system" fn GetFontResourceInfoW(
    lpFileName: LPCWSTR,
    pdwBufSize: *mut DWORD,
    lpBuffer: PVOID,
    dwType: DWORD,
) -> BOOL {
    let bRet: BOOL;
    let mut NtFileName: UNICODE_STRING = core::mem::zeroed();

    // DPRINT("GetFontResourceInfoW: dwType = %lu\n", dwType);

    if lpFileName.is_null() || pdwBufSize.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    if RtlDosPathNameToNtPathName_U(
        lpFileName,
        &mut NtFileName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        SetLastError(ERROR_PATH_NOT_FOUND);
        return FALSE;
    }

    bRet = NtGdiGetFontResourceInfoInternalW(
        NtFileName.Buffer,
        (NtFileName.Length / core::mem::size_of::<WCHAR>() as USHORT) as ULONG + 1,
        1,
        *pdwBufSize,
        pdwBufSize as LPDWORD,
        lpBuffer as LPVOID,
        dwType,
    );

    RtlFreeHeap(RtlGetProcessHeap(), 0, NtFileName.Buffer as PVOID);

    bRet
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetTextCharacterExtra(hdc: HDC, nCharExtra: INT) -> INT {
    let pdcattr: PDC_ATTR;
    let nOldCharExtra: INT;

    if nCharExtra == 0x80000000u32 as INT {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0x80000000u32 as INT;
    }

    // HANDLE_METADC16(INT, SetTextCharacterExtra, 0x80000000, hdc, nCharExtra)
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
            == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
        {
            return METADC_SetTextCharacterExtra(hdc, nCharExtra) as INT;
        }
    }

    /* Get the DC attribute */
    pdcattr = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0x8000000;
    }

    if (*NtCurrentTeb()).GdiTebBatch.HDC == hdc as HANDLE {
        if (*pdcattr).ulDirty_ & DC_FONTTEXT_DIRTY != 0 {
            NtGdiFlush(); // Sync up pdcattr from Kernel space.
            (*pdcattr).ulDirty_ &= !(DC_MODE_DIRTY | DC_FONTTEXT_DIRTY);
        }
    }

    nOldCharExtra = (*pdcattr).lTextExtra;
    (*pdcattr).lTextExtra = nCharExtra;
    nOldCharExtra
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetTextAlign(hdc: HDC) -> UINT {
    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Do not set LastError here! */
        return GDI_ERROR;
    }

    (*pdcattr).lTextAlign as UINT
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetTextColor(hdc: HDC) -> COLORREF {
    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Do not set LastError here! */
        return CLR_INVALID;
    }

    (*pdcattr).ulForegroundClr
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetTextAlign(hdc: HDC, mut fMode: UINT) -> UINT {
    let pdcattr: PDC_ATTR;
    let fOldMode: UINT;

    // HANDLE_METADC(BOOL, SetTextAlign, GDI_ERROR, hdc, fMode)
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
            == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
        {
            return (METADC_SetTextAlign(hdc, fMode) as BOOL) as UINT;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = GDI_ERROR as BOOL;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret as UINT;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_SetTextAlign(pLDC, fMode) == FALSE
            {
                return _Ret as UINT;
            }
            /*  Fall through to support information DC's.*/
        }
    }

    /* Get the DC attribute */
    pdcattr = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return GDI_ERROR;
    }

    fOldMode = (*pdcattr).lTextAlign as UINT;
    (*pdcattr).lTextAlign = fMode as LONG; // Raw
    if (*pdcattr).dwLayout & LAYOUT_RTL != 0 {
        if (fMode & TA_CENTER) != TA_CENTER {
            fMode ^= TA_RIGHT;
        }
    }

    (*pdcattr).flTextAlign = fMode & TA_MASK;
    fOldMode
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetTextColor(hdc: HDC, crColor: COLORREF) -> COLORREF {
    let pdcattr: PDC_ATTR;
    let crOldColor: COLORREF;

    // HANDLE_METADC(COLORREF, SetTextColor, CLR_INVALID, hdc, crColor)
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
            == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
        {
            return METADC_SetTextColor(hdc, crColor) as COLORREF;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: COLORREF = CLR_INVALID;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_SetTextColor(pLDC, crColor) == FALSE
            {
                return _Ret;
            }
            /*  Fall through to support information DC's.*/
        }
    }

    pdcattr = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return CLR_INVALID;
    }

    crOldColor = (*pdcattr).ulForegroundClr as COLORREF;
    (*pdcattr).ulForegroundClr = crColor as ULONG;

    if (*pdcattr).crForegroundClr != crColor {
        (*pdcattr).ulDirty_ |= DIRTY_TEXT | DIRTY_LINE | DIRTY_FILL;
        (*pdcattr).crForegroundClr = crColor;
    }

    crOldColor
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetTextJustification(
    hdc: HDC,
    nBreakExtra: INT,
    nBreakCount: INT,
) -> BOOL {
    let pdcattr: PDC_ATTR;

    // HANDLE_METADC16(BOOL, SetTextJustification, FALSE, hdc, nBreakExtra, nBreakCount)
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
            == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
        {
            return METADC_SetTextJustification(hdc, nBreakExtra, nBreakCount);
        }
    }

    /* Get the DC attribute */
    pdcattr = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Do not set LastError here! */
        return GDI_ERROR as BOOL;
    }

    if (*NtCurrentTeb()).GdiTebBatch.HDC == hdc as HANDLE {
        if (*pdcattr).ulDirty_ & DC_FONTTEXT_DIRTY != 0 {
            NtGdiFlush(); // Sync up pdcattr from Kernel space.
            (*pdcattr).ulDirty_ &= !(DC_MODE_DIRTY | DC_FONTTEXT_DIRTY);
        }
    }

    (*pdcattr).cBreak = nBreakCount;
    (*pdcattr).lBreakExtra = nBreakExtra;
    TRUE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetStringBitmapA(
    hdc: HDC,
    psz: LPSTR,
    bDoCall: BOOL,
    cj: UINT,
    lpSB: *mut BYTE,
) -> UINT {
    let Status: NTSTATUS;
    let mut pwsz: PWSTR = core::ptr::null_mut();
    let mut uResult: UINT = 0;

    if bDoCall == FALSE {
        return 0;
    }

    Status = HEAP_strdupA2W(&mut pwsz, psz);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
    } else {
        uResult = NtGdiGetStringBitmapW(hdc, pwsz, 1, lpSB, cj);
        HEAP_free(pwsz as PVOID);
    }

    uResult
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetStringBitmapW(
    hdc: HDC,
    pwsz: LPWSTR,
    bDoCall: BOOL,
    cj: UINT,
    lpSB: *mut BYTE,
) -> UINT {
    if bDoCall == FALSE {
        return 0;
    }

    NtGdiGetStringBitmapW(hdc, pwsz, 1, lpSB, cj)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetETM(hdc: HDC, petm: *mut EXTTEXTMETRIC) -> BOOL {
    let bResult: BOOL;

    bResult = NtGdiGetETM(hdc, petm);

    if bResult != FALSE && !petm.is_null() {
        (*petm).emKernPairs = GetKerningPairsA(hdc, 0, core::ptr::null_mut()) as WORD;
    }

    bResult
}
