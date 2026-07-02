//! objects/printdrv.c → Rust. Print/spooler client API.
//!
//! StartDoc{A,W}/StartPage/EndPage/EndDoc/AbortDoc/Start|EndFormPage/SetAbortProc plus
//! the GdiXxxEMF / GdiGetSpoolFileHandle spool-file stubs, and the winspool.drv lazy
//! loader `LoadTheSpoolerDrv` with its spooler function-pointer table. The Xxx`Ex`
//! FASTCALL thunks (AbortPrinterEx/DocumentEventEx/…) forward into that table.
//
// NEEDS-TYPE: `LPPRINTER_DEFAULTSW` (winspool) and `EMFPLAYPROC` (winddi) are not in
//   win32k_sys bindgen → aliased to PVOID locally (only ever stored / passed opaquely:
//   the OPENPRINTERW/RESETPRINTERW fp's are stored-never-called, and GdiPlayEMF is an
//   unimplemented stub). `DOC_INFO_1W` (winspool) also absent → local #[repr(C)] struct.
// NEEDS-TYPE: `DOCUMENTEVENT_*` event ids (winddiui.h) + SP_ERROR/SP_APPABORT (wingdi.h)
//   are not in bindgen → defined as local consts with their header values.
// NEEDS-IMPORT: kernel32 LoadLibraryW/GetProcAddress/FreeLibrary/LocalFree/SetLastError/
//   GetTickCount/Heap{Alloc,Free}/GetProcessHeap/MultiByteToWideChar (WINAPI), ntdll
//   RtlEnter/LeaveCriticalSection (NTAPI) → `extern "system"`; GdiGetLDC (FASTCALL,
//   misc.rs) → `extern "fastcall"`; GdiSAPCallback (cdecl, gdiobj.c) → `extern "C"`;
//   NtGdi* from win32k_sys glob. Data symbols: `semLocal` (dllmain.rs RTL_CRITICAL_SECTION).
// NEEDS-CHECK: `ghSpooler` + the non-static fp's `fpClosePrinter`/`fpOpenPrinterW` are
//   exported (`#[no_mangle] pub static mut`); dc.rs externs `ghSpooler`/`LoadTheSpoolerDrv`.

use win32k_sys::win32k::*;
use crate::gdihelp::*;

// win32k_sys FALSE/TRUE are bindgen u32 (#define); BOOL is i32 — shadow with BOOL-typed
// locals so `== FALSE` / `return TRUE` type-check against the i32 BOOL returns.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// ── wingdi.h status codes ──────────────────────────────────────────────────────
const SP_ERROR: INT = -1;
const SP_APPABORT: INT = -2;

// ── winddiui.h DOCUMENTEVENT_* ids ─────────────────────────────────────────────
const DOCUMENTEVENT_STARTDOC: INT = 5;
const DOCUMENTEVENT_STARTPAGE: INT = 6;
const DOCUMENTEVENT_ENDPAGE: INT = 7;
const DOCUMENTEVENT_ENDDOC: INT = 8;
const DOCUMENTEVENT_ABORTDOC: INT = 9;
const DOCUMENTEVENT_ENDDOCPOST: INT = 12;
const DOCUMENTEVENT_STARTDOCPOST: INT = 13;

// CP_ACP (winnls.h) — not in bindgen.
const CP_ACP: UINT = 0;

// GDILoObjType handle-type tags, as usize (GDI_HANDLE_GET_TYPE returns usize).
const LO_DC_TYPE: usize = GDILoObjType_GDILoObjType_LO_DC_TYPE as usize;
const LO_METADC16_TYPE: usize = GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize;

// ── types absent from win32k_sys bindgen ───────────────────────────────────────
type LPPRINTER_DEFAULTSW = PVOID; // winspool _PRINTER_DEFAULTSW* — opaque here
type EMFPLAYPROC = PVOID; // winddi EMFPLAYPROC — opaque (GdiPlayEMF is a stub)

/// winspool `DOC_INFO_1W` (not in bindgen).
#[repr(C)]
struct DOC_INFO_1W {
    pDocName: LPWSTR,
    pOutputFile: LPWSTR,
    pDatatype: LPWSTR,
}

// ── winspool / winsplp / winddiui function-pointer typedefs (all WINAPI) ────────
type ABORTPRINTER = Option<unsafe extern "system" fn(HANDLE) -> BOOL>;
type CLOSEPRINTER = Option<unsafe extern "system" fn(HANDLE) -> BOOL>;
type CLOSESPOOLFILEHANDLE = Option<unsafe extern "system" fn(HANDLE, HANDLE) -> BOOL>;
type COMMITSPOOLDATA = Option<unsafe extern "system" fn(HANDLE, HANDLE, DWORD) -> HANDLE>;
type DOCUMENTPROPERTIESW =
    Option<unsafe extern "system" fn(HWND, HANDLE, LPWSTR, PDEVMODEW, PDEVMODEW, DWORD) -> LONG>;
type ENDDOCPRINTER = Option<unsafe extern "system" fn(HANDLE) -> BOOL>;
type ENDPAGEPRINTER = Option<unsafe extern "system" fn(HANDLE) -> BOOL>;
type GETPRINTERW = Option<unsafe extern "system" fn(HANDLE, DWORD, LPBYTE, DWORD, LPDWORD) -> BOOL>;
type GETPRINTERDRIVERW =
    Option<unsafe extern "system" fn(HANDLE, LPWSTR, DWORD, LPBYTE, DWORD, LPDWORD) -> BOOL>;
type GETSPOOLFILEHANDLE = Option<unsafe extern "system" fn(HANDLE) -> HANDLE>;
type ISVALIDDEVMODEW = Option<unsafe extern "system" fn(PDEVMODEW, usize) -> BOOL>;
type OPENPRINTERW =
    Option<unsafe extern "system" fn(LPWSTR, PHANDLE, LPPRINTER_DEFAULTSW) -> BOOL>;
type READPRINTER = Option<unsafe extern "system" fn(HANDLE, PVOID, DWORD, PDWORD) -> BOOL>;
type RESETPRINTERW = Option<unsafe extern "system" fn(HANDLE, LPPRINTER_DEFAULTSW) -> BOOL>;
type STARTDOCDLGW = Option<unsafe extern "system" fn(HANDLE, *mut DOCINFOW) -> LPWSTR>;
type STARTDOCPRINTERW = Option<unsafe extern "system" fn(HANDLE, DWORD, PBYTE) -> DWORD>;
type STARTPAGEPRINTER = Option<unsafe extern "system" fn(HANDLE) -> BOOL>;
type SEEKPRINTER =
    Option<unsafe extern "system" fn(HANDLE, LARGE_INTEGER, PLARGE_INTEGER, DWORD, BOOL) -> BOOL>;
type SPLREADPRINTER = Option<unsafe extern "system" fn(HANDLE, *mut LPBYTE, DWORD) -> BOOL>;
type SPLDRIVERUNLOADCOMPLETE = Option<unsafe extern "system" fn(LPWSTR) -> BOOL>;
type DOCUMENTEVENT =
    Option<unsafe extern "system" fn(HANDLE, HDC, INT, ULONG, PVOID, ULONG, PVOID) -> INT>;
type QUERYCOLORPROFILE =
    Option<unsafe extern "system" fn(HANDLE, PDEVMODEW, ULONG, PVOID, ULONG, FLONG) -> BOOL>;
type QUERYSPOOLMODE = Option<unsafe extern "system" fn(HANDLE, DWORD, DWORD) -> DWORD>;
type QUERYREMOTEFONTS = Option<unsafe extern "system" fn(DWORD, DWORD, DWORD) -> DWORD>;

/* GLOBALS *******************************************************************/
#[no_mangle]
pub static mut ghSpooler: HANDLE = core::ptr::null_mut();

static mut fpAbortPrinter: ABORTPRINTER = None;
#[no_mangle]
pub static mut fpClosePrinter: CLOSEPRINTER = None;
static mut fpCloseSpoolFileHandle: CLOSESPOOLFILEHANDLE = None;
static mut fpCommitSpoolData: COMMITSPOOLDATA = None;
//static fpConnectToLd64In32Server;
static mut fpDocumentEvent: DOCUMENTEVENT = None;
static mut fpDocumentPropertiesW: DOCUMENTPROPERTIESW = None;
static mut fpEndDocPrinter: ENDDOCPRINTER = None;
static mut fpEndPagePrinter: ENDPAGEPRINTER = None;
static mut fpGetSpoolFileHandle: GETSPOOLFILEHANDLE = None;
static mut fpGetPrinterW: GETPRINTERW = None;
static mut fpGetPrinterDriverW: GETPRINTERDRIVERW = None;
static mut fpIsValidDevmodeW: ISVALIDDEVMODEW = None;
#[no_mangle]
pub static mut fpOpenPrinterW: OPENPRINTERW = None;
static mut fpQuerySpoolMode: QUERYSPOOLMODE = None;
static mut fpQueryRemoteFonts: QUERYREMOTEFONTS = None;
static mut fpQueryColorProfile: QUERYCOLORPROFILE = None;
static mut fpReadPrinter: READPRINTER = None;
static mut fpResetPrinterW: RESETPRINTERW = None;
static mut fpSeekPrinter: SEEKPRINTER = None;
static mut fpSplDriverUnloadComplete: SPLDRIVERUNLOADCOMPLETE = None;
static mut fpSplReadPrinter: SPLREADPRINTER = None;
static mut fpStartDocDlgW: STARTDOCDLGW = None;
static mut fpStartDocPrinterW: STARTDOCPRINTERW = None;
static mut fpStartPagePrinter: STARTPAGEPRINTER = None;

// ── cross-file imports ─────────────────────────────────────────────────────────
extern "system" {
    // kernel32 (WINAPI)
    fn SetLastError(dwErrCode: DWORD);
    fn GetTickCount() -> DWORD;
    fn LoadLibraryW(lpLibFileName: LPCWSTR) -> HMODULE;
    fn GetProcAddress(hModule: HMODULE, lpProcName: PCSTR) -> FARPROC;
    fn FreeLibrary(hLibModule: HMODULE) -> BOOL;
    fn LocalFree(hMem: HLOCAL) -> HLOCAL;
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn GetProcessHeap() -> HANDLE;
    fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: INT,
        lpWideCharStr: LPWSTR,
        cchWideChar: INT,
    ) -> INT;
    // ntdll (NTAPI)
    fn RtlEnterCriticalSection(CriticalSection: PRTL_CRITICAL_SECTION) -> NTSTATUS;
    fn RtlLeaveCriticalSection(CriticalSection: PRTL_CRITICAL_SECTION) -> NTSTATUS;
}

// gdiobj.c plain-cdecl glue (no WINAPI).
extern "C" {
    fn GdiSAPCallback(pldc: PLDC);
    static mut semLocal: RTL_CRITICAL_SECTION; // dllmain.rs
}

// misc.rs FASTCALL helper.
extern "fastcall" {
    fn GdiGetLDC(hdc: HDC) -> PLDC;
}

/// `((PW32CLIENTINFO)NtCurrentTeb()->Win32ClientInfo)->cSpins = 0;`
#[inline]
unsafe fn clear_cspins() {
    let ci = core::ptr::addr_of_mut!((*NtCurrentTeb()).Win32ClientInfo) as PW32CLIENTINFO;
    (*ci).cSpins = 0;
}

/* PRIVATE FUNCTIONS *********************************************************/

unsafe extern "fastcall" fn IntEndPage(hdc: HDC, Form: BOOL) -> INT {
    let pldc: PLDC;
    let mut Ret: INT = SP_ERROR;
    let hType: usize = GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ);

    if hType == LO_DC_TYPE || hType == LO_METADC16_TYPE {
        SetLastError(ERROR_INVALID_HANDLE);
        return SP_ERROR;
    }

    pldc = GdiGetLDC(hdc);
    if pldc.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return SP_ERROR;
    }

    if (*pldc).Flags & LDC_ATENDPAGE != 0 {
        return 1;
    }

    if (*pldc).Flags & LDC_META_PRINT != 0 {
        if Form != FALSE {
            // Do MF EndPageForm
        } else {
            // Do MF EndPage
        }
        return Ret;
    }

    if (*pldc).Flags & LDC_KILL_DOCUMENT != 0 || (*pldc).Flags & LDC_INIT_PAGE != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return SP_ERROR;
    }

    if (*pldc).Flags & LDC_SAPCALLBACK != 0 {
        GdiSAPCallback(pldc);
    }

    (*pldc).Flags &= !LDC_INIT_PAGE;

    DocumentEventEx(
        core::ptr::null_mut(),
        (*pldc).hPrinter,
        hdc,
        DOCUMENTEVENT_ENDPAGE,
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    );

    clear_cspins();

    if NtGdiEndPage(hdc) != FALSE {
        let Good: BOOL;
        //      if (pldc->pUMPDev)
        Good = EndPagePrinterEx(core::ptr::null_mut(), (*pldc).hPrinter);

        if Good != FALSE {
            (*pldc).Flags |= LDC_STARTPAGE;
        }
        Ret = 1;
    } else {
        SetLastError(ERROR_INVALID_PARAMETER);
    }
    Ret
}

/* FUNCTIONS *****************************************************************/

#[no_mangle]
pub unsafe extern "fastcall" fn AbortPrinterEx(_pvUMPDev: PVOID, hPrinter: HANDLE) -> BOOL {
    (fpAbortPrinter.unwrap())(hPrinter)
}

#[no_mangle]
pub unsafe extern "fastcall" fn DocumentEventEx(
    _pvUMPDev: PVOID,
    hPrinter: HANDLE,
    hdc: HDC,
    iEsc: INT,
    cbIn: ULONG,
    pvIn: PVOID,
    cbOut: ULONG,
    pvOut: PVOID,
) -> INT {
    (fpDocumentEvent.unwrap())(hPrinter, hdc, iEsc, cbIn, pvIn, cbOut, pvOut)
}

#[no_mangle]
pub unsafe extern "fastcall" fn EndDocPrinterEx(_pvUMPDev: PVOID, hPrinter: HANDLE) -> BOOL {
    (fpEndDocPrinter.unwrap())(hPrinter)
}

#[no_mangle]
pub unsafe extern "fastcall" fn EndPagePrinterEx(_pvUMPDev: PVOID, hPrinter: HANDLE) -> BOOL {
    (fpEndPagePrinter.unwrap())(hPrinter)
}

#[no_mangle]
pub unsafe extern "fastcall" fn LoadTheSpoolerDrv() -> BOOL {
    let mut hModWinSpoolDrv: HMODULE;

    if ghSpooler.is_null() {
        let _ = RtlEnterCriticalSection(core::ptr::addr_of_mut!(semLocal));

        // L"WINSPOOL.DRV"
        let winspool: [WCHAR; 13] = [
            b'W' as WCHAR, b'I' as WCHAR, b'N' as WCHAR, b'S' as WCHAR, b'P' as WCHAR,
            b'O' as WCHAR, b'O' as WCHAR, b'L' as WCHAR, b'.' as WCHAR, b'D' as WCHAR,
            b'R' as WCHAR, b'V' as WCHAR, 0,
        ];
        hModWinSpoolDrv = LoadLibraryW(winspool.as_ptr());

        if !hModWinSpoolDrv.is_null() {
            fpAbortPrinter =
                core::mem::transmute(GetProcAddress(hModWinSpoolDrv, b"AbortPrinter\0".as_ptr() as PCSTR));
            fpClosePrinter =
                core::mem::transmute(GetProcAddress(hModWinSpoolDrv, b"ClosePrinter\0".as_ptr() as PCSTR));
            fpCloseSpoolFileHandle = core::mem::transmute(GetProcAddress(
                hModWinSpoolDrv,
                b"CloseSpoolFileHandle\0".as_ptr() as PCSTR,
            ));
            fpCommitSpoolData = core::mem::transmute(GetProcAddress(
                hModWinSpoolDrv,
                b"CommitSpoolData\0".as_ptr() as PCSTR,
            ));
            // fpConnectToLd64In32Server = (PVOID)GetProcAddress(hModWinSpoolDrv, (LPCSTR)224);
            fpDocumentEvent =
                core::mem::transmute(GetProcAddress(hModWinSpoolDrv, b"DocumentEvent\0".as_ptr() as PCSTR));
            fpDocumentPropertiesW = core::mem::transmute(GetProcAddress(
                hModWinSpoolDrv,
                b"DocumentPropertiesW\0".as_ptr() as PCSTR,
            ));
            fpEndDocPrinter =
                core::mem::transmute(GetProcAddress(hModWinSpoolDrv, b"EndDocPrinter\0".as_ptr() as PCSTR));
            fpEndPagePrinter = core::mem::transmute(GetProcAddress(
                hModWinSpoolDrv,
                b"EndPagePrinter\0".as_ptr() as PCSTR,
            ));
            fpGetPrinterW =
                core::mem::transmute(GetProcAddress(hModWinSpoolDrv, b"GetPrinterW\0".as_ptr() as PCSTR));
            fpGetPrinterDriverW = core::mem::transmute(GetProcAddress(
                hModWinSpoolDrv,
                b"GetPrinterDriverW\0".as_ptr() as PCSTR,
            ));
            fpGetSpoolFileHandle = core::mem::transmute(GetProcAddress(
                hModWinSpoolDrv,
                b"GetSpoolFileHandle\0".as_ptr() as PCSTR,
            ));
            fpIsValidDevmodeW = core::mem::transmute(GetProcAddress(
                hModWinSpoolDrv,
                b"IsValidDevmodeW\0".as_ptr() as PCSTR,
            ));
            fpOpenPrinterW =
                core::mem::transmute(GetProcAddress(hModWinSpoolDrv, b"OpenPrinterW\0".as_ptr() as PCSTR));
            fpQueryColorProfile = core::mem::transmute(GetProcAddress(
                hModWinSpoolDrv,
                b"QueryColorProfile\0".as_ptr() as PCSTR,
            ));
            fpQueryRemoteFonts = core::mem::transmute(GetProcAddress(
                hModWinSpoolDrv,
                b"QueryRemoteFonts\0".as_ptr() as PCSTR,
            ));
            fpQuerySpoolMode = core::mem::transmute(GetProcAddress(
                hModWinSpoolDrv,
                b"QuerySpoolMode\0".as_ptr() as PCSTR,
            ));
            fpReadPrinter =
                core::mem::transmute(GetProcAddress(hModWinSpoolDrv, b"ReadPrinter\0".as_ptr() as PCSTR));
            fpResetPrinterW =
                core::mem::transmute(GetProcAddress(hModWinSpoolDrv, b"ResetPrinterW\0".as_ptr() as PCSTR));
            fpSeekPrinter =
                core::mem::transmute(GetProcAddress(hModWinSpoolDrv, b"SeekPrinter\0".as_ptr() as PCSTR));
            fpSplDriverUnloadComplete = core::mem::transmute(GetProcAddress(
                hModWinSpoolDrv,
                b"SplDriverUnloadComplete\0".as_ptr() as PCSTR,
            ));
            fpSplReadPrinter =
                core::mem::transmute(GetProcAddress(hModWinSpoolDrv, 205 as PCSTR));
            fpStartDocDlgW =
                core::mem::transmute(GetProcAddress(hModWinSpoolDrv, b"StartDocDlgW\0".as_ptr() as PCSTR));
            fpStartDocPrinterW = core::mem::transmute(GetProcAddress(
                hModWinSpoolDrv,
                b"StartDocPrinterW\0".as_ptr() as PCSTR,
            ));
            fpStartPagePrinter = core::mem::transmute(GetProcAddress(
                hModWinSpoolDrv,
                b"StartPagePrinter\0".as_ptr() as PCSTR,
            ));

            if fpAbortPrinter.is_none()
                || fpClosePrinter.is_none()
                || fpCloseSpoolFileHandle.is_none()
                || fpCommitSpoolData.is_none()
                || fpDocumentEvent.is_none()
                || fpDocumentPropertiesW.is_none()
                || fpEndDocPrinter.is_none()
                || fpEndPagePrinter.is_none()
                || fpGetPrinterW.is_none()
                || fpGetPrinterDriverW.is_none()
                || fpGetSpoolFileHandle.is_none()
                || fpIsValidDevmodeW.is_none()
                || fpOpenPrinterW.is_none()
                || fpQueryColorProfile.is_none()
                || fpQueryRemoteFonts.is_none()
                || fpQuerySpoolMode.is_none()
                || fpReadPrinter.is_none()
                || fpResetPrinterW.is_none()
                || fpSeekPrinter.is_none()
                || fpSplDriverUnloadComplete.is_none()
                || fpSplReadPrinter.is_none()
                || fpStartDocDlgW.is_none()
                || fpStartDocPrinterW.is_none()
                || fpStartPagePrinter.is_none()
            {
                FreeLibrary(hModWinSpoolDrv);
                hModWinSpoolDrv = core::ptr::null_mut();
            }
            ghSpooler = hModWinSpoolDrv as HANDLE;
        }
        let _ = RtlLeaveCriticalSection(core::ptr::addr_of_mut!(semLocal));
    } else {
        return TRUE;
    }

    if ghSpooler.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
    }

    (!ghSpooler.is_null()) as BOOL
}

/*
  Note from msdn:

   The sequence for a print job is as follows:

   1. To begin a print job, call StartDocPrinter.
   2. To begin each page, call StartPagePrinter.
   3. To write data to a page, call WritePrinter.
   4. To end each page, call EndPagePrinter.
   5. Repeat 2, 3, and 4 for as many pages as necessary.
   6. To end the print job, call EndDocPrinter.

 */
#[no_mangle]
pub unsafe extern "fastcall" fn StartDocPrinterWEx(
    _pvUMPDev: PVOID,
    hPrinter: HANDLE,
    Level: DWORD,
    pDocInfo: LPBYTE,
) -> DWORD {
    (fpStartDocPrinterW.unwrap())(hPrinter, Level, pDocInfo)
}

#[no_mangle]
pub unsafe extern "fastcall" fn StartPagePrinterEx(_pvUMPDev: PVOID, hPrinter: HANDLE) -> BOOL {
    (fpStartPagePrinter.unwrap())(hPrinter)
}

/* SYSCALLS ******************************************************************/

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn AbortDoc(hdc: HDC) -> INT {
    let pldc: PLDC;
    let mut Ret: INT = SP_ERROR;
    let hType: usize = GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ);

    if hType == LO_DC_TYPE || hType == LO_METADC16_TYPE {
        SetLastError(ERROR_INVALID_HANDLE);
        return SP_ERROR;
    }

    pldc = GdiGetLDC(hdc);
    if pldc.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return SP_ERROR;
    }

    if (*pldc).Flags & LDC_INIT_DOCUMENT == 0 {
        return 1;
    }

    DocumentEventEx(
        core::ptr::null_mut(),
        (*pldc).hPrinter,
        hdc,
        DOCUMENTEVENT_ABORTDOC,
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    );

    clear_cspins();

    if (*pldc).Flags & LDC_META_PRINT != 0 {
        // UNIMPLEMENTED;
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        return Ret;
    }

    if NtGdiAbortDoc(hdc) != FALSE {
        if (fpAbortPrinter.unwrap())((*pldc).hPrinter) != FALSE {
            Ret = 1;
        }
    } else {
        Ret = SP_ERROR;
    }

    (*pldc).Flags &= !(LDC_ATENDPAGE
        | LDC_META_PRINT
        | LDC_STARTPAGE
        | LDC_INIT_PAGE
        | LDC_INIT_DOCUMENT
        | LDC_SAPCALLBACK);

    Ret
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn EndDoc(hdc: HDC) -> INT {
    let pldc: PLDC;
    let mut Ret: INT = SP_ERROR;
    let hType: usize = GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ);

    if hType == LO_DC_TYPE || hType == LO_METADC16_TYPE {
        SetLastError(ERROR_INVALID_HANDLE);
        return SP_ERROR;
    }

    pldc = GdiGetLDC(hdc);
    if pldc.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return SP_ERROR;
    }

    if (*pldc).Flags & LDC_META_PRINT != 0 {
        // UNIMPLEMENTED;
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        return Ret;
    }

    if (*pldc).Flags & LDC_INIT_DOCUMENT != 0 {
        let mut Good: BOOL;
        if (*pldc).Flags & LDC_INIT_PAGE != 0 {
            EndPage(hdc);
        }

        DocumentEventEx(
            core::ptr::null_mut(),
            (*pldc).hPrinter,
            hdc,
            DOCUMENTEVENT_ENDDOC,
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );

        clear_cspins();

        Good = NtGdiEndDoc(hdc);

        //      if (pldc->pUMPDev)
        Good = EndDocPrinterEx(core::ptr::null_mut(), (*pldc).hPrinter);

        if Good != FALSE {
            DocumentEventEx(
                core::ptr::null_mut(),
                (*pldc).hPrinter,
                hdc,
                DOCUMENTEVENT_ENDDOCPOST,
                0,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
            );
            Ret = 1;
        }
        (*pldc).Flags &=
            !(LDC_ATENDPAGE | LDC_STARTPAGE | LDC_INIT_DOCUMENT | LDC_SAPCALLBACK);
    }
    Ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn EndFormPage(hdc: HDC) -> INT {
    IntEndPage(hdc, TRUE)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn EndPage(hdc: HDC) -> INT {
    IntEndPage(hdc, FALSE)
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiGetSpoolFileHandle(
    _pwszPrinterName: LPWSTR,
    _pDevmode: LPDEVMODEW,
    _pwszDocName: LPWSTR,
) -> HANDLE {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    core::ptr::null_mut()
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiDeleteSpoolFileHandle(_SpoolFileHandle: HANDLE) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiGetPageCount(_SpoolFileHandle: HANDLE) -> DWORD {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn StartDocW(hdc: HDC, lpdi: *const DOCINFOW) -> INT {
    let pldc: PLDC;
    let mut diW: DOCINFOW = core::mem::zeroed();
    let mut di1W: DOC_INFO_1W = core::mem::zeroed();
    let mut lpwstrRet: LPWSTR = core::ptr::null_mut();
    let mut Banding: BOOL = 0;
    let PrnJobNo: INT;
    let mut Ret: INT;
    let hType: usize = GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ);

    if hType == LO_DC_TYPE || hType == LO_METADC16_TYPE {
        return SP_ERROR;
    }

    pldc = GdiGetLDC(hdc);
    if pldc.is_null() || (*pldc).Flags & LDC_ATENDPAGE != 0 {
        SetLastError(ERROR_INVALID_HANDLE);
        return SP_ERROR;
    }

    if (*pldc).hPrinter.is_null() {
        return SP_ERROR;
    }

    (*pldc).Flags &= !LDC_KILL_DOCUMENT;

    if !lpdi.is_null() {
        diW = *lpdi;
    } else {
        diW.cbSize = core::mem::size_of::<DOCINFOW>() as INT;
        diW.lpszDocName = core::ptr::null();
        diW.lpszOutput = core::ptr::null();
        diW.lpszDatatype = core::ptr::null();
        diW.fwType = 0;
    }

    if diW.lpszOutput.is_null() && !(*pldc).pwszPort.is_null() {
        diW.lpszOutput = (*pldc).pwszPort as LPCWSTR;
    }

    lpwstrRet = (fpStartDocDlgW.unwrap())((*pldc).hPrinter, core::ptr::addr_of_mut!(diW));
    if lpwstrRet as isize == SP_APPABORT as isize {
        (*pldc).Flags |= LDC_KILL_DOCUMENT;
        return SP_ERROR;
    }
    if lpwstrRet as isize == SP_ERROR as isize {
        return SP_ERROR;
    }

    if !lpwstrRet.is_null() {
        diW.lpszOutput = lpwstrRet as LPCWSTR;
    }

    Ret = DocumentEventEx(
        core::ptr::null_mut(),
        (*pldc).hPrinter,
        hdc,
        DOCUMENTEVENT_STARTDOC,
        core::mem::size_of::<ULONG>() as ULONG,
        core::ptr::addr_of_mut!(diW) as PVOID,
        0,
        core::ptr::null_mut(),
    );

    if Ret == SP_APPABORT {
        (*pldc).Flags |= LDC_KILL_DOCUMENT;
        Ret = SP_ERROR;
    }
    if Ret == SP_ERROR {
        if !lpwstrRet.is_null() {
            LocalFree(lpwstrRet as HLOCAL);
        }
        return Ret;
    }

    di1W.pDocName = diW.lpszDocName as LPWSTR;
    di1W.pOutputFile = diW.lpszOutput as LPWSTR;
    di1W.pDatatype = diW.lpszDatatype as LPWSTR;

    Ret = SP_ERROR;

    PrnJobNo = StartDocPrinterWEx(
        core::ptr::null_mut(),
        (*pldc).hPrinter,
        1,
        core::ptr::addr_of_mut!(di1W) as LPBYTE,
    ) as INT;
    if PrnJobNo <= 0 {
        Ret = NtGdiStartDoc(hdc, core::ptr::addr_of_mut!(diW), &mut Banding, PrnJobNo);
        if Ret != 0 {
            if (*pldc).pAbortProc.is_some() {
                GdiSAPCallback(pldc);
                (*pldc).Flags |= LDC_SAPCALLBACK;
                (*pldc).CallBackTick = GetTickCount();
            }
            (*pldc).Flags |= LDC_INIT_DOCUMENT;
            if Banding == 0 {
                (*pldc).Flags |= LDC_STARTPAGE;
            }
        }
    }
    if Ret == SP_ERROR {
        //if ( pldc->pUMPDev  )
        AbortPrinterEx(core::ptr::null_mut(), (*pldc).hPrinter);
        // DPRINT1("StartDoc Died!!!\n");
    } else if DocumentEventEx(
        core::ptr::null_mut(),
        (*pldc).hPrinter,
        hdc,
        DOCUMENTEVENT_STARTDOCPOST,
        core::mem::size_of::<ULONG>() as ULONG,
        core::ptr::addr_of_mut!(Ret) as PVOID,
        0,
        core::ptr::null_mut(),
    ) == SP_ERROR
    {
        AbortDoc(hdc);
        Ret = SP_ERROR;
    }
    if !lpwstrRet.is_null() {
        LocalFree(lpwstrRet as HLOCAL);
    }
    Ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn StartDocA(hdc: HDC, lpdi: *const DOCINFOA) -> INT {
    let mut szDocName: LPWSTR = core::ptr::null_mut();
    let mut szOutput: LPWSTR = core::ptr::null_mut();
    let mut szDatatype: LPWSTR = core::ptr::null_mut();
    let mut docW: DOCINFOW = core::mem::zeroed();
    let ret: INT;
    let mut len: INT;

    docW.cbSize = (*lpdi).cbSize;
    if !(*lpdi).lpszDocName.is_null() {
        len = MultiByteToWideChar(
            CP_ACP,
            0,
            (*lpdi).lpszDocName,
            -1,
            core::ptr::null_mut(),
            0,
        );
        szDocName = HeapAlloc(
            GetProcessHeap(),
            0,
            (len as SIZE_T) * core::mem::size_of::<WCHAR>() as SIZE_T,
        ) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*lpdi).lpszDocName, -1, szDocName, len);
    }
    if !(*lpdi).lpszOutput.is_null() {
        len = MultiByteToWideChar(
            CP_ACP,
            0,
            (*lpdi).lpszOutput,
            -1,
            core::ptr::null_mut(),
            0,
        );
        szOutput = HeapAlloc(
            GetProcessHeap(),
            0,
            (len as SIZE_T) * core::mem::size_of::<WCHAR>() as SIZE_T,
        ) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*lpdi).lpszOutput, -1, szOutput, len);
    }
    if !(*lpdi).lpszDatatype.is_null() {
        len = MultiByteToWideChar(
            CP_ACP,
            0,
            (*lpdi).lpszDatatype,
            -1,
            core::ptr::null_mut(),
            0,
        );
        szDatatype = HeapAlloc(
            GetProcessHeap(),
            0,
            (len as SIZE_T) * core::mem::size_of::<WCHAR>() as SIZE_T,
        ) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*lpdi).lpszDatatype, -1, szDatatype, len);
    }

    docW.lpszDocName = szDocName as LPCWSTR;
    docW.lpszOutput = szOutput as LPCWSTR;
    docW.lpszDatatype = szDatatype as LPCWSTR;
    docW.fwType = (*lpdi).fwType;

    ret = StartDocW(hdc, core::ptr::addr_of!(docW));

    HeapFree(GetProcessHeap(), 0, szDocName as LPVOID);
    HeapFree(GetProcessHeap(), 0, szOutput as LPVOID);
    HeapFree(GetProcessHeap(), 0, szDatatype as LPVOID);

    ret
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn StartPage(hdc: HDC) -> INT {
    let pldc: PLDC;
    let hType: usize = GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ);

    if hType == LO_DC_TYPE || hType == LO_METADC16_TYPE {
        SetLastError(ERROR_INVALID_HANDLE);
        return SP_ERROR;
    }

    pldc = GdiGetLDC(hdc);
    if pldc.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return SP_ERROR;
    }

    if (*pldc).Flags & LDC_META_PRINT != 0 {
        // UNIMPLEMENTED;
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        return SP_ERROR;
    }

    (*pldc).Flags &= !(LDC_ATENDPAGE | LDC_STARTPAGE);

    if (*pldc).Flags & LDC_INIT_PAGE != 0 {
        return 1;
    }

    if DocumentEventEx(
        core::ptr::null_mut(),
        (*pldc).hPrinter,
        hdc,
        DOCUMENTEVENT_STARTPAGE,
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    ) != SP_ERROR
    {
        (*pldc).Flags |= LDC_INIT_PAGE;

        clear_cspins();

        if StartPagePrinterEx(core::ptr::null_mut(), (*pldc).hPrinter) != FALSE {
            if NtGdiStartPage(hdc) != FALSE {
                return 1;
            }
        }

        (*pldc).Flags &= !LDC_INIT_PAGE;
        EndDoc(hdc);
        SetLastError(ERROR_INVALID_HANDLE);
    }
    SP_ERROR
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn StartFormPage(hdc: HDC) -> INT {
    StartPage(hdc)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetAbortProc(hdc: HDC, lpAbortProc: ABORTPROC) -> INT {
    let pldc: PLDC;
    let hType: usize = GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ);

    if hType == LO_DC_TYPE || hType == LO_METADC16_TYPE {
        return SP_ERROR;
    }

    pldc = GdiGetLDC(hdc);
    if !pldc.is_null() {
        if lpAbortProc.is_some() {
            if (*pldc).Flags & LDC_INIT_DOCUMENT != 0 {
                (*pldc).Flags |= LDC_SAPCALLBACK;
                (*pldc).CallBackTick = GetTickCount();
            }
        } else {
            (*pldc).Flags &= !LDC_SAPCALLBACK;
        }
        (*pldc).pAbortProc = lpAbortProc;
        return 1;
    } else {
        SetLastError(ERROR_INVALID_HANDLE);
    }
    SP_ERROR
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn gdiPlaySpoolStream(
    _a0: DWORD,
    _a1: DWORD,
    _a2: DWORD,
    _a3: DWORD,
    _a4: DWORD,
    _a5: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiGetDC(_SpoolFileHandle: HANDLE) -> HDC {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    core::ptr::null_mut()
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiGetPageHandle(
    _SpoolFileHandle: HANDLE,
    _Page: DWORD,
    _pdwPageType: LPDWORD,
) -> HANDLE {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    core::ptr::null_mut()
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiStartDocEMF(
    _SpoolFileHandle: HANDLE,
    _pDocInfo: *mut DOCINFOW,
) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiStartPageEMF(_SpoolFileHandle: HANDLE) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiPlayPageEMF(
    _SpoolFileHandle: HANDLE,
    _hemf: HANDLE,
    _prectDocument: *mut RECT,
    _prectBorder: *mut RECT,
    _prectClip: *mut RECT,
) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiEndPageEMF(
    _SpoolFileHandle: HANDLE,
    _dwOptimization: DWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiEndDocEMF(_SpoolFileHandle: HANDLE) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiGetDevmodeForPage(
    _SpoolFileHandle: HANDLE,
    _dwPageNumber: DWORD,
    _pCurrDM: *mut PDEVMODEW,
    _pLastDM: *mut PDEVMODEW,
) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiResetDCEMF(
    _SpoolFileHandle: HANDLE,
    _pCurrDM: PDEVMODEW,
) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiPlayEMF(
    _pwszPrinterName: LPWSTR,
    _pDevmode: LPDEVMODEW,
    _pwszDocName: LPWSTR,
    _pfnEMFPlayFn: EMFPLAYPROC,
    _hPageQuery: HANDLE,
) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiPlayPrivatePageEMF(
    _SpoolFileHandle: HANDLE,
    _unknown: DWORD,
    _prectDocument: *mut RECT,
) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiPrinterThunk(
    _humpd: HUMPD,
    _status: *mut DWORD,
    _unuse: DWORD,
) -> BOOL {
    /* FIXME figout the protypes, the HUMPD are a STRUCT or COM object */
    /* status contain some form of return value that being save, what it is I do not known */
    /* unsue seam have zero effect, what it is for I do not known */

    // ? return NtGdiSetPUMPDOBJ(humpd->0x10,TRUE, humpd, ?) <- blackbox, OpenRCE info, and api hooks for anylaysing;
    FALSE
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiArtificialDecrementDriver(
    _pDriverName: LPWSTR,
    _unknown: BOOL,
) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}
