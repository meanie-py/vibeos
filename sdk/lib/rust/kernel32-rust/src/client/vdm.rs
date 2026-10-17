use ntdll_sys::nt::*;
// The crate's own functions: the definition is the declaration.
use crate::client::handle::CloseHandle;
use crate::file::create::CreateFileW;
use crate::file::rw::ReadFile;
use crate::prelude::*;
use core::ffi::c_void;

// Substrate-provided helpers (BaseSetLastNTError/SetLastError, the CSR_CREATE_API_NUMBER
// macro-fn, RtlCopyMemory/RtlZeroMemory, BINARY_TYPE_* enum, INVALID_HANDLE_VALUE,
// HandleToUlong, VdmEntry* enum). Csr*/Nt*/Rtl* and NtCurrentPeb/Process come via the
// `ntdll_sys::nt::*` glob.
use crate::k32_externs::{
    BaseSetLastNTError, SetLastError, CSR_CREATE_API_NUMBER, RtlCopyMemory, RtlZeroMemory,
    INVALID_HANDLE_VALUE, HandleToUlong,
};
use crate::k32_types::{
    BINARY_TYPE_COM, BINARY_TYPE_PIF, BINARY_TYPE_EXE, BINARY_TYPE_DOS,
    BINARY_TYPE_WOW, BINARY_TYPE_SEPARATE_WOW,
    VdmEntryUndo, VdmEntryUpdateProcess,
};
use crate::string::nls_subst::NT_SUCCESS;

// win/vdm.h enum: VdmEntryUndo=0, VdmEntryUpdateProcess=1, VdmEntryUpdateControlCHandler=2.
// Only the first two are in k32_types; the third is declared here.
const VdmEntryUpdateControlCHandler: ULONG = 2;

// Win32 APIs (stdcall) — resolve to other kernel32 modules' `#[no_mangle]` symbols
// (or the C TUs) at link time.
extern "system" {
    // SetFilePointer is WINAPI SetFilePointer(HANDLE, LONG, PLONG, DWORD) -> DWORD
    // (kernel32.spec: stdcall, @16). The distance args are 32-bit LONG, not i64.
    fn SetFilePointer( hFile: HANDLE, lDistanceToMove: LONG, lpDistanceToMoveHigh: *mut LONG, dwMoveMethod: u32, ) -> u32;
    fn GetSystemDirectoryW(lpBuffer: LPWSTR, uSize: UINT) -> UINT;
    fn GetShortPathNameW(lpszLongPath: LPCWSTR, lpszShortPath: LPWSTR, cchBuffer: DWORD) -> DWORD;
    fn lstrcmpiW(lpString1: LPCWSTR, lpString2: LPCWSTR) -> i32;
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    // Win32 exports used by BaseCheckVDM (resolve to other kernel32 modules).
    fn GetCurrentDirectoryW(nBufferLength: DWORD, lpBuffer: LPWSTR) -> DWORD;
    fn GetLastError() -> DWORD;
    fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: LPCWSTR,
        cchWideChar: i32,
        lpMultiByteStr: LPSTR,
        cbMultiByte: i32,
        lpDefaultChar: LPCSTR,
        lpUsedDefaultChar: *mut BOOL,
    ) -> i32;
}

// HeapAlloc/HeapFree are pure ntdll forwarders in kernel32.spec (no link-time
// local symbol) — route to the crate's thin Rtl* wrappers instead of an extern.
use crate::string::nls_subst::{HeapAlloc, HeapFree};

// CRT string helpers (cdecl).
extern "C" {
    fn memcmp( buf1: *const core::ffi::c_void, buf2: *const core::ffi::c_void, count: usize, ) -> i32;
    fn wcslen(_: *const u16) -> usize;
    fn wcsrchr(_: *const u16, _: u16) -> *mut u16;
    fn _wcsnicmp(s1: *const u16, s2: *const u16, n: usize) -> i32;
    fn strncmp(s1: *const i8, s2: *const i8, n: usize) -> i32;
    fn _snwprintf(buffer: *mut u16, count: usize, format: *const u16, ...) -> i32;
}

// RtlMoveMemory is `memmove` semantics — declared as the CRT memmove (cdecl), matching
// the C macro `#define RtlMoveMemory(d,s,l) memmove(d,s,l)`.
#[inline(always)]
unsafe fn RtlMoveMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    extern "C" { fn memmove(d: *mut c_void, s: *const c_void, n: usize) -> *mut c_void; }
    memmove(dst, src, len);
}

// FORCEINLINE helper macros from the C headers (IS_ALPHA/IS_PATH_SEPARATOR are local
// #defines in BaseCreateVDMEnvironment; ARRAYSIZE is a generic macro).
macro_rules! ARRAYSIZE { ($a:expr) => { ($a).len() as ULONG }; }
#[inline(always)]
fn IS_ALPHA(x: u16) -> bool {
    (x >= b'A' as u16 && x <= b'Z' as u16) || (x >= b'a' as u16 && x <= b'z' as u16)
}
#[inline(always)]
fn IS_PATH_SEPARATOR(x: u16) -> bool { x == b'\\' as u16 || x == b'/' as u16 }

// ---- file-scope preamble (translated) ----
pub const BINARY_UNKNOWN: u32 = 0;
pub const BINARY_PE_EXE32: u32 = 1;
pub const BINARY_PE_DLL32: u32 = 2;
pub const BINARY_PE_EXE64: u32 = 3;
pub const BINARY_PE_DLL64: u32 = 4;
pub const BINARY_WIN16: u32 = 5;
pub const BINARY_OS216: u32 = 6;
pub const BINARY_DOS: u32 = 7;
pub const BINARY_UNIX_EXE: u32 = 8;
pub const BINARY_UNIX_LIB: u32 = 9;

pub type ENV_NAME_TYPE = i32;
pub const EnvNameNotAPath: ENV_NAME_TYPE = 1;
pub const EnvNameSinglePath: ENV_NAME_TYPE = 2;
pub const EnvNameMultiplePath: ENV_NAME_TYPE = 3;

#[repr(C)]
pub struct _ENV_INFO {
    pub NameType: ENV_NAME_TYPE,
    pub NameLength: ULONG,
    pub Name: PWCHAR,
}
pub type ENV_INFO = _ENV_INFO;
pub type PENV_INFO = *mut _ENV_INFO;

pub static mut BasepEnvNameType: [ENV_INFO; 5] = [
    ENV_INFO { NameType: EnvNameMultiplePath, NameLength: 4, Name: w!("PATH\0").as_ptr() as PWCHAR },
    ENV_INFO { NameType: EnvNameSinglePath,   NameLength: 6, Name: w!("WINDIR\0").as_ptr() as PWCHAR },
    ENV_INFO { NameType: EnvNameSinglePath,   NameLength: 10, Name: w!("SYSTEMROOT\0").as_ptr() as PWCHAR },
    ENV_INFO { NameType: EnvNameMultiplePath, NameLength: 4, Name: w!("TEMP\0").as_ptr() as PWCHAR },
    ENV_INFO { NameType: EnvNameMultiplePath, NameLength: 3, Name: w!("TMP\0").as_ptr() as PWCHAR },
];

pub static mut BaseDotComSuffixName: _UNICODE_STRING = _UNICODE_STRING {
    Length: 8,
    MaximumLength: 10,
    Buffer: w!(".com\0").as_ptr() as PWCH,
};

pub static mut BaseDotPifSuffixName: _UNICODE_STRING = _UNICODE_STRING {
    Length: 8,
    MaximumLength: 10,
    Buffer: w!(".pif\0").as_ptr() as PWCH,
};

pub static mut BaseDotExeSuffixName: _UNICODE_STRING = _UNICODE_STRING {
    Length: 8,
    MaximumLength: 10,
    Buffer: w!(".exe\0").as_ptr() as PWCH,
};

// ---- conmsg.h console-server protocol (RegisterConsoleVDM uses these; they are
// NOT in the ntoskrnl bindgen surface, so the relevant subset is declared here,
// faithful to sdk/include/reactos/subsys/win/conmsg.h). ----
// COORD (wincon.h) — not in the ntoskrnl bindgen surface.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct COORD {
    pub X: SHORT,
    pub Y: SHORT,
}
pub const CONSRV_SERVERDLL_INDEX: ULONG = 2;
// CONSRV_API_NUMBER enum: ConsolepOpenConsole = CONSRV_FIRST_API_NUMBER (512);
// ConsolepRegisterVDM is the 49th (560).
pub const ConsolepRegisterVDM: ULONG = 560;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct _CONSOLE_REGISTERVDM {
    pub ConsoleHandle: HANDLE,
    pub RegisterFlags: ULONG,
    pub StartHardwareEvent: HANDLE,
    pub EndHardwareEvent: HANDLE,
    pub ErrorHardwareEvent: HANDLE,
    pub UnusedVar: ULONG,
    pub VideoStateLength: ULONG,
    pub VideoState: PVOID,
    pub UnusedBuffer: PVOID,
    pub UnusedBufferLength: ULONG,
    pub VDMBufferSize: COORD,
    pub VDMBuffer: PVOID,
}
pub type CONSOLE_REGISTERVDM = _CONSOLE_REGISTERVDM;
pub type PCONSOLE_REGISTERVDM = *mut _CONSOLE_REGISTERVDM;

// CONSOLE_API_MESSAGE: the conmsg.h message wrapper. The Data union in the real
// header holds every console request; here only RegisterVDMRequest is needed, with
// a trailing pad so the message is large enough for the CSR transport (the C header's
// CHECK_API_MSG_SIZE guarantees it fits in CSR_API_MESSAGE).
#[repr(C)]
#[derive(Copy, Clone)]
pub union _CONSOLE_API_MESSAGE__Data {
    pub RegisterVDMRequest: CONSOLE_REGISTERVDM,
    _pad: [u8; 256],
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _CONSOLE_API_MESSAGE {
    pub Header: PORT_MESSAGE,
    pub CsrCaptureData: PCSR_CAPTURE_BUFFER,
    pub ApiNumber: CSR_API_NUMBER,
    pub Status: NTSTATUS,
    pub Reserved: ULONG,
    pub Data: _CONSOLE_API_MESSAGE__Data,
}
pub type CONSOLE_API_MESSAGE = _CONSOLE_API_MESSAGE;
pub type PCONSOLE_API_MESSAGE = *mut _CONSOLE_API_MESSAGE;

// ---- win/vdm.h: VDM_COMMAND_INFO + VDMState flags (not in bindgen) ----
pub const VDM_FLAG_RETRY: USHORT = 0x08;
pub const VDM_INC_REENTER_COUNT: USHORT = 0x10;
pub const VDM_DEC_REENTER_COUNT: USHORT = 0x20;

#[repr(C)]
pub struct VDM_COMMAND_INFO {
    pub TaskId: ULONG,
    pub CreationFlags: ULONG,
    pub ExitCode: ULONG,
    pub CodePage: ULONG,
    pub StdIn: HANDLE,
    pub StdOut: HANDLE,
    pub StdErr: HANDLE,
    pub CmdLine: LPSTR,
    pub AppName: LPSTR,
    pub PifFile: LPSTR,
    pub CurDirectory: LPSTR,
    pub Env: LPSTR,
    pub EnvLen: ULONG,
    pub StartupInfo: STARTUPINFOA,
    pub Desktop: LPSTR,
    pub DesktopLen: ULONG,
    pub Title: LPSTR,
    pub TitleLen: ULONG,
    pub Reserved: LPVOID,
    pub ReservedLen: ULONG,
    pub CmdLen: USHORT,
    pub AppLen: USHORT,
    pub PifLen: USHORT,
    pub CurDirectoryLen: USHORT,
    pub VDMState: USHORT,
    pub CurrentDrive: USHORT,
    pub ComingFromBat: BOOLEAN,
}
pub type PVDM_COMMAND_INFO = *mut VDM_COMMAND_INFO;

#[no_mangle]
pub unsafe extern "system" fn BaseIsDosApplication(
    PathName: PUNICODE_STRING,
    Status: NTSTATUS,
) -> ULONG {
    let mut String: _UNICODE_STRING = core::mem::zeroed();

    /* Is it a .com? */
    String.Length = BaseDotComSuffixName.Length;
    String.Buffer = (*PathName).Buffer.offset(
        (((*PathName).Length as i32 - String.Length as i32) / core::mem::size_of::<WCHAR>() as i32) as isize,
    );
    if RtlEqualUnicodeString(&String, &raw const BaseDotComSuffixName, 1) != 0 {
        return BINARY_TYPE_COM;
    }

    /* Is it a .pif? */
    String.Length = BaseDotPifSuffixName.Length;
    String.Buffer = (*PathName).Buffer.offset(
        (((*PathName).Length as i32 - String.Length as i32) / core::mem::size_of::<WCHAR>() as i32) as isize,
    );
    if RtlEqualUnicodeString(&String, &raw const BaseDotPifSuffixName, 1) != 0 {
        return BINARY_TYPE_PIF;
    }

    /* Is it an exe? */
    String.Length = BaseDotExeSuffixName.Length;
    String.Buffer = (*PathName).Buffer.offset(
        (((*PathName).Length as i32 - String.Length as i32) / core::mem::size_of::<WCHAR>() as i32) as isize,
    );
    if RtlEqualUnicodeString(&String, &raw const BaseDotExeSuffixName, 1) != 0 {
        return BINARY_TYPE_EXE;
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn BaseUpdateVDMEntry(
    UpdateIndex: ULONG,
    WaitHandle: *mut HANDLE,
    IndexInfo: ULONG,
    BinaryType: ULONG,
) -> BOOL {
    let mut ApiMessage: BASE_API_MESSAGE = core::mem::zeroed();
    let UpdateVdmEntry: PBASE_UPDATE_VDM_ENTRY = &mut ApiMessage.Data.UpdateVDMEntryRequest;

    if UpdateIndex == VdmEntryUndo {
        (*UpdateVdmEntry).iTask = HandleToUlong(*WaitHandle);
        (*UpdateVdmEntry).VDMCreationState = IndexInfo as USHORT;
    } else if UpdateIndex == VdmEntryUpdateProcess {
        (*UpdateVdmEntry).VDMProcessHandle = *WaitHandle;
        (*UpdateVdmEntry).iTask = IndexInfo;
    }

    if BinaryType == BINARY_TYPE_WOW {
        (*UpdateVdmEntry).ConsoleHandle = -1isize as HANDLE;
    } else if (*UpdateVdmEntry).iTask != 0 {
        (*UpdateVdmEntry).ConsoleHandle = core::ptr::null_mut();
    } else {
        (*UpdateVdmEntry).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    }

    (*UpdateVdmEntry).EntryIndex = UpdateIndex as USHORT;
    (*UpdateVdmEntry).BinaryType = BinaryType;

    CsrClientCallServer(
        &mut ApiMessage as *mut BASE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepUpdateVDMEntry),
        core::mem::size_of::<BASE_UPDATE_VDM_ENTRY>() as ULONG,
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    if UpdateIndex == VdmEntryUpdateProcess {
        *WaitHandle = (*UpdateVdmEntry).WaitObjectForParent;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn BaseCheckForVDM(
    ProcessHandle: HANDLE,
    ExitCode: LPDWORD,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut EventBasicInfo: EVENT_BASIC_INFORMATION = core::mem::zeroed();
    let mut ApiMessage: BASE_API_MESSAGE = core::mem::zeroed();
    let GetVdmExitCode: PBASE_GET_VDM_EXIT_CODE = &mut ApiMessage.Data.GetVDMExitCodeRequest;

    /* It's VDM if the process is actually a wait handle (an event) */
    Status = NtQueryEvent(
        ProcessHandle,
        EventBasicInformation,
        &mut EventBasicInfo as *mut EVENT_BASIC_INFORMATION as PVOID,
        core::mem::size_of::<EVENT_BASIC_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return 0;
    }

    /* Setup the input parameters */
    (*GetVdmExitCode).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*GetVdmExitCode).hParent = ProcessHandle;

    /* Call CSRSS */
    Status = CsrClientCallServer(
        &mut ApiMessage as *mut BASE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepGetVDMExitCode),
        core::mem::size_of_val(&*GetVdmExitCode) as ULONG,
    );
    if !NT_SUCCESS(Status) {
        return 0;
    }

    /* Get the exit code from the reply */
    *ExitCode = (*GetVdmExitCode).ExitCode;
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn BaseGetVdmConfigInfo(
    CommandLineReserved: LPCWSTR,
    DosSeqId: ULONG,
    BinaryType: ULONG,
    CmdLineString: PUNICODE_STRING,
    VdmSize: PULONG,
) -> BOOL {
    let mut Buffer: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut CommandLine: [WCHAR; MAX_PATH as usize * 2] = [0; MAX_PATH as usize * 2];
    let Length: ULONG;

    /* Clear the buffer in case we fail */
    (*CmdLineString).Buffer = core::ptr::null_mut();

    /* Always return the same size: 16 Mb */
    *VdmSize = 0x1000000;

    /* Get the system directory */
    Length = GetSystemDirectoryW(Buffer.as_mut_ptr(), MAX_PATH as u32);
    if !(Length != 0) || (Length >= MAX_PATH as u32) {
        /* Eliminate no path or path too big */
        SetLastError(ERROR_INVALID_NAME);
        return 0;
    }

    /* Check if this is VDM with a DOS Sequence ID */
    if DosSeqId != 0 {
        /*
         * Build the VDM string for it:
         * -i%lx : Gives the DOS Sequence ID;
         * %s%c  : Nothing if DOS VDM, -w if WoW VDM, -ws if separate WoW VDM.
         */
        _snwprintf(
            CommandLine.as_mut_ptr(),
            core::mem::size_of::<[WCHAR; MAX_PATH as usize * 2]>() / core::mem::size_of::<WCHAR>(),
            w!("\"%s\\ntvdm.exe\" -i%lx %s%c\0").as_ptr(),
            Buffer.as_ptr(),
            DosSeqId,
            if BinaryType == BINARY_TYPE_DOS {
                w!(" \0").as_ptr()
            } else {
                w!("-w\0").as_ptr()
            },
            if BinaryType == BINARY_TYPE_SEPARATE_WOW {
                b's' as core::ffi::c_int
            } else {
                b' ' as core::ffi::c_int
            },
        );
    } else {
        /*
         * Build the string for it without the DOS Sequence ID:
         * %s%c  : Nothing if DOS VDM, -w if WoW VDM, -ws if separate WoW VDM.
         */
        _snwprintf(
            CommandLine.as_mut_ptr(),
            core::mem::size_of::<[WCHAR; MAX_PATH as usize * 2]>() / core::mem::size_of::<WCHAR>(),
            w!("\"%s\\ntvdm.exe\" %s%c\0").as_ptr(),
            Buffer.as_ptr(),
            if BinaryType == BINARY_TYPE_DOS {
                w!(" \0").as_ptr()
            } else {
                w!("-w\0").as_ptr()
            },
            if BinaryType == BINARY_TYPE_SEPARATE_WOW {
                b's' as core::ffi::c_int
            } else {
                b' ' as core::ffi::c_int
            },
        );
    }

    /* Create the actual string */
    return RtlCreateUnicodeString(CmdLineString, CommandLine.as_ptr()) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn BaseGetEnvNameType_U(
    Name: PWCHAR,
    NameLength: ULONG,
) -> ENV_NAME_TYPE {
    let mut EnvInfo: PENV_INFO;
    let mut NameType: ENV_NAME_TYPE;
    let mut i: ULONG;

    NameType = EnvNameNotAPath;

    i = 0;
    while i < ARRAYSIZE!((*(&raw const BasepEnvNameType))) {
        EnvInfo = &BasepEnvNameType[i as usize] as *const _ as PENV_INFO;

        if (*EnvInfo).NameLength == NameLength && _wcsnicmp((*EnvInfo).Name, Name, NameLength as usize) == 0
        {
            NameType = (*EnvInfo).NameType;
            break;
        }

        i += 1;
    }

    NameType
}

#[no_mangle]
pub unsafe extern "system" fn BaseCreateVDMEnvironment(
    lpEnvironment: PWCHAR,
    AnsiEnv: PANSI_STRING,
    UnicodeEnv: PUNICODE_STRING,
) -> BOOL {
    let mut Success: BOOL = 0;
    let mut Status: NTSTATUS;
    let mut EnvironmentSize: ULONG = 0;
    let mut RegionSize: SIZE_T;
    let mut Environment: PWCHAR = core::ptr::null_mut();
    let mut NewEnvironment: PWCHAR = core::ptr::null_mut();
    let mut NameType: ENV_NAME_TYPE;
    let mut NameLength: ULONG;
    let mut NumChars: ULONG;
    let mut Remaining: ULONG;
    let mut SourcePtr: PWCHAR;
    let mut DestPtr: PWCHAR;
    let mut StartPtr: PWCHAR;

    if AnsiEnv.is_null() || UnicodeEnv.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    // (raw-pointer `.is_null()` is valid; field accesses below deref explicitly)

    'cleanup_body: {
    if lpEnvironment.is_null() {
        Status = RtlCreateEnvironment(1, &mut Environment);
        if !NT_SUCCESS(Status) {
            break 'cleanup_body;
        }
    } else {
        Environment = lpEnvironment;
    }

    if Environment.is_null() {
        SetLastError(ERROR_BAD_ENVIRONMENT);
        break 'cleanup_body;
    }

    SourcePtr = Environment;
    while !((*SourcePtr.offset(0) == 0) && (*SourcePtr.offset(1) == 0)) {
        SourcePtr = SourcePtr.offset(1);
        EnvironmentSize += 1;
    }
    EnvironmentSize += 2;

    RegionSize = ((EnvironmentSize + MAX_PATH) as SIZE_T) * core::mem::size_of::<WCHAR>() as SIZE_T;
    Status = NtAllocateVirtualMemory(
        NtCurrentProcess(),
        &mut NewEnvironment as *mut PWCHAR as *mut *mut core::ffi::c_void,
        0,
        &mut RegionSize,
        MEM_COMMIT,
        PAGE_READWRITE,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        NewEnvironment = core::ptr::null_mut();
        break 'cleanup_body;
    }

    Remaining = MAX_PATH - 2;
    SourcePtr = Environment;
    DestPtr = NewEnvironment;

    while *SourcePtr != 0 {
        if *SourcePtr != '=' as u16 {
            StartPtr = SourcePtr;

            while *SourcePtr != 0 {
                *DestPtr = *SourcePtr;
                DestPtr = DestPtr.offset(1);
                if *SourcePtr == '=' as u16 {
                    SourcePtr = SourcePtr.offset(1);
                    break;
                }
                SourcePtr = SourcePtr.offset(1);
            }

            // C `SourcePtr - StartPtr - 1` counts WCHARs; BaseGetEnvNameType_U compares it against
                // a WCHAR NameLength and passes it to _wcsnicmp. A byte count never matched, so no
                // variable was ever typed as a path (found by scan_ptr_byte_diff).
                NameType = BaseGetEnvNameType_U(StartPtr, (SourcePtr.offset_from(StartPtr) - 1) as ULONG);
        } else {
            NameType = EnvNameNotAPath;

            if IS_ALPHA(*SourcePtr.offset(1)) && *SourcePtr.offset(2) == ':' as u16 && *SourcePtr.offset(3) == '=' as u16 {
                if wcslen(SourcePtr) as ULONG >= (8usize - 1) as ULONG /* ARRAYSIZE("=X:=Y:\\")-1 */
                    && !(IS_ALPHA(*SourcePtr.offset(4))
                        && *SourcePtr.offset(5) == ':' as u16
                        && IS_PATH_SEPARATOR(*SourcePtr.offset(6))
                        && *SourcePtr.offset(7) == 0)
                {
                    NameType = EnvNameSinglePath;

                    *DestPtr = *SourcePtr.offset(0);
                    DestPtr = DestPtr.offset(1);
                    *DestPtr = *SourcePtr.offset(1);
                    DestPtr = DestPtr.offset(1);
                    *DestPtr = *SourcePtr.offset(2);
                    DestPtr = DestPtr.offset(1);
                    *DestPtr = *SourcePtr.offset(3);
                    DestPtr = DestPtr.offset(1);
                    SourcePtr = SourcePtr.offset(4);
                }
            } else {
                /* NameType already EnvNameNotAPath */
            }
        }

        if NameType == EnvNameNotAPath {
            loop {
                *DestPtr = *SourcePtr;
                DestPtr = DestPtr.offset(1);
                if *SourcePtr == 0 {
                    SourcePtr = SourcePtr.offset(1);
                    break;
                }
                SourcePtr = SourcePtr.offset(1);
            }
        } else if NameType == EnvNameSinglePath {
            NameLength = wcslen(SourcePtr) as ULONG;
            NumChars = GetShortPathNameW(SourcePtr, DestPtr, NameLength + 1 + Remaining);
            if NumChars == 0 || NumChars > NameLength + Remaining {
                RtlCopyMemory(
                    DestPtr as *mut core::ffi::c_void,
                    SourcePtr as *const core::ffi::c_void,
                    (NameLength as SIZE_T) * core::mem::size_of::<WCHAR>() as SIZE_T,
                );
                NumChars = NameLength;
            }
            DestPtr = DestPtr.offset(NumChars as isize);
            if NumChars > NameLength {
                Remaining -= NumChars - NameLength;
            }

            SourcePtr = SourcePtr.offset(NameLength as isize);

            *DestPtr = *SourcePtr;
            DestPtr = DestPtr.offset(1);
            SourcePtr = SourcePtr.offset(1);
        } else {
            let mut Delimiter: WCHAR;

            loop {
                while *SourcePtr == ';' as u16 {
                    *DestPtr = *SourcePtr;
                    DestPtr = DestPtr.offset(1);
                    SourcePtr = SourcePtr.offset(1);
                }

                StartPtr = SourcePtr;

                while *SourcePtr != 0 && *SourcePtr != ';' as u16 {
                    SourcePtr = SourcePtr.offset(1);
                }
                Delimiter = *SourcePtr;

                // C `SourcePtr - StartPtr` counts WCHARs, and everything below uses it as a
                // count (GetShortPathNameW's buffer size, `* sizeof(WCHAR)` for the copy,
                // DestPtr's advance). Subtracting addresses counted BYTES, doubling all three:
                // an over-stated buffer, a 4x copy, and a 2x advance past the name.
                NameLength = SourcePtr.offset_from(StartPtr) as ULONG;
                if NameLength != 0 {
                    *SourcePtr = 0;

                    NumChars = GetShortPathNameW(StartPtr, DestPtr, NameLength + 1 + Remaining);
                    if NumChars == 0
                        || (if Delimiter == ';' as u16 {
                            NumChars > NameLength + Remaining
                        } else {
                            NumChars > NameLength
                        })
                    {
                        RtlCopyMemory(
                            DestPtr as *mut core::ffi::c_void,
                            StartPtr as *const core::ffi::c_void,
                            (NameLength as SIZE_T) * core::mem::size_of::<WCHAR>() as SIZE_T,
                        );
                        NumChars = NameLength;
                    }
                    DestPtr = DestPtr.offset(NumChars as isize);
                    if NumChars > NameLength {
                        Remaining -= NumChars - NameLength;
                    }

                    if Delimiter != 0 {
                        *SourcePtr = Delimiter;
                        *DestPtr = *SourcePtr;
                        DestPtr = DestPtr.offset(1);
                        SourcePtr = SourcePtr.offset(1);
                    }
                }

                if !(*SourcePtr != 0) {
                    break;
                }
            }

            *DestPtr = *SourcePtr;
            DestPtr = DestPtr.offset(1);
            SourcePtr = SourcePtr.offset(1);
        }
    }

    *DestPtr = 0;
    DestPtr = DestPtr.offset(1);

    RtlInitEmptyUnicodeString(
        UnicodeEnv,
        NewEnvironment,
        // C: (DestPtr - NewEnvironment) * sizeof(WCHAR), where the C pointer diff is in
        // WCHAR elements. Here the `as isize` diff is already in BYTES, so it equals the
        // C byte count directly (the element-count * sizeof(WCHAR)).
        (DestPtr as isize - NewEnvironment as isize) as USHORT,
    );
    (*UnicodeEnv).Length = (*UnicodeEnv).MaximumLength;

    Status = RtlUnicodeStringToAnsiString(AnsiEnv, UnicodeEnv, 1);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
    } else {
        Success = 1;
        NewEnvironment = core::ptr::null_mut();
    }
    } // end 'cleanup_body

    /* Cleanup */ {
        if lpEnvironment.is_null() && !Environment.is_null() {
            RtlDestroyEnvironment(Environment);
        }

        if !NewEnvironment.is_null() {
            RtlInitEmptyUnicodeString(UnicodeEnv, core::ptr::null_mut(), 0);
            RtlInitEmptyAnsiString(AnsiEnv, core::ptr::null_mut(), 0);

            RegionSize = 0;
            Status = NtFreeVirtualMemory(
                NtCurrentProcess(),
                &mut NewEnvironment as *mut PWCHAR as *mut *mut core::ffi::c_void,
                &mut RegionSize,
                MEM_RELEASE,
            );
            // ASSERT(NT_SUCCESS(Status));
        }
    }

    return Success;
}
#[no_mangle]
pub unsafe extern "system" fn BaseDestroyVDMEnvironment(
    AnsiEnv: PANSI_STRING,
    UnicodeEnv: PUNICODE_STRING,
) -> BOOL {
    let mut Dummy: SIZE_T = 0;

    /* Clear the ANSI buffer since Rtl creates this for us */
    if !(*AnsiEnv).Buffer.is_null() {
        RtlFreeAnsiString(AnsiEnv);
    }

    /* The Unicode buffer is build by hand, though */
    if !(*UnicodeEnv).Buffer.is_null() {
        /* So clear it through the API */
        NtFreeVirtualMemory(
            NtCurrentProcess(),
            &mut (*UnicodeEnv).Buffer as *mut PWSTR as *mut *mut core::ffi::c_void,
            &mut Dummy,
            MEM_RELEASE,
        );
    }

    /* All done */
    return 1;
}
unsafe fn InternalIsOS2OrOldWin(
    hFile: HANDLE,
    mz: *mut IMAGE_DOS_HEADER,
    ne: *mut IMAGE_OS2_HEADER,
) -> DWORD {
    let mut CurPos: DWORD;
    let mut modtab: LPWORD = core::ptr::null_mut();
    let mut nametab: LPSTR = core::ptr::null_mut();
    let mut Read: DWORD = 0;
    let mut Ret: DWORD;
    let mut i: i32;

    Ret = BINARY_OS216;
    CurPos = SetFilePointer(hFile, 0, core::ptr::null_mut(), FILE_CURRENT);

    'done: {
        'broken: {
            /* read modref table */
            if (SetFilePointer(hFile, (*mz).e_lfanew as LONG + (*ne).ne_modtab as LONG, core::ptr::null_mut(), FILE_BEGIN) == INVALID_SET_FILE_POINTER)
                || {
                    modtab = HeapAlloc(GetProcessHeap(), 0, ((*ne).ne_cmod as usize) * core::mem::size_of::<WORD>()) as LPWORD;
                    modtab.is_null()
                }
                || {
                    ReadFile(
                        hFile,
                        modtab as LPVOID,
                        (*ne).ne_cmod as DWORD * core::mem::size_of::<WORD>() as DWORD,
                        &mut Read as LPDWORD,
                        core::ptr::null_mut(),
                    ) == 0
                }
                || (Read != ((*ne).ne_cmod as DWORD) * core::mem::size_of::<WORD>() as DWORD)
            {
                break 'broken;
            }

            /* read imported names table */
            if (SetFilePointer(hFile, (*mz).e_lfanew as LONG + (*ne).ne_imptab as LONG, core::ptr::null_mut(), FILE_BEGIN) == INVALID_SET_FILE_POINTER)
                || {
                    nametab = HeapAlloc(
                        GetProcessHeap(),
                        0,
                        ((*ne).ne_enttab - (*ne).ne_imptab) as usize,
                    ) as LPSTR;
                    nametab.is_null()
                }
                || {
                    ReadFile(
                        hFile,
                        nametab as LPVOID,
                        ((*ne).ne_enttab - (*ne).ne_imptab) as DWORD,
                        &mut Read as LPDWORD,
                        core::ptr::null_mut(),
                    ) == 0
                }
                || (Read != ((*ne).ne_enttab - (*ne).ne_imptab) as DWORD)
            {
                break 'broken;
            }

            i = 0;
            while i < (*ne).ne_cmod as i32 {
                let module: LPSTR;
                module = &*nametab.add(modtab.add(i as usize).read() as usize) as *const i8 as LPSTR;
                if strncmp(
                    &*module.add(1) as *const i8,
                    b"KERNEL\0".as_ptr() as *const i8,
                    *module as usize,
                ) == 0
                {
                    /* very old windows file */
                    Ret = BINARY_WIN16;
                    break 'done;
                }
                i += 1;
            }
        } // 'broken:
        // DPRINT1("InternalIsOS2OrOldWin(): Binary file seems to be broken\n");
    } // 'done:

    HeapFree(GetProcessHeap(), 0, modtab as LPVOID);
    HeapFree(GetProcessHeap(), 0, nametab as LPVOID);
    SetFilePointer(hFile, CurPos as LONG, core::ptr::null_mut(), FILE_BEGIN);
    return Ret;
}
unsafe fn InternalGetBinaryType(hFile: HANDLE) -> DWORD {
    #[repr(C)]
    union HeaderUnion {
        elf: ElfStruct,
        macho: MachoStruct,
        mz: IMAGE_DOS_HEADER,
    }
    #[repr(C)]
    #[derive(Copy, Clone)]
    struct ElfStruct {
        magic: [u8; 4],
        ignored: [u8; 12],
        type_: u16,
    }
    #[repr(C)]
    #[derive(Copy, Clone)]
    struct MachoStruct {
        magic: u32,
        cputype: u32,
        cpusubtype: u32,
        filetype: u32,
    }

    let mut Header: HeaderUnion = core::mem::zeroed();
    let mut magic: [u8; 4] = core::mem::zeroed();
    let mut Read: DWORD = 0;

    if (SetFilePointer(hFile, 0, core::ptr::null_mut(), FILE_BEGIN) == INVALID_SET_FILE_POINTER)
        || (ReadFile(
            hFile,
            &mut Header as *mut HeaderUnion as *mut core::ffi::c_void,
            core::mem::size_of::<HeaderUnion>() as u32,
            &mut Read,
            core::ptr::null_mut(),
        ) == 0
            || (Read != core::mem::size_of::<HeaderUnion>() as u32))
    {
        return BINARY_UNKNOWN;
    }

    if memcmp(
        &Header.elf.magic as *const u8 as *const core::ffi::c_void,
        b"\x7fELF\0".as_ptr() as *const core::ffi::c_void,
        core::mem::size_of_val(&Header.elf.magic),
    ) == 0
    {
        match Header.elf.type_ {
            2 => return BINARY_UNIX_EXE,
            3 => return BINARY_UNIX_LIB,
            _ => {}
        }
        return BINARY_UNKNOWN;
    }

    if Header.macho.magic == 0xFEEDFACE || Header.macho.magic == 0xCEFAEDFE {
        match Header.macho.filetype {
            0x8 => return BINARY_UNIX_LIB,
            _ => {}
        }
        return BINARY_UNKNOWN;
    }

    if Header.mz.e_magic == IMAGE_DOS_SIGNATURE {
        if (SetFilePointer(hFile, Header.mz.e_lfanew as LONG, core::ptr::null_mut(), FILE_BEGIN)
            == INVALID_SET_FILE_POINTER)
            || (ReadFile(
                hFile,
                magic.as_mut_ptr() as *mut core::ffi::c_void,
                core::mem::size_of_val(&magic) as u32,
                &mut Read,
                core::ptr::null_mut(),
            ) == 0
                || (Read != core::mem::size_of_val(&magic) as u32))
        {
            return BINARY_DOS;
        }

        if memcmp(
            magic.as_ptr() as *const core::ffi::c_void,
            b"PE\0\0".as_ptr() as *const core::ffi::c_void,
            core::mem::size_of_val(&magic),
        ) == 0
        {
            let mut FileHeader: IMAGE_FILE_HEADER = core::mem::zeroed();
            if ReadFile(
                hFile,
                &mut FileHeader as *mut IMAGE_FILE_HEADER as *mut core::ffi::c_void,
                core::mem::size_of::<IMAGE_FILE_HEADER>() as u32,
                &mut Read,
                core::ptr::null_mut(),
            ) == 0 || (Read != core::mem::size_of::<IMAGE_FILE_HEADER>() as u32)
            {
                return BINARY_DOS;
            }

            if (FileHeader.Characteristics & IMAGE_FILE_DLL) != 0 {
                return BINARY_PE_DLL32;
            }
            return BINARY_PE_EXE32;
        }

        if memcmp(
            magic.as_ptr() as *const core::ffi::c_void,
            b"NE\0\0".as_ptr() as *const core::ffi::c_void,
            2,
        ) == 0
        {
            let mut ne: IMAGE_OS2_HEADER = core::mem::zeroed();
            if (SetFilePointer(hFile, Header.mz.e_lfanew as LONG, core::ptr::null_mut(), FILE_BEGIN)
                == 1)
                || ReadFile(
                    hFile,
                    &mut ne as *mut IMAGE_OS2_HEADER as *mut core::ffi::c_void,
                    core::mem::size_of::<IMAGE_OS2_HEADER>() as u32,
                    &mut Read,
                    core::ptr::null_mut(),
                ) == 0
                || (Read != core::mem::size_of::<IMAGE_OS2_HEADER>() as u32)
            {
                return BINARY_DOS;
            }

            match ne.ne_exetyp {
                2 => return BINARY_WIN16,
                5 => return BINARY_DOS,
                _ => return InternalIsOS2OrOldWin(hFile, &mut Header.mz, &mut ne),
            }
        }
        return BINARY_DOS;
    }
    return BINARY_UNKNOWN;
}

const INVALID_SET_FILE_POINTER: u32 = 0xFFFFFFFF;
// BINARY_* are defined once at file scope (top `pub const`); the duplicate local
// block was removed to fix E0428.
const IMAGE_DOS_SIGNATURE: u16 = 0x5A4D;
const IMAGE_FILE_DLL: u16 = 0x2000;

#[no_mangle]
pub unsafe extern "system" fn GetBinaryTypeW(
    lpApplicationName: LPCWSTR,
    lpBinaryType: LPDWORD,
) -> BOOL {
    let hFile: HANDLE;
    let BinType: DWORD;

    if lpApplicationName.is_null() || lpBinaryType.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    hFile = CreateFileW(
        lpApplicationName,
        GENERIC_READ,
        FILE_SHARE_READ,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        core::ptr::null_mut(),
    );
    if hFile == INVALID_HANDLE_VALUE {
        return 0;
    }

    BinType = InternalGetBinaryType(hFile);
    CloseHandle(hFile);

    match BinType {
        0 => {
            // BINARY_UNKNOWN
            let dot: *mut u16;

            /*
             * guess from filename
             */
            dot = wcsrchr(lpApplicationName, 0x2E_u16);
            if dot.is_null() {
                return 0;
            }
            if lstrcmpiW(dot, w!(".COM\0").as_ptr() as LPCWSTR) == 0 {
                *lpBinaryType = SCS_DOS_BINARY;
                return 1;
            }
            if lstrcmpiW(dot, w!(".PIF\0").as_ptr() as LPCWSTR) == 0 {
                *lpBinaryType = SCS_PIF_BINARY;
                return 1;
            }
            return 0;
        }
        1 | 2 => {
            // BINARY_PE_EXE32 | BINARY_PE_DLL32
            *lpBinaryType = SCS_32BIT_BINARY;
            return 1;
        }
        3 | 4 => {
            // BINARY_PE_EXE64 | BINARY_PE_DLL64
            *lpBinaryType = SCS_64BIT_BINARY;
            return 1;
        }
        5 => {
            // BINARY_WIN16
            *lpBinaryType = SCS_WOW_BINARY;
            return 1;
        }
        6 => {
            // BINARY_OS216
            *lpBinaryType = SCS_OS216_BINARY;
            return 1;
        }
        7 => {
            // BINARY_DOS
            *lpBinaryType = SCS_DOS_BINARY;
            return 1;
        }
        8 | 9 => {
            // BINARY_UNIX_EXE | BINARY_UNIX_LIB
            return 0;
        }
        _ => {
            // DPRINT1("Invalid binary type %lu returned!\n", BinType);
            return 0;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetBinaryTypeA(
    lpApplicationName: LPCSTR,
    lpBinaryType: LPDWORD,
) -> BOOL {
    let mut ApplicationNameString: ANSI_STRING = core::mem::zeroed();
    let mut ApplicationNameW: _UNICODE_STRING = core::mem::zeroed();
    let mut StringAllocated: BOOL = 0;
    let mut Result: BOOL;
    let Status: NTSTATUS;

    RtlInitAnsiString(&mut ApplicationNameString, lpApplicationName);

    if (ApplicationNameString.Length as u32) * core::mem::size_of::<WCHAR>() as u32
        >= (*NtCurrentTeb()).StaticUnicodeString.MaximumLength as u32
    {
        StringAllocated = 1;
        Status = RtlAnsiStringToUnicodeString(
            &mut ApplicationNameW,
            &mut ApplicationNameString,
            1,
        );
    } else {
        Status = RtlAnsiStringToUnicodeString(
            &mut (*NtCurrentTeb()).StaticUnicodeString,
            &mut ApplicationNameString,
            0,
        );
    }

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    if StringAllocated != 0 {
        Result = GetBinaryTypeW(ApplicationNameW.Buffer, lpBinaryType);
        RtlFreeUnicodeString(&mut ApplicationNameW);
    } else {
        Result = GetBinaryTypeW((*NtCurrentTeb()).StaticUnicodeString.Buffer, lpBinaryType);
    }

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn CmdBatNotification(
    Unknown_: DWORD,
) -> BOOL {
    STUB!();
    return 0i32;
}

#[no_mangle]
pub unsafe extern "system" fn ExitVDM(IsWow: BOOL, iWowTask: ULONG) {
    let mut ApiMessage: BASE_API_MESSAGE = core::mem::zeroed();
    let ExitVdm: PBASE_EXIT_VDM = &mut ApiMessage.Data.ExitVDMRequest;

    /* Setup the input parameters */
    (*ExitVdm).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*ExitVdm).iWowTask = if IsWow != 0 { iWowTask } else { 0 };
    (*ExitVdm).WaitObjectForVDM = core::ptr::null_mut();

    /* Call CSRSS */
    CsrClientCallServer(
        &mut ApiMessage as *mut BASE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepExitVDM),
        core::mem::size_of::<BASE_EXIT_VDM>() as ULONG,
    );

    /* Close the returned wait object handle, if any */
    if NT_SUCCESS(ApiMessage.Status) && !(*ExitVdm).WaitObjectForVDM.is_null() {
        CloseHandle((*ExitVdm).WaitObjectForVDM);
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetNextVDMCommand(
    CommandData: *mut VDM_COMMAND_INFO,
) -> BOOL {
    let mut Success: BOOL = 0;
    let mut Status: NTSTATUS;
    let mut ApiMessage: BASE_API_MESSAGE = core::mem::zeroed();
    let GetNextVdmCommand: PBASE_GET_NEXT_VDM_COMMAND =
        &mut (*core::ptr::addr_of_mut!(ApiMessage)).Data.GetNextVDMCommandRequest;
    let IsFirstVdm: PBASE_IS_FIRST_VDM =
        &mut (*core::ptr::addr_of_mut!(ApiMessage)).Data.IsFirstVDMRequest;
    let SetReenterCount: PBASE_SET_REENTER_COUNT =
        &mut (*core::ptr::addr_of_mut!(ApiMessage)).Data.SetReenterCountRequest;
    let mut CaptureBuffer: PCSR_CAPTURE_BUFFER = core::ptr::null_mut();
    let mut NumStrings: ULONG = 0;

    if CommandData.is_null() {
        CsrClientCallServer(
            &mut ApiMessage as *mut BASE_API_MESSAGE as PCSR_API_MESSAGE,
            core::ptr::null_mut(),
            CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepIsFirstVDM),
            core::mem::size_of::<BASE_IS_FIRST_VDM>() as ULONG,
        );
        if !NT_SUCCESS(ApiMessage.Status) {
            BaseSetLastNTError(ApiMessage.Status);
            return 0;
        }
        return (*IsFirstVdm).FirstVDM as BOOL;
    }

    if ((*CommandData).VDMState == VDM_INC_REENTER_COUNT)
        || ((*CommandData).VDMState == VDM_DEC_REENTER_COUNT)
    {
        (*SetReenterCount).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
        (*SetReenterCount).fIncDec = (*CommandData).VDMState as ULONG;

        CsrClientCallServer(
            &mut ApiMessage as *mut BASE_API_MESSAGE as PCSR_API_MESSAGE,
            core::ptr::null_mut(),
            CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepSetReenterCount),
            core::mem::size_of::<BASE_SET_REENTER_COUNT>() as ULONG,
        );
        if !NT_SUCCESS(ApiMessage.Status) {
            BaseSetLastNTError(ApiMessage.Status);
            return 0;
        }
        return 1;
    }

    'cleanup: {
    RtlZeroMemory(
        GetNextVdmCommand as *mut core::ffi::c_void,
        core::mem::size_of::<BASE_GET_NEXT_VDM_COMMAND>() as SIZE_T,
    );

    (*GetNextVdmCommand).iTask = (*CommandData).TaskId;
    (*GetNextVdmCommand).ConsoleHandle =
        (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*GetNextVdmCommand).CmdLen = (*CommandData).CmdLen;
    (*GetNextVdmCommand).AppLen = (*CommandData).AppLen;
    (*GetNextVdmCommand).PifLen = (*CommandData).PifLen;
    (*GetNextVdmCommand).CurDirectoryLen = (*CommandData).CurDirectoryLen;
    (*GetNextVdmCommand).EnvLen = (*CommandData).EnvLen;
    (*GetNextVdmCommand).DesktopLen = (*CommandData).DesktopLen;
    (*GetNextVdmCommand).TitleLen = (*CommandData).TitleLen;
    (*GetNextVdmCommand).ReservedLen = (*CommandData).ReservedLen;
    (*GetNextVdmCommand).VDMState = (*CommandData).VDMState;

    if (*CommandData).CmdLen != 0 {
        NumStrings += 1;
    }
    if (*CommandData).AppLen != 0 {
        NumStrings += 1;
    }
    if (*CommandData).PifLen != 0 {
        NumStrings += 1;
    }
    if (*CommandData).CurDirectoryLen != 0 {
        NumStrings += 1;
    }
    if (*CommandData).EnvLen != 0 {
        NumStrings += 1;
    }
    if (*CommandData).DesktopLen != 0 {
        NumStrings += 1;
    }
    if (*CommandData).TitleLen != 0 {
        NumStrings += 1;
    }
    if (*CommandData).ReservedLen != 0 {
        NumStrings += 1;
    }

    CaptureBuffer = CsrAllocateCaptureBuffer(
        NumStrings + 1,
        (*GetNextVdmCommand).CmdLen as ULONG
            + (*GetNextVdmCommand).AppLen as ULONG
            + (*GetNextVdmCommand).PifLen as ULONG
            + (*GetNextVdmCommand).CurDirectoryLen as ULONG
            + (*GetNextVdmCommand).EnvLen
            + (*GetNextVdmCommand).DesktopLen
            + (*GetNextVdmCommand).TitleLen
            + (*GetNextVdmCommand).ReservedLen
            + core::mem::size_of::<STARTUPINFOW>() as ULONG,
    );
    if CaptureBuffer.is_null() {
        BaseSetLastNTError(STATUS_NO_MEMORY);
        break 'cleanup;
    }

    CsrAllocateMessagePointer(
        CaptureBuffer,
        core::mem::size_of::<STARTUPINFOW>() as ULONG,
        &mut (*GetNextVdmCommand).StartupInfo as *mut _ as *mut PVOID,
    );

    if (*CommandData).CmdLen != 0 {
        CsrAllocateMessagePointer(
            CaptureBuffer,
            (*CommandData).CmdLen as ULONG,
            &mut (*GetNextVdmCommand).CmdLine as *mut _ as *mut PVOID,
        );
    }

    if (*CommandData).AppLen != 0 {
        CsrAllocateMessagePointer(
            CaptureBuffer,
            (*CommandData).AppLen as ULONG,
            &mut (*GetNextVdmCommand).AppName as *mut _ as *mut PVOID,
        );
    }

    if (*CommandData).PifLen != 0 {
        CsrAllocateMessagePointer(
            CaptureBuffer,
            (*CommandData).PifLen as ULONG,
            &mut (*GetNextVdmCommand).PifFile as *mut _ as *mut PVOID,
        );
    }

    if (*CommandData).CurDirectoryLen != 0 {
        CsrAllocateMessagePointer(
            CaptureBuffer,
            (*CommandData).CurDirectoryLen as ULONG,
            &mut (*GetNextVdmCommand).CurDirectory as *mut _ as *mut PVOID,
        );
    }

    if (*CommandData).EnvLen != 0 {
        CsrAllocateMessagePointer(
            CaptureBuffer,
            (*CommandData).EnvLen,
            &mut (*GetNextVdmCommand).Env as *mut _ as *mut PVOID,
        );
    }

    if (*CommandData).DesktopLen != 0 {
        CsrAllocateMessagePointer(
            CaptureBuffer,
            (*CommandData).DesktopLen,
            &mut (*GetNextVdmCommand).Desktop as *mut _ as *mut PVOID,
        );
    }

    if (*CommandData).TitleLen != 0 {
        CsrAllocateMessagePointer(
            CaptureBuffer,
            (*CommandData).TitleLen,
            &mut (*GetNextVdmCommand).Title as *mut _ as *mut PVOID,
        );
    }

    if (*CommandData).ReservedLen != 0 {
        CsrAllocateMessagePointer(
            CaptureBuffer,
            (*CommandData).ReservedLen,
            &mut (*GetNextVdmCommand).Reserved as *mut _ as *mut PVOID,
        );
    }

    loop {
        Status = CsrClientCallServer(
            &mut ApiMessage as *mut BASE_API_MESSAGE as PCSR_API_MESSAGE,
            CaptureBuffer,
            CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepGetNextVDMCommand),
            core::mem::size_of::<BASE_GET_NEXT_VDM_COMMAND>() as ULONG,
        );

        if (*GetNextVdmCommand).WaitObjectForVDM.is_null() {
            break;
        }

        Status = NtWaitForSingleObject(
            (*GetNextVdmCommand).WaitObjectForVDM,
            0,
            core::ptr::null_mut(),
        );
        if Status != STATUS_SUCCESS {
            BaseSetLastNTError(Status);
            break 'cleanup;
        }

        (*GetNextVdmCommand).VDMState |= VDM_FLAG_RETRY;
        (*GetNextVdmCommand).ExitCode = 0;
    }

    if !NT_SUCCESS(Status) {
        if Status == STATUS_INVALID_PARAMETER {
            (*CommandData).CmdLen = (*GetNextVdmCommand).CmdLen;
            (*CommandData).AppLen = (*GetNextVdmCommand).AppLen;
            (*CommandData).PifLen = (*GetNextVdmCommand).PifLen;
            (*CommandData).CurDirectoryLen = (*GetNextVdmCommand).CurDirectoryLen;
            (*CommandData).EnvLen = (*GetNextVdmCommand).EnvLen;
            (*CommandData).DesktopLen = (*GetNextVdmCommand).DesktopLen;
            (*CommandData).TitleLen = (*GetNextVdmCommand).TitleLen;
            (*CommandData).ReservedLen = (*GetNextVdmCommand).ReservedLen;
        } else {
            (*CommandData).CmdLen = 0;
            (*CommandData).AppLen = 0;
            (*CommandData).PifLen = 0;
            (*CommandData).CurDirectoryLen = 0;
            (*CommandData).EnvLen = 0;
            (*CommandData).DesktopLen = 0;
            (*CommandData).TitleLen = 0;
            (*CommandData).ReservedLen = 0;
        }

        BaseSetLastNTError(Status);
        break 'cleanup;
    }

    (*CommandData).StdIn = (*GetNextVdmCommand).StdIn;
    (*CommandData).StdOut = (*GetNextVdmCommand).StdOut;
    (*CommandData).StdErr = (*GetNextVdmCommand).StdErr;

    RtlMoveMemory(
        &mut (*CommandData).StartupInfo as *mut STARTUPINFOA as *mut core::ffi::c_void,
        (*GetNextVdmCommand).StartupInfo as *const core::ffi::c_void,
        core::mem::size_of::<STARTUPINFOW>(),
    );

    if (*CommandData).CmdLen != 0 {
        RtlMoveMemory(
            (*CommandData).CmdLine as *mut core::ffi::c_void,
            (*GetNextVdmCommand).CmdLine as *const core::ffi::c_void,
            (*GetNextVdmCommand).CmdLen as usize,
        );
        (*CommandData).CmdLen = (*GetNextVdmCommand).CmdLen;
    }

    if (*CommandData).AppLen != 0 {
        RtlMoveMemory(
            (*CommandData).AppName as *mut core::ffi::c_void,
            (*GetNextVdmCommand).AppName as *const core::ffi::c_void,
            (*GetNextVdmCommand).AppLen as usize,
        );
        (*CommandData).AppLen = (*GetNextVdmCommand).AppLen;
    }

    if (*CommandData).PifLen != 0 {
        RtlMoveMemory(
            (*CommandData).PifFile as *mut core::ffi::c_void,
            (*GetNextVdmCommand).PifFile as *const core::ffi::c_void,
            (*GetNextVdmCommand).PifLen as usize,
        );
        (*CommandData).PifLen = (*GetNextVdmCommand).PifLen;
    }

    if (*CommandData).CurDirectoryLen != 0 {
        RtlMoveMemory(
            (*CommandData).CurDirectory as *mut core::ffi::c_void,
            (*GetNextVdmCommand).CurDirectory as *const core::ffi::c_void,
            (*GetNextVdmCommand).CurDirectoryLen as usize,
        );
        (*CommandData).CurDirectoryLen = (*GetNextVdmCommand).CurDirectoryLen;
    }

    if (*CommandData).EnvLen != 0 {
        RtlMoveMemory(
            (*CommandData).Env as *mut core::ffi::c_void,
            (*GetNextVdmCommand).Env as *const core::ffi::c_void,
            (*GetNextVdmCommand).EnvLen as usize,
        );
        (*CommandData).EnvLen = (*GetNextVdmCommand).EnvLen;
    }

    if (*CommandData).DesktopLen != 0 {
        RtlMoveMemory(
            (*CommandData).Desktop as *mut core::ffi::c_void,
            (*GetNextVdmCommand).Desktop as *const core::ffi::c_void,
            (*GetNextVdmCommand).DesktopLen as usize,
        );
        (*CommandData).DesktopLen = (*GetNextVdmCommand).DesktopLen;
    }

    if (*CommandData).TitleLen != 0 {
        RtlMoveMemory(
            (*CommandData).Title as *mut core::ffi::c_void,
            (*GetNextVdmCommand).Title as *const core::ffi::c_void,
            (*GetNextVdmCommand).TitleLen as usize,
        );
        (*CommandData).TitleLen = (*GetNextVdmCommand).TitleLen;
    }

    if (*CommandData).ReservedLen != 0 {
        RtlMoveMemory(
            (*CommandData).Reserved as *mut core::ffi::c_void,
            (*GetNextVdmCommand).Reserved as *const core::ffi::c_void,
            (*GetNextVdmCommand).ReservedLen as usize,
        );
        (*CommandData).ReservedLen = (*GetNextVdmCommand).ReservedLen;
    }

    (*CommandData).TaskId = (*GetNextVdmCommand).iTask;
    (*CommandData).CreationFlags = (*GetNextVdmCommand).dwCreationFlags;
    (*CommandData).CodePage = (*GetNextVdmCommand).CodePage;
    (*CommandData).ExitCode = (*GetNextVdmCommand).ExitCode;
    (*CommandData).CurrentDrive = (*GetNextVdmCommand).CurrentDrive;
    (*CommandData).VDMState = (*GetNextVdmCommand).VDMState;
    (*CommandData).ComingFromBat = (*GetNextVdmCommand).fComingFromBat as BOOLEAN;

    Success = 1;
    } // end 'cleanup

    /* Cleanup */
    if !CaptureBuffer.is_null() {
        CsrFreeCaptureBuffer(CaptureBuffer);
    }
    return Success;
}

#[no_mangle]
pub unsafe extern "system" fn GetVDMCurrentDirectories(
    cchCurDirs: DWORD,
    lpszzCurDirs: PCHAR,
) -> DWORD {
    let mut ApiMessage: BASE_API_MESSAGE = core::mem::zeroed();
    let VDMCurrentDirsRequest: PBASE_GETSET_VDM_CURDIRS =
        &mut ApiMessage.Data.VDMCurrentDirsRequest;
    let CaptureBuffer: PCSR_CAPTURE_BUFFER;

    CaptureBuffer = CsrAllocateCaptureBuffer(1, cchCurDirs);
    if CaptureBuffer.is_null() {
        BaseSetLastNTError(STATUS_NO_MEMORY);
        return 0;
    }

    (*VDMCurrentDirsRequest).cchCurDirs = cchCurDirs;
    CsrAllocateMessagePointer(
        CaptureBuffer,
        cchCurDirs,
        &mut (*VDMCurrentDirsRequest).lpszzCurDirs as *mut _ as *mut *mut _,
    );

    CsrClientCallServer(
        &mut ApiMessage as *mut _ as *mut _,
        CaptureBuffer,
        CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepGetVDMCurDirs),
        core::mem::size_of::<BASE_GETSET_VDM_CURDIRS>() as u32,
    );

    BaseSetLastNTError(ApiMessage.Status);

    if NT_SUCCESS(ApiMessage.Status) {
        RtlMoveMemory(
            lpszzCurDirs as *mut _,
            (*VDMCurrentDirsRequest).lpszzCurDirs as *const _,
            cchCurDirs as usize,
        );
    }

    CsrFreeCaptureBuffer(CaptureBuffer);

    return if NT_SUCCESS(ApiMessage.Status)
        || (ApiMessage.Status == STATUS_BUFFER_TOO_SMALL)
    {
        (*VDMCurrentDirsRequest).cchCurDirs
    } else {
        0
    };
}

#[no_mangle]
pub unsafe extern "system" fn RegisterConsoleVDM(
    dwRegisterFlags: DWORD,
    hStartHardwareEvent: HANDLE,
    hEndHardwareEvent: HANDLE,
    hErrorHardwareEvent: HANDLE,
    dwUnusedVar: DWORD,
    lpVideoStateLength: LPDWORD,
    lpVideoState: *mut PVOID,
    lpUnusedBuffer: PVOID,
    dwUnusedBufferLength: DWORD,
    dwVDMBufferSize: COORD,
    lpVDMBuffer: *mut PVOID,
) -> BOOL {
    let mut Success: BOOL;
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let RegisterVDMRequest: PCONSOLE_REGISTERVDM = &mut ApiMessage.Data.RegisterVDMRequest;
    let mut CaptureBuffer: PCSR_CAPTURE_BUFFER = core::ptr::null_mut();

    /* Set up the data to send to the Console Server */
    (*RegisterVDMRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*RegisterVDMRequest).RegisterFlags = dwRegisterFlags;

    if dwRegisterFlags != 0 {
        (*RegisterVDMRequest).StartHardwareEvent = hStartHardwareEvent;
        (*RegisterVDMRequest).EndHardwareEvent = hEndHardwareEvent;
        (*RegisterVDMRequest).ErrorHardwareEvent = hErrorHardwareEvent;

        (*RegisterVDMRequest).VDMBufferSize = dwVDMBufferSize;
    } else {
        // CaptureBuffer = NULL;
    }

    /* Call the server */
    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        CaptureBuffer,
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepRegisterVDM),
        core::mem::size_of::<CONSOLE_REGISTERVDM>() as ULONG,
    );

    /* Check for success */
    Success = if NT_SUCCESS(ApiMessage.Status) { 1 } else { 0 };

    /* Release the capture buffer if needed */
    if !CaptureBuffer.is_null() {
        CsrFreeCaptureBuffer(CaptureBuffer);
    }

    /* Retrieve the results */
    if Success != 0 {
        if dwRegisterFlags != 0 {
            // _SEH2_TRY
            {
                *lpVideoStateLength = (*RegisterVDMRequest).VideoStateLength;
                *lpVideoState = (*RegisterVDMRequest).VideoState;
                *lpVDMBuffer = (*RegisterVDMRequest).VDMBuffer;
            }
            // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
            // {
            //     SetLastError(ERROR_INVALID_ACCESS);
            //     Success = FALSE;
            // }
            // _SEH2_END;
        }
    } else {
        BaseSetLastNTError(ApiMessage.Status);
    }

    /* Return success status */
    return Success;
}

#[no_mangle]
pub unsafe extern "system" fn RegisterWowBaseHandlers(
    Unknown0: DWORD,
) -> BOOL {
    STUB!();
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn RegisterWowExec(
    Unknown0: DWORD,
) -> BOOL {
    STUB!();
    return 0i32;
}

#[no_mangle]
pub unsafe extern "system" fn SetVDMCurrentDirectories(
    cchCurDirs: DWORD,
    lpszzCurDirs: PCHAR,
) -> BOOL {
    let mut ApiMessage: BASE_API_MESSAGE = core::mem::zeroed();
    let VDMCurrentDirsRequest: PBASE_GETSET_VDM_CURDIRS =
        &mut ApiMessage.Data.VDMCurrentDirsRequest;
    let CaptureBuffer: PCSR_CAPTURE_BUFFER;

    CaptureBuffer = CsrAllocateCaptureBuffer(1, cchCurDirs);
    if CaptureBuffer.is_null() {
        BaseSetLastNTError(STATUS_NO_MEMORY);
        return 0;
    }

    (*VDMCurrentDirsRequest).cchCurDirs = cchCurDirs;
    CsrCaptureMessageBuffer(
        CaptureBuffer,
        lpszzCurDirs as PVOID,
        cchCurDirs,
        &mut (*VDMCurrentDirsRequest).lpszzCurDirs as *mut _ as *mut *mut core::ffi::c_void,
    );

    CsrClientCallServer(
        &mut ApiMessage as *mut BASE_API_MESSAGE as PCSR_API_MESSAGE,
        CaptureBuffer,
        CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepSetVDMCurDirs),
        core::mem::size_of::<BASE_GETSET_VDM_CURDIRS>() as ULONG,
    );

    CsrFreeCaptureBuffer(CaptureBuffer);

    BaseSetLastNTError(ApiMessage.Status);

    if NT_SUCCESS(ApiMessage.Status) { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn VDMConsoleOperation(
    iFunction: DWORD,
    lpData: LPVOID,
) -> BOOL {
    STUB!();
    return 0i32;
}

#[no_mangle]
pub unsafe extern "system" fn VDMOperationStarted(
    Unknown0: ULONG,
) -> BOOL {
    BaseUpdateVDMEntry(
        VdmEntryUpdateControlCHandler,
        core::ptr::null_mut(),
        0,
        Unknown0,
    )
}

// Faithful translation of BaseCheckVDM (dll/win32/kernel32/client/vdm.c). Builds the
// BASESRV BasepCheckVDM CSR request, captures the ANSI strings into a CSR capture
// buffer, and calls CSRSS. The bulk translation dropped this 448-line body; restored.
#[no_mangle]
pub unsafe extern "system" fn BaseCheckVDM(
    BinaryType: ULONG,
    mut ApplicationName: PCWCH,
    mut CommandLine: PCWCH,
    mut CurrentDirectory: PCWCH,
    AnsiEnvironment: PANSI_STRING,
    ApiMessage: PBASE_API_MESSAGE,
    iTask: PULONG,
    CreationFlags: DWORD,
    StartupInfo: LPSTARTUPINFOW,
    hUserToken: HANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let CheckVdm: PBASE_CHECK_VDM = &raw mut (*ApiMessage).Data.CheckVDMRequest;
    let mut CaptureBuffer: PCSR_CAPTURE_BUFFER = core::ptr::null_mut();
    let mut CurrentDir: PWCHAR = core::ptr::null_mut();
    let mut ShortAppName: PWCHAR = core::ptr::null_mut();
    let mut ShortCurrentDir: PWCHAR = core::ptr::null_mut();
    let mut Length: usize;
    let mut AnsiCmdLine: PCHAR = core::ptr::null_mut();
    let mut AnsiAppName: PCHAR = core::ptr::null_mut();
    let mut AnsiCurDirectory: PCHAR = core::ptr::null_mut();
    let mut AnsiDesktop: PCHAR = core::ptr::null_mut();
    let mut AnsiTitle: PCHAR = core::ptr::null_mut();
    let mut AnsiReserved: PCHAR = core::ptr::null_mut();
    let mut AnsiStartupInfo: STARTUPINFOA = core::mem::zeroed();
    let mut NumStrings: ULONG = 5;

    /* Parameters validation */
    if ApplicationName.is_null() || CommandLine.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    /* Trim leading whitespace from ApplicationName */
    while *ApplicationName == b' ' as u16 || *ApplicationName == b'\t' as u16 {
        ApplicationName = ApplicationName.add(1);
    }

    /* Calculate the size of the short application name */
    Length = GetShortPathNameW(ApplicationName, core::ptr::null_mut(), 0) as usize;
    if Length == 0 {
        Status = STATUS_OBJECT_PATH_INVALID;
        return base_check_vdm_cleanup(
            Status, AnsiCmdLine, AnsiAppName, AnsiCurDirectory, AnsiDesktop, AnsiTitle,
            AnsiReserved, CaptureBuffer, ShortCurrentDir, CurrentDir, ShortAppName,
        );
    }

    /* Allocate memory for the short application name */
    ShortAppName = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (Length * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as PWCHAR;
    if ShortAppName.is_null() {
        Status = STATUS_NO_MEMORY;
        return base_check_vdm_cleanup(
            Status, AnsiCmdLine, AnsiAppName, AnsiCurDirectory, AnsiDesktop, AnsiTitle,
            AnsiReserved, CaptureBuffer, ShortCurrentDir, CurrentDir, ShortAppName,
        );
    }

    /* Get the short application name */
    if GetShortPathNameW(ApplicationName, ShortAppName, Length as DWORD) == 0 {
        Status = match GetLastError() {
            ERROR_NOT_ENOUGH_MEMORY => STATUS_NO_MEMORY,
            ERROR_INVALID_PARAMETER => STATUS_INVALID_PARAMETER,
            _ => STATUS_OBJECT_PATH_INVALID,
        };
        return base_check_vdm_cleanup(
            Status, AnsiCmdLine, AnsiAppName, AnsiCurDirectory, AnsiDesktop, AnsiTitle,
            AnsiReserved, CaptureBuffer, ShortCurrentDir, CurrentDir, ShortAppName,
        );
    }

    /* Trim leading whitespace from CommandLine */
    while *CommandLine == b' ' as u16 || *CommandLine == b'\t' as u16 {
        CommandLine = CommandLine.add(1);
    }

    /* Strip the first token (ApplicationName) from CommandLine */
    if *CommandLine != 0 {
        Length = wcslen(ApplicationName);
        if Length <= wcslen(CommandLine) && _wcsnicmp(ApplicationName, CommandLine, Length) == 0 {
            /* Skip it */
            CommandLine = CommandLine.add(Length);
        } else {
            /* Get rid of the first token. We stop when we see whitespace. */
            while *CommandLine != 0 && !(*CommandLine == b' ' as u16 || *CommandLine == b'\t' as u16) {
                if *CommandLine == b'\"' as u16 {
                    /* We enter a quoted part, skip it */
                    CommandLine = CommandLine.add(1);
                    while *CommandLine != 0 {
                        let c = *CommandLine;
                        CommandLine = CommandLine.add(1);
                        if c == b'\"' as u16 {
                            break;
                        }
                    }
                } else {
                    CommandLine = CommandLine.add(1);
                }
            }
        }
    }

    /* Trim remaining whitespace between the application name and the parameters */
    while *CommandLine == b' ' as u16 || *CommandLine == b'\t' as u16 {
        CommandLine = CommandLine.add(1);
    }

    /* Get the current directory */
    if CurrentDirectory.is_null() {
        Length = GetCurrentDirectoryW(0, core::ptr::null_mut()) as usize;
        CurrentDir = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (Length * core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as PWCHAR;
        if CurrentDir.is_null() {
            Status = STATUS_NO_MEMORY;
            return base_check_vdm_cleanup(
                Status, AnsiCmdLine, AnsiAppName, AnsiCurDirectory, AnsiDesktop, AnsiTitle,
                AnsiReserved, CaptureBuffer, ShortCurrentDir, CurrentDir, ShortAppName,
            );
        }
        GetCurrentDirectoryW(Length as DWORD, CurrentDir);
        CurrentDirectory = CurrentDir;
    }

    /* Calculate the size of the short current directory path */
    Length = GetShortPathNameW(CurrentDirectory, core::ptr::null_mut(), 0) as usize;

    /* Allocate memory for the short current directory path */
    ShortCurrentDir = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (Length * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as PWCHAR;
    if ShortCurrentDir.is_null() {
        Status = STATUS_NO_MEMORY;
        return base_check_vdm_cleanup(
            Status, AnsiCmdLine, AnsiAppName, AnsiCurDirectory, AnsiDesktop, AnsiTitle,
            AnsiReserved, CaptureBuffer, ShortCurrentDir, CurrentDir, ShortAppName,
        );
    }

    /* Get the short current directory path */
    if GetShortPathNameW(CurrentDirectory, ShortCurrentDir, Length as DWORD) == 0 {
        Status = match GetLastError() {
            ERROR_NOT_ENOUGH_MEMORY => STATUS_NO_MEMORY,
            ERROR_INVALID_PARAMETER => STATUS_INVALID_PARAMETER,
            _ => STATUS_OBJECT_PATH_INVALID,
        };
        return base_check_vdm_cleanup(
            Status, AnsiCmdLine, AnsiAppName, AnsiCurDirectory, AnsiDesktop, AnsiTitle,
            AnsiReserved, CaptureBuffer, ShortCurrentDir, CurrentDir, ShortAppName,
        );
    }

    /* Make sure that the command line isn't too long */
    Length = wcslen(CommandLine);
    if Length > (UNICODE_STRING_MAX_CHARS as usize) - 1 {
        Status = STATUS_INVALID_PARAMETER;
        return base_check_vdm_cleanup(
            Status, AnsiCmdLine, AnsiAppName, AnsiCurDirectory, AnsiDesktop, AnsiTitle,
            AnsiReserved, CaptureBuffer, ShortCurrentDir, CurrentDir, ShortAppName,
        );
    }

    /* Setup the input parameters */
    (*CheckVdm).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*CheckVdm).BinaryType = BinaryType;
    (*CheckVdm).CodePage = CP_ACP as ULONG;
    (*CheckVdm).dwCreationFlags = CreationFlags;
    (*CheckVdm).CurDrive = (*CurrentDirectory.add(0)).wrapping_sub(b'A' as u16) as USHORT;
    (*CheckVdm).CmdLen = (Length as USHORT) + 1;
    (*CheckVdm).AppLen = (wcslen(ShortAppName) as USHORT) + 1;
    (*CheckVdm).PifLen = 0; // TODO: PIF file support!
    (*CheckVdm).CurDirectoryLen = (wcslen(ShortCurrentDir) as USHORT) + 1;
    (*CheckVdm).EnvLen = (*AnsiEnvironment).Length as ULONG;
    (*CheckVdm).DesktopLen = if !(*StartupInfo).lpDesktop.is_null() {
        (wcslen((*StartupInfo).lpDesktop) + 1) as ULONG
    } else {
        0
    };
    (*CheckVdm).TitleLen = if !(*StartupInfo).lpTitle.is_null() {
        (wcslen((*StartupInfo).lpTitle) + 1) as ULONG
    } else {
        0
    };
    (*CheckVdm).ReservedLen = if !(*StartupInfo).lpReserved.is_null() {
        (wcslen((*StartupInfo).lpReserved) + 1) as ULONG
    } else {
        0
    };

    if (*StartupInfo).dwFlags & STARTF_USESTDHANDLES != 0 {
        (*CheckVdm).StdIn = (*StartupInfo).hStdInput;
        (*CheckVdm).StdOut = (*StartupInfo).hStdOutput;
        (*CheckVdm).StdErr = (*StartupInfo).hStdError;
    }

    /* Allocate memory for the ANSI strings (+2 for the '\r\n' on the command line) */
    AnsiCmdLine = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, ((*CheckVdm).CmdLen as SIZE_T) + 2) as PCHAR;
    AnsiAppName = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, (*CheckVdm).AppLen as SIZE_T) as PCHAR;
    AnsiCurDirectory = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, (*CheckVdm).CurDirectoryLen as SIZE_T) as PCHAR;
    if !(*StartupInfo).lpDesktop.is_null() {
        AnsiDesktop = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, (*CheckVdm).DesktopLen as SIZE_T) as PCHAR;
    }
    if !(*StartupInfo).lpTitle.is_null() {
        AnsiTitle = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, (*CheckVdm).TitleLen as SIZE_T) as PCHAR;
    }
    if !(*StartupInfo).lpReserved.is_null() {
        AnsiReserved = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, (*CheckVdm).ReservedLen as SIZE_T) as PCHAR;
    }

    if AnsiCmdLine.is_null()
        || AnsiAppName.is_null()
        || AnsiCurDirectory.is_null()
        || (!(*StartupInfo).lpDesktop.is_null() && AnsiDesktop.is_null())
        || (!(*StartupInfo).lpTitle.is_null() && AnsiTitle.is_null())
        || (!(*StartupInfo).lpReserved.is_null() && AnsiReserved.is_null())
    {
        Status = STATUS_NO_MEMORY;
        return base_check_vdm_cleanup(
            Status, AnsiCmdLine, AnsiAppName, AnsiCurDirectory, AnsiDesktop, AnsiTitle,
            AnsiReserved, CaptureBuffer, ShortCurrentDir, CurrentDir, ShortAppName,
        );
    }

    /* Convert the command line into an ANSI string */
    WideCharToMultiByte(CP_ACP, 0, CommandLine, (*CheckVdm).CmdLen as i32, AnsiCmdLine, (*CheckVdm).CmdLen as i32, core::ptr::null(), core::ptr::null_mut());
    /* Add the needed newline and NULL-terminate */
    (*CheckVdm).CmdLen -= 1; // Rewind back to the NULL character
    *AnsiCmdLine.add((*CheckVdm).CmdLen as usize) = b'\r' as CHAR;
    (*CheckVdm).CmdLen += 1;
    *AnsiCmdLine.add((*CheckVdm).CmdLen as usize) = b'\n' as CHAR;
    (*CheckVdm).CmdLen += 1;
    *AnsiCmdLine.add((*CheckVdm).CmdLen as usize) = 0;
    (*CheckVdm).CmdLen += 1;

    /* Convert the short application name into an ANSI string */
    WideCharToMultiByte(CP_ACP, 0, ShortAppName, (*CheckVdm).AppLen as i32, AnsiAppName, (*CheckVdm).AppLen as i32, core::ptr::null(), core::ptr::null_mut());

    /* Convert the short current directory path into an ANSI string */
    WideCharToMultiByte(CP_ACP, 0, ShortCurrentDir, (*CheckVdm).CurDirectoryLen as i32, AnsiCurDirectory, (*CheckVdm).CurDirectoryLen as i32, core::ptr::null(), core::ptr::null_mut());

    if !(*StartupInfo).lpDesktop.is_null() {
        WideCharToMultiByte(CP_ACP, 0, (*StartupInfo).lpDesktop, (*CheckVdm).DesktopLen as i32, AnsiDesktop, (*CheckVdm).DesktopLen as i32, core::ptr::null(), core::ptr::null_mut());
        NumStrings += 1;
    }

    if !(*StartupInfo).lpTitle.is_null() {
        WideCharToMultiByte(CP_ACP, 0, (*StartupInfo).lpTitle, (*CheckVdm).TitleLen as i32, AnsiTitle, (*CheckVdm).TitleLen as i32, core::ptr::null(), core::ptr::null_mut());
        NumStrings += 1;
    }

    if !(*StartupInfo).lpReserved.is_null() {
        WideCharToMultiByte(CP_ACP, 0, (*StartupInfo).lpReserved, (*CheckVdm).ReservedLen as i32, AnsiReserved, (*CheckVdm).ReservedLen as i32, core::ptr::null(), core::ptr::null_mut());
        NumStrings += 1;
    }

    /* Fill the ANSI startup info structure */
    RtlCopyMemory(
        &raw mut AnsiStartupInfo as PVOID,
        StartupInfo as *const c_void,
        core::mem::size_of::<STARTUPINFOA>() as SIZE_T,
    );
    AnsiStartupInfo.lpReserved = AnsiReserved;
    AnsiStartupInfo.lpDesktop = AnsiDesktop;
    AnsiStartupInfo.lpTitle = AnsiTitle;

    /* Allocate the capture buffer */
    CaptureBuffer = CsrAllocateCaptureBuffer(
        NumStrings,
        (*CheckVdm).CmdLen as ULONG
            + (*CheckVdm).AppLen as ULONG
            + (*CheckVdm).PifLen as ULONG
            + (*CheckVdm).CurDirectoryLen as ULONG
            + (*CheckVdm).DesktopLen
            + (*CheckVdm).TitleLen
            + (*CheckVdm).ReservedLen
            + (*CheckVdm).EnvLen
            + core::mem::size_of::<STARTUPINFOA>() as ULONG,
    );
    if CaptureBuffer.is_null() {
        Status = STATUS_NO_MEMORY;
        return base_check_vdm_cleanup(
            Status, AnsiCmdLine, AnsiAppName, AnsiCurDirectory, AnsiDesktop, AnsiTitle,
            AnsiReserved, CaptureBuffer, ShortCurrentDir, CurrentDir, ShortAppName,
        );
    }

    /* Capture the command line */
    CsrCaptureMessageBuffer(CaptureBuffer, AnsiCmdLine as PVOID, (*CheckVdm).CmdLen as ULONG, &raw mut (*CheckVdm).CmdLine as *mut PVOID);
    /* Capture the application name */
    CsrCaptureMessageBuffer(CaptureBuffer, AnsiAppName as PVOID, (*CheckVdm).AppLen as ULONG, &raw mut (*CheckVdm).AppName as *mut PVOID);

    (*CheckVdm).PifFile = core::ptr::null_mut(); // TODO: PIF file support!

    /* Capture the current directory */
    CsrCaptureMessageBuffer(CaptureBuffer, AnsiCurDirectory as PVOID, (*CheckVdm).CurDirectoryLen as ULONG, &raw mut (*CheckVdm).CurDirectory as *mut PVOID);
    /* Capture the environment */
    CsrCaptureMessageBuffer(CaptureBuffer, (*AnsiEnvironment).Buffer as PVOID, (*CheckVdm).EnvLen, &raw mut (*CheckVdm).Env as *mut PVOID);
    /* Capture the startup info structure */
    CsrCaptureMessageBuffer(CaptureBuffer, &raw mut AnsiStartupInfo as PVOID, core::mem::size_of::<STARTUPINFOA>() as ULONG, &raw mut (*CheckVdm).StartupInfo as *mut PVOID);

    if !(*StartupInfo).lpDesktop.is_null() {
        CsrCaptureMessageBuffer(CaptureBuffer, AnsiDesktop as PVOID, (*CheckVdm).DesktopLen, &raw mut (*CheckVdm).Desktop as *mut PVOID);
    } else {
        (*CheckVdm).Desktop = core::ptr::null_mut();
    }

    if !(*StartupInfo).lpTitle.is_null() {
        CsrCaptureMessageBuffer(CaptureBuffer, AnsiTitle as PVOID, (*CheckVdm).TitleLen, &raw mut (*CheckVdm).Title as *mut PVOID);
    } else {
        (*CheckVdm).Title = core::ptr::null_mut();
    }

    if !(*StartupInfo).lpReserved.is_null() {
        CsrCaptureMessageBuffer(CaptureBuffer, AnsiReserved as PVOID, (*CheckVdm).ReservedLen, &raw mut (*CheckVdm).Reserved as *mut PVOID);
    } else {
        (*CheckVdm).Reserved = core::ptr::null_mut();
    }

    /* Send the message to CSRSS */
    Status = CsrClientCallServer(
        ApiMessage as PCSR_API_MESSAGE,
        CaptureBuffer,
        CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepCheckVDM as ULONG) as CSR_API_NUMBER,
        core::mem::size_of::<BASE_CHECK_VDM>() as ULONG,
    );

    /* Write back the task ID */
    *iTask = (*CheckVdm).iTask;

    base_check_vdm_cleanup(
        Status, AnsiCmdLine, AnsiAppName, AnsiCurDirectory, AnsiDesktop, AnsiTitle,
        AnsiReserved, CaptureBuffer, ShortCurrentDir, CurrentDir, ShortAppName,
    )
}

// BaseCheckVDM's `Cleanup:` epilogue (frees all allocations and returns Status).
#[allow(clippy::too_many_arguments)]
unsafe fn base_check_vdm_cleanup(
    Status: NTSTATUS,
    AnsiCmdLine: PCHAR,
    AnsiAppName: PCHAR,
    AnsiCurDirectory: PCHAR,
    AnsiDesktop: PCHAR,
    AnsiTitle: PCHAR,
    AnsiReserved: PCHAR,
    CaptureBuffer: PCSR_CAPTURE_BUFFER,
    ShortCurrentDir: PWCHAR,
    CurrentDir: PWCHAR,
    ShortAppName: PWCHAR,
) -> NTSTATUS {
    if !AnsiCmdLine.is_null() { RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiCmdLine as PVOID); }
    if !AnsiAppName.is_null() { RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiAppName as PVOID); }
    if !AnsiCurDirectory.is_null() { RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiCurDirectory as PVOID); }
    if !AnsiDesktop.is_null() { RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiDesktop as PVOID); }
    if !AnsiTitle.is_null() { RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiTitle as PVOID); }
    if !AnsiReserved.is_null() { RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiReserved as PVOID); }
    if !CaptureBuffer.is_null() { CsrFreeCaptureBuffer(CaptureBuffer); }
    if !ShortCurrentDir.is_null() { RtlFreeHeap(RtlGetProcessHeap(), 0, ShortCurrentDir as PVOID); }
    if !CurrentDir.is_null() { RtlFreeHeap(RtlGetProcessHeap(), 0, CurrentDir as PVOID); }
    if !ShortAppName.is_null() { RtlFreeHeap(RtlGetProcessHeap(), 0, ShortAppName as PVOID); }
    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Win32 Base API
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            dll/win32/kernel32/client/vdm.c
//  * PURPOSE:         Virtual DOS Machines (VDM) Support
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* TYPES **********************************************************************/
//
// #define BINARY_UNKNOWN  (0)
// #define BINARY_PE_EXE32 (1)
// #define BINARY_PE_DLL32 (2)
// #define BINARY_PE_EXE64 (3)
// #define BINARY_PE_DLL64 (4)
// #define BINARY_WIN16    (5)
// #define BINARY_OS216    (6)
// #define BINARY_DOS      (7)
// #define BINARY_UNIX_EXE (8)
// #define BINARY_UNIX_LIB (9)
//
//
// typedef enum _ENV_NAME_TYPE
// {
//     EnvNameNotAPath = 1,
//     EnvNameSinglePath ,
//     EnvNameMultiplePath
// } ENV_NAME_TYPE;
//
// typedef struct _ENV_INFO
// {
//     ENV_NAME_TYPE NameType;
//     ULONG  NameLength;
//     PWCHAR Name;
// } ENV_INFO, *PENV_INFO;
//
// /* GLOBALS ********************************************************************/
//
// #define ENV_NAME_ENTRY(type, name)  \
//     {(type), _ARRAYSIZE(name) - 1, (name)}
//
// static ENV_INFO BasepEnvNameType[] =
// {
//     ENV_NAME_ENTRY(EnvNameMultiplePath, L"PATH"),
//     ENV_NAME_ENTRY(EnvNameSinglePath  , L"WINDIR"),
//     ENV_NAME_ENTRY(EnvNameSinglePath  , L"SYSTEMROOT"),
//     ENV_NAME_ENTRY(EnvNameMultiplePath, L"TEMP"),
//     ENV_NAME_ENTRY(EnvNameMultiplePath, L"TMP"),
// };
//
// static UNICODE_STRING BaseDotComSuffixName = RTL_CONSTANT_STRING(L".com");
// static UNICODE_STRING BaseDotPifSuffixName = RTL_CONSTANT_STRING(L".pif");
// static UNICODE_STRING BaseDotExeSuffixName = RTL_CONSTANT_STRING(L".exe");
//
// /* FUNCTIONS ******************************************************************/
//
// ULONG
// WINAPI
// BaseIsDosApplication(IN PUNICODE_STRING PathName,
//                      IN NTSTATUS Status)
// {
//     UNICODE_STRING String;
//
//     /* Is it a .com? */
//     String.Length = BaseDotComSuffixName.Length;
//     String.Buffer = &PathName->Buffer[(PathName->Length - String.Length) / sizeof(WCHAR)];
//     if (RtlEqualUnicodeString(&String, &BaseDotComSuffixName, TRUE)) return BINARY_TYPE_COM;
//
//     /* Is it a .pif? */
//     String.Length = BaseDotPifSuffixName.Length;
//     String.Buffer = &PathName->Buffer[(PathName->Length - String.Length) / sizeof(WCHAR)];
//     if (RtlEqualUnicodeString(&String, &BaseDotPifSuffixName, TRUE)) return BINARY_TYPE_PIF;
//
//     /* Is it an exe? */
//     String.Length = BaseDotExeSuffixName.Length;
//     String.Buffer = &PathName->Buffer[(PathName->Length - String.Length) / sizeof(WCHAR)];
//     if (RtlEqualUnicodeString(&String, &BaseDotExeSuffixName, TRUE)) return BINARY_TYPE_EXE;
//
//     return 0;
// }
//
// NTSTATUS
// WINAPI
// BaseCheckVDM(IN ULONG BinaryType,
//              IN PCWCH ApplicationName,
//              IN PCWCH CommandLine,
//              IN PCWCH CurrentDirectory,
//              IN PANSI_STRING AnsiEnvironment,
//              IN PBASE_API_MESSAGE ApiMessage,
//              IN OUT PULONG iTask,
//              IN DWORD CreationFlags,
//              IN LPSTARTUPINFOW StartupInfo,
//              IN HANDLE hUserToken OPTIONAL)
// {
//     NTSTATUS Status;
//     PBASE_CHECK_VDM CheckVdm = &ApiMessage->Data.CheckVDMRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer = NULL;
//     PWCHAR CurrentDir = NULL;
//     PWCHAR ShortAppName = NULL;
//     PWCHAR ShortCurrentDir = NULL;
//     SIZE_T Length;
//     PCHAR AnsiCmdLine = NULL;
//     PCHAR AnsiAppName = NULL;
//     PCHAR AnsiCurDirectory = NULL;
//     PCHAR AnsiDesktop = NULL;
//     PCHAR AnsiTitle = NULL;
//     PCHAR AnsiReserved = NULL;
//     STARTUPINFOA AnsiStartupInfo;
//     ULONG NumStrings = 5;
//
//     /* Parameters validation */
//     if (ApplicationName == NULL || CommandLine == NULL)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Trim leading whitespace from ApplicationName */
//     while (*ApplicationName == L' ' || *ApplicationName == L'\t')
//         ++ApplicationName;
//
//     /* Calculate the size of the short application name */
//     Length = GetShortPathNameW(ApplicationName, NULL, 0);
//     if (Length == 0)
//     {
//         Status = STATUS_OBJECT_PATH_INVALID;
//         goto Cleanup;
//     }
//
//     /* Allocate memory for the short application name */
//     ShortAppName = (PWCHAR)RtlAllocateHeap(RtlGetProcessHeap(),
//                                            HEAP_ZERO_MEMORY,
//                                            Length * sizeof(WCHAR));
//     if (!ShortAppName)
//     {
//         Status = STATUS_NO_MEMORY;
//         goto Cleanup;
//     }
//
//     /* Get the short application name */
//     if (GetShortPathNameW(ApplicationName, ShortAppName, Length) == 0)
//     {
//         /* Try to determine which error occurred */
//         switch (GetLastError())
//         {
//             case ERROR_NOT_ENOUGH_MEMORY:
//             {
//                 Status = STATUS_NO_MEMORY;
//                 break;
//             }
//
//             case ERROR_INVALID_PARAMETER:
//             {
//                 Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             default:
//             {
//                 Status = STATUS_OBJECT_PATH_INVALID;
//             }
//         }
//
//         goto Cleanup;
//     }
//
//     /* Trim leading whitespace from CommandLine */
//     while (*CommandLine == L' ' || *CommandLine == L'\t')
//         ++CommandLine;
//
//     /*
//      * CommandLine is usually formatted as: 'ApplicationName param0 ...'.
//      * So we want to strip the first token (ApplicationName) from it.
//      * Two cases are in fact possible:
//      * - either the first token is indeed ApplicationName, so we just skip it;
//      * - or the first token is not exactly ApplicationName, because it happened
//      *   that somebody else already preprocessed CommandLine. Therefore we
//      *   suppose that the first token corresponds to an application name and
//      *   we skip it. Care should be taken when quotes are present in this token.
//      */
//      if (*CommandLine)
//      {
//         /* The first part of CommandLine should be the ApplicationName... */
//         Length = wcslen(ApplicationName);
//         if (Length <= wcslen(CommandLine) &&
//             _wcsnicmp(ApplicationName, CommandLine, Length) == 0)
//         {
//             /* Skip it */
//             CommandLine += Length;
//         }
//         /*
//          * ... but it is not, however we still have a token. We suppose that
//          * it corresponds to some sort of application name, so we skip it too.
//          */
//         else
//         {
//             /* Get rid of the first token. We stop when we see whitespace. */
//             while (*CommandLine && !(*CommandLine == L' ' || *CommandLine == L'\t'))
//             {
//                 if (*CommandLine == L'\"')
//                 {
//                     /* We enter a quoted part, skip it */
//                     ++CommandLine;
//                     while (*CommandLine && *CommandLine++ != L'\"') ;
//                 }
//                 else
//                 {
//                     /* Go to the next character */
//                     ++CommandLine;
//                 }
//             }
//         }
//     }
//
//     /*
//      * Trim remaining whitespace from CommandLine that may be
//      * present between the application name and the parameters.
//      */
//     while (*CommandLine == L' ' || *CommandLine == L'\t')
//         ++CommandLine;
//
//     /* Get the current directory */
//     if (CurrentDirectory == NULL)
//     {
//         /* Allocate memory for the current directory path */
//         Length = GetCurrentDirectoryW(0, NULL);
//         CurrentDir = (PWCHAR)RtlAllocateHeap(RtlGetProcessHeap(),
//                                              HEAP_ZERO_MEMORY,
//                                              Length * sizeof(WCHAR));
//         if (CurrentDir == NULL)
//         {
//             Status = STATUS_NO_MEMORY;
//             goto Cleanup;
//         }
//
//         /* Get the current directory */
//         GetCurrentDirectoryW(Length, CurrentDir);
//         CurrentDirectory = CurrentDir;
//     }
//
//     /* Calculate the size of the short current directory path */
//     Length = GetShortPathNameW(CurrentDirectory, NULL, 0);
//
//     /* Allocate memory for the short current directory path */
//     ShortCurrentDir = (PWCHAR)RtlAllocateHeap(RtlGetProcessHeap(),
//                                               HEAP_ZERO_MEMORY,
//                                               Length * sizeof(WCHAR));
//     if (!ShortCurrentDir)
//     {
//         Status = STATUS_NO_MEMORY;
//         goto Cleanup;
//     }
//
//     /* Get the short current directory path */
//     if (!GetShortPathNameW(CurrentDirectory, ShortCurrentDir, Length))
//     {
//         /* Try to determine which error occurred */
//         switch (GetLastError())
//         {
//             case ERROR_NOT_ENOUGH_MEMORY:
//             {
//                 Status = STATUS_NO_MEMORY;
//                 break;
//             }
//
//             case ERROR_INVALID_PARAMETER:
//             {
//                 Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             default:
//             {
//                 Status = STATUS_OBJECT_PATH_INVALID;
//             }
//         }
//         goto Cleanup;
//     }
//
//     /* Make sure that the command line isn't too long */
//     Length = wcslen(CommandLine);
//     if (Length > UNICODE_STRING_MAX_CHARS - 1)
//     {
//         Status = STATUS_INVALID_PARAMETER;
//         goto Cleanup;
//     }
//
//     /* Setup the input parameters */
//     CheckVdm->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     CheckVdm->BinaryType = BinaryType;
//     CheckVdm->CodePage = CP_ACP;
//     CheckVdm->dwCreationFlags = CreationFlags;
//     CheckVdm->CurDrive = CurrentDirectory[0] - L'A';
//     CheckVdm->CmdLen = (USHORT)Length + 1;
//     CheckVdm->AppLen = (USHORT)wcslen(ShortAppName) + 1;
//     CheckVdm->PifLen = 0; // TODO: PIF file support!
//     CheckVdm->CurDirectoryLen = (USHORT)wcslen(ShortCurrentDir) + 1;
//     CheckVdm->EnvLen = AnsiEnvironment->Length;
//     CheckVdm->DesktopLen = (StartupInfo->lpDesktop != NULL) ? (wcslen(StartupInfo->lpDesktop) + 1) : 0;
//     CheckVdm->TitleLen = (StartupInfo->lpTitle != NULL) ? (wcslen(StartupInfo->lpTitle) + 1) : 0;
//     CheckVdm->ReservedLen = (StartupInfo->lpReserved != NULL) ? (wcslen(StartupInfo->lpReserved) + 1) : 0;
//
//     if (StartupInfo->dwFlags & STARTF_USESTDHANDLES)
//     {
//         /* Set the standard handles */
//         CheckVdm->StdIn  = StartupInfo->hStdInput;
//         CheckVdm->StdOut = StartupInfo->hStdOutput;
//         CheckVdm->StdErr = StartupInfo->hStdError;
//     }
//
//     /* Allocate memory for the ANSI strings */
//     // We need to add the newline characters '\r\n' to the command line
//     AnsiCmdLine = (PCHAR)RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, CheckVdm->CmdLen + 2);
//     AnsiAppName = (PCHAR)RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, CheckVdm->AppLen);
//     AnsiCurDirectory = (PCHAR)RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, CheckVdm->CurDirectoryLen);
//     if (StartupInfo->lpDesktop)
//         AnsiDesktop = (PCHAR)RtlAllocateHeap(RtlGetProcessHeap(),
//                                              HEAP_ZERO_MEMORY,
//                                              CheckVdm->DesktopLen);
//     if (StartupInfo->lpTitle)
//         AnsiTitle = (PCHAR)RtlAllocateHeap(RtlGetProcessHeap(),
//                                            HEAP_ZERO_MEMORY,
//                                            CheckVdm->TitleLen);
//     if (StartupInfo->lpReserved)
//         AnsiReserved = (PCHAR)RtlAllocateHeap(RtlGetProcessHeap(),
//                                               HEAP_ZERO_MEMORY,
//                                               CheckVdm->ReservedLen);
//
//     if (!AnsiCmdLine
//         || !AnsiAppName
//         || !AnsiCurDirectory
//         || (StartupInfo->lpDesktop && !AnsiDesktop)
//         || (StartupInfo->lpTitle && !AnsiTitle)
//         || (StartupInfo->lpReserved && !AnsiReserved))
//     {
//         Status = STATUS_NO_MEMORY;
//         goto Cleanup;
//     }
//
//     /* Convert the command line into an ANSI string */
//     WideCharToMultiByte(CP_ACP,
//                         0,
//                         CommandLine,
//                         CheckVdm->CmdLen,
//                         AnsiCmdLine,
//                         CheckVdm->CmdLen,
//                         NULL,
//                         NULL);
//     /* Add the needed newline and NULL-terminate */
//     CheckVdm->CmdLen--; // Rewind back to the NULL character
//     AnsiCmdLine[CheckVdm->CmdLen++] = '\r';
//     AnsiCmdLine[CheckVdm->CmdLen++] = '\n';
//     AnsiCmdLine[CheckVdm->CmdLen++] = 0;
//
//     /* Convert the short application name into an ANSI string */
//     WideCharToMultiByte(CP_ACP,
//                         0,
//                         ShortAppName,
//                         CheckVdm->AppLen,
//                         AnsiAppName,
//                         CheckVdm->AppLen,
//                         NULL,
//                         NULL);
//
//     /* Convert the short current directory path into an ANSI string */
//     WideCharToMultiByte(CP_ACP,
//                         0,
//                         ShortCurrentDir,
//                         CheckVdm->CurDirectoryLen,
//                         AnsiCurDirectory,
//                         CheckVdm->CurDirectoryLen,
//                         NULL,
//                         NULL);
//
//     if (StartupInfo->lpDesktop)
//     {
//         /* Convert the desktop name into an ANSI string */
//         WideCharToMultiByte(CP_ACP,
//                             0,
//                             StartupInfo->lpDesktop,
//                             CheckVdm->DesktopLen,
//                             AnsiDesktop,
//                             CheckVdm->DesktopLen,
//                             NULL,
//                             NULL);
//         NumStrings++;
//     }
//
//     if (StartupInfo->lpTitle)
//     {
//         /* Convert the title into an ANSI string */
//         WideCharToMultiByte(CP_ACP,
//                             0,
//                             StartupInfo->lpTitle,
//                             CheckVdm->TitleLen,
//                             AnsiTitle,
//                             CheckVdm->TitleLen,
//                             NULL,
//                             NULL);
//         NumStrings++;
//     }
//
//     if (StartupInfo->lpReserved)
//     {
//         /* Convert the reserved value into an ANSI string */
//         WideCharToMultiByte(CP_ACP,
//                             0,
//                             StartupInfo->lpReserved,
//                             CheckVdm->ReservedLen,
//                             AnsiReserved,
//                             CheckVdm->ReservedLen,
//                             NULL,
//                             NULL);
//         NumStrings++;
//     }
//
//     /* Fill the ANSI startup info structure */
//     RtlCopyMemory(&AnsiStartupInfo, StartupInfo, sizeof(AnsiStartupInfo));
//     AnsiStartupInfo.lpReserved = AnsiReserved;
//     AnsiStartupInfo.lpDesktop = AnsiDesktop;
//     AnsiStartupInfo.lpTitle = AnsiTitle;
//
//     /* Allocate the capture buffer */
//     CaptureBuffer = CsrAllocateCaptureBuffer(NumStrings,
//                                              CheckVdm->CmdLen
//                                              + CheckVdm->AppLen
//                                              + CheckVdm->PifLen
//                                              + CheckVdm->CurDirectoryLen
//                                              + CheckVdm->DesktopLen
//                                              + CheckVdm->TitleLen
//                                              + CheckVdm->ReservedLen
//                                              + CheckVdm->EnvLen
//                                              + sizeof(*CheckVdm->StartupInfo));
//     if (CaptureBuffer == NULL)
//     {
//         Status = STATUS_NO_MEMORY;
//         goto Cleanup;
//     }
//
//     /* Capture the command line */
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             AnsiCmdLine,
//                             CheckVdm->CmdLen,
//                             (PVOID*)&CheckVdm->CmdLine);
//
//     /* Capture the application name */
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             AnsiAppName,
//                             CheckVdm->AppLen,
//                             (PVOID*)&CheckVdm->AppName);
//
//     CheckVdm->PifFile = NULL; // TODO: PIF file support!
//
//     /* Capture the current directory */
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             AnsiCurDirectory,
//                             CheckVdm->CurDirectoryLen,
//                             (PVOID*)&CheckVdm->CurDirectory);
//
//     /* Capture the environment */
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             AnsiEnvironment->Buffer,
//                             CheckVdm->EnvLen,
//                             (PVOID*)&CheckVdm->Env);
//
//     /* Capture the startup info structure */
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             &AnsiStartupInfo,
//                             sizeof(*CheckVdm->StartupInfo),
//                             (PVOID*)&CheckVdm->StartupInfo);
//
//     if (StartupInfo->lpDesktop)
//     {
//         /* Capture the desktop name */
//         CsrCaptureMessageBuffer(CaptureBuffer,
//                                 AnsiDesktop,
//                                 CheckVdm->DesktopLen,
//                                 (PVOID*)&CheckVdm->Desktop);
//     }
//     else CheckVdm->Desktop = NULL;
//
//     if (StartupInfo->lpTitle)
//     {
//         /* Capture the title */
//         CsrCaptureMessageBuffer(CaptureBuffer,
//                                 AnsiTitle,
//                                 CheckVdm->TitleLen,
//                                 (PVOID*)&CheckVdm->Title);
//     }
//     else CheckVdm->Title = NULL;
//
//     if (StartupInfo->lpReserved)
//     {
//         /* Capture the reserved parameter */
//         CsrCaptureMessageBuffer(CaptureBuffer,
//                                 AnsiReserved,
//                                 CheckVdm->ReservedLen,
//                                 (PVOID*)&CheckVdm->Reserved);
//     }
//     else CheckVdm->Reserved = NULL;
//
//     /* Send the message to CSRSS */
//     Status = CsrClientCallServer((PCSR_API_MESSAGE)ApiMessage,
//                                  CaptureBuffer,
//                                  CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepCheckVDM),
//                                  sizeof(*CheckVdm));
//
//     /* Write back the task ID */
//     *iTask = CheckVdm->iTask;
//
// Cleanup:
//
//     /* Free the ANSI strings */
//     if (AnsiCmdLine) RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiCmdLine);
//     if (AnsiAppName) RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiAppName);
//     if (AnsiCurDirectory) RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiCurDirectory);
//     if (AnsiDesktop) RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiDesktop);
//     if (AnsiTitle) RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiTitle);
//     if (AnsiReserved) RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiReserved);
//
//     /* Free the capture buffer */
//     if (CaptureBuffer) CsrFreeCaptureBuffer(CaptureBuffer);
//
//     /* Free the current directory, if it was allocated here, and its short path */
//     if (ShortCurrentDir) RtlFreeHeap(RtlGetProcessHeap(), 0, ShortCurrentDir);
//     if (CurrentDir) RtlFreeHeap(RtlGetProcessHeap(), 0, CurrentDir);
//
//     /* Free the short app name */
//     if (ShortAppName) RtlFreeHeap(RtlGetProcessHeap(), 0, ShortAppName);
//
//     return Status;
// }
//
// BOOL
// WINAPI
// BaseUpdateVDMEntry(IN ULONG UpdateIndex,
//                    IN OUT PHANDLE WaitHandle,
//                    IN ULONG IndexInfo,
//                    IN ULONG BinaryType)
// {
//     BASE_API_MESSAGE ApiMessage;
//     PBASE_UPDATE_VDM_ENTRY UpdateVdmEntry = &ApiMessage.Data.UpdateVDMEntryRequest;
//
//     /* Check what update is being sent */
//     switch (UpdateIndex)
//     {
//         /* VDM is being undone */
//         case VdmEntryUndo:
//         {
//             /* Tell the server how far we had gotten along */
//             UpdateVdmEntry->iTask = HandleToUlong(*WaitHandle);
//             UpdateVdmEntry->VDMCreationState = IndexInfo;
//             break;
//         }
//
//         /* VDM is ready with a new process handle */
//         case VdmEntryUpdateProcess:
//         {
//             /* Send it the process handle */
//             UpdateVdmEntry->VDMProcessHandle = *WaitHandle;
//             UpdateVdmEntry->iTask = IndexInfo;
//             break;
//         }
//     }
//
//     /* Also check what kind of binary this is for the console handle */
//     if (BinaryType == BINARY_TYPE_WOW)
//     {
//         /* Magic value for 16-bit apps */
//         UpdateVdmEntry->ConsoleHandle = (HANDLE)-1;
//     }
//     else if (UpdateVdmEntry->iTask)
//     {
//         /* No handle for true VDM */
//         UpdateVdmEntry->ConsoleHandle = NULL;
//     }
//     else
//     {
//         /* Otherwise, use the regular console handle */
//         UpdateVdmEntry->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     }
//
//     /* Finally write the index and binary type */
//     UpdateVdmEntry->EntryIndex = UpdateIndex;
//     UpdateVdmEntry->BinaryType = BinaryType;
//
//     /* Send the message to CSRSS */
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepUpdateVDMEntry),
//                         sizeof(*UpdateVdmEntry));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         /* Handle failure */
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     /* If this was an update, CSRSS returns a new wait handle */
//     if (UpdateIndex == VdmEntryUpdateProcess)
//     {
//         /* Return it to the caller */
//         *WaitHandle = UpdateVdmEntry->WaitObjectForParent;
//     }
//
//     /* We made it */
//     return TRUE;
// }
//
// BOOL
// WINAPI
// BaseCheckForVDM(IN HANDLE ProcessHandle,
//                 OUT LPDWORD ExitCode)
// {
//     NTSTATUS Status;
//     EVENT_BASIC_INFORMATION EventBasicInfo;
//     BASE_API_MESSAGE ApiMessage;
//     PBASE_GET_VDM_EXIT_CODE GetVdmExitCode = &ApiMessage.Data.GetVDMExitCodeRequest;
//
//     /* It's VDM if the process is actually a wait handle (an event) */
//     Status = NtQueryEvent(ProcessHandle,
//                           EventBasicInformation,
//                           &EventBasicInfo,
//                           sizeof(EventBasicInfo),
//                           NULL);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Setup the input parameters */
//     GetVdmExitCode->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     GetVdmExitCode->hParent = ProcessHandle;
//
//     /* Call CSRSS */
//     Status = CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                                  NULL,
//                                  CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepGetVDMExitCode),
//                                  sizeof(*GetVdmExitCode));
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Get the exit code from the reply */
//     *ExitCode = GetVdmExitCode->ExitCode;
//     return TRUE;
// }
//
// BOOL
// WINAPI
// BaseGetVdmConfigInfo(IN LPCWSTR CommandLineReserved,
//                      IN ULONG DosSeqId,
//                      IN ULONG BinaryType,
//                      IN PUNICODE_STRING CmdLineString,
//                      OUT PULONG VdmSize)
// {
//     WCHAR Buffer[MAX_PATH];
//     WCHAR CommandLine[MAX_PATH * 2];
//     ULONG Length;
//
//     /* Clear the buffer in case we fail */
//     CmdLineString->Buffer = 0;
//
//     /* Always return the same size: 16 Mb */
//     *VdmSize = 0x1000000;
//
//     /* Get the system directory */
//     Length = GetSystemDirectoryW(Buffer, MAX_PATH);
//     if (!(Length) || (Length >= MAX_PATH))
//     {
//         /* Eliminate no path or path too big */
//         SetLastError(ERROR_INVALID_NAME);
//         return FALSE;
//     }
//
//     /* Check if this is VDM with a DOS Sequence ID */
//     if (DosSeqId)
//     {
//         /*
//          * Build the VDM string for it:
//          * -i%lx : Gives the DOS Sequence ID;
//          * %s%c  : Nothing if DOS VDM, -w if WoW VDM, -ws if separate WoW VDM.
//          */
//         _snwprintf(CommandLine,
//                    ARRAYSIZE(CommandLine),
//                    L"\"%s\\ntvdm.exe\" -i%lx %s%c",
//                    Buffer,
//                    DosSeqId,
//                    (BinaryType == BINARY_TYPE_DOS) ? L" " : L"-w",
//                    (BinaryType == BINARY_TYPE_SEPARATE_WOW) ? L's' : L' ');
//     }
//     else
//     {
//         /*
//          * Build the string for it without the DOS Sequence ID:
//          * %s%c  : Nothing if DOS VDM, -w if WoW VDM, -ws if separate WoW VDM.
//          */
//         _snwprintf(CommandLine,
//                    ARRAYSIZE(CommandLine),
//                    L"\"%s\\ntvdm.exe\" %s%c",
//                    Buffer,
//                    (BinaryType == BINARY_TYPE_DOS) ? L" " : L"-w",
//                    (BinaryType == BINARY_TYPE_SEPARATE_WOW) ? L's' : L' ');
//     }
//
//     /* Create the actual string */
//     return RtlCreateUnicodeString(CmdLineString, CommandLine);
// }
//
// ENV_NAME_TYPE
// WINAPI
// BaseGetEnvNameType_U(IN PWCHAR Name,
//                      IN ULONG NameLength)
// {
//     PENV_INFO EnvInfo;
//     ENV_NAME_TYPE NameType;
//     ULONG i;
//
//     /* Start by assuming the environment variable doesn't describe paths */
//     NameType = EnvNameNotAPath;
//
//     /* Loop all the environment names */
//     for (i = 0; i < ARRAYSIZE(BasepEnvNameType); i++)
//     {
//         /* Get this entry */
//         EnvInfo = &BasepEnvNameType[i];
//
//         /* Check if it matches the name */
//         if ((EnvInfo->NameLength == NameLength) &&
//             (_wcsnicmp(EnvInfo->Name, Name, NameLength) == 0))
//         {
//             /* It does, return the type */
//             NameType = EnvInfo->NameType;
//             break;
//         }
//     }
//
//     return NameType;
// }
//
// BOOL
// NTAPI
// BaseCreateVDMEnvironment(IN PWCHAR lpEnvironment,
//                          OUT PANSI_STRING AnsiEnv,
//                          OUT PUNICODE_STRING UnicodeEnv)
// {
// #define IS_ALPHA(x)   \
//     ( ((x) >= L'A' && (x) <= L'Z') || ((x) >= L'a' && (x) <= L'z') )
//
// // From lib/rtl/path.c :
// // Can be put in some .h ??
// #define IS_PATH_SEPARATOR(x)    ((x) == L'\\' || (x) == L'/')
//
//     BOOL Success = FALSE;
//     NTSTATUS Status;
//     ULONG EnvironmentSize = 0;
//     SIZE_T RegionSize;
//     PWCHAR Environment, NewEnvironment = NULL;
//     ENV_NAME_TYPE NameType;
//     ULONG NameLength, NumChars, Remaining;
//     PWCHAR SourcePtr, DestPtr, StartPtr;
//
//     /* Make sure we have both strings */
//     if (!AnsiEnv || !UnicodeEnv)
//     {
//         /* Fail */
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Check if an environment was passed in */
//     if (!lpEnvironment)
//     {
//         /* Nope, create one */
//         Status = RtlCreateEnvironment(TRUE, &Environment);
//         if (!NT_SUCCESS(Status)) goto Cleanup;
//     }
//     else
//     {
//         /* Use the one we got */
//         Environment = lpEnvironment;
//     }
//
//     /* Do we have something now ? */
//     if (!Environment)
//     {
//         /* Still not, bail out */
//         SetLastError(ERROR_BAD_ENVIRONMENT);
//         goto Cleanup;
//     }
//
//     /*
//      * Count how much space the whole environment takes. The environment block is
//      * doubly NULL-terminated (NULL from last string and final NULL terminator).
//      */
//     SourcePtr = Environment;
//     while (!(*SourcePtr++ == UNICODE_NULL && *SourcePtr == UNICODE_NULL))
//         ++EnvironmentSize;
//     EnvironmentSize += 2; // Add the two terminating NULLs
//
//     /*
//      * Allocate a new copy large enough to hold all the environment with paths
//      * in their short form. Since the short form of a path can be a bit longer
//      * than its long form, for example in the case where characters that are
//      * invalid in the 8.3 representation are present in the long path name:
//      *   'C:\\a+b' --> 'C:\\A_B~1', or:
//      *   'C:\\a b' --> 'C:\\AB2761~1' (with checksum inserted),
//      * we suppose that the possible total number of extra characters needed to
//      * convert the long paths into their short form is at most equal to MAX_PATH.
//      */
//     RegionSize = (EnvironmentSize + MAX_PATH) * sizeof(WCHAR);
//     Status = NtAllocateVirtualMemory(NtCurrentProcess(),
//                                      (PVOID*)&NewEnvironment,
//                                      0,
//                                      &RegionSize,
//                                      MEM_COMMIT,
//                                      PAGE_READWRITE);
//     if (!NT_SUCCESS(Status))
//     {
//         /* We failed, bail out */
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         NewEnvironment = NULL;
//         goto Cleanup;
//     }
//
//     /* Parse the environment block */
//     Remaining = MAX_PATH - 2; // '-2': remove the last two NULLs. FIXME: is it really needed??
//     SourcePtr = Environment;
//     DestPtr   = NewEnvironment;
//
//     /* Loop through all the environment strings */
//     while (*SourcePtr != UNICODE_NULL)
//     {
//         /*
//          * 1. Check the type of the environment variable and copy its name.
//          */
//
//         /* Regular environment variable */
//         if (*SourcePtr != L'=')
//         {
//             StartPtr = SourcePtr;
//
//             /* Copy the environment variable name, including the '=' */
//             while (*SourcePtr != UNICODE_NULL)
//             {
//                 *DestPtr++ = *SourcePtr;
//                 if (*SourcePtr++ == L'=') break;
//             }
//
//             /* Guess the type of the environment variable */
//             NameType = BaseGetEnvNameType_U(StartPtr, SourcePtr - StartPtr - 1);
//         }
//         /* 'Current directory' environment variable (i.e. of '=X:=' form) */
//         else // if (*SourcePtr == L'=')
//         {
//             /* First assume we have a possibly malformed environment variable */
//             NameType = EnvNameNotAPath;
//
//             /* Check for a valid 'Current directory' environment variable */
//             if (IS_ALPHA(SourcePtr[1]) && SourcePtr[2] == L':' && SourcePtr[3] == L'=')
//             {
//                 /*
//                  * Small optimization: convert the path to short form only if
//                  * the current directory is not the root directory (i.e. not
//                  * of the '=X:=Y:\' form), otherwise just do a simple copy.
//                  */
//                 if ( wcslen(SourcePtr) >= ARRAYSIZE("=X:=Y:\\")-1 &&
//                      !( IS_ALPHA(SourcePtr[4]) && SourcePtr[5] == L':' &&
//                         IS_PATH_SEPARATOR(SourcePtr[6]) && SourcePtr[7] == UNICODE_NULL ) )
//                 {
//                     NameType = EnvNameSinglePath;
//
//                     /* Copy the '=X:=' prefix */
//                     *DestPtr++ = SourcePtr[0];
//                     *DestPtr++ = SourcePtr[1];
//                     *DestPtr++ = SourcePtr[2];
//                     *DestPtr++ = SourcePtr[3];
//                     SourcePtr += 4;
//                 }
//             }
//             else
//             {
//                 /*
//                  * Invalid stuff starting with '=', i.e.:
//                  * =? (with '?' not being a letter)
//                  * =X??? (with '?' not being ":=" and not followed by something longer than 3 characters)
//                  * =X:=??? (with '?' not being "X:\\")
//                  *
//                  * 'NameType' is already set to 'EnvNameNotAPath'.
//                  */
//             }
//         }
//
//
//         /*
//          * 2. Copy the environment value and perform conversions accordingly.
//          */
//
//         if (NameType == EnvNameNotAPath)
//         {
//             /* Copy everything, including the NULL terminator */
//             do
//             {
//                 *DestPtr++ = *SourcePtr;
//             } while (*SourcePtr++ != UNICODE_NULL);
//         }
//         else if (NameType == EnvNameSinglePath)
//         {
//             /* Convert the path to its short form */
//             NameLength = wcslen(SourcePtr);
//             NumChars = GetShortPathNameW(SourcePtr, DestPtr, NameLength + 1 + Remaining);
//             if (NumChars == 0 || NumChars > NameLength + Remaining)
//             {
//                 /* If the conversion failed, just copy the original value */
//                 RtlCopyMemory(DestPtr, SourcePtr, NameLength * sizeof(WCHAR));
//                 NumChars = NameLength;
//             }
//             DestPtr += NumChars;
//             if (NumChars > NameLength)
//                 Remaining -= (NumChars - NameLength);
//
//             SourcePtr += NameLength;
//
//             /* Copy the NULL terminator */
//             *DestPtr++ = *SourcePtr++;
//         }
//         else // if (NameType == EnvNameMultiplePath)
//         {
//             WCHAR Delimiter;
//
//             /* Loop through the list of paths (delimited by ';') and convert each path to its short form */
//             do
//             {
//                 /* Copy any trailing ';' before going to the next path */
//                 while (*SourcePtr == L';')
//                 {
//                     *DestPtr++ = *SourcePtr++;
//                 }
//
//                 StartPtr = SourcePtr;
//
//                 /* Find the next path list delimiter or the NULL terminator */
//                 while (*SourcePtr != UNICODE_NULL && *SourcePtr != L';')
//                 {
//                     ++SourcePtr;
//                 }
//                 Delimiter = *SourcePtr;
//
//                 NameLength = SourcePtr - StartPtr;
//                 if (NameLength)
//                 {
//                     /*
//                      * Temporarily replace the possible path list delimiter by NULL.
//                      * 'lpEnvironment' must point to a read+write memory buffer!
//                      */
//                     *SourcePtr = UNICODE_NULL;
//
//                     NumChars = GetShortPathNameW(StartPtr, DestPtr, NameLength + 1 + Remaining);
//                     if ( NumChars == 0 ||
//                         (Delimiter == L';' ? NumChars > NameLength + Remaining
//                                            : NumChars > NameLength /* + Remaining ?? */) )
//                     {
//                         /* If the conversion failed, just copy the original value */
//                         RtlCopyMemory(DestPtr, StartPtr, NameLength * sizeof(WCHAR));
//                         NumChars = NameLength;
//                     }
//                     DestPtr += NumChars;
//                     if (NumChars > NameLength)
//                         Remaining -= (NumChars - NameLength);
//
//                     /* If removed, restore the path list delimiter in the source environment value and copy it */
//                     if (Delimiter != UNICODE_NULL)
//                     {
//                         *SourcePtr = Delimiter;
//                         *DestPtr++ = *SourcePtr++;
//                     }
//                 }
//             } while (*SourcePtr != UNICODE_NULL);
//
//             /* Copy the NULL terminator */
//             *DestPtr++ = *SourcePtr++;
//         }
//     }
//
//     /* NULL-terminate the environment block */
//     *DestPtr++ = UNICODE_NULL;
//
//     /* Initialize the Unicode string to hold it */
//     RtlInitEmptyUnicodeString(UnicodeEnv, NewEnvironment,
//                               (DestPtr - NewEnvironment) * sizeof(WCHAR));
//     UnicodeEnv->Length = UnicodeEnv->MaximumLength;
//
//     /* Create its ANSI version */
//     Status = RtlUnicodeStringToAnsiString(AnsiEnv, UnicodeEnv, TRUE);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Set last error if conversion failure */
//         BaseSetLastNTError(Status);
//     }
//     else
//     {
//         /* Everything went okay, so return success */
//         Success = TRUE;
//         NewEnvironment = NULL;
//     }
//
// Cleanup:
//     /* Cleanup path starts here, start by destroying the environment copy */
//     if (!lpEnvironment && Environment) RtlDestroyEnvironment(Environment);
//
//     /* See if we are here due to failure */
//     if (NewEnvironment)
//     {
//         /* Initialize the paths to be empty */
//         RtlInitEmptyUnicodeString(UnicodeEnv, NULL, 0);
//         RtlInitEmptyAnsiString(AnsiEnv, NULL, 0);
//
//         /* Free the environment copy */
//         RegionSize = 0;
//         Status = NtFreeVirtualMemory(NtCurrentProcess(),
//                                      (PVOID*)&NewEnvironment,
//                                      &RegionSize,
//                                      MEM_RELEASE);
//         ASSERT(NT_SUCCESS(Status));
//     }
//
//     /* Return the result */
//     return Success;
// }
//
// BOOL
// NTAPI
// BaseDestroyVDMEnvironment(IN PANSI_STRING AnsiEnv,
//                           IN PUNICODE_STRING UnicodeEnv)
// {
//     SIZE_T Dummy = 0;
//
//     /* Clear the ANSI buffer since Rtl creates this for us */
//     if (AnsiEnv->Buffer) RtlFreeAnsiString(AnsiEnv);
//
//     /* The Unicode buffer is build by hand, though */
//     if (UnicodeEnv->Buffer)
//     {
//         /* So clear it through the API */
//         NtFreeVirtualMemory(NtCurrentProcess(),
//                             (PVOID*)&UnicodeEnv->Buffer,
//                             &Dummy,
//                             MEM_RELEASE);
//     }
//
//     /* All done */
//     return TRUE;
// }
//
//
// /* Check whether a file is an OS/2 or a very old Windows executable
//  * by testing on import of KERNEL.
//  *
//  * FIXME: is reading the module imports the only way of discerning
//  *        old Windows binaries from OS/2 ones ? At least it seems so...
//  */
// static DWORD WINAPI
// InternalIsOS2OrOldWin(HANDLE hFile, IMAGE_DOS_HEADER *mz, IMAGE_OS2_HEADER *ne)
// {
//   DWORD CurPos;
//   LPWORD modtab = NULL;
//   LPSTR nametab = NULL;
//   DWORD Read, Ret;
//   int i;
//
//   Ret = BINARY_OS216;
//   CurPos = SetFilePointer(hFile, 0, NULL, FILE_CURRENT);
//
//   /* read modref table */
//   if((SetFilePointer(hFile, mz->e_lfanew + ne->ne_modtab, NULL, FILE_BEGIN) == INVALID_SET_FILE_POINTER) ||
//      (!(modtab = HeapAlloc(GetProcessHeap(), 0, ne->ne_cmod * sizeof(WORD)))) ||
//      (!(ReadFile(hFile, modtab, ne->ne_cmod * sizeof(WORD), &Read, NULL))) ||
//      (Read != (DWORD)ne->ne_cmod * sizeof(WORD)))
//   {
//     goto broken;
//   }
//
//   /* read imported names table */
//   if((SetFilePointer(hFile, mz->e_lfanew + ne->ne_imptab, NULL, FILE_BEGIN) == INVALID_SET_FILE_POINTER) ||
//      (!(nametab = HeapAlloc(GetProcessHeap(), 0, ne->ne_enttab - ne->ne_imptab))) ||
//      (!(ReadFile(hFile, nametab, ne->ne_enttab - ne->ne_imptab, &Read, NULL))) ||
//      (Read != (DWORD)ne->ne_enttab - ne->ne_imptab))
//   {
//     goto broken;
//   }
//
//   for(i = 0; i < ne->ne_cmod; i++)
//   {
//     LPSTR module;
//     module = &nametab[modtab[i]];
//     if(!strncmp(&module[1], "KERNEL", module[0]))
//     {
//       /* very old windows file */
//       Ret = BINARY_WIN16;
//       goto done;
//     }
//   }
//
//   broken:
//   DPRINT1("InternalIsOS2OrOldWin(): Binary file seems to be broken\n");
//
//   done:
//   HeapFree(GetProcessHeap(), 0, modtab);
//   HeapFree(GetProcessHeap(), 0, nametab);
//   SetFilePointer(hFile, CurPos, NULL, FILE_BEGIN);
//   return Ret;
// }
//
// static DWORD WINAPI
// InternalGetBinaryType(HANDLE hFile)
// {
//   union
//   {
//     struct
//     {
//       unsigned char magic[4];
//       unsigned char ignored[12];
//       unsigned short type;
//     } elf;
//     struct
//     {
//       unsigned long magic;
//       unsigned long cputype;
//       unsigned long cpusubtype;
//       unsigned long filetype;
//     } macho;
//     IMAGE_DOS_HEADER mz;
//   } Header;
//   char magic[4];
//   DWORD Read;
//
//   if((SetFilePointer(hFile, 0, NULL, FILE_BEGIN) == INVALID_SET_FILE_POINTER) ||
//      (!ReadFile(hFile, &Header, sizeof(Header), &Read, NULL) ||
//       (Read != sizeof(Header))))
//   {
//     return BINARY_UNKNOWN;
//   }
//
//   if(!memcmp(Header.elf.magic, "\177ELF", sizeof(Header.elf.magic)))
//   {
//     /* FIXME: we don't bother to check byte order, architecture, etc. */
//     switch(Header.elf.type)
//     {
//       case 2:
//         return BINARY_UNIX_EXE;
//       case 3:
//         return BINARY_UNIX_LIB;
//     }
//     return BINARY_UNKNOWN;
//   }
//
//   /* Mach-o File with Endian set to Big Endian or Little Endian*/
//   if(Header.macho.magic == 0xFEEDFACE ||
//      Header.macho.magic == 0xCEFAEDFE)
//   {
//     switch(Header.macho.filetype)
//     {
//       case 0x8:
//         /* MH_BUNDLE */
//         return BINARY_UNIX_LIB;
//     }
//     return BINARY_UNKNOWN;
//   }
//
//   /* Not ELF, try DOS */
//   if(Header.mz.e_magic == IMAGE_DOS_SIGNATURE)
//   {
//     /* We do have a DOS image so we will now try to seek into
//      * the file by the amount indicated by the field
//      * "Offset to extended header" and read in the
//      * "magic" field information at that location.
//      * This will tell us if there is more header information
//      * to read or not.
//      */
//     if((SetFilePointer(hFile, Header.mz.e_lfanew, NULL, FILE_BEGIN) == INVALID_SET_FILE_POINTER) ||
//        (!ReadFile(hFile, magic, sizeof(magic), &Read, NULL) ||
//         (Read != sizeof(magic))))
//     {
//       return BINARY_DOS;
//     }
//
//     /* Reading the magic field succeeded so
//      * we will try to determine what type it is.
//      */
//     if(!memcmp(magic, "PE\0\0", sizeof(magic)))
//     {
//       IMAGE_FILE_HEADER FileHeader;
//       if(!ReadFile(hFile, &FileHeader, sizeof(IMAGE_FILE_HEADER), &Read, NULL) ||
//          (Read != sizeof(IMAGE_FILE_HEADER)))
//       {
//         return BINARY_DOS;
//       }
//
//       /* FIXME - detect 32/64 bit */
//
//       if(FileHeader.Characteristics & IMAGE_FILE_DLL)
//         return BINARY_PE_DLL32;
//       return BINARY_PE_EXE32;
//     }
//
//     if(!memcmp(magic, "NE", 2))
//     {
//       /* This is a Windows executable (NE) header.  This can
//        * mean either a 16-bit OS/2 or a 16-bit Windows or even a
//        * DOS program (running under a DOS extender).  To decide
//        * which, we'll have to read the NE header.
//        */
//       IMAGE_OS2_HEADER ne;
//       if((SetFilePointer(hFile, Header.mz.e_lfanew, NULL, FILE_BEGIN) == 1) ||
//          !ReadFile(hFile, &ne, sizeof(IMAGE_OS2_HEADER), &Read, NULL) ||
//          (Read != sizeof(IMAGE_OS2_HEADER)))
//       {
//         /* Couldn't read header, so abort. */
//         return BINARY_DOS;
//       }
//
//       switch(ne.ne_exetyp)
//       {
//         case 2:
//           return BINARY_WIN16;
//         case 5:
//           return BINARY_DOS;
//         default:
//           return InternalIsOS2OrOldWin(hFile, &Header.mz, &ne);
//       }
//     }
//     return BINARY_DOS;
//   }
//   return BINARY_UNKNOWN;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetBinaryTypeW (
//     LPCWSTR lpApplicationName,
//     LPDWORD lpBinaryType
//     )
// {
//   HANDLE hFile;
//   DWORD BinType;
//
//   if(!lpApplicationName || !lpBinaryType)
//   {
//     SetLastError(ERROR_INVALID_PARAMETER);
//     return FALSE;
//   }
//
//   hFile = CreateFileW(lpApplicationName, GENERIC_READ, FILE_SHARE_READ, NULL,
//                       OPEN_EXISTING, 0, 0);
//   if(hFile == INVALID_HANDLE_VALUE)
//   {
//     return FALSE;
//   }
//
//   BinType = InternalGetBinaryType(hFile);
//   CloseHandle(hFile);
//
//   switch(BinType)
//   {
//     case BINARY_UNKNOWN:
//     {
//       WCHAR *dot;
//
//       /*
//        * guess from filename
//        */
//       if(!(dot = wcsrchr(lpApplicationName, L'.')))
//       {
//         return FALSE;
//       }
//       if(!lstrcmpiW(dot, L".COM"))
//       {
//         *lpBinaryType = SCS_DOS_BINARY;
//         return TRUE;
//       }
//       if(!lstrcmpiW(dot, L".PIF"))
//       {
//         *lpBinaryType = SCS_PIF_BINARY;
//         return TRUE;
//       }
//       return FALSE;
//     }
//     case BINARY_PE_EXE32:
//     case BINARY_PE_DLL32:
//     {
//       *lpBinaryType = SCS_32BIT_BINARY;
//       return TRUE;
//     }
//     case BINARY_PE_EXE64:
//     case BINARY_PE_DLL64:
//     {
//       *lpBinaryType = SCS_64BIT_BINARY;
//       return TRUE;
//     }
//     case BINARY_WIN16:
//     {
//       *lpBinaryType = SCS_WOW_BINARY;
//       return TRUE;
//     }
//     case BINARY_OS216:
//     {
//       *lpBinaryType = SCS_OS216_BINARY;
//       return TRUE;
//     }
//     case BINARY_DOS:
//     {
//       *lpBinaryType = SCS_DOS_BINARY;
//       return TRUE;
//     }
//     case BINARY_UNIX_EXE:
//     case BINARY_UNIX_LIB:
//     {
//       return FALSE;
//     }
//   }
//
//   DPRINT1("Invalid binary type %lu returned!\n", BinType);
//   return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetBinaryTypeA(IN LPCSTR lpApplicationName,
//                OUT LPDWORD lpBinaryType)
// {
//     ANSI_STRING ApplicationNameString;
//     UNICODE_STRING ApplicationNameW;
//     BOOL StringAllocated = FALSE, Result;
//     NTSTATUS Status;
//
//     RtlInitAnsiString(&ApplicationNameString, lpApplicationName);
//
//     if (ApplicationNameString.Length * sizeof(WCHAR) >= NtCurrentTeb()->StaticUnicodeString.MaximumLength)
//     {
//         StringAllocated = TRUE;
//         Status = RtlAnsiStringToUnicodeString(&ApplicationNameW, &ApplicationNameString, TRUE);
//     }
//     else
//     {
//         Status = RtlAnsiStringToUnicodeString(&(NtCurrentTeb()->StaticUnicodeString), &ApplicationNameString, FALSE);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     if (StringAllocated)
//     {
//         Result = GetBinaryTypeW(ApplicationNameW.Buffer, lpBinaryType);
//         RtlFreeUnicodeString(&ApplicationNameW);
//     }
//     else
//     {
//         Result = GetBinaryTypeW(NtCurrentTeb()->StaticUnicodeString.Buffer, lpBinaryType);
//     }
//
//     return Result;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// CmdBatNotification (
//     DWORD   Unknown
//     )
// {
//     STUB;
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// ExitVDM(BOOL IsWow, ULONG iWowTask)
// {
//     BASE_API_MESSAGE ApiMessage;
//     PBASE_EXIT_VDM ExitVdm = &ApiMessage.Data.ExitVDMRequest;
//
//     /* Setup the input parameters */
//     ExitVdm->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     ExitVdm->iWowTask = IsWow ? iWowTask : 0; /* Always zero for DOS tasks */
//     ExitVdm->WaitObjectForVDM = NULL;
//
//     /* Call CSRSS */
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepExitVDM),
//                         sizeof(*ExitVdm));
//
//     /* Close the returned wait object handle, if any */
//     if (NT_SUCCESS(ApiMessage.Status) && (ExitVdm->WaitObjectForVDM != NULL))
//     {
//         CloseHandle(ExitVdm->WaitObjectForVDM);
//     }
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetNextVDMCommand(PVDM_COMMAND_INFO CommandData)
// {
//     BOOL Success = FALSE;
//     NTSTATUS Status;
//     BASE_API_MESSAGE ApiMessage;
//     PBASE_GET_NEXT_VDM_COMMAND GetNextVdmCommand = &ApiMessage.Data.GetNextVDMCommandRequest;
//     PBASE_IS_FIRST_VDM IsFirstVdm = &ApiMessage.Data.IsFirstVDMRequest;
//     PBASE_SET_REENTER_COUNT SetReenterCount = &ApiMessage.Data.SetReenterCountRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer = NULL;
//     ULONG NumStrings = 0;
//
//     /*
//      * Special case to test whether the VDM is the first one.
//      */
//     if (CommandData == NULL)
//     {
//         /* Call CSRSS */
//         CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                             NULL,
//                             CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepIsFirstVDM),
//                             sizeof(*IsFirstVdm));
//         if (!NT_SUCCESS(ApiMessage.Status))
//         {
//             BaseSetLastNTError(ApiMessage.Status);
//             return FALSE;
//         }
//
//         /* Return TRUE if this is the first VDM */
//         return IsFirstVdm->FirstVDM;
//     }
//
//     /* CommandData != NULL */
//
//     /*
//      * Special case to increment or decrement the reentrancy count.
//      */
//     if ((CommandData->VDMState == VDM_INC_REENTER_COUNT) ||
//         (CommandData->VDMState == VDM_DEC_REENTER_COUNT))
//     {
//         /* Setup the input parameters */
//         SetReenterCount->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//         SetReenterCount->fIncDec = CommandData->VDMState;
//
//         /* Call CSRSS */
//         CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                             NULL,
//                             CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepSetReenterCount),
//                             sizeof(*SetReenterCount));
//         if (!NT_SUCCESS(ApiMessage.Status))
//         {
//             BaseSetLastNTError(ApiMessage.Status);
//             return FALSE;
//         }
//
//         return TRUE;
//     }
//
//     /*
//      * TODO!
//      * Special case to retrieve or set WOW information.
//      */
//     // TODO: if CommandData->VDMState & (VDM_LIST_WOW_PROCESSES | VDM_LIST_WOW_TASKS | VDM_ADD_WOW_TASK)
//     // then call BasepGetNextVDMCommand in a simpler way!
//
//     /*
//      * Regular case.
//      */
//
//     /* Clear the structure */
//     RtlZeroMemory(GetNextVdmCommand, sizeof(*GetNextVdmCommand));
//
//     /* Setup the input parameters */
//     GetNextVdmCommand->iTask = CommandData->TaskId;
//     GetNextVdmCommand->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     GetNextVdmCommand->CmdLen = CommandData->CmdLen;
//     GetNextVdmCommand->AppLen = CommandData->AppLen;
//     GetNextVdmCommand->PifLen = CommandData->PifLen;
//     GetNextVdmCommand->CurDirectoryLen = CommandData->CurDirectoryLen;
//     GetNextVdmCommand->EnvLen = CommandData->EnvLen;
//     GetNextVdmCommand->DesktopLen = CommandData->DesktopLen;
//     GetNextVdmCommand->TitleLen = CommandData->TitleLen;
//     GetNextVdmCommand->ReservedLen = CommandData->ReservedLen;
//     GetNextVdmCommand->VDMState = CommandData->VDMState;
//
//     /* Count the number of strings */
//     if (CommandData->CmdLen) NumStrings++;
//     if (CommandData->AppLen) NumStrings++;
//     if (CommandData->PifLen) NumStrings++;
//     if (CommandData->CurDirectoryLen) NumStrings++;
//     if (CommandData->EnvLen) NumStrings++;
//     if (CommandData->DesktopLen) NumStrings++;
//     if (CommandData->TitleLen) NumStrings++;
//     if (CommandData->ReservedLen) NumStrings++;
//
//     /* Allocate the capture buffer */
//     CaptureBuffer = CsrAllocateCaptureBuffer(NumStrings + 1,
//                                              GetNextVdmCommand->CmdLen
//                                              + GetNextVdmCommand->AppLen
//                                              + GetNextVdmCommand->PifLen
//                                              + GetNextVdmCommand->CurDirectoryLen
//                                              + GetNextVdmCommand->EnvLen
//                                              + GetNextVdmCommand->DesktopLen
//                                              + GetNextVdmCommand->TitleLen
//                                              + GetNextVdmCommand->ReservedLen
//                                              + sizeof(*GetNextVdmCommand->StartupInfo));
//     if (CaptureBuffer == NULL)
//     {
//         BaseSetLastNTError(STATUS_NO_MEMORY);
//         goto Cleanup;
//     }
//
//     /* Capture the data */
//
//     CsrAllocateMessagePointer(CaptureBuffer,
//                               sizeof(*GetNextVdmCommand->StartupInfo),
//                               (PVOID*)&GetNextVdmCommand->StartupInfo);
//
//     if (CommandData->CmdLen)
//     {
//         CsrAllocateMessagePointer(CaptureBuffer,
//                                   CommandData->CmdLen,
//                                   (PVOID*)&GetNextVdmCommand->CmdLine);
//     }
//
//     if (CommandData->AppLen)
//     {
//         CsrAllocateMessagePointer(CaptureBuffer,
//                                   CommandData->AppLen,
//                                   (PVOID*)&GetNextVdmCommand->AppName);
//     }
//
//     if (CommandData->PifLen)
//     {
//         CsrAllocateMessagePointer(CaptureBuffer,
//                                   CommandData->PifLen,
//                                   (PVOID*)&GetNextVdmCommand->PifFile);
//     }
//
//     if (CommandData->CurDirectoryLen)
//     {
//         CsrAllocateMessagePointer(CaptureBuffer,
//                                   CommandData->CurDirectoryLen,
//                                   (PVOID*)&GetNextVdmCommand->CurDirectory);
//     }
//
//     if (CommandData->EnvLen)
//     {
//         CsrAllocateMessagePointer(CaptureBuffer,
//                                   CommandData->EnvLen,
//                                   (PVOID*)&GetNextVdmCommand->Env);
//     }
//
//     if (CommandData->DesktopLen)
//     {
//         CsrAllocateMessagePointer(CaptureBuffer,
//                                   CommandData->DesktopLen,
//                                   (PVOID*)&GetNextVdmCommand->Desktop);
//     }
//
//     if (CommandData->TitleLen)
//     {
//         CsrAllocateMessagePointer(CaptureBuffer,
//                                   CommandData->TitleLen,
//                                   (PVOID*)&GetNextVdmCommand->Title);
//     }
//
//     if (CommandData->ReservedLen)
//     {
//         CsrAllocateMessagePointer(CaptureBuffer,
//                                   CommandData->ReservedLen,
//                                   (PVOID*)&GetNextVdmCommand->Reserved);
//     }
//
//     while (TRUE)
//     {
//         /* Call CSRSS */
//         Status = CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                                      CaptureBuffer,
//                                      CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepGetNextVDMCommand),
//                                      sizeof(*GetNextVdmCommand));
//
//         /* Exit the waiting loop if we did not receive any event handle */
//         if (GetNextVdmCommand->WaitObjectForVDM == NULL)
//             break;
//
//         /* Wait for the event to become signaled and try again */
//         Status = NtWaitForSingleObject(GetNextVdmCommand->WaitObjectForVDM,
//                                        FALSE, NULL);
//         if (Status != STATUS_SUCCESS)
//         {
//             /* Fail if we timed out, or if some other error happened */
//             BaseSetLastNTError(Status);
//             goto Cleanup;
//         }
//
//         /* Set the retry flag, clear the exit code, and retry a query */
//         GetNextVdmCommand->VDMState |= VDM_FLAG_RETRY;
//         GetNextVdmCommand->ExitCode = 0;
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (Status == STATUS_INVALID_PARAMETER)
//         {
//             /*
//              * One of the buffer lengths was less than required. Store the correct ones.
//              * Note that the status code is not STATUS_BUFFER_TOO_SMALL as one would expect,
//              * in order to keep compatibility with Windows 2003 BASESRV.DLL.
//              */
//             CommandData->CmdLen = GetNextVdmCommand->CmdLen;
//             CommandData->AppLen = GetNextVdmCommand->AppLen;
//             CommandData->PifLen = GetNextVdmCommand->PifLen;
//             CommandData->CurDirectoryLen = GetNextVdmCommand->CurDirectoryLen;
//             CommandData->EnvLen      = GetNextVdmCommand->EnvLen;
//             CommandData->DesktopLen  = GetNextVdmCommand->DesktopLen;
//             CommandData->TitleLen    = GetNextVdmCommand->TitleLen;
//             CommandData->ReservedLen = GetNextVdmCommand->ReservedLen;
//         }
//         else
//         {
//             /* Any other failure */
//             CommandData->CmdLen = 0;
//             CommandData->AppLen = 0;
//             CommandData->PifLen = 0;
//             CommandData->CurDirectoryLen = 0;
//             CommandData->EnvLen      = 0;
//             CommandData->DesktopLen  = 0;
//             CommandData->TitleLen    = 0;
//             CommandData->ReservedLen = 0;
//         }
//
//         BaseSetLastNTError(Status);
//         goto Cleanup;
//     }
//
//     /* Write back the standard handles */
//     CommandData->StdIn  = GetNextVdmCommand->StdIn;
//     CommandData->StdOut = GetNextVdmCommand->StdOut;
//     CommandData->StdErr = GetNextVdmCommand->StdErr;
//
//     /* Write back the startup info */
//     RtlMoveMemory(&CommandData->StartupInfo,
//                   GetNextVdmCommand->StartupInfo,
//                   sizeof(*GetNextVdmCommand->StartupInfo));
//
//     if (CommandData->CmdLen)
//     {
//         /* Write back the command line */
//         RtlMoveMemory(CommandData->CmdLine,
//                       GetNextVdmCommand->CmdLine,
//                       GetNextVdmCommand->CmdLen);
//
//         /* Set the actual length */
//         CommandData->CmdLen = GetNextVdmCommand->CmdLen;
//     }
//
//     if (CommandData->AppLen)
//     {
//         /* Write back the application name */
//         RtlMoveMemory(CommandData->AppName,
//                       GetNextVdmCommand->AppName,
//                       GetNextVdmCommand->AppLen);
//
//         /* Set the actual length */
//         CommandData->AppLen = GetNextVdmCommand->AppLen;
//     }
//
//     if (CommandData->PifLen)
//     {
//         /* Write back the PIF file name */
//         RtlMoveMemory(CommandData->PifFile,
//                       GetNextVdmCommand->PifFile,
//                       GetNextVdmCommand->PifLen);
//
//         /* Set the actual length */
//         CommandData->PifLen = GetNextVdmCommand->PifLen;
//     }
//
//     if (CommandData->CurDirectoryLen)
//     {
//         /* Write back the current directory */
//         RtlMoveMemory(CommandData->CurDirectory,
//                       GetNextVdmCommand->CurDirectory,
//                       GetNextVdmCommand->CurDirectoryLen);
//
//         /* Set the actual length */
//         CommandData->CurDirectoryLen = GetNextVdmCommand->CurDirectoryLen;
//     }
//
//     if (CommandData->EnvLen)
//     {
//         /* Write back the environment */
//         RtlMoveMemory(CommandData->Env,
//                       GetNextVdmCommand->Env,
//                       GetNextVdmCommand->EnvLen);
//
//         /* Set the actual length */
//         CommandData->EnvLen = GetNextVdmCommand->EnvLen;
//     }
//
//     if (CommandData->DesktopLen)
//     {
//         /* Write back the desktop name */
//         RtlMoveMemory(CommandData->Desktop,
//                       GetNextVdmCommand->Desktop,
//                       GetNextVdmCommand->DesktopLen);
//
//         /* Set the actual length */
//         CommandData->DesktopLen = GetNextVdmCommand->DesktopLen;
//     }
//
//     if (CommandData->TitleLen)
//     {
//         /* Write back the title */
//         RtlMoveMemory(CommandData->Title,
//                       GetNextVdmCommand->Title,
//                       GetNextVdmCommand->TitleLen);
//
//         /* Set the actual length */
//         CommandData->TitleLen = GetNextVdmCommand->TitleLen;
//     }
//
//     if (CommandData->ReservedLen)
//     {
//         /* Write back the reserved parameter */
//         RtlMoveMemory(CommandData->Reserved,
//                       GetNextVdmCommand->Reserved,
//                       GetNextVdmCommand->ReservedLen);
//
//         /* Set the actual length */
//         CommandData->ReservedLen = GetNextVdmCommand->ReservedLen;
//     }
//
//     /* Write the remaining output parameters */
//     CommandData->TaskId        = GetNextVdmCommand->iTask;
//     CommandData->CreationFlags = GetNextVdmCommand->dwCreationFlags;
//     CommandData->CodePage      = GetNextVdmCommand->CodePage;
//     CommandData->ExitCode      = GetNextVdmCommand->ExitCode;
//     CommandData->CurrentDrive  = GetNextVdmCommand->CurrentDrive;
//     CommandData->VDMState      = GetNextVdmCommand->VDMState;
//     CommandData->ComingFromBat = GetNextVdmCommand->fComingFromBat;
//
//     /* It was successful */
//     Success = TRUE;
//
// Cleanup:
//     if (CaptureBuffer != NULL) CsrFreeCaptureBuffer(CaptureBuffer);
//     return Success;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetVDMCurrentDirectories(DWORD cchCurDirs, PCHAR lpszzCurDirs)
// {
//     BASE_API_MESSAGE ApiMessage;
//     PBASE_GETSET_VDM_CURDIRS VDMCurrentDirsRequest = &ApiMessage.Data.VDMCurrentDirsRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer;
//
//     /* Allocate the capture buffer */
//     CaptureBuffer = CsrAllocateCaptureBuffer(1, cchCurDirs);
//     if (CaptureBuffer == NULL)
//     {
//         BaseSetLastNTError(STATUS_NO_MEMORY);
//         return 0;
//     }
//
//     /* Setup the input parameters */
//     VDMCurrentDirsRequest->cchCurDirs = cchCurDirs;
//     CsrAllocateMessagePointer(CaptureBuffer,
//                               cchCurDirs,
//                               (PVOID*)&VDMCurrentDirsRequest->lpszzCurDirs);
//
//     /* Call CSRSS */
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepGetVDMCurDirs),
//                         sizeof(*VDMCurrentDirsRequest));
//
//     /* Set the last error */
//     BaseSetLastNTError(ApiMessage.Status);
//
//     if (NT_SUCCESS(ApiMessage.Status))
//     {
//         /* Copy the result */
//         RtlMoveMemory(lpszzCurDirs, VDMCurrentDirsRequest->lpszzCurDirs, cchCurDirs);
//     }
//
//     /* Free the capture buffer */
//     CsrFreeCaptureBuffer(CaptureBuffer);
//
//     /* Return the size if it was successful, or if the buffer was too small */
//     return (NT_SUCCESS(ApiMessage.Status) || (ApiMessage.Status == STATUS_BUFFER_TOO_SMALL))
//            ? VDMCurrentDirsRequest->cchCurDirs : 0;
// }
//
//
// /*
//  * @implemented (undocumented)
//  */
// BOOL
// WINAPI
// RegisterConsoleVDM(IN DWORD dwRegisterFlags,
//                    IN HANDLE hStartHardwareEvent,
//                    IN HANDLE hEndHardwareEvent,
//                    IN HANDLE hErrorHardwareEvent,
//                    IN DWORD dwUnusedVar,
//                    OUT LPDWORD lpVideoStateLength,
//                    OUT PVOID* lpVideoState, // PVIDEO_HARDWARE_STATE_HEADER*
//                    IN PVOID lpUnusedBuffer,
//                    IN DWORD dwUnusedBufferLength,
//                    IN COORD dwVDMBufferSize,
//                    OUT PVOID* lpVDMBuffer)
// {
//     BOOL Success;
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_REGISTERVDM RegisterVDMRequest = &ApiMessage.Data.RegisterVDMRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer = NULL;
//
//     /* Set up the data to send to the Console Server */
//     RegisterVDMRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     RegisterVDMRequest->RegisterFlags = dwRegisterFlags;
//
//     if (dwRegisterFlags != 0)
//     {
//         RegisterVDMRequest->StartHardwareEvent = hStartHardwareEvent;
//         RegisterVDMRequest->EndHardwareEvent   = hEndHardwareEvent;
//         RegisterVDMRequest->ErrorHardwareEvent = hErrorHardwareEvent;
//
//         RegisterVDMRequest->VDMBufferSize = dwVDMBufferSize;
//
//     }
//     else
//     {
//         // CaptureBuffer = NULL;
//     }
//
//     /* Call the server */
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepRegisterVDM),
//                         sizeof(*RegisterVDMRequest));
//
//     /* Check for success */
//     Success = NT_SUCCESS(ApiMessage.Status);
//
//     /* Release the capture buffer if needed */
//     if (CaptureBuffer) CsrFreeCaptureBuffer(CaptureBuffer);
//
//     /* Retrieve the results */
//     if (Success)
//     {
//         if (dwRegisterFlags != 0)
//         {
//             _SEH2_TRY
//             {
//                 *lpVideoStateLength = RegisterVDMRequest->VideoStateLength;
//                 *lpVideoState       = RegisterVDMRequest->VideoState;
//                 *lpVDMBuffer        = RegisterVDMRequest->VDMBuffer;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 SetLastError(ERROR_INVALID_ACCESS);
//                 Success = FALSE;
//             }
//             _SEH2_END;
//         }
//     }
//     else
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//     }
//
//     /* Return success status */
//     return Success;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// RegisterWowBaseHandlers (
//     DWORD   Unknown0
//     )
// {
//     STUB;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// RegisterWowExec (
//     DWORD   Unknown0
//     )
// {
//     STUB;
//     return FALSE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetVDMCurrentDirectories(DWORD cchCurDirs, PCHAR lpszzCurDirs)
// {
//     BASE_API_MESSAGE ApiMessage;
//     PBASE_GETSET_VDM_CURDIRS VDMCurrentDirsRequest = &ApiMessage.Data.VDMCurrentDirsRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer;
//
//     /* Allocate the capture buffer */
//     CaptureBuffer = CsrAllocateCaptureBuffer(1, cchCurDirs);
//     if (CaptureBuffer == NULL)
//     {
//         BaseSetLastNTError(STATUS_NO_MEMORY);
//         return FALSE;
//     }
//
//     /* Setup the input parameters */
//     VDMCurrentDirsRequest->cchCurDirs = cchCurDirs;
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             lpszzCurDirs,
//                             cchCurDirs,
//                             (PVOID*)&VDMCurrentDirsRequest->lpszzCurDirs);
//
//     /* Call CSRSS */
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepSetVDMCurDirs),
//                         sizeof(*VDMCurrentDirsRequest));
//
//     /* Free the capture buffer */
//     CsrFreeCaptureBuffer(CaptureBuffer);
//
//     /* Set the last error */
//     BaseSetLastNTError(ApiMessage.Status);
//
//     return NT_SUCCESS(ApiMessage.Status) ? TRUE : FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// VDMConsoleOperation(
//      DWORD iFunction,
//      LPVOID lpData)
// {
//     STUB;
//     return 0;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// VDMOperationStarted(IN ULONG Unknown0)
// {
//     DPRINT1("VDMOperationStarted(%d)\n", Unknown0);
//
//     return BaseUpdateVDMEntry(VdmEntryUpdateControlCHandler,
//                               NULL,
//                               0,
//                               Unknown0);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
