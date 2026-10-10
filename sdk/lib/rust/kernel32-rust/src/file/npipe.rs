use winedebug_rust::{TRACE, WARN};
use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;
use crate::k32_externs::{
    BaseSetLastNTError, SetLastError, RtlGetProcessHeap, RtlCopyMemory,
    Basep8BitStringToDynamicUnicodeString, INVALID_HANDLE_VALUE,
    CloseHandle, CreateFileW, GetProcAddress, LoadLibraryW,
};
use crate::string::nls_subst::NT_SUCCESS;

use crate::client::loader::FreeLibrary;

// FSCTL named-pipe control codes — CTL_CODE(FILE_DEVICE_NAMED_PIPE=0x11, fn, method,
// access). Values computed from the macro (sdk/include/ndk/iotypes.h), not placeholders:
//   LISTEN  = CTL_CODE(0x11,2,BUFFERED,ANY)        = 0x110008
//   DISCONNECT = CTL_CODE(0x11,1,BUFFERED,ANY)     = 0x110004
//   PEEK    = CTL_CODE(0x11,3,BUFFERED,READ_DATA)  = 0x11400C
//   TRANSCEIVE = CTL_CODE(0x11,5,NEITHER,READ|WRITE) = 0x11C017
const FSCTL_PIPE_LISTEN: ULONG = 0x0011_0008;
const FSCTL_PIPE_DISCONNECT: ULONG = 0x0011_0004;
const FSCTL_PIPE_PEEK: ULONG = 0x0011_400C;
const FSCTL_PIPE_TRANSCEIVE: ULONG = 0x0011_C017;
// FILE_GENERIC_WRITE = STANDARD_RIGHTS_WRITE|FILE_WRITE_DATA|FILE_WRITE_ATTRIBUTES
//   |FILE_WRITE_EA|FILE_APPEND_DATA|SYNCHRONIZE (sdk/include/xdk/iotypes.h) = 0x120116
const FILE_GENERIC_WRITE: ACCESS_MASK = 0x0012_0116;

extern "system" {
    fn InterlockedIncrement(addend: *mut LONG) -> LONG;
}

// RtlExtendedLargeIntegerDivide: a real ntdll export on i386; a winnt.h
// FORCEINLINE (plain 64-bit divide) on amd64/arm64 — provide it locally there.
#[cfg(target_arch = "x86")]
extern "system" {
    fn RtlExtendedLargeIntegerDivide(dividend: LARGE_INTEGER, divisor: ULONG, remainder: *mut ULONG) -> LARGE_INTEGER;
}
#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn RtlExtendedLargeIntegerDivide(
    dividend: LARGE_INTEGER,
    divisor: ULONG,
    remainder: *mut ULONG,
) -> LARGE_INTEGER {
    let mut result: LARGE_INTEGER = core::mem::zeroed();
    result.QuadPart = dividend.QuadPart / (divisor as i64);
    if !remainder.is_null() {
        *remainder = (dividend.QuadPart % (divisor as i64)) as ULONG;
    }
    result
}

// RTL_PATH_TYPE values (sdk/include/ndk/rtltypes.h) compared as ULONG.
const RtlPathTypeUncAbsolute: ULONG = 1;
const RtlPathTypeLocalDevice: ULONG = 6;
// FSCTL_PIPE_WAIT = CTL_CODE(FILE_DEVICE_NAMED_PIPE=0x11, 6, BUFFERED, ANY) = 0x110018
const FSCTL_PIPE_WAIT: ULONG = 0x0011_0018;

extern "C" {
    fn swprintf(buffer: *mut WCHAR, format: *const WCHAR, ...) -> core::ffi::c_int;
}

// DEBUG_CHANNEL(kernel32file) TRACE/WARN — compiled with NDEBUG in the C, so these
// are no-ops. As macros they swallow the mixed (C-string / Rust-&str) call args the
// translation emitted without a type-check, matching the C #define NDEBUG behavior.

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut gDebugChannel: ULONG = 200 as ULONG;

pub static mut ProcessPipeId: LONG = 0;

#[repr(C)]
pub struct _NAMED_PIPE_CONTEXT {
    pub NamedPipeType: u32,
    pub NamedPipeConfiguration: u32,
    pub MaxInstances: u32,
    pub OutBufferSize: u32,
    pub InBufferSize: u32,
    pub DefaultTimeOut: u32,
}

pub type PNAMED_PIPE_CONTEXT = *mut _NAMED_PIPE_CONTEXT;

pub const NMPWAIT_NOWAIT: u32 = 0x00000001;
pub const NMPWAIT_WAIT_FOREVER: u32 = 0xffffffff;

pub const PIPE_UNLIMITED_INSTANCES: u32 = 255;

pub const PIPE_TYPE_BYTE: u32 = 0x00000000;
pub const PIPE_TYPE_MESSAGE: u32 = 0x00000004;

pub const PIPE_READMODE_BYTE: u32 = 0x00000000;
pub const PIPE_READMODE_MESSAGE: u32 = 0x00000002;

pub const PIPE_WAIT: u32 = 0x00000000;
pub const PIPE_NOWAIT: u32 = 0x00000001;

pub const PIPE_ACCESS_DUPLEX: u32 = 0x00000003;
pub const PIPE_ACCESS_INBOUND: u32 = 0x00000001;
pub const PIPE_ACCESS_OUTBOUND: u32 = 0x00000002;

pub const PIPE_CLIENT_END: u32 = 0x00000000;
pub const PIPE_SERVER_END: u32 = 0x00000001;

pub const PIPE_WAIT_FOR_BUFFER: u32 = 0x00000000;
pub const PIPE_COMPLETE_OPERATION: u32 = 0x00000001;

pub const PIPE_BYTE_STREAM_TYPE: u32 = 0x00000000;
pub const PIPE_MESSAGE_TYPE: u32 = 0x00000001;

pub const PIPE_BYTE_STREAM_MODE: u32 = 0x00000000;
pub const PIPE_MESSAGE_MODE: u32 = 0x00000001;

pub const PIPE_REJECT_REMOTE_CLIENTS: u32 = 0x00000008;
pub const PIPE_ACCEPT_REMOTE_CLIENTS: u32 = 0x00000000;

pub const NMPWAIT_USE_DEFAULT_WAIT: u32 = 0x00000000;
unsafe fn NpGetUserNamep(
    hNamedPipe: HANDLE,
    lpUserName: LPWSTR,
    nMaxUserNameSize: DWORD,
) -> BOOL {
    let mut Ret: BOOL = 0;
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut hAdvapi: HMODULE = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut pRevertToSelf: Option<unsafe extern "system" fn() -> BOOL> = None;
    let mut pGetUserNameW: Option<unsafe extern "system" fn(LPWSTR, LPDWORD) -> BOOL> = None;
    let mut pImpersonateNamedPipeClient: Option<unsafe extern "system" fn(HANDLE) -> BOOL> = None;

    hAdvapi = LoadLibraryW(w!("advapi32.dll\0").as_ptr() as LPCWSTR) as HMODULE;
    if hAdvapi.is_null() {
        return 0;
    }

    pRevertToSelf = core::mem::transmute::<FARPROC, Option<unsafe extern "system" fn() -> BOOL>>(
        GetProcAddress(hAdvapi as HMODULE, b"RevertToSelf\0".as_ptr() as LPCSTR),
    );
    pGetUserNameW = core::mem::transmute::<FARPROC, Option<unsafe extern "system" fn(LPWSTR, LPDWORD) -> BOOL>>(
        GetProcAddress(hAdvapi as HMODULE, b"GetUserNameW\0".as_ptr() as LPCSTR),
    );
    pImpersonateNamedPipeClient = core::mem::transmute::<FARPROC, Option<unsafe extern "system" fn(HANDLE) -> BOOL>>(
        GetProcAddress(hAdvapi as HMODULE, b"ImpersonateNamedPipeClient\0".as_ptr() as LPCSTR),
    );

    if pRevertToSelf.is_none() || pGetUserNameW.is_none() || pImpersonateNamedPipeClient.is_none() {
        FreeLibrary(hAdvapi);
        return 0;
    }

    Status = NtOpenThreadToken(
        NtCurrentThread(),
        TOKEN_IMPERSONATE as ACCESS_MASK,
        1, // TRUE
        &mut hToken as *mut HANDLE,
    );

    if pImpersonateNamedPipeClient.expect("non-null function pointer")(hNamedPipe) != 0 {
        let mut lpnSize: DWORD = 0;
        lpnSize = nMaxUserNameSize;
        Ret = pGetUserNameW.expect("non-null function pointer")(lpUserName, &mut lpnSize as *mut DWORD);

        if !NT_SUCCESS(Status) {
            pRevertToSelf.expect("non-null function pointer")();
            FreeLibrary(hAdvapi);
            return Ret;
        }

        Status = NtSetInformationThread(
            NtCurrentThread(),
            ThreadImpersonationToken,
            &mut hToken as *mut HANDLE as PVOID,
            core::mem::size_of::<HANDLE>() as ULONG,
        );

        if CloseHandle(hToken) == 0 {
            // ASSERT(FALSE) - debug assertion
            if cfg!(debug_assertions) {
                // RtlAssert would go here
            }
        }

        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
        }
    } else {
        if NT_SUCCESS(Status) {
            if CloseHandle(hToken) == 0 {
                // ASSERT(FALSE) - debug assertion
                if cfg!(debug_assertions) {
                    // RtlAssert would go here
                }
            }
        }
        Ret = 0;
    }

    FreeLibrary(hAdvapi);
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn CreatePipe(
    hReadPipe: PHANDLE,
    hWritePipe: PHANDLE,
    lpPipeAttributes: LPSECURITY_ATTRIBUTES,
    mut nSize: DWORD,
) -> BOOL {
    let mut Buffer: [WCHAR; 64] = [0; 64];
    let mut PipeName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut StatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DefaultTimeout: LARGE_INTEGER = LARGE_INTEGER {
        QuadPart: 0,
    };
    let mut Status: NTSTATUS = 0;
    let mut ReadPipeHandle: HANDLE = core::ptr::null_mut();
    let mut WritePipeHandle: HANDLE = core::ptr::null_mut();
    let mut PipeId: LONG = 0;
    let mut Attributes: ULONG = 0;
    let mut SecurityDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();

    DefaultTimeout.QuadPart = -1200000000i64;

    if nSize == 0 {
        nSize = 0x1000;
    }

    PipeId = InterlockedIncrement(&raw mut ProcessPipeId) as LONG;

    swprintf(
        Buffer.as_mut_ptr(),
        w!("\\Device\\NamedPipe\\Win32Pipes.%p.%08x\0").as_ptr() as *const u16,
        (*NtCurrentTeb()).ClientId.UniqueProcess,
        PipeId,
    );

    RtlInitUnicodeString(&raw mut PipeName, Buffer.as_ptr());

    Attributes = OBJ_CASE_INSENSITIVE as ULONG;

    if !lpPipeAttributes.is_null() {
        SecurityDescriptor = (*lpPipeAttributes).lpSecurityDescriptor;
        if (*lpPipeAttributes).bInheritHandle != 0 {
            Attributes |= OBJ_INHERIT as ULONG;
        }
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = Attributes;
    ObjectAttributes.ObjectName = &raw mut PipeName;
    ObjectAttributes.SecurityDescriptor = SecurityDescriptor;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateNamedPipeFile(
        &raw mut ReadPipeHandle,
        (GENERIC_READ | FILE_WRITE_ATTRIBUTES | SYNCHRONIZE) as ACCESS_MASK,
        &raw mut ObjectAttributes,
        &raw mut StatusBlock,
        (FILE_SHARE_READ | FILE_SHARE_WRITE) as ULONG,
        FILE_CREATE as ULONG,
        FILE_SYNCHRONOUS_IO_NONALERT as ULONG,
        FILE_PIPE_BYTE_STREAM_TYPE as ULONG,
        FILE_PIPE_BYTE_STREAM_MODE as ULONG,
        FILE_PIPE_QUEUE_OPERATION as ULONG,
        1 as ULONG,
        nSize as ULONG,
        nSize as ULONG,
        &raw mut DefaultTimeout,
    );

    if !NT_SUCCESS(Status) {
        WARN!(b"Status: %lx\n\0".as_ptr() as LPCSTR, Status);
        BaseSetLastNTError(Status);
        return 0;
    }

    Status = NtOpenFile(
        &raw mut WritePipeHandle,
        FILE_GENERIC_WRITE as ACCESS_MASK,
        &raw mut ObjectAttributes,
        &raw mut StatusBlock,
        FILE_SHARE_READ as ULONG,
        (FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE) as ULONG,
    );

    if !NT_SUCCESS(Status) {
        WARN!(b"Status: %lx\n\0".as_ptr() as LPCSTR, Status);
        NtClose(ReadPipeHandle);
        BaseSetLastNTError(Status);
        return 0;
    }

    *hReadPipe = ReadPipeHandle;
    *hWritePipe = WritePipeHandle;
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn CreateNamedPipeA(
    lpName: LPCSTR,
    dwOpenMode: DWORD,
    dwPipeMode: DWORD,
    nMaxInstances: DWORD,
    nOutBufferSize: DWORD,
    nInBufferSize: DWORD,
    nDefaultTimeOut: DWORD,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
) -> HANDLE {

    // Expansion of the ConvertWin32AnsiChangeApiToUnicodeApi(CreateNamedPipe, lpName, ...)
    // macro (dll/win32/kernel32/include/base_x.h): ANSI→Unicode via the TEB static
    // unicode string, then tail-call CreateNamedPipeW.
    let Status: NTSTATUS;
    let UnicodeCache: PUNICODE_STRING;
    let mut AnsiName: ANSI_STRING = core::mem::zeroed();

    UnicodeCache = &raw mut (*NtCurrentTeb()).StaticUnicodeString;
    RtlInitAnsiString(&raw mut AnsiName, lpName);
    Status = RtlAnsiStringToUnicodeString(UnicodeCache, &raw mut AnsiName, FALSE as BOOLEAN);
    if NT_SUCCESS(Status) {
        return CreateNamedPipeW(
            (*UnicodeCache).Buffer,
            dwOpenMode,
            dwPipeMode,
            nMaxInstances,
            nOutBufferSize,
            nInBufferSize,
            nDefaultTimeOut,
            lpSecurityAttributes,
        );
    }
    if Status == STATUS_BUFFER_OVERFLOW {
        SetLastError(ERROR_FILENAME_EXCED_RANGE);
    } else {
        BaseSetLastNTError(Status);
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn CreateNamedPipeW(
    lpName: LPCWSTR,
    dwOpenMode: DWORD,
    dwPipeMode: DWORD,
    mut nMaxInstances: DWORD,
    nOutBufferSize: DWORD,
    nInBufferSize: DWORD,
    nDefaultTimeOut: DWORD,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
) -> HANDLE {
    let mut NamedPipeName: _UNICODE_STRING = core::mem::zeroed();
    let mut Result: BOOL;
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut PipeHandle: HANDLE = core::ptr::null_mut();
    let mut DesiredAccess: ACCESS_MASK;
    let mut CreateOptions: ULONG = 0;
    let mut WriteModeMessage: ULONG;
    let mut ReadModeMessage: ULONG;
    let mut NonBlocking: ULONG;
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ShareAccess: ULONG = 0;
    let mut Attributes: ULONG;
    let mut DefaultTimeOut: LARGE_INTEGER = core::mem::zeroed();
    let mut SecurityDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();

    if nMaxInstances == 0 || nMaxInstances > PIPE_UNLIMITED_INSTANCES {
        SetLastError(ERROR_INVALID_PARAMETER);
        return INVALID_HANDLE_VALUE;
    }

    if nMaxInstances == PIPE_UNLIMITED_INSTANCES {
        nMaxInstances = -1i32 as DWORD;
    }

    Result = RtlDosPathNameToNtPathName_U(
        lpName,
        &mut NamedPipeName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) as BOOL;

    if Result == 0 {
        SetLastError(ERROR_PATH_NOT_FOUND);
        return INVALID_HANDLE_VALUE;
    }

    TRACE!("Pipe name: %wZ\n", &NamedPipeName);
    TRACE!("Pipe name: %S\n", NamedPipeName.Buffer);

    Attributes = OBJ_CASE_INSENSITIVE;
    if !lpSecurityAttributes.is_null() {
        SecurityDescriptor = (*lpSecurityAttributes).lpSecurityDescriptor;
        if (*lpSecurityAttributes).bInheritHandle != 0 {
            Attributes |= OBJ_INHERIT;
        }
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = Attributes;
    ObjectAttributes.ObjectName = &mut NamedPipeName;
    ObjectAttributes.SecurityDescriptor = SecurityDescriptor as PVOID;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    DesiredAccess = SYNCHRONIZE | (dwOpenMode & (WRITE_DAC | WRITE_OWNER | ACCESS_SYSTEM_SECURITY));

    if dwOpenMode & FILE_FLAG_WRITE_THROUGH != 0 {
        CreateOptions |= FILE_WRITE_THROUGH;
    }

    if dwOpenMode & FILE_FLAG_OVERLAPPED == 0 {
        CreateOptions |= FILE_SYNCHRONOUS_IO_NONALERT;
    }

    if dwOpenMode & PIPE_ACCESS_OUTBOUND != 0 {
        ShareAccess |= FILE_SHARE_READ;
        DesiredAccess |= GENERIC_WRITE;
    }

    if dwOpenMode & PIPE_ACCESS_INBOUND != 0 {
        ShareAccess |= FILE_SHARE_WRITE;
        DesiredAccess |= GENERIC_READ;
    }

    if dwPipeMode & PIPE_TYPE_MESSAGE != 0 {
        WriteModeMessage = FILE_PIPE_MESSAGE_TYPE;
    } else {
        WriteModeMessage = FILE_PIPE_BYTE_STREAM_TYPE;
    }

    if dwPipeMode & PIPE_READMODE_MESSAGE != 0 {
        ReadModeMessage = FILE_PIPE_MESSAGE_MODE;
    } else {
        ReadModeMessage = FILE_PIPE_BYTE_STREAM_MODE;
    }

    if dwPipeMode & PIPE_NOWAIT != 0 {
        NonBlocking = FILE_PIPE_COMPLETE_OPERATION;
    } else {
        NonBlocking = FILE_PIPE_QUEUE_OPERATION;
    }

    if nDefaultTimeOut != 0 {
        DefaultTimeOut.QuadPart = (nDefaultTimeOut as i64 * -10000i64) as LONGLONG;
    } else {
        DefaultTimeOut.QuadPart = -500000i64 as LONGLONG;
    }

    Status = NtCreateNamedPipeFile(
        &mut PipeHandle,
        DesiredAccess,
        &mut ObjectAttributes,
        &mut Iosb,
        ShareAccess,
        FILE_OPEN_IF,
        CreateOptions,
        WriteModeMessage,
        ReadModeMessage,
        NonBlocking,
        nMaxInstances,
        nInBufferSize,
        nOutBufferSize,
        &mut DefaultTimeOut,
    );

    if Status == STATUS_INVALID_DEVICE_REQUEST || Status == STATUS_NOT_SUPPORTED {
        Status = STATUS_OBJECT_NAME_INVALID;
    }

    RtlFreeHeap(
        RtlGetProcessHeap(),
        0,
        NamedPipeName.Buffer as PVOID,
    );

    if !NT_SUCCESS(Status) {
        WARN!("NtCreateNamedPipe failed (Status %x)!\n", Status);
        BaseSetLastNTError(Status);
        return INVALID_HANDLE_VALUE;
    }

    PipeHandle
}

#[no_mangle]
pub unsafe extern "system" fn WaitNamedPipeA(
    lpNamedPipeName: LPCSTR,
    nTimeOut: DWORD,
) -> BOOL {
    let mut r: BOOL = 0;
    let mut NameU: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    if Basep8BitStringToDynamicUnicodeString(
        core::ptr::addr_of_mut!(NameU),
        lpNamedPipeName,
    ) != 0
    {
        r = WaitNamedPipeW(NameU.Buffer as LPCWSTR, nTimeOut);
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(NameU));
    }

    return r;
}

// Faithful translation of WaitNamedPipeW (dll/win32/kernel32/client/file/npipe.c).
// The C body was dropped by the bulk translation; restored here verbatim in logic.
#[no_mangle]
pub unsafe extern "system" fn WaitNamedPipeW(
    lpNamedPipeName: LPCWSTR,
    nTimeOut: DWORD,
) -> BOOL {
    let mut NamedPipeName: _UNICODE_STRING = core::mem::zeroed();
    let mut NewName: _UNICODE_STRING;
    let mut DevicePath: _UNICODE_STRING = core::mem::zeroed();
    let mut PipePrefix: _UNICODE_STRING = core::mem::zeroed();
    let NameLength: ULONG;
    let mut i: ULONG;
    let mut p: *mut WCHAR;
    let Type: ULONG;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let WaitPipeInfoSize: ULONG;
    let mut DevicePathBuffer: PVOID;
    let WaitPipeInfo: PFILE_PIPE_WAIT_FOR_BUFFER;

    /* Start by making a unicode string of the name */
    TRACE!("Sent path");
    if RtlCreateUnicodeString(&raw mut NamedPipeName, lpNamedPipeName) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return FALSE as BOOL;
    }
    NameLength = (NamedPipeName.Length as ULONG) / (core::mem::size_of::<WCHAR>() as ULONG);

    /* All slashes must become backslashes */
    i = 0;
    while i < NameLength {
        /* Check and convert */
        if *NamedPipeName.Buffer.add(i as usize) == b'/' as WCHAR {
            *NamedPipeName.Buffer.add(i as usize) = b'\\' as WCHAR;
        }
        i += 1;
    }

    DevicePathBuffer = core::ptr::null_mut();

    /* Find the path type of the name we were given */
    NewName = NamedPipeName;
    Type = RtlDetermineDosPathNameType_U(lpNamedPipeName) as ULONG;

    /* Check if this was a device path, ie : "\\.\pipe\name" */
    if Type == RtlPathTypeLocalDevice {
        /* Make sure it's a valid prefix */
        RtlInitUnicodeString(&raw mut PipePrefix, w!("\\\\.\\pipe\\").as_ptr());
        if RtlPrefixUnicodeString(&raw const PipePrefix, &raw const NewName, TRUE as BOOLEAN) == 0 {
            /* The name is invalid */
            WARN!("Invalid name!");
            RtlFreeUnicodeString(&raw mut NamedPipeName);
            BaseSetLastNTError(STATUS_OBJECT_PATH_SYNTAX_BAD);
            return FALSE as BOOL;
        }

        /* Move past it */
        NewName.Buffer = NewName.Buffer.add(
            (PipePrefix.Length as usize) / core::mem::size_of::<WCHAR>(),
        );
        NewName.Length -= PipePrefix.Length;
        NewName.MaximumLength -= PipePrefix.Length;

        /* Initialize the Dos Devices name */
        TRACE!("NewName");
        RtlInitUnicodeString(&raw mut DevicePath, w!("\\DosDevices\\pipe\\").as_ptr());
    } else if Type == RtlPathTypeUncAbsolute {
        let PipeName: *mut WCHAR;

        /* The path is \\server\\pipe\name; find the pipename itself */
        p = NewName.Buffer.add(2);

        /* First loop to get past the server name */
        loop {
            /* Check if this is a backslash */
            if *p == b'\\' as WCHAR {
                break;
            }
            /* Check next */
            p = p.add(1);
            if *p == 0 {
                break;
            }
        }

        /* Now make sure the full name contains "pipe\" */
        // sizeof("pipe\\") - sizeof(ANSI_NULL) = 6 - 1 = 5 chars
        if *p != 0 && _wcsnicmp(p.add(1), w!("pipe\\").as_ptr(), 5) == 0 {
            /* Get to the pipe name itself now */
            // sizeof("pipe\\") - sizeof(ANSI_NULL) = 5 (byte count of "pipe\")
            p = p.add(5);
        } else {
            /* The name is invalid */
            WARN!("Invalid name!");
            RtlFreeUnicodeString(&raw mut NamedPipeName);
            BaseSetLastNTError(STATUS_OBJECT_PATH_SYNTAX_BAD);
            return FALSE as BOOL;
        }

        /* Skip first backslash */
        NewName.Buffer = NewName.Buffer.add(1);
        /* And skip pipe for copying name */
        // (sizeof(L"pipe\\") - sizeof(UNICODE_NULL)) / sizeof(WCHAR) = (12-2)/2 = 5
        PipeName = p.add(5);
        /* Update the string */
        NewName.Length =
            ((PipeName as usize) - (NewName.Buffer as usize)) as USHORT;
        NewName.MaximumLength =
            ((PipeName as usize) - (NewName.Buffer as usize)) as USHORT;

        /* DevicePath will contain the pipename + the DosDevice prefix */
        // sizeof(L"\\DosDevices\\UNC\\") = 17 wchars incl NUL = 34 bytes
        DevicePath.MaximumLength = (((PipeName as usize) - (NewName.Buffer as usize))
            + (core::mem::size_of::<[u16; 17]>())) as USHORT;

        /* Allocate the buffer for DevicePath */
        DevicePathBuffer =
            RtlAllocateHeap(RtlGetProcessHeap(), 0, DevicePath.MaximumLength as SIZE_T);
        if DevicePathBuffer.is_null() {
            RtlFreeUnicodeString(&raw mut NamedPipeName);
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return FALSE as BOOL;
        }

        /* Copy the prefix first */
        DevicePath.Buffer = DevicePathBuffer as *mut WCHAR;
        // sizeof(L"\\DosDevices\\UNC\\") - sizeof(UNICODE_NULL) = 34 - 2 = 32 bytes
        RtlCopyMemory(
            DevicePathBuffer,
            w!("\\DosDevices\\UNC\\").as_ptr() as *const c_void,
            32,
        );
        DevicePath.Length = 32;
        /* And append the rest */
        RtlAppendUnicodeStringToString(&raw mut DevicePath, &raw const NewName);
        /* And fix pipe name without its prefix */
        RtlInitUnicodeString(&raw mut NewName, PipeName.add(1));
    } else {
        WARN!("Invalid path type");
        RtlFreeUnicodeString(&raw mut NamedPipeName);
        BaseSetLastNTError(STATUS_OBJECT_PATH_SYNTAX_BAD);
        return FALSE as BOOL;
    }

    /* Now calculate the total length of the structure and allocate it */
    WaitPipeInfoSize = (core::mem::offset_of!(_FILE_PIPE_WAIT_FOR_BUFFER, Name) as ULONG)
        + (NewName.Length as ULONG);
    WaitPipeInfo =
        RtlAllocateHeap(RtlGetProcessHeap(), 0, WaitPipeInfoSize as SIZE_T) as PFILE_PIPE_WAIT_FOR_BUFFER;
    if WaitPipeInfo.is_null() {
        if !DevicePathBuffer.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, DevicePathBuffer);
        }
        RtlFreeUnicodeString(&raw mut NamedPipeName);
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return FALSE as BOOL;
    }

    /* Initialize the object attributes */
    TRACE!("Opening");
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &raw mut DevicePath;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as ULONG;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    /* Open the path */
    Status = NtOpenFile(
        &raw mut FileHandle,
        (FILE_READ_ATTRIBUTES | SYNCHRONIZE) as ACCESS_MASK,
        &raw mut ObjectAttributes,
        &raw mut IoStatusBlock,
        (FILE_SHARE_READ | FILE_SHARE_WRITE) as ULONG,
        FILE_SYNCHRONOUS_IO_NONALERT as ULONG,
    );

    if !DevicePathBuffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, DevicePathBuffer);
    }

    if !NT_SUCCESS(Status) {
        /* Fail; couldn't open */
        WARN!("Status");
        RtlFreeHeap(RtlGetProcessHeap(), 0, WaitPipeInfo as PVOID);
        RtlFreeUnicodeString(&raw mut NamedPipeName);
        BaseSetLastNTError(Status);
        return FALSE as BOOL;
    }

    /* Check what timeout we got */
    if nTimeOut == NMPWAIT_USE_DEFAULT_WAIT {
        /* Don't use a timeout */
        (*WaitPipeInfo).TimeoutSpecified = FALSE as BOOLEAN;
    } else {
        /* Check if we should wait forever */
        if nTimeOut == NMPWAIT_WAIT_FOREVER {
            /* Set the max */
            (*WaitPipeInfo).Timeout.u.LowPart = 0;
            (*WaitPipeInfo).Timeout.u.HighPart = 0x80000000u32 as LONG;
        } else {
            /* Convert to NT format */
            (*WaitPipeInfo).Timeout.QuadPart = (nTimeOut as i64) * -10000i64;
        }

        /* In both cases, we do have a timeout */
        (*WaitPipeInfo).TimeoutSpecified = TRUE as BOOLEAN;
    }

    /* Set the length and copy the name */
    (*WaitPipeInfo).NameLength = NewName.Length as ULONG;
    RtlCopyMemory(
        core::ptr::addr_of_mut!((*WaitPipeInfo).Name) as PVOID,
        NewName.Buffer as *const c_void,
        NewName.Length as SIZE_T,
    );

    /* Get rid of the full name */
    RtlFreeUnicodeString(&raw mut NamedPipeName);

    /* Let NPFS know of our request */
    Status = NtFsControlFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &raw mut IoStatusBlock,
        FSCTL_PIPE_WAIT,
        WaitPipeInfo as PVOID,
        WaitPipeInfoSize,
        core::ptr::null_mut(),
        0,
    );

    /* Free our pipe info data and close the handle */
    RtlFreeHeap(RtlGetProcessHeap(), 0, WaitPipeInfo as PVOID);
    NtClose(FileHandle);

    /* Check the status */
    if !NT_SUCCESS(Status) {
        /* Failure to wait on the pipe */
        WARN!("Status");
        BaseSetLastNTError(Status);
        return FALSE as BOOL;
    }

    /* Success */
    TRUE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn ConnectNamedPipe(
    hNamedPipe: HANDLE,
    lpOverlapped: LPOVERLAPPED,
) -> BOOL {
    let mut Status: NTSTATUS;
    if !lpOverlapped.is_null() {
        let ApcContext: PVOID;
        (*lpOverlapped).Internal = STATUS_PENDING as ULONG_PTR;
        ApcContext = if ((*lpOverlapped).hEvent as ULONG_PTR) & 1 != 0 {
            core::ptr::null_mut()
        } else {
            lpOverlapped as PVOID
        };
        Status = NtFsControlFile(
            hNamedPipe,
            (*lpOverlapped).hEvent,
            None,
            ApcContext,
            lpOverlapped as PIO_STATUS_BLOCK,
            FSCTL_PIPE_LISTEN,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
        );
        if !NT_SUCCESS(Status) || Status == STATUS_PENDING {
            BaseSetLastNTError(Status);
            return 0;
        }
    } else {
        let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
        Status = NtFsControlFile(
            hNamedPipe,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut Iosb as *mut _IO_STATUS_BLOCK,
            FSCTL_PIPE_LISTEN,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
        );
        if Status == STATUS_PENDING {
            Status = NtWaitForSingleObject(
                hNamedPipe,
                0,
                core::ptr::null_mut(),
            );
            if NT_SUCCESS(Status) {
                Status = Iosb.__bindgen_anon_1.Status;
            }
        }
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            return 0;
        }
    }
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetNamedPipeHandleState(
    hNamedPipe: HANDLE,
    lpMode: LPDWORD,
    lpMaxCollectionCount: LPDWORD,
    lpCollectDataTimeout: LPDWORD,
) -> BOOL {
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;

    /* Check if the Mode is being changed */
    if !lpMode.is_null() {
        let mut Settings: FILE_PIPE_INFORMATION = FILE_PIPE_INFORMATION {
            ReadMode: 0,
            CompletionMode: 0,
        };

        /* Set the Completion Mode */
        Settings.CompletionMode = if (*lpMode & PIPE_NOWAIT as u32) != 0 {
            FILE_PIPE_COMPLETE_OPERATION
        } else {
            FILE_PIPE_QUEUE_OPERATION
        };

        /* Set the Read Mode */
        Settings.ReadMode = if (*lpMode & PIPE_READMODE_MESSAGE as u32) != 0 {
            FILE_PIPE_MESSAGE_MODE
        } else {
            FILE_PIPE_BYTE_STREAM_MODE
        };

        /* Send the changes to the Driver */
        Status = NtSetInformationFile(
            hNamedPipe,
            &mut Iosb,
            &mut Settings as *mut _ as PVOID,
            core::mem::size_of::<FILE_PIPE_INFORMATION>() as u32,
            FilePipeInformation,
        );
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            return FALSE as BOOL;
        }
    }

    /* Check if the Collection count or Timeout are being changed */
    if !lpMaxCollectionCount.is_null() || !lpCollectDataTimeout.is_null() {
        let mut RemoteSettings: FILE_PIPE_REMOTE_INFORMATION = FILE_PIPE_REMOTE_INFORMATION {
            CollectDataTime: LARGE_INTEGER {
                QuadPart: 0,
            },
            MaximumCollectionCount: 0,
        };

        /* Setting one without the other would delete it, so we read old one */
        if lpMaxCollectionCount.is_null() || lpCollectDataTimeout.is_null() {
            Status = NtQueryInformationFile(
                hNamedPipe,
                &mut Iosb,
                &mut RemoteSettings as *mut _ as PVOID,
                core::mem::size_of::<FILE_PIPE_REMOTE_INFORMATION>() as u32,
                FilePipeRemoteInformation,
            );
            if !NT_SUCCESS(Status) {
                BaseSetLastNTError(Status);
                return FALSE as BOOL;
            }
        }

        /* Now set the new settings */
        RemoteSettings.MaximumCollectionCount = if !lpMaxCollectionCount.is_null() {
            *lpMaxCollectionCount
        } else {
            RemoteSettings.MaximumCollectionCount
        };

        if !lpCollectDataTimeout.is_null() {
            /* Convert it to Quad */
            RemoteSettings.CollectDataTime.QuadPart =
                (*lpCollectDataTimeout as i64) * -10000i64;
        }

        /* Tell the driver to change them */
        Status = NtSetInformationFile(
            hNamedPipe,
            &mut Iosb,
            &mut RemoteSettings as *mut _ as PVOID,
            core::mem::size_of::<FILE_PIPE_REMOTE_INFORMATION>() as u32,
            FilePipeRemoteInformation,
        );
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            return FALSE as BOOL;
        }
    }

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CallNamedPipeA(
    lpNamedPipeName: LPCSTR,
    lpInBuffer: LPVOID,
    nInBufferSize: DWORD,
    lpOutBuffer: LPVOID,
    nOutBufferSize: DWORD,
    lpBytesRead: LPDWORD,
    nTimeOut: DWORD,
) -> BOOL {
    let PipeName: PUNICODE_STRING = &raw mut (*NtCurrentTeb()).StaticUnicodeString;
    let mut AnsiPipe: ANSI_STRING = core::mem::zeroed();
    RtlInitAnsiString(&raw mut AnsiPipe, lpNamedPipeName as LPSTR);
    RtlAnsiStringToUnicodeString(PipeName, &raw mut AnsiPipe, 0);
    return CallNamedPipeW(
        (*PipeName).Buffer as LPCWSTR,
        lpInBuffer,
        nInBufferSize,
        lpOutBuffer,
        nOutBufferSize,
        lpBytesRead,
        nTimeOut,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CallNamedPipeW(
    lpNamedPipeName: LPCWSTR,
    lpInBuffer: LPVOID,
    nInBufferSize: DWORD,
    lpOutBuffer: LPVOID,
    nOutBufferSize: DWORD,
    lpBytesRead: LPDWORD,
    nTimeOut: DWORD,
) -> BOOL {
    let mut hPipe: HANDLE;
    let mut bRetry: BOOL = TRUE as BOOL;
    let mut bError: BOOL;
    let mut dwPipeMode: DWORD;

    loop {
        hPipe = CreateFileW(
            lpNamedPipeName,
            (GENERIC_READ | GENERIC_WRITE) as DWORD,
            (FILE_SHARE_READ | FILE_SHARE_WRITE) as DWORD,
            core::ptr::null_mut(),
            OPEN_EXISTING as DWORD,
            FILE_ATTRIBUTE_NORMAL as DWORD,
            core::ptr::null_mut(),
        );

        if hPipe != INVALID_HANDLE_VALUE {
            break;
        }

        if bRetry == FALSE as BOOL {
            return FALSE as BOOL;
        }

        WaitNamedPipeW(lpNamedPipeName, nTimeOut);

        bRetry = FALSE as BOOL;
    }

    dwPipeMode = (PIPE_READMODE_MESSAGE | PIPE_WAIT) as DWORD;
    bError = SetNamedPipeHandleState(
        hPipe,
        &raw mut dwPipeMode,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if bError == 0 {
        CloseHandle(hPipe);
        return FALSE as BOOL;
    }

    bError = TransactNamedPipe(
        hPipe,
        lpInBuffer,
        nInBufferSize,
        lpOutBuffer,
        nOutBufferSize,
        lpBytesRead,
        core::ptr::null_mut(),
    );

    CloseHandle(hPipe);

    return bError;
}

#[no_mangle]
pub unsafe extern "system" fn DisconnectNamedPipe(hNamedPipe: HANDLE) -> BOOL {
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Status = NtFsControlFile(
        hNamedPipe,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut Iosb,
        FSCTL_PIPE_DISCONNECT,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
    );

    if Status == STATUS_PENDING {
        Status = NtWaitForSingleObject(
            hNamedPipe,
            0, // FALSE
            core::ptr::null_mut(),
        );
        if NT_SUCCESS(Status) {
            Status = Iosb.__bindgen_anon_1.Status;
        }
    }

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0; // FALSE
    }

    return 1; // TRUE
}

#[no_mangle]
pub unsafe extern "system" fn GetNamedPipeHandleStateW(
    hNamedPipe: HANDLE,
    lpState: LPDWORD,
    lpCurInstances: LPDWORD,
    lpMaxCollectionCount: LPDWORD,
    lpCollectDataTimeout: LPDWORD,
    lpUserName: LPWSTR,
    nMaxUserNameSize: DWORD,
) -> BOOL {
    let mut StatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;

    if !lpState.is_null() {
        let mut PipeInfo: FILE_PIPE_INFORMATION = core::mem::zeroed();

        Status = NtQueryInformationFile(
            hNamedPipe,
            &mut StatusBlock,
            &mut PipeInfo as *mut _ as PVOID,
            core::mem::size_of::<FILE_PIPE_INFORMATION>() as ULONG,
            FilePipeInformation,
        );
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            return 0;
        }

        *lpState = (if PipeInfo.CompletionMode != FILE_PIPE_QUEUE_OPERATION as ULONG {
            PIPE_NOWAIT
        } else {
            PIPE_WAIT
        }) as DWORD;
        *lpState |= (if PipeInfo.ReadMode != FILE_PIPE_BYTE_STREAM_MODE as ULONG {
            PIPE_READMODE_MESSAGE
        } else {
            PIPE_READMODE_BYTE
        }) as DWORD;
    }

    if !lpCurInstances.is_null() {
        let mut LocalInfo: FILE_PIPE_LOCAL_INFORMATION = core::mem::zeroed();

        Status = NtQueryInformationFile(
            hNamedPipe,
            &mut StatusBlock,
            &mut LocalInfo as *mut _ as PVOID,
            core::mem::size_of::<FILE_PIPE_LOCAL_INFORMATION>() as ULONG,
            FilePipeLocalInformation,
        );
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            return 0;
        }

        *lpCurInstances = (if LocalInfo.CurrentInstances < 255 {
            LocalInfo.CurrentInstances
        } else {
            255
        }) as DWORD;
    }

    if !lpMaxCollectionCount.is_null() || !lpCollectDataTimeout.is_null() {
        let mut RemoteInfo: FILE_PIPE_REMOTE_INFORMATION = core::mem::zeroed();

        Status = NtQueryInformationFile(
            hNamedPipe,
            &mut StatusBlock,
            &mut RemoteInfo as *mut _ as PVOID,
            core::mem::size_of::<FILE_PIPE_REMOTE_INFORMATION>() as ULONG,
            FilePipeRemoteInformation,
        );
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            return 0;
        }

        if !lpMaxCollectionCount.is_null() {
            *lpMaxCollectionCount = RemoteInfo.MaximumCollectionCount as DWORD;
        }

        if !lpCollectDataTimeout.is_null() {
            let mut CollectDataTime: LARGE_INTEGER = core::mem::zeroed();

            RemoteInfo.CollectDataTime.QuadPart *= -1i64;
            CollectDataTime = RtlExtendedLargeIntegerDivide(
                RemoteInfo.CollectDataTime,
                10000,
                core::ptr::null_mut(),
            );
            if CollectDataTime.u.HighPart != 0 {
                *lpCollectDataTimeout = -2i32 as DWORD;
            } else {
                *lpCollectDataTimeout = CollectDataTime.u.LowPart;
            }
        }
    }

    if !lpUserName.is_null() {
        return NpGetUserNamep(hNamedPipe, lpUserName, nMaxUserNameSize);
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetNamedPipeHandleStateA(
    hNamedPipe: HANDLE,
    lpState: LPDWORD,
    lpCurInstances: LPDWORD,
    lpMaxCollectionCount: LPDWORD,
    lpCollectDataTimeout: LPDWORD,
    lpUserName: LPSTR,
    nMaxUserNameSize: DWORD,
) -> BOOL {
    let mut UserNameW = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut UserNameA = ANSI_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Ret: BOOL = 0;

    if !lpUserName.is_null() {
        UserNameW.MaximumLength = (nMaxUserNameSize as USHORT).wrapping_mul(core::mem::size_of::<WCHAR>() as USHORT);
        UserNameW.Buffer = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            UserNameW.MaximumLength as SIZE_T,
        ) as PWSTR;
        if UserNameW.Buffer.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
            return 0;
        }

        UserNameA.Buffer = lpUserName as PCHAR;
        UserNameA.Length = 0;
        UserNameA.MaximumLength = nMaxUserNameSize as USHORT;
    }

    Ret = GetNamedPipeHandleStateW(
        hNamedPipe,
        lpState,
        lpCurInstances,
        lpMaxCollectionCount,
        lpCollectDataTimeout,
        UserNameW.Buffer as LPWSTR,
        nMaxUserNameSize,
    );

    if Ret != 0 && !lpUserName.is_null() {
        let mut Status: NTSTATUS = 0;
        RtlInitUnicodeString(&raw mut UserNameW, UserNameW.Buffer as PCWSTR);
        Status = RtlUnicodeStringToAnsiString(
            &raw mut UserNameA,
            &raw mut UserNameW as PCUNICODE_STRING,
            0,
        );
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            Ret = 0;
        }
    }

    if !UserNameW.Buffer.is_null() {
        RtlFreeHeap(
            RtlGetProcessHeap(),
            0,
            UserNameW.Buffer as PVOID,
        );
    }

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn GetNamedPipeInfo(
    hNamedPipe: HANDLE,
    lpFlags: LPDWORD,
    lpOutBufferSize: LPDWORD,
    lpInBufferSize: LPDWORD,
    lpMaxInstances: LPDWORD,
) -> BOOL {
    let mut PipeLocalInformation: FILE_PIPE_LOCAL_INFORMATION = core::mem::zeroed();
    let mut StatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Status = NtQueryInformationFile(
        hNamedPipe,
        &mut StatusBlock,
        &mut PipeLocalInformation as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_PIPE_LOCAL_INFORMATION>() as u32,
        FilePipeLocalInformation,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    if !lpFlags.is_null() {
        *lpFlags = if PipeLocalInformation.NamedPipeEnd == FILE_PIPE_SERVER_END as u32 {
            PIPE_SERVER_END
        } else {
            PIPE_CLIENT_END
        } as u32;
        *lpFlags |= if PipeLocalInformation.NamedPipeType == FILE_PIPE_MESSAGE_TYPE as u32 {
            PIPE_TYPE_MESSAGE
        } else {
            PIPE_TYPE_BYTE
        } as u32;
    }

    if !lpOutBufferSize.is_null() {
        *lpOutBufferSize = PipeLocalInformation.OutboundQuota as u32;
    }

    if !lpInBufferSize.is_null() {
        *lpInBufferSize = PipeLocalInformation.InboundQuota as u32;
    }

    if !lpMaxInstances.is_null() {
        if PipeLocalInformation.MaximumInstances >= 255 {
            *lpMaxInstances = PIPE_UNLIMITED_INSTANCES as u32;
        } else {
            *lpMaxInstances = PipeLocalInformation.MaximumInstances as u32;
        }
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn PeekNamedPipe(
    hNamedPipe: HANDLE,
    lpBuffer: LPVOID,
    nBufferSize: DWORD,
    lpBytesRead: LPDWORD,
    lpTotalBytesAvail: LPDWORD,
    lpBytesLeftThisMessage: LPDWORD,
) -> BOOL {
    let mut Buffer: PFILE_PIPE_PEEK_BUFFER = core::ptr::null_mut();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut BufferSize: ULONG = 0;
    let mut BytesRead: ULONG = 0;
    let mut Status: NTSTATUS = 0;

    BufferSize = core::mem::offset_of!(FILE_PIPE_PEEK_BUFFER, Data) as ULONG + nBufferSize * core::mem::size_of::<CHAR>() as ULONG;
    Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferSize as SIZE_T) as PFILE_PIPE_PEEK_BUFFER;
    if Buffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return 0;
    }

    Status = NtFsControlFile(
        hNamedPipe,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut Iosb,
        FSCTL_PIPE_PEEK,
        core::ptr::null_mut(),
        0,
        Buffer as PVOID,
        BufferSize,
    );
    if Status == STATUS_PENDING {
        Status = NtWaitForSingleObject(hNamedPipe, 0, core::ptr::null_mut());
        if NT_SUCCESS(Status) {
            Status = Iosb.__bindgen_anon_1.Status;
        }
    }

    if Status == STATUS_BUFFER_OVERFLOW {
        Status = STATUS_SUCCESS;
    }

    if !NT_SUCCESS(Status) {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer as PVOID);
        BaseSetLastNTError(Status);
        return 0;
    }

    if !lpTotalBytesAvail.is_null() {
        *lpTotalBytesAvail = (*Buffer).ReadDataAvailable;
    }

    BytesRead = (Iosb.Information - core::mem::offset_of!(FILE_PIPE_PEEK_BUFFER, Data) as ULONG_PTR) as ULONG;

    if !lpBytesRead.is_null() {
        *lpBytesRead = BytesRead;
    }

    if !lpBytesLeftThisMessage.is_null() {
        *lpBytesLeftThisMessage = (*Buffer).MessageLength - BytesRead;
    }

    if !lpBuffer.is_null() {
        RtlCopyMemory(
            lpBuffer,
            &raw const (*Buffer).Data as *const core::ffi::c_void,
            BytesRead as SIZE_T,
        );
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer as PVOID);

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn TransactNamedPipe(
    hNamedPipe: HANDLE,
    lpInBuffer: LPVOID,
    nInBufferSize: DWORD,
    lpOutBuffer: LPVOID,
    nOutBufferSize: DWORD,
    lpBytesRead: LPDWORD,
    lpOverlapped: LPOVERLAPPED,
) -> BOOL {
    let mut Status: NTSTATUS;

    if !lpBytesRead.is_null() {
        *lpBytesRead = 0;
    }

    if !lpOverlapped.is_null() {
        let ApcContext: PVOID;

        ApcContext = if ((*lpOverlapped).hEvent as usize) & 0x1 != 0 {
            core::ptr::null_mut()
        } else {
            lpOverlapped as PVOID
        };
        (*lpOverlapped).Internal = STATUS_PENDING as ULONG_PTR;

        Status = NtFsControlFile(
            hNamedPipe,
            (*lpOverlapped).hEvent,
            None,
            ApcContext,
            lpOverlapped as PIO_STATUS_BLOCK,
            FSCTL_PIPE_TRANSCEIVE,
            lpInBuffer,
            nInBufferSize,
            lpOutBuffer,
            nOutBufferSize,
        );
        if !NT_SUCCESS(Status) || Status == STATUS_PENDING {
            BaseSetLastNTError(Status);
            return FALSE as BOOL;
        }

        if !lpBytesRead.is_null() {
            *lpBytesRead = (*lpOverlapped).InternalHigh as DWORD;
        }
    } else {
        let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();

        Status = NtFsControlFile(
            hNamedPipe,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut Iosb as *mut _IO_STATUS_BLOCK,
            FSCTL_PIPE_TRANSCEIVE,
            lpInBuffer,
            nInBufferSize,
            lpOutBuffer,
            nOutBufferSize,
        );
        if Status == STATUS_PENDING {
            Status = NtWaitForSingleObject(
                hNamedPipe,
                FALSE as BOOLEAN,
                core::ptr::null_mut(),
            );
            if NT_SUCCESS(Status) {
                Status = Iosb.__bindgen_anon_1.Status;
            }
        }

        if NT_SUCCESS(Status) {
            *lpBytesRead = Iosb.Information as DWORD;
        } else {
            BaseSetLastNTError(Status);
            return FALSE as BOOL;
        }
    }

    return TRUE as BOOL;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Win32 Kernel Library
//  * FILE:            dll/win32/kernel32/client/file/npipe.c
//  * PURPOSE:         Named Pipe Functions
//  * PROGRAMMER:      Alex Ionescu (alex@relsoft.net)
//  *                  Ariadne ( ariadne@xs4all.nl)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
// #define NDEBUG
// #include <debug.h>
// DEBUG_CHANNEL(kernel32file);
//
// /* GLOBALS ********************************************************************/
//
// LONG ProcessPipeId;
//
// /* FUNCTIONS ******************************************************************/
//
// /*
//  * @implemented
//  */
// static
// BOOL
// NpGetUserNamep(HANDLE hNamedPipe,
//                LPWSTR lpUserName,
//                DWORD nMaxUserNameSize)
// {
//     BOOL Ret;
//     HANDLE hToken;
//     HMODULE hAdvapi;
//     NTSTATUS Status;
//     BOOL (WINAPI *pRevertToSelf)(void);
//     BOOL (WINAPI *pGetUserNameW)(LPWSTR lpBuffer, LPDWORD lpnSize);
//     BOOL (WINAPI *pImpersonateNamedPipeClient)(HANDLE hNamedPipe);
//
//     /* Open advapi, we'll funcs from it */
//     hAdvapi = LoadLibraryW(L"advapi32.dll");
//     if (hAdvapi == NULL)
//     {
//         return FALSE;
//     }
//
//     /* Import the three required functions */
//     pRevertToSelf = (BOOL (WINAPI *)(void))GetProcAddress(hAdvapi, "RevertToSelf");
//     pGetUserNameW = (BOOL (WINAPI *)(LPWSTR, LPDWORD))GetProcAddress(hAdvapi, "GetUserNameW");
//     pImpersonateNamedPipeClient = (BOOL (WINAPI *)(HANDLE))GetProcAddress(hAdvapi, "ImpersonateNamedPipeClient");
//     /* If any couldn't be found, fail */
//     if (pRevertToSelf == NULL || pGetUserNameW == NULL || pImpersonateNamedPipeClient == NULL)
//     {
//         FreeLibrary(hAdvapi);
//         return FALSE;
//     }
//
//     /* Now, open the thread token for impersonation */
//     Status = NtOpenThreadToken(NtCurrentThread(), TOKEN_IMPERSONATE, TRUE, &hToken);
//     /* Try to impersonate the pipe client */
//     if (pImpersonateNamedPipeClient(hNamedPipe))
//     {
//         DWORD lpnSize;
//
//         /* It worked, get the user name */
//         lpnSize = nMaxUserNameSize;
//         Ret = pGetUserNameW(lpUserName, &lpnSize);
//         /* Failed to get the thread token? Revert to self */
//         if (!NT_SUCCESS(Status))
//         {
//             pRevertToSelf();
//
//             FreeLibrary(hAdvapi);
//             return Ret;
//         }
//
//         /* Restore the thread token */
//         Status = NtSetInformationThread(NtCurrentThread(), ThreadImpersonationToken,
//                                         &hToken, sizeof(hToken));
//         /* We cannot fail closing the thread token! */
//         if (!CloseHandle(hToken))
//         {
//             ASSERT(FALSE);
//         }
//
//         /* Set last error if it failed */
//         if (!NT_SUCCESS(Status))
//         {
//             BaseSetLastNTError(Status);
//         }
//     }
//     else
//     {
//         /* If opening the thread token succeed, close it */
//         if (NT_SUCCESS(Status))
//         {
//             /* We cannot fail closing it! */
//             if (!CloseHandle(hToken))
//             {
//                 ASSERT(FALSE);
//             }
//         }
//
//         Ret = FALSE;
//     }
//
//     FreeLibrary(hAdvapi);
//     return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CreatePipe(PHANDLE hReadPipe,
//            PHANDLE hWritePipe,
//            LPSECURITY_ATTRIBUTES lpPipeAttributes,
//            DWORD nSize)
// {
//     WCHAR Buffer[64];
//     UNICODE_STRING PipeName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK StatusBlock;
//     LARGE_INTEGER DefaultTimeout;
//     NTSTATUS Status;
//     HANDLE ReadPipeHandle;
//     HANDLE WritePipeHandle;
//     LONG PipeId;
//     ULONG Attributes;
//     PSECURITY_DESCRIPTOR SecurityDescriptor = NULL;
//
//     /* Set the timeout to 120 seconds */
//     DefaultTimeout.QuadPart = -1200000000;
//
//     /* Use default buffer size if desired */
//     if (!nSize) nSize = 0x1000;
//
//     /* Increase the Pipe ID */
//     PipeId = InterlockedIncrement(&ProcessPipeId);
//
//     /* Create the pipe name */
//     swprintf(Buffer,
//              L"\\Device\\NamedPipe\\Win32Pipes.%p.%08x",
//              NtCurrentTeb()->ClientId.UniqueProcess,
//              PipeId);
//     RtlInitUnicodeString(&PipeName, Buffer);
//
//     /* Always use case insensitive */
//     Attributes = OBJ_CASE_INSENSITIVE;
//
//     /* Check if we got attributes */
//     if (lpPipeAttributes)
//     {
//         /* Use the attributes' SD instead */
//         SecurityDescriptor = lpPipeAttributes->lpSecurityDescriptor;
//
//         /* Set OBJ_INHERIT if requested */
//         if (lpPipeAttributes->bInheritHandle) Attributes |= OBJ_INHERIT;
//     }
//
//     /* Initialize the attributes */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &PipeName,
//                                Attributes,
//                                NULL,
//                                SecurityDescriptor);
//
//     /* Create the named pipe */
//     Status = NtCreateNamedPipeFile(&ReadPipeHandle,
//                                    GENERIC_READ | FILE_WRITE_ATTRIBUTES | SYNCHRONIZE,
//                                    &ObjectAttributes,
//                                    &StatusBlock,
//                                    FILE_SHARE_READ | FILE_SHARE_WRITE,
//                                    FILE_CREATE,
//                                    FILE_SYNCHRONOUS_IO_NONALERT,
//                                    FILE_PIPE_BYTE_STREAM_TYPE,
//                                    FILE_PIPE_BYTE_STREAM_MODE,
//                                    FILE_PIPE_QUEUE_OPERATION,
//                                    1,
//                                    nSize,
//                                    nSize,
//                                    &DefaultTimeout);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Convert error and fail */
//         WARN("Status: %lx\n", Status);
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Now try opening it for write access */
//     Status = NtOpenFile(&WritePipeHandle,
//                         FILE_GENERIC_WRITE,
//                         &ObjectAttributes,
//                         &StatusBlock,
//                         FILE_SHARE_READ,
//                         FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Convert error and fail */
//         WARN("Status: %lx\n", Status);
//         NtClose(ReadPipeHandle);
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Return both handles */
//     *hReadPipe = ReadPipeHandle;
//     *hWritePipe = WritePipeHandle;
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// CreateNamedPipeA(LPCSTR lpName,
//                  DWORD dwOpenMode,
//                  DWORD dwPipeMode,
//                  DWORD nMaxInstances,
//                  DWORD nOutBufferSize,
//                  DWORD nInBufferSize,
//                  DWORD nDefaultTimeOut,
//                  LPSECURITY_ATTRIBUTES lpSecurityAttributes)
// {
//     /* Call the W(ide) function */
//     ConvertWin32AnsiChangeApiToUnicodeApi(CreateNamedPipe,
//                                           lpName,
//                                           dwOpenMode,
//                                           dwPipeMode,
//                                           nMaxInstances,
//                                           nOutBufferSize,
//                                           nInBufferSize,
//                                           nDefaultTimeOut,
//                                           lpSecurityAttributes);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// CreateNamedPipeW(LPCWSTR lpName,
//                  DWORD dwOpenMode,
//                  DWORD dwPipeMode,
//                  DWORD nMaxInstances,
//                  DWORD nOutBufferSize,
//                  DWORD nInBufferSize,
//                  DWORD nDefaultTimeOut,
//                  LPSECURITY_ATTRIBUTES lpSecurityAttributes)
// {
//     UNICODE_STRING NamedPipeName;
//     BOOL Result;
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE PipeHandle;
//     ACCESS_MASK DesiredAccess;
//     ULONG CreateOptions = 0;
//     ULONG WriteModeMessage;
//     ULONG ReadModeMessage;
//     ULONG NonBlocking;
//     IO_STATUS_BLOCK Iosb;
//     ULONG ShareAccess = 0, Attributes;
//     LARGE_INTEGER DefaultTimeOut;
//     PSECURITY_DESCRIPTOR SecurityDescriptor = NULL;
//
//     /* Check for valid instances */
//     if (nMaxInstances == 0 || nMaxInstances > PIPE_UNLIMITED_INSTANCES)
//     {
//         /* Fail */
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     /* Convert to NT syntax */
//     if (nMaxInstances == PIPE_UNLIMITED_INSTANCES)
//         nMaxInstances = -1;
//
//     /* Convert the name */
//     Result = RtlDosPathNameToNtPathName_U(lpName,
//                                            &NamedPipeName,
//                                            NULL,
//                                            NULL);
//     if (!Result)
//     {
//         /* Conversion failed */
//         SetLastError(ERROR_PATH_NOT_FOUND);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     TRACE("Pipe name: %wZ\n", &NamedPipeName);
//     TRACE("Pipe name: %S\n", NamedPipeName.Buffer);
//
//     /* Always case insensitive, check if we got extra attributes */
//     Attributes = OBJ_CASE_INSENSITIVE;
//     if (lpSecurityAttributes)
//     {
//         /* We did; get the security descriptor */
//         SecurityDescriptor = lpSecurityAttributes->lpSecurityDescriptor;
//
//         /* And check if this is pipe's handle will beinheritable */
//         if (lpSecurityAttributes->bInheritHandle)
//             Attributes |= OBJ_INHERIT;
//     }
//
//     /* Now we can initialize the object attributes */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &NamedPipeName,
//                                Attributes,
//                                NULL,
//                                SecurityDescriptor);
//
//     /* Setup the default Desired Access */
//     DesiredAccess = SYNCHRONIZE | (dwOpenMode & (WRITE_DAC |
//                                                  WRITE_OWNER |
//                                                  ACCESS_SYSTEM_SECURITY));
//
//     /* Convert to NT Create Flags */
//     if (dwOpenMode & FILE_FLAG_WRITE_THROUGH)
//     {
//         CreateOptions |= FILE_WRITE_THROUGH;
//     }
//
//     if (!(dwOpenMode & FILE_FLAG_OVERLAPPED))
//     {
//         CreateOptions |= FILE_SYNCHRONOUS_IO_NONALERT;
//     }
//
//     /* Handle all open modes */
//     if (dwOpenMode & PIPE_ACCESS_OUTBOUND)
//     {
//         ShareAccess |= FILE_SHARE_READ;
//         DesiredAccess |= GENERIC_WRITE;
//     }
//
//     if (dwOpenMode & PIPE_ACCESS_INBOUND)
//     {
//         ShareAccess |= FILE_SHARE_WRITE;
//         DesiredAccess |= GENERIC_READ;
//     }
//
//     /* Handle the type flags */
//     if (dwPipeMode & PIPE_TYPE_MESSAGE)
//     {
//         WriteModeMessage = FILE_PIPE_MESSAGE_TYPE;
//     }
//     else
//     {
//         WriteModeMessage = FILE_PIPE_BYTE_STREAM_TYPE;
//     }
//
//     /* Handle the mode flags */
//     if (dwPipeMode & PIPE_READMODE_MESSAGE)
//     {
//         ReadModeMessage = FILE_PIPE_MESSAGE_MODE;
//     }
//     else
//     {
//         ReadModeMessage = FILE_PIPE_BYTE_STREAM_MODE;
//     }
//
//     /* Handle the blocking mode */
//     if (dwPipeMode & PIPE_NOWAIT)
//     {
//         NonBlocking = FILE_PIPE_COMPLETE_OPERATION;
//     }
//     else
//     {
//         NonBlocking = FILE_PIPE_QUEUE_OPERATION;
//     }
//
//     /* Check if we have a timeout */
//     if (nDefaultTimeOut)
//     {
//         /* Convert the time to NT format */
//         DefaultTimeOut.QuadPart = nDefaultTimeOut * -10000LL;
//     }
//     else
//     {
//         /* Use default timeout of 50 ms */
//         DefaultTimeOut.QuadPart = -500000;
//     }
//
//     /* Now create the pipe */
//     Status = NtCreateNamedPipeFile(&PipeHandle,
//                                    DesiredAccess,
//                                    &ObjectAttributes,
//                                    &Iosb,
//                                    ShareAccess,
//                                    FILE_OPEN_IF,
//                                    CreateOptions,
//                                    WriteModeMessage,
//                                    ReadModeMessage,
//                                    NonBlocking,
//                                    nMaxInstances,
//                                    nInBufferSize,
//                                    nOutBufferSize,
//                                    &DefaultTimeOut);
//
//     /* Normalize special error codes */
//     if ((Status == STATUS_INVALID_DEVICE_REQUEST) ||
//         (Status == STATUS_NOT_SUPPORTED))
//     {
//         Status = STATUS_OBJECT_NAME_INVALID;
//     }
//
//     /* Free the name */
//     RtlFreeHeap(RtlGetProcessHeap(),
//                 0,
//                 NamedPipeName.Buffer);
//
//     /* Check status */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed to create it */
//         WARN("NtCreateNamedPipe failed (Status %x)!\n", Status);
//         BaseSetLastNTError (Status);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     /* Return the handle */
//     return PipeHandle;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// WaitNamedPipeA(LPCSTR lpNamedPipeName,
//                DWORD nTimeOut)
// {
//     BOOL r = FALSE;
//     UNICODE_STRING NameU;
//
//     /* Convert the name to Unicode */
//     if (Basep8BitStringToDynamicUnicodeString(&NameU, lpNamedPipeName))
//     {
//         /* Call the Unicode API */
//         r = WaitNamedPipeW(NameU.Buffer, nTimeOut);
//
//         /* Free the Unicode string */
//         RtlFreeUnicodeString(&NameU);
//     }
//
//     /* Return result */
//     return r;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// WaitNamedPipeW(LPCWSTR lpNamedPipeName,
//                DWORD nTimeOut)
// {
//     UNICODE_STRING NamedPipeName, NewName, DevicePath, PipePrefix;
//     ULONG NameLength;
//     ULONG i;
//     PWCHAR p;
//     ULONG Type;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     HANDLE FileHandle;
//     IO_STATUS_BLOCK IoStatusBlock;
//     ULONG WaitPipeInfoSize;
//     PVOID DevicePathBuffer;
//     PFILE_PIPE_WAIT_FOR_BUFFER WaitPipeInfo;
//
//     /* Start by making a unicode string of the name */
//     TRACE("Sent path: %S\n", lpNamedPipeName);
//     if (!RtlCreateUnicodeString(&NamedPipeName, lpNamedPipeName))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//     NameLength = NamedPipeName.Length / sizeof(WCHAR);
//
//     /* All slashes must become backslashes */
//     for (i = 0; i < NameLength; i++)
//     {
//         /* Check and convert */
//         if (NamedPipeName.Buffer[i] == L'/') NamedPipeName.Buffer[i] = L'\\';
//     }
//
//     DevicePathBuffer = NULL;
//
//     /* Find the path type of the name we were given */
//     NewName = NamedPipeName;
//     Type = RtlDetermineDosPathNameType_U(lpNamedPipeName);
//
//     /* Check if this was a device path, ie : "\\.\pipe\name" */
//     if (Type == RtlPathTypeLocalDevice)
//     {
//         /* Make sure it's a valid prefix */
//         RtlInitUnicodeString(&PipePrefix, L"\\\\.\\pipe\\");
//         if (!RtlPrefixUnicodeString(&PipePrefix, &NewName, TRUE))
//         {
//             /* The name is invalid */
//             WARN("Invalid name!\n");
//             RtlFreeUnicodeString(&NamedPipeName);
//             BaseSetLastNTError(STATUS_OBJECT_PATH_SYNTAX_BAD);
//             return FALSE;
//         }
//
//         /* Move past it */
//         NewName.Buffer += PipePrefix.Length / sizeof(WCHAR);
//         NewName.Length -= PipePrefix.Length;
//         NewName.MaximumLength -= PipePrefix.Length;
//
//         /* Initialize the Dos Devices name */
//         TRACE("NewName: %wZ\n", &NewName);
//         RtlInitUnicodeString(&DevicePath, L"\\DosDevices\\pipe\\");
//     }
//     else if (Type == RtlPathTypeUncAbsolute)
//     {
//         PWSTR PipeName;
//
//         /* The path is \\server\\pipe\name; find the pipename itself */
//         p = &NewName.Buffer[2];
//
//         /* First loop to get past the server name */
//         do
//         {
//             /* Check if this is a backslash */
//             if (*p == L'\\') break;
//
//             /* Check next */
//             p++;
//         } while (*p);
//
//         /* Now make sure the full name contains "pipe\" */
//         if ((*p) && !(_wcsnicmp(p + 1, L"pipe\\", sizeof("pipe\\") - sizeof(ANSI_NULL))))
//         {
//             /* Get to the pipe name itself now */
//             p += sizeof("pipe\\") - sizeof(ANSI_NULL);
//         }
//         else
//         {
//             /* The name is invalid */
//             WARN("Invalid name!\n");
//             RtlFreeUnicodeString(&NamedPipeName);
//             BaseSetLastNTError(STATUS_OBJECT_PATH_SYNTAX_BAD);
//             return FALSE;
//         }
//
//         /* Skip first backslash */
//         NewName.Buffer++;
//         /* And skip pipe for copying name */
//         PipeName = p + ((sizeof(L"pipe\\") - sizeof(UNICODE_NULL)) / sizeof(WCHAR));
//         /* Update the string */
//         NewName.Length = (USHORT)((ULONG_PTR)PipeName - (ULONG_PTR)NewName.Buffer);
//         NewName.MaximumLength = (USHORT)((ULONG_PTR)PipeName - (ULONG_PTR)NewName.Buffer);
//
//         /* DevicePath will contain the pipename + the DosDevice prefix */
//         DevicePath.MaximumLength = (USHORT)((ULONG_PTR)PipeName - (ULONG_PTR)NewName.Buffer) + sizeof(L"\\DosDevices\\UNC\\");
//
//         /* Allocate the buffer for DevicePath */
//         DevicePathBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, DevicePath.MaximumLength);
//         if (DevicePathBuffer == NULL)
//         {
//             RtlFreeUnicodeString(&NamedPipeName);
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             return FALSE;
//         }
//
//         /* Copy the prefix first */
//         DevicePath.Buffer = DevicePathBuffer;
//         RtlCopyMemory(DevicePathBuffer, L"\\DosDevices\\UNC\\", sizeof(L"\\DosDevices\\UNC\\") - sizeof(UNICODE_NULL));
//         DevicePath.Length = sizeof(L"\\DosDevices\\UNC\\") - sizeof(UNICODE_NULL);
//         /* And append the rest */
//         RtlAppendUnicodeStringToString(&DevicePath, &NewName);
//         /* And fix pipe name without its prefix */
//         RtlInitUnicodeString(&NewName, PipeName + 1);
//     }
//     else
//     {
//         WARN("Invalid path type\n");
//         RtlFreeUnicodeString(&NamedPipeName);
//         BaseSetLastNTError(STATUS_OBJECT_PATH_SYNTAX_BAD);
//         return FALSE;
//     }
//
//     /* Now calculate the total length of the structure and allocate it */
//     WaitPipeInfoSize = FIELD_OFFSET(FILE_PIPE_WAIT_FOR_BUFFER, Name[0]) +
//                        NewName.Length;
//     WaitPipeInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0, WaitPipeInfoSize);
//     if (WaitPipeInfo == NULL)
//     {
//         if (DevicePathBuffer != NULL)
//         {
//             RtlFreeHeap(RtlGetProcessHeap(), 0, DevicePathBuffer);
//         }
//
//         RtlFreeUnicodeString(&NamedPipeName);
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     /* Initialize the object attributes */
//     TRACE("Opening: %wZ\n", &DevicePath);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DevicePath,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     /* Open the path */
//     Status = NtOpenFile(&FileHandle,
//                         FILE_READ_ATTRIBUTES | SYNCHRONIZE,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE,
//                         FILE_SYNCHRONOUS_IO_NONALERT);
//
//     if (DevicePathBuffer != NULL)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, DevicePathBuffer);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail; couldn't open */
//         WARN("Status: %lx\n", Status);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, WaitPipeInfo);
//         RtlFreeUnicodeString(&NamedPipeName);
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Check what timeout we got */
//     if (nTimeOut == NMPWAIT_USE_DEFAULT_WAIT)
//     {
//         /* Don't use a timeout */
//         WaitPipeInfo->TimeoutSpecified = FALSE;
//     }
//     else
//     {
//         /* Check if we should wait forever */
//         if (nTimeOut == NMPWAIT_WAIT_FOREVER)
//         {
//             /* Set the max */
//             WaitPipeInfo->Timeout.LowPart = 0;
//             WaitPipeInfo->Timeout.HighPart = 0x80000000;
//         }
//         else
//         {
//             /* Convert to NT format */
//             WaitPipeInfo->Timeout.QuadPart = nTimeOut * -10000LL;
//         }
//
//         /* In both cases, we do have a timeout */
//         WaitPipeInfo->TimeoutSpecified = TRUE;
//     }
//
//     /* Set the length and copy the name */
//     WaitPipeInfo->NameLength = NewName.Length;
//     RtlCopyMemory(WaitPipeInfo->Name, NewName.Buffer, NewName.Length);
//
//     /* Get rid of the full name */
//     RtlFreeUnicodeString(&NamedPipeName);
//
//     /* Let NPFS know of our request */
//     Status = NtFsControlFile(FileHandle,
//                              NULL,
//                              NULL,
//                              NULL,
//                              &IoStatusBlock,
//                              FSCTL_PIPE_WAIT,
//                              WaitPipeInfo,
//                              WaitPipeInfoSize,
//                              NULL,
//                              0);
//
//     /* Free our pipe info data and close the handle */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, WaitPipeInfo);
//     NtClose(FileHandle);
//
//     /* Check the status */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failure to wait on the pipe */
//         WARN("Status: %lx\n", Status);
//         BaseSetLastNTError(Status);
//         return FALSE;
//      }
//
//     /* Success */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// ConnectNamedPipe(IN HANDLE hNamedPipe,
//                  IN LPOVERLAPPED lpOverlapped)
// {
//     NTSTATUS Status;
//
//     if (lpOverlapped != NULL)
//     {
//         PVOID ApcContext;
//
//         lpOverlapped->Internal = STATUS_PENDING;
//         ApcContext = (((ULONG_PTR)lpOverlapped->hEvent & 0x1) ? NULL : lpOverlapped);
//
//         Status = NtFsControlFile(hNamedPipe,
//                                  lpOverlapped->hEvent,
//                                  NULL,
//                                  ApcContext,
//                                  (PIO_STATUS_BLOCK)lpOverlapped,
//                                  FSCTL_PIPE_LISTEN,
//                                  NULL,
//                                  0,
//                                  NULL,
//                                  0);
//
//         /* return FALSE in case of failure and pending operations! */
//         if (!NT_SUCCESS(Status) || Status == STATUS_PENDING)
//         {
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//     }
//     else
//     {
//         IO_STATUS_BLOCK Iosb;
//
//         Status = NtFsControlFile(hNamedPipe,
//                                  NULL,
//                                  NULL,
//                                  NULL,
//                                  &Iosb,
//                                  FSCTL_PIPE_LISTEN,
//                                  NULL,
//                                  0,
//                                  NULL,
//                                  0);
//
//         /* wait in case operation is pending */
//         if (Status == STATUS_PENDING)
//         {
//              Status = NtWaitForSingleObject(hNamedPipe,
//                                             FALSE,
//                                             NULL);
//              if (NT_SUCCESS(Status))
//              {
//                  Status = Iosb.Status;
//              }
//         }
//
//         if (!NT_SUCCESS(Status))
//         {
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetNamedPipeHandleState(HANDLE hNamedPipe,
//                         LPDWORD lpMode,
//                         LPDWORD lpMaxCollectionCount,
//                         LPDWORD lpCollectDataTimeout)
// {
//     IO_STATUS_BLOCK Iosb;
//     NTSTATUS Status;
//
//     /* Check if the Mode is being changed */
//     if (lpMode)
//     {
//         FILE_PIPE_INFORMATION Settings;
//
//         /* Set the Completion Mode */
//         Settings.CompletionMode = (*lpMode & PIPE_NOWAIT) ?
//                                   FILE_PIPE_COMPLETE_OPERATION : FILE_PIPE_QUEUE_OPERATION;
//
//         /* Set the Read Mode */
//         Settings.ReadMode = (*lpMode & PIPE_READMODE_MESSAGE) ?
//                             FILE_PIPE_MESSAGE_MODE : FILE_PIPE_BYTE_STREAM_MODE;
//
//         /* Send the changes to the Driver */
//         Status = NtSetInformationFile(hNamedPipe,
//                                       &Iosb,
//                                       &Settings,
//                                       sizeof(Settings),
//                                       FilePipeInformation);
//         if (!NT_SUCCESS(Status))
//         {
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//     }
//
//     /* Check if the Collection count or Timeout are being changed */
//     if (lpMaxCollectionCount || lpCollectDataTimeout)
//     {
//         FILE_PIPE_REMOTE_INFORMATION RemoteSettings;
//
//         /* Setting one without the other would delete it, so we read old one */
//         if (!lpMaxCollectionCount || !lpCollectDataTimeout)
//         {
//             Status = NtQueryInformationFile(hNamedPipe,
//                                             &Iosb,
//                                             &RemoteSettings,
//                                             sizeof(RemoteSettings),
//                                             FilePipeRemoteInformation);
//             if (!NT_SUCCESS(Status))
//             {
//                 BaseSetLastNTError(Status);
//                 return FALSE;
//             }
//         }
//
//         /* Now set the new settings */
//         RemoteSettings.MaximumCollectionCount = (lpMaxCollectionCount) ?
//                                                 *lpMaxCollectionCount :
//                                                 RemoteSettings.MaximumCollectionCount;
//         if (lpCollectDataTimeout)
//         {
//             /* Convert it to Quad */
//             RemoteSettings.CollectDataTime.QuadPart = *lpCollectDataTimeout * -10000LL;
//         }
//
//         /* Tell the driver to change them */
//         Status = NtSetInformationFile(hNamedPipe,
//                                       &Iosb,
//                                       &RemoteSettings,
//                                       sizeof(RemoteSettings),
//                                       FilePipeRemoteInformation);
//         if (!NT_SUCCESS(Status))
//         {
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CallNamedPipeA(LPCSTR lpNamedPipeName,
//                LPVOID lpInBuffer,
//                DWORD nInBufferSize,
//                LPVOID lpOutBuffer,
//                DWORD nOutBufferSize,
//                LPDWORD lpBytesRead,
//                DWORD nTimeOut)
// {
//     PUNICODE_STRING PipeName = &NtCurrentTeb()->StaticUnicodeString;
//     ANSI_STRING AnsiPipe;
//
//     /* Initialize the string as ANSI_STRING and convert to Unicode */
//     RtlInitAnsiString(&AnsiPipe, (LPSTR)lpNamedPipeName);
//     RtlAnsiStringToUnicodeString(PipeName, &AnsiPipe, FALSE);
//
//     /* Call the Unicode function */
//     return CallNamedPipeW(PipeName->Buffer,
//                           lpInBuffer,
//                           nInBufferSize,
//                           lpOutBuffer,
//                           nOutBufferSize,
//                           lpBytesRead,
//                           nTimeOut);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CallNamedPipeW(LPCWSTR lpNamedPipeName,
//                LPVOID lpInBuffer,
//                DWORD nInBufferSize,
//                LPVOID lpOutBuffer,
//                DWORD nOutBufferSize,
//                LPDWORD lpBytesRead,
//                DWORD nTimeOut)
// {
//     HANDLE hPipe;
//     BOOL bRetry = TRUE;
//     BOOL bError;
//     DWORD dwPipeMode;
//
//     while (TRUE)
//     {
//         /* Try creating it */
//         hPipe = CreateFileW(lpNamedPipeName,
//                             GENERIC_READ | GENERIC_WRITE,
//                             FILE_SHARE_READ | FILE_SHARE_WRITE,
//                             NULL,
//                             OPEN_EXISTING,
//                             FILE_ATTRIBUTE_NORMAL,
//                             NULL);
//
//         /* Success, break out */
//         if (hPipe != INVALID_HANDLE_VALUE)
//             break;
//
//         /* Already tried twice, give up */
//         if (bRetry == FALSE)
//             return FALSE;
//
//         /* Wait on it */
//         WaitNamedPipeW(lpNamedPipeName, nTimeOut);
//
//         /* Get ready to try again */
//         bRetry = FALSE;
//     }
//
//     /* Set the pipe mode */
//     dwPipeMode = PIPE_READMODE_MESSAGE | PIPE_WAIT;
//     bError = SetNamedPipeHandleState(hPipe, &dwPipeMode, NULL, NULL);
//     if (!bError)
//     {
//         /* Couldn't change state, fail */
//         CloseHandle(hPipe);
//         return FALSE;
//     }
//
//     /* Do the transact */
//     bError = TransactNamedPipe(hPipe,
//                                lpInBuffer,
//                                nInBufferSize,
//                                lpOutBuffer,
//                                nOutBufferSize,
//                                lpBytesRead,
//                                NULL);
//
//     /* Close the handle */
//     CloseHandle(hPipe);
//
//     return bError;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DisconnectNamedPipe(HANDLE hNamedPipe)
// {
//     IO_STATUS_BLOCK Iosb;
//     NTSTATUS Status;
//
//     /* Send the FSCTL to the driver */
//     Status = NtFsControlFile(hNamedPipe,
//                              NULL,
//                              NULL,
//                              NULL,
//                              &Iosb,
//                              FSCTL_PIPE_DISCONNECT,
//                              NULL,
//                              0,
//                              NULL,
//                              0);
//     if (Status == STATUS_PENDING)
//     {
//         /* Wait on NPFS to finish and get updated status */
//         Status = NtWaitForSingleObject(hNamedPipe, FALSE, NULL);
//         if (NT_SUCCESS(Status))
//             Status = Iosb.Status;
//     }
//
//     /* Check for error */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// GetNamedPipeHandleStateW(HANDLE hNamedPipe,
//                          LPDWORD lpState,
//                          LPDWORD lpCurInstances,
//                          LPDWORD lpMaxCollectionCount,
//                          LPDWORD lpCollectDataTimeout,
//                          LPWSTR lpUserName,
//                          DWORD nMaxUserNameSize)
// {
//     IO_STATUS_BLOCK StatusBlock;
//     NTSTATUS Status;
//
//     if (lpState != NULL)
//     {
//         FILE_PIPE_INFORMATION PipeInfo;
//
//         Status = NtQueryInformationFile(hNamedPipe,
//                                         &StatusBlock,
//                                         &PipeInfo,
//                                         sizeof(PipeInfo),
//                                         FilePipeInformation);
//         if (!NT_SUCCESS(Status))
//         {
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//
//         *lpState = ((PipeInfo.CompletionMode != FILE_PIPE_QUEUE_OPERATION) ? PIPE_NOWAIT : PIPE_WAIT);
//         *lpState |= ((PipeInfo.ReadMode != FILE_PIPE_BYTE_STREAM_MODE) ? PIPE_READMODE_MESSAGE : PIPE_READMODE_BYTE);
//     }
//
//     if (lpCurInstances != NULL)
//     {
//         FILE_PIPE_LOCAL_INFORMATION LocalInfo;
//
//         Status = NtQueryInformationFile(hNamedPipe,
//                                         &StatusBlock,
//                                         &LocalInfo,
//                                         sizeof(LocalInfo),
//                                         FilePipeLocalInformation);
//         if (!NT_SUCCESS(Status))
//         {
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//
//         *lpCurInstances = min(LocalInfo.CurrentInstances, PIPE_UNLIMITED_INSTANCES);
//     }
//
//     if (lpMaxCollectionCount != NULL || lpCollectDataTimeout != NULL)
//     {
//         FILE_PIPE_REMOTE_INFORMATION RemoteInfo;
//
//         Status = NtQueryInformationFile(hNamedPipe,
//                                         &StatusBlock,
//                                         &RemoteInfo,
//                                         sizeof(RemoteInfo),
//                                         FilePipeRemoteInformation);
//         if (!NT_SUCCESS(Status))
//         {
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//
//         if (lpMaxCollectionCount != NULL)
//         {
//             *lpMaxCollectionCount = RemoteInfo.MaximumCollectionCount;
//         }
//
//         if (lpCollectDataTimeout != NULL)
//         {
//             LARGE_INTEGER CollectDataTime;
//
//             /* Convert time and return it */
//             RemoteInfo.CollectDataTime.QuadPart *= -1;
//             CollectDataTime = RtlExtendedLargeIntegerDivide(RemoteInfo.CollectDataTime, 10000, NULL);
//             /* In case of overflow, just return MAX - 1 */
//             if (CollectDataTime.HighPart != 0)
//             {
//                 *lpCollectDataTimeout = -2;
//             }
//             else
//             {
//                 *lpCollectDataTimeout = CollectDataTime.LowPart;
//             }
//         }
//     }
//
//     if (lpUserName != NULL)
//     {
//         return NpGetUserNamep(hNamedPipe, lpUserName, nMaxUserNameSize);
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetNamedPipeHandleStateA(HANDLE hNamedPipe,
//                          LPDWORD lpState,
//                          LPDWORD lpCurInstances,
//                          LPDWORD lpMaxCollectionCount,
//                          LPDWORD lpCollectDataTimeout,
//                          LPSTR lpUserName,
//                          DWORD nMaxUserNameSize)
// {
//     UNICODE_STRING UserNameW = { 0, 0, NULL };
//     ANSI_STRING UserNameA;
//     BOOL Ret;
//
//     if (lpUserName != NULL)
//     {
//         UserNameW.MaximumLength = (USHORT)nMaxUserNameSize * sizeof(WCHAR);
//         UserNameW.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, UserNameW.MaximumLength);
//         if (UserNameW.Buffer == NULL)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             return FALSE;
//         }
//
//         UserNameA.Buffer = lpUserName;
//         UserNameA.Length = 0;
//         UserNameA.MaximumLength = (USHORT)nMaxUserNameSize;
//     }
//
//     Ret = GetNamedPipeHandleStateW(hNamedPipe,
//                                    lpState,
//                                    lpCurInstances,
//                                    lpMaxCollectionCount,
//                                    lpCollectDataTimeout,
//                                    UserNameW.Buffer,
//                                    nMaxUserNameSize);
//     if (Ret && lpUserName != NULL)
//     {
//         NTSTATUS Status;
//
//         RtlInitUnicodeString(&UserNameW, UserNameW.Buffer);
//         Status = RtlUnicodeStringToAnsiString(&UserNameA, &UserNameW, FALSE);
//         if (!NT_SUCCESS(Status))
//         {
//             BaseSetLastNTError(Status);
//             Ret = FALSE;
//         }
//     }
//
//     if (UserNameW.Buffer != NULL)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, UserNameW.Buffer);
//     }
//
//     return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetNamedPipeInfo(HANDLE hNamedPipe,
//                  LPDWORD lpFlags,
//                  LPDWORD lpOutBufferSize,
//                  LPDWORD lpInBufferSize,
//                  LPDWORD lpMaxInstances)
// {
//     FILE_PIPE_LOCAL_INFORMATION PipeLocalInformation;
//     IO_STATUS_BLOCK StatusBlock;
//     NTSTATUS Status;
//
//     Status = NtQueryInformationFile(hNamedPipe,
//                                     &StatusBlock,
//                                     &PipeLocalInformation,
//                                     sizeof(PipeLocalInformation),
//                                     FilePipeLocalInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     if (lpFlags != NULL)
//     {
//         *lpFlags = (PipeLocalInformation.NamedPipeEnd == FILE_PIPE_SERVER_END) ? PIPE_SERVER_END : PIPE_CLIENT_END;
//         *lpFlags |= (PipeLocalInformation.NamedPipeType == FILE_PIPE_MESSAGE_TYPE) ? PIPE_TYPE_MESSAGE : PIPE_TYPE_BYTE;
//     }
//
//     if (lpOutBufferSize != NULL)
//         *lpOutBufferSize = PipeLocalInformation.OutboundQuota;
//
//     if (lpInBufferSize != NULL)
//         *lpInBufferSize = PipeLocalInformation.InboundQuota;
//
//     if (lpMaxInstances != NULL)
//     {
//         if (PipeLocalInformation.MaximumInstances >= 255)
//             *lpMaxInstances = PIPE_UNLIMITED_INSTANCES;
//         else
//             *lpMaxInstances = PipeLocalInformation.MaximumInstances;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// PeekNamedPipe(HANDLE hNamedPipe,
//               LPVOID lpBuffer,
//               DWORD nBufferSize,
//               LPDWORD lpBytesRead,
//               LPDWORD lpTotalBytesAvail,
//               LPDWORD lpBytesLeftThisMessage)
// {
//     PFILE_PIPE_PEEK_BUFFER Buffer;
//     IO_STATUS_BLOCK Iosb;
//     ULONG BufferSize;
//     ULONG BytesRead;
//     NTSTATUS Status;
//
//     /* Calculate the buffer space that we'll need and allocate it */
//     BufferSize = FIELD_OFFSET(FILE_PIPE_PEEK_BUFFER, Data[nBufferSize]);
//     Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferSize);
//     if (Buffer == NULL)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     /* Tell the driver to seek */
//     Status = NtFsControlFile(hNamedPipe,
//                              NULL,
//                              NULL,
//                              NULL,
//                              &Iosb,
//                              FSCTL_PIPE_PEEK,
//                              NULL,
//                              0,
//                              Buffer,
//                              BufferSize);
//     if (Status == STATUS_PENDING)
//     {
//         /* Wait for npfs to be done, and update the status */
//         Status = NtWaitForSingleObject(hNamedPipe, FALSE, NULL);
//         if (NT_SUCCESS(Status))
//             Status = Iosb.Status;
//     }
//
//     /* Overflow is success for us */
//     if (Status == STATUS_BUFFER_OVERFLOW)
//         Status = STATUS_SUCCESS;
//
//     /* If we failed */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Free the buffer and return failure */
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Check if caller requested bytes available */
//     if (lpTotalBytesAvail)
//     {
//         /* Return bytes available */
//         *lpTotalBytesAvail = Buffer->ReadDataAvailable;
//     }
//
//     /* Calculate the bytes returned, minus our structure overhead */
//     BytesRead = (ULONG)(Iosb.Information -
//                         FIELD_OFFSET(FILE_PIPE_PEEK_BUFFER, Data[0]));
//     ASSERT(BytesRead <= nBufferSize);
//
//     /* Check if caller requested bytes read */
//     if (lpBytesRead)
//     {
//         /* Return the bytes read */
//         *lpBytesRead = BytesRead;
//     }
//
//     /* Check if caller requested bytes left */
//     if (lpBytesLeftThisMessage)
//     {
//         /* Calculate total minus what we returned and our structure overhead */
//         *lpBytesLeftThisMessage = Buffer->MessageLength - BytesRead;
//     }
//
//     /* Check if the caller wanted to see the actual data */
//     if (lpBuffer)
//     {
//         /* Give him what he wants */
//         RtlCopyMemory(lpBuffer,
//                       Buffer->Data,
//                       BytesRead);
//     }
//
//     /* Free the buffer */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// TransactNamedPipe(IN HANDLE hNamedPipe,
//                   IN LPVOID lpInBuffer,
//                   IN DWORD nInBufferSize,
//                   OUT LPVOID lpOutBuffer,
//                   IN DWORD nOutBufferSize,
//                   OUT LPDWORD lpBytesRead  OPTIONAL,
//                   IN LPOVERLAPPED lpOverlapped  OPTIONAL)
// {
//     NTSTATUS Status;
//
//     if (lpBytesRead != NULL)
//     {
//         *lpBytesRead = 0;
//     }
//
//     if (lpOverlapped != NULL)
//     {
//         PVOID ApcContext;
//
//         ApcContext = (((ULONG_PTR)lpOverlapped->hEvent & 0x1) ? NULL : lpOverlapped);
//         lpOverlapped->Internal = STATUS_PENDING;
//
//         Status = NtFsControlFile(hNamedPipe,
//                                  lpOverlapped->hEvent,
//                                  NULL,
//                                  ApcContext,
//                                  (PIO_STATUS_BLOCK)lpOverlapped,
//                                  FSCTL_PIPE_TRANSCEIVE,
//                                  lpInBuffer,
//                                  nInBufferSize,
//                                  lpOutBuffer,
//                                  nOutBufferSize);
//         if (!NT_SUCCESS(Status) || Status == STATUS_PENDING)
//         {
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//
//         if (lpBytesRead != NULL)
//         {
//             *lpBytesRead = lpOverlapped->InternalHigh;
//         }
//     }
//     else
//     {
//         IO_STATUS_BLOCK Iosb;
//
//         Status = NtFsControlFile(hNamedPipe,
//                                  NULL,
//                                  NULL,
//                                  NULL,
//                                  &Iosb,
//                                  FSCTL_PIPE_TRANSCEIVE,
//                                  lpInBuffer,
//                                  nInBufferSize,
//                                  lpOutBuffer,
//                                  nOutBufferSize);
//         if (Status == STATUS_PENDING)
//         {
//             Status = NtWaitForSingleObject(hNamedPipe,
//                                            FALSE,
//                                            NULL);
//             if (NT_SUCCESS(Status))
//                 Status = Iosb.Status;
//         }
//
//         if (NT_SUCCESS(Status))
//         {
//             /* lpNumberOfBytesRead must not be NULL here, in fact Win doesn't
//                check that case either and crashes (only after the operation
//                completed) */
//             *lpBytesRead = Iosb.Information;
//         }
//         else
//         {
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//     }
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
