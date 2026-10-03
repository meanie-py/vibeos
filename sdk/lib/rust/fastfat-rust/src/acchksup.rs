use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ddk::{ACCESS_MASK, IoGetFileObjectGenericMapping, KPROCESSOR_MODE, PACCESS_MASK, PACCESS_STATE, PACCESS_TOKEN, PDEVICE_OBJECT, PRIVILEGE_SET, RtlConvertLongToLuid, SeAccessCheck, SeFilterToken, SeLockSubjectContext, SePrivilegeCheck, SeUnlockSubjectContext, TOKEN_GROUPS};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{PIRP_CONTEXT, IRP_CONTEXT};
use ntkrnl_sys::ddk::{SECURITY_SUBJECT_CONTEXT, SE_EXPORTS, _PROCESS_ACCESS_TOKEN as ACCESS_TOKEN};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DEBUG_TRACE_ACCHKSUP, FILE_DELETE_CHILD};
use ntkrnl_sys::ddk::{PPRIVILEGE_SET};
// resolve_imports.py: explicit kernel/private-type imports

// SeExports is a kernel DATA export — raw-dylib routes through the __imp_ thunk.
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static SeExports: *mut SE_EXPORTS;
}
extern "system" {
    fn PAGED_CODE();
    fn SeReleaseSubjectContext(SubjectContext: *mut SECURITY_SUBJECT_CONTEXT);
}

// ---- file-scope preamble (translated) ----
pub const Dbg: u32 = DEBUG_TRACE_ACCHKSUP;

const SPECIFIC_RIGHTS_ALL: u32 = 0x0000FFFF;
const FILE_TRAVERSE: u32 = 0x00000020;
#[no_mangle]
pub unsafe extern "system" fn FatCheckFileAccess(
    IrpContext: PIRP_CONTEXT,
    DirentAttributes: u8,
    DesiredAccess: *mut u32,
) -> i32 {
    // FAT attribute constants
    const FAT_DIRENT_ATTR_READ_ONLY: u8 = 0x01;
    const FAT_DIRENT_ATTR_VOLUME_ID: u8 = 0x08;
    const FAT_DIRENT_ATTR_DIRECTORY: u8 = 0x10;
    const FAT_DIRENT_ATTR_DEVICE: u8 = 0x40;

    // Access mask constants (not present in the global scope)
    const FILE_LIST_DIRECTORY: u32 = 0x01;
    const FILE_TRAVERSE: u32 = 0x00000020;
    const FILE_ADD_SUBDIRECTORY: u32 = 0x00000004; // note: shares value with FILE_LIST_DIRECTORY in FAT, adjust if needed
    const FILE_ADD_FILE: u32 = 0x00000002;

    // Dbg expands from #define Dbg (DEBUG_TRACE_ACCHKSUP)
    const DEBUG_TRACE_ACCHKSUP: u32 = 0x00100000;
    const Dbg: u32 = DEBUG_TRACE_ACCHKSUP;

    // Declare external functions
    unsafe extern "system" {
        } extern "C" { fn DebugTrace(level: i32, mask: u32, format: *const i8, ...); } extern "system" {
        fn DebugUnwind(fn_name: usize);
        fn PAGED_CODE();
    }

    PAGED_CODE();

    let mut result: i32 = 1; // TRUE
    let p_result = core::ptr::addr_of_mut!(result);

    // _SEH2_TRY { ... } _SEH2_FINALLY { ... }
    crate::seh::try_finally(&mut (), |_| loop {
        DebugTrace(1, Dbg, b"FatCheckFileAccess\n\0".as_ptr() as *const i8, 0);
        DebugTrace(0, Dbg, b"DirentAttributes = %8lx\n\0".as_ptr() as *const i8, DirentAttributes as usize);
        DebugTrace(0, Dbg, b"DesiredAccess    = %8lx\n\0".as_ptr() as *const i8, *DesiredAccess as usize);

        // Check for Volume ID or Device Dirents, no user access
        if (DirentAttributes & FAT_DIRENT_ATTR_VOLUME_ID) != 0 ||
           (DirentAttributes & FAT_DIRENT_ATTR_DEVICE) != 0 {
            DebugTrace(0, Dbg, b"Cannot access volume id or device\n\0".as_ptr() as *const i8, 0);
            *p_result = 0;
            break;
        }

        // Blacklist unknown desired access bits
        let allowed_mask: u32 = DELETE
            | READ_CONTROL
            | WRITE_OWNER
            | WRITE_DAC
            | SYNCHRONIZE
            | ACCESS_SYSTEM_SECURITY
            | FILE_WRITE_DATA
            | FILE_READ_EA
            | FILE_WRITE_EA
            | FILE_READ_ATTRIBUTES
            | FILE_WRITE_ATTRIBUTES
            | FILE_LIST_DIRECTORY
            | FILE_TRAVERSE
            | FILE_DELETE_CHILD
            | FILE_APPEND_DATA
            | MAXIMUM_ALLOWED;
        if (*DesiredAccess & !allowed_mask) != 0 {
            DebugTrace(0, Dbg, b"Cannot open object\n\0".as_ptr() as *const i8, 0);
            *p_result = 0;
            break;
        }

        // Check for a read-only Dirent
        if (DirentAttributes & FAT_DIRENT_ATTR_READ_ONLY) != 0 {
            let mut access_mask: u32 = DELETE
                | READ_CONTROL
                | WRITE_OWNER
                | WRITE_DAC
                | SYNCHRONIZE
                | ACCESS_SYSTEM_SECURITY
                | FILE_READ_DATA
                | FILE_READ_EA
                | FILE_WRITE_EA
                | FILE_READ_ATTRIBUTES
                | FILE_WRITE_ATTRIBUTES
                | FILE_EXECUTE
                | FILE_LIST_DIRECTORY
                | FILE_TRAVERSE;

            // Allow add file/directory and delete child for subdirectories
            if (DirentAttributes & FAT_DIRENT_ATTR_DIRECTORY) != 0 {
                access_mask |= FILE_ADD_SUBDIRECTORY | FILE_ADD_FILE | FILE_DELETE_CHILD;
            }

            if (*DesiredAccess & !access_mask) != 0 {
                DebugTrace(0, Dbg, b"Cannot open readonly\n\0".as_ptr() as *const i8, 0);
                *p_result = 0;
                break;
            }
        }

        // Normal exit from try block
        break;
    }, |_, _abnormal| {
        DebugUnwind(FatCheckFileAccess as usize);
        DebugTrace(-1, Dbg, b"FatCheckFileAccess -> %08lx\n\0".as_ptr() as *const i8, *p_result as usize);
    });

    // UNREFERENCED_PARAMETER( IrpContext )
    let _ = IrpContext;

    return result;
}


#[no_mangle]
pub unsafe extern "system" fn FatCheckManageVolumeAccess(
    IrpContext: PIRP_CONTEXT,
    AccessState: *mut ntkrnl_sys::ddk::_ACCESS_STATE,
    ProcessorMode: ntkrnl_sys::ddk::KPROCESSOR_MODE,
) -> i32 {
    // PAGED_CODE(); (macro – no-op in free build)

    let mut privilege_set: ntkrnl_sys::ddk::PRIVILEGE_SET = core::mem::zeroed();
    privilege_set.PrivilegeCount = 1;
    privilege_set.Control = ntkrnl_sys::constants::PRIVILEGE_SET_ALL_NECESSARY;
    privilege_set.Privilege[0].Luid = ntkrnl_sys::ddk::RtlConvertLongToLuid(28 /* SE_MANAGE_VOLUME_PRIVILEGE */);
    privilege_set.Privilege[0].Attributes = 0;

    if ntkrnl_sys::ddk::SePrivilegeCheck(
        &mut privilege_set as *mut ntkrnl_sys::ddk::PRIVILEGE_SET as ntkrnl_sys::ddk::PPRIVILEGE_SET,
        core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext) as ntkrnl_sys::ddk::PSECURITY_SUBJECT_CONTEXT,
        ProcessorMode,
    ) != 0 {
        let _ = IrpContext; // UNREFERENCED_PARAMETER(IrpContext)
        return 1; // TRUE
    }

    let _ = IrpContext;
    0 // FALSE
}

#[no_mangle]
pub unsafe extern "system" fn FatExplicitDeviceAccessGranted(
    IrpContext: PIRP_CONTEXT,
    DeviceObject: PDEVICE_OBJECT,
    AccessState: PACCESS_STATE,
    ProcessorMode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let _ = IrpContext;

    let mut status: NTSTATUS = 0;
    let original_access_token: PACCESS_TOKEN;
    let mut restricted_access_token: PACCESS_TOKEN = core::ptr::null_mut();
    let effective_token: *mut PACCESS_TOKEN;

    let mut granted_access: ACCESS_MASK = 0;

    if (*AccessState).PreviouslyGrantedAccess & (SPECIFIC_RIGHTS_ALL ^ FILE_TRAVERSE) != 0 {
        return STATUS_SUCCESS;
    }

    if FatCheckManageVolumeAccess(IrpContext, AccessState, ProcessorMode) != 0 {
        return STATUS_SUCCESS;
    }

    SeLockSubjectContext(core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext));

    effective_token = if !(*AccessState).SubjectSecurityContext.ClientToken.is_null() {
        core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext.ClientToken)
    } else {
        core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext.PrimaryToken)
    };

    original_access_token = *effective_token;
    status = FatCreateRestrictEveryoneToken(
        original_access_token as *mut _,
        core::ptr::addr_of_mut!(restricted_access_token) as *mut *mut _,
    );

    if !NT_SUCCESS(status) {
        SeReleaseSubjectContext(core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext));
        return status;
    }

    *effective_token = restricted_access_token;

    SeAccessCheck(
        (*DeviceObject).SecurityDescriptor,
        core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext),
        0 as BOOLEAN,
        (*AccessState).OriginalDesiredAccess,
        0 as ACCESS_MASK,
        core::ptr::null_mut() as *mut PPRIVILEGE_SET,
        IoGetFileObjectGenericMapping(),
        ProcessorMode,
        core::ptr::addr_of_mut!(granted_access),
        core::ptr::addr_of_mut!(status),
    );

    *effective_token = original_access_token;

    SeUnlockSubjectContext(core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext));
    ObDereferenceObject(restricted_access_token);

    status
}

#[no_mangle]
pub unsafe extern "system" fn FatCreateRestrictEveryoneToken(
    Token: *mut ACCESS_TOKEN,
    RestrictedToken: *mut *mut ACCESS_TOKEN,
) -> i32 {
    #[repr(C)]
    struct SE_EXPORTS {
        SeWorldSid: *mut core::ffi::c_void,
    }
    let mut Status: i32 = 0; // STATUS_SUCCESS
    let mut SidsToDisable: TOKEN_GROUPS = core::mem::zeroed();
    PAGED_CODE();
    *RestrictedToken = core::ptr::null_mut();
    SidsToDisable.GroupCount = 1;
    SidsToDisable.Groups[0].Attributes = 0;
    SidsToDisable.Groups[0].Sid = (*SeExports).SeWorldSid;
    Status = SeFilterToken(
        Token as *mut core::ffi::c_void,
        0,
        core::ptr::addr_of_mut!(SidsToDisable),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        RestrictedToken as *mut *mut core::ffi::c_void,
    );
    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (c) 1989-2000 Microsoft Corporation
//
// Module Name:
//
//     AcChkSup.c
//
// Abstract:
//
//     This module implements the FAT access checking routine
//
//
// --*/
//
// #include "fatprocs.h"
//
// //
// //  Our debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_ACCHKSUP)
//
// NTSTATUS
// FatCreateRestrictEveryoneToken(
//     IN PACCESS_TOKEN Token,
//     OUT PACCESS_TOKEN *RestrictedToken
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatCheckFileAccess)
// #pragma alloc_text(PAGE, FatCheckManageVolumeAccess)
// #pragma alloc_text(PAGE, FatCreateRestrictEveryoneToken)
// #pragma alloc_text(PAGE, FatExplicitDeviceAccessGranted)
// #endif
//
//
// BOOLEAN
// FatCheckFileAccess (
//     PIRP_CONTEXT IrpContext,
//     IN UCHAR DirentAttributes,
//     IN PACCESS_MASK DesiredAccess
//     )
//
// /*++
//
// Routine Description:
//
//     This routine checks if a desired access is allowed to a file represented
//     by the specified DirentAttriubutes.
//
// Arguments:
//
//     DirentAttributes - Supplies the Dirent attributes to check access for
//
//     DesiredAccess - Supplies the desired access mask that we are checking for
//
// Return Value:
//
//     BOOLEAN - TRUE if access is allowed and FALSE otherwise
//
// --*/
//
// {
//     BOOLEAN Result;
//
//     DebugTrace(+1, Dbg, "FatCheckFileAccess\n", 0);
//     DebugTrace( 0, Dbg, "DirentAttributes = %8lx\n", DirentAttributes);
//     DebugTrace( 0, Dbg, "DesiredAccess    = %8lx\n", *DesiredAccess);
//
//     PAGED_CODE();
//
//     //
//     //  This procedures is programmed like a string of filters each
//     //  filter checks to see if some access is allowed,  if it is not allowed
//     //  the filter return FALSE to the user without further checks otherwise
//     //  it moves on to the next filter.  The filter check is to check for
//     //  desired access flags that are not allowed for a particular dirent
//     //
//
//     Result = TRUE;
//
//     _SEH2_TRY {
//
//         //
//         //  Check for Volume ID or Device Dirents, these are not allowed user
//         //  access at all
//         //
//
//         if (FlagOn(DirentAttributes, FAT_DIRENT_ATTR_VOLUME_ID) ||
//             FlagOn(DirentAttributes, FAT_DIRENT_ATTR_DEVICE)) {
//
//             DebugTrace(0, Dbg, "Cannot access volume id or device\n", 0);
//
//             try_return( Result = FALSE );
//         }
//
//         //
//         //  Check the desired access for the object - we only blackball that
//         //  we do not understand.  The model of filesystems using ACLs is that
//         //  they do not type the ACL to the object the ACL is on.  Permissions
//         //  are not checked for consistency vs. the object type - dir/file.
//         //
//
//         if (FlagOn(*DesiredAccess, ~(DELETE |
//                                      READ_CONTROL |
//                                      WRITE_OWNER |
//                                      WRITE_DAC |
//                                      SYNCHRONIZE |
//                                      ACCESS_SYSTEM_SECURITY |
//                                      FILE_WRITE_DATA |
//                                      FILE_READ_EA |
//                                      FILE_WRITE_EA |
//                                      FILE_READ_ATTRIBUTES |
//                                      FILE_WRITE_ATTRIBUTES |
//                                      FILE_LIST_DIRECTORY |
//                                      FILE_TRAVERSE |
//                                      FILE_DELETE_CHILD |
//                                      FILE_APPEND_DATA |
//                                      MAXIMUM_ALLOWED))) {
//
//             DebugTrace(0, Dbg, "Cannot open object\n", 0);
//
//             try_return( Result = FALSE );
//         }
//
//         //
//         //  Check for a read-only Dirent
//         //
//
//         if (FlagOn(DirentAttributes, FAT_DIRENT_ATTR_READ_ONLY)) {
//
//             //
//             //  Check the desired access for a read-only dirent.  AccessMask will contain
//             //  the flags we're going to allow.
//             //
//
//             ACCESS_MASK AccessMask = DELETE | READ_CONTROL | WRITE_OWNER | WRITE_DAC |
//                                     SYNCHRONIZE | ACCESS_SYSTEM_SECURITY | FILE_READ_DATA |
//                                     FILE_READ_EA | FILE_WRITE_EA | FILE_READ_ATTRIBUTES |
//                                     FILE_WRITE_ATTRIBUTES | FILE_EXECUTE | FILE_LIST_DIRECTORY |
//                                     FILE_TRAVERSE;
//
//             //
//             //  If this is a subdirectory also allow add file/directory and delete.
//             //
//
//             if (FlagOn(DirentAttributes, FAT_DIRENT_ATTR_DIRECTORY)) {
//
//                 AccessMask |= FILE_ADD_SUBDIRECTORY | FILE_ADD_FILE | FILE_DELETE_CHILD;
//             }
//
//             if (FlagOn(*DesiredAccess, ~AccessMask)) {
//
//                 DebugTrace(0, Dbg, "Cannot open readonly\n", 0);
//
//                 try_return( Result = FALSE );
//             }
//         }
//
//     try_exit: NOTHING;
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatCheckFileAccess );
//
//         DebugTrace(-1, Dbg, "FatCheckFileAccess -> %08lx\n", Result);
//     } _SEH2_END;
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return Result;
// }
//
//
// BOOLEAN
// FatCheckManageVolumeAccess (
//      PIRP_CONTEXT IrpContext,
//      PACCESS_STATE AccessState,
//      KPROCESSOR_MODE ProcessorMode
//     )
//
// /*++
//
// Routine Description:
//
//     This function checks whether the SID described in the input access state has
//     manage volume privilege.
//
// Arguments:
//
//     AccessState - the access state describing the security context to be checked
//
//     ProcessorMode - the mode this check should occur against
//
// Return Value:
//
//     BOOLEAN - TRUE if privilege is held and FALSE otherwise
//
// --*/
//
// {
//     PRIVILEGE_SET PrivilegeSet;
//
//     PAGED_CODE();
//
//     PrivilegeSet.PrivilegeCount = 1;
//     PrivilegeSet.Control = PRIVILEGE_SET_ALL_NECESSARY;
//     PrivilegeSet.Privilege[0].Luid = RtlConvertLongToLuid( SE_MANAGE_VOLUME_PRIVILEGE );
//     PrivilegeSet.Privilege[0].Attributes = 0;
//
//     if (SePrivilegeCheck( &PrivilegeSet,
//                           &AccessState->SubjectSecurityContext,
//                           ProcessorMode )) {
//
//         return TRUE;
//     }
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return FALSE;
// }
//
//
// NTSTATUS
// FatExplicitDeviceAccessGranted (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PACCESS_STATE AccessState,
//     IN KPROCESSOR_MODE ProcessorMode
//     )
//
// /*++
//
// Routine Description:
//
//     This function asks whether the SID described in the input access state has
//     been granted any explicit access to the given device object.  It does this
//     by acquiring a token stripped of its ability to acquire access via the
//     Everyone SID and re-doing the access check.
//
// Arguments:
//
//     DeviceObject - the device whose ACL will be checked
//
//     AccessState - the access state describing the security context to be checked
//
//     ProcessorMode - the mode this check should occur against
//
// Return Value:
//
//     NTSTATUS - Indicating whether explicit access was granted.
//
// --*/
//
// {
//     NTSTATUS Status;
//
//     PACCESS_TOKEN OriginalAccessToken;
//     PACCESS_TOKEN RestrictedAccessToken;
//
//     PACCESS_TOKEN *EffectiveToken;
//
//     ACCESS_MASK GrantedAccess;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  If the access state indicates that specific access other
//     //  than traverse was acquired, either Everyone does have such
//     //  access or explicit access was granted.  In both cases, we're
//     //  happy to let this proceed.
//     //
//
//     if (AccessState->PreviouslyGrantedAccess & (SPECIFIC_RIGHTS_ALL ^
//                                                 FILE_TRAVERSE)) {
//
//         return STATUS_SUCCESS;
//     }
//
//     //
//     //  If the manage volume privilege is held, this also permits access.
//     //
//
//     if (FatCheckManageVolumeAccess( IrpContext,
//                                     AccessState,
//                                     ProcessorMode )) {
//
//         return STATUS_SUCCESS;
//     }
//
//     //
//     //  Capture the subject context as a prelude to everything below.
//     //
//
//     SeLockSubjectContext( &AccessState->SubjectSecurityContext );
//
//     //
//     //  Convert the token in the subject context into one which does not
//     //  acquire access through the Everyone SID.
//     //
//     //  The logic for deciding which token is effective comes from
//     //  SeQuerySubjectContextToken; since there is no natural way
//     //  of getting a pointer to it, do it by hand.
//     //
//
//     if (ARGUMENT_PRESENT( AccessState->SubjectSecurityContext.ClientToken )) {
//         EffectiveToken = &AccessState->SubjectSecurityContext.ClientToken;
//     } else {
//         EffectiveToken = &AccessState->SubjectSecurityContext.PrimaryToken;
//     }
//
//     OriginalAccessToken = *EffectiveToken;
//     Status = FatCreateRestrictEveryoneToken( OriginalAccessToken, &RestrictedAccessToken );
//
//     if (!NT_SUCCESS(Status)) {
//
//         SeReleaseSubjectContext( &AccessState->SubjectSecurityContext );
//         return Status;
//     }
//
//     //
//     //  Now see if the resulting context has access to the device through
//     //  its explicitly granted access.  We swap in our restricted token
//     //  for this check as the effective client token.
//     //
//
//     *EffectiveToken = RestrictedAccessToken;
//
// #ifdef _MSC_VER
// #pragma prefast( suppress: 28175, "we're a file system, this is ok to touch" )
// #endif
//     SeAccessCheck( DeviceObject->SecurityDescriptor,
//                    &AccessState->SubjectSecurityContext,
//                    FALSE,
//                    AccessState->OriginalDesiredAccess,
//                    0,
//                    NULL,
//                    IoGetFileObjectGenericMapping(),
//                    ProcessorMode,
//                    &GrantedAccess,
//                    &Status );
//
//     *EffectiveToken = OriginalAccessToken;
//
//     //
//     //  Cleanup and return.
//     //
//
//     SeUnlockSubjectContext( &AccessState->SubjectSecurityContext );
//     ObDereferenceObject( RestrictedAccessToken );
//
//     return Status;
// }
//
//
// NTSTATUS
// FatCreateRestrictEveryoneToken (
//     IN PACCESS_TOKEN Token,
//     OUT PACCESS_TOKEN *RestrictedToken
//     )
//
// /*++
//
// Routine Description:
//
//     This function takes a token as the input and returns a new restricted token
//     from which Everyone sid has been disabled.  The resulting token may be used
//     to find out if access is available to a user-sid by explicit means.
//
// Arguments:
//
//     Token - Input token from which Everyone sid needs to be deactivated.
//
//     RestrictedToken - Receives the the new restricted token.
//         This must be released using ObDereferenceObject(*RestrictedToken);
//
// Return Value:
//
//     NTSTATUS - Returned by SeFilterToken.
//
// --*/
//
// {
//     //
//     // Array of sids to disable.
//     //
//
//     TOKEN_GROUPS SidsToDisable;
//
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     //
//     //  Restricted token will contain the original sids with one change:
//     //  If Everyone sid is present in the token, it will be marked for DenyOnly.
//     //
//
//     *RestrictedToken = NULL;
//
//     //
//     //  Put Everyone sid in the array of sids to disable. This will mark it
//     //  for SE_GROUP_USE_FOR_DENY_ONLY and it'll only be applicable for Deny aces.
//     //
//
//     SidsToDisable.GroupCount = 1;
//     SidsToDisable.Groups[0].Attributes = 0;
//     SidsToDisable.Groups[0].Sid = SeExports->SeWorldSid;
//
//     Status = SeFilterToken(
//                  Token,            // Token that needs to be restricted.
//                  0,                // No flags
//                  &SidsToDisable,   // Disable everyone sid
//                  NULL,             // Do not create any restricted sids
//                  NULL,             // Do not delete any privileges
//                  RestrictedToken   // Restricted token
//                  );
//
//     return Status;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
