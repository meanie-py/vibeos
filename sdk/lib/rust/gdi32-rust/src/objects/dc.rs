//! objects/dc.c → Rust. DC lifecycle & device-caps client API.
//!
//! IntCreateDICW / CreateDC{A,W} / CreateIC{A,W} / CreateCompatibleDC / DeleteDC,
//! SaveDC / RestoreDC / CancelDC / GdiConvertAndCheckDC / GdiCleanCacheDC / GdiReleaseDC,
//! GetDeviceCaps, Get/Set{ArcDirection,RelAbs,BoundsRect}, Get/SetDC{Brush,Pen}Color,
//! Get/SetBk{Color,Mode}, Get/SetROP2, Get/SetPolyFillMode, Get/SetGraphicsMode,
//! Get/SetStretchBltMode, GetCurrentObject, EnumObjects, GetClipBox, GetAspectRatioFilterEx,
//! ResetDC{A,W}, SelectPalette, GetHFONT, GdiSelect{Bitmap,Brush,Pen,Font}, SelectObject.
//
// NEEDS-TYPE: none — all scalar aliases + GDI types come from `win32k_sys::win32k`
//   (BOOL/INT/UINT/DWORD/ULONG/USHORT/BYTE/LONG/ULONG_PTR/COLORREF/NTSTATUS/HANDLE/HDC/
//    HGDIOBJ/HBRUSH/HPEN/HFONT/HBITMAP/HPALETTE/HCOLORSPACE/PDEVCAPS/PLDC/PDC_ATTR/
//    PGDIBSOBJECT/UNICODE_STRING/ANSI_STRING/STRING/DEVMODEA/DEVMODEW/LPDEVMODEA/LPDEVMODEW/
//    GOBJENUMPROC/LPARAM/LPRECT/LPSIZE/LPPOINT/WCHAR/...).
// NEEDS-IMPORT: cross-file gdi32 client fns flagged at the extern blocks below —
//   misc.rs WINAPI exports (GdiConvertToDevmodeW/GdiFixUpHandle/GdiValidateHandle), the
//   gdi32-private FASTCALL helpers (GdiGetLDC/LoadTheSpoolerDrv/HEAP_strdupA2W), the
//   plain-cdecl GdiGetHandleUserData / GdiSAPCallback / METADC_GetDeviceCaps /
//   METADC_*/EMFDC_* recorders, the WINAPI METADC_RosGlueDeleteDC / METADC_SetD / EMFDC_SetD,
//   sibling WINAPI exports ExtSelectClipRgn/SetColorSpace/StartPage, user32 NtUserSelectPalette,
//   kernel32 SetLastError/Heap{Alloc,Free}/GetProcessHeap, and ntdll Rtl* (via import libs).
//   `GdiDevCaps` (dllmain.c) + `ghSpooler` (misc) are data symbols → `static mut` externs.
// NEEDS-CHECK: `_wcsnicmp`/`wcslen` are CRT exports — declared `extern "C"`; confirm they
//   resolve from the gdi32 link set (ntdll/crt) on all three arches.

use win32k_sys::win32k::*;
use crate::gdihelp::*;

// win32k_sys's FALSE/TRUE are bindgen u32 (#define), but BOOL is i32 — shadow with
// BOOL-typed locals so `== FALSE`/`return TRUE` type-check against the i32 BOOL returns.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// ── handle-type constants (ntgdihdl.h / GDILoObjType), as `usize` for compare ──
const LO_DC_TYPE: usize = GDILoObjType_GDILoObjType_LO_DC_TYPE as usize;
const LO_METADC16_TYPE: usize = GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize;
const LO_REGION_TYPE: usize = GDILoObjType_GDILoObjType_LO_REGION_TYPE as usize;
const LO_BITMAP_TYPE: usize = GDILoObjType_GDILoObjType_LO_BITMAP_TYPE as usize;
const LO_DIBSECTION_TYPE: usize = GDILoObjType_GDILoObjType_LO_DIBSECTION_TYPE as usize;
const LO_BRUSH_TYPE: usize = GDILoObjType_GDILoObjType_LO_BRUSH_TYPE as usize;
const LO_PEN_TYPE: usize = GDILoObjType_GDILoObjType_LO_PEN_TYPE as usize;
const LO_EXTPEN_TYPE: usize = GDILoObjType_GDILoObjType_LO_EXTPEN_TYPE as usize;
const LO_FONT_TYPE: usize = GDILoObjType_GDILoObjType_LO_FONT_TYPE as usize;
const LO_ICMLCS_TYPE: usize = GDILoObjType_GDILoObjType_LO_ICMLCS_TYPE as usize;
const LO_PALETTE_TYPE: usize = GDILoObjType_GDILoObjType_LO_PALETTE_TYPE as usize;

// ── Cross-file gdi32 / kernel32 / ntdll fns (WINAPI / NTAPI → stdcall on i386) ──
extern "system" {
    fn SetLastError(dwErrCode: DWORD);
    fn GdiConvertToDevmodeW(dmA: LPDEVMODEA) -> LPDEVMODEW; // misc.rs (gdi32.spec stdcall)
    fn GdiFixUpHandle(hGO: HGDIOBJ) -> HGDIOBJ; // misc.rs (WINAPI)
    fn GdiValidateHandle(hGdiObj: HGDIOBJ) -> BOOL; // misc.rs (WINAPI)
    fn ExtSelectClipRgn(hdc: HDC, hrgn: HGDIOBJ, iMode: INT) -> INT; // region.rs (WINAPI)
    fn SetColorSpace(hDC: HDC, hCS: HCOLORSPACE) -> HCOLORSPACE; // icm.rs (WINAPI)
    fn StartPage(hdc: HDC) -> INT; // print.rs (WINAPI)
    fn NtUserSelectPalette(hDC: HDC, hpal: HPALETTE, ForceBackground: BOOL) -> HPALETTE; // user32
    fn METADC_RosGlueDeleteDC(hdc: HDC) -> BOOL; // mfdc glue (WINAPI)
    fn METADC_SetD(hdc: HDC, dwIn: DWORD, usMF16Id: USHORT) -> BOOL; // (WINAPI)
    fn EMFDC_SetD(pldc: PLDC, dwIn: DWORD, ulMFId: ULONG) -> BOOL; // (WINAPI)
    // kernel32
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn GetProcessHeap() -> HANDLE;
    // ntdll NTAPI
    fn RtlInitUnicodeString(DestinationString: PUNICODE_STRING, SourceString: PCWSTR);
    fn RtlInitAnsiString(DestinationString: *mut STRING, SourceString: PCSTR);
    fn RtlAnsiStringToUnicodeString(
        DestinationString: PUNICODE_STRING,
        SourceString: *const STRING,
        AllocateDestinationString: BOOLEAN,
    ) -> NTSTATUS;
    fn RtlFreeUnicodeString(UnicodeString: PUNICODE_STRING);
    fn RtlNtStatusToDosError(Status: NTSTATUS) -> ULONG;
    fn RtlFreeHeap(HeapHandle: HANDLE, Flags: ULONG, P: PVOID) -> BOOLEAN;
}

// ── Plain cdecl gdi32-private glue (DECLSPEC_HIDDEN, no WINAPI) → extern "C" ────
extern "C" {
    fn GdiGetHandleUserData(hGdiObj: HGDIOBJ, ObjectType: DWORD, UserData: *mut PVOID) -> BOOL;
    fn GdiSAPCallback(pldc: PLDC);
    fn METADC_GetDeviceCaps(hdc: HDC, cap: INT) -> INT;
    fn METADC_SaveDC(hdc: HDC) -> BOOL;
    fn METADC_RestoreDC(hdc: HDC, level: INT) -> BOOL;
    fn METADC_SetBkColor(hdc: HDC, color: COLORREF) -> BOOL;
    fn METADC_SetBkMode(hdc: HDC, mode: INT) -> BOOL;
    fn METADC_SetROP2(hdc: HDC, rop: INT) -> BOOL;
    fn METADC_SetPolyFillMode(hdc: HDC, mode: INT) -> BOOL;
    fn METADC_SetStretchBltMode(hdc: HDC, mode: INT) -> BOOL;
    fn METADC_SelectObject(hdc: HDC, obj: HGDIOBJ) -> HGDIOBJ;
    fn METADC_SelectPalette(hdc: HDC, palette: HPALETTE) -> BOOL;
    fn EMFDC_SaveDC(dc_attr: PLDC) -> BOOL;
    fn EMFDC_RestoreDC(dc_attr: PLDC, level: INT) -> BOOL;
    fn EMFDC_SetBkColor(dc_attr: PLDC, color: COLORREF) -> BOOL;
    fn EMFDC_SetBkMode(dc_attr: PLDC, mode: INT) -> BOOL;
    fn EMFDC_SetROP2(dc_attr: PLDC, rop: INT) -> BOOL;
    fn EMFDC_SetPolyFillMode(dc_attr: PLDC, mode: INT) -> BOOL;
    fn EMFDC_SetStretchBltMode(dc_attr: PLDC, mode: INT) -> BOOL;
    fn EMFDC_SetDCBrushColor(dc_attr: PLDC, color: COLORREF) -> BOOL;
    fn EMFDC_SetDCPenColor(dc_attr: PLDC, color: COLORREF) -> BOOL;
    fn EMFDC_SelectObject(dc_attr: PLDC, obj: HGDIOBJ) -> BOOL;
    fn EMFDC_SelectPalette(dc_attr: PLDC, palette: HPALETTE) -> BOOL;
    // CRT (msvcrt/ntdll)
    fn wcslen(s: *const WCHAR) -> usize;
    fn _wcsnicmp(s1: *const WCHAR, s2: *const WCHAR, n: usize) -> INT;
}

// ── FASTCALL gdi32-private helpers ─────────────────────────────────────────────
extern "fastcall" {
    fn GdiGetLDC(hdc: HDC) -> PLDC; // misc.rs
    fn LoadTheSpoolerDrv() -> BOOL; // misc.rs
    fn HEAP_strdupA2W(ppszW: *mut PWSTR, lpszA: PCSTR) -> NTSTATUS; // heap.rs
}

// ── Data symbols ───────────────────────────────────────────────────────────────
extern "C" {
    static mut GdiDevCaps: PDEVCAPS; // dllmain.c (primary display caps pointer)
    static mut ghSpooler: HANDLE; // misc.c (spooler module handle)
}

/// `RtlGetProcessHeap()` — FORCEINLINE `NtCurrentPeb()->ProcessHeap` (rtlfuncs.h),
/// over the win32k_sys _TEB/_PEB (NtCurrentTeb lives in crate::gdihelp).
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

/// `UlongToHandle(ul)` (basetsd.h) — `(HANDLE)(ULONG_PTR)(ULONG)ul` (zero-extend).
#[inline]
fn UlongToHandle(ul: ULONG) -> HANDLE {
    ul as ULONG_PTR as usize as HANDLE
}

// L"\\.\DISPLAY" (11 wide chars + NUL) and L"DISPLAY" (7 wide chars + NUL).
static WSZ_DISPLAY_DEVICE: [WCHAR; 12] = [
    b'\\' as WCHAR,
    b'\\' as WCHAR,
    b'.' as WCHAR,
    b'\\' as WCHAR,
    b'D' as WCHAR,
    b'I' as WCHAR,
    b'S' as WCHAR,
    b'P' as WCHAR,
    b'L' as WCHAR,
    b'A' as WCHAR,
    b'Y' as WCHAR,
    0,
];
static WSZ_DISPLAY: [WCHAR; 8] = [
    b'D' as WCHAR,
    b'I' as WCHAR,
    b'S' as WCHAR,
    b'P' as WCHAR,
    b'L' as WCHAR,
    b'A' as WCHAR,
    b'Y' as WCHAR,
    0,
];

#[no_mangle]
pub unsafe extern "fastcall" fn IntCreateDICW(
    lpwszDriver: LPCWSTR,
    lpwszDevice: LPCWSTR,
    lpwszOutput: LPCWSTR,
    lpInitData: PDEVMODEW,
    iType: ULONG,
) -> HDC {
    let mut Device: UNICODE_STRING = core::mem::zeroed();
    let mut Output: UNICODE_STRING = core::mem::zeroed();
    let hdc: HDC;
    let mut Display: BOOL = FALSE;
    let mut Default: BOOL = FALSE;
    let mut UMdhpdev: HANDLE = core::ptr::null_mut();

    let hspool: HANDLE = core::ptr::null_mut();

    if lpwszDevice.is_null() && lpwszDriver.is_null() {
        Default = TRUE; // Ask Win32k to set Default device.
        Display = TRUE; // Most likely to be DISPLAY.
    } else {
        if !lpwszDevice.is_null() && wcslen(lpwszDevice) != 0 {
            // First
            if _wcsnicmp(lpwszDevice, WSZ_DISPLAY_DEVICE.as_ptr(), 11) == 0 {
                Display = TRUE;
            }
            RtlInitUnicodeString(&mut Device, lpwszDevice);
        } else if !lpwszDriver.is_null() {
            // Second
            if (_wcsnicmp(lpwszDriver, WSZ_DISPLAY.as_ptr(), 7) == 0)
                || (_wcsnicmp(lpwszDriver, WSZ_DISPLAY_DEVICE.as_ptr(), 11) == 0)
            {
                Display = TRUE;
            }
            RtlInitUnicodeString(&mut Device, lpwszDriver);
        }
    }

    if !lpwszOutput.is_null() {
        RtlInitUnicodeString(&mut Output, lpwszOutput);
    }

    // Handle Print device or something else.
    if Display == 0 {
        /* Only load the spooler for non-display DCs.
         * On LiveCD, LoadLibraryW("WINSPOOL.DRV") inside semLocal CS
         * blocks on CD I/O and triggers deadlock detection. */
        if ghSpooler.is_null() && LoadTheSpoolerDrv() == 0 {
            // DPRINT1("WinSpooler.Drv Did not load!\n");
        }
        // WIP - GDI Print Commit coming in soon.
        // DPRINT1("Not a DISPLAY device! %wZ\n", &Device);
        return core::ptr::null_mut(); // Return NULL until then.....
    }

    hdc = NtGdiOpenDCW(
        if Default != 0 {
            core::ptr::null_mut()
        } else {
            &mut Device as PUNICODE_STRING
        },
        lpInitData as *mut DEVMODEW,
        if !lpwszOutput.is_null() {
            &mut Output as PUNICODE_STRING
        } else {
            core::ptr::null_mut()
        },
        iType, // DCW 0 and ICW 1.
        Display,
        hspool,
        &mut UMdhpdev as *mut HANDLE as PVOID,
    );

    // #if 0 block (LDC allocation for non-DC handles) is disabled in C.
    hdc
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateCompatibleDC(hdc: HDC) -> HDC {
    // PDC_ATTR pdcattr;
    let hdcNew: HDC = NtGdiCreateCompatibleDC(hdc);
    // #if 0 ICM-compatible-DC block disabled in C.
    hdcNew
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateDCA(
    lpszDriver: LPCSTR,
    lpszDevice: LPCSTR,
    lpszOutput: LPCSTR,
    lpdvmInit: *const DEVMODEA,
) -> HDC {
    let mut DriverA: ANSI_STRING = core::mem::zeroed();
    let mut DeviceA: ANSI_STRING = core::mem::zeroed();
    let mut OutputA: ANSI_STRING = core::mem::zeroed();
    let mut DriverU: UNICODE_STRING = core::mem::zeroed();
    let mut DeviceU: UNICODE_STRING = core::mem::zeroed();
    let mut OutputU: UNICODE_STRING = core::mem::zeroed();
    let mut dvmInitW: LPDEVMODEW = core::ptr::null_mut();
    let hdc: HDC;

    /*
     * If needed, convert to Unicode
     * any string parameter.
     */

    if !lpszDriver.is_null() {
        RtlInitAnsiString(&mut DriverA, lpszDriver as LPSTR as PCSTR);
        RtlAnsiStringToUnicodeString(&mut DriverU, &DriverA, 1);
    } else {
        DriverU.Buffer = core::ptr::null_mut();
    }

    if !lpszDevice.is_null() {
        RtlInitAnsiString(&mut DeviceA, lpszDevice as LPSTR as PCSTR);
        RtlAnsiStringToUnicodeString(&mut DeviceU, &DeviceA, 1);
    } else {
        DeviceU.Buffer = core::ptr::null_mut();
    }

    if !lpszOutput.is_null() {
        RtlInitAnsiString(&mut OutputA, lpszOutput as LPSTR as PCSTR);
        RtlAnsiStringToUnicodeString(&mut OutputU, &OutputA, 1);
    } else {
        OutputU.Buffer = core::ptr::null_mut();
    }

    if !lpdvmInit.is_null() {
        dvmInitW = GdiConvertToDevmodeW(lpdvmInit as LPDEVMODEA);
    }

    hdc = IntCreateDICW(
        DriverU.Buffer,
        DeviceU.Buffer,
        OutputU.Buffer,
        if !lpdvmInit.is_null() {
            dvmInitW as PDEVMODEW
        } else {
            core::ptr::null_mut()
        },
        0,
    );
    HEAP_free(dvmInitW as PVOID);

    /* Free Unicode parameters. */
    RtlFreeUnicodeString(&mut DriverU);
    RtlFreeUnicodeString(&mut DeviceU);
    RtlFreeUnicodeString(&mut OutputU);

    /* Return the DC handle. */
    hdc
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateDCW(
    lpwszDriver: LPCWSTR,
    lpwszDevice: LPCWSTR,
    lpwszOutput: LPCWSTR,
    lpInitData: *const DEVMODEW,
) -> HDC {
    IntCreateDICW(
        lpwszDriver,
        lpwszDevice,
        lpwszOutput,
        lpInitData as PDEVMODEW,
        0,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateICW(
    lpszDriver: LPCWSTR,
    lpszDevice: LPCWSTR,
    lpszOutput: LPCWSTR,
    lpdvmInit: *const DEVMODEW,
) -> HDC {
    IntCreateDICW(
        lpszDriver,
        lpszDevice,
        lpszOutput,
        lpdvmInit as PDEVMODEW,
        1,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateICA(
    lpszDriver: LPCSTR,
    lpszDevice: LPCSTR,
    lpszOutput: LPCSTR,
    lpdvmInit: *const DEVMODEA,
) -> HDC {
    let mut Status: NTSTATUS;
    let mut lpszDriverW: LPWSTR = core::ptr::null_mut();
    let mut lpszDeviceW: LPWSTR = core::ptr::null_mut();
    let mut lpszOutputW: LPWSTR = core::ptr::null_mut();
    let mut dvmInitW: LPDEVMODEW = core::ptr::null_mut();
    let mut hdc: HDC = core::ptr::null_mut();

    Status = HEAP_strdupA2W(&mut lpszDriverW, lpszDriver);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
    } else {
        Status = HEAP_strdupA2W(&mut lpszDeviceW, lpszDevice);
        if !NT_SUCCESS(Status) {
            SetLastError(RtlNtStatusToDosError(Status));
        } else {
            Status = HEAP_strdupA2W(&mut lpszOutputW, lpszOutput);
            if !NT_SUCCESS(Status) {
                SetLastError(RtlNtStatusToDosError(Status));
            } else {
                if !lpdvmInit.is_null() {
                    dvmInitW = GdiConvertToDevmodeW(lpdvmInit as LPDEVMODEA);
                }

                hdc = IntCreateDICW(
                    lpszDriverW,
                    lpszDeviceW,
                    lpszOutputW,
                    if !lpdvmInit.is_null() {
                        dvmInitW as PDEVMODEW
                    } else {
                        core::ptr::null_mut()
                    },
                    1,
                );
                HEAP_free(dvmInitW as PVOID);
                HEAP_free(lpszOutputW as PVOID);
            }
            HEAP_free(lpszDeviceW as PVOID);
        }
        HEAP_free(lpszDriverW as PVOID);
    }

    hdc
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn DeleteDC(hdc: HDC) -> BOOL {
    let hType: usize = GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ);

    if hType != LO_DC_TYPE {
        return METADC_RosGlueDeleteDC(hdc);
    }

    //if ( ghICM || pdcattr->pvLIcm )
    //    IcmDeleteLocalDC( hdc, pdcattr, NULL );

    NtGdiDeleteObjectApp(hdc as HANDLE)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SaveDC(hdc: HDC) -> INT {
    // HANDLE_METADC1P(INT, SaveDC, 0, hdc);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_SaveDC(hdc) as INT;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: INT = 0;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_SaveDC(pLDC) == 0 {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }
    NtGdiSaveDC(hdc)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn RestoreDC(hdc: HDC, iLevel: INT) -> BOOL {
    // HANDLE_METADC(BOOL, RestoreDC, FALSE, hdc, iLevel);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_RestoreDC(hdc, iLevel);
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_RestoreDC(pLDC, iLevel) == 0 {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }
    NtGdiRestoreDC(hdc, iLevel)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CancelDC(hDC: HDC) -> BOOL {
    let mut pDc_Attr: PDC_ATTR = core::ptr::null_mut();

    if GDI_HANDLE_GET_TYPE(hDC as HGDIOBJ) != GDI_OBJECT_TYPE_DC as usize
        && GDI_HANDLE_GET_TYPE(hDC as HGDIOBJ) != GDI_OBJECT_TYPE_METADC as usize
    {
        let pLDC: PLDC = GdiGetLDC(hDC);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return FALSE;
        }
        /* If a document has started set it to die. */
        if ((*pLDC).Flags & LDC_INIT_DOCUMENT) != 0 {
            (*pLDC).Flags |= LDC_KILL_DOCUMENT;
        }

        return NtGdiCancelDC(hDC);
    }

    if GdiGetHandleUserData(
        hDC as HGDIOBJ,
        GDI_OBJECT_TYPE_DC,
        &mut pDc_Attr as *mut PDC_ATTR as *mut PVOID,
    ) != 0
    {
        (*pDc_Attr).ulDirty_ &= !(DC_PLAYMETAFILE as ULONG);
        return TRUE;
    }

    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn GetArcDirection(hdc: HDC) -> INT {
    GetDCDWord(hdc, _GETDCDWORD_GdiGetArcDirection as UINT, 0) as INT
}

#[no_mangle]
pub unsafe extern "system" fn SetArcDirection(hdc: HDC, nDirection: INT) -> INT {
    GetAndSetDCDWord(
        hdc,
        _GETSETDCDWORD_GdiGetSetArcDirection as UINT,
        nDirection as DWORD,
        EMR_SETARCDIRECTION as ULONG,
        0,
        0,
    ) as INT
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiReleaseDC(_hdc: HDC) -> BOOL {
    0
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiCleanCacheDC(hdc: HDC) -> BOOL {
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_DC_TYPE {
        return TRUE;
    }
    SetLastError(ERROR_INVALID_HANDLE);
    FALSE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiConvertAndCheckDC(hdc: HDC) -> HDC {
    let hType: usize = GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ);
    if hType == LO_DC_TYPE || hType == LO_METADC16_TYPE {
        return hdc;
    }
    let pldc: PLDC = GdiGetLDC(hdc);
    if !pldc.is_null() {
        if ((*pldc).Flags & LDC_SAPCALLBACK) != 0 {
            GdiSAPCallback(pldc);
        }
        if ((*pldc).Flags & LDC_KILL_DOCUMENT) != 0 {
            return core::ptr::null_mut();
        }
        if ((*pldc).Flags & LDC_STARTPAGE) != 0 {
            StartPage(hdc);
        }
        return hdc;
    }
    SetLastError(ERROR_INVALID_HANDLE);
    core::ptr::null_mut()
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetCurrentObject(hdc: HDC, uObjectType: UINT) -> HGDIOBJ {
    let mut pdcattr: PDC_ATTR = core::ptr::null_mut();
    let mut uObjectType = uObjectType;

    /* Check if this is a user mode object */
    if (uObjectType == OBJ_PEN)
        || (uObjectType == OBJ_EXTPEN)
        || (uObjectType == OBJ_BRUSH)
        || (uObjectType == OBJ_COLORSPACE)
    {
        /* Get the DC attribute */
        pdcattr = GdiGetDcAttr(hdc);
        if pdcattr.is_null() {
            return core::ptr::null_mut();
        }
    }

    /* Check what object was requested */
    match uObjectType {
        OBJ_EXTPEN | OBJ_PEN => return (*pdcattr).hpen as HGDIOBJ,
        OBJ_BRUSH => return (*pdcattr).hbrush as HGDIOBJ,
        OBJ_COLORSPACE => return (*pdcattr).hColorSpace as HGDIOBJ,
        OBJ_PAL => uObjectType = GDI_OBJECT_TYPE_PALETTE,
        OBJ_FONT => uObjectType = GDI_OBJECT_TYPE_FONT,
        OBJ_BITMAP => uObjectType = GDI_OBJECT_TYPE_BITMAP,
        /* All others are invalid */
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER);
            return core::ptr::null_mut();
        }
    }

    /* Pass the request to win32k */
    NtGdiGetDCObject(hdc, uObjectType as INT) as HGDIOBJ
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn EnumObjects(
    hdc: HDC,
    nObjectType: INT,
    lpObjectFunc: GOBJENUMPROC,
    lParam: LPARAM,
) -> INT {
    let ObjectsCount: ULONG;
    let Size: ULONG;
    let mut Buffer: PVOID;
    let EndOfBuffer: usize;
    let mut Result: INT = 0;

    match nObjectType as u32 {
        OBJ_BRUSH => Size = core::mem::size_of::<LOGBRUSH>() as ULONG,
        OBJ_PEN => Size = core::mem::size_of::<LOGPEN>() as ULONG,
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
    }

    ObjectsCount = NtGdiEnumObjects(hdc, nObjectType, 0, core::ptr::null_mut());
    if ObjectsCount == 0 {
        return 0;
    }

    Buffer = HeapAlloc(GetProcessHeap(), 0, (ObjectsCount * Size) as SIZE_T) as PVOID;
    if Buffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    if NtGdiEnumObjects(hdc, nObjectType, ObjectsCount * Size, Buffer) == 0 {
        HeapFree(GetProcessHeap(), 0, Buffer as LPVOID);
        return 0;
    }

    EndOfBuffer = (Buffer as usize) + (ObjectsCount * Size) as usize;
    while (Buffer as usize) < EndOfBuffer {
        Result = lpObjectFunc.unwrap()(Buffer as LPVOID, lParam);
        if Result == 0 {
            break;
        }
        Buffer = ((Buffer as usize) + Size as usize) as PVOID;
    }

    HeapFree(GetProcessHeap(), 0, Buffer as LPVOID);
    Result
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetDeviceCaps(hdc: HDC, nIndex: INT) -> INT {
    let pdcattr: PDC_ATTR;
    let pldc: PLDC;
    let hType: usize = GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ);
    let mut pDevCaps: PDEVCAPS = GdiDevCaps; // Primary display device capabilities.
    // DPRINT("Device CAPS1\n");

    // HANDLE_METADC16(INT, GetDeviceCaps, 0, hdc, nIndex);
    if hType != LO_DC_TYPE {
        if hType == LO_METADC16_TYPE {
            return METADC_GetDeviceCaps(hdc, nIndex);
        }
    }

    if hType != LO_DC_TYPE && hType != LO_METADC16_TYPE {
        pldc = GdiGetLDC(hdc);
        if pldc.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return 0;
        }
        if ((*pldc).Flags & LDC_DEVCAPS) == 0 {
            if NtGdiGetDeviceCapsAll(hdc, &mut (*pldc).DevCaps) == 0 {
                SetLastError(ERROR_INVALID_PARAMETER);
            }
            (*pldc).Flags |= LDC_DEVCAPS;
        }
        pDevCaps = &mut (*pldc).DevCaps;
    } else {
        /* Get the DC attribute */
        pdcattr = GdiGetDcAttr(hdc);
        if pdcattr.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }

        if ((*pdcattr).ulDirty_ & DC_PRIMARY_DISPLAY) == 0 {
            return NtGdiGetDeviceCaps(hdc, nIndex);
        }
    }

    match nIndex as u32 {
        DRIVERVERSION => (*pDevCaps).ulVersion as INT,
        TECHNOLOGY => (*pDevCaps).ulTechnology as INT,
        HORZSIZE => (*pDevCaps).ulHorzSize as INT,
        VERTSIZE => (*pDevCaps).ulVertSize as INT,
        HORZRES => (*pDevCaps).ulHorzRes as INT,
        VERTRES => (*pDevCaps).ulVertRes as INT,
        LOGPIXELSX => (*pDevCaps).ulLogPixelsX as INT,
        LOGPIXELSY => (*pDevCaps).ulLogPixelsY as INT,
        BITSPIXEL => (*pDevCaps).ulBitsPixel as INT,
        PLANES => (*pDevCaps).ulPlanes as INT,
        NUMBRUSHES => -1,
        NUMPENS => (*pDevCaps).ulNumPens as INT,
        NUMFONTS => (*pDevCaps).ulNumFonts as INT,
        NUMCOLORS => (*pDevCaps).ulNumColors as INT,
        ASPECTX => (*pDevCaps).ulAspectX as INT,
        ASPECTY => (*pDevCaps).ulAspectY as INT,
        ASPECTXY => (*pDevCaps).ulAspectXY as INT,
        CLIPCAPS => CP_RECTANGLE as INT,
        SIZEPALETTE => (*pDevCaps).ulSizePalette as INT,
        NUMRESERVED => 20,
        COLORRES => (*pDevCaps).ulColorRes as INT,
        DESKTOPVERTRES => (*pDevCaps).ulVertRes as INT,
        DESKTOPHORZRES => (*pDevCaps).ulHorzRes as INT,
        BLTALIGNMENT => (*pDevCaps).ulBltAlignment as INT,
        SHADEBLENDCAPS => (*pDevCaps).ulShadeBlend as INT,
        COLORMGMTCAPS => (*pDevCaps).ulColorMgmtCaps as INT,
        PHYSICALWIDTH => (*pDevCaps).ulPhysicalWidth as INT,
        PHYSICALHEIGHT => (*pDevCaps).ulPhysicalHeight as INT,
        PHYSICALOFFSETX => (*pDevCaps).ulPhysicalOffsetX as INT,
        PHYSICALOFFSETY => (*pDevCaps).ulPhysicalOffsetY as INT,
        VREFRESH => (*pDevCaps).ulVRefresh as INT,
        RASTERCAPS => (*pDevCaps).ulRasterCaps as INT,
        CURVECAPS => (CC_CIRCLES
            | CC_PIE
            | CC_CHORD
            | CC_ELLIPSES
            | CC_WIDE
            | CC_STYLED
            | CC_WIDESTYLED
            | CC_INTERIORS
            | CC_ROUNDRECT) as INT,
        LINECAPS => (LC_POLYLINE
            | LC_MARKER
            | LC_POLYMARKER
            | LC_WIDE
            | LC_STYLED
            | LC_WIDESTYLED
            | LC_INTERIORS) as INT,
        POLYGONALCAPS => (PC_POLYGON
            | PC_RECTANGLE
            | PC_WINDPOLYGON
            | PC_SCANLINE
            | PC_WIDE
            | PC_STYLED
            | PC_WIDESTYLED
            | PC_INTERIORS) as INT,
        TEXTCAPS => (*pDevCaps).ulTextCaps as INT,
        // PDEVICESIZE / SCALINGFACTORX / SCALINGFACTORY / default
        _ => 0,
    }
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetRelAbs(hdc: HDC, _dwIgnore: DWORD) -> DWORD {
    GetDCDWord(hdc, _GETDCDWORD_GdiGetRelAbs as UINT, 0)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetRelAbs(hdc: HDC, Mode: INT) -> INT {
    GetAndSetDCDWord(
        hdc,
        _GETSETDCDWORD_GdiGetSetRelAbs as UINT,
        Mode as DWORD,
        0,
        META_SETRELABS as USHORT,
        0,
    ) as INT
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetAndSetDCDWord(
    hdc: HDC,
    u: UINT,
    dwIn: DWORD,
    ulMFId: ULONG,
    usMF16Id: USHORT,
    dwError: DWORD,
) -> DWORD {
    let mut dwResult: DWORD = 0;
    let pldc: PLDC;

    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE && ulMFId != EMR_MAX + 1 {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_SetD(hdc, dwIn, usMF16Id) as DWORD;
        }
        pldc = GdiGetLDC(hdc);
        if (*pldc).iType == LDC_EMFLDC as INT && EMFDC_SetD(pldc, dwIn, ulMFId) == 0 {
            return 0;
        }
    }

    /* Call win32k to do the real work */
    if NtGdiGetAndSetDCDword(hdc, u, dwIn, &mut dwResult) == 0 {
        return dwError;
    }

    dwResult
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetDCDWord(hdc: HDC, u: UINT, dwError: DWORD) -> DWORD {
    let mut dwResult: DWORD = 0;

    if NtGdiGetDCDword(hdc, u, &mut dwResult) == 0 {
        return dwError;
    }

    dwResult
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetAspectRatioFilterEx(hdc: HDC, lpAspectRatio: LPSIZE) -> BOOL {
    NtGdiGetDCPoint(
        hdc,
        _GETDCPOINT_GdiGetAspectRatioFilter as UINT,
        lpAspectRatio as PPOINTL,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetBoundsRect(hdc: HDC, lprcBounds: LPRECT, flags: UINT) -> UINT {
    NtGdiGetBoundsRect(hdc, lprcBounds as LPRECTL, flags & !DCB_WINDOWMGR) as UINT
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetBoundsRect(hdc: HDC, prc: *const RECT, flags: UINT) -> UINT {
    /* FIXME add check for validate the flags */
    NtGdiSetBoundsRect(hdc, prc as LPRECTL, flags & !DCB_WINDOWMGR) as UINT
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetClipBox(hdc: HDC, lprc: LPRECT) -> INT {
    NtGdiGetAppClipBox(hdc, lprc as LPRECTL)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetDCBrushColor(hdc: HDC) -> COLORREF {
    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return CLR_INVALID;
    }

    (*pdcattr).ulBrushClr
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetDCPenColor(hdc: HDC) -> COLORREF {
    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return CLR_INVALID;
    }

    (*pdcattr).ulPenClr
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetDCBrushColor(hdc: HDC, crColor: COLORREF) -> COLORREF {
    let crOldColor: COLORREF;

    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return CLR_INVALID;
    }

    /* We handle only enhanced meta DCs here */
    // HANDLE_EMETAFDC(COLORREF, SetDCBrushColor, CLR_INVALID, hdc, crColor);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        let _Ret: COLORREF = CLR_INVALID;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return _Ret;
        }
        let pLDC: PLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _Ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_SetDCBrushColor(pLDC, crColor) == 0 {
            return _Ret;
        }
    }

    /* Get old color and store the new */
    crOldColor = (*pdcattr).ulBrushClr;
    (*pdcattr).ulBrushClr = crColor;

    if (*pdcattr).crBrushClr != crColor {
        (*pdcattr).ulDirty_ |= DIRTY_FILL;
        (*pdcattr).crBrushClr = crColor;
    }

    crOldColor
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetDCPenColor(hdc: HDC, crColor: COLORREF) -> COLORREF {
    let crOldColor: COLORREF;

    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return CLR_INVALID;
    }

    /* We handle only enhanced meta DCs here */
    // HANDLE_EMETAFDC(COLORREF, SetDCPenColor, CLR_INVALID, hdc, crColor);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        let _Ret: COLORREF = CLR_INVALID;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return _Ret;
        }
        let pLDC: PLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _Ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_SetDCPenColor(pLDC, crColor) == 0 {
            return _Ret;
        }
    }

    /* Get old color and store the new */
    crOldColor = (*pdcattr).ulPenClr;
    (*pdcattr).ulPenClr = crColor as ULONG;

    if (*pdcattr).crPenClr != crColor {
        (*pdcattr).ulDirty_ |= DIRTY_LINE;
        (*pdcattr).crPenClr = crColor;
    }

    crOldColor
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetBkColor(hdc: HDC) -> COLORREF {
    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Don't set LastError here! */
        return CLR_INVALID;
    }

    (*pdcattr).ulBackgroundClr
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetBkColor(hdc: HDC, crColor: COLORREF) -> COLORREF {
    let crOldColor: COLORREF;

    // HANDLE_METADC(COLORREF, SetBkColor, CLR_INVALID, hdc, crColor);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_SetBkColor(hdc, crColor) as COLORREF;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: COLORREF = CLR_INVALID;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_SetBkColor(pLDC, crColor) == 0 {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return CLR_INVALID;
    }

    /* Get old color and store the new */
    crOldColor = (*pdcattr).ulBackgroundClr;
    (*pdcattr).ulBackgroundClr = crColor;

    if (*pdcattr).crBackgroundClr != crColor {
        (*pdcattr).ulDirty_ |= DIRTY_BACKGROUND | DIRTY_LINE | DIRTY_FILL;
        (*pdcattr).crBackgroundClr = crColor;
    }

    crOldColor
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetBkMode(hdc: HDC) -> INT {
    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Don't set LastError here! */
        return 0;
    }

    (*pdcattr).lBkMode
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetBkMode(hdc: HDC, iBkMode: INT) -> INT {
    let iOldMode: INT;

    /* Avoid bad mode setting */
    if (iBkMode != TRANSPARENT as INT) && (iBkMode != OPAQUE as INT) {
        // DPRINT1("SetBkMode: Incorrect value\n");
        return 0;
    }

    // HANDLE_METADC(INT, SetBkMode, 0, hdc, iBkMode);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_SetBkMode(hdc, iBkMode) as INT;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: INT = 0;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_SetBkMode(pLDC, iBkMode) == 0 {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    iOldMode = (*pdcattr).lBkMode;
    (*pdcattr).jBkMode = iBkMode as BYTE; // Processed
    (*pdcattr).lBkMode = iBkMode; // Raw

    iOldMode
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetROP2(hdc: HDC) -> INT {
    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Do not set LastError here! */
        return 0;
    }

    (*pdcattr).jROP2 as INT
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetROP2(hdc: HDC, rop2: INT) -> INT {
    let rop2Old: INT;

    // HANDLE_METADC(INT, SetROP2, 0, hdc, rop2);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_SetROP2(hdc, rop2) as INT;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: INT = 0;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_SetROP2(pLDC, rop2) == 0 {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if (*NtCurrentTeb()).GdiTebBatch.HDC == hdc as HANDLE {
        if ((*pdcattr).ulDirty_ & DC_MODE_DIRTY) != 0 {
            NtGdiFlush();
            (*pdcattr).ulDirty_ &= !DC_MODE_DIRTY;
        }
    }

    rop2Old = (*pdcattr).jROP2 as INT;
    (*pdcattr).jROP2 = rop2 as BYTE;

    rop2Old
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetPolyFillMode(hdc: HDC) -> INT {
    /* Get DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Don't set LastError here! */
        return 0;
    }

    /* Return current fill mode */
    (*pdcattr).lFillMode
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetPolyFillMode(hdc: HDC, iPolyFillMode: INT) -> INT {
    let iOldPolyFillMode: INT;

    // HANDLE_METADC(INT, SetPolyFillMode, 0, hdc, iPolyFillMode);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_SetPolyFillMode(hdc, iPolyFillMode) as INT;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: INT = 0;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_SetPolyFillMode(pLDC, iPolyFillMode) == 0
            {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if (*NtCurrentTeb()).GdiTebBatch.HDC == hdc as HANDLE {
        if ((*pdcattr).ulDirty_ & DC_MODE_DIRTY) != 0 {
            NtGdiFlush(); // Sync up pdcattr from Kernel space.
            (*pdcattr).ulDirty_ &= !DC_MODE_DIRTY;
        }
    }

    iOldPolyFillMode = (*pdcattr).lFillMode;
    (*pdcattr).lFillMode = iPolyFillMode;

    iOldPolyFillMode
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetGraphicsMode(hdc: HDC) -> INT {
    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Don't set LastError here! */
        return 0;
    }

    /* Return current graphics mode */
    (*pdcattr).iGraphicsMode
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetGraphicsMode(hdc: HDC, iMode: INT) -> INT {
    let iOldMode: INT;

    /* Check parameters */
    if (iMode < GM_COMPATIBLE as INT) || (iMode > GM_ADVANCED as INT) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    /* Check for trivial case */
    if iMode == (*pdcattr).iGraphicsMode {
        return iMode;
    }

    if (*NtCurrentTeb()).GdiTebBatch.HDC == hdc as HANDLE {
        if ((*pdcattr).ulDirty_ & DC_MODE_DIRTY) != 0 {
            NtGdiFlush(); // Sync up pdcattr from Kernel space.
            (*pdcattr).ulDirty_ &= !DC_MODE_DIRTY;
        }
    }

    /* One would think that setting the graphics mode to GM_COMPATIBLE
     * would also reset the world transformation matrix to the unity
     * matrix. However, in Windows, this is not the case. This doesn't
     * make a lot of sense to me, but that's the way it is.
     */
    iOldMode = (*pdcattr).iGraphicsMode;
    (*pdcattr).iGraphicsMode = iMode;

    iOldMode
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn ResetDCW(hdc: HDC, lpInitData: *const DEVMODEW) -> HDC {
    NtGdiResetDC(
        hdc,
        lpInitData as LPDEVMODEW,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    hdc
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn ResetDCA(hdc: HDC, lpInitData: *const DEVMODEA) -> HDC {
    let InitDataW: LPDEVMODEW = GdiConvertToDevmodeW(lpInitData as LPDEVMODEA);

    NtGdiResetDC(
        hdc,
        InitDataW,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    HEAP_free(InitDataW as PVOID);
    hdc
}

#[no_mangle]
pub unsafe extern "system" fn SelectPalette(
    hdc: HDC,
    hpal: HPALETTE,
    bForceBackground: BOOL,
) -> HPALETTE {
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return (METADC_SelectPalette(hdc, hpal) as ULONG_PTR) as usize as HPALETTE;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return core::ptr::null_mut();
            }
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_SelectPalette(pLDC, hpal) == 0 {
                return core::ptr::null_mut();
            }
        }
    }
    NtUserSelectPalette(hdc, hpal, bForceBackground)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetStretchBltMode(hdc: HDC) -> INT {
    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Don't set LastError here! */
        return 0;
    }

    (*pdcattr).lStretchBltMode
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetStretchBltMode(hdc: HDC, iStretchMode: INT) -> INT {
    let iOldMode: INT;

    // HANDLE_METADC(INT, SetStretchBltMode, 0, hdc, iStretchMode);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_SetStretchBltMode(hdc, iStretchMode) as INT;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: INT = 0;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_SetStretchBltMode(pLDC, iStretchMode) == 0
            {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    iOldMode = (*pdcattr).lStretchBltMode;
    (*pdcattr).lStretchBltMode = iStretchMode;

    // Wine returns an error here. We set the default.
    let mut iStretchMode = iStretchMode;
    if (iStretchMode <= 0) || (iStretchMode > MAXSTRETCHBLTMODE as INT) {
        iStretchMode = WHITEONBLACK as INT;
    }

    (*pdcattr).jStretchBltMode = iStretchMode as BYTE;

    iOldMode
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetHFONT(hdc: HDC) -> HFONT {
    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Don't set LastError here! */
        return core::ptr::null_mut();
    }

    /* Return the current font */
    (*pdcattr).hlfntNew as HFONT
}

#[no_mangle]
pub unsafe extern "system" fn GdiSelectBitmap(hdc: HDC, hbmp: HBITMAP) -> HBITMAP {
    NtGdiSelectBitmap(hdc, hbmp)
}

#[no_mangle]
pub unsafe extern "system" fn GdiSelectBrush(hdc: HDC, hbr: HBRUSH) -> HBRUSH {
    let hbrOld: HBRUSH;

    // HANDLE_METADC(HBRUSH, SelectObject, NULL, hdc, hbr);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_SelectObject(hdc, hbr as HGDIOBJ) as HBRUSH;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: HBRUSH = core::ptr::null_mut();
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_SelectObject(pLDC, hbr as HGDIOBJ) == 0 {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    /* Get the current brush. If it matches the new brush, we're done */
    hbrOld = (*pdcattr).hbrush as HBRUSH;
    if hbrOld == hbr {
        return hbrOld;
    }

    /* Set the new brush and update dirty flags */
    (*pdcattr).hbrush = hbr as HANDLE;
    (*pdcattr).ulDirty_ |= DC_BRUSH_DIRTY;
    hbrOld
}

#[no_mangle]
pub unsafe extern "system" fn GdiSelectPen(hdc: HDC, hpen: HPEN) -> HPEN {
    let hpenOld: HPEN;

    // HANDLE_METADC(HPEN, SelectObject, NULL, hdc, hpen);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_SelectObject(hdc, hpen as HGDIOBJ) as HPEN;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: HPEN = core::ptr::null_mut();
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_SelectObject(pLDC, hpen as HGDIOBJ) == 0 {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return core::ptr::null_mut();
    }

    /* Get the current pen. If it matches the new pen, we're done */
    hpenOld = (*pdcattr).hpen as HPEN;
    if hpenOld == hpen {
        return hpenOld;
    }

    /* Set the new pen and update dirty flags */
    (*pdcattr).ulDirty_ |= DC_PEN_DIRTY;
    (*pdcattr).hpen = hpen as HANDLE;
    hpenOld
}

#[no_mangle]
pub unsafe extern "system" fn GdiSelectFont(hdc: HDC, hfont: HFONT) -> HFONT {
    let hfontOld: HFONT;

    // HANDLE_METADC(HFONT, SelectObject, NULL, hdc, hfont);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_SelectObject(hdc, hfont as HGDIOBJ) as HFONT;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: HFONT = core::ptr::null_mut();
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_SelectObject(pLDC, hfont as HGDIOBJ) == 0
            {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    /* Get the current font. If it matches the new font, we're done */
    hfontOld = (*pdcattr).hlfntNew as HFONT;
    if hfontOld == hfont {
        return hfontOld;
    }

    /* Set the new font and update dirty flags */
    (*pdcattr).hlfntNew = hfont as HANDLE;
    (*pdcattr).ulDirty_ &= !SLOW_WIDTHS;
    (*pdcattr).ulDirty_ |= DIRTY_CHARSET;

    /* If the DC does not have a DIB section selected, try a batch command */
    if ((*pdcattr).ulDirty_ & DC_DIBSECTION) == 0 {
        let pgO: PGDIBSOBJECT = GdiAllocBatchCommand(hdc, crate::gdihelp::GdiBCSelObj) as PGDIBSOBJECT;
        if !pgO.is_null() {
            (*pgO).hgdiobj = hfont as HGDIOBJ;
            return hfontOld;
        }
    }

    /* We could not use the batch command, call win32k */
    NtGdiSelectFont(hdc, hfont) as HFONT
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SelectObject(hdc: HDC, hobj: HGDIOBJ) -> HGDIOBJ {
    /* Fix up 16 bit handles */
    let hobj: HGDIOBJ = GdiFixUpHandle(hobj);
    if GdiValidateHandle(hobj) == 0 {
        return core::ptr::null_mut();
    }

    /* Call the appropriate select function */
    match GDI_HANDLE_GET_TYPE(hobj) {
        LO_REGION_TYPE => {
            UlongToHandle(ExtSelectClipRgn(hdc, hobj, RGN_COPY as INT) as ULONG) as HGDIOBJ
        }
        LO_BITMAP_TYPE | LO_DIBSECTION_TYPE => GdiSelectBitmap(hdc, hobj as HBITMAP) as HGDIOBJ,
        LO_BRUSH_TYPE => GdiSelectBrush(hdc, hobj as HBRUSH) as HGDIOBJ,
        LO_PEN_TYPE | LO_EXTPEN_TYPE => GdiSelectPen(hdc, hobj as HPEN) as HGDIOBJ,
        LO_FONT_TYPE => GdiSelectFont(hdc, hobj as HFONT) as HGDIOBJ,
        LO_ICMLCS_TYPE => SetColorSpace(hdc, hobj as HCOLORSPACE) as HGDIOBJ,
        LO_PALETTE_TYPE => {
            SetLastError(ERROR_INVALID_FUNCTION);
            core::ptr::null_mut()
        }
        _ => core::ptr::null_mut(),
    }
}
