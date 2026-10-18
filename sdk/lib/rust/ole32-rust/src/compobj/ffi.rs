//! Win32 / NT / CRT import surface + small helpers for the compobj.c
//! translation. Everything here is either a base typedef, a constant verified
//! against the ReactOS SDK headers, an `extern` import resolved at DLL link
//! time (kernel32/user32/advapi32/rpcrt4/ntdll/CRT), or an inline-macro
//! equivalent (Interlocked*, HRESULT_FROM_WIN32, heap_alloc, NtCurrentTeb).
//!
//! Every extern symbol assumed here is listed in MANIFEST.md.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use core::ffi::c_void;

// ---------------------------------------------------------------------------
// Base typedefs
// ---------------------------------------------------------------------------

pub type BOOL = i32;
pub type BOOLEAN = u8;
pub type BYTE = u8;
pub type WORD = u16;
pub type DWORD = u32;
pub type LONG = i32;
pub type ULONG = u32;
pub type UINT = u32;
pub type INT = i32;
pub type WCHAR = u16;
pub type CHAR = i8;
pub type HRESULT = i32;
pub type NTSTATUS = i32;
pub type LSTATUS = i32;
pub type LONGLONG = i64;
pub type ULONGLONG = u64;
pub type ULONG_PTR = usize;
pub type LONG_PTR = isize;
pub type DWORD_PTR = usize;
pub type SIZE_T = usize;
pub type HANDLE = *mut c_void;
pub type HKEY = *mut c_void;
pub type HWND = *mut c_void;
pub type HINSTANCE = *mut c_void;
pub type HMODULE = *mut c_void;
pub type HICON = *mut c_void;
pub type HCURSOR = *mut c_void;
pub type HBRUSH = *mut c_void;
pub type ATOM = u16;
pub type LPWSTR = *mut u16;
pub type LPCWSTR = *const u16;
pub type LPOLESTR = *mut u16;
pub type LPCOLESTR = *const u16;
pub type LPSTR = *mut i8;
pub type LPCSTR = *const i8;
pub type PVOID = *mut c_void;
pub type LPVOID = *mut c_void;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type LRESULT = isize;
pub type REGSAM = u32;
pub type ACCESS_MASK = u32;
pub type COINIT = u32;
pub type RPC_STATUS = i32;

pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;
pub const NULL: PVOID = core::ptr::null_mut();
pub const MAX_PATH: usize = 260;
pub const INFINITE: DWORD = 0xFFFF_FFFF;

// ---------------------------------------------------------------------------
// Wide-string literal macro (crib of advapi32-rust's `wide!`, ASCII only).
// ---------------------------------------------------------------------------

macro_rules! wide {
    ($s:literal) => {{
        const B: &[u8] = $s.as_bytes();
        const N: usize = B.len() + 1;
        const W: [u16; N] = {
            let mut a = [0u16; N];
            let mut i = 0;
            while i < B.len() {
                a[i] = B[i] as u16;
                i += 1;
            }
            a
        };
        &W
    }};
}
pub(crate) use wide;

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILETIME {
    pub dwLowDateTime: DWORD,
    pub dwHighDateTime: DWORD,
}

/// ULARGE_INTEGER — modeled as the `u.s` struct form (little-endian layout
/// identical to the C union). Used for the initialize-spy cookie.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ULARGE_INTEGER_S {
    pub LowPart: DWORD,
    pub HighPart: DWORD,
}

#[repr(C)]
pub struct UNICODE_STRING {
    pub Length: u16,
    pub MaximumLength: u16,
    pub Buffer: LPWSTR,
}

#[repr(C)]
pub struct OBJECT_ATTRIBUTES {
    pub Length: ULONG,
    pub RootDirectory: HANDLE,
    pub ObjectName: *mut UNICODE_STRING,
    pub Attributes: ULONG,
    pub SecurityDescriptor: PVOID,
    pub SecurityQualityOfService: PVOID,
}

/// RTL_CRITICAL_SECTION — identical layout on all three targets.
#[repr(C)]
pub struct CRITICAL_SECTION {
    pub DebugInfo: PVOID,
    pub LockCount: LONG,
    pub RecursionCount: LONG,
    pub OwningThread: HANDLE,
    pub LockSemaphore: HANDLE,
    pub SpinCount: ULONG_PTR,
}

/// Statically-initialized CS value (the RTL "no debug info" sentinel −1;
/// ReactOS Rtl{Enter,Leave,Delete}CriticalSection all guard on it — see
/// sdk/lib/rtl/critical.c). Replaces the wine CRITICAL_SECTION_DEBUG static
/// blocks (which only carried a debug name in Spare[0]).
pub const CRITICAL_SECTION_STATIC_INIT: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: usize::MAX as PVOID,
    LockCount: -1,
    RecursionCount: 0,
    OwningThread: core::ptr::null_mut(),
    LockSemaphore: core::ptr::null_mut(),
    SpinCount: 0,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct POINT {
    pub x: LONG,
    pub y: LONG,
}

#[repr(C)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: UINT,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: DWORD,
    pub pt: POINT,
}

pub type WNDPROC =
    Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>;

#[repr(C)]
pub struct WNDCLASSW {
    pub style: UINT,
    pub lpfnWndProc: WNDPROC,
    pub cbClsExtra: INT,
    pub cbWndExtra: INT,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,
    pub hCursor: HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: LPCWSTR,
    pub lpszClassName: LPCWSTR,
}

/// ACTCTX_SECTION_KEYED_DATA (winbase.h, post-2600 form)
#[repr(C)]
pub struct ACTCTX_SECTION_KEYED_DATA_ASSEMBLY_METADATA {
    pub lpInformation: PVOID,
    pub lpSectionBase: PVOID,
    pub ulSectionLength: ULONG,
    pub lpSectionGlobalDataBase: PVOID,
    pub ulSectionGlobalDataLength: ULONG,
}
#[repr(C)]
pub struct ACTCTX_SECTION_KEYED_DATA {
    pub cbSize: ULONG,
    pub ulDataFormatVersion: ULONG,
    pub lpData: PVOID,
    pub ulLength: ULONG,
    pub lpSectionGlobalData: PVOID,
    pub ulSectionGlobalDataLength: ULONG,
    pub lpSectionBase: PVOID,
    pub ulSectionTotalLength: ULONG,
    pub hActCtx: HANDLE,
    pub ulAssemblyRosterIndex: ULONG,
    pub ulFlags: ULONG,
    pub AssemblyMetadata: ACTCTX_SECTION_KEYED_DATA_ASSEMBLY_METADATA,
}

/// INIT_ONCE (winbase.h) — one pointer.
pub type INIT_ONCE = PVOID;
pub type PINIT_ONCE_FN =
    Option<unsafe extern "system" fn(*mut INIT_ONCE, PVOID, *mut PVOID) -> BOOL>;

// ---------------------------------------------------------------------------
// Constants (verified against ReactOS SDK headers)
// ---------------------------------------------------------------------------

// heap
pub const HEAP_ZERO_MEMORY: DWORD = 0x8;

// LoadLibraryEx
pub const LOAD_WITH_ALTERED_SEARCH_PATH: DWORD = 0x8;

// registry
pub const HKEY_CLASSES_ROOT: HKEY = 0x8000_0000u32 as i32 as isize as HKEY;
pub const KEY_READ: REGSAM = 0x2_0019;
pub const KEY_WRITE: REGSAM = 0x2_0006;
pub const KEY_WOW64_64KEY: REGSAM = 0x0100;
pub const KEY_WOW64_32KEY: REGSAM = 0x0200;
pub const MAXIMUM_ALLOWED: ACCESS_MASK = 0x0200_0000;
pub const REG_SZ: DWORD = 1;
pub const REG_EXPAND_SZ: DWORD = 2;
pub const ERROR_SUCCESS: LSTATUS = 0;
pub const ERROR_FILE_NOT_FOUND: LSTATUS = 2;
pub const ERROR_INVALID_HANDLE: LSTATUS = 6;
pub const ERROR_MORE_DATA: LSTATUS = 234;

// ntstatus
pub const STATUS_OBJECT_NAME_NOT_FOUND: NTSTATUS = 0xC000_0034u32 as NTSTATUS;
/// ntdef.h
pub const OBJ_CASE_INSENSITIVE: ULONG = 0x0000_0040;

// activation context (winnt.h)
pub const ACTIVATION_CONTEXT_SECTION_COM_SERVER_REDIRECTION: ULONG = 4;
pub const ACTIVATION_CONTEXT_SECTION_COM_INTERFACE_REDIRECTION: ULONG = 5;
pub const ACTIVATION_CONTEXT_SECTION_COM_PROGID_REDIRECTION: ULONG = 7;
pub const FIND_ACTCTX_SECTION_KEY_RETURN_HACTCTX: DWORD = 0x0000_0001;

// window messages / message pumping (winuser.h)
pub const WM_QUIT: UINT = 0x12;
pub const WM_USER: UINT = 0x400;
pub const WM_DDE_FIRST: UINT = 0x3E0;
pub const WM_DDE_LAST: UINT = WM_DDE_FIRST + 8;
pub const PM_NOREMOVE: UINT = 0x0;
pub const PM_REMOVE: UINT = 0x1;
pub const PM_NOYIELD: UINT = 0x2;
pub const QS_PAINT: UINT = 0x20;
pub const QS_SENDMESSAGE: UINT = 0x40;
pub const QS_ALLPOSTMESSAGE: UINT = 0x100;
pub const QS_POSTMESSAGE: UINT = 0x8;
pub const QS_HOTKEY: UINT = 0x80;
pub const QS_TIMER: UINT = 0x10;
pub const PM_QS_PAINT: UINT = QS_PAINT << 16;
pub const PM_QS_SENDMESSAGE: UINT = QS_SENDMESSAGE << 16;
pub const PM_QS_POSTMESSAGE: UINT = (QS_POSTMESSAGE | QS_HOTKEY | QS_TIMER) << 16;
pub const MWMO_WAITALL: DWORD = 0x1;
pub const MWMO_ALERTABLE: DWORD = 0x2;
pub const HWND_MESSAGE: HWND = -3isize as HWND;

// wait (winbase.h)
pub const WAIT_OBJECT_0: DWORD = 0;
pub const WAIT_TIMEOUT: DWORD = 258;
pub const WAIT_FAILED: DWORD = 0xFFFF_FFFF;

// rpcdce.h
pub const RPC_S_OK: RPC_STATUS = 0;
pub const RPC_S_UUID_LOCAL_ONLY: RPC_STATUS = 1824;

// ole2ver.h
pub const rmm: DWORD = 23;
pub const rup: DWORD = 824;

// DLL_* (winnt.h)
pub const DLL_PROCESS_ATTACH: DWORD = 1;
pub const DLL_THREAD_ATTACH: DWORD = 2;
pub const DLL_THREAD_DETACH: DWORD = 3;
pub const DLL_PROCESS_DETACH: DWORD = 0;

// ---------------------------------------------------------------------------
// Inline-macro equivalents
// ---------------------------------------------------------------------------

#[inline]
pub fn HRESULT_FROM_WIN32(x: DWORD) -> HRESULT {
    if x == 0 {
        0
    } else {
        ((x & 0x0000_FFFF) | 0x8007_0000) as HRESULT
    }
}

#[inline]
pub fn MAKEINTATOM(atom: ATOM) -> LPCWSTR {
    atom as usize as LPCWSTR
}

/// wine/heap.h heap_alloc / heap_alloc_zero / heap_free.
#[inline]
pub unsafe fn heap_alloc(len: usize) -> PVOID {
    HeapAlloc(GetProcessHeap(), 0, len)
}
#[inline]
pub unsafe fn heap_alloc_zero(len: usize) -> PVOID {
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len)
}
#[inline]
pub unsafe fn heap_free(mem: PVOID) -> BOOL {
    HeapFree(GetProcessHeap(), 0, mem)
}

// Interlocked* — winnt.h inline intrinsics (no exported symbol on all three
// targets) → core atomics, C return semantics.
#[inline]
pub unsafe fn InterlockedIncrement(addend: *mut LONG) -> LONG {
    (&*(addend as *const core::sync::atomic::AtomicI32))
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst)
        + 1
}
#[inline]
pub unsafe fn InterlockedDecrement(addend: *mut LONG) -> LONG {
    (&*(addend as *const core::sync::atomic::AtomicI32))
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
        - 1
}
/// Returns the PREVIOUS value (C semantics).
#[inline]
pub unsafe fn InterlockedExchangeAdd(addend: *mut LONG, value: LONG) -> LONG {
    (&*(addend as *const core::sync::atomic::AtomicI32))
        .fetch_add(value, core::sync::atomic::Ordering::SeqCst)
}
/// Returns the INITIAL value regardless of success (C semantics).
#[inline]
pub unsafe fn InterlockedCompareExchangePointer(
    dest: *mut PVOID,
    exchange: PVOID,
    comparand: PVOID,
) -> PVOID {
    let a = &*(dest as *const core::sync::atomic::AtomicPtr<c_void>);
    match a.compare_exchange(
        comparand,
        exchange,
        core::sync::atomic::Ordering::SeqCst,
        core::sync::atomic::Ordering::SeqCst,
    ) {
        Ok(v) => v,
        Err(v) => v,
    }
}

// ---------------------------------------------------------------------------
// NtCurrentTeb + TEB.ReservedForOle access.
//
// Intrinsic TEB pointer (same form as rtl-heappage/rtl-sync). ReservedForOle
// offsets are the ntkrnl-sys bindgen-PROVEN values (offset_check_i386.rs:
// 3968 == 0xF80; offset_check_{amd64,arm64}.rs: 5976 == 0x1758).
// INTEGRATION NOTE: replace with `offset_of!(TEB, ReservedForOle)` against a
// real TEB type from ntdll-sys/com-sys — scan_magic_offsets will flag these
// numbers otherwise.
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn NtCurrentTeb() -> *mut u8 {
    let teb: *mut u8;
    core::arch::asm!("mov {}, fs:[0x18]", out(reg) teb, options(nostack, pure, readonly));
    teb
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn NtCurrentTeb() -> *mut u8 {
    let teb: *mut u8;
    core::arch::asm!("mov {}, gs:[0x30]", out(reg) teb, options(nostack, pure, readonly));
    teb
}
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub unsafe fn NtCurrentTeb() -> *mut u8 {
    let teb: *mut u8;
    core::arch::asm!("mov {}, x18", out(reg) teb, options(nostack, pure, nomem));
    teb
}

#[cfg(target_arch = "x86")]
const TEB_RESERVED_FOR_OLE: usize = 0xF80; // nolint: magic-offset (proven by ntkrnl-sys offset_check_i386.rs:8374)
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
const TEB_RESERVED_FOR_OLE: usize = 0x1758; // nolint: magic-offset (proven by ntkrnl-sys offset_check_{amd64,arm64}.rs)

/// `NtCurrentTeb()->ReservedForOle` slot, as an lvalue pointer.
#[inline(always)]
pub unsafe fn teb_reserved_for_ole() -> *mut PVOID {
    NtCurrentTeb().add(TEB_RESERVED_FOR_OLE) as *mut PVOID
}

// ---------------------------------------------------------------------------
// Debug print sink for the ERR!/WARN!/FIXME! macros: formats into a fixed
// stack buffer (no alloc) and forwards to OutputDebugStringA. TRACE compiles
// out (matches the NDEBUG C build where TRACE is a no-op).
// ---------------------------------------------------------------------------

pub fn dbg_print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    struct Buf {
        b: [u8; 512],
        n: usize,
    }
    impl Write for Buf {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            for &c in s.as_bytes() {
                if self.n < self.b.len() - 1 {
                    self.b[self.n] = c;
                    self.n += 1;
                }
            }
            Ok(())
        }
    }
    let mut buf = Buf { b: [0u8; 512], n: 0 };
    let _ = buf.write_fmt(args);
    buf.b[buf.n] = 0;
    unsafe { OutputDebugStringA(buf.b.as_ptr() as *const i8) };
}

/// `debugstr_w` equivalent for the ERR!/FIXME! macros (lossy ASCII, capped).
pub struct DbgW(pub *const u16);
impl core::fmt::Display for DbgW {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.0.is_null() {
            return f.write_str("(null)");
        }
        f.write_str("\"")?;
        let mut i = 0usize;
        loop {
            let c = unsafe { *self.0.add(i) };
            if c == 0 || i >= 200 {
                break;
            }
            let ch = if (0x20..0x7f).contains(&c) { c as u8 as char } else { '?' };
            core::fmt::Write::write_char(f, ch)?;
            i += 1;
        }
        f.write_str("\"")
    }
}

// ---------------------------------------------------------------------------
// kernel32 imports
// ---------------------------------------------------------------------------

extern "system" {
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> PVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: PVOID) -> BOOL;
    pub fn GetCurrentThreadId() -> DWORD;
    pub fn GetCurrentProcessId() -> DWORD;
    pub fn GetCurrentProcess() -> HANDLE;
    pub fn GetTickCount() -> DWORD;
    pub fn GetLastError() -> DWORD;
    pub fn GetSystemTimeAsFileTime(lpSystemTimeAsFileTime: *mut FILETIME);
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn CreateEventW(
        lpEventAttributes: PVOID,
        bManualReset: BOOL,
        bInitialState: BOOL,
        lpName: LPCWSTR,
    ) -> HANDLE;
    pub fn SetEvent(hEvent: HANDLE) -> BOOL;
    pub fn ResetEvent(hEvent: HANDLE) -> BOOL;
    pub fn CreateThread(
        lpThreadAttributes: PVOID,
        dwStackSize: SIZE_T,
        lpStartAddress: unsafe extern "system" fn(PVOID) -> DWORD,
        lpParameter: PVOID,
        dwCreationFlags: DWORD,
        lpThreadId: *mut DWORD,
    ) -> HANDLE;
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
    pub fn WaitForMultipleObjects(
        nCount: DWORD,
        lpHandles: *const HANDLE,
        bWaitAll: BOOL,
        dwMilliseconds: DWORD,
    ) -> DWORD;
    pub fn WaitForMultipleObjectsEx(
        nCount: DWORD,
        lpHandles: *const HANDLE,
        bWaitAll: BOOL,
        dwMilliseconds: DWORD,
        bAlertable: BOOL,
    ) -> DWORD;
    pub fn LoadLibraryExW(lpLibFileName: LPCWSTR, hFile: HANDLE, dwFlags: DWORD) -> HMODULE;
    pub fn FreeLibrary(hLibModule: HMODULE) -> BOOL;
    pub fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> *mut c_void;
    pub fn ExpandEnvironmentStringsW(lpSrc: LPCWSTR, lpDst: LPWSTR, nSize: DWORD) -> DWORD;
    pub fn SearchPathW(
        lpPath: LPCWSTR,
        lpFileName: LPCWSTR,
        lpExtension: LPCWSTR,
        nBufferLength: DWORD,
        lpBuffer: LPWSTR,
        lpFilePart: *mut LPWSTR,
    ) -> DWORD;
    pub fn IsWow64Process(hProcess: HANDLE, Wow64Process: *mut BOOL) -> BOOL;
    pub fn lstrlenW(lpString: LPCWSTR) -> INT;
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR;
    pub fn lstrcatW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR;
    pub fn lstrcpynW(lpString1: LPWSTR, lpString2: LPCWSTR, iMaxLength: INT) -> LPWSTR;
    pub fn OutputDebugStringA(lpOutputString: LPCSTR);
    pub fn InitializeCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn DeleteCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn InitOnceExecuteOnce(
        InitOnce: *mut INIT_ONCE,
        InitFn: PINIT_ONCE_FN,
        Parameter: PVOID,
        Context: *mut PVOID,
    ) -> BOOL;
    // activation contexts
    pub fn FindActCtxSectionGuid(
        dwFlags: DWORD,
        lpExtensionGuid: *const super::com::GUID,
        ulSectionId: ULONG,
        lpGuidToFind: *const super::com::GUID,
        ReturnedData: *mut ACTCTX_SECTION_KEYED_DATA,
    ) -> BOOL;
    pub fn FindActCtxSectionStringW(
        dwFlags: DWORD,
        lpExtensionGuid: *const super::com::GUID,
        ulSectionId: ULONG,
        lpStringToFind: LPCWSTR,
        ReturnedData: *mut ACTCTX_SECTION_KEYED_DATA,
    ) -> BOOL;
    pub fn ActivateActCtx(hActCtx: HANDLE, lpCookie: *mut ULONG_PTR) -> BOOL;
    pub fn DeactivateActCtx(dwFlags: DWORD, ulCookie: ULONG_PTR) -> BOOL;
    pub fn ReleaseActCtx(hActCtx: HANDLE);
}

// ---------------------------------------------------------------------------
// user32 imports
// ---------------------------------------------------------------------------

extern "system" {
    pub fn RegisterClassW(lpWndClass: *const WNDCLASSW) -> ATOM;
    pub fn UnregisterClassW(lpClassName: LPCWSTR, hInstance: HINSTANCE) -> BOOL;
    pub fn CreateWindowExW(
        dwExStyle: DWORD,
        lpClassName: LPCWSTR,
        lpWindowName: LPCWSTR,
        dwStyle: DWORD,
        X: INT,
        Y: INT,
        nWidth: INT,
        nHeight: INT,
        hWndParent: HWND,
        hMenu: PVOID,
        hInstance: HINSTANCE,
        lpParam: PVOID,
    ) -> HWND;
    pub fn DestroyWindow(hWnd: HWND) -> BOOL;
    pub fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn PostThreadMessageW(idThread: DWORD, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    pub fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn PeekMessageW(
        lpMsg: *mut MSG,
        hWnd: HWND,
        wMsgFilterMin: UINT,
        wMsgFilterMax: UINT,
        wRemoveMsg: UINT,
    ) -> BOOL;
    pub fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT)
        -> BOOL;
    pub fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
    pub fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
    pub fn PostQuitMessage(nExitCode: INT);
    pub fn MsgWaitForMultipleObjectsEx(
        nCount: DWORD,
        pHandles: *const HANDLE,
        dwMilliseconds: DWORD,
        dwWakeMask: DWORD,
        dwFlags: DWORD,
    ) -> DWORD;
}

// ---------------------------------------------------------------------------
// advapi32 imports (registry)
// ---------------------------------------------------------------------------

extern "system" {
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS;
    pub fn RegQueryValueW(hKey: HKEY, lpSubKey: LPCWSTR, lpData: LPWSTR, lpcbData: *mut LONG)
        -> LSTATUS;
    pub fn RegQueryValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        lpReserved: *mut DWORD,
        lpType: *mut DWORD,
        lpData: *mut BYTE,
        lpcbData: *mut DWORD,
    ) -> LSTATUS;
    pub fn RegSetValueW(
        hKey: HKEY,
        lpSubKey: LPCWSTR,
        dwType: DWORD,
        lpData: LPCWSTR,
        cbData: DWORD,
    ) -> LSTATUS;
    pub fn RegDeleteKeyW(hKey: HKEY, lpSubKey: LPCWSTR) -> LSTATUS;
}

// ---------------------------------------------------------------------------
// ntdll imports
// ---------------------------------------------------------------------------

extern "system" {
    pub fn NtCreateKey(
        KeyHandle: *mut HANDLE,
        DesiredAccess: ACCESS_MASK,
        ObjectAttributes: *mut OBJECT_ATTRIBUTES,
        TitleIndex: ULONG,
        Class: *mut UNICODE_STRING,
        CreateOptions: ULONG,
        Disposition: *mut ULONG,
    ) -> NTSTATUS;
    pub fn NtOpenKey(
        KeyHandle: *mut HANDLE,
        DesiredAccess: ACCESS_MASK,
        ObjectAttributes: *mut OBJECT_ATTRIBUTES,
    ) -> NTSTATUS;
    pub fn NtClose(Handle: HANDLE) -> NTSTATUS;
    pub fn RtlInitUnicodeString(DestinationString: *mut UNICODE_STRING, SourceString: LPCWSTR);
    pub fn RtlNtStatusToDosError(Status: NTSTATUS) -> ULONG;
}

// ---------------------------------------------------------------------------
// rpcrt4 imports
// ---------------------------------------------------------------------------

extern "system" {
    pub fn UuidCreate(Uuid: *mut super::com::GUID) -> RPC_STATUS;
}

// ---------------------------------------------------------------------------
// CRT imports (cdecl)
// ---------------------------------------------------------------------------

extern "C" {
    pub fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memmove(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dst: *mut c_void, c: i32, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> i32;
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16;
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32;
    pub fn wcslen(s: *const u16) -> usize;
}

/// msvcrt `wcsicmp`/`_wcsicmp` — compobj.c uses `wcsicmp` via the ReactOS CRT.
#[inline]
pub unsafe fn wcsicmp(a: *const u16, b: *const u16) -> i32 {
    _wcsicmp(a, b)
}
