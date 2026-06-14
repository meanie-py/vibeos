//! Minimal Win32 + msvcrt FFI surface used by the CRT-startup floor.
//!
//! Every function here is *imported* (resolved by the consumer's import
//! libs — kernel32 for Win32, msvcrt for the libc/CRT helpers), exactly
//! as the C `msvcrtex` referenced them. The PE/IMAGE structs mirror
//! `<winnt.h>` for the fields the startup code reads.
#![allow(non_camel_case_types, non_snake_case, dead_code)]

use core::ffi::c_void;

// ---- scalar aliases (mirror windef.h / basetsd.h) -------------------
pub type BYTE = u8;
pub type WORD = u16;
pub type DWORD = u32;
pub type LONG = i32;
pub type ULONG = u32;
pub type ULONGLONG = u64;
pub type BOOL = i32;
pub type WINBOOL = i32;
pub type HANDLE = *mut c_void;
pub type HMODULE = *mut c_void;
pub type HINSTANCE = *mut c_void;
pub type LPVOID = *mut c_void;
pub type PVOID = *mut c_void;
pub type PBYTE = *mut u8;
// pointer-width ints
pub type UINT_PTR = usize;
pub type ULONG_PTR = usize;
pub type DWORD_PTR = usize;

pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;

// DllMain / TLS-callback reasons.
pub const DLL_PROCESS_DETACH: DWORD = 0;
pub const DLL_PROCESS_ATTACH: DWORD = 1;
pub const DLL_THREAD_ATTACH: DWORD = 2;
pub const DLL_THREAD_DETACH: DWORD = 3;

// __set_app_type args.
pub const _UNKNOWN_APP: i32 = 0;
pub const _CONSOLE_APP: i32 = 1;
pub const _GUI_APP: i32 = 2;

pub const STARTF_USESHOWWINDOW: DWORD = 0x00000001;
pub const SW_SHOWDEFAULT: DWORD = 10;

// VirtualProtect flags.
pub const PAGE_READWRITE: DWORD = 0x04;
pub const PAGE_EXECUTE_READWRITE: DWORD = 0x40;

pub const EXCEPTION_NONCONTINUABLE: DWORD = 0x1;
pub const EXCEPTION_CONTINUE_SEARCH: LONG = 0;
pub const EXCEPTION_CONTINUE_EXECUTION: LONG = -1;
pub const EXCEPTION_EXECUTE_HANDLER: LONG = 1;
pub const STATUS_STACK_BUFFER_OVERRUN: DWORD = 0xC0000409;
pub const ERROR_SUCCESS: DWORD = 0;

// ---- small structs --------------------------------------------------
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILETIME {
    pub dwLowDateTime: DWORD,
    pub dwHighDateTime: DWORD,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LARGE_INTEGER {
    pub QuadPart: i64,
}
impl LARGE_INTEGER {
    #[inline]
    pub fn low(&self) -> u32 {
        self.QuadPart as u32
    }
    #[inline]
    pub fn high(&self) -> u32 {
        (self.QuadPart >> 32) as u32
    }
}

/// STARTUPINFOA — the startup code reads only `dwFlags` + `wShowWindow`,
/// but the full layout is needed so `GetStartupInfoA` writes the right
/// number of bytes.
#[repr(C)]
pub struct STARTUPINFOA {
    pub cb: DWORD,
    pub lpReserved: *mut u8,
    pub lpDesktop: *mut u8,
    pub lpTitle: *mut u8,
    pub dwX: DWORD,
    pub dwY: DWORD,
    pub dwXSize: DWORD,
    pub dwYSize: DWORD,
    pub dwXCountChars: DWORD,
    pub dwYCountChars: DWORD,
    pub dwFillAttribute: DWORD,
    pub dwFlags: DWORD,
    pub wShowWindow: WORD,
    pub cbReserved2: WORD,
    pub lpReserved2: *mut u8,
    pub hStdInput: HANDLE,
    pub hStdOutput: HANDLE,
    pub hStdError: HANDLE,
}

#[repr(C)]
pub struct MEMORY_BASIC_INFORMATION {
    pub BaseAddress: PVOID,
    pub AllocationBase: PVOID,
    pub AllocationProtect: DWORD,
    #[cfg(target_pointer_width = "64")]
    pub PartitionId: WORD,
    pub RegionSize: usize,
    pub State: DWORD,
    pub Protect: DWORD,
    pub Type: DWORD,
}

// CRITICAL_SECTION — opaque; only ever passed by pointer to the Win32
// APIs. Sized/aligned to the real layout (RTL_CRITICAL_SECTION).
#[repr(C)]
pub struct CRITICAL_SECTION {
    DebugInfo: PVOID,
    LockCount: LONG,
    RecursionCount: LONG,
    OwningThread: HANDLE,
    LockSemaphore: HANDLE,
    SpinCount: ULONG_PTR,
}
impl CRITICAL_SECTION {
    pub const fn zeroed() -> Self {
        CRITICAL_SECTION {
            DebugInfo: core::ptr::null_mut(),
            LockCount: 0,
            RecursionCount: 0,
            OwningThread: core::ptr::null_mut(),
            LockSemaphore: core::ptr::null_mut(),
            SpinCount: 0,
        }
    }
}

// ---- PE / IMAGE structures (winnt.h) --------------------------------
pub const IMAGE_DOS_SIGNATURE: WORD = 0x5A4D; // 'MZ'
pub const IMAGE_NT_SIGNATURE: DWORD = 0x0000_4550; // 'PE\0\0'
pub const IMAGE_NT_OPTIONAL_HDR32_MAGIC: WORD = 0x10b;
pub const IMAGE_NT_OPTIONAL_HDR64_MAGIC: WORD = 0x20b;
pub const IMAGE_SIZEOF_SHORT_NAME: usize = 8;
pub const IMAGE_DIRECTORY_ENTRY_IMPORT: usize = 1;
pub const IMAGE_DIRECTORY_ENTRY_COM_DESCRIPTOR: usize = 14;
pub const IMAGE_NUMBEROF_DIRECTORY_ENTRIES: usize = 16;
pub const IMAGE_SCN_MEM_EXECUTE: DWORD = 0x2000_0000;
pub const IMAGE_SCN_MEM_WRITE: DWORD = 0x8000_0000;

#[cfg(target_arch = "x86")]
pub const IMAGE_NT_OPTIONAL_HDR_MAGIC: WORD = IMAGE_NT_OPTIONAL_HDR32_MAGIC;
#[cfg(not(target_arch = "x86"))]
pub const IMAGE_NT_OPTIONAL_HDR_MAGIC: WORD = IMAGE_NT_OPTIONAL_HDR64_MAGIC;

#[repr(C)]
pub struct IMAGE_DOS_HEADER {
    pub e_magic: WORD,
    pub e_cblp: WORD,
    pub e_cp: WORD,
    pub e_crlc: WORD,
    pub e_cparhdr: WORD,
    pub e_minalloc: WORD,
    pub e_maxalloc: WORD,
    pub e_ss: WORD,
    pub e_sp: WORD,
    pub e_csum: WORD,
    pub e_ip: WORD,
    pub e_cs: WORD,
    pub e_lfarlc: WORD,
    pub e_ovno: WORD,
    pub e_res: [WORD; 4],
    pub e_oemid: WORD,
    pub e_oeminfo: WORD,
    pub e_res2: [WORD; 10],
    pub e_lfanew: LONG,
}

#[repr(C)]
pub struct IMAGE_FILE_HEADER {
    pub Machine: WORD,
    pub NumberOfSections: WORD,
    pub TimeDateStamp: DWORD,
    pub PointerToSymbolTable: DWORD,
    pub NumberOfSymbols: DWORD,
    pub SizeOfOptionalHeader: WORD,
    pub Characteristics: WORD,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct IMAGE_DATA_DIRECTORY {
    pub VirtualAddress: DWORD,
    pub Size: DWORD,
}

#[repr(C)]
pub struct IMAGE_OPTIONAL_HEADER32 {
    pub Magic: WORD,
    pub MajorLinkerVersion: BYTE,
    pub MinorLinkerVersion: BYTE,
    pub SizeOfCode: DWORD,
    pub SizeOfInitializedData: DWORD,
    pub SizeOfUninitializedData: DWORD,
    pub AddressOfEntryPoint: DWORD,
    pub BaseOfCode: DWORD,
    pub BaseOfData: DWORD,
    pub ImageBase: DWORD,
    pub SectionAlignment: DWORD,
    pub FileAlignment: DWORD,
    pub MajorOperatingSystemVersion: WORD,
    pub MinorOperatingSystemVersion: WORD,
    pub MajorImageVersion: WORD,
    pub MinorImageVersion: WORD,
    pub MajorSubsystemVersion: WORD,
    pub MinorSubsystemVersion: WORD,
    pub Win32VersionValue: DWORD,
    pub SizeOfImage: DWORD,
    pub SizeOfHeaders: DWORD,
    pub CheckSum: DWORD,
    pub Subsystem: WORD,
    pub DllCharacteristics: WORD,
    pub SizeOfStackReserve: DWORD,
    pub SizeOfStackCommit: DWORD,
    pub SizeOfHeapReserve: DWORD,
    pub SizeOfHeapCommit: DWORD,
    pub LoaderFlags: DWORD,
    pub NumberOfRvaAndSizes: DWORD,
    pub DataDirectory: [IMAGE_DATA_DIRECTORY; IMAGE_NUMBEROF_DIRECTORY_ENTRIES],
}

#[repr(C)]
pub struct IMAGE_OPTIONAL_HEADER64 {
    pub Magic: WORD,
    pub MajorLinkerVersion: BYTE,
    pub MinorLinkerVersion: BYTE,
    pub SizeOfCode: DWORD,
    pub SizeOfInitializedData: DWORD,
    pub SizeOfUninitializedData: DWORD,
    pub AddressOfEntryPoint: DWORD,
    pub BaseOfCode: DWORD,
    pub ImageBase: ULONGLONG,
    pub SectionAlignment: DWORD,
    pub FileAlignment: DWORD,
    pub MajorOperatingSystemVersion: WORD,
    pub MinorOperatingSystemVersion: WORD,
    pub MajorImageVersion: WORD,
    pub MinorImageVersion: WORD,
    pub MajorSubsystemVersion: WORD,
    pub MinorSubsystemVersion: WORD,
    pub Win32VersionValue: DWORD,
    pub SizeOfImage: DWORD,
    pub SizeOfHeaders: DWORD,
    pub CheckSum: DWORD,
    pub Subsystem: WORD,
    pub DllCharacteristics: WORD,
    pub SizeOfStackReserve: ULONGLONG,
    pub SizeOfStackCommit: ULONGLONG,
    pub SizeOfHeapReserve: ULONGLONG,
    pub SizeOfHeapCommit: ULONGLONG,
    pub LoaderFlags: DWORD,
    pub NumberOfRvaAndSizes: DWORD,
    pub DataDirectory: [IMAGE_DATA_DIRECTORY; IMAGE_NUMBEROF_DIRECTORY_ENTRIES],
}

#[cfg(target_arch = "x86")]
pub type IMAGE_OPTIONAL_HEADER = IMAGE_OPTIONAL_HEADER32;
#[cfg(not(target_arch = "x86"))]
pub type IMAGE_OPTIONAL_HEADER = IMAGE_OPTIONAL_HEADER64;

#[repr(C)]
pub struct IMAGE_NT_HEADERS {
    pub Signature: DWORD,
    pub FileHeader: IMAGE_FILE_HEADER,
    pub OptionalHeader: IMAGE_OPTIONAL_HEADER,
}
// For the runtime Magic-dispatch in check_managed_app.
#[repr(C)]
pub struct IMAGE_NT_HEADERS32 {
    pub Signature: DWORD,
    pub FileHeader: IMAGE_FILE_HEADER,
    pub OptionalHeader: IMAGE_OPTIONAL_HEADER32,
}
#[repr(C)]
pub struct IMAGE_NT_HEADERS64 {
    pub Signature: DWORD,
    pub FileHeader: IMAGE_FILE_HEADER,
    pub OptionalHeader: IMAGE_OPTIONAL_HEADER64,
}

#[repr(C)]
pub struct IMAGE_SECTION_HEADER {
    pub Name: [BYTE; IMAGE_SIZEOF_SHORT_NAME],
    /// union { PhysicalAddress; VirtualSize } Misc
    pub VirtualSize: DWORD,
    pub VirtualAddress: DWORD,
    pub SizeOfRawData: DWORD,
    pub PointerToRawData: DWORD,
    pub PointerToRelocations: DWORD,
    pub PointerToLinenumbers: DWORD,
    pub NumberOfRelocations: WORD,
    pub NumberOfLinenumbers: WORD,
    pub Characteristics: DWORD,
}

#[repr(C)]
pub struct IMAGE_IMPORT_DESCRIPTOR {
    /// union { Characteristics; OriginalFirstThunk }
    pub OriginalFirstThunk: DWORD,
    pub TimeDateStamp: DWORD,
    pub ForwarderChain: DWORD,
    pub Name: DWORD,
    pub FirstThunk: DWORD,
}

#[repr(C)]
pub struct IMAGE_TLS_DIRECTORY {
    pub StartAddressOfRawData: ULONG_PTR,
    pub EndAddressOfRawData: ULONG_PTR,
    pub AddressOfIndex: ULONG_PTR,
    pub AddressOfCallBacks: ULONG_PTR,
    pub SizeOfZeroFill: DWORD,
    pub Characteristics: DWORD,
}
// `_tls_used` must be linkable as a static; raw pointers aren't Sync, so
// wrap and assert it.
unsafe impl Sync for IMAGE_TLS_DIRECTORY {}

/// `IMAGE_FIRST_SECTION(pNTHeader)` — section table follows the optional
/// header (whose size is variable, hence `SizeOfOptionalHeader`).
#[inline]
pub unsafe fn image_first_section(nt: *const IMAGE_NT_HEADERS) -> *const IMAGE_SECTION_HEADER {
    // offsetof(IMAGE_NT_HEADERS, OptionalHeader) + SizeOfOptionalHeader
    let opt = core::ptr::addr_of!((*nt).OptionalHeader) as usize;
    (opt + (*nt).FileHeader.SizeOfOptionalHeader as usize) as *const IMAGE_SECTION_HEADER
}

pub type PIMAGE_TLS_CALLBACK = Option<unsafe extern "system" fn(HANDLE, DWORD, LPVOID)>;
pub type LPTOP_LEVEL_EXCEPTION_FILTER =
    Option<unsafe extern "system" fn(*mut EXCEPTION_POINTERS) -> LONG>;
pub type FARPROC = Option<unsafe extern "system" fn() -> isize>;

// ---- exception records (subset) -------------------------------------
pub const EXCEPTION_MAXIMUM_PARAMETERS: usize = 15;

#[repr(C)]
pub struct EXCEPTION_RECORD {
    pub ExceptionCode: DWORD,
    pub ExceptionFlags: DWORD,
    pub ExceptionRecord: *mut EXCEPTION_RECORD,
    pub ExceptionAddress: PVOID,
    pub NumberParameters: DWORD,
    pub ExceptionInformation: [ULONG_PTR; EXCEPTION_MAXIMUM_PARAMETERS],
}

#[repr(C)]
pub struct EXCEPTION_POINTERS {
    pub ExceptionRecord: *mut EXCEPTION_RECORD,
    pub ContextRecord: *mut c_void,
}

// Win32 exception codes used by _gnu_exception_handler.
pub const EXCEPTION_ACCESS_VIOLATION: DWORD = 0xC0000005;
pub const EXCEPTION_ILLEGAL_INSTRUCTION: DWORD = 0xC000001D;
pub const EXCEPTION_PRIV_INSTRUCTION: DWORD = 0xC0000096;
pub const EXCEPTION_FLT_INVALID_OPERATION: DWORD = 0xC0000090;
pub const EXCEPTION_FLT_DIVIDE_BY_ZERO: DWORD = 0xC000008E;
pub const EXCEPTION_FLT_DENORMAL_OPERAND: DWORD = 0xC000008D;
pub const EXCEPTION_FLT_OVERFLOW: DWORD = 0xC0000091;
pub const EXCEPTION_FLT_UNDERFLOW: DWORD = 0xC0000093;
pub const EXCEPTION_FLT_INEXACT_RESULT: DWORD = 0xC000008F;
pub const EXCEPTION_INT_DIVIDE_BY_ZERO: DWORD = 0xC0000094;
pub const EXCEPTION_DATATYPE_MISALIGNMENT: DWORD = 0x80000002;
pub const EXCEPTION_ARRAY_BOUNDS_EXCEEDED: DWORD = 0xC000008C;
pub const EXCEPTION_FLT_STACK_CHECK: DWORD = 0xC0000092;
pub const EXCEPTION_INT_OVERFLOW: DWORD = 0xC0000095;
pub const EXCEPTION_INVALID_HANDLE: DWORD = 0xC0000008;

// signal() — SIG_* tokens are function-pointer sentinels.
pub const SIGINT: i32 = 2;
pub const SIGILL: i32 = 4;
pub const SIGFPE: i32 = 8;
pub const SIGSEGV: i32 = 11;
pub type SigHandler = Option<unsafe extern "C" fn(i32)>;
// SIG_DFL = 0, SIG_IGN = 1 as function pointers.
#[inline]
pub fn sig_dfl() -> SigHandler {
    unsafe { core::mem::transmute::<usize, SigHandler>(0) }
}
#[inline]
pub fn sig_ign() -> SigHandler {
    unsafe { core::mem::transmute::<usize, SigHandler>(1) }
}

// ---- TEB access (fiber id = NT_TIB.StackBase) ------------------------
/// Returns `NtCurrentTeb()->NtTib.StackBase` — used as the per-fiber
/// startup-lock token (matches the C `((PNT_TIB)NtCurrentTeb())->StackBase`).
#[inline(always)]
pub unsafe fn current_fiber_id() -> PVOID {
    #[cfg(target_arch = "x86")]
    {
        let teb: usize;
        core::arch::asm!("mov {}, fs:[0x18]", out(reg) teb, options(nostack, pure, readonly));
        *((teb + 0x04) as *const PVOID) // NT_TIB.StackBase
    }
    #[cfg(target_arch = "x86_64")]
    {
        let teb: usize;
        core::arch::asm!("mov {}, gs:[0x30]", out(reg) teb, options(nostack, pure, readonly));
        *((teb + 0x08) as *const PVOID) // NT_TIB.StackBase
    }
    #[cfg(target_arch = "aarch64")]
    {
        let teb: usize;
        core::arch::asm!("mov {}, x18", out(reg) teb, options(nostack, pure, readonly));
        *((teb + 0x08) as *const PVOID) // NT_TIB.StackBase
    }
}

// ---- kernel32 imports -----------------------------------------------
#[link(name = "kernel32")]
extern "system" {
    pub fn Sleep(dwMilliseconds: DWORD);
    pub fn GetStartupInfoA(lpStartupInfo: *mut STARTUPINFOA);
    pub fn SetUnhandledExceptionFilter(
        lpTopLevelExceptionFilter: LPTOP_LEVEL_EXCEPTION_FILTER,
    ) -> LPTOP_LEVEL_EXCEPTION_FILTER;
    pub fn UnhandledExceptionFilter(ExceptionInfo: *mut EXCEPTION_POINTERS) -> LONG;
    pub fn GetSystemTimeAsFileTime(lpSystemTimeAsFileTime: *mut FILETIME);
    pub fn GetCurrentProcessId() -> DWORD;
    pub fn GetCurrentThreadId() -> DWORD;
    pub fn GetTickCount() -> DWORD;
    pub fn QueryPerformanceCounter(lpPerformanceCount: *mut LARGE_INTEGER) -> BOOL;
    pub fn GetCurrentProcess() -> HANDLE;
    pub fn TerminateProcess(hProcess: HANDLE, uExitCode: u32) -> BOOL;
    pub fn VirtualQuery(
        lpAddress: *const c_void,
        lpBuffer: *mut MEMORY_BASIC_INFORMATION,
        dwLength: usize,
    ) -> usize;
    pub fn VirtualProtect(
        lpAddress: *mut c_void,
        dwSize: usize,
        flNewProtect: DWORD,
        lpflOldProtect: *mut DWORD,
    ) -> BOOL;
    pub fn GetModuleHandleA(lpModuleName: *const u8) -> HMODULE;
    pub fn LoadLibraryA(lpLibFileName: *const u8) -> HMODULE;
    pub fn LoadLibraryW(lpLibFileName: *const u16) -> HMODULE;
    pub fn FreeLibrary(hLibModule: HMODULE) -> BOOL;
    pub fn GetProcAddress(hModule: HMODULE, lpProcName: *const u8) -> FARPROC;
    pub fn DisableThreadLibraryCalls(hLibModule: HMODULE) -> BOOL;
    pub fn InitializeCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn DeleteCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn TlsGetValue(dwTlsIndex: DWORD) -> LPVOID;
    pub fn GetLastError() -> DWORD;
}

// RtlAddFunctionTable + unwind helpers (WIN64 ehandler/gs path only).
#[cfg(target_arch = "x86_64")]
#[link(name = "kernel32")]
extern "system" {
    pub fn RtlAddFunctionTable(
        FunctionTable: *mut RUNTIME_FUNCTION,
        EntryCount: DWORD,
        BaseAddress: u64,
    ) -> BOOL;
    pub fn RtlCaptureContext(ContextRecord: *mut c_void);
    pub fn RtlLookupFunctionEntry(
        ControlPc: u64,
        ImageBase: *mut u64,
        HistoryTable: *mut c_void,
    ) -> *mut RUNTIME_FUNCTION;
    pub fn RtlVirtualUnwind(
        HandlerType: DWORD,
        ImageBase: u64,
        ControlPc: u64,
        FunctionEntry: *mut RUNTIME_FUNCTION,
        ContextRecord: *mut c_void,
        HandlerData: *mut PVOID,
        EstablisherFrame: *mut u64,
        ContextPointers: *mut c_void,
    ) -> PVOID;
}

#[cfg(target_arch = "x86_64")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RUNTIME_FUNCTION {
    pub BeginAddress: DWORD,
    pub EndAddress: DWORD,
    pub UnwindData: DWORD,
}

// ---- msvcrt / libc imports ------------------------------------------
pub type onexit_t = Option<unsafe extern "C" fn() -> i32>;
pub type Pvfv = Option<unsafe extern "C" fn()>;
pub type Pifv = Option<unsafe extern "C" fn() -> i32>;

extern "C" {
    pub fn malloc(size: usize) -> *mut c_void;
    pub fn calloc(nmemb: usize, size: usize) -> *mut c_void;
    pub fn free(ptr: *mut c_void);
    pub fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(s: *mut c_void, c: i32, n: usize) -> *mut c_void;
    pub fn strlen(s: *const u8) -> usize;
    pub fn strncmp(a: *const u8, b: *const u8, n: usize) -> i32;

    pub fn _initterm(first: *const Pvfv, last: *const Pvfv);
    pub fn __set_app_type(app_type: i32);
    pub fn _amsg_exit(ret: i32) -> !;
    pub fn __setusermatherr(f: PVOID);
    pub fn _lock(file: i32);
    pub fn _unlock(file: i32);
    pub fn __dllonexit(
        func: onexit_t,
        pbegin: *mut *mut Pvfv,
        pend: *mut *mut Pvfv,
    ) -> onexit_t;
    pub fn _fpreset();
    pub fn _cexit();
    pub fn exit(code: i32) -> !;
    pub fn abort() -> !;
    pub fn signal(sig: i32, handler: SigHandler) -> SigHandler;
    pub fn _errno() -> *mut i32;
    pub fn __lconv_init() -> i32;

    // entry-arg fetch.
    pub fn __getmainargs(
        argc: *mut i32,
        argv: *mut *mut *mut u8,
        env: *mut *mut *mut u8,
        do_wildcard: i32,
        startinfo: *mut StartupInfo,
    ) -> i32;
    pub fn __wgetmainargs(
        argc: *mut i32,
        argv: *mut *mut *mut u16,
        env: *mut *mut *mut u16,
        do_wildcard: i32,
        startinfo: *mut StartupInfo,
    ) -> i32;

}

// msvcrt dllimport DATA (_acmdln/_wcmdln command lines + __initenv/__winitenv
// env pointers). The import lib exposes only the `__imp_` IAT slots (no direct
// symbol), so reference those — each holds the ADDRESS of the real global
// (i386 carries one extra leading underscore vs amd64/arm64). `*IMP_ACMDLN`
// yields `_acmdln` (the char*), `*IMP_INITENV = envp` writes `__initenv`.
// NOTE: i686-windows-gnu PREPENDS a leading `_` to #[link_name] (unlike
// amd64/arm64, which take it verbatim), so the i386 names carry one fewer
// underscore than the actual `__imp_*` symbol the import lib provides.
extern "C" {
    #[cfg_attr(target_arch = "x86", link_name = "_imp___acmdln")]
    #[cfg_attr(not(target_arch = "x86"), link_name = "__imp__acmdln")]
    pub static IMP_ACMDLN: *mut *mut u8;
    #[cfg_attr(target_arch = "x86", link_name = "_imp___wcmdln")]
    #[cfg_attr(not(target_arch = "x86"), link_name = "__imp__wcmdln")]
    pub static IMP_WCMDLN: *mut *mut u16;
    #[cfg_attr(target_arch = "x86", link_name = "_imp____initenv")]
    #[cfg_attr(not(target_arch = "x86"), link_name = "__imp___initenv")]
    pub static IMP_INITENV: *mut *mut *mut u8;
    #[cfg_attr(target_arch = "x86", link_name = "_imp____winitenv")]
    #[cfg_attr(not(target_arch = "x86"), link_name = "__imp___winitenv")]
    pub static IMP_WINITENV: *mut *mut *mut u16;
}

/// `_startupinfo` (internal.h).
#[repr(C)]
pub struct StartupInfo {
    pub newmode: i32,
}

// program entry points the CRT must call (user-supplied).
extern "C" {
    pub fn main(argc: i32, argv: *mut *mut u8, envp: *mut *mut u8) -> i32;
    pub fn wmain(argc: i32, argv: *mut *mut u16, envp: *mut *mut u16) -> i32;
}
