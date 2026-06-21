//! secursup.rs — secursup.c: Pipes Security Support (client security contexts).
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports, unused_variables, unused_mut, unused_parens, unused_assignments)]

use core::ptr::{addr_of, addr_of_mut, null_mut};

use ntkrnl_sys::base::{NT_SUCCESS, InitializeListHead, InsertHeadList, InsertTailList, RemoveEntryList, RemoveHeadList, RemoveTailList, IsListEmpty};
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::ex::ExFreePoolWithTag;
use ntkrnl_sys::ob::ObDereferenceObject;

use crate::glue::*;
use crate::globals::*;
use crate::types::*;
use crate::helpers::*;
use crate::CONTAINING_RECORD;

// --- constants NOT in glue.rs (added here, exact SDK-header values) ---------
// psdk/ntstatus.h: STATUS_CANNOT_IMPERSONATE == ((NTSTATUS)0xC000010D)
const STATUS_CANNOT_IMPERSONATE: NTSTATUS = 0xC000_010Du32 as NTSTATUS;
// xdk/iotypes.h: POOL_QUOTA_FAIL_INSTEAD_OF_RAISE == 0x8
const POOL_QUOTA_FAIL_INSTEAD_OF_RAISE: ULONG = 0x8;
// xdk/setypes.h: SECURITY_DYNAMIC_TRACKING == (TRUE) == 1
//   (ContextTrackingMode field is SECURITY_CONTEXT_TRACKING_MODE == BOOLEAN == u8)
const SECURITY_DYNAMIC_TRACKING: BOOLEAN = 1;
// xdk/setypes.h: enum _SECURITY_IMPERSONATION_LEVEL { SecurityAnonymous=0,
//   SecurityIdentification=1, SecurityImpersonation=2, SecurityDelegation=3 }
const SecurityImpersonation: SECURITY_IMPERSONATION_LEVEL = 2;
// xdk/setypes.h: enum _TOKEN_TYPE { TokenPrimary = 1, TokenImpersonation }
const TokenPrimary: TOKEN_TYPE = 1;

pub unsafe extern "system" fn NpImpersonateClientContext(Ccb: PNP_CCB) -> NTSTATUS {
    let Status: NTSTATUS;
    let ClientContext: PSECURITY_CLIENT_CONTEXT;
    // PAGED_CODE();

    ClientContext = (*Ccb).ClientContext;
    if !ClientContext.is_null() {
        Status = SeImpersonateClientEx(ClientContext, null_mut());
    } else {
        Status = STATUS_CANNOT_IMPERSONATE;
    }
    Status
}

pub unsafe extern "system" fn NpFreeClientSecurityContext(ClientContext: PSECURITY_CLIENT_CONTEXT) {
    let TokenType: TOKEN_TYPE;
    let ClientToken: PVOID;

    if ClientContext.is_null() {
        return;
    }

    TokenType = SeTokenType((*ClientContext).ClientToken);
    ClientToken = (*ClientContext).ClientToken as PVOID;
    if (TokenType == TokenPrimary) || (!ClientToken.is_null()) {
        ObDereferenceObject(ClientToken);
    }
    ExFreePool(ClientContext as PVOID);
}

pub unsafe extern "system" fn NpCopyClientContext(
    Ccb: PNP_CCB,
    DataQueueEntry: PNP_DATA_QUEUE_ENTRY,
) {
    // PAGED_CODE();

    if (*DataQueueEntry).ClientSecurityContext.is_null() {
        return;
    }

    NpFreeClientSecurityContext((*Ccb).ClientContext);
    (*Ccb).ClientContext = (*DataQueueEntry).ClientSecurityContext;
    (*DataQueueEntry).ClientSecurityContext = null_mut();
}

pub unsafe extern "system" fn NpUninitializeSecurity(Ccb: PNP_CCB) {
    // PAGED_CODE();

    NpFreeClientSecurityContext((*Ccb).ClientContext);
    (*Ccb).ClientContext = null_mut();
}

pub unsafe extern "system" fn NpInitializeSecurity(
    Ccb: PNP_CCB,
    SecurityQos: PSECURITY_QUALITY_OF_SERVICE,
    Thread: PETHREAD,
) -> NTSTATUS {
    let ClientContext: PSECURITY_CLIENT_CONTEXT;
    let mut Status: NTSTATUS;
    // PAGED_CODE();

    if !SecurityQos.is_null() {
        (*Ccb).ClientQos = *SecurityQos;
    } else {
        (*Ccb).ClientQos.Length = core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>() as ULONG;
        (*Ccb).ClientQos.ImpersonationLevel = SecurityImpersonation;
        (*Ccb).ClientQos.ContextTrackingMode = SECURITY_DYNAMIC_TRACKING;
        (*Ccb).ClientQos.EffectiveOnly = TRUE;
    }

    NpUninitializeSecurity(Ccb);

    if (*Ccb).ClientQos.ContextTrackingMode == SECURITY_DYNAMIC_TRACKING {
        Status = STATUS_SUCCESS;
        (*Ccb).ClientContext = null_mut();
        return Status;
    }

    ClientContext = ExAllocatePoolWithQuotaTag(
        (PagedPool as ULONG | POOL_QUOTA_FAIL_INSTEAD_OF_RAISE) as POOL_TYPE,
        core::mem::size_of::<SECURITY_CLIENT_CONTEXT>() as _,
        NPFS_CLIENT_SEC_CTX_TAG,
    ) as PSECURITY_CLIENT_CONTEXT;
    (*Ccb).ClientContext = ClientContext;
    if ClientContext.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = SeCreateClientSecurity(Thread, addr_of_mut!((*Ccb).ClientQos), 0, ClientContext);
    if !NT_SUCCESS(Status) {
        ExFreePool((*Ccb).ClientContext as PVOID);
        (*Ccb).ClientContext = null_mut();
    }

    Status
}

pub unsafe extern "system" fn NpGetClientSecurityContext(
    NamedPipeEnd: ULONG,
    Ccb: PNP_CCB,
    Thread: PETHREAD,
    Context: *mut PSECURITY_CLIENT_CONTEXT,
) -> NTSTATUS {
    let mut NewContext: PSECURITY_CLIENT_CONTEXT;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    // PAGED_CODE();

    if NamedPipeEnd == FILE_PIPE_SERVER_END
        || (*Ccb).ClientQos.ContextTrackingMode != SECURITY_DYNAMIC_TRACKING
    {
        NewContext = null_mut();
        Status = STATUS_SUCCESS;
    } else {
        NewContext = ExAllocatePoolWithQuotaTag(
            (PagedPool as ULONG | POOL_QUOTA_FAIL_INSTEAD_OF_RAISE) as POOL_TYPE,
            core::mem::size_of::<SECURITY_CLIENT_CONTEXT>() as _,
            NPFS_CLIENT_SEC_CTX_TAG,
        ) as PSECURITY_CLIENT_CONTEXT;
        if NewContext.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        Status = SeCreateClientSecurity(Thread, addr_of_mut!((*Ccb).ClientQos), 0, NewContext);
        if !NT_SUCCESS(Status) {
            ExFreePool(NewContext as PVOID);
            NewContext = null_mut();
        }
    }
    *Context = NewContext;
    Status
}

/* EOF */
