use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{STATUS_SUCCESS, NT_SUCCESS, TRUE, FALSE};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::PagedPool;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
    fn RtlEqualLuid(luid1: *const LUID, luid2: *const LUID) -> BOOLEAN;
    static mut SeTokenObjectType: PVOID;
}
extern "C" {
    fn SepAcquireTokenLockExclusive(token: PTOKEN);
    fn SepReleaseTokenLock(token: PTOKEN);
}

unsafe fn SepAdjustPrivileges(
    Token: PTOKEN,
    DisableAllPrivileges: BOOLEAN,
    NewState: PLUID_AND_ATTRIBUTES,
    NewStateCount: ULONG,
    PreviousState: PTOKEN_PRIVILEGES,
    ApplyChanges: BOOLEAN,
    ChangedPrivileges: *mut ULONG,
    ChangesMade: PBOOLEAN,
) -> NTSTATUS {
    let mut PrivilegeCount: ULONG = 0;
    let mut ChangeCount: ULONG = 0;
    let mut NewAttributes: ULONG = 0;

    *ChangesMade = 0u8;

    for i in 0..(*Token).PrivilegeCount {
        if DisableAllPrivileges != 0 {
            NewAttributes = (*(*Token).Privileges.add(i as usize)).Attributes & !SE_PRIVILEGE_ENABLED;
        } else {
            let mut j = 0;
            while j < NewStateCount {
                if RtlEqualLuid(
                    core::ptr::addr_of!((*(*Token).Privileges.add(i as usize)).Luid),
                    core::ptr::addr_of!((*NewState.offset(j as isize)).Luid),
                ) != 0
                {
                    NewAttributes = (*NewState.offset(j as isize)).Attributes;
                    NewAttributes &= SE_PRIVILEGE_ENABLED | SE_PRIVILEGE_REMOVED;
                    NewAttributes |= (*(*Token).Privileges.add(i as usize)).Attributes & !SE_PRIVILEGE_ENABLED;
                    break;
                }
                j += 1;
            }

            if j == NewStateCount {
                continue;
            }
        }

        PrivilegeCount += 1;

        if (*(*Token).Privileges.add(i as usize)).Attributes != NewAttributes {
            if !PreviousState.is_null() {
                *(*PreviousState).Privileges.as_mut_ptr().add(ChangeCount as usize) = *(*Token).Privileges.add(i as usize) ;
            }

            if ApplyChanges != 0 {
                if NewAttributes & SE_PRIVILEGE_REMOVED != 0 {
                    (*(*Token).Privileges.add(i as usize)).Attributes &= !SE_PRIVILEGE_ENABLED;
                    SepUpdateSinglePrivilegeFlagToken(Token, i);

                    SepRemovePrivilegeToken(Token, i);

                    *ChangesMade = 1u8;
                    continue;
                }

                (*(*Token).Privileges.add(i as usize)).Attributes = NewAttributes;
                SepUpdateSinglePrivilegeFlagToken(Token, i);
                *ChangesMade = 1u8;
            }

            ChangeCount += 1;
        }
    }

    if !PreviousState.is_null() {
        (*PreviousState).PrivilegeCount = ChangeCount;
    }

    *ChangedPrivileges = ChangeCount;

    if DisableAllPrivileges == 0 && PrivilegeCount < NewStateCount {
        return STATUS_NOT_ALL_ASSIGNED;
    }

    STATUS_SUCCESS
}

unsafe fn SepAdjustGroups(
    Token: PTOKEN,
    NewState: PSID_AND_ATTRIBUTES,
    NewStateCount: ULONG,
    ApplyChanges: BOOLEAN,
    ResetToDefaultStates: BOOLEAN,
    ChangesMade: PBOOLEAN,
    PreviousGroupsState: PTOKEN_GROUPS,
    ChangedGroups: *mut ULONG,
) -> NTSTATUS {
    let mut GroupsCount: ULONG = 0;
    let mut ChangeCount: ULONG = 0;
    let mut NewAttributes: ULONG = 0;

    *ChangesMade = 0u8;

    for GroupsInToken in 0..(*Token).UserAndGroupCount {
        if ResetToDefaultStates != 0 {
            if ((*(*Token).UserAndGroups.add(GroupsInToken as usize)).Attributes & SE_GROUP_ENABLED_BY_DEFAULT != 0) &&
                ((*(*Token).UserAndGroups.add(GroupsInToken as usize)).Attributes & SE_GROUP_ENABLED == 0)
            {
                NewAttributes = (*(*Token).UserAndGroups.add(GroupsInToken as usize)).Attributes | SE_GROUP_ENABLED;
            } else if ((*(*Token).UserAndGroups.add(GroupsInToken as usize)).Attributes & SE_GROUP_ENABLED_BY_DEFAULT == 0) &&
                ((*(*Token).UserAndGroups.add(GroupsInToken as usize)).Attributes & SE_GROUP_ENABLED != 0)
            {
                NewAttributes = (*(*Token).UserAndGroups.add(GroupsInToken as usize)).Attributes & !SE_GROUP_ENABLED;
            } else {
                continue;
            }
        } else {
            let mut GroupsInList = 0;
            while GroupsInList < NewStateCount {
                if RtlEqualSid(
                    ((*(*Token).UserAndGroups.add(GroupsInToken as usize))).Sid,
                    (*NewState.offset(GroupsInList as isize)).Sid,
                ) != 0
                {
                    if ((*(*Token).UserAndGroups.add(GroupsInToken as usize)).Attributes & SE_GROUP_MANDATORY != 0) &&
                        ((*NewState.offset(GroupsInList as isize)).Attributes & SE_GROUP_ENABLED == 0)
                    {
                        return STATUS_CANT_DISABLE_MANDATORY;
                    }

                    if ((*(*Token).UserAndGroups.add(GroupsInToken as usize)).Attributes & SE_GROUP_USE_FOR_DENY_ONLY != 0) &&
                        ((*NewState.offset(GroupsInList as isize)).Attributes & SE_GROUP_ENABLED != 0)
                    {
                        return STATUS_CANT_ENABLE_DENY_ONLY;
                    }

                    NewAttributes = (*NewState.offset(GroupsInList as isize)).Attributes;
                    NewAttributes &= SE_GROUP_ENABLED;
                    NewAttributes |= (*(*Token).UserAndGroups.add(GroupsInToken as usize)).Attributes & !SE_GROUP_ENABLED;
                    break;
                }
                GroupsInList += 1;
            }

            if GroupsInList == NewStateCount {
                continue;
            }

            GroupsCount += 1;
        }

        if (*(*Token).UserAndGroups.add(GroupsInToken as usize)).Attributes != NewAttributes {
            if !PreviousGroupsState.is_null() {
                *(*PreviousGroupsState).Groups.as_mut_ptr().add(ChangeCount as usize) = *(*Token).UserAndGroups.add(GroupsInToken as usize) ;
            }

            if ApplyChanges != 0 {
                (*(*Token).UserAndGroups.add(GroupsInToken as usize)).Attributes = NewAttributes;
                *ChangesMade = 1u8;
            }

            ChangeCount += 1;
        }
    }

    if !PreviousGroupsState.is_null() {
        (*PreviousGroupsState).GroupCount = ChangeCount;
    }

    *ChangedGroups = ChangeCount;

    if ResetToDefaultStates == 0 && GroupsCount < NewStateCount {
        return STATUS_NOT_ALL_ASSIGNED;
    }

    STATUS_SUCCESS
}

/// Runs `adjust` with a PreviousState it can store through. A user-mode caller's
/// buffer is replaced by a pool copy of `required` bytes, which is copied out with
/// the fault-tolerant copy; `required` is the size the dry run measured under the
/// same exclusive lock, so the adjusting pass writes no more than that.
///
/// `Err` is a fault on the caller's buffer, the C's exception code.
unsafe fn with_previous_state<T>(
    previous_state: *mut T,
    required: ULONG,
    previous_mode: KPROCESSOR_MODE,
    adjust: impl FnOnce(*mut T) -> NTSTATUS,
) -> Result<NTSTATUS, NTSTATUS> {
    if previous_state.is_null() || previous_mode == 0 {
        return Ok(adjust(previous_state));
    }
    let tag = u32::from_le_bytes(*b"SePs");
    let kbuf = ExAllocatePoolWithTag(PagedPool, required as SIZE_T, tag);
    if kbuf.is_null() {
        return Err(ntkrnl_sys::base::STATUS_INSUFFICIENT_RESOURCES);
    }
    let _guard = PoolAllocGuard::new(kbuf, tag);
    let status = adjust(kbuf as *mut T);
    safe_copy_raw(previous_state as PVOID, kbuf, required as usize)?;
    Ok(status)
}

#[no_mangle]
pub unsafe extern "system" fn NtAdjustPrivilegesToken(
    TokenHandle: HANDLE,
    DisableAllPrivileges: BOOLEAN,
    NewState: PTOKEN_PRIVILEGES,
    BufferLength: ULONG,
    PreviousState: PTOKEN_PRIVILEGES,
    ReturnLength: *mut ULONG,
) -> NTSTATUS {
    let TokenHandle = opaque_ptr(TokenHandle);
    let ReturnLength = opaque_ptr(ReturnLength);
    let mut Status: NTSTATUS;
    let PreviousMode: KPROCESSOR_MODE;
    let mut Token: PTOKEN = core::ptr::null_mut();
    let mut CapturedPrivileges: PLUID_AND_ATTRIBUTES = core::ptr::null_mut();
    let mut CapturedCount: ULONG = 0;
    let mut CapturedLength: ULONG = 0;
    let mut NewStateSize: ULONG = 0;
    let mut ChangeCount: ULONG = 0;
    let mut RequiredLength: ULONG = 0;
    let mut ChangesMade: BOOLEAN = 0u8;

    if DisableAllPrivileges == 0 && NewState.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    PreviousMode = KeGetPreviousMode();
    if PreviousMode != 0 {
        if DisableAllPrivileges == 0 {
            if let Err(s) = probe_user_buffer(NewState as _, core::mem::size_of::<TOKEN_PRIVILEGES>(), core::mem::size_of::<ULONG>(), PreviousMode) {
                return s;
            }

            CapturedCount = match probe_read_generic(core::ptr::addr_of!((*NewState).PrivilegeCount), 1, PreviousMode) {
                Ok(count) => count,
                Err(s) => return s,
            };
            NewStateSize = core::mem::size_of::<TOKEN_PRIVILEGES>() as ULONG + 
                          (CapturedCount.saturating_sub(1)) * core::mem::size_of::<LUID_AND_ATTRIBUTES>() as ULONG;

            if let Err(s) = probe_user_buffer(NewState as _, NewStateSize as usize, core::mem::size_of::<ULONG>(), PreviousMode) {
                return s;
            }
        }

        if !PreviousState.is_null() {
            if let Err(s) = probe_user_buffer(PreviousState as _, BufferLength as usize, core::mem::size_of::<ULONG>(), PreviousMode) {
                return s;
            }
            if let Err(s) = probe_write_user(ReturnLength, 0u32, PreviousMode) {
                return s;
            }
        }
    } else {
        if DisableAllPrivileges == 0 {
            CapturedCount = (*NewState).PrivilegeCount;
        }
    }

    if DisableAllPrivileges == 0 {
        Status = SeCaptureLuidAndAttributesArray(
            core::ptr::addr_of!((*NewState).Privileges[0]) as PLUID_AND_ATTRIBUTES,
            CapturedCount,
            PreviousMode,
            core::ptr::null_mut(),
            0,
            PagedPool,
            1u8,
            &mut CapturedPrivileges,
            &mut CapturedLength,
        );

        if Status < 0 {
            return Status;
        }
    }

    Status = ObReferenceObjectByHandle(
        TokenHandle,
        TOKEN_ADJUST_PRIVILEGES | (if !PreviousState.is_null() { TOKEN_QUERY } else { 0 }),
        SeTokenObjectType as *mut _OBJECT_TYPE,
        PreviousMode,
        core::ptr::addr_of_mut!(Token) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        if !CapturedPrivileges.is_null() {
            SeReleaseLuidAndAttributesArray(CapturedPrivileges, PreviousMode, 1u8);
        }

        return Status;
    }

    let _token_guard = ObjRefGuard::new(Token as PVOID);

    SepAcquireTokenLockExclusive(Token);

    Status = SepAdjustPrivileges(
        Token,
        DisableAllPrivileges,
        CapturedPrivileges,
        CapturedCount,
        core::ptr::null_mut(),
        0u8,
        &mut ChangeCount,
        &mut ChangesMade,
    );

    // C: everything below runs to a Cleanup label that regenerates ModifiedId if
    // anything changed, drops the lock, DEREFERENCES the token and releases the
    // capture. The previous translation disarmed the reference guard instead, so
    // every call leaked a token reference.
    'cleanup: {
        if !PreviousState.is_null() {
            RequiredLength = core::mem::size_of::<TOKEN_PRIVILEGES>() as ULONG +
                            (ChangeCount.saturating_sub(1)) * core::mem::size_of::<LUID_AND_ATTRIBUTES>() as ULONG;

            if let Err(s) = probe_write_user(ReturnLength, RequiredLength, PreviousMode) {
                Status = s;
                break 'cleanup;
            }

            if BufferLength < RequiredLength {
                Status = STATUS_BUFFER_TOO_SMALL;
                break 'cleanup;
            }
        }

        // The C runs this under _SEH2_TRY because it stores into PreviousState, a
        // user buffer; a fault there forces ChangesMade so the ModifiedId moves.
        Status = match with_previous_state(PreviousState, RequiredLength, PreviousMode, |ps| {
            SepAdjustPrivileges(
                Token,
                DisableAllPrivileges,
                CapturedPrivileges,
                CapturedCount,
                ps,
                1u8,
                &mut ChangeCount,
                &mut ChangesMade,
            )
        }) {
            Ok(status) => status,
            Err(status) => {
                ChangesMade = 1u8;
                status
            }
        };
    }

    if ChangesMade != 0 {
        ExAllocateLocallyUniqueId(core::ptr::addr_of_mut!((*Token).ModifiedId));
    }

    SepReleaseTokenLock(Token);
    drop(_token_guard);

    if !CapturedPrivileges.is_null() {
        SeReleaseLuidAndAttributesArray(CapturedPrivileges, PreviousMode, 1u8);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtAdjustGroupsToken(
    TokenHandle: HANDLE,
    ResetToDefault: BOOLEAN,
    NewState: PTOKEN_GROUPS,
    BufferLength: ULONG,
    PreviousState: PTOKEN_GROUPS,
    ReturnLength: *mut ULONG,
) -> NTSTATUS {
    let TokenHandle = opaque_ptr(TokenHandle);
    let ReturnLength = opaque_ptr(ReturnLength);
    let mut Token: PTOKEN = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let PreviousMode: KPROCESSOR_MODE;
    let mut ChangeCount: ULONG = 0;
    let mut RequiredLength: ULONG = 0;
    let mut CapturedCount: ULONG = 0;
    let mut CapturedLength: ULONG = 0;
    let mut NewStateSize: ULONG = 0;
    let mut CapturedGroups: PSID_AND_ATTRIBUTES = core::ptr::null_mut();
    let mut ChangesMade: BOOLEAN = 0u8;

    if ResetToDefault == 0 && NewState.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    PreviousMode = ExGetPreviousMode();

    if PreviousMode != 0 {
        if ResetToDefault == 0 {
            if let Err(s) = probe_user_buffer(NewState as _, core::mem::size_of::<TOKEN_GROUPS>(), core::mem::size_of::<ULONG>(), PreviousMode) {
                return s;
            }

            CapturedCount = match probe_read_generic(core::ptr::addr_of!((*NewState).GroupCount), 1, PreviousMode) {
                Ok(count) => count,
                Err(s) => return s,
            };
            NewStateSize = core::mem::size_of::<TOKEN_GROUPS>() as ULONG + 
                          (CapturedCount.saturating_sub(1)) * core::mem::size_of::<SID_AND_ATTRIBUTES>() as ULONG;

            if let Err(s) = probe_user_buffer(NewState as _, NewStateSize as usize, core::mem::size_of::<ULONG>(), PreviousMode) {
                return s;
            }
        }

        if !PreviousState.is_null() {
            if let Err(s) = probe_user_buffer(PreviousState as _, BufferLength as usize, core::mem::size_of::<ULONG>(), PreviousMode) {
                return s;
            }
            if let Err(s) = probe_write_user(ReturnLength, 0u32, PreviousMode) {
                return s;
            }
        }
    } else {
        if ResetToDefault == 0 {
            CapturedCount = (*NewState).GroupCount;
        }
    }

    if ResetToDefault == 0 {
        Status = SeCaptureSidAndAttributesArray(
            core::ptr::addr_of!((*NewState).Groups[0]) as PSID_AND_ATTRIBUTES,
            CapturedCount,
            PreviousMode,
            core::ptr::null_mut(),
            0,
            PagedPool,
            1u8,
            &mut CapturedGroups,
            &mut CapturedLength,
        );

        if Status < 0 {
            return Status;
        }
    }

    Status = ObReferenceObjectByHandle(
        TokenHandle,
        TOKEN_ADJUST_GROUPS | (if !PreviousState.is_null() { TOKEN_QUERY } else { 0 }),
        SeTokenObjectType as *mut _OBJECT_TYPE,
        PreviousMode,
        core::ptr::addr_of_mut!(Token) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        if !CapturedGroups.is_null() {
            SeReleaseSidAndAttributesArray(CapturedGroups, PreviousMode, 1u8);
        }

        return Status;
    }

    let _token_guard = ObjRefGuard::new(Token as PVOID);

    SepAcquireTokenLockExclusive(Token);

    Status = SepAdjustGroups(
        Token,
        CapturedGroups,
        CapturedCount,
        0u8,
        ResetToDefault,
        &mut ChangesMade,
        core::ptr::null_mut(),
        &mut ChangeCount,
    );

    // C: everything below runs to a Cleanup label that regenerates ModifiedId if
    // anything changed, drops the lock, DEREFERENCES the token and releases the
    // capture. The previous translation disarmed the reference guard instead, so
    // every call leaked a token reference.
    'cleanup: {
        if !PreviousState.is_null() {
            RequiredLength = core::mem::size_of::<TOKEN_GROUPS>() as ULONG +
                            (ChangeCount.saturating_sub(1)) * core::mem::size_of::<SID_AND_ATTRIBUTES>() as ULONG;

            if let Err(s) = probe_write_user(ReturnLength, RequiredLength, PreviousMode) {
                Status = s;
                break 'cleanup;
            }

            if BufferLength < RequiredLength {
                Status = STATUS_BUFFER_TOO_SMALL;
                break 'cleanup;
            }
        }

        // The C runs this under _SEH2_TRY because it stores into PreviousState, a
        // user buffer; a fault there forces ChangesMade so the ModifiedId moves.
        Status = match with_previous_state(PreviousState, RequiredLength, PreviousMode, |ps| {
            SepAdjustGroups(
                Token,
                CapturedGroups,
                CapturedCount,
                1u8,
                ResetToDefault,
                &mut ChangesMade,
                ps,
                &mut ChangeCount,
            )
        }) {
            Ok(status) => status,
            Err(status) => {
                ChangesMade = 1u8;
                status
            }
        };
    }

    if ChangesMade != 0 {
        ExAllocateLocallyUniqueId(core::ptr::addr_of_mut!((*Token).ModifiedId));
    }

    SepReleaseTokenLock(Token);
    drop(_token_guard);

    if !CapturedGroups.is_null() {
        SeReleaseSidAndAttributesArray(CapturedGroups, PreviousMode, 1u8);
    }

    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Access token ajusting Groups/Privileges support routines
//  * COPYRIGHT:   Copyright David Welch <welch@cwcom.net>
//  *              Copyright 2021-2022 George Bișoc <george.bisoc@reactos.org>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// /**
//  * @brief
//  * Removes a certain amount of privileges of a token based upon the request
//  * by the caller.
//  *
//  * @param[in,out] Token
//  * Token handle where the privileges are about to be modified.
//  *
//  * @param[in] DisableAllPrivileges
//  * If set to TRUE, the function disables all the privileges.
//  *
//  * @param[in] NewState
//  * A new list of privileges that the function will use it accordingly to
//  * either disable or enable the said privileges and change them.
//  *
//  * @param[in] NewStateCount
//  * The new total number count of privileges.
//  *
//  * @param[out] PreviousState
//  * If specified, the function will return the previous state list of privileges.
//  *
//  * @param[in] ApplyChanges
//  * If set to TRUE, the function will immediatelly apply the changes onto the
//  * token's privileges.
//  *
//  * @param[out] ChangedPrivileges
//  * The returned count number of changed privileges.
//  *
//  * @param[out] ChangesMade
//  * If TRUE, the function has made changes to the token's privileges. FALSE
//  * otherwise.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the function has successfully changed the list
//  * of privileges. STATUS_NOT_ALL_ASSIGNED is returned if not every privilege
//  * has been changed.
//  */
// static
// NTSTATUS
// SepAdjustPrivileges(
//     _Inout_ PTOKEN Token,
//     _In_ BOOLEAN DisableAllPrivileges,
//     _In_opt_ PLUID_AND_ATTRIBUTES NewState,
//     _In_ ULONG NewStateCount,
//     _Out_opt_ PTOKEN_PRIVILEGES PreviousState,
//     _In_ BOOLEAN ApplyChanges,
//     _Out_ PULONG ChangedPrivileges,
//     _Out_ PBOOLEAN ChangesMade)
// {
//     ULONG i, j, PrivilegeCount, ChangeCount, NewAttributes;
//
//     PAGED_CODE();
//
//     /* Count the found privileges and those that need to be changed */
//     PrivilegeCount = 0;
//     ChangeCount = 0;
//     *ChangesMade = FALSE;
//
//     /* Loop all privileges in the token */
//     for (i = 0; i < Token->PrivilegeCount; i++)
//     {
//         /* Shall all of them be disabled? */
//         if (DisableAllPrivileges)
//         {
//             /* The new attributes are the old ones, but disabled */
//             NewAttributes = Token->Privileges[i].Attributes & ~SE_PRIVILEGE_ENABLED;
//         }
//         else
//         {
//             /* Otherwise loop all provided privileges */
//             for (j = 0; j < NewStateCount; j++)
//             {
//                 /* Check if this is the LUID we are looking for */
//                 if (RtlEqualLuid(&Token->Privileges[i].Luid, &NewState[j].Luid))
//                 {
//                     DPRINT("Found privilege\n");
//
//                     /* Copy SE_PRIVILEGE_ENABLED | SE_PRIVILEGE_REMOVED */
//                     NewAttributes = NewState[j].Attributes;
//                     NewAttributes &= (SE_PRIVILEGE_ENABLED | SE_PRIVILEGE_REMOVED);
//                     NewAttributes |= Token->Privileges[i].Attributes & ~SE_PRIVILEGE_ENABLED;
//
//                     /* Stop looking */
//                     break;
//                 }
//             }
//
//             /* Check if we didn't find the privilege */
//             if (j == NewStateCount)
//             {
//                 /* Continue with the token's next privilege */
//                 continue;
//             }
//         }
//
//         /* We found a privilege, count it */
//         PrivilegeCount++;
//
//         /* Does the privilege need to be changed? */
//         if (Token->Privileges[i].Attributes != NewAttributes)
//         {
//             /* Does the caller want the old privileges? */
//             if (PreviousState != NULL)
//             {
//                 /* Copy the old privilege */
//                 PreviousState->Privileges[ChangeCount] = Token->Privileges[i];
//             }
//
//             /* Does the caller want to apply the changes? */
//             if (ApplyChanges)
//             {
//                 /* Shall we remove the privilege? */
//                 if (NewAttributes & SE_PRIVILEGE_REMOVED)
//                 {
//                     /* Set the token as disabled and update flags for it */
//                     Token->Privileges[i].Attributes &= ~SE_PRIVILEGE_ENABLED;
//                     SepUpdateSinglePrivilegeFlagToken(Token, i);
//
//                     /* Remove the privilege */
//                     SepRemovePrivilegeToken(Token, i);
//
//                     *ChangesMade = TRUE;
//
//                     /* Fix the running index and continue with next one */
//                     i--;
//                     continue;
//                 }
//
//                 /* Set the new attributes and update flags */
//                 Token->Privileges[i].Attributes = NewAttributes;
//                 SepUpdateSinglePrivilegeFlagToken(Token, i);
//                 *ChangesMade = TRUE;
//             }
//
//             /* Increment the change count */
//             ChangeCount++;
//         }
//     }
//
//     /* Set the number of saved privileges */
//     if (PreviousState != NULL)
//         PreviousState->PrivilegeCount = ChangeCount;
//
//     /* Return the number of changed privileges */
//     *ChangedPrivileges = ChangeCount;
//
//     /* Check if we missed some */
//     if (!DisableAllPrivileges && (PrivilegeCount < NewStateCount))
//     {
//         return STATUS_NOT_ALL_ASSIGNED;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Private routine that iterates over the groups of an
//  * access token to be adjusted as per on request by the
//  * caller, where a group can be enabled or disabled.
//  *
//  * @param[in] Token
//  * Access token where its groups are to be enabled or disabled.
//  *
//  * @param[in] NewState
//  * A list of groups with new state attributes to be assigned to
//  * the token.
//  *
//  * @param[in] NewStateCount
//  * The captured count number of groups in the list.
//  *
//  * @param[in] ApplyChanges
//  * If set to FALSE, the function will only iterate over the token's
//  * groups without performing any kind of modification. If set to TRUE,
//  * the changes will be applied immediately when the function has done
//  * looping the groups.
//  *
//  * @param[in] ResetToDefaultStates
//  * The function will reset the groups in an access token to default
//  * states if set to TRUE. In such scenario the function ignores
//  * NewState outright. Otherwise if set to FALSE, the function will
//  * use NewState to assign the newly attributes to adjust the token's
//  * groups. SE_GROUP_ENABLED_BY_DEFAULT is a flag indicator that is used
//  * for such purpose.
//  *
//  * @param[out] ChangesMade
//  * Returns TRUE if changes to token's groups have been made, otherwise
//  * FALSE is returned. Bear in mind such changes aren't always deterministic.
//  * See remarks for further details.
//  *
//  * @param[out] PreviousGroupsState
//  * If requested by the caller, the function will return the previous state
//  * of groups in an access token prior taking action on adjusting the token.
//  * This is a UM (user mode) pointer and it's prone to raise exceptions
//  * if such pointer address is not valid.
//  *
//  * @param[out] ChangedGroups
//  * Returns the total number of changed groups in an access token. This
//  * argument could also indicate the number of groups to be changed if
//  * the calling thread hasn't chosen to apply the changes yet. A number
//  * of 0 indicates no groups have been or to be changed because the groups'
//  * attributes in a token are the same as the ones from NewState given by
//  * the caller.
//  *
//  * @return
//  * STATUS_SUCCESS is returned if the function has successfully completed
//  * the operation of adjusting groups in a token. STATUS_CANT_DISABLE_MANDATORY
//  * is returned if there was an attempt to disable a mandatory group which is
//  * not possible. STATUS_CANT_ENABLE_DENY_ONLY is returned if there was an attempt
//  * to enable a "use for Deny only" group which is not allowed, that is, a restricted
//  * group. STATUS_NOT_ALL_ASSIGNED is returned if not all the groups are actually
//  * assigned to the token.
//  *
//  * @remarks
//  * Token groups adjusting can be judged to be deterministic or not based on the
//  * NT status code value. That is, STATUS_SUCCESS indicates the function not only
//  * has iterated over the whole groups in a token, it also has applied the changes
//  * thoroughly without impediment and the results perfectly match with the request
//  * desired by the caller. In this situation the condition is deemed deterministic.
//  * In a different situation however, if the status code was STATUS_NOT_ALL_ASSIGNED,
//  * the function would still continue looping the groups in a token and apply the
//  * changes whenever possible where the respective groups actually exist in the
//  * token. This kind of situation is deemed as indeterministic.
//  * For STATUS_CANT_DISABLE_MANDATORY and STATUS_CANT_ENABLE_DENY_ONLY the scenario
//  * is even more indeterministic as the iteration of groups comes to a halt thus
//  * leaving all other possible groups to be adjusted.
//  */
// static
// NTSTATUS
// SepAdjustGroups(
//     _In_ PTOKEN Token,
//     _In_opt_ PSID_AND_ATTRIBUTES NewState,
//     _In_ ULONG NewStateCount,
//     _In_ BOOLEAN ApplyChanges,
//     _In_ BOOLEAN ResetToDefaultStates,
//     _Out_ PBOOLEAN ChangesMade,
//     _Out_opt_ PTOKEN_GROUPS PreviousGroupsState,
//     _Out_ PULONG ChangedGroups)
// {
//     ULONG GroupsInToken, GroupsInList;
//     ULONG ChangeCount, GroupsCount, NewAttributes;
//
//     PAGED_CODE();
//
//     /* Ensure that the token we get is valid */
//     ASSERT(Token);
//
//     /* Initialize the counters and begin the work */
//     *ChangesMade = FALSE;
//     GroupsCount = 0;
//     ChangeCount = 0;
//
//     /* Begin looping all the groups in the token */
//     for (GroupsInToken = 0; GroupsInToken < Token->UserAndGroupCount; GroupsInToken++)
//     {
//         /* Does the caller want to reset groups to default states? */
//         if (ResetToDefaultStates)
//         {
//             /*
//              * SE_GROUP_ENABLED_BY_DEFAULT is a special indicator that informs us
//              * if a certain group has been enabled by default or not. In case
//              * a group is enabled by default but it is not currently enabled then
//              * at that point we must enable it back by default. For now just
//              * assign the respective SE_GROUP_ENABLED attribute as we'll do the
//              * eventual work later.
//              */
//             if ((Token->UserAndGroups[GroupsInToken].Attributes & SE_GROUP_ENABLED_BY_DEFAULT) &&
//                 (Token->UserAndGroups[GroupsInToken].Attributes & SE_GROUP_ENABLED) == 0)
//             {
//                 NewAttributes = Token->UserAndGroups[GroupsInToken].Attributes |= SE_GROUP_ENABLED;
//             }
//
//             /*
//              * Unlike the case above, a group that hasn't been enabled by
//              * default but it's currently enabled then we must disable
//              * it back.
//              */
//             if ((Token->UserAndGroups[GroupsInToken].Attributes & SE_GROUP_ENABLED_BY_DEFAULT) == 0 &&
//                 (Token->UserAndGroups[GroupsInToken].Attributes & SE_GROUP_ENABLED))
//             {
//                 NewAttributes = Token->UserAndGroups[GroupsInToken].Attributes & ~SE_GROUP_ENABLED;
//             }
//         }
//         else
//         {
//             /* Loop the provided groups in the list then */
//             for (GroupsInList = 0; GroupsInList < NewStateCount; GroupsInList++)
//             {
//                 /* Does this group exist in the token? */
//                 if (RtlEqualSid(&Token->UserAndGroups[GroupsInToken].Sid,
//                                 &NewState[GroupsInList].Sid))
//                 {
//                     /*
//                      * This is the group that we're looking for.
//                      * However, it could be that the group is a
//                      * mandatory group which we are not allowed
//                      * and cannot disable it.
//                      */
//                     if ((Token->UserAndGroups[GroupsInToken].Attributes & SE_GROUP_MANDATORY) &&
//                         (NewState[GroupsInList].Attributes & SE_GROUP_ENABLED) == 0)
//                     {
//                         /* It is mandatory, forget about this group */
//                         DPRINT1("SepAdjustGroups(): The SID group is mandatory!\n");
//                         return STATUS_CANT_DISABLE_MANDATORY;
//                     }
//
//                     /*
//                      * We've to ensure that apart the group mustn't be
//                      * mandatory, it mustn't be a restricted group as
//                      * well. That is, the group is marked with
//                      * SE_GROUP_USE_FOR_DENY_ONLY flag and no one
//                      * can enable it because it's for "deny" use only.
//                      */
//                     if ((Token->UserAndGroups[GroupsInToken].Attributes & SE_GROUP_USE_FOR_DENY_ONLY) &&
//                         (NewState[GroupsInList].Attributes & SE_GROUP_ENABLED))
//                     {
//                         /* This group is restricted, forget about it */
//                         DPRINT1("SepAdjustGroups(): The SID group is for use deny only!\n");
//                         return STATUS_CANT_ENABLE_DENY_ONLY;
//                     }
//
//                     /* Copy the attributes and stop searching */
//                     NewAttributes = NewState[GroupsInList].Attributes;
//                     NewAttributes &= SE_GROUP_ENABLED;
//                     NewAttributes = Token->UserAndGroups[GroupsInToken].Attributes & ~SE_GROUP_ENABLED;
//                     break;
//                 }
//
//                 /* Did we find the specific group we wanted? */
//                 if (GroupsInList == NewStateCount)
//                 {
//                     /* We didn't, continue with the next token's group */
//                     continue;
//                 }
//             }
//
//             /* Count the group that we found it */
//             GroupsCount++;
//
//             /* Does the token have the same attributes as the caller requested them? */
//             if (Token->UserAndGroups[GroupsInToken].Attributes != NewAttributes)
//             {
//                 /*
//                  * No, then it's time to make some adjustment to the
//                  * token's groups. Does the caller want the previous states
//                  * of groups?
//                  */
//                 if (PreviousGroupsState != NULL)
//                 {
//                     PreviousGroupsState->Groups[ChangeCount] = Token->UserAndGroups[GroupsInToken];
//                 }
//
//                 /* Time to apply the changes now? */
//                 if (ApplyChanges)
//                 {
//                     /* The caller gave us consent, apply and report that we made changes! */
//                     Token->UserAndGroups[GroupsInToken].Attributes = NewAttributes;
//                     *ChangesMade = TRUE;
//                 }
//
//                 /* Increment the count change */
//                 ChangeCount++;
//             }
//         }
//     }
//
//     /* Report the number of previous saved groups */
//     if (PreviousGroupsState != NULL)
//     {
//         PreviousGroupsState->GroupCount = ChangeCount;
//     }
//
//     /* Report the number of changed groups */
//     *ChangedGroups = ChangeCount;
//
//     /* Did we miss some groups? */
//     if (!ResetToDefaultStates && (GroupsCount < NewStateCount))
//     {
//         /*
//          * If we're at this stage then we are in a situation
//          * where the adjust changes done to token's groups is
//          * not deterministic as the caller might have wanted
//          * as per NewState parameter.
//          */
//         DPRINT1("SepAdjustGroups(): The token hasn't all the groups assigned!\n");
//         return STATUS_NOT_ALL_ASSIGNED;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /* SYSTEM CALLS ***************************************************************/
//
// /**
//  * @brief
//  * Removes a certain amount of privileges of a token based upon the request
//  * by the caller.
//  *
//  * @param[in,out] Token
//  * Token handle where the privileges are about to be modified.
//  *
//  * @param[in] DisableAllPrivileges
//  * If set to TRUE, the function disables all the privileges.
//  *
//  * @param[in] NewState
//  * A new list of privileges that the function will use it accordingly to
//  * either disable or enable the said privileges and change them.
//  *
//  * @param[in] NewStateCount
//  * The new total number count of privileges.
//  *
//  * @param[out] PreviousState
//  * If specified, the function will return the previous state list of privileges.
//  *
//  * @param[in] ApplyChanges
//  * If set to TRUE, the function will immediatelly apply the changes onto the
//  * token's privileges.
//  *
//  * @param[out] ChangedPrivileges
//  * The returned count number of changed privileges.
//  *
//  * @param[out] ChangesMade
//  * If TRUE, the function has made changes to the token's privileges. FALSE
//  * otherwise.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the function has successfully changed the list
//  * of privileges. STATUS_NOT_ALL_ASSIGNED is returned if not every privilege
//  * has been changed.
//  */
// _Must_inspect_result_
// __kernel_entry
// NTSTATUS
// NTAPI
// NtAdjustPrivilegesToken(
//     _In_ HANDLE TokenHandle,
//     _In_ BOOLEAN DisableAllPrivileges,
//     _In_opt_ PTOKEN_PRIVILEGES NewState,
//     _In_ ULONG BufferLength,
//     _Out_writes_bytes_to_opt_(BufferLength,*ReturnLength)
//         PTOKEN_PRIVILEGES PreviousState,
//     _When_(PreviousState!=NULL, _Out_) PULONG ReturnLength)
// {
//     NTSTATUS Status;
//     KPROCESSOR_MODE PreviousMode;
//     PTOKEN Token;
//     PLUID_AND_ATTRIBUTES CapturedPrivileges = NULL;
//     ULONG CapturedCount = 0;
//     ULONG CapturedLength = 0;
//     ULONG NewStateSize = 0;
//     ULONG ChangeCount;
//     ULONG RequiredLength;
//     BOOLEAN ChangesMade = FALSE;
//
//     PAGED_CODE();
//
//     DPRINT("NtAdjustPrivilegesToken() called\n");
//
//     /* Fail, if we do not disable all privileges but NewState is NULL */
//     if (DisableAllPrivileges == FALSE && NewState == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     PreviousMode = KeGetPreviousMode();
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             /* Probe NewState */
//             if (DisableAllPrivileges == FALSE)
//             {
//                 /* First probe the header */
//                 ProbeForRead(NewState, sizeof(TOKEN_PRIVILEGES), sizeof(ULONG));
//
//                 CapturedCount = NewState->PrivilegeCount;
//                 NewStateSize = FIELD_OFFSET(TOKEN_PRIVILEGES, Privileges[CapturedCount]);
//
//                 ProbeForRead(NewState, NewStateSize, sizeof(ULONG));
//             }
//
//             /* Probe PreviousState and ReturnLength */
//             if (PreviousState != NULL)
//             {
//                 ProbeForWrite(PreviousState, BufferLength, sizeof(ULONG));
//                 ProbeForWrite(ReturnLength, sizeof(ULONG), sizeof(ULONG));
//             }
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         /* This is kernel mode, we trust the caller */
//         if (DisableAllPrivileges == FALSE)
//             CapturedCount = NewState->PrivilegeCount;
//     }
//
//     /* Do we need to capture the new state? */
//     if (DisableAllPrivileges == FALSE)
//     {
//         _SEH2_TRY
//         {
//             /* Capture the new state array of privileges */
//             Status = SeCaptureLuidAndAttributesArray(NewState->Privileges,
//                                                      CapturedCount,
//                                                      PreviousMode,
//                                                      NULL,
//                                                      0,
//                                                      PagedPool,
//                                                      TRUE,
//                                                      &CapturedPrivileges,
//                                                      &CapturedLength);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//
//         if (!NT_SUCCESS(Status))
//             return Status;
//     }
//
//     /* Reference the token */
//     Status = ObReferenceObjectByHandle(TokenHandle,
//                                        TOKEN_ADJUST_PRIVILEGES | (PreviousState != NULL ? TOKEN_QUERY : 0),
//                                        SeTokenObjectType as *mut _OBJECT_TYPE,
//                                        PreviousMode,
//                                        (PVOID*)&Token,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to reference token (Status 0x%lx)\n", Status);
//
//         /* Release the captured privileges */
//         if (CapturedPrivileges != NULL)
//         {
//             SeReleaseLuidAndAttributesArray(CapturedPrivileges,
//                                             PreviousMode,
//                                             TRUE);
//         }
//
//         return Status;
//     }
//
//     /* Lock the token */
//     SepAcquireTokenLockExclusive(Token);
//
//     /* Count the privileges that need to be changed, do not apply them yet */
//     Status = SepAdjustPrivileges(Token,
//                                  DisableAllPrivileges,
//                                  CapturedPrivileges,
//                                  CapturedCount,
//                                  NULL,
//                                  FALSE,
//                                  &ChangeCount,
//                                  &ChangesMade);
//
//     /* Check if the caller asked for the previous state */
//     if (PreviousState != NULL)
//     {
//         /* Calculate the required length */
//         RequiredLength = FIELD_OFFSET(TOKEN_PRIVILEGES, Privileges[ChangeCount]);
//
//         /* Try to return the required buffer length */
//         _SEH2_TRY
//         {
//             *ReturnLength = RequiredLength;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Do cleanup and return the exception code */
//             Status = _SEH2_GetExceptionCode();
//             _SEH2_YIELD(goto Cleanup);
//         }
//         _SEH2_END;
//
//         /* Fail, if the buffer length is smaller than the required length */
//         if (BufferLength < RequiredLength)
//         {
//             Status = STATUS_BUFFER_TOO_SMALL;
//             goto Cleanup;
//         }
//     }
//
//     /* Now enter SEH, since we might return the old privileges */
//     _SEH2_TRY
//     {
//         /* This time apply the changes */
//         Status = SepAdjustPrivileges(Token,
//                                      DisableAllPrivileges,
//                                      CapturedPrivileges,
//                                      CapturedCount,
//                                      PreviousState,
//                                      TRUE,
//                                      &ChangeCount,
//                                      &ChangesMade);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Do cleanup and return the exception code */
//         Status = _SEH2_GetExceptionCode();
//         ChangesMade = TRUE; // Force write.
//         _SEH2_YIELD(goto Cleanup);
//     }
//     _SEH2_END;
//
// Cleanup:
//     /* Touch the token if we made changes */
//     if (ChangesMade)
//         ExAllocateLocallyUniqueId(&Token->ModifiedId);
//
//     /* Unlock and dereference the token */
//     SepReleaseTokenLock(Token);
//     ObDereferenceObject(Token);
//
//     /* Release the captured privileges */
//     if (CapturedPrivileges != NULL)
//     {
//         SeReleaseLuidAndAttributesArray(CapturedPrivileges,
//                                         PreviousMode,
//                                         TRUE);
//     }
//
//     DPRINT("NtAdjustPrivilegesToken() done\n");
//     return Status;
// }
//
// /**
//  * @brief
//  * Changes the list of groups by enabling or disabling them
//  * in an access token. Unlike NtAdjustPrivilegesToken,
//  * this API routine does not remove groups.
//  *
//  * @param[in] TokenHandle
//  * Token handle where the list of groups SID are to be adjusted.
//  * The access token must have TOKEN_ADJUST_GROUPS access right
//  * in order to change the groups in a token. The token must also
//  * have TOKEN_QUERY access right if the caller requests the previous
//  * states of groups list, that is, PreviousState is not NULL.
//  *
//  * @param[in] ResetToDefault
//  * If set to TRUE, the function resets the list of groups to default
//  * enabled and disabled states. NewState is ignored in this case.
//  * Otherwise if the parameter is set to FALSE, the function expects
//  * a new list of groups from NewState to be adjusted within the token.
//  *
//  * @param[in] NewState
//  * A new list of groups SID that the function will use it accordingly to
//  * modify the current list of groups SID of a token.
//  *
//  * @param[in] BufferLength
//  * The length size of the buffer that is pointed by the NewState parameter
//  * argument, in bytes.
//  *
//  * @param[out] PreviousState
//  * If specified, the function will return to the caller the old list of groups
//  * SID. If this parameter is NULL, ReturnLength must also be NULL.
//  *
//  * @param[out] ReturnLength
//  * If specified, the function will return the total size length of the old list
//  * of groups SIDs, in bytes.
//  *
//  * @return
//  * STATUS_SUCCESS is returned if the function has successfully adjusted the
//  * token's groups. STATUS_INVALID_PARAMETER is returned if the caller has
//  * submitted one or more invalid parameters, that is, the caller didn't want
//  * to reset the groups to default state but no NewState argument list has been
//  * provided. STATUS_BUFFER_TOO_SMALL is returned if the buffer length given
//  * by the caller is smaller than the required length size. A failure NTSTATUS
//  * code is returned otherwise.
//  */
// NTSTATUS
// NTAPI
// NtAdjustGroupsToken(
//     _In_ HANDLE TokenHandle,
//     _In_ BOOLEAN ResetToDefault,
//     _In_ PTOKEN_GROUPS NewState,
//     _In_ ULONG BufferLength,
//     _Out_writes_bytes_to_opt_(BufferLength, *ReturnLength)
//     PTOKEN_GROUPS PreviousState,
//     _When_(PreviousState != NULL, _Out_) PULONG ReturnLength)
// {
//     PTOKEN Token;
//     NTSTATUS Status;
//     KPROCESSOR_MODE PreviousMode;
//     ULONG ChangeCount, RequiredLength;
//     ULONG CapturedCount = 0;
//     ULONG CapturedLength = 0;
//     ULONG NewStateSize = 0;
//     PSID_AND_ATTRIBUTES CapturedGroups = NULL;
//     BOOLEAN ChangesMade = FALSE;
//
//     PAGED_CODE();
//
//     /*
//      * If the caller doesn't want to reset the groups of an
//      * access token to default states then at least we must
//      * expect a list of groups to be adjusted based on NewState
//      * parameter. Otherwise bail out because the caller has
//      * no idea what they're doing.
//      */
//     if (!ResetToDefault && !NewState)
//     {
//         DPRINT1("NtAdjustGroupsToken(): The caller hasn't provided any list of groups to adjust!\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     PreviousMode = ExGetPreviousMode();
//
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             /* Probe NewState */
//             if (!ResetToDefault)
//             {
//                 /* Probe the header */
//                 ProbeForRead(NewState, sizeof(*NewState), sizeof(ULONG));
//
//                 CapturedCount = NewState->GroupCount;
//                 NewStateSize = FIELD_OFFSET(TOKEN_GROUPS, Groups[CapturedCount]);
//
//                 ProbeForRead(NewState, NewStateSize, sizeof(ULONG));
//             }
//
//             if (PreviousState != NULL)
//             {
//                 ProbeForWrite(PreviousState, BufferLength, sizeof(ULONG));
//                 ProbeForWrite(ReturnLength, sizeof(*ReturnLength), sizeof(ULONG));
//             }
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         /*
//          * We're calling directly from the kernel, just retrieve
//          * the number count of captured groups outright.
//          */
//         if (!ResetToDefault)
//         {
//             CapturedCount = NewState->GroupCount;
//         }
//     }
//
//     /* Time to capture the NewState list */
//     if (!ResetToDefault)
//     {
//         _SEH2_TRY
//         {
//             Status = SeCaptureSidAndAttributesArray(NewState->Groups,
//                                                     CapturedCount,
//                                                     PreviousMode,
//                                                     NULL,
//                                                     0,
//                                                     PagedPool,
//                                                     TRUE,
//                                                     &CapturedGroups,
//                                                     &CapturedLength);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("NtAdjustGroupsToken(): Failed to capture the NewState list of groups (Status 0x%lx)\n", Status);
//             return Status;
//         }
//     }
//
//     /* Time to reference the token */
//     Status = ObReferenceObjectByHandle(TokenHandle,
//                                        TOKEN_ADJUST_GROUPS | (PreviousState != NULL ? TOKEN_QUERY : 0),
//                                        SeTokenObjectType as *mut _OBJECT_TYPE,
//                                        PreviousMode,
//                                        (PVOID*)&Token,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* We couldn't reference the access token, bail out */
//         DPRINT1("NtAdjustGroupsToken(): Failed to reference the token (Status 0x%lx)\n", Status);
//
//         if (CapturedGroups != NULL)
//         {
//             SeReleaseSidAndAttributesArray(CapturedGroups,
//                                            PreviousMode,
//                                            TRUE);
//         }
//
//         return Status;
//     }
//
//     /* Lock the token */
//     SepAcquireTokenLockExclusive(Token);
//
//     /* Count the number of groups to be changed */
//     Status = SepAdjustGroups(Token,
//                              CapturedGroups,
//                              CapturedCount,
//                              FALSE,
//                              ResetToDefault,
//                              &ChangesMade,
//                              NULL,
//                              &ChangeCount);
//
//     /* Does the caller want the previous state of groups? */
//     if (PreviousState != NULL)
//     {
//         /* Calculate the required length */
//         RequiredLength = FIELD_OFFSET(TOKEN_GROUPS, Groups[ChangeCount]);
//
//         /* Return the required length to the caller */
//         _SEH2_TRY
//         {
//             *ReturnLength = RequiredLength;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Bail out and return the exception code */
//             Status = _SEH2_GetExceptionCode();
//             _SEH2_YIELD(goto Quit);
//         }
//         _SEH2_END;
//
//         /* The buffer length provided is smaller than the required length, bail out */
//         if (BufferLength < RequiredLength)
//         {
//             Status = STATUS_BUFFER_TOO_SMALL;
//             goto Quit;
//         }
//     }
//
//     /*
//      * Now it's time to apply changes. Wrap the code
//      * in SEH as we are returning the old groups state
//      * list to the caller since PreviousState is a
//      * UM pointer.
//      */
//     _SEH2_TRY
//     {
//         Status = SepAdjustGroups(Token,
//                                  CapturedGroups,
//                                  CapturedCount,
//                                  TRUE,
//                                  ResetToDefault,
//                                  &ChangesMade,
//                                  PreviousState,
//                                  &ChangeCount);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Bail out and return the exception code */
//         Status = _SEH2_GetExceptionCode();
//
//         /* Force the write as we touched the token still */
//         ChangesMade = TRUE;
//         _SEH2_YIELD(goto Quit);
//     }
//     _SEH2_END;
//
// Quit:
//     /* Allocate a new ID for the token as we made changes */
//     if (ChangesMade)
//         ExAllocateLocallyUniqueId(&Token->ModifiedId);
//
//     /* Unlock and dereference the token */
//     SepReleaseTokenLock(Token);
//     ObDereferenceObject(Token);
//
//     /* Release the captured groups */
//     if (CapturedGroups != NULL)
//     {
//         SeReleaseSidAndAttributesArray(CapturedGroups,
//                                        PreviousMode,
//                                        TRUE);
//     }
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
