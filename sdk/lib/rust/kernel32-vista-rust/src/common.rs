//! Shared FFI surface + helpers for kernel32-vista-rust. The Nt*/Rtl* + win32
//! types/constants come from `ntdll_sys::nt::*`; this module declares only the
//! symbols bindgen lacks (kernel32 exports, kernel32-shared-rust exports, a few
//! ntdll Rtl routines) plus the LCID bit macros and the NT6 firmware struct.

pub use ntdll_sys::nt::*;

// ---- kernel32.dll exports (kernel32.dll defines them; kernel32_vista.dll
//      imports kernel32). WINAPI → stdcall on i386 (`_Name@N`). ----
extern "system" {
    pub fn SetLastError(dwErrCode: DWORD);
    pub fn GetUserDefaultLCID() -> LCID;
    pub fn GetSystemDefaultLCID() -> LCID;
    pub fn GetLocaleInfoW(Locale: LCID, LCType: LCTYPE, lpLCData: LPWSTR, cchData: core::ffi::c_int)
        -> core::ffi::c_int;
    pub fn lstrlenW(lpString: LPCWSTR) -> core::ffi::c_int;
}

// ---- kernel32-shared-rust ----
extern "system" {
    pub fn BaseSetLastNTError(Status: NTSTATUS) -> DWORD;
}

// ---- ntdll Rtl routines not in bindgen ----
extern "system" {
    pub fn RtlLcidToLocaleName(
        Lcid: LCID,
        LocaleName: PUNICODE_STRING,
        Flags: ULONG,
        AllocateDestinationString: BOOLEAN,
    ) -> NTSTATUS;
    pub fn RtlLocaleNameToLcid(LocaleName: PCWSTR, Lcid: *mut LCID, Flags: ULONG) -> NTSTATUS;
    pub fn RtlRunOnceExecuteOnce(
        RunOnce: PRTL_RUN_ONCE,
        InitFn: PRTL_RUN_ONCE_INIT_FN,
        Parameter: PVOID,
        Context: *mut PVOID,
    ) -> NTSTATUS;
    pub fn RtlRunOnceBeginInitialize(
        RunOnce: PRTL_RUN_ONCE,
        Flags: ULONG,
        Context: *mut PVOID,
    ) -> NTSTATUS;
    pub fn RtlRunOnceComplete(RunOnce: PRTL_RUN_ONCE, Flags: ULONG, Context: PVOID) -> NTSTATUS;
}

// ---- CRT (wine strlenW/strcpyW = wcslen/wcscpy) ----
extern "C" {
    pub fn wcslen(s: *const WCHAR) -> usize;
    pub fn wcscpy(dst: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    pub fn wcschr(s: *const WCHAR, c: WCHAR) -> *mut WCHAR;
}

// ---- more kernel32.dll exports (wave 2) ----
extern "system" {
    pub fn GetLastError() -> DWORD;
    pub fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HMODULE;
    pub fn GetFullPathNameW(
        lpFileName: LPCWSTR,
        nBufferLength: DWORD,
        lpBuffer: LPWSTR,
        lpFilePart: *mut LPWSTR,
    ) -> DWORD;
    pub fn GetVolumeNameForVolumeMountPointW(
        lpszVolumeMountPoint: LPCWSTR,
        lpszVolumeName: LPWSTR,
        cchBufferLength: DWORD,
    ) -> BOOL;
    pub fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: LPCWSTR,
        cchWideChar: core::ffi::c_int,
        lpMultiByteStr: LPSTR,
        cbMultiByte: core::ffi::c_int,
        lpDefaultChar: LPCSTR,
        lpUsedDefaultChar: *mut BOOL,
    ) -> core::ffi::c_int;
    pub fn EnumResourceLanguagesW(
        hModule: HMODULE,
        lpType: LPCWSTR,
        lpName: LPCWSTR,
        lpEnumFunc: ENUMRESLANGPROCW,
        lParam: LONG_PTR,
    ) -> BOOL;
}

// ---- kernel32-shared-rust (cdecl, no WINAPI) ----
extern "C" {
    pub fn FilenameA2W(NameA: LPCSTR, alloc: BOOL) -> PWCHAR;
    pub fn FilenameU2A_FitOrFail(DestA: LPSTR, destLen: INT, SourceU: PUNICODE_STRING) -> DWORD;
}

// ---- constants bindgen lacks (wave 2) ----
pub const PROCESS_NAME_NATIVE: DWORD = 0x0000_0001;
pub const CP_ACP: UINT = 0;
pub const WC_NO_BEST_FIT_CHARS: DWORD = 0x0000_0400;
pub const SYMBOLIC_LINK_FLAG_DIRECTORY: DWORD = 0x1;
pub const NO_ERROR: DWORD = 0;
pub const E_FAIL: HRESULT = 0x8000_4005u32 as HRESULT;
pub const FSCTL_SET_REPARSE_POINT: ULONG = 0x0009_00A4; // CTL_CODE(FS, 41, BUFFERED, ANY)
pub const VOLUME_NAME_DOS: DWORD = 0x0;
pub const VOLUME_NAME_GUID: DWORD = 0x1;
pub const VOLUME_NAME_NT: DWORD = 0x2;
pub const VOLUME_NAME_NONE: DWORD = 0x4;
pub const LOCALE_NOUSEROVERRIDE: DWORD = 0x8000_0000;
pub const LOCALE_RETURN_NUMBER: DWORD = 0x2000_0000;
pub const LOCALE_INEUTRAL: LCTYPE = 0x0071;
pub const LOCALE_WINDOWS: DWORD = 0x01;
pub const LOCALE_NEUTRALDATA: DWORD = 0x10;
pub const LOCALE_SPECIFICDATA: DWORD = 0x20;
pub const RT_STRING_ID: u16 = 6; // MAKEINTRESOURCE(6)
// CTL_CODE(MOUNTMGRCONTROLTYPE='m'=0x6D, 12, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_MOUNTMGR_QUERY_DOS_VOLUME_PATH: ULONG = (0x6D << 16) | (12 << 2);

// ---- ddk/mountmgr.h structs (not in bindgen) ----
#[repr(C)]
pub struct MOUNTMGR_TARGET_NAME {
    pub DeviceNameLength: USHORT,
    pub DeviceName: [WCHAR; 1],
}
#[repr(C)]
pub struct MOUNTMGR_VOLUME_PATHS {
    pub MultiSzLength: ULONG,
    pub MultiSz: [WCHAR; 1],
}

// ---- constants bindgen lacks ----
pub type LCTYPE = DWORD; // winnls.h
pub const LOCALE_ALLOW_NEUTRAL_NAMES: DWORD = 0x0800_0000; // winnls.h
pub const RTL_LOCALE_ALLOW_NEUTRAL_NAMES: ULONG = 0x0000_0002; // ndk/rtlfuncs.h
pub const LOCALE_USER_DEFAULT: LCID = 0x0400; // MAKELCID(LANG_USER_DEFAULT, SORT_DEFAULT)
pub const LOCALE_SNAME: LCTYPE = 0x0000_005c; // winnls.h
pub const LOCALE_SPARENT: LCTYPE = 0x0000_006d; // winnls.h

// ---- SharedUserData (KUSER_SHARED_DATA at the fixed user VA 0x7FFE0000) ----
pub const SharedUserData: *mut KUSER_SHARED_DATA = 0x7FFE_0000 as *mut KUSER_SHARED_DATA;

// ---- NT6 firmware (NTDDI_VISTA layout: no Win8 BootFlags) ----
pub type FIRMWARE_TYPE = u32; // FirmwareTypeUnknown/Bios/Uefi/...
pub type PFIRMWARE_TYPE = *mut FIRMWARE_TYPE;
#[repr(C)]
pub struct SYSTEM_BOOT_ENVIRONMENT_INFORMATION {
    pub BootIdentifier: GUID,
    pub FirmwareType: FIRMWARE_TYPE,
}

pub const UNICODE_NULL: WCHAR = 0;

#[inline]
pub fn NT_SUCCESS(Status: NTSTATUS) -> bool {
    Status >= 0
}

/// RtlCopyMemory (NDK macro) — memcpy.
#[inline]
pub unsafe fn RtlCopyMemory(Destination: PVOID, Source: *const core::ffi::c_void, Length: SIZE_T) {
    core::ptr::copy_nonoverlapping(Source as *const u8, Destination as *mut u8, Length as usize);
}

/// RtlGetProcessHeap (FORCEINLINE) — NtCurrentPeb()->ProcessHeap.
#[inline]
pub unsafe fn RtlGetProcessHeap() -> PVOID {
    (*NtCurrentPeb()).ProcessHeap
}

// ---- LCID / LANGID bit macros (winnt.h) ----
#[inline]
pub fn MAKELANGID(primary: u16, sub: u16) -> u16 {
    (sub << 10) | (primary & 0x3ff)
}
#[inline]
pub fn MAKELCID(langid: u16, sortid: u16) -> LCID {
    ((sortid as u32) << 16) | (langid as u32)
}
#[inline]
pub fn LANGIDFROMLCID(lcid: LCID) -> u16 {
    lcid as u16
}
#[inline]
pub fn PRIMARYLANGID(langid: u16) -> u16 {
    langid & 0x3ff
}
#[inline]
pub fn SUBLANGID(langid: u16) -> u16 {
    langid >> 10
}
