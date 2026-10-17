#![cfg_attr(not(debug_assertions), allow(unused_imports))]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use ntdll_sys::nt::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::k32_externs::BaseSetLastNTError;
use core::ffi::c_void;

// ---- cohab import surface (atom.c calls these from other TUs) ----
extern "system" {
    // kernel32 Base* helper (kernel32.h)
}

// RtlGetProcessHeap() is a macro `(NtCurrentPeb()->ProcessHeap)`, not an export.
#[inline]
unsafe fn RtlGetProcessHeap() -> PVOID {
    (*NtCurrentPeb()).ProcessHeap
}

// NT_SUCCESS(Status) macro: Status >= 0 (i.e. high bit clear).
#[inline]
fn NT_SUCCESS(Status: NTSTATUS) -> bool {
    Status >= 0
}

// PtrToShort(p) = (short)(LONG_PTR)(p)  (basetsd.h)
#[inline]
fn PtrToShort(p: PVOID) -> SHORT {
    (p as LONG_PTR) as SHORT
}

// ---- constants (winbase.h / winnt.h) ----
const INVALID_ATOM: ATOM = 0; // (ATOM)0
const MAXINTATOM: ATOM = 0xC000;
const UNICODE_NULL: WCHAR = 0;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut BaseLocalAtomTable: PRTL_ATOM_TABLE =
    ::core::ptr::null::<_RTL_ATOM_TABLE>() as *mut _RTL_ATOM_TABLE;

#[no_mangle]
pub unsafe extern "system" fn InternalInitAtomTable() -> *mut core::ffi::c_void {
    if BaseLocalAtomTable.is_null() {
        RtlCreateAtomTable(0, &raw mut BaseLocalAtomTable);
    }
    return BaseLocalAtomTable as *mut core::ffi::c_void;
}

#[no_mangle]
pub unsafe extern "system" fn InternalAddAtom(
    Local: BOOLEAN,
    Unicode: BOOLEAN,
    AtomName: LPCSTR,
) -> ATOM {
    let mut Status: NTSTATUS;
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let mut AtomNameString: PUNICODE_STRING = core::ptr::null_mut();
    let mut Atom: ATOM = INVALID_ATOM;

    if (AtomName as ULONG_PTR) <= 0xFFFF {
        Atom = PtrToShort(AtomName as PVOID) as ATOM;
        if Atom >= MAXINTATOM {
            BaseSetLastNTError(STATUS_INVALID_PARAMETER);
            return INVALID_ATOM;
        }
        return Atom;
    } else {
        if Unicode != 0 {
            AtomNameString = &mut UnicodeString;
            RtlInitUnicodeString(AtomNameString, AtomName as LPWSTR);
            Status = STATUS_SUCCESS;
        } else {
            RtlInitAnsiString(&mut AnsiString, AtomName);
            if AnsiString.MaximumLength > 260 {
                AtomNameString = &mut UnicodeString;
                Status = RtlAnsiStringToUnicodeString(AtomNameString, &mut AnsiString, 1);
            } else {
                AtomNameString = &mut (*NtCurrentTeb()).StaticUnicodeString;
                Status = RtlAnsiStringToUnicodeString(AtomNameString, &mut AnsiString, 0);
            }
        }

        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            return Atom;
        }
    }

    if Local != 0 {
        Status = RtlAddAtomToAtomTable(
            InternalInitAtomTable() as PRTL_ATOM_TABLE,
            (*AtomNameString).Buffer,
            &mut Atom,
        );
    } else {
        Status = NtAddAtom(
            (*AtomNameString).Buffer,
            (*AtomNameString).Length as ULONG,
            &mut Atom,
        );
    }

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
    }

    if Unicode == 0 && AtomNameString == &mut UnicodeString {
        RtlFreeUnicodeString(AtomNameString);
    }

    return Atom;
}

#[no_mangle]
pub unsafe extern "system" fn InternalFindAtom(
    Local: BOOLEAN,
    Unicode: BOOLEAN,
    AtomName: LPCSTR,
) -> ATOM {
    let mut Status: NTSTATUS;
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let mut AtomNameString: PUNICODE_STRING = core::ptr::null_mut();
    let mut Atom: ATOM = INVALID_ATOM;

    /* Check if it's an integer atom */
    if (AtomName as ULONG_PTR) <= 0xFFFF {
        /* Convert the name to an atom */
        Atom = PtrToShort(AtomName as PVOID) as ATOM;
        if Atom >= MAXINTATOM {
            /* Fail, atom number too large */
            BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        }

        /* Return it */
        return Atom;
    } else {
        /* Check if this is a unicode atom */
        if Unicode != 0 {
            /* Use a unicode string */
            AtomNameString = &mut UnicodeString;
            RtlInitUnicodeString(AtomNameString, AtomName as LPWSTR);
            Status = STATUS_SUCCESS;
        } else {
            /* Use an ansi string */
            RtlInitAnsiString(&mut AnsiString, AtomName);

            /* Check if we can abuse the TEB */
            if AnsiString.MaximumLength > 260 {
                /* We can't, allocate a new string */
                AtomNameString = &mut UnicodeString;
                Status = RtlAnsiStringToUnicodeString(
                    AtomNameString,
                    &mut AnsiString,
                    1,
                );
            } else {
                /* We can! Get the TEB String */
                AtomNameString = &mut (*NtCurrentTeb()).StaticUnicodeString;

                /* Convert it into the TEB */
                Status = RtlAnsiStringToUnicodeString(
                    AtomNameString,
                    &mut AnsiString,
                    0,
                );
            }
        }

        /* Check for failure */
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            return Atom;
        }
    }

    /* Check if we're doing local lookup */
    if Local != 0 {
        /* Do a local lookup */
        Status = RtlLookupAtomInAtomTable(
            InternalInitAtomTable() as PRTL_ATOM_TABLE,
            (*AtomNameString).Buffer,
            &mut Atom,
        );
    } else {
        /* Do a global search */
        if (*AtomNameString).Length == 0 {
            /* This is illegal in win32 */
            Status = STATUS_OBJECT_NAME_NOT_FOUND;
        } else {
            /* Call the global function */
            Status = NtFindAtom(
                (*AtomNameString).Buffer,
                (*AtomNameString).Length as ULONG,
                &mut Atom,
            );
        }
    }

    /* Check for failure */
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
    }

    /* Check if we were non-static ANSI */
    if Unicode == 0 && AtomNameString == &mut UnicodeString {
        /* Free the allocated buffer */
        RtlFreeUnicodeString(AtomNameString);
    }

    /* Return the atom */
    return Atom;
}

#[no_mangle]
pub unsafe extern "system" fn InternalDeleteAtom(
    Local: BOOLEAN,
    Atom: ATOM,
) -> ATOM {
    let mut Status: NTSTATUS = 0;

    /* Validate it */
    if Atom >= MAXINTATOM as ATOM {
        /* Check if it's a local delete */
        if Local != 0 {
            /* Delete it locally */
            Status = RtlDeleteAtomFromAtomTable(
                InternalInitAtomTable() as PRTL_ATOM_TABLE,
                Atom,
            );
        } else {
            /* Delete it globall */
            Status = NtDeleteAtom(Atom);
        }

        /* Check for success */
        if !NT_SUCCESS(Status) {
            /* Fail */
            BaseSetLastNTError(Status);
            return INVALID_ATOM;
        }
    }

    /* Return failure */
    return 0 as ATOM;
}

#[no_mangle]
pub unsafe extern "system" fn InternalGetAtomName(
    Local: BOOLEAN,
    Unicode: BOOLEAN,
    Atom: ATOM,
    AtomName: LPSTR,
    mut Size: DWORD,
) -> UINT {
    let mut Status: NTSTATUS = 0;
    let mut RetVal: DWORD = 0;
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let mut TempBuffer: PVOID = core::ptr::null_mut();
    let mut AtomNameString: PWSTR = core::ptr::null_mut();
    let mut AtomInfoLength: ULONG = 0;
    let mut AtomNameLength: ULONG = 0;
    let mut AtomInfo: PATOM_BASIC_INFORMATION = core::ptr::null_mut();

    if Unicode == 0 && Size > 0x7000 {
        Size = 0x7000;
    }

    if Size == 0 {
        BaseSetLastNTError(STATUS_BUFFER_OVERFLOW);
        return 0;
    }

    if Atom == 0 {
        BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        return 0;
    }

    if Local != 0 {
        AtomNameLength = (Size as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG;

        if Unicode != 0 {
            AtomNameString = AtomName as PWSTR;
        } else {
            TempBuffer = RtlAllocateHeap(
                RtlGetProcessHeap(),
                0,
                AtomNameLength as SIZE_T,
            );
            AtomNameString = TempBuffer as PWSTR;
        }

        Status = RtlQueryAtomInAtomTable(
            InternalInitAtomTable() as PRTL_ATOM_TABLE,
            Atom,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            AtomNameString,
            &mut AtomNameLength,
        );
    } else {
        AtomInfoLength = (core::mem::size_of::<ATOM_BASIC_INFORMATION>() as usize)
            .wrapping_add((Size as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize))
            as ULONG;

        TempBuffer = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            AtomInfoLength as SIZE_T,
        );
        AtomInfo = TempBuffer as PATOM_BASIC_INFORMATION;

        if AtomInfo.is_null() {
            BaseSetLastNTError(STATUS_NO_MEMORY);
            return 0;
        }

        Status = NtQueryInformationAtom(
            Atom,
            AtomBasicInformation,
            AtomInfo as PVOID,
            AtomInfoLength,
            &mut AtomInfoLength,
        );

        if NT_SUCCESS(Status) {
            AtomNameLength = (*AtomInfo).NameLength as ULONG;
            AtomNameString = &raw mut (*AtomInfo).Name as PWSTR;
        }
    }

    if NT_SUCCESS(Status) {
        if Unicode != 0 {
            RetVal = (AtomNameLength as usize)
                .wrapping_div(core::mem::size_of::<WCHAR>() as usize) as DWORD;

            if AtomNameString != AtomName as PWSTR {
                core::ptr::copy_nonoverlapping(
                    AtomNameString as *const core::ffi::c_void,
                    AtomName as *mut core::ffi::c_void,
                    AtomNameLength as usize,
                );
            }

            if RetVal < Size {
                *(AtomName as PWCHAR).offset(RetVal as isize) = UNICODE_NULL;
            }
        } else {
            UnicodeString.Buffer = AtomNameString;
            UnicodeString.Length = AtomNameLength as USHORT;
            UnicodeString.MaximumLength = (UnicodeString.Length as usize)
                .wrapping_add(core::mem::size_of::<WCHAR>() as usize) as USHORT;

            AnsiString.Buffer = AtomName;
            AnsiString.Length = 0;
            AnsiString.MaximumLength = Size as USHORT;

            Status = RtlUnicodeStringToAnsiString(
                &mut AnsiString,
                &UnicodeString,
                0, // FALSE
            );

            if NT_SUCCESS(Status) {
                RetVal = AnsiString.Length as DWORD;
            }
        }
    }

    if !TempBuffer.is_null() {
        RtlFreeHeap(
            RtlGetProcessHeap(),
            0,
            TempBuffer,
        );
    }

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
    }

    return RetVal;
}

#[no_mangle]
pub unsafe extern "system" fn GlobalAddAtomA(
    lpString: LPCSTR,
) -> ATOM {
    return InternalAddAtom(0, 0, lpString);
}

#[no_mangle]
pub unsafe extern "system" fn GlobalAddAtomW(
    lpString: LPCWSTR,
) -> ATOM {
    InternalAddAtom(
        0u8, // FALSE as BOOLEAN
        1u8, // TRUE as BOOLEAN
        lpString as LPSTR as LPCSTR,
    )
}

#[no_mangle]
pub unsafe extern "system" fn GlobalDeleteAtom(
    nAtom: ATOM,
) -> ATOM {
    return InternalDeleteAtom(0u8, nAtom);
}

#[no_mangle]
pub unsafe extern "system" fn GlobalFindAtomA(
    lpString: LPCSTR,
) -> ATOM {
    return InternalFindAtom(0, 0, lpString);
}

#[no_mangle]
pub unsafe extern "system" fn GlobalFindAtomW(lpString: LPCWSTR) -> ATOM {
    InternalFindAtom(0u8, 1u8, lpString as LPSTR as LPCSTR)
}

#[no_mangle]
pub unsafe extern "system" fn GlobalGetAtomNameA(
    nAtom: ATOM,
    lpBuffer: LPSTR,
    nSize: i32,
) -> UINT {
    InternalGetAtomName(
        0u8,
        0u8,
        nAtom,
        lpBuffer,
        nSize as DWORD,
    )
}

#[no_mangle]
pub unsafe extern "system" fn GlobalGetAtomNameW(
    nAtom: ATOM,
    lpBuffer: LPWSTR,
    nSize: ::core::ffi::c_int,
) -> UINT {
    return InternalGetAtomName(
        0u8,
        1u8,
        nAtom,
        lpBuffer as LPSTR,
        nSize as DWORD,
    );
}

#[no_mangle]
pub unsafe extern "system" fn InitAtomTable(mut nSize: DWORD) -> BOOL {
    if nSize < 4 || nSize > 511 {
        nSize = 37;
    }
    return NT_SUCCESS(RtlCreateAtomTable(nSize, &raw mut BaseLocalAtomTable)) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn AddAtomA(
    lpString: LPCSTR,
) -> ATOM {
    return InternalAddAtom(1i8 as BOOLEAN, 0i8 as BOOLEAN, lpString);
}

#[no_mangle]
pub unsafe extern "system" fn AddAtomW(lpString: LPCWSTR) -> ATOM {
    return InternalAddAtom(1u8, 1u8, lpString as LPSTR as LPCSTR);
}

#[no_mangle]
pub unsafe extern "system" fn DeleteAtom(
    nAtom: ATOM,
) -> ATOM {
    return InternalDeleteAtom(1i8 as BOOLEAN, nAtom);
}

#[no_mangle]
pub unsafe extern "system" fn FindAtomA(
    lpString: LPCSTR,
) -> ATOM {
    return InternalFindAtom(1i8 as BOOLEAN, 0i8 as BOOLEAN, lpString);
}

#[no_mangle]
pub unsafe extern "system" fn FindAtomW(
    lpString: LPCWSTR,
) -> ATOM {
    return InternalFindAtom(
        1u8,
        1u8,
        lpString as LPSTR as LPCSTR,
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetAtomNameA(
    nAtom: ATOM,
    lpBuffer: LPSTR,
    nSize: i32,
) -> UINT {
    return InternalGetAtomName(
        1 as BOOLEAN,
        0 as BOOLEAN,
        nAtom,
        lpBuffer,
        nSize as DWORD,
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetAtomNameW(
    nAtom: ATOM,
    lpBuffer: LPWSTR,
    nSize: i32,
) -> UINT {
    return InternalGetAtomName(
        1u8,
        1u8,
        nAtom,
        lpBuffer as LPSTR,
        nSize as DWORD,
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/atom.c
//  * PURPOSE:         Atom functions
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES ******************************************************************/
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// PRTL_ATOM_TABLE BaseLocalAtomTable = NULL;
//
// /* FUNCTIONS *****************************************************************/
//
// PVOID
// WINAPI
// InternalInitAtomTable(VOID)
// {
//     /* Create or return the local table */
//     if (!BaseLocalAtomTable) RtlCreateAtomTable(0, &BaseLocalAtomTable);
//     return BaseLocalAtomTable;
// }
//
// ATOM
// WINAPI
// InternalAddAtom(BOOLEAN Local,
//                 BOOLEAN Unicode,
//                 LPCSTR AtomName)
// {
//     NTSTATUS Status;
//     ANSI_STRING AnsiString;
//     UNICODE_STRING UnicodeString;
//     PUNICODE_STRING AtomNameString;
//     ATOM Atom = INVALID_ATOM;
//
//     /* Check if it's an integer atom */
//     if ((ULONG_PTR)AtomName <= 0xFFFF)
//     {
//         /* Convert the name to an atom */
//         Atom = (ATOM)PtrToShort((PVOID)AtomName);
//         if (Atom >= MAXINTATOM)
//         {
//             /* Fail, atom number too large */
//             BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//             return INVALID_ATOM;
//         }
//
//         /* Return it */
//         return Atom;
//     }
//     else
//     {
//         /* Check if this is a unicode atom */
//         if (Unicode)
//         {
//             /* Use a unicode string */
//             AtomNameString = &UnicodeString;
//             RtlInitUnicodeString(AtomNameString, (LPWSTR)AtomName);
//             Status = STATUS_SUCCESS;
//         }
//         else
//         {
//             /* Use an ansi string */
//             RtlInitAnsiString(&AnsiString, AtomName );
//
//             /* Check if we can abuse the TEB */
//             if (AnsiString.MaximumLength > 260)
//             {
//                 /* We can't, allocate a new string */
//                 AtomNameString = &UnicodeString;
//                 Status = RtlAnsiStringToUnicodeString(AtomNameString,
//                                                       &AnsiString,
//                                                       TRUE);
//             }
//             else
//             {
//                 /* We can! Get the TEB String */
//                 AtomNameString = &NtCurrentTeb()->StaticUnicodeString;
//
//                 /* Convert it into the TEB */
//                 Status = RtlAnsiStringToUnicodeString(AtomNameString,
//                                                       &AnsiString,
//                                                       FALSE);
//             }
//         }
//
//         /* Check for failure */
//         if (!NT_SUCCESS(Status))
//         {
//             BaseSetLastNTError(Status);
//             return Atom;
//         }
//     }
//
//     /* Check if we're doing local add */
//     if (Local)
//     {
//         /* Do a local add */
//         Status = RtlAddAtomToAtomTable(InternalInitAtomTable(),
//                                        AtomNameString->Buffer,
//                                        &Atom);
//     }
//     else
//     {
//         /* Do a global add */
//         Status = NtAddAtom(AtomNameString->Buffer,
//                            AtomNameString->Length,
//                            &Atom);
//     }
//
//     /* Check for failure */
//     if (!NT_SUCCESS(Status)) BaseSetLastNTError(Status);
//
//     /* Check if we were non-static ANSI */
//     if (!(Unicode) && (AtomNameString == &UnicodeString))
//     {
//         /* Free the allocated buffer */
//         RtlFreeUnicodeString(AtomNameString);
//     }
//
//     /* Return the atom */
//     return Atom;
// }
//
// ATOM
// WINAPI
// InternalFindAtom(BOOLEAN Local,
//                  BOOLEAN Unicode,
//                  LPCSTR AtomName)
// {
//     NTSTATUS Status;
//     ANSI_STRING AnsiString;
//     UNICODE_STRING UnicodeString;
//     PUNICODE_STRING AtomNameString;
//     ATOM Atom = INVALID_ATOM;
//
//     /* Check if it's an integer atom */
//     if ((ULONG_PTR)AtomName <= 0xFFFF)
//     {
//         /* Convert the name to an atom */
//         Atom = (ATOM)PtrToShort((PVOID)AtomName);
//         if (Atom >= MAXINTATOM)
//         {
//             /* Fail, atom number too large */
//             BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//             DPRINT1("Invalid atom\n");
//         }
//
//         /* Return it */
//         return Atom;
//     }
//     else
//     {
//         /* Check if this is a unicode atom */
//         if (Unicode)
//         {
//             /* Use a unicode string */
//             AtomNameString = &UnicodeString;
//             RtlInitUnicodeString(AtomNameString, (LPWSTR)AtomName);
//             Status = STATUS_SUCCESS;
//         }
//         else
//         {
//             /* Use an ansi string */
//             RtlInitAnsiString(&AnsiString, AtomName);
//
//             /* Check if we can abuse the TEB */
//             if (AnsiString.MaximumLength > 260)
//             {
//                 /* We can't, allocate a new string */
//                 AtomNameString = &UnicodeString;
//                 Status = RtlAnsiStringToUnicodeString(AtomNameString,
//                                                       &AnsiString,
//                                                       TRUE);
//             }
//             else
//             {
//                 /* We can! Get the TEB String */
//                 AtomNameString = &NtCurrentTeb()->StaticUnicodeString;
//
//                 /* Convert it into the TEB */
//                 Status = RtlAnsiStringToUnicodeString(AtomNameString,
//                                                       &AnsiString,
//                                                       FALSE);
//             }
//         }
//
//         /* Check for failure */
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed\n");
//             BaseSetLastNTError(Status);
//             return Atom;
//         }
//     }
//
//     /* Check if we're doing local lookup */
//     if (Local)
//     {
//         /* Do a local lookup */
//         Status = RtlLookupAtomInAtomTable(InternalInitAtomTable(),
//                                           AtomNameString->Buffer,
//                                           &Atom);
//     }
//     else
//     {
//         /* Do a global search */
//         if (!AtomNameString->Length)
//         {
//             /* This is illegal in win32 */
//             DPRINT1("No name given\n");
//             Status = STATUS_OBJECT_NAME_NOT_FOUND;
//         }
//         else
//         {
//             /* Call the global function */
//             Status = NtFindAtom(AtomNameString->Buffer,
//                                 AtomNameString->Length,
//                                 &Atom);
//         }
//     }
//
//     /* Check for failure */
//     if (!NT_SUCCESS(Status)) BaseSetLastNTError(Status);
//
//     /* Check if we were non-static ANSI */
//     if (!(Unicode) && (AtomNameString == &UnicodeString))
//     {
//         /* Free the allocated buffer */
//         RtlFreeUnicodeString(AtomNameString);
//     }
//
//     /* Return the atom */
//     return Atom;
// }
//
// ATOM
// WINAPI
// InternalDeleteAtom(BOOLEAN Local,
//                    ATOM Atom)
// {
//     NTSTATUS Status;
//
//     /* Validate it */
//     if (Atom >= MAXINTATOM)
//     {
//         /* Check if it's a local delete */
//         if (Local)
//         {
//             /* Delete it locally */
//             Status = RtlDeleteAtomFromAtomTable(InternalInitAtomTable(), Atom);
//         }
//         else
//         {
//             /* Delete it globall */
//             Status = NtDeleteAtom(Atom);
//         }
//
//         /* Check for success */
//         if (!NT_SUCCESS(Status))
//         {
//             /* Fail */
//             BaseSetLastNTError(Status);
//             return INVALID_ATOM;
//         }
//     }
//
//     /* Return failure */
//     return 0;
// }
//
// UINT
// WINAPI
// InternalGetAtomName(BOOLEAN Local,
//                     BOOLEAN Unicode,
//                     ATOM Atom,
//                     LPSTR AtomName,
//                     DWORD Size)
// {
//     NTSTATUS Status;
//     DWORD RetVal = 0;
//     ANSI_STRING AnsiString;
//     UNICODE_STRING UnicodeString;
//     PVOID TempBuffer = NULL;
//     PWSTR AtomNameString;
//     ULONG AtomInfoLength;
//     ULONG AtomNameLength;
//     PATOM_BASIC_INFORMATION AtomInfo;
//
//     /* Normalize the size as not to overflow */
//     if (!Unicode && Size > 0x7000) Size = 0x7000;
//
//     /* Make sure it's valid too */
//     if (!Size)
//     {
//         BaseSetLastNTError(STATUS_BUFFER_OVERFLOW);
//         return 0;
//     }
//     if (!Atom)
//     {
//         BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//         return 0;
//     }
//
//     /* Check if this is a global query */
//     if (Local)
//     {
//         /* Set the query length */
//         AtomNameLength = Size * sizeof(WCHAR);
//
//         /* If it's unicode, just keep the name */
//         if (Unicode)
//         {
//             AtomNameString = (PWSTR)AtomName;
//         }
//         else
//         {
//             /* Allocate memory for the ansi buffer */
//             TempBuffer = RtlAllocateHeap(RtlGetProcessHeap(),
//                                          0,
//                                          AtomNameLength);
//             AtomNameString = TempBuffer;
//         }
//
//         /* Query the name */
//         Status = RtlQueryAtomInAtomTable(InternalInitAtomTable(),
//                                          Atom,
//                                          NULL,
//                                          NULL,
//                                          AtomNameString,
//                                          &AtomNameLength);
//     }
//     else
//     {
//         /* We're going to do a global query, so allocate a buffer */
//         AtomInfoLength = sizeof(ATOM_BASIC_INFORMATION) +
//                          (Size * sizeof(WCHAR));
//         AtomInfo = TempBuffer = RtlAllocateHeap(RtlGetProcessHeap(),
//                                                 0,
//                                                 AtomInfoLength);
//
//         if (!AtomInfo)
//         {
//             BaseSetLastNTError(STATUS_NO_MEMORY);
//             return 0;
//         }
//
//         /* Query the name */
//         Status = NtQueryInformationAtom(Atom,
//                                         AtomBasicInformation,
//                                         AtomInfo,
//                                         AtomInfoLength,
//                                         &AtomInfoLength);
//         if (NT_SUCCESS(Status))
//         {
//             /* Success. Update the length and get the name */
//             AtomNameLength = (ULONG)AtomInfo->NameLength;
//             AtomNameString = AtomInfo->Name;
//         }
//     }
//
//     /* Check for global success */
//     if (NT_SUCCESS(Status))
//     {
//         /* Check if it was unicode */
//         if (Unicode)
//         {
//             /* We return the length in chars */
//             RetVal = AtomNameLength / sizeof(WCHAR);
//
//             /* Copy the memory if this was a global query */
//             if (AtomNameString != (PWSTR)AtomName)
//             {
//                 RtlMoveMemory(AtomName, AtomNameString, AtomNameLength);
//             }
//
//             /* And null-terminate it if the buffer was too large */
//             if (RetVal < Size)
//             {
//                 ((PWCHAR)AtomName)[RetVal] = UNICODE_NULL;
//             }
//         }
//         else
//         {
//             /* First create a unicode string with our data */
//             UnicodeString.Buffer = AtomNameString;
//             UnicodeString.Length = (USHORT)AtomNameLength;
//             UnicodeString.MaximumLength = (USHORT)(UnicodeString.Length +
//                                                    sizeof(WCHAR));
//
//             /* Now prepare an ansi string for conversion */
//             AnsiString.Buffer = AtomName;
//             AnsiString.Length = 0;
//             AnsiString.MaximumLength = (USHORT)Size;
//
//             /* Convert it */
//             Status = RtlUnicodeStringToAnsiString(&AnsiString,
//                                                   &UnicodeString,
//                                                   FALSE);
//
//             /* Return the length */
//             if (NT_SUCCESS(Status)) RetVal = AnsiString.Length;
//         }
//     }
//
//     /* Free the temporary buffer if we have one */
//     if (TempBuffer) RtlFreeHeap(RtlGetProcessHeap(), 0, TempBuffer);
//
//     /* Check for failure */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         DPRINT("Failed: %lx\n", Status);
//         BaseSetLastNTError(Status);
//     }
//
//     /* Return length */
//     return RetVal;
// }
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// ATOM
// WINAPI
// GlobalAddAtomA(LPCSTR lpString)
// {
//     return InternalAddAtom(FALSE, FALSE, lpString);
// }
//
// /*
//  * @implemented
//  */
// ATOM
// WINAPI
// GlobalAddAtomW(LPCWSTR lpString)
// {
//     return InternalAddAtom(FALSE, TRUE, (LPSTR)lpString);
// }
//
// /*
//  * @implemented
//  */
// ATOM
// WINAPI
// GlobalDeleteAtom(ATOM nAtom)
// {
//     return InternalDeleteAtom(FALSE, nAtom);
// }
//
// /*
//  * @implemented
//  */
// ATOM
// WINAPI
// GlobalFindAtomA(LPCSTR lpString)
// {
//     return InternalFindAtom(FALSE, FALSE, lpString);
// }
//
// /*
//  * @implemented
//  */
// ATOM
// WINAPI
// GlobalFindAtomW(LPCWSTR lpString)
// {
//     return InternalFindAtom(FALSE, TRUE, (LPSTR)lpString);
// }
//
// /*
//  * @implemented
//  */
// UINT
// WINAPI
// GlobalGetAtomNameA(ATOM nAtom,
//                    LPSTR lpBuffer,
//                    int nSize)
// {
//     return InternalGetAtomName(FALSE, FALSE, nAtom, lpBuffer, (DWORD)nSize);
// }
//
// /*
//  * @implemented
//  */
// UINT
// WINAPI
// GlobalGetAtomNameW(ATOM nAtom,
//                    LPWSTR lpBuffer,
//                    int nSize)
// {
//     return InternalGetAtomName(FALSE,
//                                TRUE,
//                                nAtom,
//                                (LPSTR)lpBuffer,
//                                (DWORD)nSize);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// InitAtomTable(DWORD nSize)
// {
//     /* Normalize size */
//     if (nSize < 4 || nSize > 511) nSize = 37;
//
//     DPRINT("Here\n");
//     return NT_SUCCESS(RtlCreateAtomTable(nSize, &BaseLocalAtomTable));
// }
//
// /*
//  * @implemented
//  */
// ATOM
// WINAPI
// AddAtomA(LPCSTR lpString)
// {
//     return InternalAddAtom(TRUE, FALSE, lpString);
// }
//
// /*
//  * @implemented
//  */
// ATOM
// WINAPI
// AddAtomW(LPCWSTR lpString)
// {
//     return InternalAddAtom(TRUE, TRUE, (LPSTR)lpString);
// }
//
// /*
//  * @implemented
//  */
// ATOM
// WINAPI
// DeleteAtom(ATOM nAtom)
// {
//     return InternalDeleteAtom(TRUE, nAtom);
// }
//
// /*
//  * @implemented
//  */
// ATOM
// WINAPI
// FindAtomA(LPCSTR lpString)
// {
//     return InternalFindAtom(TRUE, FALSE, lpString);
// }
//
// /*
//  * @implemented
//  */
// ATOM
// WINAPI
// FindAtomW(LPCWSTR lpString)
// {
//     return InternalFindAtom(TRUE, TRUE, (LPSTR)lpString);
//
// }
//
// /*
//  * @implemented
//  */
// UINT
// WINAPI
// GetAtomNameA(ATOM nAtom,
//              LPSTR lpBuffer,
//              int nSize)
// {
//     return InternalGetAtomName(TRUE, FALSE, nAtom, lpBuffer, (DWORD)nSize);
// }
//
// /*
//  * @implemented
//  */
// UINT
// WINAPI
// GetAtomNameW(ATOM nAtom,
//              LPWSTR lpBuffer,
//              int nSize)
// {
//     return InternalGetAtomName(TRUE,
//                                TRUE,
//                                nAtom,
//                                (LPSTR)lpBuffer,
//                                (DWORD)nSize);
// }
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
