use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::ex::*;

use super::oblife::ObpReapObject;

use ntkrnl_sys::ddk::{
    _OBJECT_TYPE, _OBJECT_HEADER_CREATOR_INFO,
    _OBJECT_DIRECTORY, _OBJECT_SYMBOLIC_LINK, _OBJECT_CREATE_INFORMATION,
    _OBJECT_TYPE_INITIALIZER, _OBP_LOOKUP_CONTEXT, _SECURITY_DESCRIPTOR,
    _ACL, _ACE, _GENERAL_LOOKASIDE, _KPRCB, _EPROCESS, _UNICODE_STRING,
    POBJECT_TYPE, POBJECT_HEADER, POBJECT_HEADER_CREATOR_INFO, POBJECT_HEADER_NAME_INFO,
    PSECURITY_DESCRIPTOR, PACL,
    GENERIC_MAPPING, INT_PTR, PUNICODE_STRING,
    ExCreateHandleTable, ExInitializeSystemLookasideList,
    NtCreateDirectoryObject, NtClose, ObReferenceObjectByHandle,
    ObpDeleteObjectType, ObpDeleteSymbolicLink,
    ObpParseSymbolicLink, ObpInitSdCache, ObpInitializeLookupContext,
    ObpAcquireLookupContextLock, ObpReleaseLookupContext, ObpLookupEntryDirectory,
    ObpInsertEntryDirectory, ObpCreateDosDevicesDirectory,
    RtlCreateSecurityDescriptor, RtlCreateAcl, RtlAddAccessAllowedAce,
    RtlSetDaclSecurityDescriptor, RtlInitUnicodeString, RtlLengthSid,
    PsGetCurrentThread, HANDLE, _OBJECT_ATTRIBUTES,
    PGENERAL_LOOKASIDE, POBP_LOOKUP_CONTEXT, POBJECT_DIRECTORY,
    KeInitializeEvent, ObCreateObjectType,
};

// Access rights constants not in ntkrnl-sys constants.rs
const PROCESS_ALL_ACCESS: ULONG = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0xFFFF;
const THREAD_ALL_ACCESS: ULONG = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0xFFFF;
const OBJECT_TYPE_ALL_ACCESS: ULONG = STANDARD_RIGHTS_REQUIRED | 0x1;
const OBJ_OPENLINK: ULONG = 0x00000100;

// Use actual definitions, not extern declarations (lto=false splits them)
use crate::ke::processor::KeNumberProcessors;
use crate::ke::krnlinit::KiProcessorBlock;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}

extern "C" {
    static mut SePublicDefaultUnrestrictedSd: PVOID; // PSECURITY_DESCRIPTOR
    static mut ObpTypeObjectType: POBJECT_TYPE;
    static mut ObpDefaultObject: ntkrnl_sys::ddk::KEVENT;
}

// PsInitializeQuotaSystem — declared but not exported by bindgen (forward decl in C)
extern "system" {
    fn PsInitializeQuotaSystem();
}

/// ExInitializeWorkItem — C macro, inline implementation.
#[inline]
unsafe fn ExInitializeWorkItem(
    item: *mut ntkrnl_sys::ddk::WORK_QUEUE_ITEM,
    routine: ntkrnl_sys::ddk::PWORKER_THREAD_ROUTINE,
    context: PVOID,
) {
    (*item).WorkerRoutine = routine;
    (*item).Parameter = context;
    (*item).List.Flink = core::ptr::null_mut();
}

#[no_mangle]
pub static mut ObpTypeMapping: GENERIC_MAPPING = GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ,
    GenericWrite: STANDARD_RIGHTS_WRITE,
    GenericExecute: STANDARD_RIGHTS_EXECUTE,
    GenericAll: 0x000F0001,
};

#[no_mangle]
pub static mut ObpDirectoryMapping: GENERIC_MAPPING = GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ | DIRECTORY_QUERY | DIRECTORY_TRAVERSE,
    GenericWrite: STANDARD_RIGHTS_WRITE | DIRECTORY_CREATE_SUBDIRECTORY | DIRECTORY_CREATE_OBJECT,
    GenericExecute: STANDARD_RIGHTS_EXECUTE | DIRECTORY_QUERY | DIRECTORY_TRAVERSE,
    GenericAll: DIRECTORY_ALL_ACCESS,
};

#[no_mangle]
pub static mut ObpSymbolicLinkMapping: GENERIC_MAPPING = GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ | SYMBOLIC_LINK_QUERY,
    GenericWrite: STANDARD_RIGHTS_WRITE,
    GenericExecute: STANDARD_RIGHTS_EXECUTE | SYMBOLIC_LINK_QUERY,
    GenericAll: SYMBOLIC_LINK_ALL_ACCESS,
};

#[no_mangle]
pub static mut ObpTraceLevel: ULONG = 0;

#[no_mangle]
pub static mut ObpInitializationPhase: ULONG = 0;

#[no_mangle]
pub static mut ObpObjectSecurityMode: ULONG = 0;

#[no_mangle]
pub static mut ObpProtectionMode: ULONG = 0;

// Registry-controlled values owned by ob/obinit. Concatenated into
// CmControlVector by config::cmdata. See docs/engineering/cmdata-chopping-plan.md.
pub static OBINIT_CONTROL_VECTOR: ntkrnl_sys::SubVector<2> =
    ntkrnl_sys::SubVector(unsafe { [
        ntkrnl_sys::cv!("Session Manager", "ProtectionMode",
            core::ptr::addr_of_mut!(ObpProtectionMode)),
        ntkrnl_sys::cv!("Session Manager", "ObjectSecurityMode",
            core::ptr::addr_of_mut!(ObpObjectSecurityMode)),
    ] });

unsafe fn ObpCreateKernelObjectsSD(
    security_descriptor: *mut PSECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let mut sd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut dacl: PACL;
    let acl_size: ULONG;
    let sd_size: ULONG;
    let mut status: NTSTATUS;

    acl_size = core::mem::size_of::<_ACL>() as ULONG +
               core::mem::size_of::<_ACE>() as ULONG + RtlLengthSid(SeWorldSid) +
               core::mem::size_of::<_ACE>() as ULONG + RtlLengthSid(SeAliasAdminsSid) +
               core::mem::size_of::<_ACE>() as ULONG + RtlLengthSid(SeLocalSystemSid);

    sd_size = core::mem::size_of::<_SECURITY_DESCRIPTOR>() as ULONG + acl_size;

    sd = ExAllocatePoolWithTag(PagedPool, sd_size as usize, TAG_SD) as PSECURITY_DESCRIPTOR;
    if sd.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    status = RtlCreateSecurityDescriptor(sd, SECURITY_DESCRIPTOR_REVISION);
    if status < 0 {
        ExFreePoolWithTag(sd as PVOID, TAG_SD);
        return status;
    }

    dacl = (sd as INT_PTR + core::mem::size_of::<_SECURITY_DESCRIPTOR>() as INT_PTR) as PACL;

    RtlCreateAcl(dacl, acl_size, ACL_REVISION);

    RtlAddAccessAllowedAce(dacl, ACL_REVISION, GENERIC_READ, SeWorldSid);
    RtlAddAccessAllowedAce(dacl, ACL_REVISION, GENERIC_ALL, SeAliasAdminsSid);
    RtlAddAccessAllowedAce(dacl, ACL_REVISION, GENERIC_ALL, SeLocalSystemSid);

    status = RtlSetDaclSecurityDescriptor(sd, 1u8, dacl, 0u8);
    if status < 0 {
        ExFreePoolWithTag(sd as PVOID, TAG_SD);
        return status;
    }

    *security_descriptor = sd;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ObInit2() -> BOOLEAN {
    let mut prcb: *mut _KPRCB;
    let mut current_list: PGENERAL_LOOKASIDE;

    for i in 0..KeNumberProcessors as i8 {
        prcb = *(core::ptr::addr_of!(KiProcessorBlock) as *const *mut _KPRCB).add(i as usize);

        (*prcb).PPLookasideList[LookasideCreateInfoList as usize].L =
            core::ptr::addr_of_mut!(ObpCreateInfoLookasideList) as PGENERAL_LOOKASIDE;

        current_list = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<_GENERAL_LOOKASIDE>(),
            u32::from_be_bytes(*b"ICbO"),
        ) as PGENERAL_LOOKASIDE;

        if !current_list.is_null() {
            ExInitializeSystemLookasideList(
                current_list,
                NonPagedPool,
                core::mem::size_of::<_OBJECT_CREATE_INFORMATION>() as ULONG,
                u32::from_be_bytes(*b"ICbO"),
                32,
                core::ptr::addr_of_mut!(ExSystemLookasideListHead) as *mut ntkrnl_sys::ddk::_LIST_ENTRY,
            );
        } else {
            current_list = core::ptr::addr_of_mut!(ObpCreateInfoLookasideList) as PGENERAL_LOOKASIDE;
        }

        (*prcb).PPLookasideList[LookasideCreateInfoList as usize].P = current_list;

        (*prcb).PPLookasideList[LookasideNameBufferList as usize].L =
            core::ptr::addr_of_mut!(ObpNameBufferLookasideList) as PGENERAL_LOOKASIDE;

        current_list = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<_GENERAL_LOOKASIDE>(),
            u32::from_be_bytes(*b"MNbO"),
        ) as PGENERAL_LOOKASIDE;

        if !current_list.is_null() {
            ExInitializeSystemLookasideList(
                current_list,
                PagedPool,
                248,
                u32::from_be_bytes(*b"MNbO"),
                16,
                core::ptr::addr_of_mut!(ExSystemLookasideListHead) as *mut ntkrnl_sys::ddk::_LIST_ENTRY,
            );
        } else {
            current_list = core::ptr::addr_of_mut!(ObpNameBufferLookasideList) as PGENERAL_LOOKASIDE;
        }

        (*prcb).PPLookasideList[LookasideNameBufferList as usize].P = current_list;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn ObInitSystem() -> BOOLEAN {
    let mut object_attributes: _OBJECT_ATTRIBUTES;
    let mut name: _UNICODE_STRING = core::mem::zeroed();
    let mut object_type_initializer: _OBJECT_TYPE_INITIALIZER = core::mem::zeroed();
    let mut context: _OBP_LOOKUP_CONTEXT = core::mem::zeroed();
    let mut handle: HANDLE = core::ptr::null_mut();
    let prcb: *mut _KPRCB = KeGetCurrentPrcb() as *mut _KPRCB;
    let mut list_head: *mut ntkrnl_sys::ddk::_LIST_ENTRY;
    let mut next_entry: *mut ntkrnl_sys::ddk::_LIST_ENTRY;
    let mut header: POBJECT_HEADER;
    let mut creator_info: POBJECT_HEADER_CREATOR_INFO;
    let mut name_info: POBJECT_HEADER_NAME_INFO;
    let mut kernel_objects_sd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut status: NTSTATUS;

    if ObpInitializationPhase != 0 {
        ObInit2();

        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(name),
            w!("\\").as_ptr() as *const u16,
        );

        object_attributes = _OBJECT_ATTRIBUTES {
            Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
            RootDirectory: core::ptr::null_mut(),
            ObjectName: core::ptr::addr_of_mut!(name),
            Attributes: OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
            SecurityDescriptor: SePublicDefaultUnrestrictedSd,
            SecurityQualityOfService: core::ptr::null_mut(),
        };

        status = NtCreateDirectoryObject(
            core::ptr::addr_of_mut!(handle),
            DIRECTORY_ALL_ACCESS,
            core::ptr::addr_of_mut!(object_attributes),
        );
        if status < 0 {
            return 0u8;
        }

        status = ObReferenceObjectByHandle(
            handle,
            0,
            ObpDirectoryObjectType as POBJECT_TYPE,
            0i8,
            core::ptr::addr_of_mut!(ObpRootDirectoryObject) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if status < 0 {
            return 0u8;
        }

        status = NtClose(handle);
        if status < 0 {
            return 0u8;
        }

        status = ObpCreateKernelObjectsSD(core::ptr::addr_of_mut!(kernel_objects_sd));
        if status < 0 {
            return 0u8;
        }

        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(name),
            w!("\\KernelObjects").as_ptr() as *const u16,
        );

        object_attributes = _OBJECT_ATTRIBUTES {
            Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
            RootDirectory: core::ptr::null_mut(),
            ObjectName: core::ptr::addr_of_mut!(name),
            Attributes: OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
            SecurityDescriptor: kernel_objects_sd as PVOID,
            SecurityQualityOfService: core::ptr::null_mut(),
        };

        status = NtCreateDirectoryObject(
            core::ptr::addr_of_mut!(handle),
            DIRECTORY_ALL_ACCESS,
            core::ptr::addr_of_mut!(object_attributes),
        );
        ExFreePoolWithTag(kernel_objects_sd as PVOID, TAG_SD);
        if status < 0 {
            return 0u8;
        }

        status = NtClose(handle);
        if status < 0 {
            return 0u8;
        }

        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(name),
            w!("\\ObjectTypes").as_ptr() as *const u16,
        );

        object_attributes = _OBJECT_ATTRIBUTES {
            Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
            RootDirectory: core::ptr::null_mut(),
            ObjectName: core::ptr::addr_of_mut!(name),
            Attributes: OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
            SecurityDescriptor: core::ptr::null_mut(),
            SecurityQualityOfService: core::ptr::null_mut(),
        };

        status = NtCreateDirectoryObject(
            core::ptr::addr_of_mut!(handle),
            DIRECTORY_ALL_ACCESS,
            core::ptr::addr_of_mut!(object_attributes),
        );
        if status < 0 {
            return 0u8;
        }

        status = ObReferenceObjectByHandle(
            handle,
            0,
            ObpDirectoryObjectType as POBJECT_TYPE,
            0i8,
            core::ptr::addr_of_mut!(ObpTypeDirectoryObject) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if status < 0 {
            return 0u8;
        }

        status = NtClose(handle);
        if status < 0 {
            return 0u8;
        }

        ObpInitializeLookupContext(core::ptr::addr_of_mut!(context) as POBP_LOOKUP_CONTEXT);
        ObpAcquireLookupContextLock(core::ptr::addr_of_mut!(context) as POBP_LOOKUP_CONTEXT, ObpTypeDirectoryObject as POBJECT_DIRECTORY);

        list_head = core::ptr::addr_of_mut!((*ObpTypeObjectType).TypeList) as *mut ntkrnl_sys::ddk::_LIST_ENTRY;
        next_entry = (*list_head).Flink;

        while list_head != next_entry {
            creator_info = ((next_entry as usize) -
                core::mem::offset_of!(_OBJECT_HEADER_CREATOR_INFO, TypeList)) as POBJECT_HEADER_CREATOR_INFO;

            header = (creator_info as usize + core::mem::size_of::<_OBJECT_HEADER_CREATOR_INFO>()) as POBJECT_HEADER;
            name_info = OBJECT_HEADER_TO_NAME_INFO(header);

            if !name_info.is_null() && (*name_info).Directory.is_null() {
                if ObpLookupEntryDirectory(
                    ObpTypeDirectoryObject as POBJECT_DIRECTORY,
                    core::ptr::addr_of_mut!((*name_info).Name) as PUNICODE_STRING,
                    OBJ_CASE_INSENSITIVE,
                    0u8,
                    core::ptr::addr_of_mut!(context) as POBP_LOOKUP_CONTEXT,
                ).is_null()
                {
                    ObpInsertEntryDirectory(
                        ObpTypeDirectoryObject as POBJECT_DIRECTORY,
                        core::ptr::addr_of_mut!(context) as POBP_LOOKUP_CONTEXT,
                        header,
                    );
                }
            }

            next_entry = (*next_entry).Flink;
        }

        ObpReleaseLookupContext(core::ptr::addr_of_mut!(context) as POBP_LOOKUP_CONTEXT);

        status = ObpCreateDosDevicesDirectory();
        if status < 0 {
            return 0u8;
        }

        return 1u8;
    }

    // DBG: ObInitSystem Phase 0 - before ExInitializeSystemLookasideList (NonPaged)

    ExInitializeSystemLookasideList(
        core::ptr::addr_of_mut!(ObpCreateInfoLookasideList) as PGENERAL_LOOKASIDE,
        NonPagedPool,
        core::mem::size_of::<_OBJECT_CREATE_INFORMATION>() as ULONG,
        u32::from_be_bytes(*b"ICbO"),
        32,
        core::ptr::addr_of_mut!(ExSystemLookasideListHead) as *mut ntkrnl_sys::ddk::_LIST_ENTRY,
    );

    // DBG: before ExInitializeSystemLookasideList (Paged)

    ExInitializeSystemLookasideList(
        core::ptr::addr_of_mut!(ObpNameBufferLookasideList) as PGENERAL_LOOKASIDE,
        PagedPool,
        248,
        u32::from_be_bytes(*b"MNbO"),
        16,
        core::ptr::addr_of_mut!(ExSystemLookasideListHead) as *mut ntkrnl_sys::ddk::_LIST_ENTRY,
    );

    // DBG: before PPLookasideList setup

    (*prcb).PPLookasideList[LookasideCreateInfoList as usize].L =
        core::ptr::addr_of_mut!(ObpCreateInfoLookasideList) as PGENERAL_LOOKASIDE;
    (*prcb).PPLookasideList[LookasideCreateInfoList as usize].P =
        core::ptr::addr_of_mut!(ObpCreateInfoLookasideList) as PGENERAL_LOOKASIDE;
    (*prcb).PPLookasideList[LookasideNameBufferList as usize].L =
        core::ptr::addr_of_mut!(ObpNameBufferLookasideList) as PGENERAL_LOOKASIDE;
    (*prcb).PPLookasideList[LookasideNameBufferList as usize].P =
        core::ptr::addr_of_mut!(ObpNameBufferLookasideList) as PGENERAL_LOOKASIDE;

    // DBG: before ObpInitSdCache

    ObpInitSdCache();

    // DBG: before KeInitializeEvent

    KeInitializeEvent(
        core::ptr::addr_of_mut!(ObpDefaultObject) as ntkrnl_sys::ddk::PRKEVENT,
        ntkrnl_sys::ddk::_EVENT_TYPE_NotificationEvent,
        1u8,
    );

    KeInitializeGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

    (*(PsGetCurrentProcess() as *mut _EPROCESS)).GrantedAccess = PROCESS_ALL_ACCESS;
    (*PsGetCurrentThread()).GrantedAccess = THREAD_ALL_ACCESS;

    ExInitializeWorkItem(
        core::ptr::addr_of_mut!(ObpReaperWorkItem),
        core::mem::transmute(ObpReapObject as unsafe extern "system" fn(PVOID)),
        core::ptr::null_mut(),
    );

    // DBG: before PsInitializeQuotaSystem

    PsInitializeQuotaSystem();

    // DBG: before ExCreateHandleTable

    (*(PsGetCurrentProcess() as *mut _EPROCESS)).ObjectTable = ExCreateHandleTable(core::ptr::null_mut());
    ObpKernelHandleTable = (*(PsGetCurrentProcess() as *mut _EPROCESS)).ObjectTable;

    // DBG: before ObCreateObjectType("Type")

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(object_type_initializer) as *mut u8,
        0,
        core::mem::size_of::<_OBJECT_TYPE_INITIALIZER>(),
    );
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(name),
        w!("Type").as_ptr() as *const u16,
    );
    object_type_initializer.Length = core::mem::size_of::<_OBJECT_TYPE_INITIALIZER>() as USHORT;
    object_type_initializer.ValidAccessMask = OBJECT_TYPE_ALL_ACCESS;
    object_type_initializer.UseDefaultObject = 1u8;
    object_type_initializer.MaintainTypeList = 1u8;
    object_type_initializer.PoolType = NonPagedPool;
    object_type_initializer.GenericMapping = ObpTypeMapping;
    object_type_initializer.DefaultNonPagedPoolCharge = core::mem::size_of::<_OBJECT_TYPE>() as ULONG;
    object_type_initializer.InvalidAttributes = OBJ_OPENLINK;
    object_type_initializer.DeleteProcedure = Some(ObpDeleteObjectType);
    ObCreateObjectType(
        core::ptr::addr_of_mut!(name),
        core::ptr::addr_of_mut!(object_type_initializer),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ObpTypeObjectType) as *mut POBJECT_TYPE,
    );

    // DBG: before ObCreateObjectType("Directory") - PagedPool

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(name),
        w!("Directory").as_ptr() as *const u16,
    );
    object_type_initializer.PoolType = PagedPool;
    object_type_initializer.ValidAccessMask = DIRECTORY_ALL_ACCESS;
    object_type_initializer.CaseInsensitive = 1u8;
    object_type_initializer.MaintainTypeList = 0u8;
    object_type_initializer.GenericMapping = ObpDirectoryMapping;
    object_type_initializer.DeleteProcedure = None;
    object_type_initializer.DefaultNonPagedPoolCharge = core::mem::size_of::<_OBJECT_DIRECTORY>() as ULONG;
    ObCreateObjectType(
        core::ptr::addr_of_mut!(name),
        core::ptr::addr_of_mut!(object_type_initializer),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ObpDirectoryObjectType) as *mut POBJECT_TYPE,
    );

    // DBG: before ObCreateObjectType("SymbolicLink")

    (*(ObpDirectoryObjectType as POBJECT_TYPE)).TypeInfo.ValidAccessMask &= !SYNCHRONIZE;

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(name),
        w!("SymbolicLink").as_ptr() as *const u16,
    );
    object_type_initializer.DefaultNonPagedPoolCharge = core::mem::size_of::<_OBJECT_SYMBOLIC_LINK>() as ULONG;
    object_type_initializer.GenericMapping = ObpSymbolicLinkMapping;
    object_type_initializer.ValidAccessMask = SYMBOLIC_LINK_ALL_ACCESS;
    object_type_initializer.ParseProcedure = Some(ObpParseSymbolicLink);
    object_type_initializer.DeleteProcedure = Some(ObpDeleteSymbolicLink);
    ObCreateObjectType(
        core::ptr::addr_of_mut!(name),
        core::ptr::addr_of_mut!(object_type_initializer),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ObpSymbolicLinkObjectType) as *mut POBJECT_TYPE,
    );

    // DBG: ObInitSystem Phase 0 COMPLETE

    (*(ObpSymbolicLinkObjectType as POBJECT_TYPE)).TypeInfo.ValidAccessMask &= !SYNCHRONIZE;

    ObpInitializationPhase += 1;
    1u8
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ob/obinit.c
//  * PURPOSE:         Handles Object Manager Initialization and Shutdown
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
// /* GLOBALS *******************************************************************/
//
// GENERIC_MAPPING ObpTypeMapping =
// {
//     STANDARD_RIGHTS_READ,
//     STANDARD_RIGHTS_WRITE,
//     STANDARD_RIGHTS_EXECUTE,
//     0x000F0001
// };
//
// GENERIC_MAPPING ObpDirectoryMapping =
// {
//     STANDARD_RIGHTS_READ    | DIRECTORY_QUERY               |
//     DIRECTORY_TRAVERSE,
//     STANDARD_RIGHTS_WRITE   | DIRECTORY_CREATE_SUBDIRECTORY |
//     DIRECTORY_CREATE_OBJECT,
//     STANDARD_RIGHTS_EXECUTE | DIRECTORY_QUERY               |
//     DIRECTORY_TRAVERSE,
//     DIRECTORY_ALL_ACCESS
// };
//
// GENERIC_MAPPING ObpSymbolicLinkMapping =
// {
//     STANDARD_RIGHTS_READ    | SYMBOLIC_LINK_QUERY,
//     STANDARD_RIGHTS_WRITE,
//     STANDARD_RIGHTS_EXECUTE | SYMBOLIC_LINK_QUERY,
//     SYMBOLIC_LINK_ALL_ACCESS
// };
//
// PDEVICE_MAP ObSystemDeviceMap = NULL;
// ULONG ObpTraceLevel = 0;
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// PsInitializeQuotaSystem(VOID);
//
// ULONG ObpInitializationPhase;
//
// ULONG ObpObjectSecurityMode = 0;
// ULONG ObpProtectionMode = 0;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// static
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// ObpCreateKernelObjectsSD(OUT PSECURITY_DESCRIPTOR *SecurityDescriptor)
// {
//     PSECURITY_DESCRIPTOR Sd = NULL;
//     PACL Dacl;
//     ULONG AclSize, SdSize;
//     NTSTATUS Status;
//
//     AclSize = sizeof(ACL) +
//               sizeof(ACE) + RtlLengthSid(SeWorldSid) +
//               sizeof(ACE) + RtlLengthSid(SeAliasAdminsSid) +
//               sizeof(ACE) + RtlLengthSid(SeLocalSystemSid);
//
//     SdSize = sizeof(SECURITY_DESCRIPTOR) + AclSize;
//
//     /* Allocate the SD and ACL */
//     Sd = ExAllocatePoolWithTag(PagedPool, SdSize, TAG_SD);
//     if (Sd == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Initialize the SD */
//     Status = RtlCreateSecurityDescriptor(Sd,
//                                          SECURITY_DESCRIPTOR_REVISION);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Dacl = (PACL)((INT_PTR)Sd + sizeof(SECURITY_DESCRIPTOR));
//
//     /* Initialize the DACL */
//     RtlCreateAcl(Dacl, AclSize, ACL_REVISION);
//
//     /* Add the ACEs */
//     RtlAddAccessAllowedAce(Dacl,
//                            ACL_REVISION,
//                            GENERIC_READ,
//                            SeWorldSid);
//
//     RtlAddAccessAllowedAce(Dacl,
//                            ACL_REVISION,
//                            GENERIC_ALL,
//                            SeAliasAdminsSid);
//
//     RtlAddAccessAllowedAce(Dacl,
//                            ACL_REVISION,
//                            GENERIC_ALL,
//                            SeLocalSystemSid);
//
//     /* Attach the DACL to the SD */
//     Status = RtlSetDaclSecurityDescriptor(Sd,
//                                           TRUE,
//                                           Dacl,
//                                           FALSE);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     *SecurityDescriptor = Sd;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (Sd != NULL)
//             ExFreePoolWithTag(Sd, TAG_SD);
//     }
//
//     return Status;
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// ObInit2(VOID)
// {
//     CCHAR i;
//     PKPRCB Prcb;
//     PGENERAL_LOOKASIDE CurrentList = NULL;
//
//     /* Now allocate the per-processor lists */
//     for (i = 0; i < KeNumberProcessors; i++)
//     {
//         /* Get the PRCB for this CPU */
//         Prcb = KiProcessorBlock[(int)i];
//
//         /* Set the OBJECT_CREATE_INFORMATION List */
//         Prcb->PPLookasideList[LookasideCreateInfoList].L = &ObpCreateInfoLookasideList;
//         CurrentList = ExAllocatePoolWithTag(NonPagedPool,
//                                             sizeof(GENERAL_LOOKASIDE),
//                                             'ICbO');
//         if (CurrentList)
//         {
//             /* Initialize it */
//             ExInitializeSystemLookasideList(CurrentList,
//                                             NonPagedPool,
//                                             sizeof(OBJECT_CREATE_INFORMATION),
//                                             'ICbO',
//                                             32,
//                                             &ExSystemLookasideListHead);
//         }
//         else
//         {
//             /* No list, use the static buffer */
//             CurrentList = &ObpCreateInfoLookasideList;
//         }
//
//         /* Link it */
//         Prcb->PPLookasideList[LookasideCreateInfoList].P = CurrentList;
//
//         /* Set the captured UNICODE_STRING Object Name List */
//         Prcb->PPLookasideList[LookasideNameBufferList].L = &ObpNameBufferLookasideList;
//         CurrentList = ExAllocatePoolWithTag(NonPagedPool,
//                                             sizeof(GENERAL_LOOKASIDE),
//                                             'MNbO');
//         if (CurrentList)
//         {
//             /* Initialize it */
//             ExInitializeSystemLookasideList(CurrentList,
//                                             PagedPool,
//                                             248,
//                                             'MNbO',
//                                             16,
//                                             &ExSystemLookasideListHead);
//         }
//         else
//         {
//             /* No list, use the static buffer */
//             CurrentList = &ObpNameBufferLookasideList;
//         }
//
//         /* Link it */
//         Prcb->PPLookasideList[LookasideNameBufferList].P = CurrentList;
//     }
//
//     return TRUE;
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// ObInitSystem(VOID)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING Name;
//     OBJECT_TYPE_INITIALIZER ObjectTypeInitializer;
//     OBP_LOOKUP_CONTEXT Context;
//     HANDLE Handle;
//     PKPRCB Prcb = KeGetCurrentPrcb();
//     PLIST_ENTRY ListHead, NextEntry;
//     POBJECT_HEADER Header;
//     POBJECT_HEADER_CREATOR_INFO CreatorInfo;
//     POBJECT_HEADER_NAME_INFO NameInfo;
//     PSECURITY_DESCRIPTOR KernelObjectsSD = NULL;
//     NTSTATUS Status;
//
//     /* Check if this is actually Phase 1 initialization */
//     if (ObpInitializationPhase != 0) goto ObPostPhase0;
//
//     /* Initialize the OBJECT_CREATE_INFORMATION List */
//     ExInitializeSystemLookasideList(&ObpCreateInfoLookasideList,
//                                     NonPagedPool,
//                                     sizeof(OBJECT_CREATE_INFORMATION),
//                                     'ICbO',
//                                     32,
//                                     &ExSystemLookasideListHead);
//
//     /* Set the captured UNICODE_STRING Object Name List */
//     ExInitializeSystemLookasideList(&ObpNameBufferLookasideList,
//                                     PagedPool,
//                                     248,
//                                     'MNbO',
//                                     16,
//                                     &ExSystemLookasideListHead);
//
//     /* Temporarily setup both pointers to the shared list */
//     Prcb->PPLookasideList[LookasideCreateInfoList].L = &ObpCreateInfoLookasideList;
//     Prcb->PPLookasideList[LookasideCreateInfoList].P = &ObpCreateInfoLookasideList;
//     Prcb->PPLookasideList[LookasideNameBufferList].L = &ObpNameBufferLookasideList;
//     Prcb->PPLookasideList[LookasideNameBufferList].P = &ObpNameBufferLookasideList;
//
//     /* Initialize the security descriptor cache */
//     ObpInitSdCache();
//
//     /* Initialize the Default Event */
//     KeInitializeEvent(&ObpDefaultObject, NotificationEvent, TRUE);
//
//     /* Initialize the Dos Device Map mutex */
//     KeInitializeGuardedMutex(&ObpDeviceMapLock);
//
//     /* Setup default access for the system process */
//     PsGetCurrentProcess()->GrantedAccess = PROCESS_ALL_ACCESS;
//     PsGetCurrentThread()->GrantedAccess = THREAD_ALL_ACCESS;
//
//     /* Setup the Object Reaper */
//     ExInitializeWorkItem(&ObpReaperWorkItem, ObpReapObject, NULL);
//
//     /* Initialize default Quota block */
//     PsInitializeQuotaSystem();
//
//     /* Create kernel handle table */
//     PsGetCurrentProcess()->ObjectTable = ExCreateHandleTable(NULL);
//     ObpKernelHandleTable = PsGetCurrentProcess()->ObjectTable;
//
//     /* Create the Type Type */
//     RtlZeroMemory(&ObjectTypeInitializer, sizeof(ObjectTypeInitializer));
//     RtlInitUnicodeString(&Name, L"Type");
//     ObjectTypeInitializer.Length = sizeof(ObjectTypeInitializer);
//     ObjectTypeInitializer.ValidAccessMask = OBJECT_TYPE_ALL_ACCESS;
//     ObjectTypeInitializer.UseDefaultObject = TRUE;
//     ObjectTypeInitializer.MaintainTypeList = TRUE;
//     ObjectTypeInitializer.PoolType = NonPagedPool;
//     ObjectTypeInitializer.GenericMapping = ObpTypeMapping;
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(OBJECT_TYPE);
//     ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
//     ObjectTypeInitializer.DeleteProcedure = ObpDeleteObjectType;
//     ObCreateObjectType(&Name, &ObjectTypeInitializer, NULL, &ObpTypeObjectType);
//
//     /* Create the Directory Type */
//     RtlInitUnicodeString(&Name, L"Directory");
//     ObjectTypeInitializer.PoolType = PagedPool;
//     ObjectTypeInitializer.ValidAccessMask = DIRECTORY_ALL_ACCESS;
//     ObjectTypeInitializer.CaseInsensitive = TRUE;
//     ObjectTypeInitializer.MaintainTypeList = FALSE;
//     ObjectTypeInitializer.GenericMapping = ObpDirectoryMapping;
//     ObjectTypeInitializer.DeleteProcedure = NULL;
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(OBJECT_DIRECTORY);
//     ObCreateObjectType(&Name, &ObjectTypeInitializer, NULL, &ObpDirectoryObjectType);
//     ObpDirectoryObjectType->TypeInfo.ValidAccessMask &= ~SYNCHRONIZE;
//
//     /* Create 'symbolic link' object type */
//     RtlInitUnicodeString(&Name, L"SymbolicLink");
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(OBJECT_SYMBOLIC_LINK);
//     ObjectTypeInitializer.GenericMapping = ObpSymbolicLinkMapping;
//     ObjectTypeInitializer.ValidAccessMask = SYMBOLIC_LINK_ALL_ACCESS;
//     ObjectTypeInitializer.ParseProcedure = ObpParseSymbolicLink;
//     ObjectTypeInitializer.DeleteProcedure = ObpDeleteSymbolicLink;
//     ObCreateObjectType(&Name, &ObjectTypeInitializer, NULL, &ObpSymbolicLinkObjectType);
//     ObpSymbolicLinkObjectType->TypeInfo.ValidAccessMask &= ~SYNCHRONIZE;
//
//     /* Phase 0 initialization complete */
//     ObpInitializationPhase++;
//     return TRUE;
//
// ObPostPhase0:
//
//     /* Re-initialize lookaside lists */
//     ObInit2();
//
//     /* Initialize Object Types directory attributes */
//     RtlInitUnicodeString(&Name, L"\\");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &Name,
//                                OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
//                                NULL,
//                                SePublicDefaultUnrestrictedSd);
//
//     /* Create the directory */
//     Status = NtCreateDirectoryObject(&Handle,
//                                      DIRECTORY_ALL_ACCESS,
//                                      &ObjectAttributes);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Get a handle to it */
//     Status = ObReferenceObjectByHandle(Handle,
//                                        0,
//                                        ObpDirectoryObjectType,
//                                        KernelMode,
//                                        (PVOID*)&ObpRootDirectoryObject,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Close the extra handle */
//     Status = NtClose(Handle);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Create a custom security descriptor for the KernelObjects directory */
//     Status = ObpCreateKernelObjectsSD(&KernelObjectsSD);
//     if (!NT_SUCCESS(Status))
//         return FALSE;
//
//     /* Initialize the KernelObjects directory attributes */
//     RtlInitUnicodeString(&Name, L"\\KernelObjects");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &Name,
//                                OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
//                                NULL,
//                                KernelObjectsSD);
//
//     /* Create the directory */
//     Status = NtCreateDirectoryObject(&Handle,
//                                      DIRECTORY_ALL_ACCESS,
//                                      &ObjectAttributes);
//     ExFreePoolWithTag(KernelObjectsSD, TAG_SD);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Close the extra handle */
//     Status = NtClose(Handle);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Initialize ObjectTypes directory attributes */
//     RtlInitUnicodeString(&Name, L"\\ObjectTypes");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &Name,
//                                OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
//                                NULL,
//                                NULL);
//
//     /* Create the directory */
//     Status = NtCreateDirectoryObject(&Handle,
//                                      DIRECTORY_ALL_ACCESS,
//                                      &ObjectAttributes);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Get a handle to it */
//     Status = ObReferenceObjectByHandle(Handle,
//                                        0,
//                                        ObpDirectoryObjectType,
//                                        KernelMode,
//                                        (PVOID*)&ObpTypeDirectoryObject,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Close the extra handle */
//     Status = NtClose(Handle);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Initialize the lookup context and lock it */
//     ObpInitializeLookupContext(&Context);
//     ObpAcquireLookupContextLock(&Context, ObpTypeDirectoryObject);
//
//     /* Loop the object types */
//     ListHead = &ObpTypeObjectType->TypeList;
//     NextEntry = ListHead->Flink;
//     while (ListHead != NextEntry)
//     {
//         /* Get the creator info from the list */
//         CreatorInfo = CONTAINING_RECORD(NextEntry,
//                                         OBJECT_HEADER_CREATOR_INFO,
//                                         TypeList);
//
//         /* Recover the header and the name header from the creator info */
//         Header = (POBJECT_HEADER)(CreatorInfo + 1);
//         NameInfo = OBJECT_HEADER_TO_NAME_INFO(Header);
//
//         /* Make sure we have a name, and aren't inserted yet */
//         if ((NameInfo) && !(NameInfo->Directory))
//         {
//             /* Do the initial lookup to setup the context */
//             if (!ObpLookupEntryDirectory(ObpTypeDirectoryObject,
//                                          &NameInfo->Name,
//                                          OBJ_CASE_INSENSITIVE,
//                                          FALSE,
//                                          &Context))
//             {
//                 /* Insert this object type */
//                 ObpInsertEntryDirectory(ObpTypeDirectoryObject,
//                                         &Context,
//                                         Header);
//             }
//         }
//
//         /* Move to the next entry */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Cleanup after lookup */
//     ObpReleaseLookupContext(&Context);
//
//     /* Initialize DOS Devices Directory and related Symbolic Links */
//     Status = ObpCreateDosDevicesDirectory();
//     if (!NT_SUCCESS(Status)) return FALSE;
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
