use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::probe::*;

use ntkrnl_sys::ddk::{
    ATOM_INFORMATION_CLASS,
    _ATOM_INFORMATION_CLASS_AtomBasicInformation as AtomBasicInformation,
    _ATOM_INFORMATION_CLASS_AtomTableInformation as AtomTableInformation,
    KPROCESSOR_MODE,
    PRTL_ATOM,
    PRTL_ATOM_TABLE,
    RTL_ATOM,
    _ATOM_BASIC_INFORMATION,
    _ATOM_TABLE_INFORMATION,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    fn ExGetPreviousMode() -> KPROCESSOR_MODE;
    fn RtlCreateAtomTable(NumberOfBuckets: ULONG, AtomTableHandle: *mut PRTL_ATOM_TABLE) -> NTSTATUS;
    fn RtlAddAtomToAtomTable(AtomTable: PRTL_ATOM_TABLE, AtomName: PWSTR, Atom: PRTL_ATOM) -> NTSTATUS;
    fn RtlDeleteAtomFromAtomTable(AtomTable: PRTL_ATOM_TABLE, Atom: RTL_ATOM) -> NTSTATUS;
    fn RtlLookupAtomInAtomTable(AtomTable: PRTL_ATOM_TABLE, AtomName: PWSTR, Atom: PRTL_ATOM) -> NTSTATUS;
    fn RtlQueryAtomInAtomTable( AtomTable: PRTL_ATOM_TABLE, Atom: RTL_ATOM, RefCount: *mut ULONG, PinCount: *mut ULONG, Name: PWSTR, NameLength: *mut ULONG, ) -> NTSTATUS;
    fn RtlQueryAtomListInAtomTable( AtomTable: PRTL_ATOM_TABLE, MaximumNumberOfAtoms: ULONG, NumberOfAtoms: *mut ULONG, Atoms: *mut RTL_ATOM, ) -> NTSTATUS;
}

const RTL_MAXIMUM_ATOM_LENGTH: ULONG = 255;
const TAG_ATOM: ULONG = 0x6D6F7441;
const STATUS_ACCESS_DENIED: NTSTATUS = 0xC0000022u32 as NTSTATUS;
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000Du32 as NTSTATUS;
const STATUS_INFO_LENGTH_MISMATCH: NTSTATUS = 0xC0000004u32 as NTSTATUS;
const STATUS_INVALID_INFO_CLASS: NTSTATUS = 0xC0000003u32 as NTSTATUS;
const UNICODE_NULL: u16 = 0;

static mut GlobalAtomTable: PRTL_ATOM_TABLE = core::ptr::null_mut();

unsafe fn ExpGetGlobalAtomTable() -> PRTL_ATOM_TABLE {
    if !GlobalAtomTable.is_null() {
        return GlobalAtomTable;
    }

    let Status = RtlCreateAtomTable(37, &raw mut GlobalAtomTable);

    if Status < 0 {
        return core::ptr::null_mut();
    }

    GlobalAtomTable
}

#[no_mangle]
pub unsafe extern "system" fn NtAddAtom(
    AtomName: PWSTR,
    AtomNameLength: ULONG,
    Atom: PRTL_ATOM,
) -> NTSTATUS {
    let AtomName = opaque_ptr(AtomName);
    let inner = || -> Result<NTSTATUS, NTSTATUS> {
        let AtomTable = ExpGetGlobalAtomTable();
        let PreviousMode = ExGetPreviousMode();
        let mut CapturedName: PWSTR;
        let mut SafeAtom: RTL_ATOM = 0;

        if AtomTable.is_null() {
            return Ok(STATUS_ACCESS_DENIED);
        }

        if AtomNameLength > (RTL_MAXIMUM_ATOM_LENGTH * core::mem::size_of::<u16>() as ULONG) {
            return Ok(STATUS_INVALID_PARAMETER);
        }

        if PreviousMode == 0i8 {
            CapturedName = AtomName;
        } else {
            if !AtomName.is_null() {
                let CapturedSize = (AtomNameLength + core::mem::size_of::<u16>() as ULONG) &
                                    !(core::mem::size_of::<u16>() as ULONG - 1) ;
                CapturedName = ExAllocatePoolWithTag(
                    PagedPool,
                    CapturedSize as usize,
                    TAG_ATOM,
                ) as PWSTR;

                if CapturedName.is_null() {
                    return Ok(STATUS_INSUFFICIENT_RESOURCES);
                }

                probe_user_buffer(
                    AtomName as *const core::ffi::c_void,
                    AtomNameLength as usize,
                    core::mem::size_of::<u16>(),
                    PreviousMode,
                )?;

                // The probe proves where AtomName is, not that it stays mapped; a
                // raw copy from it is an any-process bugcheck. On fault free the
                // pool buffer and return the status, as the C's __except does.
                if let Err(e) = ntkrnl_sys::probe::safe_copy_raw(
                    CapturedName as *mut core::ffi::c_void,
                    AtomName as *const core::ffi::c_void,
                    AtomNameLength as usize,
                ) {
                    ExFreePoolWithTag(CapturedName as PVOID, TAG_ATOM);
                    return Err(e);
                }
                *(CapturedName.add((AtomNameLength / core::mem::size_of::<u16>() as ULONG) as usize)) = UNICODE_NULL;

                if !Atom.is_null() {
                    probe_user_buffer(
                        Atom as *const core::ffi::c_void,
                        core::mem::size_of::<RTL_ATOM>(),
                        core::mem::align_of::<RTL_ATOM>(),
                        PreviousMode,
                    )?;
                }
            } else {
                CapturedName = core::ptr::null_mut();
            }
        }

        let mut Status = RtlAddAtomToAtomTable(AtomTable, CapturedName, &mut SafeAtom);

        if Status >= 0 && !Atom.is_null() {
            if let Err(e) = write_user(Atom, SafeAtom, PreviousMode) {
                Status = e;
            }
        }

        if !CapturedName.is_null() && CapturedName != AtomName {
            ExFreePoolWithTag(CapturedName as PVOID, TAG_ATOM);
        }

        Ok(Status)
    };

    match inner() {
        Ok(s) => s,
        Err(s) => s,
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtDeleteAtom(Atom: RTL_ATOM) -> NTSTATUS {
    let AtomTable = ExpGetGlobalAtomTable();

    if AtomTable.is_null() {
        return STATUS_ACCESS_DENIED;
    }

    RtlDeleteAtomFromAtomTable(AtomTable, Atom)
}

#[no_mangle]
pub unsafe extern "system" fn NtFindAtom(
    AtomName: PWSTR,
    AtomNameLength: ULONG,
    Atom: PRTL_ATOM,
) -> NTSTATUS {
    let AtomName = opaque_ptr(AtomName);
    let inner = || -> Result<NTSTATUS, NTSTATUS> {
        let AtomTable = ExpGetGlobalAtomTable();
        let PreviousMode = ExGetPreviousMode();
        let mut CapturedName: PWSTR = AtomName;
        let mut SafeAtom: RTL_ATOM = 0;

        if AtomTable.is_null() {
            return Ok(STATUS_ACCESS_DENIED);
        }

        if AtomNameLength > (RTL_MAXIMUM_ATOM_LENGTH * core::mem::size_of::<u16>() as ULONG) {
            return Ok(STATUS_INVALID_PARAMETER);
        }

        if PreviousMode != 0i8 {
            if !AtomName.is_null() {
                probe_user_buffer(
                    AtomName as *const core::ffi::c_void,
                    AtomNameLength as usize,
                    core::mem::size_of::<u16>(),
                    PreviousMode,
                )?;

                let CapturedSize = (AtomNameLength + core::mem::size_of::<u16>() as ULONG) &
                                    !(core::mem::size_of::<u16>() as ULONG - 1) ;
                CapturedName = ExAllocatePoolWithTag(
                    PagedPool,
                    CapturedSize as usize,
                    TAG_ATOM,
                ) as PWSTR;

                if CapturedName.is_null() {
                    return Ok(STATUS_INSUFFICIENT_RESOURCES);
                }

                // Fault-tolerant copy from the caller's page; free and bail on fault.
                if let Err(e) = ntkrnl_sys::probe::safe_copy_raw(
                    CapturedName as *mut core::ffi::c_void,
                    AtomName as *const core::ffi::c_void,
                    AtomNameLength as usize,
                ) {
                    ExFreePoolWithTag(CapturedName as PVOID, TAG_ATOM);
                    return Err(e);
                }
                *(CapturedName.add((AtomNameLength / core::mem::size_of::<u16>() as ULONG) as usize)) = UNICODE_NULL;

                if !Atom.is_null() {
                    probe_user_buffer(
                        Atom as *const core::ffi::c_void,
                        core::mem::size_of::<RTL_ATOM>(),
                        core::mem::align_of::<RTL_ATOM>(),
                        PreviousMode,
                    )?;
                }
            }
        }

        let mut Status = RtlLookupAtomInAtomTable(AtomTable, CapturedName, &mut SafeAtom);

        if Status >= 0 && !Atom.is_null() {
            if let Err(e) = write_user(Atom, SafeAtom, PreviousMode) {
                Status = e;
            }
        }

        if !CapturedName.is_null() && CapturedName != AtomName {
            ExFreePoolWithTag(CapturedName as PVOID, TAG_ATOM);
        }

        Ok(Status)
    };

    match inner() {
        Ok(s) => s,
        Err(s) => s,
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryInformationAtom(
    Atom: RTL_ATOM,
    AtomInformationClass: ATOM_INFORMATION_CLASS,
    AtomInformation: PVOID,
    AtomInformationLength: ULONG,
    ReturnLength: *mut ULONG,
) -> NTSTATUS {
    let AtomInformation = opaque_ptr(AtomInformation);
    let ReturnLength = opaque_ptr(ReturnLength);
    let inner = || -> Result<NTSTATUS, NTSTATUS> {
        let AtomTable = ExpGetGlobalAtomTable();
        let mut Status = STATUS_SUCCESS;
        let mut Flags: ULONG = 0;
        let mut UsageCount: ULONG = 0;
        let mut NameLength: ULONG;
        let mut RequiredLength: ULONG = 0;
        let PreviousMode: KPROCESSOR_MODE;

        if AtomTable.is_null() {
            return Ok(STATUS_ACCESS_DENIED);
        }

        PreviousMode = ExGetPreviousMode();

        if PreviousMode != 0i8 {
            probe_user_buffer(
                AtomInformation as *const core::ffi::c_void,
                AtomInformationLength as usize,
                core::mem::size_of::<ULONG>(),
                PreviousMode,
            )?;

            if !ReturnLength.is_null() {
                probe_user_buffer(
                    ReturnLength as *const core::ffi::c_void,
                    core::mem::size_of::<ULONG>(),
                    core::mem::align_of::<ULONG>(),
                    PreviousMode,
                )?;
            }
        }

        // The C fills AtomInformation in place under _SEH2_TRY, and the Rtl atom
        // routines write it while holding the table lock. A user-mode caller's buffer
        // is a pool copy here, copied back once below (OutputBounce).
        let bounce = OutputBounce::new(AtomInformation, AtomInformationLength as usize, PreviousMode)?;
        let BasicInformation = bounce.buffer() as *mut _ATOM_BASIC_INFORMATION;
        let TableInformation = bounce.buffer() as *mut _ATOM_TABLE_INFORMATION;

        match AtomInformationClass {
            AtomBasicInformation => {
                RequiredLength = core::mem::offset_of!(_ATOM_BASIC_INFORMATION, Name) as ULONG;
                if RequiredLength > AtomInformationLength {
                    Status = STATUS_INFO_LENGTH_MISMATCH;
                } else {
                    UsageCount = 0;
                    NameLength = AtomInformationLength - RequiredLength;
                    *((*BasicInformation).Name.as_mut_ptr()) = UNICODE_NULL;

                    Status = RtlQueryAtomInAtomTable(
                        AtomTable,
                        Atom,
                        &mut UsageCount,
                        &mut Flags,
                        (*BasicInformation).Name.as_mut_ptr(),
                        &mut NameLength,
                    );
                    if Status >= 0 {
                        (*BasicInformation).UsageCount = UsageCount as u16;
                        (*BasicInformation).Flags = Flags as u16;
                        (*BasicInformation).NameLength = NameLength as u16;
                        RequiredLength += NameLength + core::mem::size_of::<u16>() as ULONG;
                    }
                }
            }

            AtomTableInformation => {
                RequiredLength = core::mem::offset_of!(_ATOM_TABLE_INFORMATION, Atoms) as ULONG;
                if RequiredLength > AtomInformationLength {
                    Status = STATUS_INFO_LENGTH_MISMATCH;
                } else {
                    Status = RtlQueryAtomListInAtomTable(
                        AtomTable,
                        (AtomInformationLength - RequiredLength) / core::mem::size_of::<RTL_ATOM>() as ULONG,
                        &mut (*TableInformation).NumberOfAtoms,
                        (*TableInformation).Atoms.as_mut_ptr(),
                    );
                    if Status >= 0 {
                        RequiredLength += (*TableInformation).NumberOfAtoms * core::mem::size_of::<RTL_ATOM>() as ULONG;
                    }
                }
            }

            _ => {
                Status = STATUS_INVALID_INFO_CLASS;
            }
        }

        bounce.finish()?;

        if !ReturnLength.is_null() {
            write_user(ReturnLength, RequiredLength, PreviousMode)?;
        }

        Ok(Status)
    };

    match inner() {
        Ok(s) => s,
        Err(s) => s,
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * COPYRIGHT:       GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ex/atom.c
//  * PURPOSE:         Executive Atom Functions
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
//  *                  Gunnar Dalsnes
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ****************************************************************/
//
// /*
//  * FIXME: this is WRONG! The global atom table should live in the WinSta struct
//  * and accessed through a win32k callout (received in PsEstablishWin32Callouts)
//  * NOTE: There is a session/win32k global atom table also, but its private to
//  * win32k. Its used for RegisterWindowMessage() and for window classes.
//  * -Gunnar
//  */
// PRTL_ATOM_TABLE GlobalAtomTable;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// /*++
//  * @name ExpGetGlobalAtomTable
//  *
//  * Gets pointer to a global atom table, creates it if not already created
//  *
//  * @return Pointer to the RTL_ATOM_TABLE, or NULL if it's impossible
//  *         to create atom table
//  *
//  * @remarks Internal function
//  *
//  *--*/
// PRTL_ATOM_TABLE
// NTAPI
// ExpGetGlobalAtomTable(VOID)
// {
//     NTSTATUS Status;
//
//     /* Return it if we have one */
//     if (GlobalAtomTable) return GlobalAtomTable;
//
//     /* Create it */
//     Status = RtlCreateAtomTable(37, &GlobalAtomTable);
//
//     /* If we couldn't create it, return NULL */
//     if (!NT_SUCCESS(Status)) return NULL;
//
//     /* Return the newly created one */
//     return GlobalAtomTable;
// }
//
// /* FUNCTIONS ****************************************************************/
//
// /*++
//  * @name NtAddAtom
//  * @implemented
//  *
//  * Function NtAddAtom creates new Atom in Global Atom Table. If Atom
//  * with the same name already exist, internal Atom counter is incremented.
//  * See: http://undocumented.ntinternals.net/UserMode/Undocumented%20Functions/Atoms/NtAddAtom.html
//  *
//  * @param AtomName
//  *        Atom name in Unicode
//  *
//  * @param AtomNameLength
//  *        Length of the atom name
//  *
//  * @param Atom
//  *        Pointer to RTL_ATOM
//  *
//  * @return STATUS_SUCCESS in case of success, proper error code
//  *         otherwise.
//  *
//  * @remarks None
//  *
//  *--*/
// NTSTATUS
// NTAPI
// NtAddAtom(IN PWSTR AtomName,
//           IN ULONG AtomNameLength,
//           OUT PRTL_ATOM Atom)
// {
//     PRTL_ATOM_TABLE AtomTable = ExpGetGlobalAtomTable();
//     NTSTATUS Status;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     LPWSTR CapturedName;
//     ULONG CapturedSize;
//     RTL_ATOM SafeAtom;
//     PAGED_CODE();
//
//     /* Check for the table */
//     if (AtomTable == NULL) return STATUS_ACCESS_DENIED;
//
//     /* Check for valid name */
//     if (AtomNameLength > (RTL_MAXIMUM_ATOM_LENGTH * sizeof(WCHAR)))
//     {
//         /* Fail */
//         DPRINT1("Atom name too long\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Check if we're called from user-mode or kernel-mode */
//     if (PreviousMode == KernelMode)
//     {
//         /* Re-use the given name if kernel mode */
//         CapturedName = AtomName;
//     }
//     else
//     {
//         /* Check if we have a name */
//         if (AtomName)
//         {
//             /* Allocate an aligned buffer + the null char */
//             CapturedSize = ((AtomNameLength + sizeof(WCHAR)) &
//                             ~(sizeof(WCHAR) -1));
//             CapturedName = ExAllocatePoolWithTag(PagedPool,
//                                                  CapturedSize,
//                                                  TAG_ATOM);
//
//             if (!CapturedName)
//             {
//                 /* Fail the call */
//                 return STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             /* Enter SEH */
//             _SEH2_TRY
//             {
//                 /* Probe the atom */
//                 ProbeForRead(AtomName, AtomNameLength, sizeof(WCHAR));
//
//                 /* Copy the name and null-terminate it */
//                 RtlCopyMemory(CapturedName, AtomName, AtomNameLength);
//                 CapturedName[AtomNameLength / sizeof(WCHAR)] = UNICODE_NULL;
//
//                 /* Probe the atom too */
//                 if (Atom) ProbeForWriteUshort(Atom);
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Return the exception code */
//                 _SEH2_YIELD(return _SEH2_GetExceptionCode());
//             }
//             _SEH2_END;
//         }
//         else
//         {
//             /* No name */
//             CapturedName = NULL;
//         }
//     }
//
//     /* Call the runtime function */
//     Status = RtlAddAtomToAtomTable(AtomTable, CapturedName, &SafeAtom);
//     if (NT_SUCCESS(Status) && (Atom))
//     {
//         /* Success and caller wants the atom back.. .enter SEH */
//         _SEH2_TRY
//         {
//             /* Return the atom */
//             *Atom = SafeAtom;
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//
//     /* If we captured anything, free it */
//     if ((CapturedName != NULL) && (CapturedName != AtomName))
//         ExFreePoolWithTag(CapturedName, TAG_ATOM);
//
//     /* Return to caller */
//     return Status;
// }
//
// /*++
//  * @name NtDeleteAtom
//  * @implemented
//  *
//  * Removes Atom from Global Atom Table. If Atom's reference counter
//  * is greater then 1, function decrements this counter, but Atom
//  * stayed in Global Atom Table.
//  * See: http://undocumented.ntinternals.net/UserMode/Undocumented%20Functions/Atoms/NtDeleteAtom.html
//  *
//  * @param Atom
//  *        Atom identifier
//  *
//  * @return STATUS_SUCCESS in case of success, proper error code
//  *         otherwise.
//  *
//  * @remarks None
//  *
//  *--*/
// NTSTATUS
// NTAPI
// NtDeleteAtom(IN RTL_ATOM Atom)
// {
//     PRTL_ATOM_TABLE AtomTable = ExpGetGlobalAtomTable();
//     PAGED_CODE();
//
//     /* Check for valid table */
//     if (AtomTable == NULL) return STATUS_ACCESS_DENIED;
//
//     /* Call worker function */
//     return RtlDeleteAtomFromAtomTable(AtomTable, Atom);
// }
//
// /*++
//  * @name NtFindAtom
//  * @implemented
//  *
//  * Retrieves existing Atom's identifier without incrementing Atom's
//  * internal counter
//  * See: http://undocumented.ntinternals.net/UserMode/Undocumented%20Functions/Atoms/NtFindAtom.html
//  *
//  * @param AtomName
//  *        Atom name in Unicode
//  *
//  * @param AtomNameLength
//  *        Length of the atom name
//  *
//  * @param Atom
//  *        Pointer to RTL_ATOM
//  *
//  * @return STATUS_SUCCESS in case of success, proper error code
//  *         otherwise.
//  *
//  * @remarks None
//  *
//  *--*/
// NTSTATUS
// NTAPI
// NtFindAtom(IN PWSTR AtomName,
//            IN ULONG AtomNameLength,
//            OUT PRTL_ATOM Atom)
// {
//     PRTL_ATOM_TABLE AtomTable = ExpGetGlobalAtomTable();
//     NTSTATUS Status;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     _SEH2_VOLATILE LPWSTR CapturedName;
//     ULONG CapturedSize;
//     RTL_ATOM SafeAtom;
//     PAGED_CODE();
//
//     /* Check for the table */
//     if (AtomTable == NULL) return STATUS_ACCESS_DENIED;
//
//     /* Check for valid name */
//     if (AtomNameLength > (RTL_MAXIMUM_ATOM_LENGTH * sizeof(WCHAR)))
//     {
//         /* Fail */
//         DPRINT1("Atom name too long\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Re-use the given name if kernel mode or no atom name */
//     CapturedName = AtomName;
//
//     /* Check if we're called from user-mode*/
//     if (PreviousMode != KernelMode)
//     {
//         /* Enter SEH */
//         _SEH2_TRY
//         {
//             /* Check if we have a name */
//             if (AtomName)
//             {
//                 /* Probe the atom */
//                 ProbeForRead(AtomName, AtomNameLength, sizeof(WCHAR));
//
//                 /* Allocate an aligned buffer + the null char */
//                 CapturedSize = ((AtomNameLength + sizeof(WCHAR)) &~
//                                 (sizeof(WCHAR) -1));
//                 CapturedName = ExAllocatePoolWithQuotaTag(PagedPool,
//                                                           CapturedSize,
//                                                           TAG_ATOM);
//                 /* Copy the name and null-terminate it */
//                 RtlCopyMemory(CapturedName, AtomName, AtomNameLength);
//                 CapturedName[AtomNameLength / sizeof(WCHAR)] = UNICODE_NULL;
//
//                 /* Probe the atom too */
//                 if (Atom) ProbeForWriteUshort(Atom);
//             }
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             if (CapturedName != AtomName)
//             {
//                 ExFreePoolWithTag(CapturedName, TAG_ATOM);
//             }
//
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Call the runtime function */
//     Status = RtlLookupAtomInAtomTable(AtomTable, CapturedName, &SafeAtom);
//     if (NT_SUCCESS(Status) && (Atom))
//     {
//         /* Success and caller wants the atom back... enter SEH */
//         _SEH2_TRY
//         {
//             /* Return the atom */
//             *Atom = SafeAtom;
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//
//     /* If we captured anything, free it */
//     if ((CapturedName) && (CapturedName != AtomName))
//         ExFreePoolWithTag(CapturedName, TAG_ATOM);
//
//     /* Return to caller */
//     return Status;
// }
//
// /*++
//  * @name NtQueryInformationAtom
//  * @implemented
//  *
//  * Gets single Atom properties or reads Global Atom Table
//  * See: http://undocumented.ntinternals.net/UserMode/Undocumented%20Functions/Atoms/NtQueryInformationAtom.html
//  *
//  * @param Atom
//  *        Atom to query. If AtomInformationClass parameter is
//  *        AtomTableInformation, Atom parameter is not used.
//  *
//  * @param AtomInformationClass
//  *        See ATOM_INFORMATION_CLASS enumeration type for details
//  *
//  * @param AtomInformation
//  *        Result of call - pointer to user's allocated buffer for data
//  *
//  * @param AtomInformationLength
//  *        Size of AtomInformation buffer, in bytes
//  *
//  * @param ReturnLength
//  *        Pointer to ULONG value containing required AtomInformation
//  *        buffer size
//  *
//  * @return STATUS_SUCCESS in case of success, proper error code
//  *         otherwise.
//  *
//  * @remarks None
//  *
//  *--*/
// NTSTATUS
// NTAPI
// NtQueryInformationAtom(RTL_ATOM Atom,
//                        ATOM_INFORMATION_CLASS AtomInformationClass,
//                        PVOID AtomInformation,
//                        ULONG AtomInformationLength,
//                        PULONG ReturnLength)
// {
//     PRTL_ATOM_TABLE AtomTable = ExpGetGlobalAtomTable();
//     PATOM_BASIC_INFORMATION BasicInformation = AtomInformation;
//     PATOM_TABLE_INFORMATION TableInformation = AtomInformation;
//     NTSTATUS Status = STATUS_SUCCESS;
//     ULONG Flags, UsageCount, NameLength, RequiredLength = 0;
//     KPROCESSOR_MODE PreviousMode;
//
//     PAGED_CODE();
//
//     /* Check for valid table */
//     if (AtomTable == NULL) return STATUS_ACCESS_DENIED;
//
//     PreviousMode = ExGetPreviousMode();
//
//     _SEH2_TRY
//     {
//         /* Probe the parameters */
//         if (PreviousMode != KernelMode)
//         {
//             ProbeForWrite(AtomInformation,
//                           AtomInformationLength,
//                           sizeof(ULONG));
//
//             if (ReturnLength != NULL)
//             {
//                 ProbeForWriteUlong(ReturnLength);
//             }
//         }
//
//         /* Choose class */
//         switch (AtomInformationClass)
//         {
//             /* Caller requested info about an atom */
//             case AtomBasicInformation:
//
//                 /* Size check */
//                 RequiredLength = FIELD_OFFSET(ATOM_BASIC_INFORMATION, Name);
//                 if (RequiredLength > AtomInformationLength)
//                 {
//                     /* Fail */
//                     DPRINT1("Buffer too small\n");
//                     Status = STATUS_INFO_LENGTH_MISMATCH;
//                     _SEH2_LEAVE;
//                 }
//
//                 /* Prepare query */
//                 UsageCount = 0;
//                 NameLength = AtomInformationLength - RequiredLength;
//                 BasicInformation->Name[0] = UNICODE_NULL;
//
//                 /* Query the data */
//                 Status = RtlQueryAtomInAtomTable(AtomTable,
//                                                  Atom,
//                                                  &UsageCount,
//                                                  &Flags,
//                                                  BasicInformation->Name,
//                                                  &NameLength);
//                 if (NT_SUCCESS(Status))
//                 {
//                     /* Return data */
//                     BasicInformation->UsageCount = (USHORT)UsageCount;
//                     BasicInformation->Flags = (USHORT)Flags;
//                     BasicInformation->NameLength = (USHORT)NameLength;
//                     RequiredLength += NameLength + sizeof(WCHAR);
//                 }
//                 break;
//
//             /* Caller requested info about an Atom Table */
//             case AtomTableInformation:
//
//                 /* Size check */
//                 RequiredLength = FIELD_OFFSET(ATOM_TABLE_INFORMATION, Atoms);
//                 if (RequiredLength > AtomInformationLength)
//                 {
//                     /* Fail */
//                     DPRINT1("Buffer too small\n");
//                     Status = STATUS_INFO_LENGTH_MISMATCH;
//                     _SEH2_LEAVE;
//                 }
//
//                 /* Query the data */
//                 Status = RtlQueryAtomListInAtomTable(AtomTable,
//                                                      (AtomInformationLength - RequiredLength) /
//                                                      sizeof(RTL_ATOM),
//                                                      &TableInformation->NumberOfAtoms,
//                                                      TableInformation->Atoms);
//                 if (NT_SUCCESS(Status))
//                 {
//                     /* Update the return length */
//                     RequiredLength += TableInformation->NumberOfAtoms * sizeof(RTL_ATOM);
//                 }
//                 break;
//
//             /* Caller was on crack */
//             default:
//
//                 /* Unrecognized class */
//                 Status = STATUS_INVALID_INFO_CLASS;
//                 break;
//         }
//
//         /* Return the required size */
//         if (ReturnLength != NULL)
//         {
//             *ReturnLength = RequiredLength;
//         }
//     }
//     _SEH2_EXCEPT(ExSystemExceptionFilter())
//     {
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Return to caller */
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
