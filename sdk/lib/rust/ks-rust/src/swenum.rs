use crate::types::*;
use crate::glue::*;
use crate::glue::{IRP_MN_QUERY_BUS_INFORMATION, IRP_MN_QUERY_CAPABILITIES, IRP_MN_QUERY_ID, IRP_MN_QUERY_INTERFACE, IRP_MN_QUERY_PNP_DEVICE_STATE, IRP_MN_QUERY_RESOURCES, IRP_MN_QUERY_RESOURCE_REQUIREMENTS, IRP_MN_READ_CONFIG, IRP_MN_WRITE_CONFIG}; // explicit: disambiguate vs ntkrnl_sys::driver_ddk glob
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{DEVICE_CAPABILITIES, DEVICE_RELATIONS, ExGetPreviousMode, ExQueueWorkItem, _KWAIT_REASON_Executive, GUID, IRP, IoAttachDeviceToDeviceStack, IoCreateDevice, IoDeleteDevice, IoDetachDevice, IoGetCurrentIrpStackLocation, IoGetDeviceProperty, IoInvalidateDeviceRelations, IoMarkIrpPending, IoRegisterDeviceInterface, IoSetDeviceInterfaceState, KEY_BASIC_INFORMATION, KEY_FULL_INFORMATION, KPROCESSOR_MODE, KeInitializeDpc, KeInitializeEvent, KeInitializeSpinLock, KeInitializeTimer, KeQueryTimeIncrement, KeSetTimer, KeWaitForSingleObject, LARGE_INTEGER, LPWSTR, LUID, _EVENT_TYPE_NotificationEvent, _OBJECT_ATTRIBUTES, PDEVICE_CAPABILITIES, PDEVICE_OBJECT, PDEVICE_RELATIONS, PHANDLE, PIO_STACK_LOCATION, PIRP, PKEY_BASIC_INFORMATION, PNP_BUS_INFORMATION, PPNP_BUS_INFORMATION, RtlAppendUnicodeStringToString, RtlAppendUnicodeToString, RtlFreeUnicodeString, RtlGUIDFromString, RtlInitUnicodeString, RtlStringFromGUID, SeSinglePrivilegeCheck, _EVENT_TYPE_SynchronizationEvent, ZwClose, ZwCreateKey, ZwEnumerateKey, ZwOpenKey, ZwQueryKey};

// RtlConvertUlongToLuid is a header inline (ndk/rtlfuncs.h), not an ntoskrnl export — define
// it locally (LUID { LowPart: Ulong, HighPart: 0 }).
#[inline]
unsafe fn RtlConvertUlongToLuid(Ulong: u32) -> LUID {
    LUID { LowPart: Ulong, HighPart: 0 }
}
// Enum-member aliases (bindgen prefixes the enum-type name) + RtlAssert.
use ntkrnl_sys::ddk::{
    _BUS_QUERY_ID_TYPE_BusQueryDeviceID as BusQueryDeviceID,
    _BUS_QUERY_ID_TYPE_BusQueryHardwareIDs as BusQueryHardwareIDs,
    _BUS_QUERY_ID_TYPE_BusQueryInstanceID as BusQueryInstanceID,
    _DEVICE_RELATION_TYPE_BusRelations as BusRelations,
    _DEVICE_RELATION_TYPE_TargetDeviceRelation as TargetDeviceRelation,
    _WORK_QUEUE_TYPE_DelayedWorkQueue as DelayedWorkQueue,
    _INTERFACE_TYPE_InterfaceTypeUndefined as InterfaceTypeUndefined,
    _KEY_INFORMATION_CLASS_KeyBasicInformation as KeyBasicInformation,
    _KEY_INFORMATION_CLASS_KeyFullInformation as KeyFullInformation,
    _DEVICE_POWER_STATE_PowerDeviceD0 as PowerDeviceD0,
    _SYSTEM_POWER_STATE_PowerSystemWorking as PowerSystemWorking,
    DEVICE_REGISTRY_PROPERTY_DevicePropertyPhysicalDeviceObjectName as DevicePropertyPhysicalDeviceObjectName,
    RtlAssert,
};
// Pool/IRP helpers live in the sibling `misc` module; the WORK_QUEUE_ITEM init
// helper is a FORCEINLINE DDK helper provided by ntkrnl_sys.
use crate::misc::{AllocateItem, FreeItem, CompleteRequest};
use ntkrnl_sys::inline::ex::ExInitializeWorkItem;
use core::ffi::c_void;

// ---- file-local declarations (swenum-only; not shared) ----
extern "C" {
    fn wcscpy(dst: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    fn wcscat(dst: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    fn _wcsicmp(a: *const WCHAR, b: *const WCHAR) -> i32;
}
extern "system" {
    fn KeQueryTickCount(CurrentCount: *mut LARGE_INTEGER);
}

/// IsEqualGUIDAligned — byte-compare two aligned GUIDs (ks.h inline).
#[inline]
unsafe fn IsEqualGUIDAligned(a: *const GUID, b: *const GUID) -> BOOLEAN {
    let pa = a as *const u8;
    let pb = b as *const u8;
    let mut i = 0usize;
    while i < core::mem::size_of::<GUID>() {
        if *pa.add(i) != *pb.add(i) {
            return 0u8;
        }
        i += 1;
    }
    1u8
}

/// ASSERT / NT_VERIFY — retail (NDEBUG) builds evaluate the expression only.
#[inline]
unsafe fn ASSERT(_cond: bool) {}
#[inline]
unsafe fn NT_VERIFY(_cond: bool) {}

/// RtlStringCbCopyW — bounded wide-string copy (ntstrsafe.h inline).
#[inline]
unsafe fn RtlStringCbCopyW(
    dst: NTSTRSAFE_PWSTR,
    cb_dest: usize,
    src: NTSTRSAFE_PCWSTR,
) -> NTSTATUS {
    let cch = cb_dest / core::mem::size_of::<WCHAR>();
    if cch == 0 {
        return STATUS_INVALID_PARAMETER;
    }
    let mut i: usize = 0;
    while i + 1 < cch && *src.add(i) != 0 {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    *dst.add(i) = 0;
    if *src.add(i) != 0 {
        STATUS_BUFFER_OVERFLOW
    } else {
        STATUS_SUCCESS
    }
}

// ---- DDK swenum.h types (sdk/include/ddk/swenum.h) — used only by this module ----
#[repr(C)]
pub struct SWENUM_INTERFACE {
    pub Size: USHORT,
    pub Version: USHORT,
    pub Context: PVOID,
    pub InterfaceReference: Option<unsafe extern "system" fn(Context: PVOID)>,
    pub InterfaceDereference: Option<unsafe extern "system" fn(Context: PVOID)>,
}
#[repr(C)]
pub struct BUS_INTERFACE_SWENUM {
    pub Interface: SWENUM_INTERFACE,
    pub ReferenceDeviceObject: Option<unsafe extern "system" fn(Context: PVOID)>,
    pub DereferenceDeviceObject: Option<unsafe extern "system" fn(Context: PVOID)>,
    pub QueryReferenceString:
        Option<unsafe extern "system" fn(Context: PVOID, String: *mut PWCHAR) -> NTSTATUS>,
}
pub type PBUS_INTERFACE_SWENUM = *mut BUS_INTERFACE_SWENUM;

#[repr(C)]
pub struct SWENUM_INSTALL_INTERFACE {
    pub DeviceId: GUID,
    pub InterfaceId: GUID,
    pub ReferenceString: [WCHAR; 1usize],
}
pub type PSWENUM_INSTALL_INTERFACE = *mut SWENUM_INSTALL_INTERFACE;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut KsDeviceCount: LONG = 0 as LONG;

#[cfg(not(ndebug))]
const NDEBUG: u32 = 1;

pub type PKSP_BUS_ENUM_CALLBACK = Option<unsafe extern "system" fn(
    hKey: PHANDLE,
    BusDeviceExtension: PBUS_ENUM_DEVICE_EXTENSION,
    DummyEntry: PBUS_DEVICE_ENTRY,
    RootName: LPWSTR,
    DirectoryName: LPWSTR,
) -> NTSTATUS>;
#[no_mangle]
pub unsafe extern "C" fn KspCreatePDO(
    BusDeviceExtension: PBUS_ENUM_DEVICE_EXTENSION,
    DeviceEntry: PBUS_DEVICE_ENTRY,
    OutDeviceObject: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Buffer: [WCHAR; 50] = [0; 50];
    let mut CurDeviceId: ULONG;
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Status: NTSTATUS;
    let mut DeviceExtension: PCOMMON_DEVICE_EXTENSION;

    CurDeviceId = InterlockedIncrement(&raw mut KsDeviceCount) as ULONG;

    swprintf(
        Buffer.as_mut_ptr(),
        "\\Device\\KSENUM%08x\0".as_ptr() as *const WCHAR,
        CurDeviceId,
    );

    RtlInitUnicodeString(&mut DeviceName, Buffer.as_ptr());

    Status = IoCreateDevice(
        (*((*BusDeviceExtension).BusDeviceObject)).DriverObject,
        core::mem::size_of::<PVOID>() as ULONG,
        &mut DeviceName,
        FILE_DEVICE_UNKNOWN,
        0,
        0u8,
        &mut DeviceObject,
    );

    if !NT_SUCCESS(Status) {
        return Status;
    }

    DeviceExtension = AllocateItem(
        NonPagedPool,
        core::mem::size_of::<COMMON_DEVICE_EXTENSION>() as usize,
    ) as PCOMMON_DEVICE_EXTENSION;

    if DeviceExtension.is_null() {
        IoDeleteDevice(DeviceObject);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    *((*DeviceObject).DeviceExtension as *mut PVOID) = DeviceExtension as PVOID;

    (*DeviceExtension).IsBus = 0;
    *(*DeviceExtension).__bindgen_anon_1.DeviceObject.as_mut() = DeviceObject;
    *(*DeviceExtension).__bindgen_anon_2.DeviceEntry.as_mut() = DeviceEntry;
    (*DeviceExtension).BusDeviceExtension = BusDeviceExtension;

    (*DeviceEntry).DeviceState = NotStarted;

    KeQueryTickCount(&mut (*DeviceEntry).TimeCreated);

    (*DeviceObject).Flags |= DO_POWER_PAGABLE as u32;
    (*DeviceObject).Flags &= !(DO_DEVICE_INITIALIZING as u32);

    *OutDeviceObject = DeviceObject;

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn KspRegisterDeviceAssociation(
    BusDeviceExtension: PBUS_ENUM_DEVICE_EXTENSION,
    DeviceEntry: PBUS_DEVICE_ENTRY,
    BusInstanceEntry: PBUS_INSTANCE_ENTRY,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ReferenceString: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut ReferenceString, (*DeviceEntry).DeviceName);

    Status = IoRegisterDeviceInterface(
        (*BusDeviceExtension).PhysicalDeviceObject,
        &mut (*BusInstanceEntry).InterfaceGuid,
        &mut ReferenceString,
        &mut (*BusInstanceEntry).SymbolicLink,
    );

    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = IoSetDeviceInterfaceState(&mut (*BusInstanceEntry).SymbolicLink, 1u8);

    if !NT_SUCCESS(Status) {
        FreeItem((*BusInstanceEntry).SymbolicLink.Buffer as *mut core::ffi::c_void);
        return Status;
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn KspRemoveDeviceAssociations(
    DeviceEntry: PBUS_DEVICE_ENTRY,
) {
    let mut Entry: PLIST_ENTRY;
    let mut CurEntry: PBUS_INSTANCE_ENTRY;

    Entry = (*DeviceEntry).DeviceInterfaceList.Flink;

    while Entry != &mut (*DeviceEntry).DeviceInterfaceList as *mut _ as PLIST_ENTRY {
        CurEntry = core::ptr::with_exposed_provenance_mut::<BUS_INSTANCE_ENTRY>((Entry as usize)
                .wrapping_sub(core::mem::offset_of!(BUS_INSTANCE_ENTRY, Entry) as usize),
        );

        if (*CurEntry).SymbolicLink.Buffer.is_null() {
            RtlAssert(
                b"CurEntry->SymbolicLink.Buffer\0".as_ptr() as PVOID,
                b"drivers/ksfilter/ks/swenum.c\0".as_ptr() as PVOID,
                148u32,
                core::ptr::null_mut(),
            );
        }

        IoSetDeviceInterfaceState(&mut (*CurEntry).SymbolicLink, 0u8);

        FreeItem((*CurEntry).SymbolicLink.Buffer as PVOID);

        RemoveEntryList(Entry);

        Entry = (*Entry).Flink;

        FreeItem(CurEntry as PVOID);
    }
}
#[no_mangle]
pub unsafe extern "C" fn KspEnumerateBusRegistryKeys(
    hKey: HANDLE,
    ReferenceString: LPWSTR,
    Callback: PKSP_BUS_ENUM_CALLBACK,
    BusDeviceExtension: PBUS_ENUM_DEVICE_EXTENSION,
    DeviceEntry: PBUS_DEVICE_ENTRY,
) -> NTSTATUS {
    let mut String: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut hNewKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut ResultLength: ULONG = 0;
    let mut Index: ULONG = 0;
    let mut KeyInfoLength: ULONG = 0;
    let mut KeyInformation: KEY_FULL_INFORMATION = core::mem::zeroed();
    let mut KeyInfo: PKEY_BASIC_INFORMATION = core::ptr::null_mut();

    RtlInitUnicodeString(&mut String, ReferenceString);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = hKey;
    ObjectAttributes.ObjectName = &mut String;
    ObjectAttributes.Attributes = OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwOpenKey(&mut hNewKey, GENERIC_READ, &mut ObjectAttributes);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ZwQueryKey(
        hNewKey,
        KeyFullInformation,
        &mut KeyInformation as *mut _ as PVOID,
        core::mem::size_of::<KEY_FULL_INFORMATION>() as ULONG,
        &mut ResultLength,
    );
    if !NT_SUCCESS(Status) {
        ZwClose(hNewKey);
        return Status;
    }

    KeyInfoLength = (KeyInformation.MaxNameLen as usize)
        .wrapping_add(core::mem::size_of::<KEY_BASIC_INFORMATION>())
        .wrapping_add(core::mem::size_of::<WCHAR>()) as ULONG;

    KeyInfo = AllocateItem(NonPagedPool, KeyInfoLength as SIZE_T) as PKEY_BASIC_INFORMATION;
    if KeyInfo.is_null() {
        ZwClose(hNewKey);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Index = 0;
    while Index < KeyInformation.SubKeys {
        Status = ZwEnumerateKey(
            hNewKey,
            Index,
            KeyBasicInformation,
            KeyInfo as PVOID,
            KeyInfoLength,
            &mut ResultLength,
        );
        if NT_SUCCESS(Status) {
            Status = (Callback.unwrap())(
                &mut hNewKey,
                BusDeviceExtension,
                DeviceEntry,
                &mut (*KeyInfo).Name as *mut _ as LPWSTR,
                ReferenceString,
            );
            if !NT_SUCCESS(Status) {
                break;
            }
        }
        Index = Index.wrapping_add(1);
    }

    FreeItem(KeyInfo as PVOID);
    ZwClose(hNewKey);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn KspCreateDeviceAssociation(
    hKey: PHANDLE,
    BusDeviceExtension: PBUS_ENUM_DEVICE_EXTENSION,
    DeviceEntry: PBUS_DEVICE_ENTRY,
    InterfaceString: LPWSTR,
    ReferenceString: LPWSTR,
) -> NTSTATUS {
    let mut InterfaceGUID: GUID = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Entry: PLIST_ENTRY;
    let mut CurEntry: PBUS_INSTANCE_ENTRY;
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut DeviceName, InterfaceString);
    RtlGUIDFromString(&mut DeviceName, &mut InterfaceGUID);

    Entry = (*DeviceEntry).DeviceInterfaceList.Flink;
    while Entry != &mut (*DeviceEntry).DeviceInterfaceList {
        CurEntry = CONTAINING_RECORD!(Entry, BUS_INSTANCE_ENTRY, Entry);

        if IsEqualGUIDAligned(&(*CurEntry).InterfaceGuid, &InterfaceGUID) != 0 {
            return STATUS_SUCCESS;
        }

        Entry = (*Entry).Flink;
    }

    CurEntry = AllocateItem(NonPagedPool, core::mem::size_of::<BUS_INSTANCE_ENTRY>()) as PBUS_INSTANCE_ENTRY;
    if CurEntry.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::copy_nonoverlapping(
        &InterfaceGUID as *const GUID,
        &mut (*CurEntry).InterfaceGuid as *mut GUID,
        1,
    );

    Status = KspRegisterDeviceAssociation(BusDeviceExtension, DeviceEntry, CurEntry);

    if NT_SUCCESS(Status) {
        InsertTailList(&mut (*DeviceEntry).DeviceInterfaceList, &mut (*CurEntry).Entry);
    } else {
        FreeItem(CurEntry as *mut core::ffi::c_void);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KspCreateDeviceReference(
    hKey: PHANDLE,
    BusDeviceExtension: PBUS_ENUM_DEVICE_EXTENSION,
    DummyEntry: PBUS_DEVICE_ENTRY,
    InterfaceId: LPWSTR,
    DeviceId: LPWSTR,
) -> NTSTATUS {
    let mut DeviceName: LPWSTR = core::ptr::null_mut();
    let mut Length: SIZE_T;
    let mut Entry: PLIST_ENTRY;
    let mut DeviceEntry: PBUS_DEVICE_ENTRY = core::ptr::null_mut();
    let mut ItemExists: BOOLEAN = 0u8;
    let mut String: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Status: NTSTATUS;
    let mut OldLevel: KIRQL = 0;

    Length = wcslen(DeviceId) + wcslen(InterfaceId);
    Length += 2;

    DeviceName = AllocateItem(NonPagedPool, Length * core::mem::size_of::<WCHAR>()) as LPWSTR;
    if DeviceName.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    wcscpy(DeviceName, DeviceId);
    wcscat(DeviceName, L!("&\0").as_ptr() as LPWSTR);
    wcscat(DeviceName, InterfaceId);

    Entry = (*BusDeviceExtension).Common.Entry.Flink;
    while Entry != &mut (*BusDeviceExtension).Common.Entry as *mut _ as PLIST_ENTRY {
        DeviceEntry = (Entry as usize).wrapping_sub(
            core::mem::offset_of!(BUS_DEVICE_ENTRY, Entry),
        ) as *mut BUS_DEVICE_ENTRY as PBUS_DEVICE_ENTRY;

        if _wcsicmp((*DeviceEntry).DeviceName, DeviceName) == 0 {
            ItemExists = 1u8;
            break;
        }
        Entry = (*Entry).Flink;
    }

    if ItemExists == 0 {
        DeviceEntry = AllocateItem(NonPagedPool, core::mem::size_of::<BUS_DEVICE_ENTRY>()) as PBUS_DEVICE_ENTRY;
        if DeviceEntry.is_null() {
            FreeItem(DeviceName as PVOID);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        InitializeListHead(&mut (*DeviceEntry).DeviceInterfaceList);
        InitializeListHead(&mut (*DeviceEntry).IrpPendingList);

        RtlInitUnicodeString(&mut String, DeviceId);
        RtlGUIDFromString(&mut String, &mut (*DeviceEntry).DeviceGuid);

        (*DeviceEntry).DeviceName = DeviceName;
        (*DeviceEntry).Instance = DeviceName.offset(wcslen(DeviceId) as isize).offset(1);

        (*DeviceEntry).BusId = AllocateItem(
            NonPagedPool,
            (wcslen(DeviceId) + 1) * core::mem::size_of::<WCHAR>(),
        ) as LPWSTR;
        if (*DeviceEntry).BusId.is_null() {
            FreeItem(DeviceName as PVOID);
            FreeItem(DeviceEntry as PVOID);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        wcscpy((*DeviceEntry).BusId, DeviceId);
    }

    Status = KspEnumerateBusRegistryKeys(
        hKey as HANDLE,
        InterfaceId,
        Some(KspCreateDeviceAssociation as unsafe extern "system" fn(PHANDLE, PBUS_ENUM_DEVICE_EXTENSION, PBUS_DEVICE_ENTRY, LPWSTR, LPWSTR) -> NTSTATUS),
        BusDeviceExtension,
        DeviceEntry,
    );

    if IsListEmpty(&mut (*DeviceEntry).DeviceInterfaceList) {
        FreeItem((*DeviceEntry).BusId as PVOID);
        FreeItem((*DeviceEntry).DeviceName as PVOID);
        FreeItem(DeviceEntry as PVOID);
        return STATUS_INVALID_DEVICE_STATE;
    }

    if !NT_SUCCESS(Status) {
        KspRemoveDeviceAssociations(DeviceEntry);
        FreeItem((*DeviceEntry).BusId as PVOID);
        FreeItem((*DeviceEntry).DeviceName as PVOID);
        FreeItem(DeviceEntry as PVOID);
        return Status;
    }

    if ItemExists == 0 {
        KeAcquireSpinLock(&mut (*BusDeviceExtension).Lock as *mut _ as *mut usize, &mut OldLevel);
        InsertTailList(
            &mut (*BusDeviceExtension).Common.Entry,
            &mut (*DeviceEntry).Entry,
        );
        KeReleaseSpinLock(&mut (*BusDeviceExtension).Lock as *mut _ as *mut usize, OldLevel);
    }

    Status
}
#[no_mangle]
pub unsafe extern "system" fn KspCreateDeviceReferenceTrampoline(
    hKey: PHANDLE,
    BusDeviceExtension: PBUS_ENUM_DEVICE_EXTENSION,
    DummyEntry: PBUS_DEVICE_ENTRY,
    DeviceCategory: LPWSTR,
    ReferenceString: LPWSTR,
) -> NTSTATUS {
    return KspEnumerateBusRegistryKeys(
        *hKey,
        DeviceCategory,
        Some(KspCreateDeviceReference),
        BusDeviceExtension,
        DummyEntry,
    );
}
#[no_mangle]
pub unsafe extern "C" fn KspOpenBusRegistryKey(
    BusDeviceExtension: PBUS_ENUM_DEVICE_EXTENSION,
    hKey: PHANDLE,
) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut (*BusDeviceExtension).ServicePath as *mut _ as PUNICODE_STRING;
    ObjectAttributes.Attributes = OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    ZwCreateKey(
        hKey,
        GENERIC_READ | GENERIC_WRITE,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    )
}
#[no_mangle]
pub unsafe extern "C" fn KspScanBus(
    BusDeviceExtension: PBUS_ENUM_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut hKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = KspOpenBusRegistryKey(BusDeviceExtension, &mut hKey);

    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = KspEnumerateBusRegistryKeys(
        hKey,
        core::ptr::null_mut(),
        Some(KspCreateDeviceReferenceTrampoline),
        BusDeviceExtension,
        core::ptr::null_mut(),
    );

    ZwClose(hKey);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KspBusQueryReferenceString(
    Context: *mut core::ffi::c_void,
    String: *mut PWCHAR,
) -> NTSTATUS {
    let BusDeviceExtension: PBUS_ENUM_DEVICE_EXTENSION = Context as PBUS_ENUM_DEVICE_EXTENSION;

    let Length = wcslen((*BusDeviceExtension).BusIdentifier.as_ptr()).wrapping_add(1);

    let Name = AllocateItem(PagedPool, Length.wrapping_mul(core::mem::size_of::<WCHAR>())) as LPWSTR;

    if Name.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    wcscpy(Name, (*BusDeviceExtension).BusIdentifier.as_ptr());

    *String = Name as PWCHAR;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KspBusDeviceReference(Context: PVOID) {
    let ChildDeviceExtension: PCOMMON_DEVICE_EXTENSION = Context as PCOMMON_DEVICE_EXTENSION;
    InterlockedIncrement(
        core::ptr::addr_of_mut!((*ChildDeviceExtension).__bindgen_anon_1.DeviceReferenceCount) as PLONG,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KspBusDeviceDereference(
    Context: PVOID,
) {
    let ChildDeviceExtension = Context as PCOMMON_DEVICE_EXTENSION;
    InterlockedDecrement(
        core::ptr::addr_of_mut!((*ChildDeviceExtension).__bindgen_anon_1.DeviceReferenceCount) as PLONG,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KspBusReferenceDeviceObject(
    Context: PVOID,
) {
    let ChildDeviceExtension: PCOMMON_DEVICE_EXTENSION = Context as PCOMMON_DEVICE_EXTENSION;
    InterlockedIncrement(
        core::ptr::addr_of_mut!((*ChildDeviceExtension).DeviceObjectReferenceCount) as PLONG,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KspBusDereferenceDeviceObject(
    Context: PVOID,
) {
    let ChildDeviceExtension: PCOMMON_DEVICE_EXTENSION = Context as PCOMMON_DEVICE_EXTENSION;
    InterlockedDecrement(
        core::ptr::addr_of_mut!((*ChildDeviceExtension).DeviceObjectReferenceCount) as PLONG,
    );
}
#[no_mangle]
pub unsafe extern "C" fn KspQueryBusDeviceInterface(
    ChildDeviceExtension: PCOMMON_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let Interface = (*IoStack).Parameters.QueryInterface.Interface as PBUS_INTERFACE_SWENUM;
    (*Interface).Interface.Size = core::mem::size_of::<BUS_INTERFACE_SWENUM>() as USHORT;
    (*Interface).Interface.Version = BUS_INTERFACE_SWENUM_VERSION as USHORT;
    (*Interface).Interface.Context = ChildDeviceExtension as PVOID;
    (*Interface).Interface.InterfaceReference = Some(KspBusDeviceReference as unsafe extern "system" fn(PVOID) -> ());
    (*Interface).Interface.InterfaceDereference = Some(KspBusDeviceDereference as unsafe extern "system" fn(PVOID) -> ());
    (*Interface).ReferenceDeviceObject = Some(KspBusReferenceDeviceObject as unsafe extern "system" fn(PVOID) -> ());
    (*Interface).DereferenceDeviceObject = Some(KspBusDereferenceDeviceObject as unsafe extern "system" fn(PVOID) -> ());
    (*Interface).QueryReferenceString = Some(KspBusQueryReferenceString as unsafe extern "system" fn(PVOID, *mut PWCHAR) -> NTSTATUS);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn KspEnableBusDeviceInterface(
    DeviceEntry: PBUS_DEVICE_ENTRY,
    bEnable: BOOLEAN,
) -> NTSTATUS {
    let mut Entry: PLIST_ENTRY;
    let mut InstanceEntry: PBUS_INSTANCE_ENTRY;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    Entry = (*DeviceEntry).DeviceInterfaceList.Flink;

    while Entry != &mut (*DeviceEntry).DeviceInterfaceList as *mut _ as PLIST_ENTRY {
        InstanceEntry = CONTAINING_RECORD!(Entry, BUS_INSTANCE_ENTRY, Entry) as PBUS_INSTANCE_ENTRY;

        Status = IoSetDeviceInterfaceState(&mut (*InstanceEntry).SymbolicLink, bEnable);

        if !NT_SUCCESS(Status) {
            break;
        }

        Entry = (*Entry).Flink;
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn KspDoReparseForIrp(
    Irp: PIRP,
    DeviceEntry: PBUS_DEVICE_ENTRY,
) -> NTSTATUS {
    let mut Length: usize;
    let mut Buffer: LPWSTR;
    let IoStack: PIO_STACK_LOCATION;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    Length = wcslen((*DeviceEntry).PDODeviceName as *const u16);
    Length += wcslen((*DeviceEntry).Instance as *const u16);
    Length += 2;

    Buffer = ExAllocatePoolWithTag(
        NonPagedPool,
        Length.wrapping_mul(core::mem::size_of::<u16>()),
        1833856841,
    ) as LPWSTR;
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    swprintf(
        Buffer as *mut u16,
        &core::mem::transmute::<[u8; 12], [u16; 6]>(*b"%\0s\0\\\0%\0s\0\0\0") as *const u16,
        (*DeviceEntry).PDODeviceName,
        (*DeviceEntry).Instance,
    );

    ExFreePoolWithTag(
        (*(*IoStack).FileObject).FileName.Buffer as *mut core::ffi::c_void,
        1833856841,
    );

    RtlInitUnicodeString(
        &mut (*(*IoStack).FileObject).FileName,
        Buffer as *const u16,
    );

    STATUS_REPARSE
}
#[no_mangle]
pub unsafe extern "C" fn KspCompletePendingIrps(
    DeviceEntry: PBUS_DEVICE_ENTRY,
    ResultCode: NTSTATUS,
) {
    let mut Entry: PLIST_ENTRY;
    let mut Irp: PIRP;
    let mut Status: NTSTATUS;

    while !IsListEmpty(&mut (*DeviceEntry).IrpPendingList) {
        Entry = RemoveHeadList(&mut (*DeviceEntry).IrpPendingList);

        Irp = (Entry as usize).wrapping_sub(
            core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry),
        ) as *mut IRP as PIRP;

        if ResultCode == STATUS_REPARSE {
            Status = KspDoReparseForIrp(Irp, DeviceEntry);
        } else {
            Status = ResultCode;
        }

        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;

        CompleteRequest(Irp, IO_NO_INCREMENT as i8);
    }
}
#[no_mangle]
pub unsafe extern "C" fn KspStartBusDevice(
    DeviceObject: PDEVICE_OBJECT,
    ChildDeviceExtension: PCOMMON_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut PDOName: [WCHAR; 256] = [0; 256];
    let mut Status: NTSTATUS;
    let mut ResultLength: ULONG = 0;
    let mut Name: LPWSTR;
    let mut NameLength: ULONG;
    let mut DeviceEntry: PBUS_DEVICE_ENTRY;

    Status = IoGetDeviceProperty(
        DeviceObject,
        DevicePropertyPhysicalDeviceObjectName,
        core::mem::size_of::<[WCHAR; 256]>() as ULONG,
        PDOName.as_mut_ptr() as PVOID,
        &mut ResultLength,
    );

    if !NT_SUCCESS(Status) {
        return Status;
    }

    NameLength = ResultLength + core::mem::size_of::<WCHAR>() as ULONG;
    Name = AllocateItem(NonPagedPool, NameLength as SIZE_T) as LPWSTR;
    if Name.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    NT_VERIFY(NT_SUCCESS(RtlStringCbCopyW(
        Name as NTSTRSAFE_PWSTR,
        NameLength as usize,
        PDOName.as_ptr() as NTSTRSAFE_PCWSTR,
    )));

    DeviceEntry = (*(*ChildDeviceExtension).__bindgen_anon_2.DeviceEntry.as_ref()) as PBUS_DEVICE_ENTRY;

    ASSERT(!DeviceEntry.is_null());

    (*DeviceEntry).PDODeviceName = Name;
    (*DeviceEntry).DeviceState = Started;

    KeQueryTickCount(&mut (*DeviceEntry).TimeCreated);

    DbgPrint(
        b"KspStartBusDevice Name %S DeviceName %S Instance %S Started\n\0".as_ptr() as PCSTR,
        Name,
        (*DeviceEntry).DeviceName,
        (*DeviceEntry).Instance,
    );

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn KspQueryBusDeviceCapabilities(
    ChildDeviceExtension: PCOMMON_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let Capabilities = (*IoStack).Parameters.DeviceCapabilities.Capabilities;

    core::ptr::write_bytes(Capabilities as *mut u8, 0, core::mem::size_of::<DEVICE_CAPABILITIES>());

    (*Capabilities).set_UniqueID(1);
    (*Capabilities).set_SilentInstall(1);
    (*Capabilities).set_SurpriseRemovalOK(1);
    (*Capabilities).Address = 0;
    (*Capabilities).UINumber = 0;
    (*Capabilities).SystemWake = PowerSystemWorking;
    (*Capabilities).DeviceWake = PowerDeviceD0;

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn KspQueryBusInformation(
    ChildDeviceExtension: PCOMMON_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let BusInformation: PPNP_BUS_INFORMATION = AllocateItem(
        PagedPool,
        core::mem::size_of::<PNP_BUS_INFORMATION>(),
    ) as PPNP_BUS_INFORMATION;

    if BusInformation.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*BusInformation).BusNumber = 0;
    (*BusInformation).LegacyBusType = InterfaceTypeUndefined;
    core::ptr::copy_nonoverlapping(
        &KSMEDIUMSETID_Standard as *const _ as *const u8,
        &mut (*BusInformation).BusTypeGuid as *mut _ as *mut u8,
        core::mem::size_of::<GUID>(),
    );

    (*Irp).IoStatus.Information = BusInformation  as _;

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn KspQueryBusDevicePnpState(
    ChildDeviceExtension: PCOMMON_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    (*Irp).IoStatus.Information = (PNP_DEVICE_DONT_DISPLAY_IN_UI | PNP_DEVICE_NOT_DISABLEABLE)  as _;
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn KspQueryId(
    ChildDeviceExtension: PCOMMON_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let IdType = (*IoStack).Parameters.QueryId.IdType;

    if IdType == BusQueryInstanceID {
        let DeviceEntry = (*(*ChildDeviceExtension).__bindgen_anon_2.DeviceEntry.as_ref()) as PBUS_DEVICE_ENTRY;
        let Length = wcslen((*DeviceEntry).Instance).wrapping_add(2);
        let Name = AllocateItem(PagedPool, Length.wrapping_mul(core::mem::size_of::<WCHAR>())) as LPWSTR;
        if Name.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        wcscpy(Name, (*DeviceEntry).Instance);
        (*Irp).IoStatus.Information = Name  as _;
        STATUS_SUCCESS
    } else if IdType == BusQueryDeviceID || IdType == BusQueryHardwareIDs {
        let DeviceEntry = (*(*ChildDeviceExtension).__bindgen_anon_2.DeviceEntry.as_ref()) as PBUS_DEVICE_ENTRY;
        let BusDeviceExtension = (*ChildDeviceExtension).BusDeviceExtension as PBUS_ENUM_DEVICE_EXTENSION;
        let mut Length = wcslen((*BusDeviceExtension).BusIdentifier.as_ptr());
        Length = Length.wrapping_add(wcslen((*DeviceEntry).BusId));
        Length = Length.wrapping_add(4);
        let Name = AllocateItem(PagedPool, Length.wrapping_mul(core::mem::size_of::<WCHAR>())) as LPWSTR;
        if Name.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        wcscpy(Name, (*BusDeviceExtension).BusIdentifier.as_ptr());
        wcscat(Name, L!("\\\0").as_ptr() as *const WCHAR);
        wcscat(Name, (*DeviceEntry).BusId);
        (*Irp).IoStatus.Information = Name  as _;
        STATUS_SUCCESS
    } else {
        (*Irp).IoStatus.__bindgen_anon_1.Status
    }
}
#[no_mangle]
pub unsafe extern "C" fn KspInstallInterface(
    BusDeviceExtension: PBUS_ENUM_DEVICE_EXTENSION,
    InstallInterface: PSWENUM_INSTALL_INTERFACE,
) -> NTSTATUS {
    let mut Length: usize;
    let mut Index: usize;
    let mut DeviceString: _UNICODE_STRING = core::mem::zeroed();
    let mut InterfaceString: _UNICODE_STRING = core::mem::zeroed();
    let mut ReferenceString: _UNICODE_STRING = core::mem::zeroed();
    let mut hKey: HANDLE = core::ptr::null_mut();
    let mut hDeviceKey: HANDLE = core::ptr::null_mut();
    let mut hInterfaceKey: HANDLE = core::ptr::null_mut();
    let mut hReferenceKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

    Length = wcslen((*InstallInterface).ReferenceString.as_ptr()) as usize;

    Index = 0;
    while Index < Length {
        // ReferenceString is a C flexible array (`WCHAR ReferenceString[1]`,
        // ddk/swenum.h:18) in an IOCTL input buffer the caller sizes to the real
        // string, so bindgen's [WCHAR; 1] bounds-checks this scan and panics at the
        // second character. Constant index 0 is checked at compile time.
        let c = *(&raw const (*InstallInterface).ReferenceString[0]).add(Index);
        if c <= ' ' as u16 || c > '~' as u16 || c == ',' as u16 || c == '\\' as u16 || c == '/' as u16 {
            return STATUS_INVALID_PARAMETER;
        }
        Index += 1;
    }

    Status = KspOpenBusRegistryKey(BusDeviceExtension, &mut hKey);
    if NT_SUCCESS(Status) {
        Status = RtlStringFromGUID(&(*InstallInterface).DeviceId, &mut DeviceString);
        if NT_SUCCESS(Status) {
            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
            ObjectAttributes.RootDirectory = hKey;
            ObjectAttributes.ObjectName = &mut DeviceString;
            ObjectAttributes.Attributes = OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            Status = ZwCreateKey(
                &mut hDeviceKey,
                GENERIC_WRITE,
                &mut ObjectAttributes,
                0,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
            );
            if NT_SUCCESS(Status) {
                Status = RtlStringFromGUID(&(*InstallInterface).InterfaceId, &mut InterfaceString);
                if NT_SUCCESS(Status) {
                    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
                    ObjectAttributes.RootDirectory = hDeviceKey;
                    ObjectAttributes.ObjectName = &mut InterfaceString;
                    ObjectAttributes.Attributes = OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE;
                    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
                    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

                    Status = ZwCreateKey(
                        &mut hInterfaceKey,
                        GENERIC_WRITE,
                        &mut ObjectAttributes,
                        0,
                        core::ptr::null_mut(),
                        0,
                        core::ptr::null_mut(),
                    );
                    if NT_SUCCESS(Status) {
                        RtlInitUnicodeString(
                            &mut ReferenceString,
                            (*InstallInterface).ReferenceString.as_ptr(),
                        );

                        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
                        ObjectAttributes.RootDirectory = hInterfaceKey;
                        ObjectAttributes.ObjectName = &mut ReferenceString;
                        ObjectAttributes.Attributes = OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE;
                        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
                        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

                        Status = ZwCreateKey(
                            &mut hReferenceKey,
                            GENERIC_WRITE,
                            &mut ObjectAttributes,
                            0,
                            core::ptr::null_mut(),
                            0,
                            core::ptr::null_mut(),
                        );
                        if NT_SUCCESS(Status) {
                            ZwClose(hReferenceKey);
                        }
                    }
                    RtlFreeUnicodeString(&mut InterfaceString);
                    ZwClose(hInterfaceKey);
                }
                ZwClose(hDeviceKey);
            }
            RtlFreeUnicodeString(&mut DeviceString);
        }
        ZwClose(hKey);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KspInstallBusEnumInterface(
    Ctx: PVOID,
) {
    let IoStack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;
    let mut Entry: PLIST_ENTRY;
    let mut DeviceEntry: PBUS_DEVICE_ENTRY;
    let InstallInterface: PSWENUM_INSTALL_INTERFACE;
    let mut OldLevel: KIRQL = 0;
    let Context: PBUS_INSTALL_ENUM_CONTEXT = Ctx as PBUS_INSTALL_ENUM_CONTEXT;

    IoStack = IoGetCurrentIrpStackLocation((*Context).Irp);

    InstallInterface = (*(*Context).Irp).AssociatedIrp.SystemBuffer as PSWENUM_INSTALL_INTERFACE;

    if ((*IoStack).Parameters.DeviceIoControl.InputBufferLength as usize) < core::mem::size_of::<SWENUM_INSTALL_INTERFACE>() {
        (*Context).Status = STATUS_INVALID_PARAMETER;
        KeSetEvent(&mut (*Context).Event as *mut _ as PVOID, 0, 0u8);
        return;
    }

    Status = KspInstallInterface((*Context).BusDeviceExtension, InstallInterface);
    if !NT_SUCCESS(Status) {
        (*Context).Status = Status;
        KeSetEvent(&mut (*Context).Event as *mut _ as PVOID, 0, 0u8);
        return;
    }

    Status = KspScanBus((*Context).BusDeviceExtension);

    KeAcquireSpinLock(&mut (*(*Context).BusDeviceExtension).Lock as *mut _ as *mut usize, &mut OldLevel);

    Entry = (*(*Context).BusDeviceExtension).Common.Entry.Flink;
    while Entry != &mut (*(*Context).BusDeviceExtension).Common.Entry {
        DeviceEntry = CONTAINING_RECORD!(Entry, BUS_DEVICE_ENTRY, Entry) as PBUS_DEVICE_ENTRY;
        if IsEqualGUIDAligned(&mut (*DeviceEntry).DeviceGuid, &mut (*InstallInterface).DeviceId) != 0 {
            if (*DeviceEntry).PDO.is_null() {
                KeReleaseSpinLock(&mut (*(*Context).BusDeviceExtension).Lock as *mut _ as *mut usize, OldLevel);

                Status = KspCreatePDO(
                    (*Context).BusDeviceExtension,
                    DeviceEntry,
                    &mut (*DeviceEntry).PDO,
                );

                KeAcquireSpinLock(&mut (*(*Context).BusDeviceExtension).Lock as *mut _ as *mut usize, &mut OldLevel);

                break;
            }
        }

        Entry = (*Entry).Flink;
    }

    KeReleaseSpinLock(&mut (*(*Context).BusDeviceExtension).Lock as *mut _ as *mut usize, OldLevel);

    IoInvalidateDeviceRelations(
        (*(*Context).BusDeviceExtension).PhysicalDeviceObject,
        BusRelations,
    );

    (*Context).Status = Status;

    KeSetEvent(&mut (*Context).Event as *mut _ as PVOID, 0, 0u8);
}

#[no_mangle]
pub unsafe extern "system" fn KspBusWorkerRoutine(Parameter: PVOID) {
    let BusDeviceExtension: PBUS_ENUM_DEVICE_EXTENSION = Parameter as PBUS_ENUM_DEVICE_EXTENSION;
    let mut DoInvalidate: BOOLEAN = 0u8;
    let mut OldLevel: KIRQL = 0;

    KeAcquireSpinLock(&raw mut (*BusDeviceExtension).Lock as *mut usize, &mut OldLevel);

    let mut Time: LARGE_INTEGER = core::mem::zeroed();
    KeQueryTickCount(&mut Time);

    let mut Entry: PLIST_ENTRY = (*BusDeviceExtension).Common.Entry.Flink;
    while Entry != &raw mut (*BusDeviceExtension).Common.Entry {
        let DeviceEntry: PBUS_DEVICE_ENTRY = CONTAINING_RECORD!(Entry, BUS_DEVICE_ENTRY, Entry);

        if !(*DeviceEntry).PDO.is_null() {
            if (*DeviceEntry).DeviceState == NotStarted {
                let mut Diff: LARGE_INTEGER = core::mem::zeroed();
                Diff.QuadPart = (Time.QuadPart - (*DeviceEntry).TimeCreated.QuadPart) * KeQueryTimeIncrement() as i64;

                if Diff.QuadPart > 15000i64 * 10000i64 {
                    KeReleaseSpinLock(&raw mut (*BusDeviceExtension).Lock as *mut usize, OldLevel);

                    DbgPrint(
                        b"DeviceID %S Instance %S TimeCreated %I64u Now %I64u Diff %I64u hung\n\0".as_ptr() as PCHAR,
                        (*DeviceEntry).DeviceName,
                        (*DeviceEntry).Instance,
                        (*DeviceEntry).TimeCreated.QuadPart * KeQueryTimeIncrement() as i64,
                        Time.QuadPart * KeQueryTimeIncrement() as i64,
                        Diff.QuadPart,
                    );

                    KeAcquireSpinLock(&raw mut (*BusDeviceExtension).Lock as *mut usize, &mut OldLevel);

                    (*DeviceEntry).DeviceState = StopPending;
                    DoInvalidate = 1u8;
                }
            } else if (*DeviceEntry).DeviceState == Started {
                KeReleaseSpinLock(&raw mut (*BusDeviceExtension).Lock as *mut usize, OldLevel);

                KspCompletePendingIrps(DeviceEntry, STATUS_REPARSE);

                KeAcquireSpinLock(&raw mut (*BusDeviceExtension).Lock as *mut usize, &mut OldLevel);
            }
        }

        Entry = (*Entry).Flink;
    }

    KeReleaseSpinLock(&raw mut (*BusDeviceExtension).Lock as *mut usize, OldLevel);

    if DoInvalidate != 0 {
        IoInvalidateDeviceRelations((*BusDeviceExtension).PhysicalDeviceObject, BusRelations);
    }

    let mut Time: LARGE_INTEGER = core::mem::zeroed();
    Time.QuadPart = 5000i64 * -10000i64;
    KeSetTimer(&raw mut (*BusDeviceExtension).Timer, Time, &raw mut (*BusDeviceExtension).Dpc);
}

#[no_mangle]
pub unsafe extern "system" fn KspBusDpcRoutine(
    Dpc: PKDPC,
    DeferredContext: *mut core::ffi::c_void,
    SystemArgument1: *mut core::ffi::c_void,
    SystemArgument2: *mut core::ffi::c_void,
) {
    let BusDeviceExtension = DeferredContext as *mut BUS_ENUM_DEVICE_EXTENSION;
    ExQueueWorkItem(&mut (*BusDeviceExtension).WorkItem, DelayedWorkQueue);
}

#[no_mangle]
pub unsafe extern "system" fn KspRemoveBusInterface(
    Ctx: PVOID,
) {
    let Context = Ctx as PBUS_INSTALL_ENUM_CONTEXT;

    (*Context).Status = STATUS_NOT_IMPLEMENTED;

    KeSetEvent(
        &mut (*Context).Event as *mut _ as PVOID,
        IO_NO_INCREMENT as KPRIORITY,
        FALSE as BOOLEAN,
    );
}
#[no_mangle]
pub unsafe extern "C" fn KspQueryBusRelations(
    BusDeviceExtension: PBUS_ENUM_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut DeviceRelations: PDEVICE_RELATIONS;
    let mut Entry: PLIST_ENTRY;
    let mut DeviceEntry: PBUS_DEVICE_ENTRY;
    let mut Count: ULONG = 0;
    let mut Length: ULONG;
    let mut OldLevel: KIRQL = 0;

    KeAcquireSpinLock(&mut (*BusDeviceExtension).Lock as *mut _ as *mut usize, &mut OldLevel);

    Entry = (*BusDeviceExtension).Common.Entry.Flink;
    while Entry != &mut (*BusDeviceExtension).Common.Entry {
        DeviceEntry = CONTAINING_RECORD!(Entry, BUS_DEVICE_ENTRY, Entry) as PBUS_DEVICE_ENTRY;

        if !(*DeviceEntry).PDO.is_null()
            && ((*DeviceEntry).DeviceState == NotStarted || (*DeviceEntry).DeviceState == Started)
        {
            Count += 1;
        }

        Entry = (*Entry).Flink;
    }

    Length = core::mem::size_of::<DEVICE_RELATIONS>() as ULONG
        + if Count > 1 {
            core::mem::size_of::<PDEVICE_OBJECT>() as ULONG * (Count - 1)
        } else {
            0
        };

    DeviceRelations = AllocateItem(NonPagedPool, Length as usize) as PDEVICE_RELATIONS;

    if DeviceRelations.is_null() {
        KeReleaseSpinLock(&mut (*BusDeviceExtension).Lock as *mut _ as *mut usize, OldLevel);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Entry = (*BusDeviceExtension).Common.Entry.Flink;
    while Entry != &mut (*BusDeviceExtension).Common.Entry {
        DeviceEntry = CONTAINING_RECORD!(Entry, BUS_DEVICE_ENTRY, Entry) as PBUS_DEVICE_ENTRY;

        if !(*DeviceEntry).PDO.is_null()
            && ((*DeviceEntry).DeviceState == NotStarted || (*DeviceEntry).DeviceState == Started)
        {
            *core::ptr::addr_of_mut!((*DeviceRelations).Objects).cast::<PDEVICE_OBJECT>().add((*DeviceRelations).Count as usize) = (*DeviceEntry).PDO;
            ObReferenceObject((*DeviceEntry).PDO as PVOID);
            (*DeviceRelations).Count += 1;
        }

        Entry = (*Entry).Flink;
    }

    KeReleaseSpinLock(&mut (*BusDeviceExtension).Lock as *mut _ as *mut usize, OldLevel);

    (*Irp).IoStatus.Information = DeviceRelations  as _;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KsGetBusEnumIdentifier(
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let DeviceExtension: PDEV_EXTENSION = (*((*IoStack).DeviceObject)).DeviceExtension as PDEV_EXTENSION;
    let BusDeviceExtension: PBUS_ENUM_DEVICE_EXTENSION = (*DeviceExtension).Ext as PBUS_ENUM_DEVICE_EXTENSION;

    if BusDeviceExtension.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let Length: SIZE_T = (wcslen((*BusDeviceExtension).BusIdentifier.as_mut_ptr()) + 1) * core::mem::size_of::<WCHAR>();

    if (*IoStack).Parameters.DeviceIoControl.InputBufferLength != 0 {
        if Length > (*IoStack).Parameters.DeviceIoControl.InputBufferLength as usize {
            return STATUS_BUFFER_TOO_SMALL;
        }

        let Buffer: LPWSTR = AllocateItem(NonPagedPool, Length) as LPWSTR;
        if Buffer.is_null() {
            let Status: NTSTATUS = STATUS_INSUFFICIENT_RESOURCES;
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            return Status;
        } else {
            wcscpy(Buffer, (*BusDeviceExtension).BusIdentifier.as_mut_ptr());
            (*Irp).AssociatedIrp.SystemBuffer = Buffer as PVOID;
            (*Irp).Flags |= (IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER | IRP_INPUT_OPERATION) as ULONG;
            let Status: NTSTATUS = STATUS_SUCCESS;
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            return Status;
        }
    } else {
        let Status: NTSTATUS = STATUS_BUFFER_OVERFLOW;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        return Status;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KsGetBusEnumParentFDOFromChildPDO(
    DeviceObject: PDEVICE_OBJECT,
    FunctionalDeviceObject: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    let DeviceExtension = (*DeviceObject).DeviceExtension as PDEV_EXTENSION;

    if (*(*DeviceExtension).Ext).IsBus == 0 {
        *FunctionalDeviceObject = (*(*(*DeviceExtension).Ext).BusDeviceExtension).BusDeviceObject;
        return STATUS_SUCCESS;
    }

    STATUS_INVALID_PARAMETER
}

#[no_mangle]
pub unsafe extern "system" fn KsCreateBusEnumObject(
    BusIdentifier: PWCHAR,
    BusDeviceObject: PDEVICE_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
    PnpDeviceObject: PDEVICE_OBJECT,
    InterfaceGuid: *const GUID,
    ServiceRelativePath: PWCHAR,
) -> NTSTATUS {
    let mut Length: usize;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let ServiceKeyPath: _UNICODE_STRING = {
        let s = b"\\REGISTRY\\MACHINE\\SYSTEM\\CurrentControlSet\\Services\\\0";
        let len = (s.len() - 1) as u16;
        let max = s.len() as u16;
        _UNICODE_STRING {
            Length: len,
            MaximumLength: max,
            Buffer: s.as_ptr() as PWCH,
        }
    };
    let mut BusDeviceExtension: PBUS_ENUM_DEVICE_EXTENSION;
    let mut DeviceExtension: PDEV_EXTENSION;
    let mut DeviceEntry: PBUS_DEVICE_ENTRY;
    let mut Entry: PLIST_ENTRY;
    let mut OldLevel: KIRQL = 0;

    Length = wcslen(BusIdentifier) * core::mem::size_of::<WCHAR>();
    Length += core::mem::size_of::<BUS_ENUM_DEVICE_EXTENSION>();

    BusDeviceExtension = AllocateItem(NonPagedPool, Length) as PBUS_ENUM_DEVICE_EXTENSION;
    if BusDeviceExtension.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    DeviceExtension = (*BusDeviceObject).DeviceExtension as PDEV_EXTENSION;
    (*DeviceExtension).Ext = BusDeviceExtension as PCOMMON_DEVICE_EXTENSION;

    core::ptr::write_bytes(BusDeviceExtension as *mut u8, 0, core::mem::size_of::<BUS_ENUM_DEVICE_EXTENSION>());

    wcscpy((*BusDeviceExtension).BusIdentifier.as_mut_ptr(), BusIdentifier);

    Length = ServiceKeyPath.MaximumLength as usize;
    Length += (*(*(*BusDeviceObject).DriverObject).DriverExtension).ServiceKeyName.MaximumLength as usize;

    if !ServiceRelativePath.is_null() {
        Length += (wcslen(ServiceRelativePath) + 2) * core::mem::size_of::<WCHAR>();
    }

    (*BusDeviceExtension).ServicePath.Length = 0;
    (*BusDeviceExtension).ServicePath.MaximumLength = Length as u16;
    (*BusDeviceExtension).ServicePath.Buffer = AllocateItem(NonPagedPool, Length) as PWCH;

    if (*BusDeviceExtension).ServicePath.Buffer.is_null() {
        FreeItem(BusDeviceExtension as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlAppendUnicodeStringToString(&mut (*BusDeviceExtension).ServicePath, &ServiceKeyPath as PCUNICODE_STRING);
    RtlAppendUnicodeStringToString(
        &mut (*BusDeviceExtension).ServicePath,
        &(*(*(*BusDeviceObject).DriverObject).DriverExtension).ServiceKeyName as PCUNICODE_STRING,
    );

    if !ServiceRelativePath.is_null() {
        RtlAppendUnicodeToString(&mut (*BusDeviceExtension).ServicePath, b"\\\0" as *const u8 as PCWSTR);
        RtlAppendUnicodeToString(&mut (*BusDeviceExtension).ServicePath, ServiceRelativePath as PCWSTR);
    }

    if !InterfaceGuid.is_null() {
        Status = IoRegisterDeviceInterface(
            PhysicalDeviceObject,
            InterfaceGuid,
            core::ptr::null_mut(),
            &mut (*BusDeviceExtension).DeviceInterfaceLink,
        );

        if !NT_SUCCESS(Status) {
            FreeItem((*BusDeviceExtension).ServicePath.Buffer as PVOID);
            FreeItem(BusDeviceExtension as PVOID);
            return Status;
        }

        Status = IoSetDeviceInterfaceState(&mut (*BusDeviceExtension).DeviceInterfaceLink, 1u8);

        if !NT_SUCCESS(Status) {
            FreeItem((*BusDeviceExtension).ServicePath.Buffer as PVOID);
            FreeItem(BusDeviceExtension as PVOID);
            return Status;
        }
    }

    (*BusDeviceExtension).Common.BusDeviceExtension = core::ptr::null_mut();
    (*BusDeviceExtension).Common.DeviceObjectReferenceCount = 1;
    *(*BusDeviceExtension).Common.__bindgen_anon_1.DeviceReferenceCount.as_mut() = 1;
    (*BusDeviceExtension).Common.IsBus = 1;
    InitializeListHead(&mut (*BusDeviceExtension).Common.Entry);

    (*BusDeviceExtension).BusDeviceObject = BusDeviceObject;
    (*BusDeviceExtension).PhysicalDeviceObject = PhysicalDeviceObject;

    KeInitializeSpinLock(&mut (*BusDeviceExtension).Lock);
    KeInitializeTimer(&mut (*BusDeviceExtension).Timer);
    KeInitializeDpc(
        &mut (*BusDeviceExtension).Dpc,
        cb!(KspBusDpcRoutine),
        BusDeviceExtension as PVOID,
    );
    KeInitializeEvent(&mut (*BusDeviceExtension).Event, _EVENT_TYPE_SynchronizationEvent, 0u8);
    ExInitializeWorkItem(
        &mut (*BusDeviceExtension).WorkItem,
        cb!(KspBusWorkerRoutine),
        BusDeviceExtension as PVOID,
    );

    if PnpDeviceObject.is_null() {
        (*BusDeviceExtension).PnpDeviceObject = IoAttachDeviceToDeviceStack(BusDeviceObject, PhysicalDeviceObject);

        if (*BusDeviceExtension).PnpDeviceObject.is_null() {
            if !(*BusDeviceExtension).DeviceInterfaceLink.Buffer.is_null() {
                IoSetDeviceInterfaceState(&mut (*BusDeviceExtension).DeviceInterfaceLink, 0u8);
                RtlFreeUnicodeString(&mut (*BusDeviceExtension).DeviceInterfaceLink);
            }
            FreeItem((*BusDeviceExtension).ServicePath.Buffer as PVOID);
            FreeItem(BusDeviceExtension as PVOID);
            return STATUS_DEVICE_REMOVED;
        }

        (*BusDeviceExtension).DeviceAttached = 1;
    } else {
        (*BusDeviceExtension).PnpDeviceObject = PnpDeviceObject;
    }

    Status = KspScanBus(BusDeviceExtension);

    if !NT_SUCCESS(Status) {
        if !(*BusDeviceExtension).DeviceInterfaceLink.Buffer.is_null() {
            IoSetDeviceInterfaceState(&mut (*BusDeviceExtension).DeviceInterfaceLink, 0u8);
            RtlFreeUnicodeString(&mut (*BusDeviceExtension).DeviceInterfaceLink);
        }

        if (*BusDeviceExtension).DeviceAttached != 0 {
            IoDetachDevice((*BusDeviceExtension).PnpDeviceObject);
        }

        FreeItem((*BusDeviceExtension).ServicePath.Buffer as PVOID);
        FreeItem(BusDeviceExtension as PVOID);
        return Status;
    }

    KeAcquireSpinLock(&mut (*BusDeviceExtension).Lock as *mut _ as *mut usize, &mut OldLevel);

    Entry = (*BusDeviceExtension).Common.Entry.Flink;
    while Entry != &mut (*BusDeviceExtension).Common.Entry {
        DeviceEntry = CONTAINING_RECORD!(Entry, BUS_DEVICE_ENTRY, Entry) as PBUS_DEVICE_ENTRY;
        if (*DeviceEntry).PDO.is_null() {
            KeReleaseSpinLock(&mut (*BusDeviceExtension).Lock as *mut _ as *mut usize, OldLevel);
            Status = KspCreatePDO(BusDeviceExtension, DeviceEntry, &mut (*DeviceEntry).PDO);
            KeAcquireSpinLock(&mut (*BusDeviceExtension).Lock as *mut _ as *mut usize, &mut OldLevel);
            break;
        }
        Entry = (*Entry).Flink;
    }

    KeReleaseSpinLock(&mut (*BusDeviceExtension).Lock as *mut _ as *mut usize, OldLevel);

    IoInvalidateDeviceRelations((*BusDeviceExtension).PhysicalDeviceObject, BusRelations);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsGetBusEnumPnpDeviceObject(
    DeviceObject: PDEVICE_OBJECT,
    PnpDeviceObject: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    if (*DeviceObject).DeviceExtension.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let DeviceExtension = (*DeviceObject).DeviceExtension as PDEV_EXTENSION;

    let CommonDeviceExtension = (*DeviceExtension).Ext;

    if CommonDeviceExtension.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*CommonDeviceExtension).IsBus == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    let BusDeviceExtension = CommonDeviceExtension as PBUS_ENUM_DEVICE_EXTENSION;

    *PnpDeviceObject = (*BusDeviceExtension).PnpDeviceObject;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KsInstallBusEnumInterface(
    Irp: PIRP,
) -> NTSTATUS {
    let mut Context: BUS_INSTALL_ENUM_CONTEXT = core::mem::zeroed();
    let Mode: KPROCESSOR_MODE;
    let luid: LUID;
    let IoStack: PIO_STACK_LOCATION;
    let DeviceExtension: PDEV_EXTENSION;
    let BusDeviceExtension: PBUS_ENUM_DEVICE_EXTENSION;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    Mode = ExGetPreviousMode();
    luid = RtlConvertUlongToLuid(SE_LOAD_DRIVER_PRIVILEGE);
    SeSinglePrivilegeCheck(luid, Mode);
    DeviceExtension = (*((*IoStack).DeviceObject)).DeviceExtension as PDEV_EXTENSION;
    BusDeviceExtension = (*DeviceExtension).Ext as PBUS_ENUM_DEVICE_EXTENSION;

    Context.Irp = Irp;
    KeInitializeEvent(&mut Context.Event, _EVENT_TYPE_NotificationEvent, 0u8);
    Context.BusDeviceExtension = BusDeviceExtension;
    ExInitializeWorkItem(&mut Context.WorkItem, cb!(KspInstallBusEnumInterface), &mut Context as *mut _ as PVOID);
    ExQueueWorkItem(&mut Context.WorkItem, DelayedWorkQueue);
    KeWaitForSingleObject(
        &mut Context.Event as *mut _ as PVOID,
        _KWAIT_REASON_Executive,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );
    (*Irp).IoStatus.__bindgen_anon_1.Status = Context.Status;
    Context.Status
}

#[no_mangle]
pub unsafe extern "system" fn KsIsBusEnumChildDevice(
    DeviceObject: PDEVICE_OBJECT,
    ChildDevice: PBOOLEAN,
) -> NTSTATUS {
    let DeviceExtension = (*DeviceObject).DeviceExtension as PDEV_EXTENSION;
    let BusDeviceExtension = (*DeviceExtension).Ext as PBUS_ENUM_DEVICE_EXTENSION;

    if BusDeviceExtension.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *ChildDevice = ((*BusDeviceExtension).Common.IsBus == 0u32) as u8;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KsServiceBusEnumCreateRequest(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Entry: PLIST_ENTRY;
    let mut DeviceEntry: PBUS_DEVICE_ENTRY = core::ptr::null_mut();
    let IoStack: PIO_STACK_LOCATION;
    let mut ItemExists: BOOLEAN = 0u8;
    let DeviceExtension: PDEV_EXTENSION;
    let BusDeviceExtension: PBUS_ENUM_DEVICE_EXTENSION;
    let mut Status: NTSTATUS;
    let mut Time: LARGE_INTEGER = core::mem::zeroed();

    DeviceExtension = (*DeviceObject).DeviceExtension as PDEV_EXTENSION;
    BusDeviceExtension = (*DeviceExtension).Ext as PBUS_ENUM_DEVICE_EXTENSION;
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    if (*IoStack).FileObject.is_null() {
        return STATUS_SUCCESS;
    }

    if (*(*IoStack).FileObject).FileName.Buffer.is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        return STATUS_SUCCESS;
    }

    Entry = (*BusDeviceExtension).Common.Entry.Flink;

    while Entry != &mut (*BusDeviceExtension).Common.Entry {
        DeviceEntry = CONTAINING_RECORD!(Entry, BUS_DEVICE_ENTRY, Entry) as PBUS_DEVICE_ENTRY;

        if _wcsicmp(
            (*DeviceEntry).DeviceName,
            (*(*IoStack).FileObject).FileName.Buffer.offset(1),
        ) == 0
        {
            ItemExists = 1u8;
            break;
        }

        Entry = (*Entry).Flink;
    }

    if ItemExists == 0u8 {
        return STATUS_OBJECT_NAME_NOT_FOUND;
    }

    if !(*DeviceEntry).PDO.is_null() {
        if (*DeviceEntry).DeviceState == Started {
            Status = KspDoReparseForIrp(Irp, DeviceEntry);
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            (*Irp).IoStatus.Information = IO_REPARSE  as _;
            return Status;
        }

        IoMarkIrpPending(Irp);
        InsertTailList(
            &mut (*DeviceEntry).IrpPendingList,
            &mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry,
        );

        Time.QuadPart = (1500i64) * (-10000i64);
        KeSetTimer(
            &mut (*BusDeviceExtension).Timer,
            Time,
            &mut (*BusDeviceExtension).Dpc,
        );

        return STATUS_PENDING;
    } else {
        Status = KspCreatePDO(BusDeviceExtension, DeviceEntry, &mut (*DeviceEntry).PDO);

        if !NT_SUCCESS(Status) {
            return Status;
        }

        IoMarkIrpPending(Irp);
        InsertTailList(
            &mut (*DeviceEntry).IrpPendingList,
            &mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry,
        );
        IoInvalidateDeviceRelations((*BusDeviceExtension).PhysicalDeviceObject, BusRelations);

        return STATUS_PENDING;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KsServiceBusEnumPnpRequest(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceExtension: PDEV_EXTENSION;
    let mut BusDeviceExtension: PBUS_ENUM_DEVICE_EXTENSION;
    let ChildDeviceExtension: PCOMMON_DEVICE_EXTENSION;
    let IoStack: PIO_STACK_LOCATION;
    let Status: NTSTATUS;
    let mut Time: LARGE_INTEGER = core::mem::zeroed();
    let DeviceRelation: PDEVICE_RELATIONS;
    let DeviceEntry: PBUS_DEVICE_ENTRY;

    DeviceExtension = (*DeviceObject).DeviceExtension as PDEV_EXTENSION;
    BusDeviceExtension = (*DeviceExtension).Ext as PBUS_ENUM_DEVICE_EXTENSION;
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    if (*BusDeviceExtension).Common.IsBus != 0 {
        if (*IoStack).MinorFunction as i32 == IRP_MN_START_DEVICE as i32 {
            Status = STATUS_SUCCESS;
        } else if (*IoStack).MinorFunction as i32 == IRP_MN_QUERY_DEVICE_RELATIONS as i32 {
            Status = KspQueryBusRelations(BusDeviceExtension, Irp);
        } else {
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }
    } else {
        ChildDeviceExtension = (*DeviceExtension).Ext;
        BusDeviceExtension = (*ChildDeviceExtension).BusDeviceExtension as PBUS_ENUM_DEVICE_EXTENSION;

        if (*IoStack).MinorFunction as i32 == IRP_MN_QUERY_ID as i32 {
            Status = KspQueryId(ChildDeviceExtension, Irp);
        } else if (*IoStack).MinorFunction as i32 == IRP_MN_REMOVE_DEVICE as i32 {
            DeviceEntry = (*(*ChildDeviceExtension).__bindgen_anon_2.DeviceEntry.as_ref()) as PBUS_DEVICE_ENTRY;
            FreeItem(ChildDeviceExtension as PVOID);
            (*DeviceEntry).PDO = core::ptr::null_mut();
            IoDeleteDevice(DeviceObject);
            if !(*DeviceEntry).PDODeviceName.is_null() {
                FreeItem((*DeviceEntry).PDODeviceName as PVOID);
                (*DeviceEntry).PDODeviceName = core::ptr::null_mut();
            }
            (*DeviceEntry).DeviceState = _DEVICE_STATE_NotStarted;
            KspCompletePendingIrps(DeviceEntry, STATUS_DEVICE_REMOVED);
            Status = STATUS_SUCCESS;
        } else if (*IoStack).MinorFunction as i32 == IRP_MN_QUERY_BUS_INFORMATION as i32 {
            Status = KspQueryBusInformation(ChildDeviceExtension, Irp);
        } else if (*IoStack).MinorFunction as i32 == IRP_MN_QUERY_RESOURCES as i32 {
            Status = STATUS_SUCCESS;
        } else if (*IoStack).MinorFunction as i32 == IRP_MN_QUERY_RESOURCE_REQUIREMENTS as i32 {
            Status = STATUS_SUCCESS;
        } else if (*IoStack).MinorFunction as i32 == IRP_MN_START_DEVICE as i32 {
            Status = KspStartBusDevice(DeviceObject, ChildDeviceExtension, Irp);
            if NT_SUCCESS(Status) {
                KspCompletePendingIrps(
                    (*(*ChildDeviceExtension).__bindgen_anon_2.DeviceEntry.as_ref()) as PBUS_DEVICE_ENTRY,
                    STATUS_REPARSE,
                );
            }
            Time.QuadPart = (1500i64) * (-10000i64);
            KeSetTimer(
                &mut (*BusDeviceExtension).Timer,
                Time,
                &mut (*BusDeviceExtension).Dpc,
            );
        } else if (*IoStack).MinorFunction as i32 == IRP_MN_QUERY_CAPABILITIES as i32 {
            Status = KspQueryBusDeviceCapabilities(ChildDeviceExtension, Irp);
        } else if (*IoStack).MinorFunction as i32 == IRP_MN_QUERY_PNP_DEVICE_STATE as i32 {
            Status = KspQueryBusDevicePnpState(ChildDeviceExtension, Irp);
        } else if (*IoStack).MinorFunction as i32 == IRP_MN_QUERY_INTERFACE as i32 {
            Status = KspQueryBusDeviceInterface(ChildDeviceExtension, Irp);
        } else if (*IoStack).MinorFunction as i32 == IRP_MN_QUERY_DEVICE_RELATIONS as i32
            && (*IoStack).Parameters.QueryDeviceRelations.Type as u32 == TargetDeviceRelation as u32
        {
            DeviceRelation = AllocateItem(
                PagedPool,
                core::mem::size_of::<DEVICE_RELATIONS>(),
            ) as PDEVICE_RELATIONS;
            if !DeviceRelation.is_null() {
                (*DeviceRelation).Count = 1;
                (*DeviceRelation).Objects[0] = DeviceObject;
                ObReferenceObject(DeviceObject as PVOID);
                (*Irp).IoStatus.Information = DeviceRelation  as _;
                Status = STATUS_SUCCESS;
            } else {
                Status = STATUS_INSUFFICIENT_RESOURCES;
            }
        } else {
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsRemoveBusEnumInterface(
    Irp: PIRP,
) -> NTSTATUS {
    let Mode: KPROCESSOR_MODE;
    let luid: LUID;
    let mut Ctx: BUS_INSTALL_ENUM_CONTEXT = core::mem::zeroed();
    let DeviceExtension: PDEV_EXTENSION;
    let BusDeviceExtension: PBUS_ENUM_DEVICE_EXTENSION;
    let IoStack: PIO_STACK_LOCATION;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    DeviceExtension = (*((*IoStack).DeviceObject)).DeviceExtension as PDEV_EXTENSION;
    BusDeviceExtension = (*(*DeviceExtension).Ext).BusDeviceExtension as PBUS_ENUM_DEVICE_EXTENSION;
    Mode = ExGetPreviousMode();
    luid = RtlConvertUlongToLuid(SE_LOAD_DRIVER_PRIVILEGE as u32);
    if SeSinglePrivilegeCheck(luid, Mode) == 0 {
        return STATUS_PRIVILEGE_NOT_HELD;
    }
    KeInitializeEvent(&mut Ctx.Event, _EVENT_TYPE_NotificationEvent, 0u8);
    Ctx.Irp = Irp;
    Ctx.BusDeviceExtension = BusDeviceExtension;
    ExInitializeWorkItem(&mut Ctx.WorkItem, cb!(KspRemoveBusInterface), &mut Ctx as *mut _ as PVOID);
    ExQueueWorkItem(&mut Ctx.WorkItem, DelayedWorkQueue);
    KeWaitForSingleObject(
        &mut Ctx.Event as *mut _ as PVOID,
        _KWAIT_REASON_Executive,
        0i32 as i32 as KPROCESSOR_MODE,
        0u8,
        core::ptr::null_mut(),
    );
    Ctx.Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel Streaming
//  * FILE:            drivers/ksfilter/ks/swenum.c
//  * PURPOSE:         KS Software BUS functions
//  * PROGRAMMER:      Johannes Anderwald
//  */
//
// #include "precomp.h"
//
// #include <stdio.h>
// #include <swenum.h>
//
// #define NDEBUG
// #include <debug.h>
//
// LONG KsDeviceCount = 0;
//
// typedef NTSTATUS (NTAPI *PKSP_BUS_ENUM_CALLBACK)(
//     IN PHANDLE hKey,
//     IN PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension,
//     IN PBUS_DEVICE_ENTRY DummyEntry,
//     IN LPWSTR RootName,
//     IN LPWSTR DirectoryName);
//
// NTSTATUS
// KspCreatePDO(
//     IN PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension,
//     IN PBUS_DEVICE_ENTRY DeviceEntry,
//     OUT PDEVICE_OBJECT * OutDeviceObject)
// {
//     PDEVICE_OBJECT DeviceObject;
//     WCHAR Buffer[50];
//     ULONG CurDeviceId;
//     UNICODE_STRING DeviceName;
//     NTSTATUS Status;
//     PCOMMON_DEVICE_EXTENSION DeviceExtension;
//
//     /* increment device count */
//     CurDeviceId = InterlockedIncrement(&KsDeviceCount);
//
//     /* generate new device id */
//     swprintf(Buffer, L!("\\Device\\KSENUM%08x"), CurDeviceId);
//
//     /* initialize new device name */
//     RtlInitUnicodeString(&DeviceName, Buffer);
//
//     /* create new device object */
//     Status = IoCreateDevice(BusDeviceExtension->BusDeviceObject->DriverObject, sizeof(PVOID), &DeviceName, FILE_DEVICE_UNKNOWN, 0, FALSE, &DeviceObject);
//
//     /* check for success */
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed to create pdo */
//         return Status;
//     }
//
//     /* now allocate device extension */
//     DeviceExtension = (PCOMMON_DEVICE_EXTENSION)AllocateItem(NonPagedPool, sizeof(COMMON_DEVICE_EXTENSION));
//     if (!DeviceExtension)
//     {
//         /* no memory */
//         IoDeleteDevice(DeviceObject);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* store device extension */
//     *((PVOID*)DeviceObject->DeviceExtension) = DeviceExtension;
//
//     /* initialize device extension */
//     DeviceExtension->IsBus = FALSE;
//     DeviceExtension->DeviceObject = DeviceObject;
//     DeviceExtension->DeviceEntry = DeviceEntry;
//     DeviceExtension->BusDeviceExtension = BusDeviceExtension;
//
//     /* not started yet*/
//     DeviceEntry->DeviceState = NotStarted;
//
//     /* get current time */
//     KeQueryTickCount(&DeviceEntry->TimeCreated);
//
//     /* setup flags */
//     DeviceObject->Flags |= DO_POWER_PAGABLE;
//     DeviceObject->Flags &= ~ DO_DEVICE_INITIALIZING;
//     /* TODO: fire time when expired */
//
//     *OutDeviceObject = DeviceObject;
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// KspRegisterDeviceAssociation(
//     IN PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension,
//     IN PBUS_DEVICE_ENTRY DeviceEntry,
//     IN OUT PBUS_INSTANCE_ENTRY BusInstanceEntry)
// {
//     NTSTATUS Status;
//     UNICODE_STRING ReferenceString;
//
//     /* initialize reference string */
//     RtlInitUnicodeString(&ReferenceString, DeviceEntry->DeviceName);
//
//     /* register device interface */
//     Status = IoRegisterDeviceInterface(BusDeviceExtension->PhysicalDeviceObject, &BusInstanceEntry->InterfaceGuid, &ReferenceString, &BusInstanceEntry->SymbolicLink);
//
//     /* check for success */
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed */
//         return Status;
//     }
//
//     /* now enable the interface */
//     Status = IoSetDeviceInterfaceState(&BusInstanceEntry->SymbolicLink, TRUE);
//
//     /* check for success */
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed, free memory */
//         FreeItem(BusInstanceEntry->SymbolicLink.Buffer);
//         return Status;
//     }
//
//     DPRINT("Registered DeviceInterface %wZ\n", &BusInstanceEntry->SymbolicLink);
//
//
//     /* done */
//     return Status;
// }
//
// VOID
// KspRemoveDeviceAssociations(
//     IN PBUS_DEVICE_ENTRY DeviceEntry)
// {
//     PLIST_ENTRY Entry;
//     PBUS_INSTANCE_ENTRY CurEntry;
//
//     /* remove all entries */
//     Entry = DeviceEntry->DeviceInterfaceList.Flink;
//
//     while(Entry != &DeviceEntry->DeviceInterfaceList)
//     {
//          /* get offset */
//          CurEntry = (PBUS_INSTANCE_ENTRY)CONTAINING_RECORD!(Entry, BUS_INSTANCE_ENTRY, Entry);
//
//          /* sanity check */
//          ASSERT(CurEntry->SymbolicLink.Buffer);
//
//          /* de-register interface */
//          IoSetDeviceInterfaceState(&CurEntry->SymbolicLink, FALSE);
//
//          /* free symbolic link buffer */
//          FreeItem(CurEntry->SymbolicLink.Buffer);
//
//          /* remove entry from list */
//          RemoveEntryList(Entry);
//
//          /* move to next entry */
//          Entry = Entry->Flink;
//
//          /* free entry */
//          FreeItem(CurEntry);
//     }
// }
//
// NTSTATUS
// KspEnumerateBusRegistryKeys(
//     IN HANDLE hKey,
//     IN LPWSTR ReferenceString,
//     IN PKSP_BUS_ENUM_CALLBACK Callback,
//     IN PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension,
//     IN PBUS_DEVICE_ENTRY DeviceEntry)
// {
//     UNICODE_STRING String;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE hNewKey;
//     NTSTATUS Status;
//     ULONG ResultLength, Index, KeyInfoLength;
//     KEY_FULL_INFORMATION KeyInformation;
//     PKEY_BASIC_INFORMATION KeyInfo;
//
//     /* initialize key name */
//     RtlInitUnicodeString(&String, ReferenceString);
//
//     /* initialize object attributes */
//     InitializeObjectAttributes(&ObjectAttributes, &String, OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE, hKey, NULL);
//
//     /* open new key */
//     Status = ZwOpenKey(&hNewKey, GENERIC_READ, &ObjectAttributes);
//
//     /* check for success */
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed to open key */
//
//         return Status;
//     }
//
//     /* query key stats */
//     Status = ZwQueryKey(hNewKey, KeyFullInformation, &KeyInformation, sizeof(KeyInformation), &ResultLength);
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* close key */
//         ZwClose(hNewKey);
//
//         /* done */
//         return Status;
//     }
//
//     /* calculate key info length */
//     KeyInfoLength = KeyInformation.MaxNameLen + sizeof(KEY_BASIC_INFORMATION) + 1 * sizeof(WCHAR);
//
//     /* allocate buffer */
//     KeyInfo = (PKEY_BASIC_INFORMATION)AllocateItem(NonPagedPool, KeyInfoLength);
//     if (!KeyInfo)
//     {
//
//         /* no memory */
//         ZwClose(hNewKey);
//
//         /* done */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* enumerate all keys */
//     for(Index = 0; Index < KeyInformation.SubKeys; Index++)
//     {
//
//         /* query sub key */
//         Status = ZwEnumerateKey(hNewKey, Index, KeyBasicInformation, (PVOID)KeyInfo, KeyInfoLength, &ResultLength);
//
//         /* check for success */
//         if (NT_SUCCESS(Status))
//         {
//             /* perform callback */
//             Status = (Callback.unwrap())(hNewKey, BusDeviceExtension, DeviceEntry, KeyInfo->Name, ReferenceString);
//
//             /* should enumeration stop */
//             if (!NT_SUCCESS(Status))
//                 break;
//         }
//     }
//
//     /* free info buffer */
//     FreeItem(KeyInfo);
//
//     /* close key */
//     ZwClose(hNewKey);
//
//     /* done */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// KspCreateDeviceAssociation(
//     IN PHANDLE hKey,
//     IN PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension,
//     IN PBUS_DEVICE_ENTRY DeviceEntry,
//     IN LPWSTR InterfaceString,
//     IN LPWSTR ReferenceString)
// {
//     GUID InterfaceGUID;
//     NTSTATUS Status;
//     PLIST_ENTRY Entry;
//     PBUS_INSTANCE_ENTRY CurEntry;
//     UNICODE_STRING DeviceName;
//
//     /* initialize interface string */
//     RtlInitUnicodeString(&DeviceName, InterfaceString);
//
//     /* first convert device name to guid */
//     RtlGUIDFromString(&DeviceName, &InterfaceGUID);
//
//     /* check if the device is already present */
//     Entry = DeviceEntry->DeviceInterfaceList.Flink;
//     DPRINT("KspCreateDeviceAssociation ReferenceString %S\n", ReferenceString);
//     DPRINT("KspCreateDeviceAssociation InterfaceString %S\n", InterfaceString);
//
//     while(Entry != &DeviceEntry->DeviceInterfaceList)
//     {
//          /* get offset */
//          CurEntry = (PBUS_INSTANCE_ENTRY)CONTAINING_RECORD!(Entry, BUS_INSTANCE_ENTRY, Entry);
//
//          if (IsEqualGUIDAligned(&CurEntry->InterfaceGuid, &InterfaceGUID))
//          {
//              /* entry already exists */
//              return STATUS_SUCCESS;
//          }
//
//          /* move to next entry */
//          Entry = Entry->Flink;
//     }
//
//     /* time to allocate new entry */
//     CurEntry = (PBUS_INSTANCE_ENTRY)AllocateItem(NonPagedPool, sizeof(BUS_INSTANCE_ENTRY));
//
//     if (!CurEntry)
//     {
//         /* no memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* store guid */
//     RtlMoveMemory(&CurEntry->InterfaceGuid, &InterfaceGUID, sizeof(GUID));
//
//     /* now register the association */
//     Status = KspRegisterDeviceAssociation(BusDeviceExtension, DeviceEntry, CurEntry);
//
//     /* check for success */
//     if (NT_SUCCESS(Status))
//     {
//         /* store entry */
//         InsertTailList(&DeviceEntry->DeviceInterfaceList, &CurEntry->Entry);
//     }
//     else
//     {
//         /* failed to associated device */
//         FreeItem(CurEntry);
//     }
//
//      /* done */
//      return Status;
// }
//
// NTSTATUS
// NTAPI
// KspCreateDeviceReference(
//     IN PHANDLE hKey,
//     IN PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension,
//     IN PBUS_DEVICE_ENTRY DummyEntry,
//     IN LPWSTR InterfaceId,
//     IN LPWSTR DeviceId)
// {
//     LPWSTR DeviceName;
//     SIZE_T Length;
//     PLIST_ENTRY Entry;
//     PBUS_DEVICE_ENTRY DeviceEntry = NULL; /* GCC warning */
//     BOOLEAN ItemExists = FALSE;
//     UNICODE_STRING String;
//     NTSTATUS Status;
//     KIRQL OldLevel;
//
//     /* first construct device name & reference guid */
//     Length = wcslen(DeviceId) + wcslen(InterfaceId);
//
//     /* append '&' and null byte */
//     Length += 2;
//
//     /* allocate device name */
//     DeviceName = AllocateItem(NonPagedPool, Length * sizeof(WCHAR));
//
//     if (!DeviceName)
//     {
//         /* not enough memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* construct device name */
//     wcscpy(DeviceName, DeviceId);
//     wcscat(DeviceName, L!("&"));
//     wcscat(DeviceName, InterfaceId);
//
//     /* scan list and check if it is already present */
//     Entry = BusDeviceExtension->Common.Entry.Flink;
//
//     while(Entry != &BusDeviceExtension->Common.Entry)
//     {
//         /* get real offset */
//         DeviceEntry = (PBUS_DEVICE_ENTRY)CONTAINING_RECORD!(Entry, BUS_DEVICE_ENTRY, Entry);
//
//         /* check if name matches */
//         if (!_wcsicmp(DeviceEntry->DeviceName, DeviceName))
//         {
//             /* item already exists */
//             ItemExists = TRUE;
//             break;
//         }
//
//         /* move to next entry */
//         Entry = Entry->Flink;
//     }
//
//     if (!ItemExists)
//     {
//         /* allocate new device entry */
//         DeviceEntry = AllocateItem(NonPagedPool, sizeof(BUS_DEVICE_ENTRY));
//         if (!DeviceEntry)
//         {
//             /* no memory */
//             FreeItem(DeviceName);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* initialize device entry */
//         InitializeListHead(&DeviceEntry->DeviceInterfaceList);
//         InitializeListHead(&DeviceEntry->IrpPendingList);
//
//         /* copy device guid */
//         RtlInitUnicodeString(&String, DeviceId);
//         RtlGUIDFromString(&String, &DeviceEntry->DeviceGuid);
//
//         /* copy device names */
//         DeviceEntry->DeviceName = DeviceName;
//         DeviceEntry->Instance = (DeviceName + wcslen(DeviceId) + 1);
//
//         /* copy name */
//         DeviceEntry->BusId = AllocateItem(NonPagedPool, (wcslen(DeviceId) + 1) * sizeof(WCHAR));
//         if (!DeviceEntry->BusId)
//         {
//             /* no memory */
//             FreeItem(DeviceName);
//             FreeItem(DeviceEntry);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//         wcscpy(DeviceEntry->BusId, DeviceId);
//     }
//
//     /* now enumerate the interfaces */
//     Status = KspEnumerateBusRegistryKeys(hKey, InterfaceId, KspCreateDeviceAssociation, BusDeviceExtension, DeviceEntry);
//
//     /* check if list is empty */
//     if (IsListEmpty(&DeviceEntry->DeviceInterfaceList))
//     {
//         /* invalid device settings */
//         FreeItem(DeviceEntry->BusId);
//         FreeItem(DeviceEntry->DeviceName);
//         FreeItem(DeviceEntry);
//
//         ASSERT(ItemExists == FALSE);
//
//         return STATUS_INVALID_DEVICE_STATE;
//     }
//
//     /* check if enumeration failed */
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed */
//         KspRemoveDeviceAssociations(DeviceEntry);
//         FreeItem(DeviceEntry->BusId);
//         FreeItem(DeviceEntry->DeviceName);
//         FreeItem(DeviceEntry);
//
//         ASSERT(ItemExists == FALSE);
//
//         /* done */
//         return Status;
//     }
//
//     if (!ItemExists)
//     {
//         /* acquire lock */
//         KeAcquireSpinLock(&BusDeviceExtension->Lock, &OldLevel);
//
//         /* successfully initialized entry */
//         InsertTailList(&BusDeviceExtension->Common.Entry, &DeviceEntry->Entry);
//
//         /* release lock */
//         KeReleaseSpinLock(&BusDeviceExtension->Lock, OldLevel);
//     }
//
//     /* done */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// KspCreateDeviceReferenceTrampoline(
//     IN PHANDLE hKey,
//     IN PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension,
//     IN PBUS_DEVICE_ENTRY DummyEntry,
//     IN LPWSTR DeviceCategory,
//     IN LPWSTR ReferenceString)
// {
//     return KspEnumerateBusRegistryKeys(hKey, DeviceCategory, KspCreateDeviceReference, BusDeviceExtension, DummyEntry);
// }
//
//
// NTSTATUS
// KspOpenBusRegistryKey(
//     IN PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension,
//     OUT PHANDLE hKey)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//
//     /* initialize object attributes */
//     InitializeObjectAttributes(&ObjectAttributes, &BusDeviceExtension->ServicePath, OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE, NULL, NULL);
//
//     return ZwCreateKey(hKey, GENERIC_READ | GENERIC_WRITE, &ObjectAttributes, 0, NULL, 0, NULL);
// }
//
// NTSTATUS
// KspScanBus(
//     IN PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension)
// {
//     HANDLE hKey;
//     NTSTATUS Status;
//
//     /* first open key */
//     Status = KspOpenBusRegistryKey(BusDeviceExtension, &hKey);
//
//     /* check for success */
//     if (!NT_SUCCESS(Status))
//     {
//         /* no success */
//
//         return Status;
//     }
//
//     /* TODO clear reference marks */
//
//     /* construct device entries */
//     Status = KspEnumerateBusRegistryKeys(hKey, NULL, KspCreateDeviceReferenceTrampoline, BusDeviceExtension, NULL);
//
//     /* TODO: delete unreferenced devices */
//
//     /* close handle */
//     ZwClose(hKey);
//
//     /* done */
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// KspBusQueryReferenceString(
//     IN PVOID Context,
//     IN OUT PWCHAR *String)
// {
//     LPWSTR Name;
//     SIZE_T Length;
//     PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension = (PBUS_ENUM_DEVICE_EXTENSION)Context;
//
//     /* sanity checks */
//     ASSERT(BusDeviceExtension);
//     ASSERT(BusDeviceExtension->BusIdentifier);
//
//     /* calculate length */
//     Length = wcslen(BusDeviceExtension->BusIdentifier) + 1;
//
//     /* allocate buffer */
//     Name = AllocateItem(PagedPool, Length * sizeof(WCHAR));
//
//     if (!Name)
//     {
//         /* failed to allocate buffer */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* copy buffer */
//     wcscpy(Name, BusDeviceExtension->BusIdentifier);
//
//     /* store result */
//     *String = Name;
//
//     /* done */
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// KspBusDeviceReference(
//     IN PVOID Context)
// {
//     PCOMMON_DEVICE_EXTENSION ChildDeviceExtension = (PCOMMON_DEVICE_EXTENSION)Context;
//
//     /* reference count */
//     InterlockedIncrement((PLONG)&ChildDeviceExtension->DeviceReferenceCount);
// }
//
// VOID
// NTAPI
// KspBusDeviceDereference(
//     IN PVOID Context)
// {
//     PCOMMON_DEVICE_EXTENSION ChildDeviceExtension = (PCOMMON_DEVICE_EXTENSION)Context;
//
//     /* reference count */
//     InterlockedDecrement((PLONG)&ChildDeviceExtension->DeviceReferenceCount);
// }
//
// VOID
// NTAPI
// KspBusReferenceDeviceObject(
//     IN PVOID Context)
// {
//     PCOMMON_DEVICE_EXTENSION ChildDeviceExtension = (PCOMMON_DEVICE_EXTENSION)Context;
//
//     /* reference count */
//     InterlockedIncrement((PLONG)&ChildDeviceExtension->DeviceObjectReferenceCount);
// }
//
// VOID
// NTAPI
// KspBusDereferenceDeviceObject(
//     IN PVOID Context)
// {
//     PCOMMON_DEVICE_EXTENSION ChildDeviceExtension = (PCOMMON_DEVICE_EXTENSION)Context;
//
//     /* reference count */
//     InterlockedDecrement((PLONG)&ChildDeviceExtension->DeviceObjectReferenceCount);
// }
//
// NTSTATUS
// KspQueryBusDeviceInterface(
//     IN PCOMMON_DEVICE_EXTENSION ChildDeviceExtension,
//     IN PIRP Irp)
// {
//     PBUS_INTERFACE_SWENUM Interface;
//     PIO_STACK_LOCATION IoStack;
//
//     /* get current irp stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* sanity checks */
//     ASSERT(IoStack->Parameters.QueryInterface.Size == sizeof(BUS_INTERFACE_SWENUM));
//     ASSERT(IoStack->Parameters.QueryInterface.Interface);
//
//     /* fill in interface */
//     Interface = (PBUS_INTERFACE_SWENUM)IoStack->Parameters.QueryInterface.Interface;
//     Interface->Interface.Size = sizeof(BUS_INTERFACE_SWENUM);
//     Interface->Interface.Version = BUS_INTERFACE_SWENUM_VERSION;
//     Interface->Interface.Context = ChildDeviceExtension;
//     Interface->Interface.InterfaceReference = KspBusDeviceReference;
//     Interface->Interface.InterfaceDereference = KspBusDeviceDereference;
//     Interface->ReferenceDeviceObject = KspBusReferenceDeviceObject;
//     Interface->DereferenceDeviceObject = KspBusDereferenceDeviceObject;
//     Interface->QueryReferenceString = KspBusQueryReferenceString;
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// KspEnableBusDeviceInterface(
//     PBUS_DEVICE_ENTRY DeviceEntry,
//     BOOLEAN bEnable)
// {
//     PLIST_ENTRY Entry;
//     PBUS_INSTANCE_ENTRY InstanceEntry;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* enable now all interfaces */
//     Entry = DeviceEntry->DeviceInterfaceList.Flink;
//
//     while(Entry != &DeviceEntry->DeviceInterfaceList)
//     {
//         /* get bus instance entry */
//         InstanceEntry = (PBUS_INSTANCE_ENTRY)CONTAINING_RECORD!(Entry, BUS_INSTANCE_ENTRY, Entry);
//         DPRINT("Enabling %u %wZ Irql %u\n", bEnable, &InstanceEntry->SymbolicLink, KeGetCurrentIrql());
//
//         /* set interface state */
//         Status = IoSetDeviceInterfaceState(&InstanceEntry->SymbolicLink, bEnable);
//
//         if (!NT_SUCCESS(Status))
//         {
//             /* failed to set interface */
//             break;
//         }
//
//         /* move to next entry */
//         Entry = Entry->Flink;
//     }
//
//     /* done */
//     return Status;
// }
//
// NTSTATUS
// KspDoReparseForIrp(
//     PIRP Irp,
//     PBUS_DEVICE_ENTRY DeviceEntry)
// {
//     SIZE_T Length;
//     LPWSTR Buffer;
//     PIO_STACK_LOCATION IoStack;
//
//     /* get stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* sanity checks */
//     ASSERT(DeviceEntry->PDODeviceName);
//     ASSERT(DeviceEntry->Instance);
//     ASSERT(IoStack->FileObject);
//     ASSERT(IoStack->FileObject->FileName.Buffer);
//
//     /* calculate length */
//     Length = wcslen(DeviceEntry->PDODeviceName);
//     Length += wcslen(DeviceEntry->Instance);
//
//     /* zero byte and '\\' */
//     Length += 2;
//
//     /* allocate buffer */
//     Buffer = ExAllocatePoolWithTag(NonPagedPool, Length * sizeof(WCHAR), 'mNoI');
//     if (!Buffer)
//     {
//         /* no resources */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* construct buffer */
//     swprintf(Buffer, L!("%s\\%s"), DeviceEntry->PDODeviceName, DeviceEntry->Instance);
//
//     /* free old buffer*/
//     ExFreePoolWithTag(IoStack->FileObject->FileName.Buffer, 'mNoI');
//
//     /* store new file name */
//     RtlInitUnicodeString(&IoStack->FileObject->FileName, Buffer);
//
//     /* done */
//     return STATUS_REPARSE;
// }
//
// VOID
// KspCompletePendingIrps(
//     IN PBUS_DEVICE_ENTRY DeviceEntry,
//     IN OUT NTSTATUS ResultCode)
// {
//     PLIST_ENTRY Entry;
//     PIRP Irp;
//     NTSTATUS Status;
//
//     /* go through list */
//     while(!IsListEmpty(&DeviceEntry->IrpPendingList))
//     {
//         /* get first entry */
//         Entry = RemoveHeadList(&DeviceEntry->IrpPendingList);
//
//         /* get irp */
//         Irp = (PIRP)CONTAINING_RECORD!(Entry, IRP, Tail.Overlay.ListEntry);
//
//         if (ResultCode == STATUS_REPARSE)
//         {
//             /* construct reparse information */
//             Status = KspDoReparseForIrp(Irp, DeviceEntry);
//         }
//         else
//         {
//             /* use default code */
//             Status = ResultCode;
//         }
//
//         /* store result code */
//         Irp->IoStatus.Status = Status;
//
//         DPRINT("Completing IRP %p Status %x\n", Irp, Status);
//
//         /* complete the request */
//         CompleteRequest(Irp, IO_NO_INCREMENT);
//     }
//
// }
//
//
//
// NTSTATUS
// KspStartBusDevice(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PCOMMON_DEVICE_EXTENSION ChildDeviceExtension,
//     IN PIRP Irp)
// {
//     WCHAR PDOName[256];
//     NTSTATUS Status;
//     ULONG ResultLength;
//     LPWSTR Name;
//     ULONG NameLength;
//     PBUS_DEVICE_ENTRY DeviceEntry;
//
//     /* FIXME handle pending remove */
//
//     /* get full device name */
//     Status = IoGetDeviceProperty(DeviceObject, DevicePropertyPhysicalDeviceObjectName, sizeof(PDOName), PDOName, &ResultLength);
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed to get device name */
//         return Status;
//     }
//
//     /* allocate device name buffer */
//     NameLength = ResultLength + sizeof(UNICODE_NULL);
//     Name = AllocateItem(NonPagedPool, NameLength);
//     if (!Name)
//     {
//         /* no memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* copy name */
//     NT_VERIFY(NT_SUCCESS(RtlStringCbCopyW(Name, NameLength, PDOName)));
//
//     /* TODO: time stamp creation time */
//
//     /* get device entry */
//     DeviceEntry = (PBUS_DEVICE_ENTRY)ChildDeviceExtension->DeviceEntry;
//
//     /* sanity check */
//     ASSERT(DeviceEntry);
//
//     /* store device name */
//     DeviceEntry->PDODeviceName = Name;
//
//     /* mark device as started */
//     DeviceEntry->DeviceState = Started;
//
//     /* reference start time */
//     KeQueryTickCount(&DeviceEntry->TimeCreated);
//
//     DPRINT1("KspStartBusDevice Name %S DeviceName %S Instance %S Started\n", Name, DeviceEntry->DeviceName, DeviceEntry->Instance);
//
//     /* enable device classes */
//     //KspEnableBusDeviceInterface(DeviceEntry, TRUE);
//
//     /* done */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// KspQueryBusDeviceCapabilities(
//     IN PCOMMON_DEVICE_EXTENSION ChildDeviceExtension,
//     IN PIRP Irp)
// {
//     PDEVICE_CAPABILITIES Capabilities;
//     PIO_STACK_LOCATION IoStack;
//
//     /* get stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* get capabilities */
//     Capabilities = IoStack->Parameters.DeviceCapabilities.Capabilities;
//
//     RtlZeroMemory(Capabilities, sizeof(DEVICE_CAPABILITIES));
//
//     /* setup capabilities */
//     Capabilities->UniqueID = TRUE;
//     Capabilities->SilentInstall = TRUE;
//     Capabilities->SurpriseRemovalOK = TRUE;
//     Capabilities->Address = 0;
//     Capabilities->UINumber = 0;
//     Capabilities->SystemWake = PowerSystemWorking; /* FIXME common device extension */
//     Capabilities->DeviceWake = PowerDeviceD0;
//
//     /* done */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// KspQueryBusInformation(
//     IN PCOMMON_DEVICE_EXTENSION ChildDeviceExtension,
//     IN PIRP Irp)
// {
//     PPNP_BUS_INFORMATION BusInformation;
//
//     /* allocate bus information */
//     BusInformation = (PPNP_BUS_INFORMATION)AllocateItem(PagedPool, sizeof(PNP_BUS_INFORMATION));
//
//     if (!BusInformation)
//     {
//         /* no memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* return info */
//     BusInformation->BusNumber = 0;
//     BusInformation->LegacyBusType = InterfaceTypeUndefined;
//     RtlMoveMemory(&BusInformation->BusTypeGuid, &KSMEDIUMSETID_Standard, sizeof(GUID));
//
//     /* store result */
//     Irp->IoStatus.Information = (ULONG_PTR)BusInformation;
//
//     /* done */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// KspQueryBusDevicePnpState(
//     IN PCOMMON_DEVICE_EXTENSION ChildDeviceExtension,
//     IN PIRP Irp)
// {
//     /* set device flags */
//     Irp->IoStatus.Information = PNP_DEVICE_DONT_DISPLAY_IN_UI | PNP_DEVICE_NOT_DISABLEABLE;
//
//     /* done */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// KspQueryId(
//     IN PCOMMON_DEVICE_EXTENSION ChildDeviceExtension,
//     IN PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     PBUS_DEVICE_ENTRY DeviceEntry;
//     PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension;
//     LPWSTR Name;
//     SIZE_T Length;
//
//     /* get current irp stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     if (IoStack->Parameters.QueryId.IdType == BusQueryInstanceID)
//     {
//         /* get device entry */
//         DeviceEntry = (PBUS_DEVICE_ENTRY) ChildDeviceExtension->DeviceEntry;
//
//         /* sanity check */
//         ASSERT(DeviceEntry);
//         ASSERT(DeviceEntry->Instance);
//
//         /* calculate length */
//         Length = wcslen(DeviceEntry->Instance) + 2;
//
//         /* allocate buffer */
//         Name = AllocateItem(PagedPool, Length * sizeof(WCHAR));
//
//         if (!Name)
//         {
//             /* failed to allocate buffer */
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* copy buffer */
//         wcscpy(Name, DeviceEntry->Instance);
//
//         /* store result */
//         Irp->IoStatus.Information = (ULONG_PTR)Name;
//
//         /* done */
//         return STATUS_SUCCESS;
//     }
//     else if (IoStack->Parameters.QueryId.IdType == BusQueryDeviceID ||
//              IoStack->Parameters.QueryId.IdType == BusQueryHardwareIDs)
//     {
//         /* get device entry */
//         DeviceEntry = (PBUS_DEVICE_ENTRY) ChildDeviceExtension->DeviceEntry;
//
//         /* get bus device extension */
//         BusDeviceExtension = (PBUS_ENUM_DEVICE_EXTENSION) ChildDeviceExtension->BusDeviceExtension;
//
//         /* sanity check */
//         ASSERT(DeviceEntry);
//         ASSERT(DeviceEntry->BusId);
//         ASSERT(BusDeviceExtension);
//         ASSERT(BusDeviceExtension->BusIdentifier);
//
//         /* calculate length */
//         Length = wcslen(BusDeviceExtension->BusIdentifier);
//         Length += wcslen(DeviceEntry->BusId);
//
//         /* extra length for '\\' and 2 zero bytes */
//         Length += 4;
//
//         /* allocate buffer */
//         Name = AllocateItem(PagedPool, Length * sizeof(WCHAR));
//         if (!Name)
//         {
//             /* failed to allocate buffer */
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* construct id */
//         wcscpy(Name, BusDeviceExtension->BusIdentifier);
//         wcscat(Name, L!("\\"));
//         wcscat(Name, DeviceEntry->BusId);
//         //swprintf(Name, L!("%s\\%s"), BusDeviceExtension->BusIdentifier, DeviceEntry->BusId);
//
//         /* store result */
//         Irp->IoStatus.Information = (ULONG_PTR)Name;
//
//         /* done */
//         return STATUS_SUCCESS;
//     }
//     else
//     {
//         /* other ids are not supported */
//         //DPRINT1("Not Supported ID Type %x\n", IoStack->Parameters.QueryId.IdType);
//         return Irp->IoStatus.Status;
//     }
// }
//
// NTSTATUS
// KspInstallInterface(
//     IN PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension,
//     IN PSWENUM_INSTALL_INTERFACE InstallInterface)
// {
//     SIZE_T Length, Index;
//     UNICODE_STRING DeviceString, InterfaceString, ReferenceString;
//     HANDLE hKey, hDeviceKey, hInterfaceKey, hReferenceKey;
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//
//     /* sanity check */
//     ASSERT(InstallInterface);
//
//     /* calculate length */
//     Length = wcslen(InstallInterface->ReferenceString);
//
//     /* check for invalid characters */
//     for(Index = 0; Index < Length; Index++)
//     {
//         if (InstallInterface->ReferenceString[Index] <= L' ' ||
//             InstallInterface->ReferenceString[Index] > L'~' ||
//             InstallInterface->ReferenceString[Index] == L',' ||
//             InstallInterface->ReferenceString[Index] == L'\\' ||
//             InstallInterface->ReferenceString[Index] == L'/')
//         {
//             /* invalid character */
//             return STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     /* open bus key */
//     Status = KspOpenBusRegistryKey(BusDeviceExtension, &hKey);
//     if (NT_SUCCESS(Status))
//     {
//         /* convert device guid to string */
//         Status = RtlStringFromGUID(&InstallInterface->DeviceId, &DeviceString);
//         if (NT_SUCCESS(Status))
//         {
//             /* initialize object attributes */
//             InitializeObjectAttributes(&ObjectAttributes, &DeviceString, OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE, hKey, NULL);
//
//             /* construct device key */
//             Status = ZwCreateKey(&hDeviceKey, GENERIC_WRITE, &ObjectAttributes, 0, NULL, 0, NULL);
//             if (NT_SUCCESS(Status))
//             {
//                 /* convert interface guid to string */
//                 Status = RtlStringFromGUID(&InstallInterface->InterfaceId, &InterfaceString);
//                 if (NT_SUCCESS(Status))
//                 {
//                     /* initialize object attributes */
//                     InitializeObjectAttributes(&ObjectAttributes, &InterfaceString, OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE, hDeviceKey, NULL);
//
//                     /* construct device key */
//                     Status = ZwCreateKey(&hInterfaceKey, GENERIC_WRITE, &ObjectAttributes, 0, NULL, 0, NULL);
//                     if (NT_SUCCESS(Status))
//                     {
//                         /* initialize reference string */
//                         RtlInitUnicodeString(&ReferenceString, InstallInterface->ReferenceString);
//
//                         /* initialize object attributes */
//                         InitializeObjectAttributes(&ObjectAttributes, &ReferenceString, OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE, hInterfaceKey, NULL);
//
//                         /* construct device key */
//                         Status = ZwCreateKey(&hReferenceKey, GENERIC_WRITE, &ObjectAttributes, 0, NULL, 0, NULL);
//                         if (NT_SUCCESS(Status))
//                         {
//                             /* close key */
//                             ZwClose(hReferenceKey);
//                         }
//                     }
//                     /* free interface string */
//                     RtlFreeUnicodeString(&InterfaceString);
//
//                     /* close reference key */
//                     ZwClose(hInterfaceKey);
//                 }
//                 /* close device key */
//                 ZwClose(hDeviceKey);
//             }
//             /* free device string */
//             RtlFreeUnicodeString(&DeviceString);
//         }
//         /* close bus key */
//         ZwClose(hKey);
//     }
//
//     /* done */
//     return Status;
//  }
//
// VOID
// NTAPI
// KspInstallBusEnumInterface(
//     IN PVOID Ctx)
// {
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//     PLIST_ENTRY Entry;
//     PBUS_DEVICE_ENTRY DeviceEntry;
//     PSWENUM_INSTALL_INTERFACE InstallInterface;
//     KIRQL OldLevel;
//     PBUS_INSTALL_ENUM_CONTEXT Context = (PBUS_INSTALL_ENUM_CONTEXT)Ctx;
//
//     /* get current irp stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Context->Irp);
//
//     /* get install request */
//     InstallInterface = (PSWENUM_INSTALL_INTERFACE)Context->Irp->AssociatedIrp.SystemBuffer;
//
//     /* sanity check */
//     ASSERT(InstallInterface);
//     ASSERT(Context->BusDeviceExtension);
//
//     if (IoStack->Parameters.DeviceIoControl.InputBufferLength < sizeof(SWENUM_INSTALL_INTERFACE))
//     {
//         /* buffer too small */
//         Context->Status = STATUS_INVALID_PARAMETER;
//
//         /* signal completion */
//         KeSetEvent(&Context->Event, 0, FALSE);
//
//         /* done */
//         return;
//     }
//
//     /* FIXME locks */
//
//     /* now install the interface */
//     Status = KspInstallInterface(Context->BusDeviceExtension, InstallInterface);
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed to install interface */
//         Context->Status = Status;
//
//         /* signal completion */
//         KeSetEvent(&Context->Event, 0, FALSE);
//
//         /* done */
//         return;
//     }
//
//     /* now scan the bus */
//     Status = KspScanBus(Context->BusDeviceExtension);
//     // FIXME: We may need to check for success here, and properly fail...
//     ASSERT(NT_SUCCESS(Status));
//
//     /* acquire device entry lock */
//     KeAcquireSpinLock(&Context->BusDeviceExtension->Lock, &OldLevel);
//
//     /* now iterate all device entries */
//     ASSERT(!IsListEmpty(&Context->BusDeviceExtension->Common.Entry));
//     Entry = Context->BusDeviceExtension->Common.Entry.Flink;
//     while(Entry != &Context->BusDeviceExtension->Common.Entry)
//     {
//         /* get device entry */
//         DeviceEntry = (PBUS_DEVICE_ENTRY)CONTAINING_RECORD!(Entry, BUS_DEVICE_ENTRY, Entry);
//         if (IsEqualGUIDAligned(&DeviceEntry->DeviceGuid, &InstallInterface->DeviceId))
//         {
//             if (!DeviceEntry->PDO)
//             {
//                 /* release device entry lock */
//                 KeReleaseSpinLock(&Context->BusDeviceExtension->Lock, OldLevel);
//
//                 /* create pdo */
//                 Status = KspCreatePDO(Context->BusDeviceExtension, DeviceEntry, &DeviceEntry->PDO);
//
//                 /* acquire device entry lock */
//                 KeAcquireSpinLock(&Context->BusDeviceExtension->Lock, &OldLevel);
//
//                 /* done */
//                 break;
//             }
//         }
//
//         /* move to next entry */
//         Entry = Entry->Flink;
//     }
//
//     /* release device entry lock */
//     KeReleaseSpinLock(&Context->BusDeviceExtension->Lock, OldLevel);
//
//     /* signal that bus driver relations has changed */
//     IoInvalidateDeviceRelations(Context->BusDeviceExtension->PhysicalDeviceObject, BusRelations);
//
//     /* update status */
//     Context->Status = Status;
//
//     /* signal completion */
//     KeSetEvent(&Context->Event, 0, FALSE);
// }
//
//
// VOID
// NTAPI
// KspBusWorkerRoutine(
//   IN PVOID Parameter)
// {
//     PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension;
//     PBUS_DEVICE_ENTRY DeviceEntry;
//     PLIST_ENTRY Entry;
//     LARGE_INTEGER Time, Diff;
//     BOOLEAN DoInvalidate = FALSE;
//     KIRQL OldLevel;
//
//     /* get device extension */
//     BusDeviceExtension = (PBUS_ENUM_DEVICE_EXTENSION)Parameter;
//
//     /* acquire lock */
//     KeAcquireSpinLock(&BusDeviceExtension->Lock, &OldLevel);
//
//     /* get current time */
//     KeQueryTickCount(&Time);
//
//     /* enumerate all device entries */
//     Entry = BusDeviceExtension->Common.Entry.Flink;
//     while(Entry != &BusDeviceExtension->Common.Entry)
//     {
//         /* get offset to device entry */
//         DeviceEntry = (PBUS_DEVICE_ENTRY)CONTAINING_RECORD!(Entry, BUS_DEVICE_ENTRY, Entry);
//
//         /* sanity check */
//         ASSERT(DeviceEntry);
//
//         //DPRINT1("DeviceEntry %p PDO %p State %x\n", DeviceEntry, DeviceEntry->PDO, DeviceEntry->DeviceState);
//
//         if (DeviceEntry->PDO)
//         {
//             if (DeviceEntry->DeviceState == NotStarted)
//             {
//                 Diff.QuadPart = (Time.QuadPart - DeviceEntry->TimeCreated.QuadPart) * KeQueryTimeIncrement();
//
//                 /* wait for 15 sec */
//                 if (Diff.QuadPart > Int32x32To64(15000, 10000))
//                 {
//                      /* release spin lock */
//                      KeReleaseSpinLock(&BusDeviceExtension->Lock, OldLevel);
//
//                      DPRINT1("DeviceID %S Instance %S TimeCreated %I64u Now %I64u Diff %I64u hung\n",
//                         DeviceEntry->DeviceName,
//                         DeviceEntry->Instance,
//                         DeviceEntry->TimeCreated.QuadPart * KeQueryTimeIncrement(),
//                         Time.QuadPart * KeQueryTimeIncrement(),
//                         Diff.QuadPart);
//
//                      /* deactivate interfaces */
//                      //KspEnableBusDeviceInterface(DeviceEntry, FALSE);
//
//                      /* re-acquire lock */
//                      KeAcquireSpinLock(&BusDeviceExtension->Lock, &OldLevel);
//
//                      /* pending remove device object */
//                      DeviceEntry->DeviceState = StopPending;
//
//                      /* perform invalidation */
//                      DoInvalidate = TRUE;
//                 }
//             }
//             else if (DeviceEntry->DeviceState == Started)
//             {
//                 /* release spin lock */
//                 KeReleaseSpinLock(&BusDeviceExtension->Lock, OldLevel);
//
//                 /* found pending irps */
//                 KspCompletePendingIrps(DeviceEntry, STATUS_REPARSE);
//
//                 /* re-acquire lock */
//                 KeAcquireSpinLock(&BusDeviceExtension->Lock, &OldLevel);
//             }
//         }
//
//
//         /* move to next */
//         Entry = Entry->Flink;
//     }
//
//    /* release lock */
//     KeReleaseSpinLock(&BusDeviceExtension->Lock, OldLevel);
//
//     if (DoInvalidate)
//     {
//         /* invalidate device relations */
//         IoInvalidateDeviceRelations(BusDeviceExtension->PhysicalDeviceObject, BusRelations);
//     }
//
//     Time.QuadPart = Int32x32To64(5000, -10000);
//     KeSetTimer(&BusDeviceExtension->Timer, Time, &BusDeviceExtension->Dpc);
// }
//
// VOID
// NTAPI
// KspBusDpcRoutine(
//     IN PKDPC Dpc,
//     IN PVOID DeferredContext OPTIONAL,
//     IN PVOID SystemArgument1 OPTIONAL,
//     IN PVOID SystemArgument2 OPTIONAL)
// {
//     /* get device extension */
//     PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension = (PBUS_ENUM_DEVICE_EXTENSION)DeferredContext;
//
//     /* queue the item */
//     ExQueueWorkItem(&BusDeviceExtension->WorkItem, DelayedWorkQueue);
// }
//
// VOID
// NTAPI
// KspRemoveBusInterface(
//     PVOID Ctx)
// {
//     PBUS_INSTALL_ENUM_CONTEXT Context =(PBUS_INSTALL_ENUM_CONTEXT)Ctx;
//
//     /* TODO
//      * get SWENUM_INSTALL_INTERFACE struct
//      * open device key and delete the keys
//      */
//
//     UNIMPLEMENTED;
//
//     /* set status */
//     Context->Status = STATUS_NOT_IMPLEMENTED;
//
//
//     /* signal completion */
//     KeSetEvent(&Context->Event, IO_NO_INCREMENT, FALSE);
// }
//
// NTSTATUS
// KspQueryBusRelations(
//     IN PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension,
//     IN PIRP Irp)
// {
//     PDEVICE_RELATIONS DeviceRelations;
//     PLIST_ENTRY Entry;
//     PBUS_DEVICE_ENTRY DeviceEntry;
//     ULONG Count = 0, Length;
//     KIRQL OldLevel;
//
//     /* acquire lock */
//     KeAcquireSpinLock(&BusDeviceExtension->Lock, &OldLevel);
//
//     /* first scan all device entries */
//     Entry = BusDeviceExtension->Common.Entry.Flink;
//
//     while(Entry != &BusDeviceExtension->Common.Entry)
//     {
//         /* get offset to device entry */
//         DeviceEntry = (PBUS_DEVICE_ENTRY)CONTAINING_RECORD!(Entry, BUS_DEVICE_ENTRY, Entry);
//
//         /* is there a pdo yet */
//         if (DeviceEntry->PDO && (DeviceEntry->DeviceState == NotStarted || DeviceEntry->DeviceState == Started))
//         {
//             /* increment count */
//             Count++;
//         }
//
//         /* move to next entry */
//         Entry = Entry->Flink;
//     }
//
//     /* calculate length */
//     Length = sizeof(DEVICE_RELATIONS) + (Count > 1 ? sizeof(PDEVICE_OBJECT) * (Count-1) : 0);
//
//     /* allocate device relations */
//     DeviceRelations = (PDEVICE_RELATIONS)AllocateItem(NonPagedPool, Length);
//
//     if (!DeviceRelations)
//     {
//         /* not enough memory */
//         KeReleaseSpinLock(&BusDeviceExtension->Lock, OldLevel);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* rescan device entries */
//     Entry = BusDeviceExtension->Common.Entry.Flink;
//
//     while(Entry != &BusDeviceExtension->Common.Entry)
//     {
//         /* get offset to device entry */
//         DeviceEntry = (PBUS_DEVICE_ENTRY)CONTAINING_RECORD!(Entry, BUS_DEVICE_ENTRY, Entry);
//
//         /* is there a pdo yet */
//         if (DeviceEntry->PDO && (DeviceEntry->DeviceState == NotStarted || DeviceEntry->DeviceState == Started))
//         {
//             /* store pdo */
//             DeviceRelations->Objects[DeviceRelations->Count] = DeviceEntry->PDO;
//
//             /* reference device object */
//             ObReferenceObject(DeviceEntry->PDO);
//
//             /* increment pdo count */
//             DeviceRelations->Count++;
//         }
//
//         /* move to next entry */
//         Entry = Entry->Flink;
//     }
//
//     /* release lock */
//     KeReleaseSpinLock(&BusDeviceExtension->Lock, OldLevel);
//
//     /* FIXME handle existing device relations */
//     ASSERT(Irp->IoStatus.Information == 0);
//
//     /* store device relations */
//     Irp->IoStatus.Information = (ULONG_PTR)DeviceRelations;
//
//     /* done */
//     return STATUS_SUCCESS;
// }
//
// //------------------------------------------------------------------------------------
//
// /*
//     @implemented
// */
//
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsGetBusEnumIdentifier(
//     IN PIRP Irp)
// {
//     PDEV_EXTENSION DeviceExtension;
//     PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension;
//     PIO_STACK_LOCATION IoStack;
//     SIZE_T Length;
//     NTSTATUS Status;
//     LPWSTR Buffer;
//
//     DPRINT("KsGetBusEnumIdentifier\n");
//
//     /* get stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* sanity checks */
//     ASSERT(IoStack->DeviceObject);
//     ASSERT(IoStack->DeviceObject->DeviceExtension);
//
//     /* get device extension */
//     DeviceExtension = (PDEV_EXTENSION)IoStack->DeviceObject->DeviceExtension;
//
//     /* get bus device extension */
//     BusDeviceExtension = (PBUS_ENUM_DEVICE_EXTENSION)DeviceExtension->Ext;
//
//     /* sanity checks */
//     ASSERT(BusDeviceExtension);
//     ASSERT(BusDeviceExtension->Common.IsBus);
//
//     if (!BusDeviceExtension)
//     {
//         /* invalid parameter */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* get length */
//     Length = (wcslen(BusDeviceExtension->BusIdentifier)+1) * sizeof(WCHAR);
//
//     /* is there an output buffer provided */
//     if (IoStack->Parameters.DeviceIoControl.InputBufferLength)
//     {
//         if (Length > IoStack->Parameters.DeviceIoControl.InputBufferLength)
//         {
//             /* buffer is too small */
//             return STATUS_BUFFER_TOO_SMALL;
//         }
//
//         /* now allocate buffer */
//         Buffer = AllocateItem(NonPagedPool, Length);
//         if (!Buffer)
//         {
//             /* no memory */
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//         }
//         else
//         {
//             /* copy bus identifier */
//             wcscpy(Buffer, BusDeviceExtension->BusIdentifier);
//
//             /* store buffer */
//             Irp->AssociatedIrp.SystemBuffer = Buffer;
//
//             /* set flag that buffer gets copied back */
//             Irp->Flags |= IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER | IRP_INPUT_OPERATION;
//
//             /* done */
//             Status = STATUS_SUCCESS;
//         }
//     }
//     else
//     {
//         /* no buffer provided */
//         Status = STATUS_BUFFER_OVERFLOW;
//     }
//
//     /* done */
//     Irp->IoStatus.Status = Status;
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsGetBusEnumParentFDOFromChildPDO(
//     IN PDEVICE_OBJECT DeviceObject,
//     OUT PDEVICE_OBJECT *FunctionalDeviceObject)
// {
//     PDEV_EXTENSION DeviceExtension;
//
//     DPRINT("KsGetBusEnumParentFDOFromChildPDO\n");
//
//     /* get device extension */
//     DeviceExtension = (PDEV_EXTENSION)DeviceObject->DeviceExtension;
//
//     /* check if this is child pdo */
//     if (DeviceExtension->Ext->IsBus == FALSE)
//     {
//         /* return bus device object */
//         *FunctionalDeviceObject = DeviceExtension->Ext->BusDeviceExtension->BusDeviceObject;
//
//         /* done */
//         return STATUS_SUCCESS;
//     }
//
//     /* invalid parameter */
//     return STATUS_INVALID_PARAMETER;
// }
//
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsCreateBusEnumObject(
//     IN PWCHAR BusIdentifier,
//     IN PDEVICE_OBJECT BusDeviceObject,
//     IN PDEVICE_OBJECT PhysicalDeviceObject,
//     IN PDEVICE_OBJECT PnpDeviceObject OPTIONAL,
//     IN REFGUID InterfaceGuid OPTIONAL,
//     IN PWCHAR ServiceRelativePath OPTIONAL)
// {
//     SIZE_T Length;
//     NTSTATUS Status = STATUS_SUCCESS;
//     UNICODE_STRING ServiceKeyPath = RTL_CONSTANT_STRING(L!("\\REGISTRY\\MACHINE\\SYSTEM\\CurrentControlSet\\Services\\"));
//     PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension;
//     PDEV_EXTENSION DeviceExtension;
//     PBUS_DEVICE_ENTRY DeviceEntry;
//     PLIST_ENTRY Entry;
//     KIRQL OldLevel;
//
//     DPRINT1("KsCreateBusEnumObject %S BusDeviceObject %p\n", ServiceRelativePath, BusDeviceObject);
//
//     /* calculate sizeof bus enum device extension */
//     Length = wcslen(BusIdentifier) * sizeof(WCHAR);
//     Length += sizeof(BUS_ENUM_DEVICE_EXTENSION);
//
//     BusDeviceExtension = AllocateItem(NonPagedPool, Length);
//     if (!BusDeviceExtension)
//     {
//         /* not enough memory */
//
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* get device extension */
//     DeviceExtension = (PDEV_EXTENSION)BusDeviceObject->DeviceExtension;
//
//     /* store bus device extension */
//     DeviceExtension->Ext = (PCOMMON_DEVICE_EXTENSION)BusDeviceExtension;
//
//     DPRINT("DeviceExtension %p BusDeviceExtension %p\n", DeviceExtension, DeviceExtension->Ext);
//
//
//     /* zero device extension */
//     RtlZeroMemory(BusDeviceExtension, sizeof(BUS_ENUM_DEVICE_EXTENSION));
//
//     /* initialize bus device extension */
//     wcscpy(BusDeviceExtension->BusIdentifier, BusIdentifier);
//
//     /* allocate service path string */
//     Length = ServiceKeyPath.MaximumLength;
//     Length += BusDeviceObject->DriverObject->DriverExtension->ServiceKeyName.MaximumLength;
//
//     if (ServiceRelativePath)
//     {
//         /* relative path for devices */
//         Length += (wcslen(ServiceRelativePath) + 2) * sizeof(WCHAR);
//     }
//
//     BusDeviceExtension->ServicePath.Length = 0;
//     BusDeviceExtension->ServicePath.MaximumLength = (USHORT)Length;
//     BusDeviceExtension->ServicePath.Buffer = AllocateItem(NonPagedPool, Length);
//
//     if (!BusDeviceExtension->ServicePath.Buffer)
//     {
//         /* not enough memory */
//         FreeItem(BusDeviceExtension);
//
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlAppendUnicodeStringToString(&BusDeviceExtension->ServicePath, &ServiceKeyPath);
//     RtlAppendUnicodeStringToString(&BusDeviceExtension->ServicePath, &BusDeviceObject->DriverObject->DriverExtension->ServiceKeyName);
//
//     if (ServiceRelativePath)
//     {
//         RtlAppendUnicodeToString(&BusDeviceExtension->ServicePath, L!("\\"));
//         RtlAppendUnicodeToString(&BusDeviceExtension->ServicePath, ServiceRelativePath);
//     }
//
//     if (InterfaceGuid)
//     {
//         /* register an device interface */
//         Status = IoRegisterDeviceInterface(PhysicalDeviceObject, InterfaceGuid, NULL, &BusDeviceExtension->DeviceInterfaceLink);
//
//         /* check for success */
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("IoRegisterDeviceInterface failed Status %lx\n", Status);
//             FreeItem(BusDeviceExtension->ServicePath.Buffer);
//             FreeItem(BusDeviceExtension);
//             return Status;
//         }
//
//         /* now enable device interface */
//         Status = IoSetDeviceInterfaceState(&BusDeviceExtension->DeviceInterfaceLink, TRUE);
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("IoSetDeviceInterfaceState failed Status %lx\n", Status);
//             FreeItem(BusDeviceExtension->ServicePath.Buffer);
//             FreeItem(BusDeviceExtension);
//             return Status;
//         }
//     }
//
//     /* initialize common device extension */
//     BusDeviceExtension->Common.BusDeviceExtension = NULL;
//     BusDeviceExtension->Common.DeviceObjectReferenceCount = 1;
//     BusDeviceExtension->Common.DeviceReferenceCount = 1;
//     BusDeviceExtension->Common.IsBus = TRUE;
//     InitializeListHead(&BusDeviceExtension->Common.Entry);
//
//     /* store device objects */
//     BusDeviceExtension->BusDeviceObject = BusDeviceObject;
//     BusDeviceExtension->PhysicalDeviceObject = PhysicalDeviceObject;
//
//     /* initialize lock */
//     KeInitializeSpinLock(&BusDeviceExtension->Lock);
//
//     /* initialize timer */
//     KeInitializeTimer(&BusDeviceExtension->Timer);
//
//     /* initialize dpc */
//     KeInitializeDpc(&BusDeviceExtension->Dpc, KspBusDpcRoutine, (PVOID)BusDeviceExtension);
//
//     /* initialize event */
//     KeInitializeEvent(&BusDeviceExtension->Event, SynchronizationEvent, FALSE);
//
//     /* initialize work item */
//     ExInitializeWorkItem(&BusDeviceExtension->WorkItem, KspBusWorkerRoutine, (PVOID)BusDeviceExtension);
//
//     if (!PnpDeviceObject)
//     {
//         /* attach device */
//         BusDeviceExtension->PnpDeviceObject = IoAttachDeviceToDeviceStack(BusDeviceObject, PhysicalDeviceObject);
//
//         if (!BusDeviceExtension->PnpDeviceObject)
//         {
//             /* failed to attach device */
//             DPRINT1("IoAttachDeviceToDeviceStack failed with %x\n", Status);
//             if (BusDeviceExtension->DeviceInterfaceLink.Buffer)
//             {
//                 IoSetDeviceInterfaceState(&BusDeviceExtension->DeviceInterfaceLink, FALSE);
//                 RtlFreeUnicodeString(&BusDeviceExtension->DeviceInterfaceLink);
//             }
//
//             /* free device extension */
//             FreeItem(BusDeviceExtension->ServicePath.Buffer);
//             FreeItem(BusDeviceExtension);
//
//             return STATUS_DEVICE_REMOVED;
//         }
//
//         /* mark device as attached */
//         BusDeviceExtension->DeviceAttached = TRUE;
//     }
//     else
//     {
//         /* directly attach */
//         BusDeviceExtension->PnpDeviceObject = PnpDeviceObject;
//     }
//
//     /* now scan the bus */
//     Status = KspScanBus(BusDeviceExtension);
//
//     /* check for success */
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed to scan bus */
//         if (BusDeviceExtension->DeviceInterfaceLink.Buffer)
//         {
//             IoSetDeviceInterfaceState(&BusDeviceExtension->DeviceInterfaceLink, FALSE);
//             RtlFreeUnicodeString(&BusDeviceExtension->DeviceInterfaceLink);
//         }
//
//         if (BusDeviceExtension->DeviceAttached)
//         {
//             /* detach device */
//             IoDetachDevice(BusDeviceExtension->PnpDeviceObject);
//         }
//
//         /* free device extension */
//         FreeItem(BusDeviceExtension->ServicePath.Buffer);
//         FreeItem(BusDeviceExtension);
//
//         return Status;
//     }
//
//     /* acquire device entry lock */
//     KeAcquireSpinLock(&BusDeviceExtension->Lock, &OldLevel);
//
//     /* now iterate all device entries */
//     Entry = BusDeviceExtension->Common.Entry.Flink;
//     while(Entry != &BusDeviceExtension->Common.Entry)
//     {
//         /* get device entry */
//         DeviceEntry = (PBUS_DEVICE_ENTRY)CONTAINING_RECORD!(Entry, BUS_DEVICE_ENTRY, Entry);
//         if (!DeviceEntry->PDO)
//         {
//             /* release device entry lock */
//             KeReleaseSpinLock(&BusDeviceExtension->Lock, OldLevel);
//
//             /* create pdo */
//             Status = KspCreatePDO(BusDeviceExtension, DeviceEntry, &DeviceEntry->PDO);
//
//             /* acquire device entry lock */
//             KeAcquireSpinLock(&BusDeviceExtension->Lock, &OldLevel);
//
//             /* done */
//             break;
//         }
//         /* move to next entry */
//         Entry = Entry->Flink;
//     }
//
//     /* release device entry lock */
//     KeReleaseSpinLock(&BusDeviceExtension->Lock, OldLevel);
//
//
//     /* invalidate device relations */
//     IoInvalidateDeviceRelations(BusDeviceExtension->PhysicalDeviceObject, BusRelations);
//     DPRINT("KsCreateBusEnumObject Status %x\n", Status);
//     /* done */
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsGetBusEnumPnpDeviceObject(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PDEVICE_OBJECT *PnpDeviceObject)
// {
//     PDEV_EXTENSION DeviceExtension;
//     PCOMMON_DEVICE_EXTENSION CommonDeviceExtension;
//     PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension;
//
//     DPRINT("KsGetBusEnumPnpDeviceObject\n");
//
//     if (!DeviceObject->DeviceExtension)
//     {
//         /* invalid parameter */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* get device extension */
//     DeviceExtension = (PDEV_EXTENSION)DeviceObject->DeviceExtension;
//
//     /* get common device extension */
//     CommonDeviceExtension = DeviceExtension->Ext;
//
//     if (!CommonDeviceExtension)
//     {
//         /* invalid parameter */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if (!CommonDeviceExtension->IsBus)
//     {
//         /* getting pnp device object is only supported for software bus device object */
//        return STATUS_INVALID_PARAMETER;
//     }
//
//     /* sanity checks */
//     ASSERT(CommonDeviceExtension);
//     ASSERT(CommonDeviceExtension->IsBus);
//
//     /* cast to bus device extension */
//     BusDeviceExtension = (PBUS_ENUM_DEVICE_EXTENSION)CommonDeviceExtension;
//
//     /* store result */
//     *PnpDeviceObject = BusDeviceExtension->PnpDeviceObject;
//
//     /* done */
//     return STATUS_SUCCESS;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsInstallBusEnumInterface(
//     PIRP Irp)
// {
//     BUS_INSTALL_ENUM_CONTEXT Context;
//     KPROCESSOR_MODE Mode;
//     LUID luid;
//     PIO_STACK_LOCATION IoStack;
//     PDEV_EXTENSION DeviceExtension;
//     PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension;
//
//     DPRINT("KsInstallBusEnumInterface\n");
//
//     /* get current irp stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* get previous mode */
//     Mode = ExGetPreviousMode();
//
//     /* convert to luid */
//     luid = RtlConvertUlongToLuid(SE_LOAD_DRIVER_PRIVILEGE);
//
//     /* perform access check */
//     if (!SeSinglePrivilegeCheck(luid, Mode))
//     {
//         /* FIXME insufficient privileges */
//         //return STATUS_PRIVILEGE_NOT_HELD;
//     }
//
//     /* get device extension */
//     DeviceExtension = (PDEV_EXTENSION)IoStack->DeviceObject->DeviceExtension;
//
//     /* get bus device extension */
//     BusDeviceExtension = (PBUS_ENUM_DEVICE_EXTENSION)DeviceExtension->Ext;
//
//
//     /* initialize context */
//     Context.Irp = Irp;
//     KeInitializeEvent(&Context.Event, NotificationEvent, FALSE);
//     Context.BusDeviceExtension = BusDeviceExtension;
//     ExInitializeWorkItem(&Context.WorkItem, KspInstallBusEnumInterface, (PVOID)&Context);
//
//     /* queue the work item */
//     ExQueueWorkItem(&Context.WorkItem, DelayedWorkQueue);
//     /* wait for completion */
//     KeWaitForSingleObject(&Context.Event, Executive, KernelMode, FALSE, NULL);
//
//     /* store result */
//     Irp->IoStatus.Status = Context.Status;
//
//     /* done */
//     return Context.Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsIsBusEnumChildDevice(
//     IN PDEVICE_OBJECT DeviceObject,
//     OUT PBOOLEAN ChildDevice)
// {
//     PDEV_EXTENSION DeviceExtension;
//     PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension;
//
//     DPRINT("KsIsBusEnumChildDevice %p\n", DeviceObject);
//
//     /* get device extension */
//     DeviceExtension = (PDEV_EXTENSION)DeviceObject->DeviceExtension;
//
//     /* get bus device extension */
//     BusDeviceExtension = (PBUS_ENUM_DEVICE_EXTENSION)DeviceExtension->Ext;
//
//     if (!BusDeviceExtension)
//     {
//         /* not a bus device */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* store result */
//     *ChildDevice = (BusDeviceExtension->Common.IsBus == FALSE);
//
//     return STATUS_SUCCESS;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsServiceBusEnumCreateRequest(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp)
// {
//     PLIST_ENTRY Entry;
//     PBUS_DEVICE_ENTRY DeviceEntry = NULL; /* fix gcc */
//     PIO_STACK_LOCATION IoStack;
//     BOOLEAN ItemExists = FALSE;
//     PDEV_EXTENSION DeviceExtension;
//     PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension;
//     //PCOMMON_DEVICE_EXTENSION ChildDeviceExtension;
//     NTSTATUS Status;
//     LARGE_INTEGER Time;
//
//     /* FIXME: locks */
//
//     /* get device extension */
//     DeviceExtension = (PDEV_EXTENSION)DeviceObject->DeviceExtension;
//
//     /* get bus device extension */
//     BusDeviceExtension = (PBUS_ENUM_DEVICE_EXTENSION)DeviceExtension->Ext;
//
//     /* get current irp stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* sanity checks */
//     ASSERT(IoStack->FileObject);
//     if (IoStack->FileObject->FileName.Buffer == NULL)
//     {
//          DPRINT("KsServiceBusEnumCreateRequest PNP Hack\n");
//          Irp->IoStatus.Status = STATUS_SUCCESS;
//          return STATUS_SUCCESS;
//     }
//
//     ASSERT(IoStack->FileObject->FileName.Buffer);
//
//     DPRINT1("KsServiceBusEnumCreateRequest IRP %p Name %wZ\n", Irp, &IoStack->FileObject->FileName);
//
//     /* scan list and check if it is already present */
//     Entry = BusDeviceExtension->Common.Entry.Flink;
//
//     while(Entry != &BusDeviceExtension->Common.Entry)
//     {
//         /* get real offset */
//         DeviceEntry = (PBUS_DEVICE_ENTRY)CONTAINING_RECORD!(Entry, BUS_DEVICE_ENTRY, Entry);
//
//         /* check if name matches */
//         if (!_wcsicmp(DeviceEntry->DeviceName, IoStack->FileObject->FileName.Buffer + 1))
//         {
//             /* item already exists */
//             ItemExists = TRUE;
//             break;
//         }
//
//         /* move to next entry */
//         Entry = Entry->Flink;
//     }
//
//     if (!ItemExists)
//     {
//         /* interface not registered */
//         DPRINT1("Interface %wZ not registered\n", &IoStack->FileObject->FileName);
//         return STATUS_OBJECT_NAME_NOT_FOUND;
//     }
//
//     /*  is there a pdo yet */
//     if (DeviceEntry->PDO)
//     {
//         if (DeviceEntry->DeviceState == Started)
//         {
//             /* issue reparse */
//             Status =  KspDoReparseForIrp(Irp, DeviceEntry);
//             DPRINT("REPARSE Irp %p '%wZ'\n", Irp, &IoStack->FileObject->FileName);
//
//             Irp->IoStatus.Status = Status;
//             Irp->IoStatus.Information = IO_REPARSE;
//             return Status;
//         }
//
//         /* delay processing until pnp is finished with enumeration */
//         IoMarkIrpPending(Irp);
//
//         /* insert into irp pending list */
//         InsertTailList(&DeviceEntry->IrpPendingList, &Irp->Tail.Overlay.ListEntry);
//
//         Time.QuadPart = Int32x32To64(1500, -10000);
//         DbgPrint("PENDING Irp %p %wZ DeviceState %d\n", Irp, &IoStack->FileObject->FileName, DeviceEntry->DeviceState);
//
//
//         /* set timer */
//         KeSetTimer(&BusDeviceExtension->Timer, Time, &BusDeviceExtension->Dpc);
//
//         /* done for now */
//         return STATUS_PENDING;
//
//     }
//     else
//     {
//         /* time to create PDO */
//         Status = KspCreatePDO(BusDeviceExtension, DeviceEntry, &DeviceEntry->PDO);
//
//         if (!NT_SUCCESS(Status))
//         {
//             /* failed to create PDO */
//             DPRINT1("KsServiceBusEnumCreateRequest failed to create PDO with %x\n", Status);
//             return Status;
//         }
//         DPRINT("PENDING CREATE Irp %p %wZ\n", Irp, &IoStack->FileObject->FileName);
//
//         /* delay processing until pnp is finished with enumeration */
//         IoMarkIrpPending(Irp);
//
//         /* insert into irp pending list */
//         InsertTailList(&DeviceEntry->IrpPendingList, &Irp->Tail.Overlay.ListEntry);
//
//         /* invalidate device relations */
//         IoInvalidateDeviceRelations(BusDeviceExtension->PhysicalDeviceObject, BusRelations);
//
//         /* done for now */
//         return STATUS_PENDING;
//     }
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsServiceBusEnumPnpRequest(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp)
// {
//     PDEV_EXTENSION DeviceExtension;
//     PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension;
//     PCOMMON_DEVICE_EXTENSION ChildDeviceExtension;
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//     LARGE_INTEGER Time;
//     PDEVICE_RELATIONS DeviceRelation;
//     PBUS_DEVICE_ENTRY DeviceEntry;
//
//     /* get device extension */
//     DeviceExtension = (PDEV_EXTENSION)DeviceObject->DeviceExtension;
//
//     /* get bus device extension */
//     BusDeviceExtension = (PBUS_ENUM_DEVICE_EXTENSION)DeviceExtension->Ext;
//
//     /* get current irp stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     if (BusDeviceExtension->Common.IsBus)
//     {
//         if (IoStack->MinorFunction == IRP_MN_START_DEVICE)
//         {
//             /* no op for bus driver */
//             Status = STATUS_SUCCESS;
//         }
//         else if (IoStack->MinorFunction == IRP_MN_QUERY_DEVICE_RELATIONS)
//         {
//             /* handle bus device relations */
//             ASSERT(IoStack->Parameters.QueryDeviceRelations.Type == BusRelations);
//
//             Status = KspQueryBusRelations(BusDeviceExtension, Irp);
//         }
//         else
//         {
//             /* get default status */
//             Status = Irp->IoStatus.Status;
//         }
//     }
//     else
//     {
//         /* get child device extension */
//         ChildDeviceExtension = DeviceExtension->Ext;
//
//         /* get bus device extension */
//         BusDeviceExtension = ChildDeviceExtension->BusDeviceExtension;
//
//         if (IoStack->MinorFunction == IRP_MN_QUERY_ID)
//         {
//             /* query id */
//             Status = KspQueryId(ChildDeviceExtension, Irp);
//         }
//         else if (IoStack->MinorFunction == IRP_MN_REMOVE_DEVICE)
//         {
//             ASSERT(ChildDeviceExtension->DeviceEntry->DeviceState != Started || ChildDeviceExtension->DeviceEntry->DeviceState == NotStarted);
//             ASSERT(ChildDeviceExtension->DeviceEntry->PDO == DeviceObject);
//
//             /* backup device entry */
//             DeviceEntry = ChildDeviceExtension->DeviceEntry;
//
//             /* free device extension */
//             FreeItem(ChildDeviceExtension);
//
//             /* clear PDO reference */
//             DeviceEntry->PDO = NULL;
//
//             /* delete the device */
//             IoDeleteDevice(DeviceObject);
//
//             if (DeviceEntry->PDODeviceName)
//             {
//                 /* delete pdo device name */
//                 FreeItem(DeviceEntry->PDODeviceName);
//
//                 /* set to null */
//                 DeviceEntry->PDODeviceName = NULL;
//             }
//
//             /* set state no notstarted */
//             DeviceEntry->DeviceState = NotStarted;
//
//             /* complete pending irps */
//             KspCompletePendingIrps(DeviceEntry, STATUS_DEVICE_REMOVED);
//
//             /* done */
//             Status = STATUS_SUCCESS;
//         }
//         else if (IoStack->MinorFunction == IRP_MN_QUERY_BUS_INFORMATION)
//         {
//             /* query bus information */
//             Status = KspQueryBusInformation(ChildDeviceExtension, Irp);
//         }
//         else if (IoStack->MinorFunction == IRP_MN_QUERY_RESOURCES)
//         {
//             /* no op */
//             Status = STATUS_SUCCESS;
//         }
//         else if (IoStack->MinorFunction == IRP_MN_QUERY_RESOURCE_REQUIREMENTS)
//         {
//             /* no op */
//             Status = STATUS_SUCCESS;
//         }
//         else if (IoStack->MinorFunction == IRP_MN_START_DEVICE)
//         {
//             /* start bus */
//             Status = KspStartBusDevice(DeviceObject, ChildDeviceExtension, Irp);
//             if (NT_SUCCESS(Status))
//             {
//                 /* complete pending irps*/
//                 KspCompletePendingIrps(ChildDeviceExtension->DeviceEntry, STATUS_REPARSE);
//             }
//
//             /* set time out */
//             Time.QuadPart = Int32x32To64(1500, -10000);
//
//             /* sanity check */
//             ASSERT(BusDeviceExtension);
//
//             /* set timer */
//             KeSetTimer(&BusDeviceExtension->Timer, Time, &BusDeviceExtension->Dpc);
//         }
//         else if (IoStack->MinorFunction == IRP_MN_QUERY_CAPABILITIES)
//         {
//             /* query capabilities */
//             Status = KspQueryBusDeviceCapabilities(ChildDeviceExtension, Irp);
//         }
//         else if (IoStack->MinorFunction == IRP_MN_QUERY_PNP_DEVICE_STATE)
//         {
//             /* query pnp state */
//             Status = KspQueryBusDevicePnpState(ChildDeviceExtension, Irp);
//         }
//         else if (IoStack->MinorFunction == IRP_MN_QUERY_INTERFACE)
//         {
//             /* query interface */
//             Status = KspQueryBusDeviceInterface(ChildDeviceExtension, Irp);
//         }
//         else if (IoStack->MinorFunction == IRP_MN_QUERY_DEVICE_RELATIONS && IoStack->Parameters.QueryDeviceRelations.Type == TargetDeviceRelation)
//         {
//             /* handle target device relations */
//             ASSERT(Irp->IoStatus.Information == 0);
//
//             /* allocate device relation */
//             DeviceRelation = AllocateItem(PagedPool, sizeof(DEVICE_RELATIONS));
//             if (DeviceRelation)
//             {
//                 DeviceRelation->Count = 1;
//                 DeviceRelation->Objects[0] = DeviceObject;
//
//                 /* reference self */
//                 ObReferenceObject(DeviceObject);
//
//                 /* store result */
//                 Irp->IoStatus.Information = (ULONG_PTR)DeviceRelation;
//
//                 /* done */
//                 Status = STATUS_SUCCESS;
//             }
//             else
//             {
//                 /* no memory */
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//             }
//         }
//         else
//         {
//             /* get default status */
//             Status = Irp->IoStatus.Status;
//         }
//     }
//
//     DPRINT("KsServiceBusEnumPnpRequest %p Bus %u Function %x Status %x\n", DeviceObject, BusDeviceExtension->Common.IsBus, IoStack->MinorFunction, Status);
//     Irp->IoStatus.Status = Status;
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsRemoveBusEnumInterface(
//     IN PIRP Irp)
// {
//     KPROCESSOR_MODE Mode;
//     LUID luid;
//     BUS_INSTALL_ENUM_CONTEXT Ctx;
//     PDEV_EXTENSION DeviceExtension;
//     PBUS_ENUM_DEVICE_EXTENSION BusDeviceExtension;
//     PIO_STACK_LOCATION IoStack;
//
//     DPRINT("KsRemoveBusEnumInterface\n");
//
//     /* get io stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* get device extension */
//     DeviceExtension = (PDEV_EXTENSION)IoStack->DeviceObject->DeviceExtension;
//
//     /* get bus device extension */
//     BusDeviceExtension = DeviceExtension->Ext->BusDeviceExtension;
//
//     /* get previous mode */
//     Mode = ExGetPreviousMode();
//
//     /* convert to luid */
//     luid = RtlConvertUlongToLuid(SE_LOAD_DRIVER_PRIVILEGE);
//
//     /* perform access check */
//     if (!SeSinglePrivilegeCheck(luid, Mode))
//     {
//         /* insufficient privileges */
//         return STATUS_PRIVILEGE_NOT_HELD;
//     }
//
//     /* initialize context */
//     KeInitializeEvent(&Ctx.Event, NotificationEvent, FALSE);
//     Ctx.Irp = Irp;
//     Ctx.BusDeviceExtension = BusDeviceExtension;
//     ExInitializeWorkItem(&Ctx.WorkItem, KspRemoveBusInterface, (PVOID)&Ctx);
//
//     /* now queue the work item */
//     ExQueueWorkItem(&Ctx.WorkItem, DelayedWorkQueue);
//
//     /* wait for completion */
//     KeWaitForSingleObject(&Ctx.Event, Executive, KernelMode, FALSE, NULL);
//
//     /* return result */
//     return Ctx.Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
