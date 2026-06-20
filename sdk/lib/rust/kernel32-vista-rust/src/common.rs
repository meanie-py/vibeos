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

#[inline]
pub fn NT_SUCCESS(Status: NTSTATUS) -> bool {
    Status >= 0
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
