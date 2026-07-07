use ntkrnl_sys::ddk::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "C" {
    fn SepAcquireTokenLockShared(Token: PTOKEN);
    fn SepReleaseTokenLock(Token: PTOKEN);
}

#[no_mangle]
pub static mut SepSubjectContextLock: ERESOURCE = unsafe { core::mem::zeroed() };

#[no_mangle]
pub unsafe extern "system" fn SeCaptureSubjectContextEx(
    Thread: PETHREAD,
    Process: ntkrnl_sys::ps::PEPROCESS,
    SubjectContext: PSECURITY_SUBJECT_CONTEXT,
) {
    let Thread = opaque_ptr(Thread);
    let Process = opaque_ptr(Process);
    let SubjectContext = opaque_ptr(SubjectContext);
    let mut CopyOnOpen: BOOLEAN = 0;
    let mut EffectiveOnly: BOOLEAN = 0;

    (*SubjectContext).ProcessAuditId = (*(Process as *mut ntkrnl_sys::ddk::_EPROCESS)).UniqueProcessId;

    if Thread.is_null() {
        (*SubjectContext).ClientToken = core::ptr::null_mut();
    } else {
        (*SubjectContext).ClientToken = PsReferenceImpersonationToken(
            Thread,
            core::ptr::addr_of_mut!(CopyOnOpen),
            core::ptr::addr_of_mut!(EffectiveOnly),
            core::ptr::addr_of_mut!((*SubjectContext).ImpersonationLevel),
        );
    }

    (*SubjectContext).PrimaryToken = PsReferencePrimaryToken(Process as *mut _EPROCESS);
}

#[no_mangle]
pub unsafe extern "system" fn SeCaptureSubjectContext(
    SubjectContext: PSECURITY_SUBJECT_CONTEXT,
) {
    let SubjectContext = opaque_ptr(SubjectContext);
    SeCaptureSubjectContextEx(
        PsGetCurrentThread(),
        PsGetCurrentProcess(),
        SubjectContext,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SeLockSubjectContext(
    SubjectContext: PSECURITY_SUBJECT_CONTEXT,
) {
    let SubjectContext = opaque_ptr(SubjectContext);
    let PrimaryToken: PTOKEN = (*SubjectContext).PrimaryToken as PTOKEN;
    let ClientToken: PTOKEN = (*SubjectContext).ClientToken as PTOKEN;

    SepAcquireTokenLockShared(PrimaryToken);

    if ClientToken.is_null() {
        return;
    }
    SepAcquireTokenLockShared(ClientToken);
}

#[no_mangle]
pub unsafe extern "system" fn SeUnlockSubjectContext(
    SubjectContext: PSECURITY_SUBJECT_CONTEXT,
) {
    let SubjectContext = opaque_ptr(SubjectContext);
    let PrimaryToken: PTOKEN = (*SubjectContext).PrimaryToken as PTOKEN;
    let ClientToken: PTOKEN = (*SubjectContext).ClientToken as PTOKEN;

    if !ClientToken.is_null() {
        SepReleaseTokenLock(ClientToken);
    }

    SepReleaseTokenLock(PrimaryToken);
}

#[no_mangle]
pub unsafe extern "system" fn SeReleaseSubjectContext(
    SubjectContext: PSECURITY_SUBJECT_CONTEXT,
) {
    let SubjectContext = opaque_ptr(SubjectContext);
    ObFastDereferenceObject(
        core::ptr::addr_of_mut!((*(PsGetCurrentProcess() as *mut _EPROCESS)).Token) as *mut _EX_FAST_REF,
        (*SubjectContext).PrimaryToken,
    );
    (*SubjectContext).PrimaryToken = core::ptr::null_mut();

    PsDereferenceImpersonationToken((*SubjectContext).ClientToken);
    (*SubjectContext).ClientToken = core::ptr::null_mut();
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Security subject context support routines
//  * COPYRIGHT:   Copyright Alex Ionescu <alex@relsoft.net>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// ERESOURCE SepSubjectContextLock;
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /**
//  * @brief
//  * An extended function that captures the security subject context based upon
//  * the specified thread and process.
//  *
//  * @param[in] Thread
//  * A thread where the calling thread's token is to be referenced for
//  * the security context.
//  *
//  * @param[in] Process
//  * A process where the main process' token is to be referenced for
//  * the security context.
//  *
//  * @param[out] SubjectContext
//  * The returned security subject context.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SeCaptureSubjectContextEx(
//     _In_ PETHREAD Thread,
//     _In_ PEPROCESS Process,
//     _Out_ PSECURITY_SUBJECT_CONTEXT SubjectContext)
// {
//     BOOLEAN CopyOnOpen, EffectiveOnly;
//
//     PAGED_CODE();
//
//     /* Save the unique ID */
//     SubjectContext->ProcessAuditId = Process->UniqueProcessId;
//
//     /* Check if we have a thread */
//     if (!Thread)
//     {
//         /* We don't, so no token */
//         SubjectContext->ClientToken = NULL;
//     }
//     else
//     {
//         /* Get the impersonation token */
//         SubjectContext->ClientToken = PsReferenceImpersonationToken(Thread,
//                                                                     &CopyOnOpen,
//                                                                     &EffectiveOnly,
//                                                                     &SubjectContext->ImpersonationLevel);
//     }
//
//     /* Get the primary token */
//     SubjectContext->PrimaryToken = PsReferencePrimaryToken(Process);
// }
//
// /**
//  * @brief
//  * Captures the security subject context of the calling thread and calling
//  * process.
//  *
//  * @param[out] SubjectContext
//  * The returned security subject context.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SeCaptureSubjectContext(
//     _Out_ PSECURITY_SUBJECT_CONTEXT SubjectContext)
// {
//     /* Call the extended API */
//     SeCaptureSubjectContextEx(PsGetCurrentThread(),
//                               PsGetCurrentProcess(),
//                               SubjectContext);
// }
//
// /**
//  * @brief
//  * Locks both the referenced primary and client access tokens of a
//  * security subject context.
//  *
//  * @param[in] SubjectContext
//  * A valid security context with both referenced tokens.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SeLockSubjectContext(
//     _In_ PSECURITY_SUBJECT_CONTEXT SubjectContext)
// {
//     PTOKEN PrimaryToken, ClientToken;
//     PAGED_CODE();
//
//     /* Read both tokens */
//     PrimaryToken = SubjectContext->PrimaryToken;
//     ClientToken = SubjectContext->ClientToken;
//
//     /* Always lock the primary */
//     SepAcquireTokenLockShared(PrimaryToken);
//
//     /* Lock the impersonation one if it's there */
//     if (!ClientToken) return;
//     SepAcquireTokenLockShared(ClientToken);
// }
//
// /**
//  * @brief
//  * Unlocks both the referenced primary and client access tokens of a
//  * security subject context.
//  *
//  * @param[in] SubjectContext
//  * A valid security context with both referenced tokens.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SeUnlockSubjectContext(
//     _In_ PSECURITY_SUBJECT_CONTEXT SubjectContext)
// {
//     PTOKEN PrimaryToken, ClientToken;
//     PAGED_CODE();
//
//     /* Read both tokens */
//     PrimaryToken = SubjectContext->PrimaryToken;
//     ClientToken = SubjectContext->ClientToken;
//
//     /* Unlock the impersonation one if it's there */
//     if (ClientToken)
//     {
//         SepReleaseTokenLock(ClientToken);
//     }
//
//     /* Always unlock the primary one */
//     SepReleaseTokenLock(PrimaryToken);
// }
//
// /**
//  * @brief
//  * Releases both the primary and client tokens of a security
//  * subject context.
//  *
//  * @param[in] SubjectContext
//  * The captured security context.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SeReleaseSubjectContext(
//     _In_ PSECURITY_SUBJECT_CONTEXT SubjectContext)
// {
//     PAGED_CODE();
//
//     /* Drop reference on the primary */
//     ObFastDereferenceObject(&PsGetCurrentProcess()->Token, SubjectContext->PrimaryToken);
//     SubjectContext->PrimaryToken = NULL;
//
//     /* Drop reference on the impersonation, if there was one */
//     PsDereferenceImpersonationToken(SubjectContext->ClientToken);
//     SubjectContext->ClientToken = NULL;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
