//! objects/eng.c → Rust. GDI engine-entry client API.
//
// Wave-4 (closes gdi32 to 100% Rust). Same crate/substrate as Waves 1a/1b/2/2b/3a/3b
// (all landed). Imports: `win32k_sys::win32k::*` + `crate::gdihelp::*`.
//
// Thin EngXxx WINAPI wrappers (→ `extern "system"`), mostly forwarders to NtGdi*/Rtl*/
// kernel32. No #define remapping, no float-by-value.
//
// NEEDS-TYPE: `UMPDEV`/`PUMPDEV` (gdi32p.h) and `DRIVER_INFO_5W`/`PDRIVER_INFO_5W`
//   (winspool.h) are NOT in the win32k_sys bindgen surface — reproduced locally as
//   `#[repr(C)]` structs (faithful field/offset layout). `UMPDEV` is only ever cast-to and
//   read (Sig + pdi5Info); it is never instantiated here, so the trailing `apfn[INDEX_LAST]`
//   array size (winddi.h INDEX_LAST = 93 at the tree's NTDDI) is layout-immaterial.
// NEEDS-IMPORT: constants absent from win32k_sys bindgen, reproduced as local consts with
//   psdk values: PDEV_UMPD_ID (0xFEDCBA98, gdi32p.h), LOAD_LIBRARY_AS_DATAFILE (0x2,
//   winbase.h). `MAKEINTRESOURCEW` (a winuser.h macro) reimplemented as a local inline.
// NEEDS-IMPORT: cross-DLL fns declared in the extern "system" blocks below —
//   - ntdll (NTAPI):  RtlEnterCriticalSection / RtlLeaveCriticalSection /
//     RtlInitializeCriticalSection / RtlDeleteCriticalSection / RtlAllocateHeap / RtlFreeHeap
//   - kernel32 (WINAPI): FindResourceW / SizeofResource / LoadResource / LockResource /
//     FreeLibrary / GetOEMCP / GetACP / LoadLibraryExW / GlobalAlloc / GlobalFree /
//     GetProcessHeap / MultiByteToWideChar / WideCharToMultiByte / GetLocalTime
//   NtGdiEngComputeGlyphSet / NtGdiGetDhpdev come from the win32k_sys glob (do NOT redeclare).
// NEEDS-CHECK: `copy_my_glyphset` is a plain cdecl C function (no WINAPI, no `static`) →
//   `#[no_mangle] extern "C"`. Its `memcpy(src_glyphset, dst_glyphset, Size)` copies
//   dst→src (note the C memcpy(dest,source,len) order with the oddly-named params) →
//   `core::ptr::copy_nonoverlapping(dst_glyphset, src_glyphset, Size)` (Rust (src,dst,len)).

use win32k_sys::win32k::*;
use crate::gdihelp::*;

// win32k_sys's FALSE/TRUE are bindgen u32 (#define), but BOOL is i32 — shadow with
// BOOL-typed locals so `== FALSE`/`return TRUE` type-check against the i32 BOOL returns.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// gdi32p.h
const PDEV_UMPD_ID: DWORD_PTR = 0xFEDCBA98;
// winbase.h
const LOAD_LIBRARY_AS_DATAFILE: DWORD = 0x0000_0002;

// winuser.h: MAKEINTRESOURCEW(i) == (LPWSTR)((ULONG_PTR)((WORD)(i)))
#[inline]
unsafe fn MAKEINTRESOURCEW(i: INT) -> LPWSTR {
    (i as WORD as ULONG_PTR) as LPWSTR
}

// ── winspool.h _DRIVER_INFO_5W (not in win32k_sys bindgen) ─────────────────────
#[repr(C)]
pub struct _DRIVER_INFO_5W {
    pub cVersion: DWORD,
    pub pName: LPWSTR,
    pub pEnvironment: LPWSTR,
    pub pDriverPath: LPWSTR,
    pub pDataFile: LPWSTR,
    pub pConfigFile: LPWSTR,
    pub dwDriverAttributes: DWORD,
    pub dwConfigVersion: DWORD,
    pub dwDriverVersion: DWORD,
}
pub type DRIVER_INFO_5W = _DRIVER_INFO_5W;
pub type PDRIVER_INFO_5W = *mut _DRIVER_INFO_5W;

// ── gdi32p.h _UMPDEV (not in win32k_sys bindgen) ──────────────────────────────
// winddi.h:526 INDEX_LAST = 93 at the tree's NTDDI (_WIN32_WINNT=0x502, pre-Vista; 103 is
// the Win7 value). UMPDEV is only ever cast-to / read here (Sig + pdi5Info), never
// instantiated, so apfn's exact extent is layout-immaterial — but match gdi32p.h anyway.
const INDEX_LAST: usize = 93;
#[repr(C)]
pub struct _UMPDEV {
    pub Sig: DWORD_PTR, // Init with PDEV_UMPD_ID
    pub pumpdNext: *mut _UMPDEV,
    pub pdi5Info: PDRIVER_INFO_5W,
    pub hModule: HMODULE,
    pub dwFlags: DWORD,
    pub dwDriverAttributes: DWORD,
    pub dwConfigVersion: DWORD,
    pub dwDriverCount: DWORD,
    pub WOW64_UMPDev: DWORD,
    pub WOW64_hMod: DWORD,
    pub Unknown: DWORD,
    pub apfn: [PVOID; INDEX_LAST],
}
pub type UMPDEV = _UMPDEV;
pub type PUMPDEV = *mut _UMPDEV;

// ── ntdll (NTAPI → extern "system") ───────────────────────────────────────────
extern "system" {
    fn RtlEnterCriticalSection(CriticalSection: PRTL_CRITICAL_SECTION) -> NTSTATUS;
    fn RtlLeaveCriticalSection(CriticalSection: PRTL_CRITICAL_SECTION) -> NTSTATUS;
    fn RtlInitializeCriticalSection(CriticalSection: PRTL_CRITICAL_SECTION) -> NTSTATUS;
    fn RtlDeleteCriticalSection(CriticalSection: PRTL_CRITICAL_SECTION) -> NTSTATUS;
    fn RtlAllocateHeap(HeapHandle: PVOID, Flags: ULONG, Size: SIZE_T) -> PVOID;
    fn RtlFreeHeap(HeapHandle: PVOID, Flags: ULONG, BaseAddress: PVOID) -> BOOLEAN;
}

// ── kernel32 (WINAPI → extern "system") ───────────────────────────────────────
extern "system" {
    fn FindResourceW(hModule: HMODULE, lpName: LPCWSTR, lpType: LPCWSTR) -> HRSRC;
    fn SizeofResource(hModule: HMODULE, hResInfo: HRSRC) -> DWORD;
    fn LoadResource(hModule: HMODULE, hResInfo: HRSRC) -> HGLOBAL;
    fn LockResource(hResData: HGLOBAL) -> LPVOID;
    fn FreeLibrary(hLibModule: HMODULE) -> BOOL;
    fn GetOEMCP() -> UINT;
    fn GetACP() -> UINT;
    fn LoadLibraryExW(lpLibFileName: LPCWSTR, hFile: HANDLE, dwFlags: DWORD) -> HMODULE;
    fn GlobalAlloc(uFlags: UINT, dwBytes: SIZE_T) -> HGLOBAL;
    fn GlobalFree(hMem: HGLOBAL) -> HGLOBAL;
    fn GetProcessHeap() -> HANDLE;
    fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: INT,
        lpWideCharStr: LPWSTR,
        cchWideChar: INT,
    ) -> INT;
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
    fn GetLocalTime(lpSystemTime: LPSYSTEMTIME);
}

// minwinbase.h _SYSTEMTIME (not in win32k_sys bindgen).
#[repr(C)]
pub struct SYSTEMTIME {
    pub wYear: WORD,
    pub wMonth: WORD,
    pub wDayOfWeek: WORD,
    pub wDay: WORD,
    pub wHour: WORD,
    pub wMinute: WORD,
    pub wSecond: WORD,
    pub wMilliseconds: WORD,
}
pub type LPSYSTEMTIME = *mut SYSTEMTIME;

/// @implemented
#[no_mangle]
pub unsafe extern "system" fn EngAcquireSemaphore(hsem: HSEMAPHORE) {
    RtlEnterCriticalSection(hsem as PRTL_CRITICAL_SECTION);
}

/// @unimplemented
#[no_mangle]
pub unsafe extern "C" fn copy_my_glyphset(
    dst_glyphset: *mut FD_GLYPHSET,
    src_glyphset: *mut FD_GLYPHSET,
    Size: ULONG,
) -> BOOL {
    let mut retValue: BOOL = FALSE;

    // memcpy(src_glyphset, dst_glyphset, Size) — dest=src_glyphset, source=dst_glyphset
    core::ptr::copy_nonoverlapping(
        dst_glyphset as *const u8,
        src_glyphset as *mut u8,
        Size as usize,
    );
    if (*src_glyphset).cRuns == 0 {
        retValue = TRUE;
    }

    // FIXME copy wrun
    retValue
}

/// @implemented
#[no_mangle]
pub unsafe extern "system" fn EngComputeGlyphSet(
    nCodePage: INT,
    nFirstChar: INT,
    cChars: INT,
) -> *mut FD_GLYPHSET {
    let mut myfd_glyphset: *mut FD_GLYPHSET = core::ptr::null_mut();

    let ntfd_glyphset = NtGdiEngComputeGlyphSet(nCodePage, nFirstChar, cChars);

    if !ntfd_glyphset.is_null() {
        if (*ntfd_glyphset).cjThis != 0 {
            myfd_glyphset = GlobalAlloc(0, (*ntfd_glyphset).cjThis as SIZE_T) as *mut FD_GLYPHSET;
            if !myfd_glyphset.is_null() {
                if copy_my_glyphset(myfd_glyphset, ntfd_glyphset, (*ntfd_glyphset).cjThis) == FALSE {
                    GlobalFree(myfd_glyphset as HGLOBAL);
                    myfd_glyphset = core::ptr::null_mut();
                }
            }
        }
    }
    myfd_glyphset
}

/// @implemented
#[no_mangle]
pub unsafe extern "system" fn EngCreateSemaphore() -> HSEMAPHORE {
    let CritSect = RtlAllocateHeap(
        GetProcessHeap(),
        0,
        core::mem::size_of::<RTL_CRITICAL_SECTION>() as SIZE_T,
    ) as PRTL_CRITICAL_SECTION;
    if CritSect.is_null() {
        return core::ptr::null_mut();
    }

    RtlInitializeCriticalSection(CritSect);
    CritSect as HSEMAPHORE
}

/// @implemented
#[no_mangle]
pub unsafe extern "system" fn EngDeleteSemaphore(hsem: HSEMAPHORE) {
    if !hsem.is_null() {
        RtlDeleteCriticalSection(hsem as PRTL_CRITICAL_SECTION);
        RtlFreeHeap(GetProcessHeap(), 0, hsem as PVOID);
    }
}

/// @implemented
#[no_mangle]
pub unsafe extern "system" fn EngFindResource(
    h: HANDLE,
    iName: INT,
    iType: INT,
    pulSize: PULONG,
) -> PVOID {
    let mut Size: DWORD = 0;
    let mut Lock: LPVOID = core::ptr::null_mut();

    let HRSrc = FindResourceW(
        h as HMODULE,
        MAKEINTRESOURCEW(iName),
        MAKEINTRESOURCEW(iType),
    );
    if !HRSrc.is_null() {
        Size = SizeofResource(h as HMODULE, HRSrc);
        if Size != 0 {
            let Hg = LoadResource(h as HMODULE, HRSrc);
            if !Hg.is_null() {
                Lock = LockResource(Hg);
            }
        }
    }

    *pulSize = Size;
    Lock as PVOID
}

/// @implemented
#[no_mangle]
pub unsafe extern "system" fn EngFreeModule(h: HANDLE) {
    FreeLibrary(h as HMODULE);
}

/// @implemented
#[no_mangle]
pub unsafe extern "system" fn EngGetCurrentCodePage(OemCodePage: PUSHORT, AnsiCodePage: PUSHORT) {
    *OemCodePage = GetOEMCP() as USHORT;
    *AnsiCodePage = GetACP() as USHORT;
}

/// @implemented
#[no_mangle]
pub unsafe extern "system" fn EngGetDriverName(hdev: HDEV) -> LPWSTR {
    // DHPDEV from NtGdiGetDhpdev must be from print driver.
    let mut pPDev = NtGdiGetDhpdev(hdev) as PUMPDEV;

    if pPDev.is_null() {
        return core::ptr::null_mut();
    }

    if (*pPDev).Sig != PDEV_UMPD_ID {
        pPDev = (*pPDev).Sig as PUMPDEV;
    }
    (*(*pPDev).pdi5Info).pDriverPath
}

/// @implemented
#[no_mangle]
pub unsafe extern "system" fn EngGetPrinterDataFileName(hdev: HDEV) -> LPWSTR {
    let mut pPDev = NtGdiGetDhpdev(hdev) as PUMPDEV;

    if pPDev.is_null() {
        return core::ptr::null_mut();
    }

    if (*pPDev).Sig != PDEV_UMPD_ID {
        pPDev = (*pPDev).Sig as PUMPDEV;
    }
    (*(*pPDev).pdi5Info).pDataFile
}

/// @implemented
#[no_mangle]
pub unsafe extern "system" fn EngLoadModule(pwsz: LPWSTR) -> HANDLE {
    LoadLibraryExW(pwsz, core::ptr::null_mut(), LOAD_LIBRARY_AS_DATAFILE) as HANDLE
}

/// @implemented
#[no_mangle]
pub unsafe extern "system" fn EngMultiByteToWideChar(
    CodePage: UINT,
    WideCharString: LPWSTR,
    BytesInWideCharString: INT,
    MultiByteString: LPSTR,
    BytesInMultiByteString: INT,
) -> INT {
    MultiByteToWideChar(
        CodePage,
        0,
        MultiByteString,
        BytesInMultiByteString,
        WideCharString,
        BytesInWideCharString / core::mem::size_of::<WCHAR>() as INT,
    )
}

/// @implemented
#[no_mangle]
pub unsafe extern "system" fn EngQueryLocalTime(etf: PENG_TIME_FIELDS) {
    let mut SystemTime: SYSTEMTIME = core::mem::zeroed();
    GetLocalTime(&mut SystemTime);
    (*etf).usYear = SystemTime.wYear;
    (*etf).usMonth = SystemTime.wMonth;
    (*etf).usWeekday = SystemTime.wDayOfWeek;
    (*etf).usDay = SystemTime.wDay;
    (*etf).usHour = SystemTime.wHour;
    (*etf).usMinute = SystemTime.wMinute;
    (*etf).usSecond = SystemTime.wSecond;
    (*etf).usMilliseconds = SystemTime.wMilliseconds;
}

/// @implemented
#[no_mangle]
pub unsafe extern "system" fn EngReleaseSemaphore(hsem: HSEMAPHORE) {
    RtlLeaveCriticalSection(hsem as PRTL_CRITICAL_SECTION);
}

/// @implemented
#[no_mangle]
pub unsafe extern "system" fn EngWideCharToMultiByte(
    CodePage: UINT,
    WideCharString: LPWSTR,
    BytesInWideCharString: INT,
    MultiByteString: LPSTR,
    BytesInMultiByteString: INT,
) -> INT {
    WideCharToMultiByte(
        CodePage,
        0,
        WideCharString,
        BytesInWideCharString / core::mem::size_of::<WCHAR>() as INT,
        MultiByteString,
        BytesInMultiByteString,
        core::ptr::null(),
        core::ptr::null_mut(),
    )
}

/// @unimplemented
#[no_mangle]
pub unsafe extern "system" fn XLATEOBJ_piVector(XlateObj: *mut XLATEOBJ) -> *mut ULONG {
    (*XlateObj).pulXlate
}
