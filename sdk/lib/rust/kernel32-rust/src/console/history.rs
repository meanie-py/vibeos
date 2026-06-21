use ntdll_sys::nt::*;
use crate::prelude::*;
use crate::console::cffi::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _LIST_ENTRY {
    pub Flink: *mut _LIST_ENTRY,
    pub Blink: *mut _LIST_ENTRY,
}

pub type PLIST_ENTRY = *mut _LIST_ENTRY;

#[repr(C)]
pub struct _UNICODE_STRING {
    pub Length: u16,
    pub MaximumLength: u16,
    pub Buffer: *mut u16,
}

pub type PUNICODE_STRING = *mut _UNICODE_STRING;

#[repr(C)]
pub struct _STRING {
    pub Length: u16,
    pub MaximumLength: u16,
    pub Buffer: *mut i8,
}

pub type PSTRING = *mut _STRING;

#[repr(C)]
pub struct _OBJECT_ATTRIBUTES {
    pub Length: u32,
    pub RootDirectory: *mut core::ffi::c_void,
    pub ObjectName: PUNICODE_STRING,
    pub Attributes: u32,
    pub SecurityDescriptor: *mut core::ffi::c_void,
    pub SecurityQualityOfService: *mut core::ffi::c_void,
}

pub type POBJECT_ATTRIBUTES = *mut _OBJECT_ATTRIBUTES;

#[repr(C)]
pub struct _IO_STATUS_BLOCK {
    pub Status: i32,
    pub Information: usize,
}

pub type PIO_STATUS_BLOCK = *mut _IO_STATUS_BLOCK;

#[repr(C)]
pub struct _CLIENT_ID {
    pub UniqueProcess: *mut core::ffi::c_void,
    pub UniqueThread: *mut core::ffi::c_void,
}

pub type PCLIENT_ID = *mut _CLIENT_ID;

pub type HANDLE = *mut core::ffi::c_void;
pub type PHANDLE = *mut HANDLE;
pub type NTSTATUS = i32;
pub type BOOLEAN = u8;
pub type BOOL = i32;
pub type DWORD = u32;
pub type LPVOID = *mut core::ffi::c_void;
pub type LPCVOID = *const core::ffi::c_void;
pub type LPSTR = *mut i8;
pub type LPCSTR = *const i8;
pub type LPWSTR = *mut u16;
pub type LPCWSTR = *const u16;
pub type ULONG = u32;
pub type PULONG = *mut u32;
pub type USHORT = u16;
pub type PUSHORT = *mut u16;
pub type UCHAR = u8;
pub type PUCHAR = *mut u8;
pub type CHAR = i8;
pub type WCHAR = u16;

pub const FALSE: BOOL = 0;
pub const TRUE: BOOL = 1;
pub const NULL: *mut core::ffi::c_void = core::ptr::null_mut();

pub const STATUS_SUCCESS: NTSTATUS = 0x00000000u32 as NTSTATUS;
pub const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC0000001u32 as NTSTATUS;
pub const STATUS_INFO_LENGTH_MISMATCH: NTSTATUS = 0xC0000004u32 as NTSTATUS;
pub const STATUS_ACCESS_DENIED: NTSTATUS = 0xC0000022u32 as NTSTATUS;
pub const STATUS_BUFFER_TOO_SMALL: NTSTATUS = 0xC0000023u32 as NTSTATUS;
pub const STATUS_OBJECT_NAME_INVALID: NTSTATUS = 0xC0000033u32 as NTSTATUS;
pub const STATUS_OBJECT_NAME_NOT_FOUND: NTSTATUS = 0xC0000034u32 as NTSTATUS;
pub const STATUS_OBJECT_PATH_INVALID: NTSTATUS = 0xC0000039u32 as NTSTATUS;
pub const STATUS_INVALID_HANDLE: NTSTATUS = 0xC0000008u32 as NTSTATUS;
pub const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000Du32 as NTSTATUS;
pub const STATUS_NO_MEMORY: NTSTATUS = 0xC0000017u32 as NTSTATUS;
pub const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC0000002u32 as NTSTATUS;

pub const OBJ_CASE_INSENSITIVE: u32 = 0x00000040;
pub const OBJ_KERNEL_HANDLE: u32 = 0x00000200;

pub const FILE_READ_DATA: u32 = 0x0001;
pub const FILE_WRITE_DATA: u32 = 0x0002;
pub const FILE_APPEND_DATA: u32 = 0x0004;
pub const FILE_READ_EA: u32 = 0x0008;
pub const FILE_WRITE_EA: u32 = 0x0010;
pub const FILE_EXECUTE: u32 = 0x0020;
pub const FILE_READ_ATTRIBUTES: u32 = 0x0080;
pub const FILE_WRITE_ATTRIBUTES: u32 = 0x0100;

pub const FILE_SHARE_READ: u32 = 0x00000001;
pub const FILE_SHARE_WRITE: u32 = 0x00000002;
pub const FILE_SHARE_DELETE: u32 = 0x00000004;

pub const FILE_OPEN: u32 = 0x00000001;
pub const FILE_CREATE: u32 = 0x00000002;
pub const FILE_OPEN_IF: u32 = 0x00000003;
pub const FILE_OVERWRITE_IF: u32 = 0x00000005;

pub const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x00000020;
pub const FILE_NON_DIRECTORY_FILE: u32 = 0x00000040;

pub const GENERIC_READ: u32 = 0x80000000;
pub const GENERIC_WRITE: u32 = 0x40000000;

pub const INVALID_HANDLE_VALUE: HANDLE = !0 as HANDLE;
unsafe fn IntExpungeConsoleCommandHistory(
    lpExeName: LPCVOID,
    bUnicode: BOOLEAN,
) {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let ExpungeCommandHistoryRequest: PCONSOLE_EXPUNGECOMMANDHISTORY =
        &raw mut ApiMessage.Data.ExpungeCommandHistoryRequest;
    let mut CaptureBuffer: PCSR_CAPTURE_BUFFER = core::ptr::null_mut();

    let NumChars: USHORT = (if !lpExeName.is_null() {
        if bUnicode != 0 {
            wcslen(lpExeName as *const u16)
        } else {
            strlen(lpExeName as *const i8) as usize
        }
    } else {
        0usize
    }) as USHORT;

    if lpExeName.is_null() || NumChars == 0 {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return;
    }

    (*ExpungeCommandHistoryRequest).ConsoleHandle =
        (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*ExpungeCommandHistoryRequest).ExeLength = (NumChars as usize).wrapping_mul(
        if bUnicode != 0 {
            core::mem::size_of::<WCHAR>()
        } else {
            core::mem::size_of::<CHAR>()
        },
    ) as USHORT;
    (*ExpungeCommandHistoryRequest).Unicode = bUnicode;
    (*ExpungeCommandHistoryRequest).Unicode2 = (*ExpungeCommandHistoryRequest).Unicode;

    CaptureBuffer = CsrAllocateCaptureBuffer(1, (*ExpungeCommandHistoryRequest).ExeLength as ULONG);
    if CaptureBuffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return;
    }

    CsrCaptureMessageBuffer(
        CaptureBuffer,
        lpExeName as PVOID,
        (*ExpungeCommandHistoryRequest).ExeLength as ULONG,
        &raw mut (*ExpungeCommandHistoryRequest).ExeName as *mut PVOID,
    );

    CsrClientCallServer(
        &raw mut ApiMessage as PCSR_API_MESSAGE,
        CaptureBuffer,
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepExpungeCommandHistory),
        core::mem::size_of::<CONSOLE_EXPUNGECOMMANDHISTORY>() as ULONG,
    );

    CsrFreeCaptureBuffer(CaptureBuffer);

    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
    }
}
unsafe fn IntGetConsoleCommandHistory(
    lpHistory: LPVOID,
    cbHistory: DWORD,
    lpExeName: LPCVOID,
    bUnicode: BOOLEAN,
) -> DWORD {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetCommandHistoryRequest: PCONSOLE_GETCOMMANDHISTORY = &raw mut ApiMessage.Data.GetCommandHistoryRequest;
    let CaptureBuffer: PCSR_CAPTURE_BUFFER;

    let NumChars: USHORT = (if !lpExeName.is_null() {
        if bUnicode != 0 {
            wcslen(lpExeName as LPCWSTR)
        } else {
            strlen(lpExeName as LPCSTR) as usize
        }
    } else {
        0
    }) as USHORT;

    if lpExeName.is_null() || NumChars == 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    (*GetCommandHistoryRequest).ConsoleHandle = (*((*NtCurrentPeb()).ProcessParameters)).ConsoleHandle;
    (*GetCommandHistoryRequest).HistoryLength = cbHistory;
    (*GetCommandHistoryRequest).ExeLength = (NumChars as usize).wrapping_mul(
        if bUnicode != 0 {
            core::mem::size_of::<WCHAR>()
        } else {
            core::mem::size_of::<CHAR>()
        },
    ) as USHORT;
    (*GetCommandHistoryRequest).Unicode = bUnicode;
    (*GetCommandHistoryRequest).Unicode2 = (*GetCommandHistoryRequest).Unicode;

    CaptureBuffer = CsrAllocateCaptureBuffer(
        2,
        ((*GetCommandHistoryRequest).ExeLength as ULONG).wrapping_add(
            (*GetCommandHistoryRequest).HistoryLength,
        ),
    );
    if CaptureBuffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    CsrCaptureMessageBuffer(
        CaptureBuffer,
        lpExeName as PVOID,
        (*GetCommandHistoryRequest).ExeLength as ULONG,
        &raw mut (*GetCommandHistoryRequest).ExeName as PVOID as *mut PVOID,
    );

    CsrAllocateMessagePointer(
        CaptureBuffer,
        (*GetCommandHistoryRequest).HistoryLength,
        &raw mut (*GetCommandHistoryRequest).History,
    );

    CsrClientCallServer(
        &raw mut ApiMessage as PCSR_API_MESSAGE,
        CaptureBuffer,
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetCommandHistory),
        core::mem::size_of::<CONSOLE_GETCOMMANDHISTORY>() as ULONG,
    );

    if !NT_SUCCESS(ApiMessage.Status) {
        CsrFreeCaptureBuffer(CaptureBuffer);
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    RtlCopyMemory(
        lpHistory,
        (*GetCommandHistoryRequest).History as LPCVOID,
        (*GetCommandHistoryRequest).HistoryLength as SIZE_T,
    );

    CsrFreeCaptureBuffer(CaptureBuffer);

    return (*GetCommandHistoryRequest).HistoryLength;
}
unsafe fn IntGetConsoleCommandHistoryLength(
    lpExeName: LPCVOID,
    bUnicode: BOOL,
) -> DWORD {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetCommandHistoryLengthRequest: PCONSOLE_GETCOMMANDHISTORYLENGTH =
        &mut ApiMessage.Data.GetCommandHistoryLengthRequest;
    let mut CaptureBuffer: PCSR_CAPTURE_BUFFER = core::ptr::null_mut();

    let NumChars: USHORT = (if !lpExeName.is_null() {
        if bUnicode != 0 {
            wcslen(lpExeName as *const u16)
        } else {
            strlen(lpExeName as *const i8) as usize
        }
    } else {
        0
    }) as USHORT;

    if lpExeName.is_null() || NumChars == 0 {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return 0;
    }

    (*GetCommandHistoryLengthRequest).ConsoleHandle =
        (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*GetCommandHistoryLengthRequest).ExeLength = (NumChars as usize)
        .wrapping_mul(if bUnicode != 0 {
            core::mem::size_of::<WCHAR>()
        } else {
            core::mem::size_of::<CHAR>()
        }) as USHORT;
    (*GetCommandHistoryLengthRequest).Unicode = bUnicode as BOOLEAN;
    (*GetCommandHistoryLengthRequest).Unicode2 = (*GetCommandHistoryLengthRequest).Unicode;

    CaptureBuffer = CsrAllocateCaptureBuffer(
        1,
        (*GetCommandHistoryLengthRequest).ExeLength as ULONG,
    );
    if CaptureBuffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return 0;
    }

    CsrCaptureMessageBuffer(
        CaptureBuffer,
        lpExeName as PVOID,
        (*GetCommandHistoryLengthRequest).ExeLength as ULONG,
        &mut (*GetCommandHistoryLengthRequest).ExeName as *mut _ as *mut PVOID,
    );

    CsrClientCallServer(
        &mut ApiMessage as *mut _ as PCSR_API_MESSAGE,
        CaptureBuffer,
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetCommandHistoryLength),
        core::mem::size_of::<CONSOLE_GETCOMMANDHISTORYLENGTH>() as ULONG,
    );

    CsrFreeCaptureBuffer(CaptureBuffer);

    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return (*GetCommandHistoryLengthRequest).HistoryLength as DWORD;
}
unsafe fn IntSetConsoleNumberOfCommands(
    dwNumCommands: DWORD,
    lpExeName: LPCVOID,
    bUnicode: BOOLEAN,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let SetHistoryNumberCommandsRequest: PCONSOLE_SETHISTORYNUMBERCOMMANDS =
        &raw mut ApiMessage.Data.SetHistoryNumberCommandsRequest;
    let mut CaptureBuffer: PCSR_CAPTURE_BUFFER = core::ptr::null_mut();

    let NumChars: USHORT = (if !lpExeName.is_null() {
        if bUnicode != 0 {
            wcslen(lpExeName as LPCWSTR)
        } else {
            strlen(lpExeName as LPCSTR) as usize
        }
    } else {
        0
    }) as USHORT;

    if lpExeName.is_null() || NumChars == 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    (*SetHistoryNumberCommandsRequest).ConsoleHandle =
        (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*SetHistoryNumberCommandsRequest).NumCommands = dwNumCommands;
    (*SetHistoryNumberCommandsRequest).ExeLength = (NumChars as usize)
        .wrapping_mul(if bUnicode != 0 {
            core::mem::size_of::<WCHAR>()
        } else {
            core::mem::size_of::<CHAR>()
        }) as USHORT;
    (*SetHistoryNumberCommandsRequest).Unicode = bUnicode;
    (*SetHistoryNumberCommandsRequest).Unicode2 = (*SetHistoryNumberCommandsRequest).Unicode;

    CaptureBuffer = CsrAllocateCaptureBuffer(1, (*SetHistoryNumberCommandsRequest).ExeLength as ULONG);
    if CaptureBuffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return FALSE;
    }

    CsrCaptureMessageBuffer(
        CaptureBuffer,
        lpExeName as PVOID,
        (*SetHistoryNumberCommandsRequest).ExeLength as ULONG,
        &raw mut (*SetHistoryNumberCommandsRequest).ExeName as *mut PVOID,
    );

    CsrClientCallServer(
        &raw mut ApiMessage as PCSR_API_MESSAGE,
        CaptureBuffer,
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetNumberOfCommands),
        core::mem::size_of::<CONSOLE_SETHISTORYNUMBERCOMMANDS>() as ULONG,
    );

    CsrFreeCaptureBuffer(CaptureBuffer);

    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return FALSE;
    }

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn ExpungeConsoleCommandHistoryW(
    lpExeName: LPCWSTR,
) {
    IntExpungeConsoleCommandHistory(
        lpExeName as LPCVOID,
        1i8 as BOOLEAN,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ExpungeConsoleCommandHistoryA(
    lpExeName: LPCSTR,
) {
    IntExpungeConsoleCommandHistory(
        lpExeName as LPCVOID,
        FALSE as BOOLEAN,
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleCommandHistoryW(
    lpHistory: LPWSTR,
    cbHistory: DWORD,
    lpExeName: LPCWSTR,
) -> DWORD {
    return IntGetConsoleCommandHistory(
        lpHistory as LPVOID,
        cbHistory,
        lpExeName as LPCVOID,
        1i8 as BOOLEAN,
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleCommandHistoryA(
    lpHistory: LPSTR,
    cbHistory: DWORD,
    lpExeName: LPCSTR,
) -> DWORD {
    return IntGetConsoleCommandHistory(
        lpHistory as LPVOID,
        cbHistory,
        lpExeName as LPCVOID,
        0i32 as BOOLEAN,
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleCommandHistoryLengthW(
    lpExeName: LPCWSTR,
) -> DWORD {
    return IntGetConsoleCommandHistoryLength(lpExeName as LPCVOID, 1);
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleCommandHistoryLengthA(
    lpExeName: LPCSTR,
) -> DWORD {
    return IntGetConsoleCommandHistoryLength(lpExeName as LPCVOID, 0);
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleNumberOfCommandsW(
    dwNumCommands: DWORD,
    lpExeName: LPCWSTR,
) -> BOOL {
    return IntSetConsoleNumberOfCommands(dwNumCommands, lpExeName as LPCVOID, 1i8 as BOOLEAN);
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleNumberOfCommandsA(
    dwNumCommands: DWORD,
    lpExeName: LPCSTR,
) -> BOOL {
    return IntSetConsoleNumberOfCommands(dwNumCommands, lpExeName as LPCVOID, FALSE as BOOLEAN);
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleCommandHistoryMode(dwMode: DWORD) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let SetHistoryModeRequest: PCONSOLE_SETHISTORYMODE =
        &raw mut ApiMessage.Data.SetHistoryModeRequest;

    (*SetHistoryModeRequest).ConsoleHandle =
        (*(*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessParameters).ConsoleHandle;
    (*SetHistoryModeRequest).Mode = dwMode;

    CsrClientCallServer(
        &raw mut ApiMessage as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetCommandHistoryMode),
        core::mem::size_of::<CONSOLE_SETHISTORYMODE>() as ULONG,
    );

    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return FALSE;
    }

    return TRUE;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/console/history.c
//  * PURPOSE:         Win32 Console Client history functions
//  * PROGRAMMERS:     Jeffrey Morlan
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
//
// static VOID
// IntExpungeConsoleCommandHistory(LPCVOID lpExeName, BOOLEAN bUnicode)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_EXPUNGECOMMANDHISTORY ExpungeCommandHistoryRequest = &ApiMessage.Data.ExpungeCommandHistoryRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer;
//
//     USHORT NumChars = (USHORT)(lpExeName ? (bUnicode ? wcslen(lpExeName) : strlen(lpExeName)) : 0);
//
//     if (lpExeName == NULL || NumChars == 0)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return;
//     }
//
//     ExpungeCommandHistoryRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     ExpungeCommandHistoryRequest->ExeLength     = NumChars * (bUnicode ? sizeof(WCHAR) : sizeof(CHAR));
//     ExpungeCommandHistoryRequest->Unicode  =
//     ExpungeCommandHistoryRequest->Unicode2 = bUnicode;
//
//     // CaptureBuffer = CsrAllocateCaptureBuffer(1, IntStringSize(lpExeName, bUnicode));
//     CaptureBuffer = CsrAllocateCaptureBuffer(1, ExpungeCommandHistoryRequest->ExeLength);
//     if (!CaptureBuffer)
//     {
//         DPRINT1("CsrAllocateCaptureBuffer failed!\n");
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return;
//     }
//
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             (PVOID)lpExeName,
//                             ExpungeCommandHistoryRequest->ExeLength,
//                             (PVOID)&ExpungeCommandHistoryRequest->ExeName);
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepExpungeCommandHistory),
//                         sizeof(*ExpungeCommandHistoryRequest));
//
//     CsrFreeCaptureBuffer(CaptureBuffer);
//
//     if (!NT_SUCCESS(ApiMessage.Status))
//         BaseSetLastNTError(ApiMessage.Status);
// }
//
//
// static DWORD
// IntGetConsoleCommandHistory(LPVOID lpHistory, DWORD cbHistory, LPCVOID lpExeName, BOOLEAN bUnicode)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETCOMMANDHISTORY GetCommandHistoryRequest = &ApiMessage.Data.GetCommandHistoryRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer;
//
//     USHORT NumChars = (USHORT)(lpExeName ? (bUnicode ? wcslen(lpExeName) : strlen(lpExeName)) : 0);
//
//     if (lpExeName == NULL || NumChars == 0)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     GetCommandHistoryRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     GetCommandHistoryRequest->HistoryLength = cbHistory;
//     GetCommandHistoryRequest->ExeLength     = NumChars * (bUnicode ? sizeof(WCHAR) : sizeof(CHAR));
//     GetCommandHistoryRequest->Unicode  =
//     GetCommandHistoryRequest->Unicode2 = bUnicode;
//
//     // CaptureBuffer = CsrAllocateCaptureBuffer(2, IntStringSize(lpExeName, bUnicode) +
//     //                                             HistoryLength);
//     CaptureBuffer = CsrAllocateCaptureBuffer(2, GetCommandHistoryRequest->ExeLength +
//                                                 GetCommandHistoryRequest->HistoryLength);
//     if (!CaptureBuffer)
//     {
//         DPRINT1("CsrAllocateCaptureBuffer failed!\n");
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return 0;
//     }
//
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             (PVOID)lpExeName,
//                             GetCommandHistoryRequest->ExeLength,
//                             (PVOID)&GetCommandHistoryRequest->ExeName);
//
//     CsrAllocateMessagePointer(CaptureBuffer, GetCommandHistoryRequest->HistoryLength,
//                               (PVOID*)&GetCommandHistoryRequest->History);
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetCommandHistory),
//                         sizeof(*GetCommandHistoryRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         CsrFreeCaptureBuffer(CaptureBuffer);
//         BaseSetLastNTError(ApiMessage.Status);
//         return 0;
//     }
//
//     RtlCopyMemory(lpHistory,
//                   GetCommandHistoryRequest->History,
//                   GetCommandHistoryRequest->HistoryLength);
//
//     CsrFreeCaptureBuffer(CaptureBuffer);
//
//     return GetCommandHistoryRequest->HistoryLength;
// }
//
//
// static DWORD
// IntGetConsoleCommandHistoryLength(LPCVOID lpExeName, BOOL bUnicode)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETCOMMANDHISTORYLENGTH GetCommandHistoryLengthRequest = &ApiMessage.Data.GetCommandHistoryLengthRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer;
//
//     USHORT NumChars = (USHORT)(lpExeName ? (bUnicode ? wcslen(lpExeName) : strlen(lpExeName)) : 0);
//
//     if (lpExeName == NULL || NumChars == 0)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     GetCommandHistoryLengthRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     GetCommandHistoryLengthRequest->ExeLength     = NumChars * (bUnicode ? sizeof(WCHAR) : sizeof(CHAR));
//     GetCommandHistoryLengthRequest->Unicode  =
//     GetCommandHistoryLengthRequest->Unicode2 = bUnicode;
//
//     // CaptureBuffer = CsrAllocateCaptureBuffer(1, IntStringSize(lpExeName, bUnicode));
//     CaptureBuffer = CsrAllocateCaptureBuffer(1, GetCommandHistoryLengthRequest->ExeLength);
//     if (!CaptureBuffer)
//     {
//         DPRINT1("CsrAllocateCaptureBuffer failed!\n");
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return 0;
//     }
//
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             (PVOID)lpExeName,
//                             GetCommandHistoryLengthRequest->ExeLength,
//                             (PVOID)&GetCommandHistoryLengthRequest->ExeName);
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetCommandHistoryLength),
//                         sizeof(*GetCommandHistoryLengthRequest));
//
//     CsrFreeCaptureBuffer(CaptureBuffer);
//
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return 0;
//     }
//
//     return GetCommandHistoryLengthRequest->HistoryLength;
// }
//
//
// static BOOL
// IntSetConsoleNumberOfCommands(DWORD dwNumCommands,
//                               LPCVOID lpExeName,
//                               BOOLEAN bUnicode)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_SETHISTORYNUMBERCOMMANDS SetHistoryNumberCommandsRequest = &ApiMessage.Data.SetHistoryNumberCommandsRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer;
//
//     USHORT NumChars = (USHORT)(lpExeName ? (bUnicode ? wcslen(lpExeName) : strlen(lpExeName)) : 0);
//
//     if (lpExeName == NULL || NumChars == 0)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     SetHistoryNumberCommandsRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     SetHistoryNumberCommandsRequest->NumCommands   = dwNumCommands;
//     SetHistoryNumberCommandsRequest->ExeLength     = NumChars * (bUnicode ? sizeof(WCHAR) : sizeof(CHAR));
//     SetHistoryNumberCommandsRequest->Unicode  =
//     SetHistoryNumberCommandsRequest->Unicode2 = bUnicode;
//
//     // CaptureBuffer = CsrAllocateCaptureBuffer(1, IntStringSize(lpExeName, bUnicode));
//     CaptureBuffer = CsrAllocateCaptureBuffer(1, SetHistoryNumberCommandsRequest->ExeLength);
//     if (!CaptureBuffer)
//     {
//         DPRINT1("CsrAllocateCaptureBuffer failed!\n");
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             (PVOID)lpExeName,
//                             SetHistoryNumberCommandsRequest->ExeLength,
//                             (PVOID)&SetHistoryNumberCommandsRequest->ExeName);
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetNumberOfCommands),
//                         sizeof(*SetHistoryNumberCommandsRequest));
//
//     CsrFreeCaptureBuffer(CaptureBuffer);
//
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /* FUNCTIONS ******************************************************************/
//
// /*
//  * @implemented (Undocumented)
//  */
// VOID
// WINAPI
// DECLSPEC_HOTPATCH
// ExpungeConsoleCommandHistoryW(IN LPCWSTR lpExeName)
// {
//     IntExpungeConsoleCommandHistory(lpExeName, TRUE);
// }
//
//
// /*
//  * @implemented (Undocumented)
//  */
// VOID
// WINAPI
// DECLSPEC_HOTPATCH
// ExpungeConsoleCommandHistoryA(IN LPCSTR lpExeName)
// {
//     IntExpungeConsoleCommandHistory(lpExeName, FALSE);
// }
//
//
// /*
//  * @implemented (Undocumented)
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleCommandHistoryW(OUT LPWSTR lpHistory,
//                           IN DWORD cbHistory,
//                           IN LPCWSTR lpExeName)
// {
//     return IntGetConsoleCommandHistory(lpHistory, cbHistory, lpExeName, TRUE);
// }
//
//
// /*
//  * @implemented (Undocumented)
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleCommandHistoryA(OUT LPSTR lpHistory,
//                           IN DWORD cbHistory,
//                           IN LPCSTR lpExeName)
// {
//     return IntGetConsoleCommandHistory(lpHistory, cbHistory, lpExeName, FALSE);
// }
//
//
// /*
//  * @implemented (Undocumented)
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleCommandHistoryLengthW(IN LPCWSTR lpExeName)
// {
//     return IntGetConsoleCommandHistoryLength(lpExeName, TRUE);
// }
//
//
// /*
//  * @implemented (Undocumented)
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleCommandHistoryLengthA(IN LPCSTR lpExeName)
// {
//     return IntGetConsoleCommandHistoryLength(lpExeName, FALSE);
// }
//
//
// /*
//  * @implemented (Undocumented)
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleNumberOfCommandsW(IN DWORD dwNumCommands,
//                             IN LPCWSTR lpExeName)
// {
//     return IntSetConsoleNumberOfCommands(dwNumCommands, lpExeName, TRUE);
// }
//
//
// /*
//  * @implemented (Undocumented)
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleNumberOfCommandsA(IN DWORD dwNumCommands,
//                             IN LPCSTR lpExeName)
// {
//     return IntSetConsoleNumberOfCommands(dwNumCommands, lpExeName, FALSE);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleCommandHistoryMode(IN DWORD dwMode)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_SETHISTORYMODE SetHistoryModeRequest = &ApiMessage.Data.SetHistoryModeRequest;
//
//     SetHistoryModeRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     SetHistoryModeRequest->Mode          = dwMode;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetCommandHistoryMode),
//                         sizeof(*SetHistoryModeRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
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
