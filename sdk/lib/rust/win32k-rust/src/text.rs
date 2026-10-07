use win32k_sys::win32k::*;

// Not in ntkrnl_sys::base; declared per-file, as the crate does elsewhere.
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;

// ---- FASTCALL callees: C declares these `FASTCALL` (win32ss/gdi/ntgdi/text.h).
// On i386 that is `@Name@N` decoration; on amd64/arm64 it lowers to cdecl. ----
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RealizeFontInit(hf: HFONT) -> PTEXTOBJ;
    fn TextIntGetTextExtentPoint(pdc: PDC, TextObj: PTEXTOBJ, lpwsz: PCWCH, cwc: INT, MaxExtent: ULONG, Fit: *mut INT, Dx: *mut INT, psize: PSIZE, fl: FLONG) -> BOOL;
    fn ftGdiGetTextCharsetInfo(Dc: PDC, lpSig: PFONTSIGNATURE, dwFlags: DWORD) -> DWORD;
    fn ftGdiGetRasterizerCaps(praststat: LPRASTERIZER_STATUS) -> u8;
    fn ftGdiGetTextMetricsW(hdc: HDC, ptmwi: PTMW_INTERNAL) -> BOOL;
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RealizeFontInit(hf: HFONT) -> PTEXTOBJ;
    fn TextIntGetTextExtentPoint(pdc: PDC, TextObj: PTEXTOBJ, lpwsz: PCWCH, cwc: INT, MaxExtent: ULONG, Fit: *mut INT, Dx: *mut INT, psize: PSIZE, fl: FLONG) -> BOOL;
    fn ftGdiGetTextCharsetInfo(Dc: PDC, lpSig: PFONTSIGNATURE, dwFlags: DWORD) -> DWORD;
    fn ftGdiGetRasterizerCaps(praststat: LPRASTERIZER_STATUS) -> u8;
    fn ftGdiGetTextMetricsW(hdc: HDC, ptmwi: PTMW_INTERNAL) -> BOOL;
    fn SetLastNtError(Status: NTSTATUS);
}

// ---- NTAPI / APIENTRY callees: `extern "system"` = stdcall on i386, cdecl else.
// SafeProbeForWrite is NTAPI with a SIZE_T length; GreExtTextOutW is APIENTRY (freetype.c). ----
extern "system" {
    fn SafeProbeForWrite(Address: *mut core::ffi::c_void, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
    fn GreExtTextOutW(hDC: HDC, XStart: INT, YStart: INT, fuOptions: UINT, lprc: PRECTL, String: PCWCH, Count: INT, Dx: *const INT, dwCodePage: DWORD) -> BOOL;
}

// ---- cdecl CRT + NTAPI mem-copy (match proven sibling decls) ----
// MmCopyTo/FromCaller are MACROS (win32ss/user/ntuser/mmcopy.h) wrapping the real
// cdecl fns _MmCopyTo/FromCaller(PVOID,PVOID,UINT)->NTSTATUS (misc/copy.c).
extern "C" {
    fn _MmCopyToCaller(Target: *mut core::ffi::c_void, Source: *const core::ffi::c_void, Bytes: UINT) -> NTSTATUS;
    fn _MmCopyFromCaller(Target: *mut core::ffi::c_void, Source: *const core::ffi::c_void, Bytes: UINT) -> NTSTATUS;
}

// HIWORD is a C macro, not an exported symbol — reimplement inline.
#[inline(always)]
#[allow(non_snake_case)]
unsafe fn HIWORD(value: u32) -> u32 { (value >> 16) & 0xffff }

// Case-insensitive ASCII compare of a NUL-terminated wide face name against a
// lowercase byte literal. Replaces rustconv's `_wcsicmp(face, "X\0".encode_utf16()
// .collect::<Vec<u16>>().as_ptr())` — which dangles a freed Vec and needs alloc.
#[inline]
unsafe fn face_ieq(face: *const u16, lower: &[u8]) -> bool {
    let mut i = 0usize;
    loop {
        let fc = *face.add(i);
        let nc = if i < lower.len() { lower[i] as u16 } else { 0 };
        let f = if (b'A' as u16..=b'Z' as u16).contains(&fc) { fc + 32 } else { fc };
        if f != nc { return false; }
        if fc == 0 { return true; }
        i += 1;
    }
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: () = ();

// TEXTOBJ_UnlockText is FORCEINLINE (text.h); Ex/KeReleasePushLock + KeLeaveCriticalRegion
// are themselves DDK inlines (no exported symbol) — use the ntkrnl_sys inline impls
// directly (they call the exported FASTCALL ExfReleasePushLockExclusive only when contended).
#[inline(always)] unsafe fn TEXTOBJ_UnlockText(plfnt: PTEXTOBJ) {
    let pl = plfnt as PLFONT;
    ntkrnl_sys::inline::ex::ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*pl).lock) as *mut _);
    ntkrnl_sys::inline::ke::KeLeaveCriticalRegion();
    GDIOBJ_vDereferenceObject(pl as POBJ); // = LFONT_ShareUnlockFont macro
}

// ---- dropped-preamble helpers (rustconv omits these macro/CRT/kernel shims) ----
use ntkrnl_sys::base::STATUS_SUCCESS;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag};
#[inline(always)] fn NT_SUCCESS(s: NTSTATUS) -> bool { s >= 0 }
#[inline(always)] fn min<T: Ord>(a: T, b: T) -> T { if a < b { a } else { b } }
#[inline(always)] unsafe fn RtlCopyMemory(d: *mut core::ffi::c_void, s: *const core::ffi::c_void, n: usize) { core::ptr::copy_nonoverlapping(s as *const u8, d as *mut u8, n); }
#[inline(always)] unsafe fn RtlZeroMemory(d: *mut core::ffi::c_void, n: usize) { core::ptr::write_bytes(d as *mut u8, 0, n); }
#[inline(always)] unsafe fn wcslen(s: *const u16) -> usize { let mut n = 0usize; while *s.add(n) != 0 { n += 1; } n }

#[cfg(target_arch = "x86")]
#[allow(non_snake_case)]
unsafe extern "fastcall" fn IntTMWFixUp(hDC: HDC, ptm: PTMW_INTERNAL) {
    IntTMWFixUp_impl(hDC, ptm)
}

#[cfg(not(target_arch = "x86"))]
#[allow(non_snake_case)]
unsafe extern "C" fn IntTMWFixUp(hDC: HDC, ptm: PTMW_INTERNAL) {
    IntTMWFixUp_impl(hDC, ptm)
}

#[allow(non_snake_case)]
unsafe fn IntTMWFixUp_impl(hDC: HDC, ptm: PTMW_INTERNAL) {
    let mut lf: LOGFONTW = core::mem::zeroed();
    let hCurrentFont: HFONT = NtGdiGetDCObject(hDC, GDI_OBJECT_TYPE_FONT as _) as HFONT;
    GreGetObject(
        hCurrentFont as _,
        core::mem::size_of::<LOGFONTW>() as _,
        core::ptr::addr_of_mut!(lf) as _,
    );

    let face = lf.lfFaceName.as_ptr();
    if face_ieq(face, b"courier")
        || face_ieq(face, b"fixedsys")
        || face_ieq(face, b"helv")
        || face_ieq(face, b"ms sans serif")
        || face_ieq(face, b"ms serif")
        || face_ieq(face, b"system")
        || face_ieq(face, b"terminal")
        || face_ieq(face, b"tms rmn")
    {
        (*ptm).TextMetric.tmPitchAndFamily &= !((TMPF_TRUETYPE | TMPF_VECTOR) as u8);
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GreTextOutW(
    hdc: HDC,
    nXStart: INT,
    nYStart: INT,
    lpString: PCWCH,
    cchString: INT,
) -> BOOL {
    GreExtTextOutW(
        hdc,
        nXStart,
        nYStart,
        0,
        core::ptr::null_mut(),
        lpString,
        cchString,
        core::ptr::null_mut(),
        0,
    )
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GreTextOutW(
    hdc: HDC,
    nXStart: INT,
    nYStart: INT,
    lpString: PCWCH,
    cchString: INT,
) -> BOOL {
    GreExtTextOutW(
        hdc,
        nXStart,
        nYStart,
        0,
        core::ptr::null_mut(),
        lpString,
        cchString,
        core::ptr::null_mut(),
        0,
    )
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GreGetTextExtentW(
    hDC: HDC,
    lpwsz: PCWCH,
    cwc: INT,
    psize: PSIZE,
    flOpts: UINT,
) -> BOOL {
    GreGetTextExtentW_impl(hDC, lpwsz, cwc, psize, flOpts)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GreGetTextExtentW(
    hDC: HDC,
    lpwsz: PCWCH,
    cwc: INT,
    psize: PSIZE,
    flOpts: UINT,
) -> BOOL {
    GreGetTextExtentW_impl(hDC, lpwsz, cwc, psize, flOpts)
}

unsafe fn GreGetTextExtentW_impl(
    hDC: HDC,
    lpwsz: PCWCH,
    cwc: INT,
    psize: PSIZE,
    flOpts: UINT,
) -> BOOL {
    let pdc: PDC;
    let pdcattr: PDC_ATTR;
    let Result: BOOL;
    let TextObj: PTEXTOBJ;

    if cwc == 0 {
        (*psize).cx = 0;
        (*psize).cy = 0;
        return 1;
    }

    pdc = DC_LockDc(hDC);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE as _);
        return 0;
    }

    pdcattr = (*pdc).pdcattr;

    TextObj = RealizeFontInit((*pdcattr).hlfntNew as HFONT);
    if !TextObj.is_null() {
        Result = TextIntGetTextExtentPoint(
            pdc,
            TextObj,
            lpwsz,
            cwc,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            psize,
            flOpts,
        );
        TEXTOBJ_UnlockText(TextObj);
    } else {
        Result = 0;
    }

    DC_UnlockDc(pdc);
    Result
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GreGetTextExtentExW(
    hDC: HDC,
    String: PCWCH,
    Count: ULONG,
    MaxExtent: ULONG,
    mut Fit: PULONG,
    Dx: PULONG,
    pSize: PSIZE,
    fl: FLONG,
) -> BOOL {
    GreGetTextExtentExW_impl(hDC, String, Count, MaxExtent, Fit, Dx, pSize, fl)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GreGetTextExtentExW(
    hDC: HDC,
    String: PCWCH,
    Count: ULONG,
    MaxExtent: ULONG,
    mut Fit: PULONG,
    Dx: PULONG,
    pSize: PSIZE,
    fl: FLONG,
) -> BOOL {
    GreGetTextExtentExW_impl(hDC, String, Count, MaxExtent, Fit, Dx, pSize, fl)
}

unsafe fn GreGetTextExtentExW_impl(
    hDC: HDC,
    String: PCWCH,
    Count: ULONG,
    MaxExtent: ULONG,
    mut Fit: PULONG,
    Dx: PULONG,
    pSize: PSIZE,
    fl: FLONG,
) -> BOOL {
    let pdc: PDC;
    let pdcattr: PDC_ATTR;
    let Result: BOOL;
    let TextObj: PTEXTOBJ;

    if (String.is_null() && Count != 0) || pSize.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER as _);
        return 0;
    }

    if Count == 0 {
        if !Fit.is_null() {
            Fit = core::ptr::null_mut();
        }
        return 1;
    }

    pdc = DC_LockDc(hDC);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE as _);
        return 0;
    }
    pdcattr = (*pdc).pdcattr;

    TextObj = RealizeFontInit((*pdcattr).hlfntNew as HFONT);
    if !TextObj.is_null() {
        Result = TextIntGetTextExtentPoint(
            pdc,
            TextObj,
            String,
            Count as INT,
            MaxExtent,
            Fit as PINT,
            Dx as PINT,
            pSize,
            fl,
        );
        TEXTOBJ_UnlockText(TextObj);
    } else {
        Result = 0;
    }

    DC_UnlockDc(pdc);
    Result
}
#[no_mangle]
pub unsafe extern "system" fn GreGetTextMetricsW(
    hdc: HDC,
    lptm: LPTEXTMETRICW,
) -> BOOL {
    let mut tmwi: TMW_INTERNAL = core::mem::zeroed();
    if ftGdiGetTextMetricsW(hdc, &mut tmwi) == 0 {
        return 0;
    }
    IntTMWFixUp(hdc, &mut tmwi);
    *lptm = tmwi.TextMetric;
    1
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetCharSet(hDC: HDC) -> DWORD {
    let Dc: PDC;
    let pdcattr: PDC_ATTR;
    let cscp: DWORD;

    Dc = DC_LockDc(hDC);
    if Dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    cscp = ftGdiGetTextCharsetInfo(Dc, core::ptr::null_mut(), 0);
    pdcattr = (*Dc).pdcattr;
    (*pdcattr).iCS_CP = cscp;
    (*pdcattr).ulDirty_ &= !DIRTY_CHARSET;
    DC_UnlockDc(Dc);
    cscp
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetRasterizerCaps(
    praststat: LPRASTERIZER_STATUS,
    mut cjBytes: ULONG,
) -> BOOL {

    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut rsSafe: RASTERIZER_STATUS = core::mem::zeroed();

    if !praststat.is_null() && cjBytes != 0 {
        if cjBytes as usize >= core::mem::size_of::<RASTERIZER_STATUS>() {
            cjBytes = core::mem::size_of::<RASTERIZER_STATUS>() as ULONG;
        }
        if ftGdiGetRasterizerCaps(&mut rsSafe) != 0 {
            Status = SafeProbeForWrite(
                praststat as *mut core::ffi::c_void,
                core::mem::size_of::<RASTERIZER_STATUS>() as _,
                1,
            );
            // The probe VALIDATES only; this copy is the access. A fault lands in Status,
            // which is checked just below -- one _SEH2_TRY's worth of behaviour.
            if NT_SUCCESS(Status)
                && !crate::copy::publish_to_user(
                    praststat as *mut core::ffi::c_void,
                    &rsSafe as *const _ as *const core::ffi::c_void,
                    cjBytes as usize,
                )
            {
                Status = STATUS_ACCESS_VIOLATION;
            }

            if !NT_SUCCESS(Status) {
                SetLastNtError(Status);
                return 0;
            }

            return 1;
        }
    }
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetTextCharsetInfo(
    hdc: HDC,
    lpSig: LPFONTSIGNATURE,
    dwFlags: u32,
) -> i32 {

    let Dc: PDC;
    let Ret: i32;
    let mut fsSafe: FONTSIGNATURE = core::mem::zeroed();
    let mut pfsSafe: PFONTSIGNATURE = &mut fsSafe;
    let mut Status: i32 = STATUS_SUCCESS as i32;

    Dc = DC_LockDc(hdc);
    if Dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return DEFAULT_CHARSET as i32;
    }

    if lpSig.is_null() {
        pfsSafe = core::ptr::null_mut();
    }

    Ret = HIWORD(ftGdiGetTextCharsetInfo(Dc, pfsSafe, dwFlags)) as i32;

    if !lpSig.is_null() {
        if Ret == DEFAULT_CHARSET as i32 {
            RtlZeroMemory(
                pfsSafe as *mut core::ffi::c_void,
                core::mem::size_of::<FONTSIGNATURE>() as _,
            );
        }

        Status = SafeProbeForWrite(
            lpSig as *mut core::ffi::c_void,
            core::mem::size_of::<FONTSIGNATURE>() as _,
            1,
        );
        // The probe VALIDATES only; this copy is the access. A fault lands in Status,
        // which is checked just below -- one _SEH2_TRY's worth of behaviour.
        if NT_SUCCESS(Status)
            && !crate::copy::publish_to_user(
                lpSig as *mut core::ffi::c_void,
                pfsSafe as *const core::ffi::c_void,
                core::mem::size_of::<FONTSIGNATURE>() as _,
            )
        {
            Status = STATUS_ACCESS_VIOLATION;
        }

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            return DEFAULT_CHARSET as i32;
        }
    }
    DC_UnlockDc(Dc);
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetTextExtentExW(
    hDC: HDC,
    UnsafeString: PCWCH,
    Count: u32,
    MaxExtent: u32,
    UnsafeFit: *mut u32,
    UnsafeDx: *mut u32,
    UnsafeSize: PSIZE,
    fl: FLONG,
) -> i32 {
    let dc: PDC;
    let pdcattr: PDC_ATTR;
    let String: LPWSTR;
    let mut Size: SIZE = core::mem::zeroed();
    let mut Status: i32;
    let Result: BOOL;
    let mut Fit: i32 = 0;
    let Dx: PINT;
    let TextObj: PTEXTOBJ;

    const GDITAG_TEXT: u32 = u32::from_be_bytes(*b"txtG"); // C literal 'txtG' -- win32ss/user/ntuser/tags.h:172

    if (Count as i32) < 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if Count == 0 {
        Size.cx = 0;
        Size.cy = 0;
        Status = _MmCopyToCaller(
            UnsafeSize as *mut core::ffi::c_void,
            core::ptr::addr_of!(Size) as *const core::ffi::c_void,
            core::mem::size_of::<SIZE>() as _,
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            return 0;
        }
        return 1;
    }

    String = ExAllocatePoolWithTag(
        PagedPool,
        (Count as usize * core::mem::size_of::<u16>()) as _,
        GDITAG_TEXT,
    ) as LPWSTR;
    if String.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    if !UnsafeDx.is_null() {
        Dx = ExAllocatePoolWithTag(
            PagedPool,
            (Count as usize * core::mem::size_of::<i32>()) as _,
            GDITAG_TEXT,
        ) as PINT;
        if Dx.is_null() {
            ExFreePoolWithTag(String as *mut core::ffi::c_void, GDITAG_TEXT);
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
    } else {
        Dx = core::ptr::null_mut();
    }

    Status = _MmCopyFromCaller(
        String as *mut core::ffi::c_void,
        UnsafeString as *const core::ffi::c_void,
        (Count as usize * core::mem::size_of::<u16>()) as _,
    );
    if !NT_SUCCESS(Status) {
        if !Dx.is_null() {
            ExFreePoolWithTag(Dx as *mut core::ffi::c_void, GDITAG_TEXT);
        }
        ExFreePoolWithTag(String as *mut core::ffi::c_void, GDITAG_TEXT);
        SetLastNtError(Status);
        return 0;
    }

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        if !Dx.is_null() {
            ExFreePoolWithTag(Dx as *mut core::ffi::c_void, GDITAG_TEXT);
        }
        ExFreePoolWithTag(String as *mut core::ffi::c_void, GDITAG_TEXT);
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    pdcattr = (*dc).pdcattr;
    TextObj = RealizeFontInit((*pdcattr).hlfntNew as HFONT);
    if !TextObj.is_null() {
        Result = TextIntGetTextExtentPoint(
            dc,
            TextObj,
            String,
            Count as INT,
            MaxExtent,
            if UnsafeFit.is_null() {
                core::ptr::null_mut()
            } else {
                core::ptr::addr_of_mut!(Fit)
            },
            Dx,
            core::ptr::addr_of_mut!(Size),
            fl,
        );
        TEXTOBJ_UnlockText(TextObj);
    } else {
        Result = 0;
    }
    DC_UnlockDc(dc);

    ExFreePoolWithTag(String as *mut core::ffi::c_void, GDITAG_TEXT);
    if Result == 0 {
        if !Dx.is_null() {
            ExFreePoolWithTag(Dx as *mut core::ffi::c_void, GDITAG_TEXT);
        }
        return 0;
    }

    if !UnsafeFit.is_null() {
        Status = _MmCopyToCaller(
            UnsafeFit as *mut core::ffi::c_void,
            core::ptr::addr_of!(Fit) as *const core::ffi::c_void,
            core::mem::size_of::<i32>() as _,
        );
        if !NT_SUCCESS(Status) {
            if !Dx.is_null() {
                ExFreePoolWithTag(Dx as *mut core::ffi::c_void, GDITAG_TEXT);
            }
            SetLastNtError(Status);
            return 0;
        }
    }

    if !UnsafeDx.is_null() {
        Status = _MmCopyToCaller(
            UnsafeDx as *mut core::ffi::c_void,
            Dx as *const core::ffi::c_void,
            (Count as usize * core::mem::size_of::<i32>()) as _,
        );
        if !NT_SUCCESS(Status) {
            if !Dx.is_null() {
                ExFreePoolWithTag(Dx as *mut core::ffi::c_void, GDITAG_TEXT);
            }
            SetLastNtError(Status);
            return 0;
        }
    }
    if !Dx.is_null() {
        ExFreePoolWithTag(Dx as *mut core::ffi::c_void, GDITAG_TEXT);
    }

    Status = _MmCopyToCaller(
        UnsafeSize as *mut core::ffi::c_void,
        core::ptr::addr_of!(Size) as *const core::ffi::c_void,
        core::mem::size_of::<SIZE>() as _,
    );
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return 0;
    }

    1
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetTextExtent(
    hdc: HDC,
    lpwsz: PCWCH,
    cwc: i32,
    psize: PSIZE,
    flOpts: u32,
) -> i32 {
    NtGdiGetTextExtentExW(
        hdc,
        lpwsz,
        cwc as u32,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        psize,
        flOpts,
    )
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetTextJustification(
    hDC: HDC,
    BreakExtra: INT,
    BreakCount: INT,
) -> BOOL {
    let pDc: PDC;
    let pdcattr: PDC_ATTR;

    pDc = DC_LockDc(hDC);
    if pDc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    pdcattr = (*pDc).pdcattr;

    (*pdcattr).lBreakExtra = BreakExtra;
    (*pdcattr).cBreak = BreakCount;

    DC_UnlockDc(pDc);
    1
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetTextFaceW(
    hDC: HDC,
    mut Count: INT,
    FaceName: PWSTR,
    bAliasName: BOOL,
) -> INT {
    let Dc: PDC;
    let pdcattr: PDC_ATTR;
    let hFont: HFONT;
    let TextObj: PTEXTOBJ;
    let Status: NTSTATUS;
    let fLen: SIZE_T;
    let ret: INT;

    Dc = DC_LockDc(hDC);
    if Dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    pdcattr = (*Dc).pdcattr;
    hFont = (*pdcattr).hlfntNew as HFONT;
    DC_UnlockDc(Dc);

    TextObj = RealizeFontInit(hFont);
    fLen = wcslen(core::ptr::addr_of!((*TextObj).TextFace) as *const u16) as SIZE_T + 1;

    if !FaceName.is_null() {
        Count = min(Count as SIZE_T, fLen) as INT;
        Status = _MmCopyToCaller(
            FaceName as *mut core::ffi::c_void,
            core::ptr::addr_of!((*TextObj).TextFace) as *const core::ffi::c_void,
            (Count as usize * core::mem::size_of::<u16>()) as _,
        );
        if !NT_SUCCESS(Status) {
            TEXTOBJ_UnlockText(TextObj);
            SetLastNtError(Status);
            return 0;
        }
        if Count > 0 && (Count as SIZE_T) < fLen {
            // NUL-terminate in the caller's buffer fault-tolerantly: a raw store
            // through FaceName is a kernel write to a caller-chosen address that a
            // probe cannot keep mapped, the same hazard the copy above avoids.
            let nul: u16 = 0;
            let st = _MmCopyToCaller(
                FaceName.offset((Count - 1) as isize) as *mut core::ffi::c_void,
                core::ptr::addr_of!(nul) as *const core::ffi::c_void,
                core::mem::size_of::<u16>() as _,
            );
            if !NT_SUCCESS(st) {
                TEXTOBJ_UnlockText(TextObj);
                SetLastNtError(st);
                return 0;
            }
        }
        ret = Count;
    } else {
        ret = fLen as INT;
    }

    TEXTOBJ_UnlockText(TextObj);
    ret
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn NtGdiGetTextMetricsW(
    hDC: HDC,
    pUnsafeTmwi: PTMW_INTERNAL,
    cj: ULONG,
) -> BOOL {
    let mut Tmwi: TMW_INTERNAL = core::mem::zeroed();

    if cj as usize <= core::mem::size_of::<TMW_INTERNAL>() {
        if ftGdiGetTextMetricsW(hDC, &mut Tmwi) != 0 {
            IntTMWFixUp(hDC, &mut Tmwi);
            {
                let __SehStatus: NTSTATUS = SafeProbeForWrite(pUnsafeTmwi as *mut _, cj as _, 1);
                if !NT_SUCCESS(__SehStatus) {
                    SetLastNtError(__SehStatus);
                    return 0;
                }
            }
            // Validate-only probe above; this publish is the access and takes its exit.
            if !crate::copy::publish_to_user(
                pUnsafeTmwi as *mut core::ffi::c_void,
                &mut Tmwi as *mut _ as *const core::ffi::c_void,
                cj as usize,
            ) {
                SetLastNtError(STATUS_ACCESS_VIOLATION);
                return 0;
            }

            return 1;
        }
    }
    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS win32 kernel mode subsystem
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            win32ss/gdi/ntgdi/text.c
//  * PURPOSE:         Text/Font
//  * PROGRAMMER:
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
// /*
//    This is a hack. See CORE-1091.
//
//    It is needed because ReactOS does not support raster fonts now.
//    After Raster Font support is added, then it can be removed.
//    Find the current font's logfont for testing its lf.lfFaceName.
//
//    The ftGdiGetTextMetricsW function currently in ReactOS will always return a Truetype font
//    because we cannot yet handle raster fonts. So it will return flags
//    TMPF_VECTOR and TMPF_TRUETYPE, which can cause problems in edit boxes.
//  */
//
// VOID FASTCALL
// IntTMWFixUp(
//     HDC hDC,
//     PTMW_INTERNAL ptm)
// {
//     LOGFONTW lf;
//     HFONT hCurrentFont;
//
//     hCurrentFont = NtGdiGetDCObject(hDC, GDI_OBJECT_TYPE_FONT);
//     GreGetObject(hCurrentFont, sizeof(LOGFONTW), &lf);
//
//     /* To compensate for the GetTextMetricsW call changing the PitchAndFamily
//      * to a TrueType one when we have a 'Raster' font as our input we filter
//      * out the problematic TrueType and Vector bits.
//      * Our list below checks for Raster Font Facenames. */
//     DPRINT("Font Facename is '%S'.\n", lf.lfFaceName);
//     if ((_wcsicmp(lf.lfFaceName, L"Courier") == 0) ||
//         (_wcsicmp(lf.lfFaceName, L"FixedSys") == 0) ||
//         (_wcsicmp(lf.lfFaceName, L"Helv") == 0) ||
//         (_wcsicmp(lf.lfFaceName, L"MS Sans Serif") == 0) ||
//         (_wcsicmp(lf.lfFaceName, L"MS Serif") == 0) ||
//         (_wcsicmp(lf.lfFaceName, L"System") == 0) ||
//         (_wcsicmp(lf.lfFaceName, L"Terminal") == 0) ||
//         (_wcsicmp(lf.lfFaceName, L"Tms Rmn") == 0))
//     {
//         ptm->TextMetric.tmPitchAndFamily &= ~(TMPF_TRUETYPE | TMPF_VECTOR);
//     }
// }
//
// /** Functions *****************************************************************/
//
// BOOL FASTCALL
// GreTextOutW(
//      HDC hdc,
//      INT nXStart,
//      INT nYStart,
//      PCWCH lpString,
//      INT cchString)
// {
//     return GreExtTextOutW(hdc, nXStart, nYStart, 0, NULL, lpString, cchString, NULL, 0);
// }
//
// /*
//    flOpts :
//    GetTextExtentPoint32W = 0
//    GetTextExtentPointW   = 1
//  */
// BOOL
// FASTCALL
// GreGetTextExtentW(
//      HDC hDC,
//      PCWCH lpwsz,
//      INT cwc,
//      PSIZE psize,
//      UINT flOpts)
// {
//     PDC pdc;
//     PDC_ATTR pdcattr;
//     BOOL Result;
//     PTEXTOBJ TextObj;
//
//     if (!cwc)
//     {
//         psize->cx = 0;
//         psize->cy = 0;
//         return TRUE;
//     }
//
//     pdc = DC_LockDc(hDC);
//     if (!pdc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     pdcattr = pdc->pdcattr;
//
//     TextObj = RealizeFontInit(pdcattr->hlfntNew);
//     if ( TextObj )
//     {
//         Result = TextIntGetTextExtentPoint( pdc,
//                                             TextObj,
//                                             lpwsz,
//                                             cwc,
//                                             0,
//                                             NULL,
//                                             0,
//                                             psize,
//                                             flOpts);
//         TEXTOBJ_UnlockText(TextObj);
//     }
//     else
//         Result = FALSE;
//
//     DC_UnlockDc(pdc);
//     return Result;
// }
//
// /*
//    fl :
//    GetTextExtentExPointW = 0 and everything else that uses this.
//    GetTextExtentExPointI = 1
//  */
// BOOL
// FASTCALL
// GreGetTextExtentExW(
//      HDC hDC,
//      PCWCH String,
//      ULONG Count,
//      ULONG MaxExtent,
//      PULONG Fit,
//      PULONG Dx,
//      PSIZE pSize,
//      FLONG fl)
// {
//     PDC pdc;
//     PDC_ATTR pdcattr;
//     BOOL Result;
//     PTEXTOBJ TextObj;
//
//     if ( (!String && Count ) || !pSize )
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if ( !Count )
//     {
//         if ( Fit ) Fit = 0;
//         return TRUE;
//     }
//
//     pdc = DC_LockDc(hDC);
//     if (NULL == pdc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//     pdcattr = pdc->pdcattr;
//
//     TextObj = RealizeFontInit(pdcattr->hlfntNew);
//     if ( TextObj )
//     {
//         Result = TextIntGetTextExtentPoint( pdc,
//                                             TextObj,
//                                             String,
//                                             Count,
//                                             MaxExtent,
//                                             (PINT)Fit,
//                                             (PINT)Dx,
//                                             pSize,
//                                             fl);
//         TEXTOBJ_UnlockText(TextObj);
//     }
//     else
//         Result = FALSE;
//
//     DC_UnlockDc(pdc);
//     return Result;
// }
//
// BOOL
// WINAPI
// GreGetTextMetricsW(
//       HDC hdc,
//      LPTEXTMETRICW lptm)
// {
//    TMW_INTERNAL tmwi;
//    if (!ftGdiGetTextMetricsW(hdc, &tmwi)) return FALSE;
//    IntTMWFixUp(hdc, &tmwi);
//    *lptm = tmwi.TextMetric;
//    return TRUE;
// }
//
// DWORD
// __stdcall
// NtGdiGetCharSet( HDC hDC)
// {
//     PDC Dc;
//     PDC_ATTR pdcattr;
//     DWORD cscp;
//     // If here, update everything!
//     Dc = DC_LockDc(hDC);
//     if (!Dc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return 0;
//     }
//     cscp = ftGdiGetTextCharsetInfo(Dc, NULL, 0);
//     pdcattr = Dc->pdcattr;
//     pdcattr->iCS_CP = cscp;
//     pdcattr->ulDirty_ &= ~DIRTY_CHARSET;
//     DC_UnlockDc( Dc );
//     return cscp;
// }
//
// BOOL
// __stdcall
// NtGdiGetRasterizerCaps(
//      LPRASTERIZER_STATUS praststat,
//      ULONG cjBytes)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     RASTERIZER_STATUS rsSafe;
//
//     if (praststat && cjBytes)
//     {
//         if ( cjBytes >= sizeof(RASTERIZER_STATUS) ) cjBytes = sizeof(RASTERIZER_STATUS);
//         if ( ftGdiGetRasterizerCaps(&rsSafe))
//         {
//             Status = SafeProbeForWrite(praststat, sizeof(RASTERIZER_STATUS), 1);
//             if (NT_SUCCESS(Status))
//             {
//                 RtlCopyMemory(praststat, &rsSafe, cjBytes );
//             }
//
//             if (!NT_SUCCESS(Status))
//             {
//                 SetLastNtError(Status);
//                 return FALSE;
//             }
//
//             return TRUE;
//         }
//     }
//     return FALSE;
// }
//
// INT
// __stdcall
// NtGdiGetTextCharsetInfo(
//      HDC hdc,
//      LPFONTSIGNATURE lpSig,
//      DWORD dwFlags)
// {
//     PDC Dc;
//     INT Ret;
//     FONTSIGNATURE fsSafe;
//     PFONTSIGNATURE pfsSafe = &fsSafe;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     Dc = DC_LockDc(hdc);
//     if (!Dc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return DEFAULT_CHARSET;
//     }
//
//     if (!lpSig) pfsSafe = NULL;
//
//     Ret = HIWORD(ftGdiGetTextCharsetInfo( Dc, pfsSafe, dwFlags));
//
//     if (lpSig)
//     {
//         if (Ret == DEFAULT_CHARSET)
//             RtlZeroMemory(pfsSafe, sizeof(FONTSIGNATURE));
//
//         Status = SafeProbeForWrite(lpSig, sizeof(FONTSIGNATURE), 1);
//         if (NT_SUCCESS(Status))
//         {
//             RtlCopyMemory(lpSig, pfsSafe, sizeof(FONTSIGNATURE));
//         }
//
//         if (!NT_SUCCESS(Status))
//         {
//             SetLastNtError(Status);
//             return DEFAULT_CHARSET;
//         }
//     }
//     DC_UnlockDc(Dc);
//     return Ret;
// }
//
//
// /*
//    fl :
//    GetTextExtentExPointW = 0 and everything else that uses this.
//    GetTextExtentExPointI = 1
//  */
// W32KAPI
// BOOL
// __stdcall
// NtGdiGetTextExtentExW(
//      HDC hDC,
//      PCWCH UnsafeString,
//      ULONG Count,
//      ULONG MaxExtent,
//      PULONG UnsafeFit,
//      PULONG UnsafeDx,
//      PSIZE UnsafeSize,
//      FLONG fl)
// {
//     PDC dc;
//     PDC_ATTR pdcattr;
//     LPWSTR String;
//     SIZE Size;
//     NTSTATUS Status;
//     BOOLEAN Result;
//     INT Fit;
//     PINT Dx;
//     PTEXTOBJ TextObj;
//
//     if ((LONG)Count < 0)
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* FIXME: Handle fl */
//
//     if (0 == Count)
//     {
//         Size.cx = 0;
//         Size.cy = 0;
//         Status = _MmCopyToCaller(UnsafeSize, &Size, sizeof(SIZE));
//         if (! NT_SUCCESS(Status))
//         {
//             SetLastNtError(Status);
//             return FALSE;
//         }
//         return TRUE;
//     }
//
//     String = ExAllocatePoolWithTag(PagedPool, Count * sizeof(WCHAR), GDITAG_TEXT);
//     if (NULL == String)
//     {
//         EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     if (NULL != UnsafeDx)
//     {
//         Dx = ExAllocatePoolWithTag(PagedPool, Count * sizeof(INT), GDITAG_TEXT);
//         if (NULL == Dx)
//         {
//             ExFreePoolWithTag(String, GDITAG_TEXT);
//             EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             return FALSE;
//         }
//     }
//     else
//     {
//         Dx = NULL;
//     }
//
//     Status = _MmCopyFromCaller(String, UnsafeString, Count * sizeof(WCHAR));
//     if (! NT_SUCCESS(Status))
//     {
//         if (NULL != Dx)
//         {
//             ExFreePoolWithTag(Dx, GDITAG_TEXT);
//         }
//         ExFreePoolWithTag(String, GDITAG_TEXT);
//         SetLastNtError(Status);
//         return FALSE;
//     }
//
//     dc = DC_LockDc(hDC);
//     if (NULL == dc)
//     {
//         if (NULL != Dx)
//         {
//             ExFreePoolWithTag(Dx, GDITAG_TEXT);
//         }
//         ExFreePoolWithTag(String, GDITAG_TEXT);
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//     pdcattr = dc->pdcattr;
//     TextObj = RealizeFontInit(pdcattr->hlfntNew);
//     if ( TextObj )
//     {
//         Result = TextIntGetTextExtentPoint( dc,
//                                             TextObj,
//                                             String,
//                                             Count,
//                                             MaxExtent,
//                                             NULL == UnsafeFit ? NULL : &Fit,
//                                             Dx,
//                                             &Size,
//                                             fl);
//         TEXTOBJ_UnlockText(TextObj);
//     }
//     else
//         Result = FALSE;
//     DC_UnlockDc(dc);
//
//     ExFreePoolWithTag(String, GDITAG_TEXT);
//     if (! Result)
//     {
//         if (NULL != Dx)
//         {
//             ExFreePoolWithTag(Dx, GDITAG_TEXT);
//         }
//         return FALSE;
//     }
//
//     if (NULL != UnsafeFit)
//     {
//         Status = _MmCopyToCaller(UnsafeFit, &Fit, sizeof(INT));
//         if (! NT_SUCCESS(Status))
//         {
//             if (NULL != Dx)
//             {
//                 ExFreePoolWithTag(Dx, GDITAG_TEXT);
//             }
//             SetLastNtError(Status);
//             return FALSE;
//         }
//     }
//
//     if (NULL != UnsafeDx)
//     {
//         Status = _MmCopyToCaller(UnsafeDx, Dx, Count * sizeof(INT));
//         if (! NT_SUCCESS(Status))
//         {
//             if (NULL != Dx)
//             {
//                 ExFreePoolWithTag(Dx, GDITAG_TEXT);
//             }
//             SetLastNtError(Status);
//             return FALSE;
//         }
//     }
//     if (NULL != Dx)
//     {
//         ExFreePoolWithTag(Dx, GDITAG_TEXT);
//     }
//
//     Status = _MmCopyToCaller(UnsafeSize, &Size, sizeof(SIZE));
//     if (! NT_SUCCESS(Status))
//     {
//         SetLastNtError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//    flOpts :
//    GetTextExtentPoint32W = 0
//    GetTextExtentPointW   = 1
//  */
// BOOL
// __stdcall
// NtGdiGetTextExtent(
//      HDC hdc,
//      PCWCH lpwsz,
//      INT cwc,
//      PSIZE psize,
//      UINT flOpts)
// {
//     return NtGdiGetTextExtentExW(hdc, lpwsz, cwc, 0, NULL, NULL, psize, flOpts);
// }
//
// BOOL
// __stdcall
// NtGdiSetTextJustification(
//      HDC hDC,
//      INT BreakExtra,
//      INT BreakCount)
// {
//     PDC pDc;
//     PDC_ATTR pdcattr;
//
//     pDc = DC_LockDc(hDC);
//     if (!pDc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     pdcattr = pDc->pdcattr;
//
//     pdcattr->lBreakExtra = BreakExtra;
//     pdcattr->cBreak = BreakCount;
//
//     DC_UnlockDc(pDc);
//     return TRUE;
// }
//
//
// W32KAPI
// INT
// __stdcall
// NtGdiGetTextFaceW(
//      HDC hDC,
//      INT Count,
//      PWSTR FaceName,
//      BOOL bAliasName)
// {
//     PDC Dc;
//     PDC_ATTR pdcattr;
//     HFONT hFont;
//     PTEXTOBJ TextObj;
//     NTSTATUS Status;
//     SIZE_T fLen;
//     INT ret;
//
//     /* FIXME: Handle bAliasName */
//
//     Dc = DC_LockDc(hDC);
//     if (Dc == NULL)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//     pdcattr = Dc->pdcattr;
//     hFont = pdcattr->hlfntNew;
//     DC_UnlockDc(Dc);
//
//     TextObj = RealizeFontInit(hFont);
//     ASSERT(TextObj != NULL);
//     fLen = wcslen(TextObj->TextFace) + 1;
//
//     if (FaceName != NULL)
//     {
//         Count = min(Count, fLen);
//         Status = _MmCopyToCaller(FaceName, TextObj->TextFace, Count * sizeof(WCHAR));
//         if (!NT_SUCCESS(Status))
//         {
//             TEXTOBJ_UnlockText(TextObj);
//             SetLastNtError(Status);
//             return 0;
//         }
//         /* Terminate if we copied only part of the font name */
//         if (Count > 0 && Count < fLen)
//         {
//             FaceName[Count - 1] = '\0';
//         }
//         ret = Count;
//     }
//     else
//     {
//         ret = fLen;
//     }
//
//     TEXTOBJ_UnlockText(TextObj);
//     return ret;
// }
//
// W32KAPI
// BOOL
// __stdcall
// NtGdiGetTextMetricsW(
//      HDC hDC,
//      PTMW_INTERNAL pUnsafeTmwi,
//      ULONG cj)
// {
//     TMW_INTERNAL Tmwi;
//
//     if ( cj <= sizeof(TMW_INTERNAL) )
//     {
//         if (ftGdiGetTextMetricsW(hDC, &Tmwi))
//         {
//             IntTMWFixUp(hDC, &Tmwi);
//             {
//                 NTSTATUS __SehStatus = SafeProbeForWrite(pUnsafeTmwi, cj, 1);
//                 if (!NT_SUCCESS(__SehStatus))
//                 {
//                     SetLastNtError(__SehStatus);
//                     return FALSE;
//                 }
//             }
//             RtlCopyMemory(pUnsafeTmwi, &Tmwi, cj);
//
//             return TRUE;
//         }
//     }
//     return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
