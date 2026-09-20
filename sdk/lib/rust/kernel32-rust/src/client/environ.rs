use ntdll_sys::nt::*;

use crate::k32_externs::{BaseSetLastNTError, RtlCopyMemory, RtlGetProcessHeap};
use crate::string::nls_subst::NT_SUCCESS;

// ---- file-scope preamble (translated) ----
// The string/list types and the integer typedefs all come from the bindgen
// `ntdll_sys::nt::*` glob (so the Rtl* calls bind against the real layouts).
// Only the symbols the C file references that bindgen does NOT provide are
// declared here.

// RtlUnicodeStringToAnsiSize is a MACRO in C (rtlfuncs.h) that expands to a call
// to RtlxUnicodeStringToAnsiSize — bindgen doesn't emit it. Declare the export.
extern "system" {
    fn RtlxUnicodeStringToAnsiSize(UnicodeString: PCUNICODE_STRING) -> ULONG;
    fn SetLastError(dwErrCode: DWORD);
}

/// `RtlUnicodeStringToAnsiSize(s)` C macro (rtlfuncs.h).
#[inline]
unsafe fn RtlUnicodeStringToAnsiSize(s: PCUNICODE_STRING) -> ULONG {
    RtlxUnicodeStringToAnsiSize(s)
}

// `wcslen` — CRT.
extern "C" {
    fn wcslen(s: *const WCHAR) -> usize;
}

const UNICODE_NULL: WCHAR = 0;
const ANSI_NULL: CHAR = 0;
// UNICODE_STRING_MAX_CHARS = 0x7FFE, UNICODE_STRING_MAX_BYTES = 0xFFFE (ntdef.h)
const UNICODE_STRING_MAX_CHARS: USHORT = 0x7FFE;
const UNICODE_STRING_MAX_BYTES: USHORT = 0xFFFE;
const ERROR_CALL_NOT_IMPLEMENTED: DWORD = 120;

#[no_mangle]
pub unsafe extern "system" fn GetEnvironmentVariableA(
    lpName: LPCSTR,
    lpBuffer: LPSTR,
    nSize: DWORD,
) -> DWORD {
    let mut VarName: ANSI_STRING = core::mem::zeroed();
    let mut VarValue: ANSI_STRING = core::mem::zeroed();
    let mut VarNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut VarValueU: _UNICODE_STRING = core::mem::zeroed();
    let mut Buffer: PWSTR = core::ptr::null_mut();
    let mut Result: ULONG = 0;
    let mut UniSize: USHORT = 0;
    let mut Status: NTSTATUS = 0;

    /* The whole body down to Quickie lives in 'quickie; `goto Quickie` =>
     * `break 'quickie`. */
    'quickie: {
    /* Initialize all the strings */
    RtlInitAnsiString(&mut VarName, lpName);
    RtlInitUnicodeString(&mut VarNameU, core::ptr::null());
    RtlInitUnicodeString(&mut VarValueU, core::ptr::null());
    Status = RtlAnsiStringToUnicodeString(&mut VarNameU, &mut VarName, 1);
    if !NT_SUCCESS(Status) {
        break 'quickie;
    }

    /* Check if the size is too big to fit */
    UniSize = (UNICODE_STRING_MAX_CHARS - 2) as USHORT;
    if nSize <= UniSize as DWORD {
        /* It fits, but was there a string at all? */
        if nSize != 0 {
            /* Keep the given size, minus a NULL-char */
            UniSize = (nSize - 1) as USHORT;
        } else {
            /* No size */
            UniSize = 0;
        }
    } else {
        /* String is too big, so we need to return a NULL char as well */
        UniSize = UniSize.wrapping_sub(1);
    }

    /* Allocate the value string buffer */
    Buffer = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        (UniSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as PWSTR;
    if Buffer.is_null() {
        Status = STATUS_NO_MEMORY;
        break 'quickie;
    }

    /* And initialize its string */
    RtlInitEmptyUnicodeString(
        &mut VarValueU,
        Buffer,
        (UniSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as USHORT,
    );

    /* Acquire the PEB lock since we'll be querying variables now */
    RtlAcquirePebLock();

    /* Query the variable */
    Status = RtlQueryEnvironmentVariable_U(
        core::ptr::null_mut(),
        &VarNameU as PCUNICODE_STRING,
        &mut VarValueU,
    );
    if NT_SUCCESS(Status) && nSize == 0 {
        Status = STATUS_BUFFER_TOO_SMALL;
    }

    /* Check if we didn't have enough space */
    if Status == STATUS_BUFFER_TOO_SMALL {
        /* Fixup the length that the API returned */
        VarValueU.MaximumLength = (VarValueU.Length as usize)
            .wrapping_add(core::mem::size_of::<WCHAR>())
            as USHORT;

        /* Free old Unicode buffer */
        RtlFreeHeap(
            RtlGetProcessHeap(),
            0,
            VarValueU.Buffer as PVOID,
        );

        /* Allocate new one */
        Buffer = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            VarValueU.MaximumLength as SIZE_T,
        ) as PWSTR;
        if !Buffer.is_null() {
            /* Query the variable so we can know its size */
            VarValueU.Buffer = Buffer;
            Status = RtlQueryEnvironmentVariable_U(
                core::ptr::null_mut(),
                &VarNameU as PCUNICODE_STRING,
                &mut VarValueU,
            );
            if NT_SUCCESS(Status) {
                /* Get the ASCII length of the variable */
                Result = RtlUnicodeStringToAnsiSize(&VarValueU);
            }
        } else {
            /* Set failure status */
            Status = STATUS_NO_MEMORY;
            VarValueU.Buffer = core::ptr::null_mut();
        }
    } else if NT_SUCCESS(Status) {
        /* Check if the size is too big to fit */
        UniSize = (UNICODE_STRING_MAX_BYTES - 1) as USHORT;
        if nSize <= UniSize as DWORD {
            UniSize = nSize as USHORT;
        }

        /* Check the size */
        Result = RtlUnicodeStringToAnsiSize(&VarValueU);
        if Result <= UniSize as ULONG {
            /* Convert the string */
            RtlInitEmptyAnsiString(&mut VarValue, lpBuffer, UniSize);
            Status = RtlUnicodeStringToAnsiString(
                &mut VarValue,
                &VarValueU as PCUNICODE_STRING,
                0,
            );
            if NT_SUCCESS(Status) {
                /* NULL-terminate and set the final length */
                *lpBuffer.offset(VarValue.Length as isize) = ANSI_NULL;
                Result = VarValue.Length as ULONG;
            }
        }
    }

    /* Release the lock */
    RtlReleasePebLock();
    } // 'quickie

    /* Quickie: */
    /* Free the strings */
    RtlFreeUnicodeString(&mut VarNameU);
    if !VarValueU.Buffer.is_null() {
        RtlFreeHeap(
            RtlGetProcessHeap(),
            0,
            VarValueU.Buffer as PVOID,
        );
    }

    /* Check if we succeeded */
    if !NT_SUCCESS(Status) {
        /* We did not, clear the result and set the error code */
        BaseSetLastNTError(Status);
        Result = 0;
    }

    /* Return the result */
    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn GetEnvironmentVariableW(
    lpName: LPCWSTR,
    lpBuffer: LPWSTR,
    mut nSize: DWORD,
) -> DWORD {
    let mut VarName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut VarValue: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Status: NTSTATUS = 0;
    let mut UniSize: USHORT = 0;

    // A caller may legitimately pass lpBuffer = NULL to ask only for the required
    // size. Without this, UniSize stays nonzero, RtlInitEmptyUnicodeString builds a
    // UNICODE_STRING with a NULL Buffer and a nonzero MaximumLength, and the query
    // below writes through it. Reachable from any usermode caller, in a DLL every
    // process loads. Upstream-Commit: 242937b1229
    if lpBuffer.is_null() {
        nSize = 0;
    }

    if nSize <= (UNICODE_STRING_MAX_CHARS - 1) as DWORD {
        if nSize != 0 {
            UniSize = (nSize as USHORT)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as USHORT)
                .wrapping_sub(core::mem::size_of::<WCHAR>() as USHORT);
        } else {
            UniSize = 0;
        }
    } else {
        UniSize = (UNICODE_STRING_MAX_BYTES as USHORT)
            .wrapping_sub(core::mem::size_of::<WCHAR>() as USHORT);
    }

    Status = RtlInitUnicodeStringEx(&mut VarName, lpName);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    RtlInitEmptyUnicodeString(&mut VarValue, lpBuffer, UniSize);

    Status = RtlQueryEnvironmentVariable_U(
        core::ptr::null_mut(),
        &VarName,
        &mut VarValue,
    );
    if !NT_SUCCESS(Status) {
        if Status == STATUS_BUFFER_TOO_SMALL {
            return (VarValue.Length as usize)
                .wrapping_div(core::mem::size_of::<WCHAR>())
                .wrapping_add(core::mem::size_of::<CHAR>()) as DWORD;
        }
        BaseSetLastNTError(Status);
        return 0;
    }

    if !lpBuffer.is_null() {
        *lpBuffer.offset(
            (VarValue.Length as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as isize,
        ) = UNICODE_NULL;
    }

    return (VarValue.Length as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn SetEnvironmentVariableA(
    lpName: LPCSTR,
    lpValue: LPCSTR,
) -> BOOL {
    let mut VarName: ANSI_STRING = ANSI_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut VarValue: ANSI_STRING = ANSI_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut VarNameU: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut VarValueU: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Status: NTSTATUS = 0;

    RtlInitAnsiString(&mut VarName, lpName as LPSTR);
    Status = RtlAnsiStringToUnicodeString(&mut VarNameU, &mut VarName, 1);
    if NT_SUCCESS(Status) {
        if !lpValue.is_null() {
            RtlInitAnsiString(&mut VarValue, lpValue as LPSTR);
            Status = RtlAnsiStringToUnicodeString(&mut VarValueU, &mut VarValue, 1);
            if NT_SUCCESS(Status) {
                Status = RtlSetEnvironmentVariable(
                    core::ptr::null_mut(),
                    &mut VarNameU,
                    &mut VarValueU,
                );
                RtlFreeUnicodeString(&mut VarValueU);
            }
        } else {
            Status = RtlSetEnvironmentVariable(
                core::ptr::null_mut(),
                &mut VarNameU,
                core::ptr::null_mut(),
            );
        }

        RtlFreeUnicodeString(&mut VarNameU);

        if NT_SUCCESS(Status) {
            return 1;
        }
    }

    BaseSetLastNTError(Status);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn SetEnvironmentVariableW(
    lpName: LPCWSTR,
    lpValue: LPCWSTR,
) -> BOOL {
    let mut VarName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut VarValue: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Status: NTSTATUS = 0;

    Status = RtlInitUnicodeStringEx(&mut VarName, lpName);
    if NT_SUCCESS(Status) {
        if !lpValue.is_null() {
            Status = RtlInitUnicodeStringEx(&mut VarValue, lpValue);
            if NT_SUCCESS(Status) {
                Status = RtlSetEnvironmentVariable(
                    core::ptr::null_mut(),
                    &mut VarName,
                    &mut VarValue,
                );
            }
        } else {
            Status = RtlSetEnvironmentVariable(
                core::ptr::null_mut(),
                &mut VarName,
                core::ptr::null_mut(),
            );
        }
        if NT_SUCCESS(Status) {
            return TRUE as BOOL;
        }
    }

    BaseSetLastNTError(Status);
    return FALSE as BOOL;
}
// winbase.h: `#define GetEnvironmentStringsA GetEnvironmentStrings` — the C source's
// `GetEnvironmentStringsA` compiles to the symbol `GetEnvironmentStrings` (the canonical
// ANSI impl; the .spec forwards the GetEnvironmentStringsA export to it). Export that name.
#[no_mangle]
pub unsafe extern "system" fn GetEnvironmentStrings() -> LPSTR {
    let Length: ULONG;
    let mut Size: ULONG = 0;
    let mut Status: NTSTATUS;
    let Environment: PWCHAR;
    let mut p: PWCHAR;
    let mut Buffer: PCHAR = core::ptr::null_mut();

    RtlAcquirePebLock();
    p = (*(*NtCurrentPeb()).ProcessParameters).Environment;
    Environment = p;

    loop
    {
        p = p.add(wcslen(p) + 1);
        if *p == 0 { break; }
    }

    Length = (p as ULONG_PTR).wrapping_sub(Environment as ULONG_PTR) as ULONG / core::mem::size_of::<WCHAR>() as ULONG + 1;

    Status = RtlUnicodeToMultiByteSize(&mut Size, Environment, Length * core::mem::size_of::<WCHAR>() as ULONG);
    if NT_SUCCESS(Status)
    {
        Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, Size as SIZE_T) as PCHAR;
        if !Buffer.is_null()
        {
            Status = RtlUnicodeToOemN(Buffer, Size, core::ptr::null_mut(), Environment, Length * core::mem::size_of::<WCHAR>() as ULONG);
            if !NT_SUCCESS(Status)
            {
                RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer as PVOID);
                Buffer = core::ptr::null_mut();

                BaseSetLastNTError(Status);
            }
        }
        else
        {
            BaseSetLastNTError(STATUS_NO_MEMORY);
        }
    }
    else
    {
        BaseSetLastNTError(Status);
    }

    RtlReleasePebLock();
    return Buffer;
}

#[no_mangle]
pub unsafe extern "system" fn GetEnvironmentStringsW() -> LPWSTR {
    let mut Environment: PWCHAR = core::ptr::null_mut();
    let mut p: PWCHAR = core::ptr::null_mut();
    let mut Length: ULONG = 0;

    RtlAcquirePebLock();

    Environment = (*(*NtCurrentPeb()).ProcessParameters).Environment as PWCHAR;
    p = Environment;

    loop {
        p = p.add(wcslen(p as *const u16).wrapping_add(1) as usize);
        if *p == 0 {
            break;
        }
    }

    Length = (p.offset_from(Environment) as ULONG).wrapping_add(1);

    p = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        (Length as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as PWCHAR;

    if !p.is_null() {
        RtlCopyMemory(
            p as *mut core::ffi::c_void,
            Environment as *const core::ffi::c_void,
            (Length as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        );
    } else {
        BaseSetLastNTError(STATUS_NO_MEMORY);
    }

    RtlReleasePebLock();
    return p as LPWSTR;
}
#[no_mangle]
pub unsafe extern "system" fn FreeEnvironmentStringsA(EnvironmentStrings: LPSTR) -> BOOL {
    return RtlFreeHeap(
        RtlGetProcessHeap(),
        0,
        EnvironmentStrings as *mut core::ffi::c_void,
    ) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn FreeEnvironmentStringsW(
    EnvironmentStrings: LPWSTR,
) -> BOOL {
    return RtlFreeHeap(RtlGetProcessHeap(), 0, EnvironmentStrings as PVOID) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ExpandEnvironmentStringsA(
    lpSrc: LPCSTR,
    lpDst: LPSTR,
    nSize: DWORD,
) -> DWORD {
    let mut Source: ANSI_STRING = core::mem::zeroed();
    let mut Dest: ANSI_STRING = core::mem::zeroed();
    let mut SourceU: _UNICODE_STRING = core::mem::zeroed();
    let mut DestU: _UNICODE_STRING = core::mem::zeroed();
    let mut Buffer: PWSTR = core::ptr::null_mut();
    let mut Result: ULONG = 0;
    let mut Length: ULONG = 0;
    let mut UniSize: USHORT = 0;
    let mut Status: NTSTATUS = 0;

    /* Check if the size is too big to fit */
    UniSize = (UNICODE_STRING_MAX_CHARS - 2) as USHORT;
    if nSize <= UniSize as ULONG {
        UniSize = nSize as USHORT;
    }

    /* Clear the input buffer */
    if !lpDst.is_null() {
        *lpDst = 0i8; // ANSI_NULL
    }

    /* Initialize all the strings */
    RtlInitAnsiString(&mut Source, lpSrc);
    RtlInitUnicodeString(&mut SourceU, core::ptr::null());
    RtlInitUnicodeString(&mut DestU, core::ptr::null());
    Status = RtlAnsiStringToUnicodeString(&mut SourceU, &mut Source, 1u8); // TRUE
    if NT_SUCCESS(Status) {
        /* If the string fit in, make space for a NULL char */
        if UniSize != 0 {
            UniSize = UniSize.wrapping_sub(1);
        } else {
            /* No input size, so no string size */
            UniSize = 0;
        }

        /* Allocate the value string buffer */
        Buffer = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            (UniSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as PWSTR;
        if Buffer.is_null() {
            Status = STATUS_NO_MEMORY;
        } else {
            /* And initialize its string */
            RtlInitEmptyUnicodeString(
                &mut DestU,
                Buffer,
                (UniSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as USHORT,
            );

            /* Query the variable */
            Length = 0;
            Status = RtlExpandEnvironmentStrings_U(
                core::ptr::null_mut(),
                &mut SourceU,
                &mut DestU,
                &mut Length,
            );

            /* Check if we didn't have enough space */
            if Status == STATUS_BUFFER_TOO_SMALL {
                /* Fixup the length that the API returned */
                DestU.MaximumLength = Length as SHORT as USHORT;

                /* Free old Unicode buffer */
                RtlFreeHeap(RtlGetProcessHeap(), 0, DestU.Buffer as PVOID);

                /* Allocate new one */
                Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, Length as SIZE_T) as PWSTR;
                if !Buffer.is_null() {
                    /* Query the variable so we can know its size */
                    DestU.Buffer = Buffer;
                    Status = RtlExpandEnvironmentStrings_U(
                        core::ptr::null_mut(),
                        &mut SourceU,
                        &mut DestU,
                        &mut Length,
                    );
                    if NT_SUCCESS(Status) {
                        /* Get the ASCII length of the variable, add a byte for NULL */
                        Result = RtlUnicodeStringToAnsiSize(&DestU) + core::mem::size_of::<CHAR>() as ULONG;
                    }
                } else {
                    /* Set failure status */
                    Status = STATUS_NO_MEMORY;
                    DestU.Buffer = core::ptr::null_mut();
                }
            } else if NT_SUCCESS(Status) {
                /* Check if the size is too big to fit */
                UniSize = (UNICODE_STRING_MAX_BYTES - 1) as USHORT;
                if nSize <= UniSize as ULONG {
                    UniSize = nSize as USHORT;
                }

                /* Check the size */
                Result = RtlUnicodeStringToAnsiSize(&DestU);
                if Result <= UniSize as ULONG {
                    /* Convert the string */
                    RtlInitEmptyAnsiString(&mut Dest, lpDst, UniSize);
                    Status = RtlUnicodeStringToAnsiString(&mut Dest, &DestU, 0u8); // FALSE
                    /* Write a NULL-char in case of failure only */
                    if !NT_SUCCESS(Status) {
                        *lpDst = 0i8; // ANSI_NULL
                    }
                }
            }
        }
    }

    /* Quickie: (fall-through cleanup tail) */
    /* Free the strings */
    RtlFreeUnicodeString(&mut SourceU);
    if !DestU.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, DestU.Buffer as PVOID);
    }

    /* Check if we succeeded */
    if !NT_SUCCESS(Status) {
        /* We did not, clear the result and set the error code */
        BaseSetLastNTError(Status);
        Result = 0;
    }

    /* Return the result */
    Result
}

#[no_mangle]
pub unsafe extern "system" fn ExpandEnvironmentStringsW(
    lpSrc: LPCWSTR,
    lpDst: LPWSTR,
    mut nSize: DWORD,
) -> DWORD {
    let mut Source: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Destination: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Status: NTSTATUS = 0;
    let mut UniSize: USHORT = 0;

    UniSize = (if nSize < (UNICODE_STRING_MAX_CHARS - 2) as DWORD {
        nSize
    } else {
        (UNICODE_STRING_MAX_CHARS - 2) as DWORD
    }) as USHORT;

    RtlInitUnicodeString(&mut Source, lpSrc as LPWSTR);
    RtlInitEmptyUnicodeString(
        &mut Destination,
        lpDst,
        (UniSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as USHORT,
    );

    Status = RtlExpandEnvironmentStrings_U(
        core::ptr::null_mut(),
        &mut Source,
        &mut Destination,
        &mut nSize,
    );

    if NT_SUCCESS(Status) || Status == STATUS_BUFFER_TOO_SMALL {
        return (nSize as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as DWORD;
    }

    BaseSetLastNTError(Status);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn SetEnvironmentStringsA(
    NewEnvironment: LPCH,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"dll/win32/kernel32/client/environ.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        553 as ::core::ffi::c_int,
        b"SetEnvironmentStringsA\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"dll/win32/kernel32/client/environ.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            553 as ::core::ffi::c_int,
        );
    }
    return FALSE as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn SetEnvironmentStringsW(NewEnvironment: LPWCH) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"dll/win32/kernel32/client/environ.c\0"
            .as_ptr() as *const i8,
        564i32,
        b"SetEnvironmentStringsW\0".as_ptr() as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"dll/win32/kernel32/client/environ.c\0"
                .as_ptr() as *const i8,
            564i32,
        );
    }
    return 0i32;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/environ.c
//  * PURPOSE:         Environment functions
//  * PROGRAMMER:      Ariadne ( ariadne@xs4all.nl)
//  *                  Emanuele Aliberti
//  *                  Thomas Weidenmueller
//  * UPDATE HISTORY:
//  *                  Created 01/11/98
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS ******************************************************************/
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetEnvironmentVariableA(IN LPCSTR lpName,
//                         IN LPSTR lpBuffer,
//                         IN DWORD nSize)
// {
//     ANSI_STRING VarName, VarValue;
//     UNICODE_STRING VarNameU, VarValueU;
//     PWSTR Buffer;
//     ULONG Result = 0;
//     USHORT UniSize;
//     NTSTATUS Status;
//
//     /* Initialize all the strings */
//     RtlInitAnsiString(&VarName, lpName);
//     RtlInitUnicodeString(&VarNameU, NULL);
//     RtlInitUnicodeString(&VarValueU, NULL);
//     Status = RtlAnsiStringToUnicodeString(&VarNameU, &VarName, TRUE);
//     if (!NT_SUCCESS(Status)) goto Quickie;
//
//     /* Check if the size is too big to fit */
//     UniSize = UNICODE_STRING_MAX_CHARS - 2;
//     if (nSize <= UniSize)
//     {
//         /* It fits, but was there a string at all? */
//         if (nSize)
//         {
//             /* Keep the given size, minus a NULL-char */
//             UniSize = (USHORT)(nSize - 1);
//         }
//         else
//         {
//             /* No size */
//             UniSize = 0;
//         }
//     }
//     else
//     {
//         /* String is too big, so we need to return a NULL char as well */
//         UniSize--;
//     }
//
//     /* Allocate the value string buffer */
//     Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, UniSize * sizeof(WCHAR));
//     if (!Buffer)
//     {
//         Status = STATUS_NO_MEMORY;
//         goto Quickie;
//     }
//
//     /* And initialize its string */
//     RtlInitEmptyUnicodeString(&VarValueU, Buffer, UniSize * sizeof(WCHAR));
//
//     /* Acquire the PEB lock since we'll be querying variables now */
//     RtlAcquirePebLock();
//
//     /* Query the variable */
//     Status = RtlQueryEnvironmentVariable_U(NULL, &VarNameU, &VarValueU);
//     if ((NT_SUCCESS(Status)) && !(nSize)) Status = STATUS_BUFFER_TOO_SMALL;
//
//     /* Check if we didn't have enough space */
//     if (Status == STATUS_BUFFER_TOO_SMALL)
//     {
//         /* Fixup the length that the API returned */
//         VarValueU.MaximumLength = VarValueU.Length + sizeof(UNICODE_NULL);
//
//         /* Free old Unicode buffer */
//         RtlFreeHeap(RtlGetProcessHeap(), 0, VarValueU.Buffer);
//
//         /* Allocate new one */
//         Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, VarValueU.MaximumLength);
//         if (Buffer)
//         {
//             /* Query the variable so we can know its size */
//             VarValueU.Buffer = Buffer;
//             Status = RtlQueryEnvironmentVariable_U(NULL, &VarNameU, &VarValueU);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Get the ASCII length of the variable */
//                 Result = RtlUnicodeStringToAnsiSize(&VarValueU);
//             }
//         }
//         else
//         {
//             /* Set failure status */
//             Status = STATUS_NO_MEMORY;
//             VarValueU.Buffer = NULL;
//         }
//     }
//     else if (NT_SUCCESS(Status))
//     {
//         /* Check if the size is too big to fit */
//         UniSize = UNICODE_STRING_MAX_BYTES - 1;
//         if (nSize <= UniSize) UniSize = (USHORT)nSize;
//
//         /* Check the size */
//         Result = RtlUnicodeStringToAnsiSize(&VarValueU);
//         if (Result <= UniSize)
//         {
//             /* Convert the string */
//             RtlInitEmptyAnsiString(&VarValue, lpBuffer, UniSize);
//             Status = RtlUnicodeStringToAnsiString(&VarValue, &VarValueU, FALSE);
//             if (NT_SUCCESS(Status))
//             {
//                 /* NULL-terminate and set the final length */
//                 lpBuffer[VarValue.Length] = ANSI_NULL;
//                 Result = VarValue.Length;
//             }
//         }
//     }
//
//     /* Release the lock */
//     RtlReleasePebLock();
//
// Quickie:
//     /* Free the strings */
//     RtlFreeUnicodeString(&VarNameU);
//     if (VarValueU.Buffer) RtlFreeHeap(RtlGetProcessHeap(), 0, VarValueU.Buffer);
//
//     /* Check if we succeeded */
//     if (!NT_SUCCESS(Status))
//     {
//         /* We did not, clear the result and set the error code */
//         BaseSetLastNTError(Status);
//         Result = 0;
//     }
//
//     /* Return the result */
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetEnvironmentVariableW(IN LPCWSTR lpName,
//                         IN LPWSTR lpBuffer,
//                         IN DWORD nSize)
// {
//     UNICODE_STRING VarName, VarValue;
//     NTSTATUS Status;
//     USHORT UniSize;
//
//     if (nSize <= (UNICODE_STRING_MAX_CHARS - 1))
//     {
//         if (nSize)
//         {
//             UniSize = (USHORT)nSize * sizeof(WCHAR) - sizeof(UNICODE_NULL);
//         }
//         else
//         {
//             UniSize = 0;
//         }
//     }
//     else
//     {
//         UniSize = UNICODE_STRING_MAX_BYTES - sizeof(UNICODE_NULL);
//     }
//
//     Status = RtlInitUnicodeStringEx(&VarName, lpName);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return 0;
//     }
//
//     RtlInitEmptyUnicodeString(&VarValue, lpBuffer, UniSize);
//
//     Status = RtlQueryEnvironmentVariable_U(NULL, &VarName, &VarValue);
//     if (!NT_SUCCESS(Status))
//     {
//         if (Status == STATUS_BUFFER_TOO_SMALL)
//         {
//             return (VarValue.Length / sizeof(WCHAR)) + sizeof(ANSI_NULL);
//         }
//         BaseSetLastNTError (Status);
//         return 0;
//     }
//
//     if (lpBuffer)
//         lpBuffer[VarValue.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     return (VarValue.Length / sizeof(WCHAR));
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetEnvironmentVariableA(IN LPCSTR lpName,
//                         IN LPCSTR lpValue)
// {
//     ANSI_STRING VarName, VarValue;
//     UNICODE_STRING VarNameU, VarValueU;
//     NTSTATUS Status;
//
//     RtlInitAnsiString(&VarName, (LPSTR)lpName);
//     Status = RtlAnsiStringToUnicodeString(&VarNameU, &VarName, TRUE);
//     if (NT_SUCCESS(Status))
//     {
//         if (lpValue)
//         {
//             RtlInitAnsiString(&VarValue, (LPSTR)lpValue);
//             Status = RtlAnsiStringToUnicodeString(&VarValueU, &VarValue, TRUE);
//             if (NT_SUCCESS(Status))
//             {
//                 Status = RtlSetEnvironmentVariable(NULL, &VarNameU, &VarValueU);
//                 RtlFreeUnicodeString(&VarValueU);
//             }
//         }
//         else
//         {
//             Status = RtlSetEnvironmentVariable(NULL, &VarNameU, NULL);
//         }
//
//         RtlFreeUnicodeString(&VarNameU);
//
//         if (NT_SUCCESS(Status)) return TRUE;
//     }
//
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetEnvironmentVariableW(IN LPCWSTR lpName,
//                         IN LPCWSTR lpValue)
// {
//     UNICODE_STRING VarName, VarValue;
//     NTSTATUS Status;
//
//     Status = RtlInitUnicodeStringEx(&VarName, lpName);
//     if (NT_SUCCESS(Status))
//     {
//         if (lpValue)
//         {
//             Status = RtlInitUnicodeStringEx(&VarValue, lpValue);
//             if (NT_SUCCESS(Status))
//             {
//                 Status = RtlSetEnvironmentVariable(NULL, &VarName, &VarValue);
//             }
//         }
//         else
//         {
//             Status = RtlSetEnvironmentVariable(NULL, &VarName, NULL);
//         }
//
//         if (NT_SUCCESS(Status)) return TRUE;
//     }
//
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// LPSTR
// WINAPI
// GetEnvironmentStringsA(VOID)
// {
//     ULONG Length, Size;
//     NTSTATUS Status;
//     PWCHAR Environment, p;
//     PCHAR Buffer = NULL;
//
//     RtlAcquirePebLock();
//     p = Environment = NtCurrentPeb()->ProcessParameters->Environment;
//
//     do
//     {
//         p += wcslen(p) + 1;
//     } while (*p);
//
//     Length = p - Environment + 1;
//
//     Status = RtlUnicodeToMultiByteSize(&Size, Environment, Length * sizeof(WCHAR));
//     if (NT_SUCCESS(Status))
//     {
//         Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, Size);
//         if (Buffer)
//         {
//             Status = RtlUnicodeToOemN(Buffer, Size, 0, Environment, Length * sizeof(WCHAR));
//             if (!NT_SUCCESS(Status))
//             {
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
//                 Buffer = NULL;
//
//                 BaseSetLastNTError(Status);
//             }
//         }
//         else
//         {
//             BaseSetLastNTError(STATUS_NO_MEMORY);
//         }
//     }
//     else
//     {
//         BaseSetLastNTError(Status);
//     }
//
//     RtlReleasePebLock();
//     return Buffer;
// }
//
// /*
//  * @implemented
//  */
// LPWSTR
// WINAPI
// GetEnvironmentStringsW(VOID)
// {
//     PWCHAR Environment, p;
//     ULONG Length;
//
//     RtlAcquirePebLock();
//
//     p = Environment = NtCurrentPeb()->ProcessParameters->Environment;
//
//     do
//     {
//         p += wcslen(p) + 1;
//     } while (*p);
//
//     Length = p - Environment + 1;
//
//     p = RtlAllocateHeap(RtlGetProcessHeap(), 0, Length * sizeof(WCHAR));
//     if (p)
//     {
//         RtlCopyMemory(p, Environment, Length * sizeof(WCHAR));
//     }
//     else
//     {
//         BaseSetLastNTError(STATUS_NO_MEMORY);
//     }
//
//     RtlReleasePebLock();
//     return p;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// FreeEnvironmentStringsA(IN LPSTR EnvironmentStrings)
// {
//     return RtlFreeHeap(RtlGetProcessHeap(), 0, EnvironmentStrings);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// FreeEnvironmentStringsW(IN LPWSTR EnvironmentStrings)
// {
//     return RtlFreeHeap(RtlGetProcessHeap(), 0, EnvironmentStrings);
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// ExpandEnvironmentStringsA(IN LPCSTR lpSrc,
//                           IN LPSTR lpDst,
//                           IN DWORD nSize)
// {
//     ANSI_STRING Source, Dest;
//     UNICODE_STRING SourceU, DestU;
//     PWSTR Buffer;
//     ULONG Result = 0, Length;
//     USHORT UniSize;
//     NTSTATUS Status;
//
//     /* Check if the size is too big to fit */
//     UniSize = UNICODE_STRING_MAX_CHARS - 2;
//     if (nSize <= UniSize) UniSize = (USHORT)nSize;
//
//     /* Clear the input buffer */
//     if (lpDst) *lpDst = ANSI_NULL;
//
//     /* Initialize all the strings */
//     RtlInitAnsiString(&Source, lpSrc);
//     RtlInitUnicodeString(&SourceU, NULL);
//     RtlInitUnicodeString(&DestU, NULL);
//     Status = RtlAnsiStringToUnicodeString(&SourceU, &Source, TRUE);
//     if (!NT_SUCCESS(Status)) goto Quickie;
//
//     /* If the string fit in, make space for a NULL char */
//     if (UniSize)
//     {
//         UniSize--;
//     }
//     else
//     {
//         /* No input size, so no string size */
//         UniSize = 0;
//     }
//
//     /* Allocate the value string buffer */
//     Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, UniSize * sizeof(WCHAR));
//     if (!Buffer)
//     {
//         Status = STATUS_NO_MEMORY;
//         goto Quickie;
//     }
//
//     /* And initialize its string */
//     RtlInitEmptyUnicodeString(&DestU, Buffer, UniSize * sizeof(WCHAR));
//
//     /* Query the variable */
//     Length = 0;
//     Status = RtlExpandEnvironmentStrings_U(NULL, &SourceU, &DestU, &Length);
//
//     /* Check if we didn't have enough space */
//     if (Status == STATUS_BUFFER_TOO_SMALL)
//     {
//         /* Fixup the length that the API returned */
//         DestU.MaximumLength = (SHORT)Length;
//
//         /* Free old Unicode buffer */
//         RtlFreeHeap(RtlGetProcessHeap(), 0, DestU.Buffer);
//
//         /* Allocate new one */
//         Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, Length);
//         if (Buffer)
//         {
//             /* Query the variable so we can know its size */
//             DestU.Buffer = Buffer;
//             Status = RtlExpandEnvironmentStrings_U(NULL, &SourceU, &DestU, &Length);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Get the ASCII length of the variable, add a byte for NULL */
//                 Result = RtlUnicodeStringToAnsiSize(&DestU) + sizeof(ANSI_NULL);
//             }
//         }
//         else
//         {
//             /* Set failure status */
//             Status = STATUS_NO_MEMORY;
//             DestU.Buffer = NULL;
//         }
//     }
//     else if (NT_SUCCESS(Status))
//     {
//         /* Check if the size is too big to fit */
//         UniSize = UNICODE_STRING_MAX_BYTES - 1;
//         if (nSize <= UniSize) UniSize = (USHORT)nSize;
//
//         /* Check the size */
//         Result = RtlUnicodeStringToAnsiSize(&DestU);
//         if (Result <= UniSize)
//         {
//             /* Convert the string */
//             RtlInitEmptyAnsiString(&Dest, lpDst, UniSize);
//             Status = RtlUnicodeStringToAnsiString(&Dest, &DestU, FALSE);
//
//             /* Write a NULL-char in case of failure only */
//             if (!NT_SUCCESS(Status)) *lpDst = ANSI_NULL;
//         }
//     }
// Quickie:
//     /* Free the strings */
//     RtlFreeUnicodeString(&SourceU);
//     if (DestU.Buffer) RtlFreeHeap(RtlGetProcessHeap(), 0, DestU.Buffer);
//
//     /* Check if we succeeded */
//     if (!NT_SUCCESS(Status))
//     {
//         /* We did not, clear the result and set the error code */
//         BaseSetLastNTError(Status);
//         Result = 0;
//     }
//
//     /* Return the result */
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// ExpandEnvironmentStringsW(IN LPCWSTR lpSrc,
//                           IN LPWSTR lpDst,
//                           IN DWORD nSize)
// {
//     UNICODE_STRING Source, Destination;
//     NTSTATUS Status;
//     USHORT UniSize;
//
//     UniSize = min(nSize, UNICODE_STRING_MAX_CHARS - 2);
//
//     RtlInitUnicodeString(&Source, (LPWSTR)lpSrc);
//     RtlInitEmptyUnicodeString(&Destination, lpDst, UniSize * sizeof(WCHAR));
//
//     Status = RtlExpandEnvironmentStrings_U(NULL,
//                                            &Source,
//                                            &Destination,
//                                            &nSize);
//     if ((NT_SUCCESS(Status)) || (Status == STATUS_BUFFER_TOO_SMALL))
//     {
//         return nSize / sizeof(WCHAR);
//     }
//
//     BaseSetLastNTError(Status);
//     return 0;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetEnvironmentStringsA(IN LPCH NewEnvironment)
// {
//     STUB;
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetEnvironmentStringsW(IN LPWCH NewEnvironment)
// {
//     STUB;
//     return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
