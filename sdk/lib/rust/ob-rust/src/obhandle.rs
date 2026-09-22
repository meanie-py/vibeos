use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use super::oblife::{ObpCaptureObjectCreateInformation, ObpFreeObjectNameBuffer};
use super::obname::ObpLookupObjectName;

use ntkrnl_sys::ddk::{
    _HANDLE_TABLE_ENTRY, _OBJECT_HEADER, _OBJECT_TYPE, _EPROCESS, _ETHREAD,
    _OBJECT_HEADER_HANDLE_INFO, _OBJECT_HEADER_QUOTA_INFO, _OBJECT_HEADER_NAME_INFO,
    _OBJECT_HEADER_CREATOR_INFO, _OBJECT_HANDLE_COUNT_ENTRY, _OBJECT_HANDLE_COUNT_DATABASE,
    _OBJECT_HANDLE_INFORMATION, _OBP_FIND_HANDLE_DATA, _OBP_CLOSE_HANDLE_CONTEXT,
    _OBP_SET_HANDLE_ATTRIBUTES_CONTEXT, _OBP_LOOKUP_CONTEXT, _OB_TEMP_BUFFER,
    _ACCESS_STATE, _AUX_ACCESS_DATA, _OBJECT_ATTRIBUTES, _UNICODE_STRING,
    PEPROCESS, PETHREAD, POBJECT_HEADER, POBJECT_TYPE, PHANDLE_TABLE, PHANDLE_TABLE_ENTRY,
    POBJECT_HEADER_HANDLE_INFO, POBJECT_HEADER_QUOTA_INFO, POBJECT_HEADER_NAME_INFO,
    POBJECT_HEADER_CREATOR_INFO, POBJECT_HANDLE_COUNT_ENTRY, POBJECT_HANDLE_COUNT_DATABASE,
    POBJECT_HANDLE_INFORMATION, POBP_FIND_HANDLE_DATA, POBP_CLOSE_HANDLE_CONTEXT,
    POBP_SET_HANDLE_ATTRIBUTES_CONTEXT, POBP_LOOKUP_CONTEXT, POB_TEMP_BUFFER,
    PACCESS_STATE, PAUX_ACCESS_DATA, POBJECT_ATTRIBUTES, PUNICODE_STRING,
    ACCESS_MASK, KPROCESSOR_MODE, HANDLE, OB_OPEN_REASON,
    PSECURITY_INFORMATION, SECURITY_OPERATION_CODE,
    SeSinglePrivilegeCheck,
    ExAcquireRundownProtection, ExReleaseRundownProtection, ExMapHandleToPointer,
    ExUnlockHandleTableEntry, ExCreateHandle, ExDestroyHandle, ExEnumHandleTable,
    ExChangeHandle, ExSweepHandleTable, ExCreateHandleTable, ExDupHandleTable,
    ExDestroyHandleTable, ExGetPreviousMode, KeEnterCriticalRegion, KeLeaveCriticalRegion,
    KeStackAttachProcess, KeUnstackDetachProcess, KeIsAttachedProcess, KeRaiseUserException,
    IoSetThreadHardErrorMode, ExWaitForRundownProtectionRelease, ExRundownCompleted,
    _KAPC_STATE,
    ObAssignSecurity,
    ObCheckObjectAccess,
    ObGetObjectSecurity,
    ObReferenceObjectByHandle,
    ObReferenceObjectByPointer,
    ObReleaseObjectSecurity,
    ObpAcquireObjectLock,
    ObpCalloutEnd,
    ObpCalloutStart,
    ObpCreateSymbolicLinkName,
    ObpDeleteNameCheck,
    ObpDereferenceNameInfo,
    ObpEnterObjectTypeMutex,
    ObpFreeObjectCreateInformation,
    ObpInitializeLookupContext,
    ObpLeaveObjectTypeMutex,
    ObpReferenceNameInfo,
    ObpReleaseLookupContext,
    ObpReleaseObjectCreateInformation,
    ObpReleaseObjectLock,
    PCHAR,
    PGENERIC_MAPPING,
    PISECURITY_DESCRIPTOR,
    POBJECT_CREATE_INFORMATION,
    POBJECT_HANDLE_ATTRIBUTE_INFORMATION,
    PSECURITY_DESCRIPTOR,
    PsChargeSharedPoolQuota,
    PsGetCurrentThread,
    PsIsThreadTerminating,
    RtlMapGenericMask,
    SeCreateAccessState,
    SeDeleteAccessState,
    SeDetailedAuditingWithToken,
    SeReleaseSecurityDescriptor,
    _GENERIC_MAPPING,
    _OBJECT_CREATE_INFORMATION,
    _OBJECT_HANDLE_ATTRIBUTE_INFORMATION,
    _OBJECT_SYMBOLIC_LINK,
    _OBJECT_DIRECTORY,
    _OB_OPEN_REASON_ObCreateHandle,
    _OB_OPEN_REASON_ObDuplicateHandle,
    _OB_OPEN_REASON_ObInheritHandle,
    _OB_OPEN_REASON_ObOpenHandle,
    _SECURITY_DESCRIPTOR,
};


// ObAttributes field type — matches bindgen's ULONG_PTR (c_ulonglong on 64-bit, c_ulong on 32-bit)
#[cfg(target_pointer_width = "64")]
type ObAttrType = u64;
#[cfg(target_pointer_width = "32")]
type ObAttrType = u32;

// ntkrnl_sys::ddk's bindgen-generated OB_SECURITY_METHOD (and its SeDefaultObjectMethod
// extern decl) predates the upstream-sync merge's se.h/obtypes.h change that appended a
// trailing KPROCESSOR_MODE AccessMode param -- the checked-in bindgen_*.rs was never
// regenerated. Declare the REAL 9-param signature locally (shadowing the stale 8-param
// import) so the pointer-identity comparison below transmutes from a type that actually
// matches se-rust's semgr.rs definition, rather than silently truncating the arg list.
extern "system" {
    fn SeDefaultObjectMethod(
        Object: PVOID,
        OperationType: SECURITY_OPERATION_CODE,
        SecurityInformation: PSECURITY_INFORMATION,
        SecurityDescriptor: PSECURITY_DESCRIPTOR,
        ReturnLength: *mut ULONG,
        OldSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
        PoolType: POOL_TYPE,
        GenericMapping: PGENERIC_MAPPING,
        AccessMode: KPROCESSOR_MODE,
    ) -> NTSTATUS;
}

const ObCreateHandle: OB_OPEN_REASON = _OB_OPEN_REASON_ObCreateHandle;
const ObOpenHandle: OB_OPEN_REASON = _OB_OPEN_REASON_ObOpenHandle;
const ObDuplicateHandle: OB_OPEN_REASON = _OB_OPEN_REASON_ObDuplicateHandle;
const ObInheritHandle: OB_OPEN_REASON = _OB_OPEN_REASON_ObInheritHandle;

#[no_mangle]
pub static mut ObpKernelHandleTable: PHANDLE_TABLE = core::ptr::null_mut();

unsafe fn ObReferenceProcessHandleTable(process: PEPROCESS) -> PHANDLE_TABLE {
    let process = process as *mut _EPROCESS;
    let mut handle_table = core::ptr::null_mut();

    if ExAcquireRundownProtection(core::ptr::addr_of_mut!((*process).RundownProtect)) != 0 {
        handle_table = (*process).ObjectTable;
        if handle_table.is_null() {
            ExReleaseRundownProtection(core::ptr::addr_of_mut!((*process).RundownProtect));
        }
    }

    handle_table
}

unsafe fn ObDereferenceProcessHandleTable(process: PEPROCESS) {
    let process = process as *mut _EPROCESS;
    ExReleaseRundownProtection(core::ptr::addr_of_mut!((*process).RundownProtect));
}

#[no_mangle]
pub unsafe extern "system" fn ObGetProcessHandleCount(process: PEPROCESS) -> ULONG {
    let process = opaque_ptr(process);
    let handle_table = ObReferenceProcessHandleTable(process);

    if !handle_table.is_null() {
        let handle_count = (*handle_table).HandleCount;
        ObDereferenceProcessHandleTable(process);
        handle_count as ULONG
    } else {
        0
    }
}

unsafe fn ObpReferenceProcessObjectByHandle(
    mut handle: HANDLE,
    process: PEPROCESS,
    mut handle_table: PHANDLE_TABLE,
    access_mode: KPROCESSOR_MODE,
    object: *mut PVOID,
    handle_information: POBJECT_HANDLE_INFORMATION,
    audit_mask: *mut ACCESS_MASK,
) -> NTSTATUS {
    let mut handle_entry: PHANDLE_TABLE_ENTRY;
    let mut object_header: POBJECT_HEADER;
    let mut granted_access: ACCESS_MASK;
    let mut attributes: ULONG;
    let thread = PsGetCurrentThread();
    let mut status: NTSTATUS;

    *object = core::ptr::null_mut();

    if HandleToLong(handle) < 0 {
        if handle == NtCurrentProcess() {
            (*handle_information).HandleAttributes = 0;
            (*handle_information).GrantedAccess = (*(process as *mut _EPROCESS)).GrantedAccess;
            *audit_mask = 0;

            object_header = OBJECT_TO_OBJECT_HEADER(process as PVOID);
            InterlockedIncrementSizeT(
                core::ptr::addr_of_mut!((*object_header).PointerCount) as *mut isize
            );

            *object = process as PVOID;
            return STATUS_SUCCESS;
        }

        if handle == NtCurrentThread() {
            (*handle_information).HandleAttributes = 0;
            (*handle_information).GrantedAccess = (*thread).GrantedAccess;

            object_header = OBJECT_TO_OBJECT_HEADER(thread as PVOID);
            InterlockedIncrementSizeT(
                core::ptr::addr_of_mut!((*object_header).PointerCount) as *mut isize
            );

            *audit_mask = 0;
            *object = thread as PVOID;
            return STATUS_SUCCESS;
        }

        if access_mode == 0i8 {
            handle = ObKernelHandleToHandle(handle);
            handle_table = ObpKernelHandleTable;
        } else {
            return STATUS_INVALID_HANDLE;
        }
    }

    KeEnterCriticalRegion();

    handle_entry = ExMapHandleToPointer(handle_table, handle);
    if !handle_entry.is_null() {
        object_header = ObpGetHandleObject(handle_entry);
        granted_access = (*handle_entry).__bindgen_anon_2.GrantedAccess;
        attributes = ((*handle_entry).__bindgen_anon_1.ObAttributes as ULONG) & OBJ_HANDLE_ATTRIBUTES;

        (*handle_information).HandleAttributes = attributes;
        (*handle_information).GrantedAccess = granted_access;
        *audit_mask = 0;

        *object = core::ptr::addr_of_mut!((*object_header).Body) as PVOID;
        InterlockedIncrementSizeT(
            core::ptr::addr_of_mut!((*object_header).PointerCount) as *mut isize
        );

        ExUnlockHandleTableEntry(handle_table, handle_entry);
        KeLeaveCriticalRegion();
        return STATUS_SUCCESS;
    } else {
        status = STATUS_INVALID_HANDLE;
    }

    KeLeaveCriticalRegion();
    status
}

// Callback function wrappers — stdcall on i386, C on 64-bit (matching bindgen)
unsafe extern "system" fn ObpEnumFindHandleProcedure(
    handle_entry: PHANDLE_TABLE_ENTRY, _handle: HANDLE, context: PVOID,
) -> BOOLEAN { _ObpEnumFindHandleProcedure(handle_entry, _handle, context) }

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn ObpSetHandleAttributes(
    handle_entry: *mut _HANDLE_TABLE_ENTRY, context: u32,
) -> BOOLEAN { _ObpSetHandleAttributes(handle_entry, context as ULONG_PTR) }
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn ObpSetHandleAttributes(
    handle_entry: *mut _HANDLE_TABLE_ENTRY, context: u64,
) -> BOOLEAN { _ObpSetHandleAttributes(handle_entry, context as ULONG_PTR) }
unsafe extern "system" fn ObpCloseHandleCallback(
    handle_entry: *mut _HANDLE_TABLE_ENTRY, handle: HANDLE, context: PVOID,
) -> BOOLEAN { _ObpCloseHandleCallback(handle_entry, handle, context) }

unsafe extern "system" fn ObpDuplicateHandleCallback(
    process: PEPROCESS, handle_table: PHANDLE_TABLE, old_entry: PHANDLE_TABLE_ENTRY, new_entry: PHANDLE_TABLE_ENTRY,
) -> BOOLEAN { _ObpDuplicateHandleCallback(process, handle_table, old_entry, new_entry) }

unsafe fn _ObpEnumFindHandleProcedure(
    handle_entry: PHANDLE_TABLE_ENTRY,
    _handle: HANDLE,
    context: PVOID,
) -> BOOLEAN {
    let object_header: POBJECT_HEADER;
    let granted_access: ACCESS_MASK;
    let handle_attributes: ULONG;
    let find_data = context as POBP_FIND_HANDLE_DATA;

    object_header = ObpGetHandleObject(handle_entry);

    if !(*find_data).ObjectHeader.is_null() && (*find_data).ObjectHeader != object_header {
        return 0u8;
    }

    if !(*find_data).ObjectType.is_null() && (*find_data).ObjectType != (*object_header).Type {
        return 0u8;
    }

    if !(*find_data).HandleInformation.is_null() {
        granted_access = (*handle_entry).__bindgen_anon_2.GrantedAccess;
        handle_attributes = ((*handle_entry).__bindgen_anon_1.ObAttributes as ULONG) & OBJ_HANDLE_ATTRIBUTES;

        if (*(*find_data).HandleInformation).HandleAttributes != handle_attributes
            || (*(*find_data).HandleInformation).GrantedAccess != granted_access
        {
            return 0u8;
        }
    }

    1u8
}

unsafe fn ObpInsertHandleCount(object_header: POBJECT_HEADER) -> POBJECT_HANDLE_COUNT_ENTRY {
    let handle_info: POBJECT_HEADER_HANDLE_INFO;
    let free_entry: POBJECT_HANDLE_COUNT_ENTRY;
    let mut handle_database: POBJECT_HANDLE_COUNT_DATABASE;
    let old_handle_database: POBJECT_HANDLE_COUNT_DATABASE;
    let mut i: ULONG;
    let mut size: ULONG;
    let old_size: ULONG;
    let mut single_database: _OBJECT_HANDLE_COUNT_DATABASE = core::mem::zeroed();

    handle_info = OBJECT_HEADER_TO_HANDLE_INFO(object_header);
    if handle_info.is_null() {
        return core::ptr::null_mut();
    }

    if (*object_header).Flags & OB_FLAG_SINGLE_PROCESS != 0 {
        single_database.CountEntries = 1;
        single_database.HandleCountEntries[0] = (*handle_info).__bindgen_anon_1.SingleEntry;

        old_handle_database = &mut single_database as *mut _;
        old_size = core::mem::size_of::<_OBJECT_HANDLE_COUNT_DATABASE>() as ULONG;

        i = 2;
        size = core::mem::size_of::<_OBJECT_HANDLE_COUNT_DATABASE>() as ULONG
            + ((i - 1) * core::mem::size_of::<_OBJECT_HANDLE_COUNT_ENTRY>() as ULONG);
    } else {
        old_handle_database = (*handle_info).__bindgen_anon_1.HandleCountDatabase;
        i = (*old_handle_database).CountEntries;
        old_size = core::mem::size_of::<_OBJECT_HANDLE_COUNT_DATABASE>() as ULONG
            + ((i - 1) * core::mem::size_of::<_OBJECT_HANDLE_COUNT_ENTRY>() as ULONG);

        i += 4;
        size = old_size + (4 * core::mem::size_of::<_OBJECT_HANDLE_COUNT_ENTRY>() as ULONG);
    }

    handle_database = ExAllocatePoolWithTag(PagedPool, size as usize, TAG_OB_HANDLE) as POBJECT_HANDLE_COUNT_DATABASE;
    if handle_database.is_null() {
        return core::ptr::null_mut();
    }

    core::ptr::copy_nonoverlapping(old_handle_database as *const u8, handle_database as *mut u8, old_size as usize);

    if (*object_header).Flags & OB_FLAG_SINGLE_PROCESS != 0 {
        (*object_header).Flags &= !OB_FLAG_SINGLE_PROCESS;
    } else {
        ExFreePoolWithTag(old_handle_database as PVOID, TAG_OB_HANDLE);
    }

    free_entry = (handle_database as usize + old_size as usize) as PVOID as POBJECT_HANDLE_COUNT_ENTRY;
    core::ptr::write_bytes(free_entry as *mut u8, 0, (size - old_size) as usize);

    (*handle_database).CountEntries = i;
    (*handle_info).__bindgen_anon_1.HandleCountDatabase = handle_database;
    free_entry
}

unsafe fn ObpIncrementHandleDataBase(
    object_header: POBJECT_HEADER,
    process: PEPROCESS,
    new_process_handle_count: *mut ULONG,
) -> NTSTATUS {
    let handle_info: POBJECT_HEADER_HANDLE_INFO;
    let mut handle_entry: POBJECT_HANDLE_COUNT_ENTRY;
    let mut free_entry: POBJECT_HANDLE_COUNT_ENTRY = core::ptr::null_mut();
    let handle_database: POBJECT_HANDLE_COUNT_DATABASE;
    let mut i: ULONG;

    handle_info = OBJECT_HEADER_TO_HANDLE_INFO(object_header);
    if (*object_header).Flags & OB_FLAG_SINGLE_PROCESS != 0 {
        if (*handle_info).__bindgen_anon_1.SingleEntry.HandleCount == 0 {
            (*handle_info).__bindgen_anon_1.SingleEntry.HandleCount = 1;
            (*handle_info).__bindgen_anon_1.SingleEntry.Process = process;

            *new_process_handle_count = 1;
            return STATUS_SUCCESS;
        } else if (*handle_info).__bindgen_anon_1.SingleEntry.Process == process {
            (*handle_info).__bindgen_anon_1.SingleEntry.HandleCount += 1;
            *new_process_handle_count = (*handle_info).__bindgen_anon_1.SingleEntry.HandleCount;
            return STATUS_SUCCESS;
        } else {
            free_entry = ObpInsertHandleCount(object_header);
            if free_entry.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            (*free_entry).Process = process;
            (*free_entry).HandleCount = 1;

            *new_process_handle_count = 1;
            return STATUS_SUCCESS;
        }
    }

    handle_database = (*handle_info).__bindgen_anon_1.HandleCountDatabase;
    if !handle_database.is_null() {
        i = (*handle_database).CountEntries;
        handle_entry = core::ptr::addr_of_mut!((*handle_database).HandleCountEntries[0]);
        while i > 0 {
            if (*handle_entry).Process == process {
                (*handle_entry).HandleCount += 1;
                *new_process_handle_count = (*handle_entry).HandleCount;
                return STATUS_SUCCESS;
            } else if (*handle_entry).HandleCount == 0 {
                free_entry = handle_entry;
            }

            handle_entry = handle_entry.offset(1);
            i -= 1;
        }

        if free_entry.is_null() {
            free_entry = ObpInsertHandleCount(object_header);
            if free_entry.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }
        }

        (*free_entry).Process = process;
        (*free_entry).HandleCount = 1;
        *new_process_handle_count = 1;
    }

    STATUS_SUCCESS
}

unsafe fn ObpChargeQuotaForObject(
    object_header: POBJECT_HEADER,
    object_type: POBJECT_TYPE,
    new_object: *mut BOOLEAN,
) -> NTSTATUS {
    let object_quota: POBJECT_HEADER_QUOTA_INFO;
    let mut paged_pool_charge: ULONG;
    let mut non_paged_pool_charge: ULONG;

    object_quota = OBJECT_HEADER_TO_QUOTA_INFO(object_header);
    *new_object = 0u8;

    if (*object_header).Flags & OB_FLAG_CREATE_INFO != 0 {
        (*object_header).Flags &= !OB_FLAG_CREATE_INFO;
        if !object_quota.is_null() {
            paged_pool_charge = (*object_quota).PagedPoolCharge
                + (*object_quota).SecurityDescriptorCharge;
            non_paged_pool_charge = (*object_quota).NonPagedPoolCharge;
        } else {
            paged_pool_charge = (*object_type).TypeInfo.DefaultPagedPoolCharge;
            non_paged_pool_charge = (*object_type).TypeInfo.DefaultNonPagedPoolCharge;
        }

        if PsGetCurrentProcess() as PVOID == PsInitialSystemProcess {
            (*object_header).__bindgen_anon_2.QuotaBlockCharged = OBP_SYSTEM_PROCESS_QUOTA as PVOID;
        } else {
            (*object_header).__bindgen_anon_2.QuotaBlockCharged = PsChargeSharedPoolQuota(
                PsGetCurrentProcess() as *mut _EPROCESS ,
                paged_pool_charge as usize as _,
                non_paged_pool_charge as usize as _,
            ) as *mut core::ffi::c_void;
        }

        if (*object_header).__bindgen_anon_2.QuotaBlockCharged.is_null() {
            return STATUS_QUOTA_EXCEEDED;
        }

        *new_object = 1u8;
    }

    STATUS_SUCCESS
}

unsafe fn ObpValidateAccessMask(access_state: PACCESS_STATE) -> NTSTATUS {
    let security_descriptor: PISECURITY_DESCRIPTOR;

    security_descriptor = (*access_state).SecurityDescriptor as PISECURITY_DESCRIPTOR;
    if !security_descriptor.is_null() {
        if ((*security_descriptor).Control & SE_SACL_PRESENT != 0)
            && ((*access_state).PreviouslyGrantedAccess & ACCESS_SYSTEM_SECURITY == 0)
        {
            (*access_state).RemainingDesiredAccess |= ACCESS_SYSTEM_SECURITY;
        }
    }

    STATUS_SUCCESS
}

unsafe fn ObpDecrementHandleCount(
    object_body: PVOID,
    process: PEPROCESS,
    granted_access: ACCESS_MASK,
    object_type: POBJECT_TYPE,
) {
    let object_header: POBJECT_HEADER;
    let system_handle_count: LONG;
    let mut process_handle_count: LONG;
    let new_count: LONG;
    let mut callout_irql: KIRQL = 0;
    let handle_info: POBJECT_HEADER_HANDLE_INFO;
    let mut handle_entry: POBJECT_HANDLE_COUNT_ENTRY = core::ptr::null_mut();
    let handle_database: POBJECT_HANDLE_COUNT_DATABASE;
    let mut i: ULONG;

    object_header = OBJECT_TO_OBJECT_HEADER(object_body);

    ObpAcquireObjectLock(object_header);

    system_handle_count = (*object_header).__bindgen_anon_1.HandleCount as LONG;
    process_handle_count = 0;

    new_count = InterlockedDecrementSizeT(
        core::ptr::addr_of_mut!((*object_header).__bindgen_anon_1.HandleCount) as *mut isize
    ) as LONG;

    if new_count == 0 && ((*object_header).Flags & OB_FLAG_EXCLUSIVE != 0) {
        (*OBJECT_HEADER_TO_QUOTA_INFO(object_header)).ExclusiveProcess = core::ptr::null_mut();
    }

    if (*object_type).TypeInfo.MaintainHandleCount != 0 {
        handle_info = OBJECT_HEADER_TO_HANDLE_INFO(object_header);

        if (*object_header).Flags & OB_FLAG_SINGLE_PROCESS != 0 {
            /* C is `ProcessHandleCount = HandleCount--` (POST-decrement): the
             * close procedure must receive the PRE-decrement count. Reading
             * after the decrement passed 0 on the last close, so IopCloseFile
             * early-returned and IRP_MJ_CLEANUP was never sent to any driver
             * (conformance catch #2, 2026-07-10). */
            process_handle_count = (*handle_info).__bindgen_anon_1.SingleEntry.HandleCount as LONG;
            (*handle_info).__bindgen_anon_1.SingleEntry.HandleCount -= 1;
            handle_entry = core::ptr::addr_of_mut!((*handle_info).__bindgen_anon_1.SingleEntry);
        } else {
            handle_database = (*handle_info).__bindgen_anon_1.HandleCountDatabase;
            if !handle_database.is_null() {
                i = (*handle_database).CountEntries;
                handle_entry = core::ptr::addr_of_mut!((*handle_database).HandleCountEntries[0]);
                while i > 0 {
                    if (*handle_entry).HandleCount > 0 && (*handle_entry).Process == process {
                        /* same POST-decrement contract as above */
                        process_handle_count = (*handle_entry).HandleCount as LONG;
                        (*handle_entry).HandleCount -= 1;
                        break;
                    }
                    handle_entry = handle_entry.offset(1);
                    i -= 1;
                }
            } else {
                handle_entry = core::ptr::null_mut();
            }
        }

        if process_handle_count == 1 {
            (*handle_entry).Process = core::ptr::null_mut();
            (*handle_entry).HandleCount = 0;
        }
    }

    ObpReleaseObjectLock(object_header);

    if (*object_type).TypeInfo.CloseProcedure.is_some() {
        ObpCalloutStart(&mut callout_irql as *mut _);
        crate::obsecure::ObpCallCloseProcedure(
            (*object_type).TypeInfo.CloseProcedure,
            process,
            object_body,
            granted_access,
            process_handle_count as ULONG_PTR,
            system_handle_count as ULONG_PTR,
        );
        ObpCalloutEnd(callout_irql, b"Close\0".as_ptr() as PCHAR, object_type, object_body);
    }

    ObpDeleteNameCheck(object_body);

    InterlockedDecrement(core::ptr::addr_of_mut!((*object_type).TotalNumberOfHandles) as *mut LONG);
}

unsafe fn ObpCloseHandleTableEntry(
    handle_table: PHANDLE_TABLE,
    handle_entry: PHANDLE_TABLE_ENTRY,
    handle: HANDLE,
    access_mode: KPROCESSOR_MODE,
    ignore_handle_protection: BOOLEAN,
) -> NTSTATUS {
    let body: PVOID;
    let object_type: POBJECT_TYPE;
    let object_header: POBJECT_HEADER;
    let granted_access: ACCESS_MASK;
    let mut callout_irql: KIRQL = 0;

    object_header = ObpGetHandleObject(handle_entry);
    object_type = (*object_header).Type;
    body = core::ptr::addr_of_mut!((*object_header).Body) as PVOID;
    granted_access = (*handle_entry).__bindgen_anon_2.GrantedAccess;

    if (*object_type).TypeInfo.OkayToCloseProcedure.is_some() {
        ObpCalloutStart(&mut callout_irql as *mut _);
        if ((*object_type).TypeInfo.OkayToCloseProcedure.unwrap())(
            PsGetCurrentProcess() as *mut _EPROCESS ,
            body,
            handle,
            access_mode,
        ) == 0
        {
            ObpCalloutEnd(callout_irql, b"NtClose\0".as_ptr() as PCHAR, object_type, body);
            ExUnlockHandleTableEntry(handle_table, handle_entry);
            return STATUS_HANDLE_NOT_CLOSABLE;
        }
        ObpCalloutEnd(callout_irql, b"NtClose\0".as_ptr() as PCHAR, object_type, body);
    }

    if ((*handle_entry).__bindgen_anon_2.GrantedAccess & ObpAccessProtectCloseBit != 0)
        && ignore_handle_protection == 0u8
    {
        if access_mode != 0i8 {
            ExUnlockHandleTableEntry(handle_table, handle_entry);

            if !(*(PsGetCurrentProcess() as *mut _EPROCESS)).DebugPort.is_null() {
                return KeRaiseUserException(STATUS_HANDLE_NOT_CLOSABLE);
            } else {
                return STATUS_HANDLE_NOT_CLOSABLE;
            }
        } else {
            KeBugCheckEx(INVALID_KERNEL_HANDLE, handle as ULONG_PTR, 0, 0, 0);
        }
    }

    ExDestroyHandle(handle_table, handle, handle_entry);

    ObpDecrementHandleCount(body, PsGetCurrentProcess() as *mut _EPROCESS , granted_access, object_type);

    ObDereferenceObject(body);

    STATUS_SUCCESS
}

unsafe fn ObpIncrementHandleCount(
    object: PVOID,
    access_state: PACCESS_STATE,
    access_mode: KPROCESSOR_MODE,
    handle_attributes: ULONG,
    process: PEPROCESS,
    open_reason: OB_OPEN_REASON,
) -> NTSTATUS {
    let object_header: POBJECT_HEADER;
    let object_type: POBJECT_TYPE;
    let mut process_handle_count: ULONG = 0;
    let mut status: NTSTATUS;
    let exclusive_process: PEPROCESS;
    let mut exclusive = 0u8;
    let mut new_object: BOOLEAN = 0u8;
    let creator_info: POBJECT_HEADER_CREATOR_INFO;
    let mut callout_irql: KIRQL = 0;
    let probe_mode: KPROCESSOR_MODE;
    let mut total: ULONG;
    let name_info: POBJECT_HEADER_NAME_INFO;

    object_header = OBJECT_TO_OBJECT_HEADER(object);
    object_type = (*object_header).Type;

    if handle_attributes & OBJ_FORCE_ACCESS_CHECK != 0 {
        probe_mode = 1i8;
    } else {
        probe_mode = access_mode;
    }

    ObpAcquireObjectLock(object_header);

    status = ObpChargeQuotaForObject(object_header, object_type, &mut new_object as *mut _);
    if status < 0 {
        // Returning here with the object lock still held deadlocks every later handle
        // operation on this object. Upstream-Commit: 1902929c80b
        ObpReleaseObjectLock(object_header);
        return status;
    }

    if handle_attributes & OBJ_EXCLUSIVE != 0 {
        if (handle_attributes & OBJ_INHERIT != 0)
            || ((*object_header).Flags & OB_FLAG_EXCLUSIVE == 0)
        {
            status = STATUS_INVALID_PARAMETER;
            ObpReleaseObjectLock(object_header);
            return status;
        }

        exclusive_process = OBJECT_HEADER_TO_EXCLUSIVE_PROCESS(object_header);
        if (exclusive_process.is_null() && (*object_header).__bindgen_anon_1.HandleCount != 0)
            || (!exclusive_process.is_null() && exclusive_process != (PsGetCurrentProcess() as *mut _EPROCESS))
        {
            status = STATUS_ACCESS_DENIED;
            ObpReleaseObjectLock(object_header);
            return status;
        }

        exclusive = 1u8;
    } else if ((*object_header).Flags & OB_FLAG_EXCLUSIVE != 0)
        && !OBJECT_HEADER_TO_EXCLUSIVE_PROCESS(object_header).is_null()
    {
        status = STATUS_ACCESS_DENIED;
        ObpReleaseObjectLock(object_header);
        return status;
    }

    name_info = OBJECT_HEADER_TO_NAME_INFO(object_header);
    if !name_info.is_null()
        && ((*name_info).QueryReferences & OB_FLAG_KERNEL_EXCLUSIVE as u32 != 0)
        && probe_mode != 0i8
    {
        status = STATUS_ACCESS_DENIED;
        ObpReleaseObjectLock(object_header);
        return status;
    }

    if (*object_header).__bindgen_anon_1.HandleCount == 0
        && new_object == 0u8
        && (*object_type).TypeInfo.MaintainHandleCount != 0
        && (*object_type).TypeInfo.OpenProcedure.is_none()
        && (*object_type).TypeInfo.CloseProcedure.is_some()
    {
        status = STATUS_UNSUCCESSFUL;
        ObpReleaseObjectLock(object_header);
        return status;
    }

    if (open_reason == ObOpenHandle)
        || ((open_reason == ObDuplicateHandle) && !access_state.is_null())
    {
        if ObCheckObjectAccess(
            object,
            access_state,
            1u8,
            probe_mode,
            &mut status as *mut _,
        ) == 0
        {
            ObpReleaseObjectLock(object_header);
            return status;
        }
    } else if open_reason == ObCreateHandle {
        if !access_state.is_null() && (*access_state).RemainingDesiredAccess & MAXIMUM_ALLOWED != 0 {
            (*access_state).RemainingDesiredAccess &= !MAXIMUM_ALLOWED;
            (*access_state).RemainingDesiredAccess |= GENERIC_ALL;
        }

        if !access_state.is_null() && (*access_state).RemainingDesiredAccess & GENERIC_ACCESS != 0 {
            RtlMapGenericMask(
                core::ptr::addr_of_mut!((*access_state).RemainingDesiredAccess),
                core::ptr::addr_of_mut!((*object_type).TypeInfo.GenericMapping),
            );
        }

        if !access_state.is_null() && (*access_state).RemainingDesiredAccess & ACCESS_SYSTEM_SECURITY != 0 {
            // This block was translated EMPTY from upstream's `FIXME: TODO` +
            // "ACCESS_SYSTEM_SECURITY not validated!", so SACL access was handed to any
            // caller that asked for it. Upstream-Commit: c9780d9095f
            if SeSinglePrivilegeCheck(SeSecurityPrivilege, probe_mode) == 0 {
                /* FIXME: an audit alarm belongs here; the security manager must be told */
                ObpReleaseObjectLock(object_header);
                return STATUS_PRIVILEGE_NOT_HELD;
            }

            /* Privilege held — grant it so the access state reflects reality */
            (*access_state).PreviouslyGrantedAccess |= ACCESS_SYSTEM_SECURITY;
            (*access_state).RemainingDesiredAccess &= !ACCESS_SYSTEM_SECURITY;
        }
    }

    if exclusive != 0 {
        (*OBJECT_HEADER_TO_QUOTA_INFO(object_header)).ExclusiveProcess = process;
    }

    InterlockedIncrementSizeT(
        core::ptr::addr_of_mut!((*object_header).__bindgen_anon_1.HandleCount) as *mut isize
    );
    process_handle_count = 0;

    if (*object_type).TypeInfo.MaintainHandleCount != 0 {
        status = ObpIncrementHandleDataBase(
            object_header,
            process,
            &mut process_handle_count as *mut _,
        );
        if status < 0 {
            ObpReleaseObjectLock(object_header);
            return status;
        }
    }

    ObpReleaseObjectLock(object_header);

    status = STATUS_SUCCESS;
    if (*object_type).TypeInfo.OpenProcedure.is_some() {
        ObpCalloutStart(&mut callout_irql as *mut _);
        // GrantedAccess is an in/out POINTER since the merge's NT6 callback change, so
        // it needs a local the callee can write through — exactly as the C does.
        let mut granted_access: ACCESS_MASK = if !access_state.is_null() {
            (*access_state).PreviouslyGrantedAccess
        } else {
            0
        };
        status = crate::obsecure::ObpCallOpenProcedure(
            (*object_type).TypeInfo.OpenProcedure,
            open_reason,
            probe_mode,
            process,
            object,
            core::ptr::addr_of_mut!(granted_access),
            process_handle_count as ULONG,
        );
        ObpCalloutEnd(callout_irql, b"Open\0".as_ptr() as PCHAR, object_type, object);

        if status < 0 {
            return status;
        }
    }

    if open_reason == ObCreateHandle {
        creator_info = OBJECT_HEADER_TO_CREATOR_INFO(object_header);
        if !creator_info.is_null() {
            ObpEnterObjectTypeMutex(object_type);

            InsertTailList(
                core::ptr::addr_of_mut!((*object_type).TypeList) as PLIST_ENTRY,
                core::ptr::addr_of_mut!((*creator_info).TypeList) as PLIST_ENTRY,
            );

            ObpLeaveObjectTypeMutex(object_type);
        }
    }

    total = InterlockedIncrement(
        core::ptr::addr_of_mut!((*object_type).TotalNumberOfHandles) as *mut LONG
    ) as ULONG;
    if total > (*object_type).HighWaterNumberOfHandles {
        (*object_type).HighWaterNumberOfHandles = total;
    }

    status
}

unsafe fn ObpIncrementUnnamedHandleCount(
    object: PVOID,
    desired_access: *mut ACCESS_MASK,
    access_mode: KPROCESSOR_MODE,
    handle_attributes: ULONG,
    process: PEPROCESS,
) -> NTSTATUS {
    let object_header: POBJECT_HEADER;
    let object_type: POBJECT_TYPE;
    let mut process_handle_count: ULONG = 0;
    let mut status: NTSTATUS;
    let exclusive_process: PEPROCESS;
    let mut exclusive = 0u8;
    let mut new_object: BOOLEAN = 0u8;
    let creator_info: POBJECT_HEADER_CREATOR_INFO;
    let mut callout_irql: KIRQL = 0;
    let mut total: ULONG;

    object_header = OBJECT_TO_OBJECT_HEADER(object);
    object_type = (*object_header).Type;

    ObpAcquireObjectLock(object_header);

    status = ObpChargeQuotaForObject(object_header, object_type, &mut new_object as *mut _);
    if status < 0 {
        // Returning here with the object lock still held deadlocks every later handle
        // operation on this object. Upstream-Commit: 1902929c80b
        ObpReleaseObjectLock(object_header);
        return status;
    }

    if handle_attributes & OBJ_EXCLUSIVE != 0 {
        if (handle_attributes & OBJ_INHERIT != 0)
            || ((*object_header).Flags & OB_FLAG_EXCLUSIVE == 0)
        {
            status = STATUS_INVALID_PARAMETER;
            ObpReleaseObjectLock(object_header);
            return status;
        }

        exclusive_process = OBJECT_HEADER_TO_EXCLUSIVE_PROCESS(object_header);
        if (exclusive_process.is_null() && (*object_header).__bindgen_anon_1.HandleCount != 0)
            || (!exclusive_process.is_null() && exclusive_process != (PsGetCurrentProcess() as *mut _EPROCESS))
        {
            status = STATUS_ACCESS_DENIED;
            ObpReleaseObjectLock(object_header);
            return status;
        }

        exclusive = 1u8;
    } else if ((*object_header).Flags & OB_FLAG_EXCLUSIVE != 0)
        && !OBJECT_HEADER_TO_EXCLUSIVE_PROCESS(object_header).is_null()
    {
        status = STATUS_ACCESS_DENIED;
        ObpReleaseObjectLock(object_header);
        return status;
    }

    if (*object_header).__bindgen_anon_1.HandleCount == 0
        && new_object == 0u8
        && (*object_type).TypeInfo.MaintainHandleCount != 0
        && (*object_type).TypeInfo.OpenProcedure.is_none()
        && (*object_type).TypeInfo.CloseProcedure.is_some()
    {
        status = STATUS_UNSUCCESSFUL;
        ObpReleaseObjectLock(object_header);
        return status;
    }

    if *desired_access & MAXIMUM_ALLOWED != 0 {
        *desired_access &= !MAXIMUM_ALLOWED;
        *desired_access |= GENERIC_ALL;
    }

    if *desired_access & GENERIC_ACCESS != 0 {
        RtlMapGenericMask(
            desired_access,
            core::ptr::addr_of_mut!((*object_type).TypeInfo.GenericMapping),
        );
    }

    if exclusive != 0 {
        (*OBJECT_HEADER_TO_QUOTA_INFO(object_header)).ExclusiveProcess = process;
    }

    InterlockedIncrementSizeT(
        core::ptr::addr_of_mut!((*object_header).__bindgen_anon_1.HandleCount) as *mut isize
    );
    process_handle_count = 0;

    if (*object_type).TypeInfo.MaintainHandleCount != 0 {
        status = ObpIncrementHandleDataBase(
            object_header,
            process,
            &mut process_handle_count as *mut _,
        );
        if status < 0 {
            ObpReleaseObjectLock(object_header);
            return status;
        }
    }

    ObpReleaseObjectLock(object_header);

    status = STATUS_SUCCESS;
    if (*object_type).TypeInfo.OpenProcedure.is_some() {
        ObpCalloutStart(&mut callout_irql as *mut _);
        status = crate::obsecure::ObpCallOpenProcedure(
            (*object_type).TypeInfo.OpenProcedure,
            ObCreateHandle,
            access_mode,
            process,
            object,
            desired_access,
            process_handle_count as ULONG,
        );
        ObpCalloutEnd(callout_irql, b"Open\0".as_ptr() as PCHAR, object_type, object);

        if status < 0 {
            return status;
        }
    }

    creator_info = OBJECT_HEADER_TO_CREATOR_INFO(object_header);
    if !creator_info.is_null() {
        ObpEnterObjectTypeMutex(object_type);

        InsertTailList(
            core::ptr::addr_of_mut!((*object_type).TypeList) as PLIST_ENTRY,
            core::ptr::addr_of_mut!((*creator_info).TypeList) as PLIST_ENTRY,
        );

        ObpLeaveObjectTypeMutex(object_type);
    }

    total = InterlockedIncrement(
        core::ptr::addr_of_mut!((*object_type).TotalNumberOfHandles) as *mut LONG
    ) as ULONG;
    if total > (*object_type).HighWaterNumberOfHandles {
        (*object_type).HighWaterNumberOfHandles = total;
    }

    status
}

unsafe fn ObpCreateUnnamedHandle(
    object: PVOID,
    mut desired_access: ACCESS_MASK,
    additional_references: ULONG,
    handle_attributes: ULONG,
    access_mode: KPROCESSOR_MODE,
    returned_object: *mut PVOID,
    returned_handle: *mut HANDLE,
) -> NTSTATUS {
    let mut new_entry: _HANDLE_TABLE_ENTRY = core::mem::zeroed();
    let object_header: POBJECT_HEADER;
    let mut handle: HANDLE;
    let mut apc_state: _KAPC_STATE = core::mem::zeroed();
    let mut attached_to_process = 0u8;
    let mut kernel_handle = 0u8;
    let handle_table: PVOID;
    let mut status: NTSTATUS;
    let granted_access: ACCESS_MASK;
    let object_type: POBJECT_TYPE;

    object_header = OBJECT_TO_OBJECT_HEADER(object);
    object_type = (*object_header).Type;

    new_entry.__bindgen_anon_1.Object = object_header as PVOID;
    new_entry.__bindgen_anon_1.ObAttributes |= (handle_attributes & OBJ_HANDLE_ATTRIBUTES) as ObAttrType;

    if handle_attributes & OBJ_KERNEL_HANDLE != 0 {
        handle_table = ObpKernelHandleTable as PVOID;
        kernel_handle = 1u8;

        if PsGetCurrentProcess() as PVOID != PsInitialSystemProcess {
            KeStackAttachProcess(
                core::ptr::addr_of_mut!((*(PsInitialSystemProcess as *mut _EPROCESS)).Pcb),
                &mut apc_state as *mut _,
            );
            attached_to_process = 1u8;
        }
    } else {
        handle_table = (*(PsGetCurrentProcess() as *mut _EPROCESS)).ObjectTable as PVOID;
    }

    status = ObpIncrementUnnamedHandleCount(
        object,
        &mut desired_access as *mut _,
        access_mode,
        handle_attributes,
        PsGetCurrentProcess() as *mut _EPROCESS ,
    );
    if status < 0 {
        if attached_to_process != 0 {
            KeUnstackDetachProcess(&mut apc_state as *mut _);
        }
        return status;
    }

    granted_access = desired_access
        & ((*object_type).TypeInfo.ValidAccessMask | ACCESS_SYSTEM_SECURITY);

    if additional_references != 0 {
        InterlockedExchangeAddSizeT(
            core::ptr::addr_of_mut!((*object_header).PointerCount) as *mut isize,
            additional_references as isize,
        );
    }

    new_entry.__bindgen_anon_2.GrantedAccess = granted_access;

    handle = ExCreateHandle(handle_table as PHANDLE_TABLE, &mut new_entry as *mut _);

    if !handle.is_null() {
        if kernel_handle != 0 {
            handle = ObMarkHandleAsKernelHandle(handle);
        }

        *returned_handle = handle;

        if (additional_references != 0) && !returned_object.is_null() {
            *returned_object = object;
        }

        if attached_to_process != 0 {
            KeUnstackDetachProcess(&mut apc_state as *mut _);
        }

        return STATUS_SUCCESS;
    }

    if additional_references != 0 {
        InterlockedExchangeAddSizeT(
            core::ptr::addr_of_mut!((*object_header).PointerCount) as *mut isize,
            -(additional_references as LONG) as isize,
        );
    }

    ObpDecrementHandleCount(
        core::ptr::addr_of_mut!((*object_header).Body) as PVOID,
        PsGetCurrentProcess() as *mut _EPROCESS ,
        granted_access,
        object_type,
    );

    if attached_to_process != 0 {
        KeUnstackDetachProcess(&mut apc_state as *mut _);
    }
    STATUS_INSUFFICIENT_RESOURCES
}

unsafe fn ObpCreateHandle(
    open_reason: OB_OPEN_REASON,
    object: PVOID,
    type_: POBJECT_TYPE,
    access_state: PACCESS_STATE,
    additional_references: ULONG,
    mut handle_attributes: ULONG,
    context: POBP_LOOKUP_CONTEXT,
    access_mode: KPROCESSOR_MODE,
    returned_object: *mut PVOID,
    returned_handle: *mut HANDLE,
) -> NTSTATUS {
    let mut new_entry: _HANDLE_TABLE_ENTRY = core::mem::zeroed();
    let object_header: POBJECT_HEADER;
    let mut handle: HANDLE;
    let mut apc_state: _KAPC_STATE = core::mem::zeroed();
    let mut attached_to_process = 0u8;
    let mut kernel_handle = 0u8;
    let object_type: POBJECT_TYPE;
    let handle_table: PVOID;
    let mut status: NTSTATUS;
    let mut desired_access: ACCESS_MASK;
    let mut granted_access: ACCESS_MASK;
    let aux_data: PAUX_ACCESS_DATA;

    object_header = OBJECT_TO_OBJECT_HEADER(object);
    object_type = (*object_header).Type;

    if !type_.is_null() && object_type != type_ {
        if !context.is_null() {
            ObpReleaseLookupContext(context);
        }
        return STATUS_OBJECT_TYPE_MISMATCH;
    }

    new_entry.__bindgen_anon_1.Object = object_header as PVOID;

    if handle_attributes & OBJ_KERNEL_HANDLE != 0 {
        handle_table = ObpKernelHandleTable as PVOID;
        kernel_handle = 1u8;

        if PsGetCurrentProcess() as PVOID != PsInitialSystemProcess {
            KeStackAttachProcess(
                core::ptr::addr_of_mut!((*(PsInitialSystemProcess as *mut _EPROCESS)).Pcb),
                &mut apc_state as *mut _,
            );
            attached_to_process = 1u8;
        }
    } else {
        handle_table = (*(PsGetCurrentProcess() as *mut _EPROCESS)).ObjectTable as PVOID;
    }

    status = ObpIncrementHandleCount(
        object,
        access_state,
        access_mode,
        handle_attributes,
        PsGetCurrentProcess() as *mut _EPROCESS ,
        open_reason,
    );
    if status < 0 {
        if !context.is_null() {
            ObpReleaseLookupContext(context);
        }
        if attached_to_process != 0 {
            KeUnstackDetachProcess(&mut apc_state as *mut _);
        }
        return status;
    }

    if (*access_state).GenerateOnClose != 0 {
        handle_attributes |= OBJ_AUDIT_OBJECT_CLOSE;
    }

    new_entry.__bindgen_anon_1.ObAttributes |= (handle_attributes & OBJ_HANDLE_ATTRIBUTES) as ObAttrType;

    desired_access = (*access_state).RemainingDesiredAccess
        | (*access_state).PreviouslyGrantedAccess;

    granted_access = desired_access
        & ((*object_type).TypeInfo.ValidAccessMask | ACCESS_SYSTEM_SECURITY);

    (*access_state).PreviouslyGrantedAccess = granted_access;

    aux_data = (*access_state).AuxData as PAUX_ACCESS_DATA;

    if additional_references != 0 {
        InterlockedExchangeAddSizeT(
            core::ptr::addr_of_mut!((*object_header).PointerCount) as *mut isize,
            additional_references as isize,
        );
    }

    if !context.is_null() {
        ObpReleaseLookupContext(context);
    }

    new_entry.__bindgen_anon_2.GrantedAccess = granted_access;

    handle = ExCreateHandle(handle_table as PHANDLE_TABLE, &mut new_entry as *mut _);

    if !handle.is_null() {
        if kernel_handle != 0 {
            handle = ObMarkHandleAsKernelHandle(handle);
        }

        *returned_handle = handle;

        if (*access_state).GenerateAudit != 0 {
        }

        if open_reason == ObCreateHandle {
            if !aux_data.is_null()
                && !(*aux_data).PrivilegesUsed.is_null()
                && (*(*aux_data).PrivilegesUsed).PrivilegeCount != 0
            {
            }
        }

        if (additional_references != 0) && !returned_object.is_null() {
            *returned_object = object;
        }

        if attached_to_process != 0 {
            KeUnstackDetachProcess(&mut apc_state as *mut _);
        }

        return STATUS_SUCCESS;
    }

    ObpDecrementHandleCount(
        core::ptr::addr_of_mut!((*object_header).Body) as PVOID,
        PsGetCurrentProcess() as *mut _EPROCESS ,
        granted_access,
        object_type,
    );

    if additional_references != 0 {
        if additional_references > 1 {
            InterlockedExchangeAddSizeT(
                core::ptr::addr_of_mut!((*object_header).PointerCount) as *mut isize,
                -((additional_references - 1) as LONG) as isize,
            );
        }

        ObDereferenceObject(object);
    }

    if attached_to_process != 0 {
        KeUnstackDetachProcess(&mut apc_state as *mut _);
    }
    STATUS_INSUFFICIENT_RESOURCES
}

unsafe fn ObpCloseHandle(mut handle: HANDLE, access_mode: KPROCESSOR_MODE) -> NTSTATUS {
    let handle_table: PVOID;
    let mut attached_to_process = 0u8;
    let mut apc_state: _KAPC_STATE = core::mem::zeroed();
    let handle_table_entry: PHANDLE_TABLE_ENTRY;
    let mut status: NTSTATUS;
    let process = PsGetCurrentProcess() as *mut _EPROCESS ;

    if access_mode == 0i8 && handle == (-1isize) as HANDLE {
        return STATUS_INVALID_HANDLE;
    }

    if ObpIsKernelHandle(handle, access_mode) != 0 {
        handle_table = ObpKernelHandleTable as PVOID;
        handle = ObKernelHandleToHandle(handle);

        if process as PVOID != PsInitialSystemProcess {
            KeStackAttachProcess(
                core::ptr::addr_of_mut!((*(PsInitialSystemProcess as *mut _EPROCESS)).Pcb),
                &mut apc_state as *mut _,
            );
            attached_to_process = 1u8;
        }
    } else {
        handle_table = (*(process as *mut _EPROCESS)).ObjectTable as PVOID;
    }

    KeEnterCriticalRegion();

    handle_table_entry = ExMapHandleToPointer(handle_table as PHANDLE_TABLE, handle);
    if !handle_table_entry.is_null() {
        status = ObpCloseHandleTableEntry(
            handle_table as PHANDLE_TABLE,
            handle_table_entry,
            handle,
            access_mode,
            0u8,
        );

        KeLeaveCriticalRegion();

        if attached_to_process != 0 {
            KeUnstackDetachProcess(&mut apc_state as *mut _);
        }
    } else {
        KeLeaveCriticalRegion();

        if attached_to_process != 0 {
            KeUnstackDetachProcess(&mut apc_state as *mut _);
        }

        if !handle.is_null()
            && handle != NtCurrentProcess()
            && handle != NtCurrentThread()
        {
            if access_mode != 0i8 {
                if !(*(process as *mut _EPROCESS)).DebugPort.is_null() {
                    if KeIsAttachedProcess() == 0 {
                        return KeRaiseUserException(STATUS_INVALID_HANDLE);
                    }
                }
            } else {
                if PsIsThreadTerminating(PsGetCurrentThread()) == 0
                    && !(*(process as *mut _EPROCESS)).Peb.is_null()
                {
                    if KdDebuggerEnabled != 0 {
                        KeBugCheckEx(INVALID_KERNEL_HANDLE, handle as ULONG_PTR, 1, 0, 0);
                    }
                }
            }
        }

        status = STATUS_INVALID_HANDLE;
    }

    status
}

unsafe fn _ObpSetHandleAttributes(
    handle_table_entry: PHANDLE_TABLE_ENTRY,
    context: ULONG_PTR,
) -> BOOLEAN {
    let set_handle_info = context as PVOID as POBP_SET_HANDLE_ATTRIBUTES_CONTEXT;
    let object_header = ObpGetHandleObject(handle_table_entry);

    if (*set_handle_info).Information.Inherit != 0 {
        if (*(*object_header).Type).TypeInfo.InvalidAttributes & OBJ_INHERIT != 0 {
            return 0u8;
        }

        (*handle_table_entry).__bindgen_anon_1.ObAttributes |= OBJ_INHERIT as ObAttrType;
    } else {
        (*handle_table_entry).__bindgen_anon_1.ObAttributes &= !(OBJ_INHERIT as ObAttrType);
    }

    if (*set_handle_info).Information.ProtectFromClose != 0 {
        (*handle_table_entry).__bindgen_anon_2.GrantedAccess |= ObpAccessProtectCloseBit;
    } else {
        (*handle_table_entry).__bindgen_anon_2.GrantedAccess &= !ObpAccessProtectCloseBit;
    }

    1u8
}

unsafe fn _ObpCloseHandleCallback(
    handle_table_entry: PHANDLE_TABLE_ENTRY,
    handle: HANDLE,
    context: PVOID,
) -> BOOLEAN {
    let close_context = context as POBP_CLOSE_HANDLE_CONTEXT;

    ObpCloseHandleTableEntry(
        (*close_context).HandleTable,
        handle_table_entry,
        handle,
        (*close_context).AccessMode,
        1u8,
    );
    1u8
}

unsafe fn _ObpDuplicateHandleCallback(
    process: PEPROCESS,
    handle_table: PHANDLE_TABLE,
    old_entry: PHANDLE_TABLE_ENTRY,
    handle_table_entry: PHANDLE_TABLE_ENTRY,
) -> BOOLEAN {
    let object_header: POBJECT_HEADER;
    let mut ret = 0u8;
    let mut access_state: _ACCESS_STATE = core::mem::zeroed();
    let mut status: NTSTATUS;

    ret = ((*handle_table_entry).__bindgen_anon_1.ObAttributes & OBJ_INHERIT as ObAttrType != 0) as u8;
    if ret != 0 {
        object_header = ObpGetHandleObject(handle_table_entry);

        InterlockedIncrementSizeT(
            core::ptr::addr_of_mut!((*object_header).PointerCount) as *mut isize
        );

        ExUnlockHandleTableEntry(handle_table, old_entry);

        access_state.PreviouslyGrantedAccess = (*handle_table_entry).__bindgen_anon_2.GrantedAccess;

        status = ObpIncrementHandleCount(
            core::ptr::addr_of_mut!((*object_header).Body) as PVOID,
            &mut access_state as *mut _,
            0i8,
            ((*handle_table_entry).__bindgen_anon_1.ObAttributes as ULONG) & OBJ_HANDLE_ATTRIBUTES,
            process,
            ObInheritHandle,
        );
        if status < 0 {
            ObDereferenceObject(core::ptr::addr_of_mut!((*object_header).Body) as PVOID);
            ret = 0u8;
        }
    } else {
        ExUnlockHandleTableEntry(handle_table, old_entry);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn ObClearProcessHandleTable(process: PEPROCESS) {
    let process = opaque_ptr(process);
    let handle_table: PHANDLE_TABLE;
    let mut context: _OBP_CLOSE_HANDLE_CONTEXT = core::mem::zeroed();
    let mut apc_state: _KAPC_STATE = core::mem::zeroed();
    let mut attached_to_process = 0u8;

    handle_table = ObReferenceProcessHandleTable(process);
    if handle_table.is_null() {
        return;
    }

    if (PsGetCurrentProcess() as *mut _EPROCESS) != process {
        KeStackAttachProcess(
            core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).Pcb),
            &mut apc_state as *mut _,
        );
        attached_to_process = 1u8;
    }

    KeEnterCriticalRegion();

    context.AccessMode = 1i8;
    context.HandleTable = handle_table;

    ExSweepHandleTable(
        handle_table,
        Some(ObpCloseHandleCallback),
        &mut context as *mut _ as PVOID,
    );

    KeLeaveCriticalRegion();

    if attached_to_process != 0 {
        KeUnstackDetachProcess(&mut apc_state as *mut _);
    }

    ObDereferenceProcessHandleTable(process);
}

#[no_mangle]
pub unsafe extern "system" fn ObInitProcess(
    parent: PEPROCESS,
    process: PEPROCESS,
) -> NTSTATUS {
    let parent = opaque_ptr(parent);
    let process = opaque_ptr(process);
    let parent_table: PHANDLE_TABLE;
    let object_table: PHANDLE_TABLE;

    if !parent.is_null() {
        parent_table = ObReferenceProcessHandleTable(parent);
        if parent_table.is_null() {
            return STATUS_PROCESS_IS_TERMINATING;
        }

        object_table = ExDupHandleTable(
            process,
            parent_table,
            Some(ObpDuplicateHandleCallback),
            OBJ_INHERIT as _,
        );
    } else {
        parent_table = core::ptr::null_mut();
        object_table = ExCreateHandleTable(process);
    }

    if !object_table.is_null() {
        (*(process as *mut _EPROCESS)).ObjectTable = object_table;

        if SeDetailedAuditingWithToken(core::ptr::null_mut()) != 0 {
        }

        if !parent_table.is_null() {
            ObDereferenceProcessHandleTable(parent);
        }

        STATUS_SUCCESS
    } else {
        (*(process as *mut _EPROCESS)).ObjectTable = core::ptr::null_mut();
        if !parent_table.is_null() {
            ObDereferenceProcessHandleTable(parent);
        }
        STATUS_INSUFFICIENT_RESOURCES
    }
}

#[no_mangle]
pub unsafe extern "system" fn ObKillProcess(process: PEPROCESS) {
    let process = opaque_ptr(process);
    let handle_table: PHANDLE_TABLE;
    let mut context: _OBP_CLOSE_HANDLE_CONTEXT = core::mem::zeroed();
    let mut hard_errors: BOOLEAN;

    ExWaitForRundownProtectionRelease(
        core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).RundownProtect),
    );
    ExRundownCompleted(
        core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).RundownProtect),
    );

    handle_table = (*(process as *mut _EPROCESS)).ObjectTable;
    if handle_table.is_null() {
        return;
    }

    hard_errors = IoSetThreadHardErrorMode(0u8);

    KeEnterCriticalRegion();

    context.AccessMode = 0i8;
    context.HandleTable = handle_table;

    ExSweepHandleTable(
        handle_table,
        Some(ObpCloseHandleCallback),
        &mut context as *mut _ as PVOID,
    );

    KeLeaveCriticalRegion();

    IoSetThreadHardErrorMode(hard_errors);

    (*(process as *mut _EPROCESS)).ObjectTable = core::ptr::null_mut();
    ExDestroyHandleTable(handle_table, core::ptr::null_mut());
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
#[no_mangle]
pub unsafe extern "system" fn ObDuplicateObject(
    source_process: PEPROCESS,
    source_handle: HANDLE,
    target_process: PEPROCESS,
    target_handle: *mut HANDLE,
    mut desired_access: ACCESS_MASK,
    mut handle_attributes: ULONG,
    options: ULONG,
    previous_mode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let source_process = opaque_ptr(source_process);
    let source_handle = opaque_ptr(source_handle);
    let target_process = opaque_ptr(target_process);
    let target_handle = opaque_ptr(target_handle);
    let mut new_handle_entry: _HANDLE_TABLE_ENTRY = core::mem::zeroed();
    let mut attached_to_process = 0u8;
    let mut source_object: PVOID = core::ptr::null_mut();
    let object_header: POBJECT_HEADER;
    let object_type: POBJECT_TYPE;
    let mut new_handle: HANDLE;
    let mut apc_state: _KAPC_STATE = core::mem::zeroed();
    let mut status: NTSTATUS;
    let mut target_access: ACCESS_MASK;
    let source_access: ACCESS_MASK;
    let mut access_state: _ACCESS_STATE = core::mem::zeroed();
    let mut passed_access_state: PACCESS_STATE = core::ptr::null_mut();
    let mut aux_data: _AUX_ACCESS_DATA = core::mem::zeroed();
    let mut handle_table: PHANDLE_TABLE;
    let mut handle_information: _OBJECT_HANDLE_INFORMATION = core::mem::zeroed();
    let mut audit_mask: ULONG = 0;
    let mut kernel_handle = 0u8;

    if !target_handle.is_null() {
        *target_handle = core::ptr::null_mut();
    }

    if options & DUPLICATE_SAME_ACCESS == 0 {
        status = STATUS_SUCCESS;
        if status < 0 {
            return status;
        }
    }

    handle_table = ObReferenceProcessHandleTable(source_process);
    if handle_table.is_null() {
        return STATUS_PROCESS_IS_TERMINATING;
    }

    status = ObpReferenceProcessObjectByHandle(
        source_handle,
        source_process,
        handle_table,
        previous_mode,
        &mut source_object as *mut _,
        &mut handle_information as *mut _,
        &mut audit_mask as *mut _,
    );
    if status < 0 {
        ObDereferenceProcessHandleTable(source_process);
        return status;
    } else {
        if handle_information.HandleAttributes & OBJ_AUDIT_OBJECT_CLOSE == 0 {
            audit_mask = 0;
        }
    }

    if target_process.is_null() {
        if options & DUPLICATE_CLOSE_SOURCE == 0 {
            status = STATUS_INVALID_PARAMETER;
        } else {
            KeStackAttachProcess(
                core::ptr::addr_of_mut!((*(source_process as *mut _EPROCESS)).Pcb),
                &mut apc_state as *mut _,
            );

            NtClose(source_handle);
            KeUnstackDetachProcess(&mut apc_state as *mut _);
        }

        ObDereferenceProcessHandleTable(source_process);
        ObDereferenceObject(source_object);
        return status;
    }

    if previous_mode == 0i8
        && handle_attributes & OBJ_KERNEL_HANDLE != 0
        && target_process as PVOID == PsInitialSystemProcess
    {
        kernel_handle = 1u8;
    }

    handle_table = ObReferenceProcessHandleTable(target_process);
    if handle_table.is_null() {
        if options & DUPLICATE_CLOSE_SOURCE != 0 {
            KeStackAttachProcess(
                core::ptr::addr_of_mut!((*(source_process as *mut _EPROCESS)).Pcb),
                &mut apc_state as *mut _,
            );

            NtClose(source_handle);
            KeUnstackDetachProcess(&mut apc_state as *mut _);
        }

        ObDereferenceProcessHandleTable(source_process);
        ObDereferenceObject(source_object);
        return STATUS_PROCESS_IS_TERMINATING;
    }

    source_access = handle_information.GrantedAccess;

    if target_process != (PsGetCurrentProcess() as *mut _EPROCESS) {
        KeStackAttachProcess(
            core::ptr::addr_of_mut!((*(target_process as *mut _EPROCESS)).Pcb),
            &mut apc_state as *mut _,
        );
        attached_to_process = 1u8;
    }

    if options & DUPLICATE_SAME_ATTRIBUTES != 0 {
        handle_attributes = handle_information.HandleAttributes;
    } else {
        handle_attributes |= handle_information.HandleAttributes
            & OBJ_AUDIT_OBJECT_CLOSE;
    }

    if options & DUPLICATE_SAME_ACCESS != 0 {
        desired_access = source_access;
    }

    object_header = OBJECT_TO_OBJECT_HEADER(source_object);
    object_type = (*object_header).Type;

    core::ptr::write_bytes(&mut new_handle_entry as *mut _ as *mut u8, 0, core::mem::size_of::<_HANDLE_TABLE_ENTRY>());
    new_handle_entry.__bindgen_anon_1.Object = object_header as PVOID;
    new_handle_entry.__bindgen_anon_1.ObAttributes |= (handle_attributes & OBJ_HANDLE_ATTRIBUTES) as ObAttrType;

    if desired_access & GENERIC_ACCESS != 0 {
        RtlMapGenericMask(
            &mut desired_access as *mut _,
            core::ptr::addr_of_mut!((*object_type).TypeInfo.GenericMapping),
        );
    }

    target_access = desired_access
        & ((*object_type).TypeInfo.ValidAccessMask | ACCESS_SYSTEM_SECURITY);
    new_handle_entry.__bindgen_anon_2.GrantedAccess = target_access;

    if target_access & !source_access != 0 {
        if (*object_type).TypeInfo.SecurityProcedure
            == Some(core::mem::transmute(
                SeDefaultObjectMethod as unsafe extern "system" fn(_, _, _, _, _, _, _, _, _) -> NTSTATUS,
            ))
        {
            passed_access_state = &mut access_state as *mut _;
            status = SeCreateAccessState(
                &mut access_state as *mut _,
                &mut aux_data as *mut _,
                target_access,
                core::ptr::addr_of_mut!((*object_type).TypeInfo.GenericMapping),
            );
        } else {
            status = STATUS_ACCESS_DENIED;
        }
    } else {
        status = STATUS_SUCCESS;
    }

    if status >= 0 {
        status = ObpIncrementHandleCount(
            source_object,
            passed_access_state,
            previous_mode,
            handle_attributes,
            PsGetCurrentProcess() as *mut _EPROCESS ,
            ObDuplicateHandle,
        );
    }

    if attached_to_process != 0 {
        KeUnstackDetachProcess(&mut apc_state as *mut _);
        attached_to_process = 0u8;
    }

    if options & DUPLICATE_CLOSE_SOURCE != 0 {
        KeStackAttachProcess(
            core::ptr::addr_of_mut!((*(source_process as *mut _EPROCESS)).Pcb),
            &mut apc_state as *mut _,
        );
        NtClose(source_handle);
        KeUnstackDetachProcess(&mut apc_state as *mut _);
    }

    if !passed_access_state.is_null() {
        SeDeleteAccessState(passed_access_state);
    }

    if status < 0 {
        ObDereferenceProcessHandleTable(source_process);
        ObDereferenceProcessHandleTable(target_process);

        ObDereferenceObject(source_object);
        return status;
    }

    if new_handle_entry.__bindgen_anon_1.ObAttributes & OBJ_PROTECT_CLOSE as ObAttrType != 0 {
        new_handle_entry.__bindgen_anon_1.ObAttributes &= !(OBJ_PROTECT_CLOSE as ObAttrType);
        new_handle_entry.__bindgen_anon_2.GrantedAccess |= ObpAccessProtectCloseBit;
    }

    new_handle = ExCreateHandle(handle_table, &mut new_handle_entry as *mut _);
    if new_handle.is_null() {
        ObpDecrementHandleCount(
            source_object,
            target_process,
            target_access,
            object_type,
        );

        ObDereferenceObject(source_object);
        status = STATUS_INSUFFICIENT_RESOURCES;
    }

    if kernel_handle != 0 {
        new_handle = ObMarkHandleAsKernelHandle(new_handle);
    }

    if !target_handle.is_null() {
        *target_handle = new_handle;
    }

    ObDereferenceProcessHandleTable(source_process);
    ObDereferenceProcessHandleTable(target_process);

    status
}

#[no_mangle]
pub unsafe extern "system" fn ObOpenObjectByName(
    object_attributes: POBJECT_ATTRIBUTES,
    object_type: POBJECT_TYPE,
    access_mode: KPROCESSOR_MODE,
    mut passed_access_state: PACCESS_STATE,
    desired_access: ACCESS_MASK,
    parse_context: PVOID,
    handle: *mut HANDLE,
) -> NTSTATUS {
    let object_attributes = opaque_ptr(object_attributes);
    let object_type = opaque_ptr(object_type);
    let mut passed_access_state = opaque_ptr(passed_access_state);
    let parse_context = opaque_ptr(parse_context);
    let mut handle = opaque_ptr(handle);
    let mut object = core::ptr::null_mut();
    let mut object_name: _UNICODE_STRING = core::mem::zeroed();
    let mut status: NTSTATUS;
    let mut status2: NTSTATUS;
    let object_header: POBJECT_HEADER;
    let mut generic_mapping: PGENERIC_MAPPING = core::ptr::null_mut();
    let open_reason: OB_OPEN_REASON;
    let temp_buffer: POB_TEMP_BUFFER;

    *handle = core::ptr::null_mut();

    if object_attributes.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    temp_buffer = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_OB_TEMP_BUFFER>(),
        TAG_OB_TEMP_STORAGE,
    ) as POB_TEMP_BUFFER;
    if temp_buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    status = ObpCaptureObjectCreateInformation(
        object_attributes,
        access_mode,
        access_mode,
        1u8,
        &mut (*temp_buffer).ObjectCreateInfo as *mut _,
        &mut object_name as *mut _,
    );
    if status < 0 {
        ExFreePoolWithTag(temp_buffer as PVOID, TAG_OB_TEMP_STORAGE);
        return status;
    }

    if passed_access_state.is_null() {
        if !object_type.is_null() {
            generic_mapping = core::ptr::addr_of_mut!((*object_type).TypeInfo.GenericMapping);
        }

        passed_access_state = &mut (*temp_buffer).LocalAccessState as *mut _;
        status = SeCreateAccessState(
            &mut (*temp_buffer).LocalAccessState as *mut _,
            &mut (*temp_buffer).AuxData as *mut _,
            desired_access,
            generic_mapping,
        );
        if status < 0 {
            ObpReleaseObjectCreateInformation(&mut (*temp_buffer).ObjectCreateInfo as *mut _);
            if !object_name.Buffer.is_null() {
                ObpFreeObjectNameBuffer(&mut object_name as *mut _);
            }
            ExFreePoolWithTag(temp_buffer as PVOID, TAG_OB_TEMP_STORAGE);
            return status;
        }
    }

    if !(*temp_buffer).ObjectCreateInfo.SecurityDescriptor.is_null() {
        (*passed_access_state).SecurityDescriptor =
            (*temp_buffer).ObjectCreateInfo.SecurityDescriptor;
    }

    status = ObpValidateAccessMask(passed_access_state);
    if status < 0 {
        ObpReleaseLookupContext(&mut (*temp_buffer).LookupContext as *mut _);
        if passed_access_state == &mut (*temp_buffer).LocalAccessState as *mut _ {
            SeDeleteAccessState(passed_access_state);
        }
        ObpReleaseObjectCreateInformation(&mut (*temp_buffer).ObjectCreateInfo as *mut _);
        if !object_name.Buffer.is_null() {
            ObpFreeObjectNameBuffer(&mut object_name as *mut _);
        }
        ExFreePoolWithTag(temp_buffer as PVOID, TAG_OB_TEMP_STORAGE);
        return status;
    }

    status = ObpLookupObjectName(
        (*temp_buffer).ObjectCreateInfo.RootDirectory,
        &mut object_name as *mut _,
        (*temp_buffer).ObjectCreateInfo.Attributes,
        object_type,
        access_mode,
        parse_context,
        (*temp_buffer).ObjectCreateInfo.SecurityQos,
        core::ptr::null_mut(),
        passed_access_state,
        &mut (*temp_buffer).LookupContext as *mut _,
        &mut object as *mut _,
    );
    if status < 0 {
        ObpReleaseLookupContext(&mut (*temp_buffer).LookupContext as *mut _);
        if passed_access_state == &mut (*temp_buffer).LocalAccessState as *mut _ {
            SeDeleteAccessState(passed_access_state);
        }
        ObpReleaseObjectCreateInformation(&mut (*temp_buffer).ObjectCreateInfo as *mut _);
        if !object_name.Buffer.is_null() {
            ObpFreeObjectNameBuffer(&mut object_name as *mut _);
        }
        ExFreePoolWithTag(temp_buffer as PVOID, TAG_OB_TEMP_STORAGE);
        return status;
    }

    object_header = OBJECT_TO_OBJECT_HEADER(object);
    if (*object_header).Flags & OB_FLAG_CREATE_INFO != 0 {
        open_reason = ObCreateHandle;

        if !(*object_header).__bindgen_anon_2.ObjectCreateInfo.is_null() {
            ObpFreeObjectCreateInformation((*object_header).__bindgen_anon_2.ObjectCreateInfo);
            (*object_header).__bindgen_anon_2.ObjectCreateInfo = core::ptr::null_mut();
        }
    } else {
        open_reason = ObOpenHandle;
    }

    if (*(*object_header).Type).TypeInfo.InvalidAttributes
        & (*temp_buffer).ObjectCreateInfo.Attributes
        != 0
    {
        status = STATUS_INVALID_PARAMETER;

        ObpReleaseLookupContext(&mut (*temp_buffer).LookupContext as *mut _);
        ObDereferenceObject(object);
    } else {
        status2 = ObpCreateHandle(
            open_reason,
            object,
            object_type,
            passed_access_state,
            0,
            (*temp_buffer).ObjectCreateInfo.Attributes,
            &mut (*temp_buffer).LookupContext as *mut _,
            access_mode,
            core::ptr::null_mut(),
            handle,
        );
        if status2 < 0 {
            ObDereferenceObject(object);
            status = status2;
        }
    }

    if passed_access_state == &mut (*temp_buffer).LocalAccessState as *mut _ {
        SeDeleteAccessState(passed_access_state);
    }

    ObpReleaseObjectCreateInformation(&mut (*temp_buffer).ObjectCreateInfo as *mut _);
    if !object_name.Buffer.is_null() {
        ObpFreeObjectNameBuffer(&mut object_name as *mut _);
    }
    ExFreePoolWithTag(temp_buffer as PVOID, TAG_OB_TEMP_STORAGE);

    status
}

#[no_mangle]
pub unsafe extern "system" fn ObOpenObjectByPointer(
    object: PVOID,
    handle_attributes: ULONG,
    mut passed_access_state: PACCESS_STATE,
    desired_access: ACCESS_MASK,
    object_type: POBJECT_TYPE,
    access_mode: KPROCESSOR_MODE,
    handle: *mut HANDLE,
) -> NTSTATUS {
    let object = opaque_ptr(object);
    let mut passed_access_state = opaque_ptr(passed_access_state);
    let object_type = opaque_ptr(object_type);
    let mut handle = opaque_ptr(handle);
    let header: POBJECT_HEADER;
    let mut status: NTSTATUS;
    let mut access_state: _ACCESS_STATE = core::mem::zeroed();
    let mut aux_data: _AUX_ACCESS_DATA = core::mem::zeroed();

    *handle = core::ptr::null_mut();

    status = ObReferenceObjectByPointer(object, 0, object_type, access_mode);
    if status < 0 {
        return status;
    }

    header = OBJECT_TO_OBJECT_HEADER(object);

    if passed_access_state.is_null() {
        passed_access_state = &mut access_state as *mut _;
        status = SeCreateAccessState(
            &mut access_state as *mut _,
            &mut aux_data as *mut _,
            desired_access,
            core::ptr::addr_of_mut!((*(*header).Type).TypeInfo.GenericMapping),
        );
        if status < 0 {
            ObDereferenceObject(object);
            return status;
        }
    }

    if (*(*header).Type).TypeInfo.InvalidAttributes & handle_attributes != 0 {
        if passed_access_state == &mut access_state as *mut _ {
            SeDeleteAccessState(passed_access_state);
        }

        ObDereferenceObject(object);
        return STATUS_INVALID_PARAMETER;
    }

    status = ObpCreateHandle(
        ObOpenHandle,
        object,
        object_type,
        passed_access_state,
        0,
        handle_attributes,
        core::ptr::null_mut(),
        access_mode,
        core::ptr::null_mut(),
        handle,
    );
    if status < 0 {
        ObDereferenceObject(object);
    }

    if passed_access_state == &mut access_state as *mut _ {
        SeDeleteAccessState(passed_access_state);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn ObFindHandleForObject(
    process: PEPROCESS,
    object: PVOID,
    object_type: POBJECT_TYPE,
    handle_information: POBJECT_HANDLE_INFORMATION,
    handle: *mut HANDLE,
) -> BOOLEAN {
    let process = opaque_ptr(process);
    let object = opaque_ptr(object);
    let object_type = opaque_ptr(object_type);
    let mut handle = opaque_ptr(handle);
    let mut find_data: _OBP_FIND_HANDLE_DATA = core::mem::zeroed();
    let mut result = 0u8;
    let object_table: PVOID;

    object_table = ObReferenceProcessHandleTable(process) as PVOID;
    if !object_table.is_null() {
        if !object.is_null() {
            find_data.ObjectHeader = OBJECT_TO_OBJECT_HEADER(object);
        } else {
            find_data.ObjectHeader = core::ptr::null_mut();
        }

        find_data.ObjectType = object_type;
        find_data.HandleInformation = handle_information;

        if ExEnumHandleTable(
            (*(process as *mut _EPROCESS)).ObjectTable,
            Some(ObpEnumFindHandleProcedure),
            &mut find_data as *mut _ as PVOID,
            handle,
        ) != 0
        {
            result = 1u8;
        }

        ObDereferenceProcessHandleTable(process);
    }

    result
}

#[no_mangle]
pub unsafe extern "system" fn ObInsertObject(
    object: PVOID,
    mut access_state: PACCESS_STATE,
    desired_access: ACCESS_MASK,
    object_pointer_bias: ULONG,
    new_object: *mut PVOID,
    handle: *mut HANDLE,
) -> NTSTATUS {
    let object = opaque_ptr(object);
    let mut access_state = opaque_ptr(access_state);
    let new_object = opaque_ptr(new_object);
    let mut handle = opaque_ptr(handle);
    let object_create_info: POBJECT_CREATE_INFORMATION;
    let object_header: POBJECT_HEADER;
    let object_type: POBJECT_TYPE;
    let mut object_name: PUNICODE_STRING;
    let mut insert_object: PVOID;
    let mut parent_descriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut sd_allocated = 0u8;
    let object_name_info: POBJECT_HEADER_NAME_INFO;
    let mut context: _OBP_LOOKUP_CONTEXT = core::mem::zeroed();
    let mut local_access_state: _ACCESS_STATE = core::mem::zeroed();
    let mut aux_data: _AUX_ACCESS_DATA = core::mem::zeroed();
    let mut open_reason: OB_OPEN_REASON;
    let previous_mode: KPROCESSOR_MODE;
    let mut status = STATUS_SUCCESS;
    let mut real_status: NTSTATUS;
    let mut is_new_object: BOOLEAN = 0u8;

    object_header = OBJECT_TO_OBJECT_HEADER(object);

    if (*object_header).Flags & OB_FLAG_CREATE_INFO == 0 {
        DbgBreakPoint();

        ObDereferenceObject(object);
        return STATUS_INVALID_PARAMETER;
    }

    object_create_info = (*object_header).__bindgen_anon_2.ObjectCreateInfo;
    object_name_info = ObpReferenceNameInfo(object_header);
    object_type = (*object_header).Type;


    object_name = core::ptr::null_mut();

    if !object_name_info.is_null() && !(*object_name_info).Name.Buffer.is_null() {
        object_name = &mut (*object_name_info).Name as *mut _;
    }

    previous_mode = KeGetPreviousMode();
    if ((*object_type).TypeInfo.SecurityRequired == 0) && object_name.is_null() {
        *handle = core::ptr::null_mut();
        (*object_header).__bindgen_anon_2.ObjectCreateInfo = core::ptr::null_mut();

        status = ObpCreateUnnamedHandle(
            object,
            desired_access,
            object_pointer_bias + 1,
            (*object_create_info).Attributes,
            previous_mode,
            new_object,
            handle,
        );

        ObpFreeObjectCreateInformation(object_create_info);
        ObpDereferenceNameInfo(object_name_info);
        ObDereferenceObject(object);

        return status;
    }

    if access_state.is_null() {
        access_state = &mut local_access_state as *mut _;
        status = SeCreateAccessState(
            &mut local_access_state as *mut _,
            &mut aux_data as *mut _,
            desired_access,
            core::ptr::addr_of_mut!((*object_type).TypeInfo.GenericMapping),
        );
        if status < 0 {
            ObpDereferenceNameInfo(object_name_info);
            ObDereferenceObject(object);
            return status;
        }
    }

    (*access_state).SecurityDescriptor = (*object_create_info).SecurityDescriptor;

    status = ObpValidateAccessMask(access_state);
    if status < 0 {
        ObpDereferenceNameInfo(object_name_info);
        ObDereferenceObject(object);
        return status;
    }

    ObpInitializeLookupContext(&mut context as *mut _);
    insert_object = object;
    open_reason = ObCreateHandle;

    if !object_name.is_null() {
        status = ObpLookupObjectName(
            (*object_create_info).RootDirectory,
            object_name,
            (*object_create_info).Attributes,
            object_type,
            if (*object_header).Flags & OB_FLAG_KERNEL_MODE != 0 {
                0i8
            } else {
                1i8
            },
            (*object_create_info).ParseContext,
            (*object_create_info).SecurityQos,
            object,
            access_state,
            &mut context as *mut _,
            &mut insert_object as *mut _,
        );

        if status >= 0 && !insert_object.is_null() && object != insert_object {
            open_reason = ObOpenHandle;

            if (*object_create_info).Attributes & OBJ_OPENIF != 0 {
                if object_type != (*OBJECT_TO_OBJECT_HEADER(insert_object)).Type {
                    status = STATUS_OBJECT_TYPE_MISMATCH;
                } else {
                    status = STATUS_OBJECT_NAME_EXISTS;
                }
            } else {
                if (*OBJECT_TO_OBJECT_HEADER(insert_object)).Type
                    == ObpSymbolicLinkObjectType
                {
                    ObDereferenceObject(insert_object);
                }

                status = STATUS_OBJECT_NAME_COLLISION;
            }
        }

        if status < 0 {
            ObpReleaseLookupContext(&mut context as *mut _);
            ObpDereferenceNameInfo(object_name_info);
            ObDereferenceObject(object);
            if access_state == &mut local_access_state as *mut _ {
                SeDeleteAccessState(access_state);
            }

            return status;
        } else {
            if object_type == ObpSymbolicLinkObjectType {
                ObpCreateSymbolicLinkName(object as *mut _OBJECT_SYMBOLIC_LINK);
            }
        }
    }

    if insert_object == object {
        if !object_name.is_null() || (*object_type).TypeInfo.SecurityRequired != 0 {
            if !object_name_info.is_null() && !(*object_name_info).Directory.is_null() {
                ObGetObjectSecurity(
                    (*object_name_info).Directory as *mut core::ffi::c_void,
                    &mut parent_descriptor as *mut _,
                    &mut sd_allocated as *mut _,
                );
            }

            status = ObAssignSecurity(
                access_state,
                parent_descriptor,
                object,
                object_type,
            );

            if !parent_descriptor.is_null() {
                ObReleaseObjectSecurity(parent_descriptor, sd_allocated);
            } else if status >= 0 {
                SeReleaseSecurityDescriptor(
                    (*object_create_info).SecurityDescriptor,
                    (*object_create_info).ProbeMode,
                    1u8,
                );

                (*access_state).SecurityDescriptor = core::ptr::null_mut();
                (*object_create_info).SecurityDescriptor = core::ptr::null_mut();
            }
        }

        if status < 0 {
            if context.DirectoryLocked != 0 {
            }

            ObpReleaseLookupContext(&mut context as *mut _);
            ObpDereferenceNameInfo(object_name_info);
            ObDereferenceObject(object);
            if access_state == &mut local_access_state as *mut _ {
                SeDeleteAccessState(access_state);
            }

            return status;
        }
    }

    real_status = status;

    (*object_header).__bindgen_anon_2.ObjectCreateInfo = core::ptr::null_mut();
    if !handle.is_null() {
        status = ObpCreateHandle(
            open_reason,
            insert_object,
            core::ptr::null_mut(),
            access_state,
            object_pointer_bias + 1,
            (*object_create_info).Attributes,
            &mut context as *mut _,
            previous_mode,
            new_object,
            handle,
        );
        if status < 0 {
            if !object_name.is_null() {
                ObpDeleteNameCheck(object);
            }

            *handle = core::ptr::null_mut();
            real_status = status;
        }

        ObpDereferenceNameInfo(object_name_info);
        ObDereferenceObject(object);
    } else {
        ObpAcquireObjectLock(object_header);

        real_status = ObpChargeQuotaForObject(
            object_header,
            object_type,
            &mut is_new_object as *mut _,
        );

        ObpReleaseObjectLock(object_header);

        if real_status < 0 {
            ObDereferenceObject(object);
        }
    }

    ObpFreeObjectCreateInformation(object_create_info);

    if access_state == &mut local_access_state as *mut _ {
        SeDeleteAccessState(access_state);
    }

    real_status
}

#[no_mangle]
pub unsafe extern "system" fn ObSetHandleAttributes(
    mut handle: HANDLE,
    handle_flags: POBJECT_HANDLE_ATTRIBUTE_INFORMATION,
    previous_mode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let mut handle = opaque_ptr(handle);
    let mut set_handle_attributes_context: _OBP_SET_HANDLE_ATTRIBUTES_CONTEXT = core::mem::zeroed();
    let mut result: BOOLEAN;
    let mut attached_to_system_process = 0u8;
    let handle_table: PHANDLE_TABLE;
    let mut apc_state: _KAPC_STATE = core::mem::zeroed();

    if ObpIsKernelHandle(handle, previous_mode) != 0 {
        handle_table = ObpKernelHandleTable;
        handle = ObKernelHandleToHandle(handle);

        if PsGetCurrentProcess() as PVOID != PsInitialSystemProcess {
            KeStackAttachProcess(
                core::ptr::addr_of_mut!((*(PsInitialSystemProcess as *mut _EPROCESS)).Pcb),
                &mut apc_state as *mut _,
            );
            attached_to_system_process = 1u8;
        }
    } else {
        handle_table = (*(PsGetCurrentProcess() as *mut _EPROCESS)).ObjectTable;
    }

    set_handle_attributes_context.PreviousMode = previous_mode;
    set_handle_attributes_context.Information = *handle_flags;

    result = ExChangeHandle(
        handle_table,
        handle,
        Some(ObpSetHandleAttributes),
        &mut set_handle_attributes_context as *mut _ as ULONG_PTR as _,
    );

    if attached_to_system_process != 0 {
        KeUnstackDetachProcess(&mut apc_state as *mut _);
    }

    if result != 0 {
        STATUS_SUCCESS
    } else {
        STATUS_ACCESS_DENIED
    }
}

#[no_mangle]
pub unsafe extern "system" fn ObCloseHandle(
    handle: HANDLE,
    access_mode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let mut handle = opaque_ptr(handle);
    ObpCloseHandle(handle, access_mode)
}

#[no_mangle]
pub unsafe extern "system" fn NtClose(handle: HANDLE) -> NTSTATUS {
    let mut handle = opaque_ptr(handle);
    ObpCloseHandle(handle, ExGetPreviousMode())
}

#[no_mangle]
pub unsafe extern "system" fn NtDuplicateObject(
    source_process_handle: HANDLE,
    source_handle: HANDLE,
    target_process_handle: HANDLE,
    target_handle: *mut HANDLE,
    desired_access: ACCESS_MASK,
    handle_attributes: ULONG,
    options: ULONG,
) -> NTSTATUS {
    let source_process_handle = opaque_ptr(source_process_handle);
    let source_handle = opaque_ptr(source_handle);
    let target_process_handle = opaque_ptr(target_process_handle);
    let target_handle = opaque_ptr(target_handle);
    let mut source_process: PEPROCESS = core::ptr::null_mut();
    let mut target_process: PEPROCESS = core::ptr::null_mut();
    let mut target: PEPROCESS;
    let mut h_target: HANDLE = core::ptr::null_mut();
    let previous_mode = ExGetPreviousMode();
    let mut status: NTSTATUS;

    if !target_handle.is_null() && previous_mode != 0i8 {
        if let Err(s) = probe_write_handle(target_handle as _, previous_mode) {
            return s;
        }
        *target_handle = core::ptr::null_mut();
    }

    status = ObReferenceObjectByHandle(
        source_process_handle,
        PROCESS_DUP_HANDLE,
        PsProcessType,
        previous_mode,
        &mut source_process as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    if !target_process_handle.is_null() {
        status = ObReferenceObjectByHandle(
            target_process_handle,
            PROCESS_DUP_HANDLE,
            PsProcessType,
            previous_mode,
            &mut target_process as *mut _ as *mut PVOID,
            core::ptr::null_mut(),
        );
        if status >= 0 {
            target = target_process;
        } else {
            target = core::ptr::null_mut();
        }
    } else {
        status = STATUS_SUCCESS;
        target = core::ptr::null_mut();
    }

    status = ObDuplicateObject(
        source_process,
        source_handle,
        target,
        &mut h_target as *mut _,
        desired_access,
        handle_attributes,
        options,
        previous_mode,
    );

    if !target_handle.is_null() {
        *target_handle = h_target;
    }

    if !target.is_null() {
        ObDereferenceObject(target as PVOID);
    }
    ObDereferenceObject(source_process as PVOID);
    status
}

#[no_mangle]
pub unsafe extern "system" fn ObIsKernelHandle(handle: HANDLE) -> BOOLEAN {
    let mut handle = opaque_ptr(handle);
    (ObpIsKernelHandle(handle, 0i8) != 0) as u8
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ob/obhandle.c
//  * PURPOSE:         Manages all functions related to the Object Manager handle
//  *                  implementation, including creating and destroying handles
//  *                  and/or handle tables, duplicating objects, and setting the
//  *                  permanent or temporary flags.
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Eric Kohl
//  *                  Thomas Weidenmueller (w3seek@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// PHANDLE_TABLE ObpKernelHandleTable = NULL;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// PHANDLE_TABLE
// NTAPI
// ObReferenceProcessHandleTable(IN PEPROCESS Process)
// {
//     PHANDLE_TABLE HandleTable = NULL;
//
//     /* Lock the process */
//     if (ExAcquireRundownProtection(&Process->RundownProtect))
//     {
//         /* Get the handle table */
//         HandleTable = Process->ObjectTable;
//         if (!HandleTable)
//         {
//             /* No table, release the lock */
//             ExReleaseRundownProtection(&Process->RundownProtect);
//         }
//     }
//
//     /* Return the handle table */
//     return HandleTable;
// }
//
// VOID
// NTAPI
// ObDereferenceProcessHandleTable(IN PEPROCESS Process)
// {
//     /* Release the process lock */
//     ExReleaseRundownProtection(&Process->RundownProtect);
// }
//
// ULONG
// NTAPI
// ObGetProcessHandleCount(IN PEPROCESS Process)
// {
//     ULONG HandleCount;
//     PHANDLE_TABLE HandleTable;
//
//     ASSERT(Process);
//
//     /* Ensure the handle table doesn't go away while we use it */
//     HandleTable = ObReferenceProcessHandleTable(Process);
//
//     if (HandleTable != NULL)
//     {
//         /* Count the number of handles the process has */
//         HandleCount = HandleTable->HandleCount;
//
//         /* Let the handle table go */
//         ObDereferenceProcessHandleTable(Process);
//     }
//     else
//     {
//         /* No handle table, no handles */
//         HandleCount = 0;
//     }
//
//     return HandleCount;
// }
//
// NTSTATUS
// NTAPI
// ObpReferenceProcessObjectByHandle(IN HANDLE Handle,
//                                   IN PEPROCESS Process,
//                                   IN PHANDLE_TABLE HandleTable,
//                                   IN KPROCESSOR_MODE AccessMode,
//                                   OUT PVOID *Object,
//                                   OUT POBJECT_HANDLE_INFORMATION HandleInformation,
//                                   OUT PACCESS_MASK AuditMask)
// {
//     PHANDLE_TABLE_ENTRY HandleEntry;
//     POBJECT_HEADER ObjectHeader;
//     ACCESS_MASK GrantedAccess;
//     ULONG Attributes;
//     PETHREAD Thread = PsGetCurrentThread();
//     NTSTATUS Status;
//
//     /* Assume failure */
//     *Object = NULL;
//
//     /* Check if this is a special handle */
//     if (HandleToLong(Handle) < 0)
//     {
//         /* Check if the caller wants the current process */
//         if (Handle == NtCurrentProcess())
//         {
//             /* Return handle info */
//             HandleInformation->HandleAttributes = 0;
//             HandleInformation->GrantedAccess = Process->GrantedAccess;
//
//             /* No audit mask */
//             *AuditMask = 0;
//
//             /* Reference ourselves */
//             ObjectHeader = OBJECT_TO_OBJECT_HEADER(Process);
//             InterlockedIncrementSizeT(&ObjectHeader->PointerCount);
//
//             /* Return the pointer */
//             *Object = Process;
//             ASSERT(*Object != NULL);
//             return STATUS_SUCCESS;
//         }
//
//         /* Check if the caller wants the current thread */
//         if (Handle == NtCurrentThread())
//         {
//             /* Return handle information */
//             HandleInformation->HandleAttributes = 0;
//             HandleInformation->GrantedAccess = Thread->GrantedAccess;
//
//             /* Reference ourselves */
//             ObjectHeader = OBJECT_TO_OBJECT_HEADER(Thread);
//             InterlockedIncrementSizeT(&ObjectHeader->PointerCount);
//
//             /* No audit mask */
//             *AuditMask = 0;
//
//             /* Return the pointer */
//             *Object = Thread;
//             ASSERT(*Object != NULL);
//             return STATUS_SUCCESS;
//         }
//
//         /* This is a kernel handle... do we have access? */
//         if (AccessMode == KernelMode)
//         {
//             /* Use the kernel handle table and get the actual handle value */
//             Handle = ObKernelHandleToHandle(Handle);
//             HandleTable = ObpKernelHandleTable;
//         }
//         else
//         {
//             /* This is an illegal attempt to access a kernel handle */
//             return STATUS_INVALID_HANDLE;
//         }
//     }
//
//     /* Enter a critical region while we touch the handle table */
//     ASSERT(HandleTable != NULL);
//     KeEnterCriticalRegion();
//
//     /* Get the handle entry */
//     HandleEntry = ExMapHandleToPointer(HandleTable, Handle);
//     if (HandleEntry)
//     {
//         /* Get the object header and validate the type*/
//         ObjectHeader = ObpGetHandleObject(HandleEntry);
//
//         /* Get the granted access and validate it */
//         GrantedAccess = HandleEntry->GrantedAccess;
//
//         /* Mask out the internal attributes */
//         Attributes = HandleEntry->ObAttributes & OBJ_HANDLE_ATTRIBUTES;
//
//         /* Fill out the information */
//         HandleInformation->HandleAttributes = Attributes;
//         HandleInformation->GrantedAccess = GrantedAccess;
//
//         /* No audit mask (FIXME!) */
//         *AuditMask = 0;
//
//         /* Return the pointer */
//         *Object = &ObjectHeader->Body;
//
//         /* Add a reference */
//         InterlockedIncrementSizeT(&ObjectHeader->PointerCount);
//
//         /* Unlock the handle */
//         ExUnlockHandleTableEntry(HandleTable, HandleEntry);
//         KeLeaveCriticalRegion();
//
//         /* Return success */
//         ASSERT(*Object != NULL);
//         return STATUS_SUCCESS;
//     }
//     else
//     {
//         /* Invalid handle */
//         Status = STATUS_INVALID_HANDLE;
//     }
//
//     /* Return failure status */
//     KeLeaveCriticalRegion();
//     return Status;
// }
//
// BOOLEAN
// NTAPI
// ObpEnumFindHandleProcedure(IN PHANDLE_TABLE_ENTRY HandleEntry,
//                            IN HANDLE Handle,
//                            IN PVOID Context)
// {
//     POBJECT_HEADER ObjectHeader;
//     ACCESS_MASK GrantedAccess;
//     ULONG HandleAttributes;
//     POBP_FIND_HANDLE_DATA FindData = Context;
//
//     /* Get the object header */
//     ObjectHeader = ObpGetHandleObject(HandleEntry);
//
//     /* Make sure it's valid and matching */
//     if ((FindData->ObjectHeader) && (FindData->ObjectHeader != ObjectHeader))
//     {
//         /* No match, fail */
//         return FALSE;
//     }
//
//     /* Now attempt to match the object type */
//     if ((FindData->ObjectType) && (FindData->ObjectType != ObjectHeader->Type))
//     {
//         /* No match, fail */
//         return FALSE;
//     }
//
//     /* Check if we have extra information */
//     if (FindData->HandleInformation)
//     {
//         /* Get the granted access and attributes */
//         GrantedAccess = HandleEntry->GrantedAccess;
//         HandleAttributes = HandleEntry->ObAttributes & OBJ_HANDLE_ATTRIBUTES;
//
//         /* Attempt to match them */
//         if ((FindData->HandleInformation->HandleAttributes != HandleAttributes) ||
//             (FindData->HandleInformation->GrantedAccess != GrantedAccess))
//         {
//             /* No match, fail */
//             return FALSE;
//         }
//     }
//
//     /* We have a match */
//     return TRUE;
// }
//
// POBJECT_HANDLE_COUNT_ENTRY
// NTAPI
// ObpInsertHandleCount(IN POBJECT_HEADER ObjectHeader)
// {
//     POBJECT_HEADER_HANDLE_INFO HandleInfo;
//     POBJECT_HANDLE_COUNT_ENTRY FreeEntry;
//     POBJECT_HANDLE_COUNT_DATABASE HandleDatabase, OldHandleDatabase;
//     ULONG i;
//     ULONG Size, OldSize;
//     OBJECT_HANDLE_COUNT_DATABASE SingleDatabase;
//     PAGED_CODE();
//
//     /* Get the handle info */
//     HandleInfo = OBJECT_HEADER_TO_HANDLE_INFO(ObjectHeader);
//     if (!HandleInfo) return NULL;
//
//     /* Check if we only have one entry */
//     if (ObjectHeader->Flags & OB_FLAG_SINGLE_PROCESS)
//     {
//         /* Fill out the single entry */
//         SingleDatabase.CountEntries = 1;
//         SingleDatabase.HandleCountEntries[0] = HandleInfo->SingleEntry;
//
//         /* Use this as the old size */
//         OldHandleDatabase = &SingleDatabase;
//         OldSize = sizeof(SingleDatabase);
//
//         /* Now we'll have two entries, and an entire DB */
//         i = 2;
//         Size = sizeof(OBJECT_HANDLE_COUNT_DATABASE) +
//                ((i - 1) * sizeof(OBJECT_HANDLE_COUNT_ENTRY));
//     }
//     else
//     {
//         /* We already have a DB, get the information from it */
//         OldHandleDatabase = HandleInfo->HandleCountDatabase;
//         i = OldHandleDatabase->CountEntries;
//         OldSize = sizeof(OBJECT_HANDLE_COUNT_DATABASE) +
//                   ((i - 1) * sizeof(OBJECT_HANDLE_COUNT_ENTRY));
//
//         /* Add 4 more entries */
//         i += 4;
//         Size = OldSize + (4 * sizeof(OBJECT_HANDLE_COUNT_ENTRY));
//     }
//
//     /* Allocate the DB */
//     HandleDatabase = ExAllocatePoolWithTag(PagedPool, Size, TAG_OB_HANDLE);
//     if (!HandleDatabase) return NULL;
//
//     /* Copy the old database */
//     RtlCopyMemory(HandleDatabase, OldHandleDatabase, OldSize);
//
//     /* Check if we he had a single entry before */
//     if (ObjectHeader->Flags & OB_FLAG_SINGLE_PROCESS)
//     {
//         /* Now we have more */
//         ObjectHeader->Flags &= ~OB_FLAG_SINGLE_PROCESS;
//     }
//     else
//     {
//         /* Otherwise we had a DB, free it */
//         ExFreePoolWithTag(OldHandleDatabase, TAG_OB_HANDLE);
//     }
//
//     /* Find the end of the copy and zero out the new data */
//     FreeEntry = (PVOID)((ULONG_PTR)HandleDatabase + OldSize);
//     RtlZeroMemory(FreeEntry, Size - OldSize);
//
//     /* Set the new information and return the free entry */
//     HandleDatabase->CountEntries = i;
//     HandleInfo->HandleCountDatabase = HandleDatabase;
//     return FreeEntry;
// }
//
// NTSTATUS
// NTAPI
// ObpIncrementHandleDataBase(IN POBJECT_HEADER ObjectHeader,
//                            IN PEPROCESS Process,
//                            IN OUT PULONG NewProcessHandleCount)
// {
//     POBJECT_HEADER_HANDLE_INFO HandleInfo;
//     POBJECT_HANDLE_COUNT_ENTRY HandleEntry, FreeEntry = NULL;
//     POBJECT_HANDLE_COUNT_DATABASE HandleDatabase;
//     ULONG i;
//     PAGED_CODE();
//
//     /* Get the handle info and check if we only have one entry */
//     HandleInfo = OBJECT_HEADER_TO_HANDLE_INFO(ObjectHeader);
//     if (ObjectHeader->Flags & OB_FLAG_SINGLE_PROCESS)
//     {
//         /* Check if the entry is free */
//         if (!HandleInfo->SingleEntry.HandleCount)
//         {
//             /* Add ours */
//             HandleInfo->SingleEntry.HandleCount = 1;
//             HandleInfo->SingleEntry.Process = Process;
//
//             /* Return success and 1 handle */
//             *NewProcessHandleCount = 1;
//             return STATUS_SUCCESS;
//         }
//         else if (HandleInfo->SingleEntry.Process == Process)
//         {
//             /* Busy entry, but same process */
//             *NewProcessHandleCount = ++HandleInfo->SingleEntry.HandleCount;
//             return STATUS_SUCCESS;
//         }
//         else
//         {
//             /* Insert a new entry */
//             FreeEntry = ObpInsertHandleCount(ObjectHeader);
//             if (!FreeEntry) return STATUS_INSUFFICIENT_RESOURCES;
//             ASSERT(!FreeEntry->Process);
//             ASSERT(!FreeEntry->HandleCount);
//
//             /* Fill it out */
//             FreeEntry->Process = Process;
//             FreeEntry->HandleCount = 1;
//
//             /* Return success and 1 handle */
//             *NewProcessHandleCount = 1;
//             return STATUS_SUCCESS;
//         }
//     }
//
//     /* We have a database instead */
//     HandleDatabase = HandleInfo->HandleCountDatabase;
//     if (HandleDatabase)
//     {
//         /* Get the entries and loop them */
//         i = HandleDatabase->CountEntries;
//         HandleEntry = &HandleDatabase->HandleCountEntries[0];
//         while (i)
//         {
//             /* Check if this is a match */
//             if (HandleEntry->Process == Process)
//             {
//                 /* Found it, get the process handle count */
//                 *NewProcessHandleCount = ++HandleEntry->HandleCount;
//                 return STATUS_SUCCESS;
//             }
//             else if (!HandleEntry->HandleCount)
//             {
//                 /* Found a free entry */
//                 FreeEntry = HandleEntry;
//             }
//
//             /* Keep looping */
//             HandleEntry++;
//             i--;
//         }
//
//         /* Check if we couldn't find a free entry */
//         if (!FreeEntry)
//         {
//             /* Allocate one */
//             FreeEntry = ObpInsertHandleCount(ObjectHeader);
//             if (!FreeEntry) return STATUS_INSUFFICIENT_RESOURCES;
//             ASSERT(!FreeEntry->Process);
//             ASSERT(!FreeEntry->HandleCount);
//         }
//
//         /* Fill out the entry */
//         FreeEntry->Process = Process;
//         FreeEntry->HandleCount = 1;
//         *NewProcessHandleCount = 1;
//     }
//
//     /* Return success if we got here */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// ObpChargeQuotaForObject(IN POBJECT_HEADER ObjectHeader,
//                         IN POBJECT_TYPE ObjectType,
//                         OUT PBOOLEAN NewObject)
// {
//     POBJECT_HEADER_QUOTA_INFO ObjectQuota;
//     ULONG PagedPoolCharge, NonPagedPoolCharge;
//
//     /* Get quota information */
//     ObjectQuota = OBJECT_HEADER_TO_QUOTA_INFO(ObjectHeader);
//     *NewObject = FALSE;
//
//     /* Check if this is a new object */
//     if (ObjectHeader->Flags & OB_FLAG_CREATE_INFO)
//     {
//         /* Remove the flag */
//         ObjectHeader->Flags &= ~ OB_FLAG_CREATE_INFO;
//         if (ObjectQuota)
//         {
//             /* We have a quota, get the charges */
//             PagedPoolCharge = ObjectQuota->PagedPoolCharge +
//                               ObjectQuota->SecurityDescriptorCharge;
//             NonPagedPoolCharge = ObjectQuota->NonPagedPoolCharge;
//         }
//         else
//         {
//             /* Get it from the object type */
//             PagedPoolCharge = ObjectType->TypeInfo.DefaultPagedPoolCharge;
//             NonPagedPoolCharge = ObjectType->TypeInfo.DefaultNonPagedPoolCharge;
//         }
//
//         /* Is this the system process? */
//         if (PsGetCurrentProcess() == PsInitialSystemProcess)
//         {
//             /* It is, don't do anything */
//             ObjectHeader->QuotaBlockCharged = OBP_SYSTEM_PROCESS_QUOTA;
//         }
//         else
//         {
//             /* Charge the quota */
//             ObjectHeader->QuotaBlockCharged = PsChargeSharedPoolQuota(PsGetCurrentProcess(),
//                                                                       PagedPoolCharge,
//                                                                       NonPagedPoolCharge);
//         }
//
//         /* Check if we don't have a quota block */
//         if (!ObjectHeader->QuotaBlockCharged) return STATUS_QUOTA_EXCEEDED;
//
//         /* Now set the flag */
//         *NewObject = TRUE;
//     }
//
//     /* Return success */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// ObpValidateAccessMask(IN PACCESS_STATE AccessState)
// {
//     PISECURITY_DESCRIPTOR SecurityDescriptor;
//
//     /* We're only interested if the object for this access state has an SD */
//     SecurityDescriptor = AccessState->SecurityDescriptor;
//     if (SecurityDescriptor)
//     {
//         /* Check if the SD has a system ACL but hasn't been granted access to get/set it */
//         if ((SecurityDescriptor->Control & SE_SACL_PRESENT) &&
//             !(AccessState->PreviouslyGrantedAccess & ACCESS_SYSTEM_SECURITY))
//         {
//             /* We're gonna need access */
//             AccessState->RemainingDesiredAccess |= ACCESS_SYSTEM_SECURITY;
//         }
//     }
//
//     /* This can't fail */
//     return STATUS_SUCCESS;
// }
//
// /*++
// * @name ObpDecrementHandleCount
// *
// *     The ObpDecrementHandleCount routine <FILLMEIN>
// *
// * @param ObjectBody
// *        <FILLMEIN>.
// *
// * @param Process
// *        <FILLMEIN>.
// *
// * @param GrantedAccess
// *        <FILLMEIN>.
// *
// * @return None.
// *
// * @remarks None.
// *
// *--*/
// VOID
// NTAPI
// ObpDecrementHandleCount(IN PVOID ObjectBody,
//                         IN PEPROCESS Process,
//                         IN ACCESS_MASK GrantedAccess,
//                         IN POBJECT_TYPE ObjectType)
// {
//     POBJECT_HEADER ObjectHeader;
//     LONG SystemHandleCount, ProcessHandleCount;
//     LONG NewCount;
//     KIRQL CalloutIrql;
//     POBJECT_HEADER_HANDLE_INFO HandleInfo;
//     POBJECT_HANDLE_COUNT_ENTRY HandleEntry;
//     POBJECT_HANDLE_COUNT_DATABASE HandleDatabase;
//     ULONG i;
//     PAGED_CODE();
//
//     /* Get the object type and header */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(ObjectBody);
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - Decrementing count for: %p. HC PC %lx %lx\n",
//             __FUNCTION__,
//             ObjectBody,
//             ObjectHeader->HandleCount,
//             ObjectHeader->PointerCount);
//
//     /* Lock the object */
//     ObpAcquireObjectLock(ObjectHeader);
//
//     /* Set default counts */
//     SystemHandleCount = ObjectHeader->HandleCount;
//     ProcessHandleCount = 0;
//
//     /* Decrement the handle count */
//     NewCount = InterlockedDecrementSizeT(&ObjectHeader->HandleCount);
//
//     /* Check if we're out of handles and this was an exclusive object */
//     if (!(NewCount) && (ObjectHeader->Flags & OB_FLAG_EXCLUSIVE))
//     {
//         /* Clear the exclusive flag */
//         OBJECT_HEADER_TO_QUOTA_INFO(ObjectHeader)->ExclusiveProcess = NULL;
//     }
//
//     /* Is the object type keeping track of handles? */
//     if (ObjectType->TypeInfo.MaintainHandleCount)
//     {
//         /* Get handle information */
//         HandleInfo = OBJECT_HEADER_TO_HANDLE_INFO(ObjectHeader);
//
//         /* Check if there's only a single entry */
//         if (ObjectHeader->Flags & OB_FLAG_SINGLE_PROCESS)
//         {
//             /* It should be us */
//             ASSERT(HandleInfo->SingleEntry.Process == Process);
//             ASSERT(HandleInfo->SingleEntry.HandleCount > 0);
//
//             /* Get the handle counts */
//             ProcessHandleCount = HandleInfo->SingleEntry.HandleCount--;
//             HandleEntry = &HandleInfo->SingleEntry;
//         }
//         else
//         {
//             /* Otherwise, get the database */
//             HandleDatabase = HandleInfo->HandleCountDatabase;
//             if (HandleDatabase)
//             {
//                 /* Get the entries and loop them */
//                 i = HandleDatabase->CountEntries;
//                 HandleEntry = &HandleDatabase->HandleCountEntries[0];
//                 while (i)
//                 {
//                     /* Check if this is a match */
//                     if ((HandleEntry->HandleCount) &&
//                         (HandleEntry->Process == Process))
//                     {
//                         /* Found it, get the process handle count */
//                         ProcessHandleCount = HandleEntry->HandleCount--;
//                         break;
//                     }
//
//                     /* Keep looping */
//                     HandleEntry++;
//                     i--;
//                 }
//             }
//             else
//             {
//                 /* No database, so no entry */
//                 HandleEntry = NULL;
//             }
//         }
//
//         /* Check if this is the last handle */
//         if (ProcessHandleCount == 1)
//         {
//             /* Then clear the entry */
//             HandleEntry->Process = NULL;
//             HandleEntry->HandleCount = 0;
//         }
//     }
//
//     /* Release the lock */
//     ObpReleaseObjectLock(ObjectHeader);
//
//     /* Check if we have a close procedure */
//     if (ObjectType->TypeInfo.CloseProcedure)
//     {
//         /* Call it */
//         ObpCalloutStart(&CalloutIrql);
//         ObjectType->TypeInfo.CloseProcedure(Process,
//                                             ObjectBody,
//                                             GrantedAccess,
//                                             ProcessHandleCount,
//                                             SystemHandleCount);
//         ObpCalloutEnd(CalloutIrql, "Close", ObjectType, ObjectBody);
//     }
//
//     /* Check if we should delete the object */
//     ObpDeleteNameCheck(ObjectBody);
//
//     /* Decrease the total number of handles for this type */
//     InterlockedDecrement((PLONG)&ObjectType->TotalNumberOfHandles);
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - Decremented count for: %p. HC PC %lx %lx\n",
//             __FUNCTION__,
//             ObjectBody,
//             ObjectHeader->HandleCount,
//             ObjectHeader->PointerCount);
// }
//
// /*++
// * @name ObpCloseHandleTableEntry
// *
// *     The ObpCloseHandleTableEntry routine <FILLMEIN>
// *
// * @param HandleTable
// *        <FILLMEIN>.
// *
// * @param HandleEntry
// *        <FILLMEIN>.
// *
// * @param Handle
// *        <FILLMEIN>.
// *
// * @param AccessMode
// *        <FILLMEIN>.
// *
// * @param IgnoreHandleProtection
// *        <FILLMEIN>.
// *
// * @return <FILLMEIN>.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// ObpCloseHandleTableEntry(IN PHANDLE_TABLE HandleTable,
//                          IN PHANDLE_TABLE_ENTRY HandleEntry,
//                          IN HANDLE Handle,
//                          IN KPROCESSOR_MODE AccessMode,
//                          IN BOOLEAN IgnoreHandleProtection)
// {
//     PVOID Body;
//     POBJECT_TYPE ObjectType;
//     POBJECT_HEADER ObjectHeader;
//     ACCESS_MASK GrantedAccess;
//     KIRQL CalloutIrql;
//     PAGED_CODE();
//
//     /* Get the object data */
//     ObjectHeader = ObpGetHandleObject(HandleEntry);
//     ObjectType = ObjectHeader->Type;
//     Body = &ObjectHeader->Body;
//     GrantedAccess = HandleEntry->GrantedAccess;
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - Closing handle: %p for %p. HC PC %lx %lx\n",
//             __FUNCTION__,
//             Handle,
//             Body,
//             ObjectHeader->HandleCount,
//             ObjectHeader->PointerCount);
//
//     /* Check if the object has an Okay To Close procedure */
//     if (ObjectType->TypeInfo.OkayToCloseProcedure)
//     {
//         /* Call it and check if it's not letting us close it */
//         ObpCalloutStart(&CalloutIrql);
//         if (!ObjectType->TypeInfo.OkayToCloseProcedure(PsGetCurrentProcess(),
//                                                        Body,
//                                                        Handle,
//                                                        AccessMode))
//         {
//             /* Fail */
//             ObpCalloutEnd(CalloutIrql, "NtClose", ObjectType, Body);
//             ExUnlockHandleTableEntry(HandleTable, HandleEntry);
//             return STATUS_HANDLE_NOT_CLOSABLE;
//         }
//
//         /* Success, validate callout retrn */
//         ObpCalloutEnd(CalloutIrql, "NtClose", ObjectType, Body);
//     }
//
//     /* The callback allowed us to close it, but does the handle itself? */
//     if ((HandleEntry->GrantedAccess & ObpAccessProtectCloseBit) &&
//         !(IgnoreHandleProtection))
//     {
//         /* It doesn't, are we from user mode? */
//         if (AccessMode != KernelMode)
//         {
//             /* We are! Unlock the entry */
//             ExUnlockHandleTableEntry(HandleTable, HandleEntry);
//
//             /* Make sure we have a debug port */
//             if (PsGetCurrentProcess()->DebugPort)
//             {
//                 /* Raise an exception */
//                 return KeRaiseUserException(STATUS_HANDLE_NOT_CLOSABLE);
//             }
//             else
//             {
//                 /* Return the error instead */
//                 return STATUS_HANDLE_NOT_CLOSABLE;
//             }
//         }
//         else
//         {
//             /* Otherwise, bugcheck the OS */
//             KeBugCheckEx(INVALID_KERNEL_HANDLE, (ULONG_PTR)Handle, 0, 0, 0);
//         }
//     }
//
//     /* Destroy and unlock the handle entry */
//     ExDestroyHandle(HandleTable, Handle, HandleEntry);
//
//     /* Now decrement the handle count */
//     ObpDecrementHandleCount(Body,
//                             (PsGetCurrentProcess() as *mut _EPROCESS),
//                             GrantedAccess,
//                             ObjectType);
//
//     /* Dereference the object as well */
//     ObDereferenceObject(Body);
//
//     /* Return to caller */
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - Closed handle: %p for %p.\n",
//             __FUNCTION__,
//             Handle,
//             Body);
//     return STATUS_SUCCESS;
// }
//
// /*++
// * @name ObpIncrementHandleCount
// *
// *     The ObpIncrementHandleCount routine <FILLMEIN>
// *
// * @param Object
// *        <FILLMEIN>.
// *
// * @param AccessState
// *        <FILLMEIN>.
// *
// * @param AccessMode
// *        <FILLMEIN>.
// *
// * @param HandleAttributes
// *        <FILLMEIN>.
// *
// * @param Process
// *        <FILLMEIN>.
// *
// * @param OpenReason
// *        <FILLMEIN>.
// *
// * @return <FILLMEIN>.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// ObpIncrementHandleCount(IN PVOID Object,
//                         IN PACCESS_STATE AccessState OPTIONAL,
//                         IN KPROCESSOR_MODE AccessMode,
//                         IN ULONG HandleAttributes,
//                         IN PEPROCESS Process,
//                         IN OB_OPEN_REASON OpenReason)
// {
//     POBJECT_HEADER ObjectHeader;
//     POBJECT_TYPE ObjectType;
//     ULONG ProcessHandleCount;
//     NTSTATUS Status;
//     PEPROCESS ExclusiveProcess;
//     BOOLEAN Exclusive = FALSE, NewObject;
//     POBJECT_HEADER_CREATOR_INFO CreatorInfo;
//     KIRQL CalloutIrql;
//     KPROCESSOR_MODE ProbeMode;
//     ULONG Total;
//     POBJECT_HEADER_NAME_INFO NameInfo;
//     PAGED_CODE();
//
//     /* Get the object header and type */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
//     ObjectType = ObjectHeader->Type;
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - Incrementing count for: %p. Reason: %lx. HC PC %lx %lx\n",
//             __FUNCTION__,
//             Object,
//             OpenReason,
//             ObjectHeader->HandleCount,
//             ObjectHeader->PointerCount);
//
//     /* Check if caller is forcing user mode */
//     if (HandleAttributes & OBJ_FORCE_ACCESS_CHECK)
//     {
//         /* Force it */
//         ProbeMode = UserMode;
//     }
//     else
//     {
//         /* Keep original setting */
//         ProbeMode = AccessMode;
//     }
//
//     /* Lock the object */
//     ObpAcquireObjectLock(ObjectHeader);
//
//     /* Charge quota and remove the creator info flag */
//     Status = ObpChargeQuotaForObject(ObjectHeader, ObjectType, &NewObject);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Check if the open is exclusive */
//     if (HandleAttributes & OBJ_EXCLUSIVE)
//     {
//         /* Check if the object allows this, or if the inherit flag was given */
//         if ((HandleAttributes & OBJ_INHERIT) ||
//             !(ObjectHeader->Flags & OB_FLAG_EXCLUSIVE))
//         {
//             /* Incorrect attempt */
//             Status = STATUS_INVALID_PARAMETER;
//             goto Quickie;
//         }
//
//         /* Check if we have access to it */
//         ExclusiveProcess = OBJECT_HEADER_TO_EXCLUSIVE_PROCESS(ObjectHeader);
//         if ((!(ExclusiveProcess) && (ObjectHeader->HandleCount)) ||
//             ((ExclusiveProcess) && (ExclusiveProcess != (PsGetCurrentProcess() as *mut _EPROCESS))))
//         {
//             /* This isn't the right process */
//             Status = STATUS_ACCESS_DENIED;
//             goto Quickie;
//         }
//
//         /* Now you got exclusive access */
//         Exclusive = TRUE;
//     }
//     else if ((ObjectHeader->Flags & OB_FLAG_EXCLUSIVE) &&
//              (OBJECT_HEADER_TO_EXCLUSIVE_PROCESS(ObjectHeader)))
//     {
//         /* Caller didn't want exclusive access, but the object is exclusive */
//         Status = STATUS_ACCESS_DENIED;
//         goto Quickie;
//     }
//
//     /* Check for exclusive kernel object */
//     NameInfo = OBJECT_HEADER_TO_NAME_INFO(ObjectHeader);
//     if ((NameInfo) && (NameInfo->QueryReferences & OB_FLAG_KERNEL_EXCLUSIVE) &&
//         (ProbeMode != KernelMode))
//     {
//         /* Caller is not kernel, but the object is kernel exclusive */
//         Status = STATUS_ACCESS_DENIED;
//         goto Quickie;
//     }
//
//     /*
//      * Check if this is an object that went from 0 handles back to existence,
//      * but doesn't have an open procedure, only a close procedure. This means
//      * that it will never realize that the object is back alive, so we must
//      * fail the request.
//      */
//     if (!(ObjectHeader->HandleCount) &&
//         !(NewObject) &&
//         (ObjectType->TypeInfo.MaintainHandleCount) &&
//         !(ObjectType->TypeInfo.OpenProcedure) &&
//         (ObjectType->TypeInfo.CloseProcedure))
//     {
//         /* Fail */
//         Status = STATUS_UNSUCCESSFUL;
//         goto Quickie;
//     }
//
//     /* Check if we're opening an existing handle */
//     if ((OpenReason == ObOpenHandle) ||
//         ((OpenReason == ObDuplicateHandle) && (AccessState)))
//     {
//         /* Validate the caller's access to this object */
//         if (!ObCheckObjectAccess(Object,
//                                  AccessState,
//                                  TRUE,
//                                  ProbeMode,
//                                  &Status))
//         {
//             /* Access was denied, so fail */
//             goto Quickie;
//         }
//     }
//     else if (OpenReason == ObCreateHandle)
//     {
//         /* Convert MAXIMUM_ALLOWED to GENERIC_ALL */
//         if (AccessState->RemainingDesiredAccess & MAXIMUM_ALLOWED)
//         {
//             /* Mask out MAXIMUM_ALLOWED and stick GENERIC_ALL instead */
//             AccessState->RemainingDesiredAccess &= ~MAXIMUM_ALLOWED;
//             AccessState->RemainingDesiredAccess |= GENERIC_ALL;
//         }
//
//         /* Check if we have to map the GENERIC mask */
//         if (AccessState->RemainingDesiredAccess & GENERIC_ACCESS)
//         {
//             /* Map it to the correct access masks */
//             RtlMapGenericMask(&AccessState->RemainingDesiredAccess,
//                               &ObjectType->TypeInfo.GenericMapping);
//         }
//
//         /* Check if the caller is trying to access system security */
//         if (AccessState->RemainingDesiredAccess & ACCESS_SYSTEM_SECURITY)
//         {
//             /* FIXME: TODO */
//             DPRINT1("ACCESS_SYSTEM_SECURITY not validated!\n");
//         }
//     }
//
//     /* Check if this is an exclusive handle */
//     if (Exclusive)
//     {
//         /* Save the owner process */
//         OBJECT_HEADER_TO_QUOTA_INFO(ObjectHeader)->ExclusiveProcess = Process;
//     }
//
//     /* Increase the handle count */
//     InterlockedIncrementSizeT(&ObjectHeader->HandleCount);
//     ProcessHandleCount = 0;
//
//     /* Check if we have a handle database */
//     if (ObjectType->TypeInfo.MaintainHandleCount)
//     {
//         /* Increment the handle database */
//         Status = ObpIncrementHandleDataBase(ObjectHeader,
//                                             Process,
//                                             &ProcessHandleCount);
//         if (!NT_SUCCESS(Status))
//         {
//             /* FIXME: This should never happen for now */
//             DPRINT1("Unhandled case\n");
//             ASSERT(FALSE);
//             goto Quickie;
//         }
//     }
//
//     /* Release the lock */
//     ObpReleaseObjectLock(ObjectHeader);
//
//     /* Check if we have an open procedure */
//     Status = STATUS_SUCCESS;
//     if (ObjectType->TypeInfo.OpenProcedure)
//     {
//         /* Call it */
//         ObpCalloutStart(&CalloutIrql);
//         Status = ObjectType->TypeInfo.OpenProcedure(OpenReason,
//                                                     Process,
//                                                     Object,
//                                                     AccessState ?
//                                                     AccessState->
//                                                     PreviouslyGrantedAccess :
//                                                     0,
//                                                     ProcessHandleCount);
//         ObpCalloutEnd(CalloutIrql, "Open", ObjectType, Object);
//
//         /* Check if the open procedure failed */
//         if (!NT_SUCCESS(Status))
//         {
//             /* FIXME: This should never happen for now */
//             DPRINT1("Unhandled case\n");
//             ASSERT(FALSE);
//             return Status;
//         }
//     }
//
//     /* Check if this is a create operation */
//     if (OpenReason == ObCreateHandle)
//     {
//         /* Check if we have creator info */
//         CreatorInfo = OBJECT_HEADER_TO_CREATOR_INFO(ObjectHeader);
//         if (CreatorInfo)
//         {
//             /* We do, acquire the lock */
//             ObpEnterObjectTypeMutex(ObjectType);
//
//             /* Insert us on the list */
//             InsertTailList(&ObjectType->TypeList, &CreatorInfo->TypeList);
//
//             /* Release the lock */
//             ObpLeaveObjectTypeMutex(ObjectType);
//         }
//     }
//
//     /* Increase total number of handles */
//     Total = InterlockedIncrement((PLONG)&ObjectType->TotalNumberOfHandles);
//     if (Total > ObjectType->HighWaterNumberOfHandles)
//     {
//         /* Fixup count */
//         ObjectType->HighWaterNumberOfHandles = Total;
//     }
//
//     /* Trace call and return */
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - Incremented count for: %p. Reason: %lx HC PC %lx %lx\n",
//             __FUNCTION__,
//             Object,
//             OpenReason,
//             ObjectHeader->HandleCount,
//             ObjectHeader->PointerCount);
//     return Status;
//
// Quickie:
//     /* Release lock and return */
//     ObpReleaseObjectLock(ObjectHeader);
//     return Status;
// }
//
// /*++
// * @name ObpIncrementUnnamedHandleCount
// *
// *     The ObpIncrementUnnamedHandleCount routine <FILLMEIN>
// *
// * @param Object
// *        <FILLMEIN>.
// *
// * @param AccessState
// *        <FILLMEIN>.
// *
// * @param AccessMode
// *        <FILLMEIN>.
// *
// * @param HandleAttributes
// *        <FILLMEIN>.
// *
// * @param Process
// *        <FILLMEIN>.
// *
// * @param OpenReason
// *        <FILLMEIN>.
// *
// * @return <FILLMEIN>.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// ObpIncrementUnnamedHandleCount(IN PVOID Object,
//                                IN PACCESS_MASK DesiredAccess,
//                                IN KPROCESSOR_MODE AccessMode,
//                                IN ULONG HandleAttributes,
//                                IN PEPROCESS Process)
// {
//     POBJECT_HEADER ObjectHeader;
//     POBJECT_TYPE ObjectType;
//     ULONG ProcessHandleCount;
//     NTSTATUS Status;
//     PEPROCESS ExclusiveProcess;
//     BOOLEAN Exclusive = FALSE, NewObject;
//     POBJECT_HEADER_CREATOR_INFO CreatorInfo;
//     KIRQL CalloutIrql;
//     ULONG Total;
//
//     /* Get the object header and type */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
//     ObjectType = ObjectHeader->Type;
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - Incrementing count for: %p. UNNAMED. HC PC %lx %lx\n",
//             __FUNCTION__,
//             Object,
//             ObjectHeader->HandleCount,
//             ObjectHeader->PointerCount);
//
//     /* Lock the object */
//     ObpAcquireObjectLock(ObjectHeader);
//
//     /* Charge quota and remove the creator info flag */
//     Status = ObpChargeQuotaForObject(ObjectHeader, ObjectType, &NewObject);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Check if the open is exclusive */
//     if (HandleAttributes & OBJ_EXCLUSIVE)
//     {
//         /* Check if the object allows this, or if the inherit flag was given */
//         if ((HandleAttributes & OBJ_INHERIT) ||
//             !(ObjectHeader->Flags & OB_FLAG_EXCLUSIVE))
//         {
//             /* Incorrect attempt */
//             Status = STATUS_INVALID_PARAMETER;
//             goto Quickie;
//         }
//
//         /* Check if we have access to it */
//         ExclusiveProcess = OBJECT_HEADER_TO_EXCLUSIVE_PROCESS(ObjectHeader);
//         if ((!(ExclusiveProcess) && (ObjectHeader->HandleCount)) ||
//             ((ExclusiveProcess) && (ExclusiveProcess != (PsGetCurrentProcess() as *mut _EPROCESS))))
//         {
//             /* This isn't the right process */
//             Status = STATUS_ACCESS_DENIED;
//             goto Quickie;
//         }
//
//         /* Now you got exclusive access */
//         Exclusive = TRUE;
//     }
//     else if ((ObjectHeader->Flags & OB_FLAG_EXCLUSIVE) &&
//              (OBJECT_HEADER_TO_EXCLUSIVE_PROCESS(ObjectHeader)))
//     {
//         /* Caller didn't want exclusive access, but the object is exclusive */
//         Status = STATUS_ACCESS_DENIED;
//         goto Quickie;
//     }
//
//     /*
//      * Check if this is an object that went from 0 handles back to existence,
//      * but doesn't have an open procedure, only a close procedure. This means
//      * that it will never realize that the object is back alive, so we must
//      * fail the request.
//      */
//     if (!(ObjectHeader->HandleCount) &&
//         !(NewObject) &&
//         (ObjectType->TypeInfo.MaintainHandleCount) &&
//         !(ObjectType->TypeInfo.OpenProcedure) &&
//         (ObjectType->TypeInfo.CloseProcedure))
//     {
//         /* Fail */
//         Status = STATUS_UNSUCCESSFUL;
//         goto Quickie;
//     }
//
//     /* Convert MAXIMUM_ALLOWED to GENERIC_ALL */
//     if (*DesiredAccess & MAXIMUM_ALLOWED)
//     {
//         /* Mask out MAXIMUM_ALLOWED and stick GENERIC_ALL instead */
//         *DesiredAccess &= ~MAXIMUM_ALLOWED;
//         *DesiredAccess |= GENERIC_ALL;
//     }
//
//     /* Check if we have to map the GENERIC mask */
//     if (*DesiredAccess & GENERIC_ACCESS)
//     {
//         /* Map it to the correct access masks */
//         RtlMapGenericMask(DesiredAccess,
//                           &ObjectType->TypeInfo.GenericMapping);
//     }
//
//     /* Check if this is an exclusive handle */
//     if (Exclusive)
//     {
//         /* Save the owner process */
//         OBJECT_HEADER_TO_QUOTA_INFO(ObjectHeader)->ExclusiveProcess = Process;
//     }
//
//     /* Increase the handle count */
//     InterlockedIncrementSizeT(&ObjectHeader->HandleCount);
//     ProcessHandleCount = 0;
//
//     /* Check if we have a handle database */
//     if (ObjectType->TypeInfo.MaintainHandleCount)
//     {
//         /* Increment the handle database */
//         Status = ObpIncrementHandleDataBase(ObjectHeader,
//                                             Process,
//                                             &ProcessHandleCount);
//         if (!NT_SUCCESS(Status))
//         {
//             /* FIXME: This should never happen for now */
//             DPRINT1("Unhandled case\n");
//             ASSERT(FALSE);
//             goto Quickie;
//         }
//     }
//
//     /* Release the lock */
//     ObpReleaseObjectLock(ObjectHeader);
//
//     /* Check if we have an open procedure */
//     Status = STATUS_SUCCESS;
//     if (ObjectType->TypeInfo.OpenProcedure)
//     {
//         /* Call it */
//         ObpCalloutStart(&CalloutIrql);
//         Status = ObjectType->TypeInfo.OpenProcedure(ObCreateHandle,
//                                                     Process,
//                                                     Object,
//                                                     *DesiredAccess,
//                                                     ProcessHandleCount);
//         ObpCalloutEnd(CalloutIrql, "Open", ObjectType, Object);
//
//         /* Check if the open procedure failed */
//         if (!NT_SUCCESS(Status))
//         {
//             /* FIXME: This should never happen for now */
//             DPRINT1("Unhandled case\n");
//             ASSERT(FALSE);
//             return Status;
//         }
//     }
//
//     /* Check if we have creator info */
//     CreatorInfo = OBJECT_HEADER_TO_CREATOR_INFO(ObjectHeader);
//     if (CreatorInfo)
//     {
//         /* We do, acquire the lock */
//         ObpEnterObjectTypeMutex(ObjectType);
//
//         /* Insert us on the list */
//         InsertTailList(&ObjectType->TypeList, &CreatorInfo->TypeList);
//
//         /* Release the lock */
//         ObpLeaveObjectTypeMutex(ObjectType);
//     }
//
//     /* Increase total number of handles */
//     Total = InterlockedIncrement((PLONG)&ObjectType->TotalNumberOfHandles);
//     if (Total > ObjectType->HighWaterNumberOfHandles)
//     {
//         /* Fixup count */
//         ObjectType->HighWaterNumberOfHandles = Total;
//     }
//
//     /* Trace call and return */
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - Incremented count for: %p. UNNAMED HC PC %lx %lx\n",
//             __FUNCTION__,
//             Object,
//             ObjectHeader->HandleCount,
//             ObjectHeader->PointerCount);
//     return Status;
//
// Quickie:
//     /* Release lock and return */
//     ObpReleaseObjectLock(ObjectHeader);
//     return Status;
// }
//
// /*++
// * @name ObpCreateUnnamedHandle
// *
// *     The ObpCreateUnnamedHandle routine <FILLMEIN>
// *
// * @param Object
// *        <FILLMEIN>.
// *
// * @param DesiredAccess
// *        <FILLMEIN>.
// *
// * @param AdditionalReferences
// *        <FILLMEIN>.
// *
// * @param HandleAttributes
// *        <FILLMEIN>.
// *
// * @param AccessMode
// *        <FILLMEIN>.
// *
// * @param ReturnedObject
// *        <FILLMEIN>.
// *
// * @param ReturnedHandle
// *        <FILLMEIN>.
// *
// * @return <FILLMEIN>.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// ObpCreateUnnamedHandle(IN PVOID Object,
//                        IN ACCESS_MASK DesiredAccess,
//                        IN ULONG AdditionalReferences,
//                        IN ULONG HandleAttributes,
//                        IN KPROCESSOR_MODE AccessMode,
//                        OUT PVOID *ReturnedObject,
//                        OUT PHANDLE ReturnedHandle)
// {
//     HANDLE_TABLE_ENTRY NewEntry;
//     POBJECT_HEADER ObjectHeader;
//     HANDLE Handle;
//     KAPC_STATE ApcState;
//     BOOLEAN AttachedToProcess = FALSE, KernelHandle = FALSE;
//     PVOID HandleTable;
//     NTSTATUS Status;
//     ACCESS_MASK GrantedAccess;
//     POBJECT_TYPE ObjectType;
//     PAGED_CODE();
//
//     /* Get the object header and type */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
//     ObjectType = ObjectHeader->Type;
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - Creating handle for: %p. UNNAMED. HC PC %lx %lx\n",
//             __FUNCTION__,
//             Object,
//             ObjectHeader->HandleCount,
//             ObjectHeader->PointerCount);
//
//     /* Save the object header */
//     NewEntry.__bindgen_anon_1.Object = ObjectHeader;
//
//     /* Mask out the internal attributes */
//     NewEntry.ObAttributes |= HandleAttributes & OBJ_HANDLE_ATTRIBUTES;
//
//     /* Check if this is a kernel handle */
//     if (HandleAttributes & OBJ_KERNEL_HANDLE)
//     {
//         /* Set the handle table */
//         HandleTable = ObpKernelHandleTable;
//         KernelHandle = TRUE;
//
//         /* Check if we're not in the system process */
//         if (PsGetCurrentProcess() != PsInitialSystemProcess)
//         {
//             /* Attach to the system process */
//             KeStackAttachProcess(&PsInitialSystemProcess->Pcb, &ApcState);
//             AttachedToProcess = TRUE;
//         }
//     }
//     else
//     {
//         /* Get the current handle table */
//         HandleTable = (PsGetCurrentProcess() as *mut _EPROCESS)->ObjectTable;
//     }
//
//     /* Increment the handle count */
//     Status = ObpIncrementUnnamedHandleCount(Object,
//                                             &DesiredAccess,
//                                             AccessMode,
//                                             HandleAttributes,
//                                             (PsGetCurrentProcess() as *mut _EPROCESS));
//     if (!NT_SUCCESS(Status))
//     {
//         /*
//          * We failed (meaning security failure, according to NT Internals)
//          * detach and return
//          */
//         if (AttachedToProcess) KeUnstackDetachProcess(&ApcState);
//         return Status;
//     }
//
//     /* Remove what's not in the valid access mask */
//     GrantedAccess = DesiredAccess & (ObjectType->TypeInfo.ValidAccessMask |
//                                      ACCESS_SYSTEM_SECURITY);
//
//     /* Handle extra references */
//     if (AdditionalReferences)
//     {
//         /* Add them to the header */
//         InterlockedExchangeAddSizeT(&ObjectHeader->PointerCount,
//                                     AdditionalReferences);
//     }
//
//     /* Save the access mask */
//     NewEntry.GrantedAccess = GrantedAccess;
//
//     /*
//      * Create the actual handle. We'll need to do this *after* calling
//      * ObpIncrementHandleCount to make sure that Object Security is valid
//      * (specified in Gl00my documentation on Ob)
//      */
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - Handle Properties: [%p-%lx-%lx]\n",
//             __FUNCTION__,
//             NewEntry.__bindgen_anon_1.Object, NewEntry.ObAttributes & 3, NewEntry.GrantedAccess);
//     Handle = ExCreateHandle(HandleTable, &NewEntry);
//
//     /* Make sure we got a handle */
//     if (Handle)
//     {
//         /* Check if this was a kernel handle */
//         if (KernelHandle) Handle = ObMarkHandleAsKernelHandle(Handle);
//
//         /* Return handle and object */
//         *ReturnedHandle = Handle;
//
//         /* Return the new object only if caller wanted it biased */
//         if ((AdditionalReferences) && (ReturnedObject))
//         {
//             /* Return it */
//             *ReturnedObject = Object;
//         }
//
//         /* Detach if needed */
//         if (AttachedToProcess) KeUnstackDetachProcess(&ApcState);
//
//         /* Trace and return */
//         OBTRACE(OB_HANDLE_DEBUG,
//                 "%s - Returning Handle: %p HC PC %lx %lx\n",
//                 __FUNCTION__,
//                 Handle,
//                 ObjectHeader->HandleCount,
//                 ObjectHeader->PointerCount);
//         return STATUS_SUCCESS;
//     }
//
//     /* Handle extra references */
//     if (AdditionalReferences)
//     {
//         /* Dereference it as many times as required */
//         InterlockedExchangeAddSizeT(&ObjectHeader->PointerCount,
//                                     -(LONG)AdditionalReferences);
//     }
//
//     /* Decrement the handle count and detach */
//     ObpDecrementHandleCount(&ObjectHeader->Body,
//                             (PsGetCurrentProcess() as *mut _EPROCESS),
//                             GrantedAccess,
//                             ObjectType);
//
//     /* Detach and fail */
//     if (AttachedToProcess) KeUnstackDetachProcess(&ApcState);
//     return STATUS_INSUFFICIENT_RESOURCES;
// }
//
// /*++
// * @name ObpCreateHandle
// *
// *     The ObpCreateHandle routine <FILLMEIN>
// *
// * @param OpenReason
// *        <FILLMEIN>.
// *
// * @param Object
// *        <FILLMEIN>.
// *
// * @param Type
// *        <FILLMEIN>.
// *
// * @param AccessState
// *        <FILLMEIN>.
// *
// * @param AdditionalReferences
// *        <FILLMEIN>.
// *
// * @param HandleAttributes
// *        <FILLMEIN>.
// *
// * @param AccessMode
// *        <FILLMEIN>.
// *
// * @param ReturnedObject
// *        <FILLMEIN>.
// *
// * @param ReturnedHandle
// *        <FILLMEIN>.
// *
// * @return <FILLMEIN>.
// *
// * @remarks Cleans up the Lookup Context on return.
// *
// *--*/
// NTSTATUS
// NTAPI
// ObpCreateHandle(IN OB_OPEN_REASON OpenReason,
//                 IN PVOID Object,
//                 IN POBJECT_TYPE Type OPTIONAL,
//                 IN PACCESS_STATE AccessState,
//                 IN ULONG AdditionalReferences,
//                 IN ULONG HandleAttributes,
//                 IN POBP_LOOKUP_CONTEXT Context,
//                 IN KPROCESSOR_MODE AccessMode,
//                 OUT PVOID *ReturnedObject,
//                 OUT PHANDLE ReturnedHandle)
// {
//     HANDLE_TABLE_ENTRY NewEntry;
//     POBJECT_HEADER ObjectHeader;
//     HANDLE Handle;
//     KAPC_STATE ApcState;
//     BOOLEAN AttachedToProcess = FALSE, KernelHandle = FALSE;
//     POBJECT_TYPE ObjectType;
//     PVOID HandleTable;
//     NTSTATUS Status;
//     ACCESS_MASK DesiredAccess, GrantedAccess;
//     PAUX_ACCESS_DATA AuxData;
//     PAGED_CODE();
//
//     /* Get the object header and type */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
//     ObjectType = ObjectHeader->Type;
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - Creating handle for: %p. Reason: %lx. HC PC %lx %lx\n",
//             __FUNCTION__,
//             Object,
//             OpenReason,
//             ObjectHeader->HandleCount,
//             ObjectHeader->PointerCount);
//
//     /* Check if the types match */
//     if ((Type) && (ObjectType != Type))
//     {
//         /* They don't, cleanup */
//         if (Context) ObpReleaseLookupContext(Context);
//         return STATUS_OBJECT_TYPE_MISMATCH;
//     }
//
//     /* Save the object header */
//     NewEntry.__bindgen_anon_1.Object = ObjectHeader;
//
//     /* Check if this is a kernel handle */
//     if (HandleAttributes & OBJ_KERNEL_HANDLE)
//     {
//         /* Set the handle table */
//         HandleTable = ObpKernelHandleTable;
//         KernelHandle = TRUE;
//
//         /* Check if we're not in the system process */
//         if (PsGetCurrentProcess() != PsInitialSystemProcess)
//         {
//             /* Attach to the system process */
//             KeStackAttachProcess(&PsInitialSystemProcess->Pcb, &ApcState);
//             AttachedToProcess = TRUE;
//         }
//     }
//     else
//     {
//         /* Get the current handle table */
//         HandleTable = (PsGetCurrentProcess() as *mut _EPROCESS)->ObjectTable;
//     }
//
//     /* Increment the handle count */
//     Status = ObpIncrementHandleCount(Object,
//                                      AccessState,
//                                      AccessMode,
//                                      HandleAttributes,
//                                      (PsGetCurrentProcess() as *mut _EPROCESS),
//                                      OpenReason);
//     if (!NT_SUCCESS(Status))
//     {
//         /*
//          * We failed (meaning security failure, according to NT Internals)
//          * detach and return
//          */
//         if (Context) ObpReleaseLookupContext(Context);
//         if (AttachedToProcess) KeUnstackDetachProcess(&ApcState);
//         return Status;
//     }
//
//     /* Check if we are doing audits on close */
//     if (AccessState->GenerateOnClose)
//     {
//         /* Force the attribute on */
//         HandleAttributes |= OBJ_AUDIT_OBJECT_CLOSE;
//     }
//
//     /* Mask out the internal attributes */
//     NewEntry.ObAttributes |= (HandleAttributes & OBJ_HANDLE_ATTRIBUTES);
//
//     /* Get the original desired access */
//     DesiredAccess = AccessState->RemainingDesiredAccess |
//                     AccessState->PreviouslyGrantedAccess;
//
//     /* Remove what's not in the valid access mask */
//     GrantedAccess = DesiredAccess & (ObjectType->TypeInfo.ValidAccessMask |
//                                      ACCESS_SYSTEM_SECURITY);
//
//     /* Update the value in the access state */
//     AccessState->PreviouslyGrantedAccess = GrantedAccess;
//
//     /* Get the auxiliary data */
//     AuxData = AccessState->AuxData;
//
//     /* Handle extra references */
//     if (AdditionalReferences)
//     {
//         /* Add them to the header */
//         InterlockedExchangeAddSizeT(&ObjectHeader->PointerCount,
//                                     AdditionalReferences);
//     }
//
//     /* Now we can release the object */
//     if (Context) ObpReleaseLookupContext(Context);
//
//     /* Save the access mask */
//     NewEntry.GrantedAccess = GrantedAccess;
//
//     /*
//      * Create the actual handle. We'll need to do this *after* calling
//      * ObpIncrementHandleCount to make sure that Object Security is valid
//      * (specified in Gl00my documentation on Ob)
//      */
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - Handle Properties: [%p-%lx-%lx]\n",
//             __FUNCTION__,
//             NewEntry.__bindgen_anon_1.Object, NewEntry.ObAttributes & 3, NewEntry.GrantedAccess);
//     Handle = ExCreateHandle(HandleTable, &NewEntry);
//
//     /* Make sure we got a handle */
//     if (Handle)
//     {
//         /* Check if this was a kernel handle */
//         if (KernelHandle) Handle = ObMarkHandleAsKernelHandle(Handle);
//
//         /* Return it */
//         *ReturnedHandle = Handle;
//
//         /* Check if we need to generate on audit */
//         if (AccessState->GenerateAudit)
//         {
//             /* Audit the handle creation */
//             //SeAuditHandleCreation(AccessState, Handle);
//         }
//
//         /* Check if this was a create */
//         if (OpenReason == ObCreateHandle)
//         {
//             /* Check if we need to audit the privileges */
//             if ((AuxData->PrivilegesUsed) &&
//                 (AuxData->PrivilegesUsed->PrivilegeCount))
//             {
//                 /* Do the audit */
// #if 0
//                 SePrivilegeObjectAuditAlarm(Handle,
//                                             &AccessState->
//                                             SubjectSecurityContext,
//                                             GrantedAccess,
//                                             AuxData->PrivilegesUsed,
//                                             TRUE,
//                                             ExGetPreviousMode());
// #endif
//             }
//         }
//
//         /* Return the new object only if caller wanted it biased */
//         if ((AdditionalReferences) && (ReturnedObject))
//         {
//             /* Return it */
//             *ReturnedObject = Object;
//         }
//
//         /* Detach if needed */
//         if (AttachedToProcess) KeUnstackDetachProcess(&ApcState);
//
//         /* Trace and return */
//         OBTRACE(OB_HANDLE_DEBUG,
//                 "%s - Returning Handle: %p HC PC %lx %lx\n",
//                 __FUNCTION__,
//                 Handle,
//                 ObjectHeader->HandleCount,
//                 ObjectHeader->PointerCount);
//         return STATUS_SUCCESS;
//     }
//
//     /* Decrement the handle count and detach */
//     ObpDecrementHandleCount(&ObjectHeader->Body,
//                             (PsGetCurrentProcess() as *mut _EPROCESS),
//                             GrantedAccess,
//                             ObjectType);
//
//     /* Handle extra references */
//     if (AdditionalReferences)
//     {
//         /* Check how many extra references were added */
//         if (AdditionalReferences > 1)
//         {
//             /* Dereference it many times */
//             InterlockedExchangeAddSizeT(&ObjectHeader->PointerCount,
//                                         -(LONG)(AdditionalReferences - 1));
//         }
//
//         /* Dereference the object one last time */
//         ObDereferenceObject(Object);
//     }
//
//     /* Detach if necessary and fail */
//     if (AttachedToProcess) KeUnstackDetachProcess(&ApcState);
//     return STATUS_INSUFFICIENT_RESOURCES;
// }
//
// /*++
// * @name ObpCloseHandle
// *
// *     The ObpCloseHandle routine <FILLMEIN>
// *
// * @param Handle
// *        <FILLMEIN>.
// *
// * @param AccessMode
// *        <FILLMEIN>.
// *
// * @return <FILLMEIN>.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// ObpCloseHandle(IN HANDLE Handle,
//                IN KPROCESSOR_MODE AccessMode)
// {
//     PVOID HandleTable;
//     BOOLEAN AttachedToProcess = FALSE;
//     KAPC_STATE ApcState;
//     PHANDLE_TABLE_ENTRY HandleTableEntry;
//     NTSTATUS Status;
//     PEPROCESS Process = (PsGetCurrentProcess() as *mut _EPROCESS);
//     PAGED_CODE();
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - Closing handle: %p\n", __FUNCTION__, Handle);
//
//     if (AccessMode == KernelMode && Handle == (HANDLE)-1)
//         return STATUS_INVALID_HANDLE;
//
//     /* Check if we're dealing with a kernel handle */
//     if (ObpIsKernelHandle(Handle, AccessMode))
//     {
//         /* Use the kernel table and convert the handle */
//         HandleTable = ObpKernelHandleTable;
//         Handle = ObKernelHandleToHandle(Handle);
//
//         /* Check if we're not in the system process */
//         if (Process != PsInitialSystemProcess)
//         {
//             /* Attach to the system process */
//             KeStackAttachProcess(&PsInitialSystemProcess->Pcb, &ApcState);
//             AttachedToProcess = TRUE;
//         }
//     }
//     else
//     {
//         /* Use the process's handle table */
//         HandleTable = Process->ObjectTable;
//     }
//
//     /* Enter a critical region to protect handle access */
//     KeEnterCriticalRegion();
//
//     /* Get the handle entry */
//     HandleTableEntry = ExMapHandleToPointer(HandleTable, Handle);
//     if (HandleTableEntry)
//     {
//         /* Now close the entry */
//         Status = ObpCloseHandleTableEntry(HandleTable,
//                                           HandleTableEntry,
//                                           Handle,
//                                           AccessMode,
//                                           FALSE);
//
//         /* We can quit the critical region now */
//         KeLeaveCriticalRegion();
//
//         /* Detach and return success */
//         if (AttachedToProcess) KeUnstackDetachProcess(&ApcState);
//     }
//     else
//     {
//         /* We failed, quit the critical region */
//         KeLeaveCriticalRegion();
//
//         /* Detach */
//         if (AttachedToProcess) KeUnstackDetachProcess(&ApcState);
//
//         /* Check if we have a valid handle that's not the process or thread */
//         if ((Handle) &&
//             (Handle != NtCurrentProcess()) &&
//             (Handle != NtCurrentThread()))
//         {
//             /* Check if we came from user mode */
//             if (AccessMode != KernelMode)
//             {
//                 /* Check if we have no debug port */
//                 if (Process->DebugPort)
//                 {
//                     /* Make sure we're not attached */
//                     if (!KeIsAttachedProcess())
//                     {
//                         /* Raise an exception */
//                         return KeRaiseUserException(STATUS_INVALID_HANDLE);
//                     }
//                 }
//             }
//             else
//             {
//                 /* This is kernel mode. Check if we're exiting */
//                 if (!(PsIsThreadTerminating(PsGetCurrentThread())) &&
//                     (Process->Peb))
//                 {
//                     /* Check if the debugger is enabled */
//                     if (KdDebuggerEnabled)
//                     {
//                         /* Bugcheck */
//                         KeBugCheckEx(INVALID_KERNEL_HANDLE, (ULONG_PTR)Handle, 1, 0, 0);
//                     }
//                 }
//             }
//         }
//
//         /* Set invalid status */
//         Status = STATUS_INVALID_HANDLE;
//     }
//
//     /* Return status */
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - Closed handle: %p S: %lx\n",
//             __FUNCTION__, Handle, Status);
//     return Status;
// }
//
// /*++
// * @name ObpSetHandleAttributes
// *
// *     The ObpSetHandleAttributes routine <FILLMEIN>
// *
// * @param HandleTableEntry
// *        <FILLMEIN>.
// *
// * @param Context
// *        <FILLMEIN>.
// *
// * @return <FILLMEIN>.
// *
// * @remarks None.
// *
// *--*/
// BOOLEAN
// NTAPI
// ObpSetHandleAttributes(IN OUT PHANDLE_TABLE_ENTRY HandleTableEntry,
//                        IN ULONG_PTR Context)
// {
//     POBP_SET_HANDLE_ATTRIBUTES_CONTEXT SetHandleInfo = (PVOID)Context;
//     POBJECT_HEADER ObjectHeader = ObpGetHandleObject(HandleTableEntry);
//
//     /* Check if making the handle inheritable */
//     if (SetHandleInfo->Information.Inherit)
//     {
//         /* Check if inheriting is not supported for this object */
//         if (ObjectHeader->Type->TypeInfo.InvalidAttributes & OBJ_INHERIT)
//         {
//             /* Fail without changing anything */
//             return FALSE;
//         }
//
//         /* Set the flag */
//         HandleTableEntry->ObAttributes |= OBJ_INHERIT;
//     }
//     else
//     {
//         /* Otherwise this implies we're removing the flag */
//         HandleTableEntry->ObAttributes &= ~OBJ_INHERIT;
//     }
//
//     /* Check if making the handle protected */
//     if (SetHandleInfo->Information.ProtectFromClose)
//     {
//         /* Set the flag */
//         HandleTableEntry->GrantedAccess |= ObpAccessProtectCloseBit;
//     }
//     else
//     {
//         /* Otherwise, remove it */
//         HandleTableEntry->GrantedAccess &= ~ObpAccessProtectCloseBit;
//     }
//
//     /* Return success */
//     return TRUE;
// }
//
// /*++
// * @name ObpCloseHandleCallback
// *
// *     The ObpCloseHandleCallback routine <FILLMEIN>
// *
// * @param HandleTable
// *        <FILLMEIN>.
// *
// * @param Object
// *        <FILLMEIN>.
// *
// * @param GrantedAccess
// *        <FILLMEIN>.
// *
// * @param Context
// *        <FILLMEIN>.
// *
// * @return <FILLMEIN>.
// *
// * @remarks None.
// *
// *--*/
// BOOLEAN
// NTAPI
// ObpCloseHandleCallback(IN PHANDLE_TABLE_ENTRY HandleTableEntry,
//                        IN HANDLE Handle,
//                        IN PVOID Context)
// {
//     POBP_CLOSE_HANDLE_CONTEXT CloseContext = (POBP_CLOSE_HANDLE_CONTEXT)Context;
//
//     /* Simply decrement the handle count */
//     ObpCloseHandleTableEntry(CloseContext->HandleTable,
//                              HandleTableEntry,
//                              Handle,
//                              CloseContext->AccessMode,
//                              TRUE);
//     return TRUE;
// }
//
// /*++
// * @name ObpDuplicateHandleCallback
// *
// *     The ObpDuplicateHandleCallback routine <FILLMEIN>
// *
// * @param HandleTable
// *        <FILLMEIN>.
// *
// * @param HandleTableEntry
// *        <FILLMEIN>.
// *
// * @param Context
// *        <FILLMEIN>.
// *
// * @return <FILLMEIN>.
// *
// * @remarks None.
// *
// *--*/
// BOOLEAN
// NTAPI
// ObpDuplicateHandleCallback(IN PEPROCESS Process,
//                            IN PHANDLE_TABLE HandleTable,
//                            IN PHANDLE_TABLE_ENTRY OldEntry,
//                            IN PHANDLE_TABLE_ENTRY HandleTableEntry)
// {
//     POBJECT_HEADER ObjectHeader;
//     BOOLEAN Ret = FALSE;
//     ACCESS_STATE AccessState;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Make sure that the handle is inheritable */
//     Ret = (HandleTableEntry->ObAttributes & OBJ_INHERIT) != 0;
//     if (Ret)
//     {
//         /* Get the object header */
//         ObjectHeader = ObpGetHandleObject(HandleTableEntry);
//
//         /* Increment the pointer count */
//         InterlockedIncrementSizeT(&ObjectHeader->PointerCount);
//
//         /* Release the handle lock */
//         ExUnlockHandleTableEntry(HandleTable, OldEntry);
//
//         /* Setup the access state */
//         AccessState.PreviouslyGrantedAccess = HandleTableEntry->GrantedAccess;
//
//         /* Call the shared routine for incrementing handles */
//         Status = ObpIncrementHandleCount(&ObjectHeader->Body,
//                                          &AccessState,
//                                          KernelMode,
//                                          HandleTableEntry->ObAttributes & OBJ_HANDLE_ATTRIBUTES,
//                                          Process,
//                                          ObInheritHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Return failure */
//             ObDereferenceObject(&ObjectHeader->Body);
//             Ret = FALSE;
//         }
//     }
//     else
//     {
//         /* Release the handle lock */
//         ExUnlockHandleTableEntry(HandleTable, OldEntry);
//     }
//
//     /* Return duplication result */
//     return Ret;
// }
//
// /*++
// * @name ObClearProcessHandleTable
// *
// *     The ObClearProcessHandleTable routine clears the handle table
// *     of the given process.
// *
// * @param Process
// *        The process of which the handle table should be cleared.
// *
// * @return None.
// *
// * @remarks None.
// *
// *--*/
// VOID
// NTAPI
// ObClearProcessHandleTable(IN PEPROCESS Process)
// {
//     PHANDLE_TABLE HandleTable;
//     OBP_CLOSE_HANDLE_CONTEXT Context;
//     KAPC_STATE ApcState;
//     BOOLEAN AttachedToProcess = FALSE;
//
//     ASSERT(Process);
//
//     /* Ensure the handle table doesn't go away while we use it */
//     HandleTable = ObReferenceProcessHandleTable(Process);
//     if (!HandleTable) return;
//
//     /* Attach to the current process if needed */
//     if (PsGetCurrentProcess() != Process)
//     {
//         KeStackAttachProcess(&Process->Pcb, &ApcState);
//         AttachedToProcess = TRUE;
//     }
//
//     /* Enter a critical region */
//     KeEnterCriticalRegion();
//
//     /* Fill out the context */
//     Context.AccessMode = UserMode;
//     Context.HandleTable = HandleTable;
//
//     /* Sweep the handle table to close all handles */
//     ExSweepHandleTable(HandleTable,
//                        ObpCloseHandleCallback,
//                        &Context);
//
//     /* Leave the critical region */
//     KeLeaveCriticalRegion();
//
//     /* Detach if needed */
//     if (AttachedToProcess)
//         KeUnstackDetachProcess(&ApcState);
//
//     /* Let the handle table go */
//     ObDereferenceProcessHandleTable(Process);
// }
//
// /*++
// * @name ObInitProcess
// *
// *     The ObInitProcess routine initializes the handle table for the process
// *     to be initialized, by either creating a new one or duplicating it from
// *     the parent process.
// *
// * @param Parent
// *        A parent process (optional).
// *
// * @param Process
// *        The process to initialize.
// *
// * @return Success or failure.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// ObInitProcess(IN PEPROCESS Parent OPTIONAL,
//               IN PEPROCESS Process)
// {
//     PHANDLE_TABLE ParentTable, ObjectTable;
//
//     /* Check for a parent */
//     if (Parent)
//     {
//         /* Reference the parent's table */
//         ParentTable = ObReferenceProcessHandleTable(Parent);
//         if (!ParentTable) return STATUS_PROCESS_IS_TERMINATING;
//
//         /* Duplicate it */
//         ObjectTable = ExDupHandleTable(Process,
//                                        ParentTable,
//                                        ObpDuplicateHandleCallback,
//                                        OBJ_INHERIT);
//     }
//     else
//     {
//         /* Otherwise just create a new table */
//         ParentTable = NULL;
//         ObjectTable = ExCreateHandleTable(Process);
//     }
//
//     /* Make sure we have a table */
//     if (ObjectTable)
//     {
//         /* Associate it */
//         Process->ObjectTable = ObjectTable;
//
//         /* Check for auditing */
//         if (SeDetailedAuditingWithToken(NULL))
//         {
//             /* FIXME: TODO */
//             DPRINT1("Need auditing!\n");
//         }
//
//         /* Get rid of the old table now */
//         if (ParentTable) ObDereferenceProcessHandleTable(Parent);
//
//         /* We are done */
//         return STATUS_SUCCESS;
//     }
//     else
//     {
//         /* Fail */
//         Process->ObjectTable = NULL;
//         if (ParentTable) ObDereferenceProcessHandleTable(Parent);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
// }
//
// /*++
// * @name ObKillProcess
// *
// *     The ObKillProcess routine performs rundown operations on the process,
// *     then clears and destroys its handle table.
// *
// * @param Process
// *        The process to be killed.
// *
// * @return None.
// *
// * @remarks Called by the Object Manager cleanup code (kernel)
// *          when a process is to be destroyed.
// *
// *--*/
// VOID
// NTAPI
// ObKillProcess(IN PEPROCESS Process)
// {
//     PHANDLE_TABLE HandleTable;
//     OBP_CLOSE_HANDLE_CONTEXT Context;
//     BOOLEAN HardErrors;
//     PAGED_CODE();
//
//     /* Wait for process rundown and then complete it */
//     ExWaitForRundownProtectionRelease(&Process->RundownProtect);
//     ExRundownCompleted(&Process->RundownProtect);
//
//     /* Get the object table */
//     HandleTable = Process->ObjectTable;
//     if (!HandleTable) return;
//
//     /* Disable hard errors while we close handles */
//     HardErrors = IoSetThreadHardErrorMode(FALSE);
//
//     /* Enter a critical region */
//     KeEnterCriticalRegion();
//
//     /* Fill out the context */
//     Context.AccessMode = KernelMode;
//     Context.HandleTable = HandleTable;
//
//     /* Sweep the handle table to close all handles */
//     ExSweepHandleTable(HandleTable,
//                        ObpCloseHandleCallback,
//                        &Context);
//     ASSERT(HandleTable->HandleCount == 0);
//
//     /* Leave the critical region */
//     KeLeaveCriticalRegion();
//
//     /* Re-enable hard errors */
//     IoSetThreadHardErrorMode(HardErrors);
//
//     /* Destroy the object table */
//     Process->ObjectTable = NULL;
//     ExDestroyHandleTable(HandleTable, NULL);
// }
//
// NTSTATUS
// NTAPI
// ObDuplicateObject(IN PEPROCESS SourceProcess,
//                   IN HANDLE SourceHandle,
//                   IN PEPROCESS TargetProcess OPTIONAL,
//                   IN PHANDLE TargetHandle OPTIONAL,
//                   IN ACCESS_MASK DesiredAccess,
//                   IN ULONG HandleAttributes,
//                   IN ULONG Options,
//                   IN KPROCESSOR_MODE PreviousMode)
// {
//     HANDLE_TABLE_ENTRY NewHandleEntry;
//     BOOLEAN AttachedToProcess = FALSE;
//     PVOID SourceObject;
//     POBJECT_HEADER ObjectHeader;
//     POBJECT_TYPE ObjectType;
//     HANDLE NewHandle;
//     KAPC_STATE ApcState;
//     NTSTATUS Status;
//     ACCESS_MASK TargetAccess, SourceAccess;
//     ACCESS_STATE AccessState;
//     PACCESS_STATE PassedAccessState = NULL;
//     AUX_ACCESS_DATA AuxData;
//     PHANDLE_TABLE HandleTable;
//     OBJECT_HANDLE_INFORMATION HandleInformation;
//     ULONG AuditMask;
//     BOOLEAN KernelHandle = FALSE;
//
//     PAGED_CODE();
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - Duplicating handle: %p for %p into %p\n",
//             __FUNCTION__,
//             SourceHandle,
//             SourceProcess,
//             TargetProcess);
//
//     /* Assume failure */
//     if (TargetHandle) *TargetHandle = NULL;
//
//     /* Check if we're not duplicating the same access */
//     if (!(Options & DUPLICATE_SAME_ACCESS))
//     {
//         /* Validate the desired access */
//         Status = STATUS_SUCCESS; //ObpValidateDesiredAccess(DesiredAccess);
//         if (!NT_SUCCESS(Status)) return Status;
//     }
//
//     /* Reference the object table */
//     HandleTable = ObReferenceProcessHandleTable(SourceProcess);
//     if (!HandleTable) return STATUS_PROCESS_IS_TERMINATING;
//
//     /* Reference the process object */
//     Status = ObpReferenceProcessObjectByHandle(SourceHandle,
//                                                SourceProcess,
//                                                HandleTable,
//                                                PreviousMode,
//                                                &SourceObject,
//                                                &HandleInformation,
//                                                &AuditMask);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         ObDereferenceProcessHandleTable(SourceProcess);
//         return Status;
//     }
//     else
//     {
//         /* Check if we have to don't have to audit object close */
//         if (!(HandleInformation.HandleAttributes & OBJ_AUDIT_OBJECT_CLOSE))
//         {
//             /* Then there is no audit mask */
//             AuditMask = 0;
//         }
//     }
//
//     /* Check if there's no target process */
//     if (!TargetProcess)
//     {
//         /* Check if the caller wanted actual duplication */
//         if (!(Options & DUPLICATE_CLOSE_SOURCE))
//         {
//             /* Invalid request */
//             Status = STATUS_INVALID_PARAMETER;
//         }
//         else
//         {
//             /* Otherwise, do the attach */
//             KeStackAttachProcess(&SourceProcess->Pcb, &ApcState);
//
//             /* Close the handle and detach */
//             NtClose(SourceHandle);
//             KeUnstackDetachProcess(&ApcState);
//         }
//
//         /* Return */
//         ObDereferenceProcessHandleTable(SourceProcess);
//         ObDereferenceObject(SourceObject);
//         return Status;
//     }
//
//     /* Create a kernel handle if asked, but only in the system process */
//     if (PreviousMode == KernelMode &&
//         HandleAttributes & OBJ_KERNEL_HANDLE &&
//         TargetProcess == PsInitialSystemProcess)
//     {
//         KernelHandle = TRUE;
//     }
//
//     /* Get the target handle table */
//     HandleTable = ObReferenceProcessHandleTable(TargetProcess);
//     if (!HandleTable)
//     {
//         /* Check if the caller wanted us to close the handle */
//         if (Options & DUPLICATE_CLOSE_SOURCE)
//         {
//             /* Do the attach */
//             KeStackAttachProcess(&SourceProcess->Pcb, &ApcState);
//
//             /* Close the handle and detach */
//             NtClose(SourceHandle);
//             KeUnstackDetachProcess(&ApcState);
//         }
//
//         /* Return */
//         ObDereferenceProcessHandleTable(SourceProcess);
//         ObDereferenceObject(SourceObject);
//         return STATUS_PROCESS_IS_TERMINATING;
//     }
//
//     /* Get the source access */
//     SourceAccess = HandleInformation.GrantedAccess;
//
//     /* Check if we're not in the target process */
//     if (TargetProcess != (PsGetCurrentProcess() as *mut _EPROCESS))
//     {
//         /* Attach to it */
//         KeStackAttachProcess(&TargetProcess->Pcb, &ApcState);
//         AttachedToProcess = TRUE;
//     }
//
//     /* Check if we're duplicating the attributes */
//     if (Options & DUPLICATE_SAME_ATTRIBUTES)
//     {
//         /* Duplicate them */
//         HandleAttributes = HandleInformation.HandleAttributes;
//     }
//     else
//     {
//         /* Don't allow caller to bypass auditing */
//         HandleAttributes |= HandleInformation.HandleAttributes &
//                             OBJ_AUDIT_OBJECT_CLOSE;
//     }
//
//     /* Check if we're duplicating the access */
//     if (Options & DUPLICATE_SAME_ACCESS) DesiredAccess = SourceAccess;
//
//     /* Get object data */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(SourceObject);
//     ObjectType = ObjectHeader->Type;
//
//     /* Fill out the entry */
//     RtlZeroMemory(&NewHandleEntry, sizeof(HANDLE_TABLE_ENTRY));
//     NewHandleEntry.__bindgen_anon_1.Object = ObjectHeader;
//     NewHandleEntry.ObAttributes |= (HandleAttributes & OBJ_HANDLE_ATTRIBUTES);
//
//     /* Check if we're using a generic mask */
//     if (DesiredAccess & GENERIC_ACCESS)
//     {
//         /* Map it */
//         RtlMapGenericMask(&DesiredAccess,
//                           &ObjectType->TypeInfo.GenericMapping);
//     }
//
//     /* Set the target access, always propagate ACCESS_SYSTEM_SECURITY */
//     TargetAccess = DesiredAccess & (ObjectType->TypeInfo.ValidAccessMask |
//                                     ACCESS_SYSTEM_SECURITY);
//     NewHandleEntry.GrantedAccess = TargetAccess;
//
//     /* Check if we're asking for new access */
//     if (TargetAccess & ~SourceAccess)
//     {
//         /* We are. We need the security procedure to validate this */
//         if (ObjectType->TypeInfo.SecurityProcedure == SeDefaultObjectMethod)
//         {
//             /* Use our built-in access state */
//             PassedAccessState = &AccessState;
//             Status = SeCreateAccessState(&AccessState,
//                                          &AuxData,
//                                          TargetAccess,
//                                          &ObjectType->TypeInfo.GenericMapping);
//         }
//         else
//         {
//             /* Otherwise we can't allow this privilege elevation */
//             Status = STATUS_ACCESS_DENIED;
//         }
//     }
//     else
//     {
//         /* We don't need an access state */
//         Status = STATUS_SUCCESS;
//     }
//
//     /* Make sure the access state was created OK */
//     if (NT_SUCCESS(Status))
//     {
//         /* Add a new handle */
//         Status = ObpIncrementHandleCount(SourceObject,
//                                          PassedAccessState,
//                                          PreviousMode,
//                                          HandleAttributes,
//                                          (PsGetCurrentProcess() as *mut _EPROCESS),
//                                          ObDuplicateHandle);
//     }
//
//     /* Check if we were attached */
//     if (AttachedToProcess)
//     {
//         /* We can safely detach now */
//         KeUnstackDetachProcess(&ApcState);
//         AttachedToProcess = FALSE;
//     }
//
//     /* Check if we have to close the source handle */
//     if (Options & DUPLICATE_CLOSE_SOURCE)
//     {
//         /* Attach and close */
//         KeStackAttachProcess(&SourceProcess->Pcb, &ApcState);
//         NtClose(SourceHandle);
//         KeUnstackDetachProcess(&ApcState);
//     }
//
//     /* Check if we had an access state */
//     if (PassedAccessState) SeDeleteAccessState(PassedAccessState);
//
//     /* Now check if incrementing actually failed */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Dereference handle tables */
//         ObDereferenceProcessHandleTable(SourceProcess);
//         ObDereferenceProcessHandleTable(TargetProcess);
//
//         /* Dereference the source object */
//         ObDereferenceObject(SourceObject);
//         return Status;
//     }
//
//     if (NewHandleEntry.ObAttributes & OBJ_PROTECT_CLOSE)
//     {
//         NewHandleEntry.ObAttributes &= ~OBJ_PROTECT_CLOSE;
//         NewHandleEntry.GrantedAccess |= ObpAccessProtectCloseBit;
//     }
//
//     /* Now create the handle */
//     NewHandle = ExCreateHandle(HandleTable, &NewHandleEntry);
//     if (!NewHandle)
//     {
//         /* Undo the increment */
//         ObpDecrementHandleCount(SourceObject,
//                                 TargetProcess,
//                                 TargetAccess,
//                                 ObjectType);
//
//         /* Deference the object and set failure status */
//         ObDereferenceObject(SourceObject);
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Mark it as a kernel handle if requested */
//     if (KernelHandle)
//     {
//         NewHandle = ObMarkHandleAsKernelHandle(NewHandle);
//     }
//
//     /* Return the handle */
//     if (TargetHandle) *TargetHandle = NewHandle;
//
//     /* Dereference handle tables */
//     ObDereferenceProcessHandleTable(SourceProcess);
//     ObDereferenceProcessHandleTable(TargetProcess);
//
//     /* Return status */
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - Duplicated handle: %p for %p into %p. Source: %p HC PC %lx %lx\n",
//             __FUNCTION__,
//             NewHandle,
//             SourceProcess,
//             TargetProcess,
//             SourceObject,
//             ObjectHeader->PointerCount,
//             ObjectHeader->HandleCount);
//     return Status;
// }
//
// /* PUBLIC FUNCTIONS *********************************************************/
//
// /*++
// * @name ObOpenObjectByName
// * @implemented NT4
// *
// *     The ObOpenObjectByName routine <FILLMEIN>
// *
// * @param ObjectAttributes
// *        <FILLMEIN>.
// *
// * @param ObjectType
// *        <FILLMEIN>.
// *
// * @param AccessMode
// *        <FILLMEIN>.
// *
// * @param PassedAccessState
// *        <FILLMEIN>.
// *
// * @param DesiredAccess
// *        <FILLMEIN>.
// *
// * @param ParseContext
// *        <FILLMEIN>.
// *
// * @param Handle
// *        <FILLMEIN>.
// *
// * @return <FILLMEIN>.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// ObOpenObjectByName(IN POBJECT_ATTRIBUTES ObjectAttributes,
//                    IN POBJECT_TYPE ObjectType,
//                    IN KPROCESSOR_MODE AccessMode,
//                    IN PACCESS_STATE PassedAccessState,
//                    IN ACCESS_MASK DesiredAccess,
//                    IN OUT PVOID ParseContext,
//                    OUT PHANDLE Handle)
// {
//     PVOID Object = NULL;
//     UNICODE_STRING ObjectName;
//     NTSTATUS Status, Status2;
//     POBJECT_HEADER ObjectHeader;
//     PGENERIC_MAPPING GenericMapping = NULL;
//     OB_OPEN_REASON OpenReason;
//     POB_TEMP_BUFFER TempBuffer;
//     PAGED_CODE();
//
//     /* Assume failure */
//     *Handle = NULL;
//
//     /* Check if we didn't get any Object Attributes */
//     if (!ObjectAttributes)
//     {
//         /* Fail with special status code */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Allocate the temporary buffer */
//     TempBuffer = ExAllocatePoolWithTag(NonPagedPool,
//                                        sizeof(OB_TEMP_BUFFER),
//                                        TAG_OB_TEMP_STORAGE);
//     if (!TempBuffer) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Capture all the info */
//     Status = ObpCaptureObjectCreateInformation(ObjectAttributes,
//                                                AccessMode,
//                                                AccessMode,
//                                                TRUE,
//                                                &TempBuffer->ObjectCreateInfo,
//                                                &ObjectName);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         ExFreePoolWithTag(TempBuffer, TAG_OB_TEMP_STORAGE);
//         return Status;
//     }
//
//     /* Check if we didn't get an access state */
//     if (!PassedAccessState)
//     {
//         /* Try to get the generic mapping if we can */
//         if (ObjectType) GenericMapping = &ObjectType->TypeInfo.GenericMapping;
//
//         /* Use our built-in access state */
//         PassedAccessState = &TempBuffer->LocalAccessState;
//         Status = SeCreateAccessState(&TempBuffer->LocalAccessState,
//                                      &TempBuffer->AuxData,
//                                      DesiredAccess,
//                                      GenericMapping);
//         if (!NT_SUCCESS(Status)) goto Quickie;
//     }
//
//     /* Get the security descriptor */
//     if (TempBuffer->ObjectCreateInfo.SecurityDescriptor)
//     {
//         /* Save it in the access state */
//         PassedAccessState->SecurityDescriptor =
//             TempBuffer->ObjectCreateInfo.SecurityDescriptor;
//     }
//
//     /* Validate the access mask */
//     Status = ObpValidateAccessMask(PassedAccessState);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Cleanup after lookup */
//         ObpReleaseLookupContext(&TempBuffer->LookupContext);
//         goto Cleanup;
//     }
//
//     /* Now do the lookup */
//     Status = ObpLookupObjectName(TempBuffer->ObjectCreateInfo.RootDirectory,
//                                  &ObjectName,
//                                  TempBuffer->ObjectCreateInfo.Attributes,
//                                  ObjectType,
//                                  AccessMode,
//                                  ParseContext,
//                                  TempBuffer->ObjectCreateInfo.SecurityQos,
//                                  NULL,
//                                  PassedAccessState,
//                                  &TempBuffer->LookupContext,
//                                  &Object);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Cleanup after lookup */
//         ObpReleaseLookupContext(&TempBuffer->LookupContext);
//         goto Cleanup;
//     }
//
//     /* Check if this object has create information */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
//     if (ObjectHeader->Flags & OB_FLAG_CREATE_INFO)
//     {
//         /* Then we are creating a new handle */
//         OpenReason = ObCreateHandle;
//
//         /* Check if we still have create info */
//         if (ObjectHeader->ObjectCreateInfo)
//         {
//             /* Free it */
//             ObpFreeObjectCreateInformation(ObjectHeader->
//                                                 ObjectCreateInfo);
//             ObjectHeader->ObjectCreateInfo = NULL;
//         }
//     }
//     else
//     {
//         /* Otherwise, we are merely opening it */
//         OpenReason = ObOpenHandle;
//     }
//
//     /* Check if we have invalid object attributes */
//     if (ObjectHeader->Type->TypeInfo.InvalidAttributes &
//         TempBuffer->ObjectCreateInfo.Attributes)
//     {
//         /* Set failure code */
//         Status = STATUS_INVALID_PARAMETER;
//
//         /* Cleanup after lookup */
//         ObpReleaseLookupContext(&TempBuffer->LookupContext);
//
//         /* Dereference the object */
//         ObDereferenceObject(Object);
//     }
//     else
//     {
//         /* Create the actual handle now */
//         Status2 = ObpCreateHandle(OpenReason,
//                                   Object,
//                                   ObjectType,
//                                   PassedAccessState,
//                                   0,
//                                   TempBuffer->ObjectCreateInfo.Attributes,
//                                   &TempBuffer->LookupContext,
//                                   AccessMode,
//                                   NULL,
//                                   Handle);
//         if (!NT_SUCCESS(Status2))
//         {
//             ObDereferenceObject(Object);
//             Status = Status2;
//         }
//     }
//
// Cleanup:
//     /* Delete the access state */
//     if (PassedAccessState == &TempBuffer->LocalAccessState)
//     {
//         SeDeleteAccessState(PassedAccessState);
//     }
//
// Quickie:
//     /* Release the object attributes and temporary buffer */
//     ObpReleaseObjectCreateInformation(&TempBuffer->ObjectCreateInfo);
//     if (ObjectName.Buffer) ObpFreeObjectNameBuffer(&ObjectName);
//     ExFreePoolWithTag(TempBuffer, TAG_OB_TEMP_STORAGE);
//
//     /* Return status */
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - returning Object %p with PC S: %lx %lx\n",
//             __FUNCTION__,
//             Object,
//             Object ? OBJECT_TO_OBJECT_HEADER(Object)->PointerCount : -1,
//             Status);
//     return Status;
// }
//
// /*++
// * @name ObOpenObjectByPointer
// * @implemented NT4
// *
// *     The ObOpenObjectByPointer routine <FILLMEIN>
// *
// * @param Object
// *        <FILLMEIN>.
// *
// * @param HandleAttributes
// *        <FILLMEIN>.
// *
// * @param PassedAccessState
// *        <FILLMEIN>.
// *
// * @param DesiredAccess
// *        <FILLMEIN>.
// *
// * @param ObjectType
// *        <FILLMEIN>.
// *
// * @param AccessMode
// *        <FILLMEIN>.
// *
// * @param Handle
// *        <FILLMEIN>.
// *
// * @return <FILLMEIN>.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// ObOpenObjectByPointer(IN PVOID Object,
//                       IN ULONG HandleAttributes,
//                       IN PACCESS_STATE PassedAccessState,
//                       IN ACCESS_MASK DesiredAccess,
//                       IN POBJECT_TYPE ObjectType,
//                       IN KPROCESSOR_MODE AccessMode,
//                       OUT PHANDLE Handle)
// {
//     POBJECT_HEADER Header;
//     NTSTATUS Status;
//     ACCESS_STATE AccessState;
//     AUX_ACCESS_DATA AuxData;
//     PAGED_CODE();
//
//     /* Assume failure */
//     *Handle = NULL;
//
//     /* Reference the object */
//     Status = ObReferenceObjectByPointer(Object,
//                                         0,
//                                         ObjectType,
//                                         AccessMode);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Get the Header Info */
//     Header = OBJECT_TO_OBJECT_HEADER(Object);
//
//     /* Check if we didn't get an access state */
//     if (!PassedAccessState)
//     {
//         /* Use our built-in access state */
//         PassedAccessState = &AccessState;
//         Status = SeCreateAccessState(&AccessState,
//                                      &AuxData,
//                                      DesiredAccess,
//                                      &Header->Type->TypeInfo.GenericMapping);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Fail */
//             ObDereferenceObject(Object);
//             return Status;
//         }
//     }
//
//     /* Check if we have invalid object attributes */
//     if (Header->Type->TypeInfo.InvalidAttributes & HandleAttributes)
//     {
//         /* Delete the access state */
//         if (PassedAccessState == &AccessState)
//         {
//             SeDeleteAccessState(PassedAccessState);
//         }
//
//         /* Dereference the object */
//         ObDereferenceObject(Object);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Create the handle */
//     Status = ObpCreateHandle(ObOpenHandle,
//                              Object,
//                              ObjectType,
//                              PassedAccessState,
//                              0,
//                              HandleAttributes,
//                              NULL,
//                              AccessMode,
//                              NULL,
//                              Handle);
//     if (!NT_SUCCESS(Status)) ObDereferenceObject(Object);
//
//     /* Delete the access state */
//     if (PassedAccessState == &AccessState)
//     {
//         SeDeleteAccessState(PassedAccessState);
//     }
//
//     /* Return */
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - returning Object with PC S: %lx %lx\n",
//             __FUNCTION__,
//             OBJECT_TO_OBJECT_HEADER(Object)->PointerCount,
//             Status);
//     return Status;
// }
//
// /*++
// * @name ObFindHandleForObject
// * @implemented NT4
// *
// *     The ObFindHandleForObject routine <FILLMEIN>
// *
// * @param Process
// *        <FILLMEIN>.
// *
// * @param Object
// *        <FILLMEIN>.
// *
// * @param ObjectType
// *        <FILLMEIN>.
// *
// * @param HandleInformation
// *        <FILLMEIN>.
// *
// * @param HandleReturn
// *        <FILLMEIN>.
// *
// * @return <FILLMEIN>.
// *
// * @remarks None.
// *
// *--*/
// BOOLEAN
// NTAPI
// ObFindHandleForObject(IN PEPROCESS Process,
//                       IN PVOID Object,
//                       IN POBJECT_TYPE ObjectType,
//                       IN POBJECT_HANDLE_INFORMATION HandleInformation,
//                       OUT PHANDLE Handle)
// {
//     OBP_FIND_HANDLE_DATA FindData;
//     BOOLEAN Result = FALSE;
//     PVOID ObjectTable;
//
//     /* Make sure we have an object table */
//     ObjectTable = ObReferenceProcessHandleTable(Process);
//     if (ObjectTable)
//     {
//         /* Check if we have an object */
//         if (Object)
//         {
//             /* Set its header */
//             FindData.ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
//         }
//         else
//         {
//             /* Otherwise, no object to match*/
//             FindData.ObjectHeader = NULL;
//         }
//
//         /* Set other information */
//         FindData.ObjectType = ObjectType;
//         FindData.HandleInformation = HandleInformation;
//
//         /* Enumerate the handle table */
//         if (ExEnumHandleTable(Process->ObjectTable,
//                               ObpEnumFindHandleProcedure,
//                               &FindData,
//                               Handle))
//         {
//             /* Set success */
//             Result = TRUE;
//         }
//
//         /* Let go of the table */
//         ObDereferenceProcessHandleTable(Process);
//     }
//
//     /* Return the result */
//     return Result;
// }
//
// /*++
// * @name ObInsertObject
// * @implemented NT4
// *
// *     The ObInsertObject routine <FILLMEIN>
// *
// * @param Object
// *        <FILLMEIN>.
// *
// * @param PassedAccessState
// *        <FILLMEIN>.
// *
// * @param DesiredAccess
// *        <FILLMEIN>.
// *
// * @param AdditionalReferences
// *        <FILLMEIN>.
// *
// * @param ReferencedObject
// *        <FILLMEIN>.
// *
// * @param Handle
// *        <FILLMEIN>.
// *
// * @return <FILLMEIN>.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// ObInsertObject(IN PVOID Object,
//                IN PACCESS_STATE AccessState OPTIONAL,
//                IN ACCESS_MASK DesiredAccess,
//                IN ULONG ObjectPointerBias,
//                OUT PVOID *NewObject OPTIONAL,
//                OUT PHANDLE Handle)
// {
//     POBJECT_CREATE_INFORMATION ObjectCreateInfo;
//     POBJECT_HEADER ObjectHeader;
//     POBJECT_TYPE ObjectType;
//     PUNICODE_STRING ObjectName;
//     PVOID InsertObject;
//     PSECURITY_DESCRIPTOR ParentDescriptor = NULL;
//     BOOLEAN SdAllocated = FALSE;
//     POBJECT_HEADER_NAME_INFO ObjectNameInfo;
//     OBP_LOOKUP_CONTEXT Context;
//     ACCESS_STATE LocalAccessState;
//     AUX_ACCESS_DATA AuxData;
//     OB_OPEN_REASON OpenReason;
//     KPROCESSOR_MODE PreviousMode;
//     NTSTATUS Status = STATUS_SUCCESS, RealStatus;
//     BOOLEAN IsNewObject;
//     PAGED_CODE();
//
//     /* Get the Header */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
//
//     /* Detect invalid insert */
//     if (!(ObjectHeader->Flags & OB_FLAG_CREATE_INFO))
//     {
//         /* Display warning and break into debugger */
//         DPRINT1("OB: Attempting to insert existing object %p\n", Object);
//         DbgBreakPoint();
//
//         /* Allow debugger to continue */
//         ObDereferenceObject(Object);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Get the create and name info, as well as the object type */
//     ObjectCreateInfo = ObjectHeader->ObjectCreateInfo;
//     ObjectNameInfo = ObpReferenceNameInfo(ObjectHeader);
//     ObjectType = ObjectHeader->Type;
//     ObjectName = NULL;
//
//     /* Check if this is an named object */
//     if ((ObjectNameInfo) && (ObjectNameInfo->Name.Buffer))
//     {
//         /* Get the object name */
//         ObjectName = &ObjectNameInfo->Name;
//     }
//
//     /* Sanity check */
//     ASSERT((Handle) ||
//            ((ObjectPointerBias == 0) &&
//             (ObjectName == NULL) &&
//             (ObjectType->TypeInfo.SecurityRequired) &&
//             (NewObject == NULL)));
//
//     /* Check if the object is unnamed and also doesn't have security */
//     PreviousMode = KeGetPreviousMode();
//     if (!(ObjectType->TypeInfo.SecurityRequired) && !(ObjectName))
//     {
//         /* Assume failure */
//         *Handle = NULL;
//         ObjectHeader->ObjectCreateInfo = NULL;
//
//         /* Create the handle */
//         Status = ObpCreateUnnamedHandle(Object,
//                                         DesiredAccess,
//                                         ObjectPointerBias + 1,
//                                         ObjectCreateInfo->Attributes,
//                                         PreviousMode,
//                                         NewObject,
//                                         Handle);
//
//         /* Free the create information */
//         ObpFreeObjectCreateInformation(ObjectCreateInfo);
//
//         /* Release the object name information */
//         ObpDereferenceNameInfo(ObjectNameInfo);
//
//         /* Remove the extra keep-alive reference */
//         ObDereferenceObject(Object);
//
//         /* Return */
//         OBTRACE(OB_HANDLE_DEBUG,
//                 "%s - returning Object with PC S: %lx %lx\n",
//                 __FUNCTION__,
//                 ObjectHeader->PointerCount,
//                 Status);
//         return Status;
//     }
//
//     /* Check if we didn't get an access state */
//     if (!AccessState)
//     {
//         /* Use our built-in access state */
//         AccessState = &LocalAccessState;
//         Status = SeCreateAccessState(&LocalAccessState,
//                                      &AuxData,
//                                      DesiredAccess,
//                                      &ObjectType->TypeInfo.GenericMapping);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Fail */
//             ObpDereferenceNameInfo(ObjectNameInfo);
//             ObDereferenceObject(Object);
//             return Status;
//         }
//     }
//
//     /* Save the security descriptor */
//     AccessState->SecurityDescriptor = ObjectCreateInfo->SecurityDescriptor;
//
//     /* Validate the access mask */
//     Status = ObpValidateAccessMask(AccessState);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         ObpDereferenceNameInfo(ObjectNameInfo);
//         ObDereferenceObject(Object);
//         return Status;
//     }
//
//     /* Setup a lookup context */
//     ObpInitializeLookupContext(&Context);
//     InsertObject = Object;
//     OpenReason = ObCreateHandle;
//
//     /* Check if the object is named */
//     if (ObjectName)
//     {
//         /* Look it up */
//         Status = ObpLookupObjectName(ObjectCreateInfo->RootDirectory,
//                                      ObjectName,
//                                      ObjectCreateInfo->Attributes,
//                                      ObjectType,
//                                      (ObjectHeader->Flags & OB_FLAG_KERNEL_MODE) ?
//                                      KernelMode : UserMode,
//                                      ObjectCreateInfo->ParseContext,
//                                      ObjectCreateInfo->SecurityQos,
//                                      Object,
//                                      AccessState,
//                                      &Context,
//                                      &InsertObject);
//
//         /* Check if we found an object that doesn't match the one requested */
//         if ((NT_SUCCESS(Status)) && (InsertObject) && (Object != InsertObject))
//         {
//             /* This means we're opening an object, not creating a new one */
//             OpenReason = ObOpenHandle;
//
//             /* Make sure the caller said it's OK to do this */
//             if (ObjectCreateInfo->Attributes & OBJ_OPENIF)
//             {
//                 /* He did, but did he want this type? */
//                 if (ObjectType != OBJECT_TO_OBJECT_HEADER(InsertObject)->Type)
//                 {
//                     /* Wrong type, so fail */
//                     Status = STATUS_OBJECT_TYPE_MISMATCH;
//                 }
//                 else
//                 {
//                     /* Right type, so warn */
//                     Status = STATUS_OBJECT_NAME_EXISTS;
//                 }
//             }
//             else
//             {
//                 /* Check if this was a symbolic link */
//                 if (OBJECT_TO_OBJECT_HEADER(InsertObject)->Type ==
//                     ObpSymbolicLinkObjectType)
//                 {
//                     /* Dereference it */
//                     ObDereferenceObject(InsertObject);
//                 }
//
//                 /* Caller wanted to create a new object, fail */
//                 Status = STATUS_OBJECT_NAME_COLLISION;
//             }
//         }
//
//         /* Check if anything until now failed */
//         if (!NT_SUCCESS(Status))
//         {
//             /* Cleanup after lookup */
//             ObpReleaseLookupContext(&Context);
//
//             /* Remove query reference that we added */
//             ObpDereferenceNameInfo(ObjectNameInfo);
//
//             /* Dereference the object and delete the access state */
//             ObDereferenceObject(Object);
//             if (AccessState == &LocalAccessState)
//             {
//                 /* We used a local one; delete it */
//                 SeDeleteAccessState(AccessState);
//             }
//
//             /* Return failure code */
//             return Status;
//         }
//         else
//         {
//             /* Check if this is a symbolic link */
//             if (ObjectType == ObpSymbolicLinkObjectType)
//             {
//                 /* Create the internal name */
//                 ObpCreateSymbolicLinkName(Object);
//             }
//         }
//     }
//
//     /* Now check if this object is being created */
//     if (InsertObject == Object)
//     {
//         /* Check if it's named or forces security */
//         if ((ObjectName) || (ObjectType->TypeInfo.SecurityRequired))
//         {
//             /* Make sure it's inserted into an object directory */
//             if ((ObjectNameInfo) && (ObjectNameInfo->Directory))
//             {
//                 /* Get the current descriptor */
//                 ObGetObjectSecurity(ObjectNameInfo->Directory,
//                                     &ParentDescriptor,
//                                     &SdAllocated);
//             }
//
//             /* Now assign it */
//             Status = ObAssignSecurity(AccessState,
//                                       ParentDescriptor,
//                                       Object,
//                                       ObjectType);
//
//             /* Check if we captured one */
//             if (ParentDescriptor)
//             {
//                 /* We did, release it */
//                 ObReleaseObjectSecurity(ParentDescriptor, SdAllocated);
//             }
//             else if (NT_SUCCESS(Status))
//             {
//                 /* Other we didn't, but we were able to use the current SD */
//                 SeReleaseSecurityDescriptor(ObjectCreateInfo->SecurityDescriptor,
//                                             ObjectCreateInfo->ProbeMode,
//                                             TRUE);
//
//                 /* Clear the current one */
//                 AccessState->SecurityDescriptor =
//                 ObjectCreateInfo->SecurityDescriptor = NULL;
//             }
//         }
//
//         /* Check if anything until now failed */
//         if (!NT_SUCCESS(Status))
//         {
//             /* Check if the directory was added */
//             if (Context.DirectoryLocked)
//             {
//                 /* Weird case where we need to do a manual delete */
//                 DPRINT1("Unhandled path\n");
//                 ASSERT(FALSE);
//             }
//
//             /* Cleanup the lookup */
//             ObpReleaseLookupContext(&Context);
//
//             /* Remove query reference that we added */
//             ObpDereferenceNameInfo(ObjectNameInfo);
//
//             /* Dereference the object and delete the access state */
//             ObDereferenceObject(Object);
//             if (AccessState == &LocalAccessState)
//             {
//                 /* We used a local one; delete it */
//                 SeDeleteAccessState(AccessState);
//             }
//
//             /* Return failure code */
//             ASSERT(FALSE);
//             return Status;
//         }
//     }
//
//     /* Save the actual status until here */
//     RealStatus = Status;
//
//     /* Check if caller wants us to create a handle */
//     ObjectHeader->ObjectCreateInfo = NULL;
//     if (Handle)
//     {
//         /* Create the handle */
//         Status = ObpCreateHandle(OpenReason,
//                                  InsertObject,
//                                  NULL,
//                                  AccessState,
//                                  ObjectPointerBias + 1,
//                                  ObjectCreateInfo->Attributes,
//                                  &Context,
//                                  PreviousMode,
//                                  NewObject,
//                                  Handle);
//         if (!NT_SUCCESS(Status))
//         {
//             /* If the object had a name, backout everything */
//             if (ObjectName) ObpDeleteNameCheck(Object);
//
//             /* Return the status of the failure */
//             *Handle = NULL;
//             RealStatus = Status;
//         }
//
//         /* Remove a query reference */
//         ObpDereferenceNameInfo(ObjectNameInfo);
//
//         /* Remove the extra keep-alive reference */
//         ObDereferenceObject(Object);
//     }
//     else
//     {
//         /* Otherwise, lock the object */
//         ObpAcquireObjectLock(ObjectHeader);
//
//         /* And charge quota for the process to make it appear as used */
//         RealStatus = ObpChargeQuotaForObject(ObjectHeader,
//                                              ObjectType,
//                                              &IsNewObject);
//
//         /* Release the lock */
//         ObpReleaseObjectLock(ObjectHeader);
//
//         /* Check if we failed and dereference the object if so */
//         if (!NT_SUCCESS(RealStatus)) ObDereferenceObject(Object);
//     }
//
//     /* We can delete the Create Info now */
//     ObpFreeObjectCreateInformation(ObjectCreateInfo);
//
//     /* Check if we created our own access state and delete it if so */
//     if (AccessState == &LocalAccessState) SeDeleteAccessState(AccessState);
//
//     /* Return status code */
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - returning Object with PC RS/S: %lx %lx %lx\n",
//             __FUNCTION__,
//             OBJECT_TO_OBJECT_HEADER(Object)->PointerCount,
//             RealStatus, Status);
//     return RealStatus;
// }
//
// /*++
// * @name ObSetHandleAttributes
// * @implemented NT5.1
// *
// *     The ObSetHandleAttributes routine <FILLMEIN>
// *
// * @param Handle
// *        <FILLMEIN>.
// *
// * @param HandleFlags
// *        <FILLMEIN>.
// *
// * @param PreviousMode
// *        <FILLMEIN>.
// *
// * @return <FILLMEIN>.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// ObSetHandleAttributes(IN HANDLE Handle,
//                       IN POBJECT_HANDLE_ATTRIBUTE_INFORMATION HandleFlags,
//                       IN KPROCESSOR_MODE PreviousMode)
// {
//     OBP_SET_HANDLE_ATTRIBUTES_CONTEXT SetHandleAttributesContext;
//     BOOLEAN Result, AttachedToSystemProcess = FALSE;
//     PHANDLE_TABLE HandleTable;
//     KAPC_STATE ApcState;
//     PAGED_CODE();
//
//     /* Check if this is a kernel handle */
//     if (ObpIsKernelHandle(Handle, PreviousMode))
//     {
//         /* Use the kernel table and convert the handle */
//         HandleTable = ObpKernelHandleTable;
//         Handle = ObKernelHandleToHandle(Handle);
//
//         /* Check if we're not in the system process */
//         if (PsGetCurrentProcess() != PsInitialSystemProcess)
//         {
//             /* Attach to the system process */
//             KeStackAttachProcess(&PsInitialSystemProcess->Pcb, &ApcState);
//             AttachedToSystemProcess = TRUE;
//         }
//     }
//     else
//     {
//         /* Get the current process' handle table */
//         HandleTable = (PsGetCurrentProcess() as *mut _EPROCESS)->ObjectTable;
//     }
//
//     /* Initialize the handle attribute context */
//     SetHandleAttributesContext.PreviousMode = PreviousMode;
//     SetHandleAttributesContext.Information = *HandleFlags;
//
//     /* Invoke the ObpSetHandleAttributes callback */
//     Result = ExChangeHandle(HandleTable,
//                             Handle,
//                             ObpSetHandleAttributes,
//                             (ULONG_PTR)&SetHandleAttributesContext);
//
//     /* Did we attach to the system process? */
//     if (AttachedToSystemProcess)
//     {
//         /* Detach from it */
//         KeUnstackDetachProcess(&ApcState);
//     }
//
//     /* Return the result as an NTSTATUS value */
//     return Result ? STATUS_SUCCESS : STATUS_ACCESS_DENIED;
// }
//
// /*++
// * @name ObCloseHandle
// * @implemented NT5.1
// *
// *     The ObCloseHandle routine <FILLMEIN>
// *
// * @param Handle
// *        <FILLMEIN>.
// *
// * @param AccessMode
// *        <FILLMEIN>.
// *
// * @return <FILLMEIN>.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// ObCloseHandle(IN HANDLE Handle,
//               IN KPROCESSOR_MODE AccessMode)
// {
//     /* Call the internal API */
//     return ObpCloseHandle(Handle, AccessMode);
// }
//
// /*++
// * @name NtClose
// * @implemented NT4
// *
// *     The NtClose routine <FILLMEIN>
// *
// * @param Handle
// *        <FILLMEIN>.
// *
// * @return <FILLMEIN>.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// NtClose(IN HANDLE Handle)
// {
//     /* Call the internal API */
//     return ObpCloseHandle(Handle, ExGetPreviousMode());
// }
//
// NTSTATUS
// NTAPI
// NtDuplicateObject(IN HANDLE SourceProcessHandle,
//                   IN HANDLE SourceHandle,
//                   IN HANDLE TargetProcessHandle OPTIONAL,
//                   OUT PHANDLE TargetHandle OPTIONAL,
//                   IN ACCESS_MASK DesiredAccess,
//                   IN ULONG HandleAttributes,
//                   IN ULONG Options)
// {
//     PEPROCESS SourceProcess, TargetProcess, Target;
//     HANDLE hTarget;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - Duplicating handle: %p for %p into %p.\n",
//             __FUNCTION__,
//             SourceHandle,
//             SourceProcessHandle,
//             TargetProcessHandle);
//
//     /* Check if we have a target handle */
//     if ((TargetHandle) && (PreviousMode != KernelMode))
//     {
//         /* Enter SEH */
//         _SEH2_TRY
//         {
//             /* Probe the handle and assume failure */
//             ProbeForWriteHandle(TargetHandle);
//             *TargetHandle = NULL;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Now reference the input handle */
//     Status = ObReferenceObjectByHandle(SourceProcessHandle,
//                                        PROCESS_DUP_HANDLE,
//                                        PsProcessType,
//                                        PreviousMode,
//                                        (PVOID*)&SourceProcess,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Check if got a target handle */
//     if (TargetProcessHandle)
//     {
//         /* Now reference the output handle */
//         Status = ObReferenceObjectByHandle(TargetProcessHandle,
//                                            PROCESS_DUP_HANDLE,
//                                            PsProcessType,
//                                            PreviousMode,
//                                            (PVOID*)&TargetProcess,
//                                            NULL);
//         if (NT_SUCCESS(Status))
//         {
//             /* Use this target process */
//             Target = TargetProcess;
//         }
//         else
//         {
//             /* No target process */
//             Target = NULL;
//         }
//     }
//     else
//     {
//         /* No target process */
//         Status = STATUS_SUCCESS;
//         Target = NULL;
//     }
//
//     /* Call the internal routine */
//     Status = ObDuplicateObject(SourceProcess,
//                                SourceHandle,
//                                Target,
//                                &hTarget,
//                                DesiredAccess,
//                                HandleAttributes,
//                                Options,
//                                PreviousMode);
//
//     /* Check if the caller wanted the return handle */
//     if (TargetHandle)
//     {
//         /* Protect the write to user mode */
//         _SEH2_TRY
//         {
//             /* Write the new handle */
//             *TargetHandle = hTarget;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Otherwise, get the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//
//     /* Dereference the processes */
//     OBTRACE(OB_HANDLE_DEBUG,
//             "%s - Duplicated handle: %p into %p S %lx\n",
//             __FUNCTION__,
//             hTarget,
//             TargetProcessHandle,
//             Status);
//     if (Target) ObDereferenceObject(Target);
//     ObDereferenceObject(SourceProcess);
//     return Status;
// }
//
// BOOLEAN
// NTAPI
// ObIsKernelHandle(IN HANDLE Handle)
// {
//     /* Use the inlined version. We know we are in kernel mode. */
//     return ObpIsKernelHandle(Handle, KernelMode);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
