// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_47C5245EF209: [u16; 64] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0079,
    0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
    0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
    0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065,
    0x0073, 0x005C, 0x0025, 0x0077, 0x0073, 0x0025, 0x0030, 0x0032, 0x0064, 0x0025,
    0x0030, 0x0032, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_5B2AEEDF221A: [u16; 60] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0079,
    0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
    0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
    0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C,
    0x005C, 0x0057, 0x0064, 0x0066, 0x005C, 0x004B, 0x006D, 0x0064, 0x0066, 0x0000,
];
pub static RUSTCONV_WSTR_CF50DD39C387: [u16; 8] = [0x0053u16, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_DA0DD5ADE145: [u16; 12] = [
    0x0025u16, 0x0073, 0x005C, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E,
    0x0073, 0x0000,
];

use crate::prelude::*;
use ntkrnl_sys::ddk::{ExAcquireResourceExclusiveLite, ExDeleteResourceLite, ExInitializeResourceLite, ExReleaseResourceLite, IoGetDeviceObjectPointer, KeEnterCriticalRegion, KeLeaveCriticalRegion, LIST_ENTRY, _OBJECT_ATTRIBUTES, PDEVICE_OBJECT, PERESOURCE, PHANDLE, PKEY_VALUE_PARTIAL_INFORMATION, PUNICODE_STRING, RtlCopyUnicodeString, RtlEqualUnicodeString, RtlFreeUnicodeString, RtlIntegerToUnicodeString, ZwClose, ZwLoadDriver, ZwOpenKey, ZwUnloadDriver};
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn ClassAcquireClientLock(Resource: PERESOURCE) {
    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(Resource, TRUE as BOOLEAN);
}
#[no_mangle]
pub unsafe extern "C" fn ClassReleaseClientLock(mut Resource: PERESOURCE) {
    ExReleaseResourceLite(Resource);
    KeLeaveCriticalRegion();
}
#[no_mangle]
pub unsafe extern "C" fn ClassOpen(
    mut ClassModule: PCLASS_MODULE,
    mut ObjectName: PUNICODE_STRING,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut deviceObject: PDEVICE_OBJECT = core::ptr::null_mut();

    if ObjectName.is_null() {
        return STATUS_SUCCESS;
    }

    status = IoGetDeviceObjectPointer(
        ObjectName,
        FILE_READ_DATA as ACCESS_MASK,
        core::ptr::addr_of_mut!((*ClassModule).ClassFileObject),
        core::ptr::addr_of_mut!(deviceObject),
    );

    if NT_SUCCESS!(status) {
        (*ClassModule).ClassDriverObject = (*deviceObject).DriverObject;
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn ClassClose(ClassModule: PCLASS_MODULE) {
    if !(*ClassModule).ClassFileObject.is_null() {
        ObDereferenceObject((*ClassModule).ClassFileObject as PVOID);
        (*ClassModule).ClassFileObject = core::ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClassCleanupAndFree(ClassModule: PCLASS_MODULE) {
    ClassClose(ClassModule);

    if !(*ClassModule).Service.Buffer.is_null() {
        ExFreePoolWithTag((*ClassModule).Service.Buffer as PVOID, WDFLDR_TAG as ULONG);
    }

    if !(*ClassModule).ImageName.Buffer.is_null() {
        ExFreePoolWithTag((*ClassModule).ImageName.Buffer as PVOID, WDFLDR_TAG as ULONG);
    }

    ExDeleteResourceLite(core::ptr::addr_of_mut!((*ClassModule).ClientsListLock));
    ExFreePoolWithTag(ClassModule as PVOID, WDFLDR_TAG as ULONG);
}
#[no_mangle]
pub unsafe extern "C" fn ClassCreate(
    ClassLibInfo: PWDF_CLASS_LIBRARY_INFO,
    LibModule: PLIBRARY_MODULE,
    ServiceName: PUNICODE_STRING,
) -> PCLASS_MODULE {
    let mut pNewClassModule: PCLASS_MODULE;
    let mut status: NTSTATUS;

    pNewClassModule = ExAllocatePoolZero(
        NonPagedPool,
        (core::mem::size_of::<_CLASS_MODULE>() as SIZE_T) as _,
        WDFLDR_TAG,
    ) as PCLASS_MODULE;
    if pNewClassModule.is_null() {
        return core::ptr::null_mut();
    }

    if !ClassLibInfo.is_null() {
        (*pNewClassModule).ImplicitlyLoaded = TRUE;
        (*pNewClassModule).ClassLibraryInfo = ClassLibInfo;
        (*pNewClassModule).Version.Major = (*ClassLibInfo).Version.Major;
        (*pNewClassModule).Version.Minor = (*ClassLibInfo).Version.Minor;
        (*pNewClassModule).Version.Build = (*ClassLibInfo).Version.Build;
    }

    (*pNewClassModule).ClassRefCount = 1;
    (*pNewClassModule).Library = LibModule;
    ExInitializeResourceLite(core::ptr::addr_of_mut!((*pNewClassModule).ClientsListLock));
    InitializeListHead(core::ptr::addr_of_mut!((*pNewClassModule).ClientsListHead));
    InitializeListHead(core::ptr::addr_of_mut!((*pNewClassModule).LibraryLinkage));

    (*pNewClassModule).Service.Buffer = ExAllocatePoolZero(
        PagedPool,
        ((*ServiceName).MaximumLength as SIZE_T) as _,
        WDFLDR_TAG,
    ) as PWCH;

    'clean: {
        if (*pNewClassModule).Service.Buffer.is_null() {
            break 'clean;
        }
        (*pNewClassModule).Service.MaximumLength = (*ServiceName).MaximumLength;
        RtlCopyUnicodeString(
            core::ptr::addr_of_mut!((*pNewClassModule).Service),
            ServiceName,
        );
        status = GetImageName(
            ServiceName,
            core::ptr::addr_of_mut!((*pNewClassModule).ImageName),
        );

        if !NT_SUCCESS!(status) {
            break 'clean;
        }

        (*pNewClassModule).IsBootDriver =
            ServiceCheckBootStart(core::ptr::addr_of_mut!((*pNewClassModule).Service));
        status = GetImageInfo(
            core::ptr::addr_of_mut!((*pNewClassModule).ImageName),
            core::ptr::addr_of_mut!((*pNewClassModule).ImageAddress),
            core::ptr::addr_of_mut!((*pNewClassModule).ImageSize),
        );

        if NT_SUCCESS!(status) {
            return pNewClassModule;
        } else {
            __DBGPRINT!(b"GetImageBase failed\n\0" as *const u8 as PCSTR);
        }
    }

    // clean:
    ClassCleanupAndFree(pNewClassModule);

    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn ClassClientCreate() -> PCLASS_CLIENT_MODULE {
    let mut pNewClassClientModule: PCLASS_CLIENT_MODULE = core::ptr::null_mut();

    pNewClassClientModule = ExAllocatePoolZero(
        NonPagedPool,
        (core::mem::size_of::<CLASS_CLIENT_MODULE>() as SIZE_T) as _,
        WDFLDR_TAG as ULONG,
    ) as PCLASS_CLIENT_MODULE;
    if !pNewClassClientModule.is_null() {
        InitializeListHead(core::ptr::addr_of_mut!((*pNewClassClientModule).ClassLinkage));
        InitializeListHead(core::ptr::addr_of_mut!((*pNewClassClientModule).ClientLinkage));
    }

    return pNewClassClientModule;
}
#[no_mangle]
pub unsafe extern "C" fn LibraryFindClientLocked(
    mut LibModule: PLIBRARY_MODULE,
    mut BindInfo: PWDF_BIND_INFO,
) -> PCLIENT_MODULE {
    let mut pFoundClient: PCLIENT_MODULE = core::ptr::null_mut();
    let mut entry: PLIST_ENTRY = core::ptr::null_mut();
    entry = (*LibModule).ClientsListHead.Flink as PLIST_ENTRY;
    while entry != core::ptr::addr_of_mut!((*LibModule).ClientsListHead) {
        pFoundClient = (entry as usize)
            .wrapping_sub(core::mem::offset_of!(CLIENT_MODULE, LibListEntry))
            as PCLIENT_MODULE;
        if (*pFoundClient).Info == BindInfo {
            return pFoundClient;
        }
        entry = (*entry).Flink as PLIST_ENTRY;
    }
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn ClassLinkInClient(ClassModule: PCLASS_MODULE, ClassBindInfo: PWDF_CLASS_BIND_INFO, BindInfo: PWDF_BIND_INFO, ClassClientModule: PCLASS_CLIENT_MODULE) -> NTSTATUS {
    let mut pClienInfo: PCLIENT_MODULE = core::ptr::null_mut();
    let mut status: NTSTATUS;

    (*ClassClientModule).Class = ClassModule;
    (*ClassClientModule).ClientClassBindInfo = ClassBindInfo;
    LibraryAcquireClientLock((*ClassModule).Library);
    pClienInfo = LibraryFindClientLocked((*ClassModule).Library, BindInfo);

    if !pClienInfo.is_null() {
        (*ClassClientModule).Client = pClienInfo;
        InsertTailList(
            &raw mut (*pClienInfo).ClassListHead,
            &raw mut (*ClassClientModule).ClassLinkage,
        );
    }

    LibraryReleaseClientLock((*ClassModule).Library);

    if !pClienInfo.is_null() {
        ClassAcquireClientLock(&raw mut (*ClassModule).ClientsListLock);
        InsertTailList(
            &raw mut (*ClassModule).ClientsListHead,
            &raw mut (*ClassClientModule).ClientLinkage,
        );
        ClassReleaseClientLock(&raw mut (*ClassModule).ClientsListLock);
        status = STATUS_SUCCESS;
    } else {
        status = STATUS_INVALID_DEVICE_STATE;
        __DBGPRINT!(
            b"ERROR: Could not locate client from Info %p, status 0x%x\n\0".as_ptr(),
            BindInfo,
            status,
        );
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn GetClassRegistryHandle(ClassBindInfo: PWDF_CLASS_BIND_INFO, BindInfo: PWDF_BIND_INFO, KeyHandle: PHANDLE) -> NTSTATUS {
    let mut classVersions: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut objectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut rootHandle: HANDLE = core::ptr::null_mut();
    let mut classVersionsHandle: HANDLE = core::ptr::null_mut();
    let mut classVersionHandle: HANDLE = core::ptr::null_mut();
    let mut status: NTSTATUS = 0;
    let mut classNameLength: SIZE_T = 0;
    let mut bufferSize: SIZE_T = 0;
    let mut objectName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_5B2AEEDF221A.as_ptr());
    // DECLARE_UNICODE_STRING_SIZE(versionString, COMMENT_TOKEN_ECHO____) expanded inline
    let mut versionString_buffer: [WCHAR; 11] = [0; 11];
    let mut versionString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: (11 as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()) as USHORT,
        Buffer: core::ptr::addr_of_mut!(versionString_buffer) as PWCH,
    };

    classVersions.Length = 0;
    classVersions.Buffer = core::ptr::null_mut();
    *KeyHandle = core::ptr::null_mut();

    /* Open root registry key */
    // InitializeObjectAttributes(&objectAttributes, &objectName, FACT_0002 | FACT_0003, NULL, NULL) expanded inline
    objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    objectAttributes.RootDirectory = core::ptr::null_mut();
    objectAttributes.ObjectName = core::ptr::addr_of_mut!(objectName);
    objectAttributes.Attributes = (OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE) as ULONG;
    objectAttributes.SecurityDescriptor = core::ptr::null_mut();
    objectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    status = ZwOpenKey(
        core::ptr::addr_of_mut!(rootHandle),
        KEY_QUERY_VALUE,
        core::ptr::addr_of_mut!(objectAttributes),
    );
    'cleanup: {
        if !NT_SUCCESS!(status) {
            __DBGPRINT!(("ZwOpenKey failed for root key, status 0x%08X\n", status));
            break 'cleanup;
        }

        classNameLength = wcslen((*ClassBindInfo).ClassName) as SIZE_T;
        bufferSize = (classNameLength as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>())
            .wrapping_add(core::mem::size_of_val(&[0x005Cu16, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x0073, 0x0000])) as SIZE_T;

        classVersions.Buffer = ExAllocatePoolWithTag(
            PagedPool,
            bufferSize,
            WDFLDR_TAG,
        ) as PWCH;
        if classVersions.Buffer.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
            break 'cleanup;
        }

        classVersions.MaximumLength = bufferSize as USHORT;
        status = RtlUnicodeStringPrintf!(
            core::ptr::addr_of_mut!(classVersions),
            RUSTCONV_WSTR_DA0DD5ADE145.as_ptr(),
            (*ClassBindInfo).ClassName,
        );
        if !NT_SUCCESS!(status) {
            __DBGPRINT!(("RtlUnicodeStringPrintf failed, status 0x%08X\n", status));
            break 'cleanup;
        }

        // InitializeObjectAttributes(&objectAttributes, &classVersions, FACT_0008 | FACT_0009, rootHandle, NULL) expanded inline
        objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        objectAttributes.RootDirectory = rootHandle;
        objectAttributes.ObjectName = core::ptr::addr_of_mut!(classVersions);
        objectAttributes.Attributes = (OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE) as ULONG;
        objectAttributes.SecurityDescriptor = core::ptr::null_mut();
        objectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        status = ZwOpenKey(
            core::ptr::addr_of_mut!(classVersionsHandle),
            KEY_QUERY_VALUE,
            core::ptr::addr_of_mut!(objectAttributes),
        );
        if !NT_SUCCESS!(status) {
            __DBGPRINT!(("ZwOpenKey failed for class versions, status 0x%08X\n", status));
            break 'cleanup;
        }

        status = RtlIntegerToUnicodeString(
            (*ClassBindInfo).Version.Major,
            10,
            core::ptr::addr_of_mut!(versionString),
        );
        if !NT_SUCCESS!(status) {
            __DBGPRINT!(("RtlIntegerToUnicodeString failed for class version, status 0x%08X\n", status));
            break 'cleanup;
        }

        // InitializeObjectAttributes(&objectAttributes, &versionString, FACT_0012 | FACT_0013, classVersionsHandle, NULL) expanded inline
        objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        objectAttributes.RootDirectory = classVersionsHandle;
        objectAttributes.ObjectName = core::ptr::addr_of_mut!(versionString);
        objectAttributes.Attributes = (OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE) as ULONG;
        objectAttributes.SecurityDescriptor = core::ptr::null_mut();
        objectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        status = ZwOpenKey(
            core::ptr::addr_of_mut!(classVersionHandle),
            KEY_QUERY_VALUE,
            core::ptr::addr_of_mut!(objectAttributes),
        );
        if !NT_SUCCESS!(status) {
            __DBGPRINT!(("ZwOpenKey failed for class version, status 0x%08X\n", status));
            break 'cleanup;
        }

        status = RtlIntegerToUnicodeString(
            (*BindInfo).Version.Major,
            10,
            core::ptr::addr_of_mut!(versionString),
        );
        if !NT_SUCCESS!(status) {
            __DBGPRINT!(("RtlIntegerToUnicodeString failed for bind version, status 0x%08X\n", status));
            break 'cleanup;
        }

        // InitializeObjectAttributes(&objectAttributes, &versionString, FACT_0016 | FACT_0017, classVersionHandle, NULL) expanded inline
        objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        objectAttributes.RootDirectory = classVersionHandle;
        objectAttributes.ObjectName = core::ptr::addr_of_mut!(versionString);
        objectAttributes.Attributes = (OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE) as ULONG;
        objectAttributes.SecurityDescriptor = core::ptr::null_mut();
        objectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        status = ZwOpenKey(
            KeyHandle,
            KEY_QUERY_VALUE,
            core::ptr::addr_of_mut!(objectAttributes),
        );
        if !NT_SUCCESS!(status) {
            __DBGPRINT!(("ZwOpenKey failed for bind version, status 0x%08X\n", status));
            break 'cleanup;
        }
    }

    // cleanup:
    if !classVersionHandle.is_null() {
        ZwClose(classVersionHandle);
    }
    if !classVersionsHandle.is_null() {
        ZwClose(classVersionsHandle);
    }
    if !rootHandle.is_null() {
        ZwClose(rootHandle);
    }

    if !classVersions.Buffer.is_null() {
        ExFreePoolWithTag(classVersions.Buffer as PVOID, WDFLDR_TAG);
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn GetDefaultClassServiceName(mut ClassBindInfo: PWDF_CLASS_BIND_INFO, mut BindInfo: PWDF_BIND_INFO, mut ServiceName: PUNICODE_STRING) -> NTSTATUS {
    let mut classNameLength: SIZE_T = 0;
    let mut bufferSize: SIZE_T = 0;
    let mut status: NTSTATUS = 0;
    classNameLength = wcslen((*ClassBindInfo).ClassName as *const WCHAR) as SIZE_T;
    bufferSize = (classNameLength as usize)
        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
        .wrapping_add(
            (::core::mem::size_of_val(&[
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0079,
    0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
    0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
    0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065,
    0x0073, 0x005C, 0x0000,
]) as usize)
                .wrapping_add(
                    (4 as usize)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize),
                )
                .wrapping_add(::core::mem::size_of_val(&UNICODE_NULL) as usize),
        ) as SIZE_T;
    (*ServiceName).Buffer = ExAllocatePoolWithTag(
        PagedPool,
        bufferSize,
        WDFLDR_TAG as ULONG,
    ) as PWCH;
    if (*ServiceName).Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    (*ServiceName).MaximumLength = bufferSize as USHORT;
    status = RtlUnicodeStringPrintf!(
        ServiceName,
        RUSTCONV_WSTR_47C5245EF209.as_ptr(),
        (*ClassBindInfo).ClassName,
        (*ClassBindInfo).Version.Major,
        (*BindInfo).Version.Major,
    );

    if !NT_SUCCESS!(status) {
        ExFreePoolWithTag((*ServiceName).Buffer as PVOID, WDFLDR_TAG as ULONG);
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn GetClassServicePath(
    mut ClassBindInfo: PWDF_CLASS_BIND_INFO,
    mut BindInfo: PWDF_BIND_INFO,
    mut ServicePath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut pKeyValPartial: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();
    let mut ValueName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_CF50DD39C387.as_ptr());

    (*ServicePath).Length = 0 as USHORT;
    (*ServicePath).Buffer = core::ptr::null_mut();

    status = GetClassRegistryHandle(ClassBindInfo, BindInfo, core::ptr::addr_of_mut!(Handle));

    if !NT_SUCCESS!(status) {
        __DBGPRINT!(("ERROR: GetClassRegistryHandle failed with status 0x%x\n", status));
    } else {
        status = FxLdrQueryData(
            Handle,
            core::ptr::addr_of_mut!(ValueName),
            WDFLDR_TAG as ULONG,
            core::ptr::addr_of_mut!(pKeyValPartial),
        );

        if !NT_SUCCESS!(status) {
            __DBGPRINT!(("ERROR: QueryData failed with status 0x%x\n", status));
        } else {
            status = BuildServicePath(pKeyValPartial, ServicePath);
        }
    }

    if !NT_SUCCESS!(status) {
        status = GetDefaultClassServiceName(ClassBindInfo, BindInfo, ServicePath);

        if !NT_SUCCESS!(status) {
            __DBGPRINT!(("ERROR: GetDefaultClassServiceName failed, status 0x%x\n", status));
        }
    }

    if !Handle.is_null() {
        ZwClose(Handle);
    }
    if !pKeyValPartial.is_null() {
        ExFreePoolWithTag(pKeyValPartial as PVOID, WDFLDR_TAG as ULONG);
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn ReferenceClassVersion(
    ClassBindInfo: PWDF_CLASS_BIND_INFO,
    BindInfo: PWDF_BIND_INFO,
    ClassModule: *mut PCLASS_MODULE,
) -> NTSTATUS {
    let mut pBindInfo: PWDF_BIND_INFO = core::ptr::null_mut();
    let mut pClassModule: PCLASS_MODULE = core::ptr::null_mut();
    let mut driverServiceName: _UNICODE_STRING = core::mem::zeroed();
    let mut pLibModule: PLIBRARY_MODULE = core::ptr::null_mut();
    let mut created: BOOLEAN;
    let mut status: NTSTATUS;

    *ClassModule = core::ptr::null_mut();
    pBindInfo = BindInfo;
    created = FALSE;
    status = GetClassServicePath(ClassBindInfo, BindInfo, core::ptr::addr_of_mut!(driverServiceName));

    if !NT_SUCCESS!(status) {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(driverServiceName));
        return status;
    }
    FxLdrAcquireLoadedModuleLock();
    pClassModule = FindClassByServiceNameLocked(core::ptr::addr_of_mut!(driverServiceName), core::ptr::addr_of_mut!(pLibModule));

    if pLibModule.is_null() {
        pLibModule = (*pBindInfo).Module as PLIBRARY_MODULE;
    }

    if pLibModule == (*pBindInfo).Module {
        if !pClassModule.is_null() {
            InterlockedExchangeAdd(
                core::ptr::addr_of_mut!((*pClassModule).ClassRefCount) as *mut ::core::ffi::c_long,
                1,
            );
        } else {
            pClassModule = ClassCreate(core::ptr::null_mut(), pLibModule, core::ptr::addr_of_mut!(driverServiceName));

            if !pClassModule.is_null() {
                InterlockedExchangeAdd(
                    core::ptr::addr_of_mut!((*pClassModule).ClassRefCount) as *mut ::core::ffi::c_long,
                    1,
                );
                LibraryAddToClassListLocked(pLibModule, pClassModule);
                created = TRUE;
            } else {
                status = STATUS_INSUFFICIENT_RESOURCES;
            }
        }
    } else {
        status = STATUS_REVISION_MISMATCH;

        __DBGPRINT!(
            b"class %S bound to library %p,client bound to library %p, status 0x%x\n\0".as_ptr() as *const i8,
            (*ClassBindInfo).ClassName,
            pLibModule,
            (*pBindInfo).Module,
            status,
        );
    }
    FxLdrReleaseLoadedModuleLock();

    'exit_success: {
        'clean: {
            if created == 0 {
                if !NT_SUCCESS!(status) {
                    break 'clean;
                }

                (*ClassBindInfo).ClassModule = pClassModule as PVOID;
                *ClassModule = pClassModule;
            }

            /* Class logic: Only call ZwLoadDriver if:
             * 1. Class module was newly created, OR
             * 2. Class module exists but has no ClassLibraryInfo
             */
            if created != 0 || (!pClassModule.is_null() && (*pClassModule).ClassLibraryInfo.is_null()) {
                status = ZwLoadDriver(core::ptr::addr_of_mut!(driverServiceName));
                if !NT_SUCCESS!(status) {
                    if status == STATUS_OBJECT_PATH_NOT_FOUND
                        || status == STATUS_OBJECT_NAME_NOT_FOUND
                        || status == STATUS_IMAGE_ALREADY_LOADED
                    {
                        /* Driver might already be loaded as boot driver - UCX*/
                        status = STATUS_SUCCESS;
                    }
                } else if !pClassModule.is_null() && (*pClassModule).ClassLibraryInfo.is_null() {
                    status = STATUS_DRIVER_INTERNAL_ERROR;
                }
            } else {
                /* Class module exists and has ClassLibraryInfo - skip ZwLoadDriver */
                status = STATUS_SUCCESS;
            }

            /* Always set the class module regardless of ZwLoadDriver result (Again.. UCX) */
            if !pClassModule.is_null() && NT_SUCCESS!(status) {
                (*ClassBindInfo).ClassModule = pClassModule as PVOID;
                *ClassModule = pClassModule;

                InterlockedExchangeAdd(
                    core::ptr::addr_of_mut!((*pClassModule).ClientRefCount) as *mut ::core::ffi::c_long,
                    1,
                );
                break 'exit_success;
            }
        }

        // clean:
        if !pClassModule.is_null()
            && InterlockedExchangeAdd(
                core::ptr::addr_of_mut!((*pClassModule).ClassRefCount) as *mut ::core::ffi::c_long,
                -1,
            ) == 1
        {
            ClassCleanupAndFree(pClassModule);
        }
    }

    // exit_success:
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(driverServiceName));

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn ClassUnload(ClassModule: PCLASS_MODULE, RemoveFromList: BOOLEAN) {
    let mut pClassModule: PCLASS_MODULE = core::ptr::null_mut();
    let mut pClassLibInfo: PWDF_CLASS_LIBRARY_INFO = core::ptr::null_mut();
    let mut status: NTSTATUS;

    pClassModule = ClassModule;
    pClassLibInfo = (*ClassModule).ClassLibraryInfo;

    if !pClassLibInfo.is_null() && (*pClassLibInfo).ClassLibraryDeinitialize.is_some() {
        __DBGPRINT!(
            b"calling ClassLibraryDeinitialize (%p)\n\0" as *const u8 as PCSTR,
            (*(*ClassModule).ClassLibraryInfo).ClassLibraryDeinitialize,
        );

        (*(*ClassModule).ClassLibraryInfo)
            .ClassLibraryDeinitialize
            .unwrap_unchecked()();
    }

    __DBGPRINT!(
        b"Unload class library %wZ\n\0" as *const u8 as PCSTR,
        core::ptr::addr_of_mut!((*ClassModule).Service),
    );

    ClassClose(ClassModule);
    status = ZwUnloadDriver(core::ptr::addr_of_mut!((*ClassModule).Service));

    if !NT_SUCCESS!(status) {
        __DBGPRINT!(
            b"unload of class %wZ returned 0x%x (this may not be a true error if someone else attempted to stop the service first)\n\0"
                as *const u8 as PCSTR,
            core::ptr::addr_of_mut!((*pClassModule).Service),
            status,
        );
    }

    if RemoveFromList != 0 {
        ClassRemoveFromLibraryList(pClassModule);
    }
}
#[no_mangle]
pub unsafe extern "C" fn FindClassByServiceNameLocked(
    mut ServicePath: PUNICODE_STRING,
    mut LibModule: *mut PLIBRARY_MODULE,
) -> PCLASS_MODULE {
    let mut libEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut pLibModule: PLIBRARY_MODULE = core::ptr::null_mut();
    let mut classEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut pClassModule: PCLASS_MODULE = core::ptr::null_mut();
    let mut needleName: _UNICODE_STRING = core::mem::zeroed();

    'end: {
        GetNameFromPath(ServicePath, core::ptr::addr_of_mut!(needleName));

        if IsListEmpty(core::ptr::addr_of_mut!(WdfLdrGlobals.LoadedModulesList)) != 0 {
            break 'end;
        }

        libEntry = WdfLdrGlobals.LoadedModulesList.Flink;
        while libEntry != core::ptr::addr_of_mut!(WdfLdrGlobals.LoadedModulesList) {
            pLibModule = CONTAINING_RECORD!(libEntry, LIBRARY_MODULE, LibraryListEntry);

            classEntry = (*pLibModule).ClassListHead.Flink;
            while classEntry != core::ptr::addr_of_mut!((*pLibModule).ClassListHead) {
                let mut haystackName: _UNICODE_STRING = core::mem::zeroed();
                pClassModule = CONTAINING_RECORD!(classEntry, CLASS_MODULE, LibraryLinkage);
                GetNameFromPath(
                    core::ptr::addr_of_mut!((*pClassModule).Service),
                    core::ptr::addr_of_mut!(haystackName),
                );

                if RtlEqualUnicodeString(
                    core::ptr::addr_of_mut!(needleName),
                    core::ptr::addr_of_mut!(haystackName),
                    TRUE as BOOLEAN,
                ) != 0
                {
                    if !LibModule.is_null() {
                        *LibModule = pLibModule;
                    }

                    return pClassModule;
                }

                classEntry = (*classEntry).Flink;
            }

            libEntry = (*libEntry).Flink;
        }
    }

    if !LibModule.is_null() {
        *LibModule = core::ptr::null_mut();
    }

    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn LibraryUnloadClasses(LibModule: PLIBRARY_MODULE) -> PLIST_ENTRY {
    let mut classListHead: PLIST_ENTRY;
    let mut entry: PLIST_ENTRY;
    let mut pClassModule: PCLASS_MODULE;
    let mut removedList: LIST_ENTRY = core::mem::zeroed();

    InitializeListHead(core::ptr::addr_of_mut!(removedList));
    classListHead = core::ptr::addr_of_mut!((*LibModule).ClassListHead);

    if IsListEmpty(classListHead as *const LIST_ENTRY) != 0 {
        return classListHead;
    }

    while IsListEmpty(classListHead as *const LIST_ENTRY) == 0 {
        entry = RemoveHeadList(classListHead);
        InsertTailList(core::ptr::addr_of_mut!(removedList), entry);
    }

    while IsListEmpty(core::ptr::addr_of_mut!(removedList)) == 0 {
        entry = RemoveHeadList(core::ptr::addr_of_mut!(removedList));
        InitializeListHead(entry);
        pClassModule = CONTAINING_RECORD!(entry, CLASS_MODULE, LibraryLinkage);

        __DBGPRINT!((
            b"Unload class library %wZ (%p)\n\0".as_ptr() as PCSTR,
            core::ptr::addr_of_mut!((*pClassModule).Service),
            pClassModule
        ));

        ClassUnload(pClassModule, 0);
        if InterlockedExchangeAdd(
            core::ptr::addr_of_mut!((*pClassModule).ClassRefCount),
            -1,
        ) == 1
        {
            ClassCleanupAndFree(pClassModule);
        }
    }

    return classListHead;
}
#[no_mangle]
pub unsafe extern "C" fn LibraryAddToClassListLocked(
    LibModule: PLIBRARY_MODULE,
    ClassModule: PCLASS_MODULE,
) -> PLIST_ENTRY {
    let mut result: PLIST_ENTRY = core::ptr::null_mut();

    if LibModule.is_null() || ClassModule.is_null() {
        return core::ptr::null_mut();
    }

    result = core::ptr::addr_of_mut!((*ClassModule).LibraryLinkage) as PLIST_ENTRY;
    InsertHeadList(
        core::ptr::addr_of_mut!((*LibModule).ClassListHead),
        core::ptr::addr_of_mut!((*ClassModule).LibraryLinkage),
    );

    DPRINT_VERBOSE!(
        "Added class %wZ to library %wZ\n",
        core::ptr::addr_of_mut!((*ClassModule).Service),
        core::ptr::addr_of_mut!((*LibModule).ServicePath)
    );

    return result;
}
#[no_mangle]
pub unsafe extern "C" fn ClassRemoveFromLibraryList(ClassModule: PCLASS_MODULE) {
    let mut libLinkEntry: PLIST_ENTRY;
    let mut removed: BOOLEAN;

    FxLdrAcquireLoadedModuleLock();
    libLinkEntry = core::ptr::addr_of_mut!((*ClassModule).LibraryLinkage) as PLIST_ENTRY;

    if IsListEmpty(libLinkEntry as *const LIST_ENTRY) != 0 {
        removed = FALSE as BOOLEAN;
    } else {
        RemoveEntryList(libLinkEntry);
        InitializeListHead(libLinkEntry);
        removed = TRUE as BOOLEAN;
    }
    FxLdrReleaseLoadedModuleLock();

    if removed != 0 {
        if InterlockedExchangeAdd(
            core::ptr::addr_of_mut!((*ClassModule).ClassRefCount),
            -1 as core::ffi::c_long,
        ) == 1 as core::ffi::c_long
        {
            ClassCleanupAndFree(ClassModule);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClassUnlinkClient(
    ClassModule: PCLASS_MODULE,
    ClassBindInfo: PWDF_CLASS_BIND_INFO,
) {
    let mut client: PCLASS_CLIENT_MODULE = core::ptr::null_mut();
    let mut isUnlinked: BOOLEAN = FALSE;
    let mut entry: PLIST_ENTRY = core::ptr::null_mut();

    client = core::ptr::null_mut();
    ClassAcquireClientLock(core::ptr::addr_of_mut!((*ClassModule).ClientsListLock));

    entry = (*ClassModule).ClientsListHead.Flink;
    while entry != core::ptr::addr_of_mut!((*ClassModule).ClientsListHead) {
        client = CONTAINING_RECORD!(entry, CLASS_CLIENT_MODULE, ClassLinkage);
        if (*CONTAINING_RECORD!(entry, CLASS_CLIENT_MODULE, ClassLinkage)).ClientClassBindInfo
            == ClassBindInfo
        {
            isUnlinked = TRUE;
            break;
        } else {
            entry = (*entry).Flink;
        }
    }

    ClassReleaseClientLock(core::ptr::addr_of_mut!((*ClassModule).ClientsListLock));

    if isUnlinked != 0 {
        RemoveEntryList(entry);
        InitializeListHead(entry);
        LibraryAcquireClientLock((*ClassModule).Library);
        InitializeListHead(core::ptr::addr_of_mut!((*client).ClientLinkage));
        RemoveEntryList(core::ptr::addr_of_mut!((*client).ClientLinkage));
        LibraryReleaseClientLock((*ClassModule).Library);
        ExFreePoolWithTag(client as PVOID, WDFLDR_TAG as ULONG);
    }
}
#[no_mangle]
pub unsafe extern "system" fn ClassAddReference(ClassModule: PCLASS_MODULE) {
    InterlockedIncrement(core::ptr::addr_of_mut!((*ClassModule).ClassRefCount));
    DPRINT_VERBOSE!(
        b"Added reference to class %wZ, RefCount=%d\n\0" as *const u8 as PCSTR,
        core::ptr::addr_of_mut!((*ClassModule).Service),
        (*ClassModule).ClassRefCount
    );
}
#[no_mangle]
pub unsafe extern "C" fn ClassReleaseClientReference(mut ClassModule: PCLASS_MODULE) {
    let mut refs: i32;
    __DBGPRINT!("Dereference module %wZ\n", core::ptr::addr_of_mut!((*ClassModule).Service));
    refs = InterlockedDecrement(core::ptr::addr_of_mut!((*ClassModule).ClientRefCount)) as i32;

    if refs <= 0 {
        ClassUnload(ClassModule, TRUE);
    } else {
        __DBGPRINT!(
            "Dereference module %wZ still has %d references\n",
            core::ptr::addr_of_mut!((*ClassModule).Service),
            refs
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn DereferenceClassVersion(
    ClassBindInfo: PWDF_CLASS_BIND_INFO,
    BindInfo: PWDF_BIND_INFO,
    mut Globals: PWDF_COMPONENT_GLOBALS,
) {
    let mut pClassModule: PCLASS_MODULE = core::ptr::null_mut();

    UNREFERENCED_PARAMETER!(BindInfo);
    pClassModule = (*ClassBindInfo).ClassModule as PCLASS_MODULE;

    if !pClassModule.is_null() {
        if !(*pClassModule).ClassLibraryInfo.is_null()
            && (*(*pClassModule).ClassLibraryInfo)
                .ClassLibraryUnbindClient
                .is_some()
        {
            (*(*pClassModule).ClassLibraryInfo)
                .ClassLibraryUnbindClient
                .unwrap_unchecked()(
                ClassBindInfo,
                core::ptr::addr_of_mut!(Globals),
            );
        }
        ClassUnlinkClient(pClassModule, ClassBindInfo);
        ClassReleaseClientReference(pClassModule);
        (*ClassBindInfo).ClassModule = core::ptr::null_mut();
    }
}
