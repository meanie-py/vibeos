//! k32_externs.rs — the cohab import surface for proc.c.
//!
//! Everything proc.c *calls* or *reads* that is defined in another translation
//! unit. Three groups:
//!   1. kernel32 `Base*` helpers defined in the OTHER kernel32 .c files that
//!      stay C (utils.c / path.c / vdm.c / thread.c / appcache / ...).
//!   2. cross-file Win32 exports proc.c calls (file.c / loader.c / except.c ...).
//!   3. extern data + the SharedUserData fixed-VA accessors + the C macros
//!      proc.c uses (IsConsoleHandle, AddToHandle, MAKELONG, HandleToUlong, ...).
//!
//! WINAPI and NTAPI both lower to `extern "system"` (stdcall on i386, cdecl on
//! amd64/arm64) — matches the C headers exactly. `WaitForInputIdleType` is the
//! lone __cdecl callback (see k32_types).

use ntdll_sys::nt::*;
// The crate's own functions: the definition is the declaration.
pub use crate::client::utils::Basep8BitStringToDynamicUnicodeString;
pub use crate::client::handle::CloseHandle;
pub use crate::file::create::CreateFileW;
pub use crate::client::thread::ExitThread;

extern "system" {
    /* ---- 1. kernel32 Base* helpers (kernel32.h) ----------------------- */
    pub fn BaseSetLastNTError(Status: NTSTATUS) -> DWORD;
    pub fn BaseFormatObjectAttributes(
        ObjectAttributes: POBJECT_ATTRIBUTES,
        SecurityAttributes: LPSECURITY_ATTRIBUTES,
        ObjectName: PUNICODE_STRING,
    ) -> POBJECT_ATTRIBUTES;
    pub fn BaseCreateStack(
        hProcess: HANDLE,
        StackCommit: SIZE_T,
        StackReserve: SIZE_T,
        InitialTeb: PINITIAL_TEB,
    ) -> NTSTATUS;
    pub fn BaseInitializeContext(
        Context: PCONTEXT,
        Parameter: PVOID,
        StartAddress: PVOID,
        StackAddress: PVOID,
        ContextType: ULONG,
    );
    pub fn BasepIsRealtimeAllowed(Keep: BOOLEAN) -> PVOID;
    pub fn BasepAnsiStringToHeapUnicodeString(AnsiString: LPCSTR, UnicodeString: *mut LPWSTR);
    pub fn BaseComputeProcessDllPath(FullPath: LPWSTR, Environment: PVOID) -> LPWSTR;
    pub fn BaseComputeProcessExePath(FullPath: LPWSTR) -> LPWSTR;
    pub fn BaseIsDosApplication(PathName: PUNICODE_STRING, Status: NTSTATUS) -> ULONG;
    pub fn BasepCheckBadapp(
        FileHandle: HANDLE,
        ApplicationName: PWCHAR,
        Environment: PWCHAR,
        ExeType: USHORT,
        SdbQueryAppCompatData: *mut PVOID,
        SdbQueryAppCompatDataSize: PULONG,
        SxsData: *mut PVOID,
        SxsDataSize: PULONG,
        FusionFlags: PULONG,
    ) -> NTSTATUS;
    pub fn BasepFreeAppCompatData(AppCompatData: PVOID, AppCompatSxsData: PVOID);
    pub fn BasepCheckWinSaferRestrictions(
        UserToken: HANDLE,
        ApplicationName: LPWSTR,
        FileHandle: HANDLE,
        InJob: PBOOLEAN,
        NewToken: PHANDLE,
        JobHandle: PHANDLE,
    ) -> NTSTATUS;

    /* ---- VDM (vdm.c / vdm.h) ------------------------------------------ */
    pub fn BaseCreateVDMEnvironment(
        lpEnvironment: PWCHAR,
        AnsiEnv: PANSI_STRING,
        UnicodeEnv: PUNICODE_STRING,
    ) -> BOOL;
    pub fn BaseDestroyVDMEnvironment(AnsiEnv: PANSI_STRING, UnicodeEnv: PUNICODE_STRING) -> BOOL;
    pub fn BaseGetVdmConfigInfo(
        CommandLineReserved: LPCWSTR,
        DosSeqId: ULONG,
        BinaryType: ULONG,
        CmdLineString: PUNICODE_STRING,
        VdmSize: PULONG,
    ) -> BOOL;
    pub fn BaseUpdateVDMEntry(
        UpdateIndex: ULONG,
        WaitHandle: PHANDLE,
        IndexInfo: ULONG,
        BinaryType: ULONG,
    ) -> BOOL;
    pub fn BaseCheckForVDM(ProcessHandle: HANDLE, ExitCode: LPDWORD) -> BOOL;
    pub fn BaseCheckVDM(
        BinaryType: ULONG,
        ApplicationName: PCWCH,
        CommandLine: PCWCH,
        CurrentDirectory: PCWCH,
        AnsiEnvironment: PANSI_STRING,
        ApiMessage: PBASE_API_MESSAGE,
        iTask: PULONG,
        CreationFlags: DWORD,
        StartupInfo: LPSTARTUPINFOW,
        hUserToken: HANDLE,
    ) -> NTSTATUS;

    /* ---- 2. cross-file Win32 exports (other kernel32 .c) -------------- */
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD;
    pub fn SearchPathW(
        lpPath: LPCWSTR,
        lpFileName: LPCWSTR,
        lpExtension: LPCWSTR,
        nBufferLength: DWORD,
        lpBuffer: LPWSTR,
        lpFilePart: *mut LPWSTR,
    ) -> DWORD;
    pub fn GetLastError() -> DWORD;
    pub fn SetLastError(dwErrCode: DWORD);
    pub fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HMODULE;
    pub fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> FARPROC;
    pub fn LoadLibraryW(lpLibFileName: LPCWSTR) -> HMODULE;
    pub fn GetSystemDirectoryW(lpBuffer: LPWSTR, uSize: UINT) -> UINT;
    pub fn GetFullPathNameW(
        lpFileName: LPCWSTR,
        nBufferLength: DWORD,
        lpBuffer: LPWSTR,
        lpFilePart: *mut LPWSTR,
    ) -> DWORD;
    pub fn UnhandledExceptionFilter(ExceptionInfo: LPEXCEPTION_POINTERS) -> LONG;
    pub fn IsBadWritePtr(lp: LPVOID, ucb: UINT_PTR) -> BOOL;

}

/* ---- 3a. ntdll inline/macro helpers (NOT real ntdll exports) ---------
 * RtlGetProcessHeap() is a macro `(NtCurrentPeb()->ProcessHeap)` in
 * ndk/rtlfuncs.h, not an exported symbol — implement it in-crate so call
 * sites bind directly (no `_RtlGetProcessHeap@0` import). The RtlInitEmpty*
 * FORCEINLINEs live in k32_inlines (provided as #[no_mangle] to satisfy the
 * ntdll-sys extern decls the call sites resolve to). */
#[inline]
pub fn RtlGetProcessHeap() -> PVOID {
    unsafe { (*NtCurrentPeb()).ProcessHeap }
}

/* ---- 3b. extern data (defined elsewhere) ----------------------------- */
// NOTE: BaseStaticServerData is already declared by ntdll-sys bindgen
// (csr/basesrv surface) — re-exported via substrate. Do not redeclare it or the
// glob imports become ambiguous. BaseRunningInServerProcess is NOT in bindgen.
extern "C" {
    pub static mut BaseRunningInServerProcess: BOOLEAN;
}

#[cfg(target_arch = "aarch64")]
extern "C" {
    pub static mut BaseCsrConnectionEstablished: BOOLEAN;
}

/* ---- 3c. SharedUserData (fixed VA 0x7FFE0000) ------------------------ */
// KUSER_SHARED_DATA is not in ntdll-sys; rather than mirror the whole struct
// (the "hand offset" hazard from the field manual), read individual fields at
// their verified offsets. Offsets confirmed via in-repo clang __builtin_offsetof
// against <ndk/ketypes.h> (i386+amd64, layout is arch-independent):
//   NtMajorVersion 0x26C, NtMinorVersion 0x270, SuiteMask 0x2D0,
//   ImageNumberLow 0x2C, ImageNumberHigh 0x2E.
pub const K32_USER_SHARED_DATA: usize = 0x7FFE_0000;

#[inline]
pub unsafe fn SharedUserData_NtMajorVersion() -> ULONG {
    core::ptr::read_volatile((K32_USER_SHARED_DATA + 0x26C) as *const ULONG)
}
#[inline]
pub unsafe fn SharedUserData_NtMinorVersion() -> ULONG {
    core::ptr::read_volatile((K32_USER_SHARED_DATA + 0x270) as *const ULONG)
}
#[inline]
pub unsafe fn SharedUserData_SuiteMask() -> ULONG {
    core::ptr::read_volatile((K32_USER_SHARED_DATA + 0x2D0) as *const ULONG)
}
#[inline]
pub unsafe fn SharedUserData_ImageNumberLow() -> USHORT {
    core::ptr::read_volatile((K32_USER_SHARED_DATA + 0x2C) as *const USHORT)
}
#[inline]
pub unsafe fn SharedUserData_ImageNumberHigh() -> USHORT {
    core::ptr::read_volatile((K32_USER_SHARED_DATA + 0x2E) as *const USHORT)
}

/* ---- 3c2. constants/macros missing from ntdll-sys bindgen ------------ */

pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
pub const STARTF_SHELLPRIVATE: DWORD = 0x400; // == STARTF_HASSHELLDATA

pub const STATUS_FATAL_APP_EXIT: NTSTATUS = 0x4000_0015u32 as NTSTATUS;
pub const STATUS_INVALID_IMAGE_FORMAT: NTSTATUS = 0xC000_007Bu32 as NTSTATUS;
pub const STATUS_ENTRYPOINT_NOT_FOUND: NTSTATUS = 0xC000_0139u32 as NTSTATUS;

/// `CSR_CREATE_API_NUMBER(DllIndex, ApiIndex)` (csr/csr.h).
#[inline]
pub fn CSR_CREATE_API_NUMBER(DllIndex: ULONG, ApiIndex: ULONG) -> ULONG {
    (DllIndex << 16) | ApiIndex
}

/// `RtlZeroMemory(Destination, Length)` — memset 0.
#[inline]
pub unsafe fn RtlZeroMemory(Destination: PVOID, Length: SIZE_T) {
    core::ptr::write_bytes(Destination as *mut u8, 0, Length as usize);
}

/// `RtlCopyMemory(Destination, Source, Length)` — memcpy (non-overlapping).
#[inline]
pub unsafe fn RtlCopyMemory(Destination: PVOID, Source: *const core::ffi::c_void, Length: SIZE_T) {
    core::ptr::copy_nonoverlapping(Source as *const u8, Destination as *mut u8, Length as usize);
}

/// `InitializeObjectAttributes(p, n, a, r, s)` (xdk macro).
#[inline]
pub unsafe fn InitializeObjectAttributes(
    p: POBJECT_ATTRIBUTES,
    n: PUNICODE_STRING,
    a: ULONG,
    r: HANDLE,
    s: PVOID,
) {
    (*p).Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
    (*p).RootDirectory = r;
    (*p).Attributes = a;
    (*p).ObjectName = n;
    (*p).SecurityDescriptor = s;
    (*p).SecurityQualityOfService = core::ptr::null_mut();
}

/* ---- 3d. console pseudo-handle constants (include/console.h) --------- */
pub const HANDLE_DETACHED_PROCESS: HANDLE = -1isize as HANDLE;
pub const HANDLE_CREATE_NEW_CONSOLE: HANDLE = -2isize as HANDLE;
pub const HANDLE_CREATE_NO_WINDOW: HANDLE = -3isize as HANDLE;

/* ---- 3e. C macros proc.c uses (inlined) ------------------------------ */

/// `IsConsoleHandle(h)` == `(((ULONG_PTR)(h) & 0x10000003) == 0x3)`
#[inline]
pub fn IsConsoleHandle(h: HANDLE) -> bool {
    (h as usize & 0x1000_0003) == 0x3
}

/// `AddToHandle(x,y)` == `(x) | (y)` — returns the new handle (C mutates in place).
#[inline]
pub fn AddToHandle(x: HANDLE, y: usize) -> HANDLE {
    ((x as usize) | y) as HANDLE
}

/// `RemoveFromHandle(x,y)` == `(x) & ~(y)`.
#[inline]
pub fn RemoveFromHandle(x: HANDLE, y: usize) -> HANDLE {
    ((x as usize) & !y) as HANDLE
}

/// `MAKELONG(lo,hi)` — args are WORDs (u16) per the wingdi macro.
#[inline]
pub fn MAKELONG(lo: u16, hi: u16) -> DWORD {
    ((lo as u32) | ((hi as u32) << 16)) as DWORD
}

/// `HandleToUlong(h)` — truncate pointer-width handle to 32-bit.
#[inline]
pub fn HandleToUlong(h: HANDLE) -> ULONG {
    h as usize as ULONG
}

/// `UlongToHandle(u)`
#[inline]
pub fn UlongToHandle(u: ULONG) -> HANDLE {
    u as usize as HANDLE
}

/// `InterlockedCompareExchangePointer(Dest, Exchange, Comparand)`
#[inline]
pub unsafe fn InterlockedCompareExchangePointer(
    dest: *mut PVOID,
    exchange: PVOID,
    comparand: PVOID,
) -> PVOID {
    use core::sync::atomic::{AtomicPtr, Ordering};
    let a = &*(dest as *const AtomicPtr<core::ffi::c_void>);
    match a.compare_exchange(
        comparand as *mut core::ffi::c_void,
        exchange as *mut core::ffi::c_void,
        Ordering::SeqCst,
        Ordering::SeqCst,
    ) {
        Ok(v) | Err(v) => v as PVOID,
    }
}
