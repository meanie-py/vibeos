use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ACCESS_MASK, CM_PARSE_CONTEXT, CmDeleteKey, CmDeleteValueKey, CmFlushKey, CmSaveKey, CmSaveMergedKeys, CmiCallRegisteredCallbacks, CmpCmdInit, CmpLazyFlush, CmpLockRegistry, CmpLockRegistryExclusive, CmpReleaseKcbLock, CmpSaveBootControlSet, CmpSetVersionData, CmpUnlockRegistry, ExGetPreviousMode, KPROCESSOR_MODE, KeEnterCriticalRegion, KeLeaveCriticalRegion, _OBJECT_ATTRIBUTES, _OBJECT_HANDLE_INFORMATION, ObCloseHandle, ObOpenObjectByName, ObOpenObjectByPointer, ObReferenceObjectByHandle, ObpValidateAttributes, PCM_KEY_BODY, PHANDLE, PIO_APC_ROUTINE, PIO_STATUS_BLOCK, PKEY_VALUE_ENTRY, POBJECT_ATTRIBUTES, POBJECT_TYPE, PSECURITY_DESCRIPTOR, PUNICODE_STRING, REG_DELETE_KEY_INFORMATION, REG_DELETE_VALUE_KEY_INFORMATION, REG_ENUMERATE_KEY_INFORMATION, REG_ENUMERATE_VALUE_KEY_INFORMATION, REG_POST_OPERATION_INFORMATION, REG_QUERY_KEY_INFORMATION, REG_QUERY_VALUE_KEY_INFORMATION, REG_SET_VALUE_KEY_INFORMATION, RtlInitEmptyUnicodeString, SeCaptureSecurityDescriptor, SeReleaseSecurityDescriptor, SeSinglePrivilegeCheck, ZwInitializeRegistry};
use ntkrnl_sys::ddk::{KEY_INFORMATION_CLASS, KEY_VALUE_INFORMATION_CLASS, KEY_SET_INFORMATION_CLASS, _UNICODE_STRING, LUID, _GENERIC_MAPPING};
use ntkrnl_sys::opaque_ptr;
// `_KPROCESSOR_MODE_KernelMode` and `_MODE_KernelMode` aren't usable from ddk for
// our comparisons (KPROCESSOR_MODE = i8; the bindgen enum variants are u32). Define
// local i8 constants — KernelMode == 0.
#[allow(non_upper_case_globals)]
const _KPROCESSOR_MODE_KernelMode: KPROCESSOR_MODE = 0i8;
const _MODE_KernelMode: KPROCESSOR_MODE = 0i8;

// Bindgen enum variant constants
use ntkrnl_sys::ddk::{
    _KEY_INFORMATION_CLASS_KeyBasicInformation,
    _KEY_INFORMATION_CLASS_KeyNodeInformation,
    _KEY_INFORMATION_CLASS_KeyFullInformation,
    _KEY_INFORMATION_CLASS_KeyNameInformation,
    _KEY_INFORMATION_CLASS_KeyCachedInformation,
    _KEY_INFORMATION_CLASS_KeyFlagsInformation,
    _KEY_VALUE_INFORMATION_CLASS_KeyValueBasicInformation,
    _KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformation,
    _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation,
    _KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformationAlign64,
    _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformationAlign64,
    _REG_NOTIFY_CLASS_RegNtPreSetValueKey,
    _REG_NOTIFY_CLASS_RegNtPostSetValueKey,
    _REG_NOTIFY_CLASS_RegNtPreQueryKey,
    _REG_NOTIFY_CLASS_RegNtPostQueryKey,
    _REG_NOTIFY_CLASS_RegNtPreQueryValueKey,
    _REG_NOTIFY_CLASS_RegNtPostQueryValueKey,
    _REG_NOTIFY_CLASS_RegNtPreEnumerateKey,
    _REG_NOTIFY_CLASS_RegNtPostEnumerateKey,
    _REG_NOTIFY_CLASS_RegNtPreEnumerateValueKey,
    _REG_NOTIFY_CLASS_RegNtPostEnumerateValueKey,
    _REG_NOTIFY_CLASS_RegNtPreDeleteKey,
    _REG_NOTIFY_CLASS_RegNtPostDeleteKey,
    _REG_NOTIFY_CLASS_RegNtPreDeleteValueKey,
    _REG_NOTIFY_CLASS_RegNtPostDeleteValueKey,
};
use core::ffi::c_void;

// REG_NOTIFY_CLASS bindgen variants (aliased for translation convenience)
const RegNtPreDeleteKey: u32 = _REG_NOTIFY_CLASS_RegNtPreDeleteKey;
const RegNtPostDeleteKey: u32 = _REG_NOTIFY_CLASS_RegNtPostDeleteKey;
const RegNtPreDeleteValueKey: u32 = _REG_NOTIFY_CLASS_RegNtPreDeleteValueKey;
const RegNtPostDeleteValueKey: u32 = _REG_NOTIFY_CLASS_RegNtPostDeleteValueKey;
const RegNtPreSetValueKey: u32 = _REG_NOTIFY_CLASS_RegNtPreSetValueKey;
const RegNtPostSetValueKey: u32 = _REG_NOTIFY_CLASS_RegNtPostSetValueKey;
const RegNtPreEnumerateKey: u32 = _REG_NOTIFY_CLASS_RegNtPreEnumerateKey;
const RegNtPostEnumerateKey: u32 = _REG_NOTIFY_CLASS_RegNtPostEnumerateKey;
const RegNtPreEnumerateValueKey: u32 = _REG_NOTIFY_CLASS_RegNtPreEnumerateValueKey;
const RegNtPostEnumerateValueKey: u32 = _REG_NOTIFY_CLASS_RegNtPostEnumerateValueKey;
const RegNtPreQueryKey: u32 = _REG_NOTIFY_CLASS_RegNtPreQueryKey;
const RegNtPostQueryKey: u32 = _REG_NOTIFY_CLASS_RegNtPostQueryKey;
const RegNtPreQueryValueKey: u32 = _REG_NOTIFY_CLASS_RegNtPreQueryValueKey;
const RegNtPostQueryValueKey: u32 = _REG_NOTIFY_CLASS_RegNtPostQueryValueKey;

// Registry status constants (not in ntkrnl-sys)
const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC0000002_u32 as i32;
const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000009A_u32 as i32;

// CM key access rights (cm.h / winnt.h)
const KEY_QUERY_VALUE: ACCESS_MASK = 0x0001;
const KEY_SET_VALUE: ACCESS_MASK = 0x0002;
const KEY_CREATE_SUB_KEY: ACCESS_MASK = 0x0004;
const KEY_ENUMERATE_SUB_KEYS: ACCESS_MASK = 0x0008;
const KEY_NOTIFY: ACCESS_MASK = 0x0010;
const KEY_READ: ACCESS_MASK = 0x00020019;
const KEY_WRITE: ACCESS_MASK = 0x00020006;
const KEY_WOW64_RES: ACCESS_MASK = 0x00000300;
const DELETE: ACCESS_MASK = 0x00010000;
const FILE_TRAVERSE: ACCESS_MASK = 0x00000020;

// CM/registry constants
const REG_NO_LAZY_FLUSH: u32 = 0x00000004;
const REG_STANDARD_FORMAT: u32 = 1;
const REG_LATEST_FORMAT: u32 = 2;
const REG_NO_COMPRESSION: u32 = 4;
const REG_OPTION_BACKUP_RESTORE: u32 = 0x00000004;

// CMP boot flags (from cm.h)
const CM_BOOT_FLAG_SETUP: u16 = 1;
const CM_BOOT_FLAG_ACCEPTED: u16 = 2;
const CM_BOOT_FLAG_MAX: u16 = CM_BOOT_FLAG_ACCEPTED + 999;

// CM KCB ExtFlags (from cm.h)
const CM_KCB_READ_ONLY_KEY: u32 = 0x80;

// Pool tag for CM allocations
const TAG_CM: ULONG = u32::from_le_bytes(*b"MCmC");

// Misc
const PRODUCT_ACTIVATION_VERSION: ULONG = 0x0000;
const FILE_WRITE_DATA: ACCESS_MASK = 0x00000002;

// Stub macros (DUMMY_PSEH builds — these compile-out)
macro_rules! UNIMPLEMENTED_ONCE {
    () => {};
}

// Local helper: replicate ProbeForReadUnicodeString macro from probe.h.
// With DUMMY_PSEH, alignment + range checks are skipped (the C macro depends on
// SEH + ProbeForRead). This is just a volatile read.
#[inline]
unsafe fn ProbeForReadUnicodeString(ptr: PUNICODE_STRING) -> _UNICODE_STRING {
    core::ptr::read_volatile(ptr as *const _UNICODE_STRING)
}

// Probe shims (defined as `extern "C"` in shim_exports.rs)
extern "C" {
    fn ProbeForWriteHandle(Address: PHANDLE);
    fn ProbeAndCaptureUnicodeString(
        CapturedString: *mut _UNICODE_STRING,
        PreviousMode: KPROCESSOR_MODE,
        String: *const _UNICODE_STRING,
    ) -> NTSTATUS;
    fn ReleaseCapturedUnicodeString(
        CapturedString: *mut _UNICODE_STRING,
        PreviousMode: KPROCESSOR_MODE,
    );
}
// NTAPI (stdcall on i386) — defs in shim_exports/common_inlines are extern "system"
extern "system" {
    fn ProbeForWriteUlong(Address: *mut ULONG);
}

// Cross-module Rust functions referenced by name (these are #[no_mangle] elsewhere)
extern "C" {
    fn HalEndOfBoot();
    fn Ki386PerfEnd();
}

extern "system" {
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
    fn ZwClose(Handle: HANDLE) -> NTSTATUS;
    fn CmLoadKey(
        TargetKey: POBJECT_ATTRIBUTES,
        SourceFile: POBJECT_ATTRIBUTES,
        Flags: ULONG,
        TrustClassKey: PCM_KEY_BODY,
    ) -> NTSTATUS;
    fn CmUnloadKey(
        KeyControlBlock: *mut ntkrnl_sys::ddk::_CM_KEY_CONTROL_BLOCK,
        Flags: ULONG,
    ) -> NTSTATUS;
    fn CmQueryKey(
        KeyControlBlock: *mut ntkrnl_sys::ddk::_CM_KEY_CONTROL_BLOCK,
        KeyInformationClass: KEY_INFORMATION_CLASS,
        KeyInformation: PVOID,
        Length: ULONG,
        ResultLength: PULONG,
    ) -> NTSTATUS;
    fn CmQueryValueKey(
        KeyControlBlock: *mut ntkrnl_sys::ddk::_CM_KEY_CONTROL_BLOCK,
        ValueName: _UNICODE_STRING,
        KeyValueInformationClass: KEY_VALUE_INFORMATION_CLASS,
        KeyValueInformation: PVOID,
        Length: ULONG,
        ResultLength: PULONG,
    ) -> NTSTATUS;
    fn CmSetValueKey(
        KeyControlBlock: *mut ntkrnl_sys::ddk::_CM_KEY_CONTROL_BLOCK,
        ValueName: PUNICODE_STRING,
        Type: ULONG,
        Data: PVOID,
        DataSize: ULONG,
    ) -> NTSTATUS;
    fn CmEnumerateValueKey(
        KeyControlBlock: *mut ntkrnl_sys::ddk::_CM_KEY_CONTROL_BLOCK,
        Index: ULONG,
        KeyValueInformationClass: KEY_VALUE_INFORMATION_CLASS,
        KeyValueInformation: PVOID,
        Length: ULONG,
        ResultLength: PULONG,
    ) -> NTSTATUS;
    fn CmEnumerateKey(
        KeyControlBlock: *mut ntkrnl_sys::ddk::_CM_KEY_CONTROL_BLOCK,
        Index: ULONG,
        KeyInformationClass: KEY_INFORMATION_CLASS,
        KeyInformation: PVOID,
        Length: ULONG,
        ResultLength: PULONG,
    ) -> NTSTATUS;
}

// CmpAcquireKcbLockShared is exported as cdecl by cm_rust_shims.c.
extern "C" {
    fn CmpAcquireKcbLockShared(Kcb: *mut ntkrnl_sys::ddk::_CM_KEY_CONTROL_BLOCK);
}

// ExAcquirePushLockExclusive / ExReleasePushLockExclusive are FORCEINLINE in ex.h
// — i386 has no exported symbol. Use the Rust inline impls in ntkrnl_sys::inline::ex.
use ntkrnl_sys::inline::ex::{ExAcquirePushLockExclusive, ExReleasePushLockExclusive};

use ntkrnl_sys::ddk::CmpEnumerateOpenSubKeys;

// Globals owned by ntapi.c (excluded from C build); defined here.
#[no_mangle] pub static mut CmFirstTime: u8 = 1;          // C: BOOLEAN ... = TRUE;
#[no_mangle] pub static mut CmBootAcceptFirstTime: u8 = 1;  // C: BOOLEAN ... = TRUE;

// cmdata.c globals (C linkage; consumed by Rust translation here)
extern "C" {
    static mut CmpHoldLazyFlush: u8;
    static CmpKeyObjectType: POBJECT_TYPE;
    static IoFileObjectType: POBJECT_TYPE;
    static MmUserProbeAddress: usize;
    static InitSafeBootMode: ULONG;
    static mut SeBackupPrivilege: LUID;
    static mut SeRestorePrivilege: LUID;
    static mut CmpLoadHiveLock: ntkrnl_sys::ddk::_EX_PUSH_LOCK;
}

unsafe fn ReleaseCapturedObjectAttributes(
    CapturedObjectAttributes: POBJECT_ATTRIBUTES,
    AccessMode: KPROCESSOR_MODE,
) {
    if !(*CapturedObjectAttributes).SecurityDescriptor.is_null() {
        SeReleaseSecurityDescriptor(
            (*CapturedObjectAttributes).SecurityDescriptor,
            AccessMode,
            1u8,
        );
        (*CapturedObjectAttributes).SecurityDescriptor = core::ptr::null_mut();
    }

    if !(*CapturedObjectAttributes).ObjectName.is_null() {
        ReleaseCapturedUnicodeString((*CapturedObjectAttributes).ObjectName, AccessMode);
    }
}

pub unsafe extern "system" fn ProbeAndCaptureObjectAttributes(
    CapturedObjectAttributes: POBJECT_ATTRIBUTES,
    ObjectName: PUNICODE_STRING,
    AccessMode: KPROCESSOR_MODE,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    CaptureSecurity: u8,
) -> NTSTATUS {
    let CapturedObjectAttributes = opaque_ptr(CapturedObjectAttributes);
    let ObjectName = opaque_ptr(ObjectName);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut SecurityDescriptor: PSECURITY_DESCRIPTOR;
    let mut LocalObjectName: PUNICODE_STRING = core::ptr::null_mut();

    core::ptr::write_bytes(CapturedObjectAttributes, 0, 1);

    if !ObjectAttributes.is_null() {
        if AccessMode != 0 {
            if (ObjectAttributes as usize) < 0x10000 ||
               (ObjectAttributes as usize).wrapping_add(core::mem::size_of::<_OBJECT_ATTRIBUTES>()) > 0x7FFFFFFF {
                ReleaseCapturedObjectAttributes(CapturedObjectAttributes, AccessMode);
                return STATUS_ACCESS_VIOLATION;
            }
        }

        if (*ObjectAttributes).Length != core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32 ||
           ((*ObjectAttributes).Attributes & !0x000012F2u32) != 0 {
            return STATUS_INVALID_PARAMETER;
        }

        (*CapturedObjectAttributes).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        (*CapturedObjectAttributes).RootDirectory = (*ObjectAttributes).RootDirectory;
        (*CapturedObjectAttributes).Attributes = ObpValidateAttributes((*ObjectAttributes).Attributes, AccessMode);
        LocalObjectName = (*ObjectAttributes).ObjectName;
        SecurityDescriptor = (*ObjectAttributes).SecurityDescriptor;

        if CaptureSecurity != 0 && !SecurityDescriptor.is_null() {
            Status = SeCaptureSecurityDescriptor(
                SecurityDescriptor,
                AccessMode,
                NonPagedPool,
                1u8,
                core::ptr::addr_of_mut!((*CapturedObjectAttributes).SecurityDescriptor),
            );
            if !NT_SUCCESS(Status) {
                (*CapturedObjectAttributes).SecurityDescriptor = core::ptr::null_mut();
                return Status;
            }
        } else {
            (*CapturedObjectAttributes).SecurityDescriptor = core::ptr::null_mut();
        }

        (*CapturedObjectAttributes).SecurityQualityOfService = core::ptr::null_mut();
    } else {
        LocalObjectName = core::ptr::null_mut();
    }

    if !LocalObjectName.is_null() {
        Status = ProbeAndCaptureUnicodeString(ObjectName, AccessMode, LocalObjectName);
    } else {
        RtlInitEmptyUnicodeString(ObjectName, core::ptr::null_mut(), 0);

        if !(*CapturedObjectAttributes).RootDirectory.is_null() {
            Status = STATUS_OBJECT_NAME_INVALID;
        }
    }

    (*CapturedObjectAttributes).ObjectName = ObjectName;

    if !NT_SUCCESS(Status) {
        ReleaseCapturedObjectAttributes(CapturedObjectAttributes, AccessMode);
    }

    Status
}

unsafe fn CmpConvertHandleToKernelHandle(
    SourceHandle: HANDLE,
    ObjectType: POBJECT_TYPE,
    DesiredAccess: ACCESS_MASK,
    AccessMode: KPROCESSOR_MODE,
    KernelHandle: PHANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Object: PVOID = core::ptr::null_mut();

    *KernelHandle = core::ptr::null_mut();

    if SourceHandle == core::ptr::null_mut() {
        return STATUS_SUCCESS;
    }

    Status = ObReferenceObjectByHandle(
        SourceHandle,
        DesiredAccess,
        ObjectType,
        AccessMode,
        &mut Object,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ObOpenObjectByPointer(
        Object,
        OBJ_KERNEL_HANDLE,
        core::ptr::null_mut(),
        DesiredAccess,
        ObjectType,
        0i8,
        KernelHandle,
    );

    ObDereferenceObject(Object);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateKey(
    KeyHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    TitleIndex: ULONG,
    Class: PUNICODE_STRING,
    CreateOptions: ULONG,
    Disposition: PULONG,
) -> NTSTATUS {
    let KeyHandle = opaque_ptr(KeyHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let Class = opaque_ptr(Class);
    let Disposition = opaque_ptr(Disposition);
    let mut Status: NTSTATUS;
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut ParseContext: CM_PARSE_CONTEXT = core::mem::zeroed();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut DesiredAccess = DesiredAccess;

    DesiredAccess &= !KEY_WOW64_RES;

    if PreviousMode != 0i8 {
        if !Class.is_null() {
            ParseContext.Class = ProbeForReadUnicodeString(Class);
            if ParseContext.Class.Length > 0 {
                let _ = core::slice::from_raw_parts(
                    ParseContext.Class.Buffer,
                    (ParseContext.Class.Length as usize) / core::mem::size_of::<WCHAR>(),
                );
            }
        }

        ProbeForWriteHandle(KeyHandle);
        *KeyHandle = core::ptr::null_mut();

        let _ = core::slice::from_raw_parts(
            ObjectAttributes as *const u8,
            core::mem::size_of::<_OBJECT_ATTRIBUTES>(),
        );

        if !Disposition.is_null() {
            ProbeForWriteUlong(Disposition);
        }
    } else {
        if !Class.is_null() {
            ParseContext.Class = *Class;
        }
    }

    ParseContext.CreateOperation = 1u8;
    ParseContext.CreateOptions = CreateOptions;

    Status = ObOpenObjectByName(
        ObjectAttributes,
        CmpKeyObjectType,
        PreviousMode,
        core::ptr::null_mut(),
        DesiredAccess,
        core::ptr::addr_of_mut!(ParseContext) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(Handle),
    );

    if NT_SUCCESS(Status) {
        *KeyHandle = Handle;
    }
    if !Disposition.is_null() {
        *Disposition = ParseContext.Disposition;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenKey(
    KeyHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let KeyHandle = opaque_ptr(KeyHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let mut ParseContext: CM_PARSE_CONTEXT = core::mem::zeroed();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();

    let mut DesiredAccess = DesiredAccess & !0x0100u32;

    if PreviousMode != 0 {
        *KeyHandle = core::ptr::null_mut();
        if ((KeyHandle as usize) & (core::mem::size_of::<HANDLE>() - 1)) != 0 {
            return 0xC0000005u32 as i32;
        }
        if ((ObjectAttributes as usize) & (core::mem::size_of::<u32>() - 1)) != 0 {
            return 0xC0000005u32 as i32;
        }
        core::ptr::read_volatile(ObjectAttributes as *const _OBJECT_ATTRIBUTES);
    }

    Status = ObOpenObjectByName(
        ObjectAttributes,
        CmpKeyObjectType,
        PreviousMode,
        core::ptr::null_mut(),
        DesiredAccess,
        &mut ParseContext as *mut _ as *mut core::ffi::c_void,
        &mut Handle,
    );

    // Note: The C source NtOpenKey does NOT have a direct CmpDoOpen reparse call;
    // STATUS_REPARSE is handled internally by ObOpenObjectByName via the
    // CmpParseKey object-type parser. The hallucinated call has been removed.

    if NT_SUCCESS(Status) {
        if PreviousMode != 0 {
            *KeyHandle = Handle;
        } else {
            *KeyHandle = Handle;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtDeleteKey(KeyHandle: HANDLE) -> NTSTATUS {
    let KeyHandle = opaque_ptr(KeyHandle);
    let mut KeyObject: PCM_KEY_BODY = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut DeleteKeyInfo: REG_DELETE_KEY_INFORMATION = core::mem::zeroed();
    let mut PostOperationInfo: REG_POST_OPERATION_INFORMATION = core::mem::zeroed();

    Status = ObReferenceObjectByHandle(
        KeyHandle,
        DELETE,
        CmpKeyObjectType,
        ExGetPreviousMode(),
        &mut KeyObject as *mut PCM_KEY_BODY as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    PostOperationInfo.Object = KeyObject as PVOID;
    DeleteKeyInfo.Object = KeyObject as PVOID;
    Status = CmiCallRegisteredCallbacks(RegNtPreDeleteKey, &mut DeleteKeyInfo as *mut _ as PVOID);
    if NT_SUCCESS(Status) {
        if (((*(*KeyObject).KeyControlBlock).__bindgen_anon_1.ExtFlags() & CM_KCB_READ_ONLY_KEY) != 0) ||
           (((*(*(*KeyObject).KeyControlBlock).ParentKcb).__bindgen_anon_1.ExtFlags() & CM_KCB_READ_ONLY_KEY) != 0)
        {
            Status = STATUS_ACCESS_DENIED;
        } else {
            Status = CmDeleteKey(KeyObject);
        }

        PostOperationInfo.Status = Status;
        CmiCallRegisteredCallbacks(RegNtPostDeleteKey, &mut PostOperationInfo as *mut _ as PVOID);
    }

    ObDereferenceObject(KeyObject as PVOID);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtEnumerateKey(
    KeyHandle: HANDLE,
    Index: ULONG,
    KeyInformationClass: KEY_INFORMATION_CLASS,
    KeyInformation: PVOID,
    Length: ULONG,
    ResultLength: PULONG,
) -> NTSTATUS {
    let KeyHandle = opaque_ptr(KeyHandle);
    let KeyInformation = opaque_ptr(KeyInformation);
    let ResultLength = opaque_ptr(ResultLength);
    let PreviousMode = ExGetPreviousMode();
    let mut Status: NTSTATUS;
    let mut KeyObject: PCM_KEY_BODY = core::ptr::null_mut();
    let mut EnumerateKeyInfo: REG_ENUMERATE_KEY_INFORMATION = core::mem::zeroed();
    let mut PostOperationInfo: REG_POST_OPERATION_INFORMATION = core::mem::zeroed();

    if (KeyInformationClass != _KEY_INFORMATION_CLASS_KeyBasicInformation)
        && (KeyInformationClass != _KEY_INFORMATION_CLASS_KeyNodeInformation)
        && (KeyInformationClass != _KEY_INFORMATION_CLASS_KeyFullInformation)
    {
        return STATUS_INVALID_PARAMETER;
    }

    Status = ObReferenceObjectByHandle(
        KeyHandle,
        KEY_ENUMERATE_SUB_KEYS,
        CmpKeyObjectType,
        PreviousMode,
        &mut KeyObject as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if PreviousMode != _KPROCESSOR_MODE_KernelMode {
        if ResultLength.is_null()
            || (ResultLength as usize) < core::mem::size_of::<ULONG>()
            || ((ResultLength as usize) & (core::mem::size_of::<ULONG>() - 1)) != 0
        {
            ObDereferenceObject(KeyObject as *mut core::ffi::c_void);
            return STATUS_DATATYPE_MISALIGNMENT;
        }

        if KeyInformation.is_null()
            || (Length > 0 && ((KeyInformation as usize) & (core::mem::size_of::<ULONG>() - 1)) != 0)
        {
            ObDereferenceObject(KeyObject as *mut core::ffi::c_void);
            return STATUS_DATATYPE_MISALIGNMENT;
        }
    }

    EnumerateKeyInfo.Object = (*KeyObject).KeyControlBlock as PVOID;
    EnumerateKeyInfo.Index = Index;
    EnumerateKeyInfo.KeyInformationClass = KeyInformationClass;
    EnumerateKeyInfo.Length = Length;
    EnumerateKeyInfo.ResultLength = ResultLength;

    PostOperationInfo.Object = KeyObject as PVOID;
    EnumerateKeyInfo.Object = KeyObject as PVOID;
    Status = CmiCallRegisteredCallbacks(RegNtPreEnumerateKey, &mut EnumerateKeyInfo as *mut _ as PVOID);
    if NT_SUCCESS(Status) {
        Status = CmEnumerateKey(
            (*KeyObject).KeyControlBlock,
            Index,
            KeyInformationClass,
            KeyInformation,
            Length,
            ResultLength,
        );

        PostOperationInfo.Status = Status;
        CmiCallRegisteredCallbacks(RegNtPostEnumerateKey, &mut PostOperationInfo as *mut _ as PVOID);
    }

    ObDereferenceObject(KeyObject as *mut core::ffi::c_void);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtEnumerateValueKey(
    KeyHandle: HANDLE,
    Index: ULONG,
    KeyValueInformationClass: KEY_VALUE_INFORMATION_CLASS,
    KeyValueInformation: PVOID,
    Length: ULONG,
    ResultLength: PULONG,
) -> NTSTATUS {
    let KeyHandle = opaque_ptr(KeyHandle);
    let KeyValueInformation = opaque_ptr(KeyValueInformation);
    let ResultLength = opaque_ptr(ResultLength);
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut Status: NTSTATUS;
    let mut KeyObject: PCM_KEY_BODY = core::ptr::null_mut();
    let mut EnumerateValueKeyInfo: REG_ENUMERATE_VALUE_KEY_INFORMATION = core::mem::zeroed();
    let mut PostOperationInfo: REG_POST_OPERATION_INFORMATION = core::mem::zeroed();

    if (KeyValueInformationClass != _KEY_VALUE_INFORMATION_CLASS_KeyValueBasicInformation) &&
       (KeyValueInformationClass != _KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformation) &&
       (KeyValueInformationClass != _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation) &&
       (KeyValueInformationClass != _KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformationAlign64) &&
       (KeyValueInformationClass != _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformationAlign64)
    {
        return STATUS_INVALID_PARAMETER;
    }

    Status = ObReferenceObjectByHandle(
        KeyHandle,
        KEY_QUERY_VALUE,
        CmpKeyObjectType,
        PreviousMode,
        &mut KeyObject as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if PreviousMode != _MODE_KernelMode {
        if (ResultLength as usize) < core::mem::size_of::<ULONG>() ||
           (ResultLength as usize).wrapping_add(core::mem::size_of::<ULONG>() - 1) < (ResultLength as usize) ||
           (KeyValueInformation as usize) < core::mem::size_of::<ULONG>() ||
           (KeyValueInformation as usize).wrapping_add(Length as usize) < (KeyValueInformation as usize) ||
           (KeyValueInformation as usize).wrapping_add(Length as usize) > 0x7FFFFFFF
        {
            ObDereferenceObject(KeyObject as PVOID);
            return STATUS_ACCESS_VIOLATION;
        }
        core::ptr::write_volatile(ResultLength, core::ptr::read_volatile(ResultLength));
        core::ptr::write_bytes(KeyValueInformation, 0, Length as usize);
    }

    EnumerateValueKeyInfo.Object = KeyObject as PVOID;
    EnumerateValueKeyInfo.Index = Index;
    EnumerateValueKeyInfo.KeyValueInformationClass = KeyValueInformationClass;
    EnumerateValueKeyInfo.KeyValueInformation = KeyValueInformation;
    EnumerateValueKeyInfo.Length = Length;
    EnumerateValueKeyInfo.ResultLength = ResultLength;

    PostOperationInfo.Object = KeyObject as PVOID;
    Status = CmiCallRegisteredCallbacks(
        RegNtPreEnumerateValueKey,
        &mut EnumerateValueKeyInfo as *mut _ as PVOID,
    );
    if NT_SUCCESS(Status) {
        Status = CmEnumerateValueKey(
            (*KeyObject).KeyControlBlock,
            Index,
            KeyValueInformationClass,
            KeyValueInformation,
            Length,
            ResultLength,
        );

        PostOperationInfo.Status = Status;
        CmiCallRegisteredCallbacks(
            RegNtPostEnumerateValueKey,
            &mut PostOperationInfo as *mut _ as PVOID,
        );
    }

    ObDereferenceObject(KeyObject as PVOID);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryKey(
    KeyHandle: HANDLE,
    KeyInformationClass: KEY_INFORMATION_CLASS,
    KeyInformation: PVOID,
    Length: ULONG,
    ResultLength: PULONG,
) -> NTSTATUS {
    let KeyHandle = opaque_ptr(KeyHandle);
    let KeyInformation = opaque_ptr(KeyInformation);
    let ResultLength = opaque_ptr(ResultLength);
    let PreviousMode = ExGetPreviousMode();
    let mut Status: NTSTATUS;
    let mut KeyObject: PCM_KEY_BODY = core::ptr::null_mut();
    let mut QueryKeyInfo: REG_QUERY_KEY_INFORMATION = core::mem::zeroed();
    let mut PostOperationInfo: REG_POST_OPERATION_INFORMATION = core::mem::zeroed();
    let mut HandleInfo: _OBJECT_HANDLE_INFORMATION = core::mem::zeroed();

    if KeyInformationClass != _KEY_INFORMATION_CLASS_KeyBasicInformation
        && KeyInformationClass != _KEY_INFORMATION_CLASS_KeyNodeInformation
        && KeyInformationClass != _KEY_INFORMATION_CLASS_KeyFullInformation
        && KeyInformationClass != _KEY_INFORMATION_CLASS_KeyNameInformation
        && KeyInformationClass != _KEY_INFORMATION_CLASS_KeyCachedInformation
        && KeyInformationClass != _KEY_INFORMATION_CLASS_KeyFlagsInformation
    {
        return STATUS_INVALID_PARAMETER;
    }

    if KeyInformationClass == _KEY_INFORMATION_CLASS_KeyNameInformation {
        Status = ObReferenceObjectByHandle(
            KeyHandle,
            0,
            CmpKeyObjectType,
            PreviousMode,
            &mut KeyObject as *mut _ as *mut PVOID,
            &mut HandleInfo,
        );
        if NT_SUCCESS(Status) {
            if HandleInfo.GrantedAccess == 0 {
                ObDereferenceObject(KeyObject as PVOID);
                Status = STATUS_ACCESS_DENIED;
            }
        }
    } else {
        Status = ObReferenceObjectByHandle(
            KeyHandle,
            KEY_QUERY_VALUE,
            CmpKeyObjectType,
            PreviousMode,
            &mut KeyObject as *mut _ as *mut PVOID,
            core::ptr::null_mut(),
        );
    }

    if !NT_SUCCESS(Status) {
        return Status;
    }

    if PreviousMode != 0i8 {
        if !ResultLength.is_null() && (ResultLength as usize) < 0x80000000 {
            if (ResultLength as usize).wrapping_add(core::mem::size_of::<ULONG>()) <= 0x80000000 {
                let aligned = (ResultLength as usize) & !(core::mem::size_of::<ULONG>() - 1);
                if aligned == (ResultLength as usize) {
                    core::ptr::write_volatile(ResultLength, core::ptr::read_volatile(ResultLength));
                }
            }
        }
        if !KeyInformation.is_null() && (KeyInformation as usize) < 0x80000000 {
            if (KeyInformation as usize).wrapping_add(Length as usize) <= 0x80000000 {
                let align = core::mem::size_of::<ULONG>();
                let aligned = (KeyInformation as usize) & !(align - 1);
                if aligned == (KeyInformation as usize) {
                    let ptr = KeyInformation as *mut u8;
                    for i in 0..Length {
                        core::ptr::write_volatile(ptr.add(i as usize), core::ptr::read_volatile(ptr.add(i as usize)));
                    }
                }
            }
        }
    }

    PostOperationInfo.Object = KeyObject as PVOID;
    QueryKeyInfo.Object = KeyObject as PVOID;
    QueryKeyInfo.KeyInformationClass = KeyInformationClass;
    QueryKeyInfo.KeyInformation = KeyInformation;
    QueryKeyInfo.Length = Length;
    QueryKeyInfo.ResultLength = ResultLength;

    Status = CmiCallRegisteredCallbacks(RegNtPreQueryKey, &mut QueryKeyInfo as *mut _ as PVOID);
    if NT_SUCCESS(Status) {
        Status = CmQueryKey(
            (*KeyObject).KeyControlBlock,
            KeyInformationClass,
            KeyInformation,
            Length,
            ResultLength,
        );

        PostOperationInfo.Status = Status;
        CmiCallRegisteredCallbacks(RegNtPostQueryKey, &mut PostOperationInfo as *mut _ as PVOID);
    }

    ObDereferenceObject(KeyObject as PVOID);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryValueKey(
    KeyHandle: HANDLE,
    ValueName: PUNICODE_STRING,
    KeyValueInformationClass: KEY_VALUE_INFORMATION_CLASS,
    KeyValueInformation: PVOID,
    Length: ULONG,
    ResultLength: PULONG,
) -> NTSTATUS {
    let KeyHandle = opaque_ptr(KeyHandle);
    let ValueName = opaque_ptr(ValueName);
    let KeyValueInformation = opaque_ptr(KeyValueInformation);
    let ResultLength = opaque_ptr(ResultLength);
    let mut Status: NTSTATUS;
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut KeyObject: PCM_KEY_BODY = core::ptr::null_mut();
    let mut QueryValueKeyInfo: REG_QUERY_VALUE_KEY_INFORMATION = core::mem::zeroed();
    let mut PostOperationInfo: REG_POST_OPERATION_INFORMATION = core::mem::zeroed();
    let mut ValueNameCopy: _UNICODE_STRING = core::mem::zeroed();

    if (KeyValueInformationClass != _KEY_VALUE_INFORMATION_CLASS_KeyValueBasicInformation)
        && (KeyValueInformationClass != _KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformation)
        && (KeyValueInformationClass != _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation)
        && (KeyValueInformationClass != _KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformationAlign64)
        && (KeyValueInformationClass != _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformationAlign64)
    {
        return STATUS_INVALID_PARAMETER;
    }

    Status = ObReferenceObjectByHandle(
        KeyHandle,
        KEY_QUERY_VALUE,
        CmpKeyObjectType,
        PreviousMode,
        &mut KeyObject as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if PreviousMode != _MODE_KernelMode {
        if !ResultLength.is_null() {
            core::ptr::write_volatile(ResultLength, core::ptr::read_volatile(ResultLength));
        }
        if !KeyValueInformation.is_null() && Length > 0 {
            core::ptr::write_bytes(KeyValueInformation as *mut u8, 0, 1);
        }
    }

    Status = ProbeAndCaptureUnicodeString(&mut ValueNameCopy, PreviousMode, ValueName);
    if !NT_SUCCESS(Status) {
        ObDereferenceObject(KeyObject as PVOID);
        return Status;
    }

    if (ValueNameCopy.Length & (core::mem::size_of::<WCHAR>() as u16 - 1)) != 0 {
        ReleaseCapturedUnicodeString(&mut ValueNameCopy, PreviousMode);
        ObDereferenceObject(KeyObject as PVOID);
        return STATUS_INVALID_PARAMETER;
    }

    while ValueNameCopy.Length != 0
        && *ValueNameCopy.Buffer.add(
            ValueNameCopy.Length as usize / core::mem::size_of::<WCHAR>() - 1,
        ) == 0
    {
        ValueNameCopy.Length -= core::mem::size_of::<WCHAR>() as u16;
    }

    PostOperationInfo.Object = KeyObject as PVOID;
    QueryValueKeyInfo.Object = KeyObject as PVOID;
    QueryValueKeyInfo.ValueName = &mut ValueNameCopy;
    QueryValueKeyInfo.KeyValueInformationClass = KeyValueInformationClass;
    QueryValueKeyInfo.Length = Length;
    QueryValueKeyInfo.ResultLength = ResultLength;

    Status = CmiCallRegisteredCallbacks(
        RegNtPreQueryValueKey,
        &mut QueryValueKeyInfo as *mut _ as PVOID,
    );
    if NT_SUCCESS(Status) {
        Status = CmQueryValueKey(
            (*KeyObject).KeyControlBlock,
            ValueNameCopy,
            KeyValueInformationClass,
            KeyValueInformation,
            Length,
            ResultLength,
        );

        PostOperationInfo.Status = Status;
        CmiCallRegisteredCallbacks(
            RegNtPostQueryValueKey,
            &mut PostOperationInfo as *mut _ as PVOID,
        );
    }

    if !ValueNameCopy.Buffer.is_null() {
        ReleaseCapturedUnicodeString(&mut ValueNameCopy, PreviousMode);
    }

    ObDereferenceObject(KeyObject as PVOID);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetValueKey(
    KeyHandle: HANDLE,
    ValueName: PUNICODE_STRING,
    TitleIndex: ULONG,
    Type: ULONG,
    Data: PVOID,
    DataSize: ULONG,
) -> NTSTATUS {
    let KeyHandle = opaque_ptr(KeyHandle);
    let ValueName = opaque_ptr(ValueName);
    let Data = opaque_ptr(Data);
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let PreviousMode: KPROCESSOR_MODE;
    let mut KeyObject: PCM_KEY_BODY = core::ptr::null_mut();
    let mut SetValueKeyInfo: REG_SET_VALUE_KEY_INFORMATION = core::mem::zeroed();
    let mut PostOperationInfo: REG_POST_OPERATION_INFORMATION = core::mem::zeroed();
    let mut ValueNameCopy: _UNICODE_STRING = core::mem::zeroed();

    PreviousMode = ExGetPreviousMode();

    Status = ObReferenceObjectByHandle(
        KeyHandle,
        KEY_SET_VALUE,
        CmpKeyObjectType,
        PreviousMode,
        &mut KeyObject as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let mut Data = Data;
    if DataSize == 0 {
        Data = core::ptr::null_mut();
    }

    if PreviousMode != 0i8 && DataSize != 0 {
        let mut DataCopy: PVOID = core::ptr::null_mut();

        Status = STATUS_SUCCESS;

        if NT_SUCCESS(Status) {
            DataCopy = ExAllocatePoolWithTag(PagedPool, DataSize as _, TAG_CM);
            if DataCopy.is_null() {
                ObDereferenceObject(KeyObject as PVOID);
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            core::ptr::copy_nonoverlapping(Data as *const u8, DataCopy as *mut u8, DataSize as usize);

            if !NT_SUCCESS(Status) {
                ExFreePoolWithTag(DataCopy, TAG_CM);
                ObDereferenceObject(KeyObject as PVOID);
                return Status;
            }

            Data = DataCopy;
        } else {
            ObDereferenceObject(KeyObject as PVOID);
            return Status;
        }
    }

    Status = ProbeAndCaptureUnicodeString(&mut ValueNameCopy, PreviousMode, ValueName);
    if NT_SUCCESS(Status) {
        if (ValueNameCopy.Length > 32767)
            || (ValueNameCopy.Length & (core::mem::size_of::<WCHAR>() as u16 - 1)) != 0
            || (DataSize > 0x80000000)
        {
            Status = STATUS_INVALID_PARAMETER;
        }
    }

    while NT_SUCCESS(Status)
        && ValueNameCopy.Length != 0
        && *ValueNameCopy
            .Buffer
            .add((ValueNameCopy.Length / core::mem::size_of::<WCHAR>() as u16 - 1) as usize)
            == 0
    {
        ValueNameCopy.Length -= core::mem::size_of::<WCHAR>() as u16;
    }

    if NT_SUCCESS(Status)
        && (((*(*KeyObject).KeyControlBlock).__bindgen_anon_1.ExtFlags() & CM_KCB_READ_ONLY_KEY) != 0)
    {
        Status = STATUS_ACCESS_DENIED;
    }

    if NT_SUCCESS(Status) {
        PostOperationInfo.Object = KeyObject as PVOID;
        SetValueKeyInfo.Object = KeyObject as PVOID;
        SetValueKeyInfo.ValueName = &mut ValueNameCopy;
        SetValueKeyInfo.TitleIndex = TitleIndex;
        SetValueKeyInfo.Type = Type;
        SetValueKeyInfo.Data = Data;
        SetValueKeyInfo.DataSize = DataSize;

        Status = CmiCallRegisteredCallbacks(
            RegNtPreSetValueKey,
            &mut SetValueKeyInfo as *mut _ as PVOID,
        );
        if NT_SUCCESS(Status) {
            Status = CmSetValueKey(
                (*KeyObject).KeyControlBlock,
                &mut ValueNameCopy,
                Type,
                Data,
                DataSize,
            );

            PostOperationInfo.Status = Status;
            CmiCallRegisteredCallbacks(
                RegNtPostSetValueKey,
                &mut PostOperationInfo as *mut _ as PVOID,
            );
        }
    }

    if PreviousMode != 0i8 && DataSize != 0 && !Data.is_null() {
        ExFreePoolWithTag(Data, TAG_CM);
    }

    if !ValueNameCopy.Buffer.is_null() {
        ReleaseCapturedUnicodeString(&mut ValueNameCopy, PreviousMode);
    }

    ObDereferenceObject(KeyObject as PVOID);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtDeleteValueKey(
    KeyHandle: HANDLE,
    ValueName: PUNICODE_STRING,
) -> NTSTATUS {
    let KeyHandle = opaque_ptr(KeyHandle);
    let ValueName = opaque_ptr(ValueName);
    let mut Status: NTSTATUS;
    let mut KeyObject: PCM_KEY_BODY = core::ptr::null_mut();
    let mut DeleteValueKeyInfo: REG_DELETE_VALUE_KEY_INFORMATION = core::mem::zeroed();
    let mut PostOperationInfo: REG_POST_OPERATION_INFORMATION = core::mem::zeroed();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut ValueNameCopy: _UNICODE_STRING = core::mem::zeroed();

    Status = ObReferenceObjectByHandle(
        KeyHandle,
        KEY_SET_VALUE,
        CmpKeyObjectType,
        PreviousMode,
        &mut KeyObject as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ProbeAndCaptureUnicodeString(&mut ValueNameCopy, PreviousMode, ValueName);
    if !NT_SUCCESS(Status) {
        ObDereferenceObject(KeyObject as *mut _);
        return Status;
    }

    'Quit: loop {
        if (ValueNameCopy.Length & (core::mem::size_of::<WCHAR>() as u16 - 1)) != 0 {
            Status = STATUS_INVALID_PARAMETER;
            break 'Quit;
        }

        if ((*(*KeyObject).KeyControlBlock).__bindgen_anon_1.ExtFlags() & CM_KCB_READ_ONLY_KEY) != 0 {
            Status = STATUS_ACCESS_DENIED;
            break 'Quit;
        }

        DeleteValueKeyInfo.Object = KeyObject as PVOID;
        DeleteValueKeyInfo.ValueName = ValueName;
        Status = CmiCallRegisteredCallbacks(
            RegNtPreDeleteValueKey,
            &mut DeleteValueKeyInfo as *mut _ as PVOID,
        );
        if NT_SUCCESS(Status) {
            Status = CmDeleteValueKey((*KeyObject).KeyControlBlock, ValueNameCopy);

            PostOperationInfo.Object = KeyObject as PVOID;
            PostOperationInfo.Status = Status;
            CmiCallRegisteredCallbacks(
                RegNtPostDeleteValueKey,
                &mut PostOperationInfo as *mut _ as PVOID,
            );
        }
        break 'Quit;
    }

    if !ValueNameCopy.Buffer.is_null() {
        ReleaseCapturedUnicodeString(&mut ValueNameCopy, PreviousMode);
    }

    ObDereferenceObject(KeyObject as *mut _);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtFlushKey(KeyHandle: HANDLE) -> NTSTATUS {
    let KeyHandle = opaque_ptr(KeyHandle);
    let mut Status: NTSTATUS;
    let mut KeyObject: PCM_KEY_BODY = core::ptr::null_mut();

    Status = ObReferenceObjectByHandle(
        KeyHandle,
        0,
        CmpKeyObjectType,
        ExGetPreviousMode(),
        &mut KeyObject as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    CmpLockRegistry();

    CmpAcquireKcbLockShared((*KeyObject).KeyControlBlock);

    if (*(*KeyObject).KeyControlBlock).__bindgen_anon_1.Delete() != 0 {
        Status = STATUS_KEY_DELETED;
    } else {
        Status = CmFlushKey((*KeyObject).KeyControlBlock, 0u8);
    }

    CmpReleaseKcbLock((*KeyObject).KeyControlBlock);
    CmpUnlockRegistry();

    ObDereferenceObject(KeyObject as PVOID);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtLoadKey(
    KeyObjectAttributes: POBJECT_ATTRIBUTES,
    FileObjectAttributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let KeyObjectAttributes = opaque_ptr(KeyObjectAttributes);
    let FileObjectAttributes = opaque_ptr(FileObjectAttributes);
    NtLoadKeyEx(KeyObjectAttributes, FileObjectAttributes, 0, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn NtLoadKey2(
    KeyObjectAttributes: POBJECT_ATTRIBUTES,
    FileObjectAttributes: POBJECT_ATTRIBUTES,
    Flags: u32,
) -> NTSTATUS {
    let KeyObjectAttributes = opaque_ptr(KeyObjectAttributes);
    let FileObjectAttributes = opaque_ptr(FileObjectAttributes);
    NtLoadKeyEx(KeyObjectAttributes, FileObjectAttributes, Flags, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn NtLoadKeyEx(
    TargetKey: POBJECT_ATTRIBUTES,
    SourceFile: POBJECT_ATTRIBUTES,
    Flags: u32,
    TrustClassKey: HANDLE,
) -> NTSTATUS {
    let TargetKey = opaque_ptr(TargetKey);
    let SourceFile = opaque_ptr(SourceFile);
    let TrustClassKey = opaque_ptr(TrustClassKey);
    let mut Status: NTSTATUS;
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut CapturedTargetKey: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut CapturedSourceFile: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut TargetKeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut SourceFileName: _UNICODE_STRING = core::mem::zeroed();
    let mut KmTargetKeyRootDir: HANDLE = core::ptr::null_mut();
    let mut KmSourceFileRootDir: HANDLE = core::ptr::null_mut();
    let mut KeyBody: PCM_KEY_BODY = core::ptr::null_mut();

    if Flags & !REG_NO_LAZY_FLUSH != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if SeSinglePrivilegeCheck(*core::ptr::addr_of!(SeRestorePrivilege), PreviousMode) == 0 {
        return STATUS_PRIVILEGE_NOT_HELD;
    }

    KeEnterCriticalRegion();

    if PreviousMode != 0i8 {
        core::ptr::read_volatile(TargetKey as *const u8);
        core::ptr::read_volatile(SourceFile as *const u8);
    }

    Status = ProbeAndCaptureObjectAttributes(
        &mut CapturedTargetKey,
        &mut TargetKeyName,
        PreviousMode,
        TargetKey,
        1u8,
    );
    if !NT_SUCCESS(Status) {
        KeLeaveCriticalRegion();
        return Status;
    }

    Status = ProbeAndCaptureObjectAttributes(
        &mut CapturedSourceFile,
        &mut SourceFileName,
        PreviousMode,
        SourceFile,
        0u8,
    );
    if !NT_SUCCESS(Status) {
        ReleaseCapturedObjectAttributes(&mut CapturedTargetKey, PreviousMode);
        KeLeaveCriticalRegion();
        return Status;
    }

    Status = CmpConvertHandleToKernelHandle(
        CapturedTargetKey.RootDirectory,
        CmpKeyObjectType,
        KEY_READ,
        PreviousMode,
        &mut KmTargetKeyRootDir,
    );
    if !NT_SUCCESS(Status) {
        ReleaseCapturedObjectAttributes(&mut CapturedSourceFile, PreviousMode);
        ReleaseCapturedObjectAttributes(&mut CapturedTargetKey, PreviousMode);
        KeLeaveCriticalRegion();
        return Status;
    }
    CapturedTargetKey.RootDirectory = KmTargetKeyRootDir;
    CapturedTargetKey.Attributes |= OBJ_KERNEL_HANDLE;

    Status = CmpConvertHandleToKernelHandle(
        CapturedSourceFile.RootDirectory,
        IoFileObjectType,
        FILE_TRAVERSE,
        PreviousMode,
        &mut KmSourceFileRootDir,
    );
    if !NT_SUCCESS(Status) {
        if !KmTargetKeyRootDir.is_null() {
            ObCloseHandle(KmTargetKeyRootDir, 0i8);
        }
        ReleaseCapturedObjectAttributes(&mut CapturedSourceFile, PreviousMode);
        ReleaseCapturedObjectAttributes(&mut CapturedTargetKey, PreviousMode);
        KeLeaveCriticalRegion();
        return Status;
    }
    CapturedSourceFile.RootDirectory = KmSourceFileRootDir;
    CapturedSourceFile.Attributes |= OBJ_KERNEL_HANDLE;

    if !TrustClassKey.is_null() {
        Status = ObReferenceObjectByHandle(
            TrustClassKey,
            0,
            CmpKeyObjectType,
            PreviousMode,
            &mut KeyBody as *mut _ as *mut PVOID,
            core::ptr::null_mut(),
        );
    }

    Status = CmLoadKey(&mut CapturedTargetKey, &mut CapturedSourceFile, Flags, KeyBody);

    if !KeyBody.is_null() {
        ObDereferenceObject(KeyBody as PVOID);
    }
    if !KmSourceFileRootDir.is_null() {
        ObCloseHandle(KmSourceFileRootDir, 0i8);
    }
    if !KmTargetKeyRootDir.is_null() {
        ObCloseHandle(KmTargetKeyRootDir, 0i8);
    }

    ReleaseCapturedObjectAttributes(&mut CapturedSourceFile, PreviousMode);
    ReleaseCapturedObjectAttributes(&mut CapturedTargetKey, PreviousMode);
    KeLeaveCriticalRegion();

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtNotifyChangeKey(
    KeyHandle: HANDLE,
    Event: HANDLE,
    ApcRoutine: PIO_APC_ROUTINE,
    ApcContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    CompletionFilter: ULONG,
    WatchTree: BOOLEAN,
    Buffer: PVOID,
    Length: ULONG,
    Asynchronous: BOOLEAN,
) -> NTSTATUS {
    let KeyHandle = opaque_ptr(KeyHandle);
    let Event = opaque_ptr(Event);
    let ApcContext = opaque_ptr(ApcContext);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let Buffer = opaque_ptr(Buffer);
    NtNotifyChangeMultipleKeys(
        KeyHandle,
        0,
        core::ptr::null_mut(),
        Event,
        ApcRoutine,
        ApcContext,
        IoStatusBlock,
        CompletionFilter,
        WatchTree,
        Buffer,
        Length,
        Asynchronous,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtInitializeRegistry(Flag: USHORT) -> NTSTATUS {
    let SetupBoot: BOOLEAN;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if KeGetPreviousMode() != 0 {
        return ZwInitializeRegistry(Flag);
    }

    Ki386PerfEnd();

    if Flag > CM_BOOT_FLAG_MAX {
        return STATUS_INVALID_PARAMETER;
    }

    if Flag >= CM_BOOT_FLAG_ACCEPTED && Flag <= CM_BOOT_FLAG_MAX {
        if CmBootAcceptFirstTime == 0 {
            return STATUS_ACCESS_DENIED;
        }
        CmBootAcceptFirstTime = 0;

        let AdjustedFlag = Flag - CM_BOOT_FLAG_ACCEPTED;
        if AdjustedFlag != 0 {
            Status = CmpSaveBootControlSet(AdjustedFlag);
            HalEndOfBoot();
            CmpHoldLazyFlush = 0;
            CmpLazyFlush();
            return Status;
        }

        return STATUS_INVALID_PARAMETER;
    }

    SetupBoot = if Flag == CM_BOOT_FLAG_SETUP { 1 } else { 0 };

    if CmFirstTime == 0 {
        return STATUS_ACCESS_DENIED;
    }
    CmFirstTime = 0;

    CmpLockRegistryExclusive();
    CmpCmdInit(SetupBoot);
    CmpSetVersionData();
    CmpUnlockRegistry();

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtCompactKeys(
    Count: u32,
    KeyArray: PHANDLE,
) -> i32 {
    let KeyArray = opaque_ptr(KeyArray);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtCompressKey(Key: HANDLE) -> NTSTATUS {
    let Key = opaque_ptr(Key);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtLockProductActivationKeys(
    pPrivateVer: *mut ULONG,
    pSafeMode: *mut ULONG,
) -> NTSTATUS {
    let pPrivateVer = opaque_ptr(pPrivateVer);
    let pSafeMode = opaque_ptr(pSafeMode);
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();

    if !pPrivateVer.is_null() {
        if PreviousMode != 0i8 {
            ProbeForWriteUlong(pPrivateVer);
        }
        *pPrivateVer = PRODUCT_ACTIVATION_VERSION;
    }

    if !pSafeMode.is_null() {
        if PreviousMode != 0i8 {
            ProbeForWriteUlong(pSafeMode);
        }
        *pSafeMode = InitSafeBootMode;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtLockRegistryKey(KeyHandle: HANDLE) -> NTSTATUS {
    let KeyHandle = opaque_ptr(KeyHandle);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtNotifyChangeMultipleKeys(
    MasterKeyHandle: HANDLE,
    Count: ULONG,
    SlaveObjects: POBJECT_ATTRIBUTES,
    Event: HANDLE,
    ApcRoutine: PIO_APC_ROUTINE,
    ApcContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    CompletionFilter: ULONG,
    WatchTree: BOOLEAN,
    Buffer: PVOID,
    Length: ULONG,
    Asynchronous: BOOLEAN,
) -> NTSTATUS {
    let MasterKeyHandle = opaque_ptr(MasterKeyHandle);
    let SlaveObjects = opaque_ptr(SlaveObjects);
    let Event = opaque_ptr(Event);
    let ApcContext = opaque_ptr(ApcContext);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let Buffer = opaque_ptr(Buffer);
    UNIMPLEMENTED_ONCE!();
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryMultipleValueKey(
    KeyHandle: HANDLE,
    ValueList: PKEY_VALUE_ENTRY,
    NumberOfValues: ULONG,
    Buffer: PVOID,
    Length: PULONG,
    ReturnLength: PULONG,
) -> NTSTATUS {
    let KeyHandle = opaque_ptr(KeyHandle);
    let ValueList = opaque_ptr(ValueList);
    let Buffer = opaque_ptr(Buffer);
    let Length = opaque_ptr(Length);
    let ReturnLength = opaque_ptr(ReturnLength);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryOpenSubKeys(
    TargetKey: POBJECT_ATTRIBUTES,
    HandleCount: *mut ULONG,
) -> NTSTATUS {
    let TargetKey = opaque_ptr(TargetKey);
    let HandleCount = opaque_ptr(HandleCount);
    let mut KeyBody: PCM_KEY_BODY = core::ptr::null_mut();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut SubKeys: ULONG = 0;

    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();

    if PreviousMode != 0i8 {
        if TargetKey as usize % core::mem::align_of::<ULONG>() != 0 
            || (TargetKey as usize).wrapping_add(core::mem::size_of::<_OBJECT_ATTRIBUTES>()) < TargetKey as usize 
            || (TargetKey as usize).wrapping_add(core::mem::size_of::<_OBJECT_ATTRIBUTES>()) > MmUserProbeAddress as usize {
            return STATUS_ACCESS_VIOLATION;
        }

        ProbeForWriteUlong(HandleCount);
    }

    Status = ObOpenObjectByName(
        TargetKey,
        CmpKeyObjectType,
        PreviousMode,
        core::ptr::null_mut(),
        KEY_READ,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(KeyHandle),
    );

    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ObReferenceObjectByHandle(
        KeyHandle,
        KEY_READ,
        CmpKeyObjectType,
        PreviousMode,
        core::ptr::addr_of_mut!(KeyBody) as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    ZwClose(KeyHandle);

    if NT_SUCCESS(Status) {
        CmpLockRegistryExclusive();

        if (*(*KeyBody).KeyControlBlock).__bindgen_anon_2.__bindgen_anon_1.KeyCell
            != (*(*(*KeyBody).KeyControlBlock).__bindgen_anon_2.__bindgen_anon_1.KeyHive)
                .BaseBlock
                .as_ref()
                .unwrap()
                .RootCell
        {
            CmpUnlockRegistry();
            ObDereferenceObject(KeyBody as *mut core::ffi::c_void);
            return STATUS_INVALID_PARAMETER;
        }

        SubKeys = CmpEnumerateOpenSubKeys(
            (*KeyBody).KeyControlBlock,
            0u8,  // RemoveEmptyCacheEntries
            0u8,  // DereferenceOpenedEntries
        );

        CmpUnlockRegistry();

        *HandleCount = SubKeys;

        ObDereferenceObject(KeyBody as *mut core::ffi::c_void);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryOpenSubKeysEx(
    TargetKey: POBJECT_ATTRIBUTES,
    BufferLength: ULONG,
    Buffer: PVOID,
    RequiredSize: PULONG,
) -> NTSTATUS {
    let TargetKey = opaque_ptr(TargetKey);
    let Buffer = opaque_ptr(Buffer);
    let RequiredSize = opaque_ptr(RequiredSize);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtRenameKey(
    KeyHandle: HANDLE,
    ReplacementName: PUNICODE_STRING,
) -> NTSTATUS {
    let KeyHandle = opaque_ptr(KeyHandle);
    let ReplacementName = opaque_ptr(ReplacementName);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtReplaceKey(
    ObjectAttributes: POBJECT_ATTRIBUTES,
    Key: HANDLE,
    ReplacedObjectAttributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let Key = opaque_ptr(Key);
    let ReplacedObjectAttributes = opaque_ptr(ReplacedObjectAttributes);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtRestoreKey(
    KeyHandle: HANDLE,
    FileHandle: HANDLE,
    RestoreFlags: ULONG,
) -> NTSTATUS {
    let KeyHandle = opaque_ptr(KeyHandle);
    let FileHandle = opaque_ptr(FileHandle);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtSaveKey(
    KeyHandle: HANDLE,
    FileHandle: HANDLE,
) -> NTSTATUS {
    let KeyHandle = opaque_ptr(KeyHandle);
    let FileHandle = opaque_ptr(FileHandle);
    NtSaveKeyEx(KeyHandle, FileHandle, REG_STANDARD_FORMAT)
}

#[no_mangle]
pub unsafe extern "system" fn NtSaveKeyEx(
    KeyHandle: HANDLE,
    FileHandle: HANDLE,
    Flags: ULONG,
) -> NTSTATUS {
    let KeyHandle = opaque_ptr(KeyHandle);
    let FileHandle = opaque_ptr(FileHandle);
    let mut Status: NTSTATUS;
    let mut KmFileHandle: HANDLE = core::ptr::null_mut();
    let mut KeyObject: PCM_KEY_BODY = core::ptr::null_mut();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();

    if (Flags != REG_STANDARD_FORMAT)
        && (Flags != REG_LATEST_FORMAT)
        && (Flags != REG_NO_COMPRESSION)
    {
        return STATUS_INVALID_PARAMETER;
    }

    if SeSinglePrivilegeCheck(SeBackupPrivilege, PreviousMode) == 0 {
        return STATUS_PRIVILEGE_NOT_HELD;
    }

    Status = CmpConvertHandleToKernelHandle(
        FileHandle,
        IoFileObjectType,
        FILE_WRITE_DATA,
        PreviousMode,
        &mut KmFileHandle,
    );
    if !NT_SUCCESS(Status) {
        'Quit: loop {
            if !KmFileHandle.is_null() {
                ObCloseHandle(KmFileHandle, 0i8);
            }
            return Status;
        }
    }

    Status = ObReferenceObjectByHandle(
        KeyHandle,
        KEY_READ,
        CmpKeyObjectType,
        PreviousMode,
        &mut KeyObject as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        'Quit: loop {
            if !KmFileHandle.is_null() {
                ObCloseHandle(KmFileHandle, 0i8);
            }
            return Status;
        }
    }

    Status = CmSaveKey((*KeyObject).KeyControlBlock, KmFileHandle, Flags);

    ObDereferenceObject(KeyObject as PVOID);

    if !KmFileHandle.is_null() {
        ObCloseHandle(KmFileHandle, 0i8);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSaveMergedKeys(
    HighPrecedenceKeyHandle: HANDLE,
    LowPrecedenceKeyHandle: HANDLE,
    FileHandle: HANDLE,
) -> NTSTATUS {
    let HighPrecedenceKeyHandle = opaque_ptr(HighPrecedenceKeyHandle);
    let LowPrecedenceKeyHandle = opaque_ptr(LowPrecedenceKeyHandle);
    let FileHandle = opaque_ptr(FileHandle);
    let mut Status: NTSTATUS;
    let PreviousMode: KPROCESSOR_MODE;
    let mut KmFileHandle: HANDLE = core::ptr::null_mut();
    let mut HighPrecedenceKeyObject: PCM_KEY_BODY = core::ptr::null_mut();
    let mut LowPrecedenceKeyObject: PCM_KEY_BODY = core::ptr::null_mut();

    PreviousMode = ExGetPreviousMode();

    if SeSinglePrivilegeCheck(*core::ptr::addr_of!(SeBackupPrivilege), PreviousMode) == 0 {
        return STATUS_PRIVILEGE_NOT_HELD;
    }

    Status = CmpConvertHandleToKernelHandle(
        FileHandle,
        IoFileObjectType,
        FILE_WRITE_DATA,
        PreviousMode,
        core::ptr::addr_of_mut!(KmFileHandle),
    );
    if !NT_SUCCESS(Status) {
        Status
    } else {
        Status = ObReferenceObjectByHandle(
            HighPrecedenceKeyHandle,
            KEY_READ,
            CmpKeyObjectType,
            PreviousMode,
            core::ptr::addr_of_mut!(HighPrecedenceKeyObject) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            Status
        } else {
            Status = ObReferenceObjectByHandle(
                LowPrecedenceKeyHandle,
                KEY_READ,
                CmpKeyObjectType,
                PreviousMode,
                core::ptr::addr_of_mut!(LowPrecedenceKeyObject) as *mut PVOID,
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(Status) {
                Status
            } else {
                Status = CmSaveMergedKeys(
                    (*HighPrecedenceKeyObject).KeyControlBlock,
                    (*LowPrecedenceKeyObject).KeyControlBlock,
                    KmFileHandle,
                );
                Status
            }
        }
    };

    if !LowPrecedenceKeyObject.is_null() {
        ObDereferenceObject(LowPrecedenceKeyObject as PVOID);
    }
    if !HighPrecedenceKeyObject.is_null() {
        ObDereferenceObject(HighPrecedenceKeyObject as PVOID);
    }

    if !KmFileHandle.is_null() {
        ObCloseHandle(KmFileHandle, 0i8);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetInformationKey(
    KeyHandle: HANDLE,
    KeyInformationClass: KEY_SET_INFORMATION_CLASS,
    KeyInformation: PVOID,
    KeyInformationLength: ULONG,
) -> NTSTATUS {
    let KeyHandle = opaque_ptr(KeyHandle);
    let KeyInformation = opaque_ptr(KeyInformation);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtUnloadKey(
    KeyObjectAttributes: POBJECT_ATTRIBUTES,
) -> i32 {
    let KeyObjectAttributes = opaque_ptr(KeyObjectAttributes);
    NtUnloadKey2(KeyObjectAttributes, 0)
}

#[no_mangle]
pub unsafe extern "system" fn NtUnloadKey2(
    TargetKey: POBJECT_ATTRIBUTES,
    Flags: u32,
) -> i32 {
    let TargetKey = opaque_ptr(TargetKey);
    let mut Status: i32;
    let mut CapturedTargetKey: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut ObjectName: _UNICODE_STRING = core::mem::zeroed();
    let mut KmTargetKeyRootDir: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ParseContext: CM_PARSE_CONTEXT = core::mem::zeroed();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut KeyBody: PCM_KEY_BODY = core::ptr::null_mut();
    let mut Handle: HANDLE = core::ptr::null_mut();

    if SeSinglePrivilegeCheck(SeRestorePrivilege, PreviousMode) == 0 {
        return STATUS_PRIVILEGE_NOT_HELD;
    }

    if PreviousMode != 0 {
        if (TargetKey as usize) < 0x80000000 {
            if (TargetKey as usize).wrapping_add(core::mem::size_of::<_OBJECT_ATTRIBUTES>()) > 0x80000000 
                || (TargetKey as usize).wrapping_add(core::mem::size_of::<_OBJECT_ATTRIBUTES>()) < (TargetKey as usize) {
                return STATUS_ACCESS_VIOLATION;
            }
        }

        CapturedTargetKey = *TargetKey;

        ObjectName = ProbeForReadUnicodeString(CapturedTargetKey.ObjectName);
        
        if (ObjectName.Buffer as usize) < 0x80000000 {
            if (ObjectName.Buffer as usize).wrapping_add(ObjectName.Length as usize) > 0x80000000 
                || (ObjectName.Buffer as usize).wrapping_add(ObjectName.Length as usize) < (ObjectName.Buffer as usize) {
                return STATUS_ACCESS_VIOLATION;
            }
        }

        CapturedTargetKey.ObjectName = core::ptr::addr_of_mut!(ObjectName);
    } else {
        CapturedTargetKey = *TargetKey;
    }

    Status = CmpConvertHandleToKernelHandle(
        CapturedTargetKey.RootDirectory,
        CmpKeyObjectType,
        KEY_WRITE,
        PreviousMode,
        &mut KmTargetKeyRootDir,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }
    CapturedTargetKey.RootDirectory = KmTargetKeyRootDir;
    CapturedTargetKey.Attributes |= OBJ_KERNEL_HANDLE;

    ParseContext.CreateOperation = 1u8;
    ParseContext.CreateOptions = REG_OPTION_BACKUP_RESTORE;

    Status = ObOpenObjectByName(
        &mut CapturedTargetKey,
        CmpKeyObjectType,
        0i8,
        core::ptr::null_mut(),
        KEY_WRITE,
        &mut ParseContext as *mut _ as *mut core::ffi::c_void,
        &mut Handle,
    );
    if NT_SUCCESS(Status) {
        Status = ObReferenceObjectByHandle(
            Handle,
            KEY_WRITE,
            CmpKeyObjectType,
            0i8,
            &mut KeyBody as *mut _ as *mut PVOID,
            core::ptr::null_mut(),
        );
        ObCloseHandle(Handle, 0i8);
    }

    if !KmTargetKeyRootDir.is_null() {
        ObCloseHandle(KmTargetKeyRootDir, 0i8);
    }

    if !NT_SUCCESS(Status) {
        return Status;
    }

    CmpLockRegistryExclusive();
    ExAcquirePushLockExclusive(&raw mut CmpLoadHiveLock);

    if (*(*KeyBody).KeyControlBlock).__bindgen_anon_1.Delete() != 0 {
        Status = STATUS_KEY_DELETED;
    } else if ((*(*KeyBody).KeyControlBlock).__bindgen_anon_1.ExtFlags() & CM_KCB_READ_ONLY_KEY) != 0 {
        Status = STATUS_ACCESS_DENIED;
    } else {
        Status = CmUnloadKey((*KeyBody).KeyControlBlock, Flags);
    }

    if !NT_SUCCESS(Status) {
        ExReleasePushLockExclusive(&raw mut CmpLoadHiveLock);
        CmpUnlockRegistry();
    }

    ObDereferenceObject(KeyBody as PVOID);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtUnloadKeyEx(
    TargetKey: POBJECT_ATTRIBUTES,
    Event: HANDLE,
) -> NTSTATUS {
    let TargetKey = opaque_ptr(TargetKey);
    let Event = opaque_ptr(Event);
    STATUS_NOT_IMPLEMENTED
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/config/ntapi.c
//  * PURPOSE:         Configuration Manager - Internal Registry APIs
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Eric Kohl
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "ntoskrnl.h"
// #define NDEBUG
// #include "debug.h"
//
// BOOLEAN CmBootAcceptFirstTime = TRUE;
// BOOLEAN CmFirstTime = TRUE;
// extern ULONG InitSafeBootMode;
//
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// /*
//  * Adapted from ntoskrnl/include/internal/ob_x.h:ObpReleaseObjectCreateInformation()
//  */
// VOID
// ReleaseCapturedObjectAttributes(
//     _In_ POBJECT_ATTRIBUTES CapturedObjectAttributes,
//     _In_ KPROCESSOR_MODE AccessMode)
// {
//     /* Check if we have a security descriptor */
//     if (CapturedObjectAttributes->SecurityDescriptor)
//     {
//         /* Release it */
//         SeReleaseSecurityDescriptor(CapturedObjectAttributes->SecurityDescriptor,
//                                     AccessMode,
//                                     TRUE);
//         CapturedObjectAttributes->SecurityDescriptor = NULL;
//     }
//
//     /* Check if we have an object name */
//     if (CapturedObjectAttributes->ObjectName)
//     {
//         /* Release it */
//         ReleaseCapturedUnicodeString(CapturedObjectAttributes->ObjectName, AccessMode);
//     }
// }
//
// /*
//  * Adapted from ntoskrnl/ob/oblife.c:ObpCaptureObjectCreateInformation()
//  */
// NTSTATUS
// ProbeAndCaptureObjectAttributes(
//     _Out_ POBJECT_ATTRIBUTES CapturedObjectAttributes,
//     _Out_ PUNICODE_STRING ObjectName,
//     _In_ KPROCESSOR_MODE AccessMode,
//     _In_ POBJECT_ATTRIBUTES ObjectAttributes,
//     _In_ BOOLEAN CaptureSecurity)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PSECURITY_DESCRIPTOR SecurityDescriptor;
//     // PSECURITY_QUALITY_OF_SERVICE SecurityQos;
//     PUNICODE_STRING LocalObjectName = NULL;
//
//     /* Zero out the Capture Data */
//     RtlZeroMemory(CapturedObjectAttributes, sizeof(*CapturedObjectAttributes));
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
//                 (ObjectAttributes->Attributes & ~OBJ_VALID_KERNEL_ATTRIBUTES))  // Understood as all the possible valid attributes
//             {
//                 /* Invalid combination, fail */
//                 _SEH2_YIELD(return STATUS_INVALID_PARAMETER);
//             }
//
//             /* Set some Create Info and do not allow user-mode kernel handles */
//             CapturedObjectAttributes->Length = sizeof(OBJECT_ATTRIBUTES);
//             CapturedObjectAttributes->RootDirectory = ObjectAttributes->RootDirectory;
//             CapturedObjectAttributes->Attributes = ObpValidateAttributes(ObjectAttributes->Attributes, AccessMode);
//             LocalObjectName = ObjectAttributes->ObjectName;
//             SecurityDescriptor = ObjectAttributes->SecurityDescriptor;
//             // SecurityQos = ObjectAttributes->SecurityQualityOfService;
//
//             /* Check if we have a security descriptor */
//             if (CaptureSecurity && SecurityDescriptor)
//             {
//                 /*
//                  * Capture it.
//                  * Note: This has an implicit memory barrier due
//                  * to the function call, so cleanup is safe here.
//                  */
//                 Status = SeCaptureSecurityDescriptor(SecurityDescriptor,
//                                                      AccessMode,
//                                                      NonPagedPool,
//                                                      TRUE,
//                                                      &CapturedObjectAttributes->
//                                                         SecurityDescriptor);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     /* Capture failed, quit */
//                     CapturedObjectAttributes->SecurityDescriptor = NULL;
//                     _SEH2_YIELD(return Status);
//                 }
//             }
//             else
//             {
//                 CapturedObjectAttributes->SecurityDescriptor = NULL;
//             }
//
// #if 0
// // We don't use the QoS!
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
//                 CapturedObjectAttributes->SecurityQualityOfService = *SecurityQos;
//                 CapturedObjectAttributes->SecurityQos =
//                     &CapturedObjectAttributes->SecurityQualityOfService;
//             }
// #else
//             CapturedObjectAttributes->SecurityQualityOfService = NULL;
// #endif
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
//         ReleaseCapturedObjectAttributes(CapturedObjectAttributes, AccessMode);
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     /* Now check if the Object Attributes had an Object Name */
//     if (LocalObjectName)
//     {
//         Status = ProbeAndCaptureUnicodeString(ObjectName, AccessMode, LocalObjectName);
//     }
//     else
//     {
//         /* Clear the string */
//         RtlInitEmptyUnicodeString(ObjectName, NULL, 0);
//
//         /* It cannot have specified a Root Directory */
//         if (CapturedObjectAttributes->RootDirectory)
//         {
//             Status = STATUS_OBJECT_NAME_INVALID;
//         }
//     }
//
//     /* Set the caputured object attributes name pointer to the one the user gave to us */
//     CapturedObjectAttributes->ObjectName = ObjectName;
//
//     /* Cleanup if we failed */
//     if (!NT_SUCCESS(Status))
//     {
//         ReleaseCapturedObjectAttributes(CapturedObjectAttributes, AccessMode);
//     }
//
//     /* Return status to caller */
//     return Status;
// }
//
// static
// NTSTATUS
// CmpConvertHandleToKernelHandle(
//     _In_ HANDLE SourceHandle,
//     _In_opt_ POBJECT_TYPE ObjectType,
//     _In_ ACCESS_MASK DesiredAccess,
//     _In_ KPROCESSOR_MODE AccessMode,
//     _Out_ PHANDLE KernelHandle)
// {
//     NTSTATUS Status;
//     PVOID Object;
//
//     *KernelHandle = NULL;
//
//     /* NULL handle is valid */
//     if (SourceHandle == NULL)
//         return STATUS_SUCCESS;
//
//     /* Get the object pointer */
//     Status = ObReferenceObjectByHandle(SourceHandle,
//                                        DesiredAccess,
//                                        ObjectType,
//                                        AccessMode,
//                                        &Object,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Create a kernel handle from the pointer */
//     Status = ObOpenObjectByPointer(Object,
//                                    OBJ_KERNEL_HANDLE,
//                                    NULL,
//                                    DesiredAccess,
//                                    ObjectType,
//                                    KernelMode,
//                                    KernelHandle);
//
//     /* Dereference the object */
//     ObDereferenceObject(Object);
//     return Status;
// }
//
//
// /* FUNCTIONS *****************************************************************/
//
// NTSTATUS
// NTAPI
// NtCreateKey(OUT PHANDLE KeyHandle,
//             IN ACCESS_MASK DesiredAccess,
//             IN POBJECT_ATTRIBUTES ObjectAttributes,
//             IN ULONG TitleIndex,
//             IN PUNICODE_STRING Class OPTIONAL,
//             IN ULONG CreateOptions,
//             OUT PULONG Disposition OPTIONAL)
// {
//     NTSTATUS Status;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     CM_PARSE_CONTEXT ParseContext = {0};
//     HANDLE Handle;
//     PAGED_CODE();
//
//     DPRINT("NtCreateKey(Path: %wZ, Root %x, Access: %x, CreateOptions %x)\n",
//             ObjectAttributes->ObjectName, ObjectAttributes->RootDirectory,
//             DesiredAccess, CreateOptions);
//
//     /* Ignore the WOW64 flag, it's not valid in the kernel */
//     DesiredAccess &= ~KEY_WOW64_RES;
//
//     /* Check for user-mode caller */
//     if (PreviousMode != KernelMode)
//     {
//         /* Prepare to probe parameters */
//         _SEH2_TRY
//         {
//             /* Check if we have a class */
//             if (Class)
//             {
//                 /* Probe it */
//                 ParseContext.Class = ProbeForReadUnicodeString(Class);
//                 ProbeForRead(ParseContext.Class.Buffer,
//                              ParseContext.Class.Length,
//                              sizeof(WCHAR));
//             }
//
//             /* Probe the key handle */
//             ProbeForWriteHandle(KeyHandle);
//             *KeyHandle = NULL;
//
//             /* Probe object attributes */
//             ProbeForRead(ObjectAttributes,
//                          sizeof(OBJECT_ATTRIBUTES),
//                          sizeof(ULONG));
//
//             if (Disposition)
//                 ProbeForWriteUlong(Disposition);
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
//         /* Save the class directly */
//         if (Class) ParseContext.Class = *Class;
//     }
//
//     /* Setup the parse context */
//     ParseContext.CreateOperation = TRUE;
//     ParseContext.CreateOptions = CreateOptions;
//
//     /* Do the create */
//     Status = ObOpenObjectByName(ObjectAttributes,
//                                 CmpKeyObjectType,
//                                 PreviousMode,
//                                 NULL,
//                                 DesiredAccess,
//                                 &ParseContext,
//                                 &Handle);
//
//     _SEH2_TRY
//     {
//         /* Return data to user */
//         if (NT_SUCCESS(Status)) *KeyHandle = Handle;
//         if (Disposition) *Disposition = ParseContext.Disposition;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Get the status */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     DPRINT("Returning handle %x, Status %x.\n", Handle, Status);
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtOpenKey(OUT PHANDLE KeyHandle,
//           IN ACCESS_MASK DesiredAccess,
//           IN POBJECT_ATTRIBUTES ObjectAttributes)
// {
//     CM_PARSE_CONTEXT ParseContext = {0};
//     HANDLE Handle;
//     NTSTATUS Status;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     PAGED_CODE();
//     DPRINT("NtOpenKey(Path: %wZ, Root %x, Access: %x)\n",
//             ObjectAttributes->ObjectName, ObjectAttributes->RootDirectory, DesiredAccess);
//
//     /* Ignore the WOW64 flag, it's not valid in the kernel */
//     DesiredAccess &= ~KEY_WOW64_RES;
//
//     /* Check for user-mode caller */
//     if (PreviousMode != KernelMode)
//     {
//         /* Prepare to probe parameters */
//         _SEH2_TRY
//         {
//             /* Probe the key handle */
//             ProbeForWriteHandle(KeyHandle);
//             *KeyHandle = NULL;
//
//             /* Probe object attributes */
//             ProbeForRead(ObjectAttributes,
//                          sizeof(OBJECT_ATTRIBUTES),
//                          sizeof(ULONG));
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Just let the object manager handle this */
//     Status = ObOpenObjectByName(ObjectAttributes,
//                                 CmpKeyObjectType,
//                                 PreviousMode,
//                                 NULL,
//                                 DesiredAccess,
//                                 &ParseContext,
//                                 &Handle);
//
//     /* Only do this if we succeeded */
//     if (NT_SUCCESS(Status))
//     {
//         _SEH2_TRY
//         {
//             /* Return the handle to caller */
//             *KeyHandle = Handle;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Get the status */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//
//     DPRINT("Returning handle %x, Status %x.\n", Handle, Status);
//
//     /* Return status */
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// NtDeleteKey(IN HANDLE KeyHandle)
// {
//     PCM_KEY_BODY KeyObject;
//     NTSTATUS Status;
//     REG_DELETE_KEY_INFORMATION DeleteKeyInfo;
//     REG_POST_OPERATION_INFORMATION PostOperationInfo;
//     PAGED_CODE();
//     DPRINT("NtDeleteKey(KH 0x%p)\n", KeyHandle);
//
//     /* Verify that the handle is valid and is a registry key */
//     Status = ObReferenceObjectByHandle(KeyHandle,
//                                        DELETE,
//                                        CmpKeyObjectType,
//                                        ExGetPreviousMode(),
//                                        (PVOID*)&KeyObject,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Setup the callback */
//     PostOperationInfo.Object = (PVOID)KeyObject;
//     DeleteKeyInfo.Object = (PVOID)KeyObject;
//     Status = CmiCallRegisteredCallbacks(RegNtPreDeleteKey, &DeleteKeyInfo);
//     if (NT_SUCCESS(Status))
//     {
//         /* Check if we are read-only */
//         if ((KeyObject->KeyControlBlock->ExtFlags & CM_KCB_READ_ONLY_KEY) ||
//             (KeyObject->KeyControlBlock->ParentKcb->ExtFlags & CM_KCB_READ_ONLY_KEY))
//         {
//             /* Fail */
//             Status = STATUS_ACCESS_DENIED;
//         }
//         else
//         {
//             /* Call the internal API */
//             Status = CmDeleteKey(KeyObject);
//         }
//
//         /* Do post callback */
//         PostOperationInfo.Status = Status;
//         CmiCallRegisteredCallbacks(RegNtPostDeleteKey, &PostOperationInfo);
//     }
//
//     /* Dereference and return status */
//     ObDereferenceObject(KeyObject);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtEnumerateKey(IN HANDLE KeyHandle,
//                IN ULONG Index,
//                IN KEY_INFORMATION_CLASS KeyInformationClass,
//                OUT PVOID KeyInformation,
//                IN ULONG Length,
//                OUT PULONG ResultLength)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PCM_KEY_BODY KeyObject;
//     REG_ENUMERATE_KEY_INFORMATION EnumerateKeyInfo;
//     REG_POST_OPERATION_INFORMATION PostOperationInfo;
//     PAGED_CODE();
//     DPRINT("NtEnumerateKey() KH 0x%p, Index 0x%x, KIC %d, Length %lu\n",
//            KeyHandle, Index, KeyInformationClass, Length);
//
//     /* Reject classes we don't know about */
//     if ((KeyInformationClass != KeyBasicInformation) &&
//         (KeyInformationClass != KeyNodeInformation)  &&
//         (KeyInformationClass != KeyFullInformation))
//     {
//         /* Fail */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Verify that the handle is valid and is a registry key */
//     Status = ObReferenceObjectByHandle(KeyHandle,
//                                        KEY_ENUMERATE_SUB_KEYS,
//                                        CmpKeyObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&KeyObject,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             ProbeForWriteUlong(ResultLength);
//             ProbeForWrite(KeyInformation,
//                           Length,
//                           sizeof(ULONG));
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Dereference and return status */
//             ObDereferenceObject(KeyObject);
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Setup the callback */
//     PostOperationInfo.Object = (PVOID)KeyObject;
//     EnumerateKeyInfo.Object = (PVOID)KeyObject;
//     EnumerateKeyInfo.Index = Index;
//     EnumerateKeyInfo.KeyInformationClass = KeyInformationClass;
//     EnumerateKeyInfo.Length = Length;
//     EnumerateKeyInfo.ResultLength = ResultLength;
//
//     /* Do the callback */
//     Status = CmiCallRegisteredCallbacks(RegNtPreEnumerateKey, &EnumerateKeyInfo);
//     if (NT_SUCCESS(Status))
//     {
//         /* Call the internal API */
//         Status = CmEnumerateKey(KeyObject->KeyControlBlock,
//                                 Index,
//                                 KeyInformationClass,
//                                 KeyInformation,
//                                 Length,
//                                 ResultLength);
//
//         /* Do the post callback */
//         PostOperationInfo.Status = Status;
//         CmiCallRegisteredCallbacks(RegNtPostEnumerateKey, &PostOperationInfo);
//     }
//
//     /* Dereference and return status */
//     ObDereferenceObject(KeyObject);
//     DPRINT("Returning status %x.\n", Status);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtEnumerateValueKey(IN HANDLE KeyHandle,
//                     IN ULONG Index,
//                     IN KEY_VALUE_INFORMATION_CLASS KeyValueInformationClass,
//                     OUT PVOID KeyValueInformation,
//                     IN ULONG Length,
//                     OUT PULONG ResultLength)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PCM_KEY_BODY KeyObject;
//     REG_ENUMERATE_VALUE_KEY_INFORMATION EnumerateValueKeyInfo;
//     REG_POST_OPERATION_INFORMATION PostOperationInfo;
//
//     PAGED_CODE();
//
//     DPRINT("NtEnumerateValueKey() KH 0x%p, Index 0x%x, KVIC %d, Length %lu\n",
//            KeyHandle, Index, KeyValueInformationClass, Length);
//
//     /* Reject classes we don't know about */
//     if ((KeyValueInformationClass != KeyValueBasicInformation)       &&
//         (KeyValueInformationClass != KeyValueFullInformation)        &&
//         (KeyValueInformationClass != KeyValuePartialInformation)     &&
//         (KeyValueInformationClass != KeyValueFullInformationAlign64) &&
//         (KeyValueInformationClass != KeyValuePartialInformationAlign64))
//     {
//         /* Fail */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Verify that the handle is valid and is a registry key */
//     Status = ObReferenceObjectByHandle(KeyHandle,
//                                        KEY_QUERY_VALUE,
//                                        CmpKeyObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&KeyObject,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             ProbeForWriteUlong(ResultLength);
//             ProbeForWrite(KeyValueInformation,
//                           Length,
//                           sizeof(ULONG));
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Dereference and return status */
//             ObDereferenceObject(KeyObject);
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Setup the callback */
//     PostOperationInfo.Object = (PVOID)KeyObject;
//     EnumerateValueKeyInfo.Object = (PVOID)KeyObject;
//     EnumerateValueKeyInfo.Index = Index;
//     EnumerateValueKeyInfo.KeyValueInformationClass = KeyValueInformationClass;
//     EnumerateValueKeyInfo.KeyValueInformation = KeyValueInformation;
//     EnumerateValueKeyInfo.Length = Length;
//     EnumerateValueKeyInfo.ResultLength = ResultLength;
//
//     /* Do the callback */
//     Status = CmiCallRegisteredCallbacks(RegNtPreEnumerateValueKey,
//                                         &EnumerateValueKeyInfo);
//     if (NT_SUCCESS(Status))
//     {
//         /* Call the internal API */
//         Status = CmEnumerateValueKey(KeyObject->KeyControlBlock,
//                                      Index,
//                                      KeyValueInformationClass,
//                                      KeyValueInformation,
//                                      Length,
//                                      ResultLength);
//
//         /* Do the post callback */
//         PostOperationInfo.Status = Status;
//         CmiCallRegisteredCallbacks(RegNtPostEnumerateValueKey, &PostOperationInfo);
//     }
//
//     /* Dereference and return status */
//     ObDereferenceObject(KeyObject);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtQueryKey(IN HANDLE KeyHandle,
//            IN KEY_INFORMATION_CLASS KeyInformationClass,
//            OUT PVOID KeyInformation,
//            IN ULONG Length,
//            OUT PULONG ResultLength)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PCM_KEY_BODY KeyObject;
//     REG_QUERY_KEY_INFORMATION QueryKeyInfo;
//     REG_POST_OPERATION_INFORMATION PostOperationInfo;
//     OBJECT_HANDLE_INFORMATION HandleInfo;
//     PAGED_CODE();
//     DPRINT("NtQueryKey() KH 0x%p, KIC %d, Length %lu\n",
//            KeyHandle, KeyInformationClass, Length);
//
//     /* Reject invalid classes */
//     if ((KeyInformationClass != KeyBasicInformation) &&
//         (KeyInformationClass != KeyNodeInformation)  &&
//         (KeyInformationClass != KeyFullInformation)  &&
//         (KeyInformationClass != KeyNameInformation) &&
//         (KeyInformationClass != KeyCachedInformation) &&
//         (KeyInformationClass != KeyFlagsInformation))
//     {
//         /* Fail */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Check if just the name is required */
//     if (KeyInformationClass == KeyNameInformation)
//     {
//         /* Ignore access level */
//         Status = ObReferenceObjectByHandle(KeyHandle,
//                                            0,
//                                            CmpKeyObjectType,
//                                            PreviousMode,
//                                            (PVOID*)&KeyObject,
//                                            &HandleInfo);
//         if (NT_SUCCESS(Status))
//         {
//             /* At least a single bit of access is required */
//             if (!HandleInfo.GrantedAccess)
//             {
//                 /* No such luck */
//                 ObDereferenceObject(KeyObject);
//                 Status = STATUS_ACCESS_DENIED;
//             }
//         }
//     }
//     else
//     {
//         /* Get a reference */
//         Status = ObReferenceObjectByHandle(KeyHandle,
//                                            KEY_QUERY_VALUE,
//                                            CmpKeyObjectType,
//                                            PreviousMode,
//                                            (PVOID*)&KeyObject,
//                                            NULL);
//     }
//
//     /* Quit on failure */
//     if (!NT_SUCCESS(Status)) return Status;
//
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             ProbeForWriteUlong(ResultLength);
//             ProbeForWrite(KeyInformation,
//                           Length,
//                           sizeof(ULONG));
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Dereference and return status */
//             ObDereferenceObject(KeyObject);
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Setup the callback */
//     PostOperationInfo.Object = (PVOID)KeyObject;
//     QueryKeyInfo.Object = (PVOID)KeyObject;
//     QueryKeyInfo.KeyInformationClass = KeyInformationClass;
//     QueryKeyInfo.KeyInformation = KeyInformation;
//     QueryKeyInfo.Length = Length;
//     QueryKeyInfo.ResultLength = ResultLength;
//
//     /* Do the callback */
//     Status = CmiCallRegisteredCallbacks(RegNtPreQueryKey, &QueryKeyInfo);
//     if (NT_SUCCESS(Status))
//     {
//         /* Call the internal API */
//         Status = CmQueryKey(KeyObject->KeyControlBlock,
//                             KeyInformationClass,
//                             KeyInformation,
//                             Length,
//                             ResultLength);
//
//         /* Do the post callback */
//         PostOperationInfo.Status = Status;
//         CmiCallRegisteredCallbacks(RegNtPostQueryKey, &PostOperationInfo);
//     }
//
//     /* Dereference and return status */
//     ObDereferenceObject(KeyObject);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtQueryValueKey(IN HANDLE KeyHandle,
//                 IN PUNICODE_STRING ValueName,
//                 IN KEY_VALUE_INFORMATION_CLASS KeyValueInformationClass,
//                 OUT PVOID KeyValueInformation,
//                 IN ULONG Length,
//                 OUT PULONG ResultLength)
// {
//     NTSTATUS Status;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     PCM_KEY_BODY KeyObject;
//     REG_QUERY_VALUE_KEY_INFORMATION QueryValueKeyInfo;
//     REG_POST_OPERATION_INFORMATION PostOperationInfo;
//     UNICODE_STRING ValueNameCopy;
//
//     PAGED_CODE();
//
//     DPRINT("NtQueryValueKey() KH 0x%p, VN '%wZ', KVIC %d, Length %lu\n",
//         KeyHandle, ValueName, KeyValueInformationClass, Length);
//
//     /* Reject classes we don't know about */
//     if ((KeyValueInformationClass != KeyValueBasicInformation)       &&
//         (KeyValueInformationClass != KeyValueFullInformation)        &&
//         (KeyValueInformationClass != KeyValuePartialInformation)     &&
//         (KeyValueInformationClass != KeyValueFullInformationAlign64) &&
//         (KeyValueInformationClass != KeyValuePartialInformationAlign64))
//     {
//         /* Fail */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Verify that the handle is valid and is a registry key */
//     Status = ObReferenceObjectByHandle(KeyHandle,
//                                        KEY_QUERY_VALUE,
//                                        CmpKeyObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&KeyObject,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             ProbeForWriteUlong(ResultLength);
//             ProbeForWrite(KeyValueInformation,
//                           Length,
//                           sizeof(ULONG));
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Dereference and return status */
//             ObDereferenceObject(KeyObject);
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Capture the string */
//     Status = ProbeAndCaptureUnicodeString(&ValueNameCopy, PreviousMode, ValueName);
//     if (!NT_SUCCESS(Status))
//         goto Quit;
//
//     /* Make sure the name is aligned properly */
//     if (ValueNameCopy.Length & (sizeof(WCHAR) - 1))
//     {
//         /* It isn't, so we'll fail */
//         Status = STATUS_INVALID_PARAMETER;
//         goto Quit;
//     }
//
//     /* Ignore any null characters at the end */
//     while (ValueNameCopy.Length &&
//            !(ValueNameCopy.Buffer[ValueNameCopy.Length / sizeof(WCHAR) - 1]))
//     {
//         /* Skip it */
//         ValueNameCopy.Length -= sizeof(WCHAR);
//     }
//
//     /* Setup the callback */
//     PostOperationInfo.Object = (PVOID)KeyObject;
//     QueryValueKeyInfo.Object = (PVOID)KeyObject;
//     QueryValueKeyInfo.ValueName = &ValueNameCopy;
//     QueryValueKeyInfo.KeyValueInformationClass = KeyValueInformationClass;
//     QueryValueKeyInfo.Length = Length;
//     QueryValueKeyInfo.ResultLength = ResultLength;
//
//     /* Do the callback */
//     Status = CmiCallRegisteredCallbacks(RegNtPreQueryValueKey, &QueryValueKeyInfo);
//     if (NT_SUCCESS(Status))
//     {
//         /* Call the internal API */
//         Status = CmQueryValueKey(KeyObject->KeyControlBlock,
//                                  ValueNameCopy,
//                                  KeyValueInformationClass,
//                                  KeyValueInformation,
//                                  Length,
//                                  ResultLength);
//
//         /* Do the post callback */
//         PostOperationInfo.Status = Status;
//         CmiCallRegisteredCallbacks(RegNtPostQueryValueKey, &PostOperationInfo);
//     }
//
// Quit:
//     if (ValueNameCopy.Buffer)
//         ReleaseCapturedUnicodeString(&ValueNameCopy, PreviousMode);
//
//     /* Dereference and return status */
//     ObDereferenceObject(KeyObject);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtSetValueKey(IN HANDLE KeyHandle,
//               IN PUNICODE_STRING ValueName,
//               IN ULONG TitleIndex,
//               IN ULONG Type,
//               IN PVOID Data,
//               IN ULONG DataSize)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     KPROCESSOR_MODE PreviousMode;
//     PCM_KEY_BODY KeyObject;
//     REG_SET_VALUE_KEY_INFORMATION SetValueKeyInfo;
//     REG_POST_OPERATION_INFORMATION PostOperationInfo;
//     UNICODE_STRING ValueNameCopy;
//
//     PAGED_CODE();
//
//     PreviousMode = ExGetPreviousMode();
//
//     /* Verify that the handle is valid and is a registry key */
//     Status = ObReferenceObjectByHandle(KeyHandle,
//                                        KEY_SET_VALUE,
//                                        CmpKeyObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&KeyObject,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     if (!DataSize)
//         Data = NULL;
//
//     /* Probe and copy the data */
//     if ((PreviousMode != KernelMode) && (DataSize != 0))
//     {
//         PVOID DataCopy = NULL;
//
//         _SEH2_TRY
//         {
//             ProbeForRead(Data, DataSize, 1);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//
//         if (!NT_SUCCESS(Status))
//         {
//             /* Dereference and return status */
//             ObDereferenceObject(KeyObject);
//             return Status;
//         }
//
//         DataCopy = ExAllocatePoolWithTag(PagedPool, DataSize, TAG_CM);
//         if (!DataCopy)
//         {
//             /* Dereference and return status */
//             ObDereferenceObject(KeyObject);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         _SEH2_TRY
//         {
//             RtlCopyMemory(DataCopy, Data, DataSize);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//
//         if (!NT_SUCCESS(Status))
//         {
//             /* Dereference and return status */
//             ExFreePoolWithTag(DataCopy, TAG_CM);
//             ObDereferenceObject(KeyObject);
//             return Status;
//         }
//
//         Data = DataCopy;
//     }
//
//     /* Capture the string */
//     Status = ProbeAndCaptureUnicodeString(&ValueNameCopy, PreviousMode, ValueName);
//     if (!NT_SUCCESS(Status))
//         goto Quit;
//
//     DPRINT("NtSetValueKey() KH 0x%p, VN '%wZ', TI %x, T %lu, DS %lu\n",
//         KeyHandle, &ValueNameCopy, TitleIndex, Type, DataSize);
//
//     /* Make sure the name is aligned, not too long, and the data under 4GB */
//     if ((ValueNameCopy.Length > 32767) ||
//         (ValueNameCopy.Length & (sizeof(WCHAR) - 1)) ||
//         (DataSize > 0x80000000))
//     {
//         /* Fail */
//         Status = STATUS_INVALID_PARAMETER;
//         goto Quit;
//     }
//
//     /* Ignore any null characters at the end */
//     while (ValueNameCopy.Length &&
//            !(ValueNameCopy.Buffer[ValueNameCopy.Length / sizeof(WCHAR) - 1]))
//     {
//         /* Skip it */
//         ValueNameCopy.Length -= sizeof(WCHAR);
//     }
//
//     /* Don't touch read-only keys */
//     if (KeyObject->KeyControlBlock->ExtFlags & CM_KCB_READ_ONLY_KEY)
//     {
//         /* Fail */
//         Status = STATUS_ACCESS_DENIED;
//         goto Quit;
//     }
//
//     /* Setup callback */
//     PostOperationInfo.Object = (PVOID)KeyObject;
//     SetValueKeyInfo.Object = (PVOID)KeyObject;
//     SetValueKeyInfo.ValueName = &ValueNameCopy;
//     SetValueKeyInfo.TitleIndex = TitleIndex;
//     SetValueKeyInfo.Type = Type;
//     SetValueKeyInfo.Data = Data;
//     SetValueKeyInfo.DataSize = DataSize;
//
//     /* Do the callback */
//     Status = CmiCallRegisteredCallbacks(RegNtPreSetValueKey, &SetValueKeyInfo);
//     if (NT_SUCCESS(Status))
//     {
//         /* Call the internal API */
//         Status = CmSetValueKey(KeyObject->KeyControlBlock,
//                                &ValueNameCopy,
//                                Type,
//                                Data,
//                                DataSize);
//
//         /* Do the post-callback */
//         PostOperationInfo.Status = Status;
//         CmiCallRegisteredCallbacks(RegNtPostSetValueKey, &PostOperationInfo);
//     }
//
// Quit:
//     if (ValueNameCopy.Buffer)
//         ReleaseCapturedUnicodeString(&ValueNameCopy, PreviousMode);
//
//     if ((PreviousMode != KernelMode) && Data)
//         ExFreePoolWithTag(Data, TAG_CM);
//
//     /* Dereference and return status */
//     ObDereferenceObject(KeyObject);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtDeleteValueKey(IN HANDLE KeyHandle,
//                  IN PUNICODE_STRING ValueName)
// {
//     NTSTATUS Status;
//     PCM_KEY_BODY KeyObject;
//     REG_DELETE_VALUE_KEY_INFORMATION DeleteValueKeyInfo;
//     REG_POST_OPERATION_INFORMATION PostOperationInfo;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     UNICODE_STRING ValueNameCopy;
//
//     PAGED_CODE();
//
//     /* Verify that the handle is valid and is a registry key */
//     Status = ObReferenceObjectByHandle(KeyHandle,
//                                        KEY_SET_VALUE,
//                                        CmpKeyObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&KeyObject,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Capture the string */
//     Status = ProbeAndCaptureUnicodeString(&ValueNameCopy, PreviousMode, ValueName);
//     if (!NT_SUCCESS(Status))
//         goto Quit;
//
//     /* Make sure the name is aligned properly */
//     if (ValueNameCopy.Length & (sizeof(WCHAR) - 1))
//     {
//         /* It isn't, so we'll fail */
//         Status = STATUS_INVALID_PARAMETER;
//         goto Quit;
//     }
//
//     /* Don't touch read-only keys */
//     if (KeyObject->KeyControlBlock->ExtFlags & CM_KCB_READ_ONLY_KEY)
//     {
//         /* Fail */
//         Status = STATUS_ACCESS_DENIED;
//         goto Quit;
//     }
//
//     /* Do the callback */
//     DeleteValueKeyInfo.Object = (PVOID)KeyObject;
//     DeleteValueKeyInfo.ValueName = ValueName;
//     Status = CmiCallRegisteredCallbacks(RegNtPreDeleteValueKey,
//                                         &DeleteValueKeyInfo);
//     if (NT_SUCCESS(Status))
//     {
//         /* Call the internal API */
//         Status = CmDeleteValueKey(KeyObject->KeyControlBlock, ValueNameCopy);
//
//         /* Do the post callback */
//         PostOperationInfo.Object = (PVOID)KeyObject;
//         PostOperationInfo.Status = Status;
//         CmiCallRegisteredCallbacks(RegNtPostDeleteValueKey,
//                                    &PostOperationInfo);
//     }
//
// Quit:
//     if (ValueNameCopy.Buffer)
//         ReleaseCapturedUnicodeString(&ValueNameCopy, PreviousMode);
//
//     /* Dereference and return status */
//     ObDereferenceObject(KeyObject);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtFlushKey(IN HANDLE KeyHandle)
// {
//     NTSTATUS Status;
//     PCM_KEY_BODY KeyObject;
//     PAGED_CODE();
//
//     /* Get the key object */
//     Status = ObReferenceObjectByHandle(KeyHandle,
//                                        0,
//                                        CmpKeyObjectType,
//                                        ExGetPreviousMode(),
//                                        (PVOID*)&KeyObject,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Lock the registry */
//     CmpLockRegistry();
//
//     /* Lock the KCB */
//     CmpAcquireKcbLockShared(KeyObject->KeyControlBlock);
//
//     /* Make sure KCB isn't deleted */
//     if (KeyObject->KeyControlBlock->Delete)
//     {
//         /* Fail */
//         Status = STATUS_KEY_DELETED;
//     }
//     else
//     {
//         /* Call the internal API */
//         Status = CmFlushKey(KeyObject->KeyControlBlock, FALSE);
//     }
//
//     /* Release the locks */
//     CmpReleaseKcbLock(KeyObject->KeyControlBlock);
//     CmpUnlockRegistry();
//
//     /* Dereference the object and return status */
//     ObDereferenceObject(KeyObject);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtLoadKey(IN POBJECT_ATTRIBUTES KeyObjectAttributes,
//           IN POBJECT_ATTRIBUTES FileObjectAttributes)
// {
//     /* Call the newer API */
//     return NtLoadKeyEx(KeyObjectAttributes, FileObjectAttributes, 0, NULL);
// }
//
// NTSTATUS
// NTAPI
// NtLoadKey2(IN POBJECT_ATTRIBUTES KeyObjectAttributes,
//            IN POBJECT_ATTRIBUTES FileObjectAttributes,
//            IN ULONG Flags)
// {
//     /* Call the newer API */
//     return NtLoadKeyEx(KeyObjectAttributes, FileObjectAttributes, Flags, NULL);
// }
//
// NTSTATUS
// NTAPI
// NtLoadKeyEx(IN POBJECT_ATTRIBUTES TargetKey,
//             IN POBJECT_ATTRIBUTES SourceFile,
//             IN ULONG Flags,
//             IN HANDLE TrustClassKey)
// {
//     NTSTATUS Status;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     OBJECT_ATTRIBUTES CapturedTargetKey;
//     OBJECT_ATTRIBUTES CapturedSourceFile;
//     UNICODE_STRING TargetKeyName, SourceFileName;
//     HANDLE KmTargetKeyRootDir = NULL, KmSourceFileRootDir = NULL;
//     PCM_KEY_BODY KeyBody = NULL;
//
//     PAGED_CODE();
//
//     /* Validate flags */
//     if (Flags & ~REG_NO_LAZY_FLUSH)
//         return STATUS_INVALID_PARAMETER;
//
//     /* Validate privilege */
//     if (!SeSinglePrivilegeCheck(SeRestorePrivilege, PreviousMode))
//     {
//         DPRINT1("Restore Privilege missing!\n");
//         return STATUS_PRIVILEGE_NOT_HELD;
//     }
//
//     /* Block APCs */
//     KeEnterCriticalRegion();
//
//     /* Check for user-mode caller */
//     if (PreviousMode != KernelMode)
//     {
//         /* Prepare to probe parameters */
//         _SEH2_TRY
//         {
//             /* Probe target key */
//             ProbeForRead(TargetKey,
//                          sizeof(OBJECT_ATTRIBUTES),
//                          sizeof(ULONG));
//
//             /* Probe source file */
//             ProbeForRead(SourceFile,
//                          sizeof(OBJECT_ATTRIBUTES),
//                          sizeof(ULONG));
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             Status = _SEH2_GetExceptionCode();
//             _SEH2_YIELD(goto Quit);
//         }
//         _SEH2_END;
//     }
//
//     /* Probe and capture the target key attributes, including the security */
//     Status = ProbeAndCaptureObjectAttributes(&CapturedTargetKey,
//                                              &TargetKeyName,
//                                              PreviousMode,
//                                              TargetKey,
//                                              TRUE);
//     if (!NT_SUCCESS(Status))
//         goto Quit;
//
//     /*
//      * Probe and capture the source file attributes, but not the security.
//      * A proper security context is built by CmLoadKey().
//      */
//     Status = ProbeAndCaptureObjectAttributes(&CapturedSourceFile,
//                                              &SourceFileName,
//                                              PreviousMode,
//                                              SourceFile,
//                                              FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         ReleaseCapturedObjectAttributes(&CapturedTargetKey, PreviousMode);
//         goto Quit;
//     }
//
//     /* Make sure the target key root directory handle is a kernel handle */
//     Status = CmpConvertHandleToKernelHandle(CapturedTargetKey.RootDirectory,
//                                             CmpKeyObjectType,
//                                             KEY_READ,
//                                             PreviousMode,
//                                             &KmTargetKeyRootDir);
//     if (!NT_SUCCESS(Status))
//         goto Cleanup;
//     CapturedTargetKey.RootDirectory = KmTargetKeyRootDir;
//     CapturedTargetKey.Attributes |= OBJ_KERNEL_HANDLE;
//
//     /* Make sure the source file root directory handle is a kernel handle */
//     Status = CmpConvertHandleToKernelHandle(CapturedSourceFile.RootDirectory,
//                                             IoFileObjectType,
//                                             FILE_TRAVERSE,
//                                             PreviousMode,
//                                             &KmSourceFileRootDir);
//     if (!NT_SUCCESS(Status))
//         goto Cleanup;
//     CapturedSourceFile.RootDirectory = KmSourceFileRootDir;
//     CapturedSourceFile.Attributes |= OBJ_KERNEL_HANDLE;
//
//     /* Check if we have a trust class */
//     if (TrustClassKey)
//     {
//         /* Reference it */
//         Status = ObReferenceObjectByHandle(TrustClassKey,
//                                            0,
//                                            CmpKeyObjectType,
//                                            PreviousMode,
//                                            (PVOID*)&KeyBody,
//                                            NULL);
//     }
//
//     /* Call the internal API */
//     Status = CmLoadKey(&CapturedTargetKey,
//                        &CapturedSourceFile,
//                        Flags,
//                        KeyBody);
//
//     /* Dereference the trust key, if any */
//     if (KeyBody) ObDereferenceObject(KeyBody);
//
// Cleanup:
//     /* Close the local kernel handles */
//     if (KmSourceFileRootDir)
//         ObCloseHandle(KmSourceFileRootDir, KernelMode);
//     if (KmTargetKeyRootDir)
//         ObCloseHandle(KmTargetKeyRootDir, KernelMode);
//
//     /* Release the captured object attributes */
//     ReleaseCapturedObjectAttributes(&CapturedSourceFile, PreviousMode);
//     ReleaseCapturedObjectAttributes(&CapturedTargetKey, PreviousMode);
//
// Quit:
//     /* Bring back APCs */
//     KeLeaveCriticalRegion();
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtNotifyChangeKey(IN HANDLE KeyHandle,
//                   IN HANDLE Event,
//                   IN PIO_APC_ROUTINE ApcRoutine OPTIONAL,
//                   IN PVOID ApcContext OPTIONAL,
//                   OUT PIO_STATUS_BLOCK IoStatusBlock,
//                   IN ULONG CompletionFilter,
//                   IN BOOLEAN WatchTree,
//                   OUT PVOID Buffer,
//                   IN ULONG Length,
//                   IN BOOLEAN Asynchronous)
// {
//     /* Call the newer API */
//     return NtNotifyChangeMultipleKeys(KeyHandle,
//                                       0,
//                                       NULL,
//                                       Event,
//                                       ApcRoutine,
//                                       ApcContext,
//                                       IoStatusBlock,
//                                       CompletionFilter,
//                                       WatchTree,
//                                       Buffer,
//                                       Length,
//                                       Asynchronous);
// }
//
// NTSTATUS
// NTAPI
// NtInitializeRegistry(IN USHORT Flag)
// {
//     BOOLEAN SetupBoot;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PAGED_CODE();
//
//     /* Always do this as kernel mode */
//     if (KeGetPreviousMode() == UserMode)
//         return ZwInitializeRegistry(Flag);
//
//     /* Enough of the system has booted by now */
//     Ki386PerfEnd();
//
//     /* Validate flag */
//     if (Flag > CM_BOOT_FLAG_MAX) return STATUS_INVALID_PARAMETER;
//
//     /* Check if boot was accepted */
//     if ((Flag >= CM_BOOT_FLAG_ACCEPTED) && (Flag <= CM_BOOT_FLAG_MAX))
//     {
//         /* Only allow once */
//         if (!CmBootAcceptFirstTime) return STATUS_ACCESS_DENIED;
//         CmBootAcceptFirstTime = FALSE;
//
//         /* Get the control set accepted */
//         Flag -= CM_BOOT_FLAG_ACCEPTED;
//         if (Flag)
//         {
//             /* Save the last known good boot */
//             Status = CmpSaveBootControlSet(Flag);
//
//             /* Notify HAL */
//             HalEndOfBoot();
//
//             /* Enable lazy flush */
//             CmpHoldLazyFlush = FALSE;
//             CmpLazyFlush();
//             return Status;
//         }
//
//         /* Otherwise, invalid boot */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Check if this was a setup boot */
//     SetupBoot = (Flag == CM_BOOT_FLAG_SETUP ? TRUE : FALSE);
//
//     /* Make sure we're only called once */
//     if (!CmFirstTime) return STATUS_ACCESS_DENIED;
//     CmFirstTime = FALSE;
//
//     /* Lock the registry exclusively */
//     CmpLockRegistryExclusive();
//
//     /* Initialize the hives and lazy flusher */
//     CmpCmdInit(SetupBoot);
//
//     /* Save version data */
//     CmpSetVersionData();
//
//     /* Release the registry lock */
//     CmpUnlockRegistry();
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// NtCompactKeys(IN ULONG Count,
//               IN PHANDLE KeyArray)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtCompressKey(IN HANDLE Key)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// // FIXME: different for different windows versions!
// #define PRODUCT_ACTIVATION_VERSION 7749
//
// NTSTATUS
// NTAPI
// NtLockProductActivationKeys(IN PULONG pPrivateVer,
//                             IN PULONG pSafeMode)
// {
//     KPROCESSOR_MODE PreviousMode;
//
//     PreviousMode = ExGetPreviousMode();
//     _SEH2_TRY
//     {
//         /* Check if the caller asked for the version */
//         if (pPrivateVer != NULL)
//         {
//             /* For user mode, probe it */
//             if (PreviousMode != KernelMode)
//             {
//                 ProbeForWriteUlong(pPrivateVer);
//             }
//
//             /* Return the expected version */
//             *pPrivateVer = PRODUCT_ACTIVATION_VERSION;
//         }
//
//         /* Check if the caller asked for safe mode mode state */
//         if (pSafeMode != NULL)
//         {
//             /* For user mode, probe it */
//             if (PreviousMode != KernelMode)
//             {
//                 ProbeForWriteUlong(pSafeMode);
//             }
//
//             /* Return the safe boot mode state */
//             *pSafeMode = InitSafeBootMode;
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// NtLockRegistryKey(IN HANDLE KeyHandle)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtNotifyChangeMultipleKeys(IN HANDLE MasterKeyHandle,
//                            IN ULONG Count,
//                            IN POBJECT_ATTRIBUTES SlaveObjects,
//                            IN HANDLE Event,
//                            IN PIO_APC_ROUTINE ApcRoutine OPTIONAL,
//                            IN PVOID ApcContext OPTIONAL,
//                            OUT PIO_STATUS_BLOCK IoStatusBlock,
//                            IN ULONG CompletionFilter,
//                            IN BOOLEAN WatchTree,
//                            OUT PVOID Buffer,
//                            IN ULONG Length,
//                            IN BOOLEAN Asynchronous)
// {
//     UNIMPLEMENTED_ONCE;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtQueryMultipleValueKey(IN HANDLE KeyHandle,
//                         IN OUT PKEY_VALUE_ENTRY ValueList,
//                         IN ULONG NumberOfValues,
//                         OUT PVOID Buffer,
//                         IN OUT PULONG Length,
//                         OUT PULONG ReturnLength)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtQueryOpenSubKeys(IN POBJECT_ATTRIBUTES TargetKey,
//                    OUT PULONG HandleCount)
// {
//     KPROCESSOR_MODE PreviousMode;
//     PCM_KEY_BODY KeyBody = NULL;
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//     ULONG SubKeys;
//
//     DPRINT("NtQueryOpenSubKeys()\n");
//
//     PAGED_CODE();
//
//     /* Get the processor mode */
//     PreviousMode = KeGetPreviousMode();
//
//     /* Check for user-mode caller */
//     if (PreviousMode != KernelMode)
//     {
//         /* Prepare to probe parameters */
//         _SEH2_TRY
//         {
//             /* Probe target key */
//             ProbeForRead(TargetKey,
//                          sizeof(OBJECT_ATTRIBUTES),
//                          sizeof(ULONG));
//
//             /* Probe handle count */
//             ProbeForWriteUlong(HandleCount);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Open a handle to the key */
//     Status = ObOpenObjectByName(TargetKey,
//                                 CmpKeyObjectType,
//                                 PreviousMode,
//                                 NULL,
//                                 KEY_READ,
//                                 NULL,
//                                 &KeyHandle);
//     if (NT_SUCCESS(Status))
//     {
//         /* Reference the key object */
//         Status = ObReferenceObjectByHandle(KeyHandle,
//                                            KEY_READ,
//                                            CmpKeyObjectType,
//                                            PreviousMode,
//                                            (PVOID*)&KeyBody,
//                                            NULL);
//
//         /* Close the handle */
//         NtClose(KeyHandle);
//     }
//
//     /* Fail, if the key object could not be referenced */
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Lock the registry exclusively */
//     CmpLockRegistryExclusive();
//
//     /* Fail, if we did not open a hive root key */
//     if (KeyBody->KeyControlBlock->KeyCell !=
//         KeyBody->KeyControlBlock->KeyHive->BaseBlock->RootCell)
//     {
//         DPRINT("Error: Key is not a hive root key!\n");
//         CmpUnlockRegistry();
//         ObDereferenceObject(KeyBody);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Call the internal API */
//     SubKeys = CmpEnumerateOpenSubKeys(KeyBody->KeyControlBlock,
//                                       FALSE, FALSE);
//
//     /* Unlock the registry */
//     CmpUnlockRegistry();
//
//     /* Dereference the key object */
//     ObDereferenceObject(KeyBody);
//
//     /* Write back the result */
//     _SEH2_TRY
//     {
//         *HandleCount = SubKeys;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     DPRINT("Done.\n");
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtQueryOpenSubKeysEx(IN POBJECT_ATTRIBUTES TargetKey,
//                      IN ULONG BufferLength,
//                      IN PVOID Buffer,
//                      IN PULONG RequiredSize)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtRenameKey(IN HANDLE KeyHandle,
//             IN PUNICODE_STRING ReplacementName)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtReplaceKey(IN POBJECT_ATTRIBUTES ObjectAttributes,
//              IN HANDLE Key,
//              IN POBJECT_ATTRIBUTES ReplacedObjectAttributes)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtRestoreKey(IN HANDLE KeyHandle,
//              IN HANDLE FileHandle,
//              IN ULONG RestoreFlags)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtSaveKey(IN HANDLE KeyHandle,
//           IN HANDLE FileHandle)
// {
//     /* Call the extended API */
//     return NtSaveKeyEx(KeyHandle, FileHandle, REG_STANDARD_FORMAT);
// }
//
// NTSTATUS
// NTAPI
// NtSaveKeyEx(IN HANDLE KeyHandle,
//             IN HANDLE FileHandle,
//             IN ULONG Flags)
// {
//     NTSTATUS Status;
//     HANDLE KmFileHandle = NULL;
//     PCM_KEY_BODY KeyObject;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//
//     PAGED_CODE();
//
//     DPRINT("NtSaveKeyEx(0x%p, 0x%p, %lu)\n", KeyHandle, FileHandle, Flags);
//
//     /* Verify the flags */
//     if ((Flags != REG_STANDARD_FORMAT)
//         && (Flags != REG_LATEST_FORMAT)
//         && (Flags != REG_NO_COMPRESSION))
//     {
//         /* Only one of these values can be specified */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Validate privilege */
//     if (!SeSinglePrivilegeCheck(SeBackupPrivilege, PreviousMode))
//     {
//         return STATUS_PRIVILEGE_NOT_HELD;
//     }
//
//     /* Make sure the target file handle is a kernel handle */
//     Status = CmpConvertHandleToKernelHandle(FileHandle,
//                                             IoFileObjectType,
//                                             FILE_WRITE_DATA,
//                                             PreviousMode,
//                                             &KmFileHandle);
//     if (!NT_SUCCESS(Status))
//         goto Quit;
//
//     /* Verify that the handle is valid and is a registry key */
//     Status = ObReferenceObjectByHandle(KeyHandle,
//                                        KEY_READ,
//                                        CmpKeyObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&KeyObject,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//         goto Quit;
//
//     /* Call the internal API */
//     Status = CmSaveKey(KeyObject->KeyControlBlock, KmFileHandle, Flags);
//
//     /* Dereference the registry key */
//     ObDereferenceObject(KeyObject);
//
// Quit:
//     /* Close the local kernel handle */
//     if (KmFileHandle)
//         ObCloseHandle(KmFileHandle, KernelMode);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtSaveMergedKeys(IN HANDLE HighPrecedenceKeyHandle,
//                  IN HANDLE LowPrecedenceKeyHandle,
//                  IN HANDLE FileHandle)
// {
//     NTSTATUS Status;
//     KPROCESSOR_MODE PreviousMode;
//     HANDLE KmFileHandle = NULL;
//     PCM_KEY_BODY HighPrecedenceKeyObject = NULL;
//     PCM_KEY_BODY LowPrecedenceKeyObject = NULL;
//
//     PAGED_CODE();
//
//     DPRINT("NtSaveMergedKeys(0x%p, 0x%p, 0x%p)\n",
//            HighPrecedenceKeyHandle, LowPrecedenceKeyHandle, FileHandle);
//
//     PreviousMode = ExGetPreviousMode();
//
//     /* Validate privilege */
//     if (!SeSinglePrivilegeCheck(SeBackupPrivilege, PreviousMode))
//     {
//         return STATUS_PRIVILEGE_NOT_HELD;
//     }
//
//     /* Make sure the target file handle is a kernel handle */
//     Status = CmpConvertHandleToKernelHandle(FileHandle,
//                                             IoFileObjectType,
//                                             FILE_WRITE_DATA,
//                                             PreviousMode,
//                                             &KmFileHandle);
//     if (!NT_SUCCESS(Status))
//         goto Quit;
//
//     /* Verify that the handles are valid and are registry keys */
//     Status = ObReferenceObjectByHandle(HighPrecedenceKeyHandle,
//                                        KEY_READ,
//                                        CmpKeyObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&HighPrecedenceKeyObject,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//         goto Quit;
//
//     Status = ObReferenceObjectByHandle(LowPrecedenceKeyHandle,
//                                        KEY_READ,
//                                        CmpKeyObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&LowPrecedenceKeyObject,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//         goto Quit;
//
//     /* Call the internal API */
//     Status = CmSaveMergedKeys(HighPrecedenceKeyObject->KeyControlBlock,
//                               LowPrecedenceKeyObject->KeyControlBlock,
//                               KmFileHandle);
//
// Quit:
//     /* Dereference the opened key objects */
//     if (LowPrecedenceKeyObject)
//         ObDereferenceObject(LowPrecedenceKeyObject);
//     if (HighPrecedenceKeyObject)
//         ObDereferenceObject(HighPrecedenceKeyObject);
//
//     /* Close the local kernel handle */
//     if (KmFileHandle)
//         ObCloseHandle(KmFileHandle, KernelMode);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtSetInformationKey(IN HANDLE KeyHandle,
//                     IN KEY_SET_INFORMATION_CLASS KeyInformationClass,
//                     IN PVOID KeyInformation,
//                     IN ULONG KeyInformationLength)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtUnloadKey(IN POBJECT_ATTRIBUTES KeyObjectAttributes)
// {
//     return NtUnloadKey2(KeyObjectAttributes, 0);
// }
//
// NTSTATUS
// NTAPI
// NtUnloadKey2(IN POBJECT_ATTRIBUTES TargetKey,
//              IN ULONG Flags)
// {
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES CapturedTargetKey;
//     UNICODE_STRING ObjectName;
//     HANDLE KmTargetKeyRootDir = NULL;
//     CM_PARSE_CONTEXT ParseContext = {0};
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     PCM_KEY_BODY KeyBody = NULL;
//     HANDLE Handle;
//
//     PAGED_CODE();
//
//     /* Validate privilege */
//     if (!SeSinglePrivilegeCheck(SeRestorePrivilege, PreviousMode))
//     {
//         DPRINT1("Restore Privilege missing!\n");
//         return STATUS_PRIVILEGE_NOT_HELD;
//     }
//
//     /* Check for user-mode caller */
//     if (PreviousMode != KernelMode)
//     {
//         /* Prepare to probe parameters */
//         _SEH2_TRY
//         {
//             /* Probe object attributes */
//             ProbeForRead(TargetKey,
//                          sizeof(OBJECT_ATTRIBUTES),
//                          sizeof(ULONG));
//
//             CapturedTargetKey = *TargetKey;
//
//             /* Probe the string */
//             ObjectName = ProbeForReadUnicodeString(CapturedTargetKey.ObjectName);
//             ProbeForRead(ObjectName.Buffer,
//                          ObjectName.Length,
//                          sizeof(WCHAR));
//
//             CapturedTargetKey.ObjectName = &ObjectName;
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
//         /* Save the target attributes directly */
//         CapturedTargetKey = *TargetKey;
//     }
//
//     /* Make sure the target key root directory handle is a kernel handle */
//     Status = CmpConvertHandleToKernelHandle(CapturedTargetKey.RootDirectory,
//                                             CmpKeyObjectType,
//                                             KEY_WRITE,
//                                             PreviousMode,
//                                             &KmTargetKeyRootDir);
//     if (!NT_SUCCESS(Status))
//         return Status;
//     CapturedTargetKey.RootDirectory = KmTargetKeyRootDir;
//     CapturedTargetKey.Attributes |= OBJ_KERNEL_HANDLE;
//
//     /* Setup the parse context */
//     ParseContext.CreateOperation = TRUE;
//     ParseContext.CreateOptions = REG_OPTION_BACKUP_RESTORE;
//
//     /* Do the create */
//     /* Open a local handle to the key */
//     Status = ObOpenObjectByName(&CapturedTargetKey,
//                                 CmpKeyObjectType,
//                                 KernelMode,
//                                 NULL,
//                                 KEY_WRITE,
//                                 &ParseContext,
//                                 &Handle);
//     if (NT_SUCCESS(Status))
//     {
//         /* Reference the key object */
//         Status = ObReferenceObjectByHandle(Handle,
//                                            KEY_WRITE,
//                                            CmpKeyObjectType,
//                                            KernelMode,
//                                            (PVOID*)&KeyBody,
//                                            NULL);
//
//         /* Close the handle */
//         ObCloseHandle(Handle, KernelMode);
//     }
//
//     /* Close the local kernel handle */
//     if (KmTargetKeyRootDir)
//         ObCloseHandle(KmTargetKeyRootDir, KernelMode);
//
//     /* Return if a failure was encountered */
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /*
//      * Lock down the entire registry when we unload a hive.
//      *
//      * NOTE: We might block other threads of other processes that do
//      * operations with unrelated keys of other hives when we lock
//      * the registry for exclusive use by the calling thread that does
//      * the unloading. If this turns out to cause a major overhead we
//      * have to rethink the locking mechanism here (prior commit - f1d2a44).
//      */
//     CmpLockRegistryExclusive();
//     ExAcquirePushLockExclusive(&CmpLoadHiveLock);
//
//     /* Check if it's being deleted already */
//     if (KeyBody->KeyControlBlock->Delete)
//     {
//         /* Return appropriate status */
//         Status = STATUS_KEY_DELETED;
//         goto Quit;
//     }
//
//     /* Check if it's a read-only key */
//     if (KeyBody->KeyControlBlock->ExtFlags & CM_KCB_READ_ONLY_KEY)
//     {
//         /* Return appropriate status */
//         Status = STATUS_ACCESS_DENIED;
//         goto Quit;
//     }
//
//     /* Call the internal API. Note that CmUnloadKey() unlocks the registry only on success. */
//     Status = CmUnloadKey(KeyBody->KeyControlBlock, Flags);
//
// Quit:
//     /* If CmUnloadKey() failed we need to unlock registry ourselves */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Unlock the hive loading and registry locks */
//         ExReleasePushLockExclusive(&CmpLoadHiveLock);
//         CmpUnlockRegistry();
//     }
//
//     /* Dereference the key */
//     ObDereferenceObject(KeyBody);
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtUnloadKeyEx(IN POBJECT_ATTRIBUTES TargetKey,
//               IN HANDLE Event)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
