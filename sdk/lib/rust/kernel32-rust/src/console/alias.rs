use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _ALIAS_ENTRY {
    pub Name: PWSTR,
    pub NameLength: ULONG,
    pub Value: PWSTR,
    pub ValueLength: ULONG,
}
pub type PALIAS_ENTRY = *mut _ALIAS_ENTRY;

#[repr(C)]
pub struct _ALIAS_HEADER {
    pub Magic: ULONG,
    pub Count: ULONG,
    pub Entries: [ALIAS_ENTRY; 1],
}
pub type PALIAS_HEADER = *mut _ALIAS_HEADER;

pub const ALIAS_MAGIC: ULONG = 0x414C4941; // 'ALIA'

pub const CSR_ALIAS_QUERY: ULONG = 0x00000001;
pub const CSR_ALIAS_SET: ULONG = 0x00000002;
pub const CSR_ALIAS_DELETE: ULONG = 0x00000003;
pub const CSR_ALIAS_ENUMERATE: ULONG = 0x00000004;

pub const ALIAS_MAX_NAME_LENGTH: ULONG = 256;
pub const ALIAS_MAX_VALUE_LENGTH: ULONG = 1024;
pub const ALIAS_MAX_ENTRIES: ULONG = 1024;
unsafe fn IntAddConsoleAlias(
    Source: LPCVOID,
    SourceBufferLength: USHORT,
    Target: LPCVOID,
    TargetBufferLength: USHORT,
    ExeName: LPCVOID,
    bUnicode: BOOLEAN,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let ConsoleAliasRequest: PCONSOLE_ADDGETALIAS = &raw mut ApiMessage.Data.ConsoleAliasRequest;
    let mut CaptureBuffer: PCSR_CAPTURE_BUFFER;
    let mut CapturedStrings: ULONG;

    let NumChars: USHORT = (if !ExeName.is_null() {
        if bUnicode != 0 {
            wcslen(ExeName as LPCWSTR)
        } else {
            strlen(ExeName as LPCSTR) as usize
        }
    } else {
        0
    }) as USHORT;

    if ExeName.is_null() || NumChars == 0 {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE;
    }

    (*ConsoleAliasRequest).ConsoleHandle = (*((*NtCurrentPeb()).ProcessParameters)).ConsoleHandle;

    (*ConsoleAliasRequest).SourceLength = SourceBufferLength;
    (*ConsoleAliasRequest).ExeLength = (NumChars as usize * if bUnicode != 0 {
        core::mem::size_of::<WCHAR>()
    } else {
        core::mem::size_of::<CHAR>()
    }) as USHORT;
    (*ConsoleAliasRequest).Unicode = bUnicode;
    (*ConsoleAliasRequest).Unicode2 = bUnicode;

    CapturedStrings = 2;

    if !Target.is_null() {
        (*ConsoleAliasRequest).TargetLength = TargetBufferLength;
        CapturedStrings += 1;
    } else {
        (*ConsoleAliasRequest).TargetLength = 0;
    }

    CaptureBuffer = CsrAllocateCaptureBuffer(
        CapturedStrings,
        ((*ConsoleAliasRequest).SourceLength as ULONG)
            .wrapping_add((*ConsoleAliasRequest).ExeLength as ULONG)
            .wrapping_add((*ConsoleAliasRequest).TargetLength as ULONG),
    );

    if CaptureBuffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return FALSE;
    }

    CsrCaptureMessageBuffer(
        CaptureBuffer,
        Source as PVOID,
        (*ConsoleAliasRequest).SourceLength as ULONG,
        &raw mut (*ConsoleAliasRequest).Source,
    );

    CsrCaptureMessageBuffer(
        CaptureBuffer,
        ExeName as PVOID,
        (*ConsoleAliasRequest).ExeLength as ULONG,
        &raw mut (*ConsoleAliasRequest).ExeName,
    );

    if !Target.is_null() {
        CsrCaptureMessageBuffer(
            CaptureBuffer,
            Target as PVOID,
            (*ConsoleAliasRequest).TargetLength as ULONG,
            &raw mut (*ConsoleAliasRequest).Target,
        );
    } else {
        (*ConsoleAliasRequest).Target = core::ptr::null_mut();
    }

    CsrClientCallServer(
        &raw mut ApiMessage as PCSR_API_MESSAGE,
        CaptureBuffer,
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepAddAlias),
        core::mem::size_of::<CONSOLE_ADDGETALIAS>() as ULONG,
    );

    CsrFreeCaptureBuffer(CaptureBuffer);

    if !NT_SUCCESS(ApiMessage.__bindgen_anon_1.Status) {
        BaseSetLastNTError(ApiMessage.__bindgen_anon_1.Status);
        return FALSE;
    }

    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn AddConsoleAliasW(
    Source: LPCWSTR,
    Target: LPCWSTR,
    ExeName: LPCWSTR,
) -> BOOL {
    let SourceBufferLength: USHORT = (wcslen(Source as *const u16) as USHORT as usize)
        .wrapping_mul(core::mem::size_of::<WCHAR>() as usize)
        as USHORT;
    let TargetBufferLength: USHORT = (if !Target.is_null() {
        (wcslen(Target as *const u16) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as usize)
    } else {
        0 as usize
    }) as USHORT;

    return IntAddConsoleAlias(
        Source as LPCVOID,
        SourceBufferLength,
        Target as LPCVOID,
        TargetBufferLength,
        ExeName as LPCVOID,
        TRUE as BOOLEAN,
    );
}
#[no_mangle]
pub unsafe extern "system" fn AddConsoleAliasA(
    Source: LPCSTR,
    Target: LPCSTR,
    ExeName: LPCSTR,
) -> BOOL {
    let SourceBufferLength: USHORT = (strlen(Source as *const i8) as USHORT as usize)
        .wrapping_mul(core::mem::size_of::<CHAR>() as usize) as USHORT;
    let TargetBufferLength: USHORT = (if !Target.is_null() {
        (strlen(Target as *const i8) as usize)
            .wrapping_mul(core::mem::size_of::<CHAR>() as usize)
    } else {
        0 as usize
    }) as USHORT;

    return IntAddConsoleAlias(
        Source as LPCVOID,
        SourceBufferLength,
        Target as LPCVOID,
        TargetBufferLength,
        ExeName as LPCVOID,
        FALSE as BOOLEAN,
    );
}
unsafe fn IntGetConsoleAlias(
    Source: LPCVOID,
    SourceBufferLength: USHORT,
    Target: LPVOID,
    TargetBufferLength: USHORT,
    ExeName: LPCVOID,
    bUnicode: BOOLEAN,
) -> DWORD {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let ConsoleAliasRequest: PCONSOLE_ADDGETALIAS = &raw mut ApiMessage.Data.ConsoleAliasRequest;
    let mut CaptureBuffer: PCSR_CAPTURE_BUFFER;

    let NumChars: USHORT = (if !ExeName.is_null() {
        if bUnicode != 0 {
            wcslen(ExeName as LPCWSTR)
        } else {
            strlen(ExeName as LPCSTR) as usize
        }
    } else {
        0
    }) as USHORT;

    if Source.is_null() || Target.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if ExeName.is_null() || NumChars == 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    (*ConsoleAliasRequest).ConsoleHandle = (*((*NtCurrentPeb()).ProcessParameters)).ConsoleHandle;

    (*ConsoleAliasRequest).SourceLength = SourceBufferLength;
    (*ConsoleAliasRequest).ExeLength = (NumChars as usize * if bUnicode != 0 {
        core::mem::size_of::<WCHAR>()
    } else {
        core::mem::size_of::<CHAR>()
    }) as USHORT;
    (*ConsoleAliasRequest).Unicode = bUnicode;
    (*ConsoleAliasRequest).Unicode2 = (*ConsoleAliasRequest).Unicode;
    (*ConsoleAliasRequest).TargetLength = TargetBufferLength;

    CaptureBuffer = CsrAllocateCaptureBuffer(
        3,
        ((*ConsoleAliasRequest).SourceLength as u32)
            .wrapping_add((*ConsoleAliasRequest).ExeLength as u32)
            .wrapping_add((*ConsoleAliasRequest).TargetLength as u32),
    );
    if CaptureBuffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    CsrCaptureMessageBuffer(
        CaptureBuffer,
        Source as PVOID,
        (*ConsoleAliasRequest).SourceLength as ULONG,
        &raw mut (*ConsoleAliasRequest).Source as *mut PVOID,
    );

    CsrCaptureMessageBuffer(
        CaptureBuffer,
        ExeName as PVOID,
        (*ConsoleAliasRequest).ExeLength as ULONG,
        &raw mut (*ConsoleAliasRequest).ExeName as *mut PVOID,
    );

    CsrAllocateMessagePointer(
        CaptureBuffer,
        (*ConsoleAliasRequest).TargetLength as ULONG,
        &raw mut (*ConsoleAliasRequest).Target as *mut PVOID,
    );

    CsrClientCallServer(
        &raw mut ApiMessage as PCSR_API_MESSAGE,
        CaptureBuffer,
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetAlias),
        core::mem::size_of::<CONSOLE_ADDGETALIAS>() as ULONG,
    );

    if !NT_SUCCESS(ApiMessage.__bindgen_anon_1.Status) {
        CsrFreeCaptureBuffer(CaptureBuffer);
        BaseSetLastNTError(ApiMessage.__bindgen_anon_1.Status);

        if ApiMessage.__bindgen_anon_1.Status == STATUS_BUFFER_TOO_SMALL {
            return (*ConsoleAliasRequest).TargetLength as DWORD;
        } else {
            return 0;
        }
    }

    RtlCopyMemory(
        Target,
        (*ConsoleAliasRequest).Target as LPCVOID,
        (*ConsoleAliasRequest).TargetLength as usize,
    );

    CsrFreeCaptureBuffer(CaptureBuffer);

    return (*ConsoleAliasRequest).TargetLength as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleAliasW(
    Source: LPCWSTR,
    TargetBuffer: LPWSTR,
    TargetBufferLength: DWORD,
    ExeName: LPCWSTR,
) -> DWORD {
    return IntGetConsoleAlias(
        Source as LPCVOID,
        (wcslen(Source as *const u16) as USHORT as usize)
            .wrapping_mul(core::mem::size_of::<u16>() as usize) as USHORT,
        TargetBuffer as LPVOID,
        TargetBufferLength as USHORT,
        ExeName as LPCVOID,
        1 as BOOLEAN,
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleAliasA(
    Source: LPCSTR,
    TargetBuffer: LPSTR,
    TargetBufferLength: DWORD,
    ExeName: LPCSTR,
) -> DWORD {
    IntGetConsoleAlias(
        Source as LPCVOID,
        (strlen(Source as *const ::core::ffi::c_char) as USHORT as usize)
            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as USHORT,
        TargetBuffer as LPVOID,
        TargetBufferLength as USHORT,
        ExeName as LPCVOID,
        FALSE as BOOLEAN,
    )
}
unsafe fn IntGetConsoleAliases(
    AliasBuffer: LPVOID,
    AliasBufferLength: DWORD,
    ExeName: LPCVOID,
    bUnicode: BOOLEAN,
) -> DWORD {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetAllAliasesRequest: PCONSOLE_GETALLALIASES = &raw mut ApiMessage.Data.GetAllAliasesRequest;
    let mut CaptureBuffer: PCSR_CAPTURE_BUFFER = core::ptr::null_mut();

    let NumChars: USHORT = (if !ExeName.is_null() {
        if bUnicode != 0 {
            wcslen(ExeName as *const u16)
        } else {
            strlen(ExeName as *const i8) as usize
        }
    } else {
        0
    }) as USHORT;

    if ExeName.is_null() || NumChars == 0 {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return 0;
    }

    (*GetAllAliasesRequest).ConsoleHandle =
        (*(*(*NtCurrentPeb()).ProcessParameters)).ConsoleHandle;

    (*GetAllAliasesRequest).ExeLength = (NumChars as usize).wrapping_mul(
        if bUnicode != 0 {
            core::mem::size_of::<WCHAR>()
        } else {
            core::mem::size_of::<CHAR>()
        },
    ) as USHORT;

    (*GetAllAliasesRequest).Unicode2 = bUnicode;
    (*GetAllAliasesRequest).Unicode = (*GetAllAliasesRequest).Unicode2;

    (*GetAllAliasesRequest).AliasesBufferLength = AliasBufferLength as ULONG;

    CaptureBuffer = CsrAllocateCaptureBuffer(
        2,
        ((*GetAllAliasesRequest).ExeLength as ULONG)
            .wrapping_add((*GetAllAliasesRequest).AliasesBufferLength),
    );

    if CaptureBuffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return 0;
    }

    CsrCaptureMessageBuffer(
        CaptureBuffer,
        ExeName as PVOID,
        (*GetAllAliasesRequest).ExeLength as ULONG,
        &raw mut (*GetAllAliasesRequest).ExeName,
    );

    CsrAllocateMessagePointer(
        CaptureBuffer,
        (*GetAllAliasesRequest).AliasesBufferLength,
        &raw mut (*GetAllAliasesRequest).AliasesBuffer,
    );

    CsrClientCallServer(
        &raw mut ApiMessage as PCSR_API_MESSAGE,
        CaptureBuffer,
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetAliases),
        core::mem::size_of::<CONSOLE_GETALLALIASES>() as ULONG,
    );

    if !NT_SUCCESS(ApiMessage.__bindgen_anon_1.Status) {
        CsrFreeCaptureBuffer(CaptureBuffer);
        BaseSetLastNTError(ApiMessage.__bindgen_anon_1.Status);
        return 0;
    }

    RtlCopyMemory(
        AliasBuffer,
        (*GetAllAliasesRequest).AliasesBuffer as LPCVOID,
        (*GetAllAliasesRequest).AliasesBufferLength as usize,
    );

    CsrFreeCaptureBuffer(CaptureBuffer);

    return (*GetAllAliasesRequest).AliasesBufferLength as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleAliasesW(
    AliasBuffer: LPWSTR,
    AliasBufferLength: DWORD,
    ExeName: LPCWSTR,
) -> DWORD {
    IntGetConsoleAliases(
        AliasBuffer as LPVOID,
        AliasBufferLength,
        ExeName as LPCVOID,
        1i8 as BOOLEAN,
    )
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleAliasesA(
    AliasBuffer: LPSTR,
    AliasBufferLength: DWORD,
    ExeName: LPCSTR,
) -> DWORD {
    IntGetConsoleAliases(
        AliasBuffer as LPVOID,
        AliasBufferLength,
        ExeName as LPCVOID,
        0i8, // FALSE as BOOLEAN
    )
}
unsafe fn IntGetConsoleAliasesLength(
    ExeName: LPCVOID,
    bUnicode: BOOLEAN,
) -> DWORD {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetAllAliasesLengthRequest: PCONSOLE_GETALLALIASESLENGTH =
        &raw mut ApiMessage.Data.GetAllAliasesLengthRequest;
    let CaptureBuffer: PCSR_CAPTURE_BUFFER;

    let NumChars: USHORT = (if !ExeName.is_null() {
        if bUnicode != 0 {
            wcslen(ExeName as LPCWSTR)
        } else {
            strlen(ExeName as LPCSTR) as usize
        }
    } else {
        0
    }) as USHORT;

    if ExeName.is_null() || NumChars == 0 {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return 0;
    }

    (*GetAllAliasesLengthRequest).ConsoleHandle =
        (*(*(*NtCurrentPeb()).ProcessParameters)).ConsoleHandle;
    (*GetAllAliasesLengthRequest).ExeLength = (NumChars as usize)
        .wrapping_mul(if bUnicode != 0 {
            core::mem::size_of::<WCHAR>()
        } else {
            core::mem::size_of::<CHAR>()
        }) as USHORT;
    (*GetAllAliasesLengthRequest).Unicode = bUnicode;
    (*GetAllAliasesLengthRequest).Unicode2 = (*GetAllAliasesLengthRequest).Unicode;

    CaptureBuffer = CsrAllocateCaptureBuffer(1, (*GetAllAliasesLengthRequest).ExeLength as ULONG);
    if CaptureBuffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return 0;
    }

    CsrCaptureMessageBuffer(
        CaptureBuffer,
        ExeName as PVOID,
        (*GetAllAliasesLengthRequest).ExeLength as ULONG,
        &raw mut (*GetAllAliasesLengthRequest).ExeName as *mut PVOID,
    );

    CsrClientCallServer(
        &raw mut ApiMessage as PCSR_API_MESSAGE,
        CaptureBuffer,
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetAliasesLength),
        core::mem::size_of::<CONSOLE_GETALLALIASESLENGTH>() as ULONG,
    );

    CsrFreeCaptureBuffer(CaptureBuffer);

    if !NT_SUCCESS(ApiMessage.__bindgen_anon_1.Status) {
        BaseSetLastNTError(ApiMessage.__bindgen_anon_1.Status);
        return 0;
    }

    return (*GetAllAliasesLengthRequest).Length as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleAliasesLengthW(
    ExeName: LPCWSTR,
) -> DWORD {
    return IntGetConsoleAliasesLength(ExeName as LPCVOID, 1i8 as BOOLEAN);
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleAliasesLengthA(
    ExeName: LPCSTR,
) -> DWORD {
    return IntGetConsoleAliasesLength(ExeName as LPCVOID, 0i32 as BOOLEAN);
}
unsafe fn IntGetConsoleAliasExes(
    lpExeNameBuffer: PVOID,
    ExeNameBufferLength: DWORD,
    bUnicode: BOOLEAN,
) -> DWORD {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetAliasesExesRequest: PCONSOLE_GETALIASESEXES =
        &mut ApiMessage.Data.GetAliasesExesRequest;
    let CaptureBuffer: PCSR_CAPTURE_BUFFER;

    (*GetAliasesExesRequest).ConsoleHandle = (*((*NtCurrentPeb()).ProcessParameters)).ConsoleHandle;
    (*GetAliasesExesRequest).Length = ExeNameBufferLength;
    (*GetAliasesExesRequest).Unicode = bUnicode;

    CaptureBuffer = CsrAllocateCaptureBuffer(1, ExeNameBufferLength);
    if CaptureBuffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    CsrAllocateMessagePointer(
        CaptureBuffer,
        ExeNameBufferLength,
        &mut (*GetAliasesExesRequest).ExeNames as *mut _ as *mut PVOID,
    );

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        CaptureBuffer,
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetAliasExes),
        core::mem::size_of::<CONSOLE_GETALIASESEXES>() as ULONG,
    );

    if !NT_SUCCESS(ApiMessage.__bindgen_anon_1.Status) {
        CsrFreeCaptureBuffer(CaptureBuffer);
        BaseSetLastNTError(ApiMessage.__bindgen_anon_1.Status);
        return 0;
    }

    RtlCopyMemory(
        lpExeNameBuffer,
        (*GetAliasesExesRequest).ExeNames as PVOID,
        (*GetAliasesExesRequest).Length as usize,
    );

    CsrFreeCaptureBuffer(CaptureBuffer);

    return (*GetAliasesExesRequest).Length;
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleAliasExesW(
    lpExeNameBuffer: LPWSTR,
    ExeNameBufferLength: DWORD,
) -> DWORD {
    IntGetConsoleAliasExes(
        lpExeNameBuffer as PVOID,
        ExeNameBufferLength,
        1i8 as BOOLEAN,
    )
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleAliasExesA(
    lpExeNameBuffer: LPSTR,
    ExeNameBufferLength: DWORD,
) -> DWORD {
    IntGetConsoleAliasExes(
        lpExeNameBuffer as PVOID,
        ExeNameBufferLength,
        0i8 as BOOLEAN,
    )
}
unsafe fn IntGetConsoleAliasExesLength(mut bUnicode: BOOLEAN) -> DWORD {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetAliasesExesLengthRequest: PCONSOLE_GETALIASESEXESLENGTH =
        &raw mut ApiMessage.Data.GetAliasesExesLengthRequest;

    (*GetAliasesExesLengthRequest).ConsoleHandle =
        (*(*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessParameters).ConsoleHandle;
    (*GetAliasesExesLengthRequest).Unicode = bUnicode;

    CsrClientCallServer(
        &raw mut ApiMessage as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetAliasExesLength),
        core::mem::size_of::<CONSOLE_GETALIASESEXESLENGTH>() as ULONG,
    );
    if !NT_SUCCESS(ApiMessage.__bindgen_anon_1.Status) {
        BaseSetLastNTError(ApiMessage.__bindgen_anon_1.Status);
        return 0;
    }

    return (*GetAliasesExesLengthRequest).Length;
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleAliasExesLengthW() -> DWORD {
    return IntGetConsoleAliasExesLength(1i32 as BOOLEAN);
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleAliasExesLengthA() -> DWORD {
    IntGetConsoleAliasExesLength(0i8 as BOOLEAN)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/console/alias.c
//  * PURPOSE:         Win32 Console Client Alias support functions
//  * PROGRAMMERS:     David Welch (welch@cwcom.net) (welch@mcmail.com)
//  *                  Christoph von Wittich (christoph_vw@reactos.org)
//  *                  Johannes Anderwald (johannes.anderwald@reactos.org)
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
// /* FUNCTIONS ******************************************************************/
//
// static BOOL
// IntAddConsoleAlias(LPCVOID Source,
//                    USHORT SourceBufferLength,
//                    LPCVOID Target,
//                    USHORT TargetBufferLength,
//                    LPCVOID ExeName,
//                    BOOLEAN bUnicode)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_ADDGETALIAS ConsoleAliasRequest = &ApiMessage.Data.ConsoleAliasRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer;
//     ULONG CapturedStrings;
//
//     USHORT NumChars = (USHORT)(ExeName ? (bUnicode ? wcslen(ExeName) : strlen(ExeName)) : 0);
//
//     if (ExeName == NULL || NumChars == 0)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     ConsoleAliasRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//
//     /* Determine the needed sizes */
//     ConsoleAliasRequest->SourceLength = SourceBufferLength;
//     ConsoleAliasRequest->ExeLength    = NumChars * (bUnicode ? sizeof(WCHAR) : sizeof(CHAR));
//     ConsoleAliasRequest->Unicode  =
//     ConsoleAliasRequest->Unicode2 = bUnicode;
//
//     CapturedStrings = 2;
//
//     if (Target) /* The target can be optional */
//     {
//         ConsoleAliasRequest->TargetLength = TargetBufferLength;
//         CapturedStrings++;
//     }
//     else
//     {
//         ConsoleAliasRequest->TargetLength = 0;
//     }
//
//     /* Allocate a Capture Buffer */
//     CaptureBuffer = CsrAllocateCaptureBuffer(CapturedStrings,
//                                              ConsoleAliasRequest->SourceLength +
//                                              ConsoleAliasRequest->ExeLength    +
//                                              ConsoleAliasRequest->TargetLength);
//     if (CaptureBuffer == NULL)
//     {
//         DPRINT1("CsrAllocateCaptureBuffer failed!\n");
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     /* Capture the strings */
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             (PVOID)Source,
//                             ConsoleAliasRequest->SourceLength,
//                             (PVOID*)&ConsoleAliasRequest->Source);
//
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             (PVOID)ExeName,
//                             ConsoleAliasRequest->ExeLength,
//                             (PVOID*)&ConsoleAliasRequest->ExeName);
//
//     if (Target) /* The target can be optional */
//     {
//         CsrCaptureMessageBuffer(CaptureBuffer,
//                                 (PVOID)Target,
//                                 ConsoleAliasRequest->TargetLength,
//                                 (PVOID*)&ConsoleAliasRequest->Target);
//     }
//     else
//     {
//         ConsoleAliasRequest->Target = NULL;
//     }
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepAddAlias),
//                         sizeof(*ConsoleAliasRequest));
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
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// AddConsoleAliasW(
//      LPCWSTR Source,
//      LPCWSTR Target,
//      LPCWSTR ExeName)
// {
//     USHORT SourceBufferLength = (USHORT)wcslen(Source) * sizeof(WCHAR);
//     USHORT TargetBufferLength = (USHORT)(Target ? wcslen(Target) * sizeof(WCHAR) : 0);
//
//     DPRINT("AddConsoleAliasW entered with Source '%S' Target '%S' ExeName '%S'\n",
//            Source, Target, ExeName);
//
//     return IntAddConsoleAlias(Source,
//                               SourceBufferLength,
//                               Target,
//                               TargetBufferLength,
//                               ExeName,
//                               TRUE);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// AddConsoleAliasA(
//      LPCSTR Source,
//      LPCSTR Target,
//      LPCSTR ExeName)
// {
//     USHORT SourceBufferLength = (USHORT)strlen(Source) * sizeof(CHAR);
//     USHORT TargetBufferLength = (USHORT)(Target ? strlen(Target) * sizeof(CHAR) : 0);
//
//     DPRINT("AddConsoleAliasA entered with Source '%s' Target '%s' ExeName '%s'\n",
//            Source, Target, ExeName);
//
//     return IntAddConsoleAlias(Source,
//                               SourceBufferLength,
//                               Target,
//                               TargetBufferLength,
//                               ExeName,
//                               FALSE);
// }
//
//
// static DWORD
// IntGetConsoleAlias(LPCVOID Source,
//                    USHORT SourceBufferLength,
//                    LPVOID Target,
//                    USHORT TargetBufferLength,
//                    LPCVOID ExeName,
//                    BOOLEAN bUnicode)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_ADDGETALIAS ConsoleAliasRequest = &ApiMessage.Data.ConsoleAliasRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer;
//
//     USHORT NumChars = (USHORT)(ExeName ? (bUnicode ? wcslen(ExeName) : strlen(ExeName)) : 0);
//
//     if (Source == NULL || Target == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     if (ExeName == NULL || NumChars == 0)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     ConsoleAliasRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//
//     /* Determine the needed sizes */
//     ConsoleAliasRequest->SourceLength = SourceBufferLength;
//     ConsoleAliasRequest->ExeLength    = NumChars * (bUnicode ? sizeof(WCHAR) : sizeof(CHAR));
//     ConsoleAliasRequest->Unicode  =
//     ConsoleAliasRequest->Unicode2 = bUnicode;
//
//     ConsoleAliasRequest->TargetLength = TargetBufferLength;
//
//     /* Allocate a Capture Buffer */
//     CaptureBuffer = CsrAllocateCaptureBuffer(3, ConsoleAliasRequest->SourceLength +
//                                                 ConsoleAliasRequest->ExeLength    +
//                                                 ConsoleAliasRequest->TargetLength);
//     if (!CaptureBuffer)
//     {
//         DPRINT1("CsrAllocateCaptureBuffer failed!\n");
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return 0;
//     }
//
//     /* Capture the strings */
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             (PVOID)Source,
//                             ConsoleAliasRequest->SourceLength,
//                             (PVOID*)&ConsoleAliasRequest->Source);
//
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             (PVOID)ExeName,
//                             ConsoleAliasRequest->ExeLength,
//                             (PVOID*)&ConsoleAliasRequest->ExeName);
//
//     /* Allocate space for the target buffer */
//     CsrAllocateMessagePointer(CaptureBuffer,
//                               ConsoleAliasRequest->TargetLength,
//                               (PVOID*)&ConsoleAliasRequest->Target);
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetAlias),
//                         sizeof(*ConsoleAliasRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         CsrFreeCaptureBuffer(CaptureBuffer);
//         BaseSetLastNTError(ApiMessage.Status);
//
//         if (ApiMessage.Status == STATUS_BUFFER_TOO_SMALL)
//             return ConsoleAliasRequest->TargetLength;
//         else
//             return 0;
//     }
//
//     /* Copy the returned target string into the user buffer */
//     RtlCopyMemory(Target,
//                   ConsoleAliasRequest->Target,
//                   ConsoleAliasRequest->TargetLength);
//
//     /* Release the capture buffer and exit */
//     CsrFreeCaptureBuffer(CaptureBuffer);
//
//     return ConsoleAliasRequest->TargetLength;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleAliasW(
//      LPCWSTR Source,
//      LPWSTR TargetBuffer,
//      DWORD TargetBufferLength,
//      LPCWSTR ExeName)
// {
//     DPRINT("GetConsoleAliasW entered with Source '%S' ExeName '%S'\n",
//            Source, ExeName);
//
//     return IntGetConsoleAlias(Source,
//                               (USHORT)wcslen(Source) * sizeof(WCHAR),
//                               TargetBuffer,
//                               TargetBufferLength,
//                               ExeName,
//                               TRUE);
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleAliasA(
//      LPCSTR Source,
//      LPSTR TargetBuffer,
//      DWORD TargetBufferLength,
//      LPCSTR ExeName)
// {
//     DPRINT("GetConsoleAliasA entered with Source '%s' ExeName '%s'\n",
//            Source, ExeName);
//
//     return IntGetConsoleAlias(Source,
//                               (USHORT)strlen(Source) * sizeof(CHAR),
//                               TargetBuffer,
//                               TargetBufferLength,
//                               ExeName,
//                               FALSE);
// }
//
//
// static DWORD
// IntGetConsoleAliases(LPVOID AliasBuffer,
//                      DWORD AliasBufferLength,
//                      LPCVOID ExeName,
//                      BOOLEAN bUnicode)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETALLALIASES GetAllAliasesRequest = &ApiMessage.Data.GetAllAliasesRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer;
//
//     USHORT NumChars = (USHORT)(ExeName ? (bUnicode ? wcslen(ExeName) : strlen(ExeName)) : 0);
//
//     if (ExeName == NULL || NumChars == 0)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     GetAllAliasesRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//
//     /* Determine the needed sizes */
//     GetAllAliasesRequest->ExeLength = NumChars * (bUnicode ? sizeof(WCHAR) : sizeof(CHAR));
//     GetAllAliasesRequest->Unicode   =
//     GetAllAliasesRequest->Unicode2  = bUnicode;
//
//     GetAllAliasesRequest->AliasesBufferLength = AliasBufferLength;
//
//     /* Allocate a Capture Buffer */
//     CaptureBuffer = CsrAllocateCaptureBuffer(2, GetAllAliasesRequest->ExeLength +
//                                                 GetAllAliasesRequest->AliasesBufferLength);
//     if (!CaptureBuffer)
//     {
//         DPRINT1("CsrAllocateCaptureBuffer failed!\n");
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return 0;
//     }
//
//     /* Capture the exe name and allocate space for the aliases buffer */
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             (PVOID)ExeName,
//                             GetAllAliasesRequest->ExeLength,
//                             (PVOID*)&GetAllAliasesRequest->ExeName);
//
//     CsrAllocateMessagePointer(CaptureBuffer,
//                               GetAllAliasesRequest->AliasesBufferLength,
//                               (PVOID*)&GetAllAliasesRequest->AliasesBuffer);
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetAliases),
//                         sizeof(*GetAllAliasesRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         CsrFreeCaptureBuffer(CaptureBuffer);
//         BaseSetLastNTError(ApiMessage.Status);
//         return 0;
//     }
//
//     /* Copy the returned aliases string into the user buffer */
//     RtlCopyMemory(AliasBuffer,
//                   GetAllAliasesRequest->AliasesBuffer,
//                   GetAllAliasesRequest->AliasesBufferLength);
//
//     /* Release the capture buffer and exit */
//     CsrFreeCaptureBuffer(CaptureBuffer);
//
//     return GetAllAliasesRequest->AliasesBufferLength;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleAliasesW(
//      LPWSTR AliasBuffer,
//      DWORD AliasBufferLength,
//      LPCWSTR ExeName)
// {
//     DPRINT("GetConsoleAliasesW entered with ExeName '%S'\n",
//             ExeName);
//
//     return IntGetConsoleAliases(AliasBuffer,
//                                 AliasBufferLength,
//                                 ExeName,
//                                 TRUE);
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleAliasesA(
//      LPSTR AliasBuffer,
//      DWORD AliasBufferLength,
//      LPCSTR ExeName)
// {
//     DPRINT("GetConsoleAliasesA entered with ExeName '%s'\n",
//             ExeName);
//
//     return IntGetConsoleAliases(AliasBuffer,
//                                 AliasBufferLength,
//                                 ExeName,
//                                 FALSE);
// }
//
//
// static DWORD
// IntGetConsoleAliasesLength(LPCVOID ExeName, BOOLEAN bUnicode)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETALLALIASESLENGTH GetAllAliasesLengthRequest = &ApiMessage.Data.GetAllAliasesLengthRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer;
//
//     USHORT NumChars = (USHORT)(ExeName ? (bUnicode ? wcslen(ExeName) : strlen(ExeName)) : 0);
//
//     if (ExeName == NULL || NumChars == 0)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     GetAllAliasesLengthRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     GetAllAliasesLengthRequest->ExeLength     = NumChars * (bUnicode ? sizeof(WCHAR) : sizeof(CHAR));
//     GetAllAliasesLengthRequest->Unicode  =
//     GetAllAliasesLengthRequest->Unicode2 = bUnicode;
//
//     CaptureBuffer = CsrAllocateCaptureBuffer(1, GetAllAliasesLengthRequest->ExeLength);
//     if (!CaptureBuffer)
//     {
//         DPRINT1("CsrAllocateCaptureBuffer failed!\n");
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return 0;
//     }
//
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             (PVOID)ExeName,
//                             GetAllAliasesLengthRequest->ExeLength,
//                             (PVOID)&GetAllAliasesLengthRequest->ExeName);
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetAliasesLength),
//                         sizeof(*GetAllAliasesLengthRequest));
//
//     CsrFreeCaptureBuffer(CaptureBuffer);
//
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return 0;
//     }
//
//     return GetAllAliasesLengthRequest->Length;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleAliasesLengthW(
//      LPCWSTR ExeName)
// {
//     return IntGetConsoleAliasesLength(ExeName, TRUE);
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleAliasesLengthA(
//      LPCSTR ExeName)
// {
//     return IntGetConsoleAliasesLength(ExeName, FALSE);
// }
//
//
// static DWORD
// IntGetConsoleAliasExes(PVOID lpExeNameBuffer,
//                        DWORD ExeNameBufferLength,
//                        BOOLEAN bUnicode)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETALIASESEXES GetAliasesExesRequest = &ApiMessage.Data.GetAliasesExesRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer;
//
//     GetAliasesExesRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     GetAliasesExesRequest->Length        = ExeNameBufferLength;
//     GetAliasesExesRequest->Unicode       = bUnicode;
//
//     CaptureBuffer = CsrAllocateCaptureBuffer(1, ExeNameBufferLength);
//     if (!CaptureBuffer)
//     {
//         DPRINT1("CsrAllocateCaptureBuffer failed!\n");
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return 0;
//     }
//
//     CsrAllocateMessagePointer(CaptureBuffer,
//                               ExeNameBufferLength,
//                               (PVOID*)&GetAliasesExesRequest->ExeNames);
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetAliasExes),
//                         sizeof(*GetAliasesExesRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         CsrFreeCaptureBuffer(CaptureBuffer);
//         BaseSetLastNTError(ApiMessage.Status);
//         return 0;
//     }
//
//     RtlCopyMemory(lpExeNameBuffer,
//                   GetAliasesExesRequest->ExeNames,
//                   GetAliasesExesRequest->Length);
//
//     CsrFreeCaptureBuffer(CaptureBuffer);
//
//     return GetAliasesExesRequest->Length;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleAliasExesW(LPWSTR lpExeNameBuffer,
//                      DWORD ExeNameBufferLength)
// {
//     DPRINT("GetConsoleAliasExesW called\n");
//     return IntGetConsoleAliasExes(lpExeNameBuffer, ExeNameBufferLength, TRUE);
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleAliasExesA(LPSTR lpExeNameBuffer,
//                      DWORD ExeNameBufferLength)
// {
//     DPRINT("GetConsoleAliasExesA called\n");
//     return IntGetConsoleAliasExes(lpExeNameBuffer, ExeNameBufferLength, FALSE);
// }
//
//
// static DWORD
// IntGetConsoleAliasExesLength(BOOLEAN bUnicode)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETALIASESEXESLENGTH GetAliasesExesLengthRequest = &ApiMessage.Data.GetAliasesExesLengthRequest;
//
//     GetAliasesExesLengthRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     GetAliasesExesLengthRequest->Unicode = bUnicode;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetAliasExesLength),
//                         sizeof(*GetAliasesExesLengthRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return 0;
//     }
//
//     return GetAliasesExesLengthRequest->Length;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleAliasExesLengthW(VOID)
// {
//     DPRINT("GetConsoleAliasExesLengthW called\n");
//     return IntGetConsoleAliasExesLength(TRUE);
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleAliasExesLengthA(VOID)
// {
//     DPRINT("GetConsoleAliasExesLengthA called\n");
//     return IntGetConsoleAliasExesLength(FALSE);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
