use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use crate::obhandle::ObpSetHandleAttributes;
use ntkrnl_sys::ddk::{
    _OBJECT_HEADER, _OBJECT_TYPE, _OBJECT_HEADER_HANDLE_INFO, _OBJECT_HEADER_NAME_INFO,
    _OBJECT_HEADER_CREATOR_INFO, _OBJECT_HEADER_QUOTA_INFO, _OBJECT_CREATE_INFORMATION,
    _OBJECT_TYPE_INITIALIZER, _OBJECT_ATTRIBUTES, _FILE_OBJECT, _LPCP_PORT_OBJECT,
    _OBJECT_BASIC_INFORMATION, _OBJECT_HANDLE_FLAG_INFORMATION, _OBJECT_NAME_INFORMATION,
    _OBJECT_TYPE_INFORMATION, _KAPC_STATE, _EPROCESS, _UNICODE_STRING, _SECURITY_QUALITY_OF_SERVICE,
    _EPROCESS_QUOTA_BLOCK, _GENERAL_LOOKASIDE,
    POBJECT_HEADER, POBJECT_TYPE, POBJECT_HEADER_HANDLE_INFO, POBJECT_HEADER_NAME_INFO,
    POBJECT_HEADER_CREATOR_INFO, POBJECT_HEADER_QUOTA_INFO, POBJECT_CREATE_INFORMATION,
    POBJECT_TYPE_INITIALIZER, POBJECT_ATTRIBUTES,
    POBJECT_BASIC_INFORMATION, POBJECT_HANDLE_FLAG_INFORMATION, POBJECT_NAME_INFORMATION,
    POBJECT_TYPE_INFORMATION, POBJECT_DIRECTORY, PKAPC_STATE,
    KPROCESSOR_MODE, HANDLE,
    PUNICODE_STRING, PCUNICODE_STRING, PANSI_STRING,
    ANSI_STRING, KEVENT,
    SECURITY_INFORMATION, PSECURITY_DESCRIPTOR,
    PSECURITY_INFORMATION, SECURITY_OPERATION_CODE, PGENERIC_MAPPING,
    _SECURITY_OPERATION_CODE_DeleteSecurityDescriptor, _SECURITY_OPERATION_CODE_QuerySecurityDescriptor,
    _OBJECT_INFORMATION_CLASS_ObjectBasicInformation, _OBJECT_INFORMATION_CLASS_ObjectTypeInformation,
    _OBJECT_INFORMATION_CLASS_ObjectNameInformation, _OBJECT_INFORMATION_CLASS_ObjectTypesInformation,
    _OBJECT_INFORMATION_CLASS_ObjectHandleFlagInformation, _OBJECT_INFORMATION_CLASS_ObjectSessionInformation,
    _PP_NPAGED_LOOKASIDE_NUMBER_LookasideCreateInfoList,
    _PP_NPAGED_LOOKASIDE_NUMBER_LookasideNameBufferList,
    OBJECT_HANDLE_INFORMATION, POBJECT_SYMBOLIC_LINK,
    OBJECT_INFORMATION_CLASS, WORK_QUEUE_ITEM,
    OBP_LOOKUP_CONTEXT, OBP_SET_HANDLE_ATTRIBUTES_CONTEXT,
    ExChangeHandle, ExDeleteResourceLite, ExFreePool, ExInitializeResourceLite,
    KeStackAttachProcess, KeUnstackDetachProcess,
    ObReferenceObjectByHandle,
    ObpAcquireDirectoryLockExclusive, ObpAcquireLookupContextLock, ObpAcquireObjectLock,
    ObpAllocateObjectCreateInfoBuffer, ObpCalloutEnd, ObpCalloutStart,
    ObpDeleteNameCheck, ObpEnterObjectTypeMutex,
    ObpFreeCapturedAttributes, ObpFreeObjectCreateInformation, ObpInitializeLookupContext,
    ObpInsertEntryDirectory, ObpLeaveObjectTypeMutex, ObpLookupEntryDirectory,
    ObpReleaseDirectoryLock, ObpReleaseLookupContext, ObpReleaseObjectLock,
    ObpReleaseObjectCreateInformation, ObQueryNameString,
    PsGetCurrentProcessId, PsGetCurrentProcessSessionId, PsReturnSharedPoolQuota,
    SeCaptureSecurityDescriptor, SeComputeQuotaInformationSize,
    SeSinglePrivilegeCheck,
    RtlCopyUnicodeString, RtlInitEmptyUnicodeString,
    RtlUnicodeStringToAnsiString, RtlFreeAnsiString,
};

const LookasideNameBufferList: u32 = _PP_NPAGED_LOOKASIDE_NUMBER_LookasideNameBufferList;
const LookasideCreateInfoList: u32 = _PP_NPAGED_LOOKASIDE_NUMBER_LookasideCreateInfoList;

const TAG_OBJECT_TYPE: ULONG = 0x546A624F;
const FLG_MAINTAIN_OBJECT_TYPELIST: ULONG = 0x4000;

// ntkrnl_sys::ddk's bindgen-generated OB_SECURITY_METHOD (and its SeDefaultObjectMethod
// extern decl) predates the upstream-sync merge's se.h/obtypes.h change that appended a
// trailing KPROCESSOR_MODE AccessMode param -- the checked-in bindgen_*.rs was never
// regenerated. Declare the REAL 9-param signature locally (shadowing the stale 8-param
// import) so the pointer-identity assignment below transmutes from a type that actually
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

extern "C" {
    static mut NtGlobalFlag: ULONG;
    static mut PsThreadType: POBJECT_TYPE;
}

// DEBUG: UART trace for crash diagnosis

#[no_mangle]
pub static mut ObpTypeObjectType: POBJECT_TYPE = core::ptr::null_mut();
#[no_mangle]
pub static mut ObpDefaultObject: KEVENT = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut ObpReaperWorkItem: WORK_QUEUE_ITEM = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut ObpReaperList: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut ObpNameBufferLookasideList: _GENERAL_LOOKASIDE = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut ObpCreateInfoLookasideList: _GENERAL_LOOKASIDE = unsafe { core::mem::zeroed() };
static mut ObpObjectsCreated: ULONG = 0;
static mut ObpObjectsWithName: ULONG = 0;
static mut ObpObjectsWithPoolQuota: ULONG = 0;
static mut ObpObjectsWithHandleDB: ULONG = 0;
static mut ObpObjectsWithCreatorInfo: ULONG = 0;
static mut ObpObjectTypes: [POBJECT_TYPE; 32] = [core::ptr::null_mut(); 32];

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn ObpDeallocateObject(Object: PVOID) {
    ObpDeallocateObjectImpl(Object)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn ObpDeallocateObject(Object: PVOID) {
    ObpDeallocateObjectImpl(Object)
}

unsafe fn ObpDeallocateObjectImpl(Object: PVOID) {
    let mut HeaderLocation: PVOID;
    let Header = OBJECT_TO_OBJECT_HEADER(Object);
    let ObjectType = (*Header).Type;
    HeaderLocation = Header as PVOID;

    let CreatorInfo = OBJECT_HEADER_TO_CREATOR_INFO(Header);
    if !CreatorInfo.is_null() {
        HeaderLocation = CreatorInfo as PVOID;
    }
    let NameInfo = OBJECT_HEADER_TO_NAME_INFO(Header);
    if !NameInfo.is_null() {
        HeaderLocation = NameInfo as PVOID;
    }
    let HandleInfo = OBJECT_HEADER_TO_HANDLE_INFO(Header);
    if !HandleInfo.is_null() {
        HeaderLocation = HandleInfo as PVOID;
    }
    let QuotaInfo = OBJECT_HEADER_TO_QUOTA_INFO(Header);
    if !QuotaInfo.is_null() {
        HeaderLocation = QuotaInfo as PVOID;
    }

    InterlockedDecrement(core::ptr::addr_of_mut!((*ObjectType).TotalNumberOfObjects) as *mut LONG);

    if ((*Header).Flags & OB_FLAG_CREATE_INFO) != 0 {
        if !(*Header).__bindgen_anon_2.ObjectCreateInfo.is_null() {
            ObpFreeObjectCreateInformation((*Header).__bindgen_anon_2.ObjectCreateInfo);
            (*Header).__bindgen_anon_2.ObjectCreateInfo = core::ptr::null_mut();
        }
    } else {
        if !(*Header).__bindgen_anon_2.QuotaBlockCharged.is_null() {
            let (PagedPoolCharge, NonPagedPoolCharge) = if !QuotaInfo.is_null() {
                (
                    (*QuotaInfo).PagedPoolCharge + (*QuotaInfo).SecurityDescriptorCharge,
                    (*QuotaInfo).NonPagedPoolCharge,
                )
            } else {
                let mut paged = (*ObjectType).TypeInfo.DefaultPagedPoolCharge;
                if ((*Header).Flags & OB_FLAG_SECURITY) != 0 {
                    paged += 2048;
                }
                (paged, (*ObjectType).TypeInfo.DefaultNonPagedPoolCharge)
            };

            if (*Header).__bindgen_anon_2.QuotaBlockCharged != OBP_SYSTEM_PROCESS_QUOTA as PVOID {
                PsReturnSharedPoolQuota(
                    (*Header).__bindgen_anon_2.QuotaBlockCharged as *mut _EPROCESS_QUOTA_BLOCK,
                    PagedPoolCharge as _,
                    NonPagedPoolCharge as _,
                );
            }
        }
    }

    if !HandleInfo.is_null() && ((*Header).Flags & OB_FLAG_SINGLE_PROCESS) == 0 {
        ExFreePool((*HandleInfo).__bindgen_anon_1.HandleCountDatabase as PVOID);
        (*HandleInfo).__bindgen_anon_1.HandleCountDatabase = core::ptr::null_mut();
    }

    if !NameInfo.is_null() && !(*NameInfo).Name.Buffer.is_null() {
        ExFreePool((*NameInfo).Name.Buffer as PVOID);
        (*NameInfo).Name.Buffer = core::ptr::null_mut();
    }

    (*Header).Type = 0xBAADB0B0BAADB0B0u64 as POBJECT_TYPE;

    ExFreePoolWithTag(HeaderLocation, (*ObjectType).Key);
}

#[no_mangle]
pub unsafe extern "system" fn ObpDeleteObject(Object: PVOID, CalledFromWorkerThread: BOOLEAN) {
    let Object = opaque_ptr(Object);
    let Header = OBJECT_TO_OBJECT_HEADER(Object);
    let ObjectType = (*Header).Type;

    let NameInfo = OBJECT_HEADER_TO_NAME_INFO(Header);
    let CreatorInfo = OBJECT_HEADER_TO_CREATOR_INFO(Header);

    if !CreatorInfo.is_null() && !IsListEmpty(core::ptr::addr_of_mut!((*CreatorInfo).TypeList) as *const LIST_ENTRY) {
        ObpEnterObjectTypeMutex(ObjectType);
        RemoveEntryList(core::ptr::addr_of_mut!((*CreatorInfo).TypeList) as *mut LIST_ENTRY);
        ObpLeaveObjectTypeMutex(ObjectType);
    }

    if !NameInfo.is_null() && !(*NameInfo).Name.Buffer.is_null() {
        ExFreePool((*NameInfo).Name.Buffer as PVOID);
        RtlInitEmptyUnicodeString(
            core::ptr::addr_of_mut!((*NameInfo).Name) as PUNICODE_STRING,
            core::ptr::null_mut(),
            0,
        );
    }

    if !(*Header).SecurityDescriptor.is_null() {
        let mut CalloutIrql: KIRQL = 0;
        ObpCalloutStart(core::ptr::addr_of_mut!(CalloutIrql));
        crate::obsecure::ObpCallSecurityProcedure(
            (*ObjectType).TypeInfo.SecurityProcedure,
            Object,
            _SECURITY_OPERATION_CODE_DeleteSecurityDescriptor,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*Header).SecurityDescriptor),
            0,
            core::ptr::null_mut(),
        );
        ObpCalloutEnd(CalloutIrql, b"Security\0".as_ptr() as PCHAR, ObjectType, Object);
    }

    if (*ObjectType).TypeInfo.DeleteProcedure.is_some() {
        if CalledFromWorkerThread == 0 {
            (*Header).Flags |= OB_FLAG_DEFER_DELETE;
        }

        let mut CalloutIrql: KIRQL = 0;
        ObpCalloutStart(core::ptr::addr_of_mut!(CalloutIrql));
        ((*ObjectType).TypeInfo.DeleteProcedure.unwrap())(Object);
        ObpCalloutEnd(CalloutIrql, b"Delete\0".as_ptr() as PCHAR, ObjectType, Object);
    }

    ObpDeallocateObject(Object);
}

#[no_mangle]
pub unsafe extern "system" fn ObpReapObject(_Parameter: PVOID) {
    let _Parameter = opaque_ptr(_Parameter);
    loop {
        let mut ReapObject = InterlockedExchangePointer(
            core::ptr::addr_of_mut!(ObpReaperList),
            1 as PVOID,
        ) as POBJECT_HEADER;

        loop {
            let NextObject = (*ReapObject).__bindgen_anon_1.NextToFree as POBJECT_HEADER;

            ObpDeleteObject(
                core::ptr::addr_of_mut!((*ReapObject).Body) as PVOID,
                1u8,
            );

            ReapObject = NextObject;
            if ReapObject.is_null() || ReapObject == 1 as PVOID as POBJECT_HEADER {
                break;
            }
        }

        if ObpReaperList != 1 as PVOID {
            continue;
        }
        if InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!(ObpReaperList),
            core::ptr::null_mut(),
            1 as PVOID,
        ) == 1 as PVOID
        {
            break;
        }
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn ObpSetPermanentObject(ObjectBody: PVOID, Permanent: BOOLEAN) {
    ObpSetPermanentObjectImpl(ObjectBody, Permanent)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn ObpSetPermanentObject(ObjectBody: PVOID, Permanent: BOOLEAN) {
    ObpSetPermanentObjectImpl(ObjectBody, Permanent)
}

unsafe fn ObpSetPermanentObjectImpl(ObjectBody: PVOID, Permanent: BOOLEAN) {
    let ObjectHeader = OBJECT_TO_OBJECT_HEADER(ObjectBody);

    ObpAcquireObjectLock(ObjectHeader);

    if Permanent != 0 {
        (*ObjectHeader).Flags |= OB_FLAG_PERMANENT;
        ObpReleaseObjectLock(ObjectHeader);
    } else {
        (*ObjectHeader).Flags &= !OB_FLAG_PERMANENT;
        ObpReleaseObjectLock(ObjectHeader);
        ObpDeleteNameCheck(ObjectBody);
    }
}

unsafe fn ObpAllocateObjectNameBuffer(
    Length: ULONG,
    UseLookaside: BOOLEAN,
    ObjectName: *mut _UNICODE_STRING,
) -> PWCHAR {
    let mut MaximumLength = Length + core::mem::size_of::<u16>() as ULONG;

    let Buffer = if UseLookaside == 0 || MaximumLength > OBP_NAME_LOOKASIDE_MAX_SIZE {
        if MaximumLength <= OBP_NAME_LOOKASIDE_MAX_SIZE {
            MaximumLength = OBP_NAME_LOOKASIDE_MAX_SIZE + core::mem::size_of::<u16>() as ULONG;
        }
        ExAllocatePoolWithTag(PagedPool, MaximumLength as usize, OB_NAME_TAG)
    } else {
        MaximumLength = OBP_NAME_LOOKASIDE_MAX_SIZE;
        ObpAllocateObjectCreateInfoBuffer(LookasideNameBufferList)
    };

    (*ObjectName).MaximumLength = MaximumLength as USHORT;
    (*ObjectName).Length = Length as USHORT;
    (*ObjectName).Buffer = Buffer as PWCHAR;
    Buffer as PWCHAR
}

#[no_mangle]
pub unsafe extern "system" fn ObpFreeObjectNameBuffer(Name: *mut _UNICODE_STRING) {
    let Name = opaque_ptr(Name);
    let Buffer = (*Name).Buffer as PVOID;

    if (*Name).MaximumLength as ULONG != OBP_NAME_LOOKASIDE_MAX_SIZE {
        ExFreePool(Buffer);
    } else {
        ObpFreeCapturedAttributes(Buffer, LookasideNameBufferList);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ObpCaptureObjectName(
    CapturedName: *mut _UNICODE_STRING,
    ObjectName: *mut _UNICODE_STRING,
    AccessMode: KPROCESSOR_MODE,
    UseLookaside: BOOLEAN,
) -> NTSTATUS {
    let CapturedName = opaque_ptr(CapturedName);
    let ObjectName = opaque_ptr(ObjectName);
    let mut Status = STATUS_SUCCESS;
    let mut StringBuffer: PWCHAR = core::ptr::null_mut();
    let mut LocalName: _UNICODE_STRING = core::mem::zeroed();

    RtlInitEmptyUnicodeString(CapturedName as PUNICODE_STRING, core::ptr::null_mut(), 0);

    if AccessMode != 0 {
        if let Err(s) = probe_user_buffer(ObjectName as _, core::mem::size_of::<_UNICODE_STRING>(), core::mem::size_of::<ULONG>(), AccessMode) {
            return s;
        }
        LocalName = core::ptr::read_volatile(ObjectName);
        if let Err(s) = probe_user_buffer(
            LocalName.Buffer as *const u8 as _,
            LocalName.Length as usize,
            core::mem::size_of::<u16>(),
            AccessMode,
        ) {
            return s;
        }
    } else {
        LocalName = *ObjectName;
    }

    let StringLength = LocalName.Length as ULONG;
    if StringLength != 0 {
        if (StringLength & (core::mem::size_of::<u16>() as ULONG - 1)) != 0
            || StringLength == (0xFFFFu16 - core::mem::size_of::<u16>() as u16 + 1) as ULONG
        {
            Status = STATUS_OBJECT_NAME_INVALID;
        } else {
            StringBuffer = ObpAllocateObjectNameBuffer(StringLength, UseLookaside, CapturedName);
            if StringBuffer.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                core::ptr::copy_nonoverlapping(
                    LocalName.Buffer as *const u8,
                    StringBuffer as *mut u8,
                    StringLength as usize,
                );
                *StringBuffer.offset((StringLength / core::mem::size_of::<u16>() as ULONG) as isize) = 0;
            }
        }
    }

    if !NT_SUCCESS(Status) && !StringBuffer.is_null() {
        ObpFreeObjectNameBuffer(CapturedName);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ObpCaptureObjectCreateInformation(
    ObjectAttributes: POBJECT_ATTRIBUTES,
    AccessMode: KPROCESSOR_MODE,
    CreatorMode: KPROCESSOR_MODE,
    AllocateFromLookaside: BOOLEAN,
    ObjectCreateInfo: POBJECT_CREATE_INFORMATION,
    ObjectName: *mut _UNICODE_STRING,
) -> NTSTATUS {
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let ObjectCreateInfo = opaque_ptr(ObjectCreateInfo);
    let ObjectName = opaque_ptr(ObjectName);
    let mut SdCharge: ULONG;
    let mut QuotaInfoSize: ULONG = 0;
    let mut Status = STATUS_SUCCESS;
    let mut SecurityDescriptor: PSECURITY_DESCRIPTOR;
    let mut SecurityQos: PVOID;
    let mut LocalObjectName: PUNICODE_STRING = core::ptr::null_mut();

    core::ptr::write_bytes(ObjectCreateInfo as *mut u8, 0, core::mem::size_of::<_OBJECT_CREATE_INFORMATION>());

    if !ObjectAttributes.is_null() {
        if AccessMode != 0 {
            if let Err(s) = probe_user_buffer(
                ObjectAttributes as *const u8 as _,
                core::mem::size_of::<_OBJECT_ATTRIBUTES>(),
                core::mem::size_of::<ULONG>(),
                AccessMode,
            ) {
                return s;
            }
        }

        if (*ObjectAttributes).Length != core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG
            || ((*ObjectAttributes).Attributes & !OBJ_VALID_KERNEL_ATTRIBUTES) != 0
        {
            return STATUS_INVALID_PARAMETER;
        }

        (*ObjectCreateInfo).RootDirectory = (*ObjectAttributes).RootDirectory;
        (*ObjectCreateInfo).Attributes = (*ObjectAttributes).Attributes & OBJ_VALID_KERNEL_ATTRIBUTES;
        if CreatorMode != 0 {
            (*ObjectCreateInfo).Attributes &= !OBJ_KERNEL_HANDLE;
        }
        LocalObjectName = (*ObjectAttributes).ObjectName as PUNICODE_STRING;
        SecurityDescriptor = (*ObjectAttributes).SecurityDescriptor;
        SecurityQos = (*ObjectAttributes).SecurityQualityOfService;

        if !SecurityDescriptor.is_null() {
            Status = SeCaptureSecurityDescriptor(
                SecurityDescriptor,
                AccessMode,
                NonPagedPool,
                1u8,
                core::ptr::addr_of_mut!((*ObjectCreateInfo).SecurityDescriptor),
            );
            if !NT_SUCCESS(Status) {
                (*ObjectCreateInfo).SecurityDescriptor = core::ptr::null_mut();
                return Status;
            }

            SdCharge = 2048;
            SeComputeQuotaInformationSize(
                (*ObjectCreateInfo).SecurityDescriptor,
                core::ptr::addr_of_mut!(QuotaInfoSize),
            );
            if (2 * QuotaInfoSize) > 2048 {
                SdCharge = 2 * QuotaInfoSize;
            }

            (*ObjectCreateInfo).SecurityDescriptorCharge = SdCharge;
            (*ObjectCreateInfo).ProbeMode = AccessMode;
        }

        if !SecurityQos.is_null() {
            if AccessMode != 0 {
                if let Err(s) = probe_user_buffer(
                    SecurityQos as *const u8 as _,
                    core::mem::size_of::<_SECURITY_QUALITY_OF_SERVICE>(),
                    core::mem::size_of::<ULONG>(),
                    AccessMode,
                ) {
                    return s;
                }
            }

            (*ObjectCreateInfo).SecurityQualityOfService = *(SecurityQos as *const _SECURITY_QUALITY_OF_SERVICE);
            (*ObjectCreateInfo).SecurityQos = core::ptr::addr_of_mut!((*ObjectCreateInfo).SecurityQualityOfService);
        }
    } else {
        LocalObjectName = core::ptr::null_mut();
    }

    if !LocalObjectName.is_null() {
        Status = ObpCaptureObjectName(ObjectName, LocalObjectName as *mut _UNICODE_STRING, AccessMode, AllocateFromLookaside);
    } else {
        RtlInitEmptyUnicodeString(ObjectName as PUNICODE_STRING, core::ptr::null_mut(), 0);

        if !(*ObjectCreateInfo).RootDirectory.is_null() {
            Status = STATUS_OBJECT_NAME_INVALID;
        }
    }

    if !NT_SUCCESS(Status) {
        ObpReleaseObjectCreateInformation(ObjectCreateInfo);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ObFreeObjectCreateInfoBuffer(ObjectCreateInfo: POBJECT_CREATE_INFORMATION) {
    let ObjectCreateInfo = opaque_ptr(ObjectCreateInfo);
    ObpFreeCapturedAttributes(ObjectCreateInfo as PVOID, LookasideCreateInfoList);
}

unsafe fn ObpAllocateObject(
    ObjectCreateInfo: POBJECT_CREATE_INFORMATION,
    ObjectName: *mut _UNICODE_STRING,
    ObjectType: POBJECT_TYPE,
    ObjectSize: ULONG,
    PreviousMode: KPROCESSOR_MODE,
    ObjectHeader: *mut POBJECT_HEADER,
) -> NTSTATUS {
    let mut Header: POBJECT_HEADER;
    let QuotaSize: ULONG;
    let HandleSize: ULONG;
    let NameSize: ULONG;
    let CreatorSize: ULONG;
    let PoolType: POOL_TYPE;
    let FinalSize: ULONG;
    let Tag: ULONG;

    ObpObjectsCreated += 1;

    let (pool_type, tag) = if ObjectType.is_null() {
        (NonPagedPool, TAG_OBJECT_TYPE)
    } else {
        ((*ObjectType).TypeInfo.PoolType, (*ObjectType).Key)
    };
    let PoolType = pool_type;
    let Tag = tag;

    let (quota_size, handle_size, name_size, creator_size) = if ObjectCreateInfo.is_null() {
        (0, 0, core::mem::size_of::<_OBJECT_HEADER_NAME_INFO>() as ULONG, core::mem::size_of::<_OBJECT_HEADER_CREATOR_INFO>() as ULONG)
    } else {
        let quota = if (((*ObjectCreateInfo).PagedPoolCharge != (*ObjectType).TypeInfo.DefaultPagedPoolCharge)
            || ((*ObjectCreateInfo).NonPagedPoolCharge != (*ObjectType).TypeInfo.DefaultNonPagedPoolCharge)
            || ((*ObjectCreateInfo).SecurityDescriptorCharge > 2048))
            && (PsGetCurrentProcess() as PVOID != PsInitialSystemProcess)
            || ((*ObjectCreateInfo).Attributes & OBJ_EXCLUSIVE) != 0
        {
            ObpObjectsWithPoolQuota += 1;
            core::mem::size_of::<_OBJECT_HEADER_QUOTA_INFO>() as ULONG
        } else {
            0
        };

        let handle = if (*ObjectType).TypeInfo.MaintainHandleCount != 0 {
            ObpObjectsWithHandleDB += 1;
            core::mem::size_of::<_OBJECT_HEADER_HANDLE_INFO>() as ULONG
        } else {
            0
        };

        let name = if !(*ObjectName).Buffer.is_null() {
            ObpObjectsWithName += 1;
            core::mem::size_of::<_OBJECT_HEADER_NAME_INFO>() as ULONG
        } else {
            0
        };

        let creator = if (*ObjectType).TypeInfo.MaintainTypeList != 0 {
            ObpObjectsWithCreatorInfo += 1;
            core::mem::size_of::<_OBJECT_HEADER_CREATOR_INFO>() as ULONG
        } else {
            0
        };

        (quota, handle, name, creator)
    };
    let QuotaSize = quota_size;
    let HandleSize = handle_size;
    let NameSize = name_size;
    let CreatorSize = creator_size;

    let FinalSize = QuotaSize + HandleSize + NameSize + CreatorSize + core::mem::offset_of!(_OBJECT_HEADER, Body) as ULONG;

    Header = ExAllocatePoolWithTag(PoolType, (FinalSize + ObjectSize) as usize, Tag) as POBJECT_HEADER;
    if Header.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }



    if QuotaSize != 0 {
        let QuotaInfo = Header as POBJECT_HEADER_QUOTA_INFO;
        (*QuotaInfo).PagedPoolCharge = (*ObjectCreateInfo).PagedPoolCharge;
        (*QuotaInfo).NonPagedPoolCharge = (*ObjectCreateInfo).NonPagedPoolCharge;
        (*QuotaInfo).SecurityDescriptorCharge = (*ObjectCreateInfo).SecurityDescriptorCharge;
        (*QuotaInfo).ExclusiveProcess = core::ptr::null_mut();
        Header = (QuotaInfo.offset(1)) as POBJECT_HEADER;
    }

    if HandleSize != 0 {
        let HandleInfo = Header as POBJECT_HEADER_HANDLE_INFO;
        (*HandleInfo).__bindgen_anon_1.SingleEntry.HandleCount = 0;
        Header = (HandleInfo.offset(1)) as POBJECT_HEADER;
    }

    if NameSize != 0 {
        let NameInfo = Header as POBJECT_HEADER_NAME_INFO;
        (*NameInfo).Name = *ObjectName;
        (*NameInfo).Directory = core::ptr::null_mut();
        (*NameInfo).QueryReferences = 1;

        if PreviousMode == 0 && !ObjectCreateInfo.is_null() && ((*ObjectCreateInfo).Attributes & OBJ_KERNEL_EXCLUSIVE) != 0 {
            (*NameInfo).QueryReferences |= OB_FLAG_KERNEL_EXCLUSIVE;
        }

        Header = (NameInfo.offset(1)) as POBJECT_HEADER;
    }

    if CreatorSize != 0 {
        let CreatorInfo = Header as POBJECT_HEADER_CREATOR_INFO;
        (*CreatorInfo).CreatorBackTraceIndex = 0;
        (*CreatorInfo).CreatorUniqueProcess = PsGetCurrentProcessId();
        InitializeListHead(core::ptr::addr_of_mut!((*CreatorInfo).TypeList) as *mut LIST_ENTRY);
        Header = (CreatorInfo.offset(1)) as POBJECT_HEADER;
    }

    (*Header).QuotaInfoOffset = if QuotaSize != 0 { (QuotaSize + HandleSize + NameSize + CreatorSize) as UCHAR } else { 0 };
    (*Header).HandleInfoOffset = if HandleSize != 0 { (HandleSize + NameSize + CreatorSize) as UCHAR } else { 0 };
    (*Header).NameInfoOffset = if NameSize != 0 { (NameSize + CreatorSize) as UCHAR } else { 0 };

    (*Header).Flags = OB_FLAG_CREATE_INFO;
    if CreatorSize != 0 {
        (*Header).Flags |= OB_FLAG_CREATOR_INFO;
    }
    if HandleSize != 0 {
        (*Header).Flags |= OB_FLAG_SINGLE_PROCESS;
    }

    (*Header).PointerCount = 1;
    (*Header).__bindgen_anon_1.HandleCount = 0;
    (*Header).Type = ObjectType;
    (*Header).__bindgen_anon_2.ObjectCreateInfo = ObjectCreateInfo;
    (*Header).SecurityDescriptor = core::ptr::null_mut();

    if !ObjectCreateInfo.is_null() && ((*ObjectCreateInfo).Attributes & OBJ_PERMANENT) != 0 {
        (*Header).Flags |= OB_FLAG_PERMANENT;
    }

    if !ObjectCreateInfo.is_null() && ((*ObjectCreateInfo).Attributes & OBJ_EXCLUSIVE) != 0 {
        (*Header).Flags |= OB_FLAG_EXCLUSIVE;
    }

    if PreviousMode == 0 {
        (*Header).Flags |= OB_FLAG_KERNEL_MODE;
    }

    if !ObjectType.is_null() {
        InterlockedIncrement(core::ptr::addr_of_mut!((*ObjectType).TotalNumberOfObjects) as *mut LONG);

        (*ObjectType).HighWaterNumberOfObjects = core::cmp::max(
            (*ObjectType).TotalNumberOfObjects,
            (*ObjectType).HighWaterNumberOfObjects,
        );
    }

    *ObjectHeader = Header;
    STATUS_SUCCESS
}

unsafe fn ObpQueryNameInfoSize(ObjectHeader: POBJECT_HEADER) -> ULONG {
    let mut NameSize: ULONG = 0;

    let NameInfo = OBJECT_HEADER_TO_NAME_INFO(ObjectHeader);
    if NameInfo.is_null() {
        return 0;
    }

    let mut ParentDirectory = (*NameInfo).Directory;
    if ParentDirectory.is_null() {
        return 0;
    }

    NameSize = core::mem::size_of::<u16>() as ULONG + (*NameInfo).Name.Length as ULONG;
    loop {
        let NameInfoParent = OBJECT_HEADER_TO_NAME_INFO(OBJECT_TO_OBJECT_HEADER(ParentDirectory as PVOID));
        if NameInfoParent.is_null() {
            break;
        }

        ParentDirectory = (*NameInfoParent).Directory;
        if ParentDirectory.is_null() {
            break;
        }

        NameSize += core::mem::size_of::<u16>() as ULONG + (*NameInfoParent).Name.Length as ULONG;
    }

    NameSize += core::mem::size_of::<_OBJECT_NAME_INFORMATION>() as ULONG + core::mem::size_of::<u16>() as ULONG;
    NameSize
}

unsafe fn ObQueryTypeInfo(
    ObjectType: POBJECT_TYPE,
    ObjectTypeInfo: POBJECT_TYPE_INFORMATION,
    Length: ULONG,
    ReturnLength: *mut ULONG,
) -> NTSTATUS {
    let mut Status = STATUS_SUCCESS;

    *ReturnLength += core::mem::size_of::<_OBJECT_TYPE_INFORMATION>() as ULONG
        + (((*ObjectType).Name.Length as ULONG + core::mem::size_of::<u16>() as ULONG + core::mem::size_of::<usize>() as ULONG - 1)
            & !(core::mem::size_of::<usize>() as ULONG - 1));

    if Length < *ReturnLength {
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    (*ObjectTypeInfo).TotalNumberOfHandles = (*ObjectType).TotalNumberOfHandles;
    (*ObjectTypeInfo).TotalNumberOfObjects = (*ObjectType).TotalNumberOfObjects;
    (*ObjectTypeInfo).HighWaterNumberOfHandles = (*ObjectType).HighWaterNumberOfHandles;
    (*ObjectTypeInfo).HighWaterNumberOfObjects = (*ObjectType).HighWaterNumberOfObjects;
    (*ObjectTypeInfo).PoolType = (*ObjectType).TypeInfo.PoolType;
    (*ObjectTypeInfo).DefaultNonPagedPoolCharge = (*ObjectType).TypeInfo.DefaultNonPagedPoolCharge;
    (*ObjectTypeInfo).DefaultPagedPoolCharge = (*ObjectType).TypeInfo.DefaultPagedPoolCharge;
    (*ObjectTypeInfo).ValidAccessMask = (*ObjectType).TypeInfo.ValidAccessMask;
    (*ObjectTypeInfo).SecurityRequired = (*ObjectType).TypeInfo.SecurityRequired;
    (*ObjectTypeInfo).InvalidAttributes = (*ObjectType).TypeInfo.InvalidAttributes;
    (*ObjectTypeInfo).GenericMapping = (*ObjectType).TypeInfo.GenericMapping;
    (*ObjectTypeInfo).MaintainHandleCount = (*ObjectType).TypeInfo.MaintainHandleCount;

    let InfoBuffer = (ObjectTypeInfo.offset(1)) as PWSTR;
    (*ObjectTypeInfo).TypeName.Buffer = InfoBuffer;
    (*ObjectTypeInfo).TypeName.MaximumLength = (*ObjectType).Name.MaximumLength;
    (*ObjectTypeInfo).TypeName.Length = (*ObjectType).Name.Length;

    core::ptr::copy_nonoverlapping(
        (*ObjectType).Name.Buffer as *const u8,
        InfoBuffer as *mut u8,
        (*ObjectType).Name.Length as usize,
    );

    *InfoBuffer.offset(((*ObjectType).Name.Length as usize / core::mem::size_of::<u16>()) as isize) = 0;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ObCreateObject(
    ProbeMode: KPROCESSOR_MODE,
    Type: POBJECT_TYPE,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    AccessMode: KPROCESSOR_MODE,
    ParseContext: PVOID,
    ObjectSize: ULONG,
    PagedPoolCharge: ULONG,
    NonPagedPoolCharge: ULONG,
    Object: *mut PVOID,
) -> NTSTATUS {
    let Type = opaque_ptr(Type);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let ParseContext = opaque_ptr(ParseContext);
    let Object = opaque_ptr(Object);
    let mut Status: NTSTATUS;
    let ObjectCreateInfo = ObpAllocateObjectCreateInfoBuffer(LookasideCreateInfoList) as POBJECT_CREATE_INFORMATION;
    if ObjectCreateInfo.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let mut ObjectName: _UNICODE_STRING = core::mem::zeroed();
    Status = ObpCaptureObjectCreateInformation(
        ObjectAttributes,
        ProbeMode,
        AccessMode,
        0u8,
        ObjectCreateInfo,
        core::ptr::addr_of_mut!(ObjectName),
    );
    if NT_SUCCESS(Status) {
        if ((*Type).TypeInfo.InvalidAttributes & (*ObjectCreateInfo).Attributes) != 0 {
            Status = STATUS_INVALID_PARAMETER;
        } else {
            let paged_charge = if PagedPoolCharge != 0 { PagedPoolCharge } else { (*Type).TypeInfo.DefaultPagedPoolCharge };
            let nonpaged_charge = if NonPagedPoolCharge != 0 { NonPagedPoolCharge } else { (*Type).TypeInfo.DefaultNonPagedPoolCharge };

            (*ObjectCreateInfo).PagedPoolCharge = paged_charge;
            (*ObjectCreateInfo).NonPagedPoolCharge = nonpaged_charge;

            let mut Header: POBJECT_HEADER = core::ptr::null_mut();
            Status = ObpAllocateObject(
                ObjectCreateInfo,
                core::ptr::addr_of_mut!(ObjectName),
                Type,
                ObjectSize,
                AccessMode,
                core::ptr::addr_of_mut!(Header),
            );
            if NT_SUCCESS(Status) {
                *Object = core::ptr::addr_of_mut!((*Header).Body) as PVOID;

                if ((*Header).Flags & OB_FLAG_PERMANENT) != 0 {
                    if SeSinglePrivilegeCheck(SeCreatePermanentPrivilege, ProbeMode) == 0 {
                        ObpDeallocateObject(*Object);
                        Status = STATUS_PRIVILEGE_NOT_HELD;
                    }
                }

                return Status;
            }
        }

        ObpFreeObjectCreateInformation(ObjectCreateInfo);
        if !ObjectName.Buffer.is_null() {
            ObpFreeObjectNameBuffer(core::ptr::addr_of_mut!(ObjectName));
        }
        return Status;
    }

    ObpFreeCapturedAttributes(ObjectCreateInfo as PVOID, LookasideCreateInfoList);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn ObCreateObjectType(
    TypeName: PUNICODE_STRING,
    ObjectTypeInitializer: POBJECT_TYPE_INITIALIZER,
    Reserved: PVOID,
    ObjectType: *mut POBJECT_TYPE,
) -> NTSTATUS {
    let TypeName = opaque_ptr(TypeName);
    let Reserved = opaque_ptr(Reserved);
    let ObjectType = opaque_ptr(ObjectType);
    let mut Header: POBJECT_HEADER = core::ptr::null_mut();
    let mut LocalObjectType: POBJECT_TYPE;
    let HeaderSize: ULONG;
    let mut Status: NTSTATUS;
    let mut Context: OBP_LOOKUP_CONTEXT = core::mem::zeroed();
    let mut p: PWCHAR;
    let mut i: ULONG;
    let mut ObjectName: _UNICODE_STRING = core::mem::zeroed();
    let mut AnsiName: ANSI_STRING = core::mem::zeroed();

    if TypeName.is_null()
        || (*TypeName).Length == 0
        || ((*TypeName).Length % core::mem::size_of::<u16>() as u16) != 0
        || ObjectTypeInitializer.is_null()
        || (*ObjectTypeInitializer).Length != core::mem::size_of::<_OBJECT_TYPE_INITIALIZER>() as USHORT
        || ((*ObjectTypeInitializer).InvalidAttributes & !OBJ_VALID_KERNEL_ATTRIBUTES) != 0
        || ((*ObjectTypeInitializer).MaintainHandleCount != 0
            && (*ObjectTypeInitializer).OpenProcedure.is_none()
            && (*ObjectTypeInitializer).CloseProcedure.is_none())
        || ((*ObjectTypeInitializer).UseDefaultObject == 0 && (*ObjectTypeInitializer).PoolType != NonPagedPool)
    {
        return STATUS_INVALID_PARAMETER;
    }

    p = (*TypeName).Buffer;
    i = ((*TypeName).Length / core::mem::size_of::<u16>() as u16) as ULONG;
    while i > 0 {
        if *p == OBJ_NAME_PATH_SEPARATOR as u16 {
            return STATUS_OBJECT_NAME_INVALID;
        }
        p = p.offset(1);
        i -= 1;
    }

    ObpInitializeLookupContext(core::ptr::addr_of_mut!(Context));

    if !ObpTypeDirectoryObject.is_null() {
        ObpAcquireLookupContextLock(core::ptr::addr_of_mut!(Context), ObpTypeDirectoryObject);

        if !ObpLookupEntryDirectory(
            ObpTypeDirectoryObject,
            TypeName,
            OBJ_CASE_INSENSITIVE,
            0u8,
            core::ptr::addr_of_mut!(Context),
        ).is_null()
        {
            ObpReleaseLookupContext(core::ptr::addr_of_mut!(Context));
            return STATUS_OBJECT_NAME_COLLISION;
        }
    }

    ObjectName.Buffer = ExAllocatePoolWithTag(
        PagedPool,
        (*TypeName).MaximumLength as usize,
        OB_NAME_TAG,
    ) as PWCHAR;
    if ObjectName.Buffer.is_null() {
        ObpReleaseLookupContext(core::ptr::addr_of_mut!(Context));
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    ObjectName.MaximumLength = (*TypeName).MaximumLength;
    RtlCopyUnicodeString(core::ptr::addr_of_mut!(ObjectName) as PUNICODE_STRING, TypeName as PCUNICODE_STRING);

    Status = ObpAllocateObject(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ObjectName),
        ObpTypeObjectType,
        core::mem::size_of::<_OBJECT_TYPE>() as ULONG,
        0i8,
        core::ptr::addr_of_mut!(Header),
    );
    if !NT_SUCCESS(Status) {
        ObpReleaseLookupContext(core::ptr::addr_of_mut!(Context));
        ExFreePool(ObjectName.Buffer as PVOID);
        return Status;
    }

    LocalObjectType = core::ptr::addr_of_mut!((*Header).Body) as POBJECT_TYPE;
    (*LocalObjectType).Name = ObjectName;
    (*Header).Flags |= OB_FLAG_KERNEL_MODE | OB_FLAG_PERMANENT;

    (*LocalObjectType).TotalNumberOfObjects = 0;
    (*LocalObjectType).TotalNumberOfHandles = 0;
    (*LocalObjectType).HighWaterNumberOfObjects = 0;
    (*LocalObjectType).HighWaterNumberOfHandles = 0;

    if ObpTypeObjectType.is_null() {
        ObpTypeObjectType = LocalObjectType;
        (*Header).Type = ObpTypeObjectType;

        (*LocalObjectType).TotalNumberOfObjects = 1;
        (*LocalObjectType).Key = TAG_OBJECT_TYPE;
    } else {
        Status = RtlUnicodeStringToAnsiString(core::ptr::addr_of_mut!(AnsiName) as PANSI_STRING, TypeName as PCUNICODE_STRING, 1u8);
        if NT_SUCCESS(Status) {
            i = 3;
            while i >= AnsiName.Length as ULONG {
                *AnsiName.Buffer.offset(i as isize) = b' ' as i8;
                if i == 0 { break; }
                i -= 1;
            }

            (*LocalObjectType).Key = *(AnsiName.Buffer as *const ULONG);
            RtlFreeAnsiString(core::ptr::addr_of_mut!(AnsiName));
        } else {
            (*LocalObjectType).Key = *((*TypeName).Buffer as *const ULONG);
        }
    }

    (*LocalObjectType).TypeInfo = *ObjectTypeInitializer;
    (*LocalObjectType).TypeInfo.PoolType = (*ObjectTypeInitializer).PoolType;

    if (NtGlobalFlag & FLG_MAINTAIN_OBJECT_TYPELIST) != 0 {
        (*LocalObjectType).TypeInfo.MaintainTypeList = 1u8;
    }

    HeaderSize = core::mem::size_of::<_OBJECT_HEADER>() as ULONG
        + core::mem::size_of::<_OBJECT_HEADER_NAME_INFO>() as ULONG
        + (if (*ObjectTypeInitializer).MaintainHandleCount != 0 {
            core::mem::size_of::<_OBJECT_HEADER_HANDLE_INFO>() as ULONG
        } else {
            0
        });

    if (*ObjectTypeInitializer).PoolType == NonPagedPool {
        (*LocalObjectType).TypeInfo.DefaultNonPagedPoolCharge += HeaderSize;
    } else {
        (*LocalObjectType).TypeInfo.DefaultPagedPoolCharge += HeaderSize;
    }

    if (*ObjectTypeInitializer).SecurityProcedure.is_none() {
        (*LocalObjectType).TypeInfo.SecurityProcedure = Some(SeDefaultObjectMethod);
    }

    if (*LocalObjectType).TypeInfo.UseDefaultObject != 0 {
        (*LocalObjectType).TypeInfo.ValidAccessMask |= SYNCHRONIZE;
        (*LocalObjectType).DefaultObject = core::ptr::addr_of_mut!(ObpDefaultObject) as PVOID;
    } else if (*TypeName).Length == 8 && {
        let buf = core::slice::from_raw_parts((*TypeName).Buffer, 4);
        buf == &[b'F' as u16, b'i' as u16, b'l' as u16, b'e' as u16]
    } {
        (*LocalObjectType).DefaultObject = core::mem::offset_of!(_FILE_OBJECT, Event) as PVOID;
    } else if (*TypeName).Length == 24 && {
        let buf = core::slice::from_raw_parts((*TypeName).Buffer, 12);
        buf == &[
            b'W' as u16, b'a' as u16, b'i' as u16, b't' as u16, b'a' as u16, b'b' as u16,
            b'l' as u16, b'e' as u16, b'P' as u16, b'o' as u16, b'r' as u16, b't' as u16,
        ]
    } {
        (*LocalObjectType).DefaultObject = core::mem::offset_of!(_LPCP_PORT_OBJECT, WaitEvent) as PVOID;
    } else {
        (*LocalObjectType).DefaultObject = core::ptr::null_mut();
    }

    ExInitializeResourceLite(core::ptr::addr_of_mut!((*LocalObjectType).Mutex));
    i = 0;
    while i < 4 {
        ExInitializeResourceLite(core::ptr::addr_of_mut!((*LocalObjectType).ObjectLocks[i as usize]));
        i += 1;
    }
    InitializeListHead(core::ptr::addr_of_mut!((*LocalObjectType).TypeList) as *mut LIST_ENTRY);

    ObpEnterObjectTypeMutex(ObpTypeObjectType);

    let CreatorInfo = OBJECT_HEADER_TO_CREATOR_INFO(Header);
    if !CreatorInfo.is_null() {
        InsertTailList(
            core::ptr::addr_of_mut!((*ObpTypeObjectType).TypeList) as *mut LIST_ENTRY,
            core::ptr::addr_of_mut!((*CreatorInfo).TypeList) as *mut LIST_ENTRY,
        );

        (*Header).Flags &= !OB_FLAG_CREATE_INFO;
    }

    (*LocalObjectType).Index = (*ObpTypeObjectType).TotalNumberOfObjects;

    if (*LocalObjectType).Index != 0 && ((*LocalObjectType).Index as usize) < 32 {
        ObpObjectTypes[((*LocalObjectType).Index - 1) as usize] = LocalObjectType;
    }

    ObpLeaveObjectTypeMutex(ObpTypeObjectType);

    if ObpTypeDirectoryObject.is_null()
        || ObpInsertEntryDirectory(ObpTypeDirectoryObject, core::ptr::addr_of_mut!(Context), Header) != 0
    {
        if !ObpTypeDirectoryObject.is_null() {
            ObReferenceObject(ObpTypeDirectoryObject as PVOID);
        }

        ObpReleaseLookupContext(core::ptr::addr_of_mut!(Context));

        *ObjectType = LocalObjectType;
        return STATUS_SUCCESS;
    }

    ObpReleaseLookupContext(core::ptr::addr_of_mut!(Context));
    STATUS_INSUFFICIENT_RESOURCES
}

#[no_mangle]
pub unsafe extern "system" fn ObDeleteCapturedInsertInfo(Object: PVOID) {
    let Object = opaque_ptr(Object);
    let ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
    if ((*ObjectHeader).Flags & OB_FLAG_CREATE_INFO) != 0 && !(*ObjectHeader).__bindgen_anon_2.ObjectCreateInfo.is_null() {
        ObpFreeObjectCreateInformation((*ObjectHeader).__bindgen_anon_2.ObjectCreateInfo);
        (*ObjectHeader).__bindgen_anon_2.ObjectCreateInfo = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "system" fn ObpDeleteObjectType(Object: PVOID) {
    let Object = opaque_ptr(Object);
    let ObjectType = Object as POBJECT_TYPE;

    let mut i: ULONG = 0;
    while i < 4 {
        ExDeleteResourceLite(core::ptr::addr_of_mut!((*ObjectType).ObjectLocks[i as usize]));
        i += 1;
    }

    ExDeleteResourceLite(core::ptr::addr_of_mut!((*ObjectType).Mutex));
}

#[no_mangle]
pub unsafe extern "system" fn ObMakeTemporaryObject(ObjectBody: PVOID) {
    let ObjectBody = opaque_ptr(ObjectBody);
    ObpSetPermanentObject(ObjectBody, 0u8);
}

#[no_mangle]
pub unsafe extern "system" fn NtMakeTemporaryObject(ObjectHandle: HANDLE) -> NTSTATUS {
    let ObjectHandle = opaque_ptr(ObjectHandle);
    let mut ObjectBody: PVOID = core::ptr::null_mut();
    let Status = ObReferenceObjectByHandle(
        ObjectHandle,
        DELETE,
        core::ptr::null_mut(),
        KeGetPreviousMode(),
        core::ptr::addr_of_mut!(ObjectBody),
        core::ptr::null_mut(),
    );
    if Status != STATUS_SUCCESS {
        return Status;
    }

    ObpSetPermanentObject(ObjectBody, 0u8);
    ObDereferenceObject(ObjectBody);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtMakePermanentObject(ObjectHandle: HANDLE) -> NTSTATUS {
    let ObjectHandle = opaque_ptr(ObjectHandle);
    let mut ObjectBody: PVOID = core::ptr::null_mut();
    let PreviousMode = ExGetPreviousMode();

    if SeSinglePrivilegeCheck(SeCreatePermanentPrivilege, PreviousMode) == 0 {
        return STATUS_PRIVILEGE_NOT_HELD;
    }

    let Status = ObReferenceObjectByHandle(
        ObjectHandle,
        0,
        core::ptr::null_mut(),
        PreviousMode,
        core::ptr::addr_of_mut!(ObjectBody),
        core::ptr::null_mut(),
    );
    if Status != STATUS_SUCCESS {
        return Status;
    }

    ObpSetPermanentObject(ObjectBody, 1u8);
    ObDereferenceObject(ObjectBody);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryObject(
    ObjectHandle: HANDLE,
    ObjectInformationClass: OBJECT_INFORMATION_CLASS,
    ObjectInformation: PVOID,
    Length: ULONG,
    ResultLength: *mut ULONG,
) -> NTSTATUS {
    let ObjectHandle = opaque_ptr(ObjectHandle);
    let ObjectInformation = opaque_ptr(ObjectInformation);
    let ResultLength = opaque_ptr(ResultLength);
    let mut HandleInfo: OBJECT_HANDLE_INFORMATION = core::mem::zeroed();
    let mut ObjectHeader: POBJECT_HEADER = core::ptr::null_mut();
    let mut InfoLength: ULONG = 0;
    let mut Object: PVOID = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut ObjectQuota: POBJECT_HEADER_QUOTA_INFO;
    let mut SecurityInformation: SECURITY_INFORMATION;
    let mut ObjectType: POBJECT_TYPE = core::ptr::null_mut();
    let PreviousMode = ExGetPreviousMode();

    if PreviousMode != 0 {
        if let Err(s) = probe_user_buffer(ObjectInformation as _, Length as usize, core::mem::size_of::<UCHAR>(), PreviousMode) {
            return s;
        }
        if !ResultLength.is_null() {
            if let Err(s) = probe_write_ulong(ResultLength, PreviousMode) {
                return s;
            }
        }
    }

    if ObjectInformationClass != _OBJECT_INFORMATION_CLASS_ObjectTypesInformation {
        Status = ObReferenceObjectByHandle(
            ObjectHandle,
            0,
            core::ptr::null_mut(),
            KeGetPreviousMode(),
            core::ptr::addr_of_mut!(Object),
            core::ptr::addr_of_mut!(HandleInfo),
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }

        ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
        ObjectType = (*ObjectHeader).Type;
    }

    match ObjectInformationClass {
        _OBJECT_INFORMATION_CLASS_ObjectBasicInformation => {
            InfoLength = core::mem::size_of::<_OBJECT_BASIC_INFORMATION>() as ULONG;
            if Length != core::mem::size_of::<_OBJECT_BASIC_INFORMATION>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                let BasicInfo = ObjectInformation as POBJECT_BASIC_INFORMATION;
                (*BasicInfo).Attributes = HandleInfo.HandleAttributes;
                (*BasicInfo).GrantedAccess = HandleInfo.GrantedAccess;
                (*BasicInfo).HandleCount = (*ObjectHeader).__bindgen_anon_1.HandleCount as ULONG;
                (*BasicInfo).PointerCount = (*ObjectHeader).PointerCount as ULONG;

                if ((*ObjectHeader).Flags & OB_FLAG_EXCLUSIVE) != 0 {
                    (*BasicInfo).Attributes |= OBJ_EXCLUSIVE;
                }
                if ((*ObjectHeader).Flags & OB_FLAG_PERMANENT) != 0 {
                    (*BasicInfo).Attributes |= OBJ_PERMANENT;
                }

                ObjectQuota = OBJECT_HEADER_TO_QUOTA_INFO(ObjectHeader);
                if !ObjectQuota.is_null() {
                    (*BasicInfo).PagedPoolCharge = (*ObjectQuota).PagedPoolCharge;
                    (*BasicInfo).NonPagedPoolCharge = (*ObjectQuota).NonPagedPoolCharge;
                } else {
                    (*BasicInfo).PagedPoolCharge = 0;
                    (*BasicInfo).NonPagedPoolCharge = 0;
                }

                (*BasicInfo).NameInfoSize = ObpQueryNameInfoSize(ObjectHeader);
                (*BasicInfo).TypeInfoSize = core::mem::size_of::<_OBJECT_TYPE_INFORMATION>() as ULONG
                    + (*ObjectType).Name.Length as ULONG
                    + core::mem::size_of::<u16>() as ULONG;

                if (*ObjectHeader).Type == ObpSymbolicLinkObjectType {
                    (*BasicInfo).CreationTime.QuadPart = (*(Object as POBJECT_SYMBOLIC_LINK)).CreationTime.QuadPart;
                } else {
                    (*BasicInfo).CreationTime.QuadPart = 0;
                }

                (*BasicInfo).SecurityDescriptorSize = 0;
                if ((HandleInfo.GrantedAccess & READ_CONTROL) != 0) && !(*ObjectHeader).SecurityDescriptor.is_null() {
                    SecurityInformation = OWNER_SECURITY_INFORMATION
                        | GROUP_SECURITY_INFORMATION
                        | DACL_SECURITY_INFORMATION
                        | SACL_SECURITY_INFORMATION;

                    crate::obsecure::ObpCallSecurityProcedure(
            (*ObjectType).TypeInfo.SecurityProcedure,
                        Object,
                        _SECURITY_OPERATION_CODE_QuerySecurityDescriptor,
                        core::ptr::addr_of_mut!(SecurityInformation),
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!((*BasicInfo).SecurityDescriptorSize),
                        core::ptr::addr_of_mut!((*ObjectHeader).SecurityDescriptor),
                        (*ObjectType).TypeInfo.PoolType,
                        core::ptr::addr_of_mut!((*ObjectType).TypeInfo.GenericMapping),
                    );
                }

                Status = STATUS_SUCCESS;
            }
        }
        _OBJECT_INFORMATION_CLASS_ObjectNameInformation => {
            Status = ObQueryNameString(
                Object,
                ObjectInformation as POBJECT_NAME_INFORMATION,
                Length,
                core::ptr::addr_of_mut!(InfoLength),
            );
        }
        _OBJECT_INFORMATION_CLASS_ObjectTypeInformation => {
            Status = ObQueryTypeInfo(
                (*ObjectHeader).Type,
                ObjectInformation as POBJECT_TYPE_INFORMATION,
                Length,
                core::ptr::addr_of_mut!(InfoLength),
            );
        }
        _OBJECT_INFORMATION_CLASS_ObjectTypesInformation => {
            InfoLength = Length;
            Status = STATUS_NOT_IMPLEMENTED;
        }
        _OBJECT_INFORMATION_CLASS_ObjectHandleFlagInformation => {
            InfoLength = core::mem::size_of::<_OBJECT_HANDLE_FLAG_INFORMATION>() as ULONG;
            if Length != core::mem::size_of::<_OBJECT_HANDLE_FLAG_INFORMATION>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                let HandleFlags = ObjectInformation as POBJECT_HANDLE_FLAG_INFORMATION;

                (*HandleFlags).Inherit = if (HandleInfo.HandleAttributes & OBJ_INHERIT) != 0 { TRUE } else { FALSE };
                (*HandleFlags).ProtectFromClose = if (HandleInfo.HandleAttributes & OBJ_PROTECT_CLOSE) != 0 { TRUE } else { FALSE };

                Status = STATUS_SUCCESS;
            }
        }
        _ => {
            InfoLength = Length;
            Status = STATUS_INVALID_INFO_CLASS;
        }
    }

    if !ResultLength.is_null() {
        *ResultLength = InfoLength;
    }

    if !Object.is_null() {
        ObDereferenceObject(Object);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetInformationObject(
    ObjectHandle: HANDLE,
    ObjectInformationClass: OBJECT_INFORMATION_CLASS,
    ObjectInformation: PVOID,
    Length: ULONG,
) -> NTSTATUS {
    let ObjectHandle = opaque_ptr(ObjectHandle);
    let ObjectInformation = opaque_ptr(ObjectInformation);
    let mut Status: NTSTATUS;
    let mut Context: OBP_SET_HANDLE_ATTRIBUTES_CONTEXT = core::mem::zeroed();
    let mut ApcState: _KAPC_STATE = core::mem::zeroed();
    let mut Directory: POBJECT_DIRECTORY = core::ptr::null_mut();
    let mut PreviousMode: KPROCESSOR_MODE = 0;
    let mut AttachedToProcess = 0u8;

    match ObjectInformationClass {
        _OBJECT_INFORMATION_CLASS_ObjectHandleFlagInformation => {
            if Length != core::mem::size_of::<_OBJECT_HANDLE_FLAG_INFORMATION>() as ULONG {
                return STATUS_INFO_LENGTH_MISMATCH;
            }

            Context.PreviousMode = ExGetPreviousMode();

            if Context.PreviousMode != 0 {
                if let Err(s) = probe_user_buffer(
                    ObjectInformation as _,
                    core::mem::size_of::<_OBJECT_HANDLE_FLAG_INFORMATION>(),
                    core::mem::size_of::<BOOLEAN>(),
                    Context.PreviousMode,
                ) {
                    return s;
                }
                Context.Information = *(ObjectInformation as POBJECT_HANDLE_FLAG_INFORMATION);
            } else {
                Context.Information = *(ObjectInformation as POBJECT_HANDLE_FLAG_INFORMATION);
            }

            let mut handle = ObjectHandle;
            if ObpIsKernelHandle(ObjectHandle, Context.PreviousMode) != 0 {
                handle = ObKernelHandleToHandle(ObjectHandle);
                let ObjectTable = ObpKernelHandleTable;

                if PsGetCurrentProcess() as PVOID != PsInitialSystemProcess {
                    KeStackAttachProcess(
                        core::ptr::addr_of_mut!((*(PsInitialSystemProcess as *mut _EPROCESS)).Pcb),
                        core::ptr::addr_of_mut!(ApcState) as PKAPC_STATE,
                    );
                    AttachedToProcess = 1u8;
                }

                if ExChangeHandle(
                    ObjectTable,
                    handle,
                    Some(crate::obhandle::ObpSetHandleAttributes),
                    core::ptr::addr_of_mut!(Context) as ULONG_PTR as _,
                ) == 0
                {
                    Status = STATUS_ACCESS_DENIED;
                } else {
                    Status = STATUS_SUCCESS;
                }
            } else {
                let ObjectTable = (*(PsGetCurrentProcess() as *mut _EPROCESS)).ObjectTable;

                if ExChangeHandle(
                    ObjectTable,
                    handle,
                    Some(crate::obhandle::ObpSetHandleAttributes),
                    core::ptr::addr_of_mut!(Context) as ULONG_PTR as _,
                ) == 0
                {
                    Status = STATUS_ACCESS_DENIED;
                } else {
                    Status = STATUS_SUCCESS;
                }
            }

            if AttachedToProcess != 0 {
                KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState) as PKAPC_STATE);
            }
        }
        _OBJECT_INFORMATION_CLASS_ObjectSessionInformation => {
            PreviousMode = ExGetPreviousMode();
            if SeSinglePrivilegeCheck(SeTcbPrivilege, PreviousMode) == 0 {
                Status = STATUS_PRIVILEGE_NOT_HELD;
            } else {
                Status = ObReferenceObjectByHandle(
                    ObjectHandle,
                    0,
                    ObpDirectoryObjectType,
                    PreviousMode,
                    core::ptr::addr_of_mut!(Directory) as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if NT_SUCCESS(Status) {
                    let mut LookupContext: OBP_LOOKUP_CONTEXT = core::mem::zeroed();
                    ObpInitializeLookupContext(core::ptr::addr_of_mut!(LookupContext));

                    ObpAcquireDirectoryLockExclusive(Directory, core::ptr::addr_of_mut!(LookupContext));
                    (*Directory).SessionId = PsGetCurrentProcessSessionId();
                    ObpReleaseDirectoryLock(Directory, core::ptr::addr_of_mut!(LookupContext));

                    ObpReleaseLookupContext(core::ptr::addr_of_mut!(LookupContext));
                    ObDereferenceObject(Directory as PVOID);
                }
            }
        }
        _ => {
            Status = STATUS_INVALID_INFO_CLASS;
        }
    }

    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ob/oblife.c
//  * PURPOSE:         Manages the lifetime of an Object, including its creation,
//  *                  and deletion, as well as setting or querying any of its
//  *                  information while it is active. Since Object Types are also
//  *                  Objects, those are also managed here.
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
// extern ULONG NtGlobalFlag;
//
// POBJECT_TYPE ObpTypeObjectType = NULL;
// KEVENT ObpDefaultObject;
// KGUARDED_MUTEX ObpDeviceMapLock;
//
// GENERAL_LOOKASIDE ObpNameBufferLookasideList, ObpCreateInfoLookasideList;
//
// WORK_QUEUE_ITEM ObpReaperWorkItem;
// volatile PVOID ObpReaperList;
//
// ULONG ObpObjectsCreated, ObpObjectsWithName, ObpObjectsWithPoolQuota;
// ULONG ObpObjectsWithHandleDB, ObpObjectsWithCreatorInfo;
// POBJECT_TYPE ObpObjectTypes[32];
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// FASTCALL
// ObpDeallocateObject(IN PVOID Object)
// {
//     PVOID HeaderLocation;
//     POBJECT_HEADER Header;
//     POBJECT_TYPE ObjectType;
//     POBJECT_HEADER_HANDLE_INFO HandleInfo;
//     POBJECT_HEADER_NAME_INFO NameInfo;
//     POBJECT_HEADER_CREATOR_INFO CreatorInfo;
//     POBJECT_HEADER_QUOTA_INFO QuotaInfo;
//     ULONG PagedPoolCharge, NonPagedPoolCharge;
//     PAGED_CODE();
//
//     /* Get the header and assume this is what we'll free */
//     Header = OBJECT_TO_OBJECT_HEADER(Object);
//     ObjectType = Header->Type;
//     HeaderLocation = Header;
//
//     /* To find the header, walk backwards from how we allocated */
//     if ((CreatorInfo = OBJECT_HEADER_TO_CREATOR_INFO(Header)))
//     {
//         HeaderLocation = CreatorInfo;
//     }
//     if ((NameInfo = OBJECT_HEADER_TO_NAME_INFO(Header)))
//     {
//         HeaderLocation = NameInfo;
//     }
//     if ((HandleInfo = OBJECT_HEADER_TO_HANDLE_INFO(Header)))
//     {
//         HeaderLocation = HandleInfo;
//     }
//     if ((QuotaInfo = OBJECT_HEADER_TO_QUOTA_INFO(Header)))
//     {
//         HeaderLocation = QuotaInfo;
//     }
//
//     /* Decrease the total */
//     InterlockedDecrement((PLONG)&ObjectType->TotalNumberOfObjects);
//
//     /* Check if we have create info */
//     if (Header->Flags & OB_FLAG_CREATE_INFO)
//     {
//         /* Double-check that it exists */
//         if (Header->ObjectCreateInfo)
//         {
//             /* Free it */
//             ObpFreeObjectCreateInformation(Header->ObjectCreateInfo);
//             Header->ObjectCreateInfo = NULL;
//         }
//     }
//     else
//     {
//         /* Check if it has a quota block */
//         if (Header->QuotaBlockCharged)
//         {
//             /* Check if we have quota information */
//             if (QuotaInfo)
//             {
//                 /* Get charges from quota information */
//                 PagedPoolCharge = QuotaInfo->PagedPoolCharge +
//                                   QuotaInfo->SecurityDescriptorCharge;
//                 NonPagedPoolCharge = QuotaInfo->NonPagedPoolCharge;
//             }
//             else
//             {
//                 /* Get charges from object type */
//                 PagedPoolCharge = ObjectType->TypeInfo.DefaultPagedPoolCharge;
//                 NonPagedPoolCharge = ObjectType->
//                                      TypeInfo.DefaultNonPagedPoolCharge;
//
//                 /* Add the SD charge too */
//                 if (Header->Flags & OB_FLAG_SECURITY) PagedPoolCharge += 2048;
//             }
//
//             /* Return the quota */
//             if (Header->QuotaBlockCharged != OBP_SYSTEM_PROCESS_QUOTA)
//             {
//                 PsReturnSharedPoolQuota(Header->QuotaBlockCharged,
//                                         PagedPoolCharge,
//                                         NonPagedPoolCharge);
//             }
//         }
//     }
//
//     /* Check if a handle database was active */
//     if ((HandleInfo) && !(Header->Flags & OB_FLAG_SINGLE_PROCESS))
//     {
//         /* Free it */
//         ExFreePool(HandleInfo->HandleCountDatabase);
//         HandleInfo->HandleCountDatabase = NULL;
//     }
//
//     /* Check if we have a name */
//     if ((NameInfo) && (NameInfo->Name.Buffer))
//     {
//         /* Free it */
//         ExFreePool(NameInfo->Name.Buffer);
//         NameInfo->Name.Buffer = NULL;
//     }
//
//     /* Catch invalid access */
//     Header->Type = (POBJECT_TYPE)(ULONG_PTR)0xBAADB0B0BAADB0B0ULL;
//
//     /* Free the object using the same allocation tag */
//     ExFreePoolWithTag(HeaderLocation, ObjectType->Key);
// }
//
// VOID
// NTAPI
// ObpDeleteObject(IN PVOID Object,
//                 IN BOOLEAN CalledFromWorkerThread)
// {
//     POBJECT_HEADER Header;
//     POBJECT_TYPE ObjectType;
//     POBJECT_HEADER_NAME_INFO NameInfo;
//     POBJECT_HEADER_CREATOR_INFO CreatorInfo;
//     KIRQL CalloutIrql;
//     PAGED_CODE();
//
//     /* Get the header and type */
//     Header = OBJECT_TO_OBJECT_HEADER(Object);
//     ObjectType = Header->Type;
//
//     /* Get creator and name information */
//     NameInfo = OBJECT_HEADER_TO_NAME_INFO(Header);
//     CreatorInfo = OBJECT_HEADER_TO_CREATOR_INFO(Header);
//
//     /* Check if the object is on a type list */
//     if ((CreatorInfo) && !(IsListEmpty(&CreatorInfo->TypeList)))
//     {
//         /* Lock the object type */
//         ObpEnterObjectTypeMutex(ObjectType);
//
//         /* Remove the object from the type list */
//         RemoveEntryList(&CreatorInfo->TypeList);
//
//         /* Release the lock */
//         ObpLeaveObjectTypeMutex(ObjectType);
//     }
//
//     /* Check if we have a name */
//     if ((NameInfo) && (NameInfo->Name.Buffer))
//     {
//         /* Free it */
//         ExFreePool(NameInfo->Name.Buffer);
//         RtlInitEmptyUnicodeString(&NameInfo->Name, NULL, 0);
//     }
//
//     /* Check if we have a security descriptor */
//     if (Header->SecurityDescriptor)
//     {
//         /* Call the security procedure to delete it */
//         ObpCalloutStart(&CalloutIrql);
//         ObjectType->TypeInfo.SecurityProcedure(Object,
//                                                DeleteSecurityDescriptor,
//                                                0,
//                                                NULL,
//                                                NULL,
//                                                &Header->SecurityDescriptor,
//                                                0,
//                                                NULL);
//         ObpCalloutEnd(CalloutIrql, "Security", ObjectType, Object);
//     }
//
//     /* Check if we have a delete procedure */
//     if (ObjectType->TypeInfo.DeleteProcedure)
//     {
//         /* Save whether we were deleted from worker thread or not */
//         if (!CalledFromWorkerThread) Header->Flags |= OB_FLAG_DEFER_DELETE;
//
//         /* Call it */
//         ObpCalloutStart(&CalloutIrql);
//         ObjectType->TypeInfo.DeleteProcedure(Object);
//         ObpCalloutEnd(CalloutIrql, "Delete", ObjectType, Object);
//     }
//
//     /* Now de-allocate all object members */
//     ObpDeallocateObject(Object);
// }
//
// VOID
// NTAPI
// ObpReapObject(IN PVOID Parameter)
// {
//     POBJECT_HEADER ReapObject, NextObject;
//
//     /* Start reaping */
//     do
//     {
//         /* Get the reap object */
//         ReapObject = InterlockedExchangePointer(&ObpReaperList, (PVOID)1);
//
//         /* Start deletion loop */
//         do
//         {
//             /* Get the next object */
//             NextObject = ReapObject->NextToFree;
//
//             /* Delete the object */
//             ObpDeleteObject(&ReapObject->Body, TRUE);
//
//             /* Move to the next one */
//             ReapObject = NextObject;
//         } while ((ReapObject) && (ReapObject != (PVOID)1));
//     } while ((ObpReaperList != (PVOID)1) ||
//              (InterlockedCompareExchange((PLONG)&ObpReaperList, 0, 1) != 1));
// }
//
// /*++
// * @name ObpSetPermanentObject
// *
// *     The ObpSetPermanentObject routine makes an sets or clears the permanent
// *     flag of an object, thus making it either permanent or temporary.
// *
// * @param ObjectBody
// *        Pointer to the object to make permanent or temporary.
// *
// * @param Permanent
// *        Flag specifying which operation to perform.
// *
// * @return None.
// *
// * @remarks If the object is being made temporary, then it will be checked
// *          as a candidate for immediate removal from the namespace.
// *
// *--*/
// VOID
// FASTCALL
// ObpSetPermanentObject(IN PVOID ObjectBody,
//                       IN BOOLEAN Permanent)
// {
//     POBJECT_HEADER ObjectHeader;
//
//     /* Get the header */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(ObjectBody);
//
//     /* Acquire object lock */
//     ObpAcquireObjectLock(ObjectHeader);
//
//     /* Check what we're doing to it */
//     if (Permanent)
//     {
//         /* Set it to permanent */
//         ObjectHeader->Flags |= OB_FLAG_PERMANENT;
//
//         /* Release the lock */
//         ObpReleaseObjectLock(ObjectHeader);
//     }
//     else
//     {
//         /* Remove the flag */
//         ObjectHeader->Flags &= ~OB_FLAG_PERMANENT;
//
//         /* Release the lock */
//         ObpReleaseObjectLock(ObjectHeader);
//
//         /* Check if we should delete the object now */
//         ObpDeleteNameCheck(ObjectBody);
//     }
// }
//
// PWCHAR
// NTAPI
// ObpAllocateObjectNameBuffer(IN ULONG Length,
//                             IN BOOLEAN UseLookaside,
//                             IN OUT PUNICODE_STRING ObjectName)
// {
//     ULONG MaximumLength;
//     PVOID Buffer;
//
//     /* Set the maximum length to the length plus the terminator */
//     MaximumLength = Length + sizeof(UNICODE_NULL);
//
//     /* Check if we should use the lookaside buffer */
//     if (!(UseLookaside) || (MaximumLength > OBP_NAME_LOOKASIDE_MAX_SIZE))
//     {
//         /* Nope, allocate directly from pool */
//         /* Since we later use MaximumLength to detect that we're not allocating
//          * from a list, we need at least MaximumLength + sizeof(UNICODE_NULL)
//          * here.
//          *
//          * People do call this with UseLookasideList FALSE so the distinction
//          * is critical.
//          */
//         if (MaximumLength <= OBP_NAME_LOOKASIDE_MAX_SIZE)
//         {
//             MaximumLength = OBP_NAME_LOOKASIDE_MAX_SIZE + sizeof(UNICODE_NULL);
//         }
//         Buffer = ExAllocatePoolWithTag(PagedPool,
//                                        MaximumLength,
//                                        OB_NAME_TAG);
//     }
//     else
//     {
//         /* Allocate from the lookaside */
//         MaximumLength = OBP_NAME_LOOKASIDE_MAX_SIZE;
//         Buffer = ObpAllocateObjectCreateInfoBuffer(LookasideNameBufferList);
//     }
//
//     /* Setup the string */
//     ObjectName->MaximumLength = (USHORT)MaximumLength;
//     ObjectName->Length = (USHORT)Length;
//     ObjectName->Buffer = Buffer;
//     return Buffer;
// }
//
// VOID
// NTAPI
// ObpFreeObjectNameBuffer(IN PUNICODE_STRING Name)
// {
//     PVOID Buffer = Name->Buffer;
//
//     /* We know this is a pool-allocation if the size doesn't match */
//     if (Name->MaximumLength != OBP_NAME_LOOKASIDE_MAX_SIZE)
//     {
//         /*
//          * Free it from the pool.
//          *
//          * We cannot use here ExFreePoolWithTag(..., OB_NAME_TAG); , because
//          * the object name may have been massaged during operation by different
//          * object parse routines. If the latter ones have to resolve a symbolic
//          * link (e.g. as is done by CmpParseKey() and CmpGetSymbolicLink()),
//          * the original object name is freed and re-allocated from the pool,
//          * possibly with a different pool tag. At the end of the day, the new
//          * object name can be reallocated and completely different, but we
//          * should still be able to free it!
//          */
//         ExFreePool(Buffer);
//     }
//     else
//     {
//         /* Otherwise, free from the lookaside */
//         ObpFreeCapturedAttributes(Buffer, LookasideNameBufferList);
//     }
// }
//
// NTSTATUS
// NTAPI
// ObpCaptureObjectName(IN OUT PUNICODE_STRING CapturedName,
//                      IN PUNICODE_STRING ObjectName,
//                      IN KPROCESSOR_MODE AccessMode,
//                      IN BOOLEAN UseLookaside)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     ULONG StringLength;
//     PWCHAR _SEH2_VOLATILE StringBuffer = NULL;
//     UNICODE_STRING LocalName;
//     PAGED_CODE();
//
//     /* Initialize the Input String */
//     RtlInitEmptyUnicodeString(CapturedName, NULL, 0);
//
//     /* Protect everything */
//     _SEH2_TRY
//     {
//         /* Check if we came from user mode */
//         if (AccessMode != KernelMode)
//         {
//             /* First Probe the String */
//             LocalName = ProbeForReadUnicodeString(ObjectName);
//             ProbeForRead(LocalName.Buffer, LocalName.Length, sizeof(WCHAR));
//         }
//         else
//         {
//             /* No probing needed */
//             LocalName = *ObjectName;
//         }
//
//         /* Make sure there really is a string */
//         StringLength = LocalName.Length;
//         if (StringLength)
//         {
//             /* Check that the size is a valid WCHAR multiple */
//             if ((StringLength & (sizeof(WCHAR) - 1)) ||
//                 /* Check that the NULL-termination below will work */
//                 (StringLength == (MAXUSHORT - sizeof(UNICODE_NULL) + 1)))
//             {
//                 /* PS: Please keep the checks above expanded for clarity */
//                 Status = STATUS_OBJECT_NAME_INVALID;
//             }
//             else
//             {
//                 /* Allocate the string buffer */
//                 StringBuffer = ObpAllocateObjectNameBuffer(StringLength,
//                                                            UseLookaside,
//                                                            CapturedName);
//                 if (!StringBuffer)
//                 {
//                     /* Set failure code */
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                 }
//                 else
//                 {
//                     /* Copy the name */
//                     RtlCopyMemory(StringBuffer, LocalName.Buffer, StringLength);
//                     StringBuffer[StringLength / sizeof(WCHAR)] = UNICODE_NULL;
//                 }
//             }
//         }
//     }
//     _SEH2_EXCEPT(ExSystemExceptionFilter())
//     {
//         /* Handle exception and free the string buffer */
//         Status = _SEH2_GetExceptionCode();
//         if (StringBuffer)
//         {
//             ObpFreeObjectNameBuffer(CapturedName);
//         }
//     }
//     _SEH2_END;
//
//     /* Return */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// ObpCaptureObjectCreateInformation(IN POBJECT_ATTRIBUTES ObjectAttributes,
//                                   IN KPROCESSOR_MODE AccessMode,
//                                   IN KPROCESSOR_MODE CreatorMode,
//                                   IN BOOLEAN AllocateFromLookaside,
//                                   IN POBJECT_CREATE_INFORMATION ObjectCreateInfo,
//                                   OUT PUNICODE_STRING ObjectName)
// {
//     ULONG SdCharge, QuotaInfoSize;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PSECURITY_DESCRIPTOR SecurityDescriptor;
//     PSECURITY_QUALITY_OF_SERVICE SecurityQos;
//     PUNICODE_STRING LocalObjectName = NULL;
//     PAGED_CODE();
//
//     /* Zero out the Capture Data */
//     RtlZeroMemory(ObjectCreateInfo, sizeof(OBJECT_CREATE_INFORMATION));
//
//     /* SEH everything here for protection */
//     _SEH2_TRY
//     {
//         /* Check if we got attributes */
//         if (ObjectAttributes)
//         {
//             /* Check if we're in user mode */
//             if (AccessMode != KernelMode)
//             {
//                 /* Probe the attributes */
//                 ProbeForRead(ObjectAttributes,
//                              sizeof(OBJECT_ATTRIBUTES),
//                              sizeof(ULONG));
//             }
//
//             /* Validate the Size and Attributes */
//             if ((ObjectAttributes->Length != sizeof(OBJECT_ATTRIBUTES)) ||
//                 (ObjectAttributes->Attributes & ~OBJ_VALID_KERNEL_ATTRIBUTES))
//             {
//                 /* Invalid combination, fail */
//                 _SEH2_YIELD(return STATUS_INVALID_PARAMETER);
//             }
//
//             /* Set some Create Info and do not allow user-mode kernel handles */
//             ObjectCreateInfo->RootDirectory = ObjectAttributes->RootDirectory;
//             ObjectCreateInfo->Attributes = ObjectAttributes->Attributes & OBJ_VALID_KERNEL_ATTRIBUTES;
//             if (CreatorMode != KernelMode) ObjectCreateInfo->Attributes &= ~OBJ_KERNEL_HANDLE;
//             LocalObjectName = ObjectAttributes->ObjectName;
//             SecurityDescriptor = ObjectAttributes->SecurityDescriptor;
//             SecurityQos = ObjectAttributes->SecurityQualityOfService;
//
//             /* Check if we have a security descriptor */
//             if (SecurityDescriptor)
//             {
//                 /* Capture it. Note: This has an implicit memory barrier due
//                    to the function call, so cleanup is safe here.) */
//                 Status = SeCaptureSecurityDescriptor(SecurityDescriptor,
//                                                      AccessMode,
//                                                      NonPagedPool,
//                                                      TRUE,
//                                                      &ObjectCreateInfo->
//                                                      SecurityDescriptor);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     /* Capture failed, quit */
//                     ObjectCreateInfo->SecurityDescriptor = NULL;
//                     _SEH2_YIELD(return Status);
//                 }
//
//                 /*
//                  * By default, assume a SD size of 1024 and allow twice its
//                  * size.
//                  * If SD size happen to be bigger than that, then allow it
//                  */
//                 SdCharge = 2048;
//                 SeComputeQuotaInformationSize(ObjectCreateInfo->SecurityDescriptor,
//                                               &QuotaInfoSize);
//                 if ((2 * QuotaInfoSize) > 2048)
//                 {
//                     SdCharge = 2 * QuotaInfoSize;
//                 }
//
//                 /* Save the probe mode and security descriptor size */
//                 ObjectCreateInfo->SecurityDescriptorCharge = SdCharge;
//                 ObjectCreateInfo->ProbeMode = AccessMode;
//             }
//
//             /* Check if we have QoS */
//             if (SecurityQos)
//             {
//                 /* Check if we came from user mode */
//                 if (AccessMode != KernelMode)
//                 {
//                     /* Validate the QoS */
//                     ProbeForRead(SecurityQos,
//                                  sizeof(SECURITY_QUALITY_OF_SERVICE),
//                                  sizeof(ULONG));
//                 }
//
//                 /* Save Info */
//                 ObjectCreateInfo->SecurityQualityOfService = *SecurityQos;
//                 ObjectCreateInfo->SecurityQos =
//                     &ObjectCreateInfo->SecurityQualityOfService;
//             }
//         }
//         else
//         {
//             /* We don't have a name */
//             LocalObjectName = NULL;
//         }
//     }
//     _SEH2_EXCEPT(ExSystemExceptionFilter())
//     {
//         /* Cleanup and return the exception code */
//         ObpReleaseObjectCreateInformation(ObjectCreateInfo);
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     /* Now check if the Object Attributes had an Object Name */
//     if (LocalObjectName)
//     {
//         Status = ObpCaptureObjectName(ObjectName,
//                                       LocalObjectName,
//                                       AccessMode,
//                                       AllocateFromLookaside);
//     }
//     else
//     {
//         /* Clear the string */
//         RtlInitEmptyUnicodeString(ObjectName, NULL, 0);
//
//         /* It cannot have specified a Root Directory */
//         if (ObjectCreateInfo->RootDirectory)
//         {
//             Status = STATUS_OBJECT_NAME_INVALID;
//         }
//     }
//
//     /* Cleanup if we failed */
//     if (!NT_SUCCESS(Status))
//     {
//         ObpReleaseObjectCreateInformation(ObjectCreateInfo);
//     }
//
//     /* Return status to caller */
//     return Status;
// }
//
// VOID
// NTAPI
// ObFreeObjectCreateInfoBuffer(IN POBJECT_CREATE_INFORMATION ObjectCreateInfo)
// {
//     /* Call the macro. We use this function to isolate Ob internals from Io */
//     ObpFreeCapturedAttributes(ObjectCreateInfo, LookasideCreateInfoList);
// }
//
// NTSTATUS
// NTAPI
// ObpAllocateObject(IN POBJECT_CREATE_INFORMATION ObjectCreateInfo,
//                   IN PUNICODE_STRING ObjectName,
//                   IN POBJECT_TYPE ObjectType,
//                   IN ULONG ObjectSize,
//                   IN KPROCESSOR_MODE PreviousMode,
//                   IN POBJECT_HEADER *ObjectHeader)
// {
//     POBJECT_HEADER Header;
//     ULONG QuotaSize, HandleSize, NameSize, CreatorSize;
//     POBJECT_HEADER_HANDLE_INFO HandleInfo;
//     POBJECT_HEADER_NAME_INFO NameInfo;
//     POBJECT_HEADER_CREATOR_INFO CreatorInfo;
//     POBJECT_HEADER_QUOTA_INFO QuotaInfo;
//     POOL_TYPE PoolType;
//     ULONG FinalSize;
//     ULONG Tag;
//     PAGED_CODE();
//
//     /* Accounting */
//     ObpObjectsCreated++;
//
//     /* Check if we don't have an Object Type yet */
//     if (!ObjectType)
//     {
//         /* Use default tag and non-paged pool */
//         PoolType = NonPagedPool;
//         Tag = TAG_OBJECT_TYPE;
//     }
//     else
//     {
//         /* Use the pool and tag given */
//         PoolType = ObjectType->TypeInfo.PoolType;
//         Tag = ObjectType->Key;
//     }
//
//     /* Check if we have no create information (ie: we're an object type) */
//     if (!ObjectCreateInfo)
//     {
//         /* Use defaults */
//         QuotaSize = HandleSize = 0;
//         NameSize = sizeof(OBJECT_HEADER_NAME_INFO);
//         CreatorSize = sizeof(OBJECT_HEADER_CREATOR_INFO);
//     }
//     else
//     {
//         /* Check if we have quota */
//         if ((((ObjectCreateInfo->PagedPoolCharge !=
//                ObjectType->TypeInfo.DefaultPagedPoolCharge) ||
//               (ObjectCreateInfo->NonPagedPoolCharge !=
//                ObjectType->TypeInfo.DefaultNonPagedPoolCharge) ||
//               (ObjectCreateInfo->SecurityDescriptorCharge > 2048)) &&
//              (PsGetCurrentProcess() != PsInitialSystemProcess)) ||
//             (ObjectCreateInfo->Attributes & OBJ_EXCLUSIVE))
//         {
//             /* Set quota size */
//             QuotaSize = sizeof(OBJECT_HEADER_QUOTA_INFO);
//             ObpObjectsWithPoolQuota++;
//         }
//         else
//         {
//             /* No Quota */
//             QuotaSize = 0;
//         }
//
//         /* Check if we have a handle database */
//         if (ObjectType->TypeInfo.MaintainHandleCount)
//         {
//             /* Set handle database size */
//             HandleSize = sizeof(OBJECT_HEADER_HANDLE_INFO);
//             ObpObjectsWithHandleDB++;
//         }
//         else
//         {
//             /* None */
//             HandleSize = 0;
//         }
//
//         /* Check if the Object has a name */
//         if (ObjectName->Buffer)
//         {
//             /* Set name size */
//             NameSize = sizeof(OBJECT_HEADER_NAME_INFO);
//             ObpObjectsWithName++;
//         }
//         else
//         {
//             /* No name */
//             NameSize = 0;
//         }
//
//         /* Check if the Object maintains type lists */
//         if (ObjectType->TypeInfo.MaintainTypeList)
//         {
//             /* Set owner/creator size */
//             CreatorSize = sizeof(OBJECT_HEADER_CREATOR_INFO);
//             ObpObjectsWithCreatorInfo++;
//         }
//         else
//         {
//             /* No info */
//             CreatorSize = 0;
//         }
//     }
//
//     /* Set final header size */
//     FinalSize = QuotaSize +
//                 HandleSize +
//                 NameSize +
//                 CreatorSize +
//                 FIELD_OFFSET(OBJECT_HEADER, Body);
//
//     /* Allocate memory for the Object and Header */
//     Header = ExAllocatePoolWithTag(PoolType, FinalSize + ObjectSize, Tag);
//     if (!Header) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Check if we have a quota header */
//     if (QuotaSize)
//     {
//         /* Initialize quota info */
//         QuotaInfo = (POBJECT_HEADER_QUOTA_INFO)Header;
//         QuotaInfo->PagedPoolCharge = ObjectCreateInfo->PagedPoolCharge;
//         QuotaInfo->NonPagedPoolCharge = ObjectCreateInfo->NonPagedPoolCharge;
//         QuotaInfo->SecurityDescriptorCharge = ObjectCreateInfo->SecurityDescriptorCharge;
//         QuotaInfo->ExclusiveProcess = NULL;
//         Header = (POBJECT_HEADER)(QuotaInfo + 1);
//     }
//
//     /* Check if we have a handle database header */
//     if (HandleSize)
//     {
//         /* Initialize Handle Info */
//         HandleInfo = (POBJECT_HEADER_HANDLE_INFO)Header;
//         HandleInfo->SingleEntry.HandleCount = 0;
//         Header = (POBJECT_HEADER)(HandleInfo + 1);
//     }
//
//     /* Check if we have a name header */
//     if (NameSize)
//     {
//         /* Initialize the Object Name Info */
//         NameInfo = (POBJECT_HEADER_NAME_INFO)Header;
//         NameInfo->Name = *ObjectName;
//         NameInfo->Directory = NULL;
//         NameInfo->QueryReferences = 1;
//
//         /* Check if this is a call with the special protection flag */
//         if ((PreviousMode == KernelMode) &&
//             (ObjectCreateInfo) &&
//             (ObjectCreateInfo->Attributes & OBJ_KERNEL_EXCLUSIVE))
//         {
//             /* Set flag which will make the object protected from user-mode */
//             NameInfo->QueryReferences |= OB_FLAG_KERNEL_EXCLUSIVE;
//         }
//
//         /* Set the header pointer */
//         Header = (POBJECT_HEADER)(NameInfo + 1);
//     }
//
//     /* Check if we have a creator header */
//     if (CreatorSize)
//     {
//         /* Initialize Creator Info */
//         CreatorInfo = (POBJECT_HEADER_CREATOR_INFO)Header;
//         CreatorInfo->CreatorBackTraceIndex = 0;
//         CreatorInfo->CreatorUniqueProcess = PsGetCurrentProcessId();
//         InitializeListHead(&CreatorInfo->TypeList);
//         Header = (POBJECT_HEADER)(CreatorInfo + 1);
//     }
//
//     /* Check for quota information */
//     if (QuotaSize)
//     {
//         /* Set the offset */
//         Header->QuotaInfoOffset = (UCHAR)(QuotaSize +
//                                           HandleSize +
//                                           NameSize +
//                                           CreatorSize);
//     }
//     else
//     {
//         /* No offset */
//         Header->QuotaInfoOffset = 0;
//     }
//
//     /* Check for handle information */
//     if (HandleSize)
//     {
//         /* Set the offset */
//         Header->HandleInfoOffset = (UCHAR)(HandleSize +
//                                            NameSize +
//                                            CreatorSize);
//     }
//     else
//     {
//         /* No offset */
//         Header->HandleInfoOffset = 0;
//     }
//
//     /* Check for name information */
//     if (NameSize)
//     {
//         /* Set the offset */
//         Header->NameInfoOffset = (UCHAR)(NameSize + CreatorSize);
//     }
//     else
//     {
//         /* No Name */
//         Header->NameInfoOffset = 0;
//     }
//
//     /* Set the new object flag */
//     Header->Flags = OB_FLAG_CREATE_INFO;
//
//     /* Remember if we have creator info */
//     if (CreatorSize) Header->Flags |= OB_FLAG_CREATOR_INFO;
//
//     /* Remember if we have handle info */
//     if (HandleSize) Header->Flags |= OB_FLAG_SINGLE_PROCESS;
//
//     /* Initialize the object header */
//     Header->PointerCount = 1;
//     Header->HandleCount = 0;
//     Header->Type = ObjectType;
//     Header->ObjectCreateInfo = ObjectCreateInfo;
//     Header->SecurityDescriptor = NULL;
//
//     /* Check if this is a permanent object */
//     if ((ObjectCreateInfo) && (ObjectCreateInfo->Attributes & OBJ_PERMANENT))
//     {
//         /* Set the needed flag so we can check */
//         Header->Flags |= OB_FLAG_PERMANENT;
//     }
//
//     /* Check if this is an exclusive object */
//     if ((ObjectCreateInfo) && (ObjectCreateInfo->Attributes & OBJ_EXCLUSIVE))
//     {
//         /* Set the needed flag so we can check */
//         Header->Flags |= OB_FLAG_EXCLUSIVE;
//     }
//
//     /* Set kernel-mode flag */
//     if (PreviousMode == KernelMode) Header->Flags |= OB_FLAG_KERNEL_MODE;
//
//     /* Check if we have a type */
//     if (ObjectType)
//     {
//         /* Increase the number of objects of this type */
//         InterlockedIncrement((PLONG)&ObjectType->TotalNumberOfObjects);
//
//         /* Update the high water */
//         ObjectType->HighWaterNumberOfObjects = max(ObjectType->
//                                                    TotalNumberOfObjects,
//                                                    ObjectType->
//                                                    HighWaterNumberOfObjects);
//     }
//
//     /* Return Header */
//     *ObjectHeader = Header;
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Queries the name info size of a given resource object.
//  * The function loops through all the parent directories
//  * of the object and computes the name size.
//  *
//  * @param[in] ObjectHeader
//  * A pointer to an object header, of which name and
//  * directory info are to be retrieved.
//  *
//  * @return
//  * Returns the name info size that is pointed by the
//  * given object by the caller of this function. If
//  * an object does not have a name or no directories,
//  * it returns 0.
//  */
// static
// ULONG
// ObpQueryNameInfoSize(
//     _In_ POBJECT_HEADER ObjectHeader)
// {
//     ULONG NameSize = 0;
//     POBJECT_DIRECTORY ParentDirectory;
//     POBJECT_HEADER_NAME_INFO NameInfo;
//     PAGED_CODE();
//
//     /* Get the name info */
//     NameInfo = OBJECT_HEADER_TO_NAME_INFO(ObjectHeader);
//     if (!NameInfo)
//     {
//         return 0;
//     }
//
//     /* Get the parent directory from the object name too */
//     ParentDirectory = NameInfo->Directory;
//     if (!ParentDirectory)
//     {
//         return 0;
//     }
//
//     /* Take into account the name size of this object and loop for all parent directories */
//     NameSize = sizeof(OBJ_NAME_PATH_SEPARATOR) + NameInfo->Name.Length;
//     for (;;)
//     {
//         /* Get the name info from the parent directory */
//         NameInfo = OBJECT_HEADER_TO_NAME_INFO(
//             OBJECT_TO_OBJECT_HEADER(ParentDirectory));
//         if (!NameInfo)
//         {
//             /* Stop looking if this is the last one */
//             break;
//         }
//
//         /* Get the parent directory */
//         ParentDirectory = NameInfo->Directory;
//         if (!ParentDirectory)
//         {
//             /* This is the last directory, stop looking */
//             break;
//         }
//
//         /*
//          * Take into account the size of this name info,
//          * keep looking for other parent directories.
//          */
//         NameSize += sizeof(OBJ_NAME_PATH_SEPARATOR) + NameInfo->Name.Length;
//     }
//
//     /* Include the size of the object name information as well as the NULL terminator */
//     NameSize += sizeof(OBJECT_NAME_INFORMATION) + sizeof(UNICODE_NULL);
//     return NameSize;
// }
//
// NTSTATUS
// NTAPI
// ObQueryTypeInfo(
//     _In_ POBJECT_TYPE ObjectType,
//     _Out_writes_bytes_to_(Length, *ReturnLength)
//         POBJECT_TYPE_INFORMATION ObjectTypeInfo,
//     _In_ ULONG Length,
//     _Out_ PULONG ReturnLength)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PWSTR InfoBuffer;
//
//     /* The string of the object type name has to be NULL-terminated */
//     ASSERT(ObjectType->Name.MaximumLength >= ObjectType->Name.Length + sizeof(UNICODE_NULL));
//
//     /* Enter SEH */
//     _SEH2_TRY
//     {
//         /*
//          * Set return length aligned to 4-byte or 8-byte boundary. Windows has a bug
//          * where the returned length pointer is always aligned to a 4-byte boundary.
//          * If one were to allocate a pool of memory in kernel mode to retrieve all
//          * the object types info with this return length, Windows will bugcheck with
//          * BAD_POOL_HEADER in 64-bit upon you free the said allocated memory.
//          *
//          * More than that, Windows uses MaximumLength for the calculation of the returned
//          * length and MaximumLength does not always guarantee the name type is NULL-terminated
//          * leading the ObQueryTypeInfo function to overrun the buffer.
//          */
//         *ReturnLength += sizeof(*ObjectTypeInfo) +
//                          ALIGN_UP(ObjectType->Name.Length + sizeof(UNICODE_NULL), ULONG_PTR);
//
//         /* Check if that is too much */
//         if (Length < *ReturnLength)
//         {
//             _SEH2_YIELD(return STATUS_INFO_LENGTH_MISMATCH);
//         }
//
//         /* Build the data */
//         ObjectTypeInfo->TotalNumberOfHandles =
//             ObjectType->TotalNumberOfHandles;
//         ObjectTypeInfo->TotalNumberOfObjects =
//             ObjectType->TotalNumberOfObjects;
//         ObjectTypeInfo->HighWaterNumberOfHandles =
//             ObjectType->HighWaterNumberOfHandles;
//         ObjectTypeInfo->HighWaterNumberOfObjects =
//             ObjectType->HighWaterNumberOfObjects;
//         ObjectTypeInfo->PoolType =
//             ObjectType->TypeInfo.PoolType;
//         ObjectTypeInfo->DefaultNonPagedPoolCharge =
//             ObjectType->TypeInfo.DefaultNonPagedPoolCharge;
//         ObjectTypeInfo->DefaultPagedPoolCharge =
//             ObjectType->TypeInfo.DefaultPagedPoolCharge;
//         ObjectTypeInfo->ValidAccessMask =
//             ObjectType->TypeInfo.ValidAccessMask;
//         ObjectTypeInfo->SecurityRequired =
//             ObjectType->TypeInfo.SecurityRequired;
//         ObjectTypeInfo->InvalidAttributes =
//             ObjectType->TypeInfo.InvalidAttributes;
//         ObjectTypeInfo->GenericMapping =
//             ObjectType->TypeInfo.GenericMapping;
//         ObjectTypeInfo->MaintainHandleCount =
//             ObjectType->TypeInfo.MaintainHandleCount;
//
//         /* Setup the name buffer */
//         InfoBuffer = (PWSTR)(ObjectTypeInfo + 1);
//         ObjectTypeInfo->TypeName.Buffer = InfoBuffer;
//         ObjectTypeInfo->TypeName.MaximumLength = ObjectType->Name.MaximumLength;
//         ObjectTypeInfo->TypeName.Length = ObjectType->Name.Length;
//
//         /* Copy it */
//         RtlCopyMemory(InfoBuffer,
//                       ObjectType->Name.Buffer,
//                       ObjectType->Name.Length);
//
//         /* Null-terminate it */
//         (InfoBuffer)[ObjectType->Name.Length / sizeof(WCHAR)] = UNICODE_NULL;
//     }
//     _SEH2_EXCEPT(ExSystemExceptionFilter())
//     {
//         /* Otherwise, get the exception code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Return status to caller */
//     return Status;
// }
//
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// NTSTATUS
// NTAPI
// ObCreateObject(IN KPROCESSOR_MODE ProbeMode OPTIONAL,
//                IN POBJECT_TYPE Type,
//                IN POBJECT_ATTRIBUTES ObjectAttributes OPTIONAL,
//                IN KPROCESSOR_MODE AccessMode,
//                IN OUT PVOID ParseContext OPTIONAL,
//                IN ULONG ObjectSize,
//                IN ULONG PagedPoolCharge OPTIONAL,
//                IN ULONG NonPagedPoolCharge OPTIONAL,
//                OUT PVOID *Object)
// {
//     NTSTATUS Status;
//     POBJECT_CREATE_INFORMATION ObjectCreateInfo;
//     UNICODE_STRING ObjectName;
//     POBJECT_HEADER Header;
//
//     /* Allocate a capture buffer */
//     ObjectCreateInfo = ObpAllocateObjectCreateInfoBuffer(LookasideCreateInfoList);
//     if (!ObjectCreateInfo) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Capture all the info */
//     Status = ObpCaptureObjectCreateInformation(ObjectAttributes,
//                                                ProbeMode,
//                                                AccessMode,
//                                                FALSE,
//                                                ObjectCreateInfo,
//                                                &ObjectName);
//     if (NT_SUCCESS(Status))
//     {
//         /* Validate attributes */
//         if (Type->TypeInfo.InvalidAttributes & ObjectCreateInfo->Attributes)
//         {
//             /* Fail */
//             Status = STATUS_INVALID_PARAMETER;
//         }
//         else
//         {
//             /* Check if we have a paged charge */
//             if (!PagedPoolCharge)
//             {
//                 /* Save it */
//                 PagedPoolCharge = Type->TypeInfo.DefaultPagedPoolCharge;
//             }
//
//             /* Check for nonpaged charge */
//             if (!NonPagedPoolCharge)
//             {
//                 /* Save it */
//                 NonPagedPoolCharge = Type->TypeInfo.DefaultNonPagedPoolCharge;
//             }
//
//             /* Write the pool charges */
//             ObjectCreateInfo->PagedPoolCharge = PagedPoolCharge;
//             ObjectCreateInfo->NonPagedPoolCharge = NonPagedPoolCharge;
//
//             /* Allocate the Object */
//             Status = ObpAllocateObject(ObjectCreateInfo,
//                                        &ObjectName,
//                                        Type,
//                                        ObjectSize,
//                                        AccessMode,
//                                        &Header);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Return the Object */
//                 *Object = &Header->Body;
//
//                 /* Check if this is a permanent object */
//                 if (Header->Flags & OB_FLAG_PERMANENT)
//                 {
//                     /* Do the privilege check */
//                     if (!SeSinglePrivilegeCheck(SeCreatePermanentPrivilege,
//                                                 ProbeMode))
//                     {
//                         /* Fail */
//                         ObpDeallocateObject(*Object);
//                         Status = STATUS_PRIVILEGE_NOT_HELD;
//                     }
//                 }
//
//                 /* Return status */
//                 return Status;
//             }
//         }
//
//         /* Release the Capture Info, we don't need it */
//         ObpFreeObjectCreateInformation(ObjectCreateInfo);
//         if (ObjectName.Buffer) ObpFreeObjectNameBuffer(&ObjectName);
//         return Status;
//     }
//
//     /* We failed, so release the Buffer */
//     ObpFreeCapturedAttributes(ObjectCreateInfo, LookasideCreateInfoList);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// ObCreateObjectType(IN PUNICODE_STRING TypeName,
//                    IN POBJECT_TYPE_INITIALIZER ObjectTypeInitializer,
//                    IN PVOID Reserved,
//                    OUT POBJECT_TYPE *ObjectType)
// {
//     POBJECT_HEADER Header;
//     POBJECT_TYPE LocalObjectType;
//     ULONG HeaderSize;
//     NTSTATUS Status;
//     OBP_LOOKUP_CONTEXT Context;
//     PWCHAR p;
//     ULONG i;
//     UNICODE_STRING ObjectName;
//     ANSI_STRING AnsiName;
//     POBJECT_HEADER_CREATOR_INFO CreatorInfo;
//
//     /* Verify parameters */
//     if (!(TypeName) ||
//         !(TypeName->Length) ||
//         (TypeName->Length % sizeof(WCHAR)) ||
//         !(ObjectTypeInitializer) ||
//         (ObjectTypeInitializer->Length != sizeof(*ObjectTypeInitializer)) ||
//         (ObjectTypeInitializer->InvalidAttributes & ~OBJ_VALID_KERNEL_ATTRIBUTES) ||
//         (ObjectTypeInitializer->MaintainHandleCount &&
//          (!(ObjectTypeInitializer->OpenProcedure) &&
//           !ObjectTypeInitializer->CloseProcedure)) ||
//         ((!ObjectTypeInitializer->UseDefaultObject) &&
//          (ObjectTypeInitializer->PoolType != NonPagedPool)))
//     {
//         /* Fail */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Make sure the name doesn't have a separator */
//     p = TypeName->Buffer;
//     i = TypeName->Length / sizeof(WCHAR);
//     while (i--)
//     {
//         /* Check for one and fail */
//         if (*p++ == OBJ_NAME_PATH_SEPARATOR) return STATUS_OBJECT_NAME_INVALID;
//     }
//
//     /* Setup a lookup context */
//     ObpInitializeLookupContext(&Context);
//
//     /* Check if we've already created the directory of types */
//     if (ObpTypeDirectoryObject)
//     {
//         /* Lock the lookup context */
//         ObpAcquireLookupContextLock(&Context, ObpTypeDirectoryObject);
//
//         /* Do the lookup */
//         if (ObpLookupEntryDirectory(ObpTypeDirectoryObject,
//                                     TypeName,
//                                     OBJ_CASE_INSENSITIVE,
//                                     FALSE,
//                                     &Context))
//         {
//             /* We have already created it, so fail */
//             ObpReleaseLookupContext(&Context);
//             return STATUS_OBJECT_NAME_COLLISION;
//         }
//     }
//
//     /* Now make a copy of the object name */
//     ObjectName.Buffer = ExAllocatePoolWithTag(PagedPool,
//                                               TypeName->MaximumLength,
//                                               OB_NAME_TAG);
//     if (!ObjectName.Buffer)
//     {
//         /* Out of memory, fail */
//         ObpReleaseLookupContext(&Context);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Set the length and copy the name */
//     ObjectName.MaximumLength = TypeName->MaximumLength;
//     RtlCopyUnicodeString(&ObjectName, TypeName);
//
//     /* Allocate the Object */
//     Status = ObpAllocateObject(NULL,
//                                &ObjectName,
//                                ObpTypeObjectType,
//                                sizeof(OBJECT_TYPE),
//                                KernelMode,
//                                &Header);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Free the name and fail */
//         ObpReleaseLookupContext(&Context);
//         ExFreePool(ObjectName.Buffer);
//         return Status;
//     }
//
//     /* Setup the flags and name */
//     LocalObjectType = (POBJECT_TYPE)&Header->Body;
//     LocalObjectType->Name = ObjectName;
//     Header->Flags |= OB_FLAG_KERNEL_MODE | OB_FLAG_PERMANENT;
//
//     /* Clear accounting data */
//     LocalObjectType->TotalNumberOfObjects =
//     LocalObjectType->TotalNumberOfHandles =
//     LocalObjectType->HighWaterNumberOfObjects =
//     LocalObjectType->HighWaterNumberOfHandles = 0;
//
//     /* Check if this is the first Object Type */
//     if (!ObpTypeObjectType)
//     {
//         /* It is, so set this as the type object */
//         ObpTypeObjectType = LocalObjectType;
//         Header->Type = ObpTypeObjectType;
//
//         /* Set the hard-coded key and object count */
//         LocalObjectType->TotalNumberOfObjects = 1;
//         LocalObjectType->Key = TAG_OBJECT_TYPE;
//     }
//     else
//     {
//         /* Convert the tag to ASCII */
//         Status = RtlUnicodeStringToAnsiString(&AnsiName, TypeName, TRUE);
//         if (NT_SUCCESS(Status))
//         {
//             /* For every missing character, use a space */
//             for (i = 3; i >= AnsiName.Length; i--) AnsiName.Buffer[i] = ' ';
//
//             /* Set the key and free the converted name */
//             LocalObjectType->Key = *(PULONG)AnsiName.Buffer;
//             RtlFreeAnsiString(&AnsiName);
//         }
//         else
//         {
//             /* Just copy the characters */
//             LocalObjectType->Key = *(PULONG)TypeName->Buffer;
//         }
//     }
//
//     /* Set up the type information */
//     LocalObjectType->TypeInfo = *ObjectTypeInitializer;
//     LocalObjectType->TypeInfo.PoolType = ObjectTypeInitializer->PoolType;
//
//     /* Check if we have to maintain a type list */
//     if (NtGlobalFlag & FLG_MAINTAIN_OBJECT_TYPELIST)
//     {
//         /* Enable support */
//         LocalObjectType->TypeInfo.MaintainTypeList = TRUE;
//     }
//
//     /* Calculate how much space our header'll take up */
//     HeaderSize = sizeof(OBJECT_HEADER) +
//                  sizeof(OBJECT_HEADER_NAME_INFO) +
//                  (ObjectTypeInitializer->MaintainHandleCount ?
//                   sizeof(OBJECT_HEADER_HANDLE_INFO) : 0);
//
//     /* Check the pool type */
//     if (ObjectTypeInitializer->PoolType == NonPagedPool)
//     {
//         /* Update the NonPaged Pool charge */
//         LocalObjectType->TypeInfo.DefaultNonPagedPoolCharge += HeaderSize;
//     }
//     else
//     {
//         /* Update the Paged Pool charge */
//         LocalObjectType->TypeInfo.DefaultPagedPoolCharge += HeaderSize;
//     }
//
//     /* All objects types need a security procedure */
//     if (!ObjectTypeInitializer->SecurityProcedure)
//     {
//         LocalObjectType->TypeInfo.SecurityProcedure = SeDefaultObjectMethod;
//     }
//
//     /* Select the Wait Object */
//     if (LocalObjectType->TypeInfo.UseDefaultObject)
//     {
//         /* Add the SYNCHRONIZE access mask since it's waitable */
//         LocalObjectType->TypeInfo.ValidAccessMask |= SYNCHRONIZE;
//
//         /* Use the "Default Object", a simple event */
//         LocalObjectType->DefaultObject = &ObpDefaultObject;
//     }
//     /* The File Object gets an optimized hack so it can be waited on */
//     else if ((TypeName->Length == 8) && !(wcscmp(TypeName->Buffer, L"File")))
//     {
//         /* Wait on the File Object's event directly */
//         LocalObjectType->DefaultObject = UlongToPtr(FIELD_OFFSET(FILE_OBJECT,
//                                                                  Event));
//     }
//     else if ((TypeName->Length == 24) && !(wcscmp(TypeName->Buffer, L"WaitablePort")))
//     {
//         /* Wait on the LPC Port's object directly */
//         LocalObjectType->DefaultObject = UlongToPtr(FIELD_OFFSET(LPCP_PORT_OBJECT,
//                                                                  WaitEvent));
//     }
//     else
//     {
//         /* No default Object */
//         LocalObjectType->DefaultObject = NULL;
//     }
//
//     /* Initialize Object Type components */
//     ExInitializeResourceLite(&LocalObjectType->Mutex);
//     for (i = 0; i < 4; i++)
//     {
//         /* Initialize the object locks */
//         ExInitializeResourceLite(&LocalObjectType->ObjectLocks[i]);
//     }
//     InitializeListHead(&LocalObjectType->TypeList);
//
//     /* Lock the object type */
//     ObpEnterObjectTypeMutex(ObpTypeObjectType);
//
//     /* Get creator info and insert it into the type list */
//     CreatorInfo = OBJECT_HEADER_TO_CREATOR_INFO(Header);
//     if (CreatorInfo)
//     {
//         InsertTailList(&ObpTypeObjectType->TypeList,
//                        &CreatorInfo->TypeList);
//
//         /* CORE-8423: Avoid inserting this a second time if someone creates a
//          * handle to the object type (bug in Windows 2003) */
//         Header->Flags &= ~OB_FLAG_CREATE_INFO;
//     }
//
//     /* Set the index and the entry into the object type array */
//     LocalObjectType->Index = ObpTypeObjectType->TotalNumberOfObjects;
//
//     ASSERT(LocalObjectType->Index != 0);
//
//     if (LocalObjectType->Index < RTL_NUMBER_OF(ObpObjectTypes))
//     {
//         /* It fits, insert it */
//         ObpObjectTypes[LocalObjectType->Index - 1] = LocalObjectType;
//     }
//
//     /* Release the object type */
//     ObpLeaveObjectTypeMutex(ObpTypeObjectType);
//
//     /* Check if we're actually creating the directory object itself */
//     if (!(ObpTypeDirectoryObject) ||
//         (ObpInsertEntryDirectory(ObpTypeDirectoryObject, &Context, Header)))
//     {
//         /* Check if the type directory exists */
//         if (ObpTypeDirectoryObject)
//         {
//             /* Reference it */
//             ObReferenceObject(ObpTypeDirectoryObject);
//         }
//
//         /* Cleanup the lookup context */
//         ObpReleaseLookupContext(&Context);
//
//         /* Return the object type and success */
//         *ObjectType = LocalObjectType;
//         return STATUS_SUCCESS;
//     }
//
//     /* If we got here, then we failed */
//     ObpReleaseLookupContext(&Context);
//     return STATUS_INSUFFICIENT_RESOURCES;
// }
//
// VOID
// NTAPI
// ObDeleteCapturedInsertInfo(IN PVOID Object)
// {
//     POBJECT_HEADER ObjectHeader;
//     PAGED_CODE();
//
//     /* Check if there is anything to free */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
//     if ((ObjectHeader->Flags & OB_FLAG_CREATE_INFO) &&
//         (ObjectHeader->ObjectCreateInfo != NULL))
//     {
//         /* Free the create info */
//         ObpFreeObjectCreateInformation(ObjectHeader->ObjectCreateInfo);
//         ObjectHeader->ObjectCreateInfo = NULL;
//     }
// }
//
// VOID
// NTAPI
// ObpDeleteObjectType(IN PVOID Object)
// {
//     ULONG i;
//     POBJECT_TYPE ObjectType = (PVOID)Object;
//
//     /* Loop our locks */
//     for (i = 0; i < 4; i++)
//     {
//         /* Delete each one */
//         ExDeleteResourceLite(&ObjectType->ObjectLocks[i]);
//     }
//
//     /* Delete our main mutex */
//     ExDeleteResourceLite(&ObjectType->Mutex);
// }
//
// /*++
// * @name ObMakeTemporaryObject
// * @implemented NT4
// *
// *     The ObMakeTemporaryObject routine <FILLMEIN>
// *
// * @param ObjectBody
// *        <FILLMEIN>
// *
// * @return None.
// *
// * @remarks None.
// *
// *--*/
// VOID
// NTAPI
// ObMakeTemporaryObject(IN PVOID ObjectBody)
// {
//     PAGED_CODE();
//
//     /* Call the internal API */
//     ObpSetPermanentObject(ObjectBody, FALSE);
// }
//
// /*++
// * @name NtMakeTemporaryObject
// * @implemented NT4
// *
// *     The NtMakeTemporaryObject routine <FILLMEIN>
// *
// * @param ObjectHandle
// *        <FILLMEIN>
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// NtMakeTemporaryObject(IN HANDLE ObjectHandle)
// {
//     PVOID ObjectBody;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Reference the object for DELETE access */
//     Status = ObReferenceObjectByHandle(ObjectHandle,
//                                        DELETE,
//                                        NULL,
//                                        KeGetPreviousMode(),
//                                        &ObjectBody,
//                                        NULL);
//     if (Status != STATUS_SUCCESS) return Status;
//
//     /* Set it as temporary and dereference it */
//     ObpSetPermanentObject(ObjectBody, FALSE);
//     ObDereferenceObject(ObjectBody);
//     return STATUS_SUCCESS;
// }
//
// /*++
// * @name NtMakePermanentObject
// * @implemented NT4
// *
// *     The NtMakePermanentObject routine <FILLMEIN>
// *
// * @param ObjectHandle
// *        <FILLMEIN>
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// NtMakePermanentObject(IN HANDLE ObjectHandle)
// {
//     PVOID ObjectBody;
//     NTSTATUS Status;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     PAGED_CODE();
//
//     /* Make sure that the caller has SeCreatePermanentPrivilege */
//     if (!SeSinglePrivilegeCheck(SeCreatePermanentPrivilege, PreviousMode))
//     {
//         return STATUS_PRIVILEGE_NOT_HELD;
//     }
//
//     /* Reference the object */
//     Status = ObReferenceObjectByHandle(ObjectHandle,
//                                        0,
//                                        NULL,
//                                        PreviousMode,
//                                        &ObjectBody,
//                                        NULL);
//     if (Status != STATUS_SUCCESS) return Status;
//
//     /* Set it as permanent and dereference it */
//     ObpSetPermanentObject(ObjectBody, TRUE);
//     ObDereferenceObject(ObjectBody);
//     return STATUS_SUCCESS;
// }
//
// /*++
// * @name NtQueryObject
// * @implemented NT4
// *
// *     The NtQueryObject routine <FILLMEIN>
// *
// * @param ObjectHandle
// *        <FILLMEIN>
// *
// * @param ObjectInformationClass
// *        <FILLMEIN>
// *
// * @param ObjectInformation
// *        <FILLMEIN>
// *
// * @param Length
// *        <FILLMEIN>
// *
// * @param ResultLength
// *        <FILLMEIN>
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// NtQueryObject(IN HANDLE ObjectHandle,
//               IN OBJECT_INFORMATION_CLASS ObjectInformationClass,
//               OUT PVOID ObjectInformation,
//               IN ULONG Length,
//               OUT PULONG ResultLength OPTIONAL)
// {
//     OBJECT_HANDLE_INFORMATION HandleInfo;
//     POBJECT_HEADER ObjectHeader = NULL;
//     POBJECT_HANDLE_FLAG_INFORMATION HandleFlags;
//     POBJECT_BASIC_INFORMATION BasicInfo;
//     ULONG InfoLength = 0;
//     PVOID Object = NULL;
//     NTSTATUS Status;
//     POBJECT_HEADER_QUOTA_INFO ObjectQuota;
//     SECURITY_INFORMATION SecurityInformation;
//     POBJECT_TYPE ObjectType;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     PAGED_CODE();
//
//     /* Check if the caller is from user mode */
//     if (PreviousMode != KernelMode)
//     {
//         /* Protect validation with SEH */
//         _SEH2_TRY
//         {
//             /* Probe the input structure */
//             ProbeForWrite(ObjectInformation, Length, sizeof(UCHAR));
//
//             /* If we have a result length, probe it too */
//             if (ResultLength) ProbeForWriteUlong(ResultLength);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /*
//      * Make sure this isn't a generic type query, since the caller doesn't
//      * have to give a handle for it
//      */
//     if (ObjectInformationClass != ObjectTypesInformation)
//     {
//         /* Reference the object */
//         Status = ObReferenceObjectByHandle(ObjectHandle,
//                                            0,
//                                            NULL,
//                                            KeGetPreviousMode(),
//                                            &Object,
//                                            &HandleInfo);
//         if (!NT_SUCCESS (Status)) return Status;
//
//         /* Get the object header */
//         ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
//         ObjectType = ObjectHeader->Type;
//     }
//
//     _SEH2_TRY
//     {
//         /* Check the information class */
//         switch (ObjectInformationClass)
//         {
//             /* Basic info */
//             case ObjectBasicInformation:
//
//                 /* Validate length */
//                 InfoLength = sizeof(OBJECT_BASIC_INFORMATION);
//                 if (Length != sizeof(OBJECT_BASIC_INFORMATION))
//                 {
//                     /* Fail */
//                     Status = STATUS_INFO_LENGTH_MISMATCH;
//                     break;
//                 }
//
//                 /* Fill out the basic information */
//                 BasicInfo = (POBJECT_BASIC_INFORMATION)ObjectInformation;
//                 BasicInfo->Attributes = HandleInfo.HandleAttributes;
//                 BasicInfo->GrantedAccess = HandleInfo.GrantedAccess;
//                 BasicInfo->HandleCount = ObjectHeader->HandleCount;
//                 BasicInfo->PointerCount = ObjectHeader->PointerCount;
//
//                 /* Permanent/Exclusive Flags are NOT in Handle attributes! */
//                 if (ObjectHeader->Flags & OB_FLAG_EXCLUSIVE)
//                 {
//                     /* Set the flag */
//                     BasicInfo->Attributes |= OBJ_EXCLUSIVE;
//                 }
//                 if (ObjectHeader->Flags & OB_FLAG_PERMANENT)
//                 {
//                     /* Set the flag */
//                     BasicInfo->Attributes |= OBJ_PERMANENT;
//                 }
//
//                 /* Copy quota information */
//                 ObjectQuota = OBJECT_HEADER_TO_QUOTA_INFO(ObjectHeader);
//                 if (ObjectQuota != NULL)
//                 {
//                     BasicInfo->PagedPoolCharge = ObjectQuota->PagedPoolCharge;
//                     BasicInfo->NonPagedPoolCharge = ObjectQuota->NonPagedPoolCharge;
//                 }
//                 else
//                 {
//                     BasicInfo->PagedPoolCharge = 0;
//                     BasicInfo->NonPagedPoolCharge = 0;
//                 }
//
//                 /* Copy name information */
//                 BasicInfo->NameInfoSize = ObpQueryNameInfoSize(ObjectHeader);
//                 BasicInfo->TypeInfoSize = sizeof(OBJECT_TYPE_INFORMATION) + ObjectType->Name.Length +
//                                           sizeof(UNICODE_NULL);
//
//                 /* Check if this is a symlink */
//                 if (ObjectHeader->Type == ObpSymbolicLinkObjectType)
//                 {
//                     /* Return the creation time */
//                     BasicInfo->CreationTime.QuadPart =
//                         ((POBJECT_SYMBOLIC_LINK)Object)->CreationTime.QuadPart;
//                 }
//                 else
//                 {
//                     /* Otherwise return 0 */
//                     BasicInfo->CreationTime.QuadPart = (ULONGLONG)0;
//                 }
//
//                 /* Copy security information */
//                 BasicInfo->SecurityDescriptorSize = 0;
//                 if (BooleanFlagOn(HandleInfo.GrantedAccess, READ_CONTROL) &&
//                     ObjectHeader->SecurityDescriptor != NULL)
//                 {
//                     SecurityInformation = OWNER_SECURITY_INFORMATION |
//                                           GROUP_SECURITY_INFORMATION |
//                                           DACL_SECURITY_INFORMATION |
//                                           SACL_SECURITY_INFORMATION;
//
//                     ObjectType->TypeInfo.SecurityProcedure(Object,
//                                                            QuerySecurityDescriptor,
//                                                            &SecurityInformation,
//                                                            NULL,
//                                                            &BasicInfo->SecurityDescriptorSize,
//                                                            &ObjectHeader->SecurityDescriptor,
//                                                            ObjectType->TypeInfo.PoolType,
//                                                            &ObjectType->TypeInfo.GenericMapping);
//                 }
//
//                 /* Break out with success */
//                 Status = STATUS_SUCCESS;
//                 break;
//
//             /* Name information */
//             case ObjectNameInformation:
//
//                 /* Call the helper and break out */
//                 Status = ObQueryNameString(Object,
//                                            (POBJECT_NAME_INFORMATION)
//                                            ObjectInformation,
//                                            Length,
//                                            &InfoLength);
//                 break;
//
//             /* Information about this type */
//             case ObjectTypeInformation:
//
//                 /* Call the helper and break out */
//                 Status = ObQueryTypeInfo(ObjectHeader->Type,
//                                          (POBJECT_TYPE_INFORMATION)
//                                          ObjectInformation,
//                                          Length,
//                                          &InfoLength);
//                 break;
//
//             /* Information about all types */
//             case ObjectTypesInformation:
//                 DPRINT1("NOT IMPLEMENTED!\n");
//                 InfoLength = Length;
//                 Status = STATUS_NOT_IMPLEMENTED;
//                 break;
//
//             /* Information about the handle flags */
//             case ObjectHandleFlagInformation:
//
//                 /* Validate length */
//                 InfoLength = sizeof (OBJECT_HANDLE_FLAG_INFORMATION);
//                 if (Length != sizeof (OBJECT_HANDLE_FLAG_INFORMATION))
//                 {
//                     Status = STATUS_INFO_LENGTH_MISMATCH;
//                     break;
//                 }
//
//                 /* Get the structure */
//                 HandleFlags = (POBJECT_HANDLE_FLAG_INFORMATION)
//                                ObjectInformation;
//
//                 /* Set the flags */
//                 HandleFlags->Inherit = HandleInfo.HandleAttributes & OBJ_INHERIT;
//                 HandleFlags->ProtectFromClose = (HandleInfo.HandleAttributes &
//                                                  OBJ_PROTECT_CLOSE) != 0;
//
//                 /* Break out with success */
//                 Status = STATUS_SUCCESS;
//                 break;
//
//             /* Anything else */
//             default:
//
//                 /* Fail it */
//                 InfoLength = Length;
//                 Status = STATUS_INVALID_INFO_CLASS;
//                 break;
//         }
//
//         /* Check if the caller wanted the return length */
//         if (ResultLength)
//         {
//             /* Write the length */
//             *ResultLength = InfoLength;
//         }
//     }
//     _SEH2_EXCEPT(ExSystemExceptionFilter())
//     {
//         /* Otherwise, get the exception code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Dereference the object if we had referenced it */
//     if (Object) ObDereferenceObject(Object);
//
//     /* Return status */
//     return Status;
// }
//
// /*++
// * @name NtSetInformationObject
// * @implemented NT4
// *
// *     The NtSetInformationObject routine <FILLMEIN>
// *
// * @param ObjectHandle
// *        <FILLMEIN>
// *
// * @param ObjectInformationClass
// *        <FILLMEIN>
// *
// * @param ObjectInformation
// *        <FILLMEIN>
// *
// * @param Length
// *        <FILLMEIN>
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// NtSetInformationObject(IN HANDLE ObjectHandle,
//                        IN OBJECT_INFORMATION_CLASS ObjectInformationClass,
//                        IN PVOID ObjectInformation,
//                        IN ULONG Length)
// {
//     NTSTATUS Status;
//     OBP_SET_HANDLE_ATTRIBUTES_CONTEXT Context;
//     PVOID ObjectTable;
//     KAPC_STATE ApcState;
//     POBJECT_DIRECTORY Directory;
//     KPROCESSOR_MODE PreviousMode;
//     BOOLEAN AttachedToProcess = FALSE;
//     PAGED_CODE();
//
//     /* Validate the information class */
//     switch (ObjectInformationClass)
//     {
//         case ObjectHandleFlagInformation:
//
//             /* Validate the length */
//             if (Length != sizeof(OBJECT_HANDLE_FLAG_INFORMATION))
//             {
//                 /* Invalid length */
//                 return STATUS_INFO_LENGTH_MISMATCH;
//             }
//
//             /* Save the previous mode */
//             Context.PreviousMode = ExGetPreviousMode();
//
//             /* Check if we were called from user mode */
//             if (Context.PreviousMode != KernelMode)
//             {
//                 /* Enter SEH */
//                 _SEH2_TRY
//                 {
//                     /* Probe and capture the attribute buffer */
//                     ProbeForRead(ObjectInformation,
//                                  sizeof(OBJECT_HANDLE_FLAG_INFORMATION),
//                                  sizeof(BOOLEAN));
//                     Context.Information = *(POBJECT_HANDLE_FLAG_INFORMATION)
//                                             ObjectInformation;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     /* Return the exception code */
//                     _SEH2_YIELD(return _SEH2_GetExceptionCode());
//                 }
//                 _SEH2_END;
//             }
//             else
//             {
//                 /* Just copy the buffer directly */
//                 Context.Information = *(POBJECT_HANDLE_FLAG_INFORMATION)
//                                         ObjectInformation;
//             }
//
//             /* Check if this is a kernel handle */
//             if (ObpIsKernelHandle(ObjectHandle, Context.PreviousMode))
//             {
//                 /* Get the actual handle */
//                 ObjectHandle = ObKernelHandleToHandle(ObjectHandle);
//                 ObjectTable = ObpKernelHandleTable;
//
//                 /* Check if we're not in the system process */
//                 if (PsGetCurrentProcess() != PsInitialSystemProcess)
//                 {
//                     /* Attach to it */
//                     KeStackAttachProcess(&PsInitialSystemProcess->Pcb, &ApcState);
//                     AttachedToProcess = TRUE;
//                 }
//             }
//             else
//             {
//                 /* Use the current table */
//                 ObjectTable = PsGetCurrentProcess()->ObjectTable;
//             }
//
//             /* Change the handle attributes */
//             if (!ExChangeHandle(ObjectTable,
//                                 ObjectHandle,
//                                 ObpSetHandleAttributes,
//                                 (ULONG_PTR)&Context))
//             {
//                 /* Some failure */
//                 Status = STATUS_ACCESS_DENIED;
//             }
//             else
//             {
//                 /* We are done */
//                 Status = STATUS_SUCCESS;
//             }
//
//             /* De-attach if we were attached, and return status */
//             if (AttachedToProcess) KeUnstackDetachProcess(&ApcState);
//             break;
//
//         case ObjectSessionInformation:
//
//             /* Only a system process can do this */
//             PreviousMode = ExGetPreviousMode();
//             if (!SeSinglePrivilegeCheck(SeTcbPrivilege, PreviousMode))
//             {
//                 /* Fail */
//                 DPRINT1("Privilege not held\n");
//                 Status = STATUS_PRIVILEGE_NOT_HELD;
//             }
//             else
//             {
//                 /* Get the object directory */
//                 Status = ObReferenceObjectByHandle(ObjectHandle,
//                                                    0,
//                                                    ObpDirectoryObjectType,
//                                                    PreviousMode,
//                                                    (PVOID*)&Directory,
//                                                    NULL);
//                 if (NT_SUCCESS(Status))
//                 {
//                     /* Setup a lookup context */
//                     OBP_LOOKUP_CONTEXT LookupContext;
//                     ObpInitializeLookupContext(&LookupContext);
//
//                     /* Set the directory session ID */
//                     ObpAcquireDirectoryLockExclusive(Directory, &LookupContext);
//                     Directory->SessionId = PsGetCurrentProcessSessionId();
//                     ObpReleaseDirectoryLock(Directory, &LookupContext);
//
//                     /* We're done, release the context and dereference the directory */
//                     ObpReleaseLookupContext(&LookupContext);
//                     ObDereferenceObject(Directory);
//                 }
//             }
//             break;
//
//         default:
//             /* Unsupported class */
//             Status = STATUS_INVALID_INFO_CLASS;
//             break;
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
