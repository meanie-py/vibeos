use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _DEVICE_NODE, _DEVICE_OBJECT, _ERESOURCE, _FAST_MUTEX, _IO_BUS_TYPE_GUID_LIST,
    _IO_STACK_LOCATION, _IO_STATUS_BLOCK, _KEY_BASIC_INFORMATION, _KEY_VALUE_FULL_INFORMATION,
    _KEY_VALUE_PARTIAL_INFORMATION, _OBJECT_ATTRIBUTES, _OBJECT_NAME_INFORMATION,
    _UNICODE_STRING,
    _RTL_AVL_TABLE, DEVICE_CAPABILITIES, DEVICE_REGISTRY_PROPERTY, DEVICE_RELATION_TYPE,
    PDEVICE_NODE, PDEVICE_OBJECT, PDRIVER_OBJECT, PHANDLE, PHYSICAL_ADDRESS,
    PIO_BUS_TYPE_GUID_LIST, PIO_STACK_LOCATION, PKEY_BASIC_INFORMATION,
    PKEY_VALUE_FULL_INFORMATION, PKEY_VALUE_PARTIAL_INFORMATION, POBJECT_NAME_INFORMATION,
    PPHYSICAL_ADDRESS, ACCESS_MASK, CLONG, GUID, INTERFACE_TYPE, LPGUID, PCUNICODE_STRING,
    PUNICODE_STRING, UCHAR, WCHAR, RTL_GENERIC_COMPARE_RESULTS, PRTL_AVL_TABLE,
    DEVICE_REMOVAL_POLICY, CM_RESOURCE_LIST, PCM_RESOURCE_LIST, CM_FULL_RESOURCE_DESCRIPTOR,
    PCM_FULL_RESOURCE_DESCRIPTOR, CM_PARTIAL_RESOURCE_LIST, CM_PARTIAL_RESOURCE_DESCRIPTOR,
    PCM_PARTIAL_RESOURCE_DESCRIPTOR, ExAcquireFastMutex, ExReleaseFastMutex,
    ExAllocatePool, ExFreePool,
    ExInitializeResourceLite, HalTranslateBusAddress, ObQueryNameString,
    ULONG_PTR,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "C" {
    fn wcslen(s: PCWSTR) -> SIZE_T;
    fn wcschr(s: PCWSTR, c: WCHAR) -> PWSTR;
    fn _wcsnicmp(s1: PCWSTR, s2: PCWSTR, n: SIZE_T) -> i32;
}

#[inline(always)]
unsafe fn InitializeObjectAttributes(
    p: *mut _OBJECT_ATTRIBUTES,
    n: *mut _UNICODE_STRING,
    a: ULONG,
    r: HANDLE,
    s: PVOID,
) {
    (*p).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*p).RootDirectory = r;
    (*p).Attributes = a;
    (*p).ObjectName = n;
    (*p).SecurityDescriptor = s;
    (*p).SecurityQualityOfService = core::ptr::null_mut();
}
extern "system" {
    fn ZwOpenKey( KeyHandle: PHANDLE, DesiredAccess: ACCESS_MASK, ObjectAttributes: *mut _OBJECT_ATTRIBUTES, ) -> NTSTATUS;
    fn ZwClose(Handle: HANDLE) -> NTSTATUS;
    fn ZwQueryValueKey( KeyHandle: HANDLE, ValueName: PUNICODE_STRING, KeyValueInformationClass: ULONG, KeyValueInformation: PVOID, Length: ULONG, ResultLength: *mut ULONG, ) -> NTSTATUS;
    fn ZwEnumerateKey( KeyHandle: HANDLE, Index: ULONG, KeyInformationClass: ULONG, KeyInformation: PVOID, Length: ULONG, ResultLength: *mut ULONG, ) -> NTSTATUS;
    fn ZwSetValueKey( KeyHandle: HANDLE, ValueName: PUNICODE_STRING, TitleIndex: ULONG, Type: ULONG, Data: PVOID, DataSize: ULONG, ) -> NTSTATUS;
    fn ZwCreateKey( KeyHandle: PHANDLE, DesiredAccess: ACCESS_MASK, ObjectAttributes: *mut _OBJECT_ATTRIBUTES, TitleIndex: ULONG, Class: PUNICODE_STRING, CreateOptions: ULONG, Disposition: *mut ULONG, ) -> NTSTATUS;
    fn RtlEqualUnicodeString( String1: PCUNICODE_STRING, String2: PCUNICODE_STRING, CaseInSensitive: BOOLEAN, ) -> BOOLEAN;
    fn RtlCopyUnicodeString( DestinationString: PUNICODE_STRING, SourceString: PCUNICODE_STRING, );
    fn RtlAppendUnicodeStringToString( Destination: PUNICODE_STRING, Source: PCUNICODE_STRING, ) -> NTSTATUS;
    fn RtlAppendUnicodeToString( Destination: PUNICODE_STRING, Source: PCWSTR, ) -> NTSTATUS;
    fn RtlDuplicateUnicodeString( Flags: ULONG, SourceString: PCUNICODE_STRING, DestinationString: PUNICODE_STRING, ) -> NTSTATUS;
    fn RtlFreeUnicodeString(UnicodeString: PUNICODE_STRING);
    fn RtlInitUnicodeString(DestinationString: PUNICODE_STRING, SourceString: PCWSTR);
    fn RtlComputeCrc32(InitialCrc: ULONG, Buffer: *const UCHAR, Length: ULONG) -> ULONG;
    fn RtlCompareMemory(Source1: PVOID, Source2: PVOID, Length: SIZE_T) -> SIZE_T;
    fn RtlInitializeGenericTableAvl( Table: PRTL_AVL_TABLE, CompareRoutine: PVOID, AllocateRoutine: PVOID, FreeRoutine: PVOID, TableContext: PVOID, );
}
extern "C" {
    static mut ExpInitializationPhase: ULONG;
    static mut PopSystemPowerDeviceNode: PDEVICE_NODE;
    static mut IopDeviceTreeLock: KSPIN_LOCK;
}
extern "system" {
    fn IopIsValidPhysicalDeviceObject(DeviceObject: PDEVICE_OBJECT) -> BOOLEAN;
    fn IopQueryDeviceCapabilities( DeviceNode: PDEVICE_NODE, DeviceCaps: *mut DEVICE_CAPABILITIES, ) -> NTSTATUS;
}
// cdecl functions (no NTAPI in C declaration)
extern "C" {
    fn PiQueueDeviceAction( DeviceObject: PDEVICE_OBJECT, Action: ULONG, Request: PVOID, Context: PVOID, );
    fn PiPerformSyncDeviceAction(DeviceObject: PDEVICE_OBJECT, Action: ULONG) -> NTSTATUS;
    fn IopSynchronousCall( DeviceObject: PDEVICE_OBJECT, IoStackLocation: PIO_STACK_LOCATION, Information: *mut PVOID, ) -> NTSTATUS;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn IopGetDeviceNode(DeviceObject: PDEVICE_OBJECT) -> PDEVICE_NODE;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn IopGetDeviceNode(DeviceObject: PDEVICE_OBJECT) -> PDEVICE_NODE;
}
// `IopGetBusTypeGuidIndex_c` deleted 2026-05-06 — Rust impl below replaces it.
// Historical comment in pnpmgr.rs:631 said "Rust version causes amd64-specific
// hang"; that hang was almost certainly the same captures(none) pattern that
// bit `device.rs` (Ob*_shim trio, eliminated in commit 2ba8dab4). The
// workspace `lto = false` + `codegen-units = 256` firewall keeps the callee
// bodies (`ExAcquireFastMutex`, `ExAllocatePool`, etc.) in different CGUs
// from this caller, so the propagation can't happen.

const KeyValuePartialInformation: ULONG = 2;
const KeyValueFullInformation: ULONG = 1;
const KeyValueBasicInformation: ULONG = 0;
const KeyBasicInformation: ULONG = 0;
const REG_SZ: ULONG = 1;
const REG_DWORD: ULONG = 4;
const REG_MULTI_SZ: ULONG = 7;
const REG_RESOURCE_LIST: ULONG = 8;
const REG_RESOURCE_REQUIREMENTS_LIST: ULONG = 10;
const OBJ_NAME_PATH_SEPARATOR: WCHAR = b'\\' as u16;
const UNICODE_NULL: WCHAR = 0u16;
const RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE: ULONG = 1;
const FIELD_OFFSET_KEY_VALUE_PARTIAL_INFORMATION_Data: usize = 12;
const FIELD_OFFSET_CM_RESOURCE_LIST_List: usize = 4;
const FIELD_OFFSET_CM_FULL_RESOURCE_DESCRIPTOR_PartialResourceList: usize = 8;
const FIELD_OFFSET_CM_PARTIAL_RESOURCE_LIST_PartialDescriptors: usize = 8;
const MAXULONG: ULONG = 0xFFFFFFFF;
const ENUM_ROOT: &[u16] = &[
    b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16, b't' as u16,
    b'r' as u16, b'y' as u16, b'\\' as u16, b'M' as u16, b'a' as u16, b'c' as u16, b'h' as u16,
    b'i' as u16, b'n' as u16, b'e' as u16, b'\\' as u16, b'S' as u16, b'y' as u16, b's' as u16,
    b't' as u16, b'e' as u16, b'm' as u16, b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16,
    b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16,
    b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16, b'e' as u16, b't' as u16,
    b'\\' as u16, b'E' as u16, b'n' as u16, b'u' as u16, b'm' as u16, 0u16,
];
const KEY_READ: ACCESS_MASK = 0x20019;
const KEY_QUERY_VALUE: ACCESS_MASK = 0x0001;
const KEY_CREATE_SUB_KEY: ACCESS_MASK = 0x0004;
const KEY_ENUMERATE_SUB_KEYS: ACCESS_MASK = 0x0008;
const KEY_ALL_ACCESS: ACCESS_MASK = 0xF003F;
const REG_OPTION_NON_VOLATILE: ULONG = 0x00000000;
const REG_OPTION_VOLATILE: ULONG = 0x00000001;
const TAG_IO: ULONG = 0x20206F49; // 'Io '
const PNP_DETECTED_FATAL_ERROR: ULONG = 0xCA;
const UNEXPECTED_INITIALIZATION_CALL: ULONG = 0x33;
const InterfaceTypeUndefined: INTERFACE_TYPE = -1i32;
const PiActionEnumDeviceTree: ULONG = 0;
const PiActionQueryState: ULONG = 0x00000005;
const PLUGPLAY_REGKEY_DEVICE: ULONG = 1;
const PLUGPLAY_REGKEY_DRIVER: ULONG = 2;
const PLUGPLAY_REGKEY_CURRENT_HWPROFILE: ULONG = 4;
const IRP_MJ_PNP: UCHAR = 0x1B;
const CmResourceTypeDeviceSpecific: UCHAR = 0x8;
const BusRelations: DEVICE_RELATION_TYPE = 0;
const PowerRelations: DEVICE_RELATION_TYPE = 0x00000002;
const TargetDeviceRelation: DEVICE_RELATION_TYPE = 0x00000004;

const REGSTR_VAL_UI_NUMBER: &[u16] = &[b'U' as u16, b'I' as u16, b'N' as u16, b'u' as u16, b'm' as u16, b'b' as u16, b'e' as u16, b'r' as u16, 0];
const REGSTR_VAL_LOCATION_INFORMATION: &[u16] = &[b'L' as u16, b'o' as u16, b'c' as u16, b'a' as u16, b't' as u16, b'i' as u16, b'o' as u16, b'n' as u16, b'I' as u16, b'n' as u16, b'f' as u16, b'o' as u16, b'r' as u16, b'm' as u16, b'a' as u16, b't' as u16, b'i' as u16, b'o' as u16, b'n' as u16, 0];
const REGSTR_VAL_DEVDESC: &[u16] = &[b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'D' as u16, b'e' as u16, b's' as u16, b'c' as u16, 0];
const REGSTR_VAL_HARDWAREID: &[u16] = &[b'H' as u16, b'a' as u16, b'r' as u16, b'd' as u16, b'w' as u16, b'a' as u16, b'r' as u16, b'e' as u16, b'I' as u16, b'D' as u16, 0];
const REGSTR_VAL_COMPATIBLEIDS: &[u16] = &[b'C' as u16, b'o' as u16, b'm' as u16, b'p' as u16, b'a' as u16, b't' as u16, b'i' as u16, b'b' as u16, b'l' as u16, b'e' as u16, b'I' as u16, b'D' as u16, b's' as u16, 0];
const REGSTR_VAL_BOOTCONFIG: &[u16] = &[b'B' as u16, b'o' as u16, b'o' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b'f' as u16, b'i' as u16, b'g' as u16, 0];
const REGSTR_VAL_CLASS: &[u16] = &[b'C' as u16, b'l' as u16, b'a' as u16, b's' as u16, b's' as u16, 0];
const REGSTR_VAL_CLASSGUID: &[u16] = &[b'C' as u16, b'l' as u16, b'a' as u16, b's' as u16, b's' as u16, b'G' as u16, b'U' as u16, b'I' as u16, b'D' as u16, 0];
const REGSTR_VAL_DRIVER: &[u16] = &[b'D' as u16, b'r' as u16, b'i' as u16, b'v' as u16, b'e' as u16, b'r' as u16, 0];
const REGSTR_VAL_MFG: &[u16] = &[b'M' as u16, b'f' as u16, b'g' as u16, 0];
const REGSTR_VAL_FRIENDLYNAME: &[u16] = &[b'F' as u16, b'r' as u16, b'i' as u16, b'e' as u16, b'n' as u16, b'd' as u16, b'l' as u16, b'y' as u16, b'N' as u16, b'a' as u16, b'm' as u16, b'e' as u16, 0];
const REGSTR_VAL_CONFIGFLAGS: &[u16] = &[b'C' as u16, b'o' as u16, b'n' as u16, b'f' as u16, b'i' as u16, b'g' as u16, b'F' as u16, b'l' as u16, b'a' as u16, b'g' as u16, b's' as u16, 0];

const DevicePropertyDeviceDescription: DEVICE_REGISTRY_PROPERTY = 0;
const DevicePropertyHardwareID: DEVICE_REGISTRY_PROPERTY = 1;
const DevicePropertyCompatibleIDs: DEVICE_REGISTRY_PROPERTY = 2;
const DevicePropertyBootConfiguration: DEVICE_REGISTRY_PROPERTY = 3;
const DevicePropertyBootConfigurationTranslated: DEVICE_REGISTRY_PROPERTY = 4;
const DevicePropertyClassName: DEVICE_REGISTRY_PROPERTY = 5;
const DevicePropertyClassGuid: DEVICE_REGISTRY_PROPERTY = 6;
const DevicePropertyDriverKeyName: DEVICE_REGISTRY_PROPERTY = 7;
const DevicePropertyManufacturer: DEVICE_REGISTRY_PROPERTY = 8;
const DevicePropertyFriendlyName: DEVICE_REGISTRY_PROPERTY = 9;
const DevicePropertyLocationInformation: DEVICE_REGISTRY_PROPERTY = 10;
const DevicePropertyPhysicalDeviceObjectName: DEVICE_REGISTRY_PROPERTY = 11;
const DevicePropertyBusTypeGuid: DEVICE_REGISTRY_PROPERTY = 12;
const DevicePropertyLegacyBusType: DEVICE_REGISTRY_PROPERTY = 13;
const DevicePropertyBusNumber: DEVICE_REGISTRY_PROPERTY = 14;
const DevicePropertyEnumeratorName: DEVICE_REGISTRY_PROPERTY = 15;
const DevicePropertyAddress: DEVICE_REGISTRY_PROPERTY = 16;
const DevicePropertyUINumber: DEVICE_REGISTRY_PROPERTY = 17;
const DevicePropertyInstallState: DEVICE_REGISTRY_PROPERTY = 18;
const DevicePropertyRemovalPolicy: DEVICE_REGISTRY_PROPERTY = 19;
const DevicePropertyResourceRequirements: DEVICE_REGISTRY_PROPERTY = 20;
const DevicePropertyAllocatedResources: DEVICE_REGISTRY_PROPERTY = 21;
const DevicePropertyContainerID: DEVICE_REGISTRY_PROPERTY = 22;

#[no_mangle]
#[used]
pub static mut PpRegistryDeviceResource: _ERESOURCE = unsafe { core::mem::zeroed() };
#[no_mangle]
#[used]
pub static mut PpDeviceReferenceTableLock: KGUARDED_MUTEX = unsafe { core::mem::zeroed() };
#[no_mangle]
#[used]
pub static mut PpDeviceReferenceTable: _RTL_AVL_TABLE = unsafe { core::mem::zeroed() };

#[no_mangle]
#[used]
pub static mut IopRootDriverObject: PDRIVER_OBJECT = core::ptr::null_mut();
#[no_mangle]
#[used]
pub static mut PnpBusTypeGuidList: PIO_BUS_TYPE_GUID_LIST = core::ptr::null_mut();

unsafe fn IopFixupDeviceId(String: PWSTR) {
    let Length = wcslen(String);
    for i in 0..Length as usize {
        if *String.add(i) == b'\\' as u16 {
            *String.add(i) = b'#' as u16;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn IopInstallCriticalDevice(DeviceNode: PDEVICE_NODE) {
    let mut Status: NTSTATUS;
    let mut CriticalDeviceKey: HANDLE = core::ptr::null_mut();
    let mut InstanceKey: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    // L"\\Registry\\Machine\\System\\CurrentControlSet\\Control\\CriticalDeviceDatabase"
    static CRITICAL_DEVICE_KEY_STR: [u16; 74] = [
        b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16, b't' as u16, b'r' as u16, b'y' as u16,
        b'\\' as u16, b'M' as u16, b'a' as u16, b'c' as u16, b'h' as u16, b'i' as u16, b'n' as u16, b'e' as u16,
        b'\\' as u16, b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16,
        b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16,
        b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16, b'e' as u16, b't' as u16,
        b'\\' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16,
        b'\\' as u16, b'C' as u16, b'r' as u16, b'i' as u16, b't' as u16, b'i' as u16, b'c' as u16, b'a' as u16, b'l' as u16,
        b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16,
        b'D' as u16, b'a' as u16, b't' as u16, b'a' as u16, b'b' as u16, b'a' as u16, b's' as u16, b'e' as u16, 0,
    ];
    static COMPATIBLE_IDS_STR: [u16; 14] = [b'C' as u16, b'o' as u16, b'm' as u16, b'p' as u16, b'a' as u16, b't' as u16, b'i' as u16, b'b' as u16, b'l' as u16, b'e' as u16, b'I' as u16, b'D' as u16, b's' as u16, 0];
    static HARDWARE_ID_STR: [u16; 11] = [b'H' as u16, b'a' as u16, b'r' as u16, b'd' as u16, b'w' as u16, b'a' as u16, b'r' as u16, b'e' as u16, b'I' as u16, b'D' as u16, 0];
    static SERVICE_STR: [u16; 8] = [b'S' as u16, b'e' as u16, b'r' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, 0];
    static CLASSGUID_STR: [u16; 10] = [b'C' as u16, b'l' as u16, b'a' as u16, b's' as u16, b's' as u16, b'G' as u16, b'U' as u16, b'I' as u16, b'D' as u16, 0];

    let mut CriticalDeviceKeyU = _UNICODE_STRING {
        Length: 146, // 73 chars * 2
        MaximumLength: 148,
        Buffer: CRITICAL_DEVICE_KEY_STR.as_ptr() as *mut u16,
    };
    let mut CompatibleIdU = _UNICODE_STRING {
        Length: 26, // 13 chars * 2
        MaximumLength: 28,
        Buffer: COMPATIBLE_IDS_STR.as_ptr() as *mut u16,
    };
    let mut HardwareIdU = _UNICODE_STRING {
        Length: 20, // 10 chars * 2
        MaximumLength: 22,
        Buffer: HARDWARE_ID_STR.as_ptr() as *mut u16,
    };
    let mut ServiceU = _UNICODE_STRING {
        Length: 14, // 7 chars * 2
        MaximumLength: 16,
        Buffer: SERVICE_STR.as_ptr() as *mut u16,
    };
    let mut ClassGuidU = _UNICODE_STRING {
        Length: 18, // 9 chars * 2
        MaximumLength: 20,
        Buffer: CLASSGUID_STR.as_ptr() as *mut u16,
    };
    let mut PartialInfo: PKEY_VALUE_PARTIAL_INFORMATION;
    let mut HidLength: ULONG = 0;
    let mut CidLength: ULONG = 0;
    let mut BufferLength: ULONG;
    let mut IdBuffer: PWSTR;
    let OriginalIdBuffer: PWSTR;

    Status = IopCreateDeviceKeyPath(
        core::ptr::addr_of_mut!((*DeviceNode).InstancePath),
        REG_OPTION_NON_VOLATILE,
        &mut InstanceKey,
    );
    if Status != STATUS_SUCCESS {
        return;
    }

    crate::dbg_print!(
        b"[CDDB] check %wZ\r\n\0",
        core::ptr::addr_of!((*DeviceNode).InstancePath)
    );

    Status = ZwQueryValueKey(
        InstanceKey,
        &mut HardwareIdU,
        KeyValuePartialInformation,
        core::ptr::null_mut(),
        0,
        &mut HidLength,
    );
    if Status != STATUS_BUFFER_OVERFLOW && Status != STATUS_BUFFER_TOO_SMALL {
        ZwClose(InstanceKey);
        return;
    }

    Status = ZwQueryValueKey(
        InstanceKey,
        &mut CompatibleIdU,
        KeyValuePartialInformation,
        core::ptr::null_mut(),
        0,
        &mut CidLength,
    );
    if Status != STATUS_BUFFER_OVERFLOW && Status != STATUS_BUFFER_TOO_SMALL {
        CidLength = 0;
    }

    BufferLength = HidLength + CidLength;
    BufferLength -= (if CidLength != 0 { 2 } else { 1 })
        * FIELD_OFFSET_KEY_VALUE_PARTIAL_INFORMATION_Data as ULONG;

    OriginalIdBuffer = ExAllocatePool(PagedPool, BufferLength as ULONG_PTR) as PWSTR;
    IdBuffer = OriginalIdBuffer;
    if IdBuffer.is_null() {
        ZwClose(InstanceKey);
        return;
    }

    if HidLength > CidLength {
        BufferLength = HidLength;
    } else {
        BufferLength = CidLength;
    }

    PartialInfo = ExAllocatePool(PagedPool, BufferLength as ULONG_PTR) as PKEY_VALUE_PARTIAL_INFORMATION;
    if PartialInfo.is_null() {
        ZwClose(InstanceKey);
        ExFreePool(OriginalIdBuffer as PVOID);
        return;
    }

    Status = ZwQueryValueKey(
        InstanceKey,
        &mut HardwareIdU,
        KeyValuePartialInformation,
        PartialInfo as PVOID,
        HidLength,
        &mut HidLength,
    );
    if Status != STATUS_SUCCESS {
        ExFreePool(PartialInfo as PVOID);
        ExFreePool(OriginalIdBuffer as PVOID);
        ZwClose(InstanceKey);
        return;
    }

    HidLength = (*PartialInfo).DataLength
        - if CidLength != 0 {
            core::mem::size_of::<WCHAR>() as ULONG
        } else {
            0
        };
    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of!((*PartialInfo).Data) as *const u8,
        IdBuffer as *mut u8,
        HidLength as usize,
    );

    if CidLength != 0 {
        Status = ZwQueryValueKey(
            InstanceKey,
            &mut CompatibleIdU,
            KeyValuePartialInformation,
            PartialInfo as PVOID,
            CidLength,
            &mut CidLength,
        );
        if Status != STATUS_SUCCESS {
            ExFreePool(PartialInfo as PVOID);
            ExFreePool(OriginalIdBuffer as PVOID);
            ZwClose(InstanceKey);
            return;
        }

        CidLength = (*PartialInfo).DataLength;
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!((*PartialInfo).Data) as *const u8,
            (IdBuffer as *mut u8).add(HidLength as usize),
            CidLength as usize,
        );
    }

    ExFreePool(PartialInfo as PVOID);

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut CriticalDeviceKeyU,
        OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = ZwOpenKey(
        &mut CriticalDeviceKey,
        KEY_ENUMERATE_SUB_KEYS,
        &mut ObjectAttributes,
    );
    if Status < 0 {
        ExFreePool(OriginalIdBuffer as PVOID);
        ZwClose(InstanceKey);
        return;
    }

    while *IdBuffer != 0 {
        let StringLength = wcslen(IdBuffer) as USHORT + 1;

        IopFixupDeviceId(IdBuffer);

        {
            let IdU = _UNICODE_STRING {
                Length: (StringLength - 1) * core::mem::size_of::<WCHAR>() as USHORT,
                MaximumLength: StringLength * core::mem::size_of::<WCHAR>() as USHORT,
                Buffer: IdBuffer,
            };
            crate::dbg_print!(b"[CDDB]   try %wZ\r\n\0", core::ptr::addr_of!(IdU));
        }

        let mut Index: USHORT = 0;
        loop {
            let mut NeededLength: ULONG = 0;
            let BasicInfo: PKEY_BASIC_INFORMATION;

            Status = ZwEnumerateKey(
                CriticalDeviceKey,
                Index as ULONG,
                KeyBasicInformation,
                core::ptr::null_mut(),
                0,
                &mut NeededLength,
            );
            if Status == STATUS_NO_MORE_ENTRIES {
                break;
            } else if Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL {
                BasicInfo = ExAllocatePool(PagedPool, NeededLength as ULONG_PTR) as PKEY_BASIC_INFORMATION;
                if BasicInfo.is_null() {
                    ExFreePool(OriginalIdBuffer as PVOID);
                    ZwClose(CriticalDeviceKey);
                    ZwClose(InstanceKey);
                    return;
                }

                Status = ZwEnumerateKey(
                    CriticalDeviceKey,
                    Index as ULONG,
                    KeyBasicInformation,
                    BasicInfo as PVOID,
                    NeededLength,
                    &mut NeededLength,
                );
                if Status != STATUS_SUCCESS {
                    ExFreePool(BasicInfo as PVOID);
                    Index += 1;
                    continue;
                }

                let mut ChildIdNameU = _UNICODE_STRING {
                    Buffer: IdBuffer,
                    MaximumLength: (StringLength - 1) * core::mem::size_of::<WCHAR>() as USHORT,
                    Length: (StringLength - 1) * core::mem::size_of::<WCHAR>() as USHORT,
                };
                let mut RegKeyNameU = _UNICODE_STRING {
                    Buffer: core::ptr::addr_of_mut!((*BasicInfo).Name) as PWSTR,
                    MaximumLength: (*BasicInfo).NameLength as USHORT,
                    Length: (*BasicInfo).NameLength as USHORT,
                };

                if RtlEqualUnicodeString(&ChildIdNameU, &RegKeyNameU, 1u8) != 0 {
                    let mut ChildKeyHandle: HANDLE = core::ptr::null_mut();

                    InitializeObjectAttributes(
                        &mut ObjectAttributes,
                        &mut ChildIdNameU,
                        OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
                        CriticalDeviceKey,
                        core::ptr::null_mut(),
                    );

                    Status = ZwOpenKey(&mut ChildKeyHandle, KEY_QUERY_VALUE, &mut ObjectAttributes);
                    if Status != STATUS_SUCCESS {
                        ExFreePool(BasicInfo as PVOID);
                        Index += 1;
                        continue;
                    }

                    Status = ZwQueryValueKey(
                        InstanceKey,
                        &mut ClassGuidU,
                        KeyValuePartialInformation,
                        core::ptr::null_mut(),
                        0,
                        &mut NeededLength,
                    );
                    if Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL {
                        ExFreePool(BasicInfo as PVOID);
                        Index += 1;
                        continue;
                    }

                    Status = ZwQueryValueKey(
                        ChildKeyHandle,
                        &mut ClassGuidU,
                        KeyValuePartialInformation,
                        core::ptr::null_mut(),
                        0,
                        &mut NeededLength,
                    );
                    if Status != STATUS_BUFFER_OVERFLOW && Status != STATUS_BUFFER_TOO_SMALL {
                        ExFreePool(BasicInfo as PVOID);
                        Index += 1;
                        continue;
                    }

                    PartialInfo = ExAllocatePool(PagedPool, NeededLength as ULONG_PTR)
                        as PKEY_VALUE_PARTIAL_INFORMATION;
                    if PartialInfo.is_null() {
                        ExFreePool(OriginalIdBuffer as PVOID);
                        ExFreePool(BasicInfo as PVOID);
                        ZwClose(InstanceKey);
                        ZwClose(ChildKeyHandle);
                        ZwClose(CriticalDeviceKey);
                        return;
                    }

                    Status = ZwQueryValueKey(
                        ChildKeyHandle,
                        &mut ClassGuidU,
                        KeyValuePartialInformation,
                        PartialInfo as PVOID,
                        NeededLength,
                        &mut NeededLength,
                    );
                    if Status != STATUS_SUCCESS {
                        ExFreePool(BasicInfo as PVOID);
                        Index += 1;
                        continue;
                    }

                    Status = ZwSetValueKey(
                        InstanceKey,
                        &mut ClassGuidU,
                        0,
                        REG_SZ,
                        core::ptr::addr_of!((*PartialInfo).Data) as PVOID,
                        (*PartialInfo).DataLength,
                    );
                    if Status != STATUS_SUCCESS {
                        ExFreePool(BasicInfo as PVOID);
                        ExFreePool(PartialInfo as PVOID);
                        ZwClose(ChildKeyHandle);
                        Index += 1;
                        continue;
                    }

                    Status = ZwQueryValueKey(
                        ChildKeyHandle,
                        &mut ServiceU,
                        KeyValuePartialInformation,
                        core::ptr::null_mut(),
                        0,
                        &mut NeededLength,
                    );
                    if Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL {
                        ExFreePool(PartialInfo as PVOID);
                        PartialInfo = ExAllocatePool(PagedPool, NeededLength as ULONG_PTR)
                            as PKEY_VALUE_PARTIAL_INFORMATION;
                        if PartialInfo.is_null() {
                            ExFreePool(OriginalIdBuffer as PVOID);
                            ExFreePool(BasicInfo as PVOID);
                            ZwClose(InstanceKey);
                            ZwClose(ChildKeyHandle);
                            ZwClose(CriticalDeviceKey);
                            return;
                        }

                        Status = ZwQueryValueKey(
                            ChildKeyHandle,
                            &mut ServiceU,
                            KeyValuePartialInformation,
                            PartialInfo as PVOID,
                            NeededLength,
                            &mut NeededLength,
                        );
                        if Status != STATUS_SUCCESS {
                            ExFreePool(BasicInfo as PVOID);
                            ExFreePool(PartialInfo as PVOID);
                            ZwClose(ChildKeyHandle);
                            Index += 1;
                            continue;
                        }

                        Status = ZwSetValueKey(
                            InstanceKey,
                            &mut ServiceU,
                            0,
                            REG_SZ,
                            core::ptr::addr_of!((*PartialInfo).Data) as PVOID,
                            (*PartialInfo).DataLength,
                        );
                        if Status != STATUS_SUCCESS {
                            ExFreePool(BasicInfo as PVOID);
                            ExFreePool(PartialInfo as PVOID);
                            ZwClose(ChildKeyHandle);
                            Index += 1;
                            continue;
                        }
                    }

                    crate::dbg_print!(
                        b"[CDDB]   MATCH %wZ -> Service/ClassGUID written\r\n\0",
                        core::ptr::addr_of!(ChildIdNameU)
                    );

                    ExFreePool(OriginalIdBuffer as PVOID);
                    ExFreePool(PartialInfo as PVOID);
                    ExFreePool(BasicInfo as PVOID);
                    ZwClose(InstanceKey);
                    ZwClose(ChildKeyHandle);
                    ZwClose(CriticalDeviceKey);

                    return;
                }

                ExFreePool(BasicInfo as PVOID);
            } else {
                Index += 1;
                continue;
            }

            Index += 1;
        }

        IdBuffer = IdBuffer.add(StringLength as usize);
    }

    crate::dbg_print!(b"[CDDB]   no match\r\n\0");

    ExFreePool(OriginalIdBuffer as PVOID);
    ZwClose(InstanceKey);
    ZwClose(CriticalDeviceKey);
}

// True C ABI is cdecl (declared in ntoskrnl/include/internal/io.h WITHOUT NTAPI),
// so `extern "C"` — matches the ntkrnl_sys::ddk bindgen decl callers now resolve
// through post-extraction. The old `extern "system"` (i386 stdcall → `@4`
// decoration) only linked while the sole caller was an intra-crate Rust path.
#[no_mangle]
pub unsafe extern "C" fn IopGetSystemPowerDeviceObject(DeviceObject: *mut PDEVICE_OBJECT) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let mut OldIrql: KIRQL = 0;

    if !PopSystemPowerDeviceNode.is_null() {
        KeAcquireSpinLock(
            core::ptr::addr_of_mut!(IopDeviceTreeLock),
            &mut OldIrql,
        );
        core::ptr::write_volatile(DeviceObject, (*PopSystemPowerDeviceNode).PhysicalDeviceObject);
        KeReleaseSpinLock(
            core::ptr::addr_of_mut!(IopDeviceTreeLock),
            OldIrql,
        );

        return STATUS_SUCCESS;
    }

    STATUS_UNSUCCESSFUL
}

/// tag.h `TAG_PNP_GUIDS` ('GPnP').
pub(crate) const TAG_PNP_GUIDS: ULONG = u32::from_ne_bytes(*b"PnPG");

/// `IopGetBusTypeGuidIndex` — find or insert a bus-type GUID into the global
/// `PnpBusTypeGuidList`, returning its index (or 0xFFFF on allocation failure).
///
/// Port of upstream `a7f658e3221` ("[NTOS:IO] Fix IopGetBusTypeGuidIndex"): the list is
/// now a header with a separately allocated `Guids` array that grows 8 at a time. The old
/// shape (an inline `Guids[1]` tail, regrown one slot at a time by reallocating the whole
/// header, freeing the list on failure and then releasing its lock) was what upstream
/// fixed. Once ntkrnl-sys was regenerated against the new header, the old body wrote each
/// GUID over the `Guids` pointer and past the end of the struct: BAD_POOL_HEADER on i386.
/// Divergence: the global stays a pointer to a pool header (pnpinit.rs allocates it) where
/// upstream made it a static struct, because global_barrier.rs publishes it by pointer.
#[no_mangle]
pub unsafe extern "system" fn IopGetBusTypeGuidIndex(BusTypeGuid: LPGUID) -> USHORT {
    let List = PnpBusTypeGuidList;
    let mut FoundIndex: USHORT = 0xFFFF;

    ExAcquireFastMutex(core::ptr::addr_of_mut!((*List).Lock));

    'quickie: {
        for i in 0..(*List).GuidCount {
            if RtlCompareMemory(
                BusTypeGuid as PVOID,
                (*List).Guids.add(i as usize) as PVOID,
                core::mem::size_of::<GUID>() as SIZE_T,
            ) == core::mem::size_of::<GUID>() as SIZE_T
            {
                FoundIndex = i as USHORT;
                break 'quickie;
            }
        }

        if (*List).GuidCount == (*List).AllocatedCount {
            let NewAllocatedCount = (*List).AllocatedCount + 8;
            let NewList = ExAllocatePoolWithTag(
                PagedPool,
                NewAllocatedCount as usize * core::mem::size_of::<GUID>(),
                TAG_PNP_GUIDS,
            ) as *mut GUID;
            if NewList.is_null() {
                break 'quickie;
            }
            core::ptr::copy_nonoverlapping((*List).Guids, NewList, (*List).GuidCount as usize);
            core::ptr::write_bytes(
                NewList.add((*List).GuidCount as usize),
                0,
                (NewAllocatedCount - (*List).GuidCount) as usize,
            );
            ExFreePool((*List).Guids as PVOID);
            (*List).Guids = NewList;
            (*List).AllocatedCount = NewAllocatedCount;
        }

        *(*List).Guids.add((*List).GuidCount as usize) = *BusTypeGuid;
        FoundIndex = (*List).GuidCount as USHORT;
        (*List).GuidCount += 1;
    }

    ExReleaseFastMutex(core::ptr::addr_of_mut!((*List).Lock));
    FoundIndex
}

#[no_mangle]
pub unsafe extern "system" fn IopInitiatePnpIrp(
    DeviceObject: PDEVICE_OBJECT,
    IoStatusBlock: *mut _IO_STATUS_BLOCK,
    MinorFunction: UCHAR,
    Stack: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let Stack = opaque_ptr(Stack);
    let mut IoStackLocation: _IO_STACK_LOCATION = core::mem::zeroed();

    core::ptr::write_bytes(&mut IoStackLocation as *mut _ as *mut u8, 0, core::mem::size_of::<_IO_STACK_LOCATION>());
    IoStackLocation.MajorFunction = IRP_MJ_PNP;
    IoStackLocation.MinorFunction = MinorFunction;
    if !Stack.is_null() {
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!((*Stack).Parameters) as *const u8,
            core::ptr::addr_of_mut!(IoStackLocation.Parameters) as *mut u8,
            core::mem::size_of_val(&(*Stack).Parameters),
        );
    }

    (*IoStatusBlock).__bindgen_anon_1.Status = IopSynchronousCall(
        DeviceObject,
        &mut IoStackLocation,
        core::ptr::addr_of_mut!((*IoStatusBlock).Information).cast(),
    );
    (*IoStatusBlock).__bindgen_anon_1.Status
}

#[no_mangle]
pub unsafe extern "system" fn IopCreateDeviceKeyPath(
    RegistryPath: PCUNICODE_STRING,
    CreateOptions: ULONG,
    Handle: PHANDLE,
) -> NTSTATUS {
    let Handle = opaque_ptr(Handle);
    let EnumU = _UNICODE_STRING {
        Length: (ENUM_ROOT.len() * 2 - 2) as USHORT,
        MaximumLength: (ENUM_ROOT.len() * 2) as USHORT,
        Buffer: ENUM_ROOT.as_ptr() as *mut u16,
    };
    let mut hParent: HANDLE = core::ptr::null_mut();
    let mut hKey: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut Current: PCWSTR;
    let Last: PCWSTR;
    let mut Length: USHORT;
    let mut Status: NTSTATUS;

    core::ptr::write_volatile(Handle, core::ptr::null_mut());

    Status = IopOpenRegistryKeyEx(&mut hParent, core::ptr::null_mut(), &EnumU as *const _ as *mut _, KEY_CREATE_SUB_KEY);
    if Status < 0 {
        return Status;
    }

    KeyName.Buffer = (*RegistryPath).Buffer;
    Current = KeyName.Buffer;
    Last = ((*RegistryPath).Buffer as *const u16).add((*RegistryPath).Length as usize / core::mem::size_of::<WCHAR>());

    while (Current as usize) <= (Last as usize) {
        if (Current as usize) != (Last as usize) && *Current != b'\\' as u16 {
            Current = Current.add(1);
            continue;
        }

        Length = (Current as usize - KeyName.Buffer as usize) as USHORT ;
        KeyName.MaximumLength = Length;
        KeyName.Length = Length;

        InitializeObjectAttributes(
            &mut ObjectAttributes,
            &mut KeyName,
            OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
            hParent,
            core::ptr::null_mut(),
        );
        Status = ZwCreateKey(
            &mut hKey as *mut _ as PHANDLE,
            if (Current as usize) == (Last as usize) {
                KEY_ALL_ACCESS
            } else {
                KEY_CREATE_SUB_KEY
            },
            &mut ObjectAttributes,
            0,
            core::ptr::null_mut(),
            CreateOptions,
            core::ptr::null_mut(),
        );

        if !hParent.is_null() {
            ZwClose(hParent);
        }

        if Status < 0 {
            return Status;
        }

        if (Current as usize) == (Last as usize) {
            core::ptr::write_volatile(Handle, hKey);
            return STATUS_SUCCESS;
        }

        hParent = hKey;
        Current = Current.add(1);
        KeyName.Buffer = Current as *mut u16;
    }

    STATUS_UNSUCCESSFUL
}

#[no_mangle]
pub unsafe extern "C" fn IopSetDeviceInstanceData(InstanceKey: HANDLE, DeviceNode: PDEVICE_NODE) -> NTSTATUS {
    let InstanceKey = opaque_ptr(InstanceKey);
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut LogConfKey: HANDLE = core::ptr::null_mut();
    let mut ControlKey: HANDLE = core::ptr::null_mut();
    let mut DeviceParamsKey: HANDLE = core::ptr::null_mut();
    let ResCount: ULONG;
    let mut ResultLength: ULONG = 0;
    let mut Status: NTSTATUS;

    {static W: [u16; 8] = [b'L' as u16, b'o' as u16, b'g' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b'f' as u16, 0]; RtlInitUnicodeString(&mut KeyName, W.as_ptr() as PCWSTR);}
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut KeyName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        InstanceKey,
        core::ptr::null_mut(),
    );
    Status = ZwCreateKey(
        &mut LogConfKey,
        KEY_ALL_ACCESS,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        REG_OPTION_VOLATILE,
        core::ptr::null_mut(),
    );
    if Status >= 0 {
        if !(*DeviceNode).BootResources.is_null() {
            ResCount = (*((*DeviceNode).BootResources)).Count;
            if ResCount != 0 {
                {static W: [u16; 11] = [b'B' as u16, b'o' as u16, b'o' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b'f' as u16, b'i' as u16, b'g' as u16, 0]; RtlInitUnicodeString(&mut KeyName, W.as_ptr() as PCWSTR);}
                Status = ZwSetValueKey(
                    LogConfKey,
                    &mut KeyName,
                    0,
                    REG_RESOURCE_LIST,
                    (*DeviceNode).BootResources as PVOID,
                    PnpDetermineResourceListSize((*DeviceNode).BootResources),
                );
            }
        }

        if !(*DeviceNode).ResourceRequirements.is_null()
            && (*(*DeviceNode).ResourceRequirements).ListSize != 0
        {
            {static W: [u16; 18] = [b'B' as u16, b'a' as u16, b's' as u16, b'i' as u16, b'c' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b'f' as u16, b'i' as u16, b'g' as u16, b'V' as u16, b'e' as u16, b'c' as u16, b't' as u16, b'o' as u16, b'r' as u16, 0]; RtlInitUnicodeString(&mut KeyName, W.as_ptr() as PCWSTR);}
            Status = ZwSetValueKey(
                LogConfKey,
                &mut KeyName,
                0,
                REG_RESOURCE_REQUIREMENTS_LIST,
                (*DeviceNode).ResourceRequirements as PVOID,
                (*(*DeviceNode).ResourceRequirements).ListSize,
            );
        }

        ZwClose(LogConfKey);
    }

    {static W: [u16; 12] = [b'C' as u16, b'o' as u16, b'n' as u16, b'f' as u16, b'i' as u16, b'g' as u16, b'F' as u16, b'l' as u16, b'a' as u16, b'g' as u16, b's' as u16, 0]; RtlInitUnicodeString(&mut KeyName, W.as_ptr() as PCWSTR);}
    Status = ZwQueryValueKey(
        InstanceKey,
        &mut KeyName,
        KeyValueBasicInformation,
        core::ptr::null_mut(),
        0,
        &mut ResultLength,
    );
    if Status == STATUS_OBJECT_NAME_NOT_FOUND {
        let DefaultConfigFlags: ULONG = 0;
        Status = ZwSetValueKey(
            InstanceKey,
            &mut KeyName,
            0,
            REG_DWORD,
            core::ptr::addr_of!(DefaultConfigFlags) as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
        );
    }

    {static W: [u16; 8] = [b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, 0]; RtlInitUnicodeString(&mut KeyName, W.as_ptr() as PCWSTR);}
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut KeyName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        InstanceKey,
        core::ptr::null_mut(),
    );
    Status = ZwCreateKey(
        &mut ControlKey,
        0,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        REG_OPTION_VOLATILE,
        core::ptr::null_mut(),
    );
    if Status >= 0 {
        ZwClose(ControlKey);
    }

    if _wcsnicmp((*DeviceNode).InstancePath.Buffer, {static W: [u16; 6] = [b'A' as u16, b'C' as u16, b'P' as u16, b'I' as u16, b'\\' as u16, 0]; W.as_ptr() as PCWSTR}, 5) == 0 {
        {static W: [u16; 18] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b' ' as u16, b'P' as u16, b'a' as u16, b'r' as u16, b'a' as u16, b'm' as u16, b'e' as u16, b't' as u16, b'e' as u16, b'r' as u16, b's' as u16, 0]; RtlInitUnicodeString(&mut KeyName, W.as_ptr() as PCWSTR);}
        InitializeObjectAttributes(
            &mut ObjectAttributes,
            &mut KeyName,
            OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
            InstanceKey,
            core::ptr::null_mut(),
        );
        Status = ZwCreateKey(
            &mut DeviceParamsKey,
            0,
            &mut ObjectAttributes,
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            core::ptr::null_mut(),
        );
        if Status >= 0 {
            let FirmwareIdentified: ULONG = 1;
            {static W: [u16; 19] = [b'F' as u16, b'i' as u16, b'r' as u16, b'm' as u16, b'w' as u16, b'a' as u16, b'r' as u16, b'e' as u16, b'I' as u16, b'd' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'i' as u16, b'f' as u16, b'i' as u16, b'e' as u16, b'd' as u16, 0]; RtlInitUnicodeString(&mut KeyName, W.as_ptr() as PCWSTR);}
            Status = ZwSetValueKey(
                DeviceParamsKey,
                &mut KeyName,
                0,
                REG_DWORD,
                core::ptr::addr_of!(FirmwareIdentified) as PVOID,
                core::mem::size_of::<ULONG>() as ULONG,
            );

            ZwClose(DeviceParamsKey);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "C" fn IopGetParentIdPrefix(
    DeviceNode: PDEVICE_NODE,
    ParentIdPrefix: PUNICODE_STRING,
) -> NTSTATUS {
    let ParentIdPrefix = opaque_ptr(ParentIdPrefix);
    // L"\\Registry\\Machine\\System\\CurrentControlSet\\Enum\\"
    static ENUM_KEY_PATH_STR: [u16; 49] = [
        b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16, b't' as u16, b'r' as u16, b'y' as u16,
        b'\\' as u16, b'M' as u16, b'a' as u16, b'c' as u16, b'h' as u16, b'i' as u16, b'n' as u16, b'e' as u16,
        b'\\' as u16, b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16,
        b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16,
        b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16, b'e' as u16, b't' as u16,
        b'\\' as u16, b'E' as u16, b'n' as u16, b'u' as u16, b'm' as u16, b'\\' as u16, 0,
    ];
    let EnumKeyPath = _UNICODE_STRING {
        Length: 96,  // 48 chars * 2
        MaximumLength: 98,
        Buffer: ENUM_KEY_PATH_STR.as_ptr() as *mut u16,
    };
    let mut KeyNameBufferLength: ULONG;
    let mut ParentIdPrefixInformation: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();
    let mut KeyName = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut KeyValue: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut hKey: HANDLE = core::ptr::null_mut();
    let crc32: ULONG;
    let mut Status: NTSTATUS;

    if (*(*DeviceNode).Parent).InstancePath.Length == 0 {
        return STATUS_UNSUCCESSFUL;
    }

    KeyNameBufferLength = FIELD_OFFSET_KEY_VALUE_PARTIAL_INFORMATION_Data as ULONG
        + (18 * core::mem::size_of::<WCHAR>()) as ULONG;  // sizeof(L"12345678&12345678") = 18 wchars * 2
    ParentIdPrefixInformation = ExAllocatePoolWithTag(
        PagedPool,
        (KeyNameBufferLength + core::mem::size_of::<WCHAR>() as ULONG) as SIZE_T,
        TAG_IO,
    ) as PKEY_VALUE_PARTIAL_INFORMATION;
    if ParentIdPrefixInformation.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    KeyName.Length = 0;
    KeyName.MaximumLength = EnumKeyPath.Length
        + (*(*DeviceNode).Parent).InstancePath.Length
        + core::mem::size_of::<WCHAR>() as USHORT;
    KeyName.Buffer = ExAllocatePoolWithTag(PagedPool, KeyName.MaximumLength as SIZE_T, TAG_IO) as PWSTR;
    if KeyName.Buffer.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        ExFreePoolWithTag(ParentIdPrefixInformation as PVOID, TAG_IO);
        return Status;
    }

    RtlCopyUnicodeString(&mut KeyName, &EnumKeyPath);
    RtlAppendUnicodeStringToString(&mut KeyName, &(*(*DeviceNode).Parent).InstancePath);

    Status = IopOpenRegistryKeyEx(&mut hKey, core::ptr::null_mut(), &mut KeyName, KEY_QUERY_VALUE | KEY_SET_VALUE);
    if Status < 0 {
        RtlFreeUnicodeString(&mut KeyName);
        ExFreePoolWithTag(ParentIdPrefixInformation as PVOID, TAG_IO);
        return Status;
    }
    {static W: [u16; 15] = [b'P' as u16, b'a' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'I' as u16, b'd' as u16, b'P' as u16, b'r' as u16, b'e' as u16, b'f' as u16, b'i' as u16, b'x' as u16, 0]; RtlInitUnicodeString(&mut ValueName, W.as_ptr() as PCWSTR);}
    Status = ZwQueryValueKey(
        hKey,
        &mut ValueName,
        KeyValuePartialInformation,
        ParentIdPrefixInformation as PVOID,
        KeyNameBufferLength,
        &mut KeyNameBufferLength,
    );
    if Status >= 0 {
        if (*ParentIdPrefixInformation).Type != REG_SZ {
            Status = STATUS_UNSUCCESSFUL;
        } else {
            KeyValue.MaximumLength = (*ParentIdPrefixInformation).DataLength as USHORT;
            KeyValue.Length = KeyValue.MaximumLength - core::mem::size_of::<WCHAR>() as USHORT;
            KeyValue.Buffer = core::ptr::addr_of!((*ParentIdPrefixInformation).Data) as PWSTR;
        }
        RtlFreeUnicodeString(&mut KeyName);
        if !hKey.is_null() {
            ZwClose(hKey);
        }
        if Status >= 0 {
            Status = RtlDuplicateUnicodeString(
                RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
                &KeyValue,
                ParentIdPrefix,
            );
        }
        ExFreePoolWithTag(ParentIdPrefixInformation as PVOID, TAG_IO);
        return Status;
    }
    if Status != STATUS_OBJECT_NAME_NOT_FOUND {
        KeyValue.MaximumLength = (*ParentIdPrefixInformation).DataLength as USHORT;
        KeyValue.Length = KeyValue.MaximumLength - core::mem::size_of::<WCHAR>() as USHORT;
        KeyValue.Buffer = core::ptr::addr_of!((*ParentIdPrefixInformation).Data) as PWSTR;
        RtlFreeUnicodeString(&mut KeyName);
        if !hKey.is_null() {
            ZwClose(hKey);
        }
        if Status >= 0 {
            Status = RtlDuplicateUnicodeString(
                RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
                &KeyValue,
                ParentIdPrefix,
            );
        }
        ExFreePoolWithTag(ParentIdPrefixInformation as PVOID, TAG_IO);
        return Status;
    }

    crc32 = RtlComputeCrc32(
        0,
        (*(*DeviceNode).Parent).InstancePath.Buffer as *const UCHAR,
        (*(*DeviceNode).Parent).InstancePath.Length as ULONG,
    );

    crate::rtl_string_cb_printf_w!(
        core::ptr::addr_of_mut!((*ParentIdPrefixInformation).Data) as *mut u16,
        KeyNameBufferLength as usize,
        crate::wstr!("%lx&%lx"),
        (*(*DeviceNode).Parent).Level,
        crc32,
    );
    RtlInitUnicodeString(&mut KeyValue, core::ptr::addr_of!((*ParentIdPrefixInformation).Data) as PWSTR);

    Status = ZwSetValueKey(
        hKey,
        &mut ValueName,
        0,
        REG_SZ,
        KeyValue.Buffer as PVOID,
        ((wcslen(KeyValue.Buffer) as usize + 1) * core::mem::size_of::<WCHAR>()) as ULONG,
    );

    RtlFreeUnicodeString(&mut KeyName);
    if !hKey.is_null() {
        ZwClose(hKey);
    }
    if Status >= 0 {
        Status = RtlDuplicateUnicodeString(
            RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
            &KeyValue,
            ParentIdPrefix,
        );
    }
    ExFreePoolWithTag(ParentIdPrefixInformation as PVOID, TAG_IO);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IopOpenRegistryKeyEx(
    KeyHandle: PHANDLE,
    ParentKey: HANDLE,
    Name: PUNICODE_STRING,
    DesiredAccess: ACCESS_MASK,
) -> NTSTATUS {
    let KeyHandle = opaque_ptr(KeyHandle);
    let ParentKey = opaque_ptr(ParentKey);
    let Name = opaque_ptr(Name);
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

    core::ptr::write_volatile(KeyHandle, core::ptr::null_mut());

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        Name,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        ParentKey,
        core::ptr::null_mut(),
    );

    ZwOpenKey(KeyHandle, DesiredAccess, &mut ObjectAttributes)
}

#[no_mangle]
pub unsafe extern "system" fn IopCreateRegistryKeyEx(
    Handle: PHANDLE,
    RootHandle: HANDLE,
    KeyName: PUNICODE_STRING,
    DesiredAccess: ACCESS_MASK,
    CreateOptions: ULONG,
    Disposition: *mut ULONG,
) -> NTSTATUS {
    let Handle = opaque_ptr(Handle);
    let RootHandle = opaque_ptr(RootHandle);
    let KeyName = opaque_ptr(KeyName);
    let Disposition = opaque_ptr(Disposition);
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyDisposition: ULONG = 0;
    let mut RootHandleIndex: ULONG = 0;
    let mut i: ULONG = 1;
    let mut NestedCloseLevel: ULONG = 0;
    let mut Length: USHORT;
    let mut HandleArray: [HANDLE; 2] = [core::ptr::null_mut(); 2];
    let mut Recursing: BOOLEAN = 1u8;
    let pp: PWCHAR;
    let mut p: PWCHAR;
    let mut p1: PWCHAR;
    let mut KeyString: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    p1 = (*KeyName).Buffer;
    pp = (p1 as usize + (*KeyName).Length as usize) as PVOID as PWCHAR;

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        KeyName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        RootHandle,
        core::ptr::null_mut(),
    );
    Status = ZwCreateKey(
        &mut HandleArray[i as usize],
        DesiredAccess,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        CreateOptions,
        &mut KeyDisposition,
    );

    if (Status == STATUS_OBJECT_NAME_NOT_FOUND) && (!RootHandle.is_null()) {
        HandleArray[0] = core::ptr::null_mut();
        HandleArray[1] = RootHandle;

        while Recursing != 0 {
            if NestedCloseLevel > 1 {
                ZwClose(HandleArray[RootHandleIndex as usize]);
            }

            RootHandleIndex = i;
            i = (i + 1) & 1;

            HandleArray[i as usize] = core::ptr::null_mut();

            p = p1;
            while (p as usize) < (pp as usize) && (*p != OBJ_NAME_PATH_SEPARATOR) {
                p = p.add(1);
            }
            Length = ((p as usize - p1 as usize) / core::mem::size_of::<WCHAR>()) as USHORT
                * core::mem::size_of::<WCHAR>() as USHORT;

            if Length != 0 {
                KeyString.Buffer = p1;
                KeyString.Length = Length;
                KeyString.MaximumLength = Length;

                InitializeObjectAttributes(
                    &mut ObjectAttributes,
                    &mut KeyString,
                    OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                    HandleArray[RootHandleIndex as usize],
                    core::ptr::null_mut(),
                );
                Status = ZwCreateKey(
                    &mut HandleArray[i as usize],
                    DesiredAccess,
                    &mut ObjectAttributes,
                    0,
                    core::ptr::null_mut(),
                    CreateOptions,
                    &mut KeyDisposition,
                );
                if Status >= 0 {
                    NestedCloseLevel += 1;
                } else {
                    Recursing = 0u8;
                    continue;
                }
            } else {
                Status = STATUS_INVALID_PARAMETER;
                Recursing = 0u8;
                continue;
            }

            p1 = p.add(1);
            if (p as usize) == (pp as usize) || (p1 as usize) == (pp as usize) {
                Recursing = 0u8;
            }
        }

        if NestedCloseLevel > 1 {
            ZwClose(HandleArray[RootHandleIndex as usize]);
        }
    }

    if Status >= 0 {
        core::ptr::write_volatile(Handle, HandleArray[i as usize]);
        if !Disposition.is_null() {
            core::ptr::write_volatile(Disposition, KeyDisposition);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IopGetRegistryValue(
    Handle: HANDLE,
    ValueName: PWSTR,
    Information: *mut PKEY_VALUE_FULL_INFORMATION,
) -> NTSTATUS {
    let Handle = opaque_ptr(Handle);
    let ValueName = opaque_ptr(ValueName);
    let Information = opaque_ptr(Information);
    let mut ValueString: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let FullInformation: PKEY_VALUE_FULL_INFORMATION;
    let mut Size: ULONG = 0;

    RtlInitUnicodeString(&mut ValueString, ValueName);

    Status = ZwQueryValueKey(
        Handle,
        &mut ValueString,
        KeyValueFullInformation,
        core::ptr::null_mut(),
        0,
        &mut Size,
    );
    if (Status != STATUS_BUFFER_OVERFLOW) && (Status != STATUS_BUFFER_TOO_SMALL) {
        return Status;
    }

    FullInformation = ExAllocatePool(NonPagedPool, Size as ULONG_PTR) as PKEY_VALUE_FULL_INFORMATION;
    if FullInformation.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = ZwQueryValueKey(
        Handle,
        &mut ValueString,
        KeyValueFullInformation,
        FullInformation as PVOID,
        Size,
        &mut Size,
    );
    if Status < 0 {
        ExFreePool(FullInformation as PVOID);
        return Status;
    }

    core::ptr::write_volatile(Information, FullInformation);
    STATUS_SUCCESS
}

unsafe extern "system" fn PiCompareInstancePath(
    _Table: PRTL_AVL_TABLE,
    _FirstStruct: PVOID,
    _SecondStruct: PVOID,
) -> RTL_GENERIC_COMPARE_RESULTS {
    0
}

unsafe extern "system" fn PiAllocateGenericTableEntry(
    _Table: PRTL_AVL_TABLE,
    _ByteSize: CLONG,
) -> PVOID {
    core::ptr::null_mut()
}

unsafe extern "system" fn PiFreeGenericTableEntry(_Table: PRTL_AVL_TABLE, _Buffer: PVOID) {}

unsafe fn PpInitializeDeviceReferenceTable() {
    KeInitializeGuardedMutex(core::ptr::addr_of_mut!(PpDeviceReferenceTableLock));
    RtlInitializeGenericTableAvl(
        core::ptr::addr_of_mut!(PpDeviceReferenceTable),
        PiCompareInstancePath as PVOID,
        PiAllocateGenericTableEntry as PVOID,
        PiFreeGenericTableEntry as PVOID,
        core::ptr::null_mut(),
    );
}

unsafe fn PiInitPhase0() -> BOOLEAN {
    ExInitializeResourceLite(core::ptr::addr_of_mut!(PpRegistryDeviceResource));

    PpInitializeDeviceReferenceTable();
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn PpInitSystem() -> BOOLEAN {
    match ExpInitializationPhase {
        0 => PiInitPhase0(),
        1 => 1u8,
        _ => {
            KeBugCheckEx(UNEXPECTED_INITIALIZATION_CALL, 0, 0, 0, 0);
            0u8
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn PnpBusTypeGuidGet(
    Index: USHORT,
    BusTypeGuid: LPGUID,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    ExAcquireFastMutex(core::ptr::addr_of_mut!((*PnpBusTypeGuidList).Lock));

    if (Index as ULONG) < (*PnpBusTypeGuidList).GuidCount {
        *BusTypeGuid = *(*PnpBusTypeGuidList).Guids.add(Index as usize);
    } else {
        Status = STATUS_OBJECT_NAME_NOT_FOUND;
    }

    ExReleaseFastMutex(core::ptr::addr_of_mut!((*PnpBusTypeGuidList).Lock));
    Status
}

unsafe fn PnpDeviceObjectToDeviceInstance(
    DeviceObject: PDEVICE_OBJECT,
    DeviceInstanceHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let DeviceNode: PDEVICE_NODE;
    // L"\\REGISTRY\\MACHINE\\SYSTEM\\CURRENTCONTROLSET\\ENUM"
    static ENUM_REG_STR: [u16; 48] = [
        b'\\' as u16, b'R' as u16, b'E' as u16, b'G' as u16, b'I' as u16, b'S' as u16, b'T' as u16, b'R' as u16, b'Y' as u16,
        b'\\' as u16, b'M' as u16, b'A' as u16, b'C' as u16, b'H' as u16, b'I' as u16, b'N' as u16, b'E' as u16,
        b'\\' as u16, b'S' as u16, b'Y' as u16, b'S' as u16, b'T' as u16, b'E' as u16, b'M' as u16,
        b'\\' as u16, b'C' as u16, b'U' as u16, b'R' as u16, b'R' as u16, b'E' as u16, b'N' as u16, b'T' as u16,
        b'C' as u16, b'O' as u16, b'N' as u16, b'T' as u16, b'R' as u16, b'O' as u16, b'L' as u16, b'S' as u16, b'E' as u16, b'T' as u16,
        b'\\' as u16, b'E' as u16, b'N' as u16, b'U' as u16, b'M' as u16, 0,
    ];
    let mut KeyName = _UNICODE_STRING {
        Length: 94, // 47 chars * 2
        MaximumLength: 96,
        Buffer: ENUM_REG_STR.as_ptr() as *mut u16,
    };

    Status = IopOpenRegistryKeyEx(&mut KeyHandle, core::ptr::null_mut(), &mut KeyName, KEY_READ);
    if Status < 0 {
        return Status;
    }

    DeviceNode = IopGetDeviceNode(DeviceObject);
    if (!DeviceNode.is_null()) && ((*DeviceNode).InstancePath.Length != 0) {
        Status = IopOpenRegistryKeyEx(
            DeviceInstanceHandle,
            KeyHandle,
            core::ptr::addr_of_mut!((*DeviceNode).InstancePath),
            DesiredAccess,
        );
    } else {
        Status = STATUS_INVALID_DEVICE_REQUEST;
    }

    ZwClose(KeyHandle);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn PnpDetermineResourceListSize(
    ResourceList: PCM_RESOURCE_LIST,
) -> ULONG {
    let mut FinalSize: ULONG;
    let mut PartialSize: ULONG;
    let mut EntrySize: ULONG;
    let mut i: ULONG;
    let mut j: ULONG;
    let mut FullDescriptor: PCM_FULL_RESOURCE_DESCRIPTOR;
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;

    if ResourceList.is_null() {
        return 0;
    }

    FinalSize = FIELD_OFFSET_CM_RESOURCE_LIST_List as ULONG;

    FullDescriptor = core::ptr::addr_of!((*ResourceList).List) as PCM_FULL_RESOURCE_DESCRIPTOR;
    i = 0;
    while i < (*ResourceList).Count {
        PartialSize = FIELD_OFFSET_CM_FULL_RESOURCE_DESCRIPTOR_PartialResourceList as ULONG
            + FIELD_OFFSET_CM_PARTIAL_RESOURCE_LIST_PartialDescriptors as ULONG;

        PartialDescriptor = core::ptr::addr_of!((*FullDescriptor).PartialResourceList.PartialDescriptors)
            as PCM_PARTIAL_RESOURCE_DESCRIPTOR;
        j = 0;
        while j < (*FullDescriptor).PartialResourceList.Count as ULONG {
            EntrySize = core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>() as ULONG;

            if (*PartialDescriptor).Type == CmResourceTypeDeviceSpecific {
                EntrySize += unsafe {
                    (*PartialDescriptor)
                        .u
                        .DeviceSpecificData
                        .DataSize
                };
            }

            PartialSize += EntrySize;

            PartialDescriptor = (PartialDescriptor as usize + EntrySize as usize) as PCM_PARTIAL_RESOURCE_DESCRIPTOR;
            j += 1;
        }

        FinalSize += PartialSize;

        FullDescriptor = (FullDescriptor as usize + PartialSize as usize) as PCM_FULL_RESOURCE_DESCRIPTOR;
        i += 1;
    }

    FinalSize
}

unsafe fn PiGetDeviceRegistryProperty(
    DeviceObject: PDEVICE_OBJECT,
    ValueType: ULONG,
    ValueName: PWSTR,
    KeyName: PWSTR,
    Buffer: PVOID,
    BufferLength: *mut ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut SubHandle: HANDLE = core::ptr::null_mut();
    let mut KeyString: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyValueInfo: PKEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut Length: ULONG;

    Status = PnpDeviceObjectToDeviceInstance(DeviceObject, &mut KeyHandle, KEY_READ);
    if Status >= 0 {
        if !KeyName.is_null() {
            RtlInitUnicodeString(&mut KeyString, KeyName);
            Status = IopOpenRegistryKeyEx(&mut SubHandle, KeyHandle, &mut KeyString, KEY_READ);
            if Status >= 0 {
                ZwClose(KeyHandle);
                KeyHandle = SubHandle;
            }
        }

        if Status >= 0 {
            Status = IopGetRegistryValue(KeyHandle, ValueName, &mut KeyValueInfo);
        }

        ZwClose(KeyHandle);
    }

    if Status < 0 {
        return Status;
    }

    Length = (*KeyValueInfo).DataLength;
    if *BufferLength >= Length {
        if (*KeyValueInfo).Type == ValueType {
            core::ptr::copy_nonoverlapping(
                ((KeyValueInfo as usize + (*KeyValueInfo).DataOffset as usize) as PVOID) as *const u8,
                Buffer as *mut u8,
                Length as usize,
            );
        } else {
            Status = STATUS_INVALID_PARAMETER_2;
        }
    } else {
        Status = STATUS_BUFFER_TOO_SMALL;
    }

    *BufferLength = Length;
    ExFreePool(KeyValueInfo as PVOID);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoGetDeviceProperty(
    DeviceObject: PDEVICE_OBJECT,
    DeviceProperty: DEVICE_REGISTRY_PROPERTY,
    BufferLength: ULONG,
    PropertyBuffer: PVOID,
    ResultLength: *mut ULONG,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let PropertyBuffer = opaque_ptr(PropertyBuffer);
    let ResultLength = opaque_ptr(ResultLength);
    let DeviceNode = IopGetDeviceNode(DeviceObject);
    let mut DeviceCaps: DEVICE_CAPABILITIES = core::mem::zeroed();
    let mut ReturnLength: ULONG = 0;
    let mut ValueType: ULONG = 0;
    let mut ValueName: PWSTR = core::ptr::null_mut();
    let EnumeratorNameEnd: PWSTR;
    let DeviceInstanceName: PWSTR;
    let mut Data: PVOID = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_BUFFER_TOO_SMALL;
    let mut BusTypeGuid: GUID = core::mem::zeroed();
    let mut ObjectNameInfo: POBJECT_NAME_INFORMATION = core::ptr::null_mut();
    let mut NullTerminate: BOOLEAN = 0u8;
    let Policy: DEVICE_REMOVAL_POLICY;
    let mut Length: ULONG = 0;

    core::ptr::write_volatile(ResultLength, 0);

    if DeviceNode.is_null() {
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    match DeviceProperty {
        DevicePropertyBusTypeGuid => {
            Status = PnpBusTypeGuidGet((*DeviceNode).ChildBusTypeIndex, &mut BusTypeGuid);
            if Status < 0 {
                return Status;
            }

            ReturnLength = core::mem::size_of::<GUID>() as ULONG;
            Data = core::ptr::addr_of_mut!(BusTypeGuid) as PVOID;
            Status = STATUS_SUCCESS;
        }
        DevicePropertyLegacyBusType => {
            if (*DeviceNode).ChildInterfaceType == InterfaceTypeUndefined {
                return STATUS_OBJECT_NAME_NOT_FOUND;
            }

            ReturnLength = core::mem::size_of::<INTERFACE_TYPE>() as ULONG;
            Data = core::ptr::addr_of_mut!((*DeviceNode).ChildInterfaceType) as PVOID;
            Status = STATUS_SUCCESS;
        }
        DevicePropertyBusNumber => {
            if ((*DeviceNode).ChildBusNumber & 0x80000000) == 0x80000000 {
                return STATUS_OBJECT_NAME_NOT_FOUND;
            }

            ReturnLength = core::mem::size_of::<ULONG>() as ULONG;
            Data = core::ptr::addr_of_mut!((*DeviceNode).ChildBusNumber) as PVOID;
            Status = STATUS_SUCCESS;
        }
        DevicePropertyEnumeratorName => {
            DeviceInstanceName = (*DeviceNode).InstancePath.Buffer;

            EnumeratorNameEnd = wcschr(DeviceInstanceName, OBJ_NAME_PATH_SEPARATOR);

            NullTerminate = 1u8;

            ReturnLength = ((EnumeratorNameEnd as usize - DeviceInstanceName as usize)
                / core::mem::size_of::<WCHAR>()) as ULONG
                * core::mem::size_of::<WCHAR>() as ULONG;
            Data = DeviceInstanceName as PVOID;
            Status = STATUS_SUCCESS;
        }
        DevicePropertyAddress => {
            Status = IopQueryDeviceCapabilities(DeviceNode, &mut DeviceCaps);
            if Status < 0 || (DeviceCaps.Address == MAXULONG) {
                return STATUS_OBJECT_NAME_NOT_FOUND;
            }

            ReturnLength = core::mem::size_of::<ULONG>() as ULONG;
            Data = core::ptr::addr_of_mut!(DeviceCaps.Address) as PVOID;
            Status = STATUS_SUCCESS;
        }
        DevicePropertyBootConfigurationTranslated => {
            if (*DeviceNode).BootResources.is_null() {
                core::ptr::write_volatile(ResultLength, 0);
                return STATUS_SUCCESS;
            }

            ReturnLength = PnpDetermineResourceListSize((*DeviceNode).BootResources);
            Data = (*DeviceNode).BootResources as PVOID;
            Status = STATUS_SUCCESS;
        }
        DevicePropertyPhysicalDeviceObjectName => {
            Length = BufferLength + core::mem::size_of::<_OBJECT_NAME_INFORMATION>() as ULONG;
            ObjectNameInfo = ExAllocatePool(PagedPool, Length as ULONG_PTR) as POBJECT_NAME_INFORMATION;
            if ObjectNameInfo.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            Status = ObQueryNameString(DeviceObject as PVOID, ObjectNameInfo, Length, ResultLength);
            if Status == STATUS_INFO_LENGTH_MISMATCH {
                Status = STATUS_BUFFER_TOO_SMALL;
            }

            NullTerminate = 1u8;

            if Status >= 0 {
                ReturnLength = (*ObjectNameInfo).Name.Length as ULONG;
                Data = (*ObjectNameInfo).Name.Buffer as PVOID;
                Status = STATUS_SUCCESS;
            } else {
                let adjusted_length = core::ptr::read_volatile(ResultLength)
                    - core::mem::size_of::<_OBJECT_NAME_INFORMATION>() as ULONG;
                core::ptr::write_volatile(ResultLength, adjusted_length);
                if !ObjectNameInfo.is_null() {
                    ExFreePool(ObjectNameInfo as PVOID);
                }
                return Status;
            }
        }
        DevicePropertyRemovalPolicy => {
            Policy = (*DeviceNode).RemovalPolicy as DEVICE_REMOVAL_POLICY;
            ReturnLength = core::mem::size_of::<DEVICE_REMOVAL_POLICY>() as ULONG;
            Data = core::ptr::addr_of!(Policy) as PVOID;
            Status = STATUS_SUCCESS;
        }
        DevicePropertyUINumber => {
            ValueName = REGSTR_VAL_UI_NUMBER.as_ptr() as PWSTR;
            ValueType = REG_DWORD;
        }
        DevicePropertyLocationInformation => {
            ValueName = REGSTR_VAL_LOCATION_INFORMATION.as_ptr() as PWSTR;
            ValueType = REG_SZ;
        }
        DevicePropertyDeviceDescription => {
            ValueName = REGSTR_VAL_DEVDESC.as_ptr() as PWSTR;
            ValueType = REG_SZ;
        }
        DevicePropertyHardwareID => {
            ValueName = REGSTR_VAL_HARDWAREID.as_ptr() as PWSTR;
            ValueType = REG_MULTI_SZ;
        }
        DevicePropertyCompatibleIDs => {
            ValueName = REGSTR_VAL_COMPATIBLEIDS.as_ptr() as PWSTR;
            ValueType = REG_MULTI_SZ;
        }
        DevicePropertyBootConfiguration => {
            ValueName = REGSTR_VAL_BOOTCONFIG.as_ptr() as PWSTR;
            ValueType = REG_RESOURCE_LIST;
        }
        DevicePropertyClassName => {
            ValueName = REGSTR_VAL_CLASS.as_ptr() as PWSTR;
            ValueType = REG_SZ;
        }
        DevicePropertyClassGuid => {
            ValueName = REGSTR_VAL_CLASSGUID.as_ptr() as PWSTR;
            ValueType = REG_SZ;
        }
        DevicePropertyDriverKeyName => {
            ValueName = REGSTR_VAL_DRIVER.as_ptr() as PWSTR;
            ValueType = REG_SZ;
        }
        DevicePropertyManufacturer => {
            ValueName = REGSTR_VAL_MFG.as_ptr() as PWSTR;
            ValueType = REG_SZ;
        }
        DevicePropertyFriendlyName => {
            ValueName = REGSTR_VAL_FRIENDLYNAME.as_ptr() as PWSTR;
            ValueType = REG_SZ;
        }
        DevicePropertyContainerID => {}
        DevicePropertyInstallState => {
            ValueName = REGSTR_VAL_CONFIGFLAGS.as_ptr() as PWSTR;
            ValueType = REG_DWORD;
        }
        DevicePropertyResourceRequirements => {}
        DevicePropertyAllocatedResources => {}
        _ => return STATUS_INVALID_PARAMETER_2,
    }

    if !ValueName.is_null() {
        core::ptr::write_volatile(ResultLength, BufferLength);

        Status = PiGetDeviceRegistryProperty(
            DeviceObject,
            ValueType,
            ValueName,
            if DeviceProperty == DevicePropertyBootConfiguration {
                {static W: [u16; 8] = [b'L' as u16, b'o' as u16, b'g' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b'f' as u16, 0]; W.as_ptr() as PWSTR}
            } else {
                core::ptr::null_mut()
            },
            PropertyBuffer,
            ResultLength,
        );
    } else if Status >= 0 {
        let result_length = ReturnLength + if NullTerminate != 0 { core::mem::size_of::<WCHAR>() as ULONG } else { 0 };
        core::ptr::write_volatile(ResultLength, result_length);
        if result_length <= BufferLength {
            core::ptr::copy_nonoverlapping(Data as *const u8, PropertyBuffer as *mut u8, ReturnLength as usize);

            if NullTerminate != 0 {
                *((PropertyBuffer as *mut WCHAR).add(ReturnLength as usize / core::mem::size_of::<WCHAR>())) = UNICODE_NULL;
            }

            Status = STATUS_SUCCESS;
        } else {
            Status = STATUS_BUFFER_TOO_SMALL;
        }
    }

    if !ObjectNameInfo.is_null() {
        ExFreePool(ObjectNameInfo as PVOID);
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoOpenDeviceRegistryKey(
    DeviceObject: PDEVICE_OBJECT,
    DevInstKeyType: ULONG,
    DesiredAccess: ACCESS_MASK,
    DevInstRegKey: PHANDLE,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let DevInstRegKey = opaque_ptr(DevInstRegKey);
    static RootKeyName: [WCHAR; 44] = [
        b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16, b't' as u16, b'r' as u16,
        b'y' as u16, b'\\' as u16, b'M' as u16, b'a' as u16, b'c' as u16, b'h' as u16, b'i' as u16, b'n' as u16,
        b'e' as u16, b'\\' as u16, b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16,
        b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16,
        b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16,
        b'e' as u16, b't' as u16, b'\\' as u16, 0,
    ];
    static ProfileKeyName: [WCHAR; 52] = [
        b'H' as u16, b'a' as u16, b'r' as u16, b'd' as u16, b'w' as u16, b'a' as u16, b'r' as u16, b'e' as u16,
        b' ' as u16, b'P' as u16, b'r' as u16, b'o' as u16, b'f' as u16, b'i' as u16, b'l' as u16, b'e' as u16,
        b's' as u16, b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16,
        b't' as u16, b'\\' as u16, b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16,
        b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16,
        b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16,
        b'e' as u16, b't' as u16, b'\\' as u16, 0,
    ];
    static ClassKeyName: [WCHAR; 15] = [
        b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'\\' as u16,
        b'C' as u16, b'l' as u16, b'a' as u16, b's' as u16, b's' as u16, b'\\' as u16, 0,
    ];
    static EnumKeyName: [WCHAR; 6] = [b'E' as u16, b'n' as u16, b'u' as u16, b'm' as u16, b'\\' as u16, 0];
    static DeviceParametersKeyName: [WCHAR; 18] = [
        b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b' ' as u16, b'P' as u16,
        b'a' as u16, b'r' as u16, b'a' as u16, b'm' as u16, b'e' as u16, b't' as u16, b'e' as u16, b'r' as u16,
        b's' as u16, 0,
    ];
    let mut KeyNameLength: ULONG;
    let KeyNameBuffer: PWSTR;
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut DriverKeyLength: ULONG = 0;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut DeviceNode: PDEVICE_NODE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if (DevInstKeyType & (PLUGPLAY_REGKEY_DEVICE | PLUGPLAY_REGKEY_DRIVER)) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if IopIsValidPhysicalDeviceObject(DeviceObject) == 0 {
        return STATUS_INVALID_DEVICE_REQUEST;
    }
    DeviceNode = IopGetDeviceNode(DeviceObject);

    KeyNameLength = core::mem::size_of_val(&RootKeyName) as ULONG;
    if (DevInstKeyType & PLUGPLAY_REGKEY_CURRENT_HWPROFILE) != 0 {
        KeyNameLength += core::mem::size_of_val(&ProfileKeyName) as ULONG
            - core::mem::size_of::<WCHAR>() as ULONG;
    }
    if (DevInstKeyType & PLUGPLAY_REGKEY_DRIVER) != 0 {
        KeyNameLength += core::mem::size_of_val(&ClassKeyName) as ULONG
            - core::mem::size_of::<WCHAR>() as ULONG;
        Status = IoGetDeviceProperty(
            DeviceObject,
            DevicePropertyDriverKeyName,
            0,
            core::ptr::null_mut(),
            &mut DriverKeyLength,
        );
        if Status != STATUS_BUFFER_TOO_SMALL {
            return Status;
        }
        KeyNameLength += DriverKeyLength;
    } else {
        KeyNameLength += core::mem::size_of_val(&EnumKeyName) as ULONG
            - core::mem::size_of::<WCHAR>() as ULONG
            + (*DeviceNode).InstancePath.Length as ULONG;
    }

    KeyNameBuffer = ExAllocatePool(PagedPool, KeyNameLength as ULONG_PTR) as PWSTR;
    if KeyNameBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    KeyName.Length = 0;
    KeyName.MaximumLength = KeyNameLength as USHORT;
    KeyName.Buffer = KeyNameBuffer;

    KeyName.Length += (core::mem::size_of_val(&RootKeyName) - core::mem::size_of::<WCHAR>()) as USHORT;
    core::ptr::copy_nonoverlapping(
        RootKeyName.as_ptr() as *const u8,
        KeyNameBuffer as *mut u8,
        KeyName.Length as usize,
    );

    if (DevInstKeyType & PLUGPLAY_REGKEY_CURRENT_HWPROFILE) != 0 {
        RtlAppendUnicodeToString(&mut KeyName, ProfileKeyName.as_ptr());
    }

    if (DevInstKeyType & PLUGPLAY_REGKEY_DRIVER) != 0 {
        RtlAppendUnicodeToString(&mut KeyName, ClassKeyName.as_ptr());
        Status = IoGetDeviceProperty(
            DeviceObject,
            DevicePropertyDriverKeyName,
            DriverKeyLength,
            KeyNameBuffer.add(KeyName.Length as usize / core::mem::size_of::<WCHAR>()) as PVOID,
            &mut DriverKeyLength,
        );
        if Status < 0 {
            ExFreePool(KeyNameBuffer as PVOID);
            return Status;
        }
        KeyName.Length += DriverKeyLength as USHORT - core::mem::size_of::<WCHAR>() as USHORT;
    } else {
        RtlAppendUnicodeToString(&mut KeyName, EnumKeyName.as_ptr());
        Status = RtlAppendUnicodeStringToString(&mut KeyName, &(*DeviceNode).InstancePath);
        if (*DeviceNode).InstancePath.Length == 0 {
            ExFreePool(KeyNameBuffer as PVOID);
            return Status;
        }
    }

    Status = IopOpenRegistryKeyEx(DevInstRegKey, core::ptr::null_mut(), &mut KeyName, DesiredAccess);
    if Status < 0 {
        ExFreePool(KeyNameBuffer as PVOID);
        return Status;
    }
    ExFreePool(KeyNameBuffer as PVOID);

    if (DevInstKeyType & PLUGPLAY_REGKEY_DRIVER) != 0 {
        return Status;
    }

    RtlInitUnicodeString(&mut KeyName, DeviceParametersKeyName.as_ptr());
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut KeyName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        *DevInstRegKey,
        core::ptr::null_mut(),
    );
    Status = ZwCreateKey(
        DevInstRegKey,
        DesiredAccess,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        core::ptr::null_mut(),
    );
    ZwClose(ObjectAttributes.RootDirectory);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoInvalidateDeviceRelations(
    DeviceObject: PDEVICE_OBJECT,
    Type: DEVICE_RELATION_TYPE,
) {
    let DeviceObject = opaque_ptr(DeviceObject);
    if IopIsValidPhysicalDeviceObject(DeviceObject) == 0 {
        KeBugCheckEx(PNP_DETECTED_FATAL_ERROR, 0x2usize, DeviceObject as usize, 0usize, 0usize);
    }

    match Type {
        0 => {
            PiQueueDeviceAction(DeviceObject, PiActionEnumDeviceTree, core::ptr::null_mut(), core::ptr::null_mut());
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoSynchronousInvalidateDeviceRelations(
    DeviceObject: PDEVICE_OBJECT,
    Type: DEVICE_RELATION_TYPE,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    if IopIsValidPhysicalDeviceObject(DeviceObject) == 0 {
        KeBugCheckEx(PNP_DETECTED_FATAL_ERROR, 0x2usize, DeviceObject as usize, 0usize, 0usize);
    }

    match Type {
        0 => PiPerformSyncDeviceAction(DeviceObject, PiActionEnumDeviceTree),
        1 => STATUS_NOT_IMPLEMENTED,
        2 => STATUS_SUCCESS,
        _ => STATUS_NOT_SUPPORTED,
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoTranslateBusAddress(
    InterfaceType: INTERFACE_TYPE,
    BusNumber: ULONG,
    BusAddress: PHYSICAL_ADDRESS,
    AddressSpace: *mut ULONG,
    TranslatedAddress: PPHYSICAL_ADDRESS,
) -> BOOLEAN {
    let AddressSpace = opaque_ptr(AddressSpace);
    HalTranslateBusAddress(InterfaceType, BusNumber, BusAddress, AddressSpace, TranslatedAddress)
}

#[no_mangle]
pub unsafe extern "system" fn IoInvalidateDeviceState(DeviceObject: PDEVICE_OBJECT) {
    let DeviceObject = opaque_ptr(DeviceObject);
    if IopIsValidPhysicalDeviceObject(DeviceObject) == 0 {
        KeBugCheckEx(PNP_DETECTED_FATAL_ERROR, 0x2usize, DeviceObject as usize, 0usize, 0usize);
    }

    PiQueueDeviceAction(DeviceObject, PiActionQueryState, core::ptr::null_mut(), core::ptr::null_mut());
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * COPYRIGHT:       GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/pnpmgr/pnpmgr.c
//  * PURPOSE:         Initializes the PnP manager
//  * PROGRAMMERS:     Casper S. Hornstrup (chorns@users.sourceforge.net)
//  *                  Copyright 2007 Hervé Poussineau (hpoussin@reactos.org)
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
// ERESOURCE PpRegistryDeviceResource;
// KGUARDED_MUTEX PpDeviceReferenceTableLock;
// RTL_AVL_TABLE PpDeviceReferenceTable;
//
// extern ULONG ExpInitializationPhase;
//
// /* DATA **********************************************************************/
//
// PDRIVER_OBJECT IopRootDriverObject;
// PIO_BUS_TYPE_GUID_LIST PnpBusTypeGuidList = NULL;
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// IopFixupDeviceId(PWCHAR String)
// {
//     SIZE_T Length = wcslen(String), i;
//
//     for (i = 0; i < Length; i++)
//     {
//         if (String[i] == L'\\')
//             String[i] = L'#';
//     }
// }
//
// VOID
// NTAPI
// IopInstallCriticalDevice(PDEVICE_NODE DeviceNode)
// {
//     NTSTATUS Status;
//     HANDLE CriticalDeviceKey, InstanceKey;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING CriticalDeviceKeyU = RTL_CONSTANT_STRING(L"\\Registry\\Machine\\System\\CurrentControlSet\\Control\\CriticalDeviceDatabase");
//     UNICODE_STRING CompatibleIdU = RTL_CONSTANT_STRING(L"CompatibleIDs");
//     UNICODE_STRING HardwareIdU = RTL_CONSTANT_STRING(L"HardwareID");
//     UNICODE_STRING ServiceU = RTL_CONSTANT_STRING(L"Service");
//     UNICODE_STRING ClassGuidU = RTL_CONSTANT_STRING(L"ClassGUID");
//     PKEY_VALUE_PARTIAL_INFORMATION PartialInfo;
//     ULONG HidLength = 0, CidLength = 0, BufferLength;
//     PWCHAR IdBuffer, OriginalIdBuffer;
//
//     /* Open the device instance key */
//     Status = IopCreateDeviceKeyPath(&DeviceNode->InstancePath, REG_OPTION_NON_VOLATILE, &InstanceKey);
//     if (Status != STATUS_SUCCESS)
//         return;
//
//     Status = ZwQueryValueKey(InstanceKey,
//                              &HardwareIdU,
//                              KeyValuePartialInformation,
//                              NULL,
//                              0,
//                              &HidLength);
//     if (Status != STATUS_BUFFER_OVERFLOW && Status != STATUS_BUFFER_TOO_SMALL)
//     {
//         ZwClose(InstanceKey);
//         return;
//     }
//
//     Status = ZwQueryValueKey(InstanceKey,
//                              &CompatibleIdU,
//                              KeyValuePartialInformation,
//                              NULL,
//                              0,
//                              &CidLength);
//     if (Status != STATUS_BUFFER_OVERFLOW && Status != STATUS_BUFFER_TOO_SMALL)
//     {
//         CidLength = 0;
//     }
//
//     BufferLength = HidLength + CidLength;
//     BufferLength -= (((CidLength != 0) ? 2 : 1) * FIELD_OFFSET(KEY_VALUE_PARTIAL_INFORMATION, Data));
//
//     /* Allocate a buffer to hold data from both */
//     OriginalIdBuffer = IdBuffer = ExAllocatePool(PagedPool, BufferLength);
//     if (!IdBuffer)
//     {
//         ZwClose(InstanceKey);
//         return;
//     }
//
//     /* Compute the buffer size */
//     if (HidLength > CidLength)
//         BufferLength = HidLength;
//     else
//         BufferLength = CidLength;
//
//     PartialInfo = ExAllocatePool(PagedPool, BufferLength);
//     if (!PartialInfo)
//     {
//         ZwClose(InstanceKey);
//         ExFreePool(OriginalIdBuffer);
//         return;
//     }
//
//     Status = ZwQueryValueKey(InstanceKey,
//                              &HardwareIdU,
//                              KeyValuePartialInformation,
//                              PartialInfo,
//                              HidLength,
//                              &HidLength);
//     if (Status != STATUS_SUCCESS)
//     {
//         ExFreePool(PartialInfo);
//         ExFreePool(OriginalIdBuffer);
//         ZwClose(InstanceKey);
//         return;
//     }
//
//     /* Copy in HID info first (without 2nd terminating NULL if CID is present) */
//     HidLength = PartialInfo->DataLength - ((CidLength != 0) ? sizeof(WCHAR) : 0);
//     RtlCopyMemory(IdBuffer, PartialInfo->Data, HidLength);
//
//     if (CidLength != 0)
//     {
//         Status = ZwQueryValueKey(InstanceKey,
//                                  &CompatibleIdU,
//                                  KeyValuePartialInformation,
//                                  PartialInfo,
//                                  CidLength,
//                                  &CidLength);
//         if (Status != STATUS_SUCCESS)
//         {
//             ExFreePool(PartialInfo);
//             ExFreePool(OriginalIdBuffer);
//             ZwClose(InstanceKey);
//             return;
//         }
//
//         /* Copy CID next */
//         CidLength = PartialInfo->DataLength;
//         RtlCopyMemory(((PUCHAR)IdBuffer) + HidLength, PartialInfo->Data, CidLength);
//     }
//
//     /* Free our temp buffer */
//     ExFreePool(PartialInfo);
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &CriticalDeviceKeyU,
//                                OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = ZwOpenKey(&CriticalDeviceKey,
//                        KEY_ENUMERATE_SUB_KEYS,
//                        &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         /* The critical device database doesn't exist because
//          * we're probably in 1st stage setup, but it's ok */
//         ExFreePool(OriginalIdBuffer);
//         ZwClose(InstanceKey);
//         return;
//     }
//
//     while (*IdBuffer)
//     {
//         USHORT StringLength = (USHORT)wcslen(IdBuffer) + 1, Index;
//
//         IopFixupDeviceId(IdBuffer);
//
//         /* Look through all subkeys for a match */
//         for (Index = 0; TRUE; Index++)
//         {
//             ULONG NeededLength;
//             PKEY_BASIC_INFORMATION BasicInfo;
//
//             Status = ZwEnumerateKey(CriticalDeviceKey,
//                                     Index,
//                                     KeyBasicInformation,
//                                     NULL,
//                                     0,
//                                     &NeededLength);
//             if (Status == STATUS_NO_MORE_ENTRIES)
//                 break;
//             else if (Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL)
//             {
//                 UNICODE_STRING ChildIdNameU, RegKeyNameU;
//
//                 BasicInfo = ExAllocatePool(PagedPool, NeededLength);
//                 if (!BasicInfo)
//                 {
//                     /* No memory */
//                     ExFreePool(OriginalIdBuffer);
//                     ZwClose(CriticalDeviceKey);
//                     ZwClose(InstanceKey);
//                     return;
//                 }
//
//                 Status = ZwEnumerateKey(CriticalDeviceKey,
//                                         Index,
//                                         KeyBasicInformation,
//                                         BasicInfo,
//                                         NeededLength,
//                                         &NeededLength);
//                 if (Status != STATUS_SUCCESS)
//                 {
//                     /* This shouldn't happen */
//                     ExFreePool(BasicInfo);
//                     continue;
//                 }
//
//                 ChildIdNameU.Buffer = IdBuffer;
//                 ChildIdNameU.MaximumLength = ChildIdNameU.Length = (StringLength - 1) * sizeof(WCHAR);
//                 RegKeyNameU.Buffer = BasicInfo->Name;
//                 RegKeyNameU.MaximumLength = RegKeyNameU.Length = (USHORT)BasicInfo->NameLength;
//
//                 if (RtlEqualUnicodeString(&ChildIdNameU, &RegKeyNameU, TRUE))
//                 {
//                     HANDLE ChildKeyHandle;
//
//                     InitializeObjectAttributes(&ObjectAttributes,
//                                                &ChildIdNameU,
//                                                OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
//                                                CriticalDeviceKey,
//                                                NULL);
//
//                     Status = ZwOpenKey(&ChildKeyHandle,
//                                        KEY_QUERY_VALUE,
//                                        &ObjectAttributes);
//                     if (Status != STATUS_SUCCESS)
//                     {
//                         ExFreePool(BasicInfo);
//                         continue;
//                     }
//
//                     /* Check if there's already a driver installed */
//                     Status = ZwQueryValueKey(InstanceKey,
//                                              &ClassGuidU,
//                                              KeyValuePartialInformation,
//                                              NULL,
//                                              0,
//                                              &NeededLength);
//                     if (Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL)
//                     {
//                         ExFreePool(BasicInfo);
//                         continue;
//                     }
//
//                     Status = ZwQueryValueKey(ChildKeyHandle,
//                                              &ClassGuidU,
//                                              KeyValuePartialInformation,
//                                              NULL,
//                                              0,
//                                              &NeededLength);
//                     if (Status != STATUS_BUFFER_OVERFLOW && Status != STATUS_BUFFER_TOO_SMALL)
//                     {
//                         ExFreePool(BasicInfo);
//                         continue;
//                     }
//
//                     PartialInfo = ExAllocatePool(PagedPool, NeededLength);
//                     if (!PartialInfo)
//                     {
//                         ExFreePool(OriginalIdBuffer);
//                         ExFreePool(BasicInfo);
//                         ZwClose(InstanceKey);
//                         ZwClose(ChildKeyHandle);
//                         ZwClose(CriticalDeviceKey);
//                         return;
//                     }
//
//                     /* Read ClassGUID entry in the CDDB */
//                     Status = ZwQueryValueKey(ChildKeyHandle,
//                                              &ClassGuidU,
//                                              KeyValuePartialInformation,
//                                              PartialInfo,
//                                              NeededLength,
//                                              &NeededLength);
//                     if (Status != STATUS_SUCCESS)
//                     {
//                         ExFreePool(BasicInfo);
//                         continue;
//                     }
//
//                     /* Write it to the ENUM key */
//                     Status = ZwSetValueKey(InstanceKey,
//                                            &ClassGuidU,
//                                            0,
//                                            REG_SZ,
//                                            PartialInfo->Data,
//                                            PartialInfo->DataLength);
//                     if (Status != STATUS_SUCCESS)
//                     {
//                         ExFreePool(BasicInfo);
//                         ExFreePool(PartialInfo);
//                         ZwClose(ChildKeyHandle);
//                         continue;
//                     }
//
//                     Status = ZwQueryValueKey(ChildKeyHandle,
//                                              &ServiceU,
//                                              KeyValuePartialInformation,
//                                              NULL,
//                                              0,
//                                              &NeededLength);
//                     if (Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL)
//                     {
//                         ExFreePool(PartialInfo);
//                         PartialInfo = ExAllocatePool(PagedPool, NeededLength);
//                         if (!PartialInfo)
//                         {
//                             ExFreePool(OriginalIdBuffer);
//                             ExFreePool(BasicInfo);
//                             ZwClose(InstanceKey);
//                             ZwClose(ChildKeyHandle);
//                             ZwClose(CriticalDeviceKey);
//                             return;
//                         }
//
//                         /* Read the service entry from the CDDB */
//                         Status = ZwQueryValueKey(ChildKeyHandle,
//                                                  &ServiceU,
//                                                  KeyValuePartialInformation,
//                                                  PartialInfo,
//                                                  NeededLength,
//                                                  &NeededLength);
//                         if (Status != STATUS_SUCCESS)
//                         {
//                             ExFreePool(BasicInfo);
//                             ExFreePool(PartialInfo);
//                             ZwClose(ChildKeyHandle);
//                             continue;
//                         }
//
//                         /* Write it to the ENUM key */
//                         Status = ZwSetValueKey(InstanceKey,
//                                                &ServiceU,
//                                                0,
//                                                REG_SZ,
//                                                PartialInfo->Data,
//                                                PartialInfo->DataLength);
//                         if (Status != STATUS_SUCCESS)
//                         {
//                             ExFreePool(BasicInfo);
//                             ExFreePool(PartialInfo);
//                             ZwClose(ChildKeyHandle);
//                             continue;
//                         }
//
//                         DPRINT("Installed service '%S' for critical device '%wZ'\n", PartialInfo->Data, &ChildIdNameU);
//                     }
//                     else
//                     {
//                         DPRINT1("Installed NULL service for critical device '%wZ'\n", &ChildIdNameU);
//                     }
//
//                     ExFreePool(OriginalIdBuffer);
//                     ExFreePool(PartialInfo);
//                     ExFreePool(BasicInfo);
//                     ZwClose(InstanceKey);
//                     ZwClose(ChildKeyHandle);
//                     ZwClose(CriticalDeviceKey);
//
//                     /* That's it */
//                     return;
//                 }
//
//                 ExFreePool(BasicInfo);
//             }
//             else
//             {
//                 /* Umm, not sure what happened here */
//                 continue;
//             }
//         }
//
//         /* Advance to the next ID */
//         IdBuffer += StringLength;
//     }
//
//     ExFreePool(OriginalIdBuffer);
//     ZwClose(InstanceKey);
//     ZwClose(CriticalDeviceKey);
// }
//
// NTSTATUS
// IopGetSystemPowerDeviceObject(PDEVICE_OBJECT *DeviceObject)
// {
//     KIRQL OldIrql;
//
//     if (PopSystemPowerDeviceNode)
//     {
//         KeAcquireSpinLock(&IopDeviceTreeLock, &OldIrql);
//         *DeviceObject = PopSystemPowerDeviceNode->PhysicalDeviceObject;
//         KeReleaseSpinLock(&IopDeviceTreeLock, OldIrql);
//
//         return STATUS_SUCCESS;
//     }
//
//     return STATUS_UNSUCCESSFUL;
// }
//
// USHORT
// NTAPI
// IopGetBusTypeGuidIndex(LPGUID BusTypeGuid)
// {
//     USHORT i = 0, FoundIndex = 0xFFFF;
//     ULONG NewSize;
//     PVOID NewList;
//
//     /* Acquire the lock */
//     ExAcquireFastMutex(&PnpBusTypeGuidList->Lock);
//
//     /* Loop all entries */
//     while (i < PnpBusTypeGuidList->GuidCount)
//     {
//          /* Try to find a match */
//          if (RtlCompareMemory(BusTypeGuid,
//                               &PnpBusTypeGuidList->Guids[i],
//                               sizeof(GUID)) == sizeof(GUID))
//          {
//               /* Found it */
//               FoundIndex = i;
//               goto Quickie;
//          }
//          i++;
//     }
//
//     /* Check if we have to grow the list */
//     if (PnpBusTypeGuidList->GuidCount)
//     {
//         /* Calculate the new size */
//         NewSize = sizeof(IO_BUS_TYPE_GUID_LIST) +
//                  (sizeof(GUID) * PnpBusTypeGuidList->GuidCount);
//
//         /* Allocate the new copy */
//         NewList = ExAllocatePool(PagedPool, NewSize);
//
//         if (!NewList)
//         {
//             /* Fail */
//             ExFreePool(PnpBusTypeGuidList);
//             goto Quickie;
//         }
//
//         /* Now copy them, decrease the size too */
//         NewSize -= sizeof(GUID);
//         RtlCopyMemory(NewList, PnpBusTypeGuidList, NewSize);
//
//         /* Free the old list */
//         ExFreePool(PnpBusTypeGuidList);
//
//         /* Use the new buffer */
//         PnpBusTypeGuidList = NewList;
//     }
//
//     /* Copy the new GUID */
//     RtlCopyMemory(&PnpBusTypeGuidList->Guids[PnpBusTypeGuidList->GuidCount],
//                   BusTypeGuid,
//                   sizeof(GUID));
//
//     /* The new entry is the index */
//     FoundIndex = (USHORT)PnpBusTypeGuidList->GuidCount;
//     PnpBusTypeGuidList->GuidCount++;
//
// Quickie:
//     ExReleaseFastMutex(&PnpBusTypeGuidList->Lock);
//     return FoundIndex;
// }
//
// NTSTATUS
// NTAPI
// IopInitiatePnpIrp(IN PDEVICE_OBJECT DeviceObject,
//                   IN OUT PIO_STATUS_BLOCK IoStatusBlock,
//                   IN UCHAR MinorFunction,
//                   IN PIO_STACK_LOCATION Stack OPTIONAL)
// {
//     IO_STACK_LOCATION IoStackLocation;
//
//     /* Fill out the stack information */
//     RtlZeroMemory(&IoStackLocation, sizeof(IO_STACK_LOCATION));
//     IoStackLocation.MajorFunction = IRP_MJ_PNP;
//     IoStackLocation.MinorFunction = MinorFunction;
//     if (Stack)
//     {
//         /* Copy the rest */
//         RtlCopyMemory(&IoStackLocation.Parameters,
//                       &Stack->Parameters,
//                       sizeof(Stack->Parameters));
//     }
//
//     /* Do the PnP call */
//     IoStatusBlock->Status = IopSynchronousCall(DeviceObject,
//                                                &IoStackLocation,
//                                                (PVOID)&IoStatusBlock->Information);
//     return IoStatusBlock->Status;
// }
//
// /*
//  * IopCreateDeviceKeyPath
//  *
//  * Creates a registry key
//  *
//  * Parameters
//  *    RegistryPath
//  *        Name of the key to be created.
//  *    Handle
//  *        Handle to the newly created key
//  *
//  * Remarks
//  *     This method can create nested trees, so parent of RegistryPath can
//  *     be not existant, and will be created if needed.
//  */
// NTSTATUS
// NTAPI
// IopCreateDeviceKeyPath(IN PCUNICODE_STRING RegistryPath,
//                        IN ULONG CreateOptions,
//                        OUT PHANDLE Handle)
// {
//     UNICODE_STRING EnumU = RTL_CONSTANT_STRING(ENUM_ROOT);
//     HANDLE hParent = NULL, hKey;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING KeyName;
//     PCWSTR Current, Last;
//     USHORT Length;
//     NTSTATUS Status;
//
//     /* Assume failure */
//     *Handle = NULL;
//
//     /* Open root key for device instances */
//     Status = IopOpenRegistryKeyEx(&hParent, NULL, &EnumU, KEY_CREATE_SUB_KEY);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwOpenKey('%wZ') failed with status 0x%08lx\n", &EnumU, Status);
//         return Status;
//     }
//
//     Current = KeyName.Buffer = RegistryPath->Buffer;
//     Last = &RegistryPath->Buffer[RegistryPath->Length / sizeof(WCHAR)];
//
//     /* Go up to the end of the string */
//     while (Current <= Last)
//     {
//         if (Current != Last && *Current != L'\\')
//         {
//             /* Not the end of the string and not a separator */
//             Current++;
//             continue;
//         }
//
//         /* Prepare relative key name */
//         Length = (USHORT)((ULONG_PTR)Current - (ULONG_PTR)KeyName.Buffer);
//         KeyName.MaximumLength = KeyName.Length = Length;
//         DPRINT("Create '%wZ'\n", &KeyName);
//
//         /* Open key */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &KeyName,
//                                    OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                    hParent,
//                                    NULL);
//         Status = ZwCreateKey(&hKey,
//                              Current == Last ? KEY_ALL_ACCESS : KEY_CREATE_SUB_KEY,
//                              &ObjectAttributes,
//                              0,
//                              NULL,
//                              CreateOptions,
//                              NULL);
//
//         /* Close parent key handle, we don't need it anymore */
//         if (hParent)
//             ZwClose(hParent);
//
//         /* Key opening/creating failed? */
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("ZwCreateKey('%wZ') failed with status 0x%08lx\n", &KeyName, Status);
//             return Status;
//         }
//
//         /* Check if it is the end of the string */
//         if (Current == Last)
//         {
//             /* Yes, return success */
//             *Handle = hKey;
//             return STATUS_SUCCESS;
//         }
//
//         /* Start with this new parent key */
//         hParent = hKey;
//         Current++;
//         KeyName.Buffer = (PWSTR)Current;
//     }
//
//     return STATUS_UNSUCCESSFUL;
// }
//
// NTSTATUS
// IopSetDeviceInstanceData(HANDLE InstanceKey,
//                          PDEVICE_NODE DeviceNode)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING KeyName;
//     HANDLE LogConfKey, ControlKey, DeviceParamsKey;
//     ULONG ResCount;
//     ULONG ResultLength;
//     NTSTATUS Status;
//
//     DPRINT("IopSetDeviceInstanceData() called\n");
//
//     /* Create the 'LogConf' key */
//     RtlInitUnicodeString(&KeyName, L"LogConf");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                InstanceKey,
//                                NULL);
//     Status = ZwCreateKey(&LogConfKey,
//                          KEY_ALL_ACCESS,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          // FIXME? In r53694 it was silently turned from non-volatile into this,
//                          // without any extra warning. Is this still needed??
//                          REG_OPTION_VOLATILE,
//                          NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Set 'BootConfig' value */
//         if (DeviceNode->BootResources != NULL)
//         {
//             ResCount = DeviceNode->BootResources->Count;
//             if (ResCount != 0)
//             {
//                 RtlInitUnicodeString(&KeyName, L"BootConfig");
//                 Status = ZwSetValueKey(LogConfKey,
//                                        &KeyName,
//                                        0,
//                                        REG_RESOURCE_LIST,
//                                        DeviceNode->BootResources,
//                                        PnpDetermineResourceListSize(DeviceNode->BootResources));
//             }
//         }
//
//         /* Set 'BasicConfigVector' value */
//         if (DeviceNode->ResourceRequirements != NULL &&
//             DeviceNode->ResourceRequirements->ListSize != 0)
//         {
//             RtlInitUnicodeString(&KeyName, L"BasicConfigVector");
//             Status = ZwSetValueKey(LogConfKey,
//                                    &KeyName,
//                                    0,
//                                    REG_RESOURCE_REQUIREMENTS_LIST,
//                                    DeviceNode->ResourceRequirements,
//                                    DeviceNode->ResourceRequirements->ListSize);
//         }
//
//         ZwClose(LogConfKey);
//     }
//
//     /* Set the 'ConfigFlags' value */
//     RtlInitUnicodeString(&KeyName, L"ConfigFlags");
//     Status = ZwQueryValueKey(InstanceKey,
//                              &KeyName,
//                              KeyValueBasicInformation,
//                              NULL,
//                              0,
//                              &ResultLength);
//     if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//     {
//         /* Write the default value */
//         ULONG DefaultConfigFlags = 0;
//         Status = ZwSetValueKey(InstanceKey,
//                                &KeyName,
//                                0,
//                                REG_DWORD,
//                                &DefaultConfigFlags,
//                                sizeof(DefaultConfigFlags));
//     }
//
//     /* Create the 'Control' key */
//     RtlInitUnicodeString(&KeyName, L"Control");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                InstanceKey,
//                                NULL);
//     Status = ZwCreateKey(&ControlKey,
//                          0,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_VOLATILE,
//                          NULL);
//     if (NT_SUCCESS(Status))
//         ZwClose(ControlKey);
//
//     /* Create the 'Device Parameters' key and set the 'FirmwareIdentified' value for all ACPI-enumerated devices */
//     if (_wcsnicmp(DeviceNode->InstancePath.Buffer, L"ACPI\\", 5) == 0)
//     {
//         RtlInitUnicodeString(&KeyName, L"Device Parameters");
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &KeyName,
//                                    OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                    InstanceKey,
//                                    NULL);
//         Status = ZwCreateKey(&DeviceParamsKey,
//                              0,
//                              &ObjectAttributes,
//                              0,
//                              NULL,
//                              REG_OPTION_NON_VOLATILE,
//                              NULL);
//         if (NT_SUCCESS(Status))
//         {
//             ULONG FirmwareIdentified = 1;
//             RtlInitUnicodeString(&KeyName, L"FirmwareIdentified");
//             Status = ZwSetValueKey(DeviceParamsKey,
//                                    &KeyName,
//                                    0,
//                                    REG_DWORD,
//                                    &FirmwareIdentified,
//                                    sizeof(FirmwareIdentified));
//
//             ZwClose(DeviceParamsKey);
//         }
//     }
//
//     DPRINT("IopSetDeviceInstanceData() done\n");
//
//     return Status;
// }
//
// /*
//  * IopGetParentIdPrefix
//  *
//  * Retrieve (or create) a string which identifies a device.
//  *
//  * Parameters
//  *    DeviceNode
//  *        Pointer to device node.
//  *    ParentIdPrefix
//  *        Pointer to the string where is returned the parent node identifier
//  *
//  * Remarks
//  *     If the return code is STATUS_SUCCESS, the ParentIdPrefix string is
//  *     valid and its Buffer field is NULL-terminated. The caller needs to
//  *     to free the string with RtlFreeUnicodeString when it is no longer
//  *     needed.
//  */
//
// NTSTATUS
// IopGetParentIdPrefix(PDEVICE_NODE DeviceNode,
//                      PUNICODE_STRING ParentIdPrefix)
// {
//     const UNICODE_STRING EnumKeyPath = RTL_CONSTANT_STRING(L"\\Registry\\Machine\\System\\CurrentControlSet\\Enum\\");
//     ULONG KeyNameBufferLength;
//     PKEY_VALUE_PARTIAL_INFORMATION ParentIdPrefixInformation = NULL;
//     UNICODE_STRING KeyName = {0, 0, NULL};
//     UNICODE_STRING KeyValue;
//     UNICODE_STRING ValueName;
//     HANDLE hKey = NULL;
//     ULONG crc32;
//     NTSTATUS Status;
//
//     /* HACK: As long as some devices have a NULL device
//      * instance path, the following test is required :(
//      */
//     if (DeviceNode->Parent->InstancePath.Length == 0)
//     {
//         DPRINT1("Parent of %wZ has NULL Instance path, please report!\n",
//                 &DeviceNode->InstancePath);
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     /* 1. Try to retrieve ParentIdPrefix from registry */
//     KeyNameBufferLength = FIELD_OFFSET(KEY_VALUE_PARTIAL_INFORMATION, Data) + sizeof(L"12345678&12345678");
//     ParentIdPrefixInformation = ExAllocatePoolWithTag(PagedPool,
//                                                       KeyNameBufferLength + sizeof(UNICODE_NULL),
//                                                       TAG_IO);
//     if (!ParentIdPrefixInformation)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     KeyName.Length = 0;
//     KeyName.MaximumLength = EnumKeyPath.Length +
//                             DeviceNode->Parent->InstancePath.Length +
//                             sizeof(UNICODE_NULL);
//     KeyName.Buffer = ExAllocatePoolWithTag(PagedPool,
//                                            KeyName.MaximumLength,
//                                            TAG_IO);
//     if (!KeyName.Buffer)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto cleanup;
//     }
//
//     RtlCopyUnicodeString(&KeyName, &EnumKeyPath);
//     RtlAppendUnicodeStringToString(&KeyName, &DeviceNode->Parent->InstancePath);
//
//     Status = IopOpenRegistryKeyEx(&hKey, NULL, &KeyName, KEY_QUERY_VALUE | KEY_SET_VALUE);
//     if (!NT_SUCCESS(Status))
//     {
//         goto cleanup;
//     }
//     RtlInitUnicodeString(&ValueName, L"ParentIdPrefix");
//     Status = ZwQueryValueKey(hKey,
//                              &ValueName,
//                              KeyValuePartialInformation,
//                              ParentIdPrefixInformation,
//                              KeyNameBufferLength,
//                              &KeyNameBufferLength);
//     if (NT_SUCCESS(Status))
//     {
//         if (ParentIdPrefixInformation->Type != REG_SZ)
//         {
//             Status = STATUS_UNSUCCESSFUL;
//         }
//         else
//         {
//             KeyValue.MaximumLength = (USHORT)ParentIdPrefixInformation->DataLength;
//             KeyValue.Length = KeyValue.MaximumLength - sizeof(UNICODE_NULL);
//             KeyValue.Buffer = (PWSTR)ParentIdPrefixInformation->Data;
//             ASSERT(KeyValue.Buffer[KeyValue.Length / sizeof(WCHAR)] == UNICODE_NULL);
//         }
//         goto cleanup;
//     }
//     if (Status != STATUS_OBJECT_NAME_NOT_FOUND)
//     {
//         /* FIXME how do we get here and why is ParentIdPrefixInformation valid? */
//         KeyValue.MaximumLength = (USHORT)ParentIdPrefixInformation->DataLength;
//         KeyValue.Length = KeyValue.MaximumLength - sizeof(UNICODE_NULL);
//         KeyValue.Buffer = (PWSTR)ParentIdPrefixInformation->Data;
//         ASSERT(KeyValue.Buffer[KeyValue.Length / sizeof(WCHAR)] == UNICODE_NULL);
//         goto cleanup;
//     }
//
//     /* 2. Create the ParentIdPrefix value */
//     crc32 = RtlComputeCrc32(0,
//                             (PUCHAR)DeviceNode->Parent->InstancePath.Buffer,
//                             DeviceNode->Parent->InstancePath.Length);
//
//     RtlStringCbPrintfW((PWSTR)ParentIdPrefixInformation,
//                        KeyNameBufferLength,
//                        L"%lx&%lx",
//                        DeviceNode->Parent->Level,
//                        crc32);
//     RtlInitUnicodeString(&KeyValue, (PWSTR)ParentIdPrefixInformation);
//
//     /* 3. Try to write the ParentIdPrefix to registry */
//     Status = ZwSetValueKey(hKey,
//                            &ValueName,
//                            0,
//                            REG_SZ,
//                            KeyValue.Buffer,
//                            ((ULONG)wcslen(KeyValue.Buffer) + 1) * sizeof(WCHAR));
//
// cleanup:
//     if (NT_SUCCESS(Status))
//     {
//         /* Duplicate the string to return it */
//         Status = RtlDuplicateUnicodeString(RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
//                                            &KeyValue,
//                                            ParentIdPrefix);
//     }
//     ExFreePoolWithTag(ParentIdPrefixInformation, TAG_IO);
//     RtlFreeUnicodeString(&KeyName);
//     if (hKey != NULL)
//     {
//         ZwClose(hKey);
//     }
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IopOpenRegistryKeyEx(PHANDLE KeyHandle,
//                      HANDLE ParentKey,
//                      PUNICODE_STRING Name,
//                      ACCESS_MASK DesiredAccess)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     *KeyHandle = NULL;
//
//     InitializeObjectAttributes(&ObjectAttributes,
//         Name,
//         OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//         ParentKey,
//         NULL);
//
//     Status = ZwOpenKey(KeyHandle, DesiredAccess, &ObjectAttributes);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IopCreateRegistryKeyEx(OUT PHANDLE Handle,
//                        IN HANDLE RootHandle OPTIONAL,
//                        IN PUNICODE_STRING KeyName,
//                        IN ACCESS_MASK DesiredAccess,
//                        IN ULONG CreateOptions,
//                        OUT PULONG Disposition OPTIONAL)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     ULONG KeyDisposition, RootHandleIndex = 0, i = 1, NestedCloseLevel = 0;
//     USHORT Length;
//     HANDLE HandleArray[2];
//     BOOLEAN Recursing = TRUE;
//     PWCHAR pp, p, p1;
//     UNICODE_STRING KeyString;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PAGED_CODE();
//
//     /* P1 is start, pp is end */
//     p1 = KeyName->Buffer;
//     pp = (PVOID)((ULONG_PTR)p1 + KeyName->Length);
//
//     /* Create the target key */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                KeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                RootHandle,
//                                NULL);
//     Status = ZwCreateKey(&HandleArray[i],
//                          DesiredAccess,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          CreateOptions,
//                          &KeyDisposition);
//
//     /* Now we check if this failed */
//     if ((Status == STATUS_OBJECT_NAME_NOT_FOUND) && (RootHandle))
//     {
//         /* Target key failed, so we'll need to create its parent. Setup array */
//         HandleArray[0] = NULL;
//         HandleArray[1] = RootHandle;
//
//         /* Keep recursing for each missing parent */
//         while (Recursing)
//         {
//             /* And if we're deep enough, close the last handle */
//             if (NestedCloseLevel > 1) ZwClose(HandleArray[RootHandleIndex]);
//
//             /* We're setup to ping-pong between the two handle array entries */
//             RootHandleIndex = i;
//             i = (i + 1) & 1;
//
//             /* Clear the one we're attempting to open now */
//             HandleArray[i] = NULL;
//
//             /* Process the parent key name */
//             for (p = p1; ((p < pp) && (*p != OBJ_NAME_PATH_SEPARATOR)); p++);
//             Length = (USHORT)(p - p1) * sizeof(WCHAR);
//
//             /* Is there a parent name? */
//             if (Length)
//             {
//                 /* Build the unicode string for it */
//                 KeyString.Buffer = p1;
//                 KeyString.Length = KeyString.MaximumLength = Length;
//
//                 /* Now try opening the parent */
//                 InitializeObjectAttributes(&ObjectAttributes,
//                                            &KeyString,
//                                            OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                            HandleArray[RootHandleIndex],
//                                            NULL);
//                 Status = ZwCreateKey(&HandleArray[i],
//                                      DesiredAccess,
//                                      &ObjectAttributes,
//                                      0,
//                                      NULL,
//                                      CreateOptions,
//                                      &KeyDisposition);
//                 if (NT_SUCCESS(Status))
//                 {
//                     /* It worked, we have one more handle */
//                     NestedCloseLevel++;
//                 }
//                 else
//                 {
//                     /* Parent key creation failed, abandon loop */
//                     Recursing = FALSE;
//                     continue;
//                 }
//             }
//             else
//             {
//                 /* We don't have a parent name, probably corrupted key name */
//                 Status = STATUS_INVALID_PARAMETER;
//                 Recursing = FALSE;
//                 continue;
//             }
//
//             /* Now see if there's more parents to create */
//             p1 = p + 1;
//             if ((p == pp) || (p1 == pp))
//             {
//                 /* We're done, hopefully successfully, so stop */
//                 Recursing = FALSE;
//             }
//         }
//
//         /* Outer loop check for handle nesting that requires closing the top handle */
//         if (NestedCloseLevel > 1) ZwClose(HandleArray[RootHandleIndex]);
//     }
//
//     /* Check if we broke out of the loop due to success */
//     if (NT_SUCCESS(Status))
//     {
//         /* Return the target handle (we closed all the parent ones) and disposition */
//         *Handle = HandleArray[i];
//         if (Disposition) *Disposition = KeyDisposition;
//     }
//
//     /* Return the success state */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IopGetRegistryValue(IN HANDLE Handle,
//                     IN PWSTR ValueName,
//                     OUT PKEY_VALUE_FULL_INFORMATION *Information)
// {
//     UNICODE_STRING ValueString;
//     NTSTATUS Status;
//     PKEY_VALUE_FULL_INFORMATION FullInformation;
//     ULONG Size;
//     PAGED_CODE();
//
//     RtlInitUnicodeString(&ValueString, ValueName);
//
//     Status = ZwQueryValueKey(Handle,
//                              &ValueString,
//                              KeyValueFullInformation,
//                              NULL,
//                              0,
//                              &Size);
//     if ((Status != STATUS_BUFFER_OVERFLOW) &&
//         (Status != STATUS_BUFFER_TOO_SMALL))
//     {
//         return Status;
//     }
//
//     FullInformation = ExAllocatePool(NonPagedPool, Size);
//     if (!FullInformation) return STATUS_INSUFFICIENT_RESOURCES;
//
//     Status = ZwQueryValueKey(Handle,
//                              &ValueString,
//                              KeyValueFullInformation,
//                              FullInformation,
//                              Size,
//                              &Size);
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePool(FullInformation);
//         return Status;
//     }
//
//     *Information = FullInformation;
//     return STATUS_SUCCESS;
// }
//
// RTL_GENERIC_COMPARE_RESULTS
// NTAPI
// PiCompareInstancePath(IN PRTL_AVL_TABLE Table,
//                       IN PVOID FirstStruct,
//                       IN PVOID SecondStruct)
// {
//     /* FIXME: TODO */
//     ASSERT(FALSE);
//     return 0;
// }
//
// //
// //  The allocation function is called by the generic table package whenever
// //  it needs to allocate memory for the table.
// //
//
// PVOID
// NTAPI
// PiAllocateGenericTableEntry(IN PRTL_AVL_TABLE Table,
//                             IN CLONG ByteSize)
// {
//     /* FIXME: TODO */
//     ASSERT(FALSE);
//     return NULL;
// }
//
// VOID
// NTAPI
// PiFreeGenericTableEntry(IN PRTL_AVL_TABLE Table,
//                         IN PVOID Buffer)
// {
//     /* FIXME: TODO */
//     ASSERT(FALSE);
// }
//
// VOID
// NTAPI
// PpInitializeDeviceReferenceTable(VOID)
// {
//     /* Setup the guarded mutex and AVL table */
//     KeInitializeGuardedMutex(&PpDeviceReferenceTableLock);
//     RtlInitializeGenericTableAvl(
//         &PpDeviceReferenceTable,
//         (PRTL_AVL_COMPARE_ROUTINE)PiCompareInstancePath,
//         (PRTL_AVL_ALLOCATE_ROUTINE)PiAllocateGenericTableEntry,
//         (PRTL_AVL_FREE_ROUTINE)PiFreeGenericTableEntry,
//         NULL);
// }
//
// BOOLEAN
// NTAPI
// PiInitPhase0(VOID)
// {
//     /* Initialize the resource when accessing device registry data */
//     ExInitializeResourceLite(&PpRegistryDeviceResource);
//
//     /* Setup the device reference AVL table */
//     PpInitializeDeviceReferenceTable();
//     return TRUE;
// }
//
// BOOLEAN
// NTAPI
// PpInitSystem(VOID)
// {
//     /* Check the initialization phase */
//     switch (ExpInitializationPhase)
//     {
//     case 0:
//
//         /* Do Phase 0 */
//         return PiInitPhase0();
//
//     case 1:
//
//         /* Do Phase 1 */
//         return TRUE;
//         //return PiInitPhase1();
//
//     default:
//
//         /* Don't know any other phase! Bugcheck! */
//         KeBugCheck(UNEXPECTED_INITIALIZATION_CALL);
//         return FALSE;
//     }
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// NTSTATUS
// NTAPI
// PnpBusTypeGuidGet(IN USHORT Index,
//                   IN LPGUID BusTypeGuid)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* Acquire the lock */
//     ExAcquireFastMutex(&PnpBusTypeGuidList->Lock);
//
//     /* Validate size */
//     if (Index < PnpBusTypeGuidList->GuidCount)
//     {
//         /* Copy the data */
//         RtlCopyMemory(BusTypeGuid, &PnpBusTypeGuidList->Guids[Index], sizeof(GUID));
//     }
//     else
//     {
//         /* Failure path */
//         Status = STATUS_OBJECT_NAME_NOT_FOUND;
//     }
//
//     /* Release lock and return status */
//     ExReleaseFastMutex(&PnpBusTypeGuidList->Lock);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// PnpDeviceObjectToDeviceInstance(IN PDEVICE_OBJECT DeviceObject,
//                                 IN PHANDLE DeviceInstanceHandle,
//                                 IN ACCESS_MASK DesiredAccess)
// {
//     NTSTATUS Status;
//     HANDLE KeyHandle;
//     PDEVICE_NODE DeviceNode;
//     UNICODE_STRING KeyName = RTL_CONSTANT_STRING(L"\\REGISTRY\\MACHINE\\SYSTEM\\CURRENTCONTROLSET\\ENUM");
//     PAGED_CODE();
//
//     /* Open the enum key */
//     Status = IopOpenRegistryKeyEx(&KeyHandle,
//                                   NULL,
//                                   &KeyName,
//                                   KEY_READ);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Make sure we have an instance path */
//     DeviceNode = IopGetDeviceNode(DeviceObject);
//     if ((DeviceNode) && (DeviceNode->InstancePath.Length))
//     {
//         /* Get the instance key */
//         Status = IopOpenRegistryKeyEx(DeviceInstanceHandle,
//                                       KeyHandle,
//                                       &DeviceNode->InstancePath,
//                                       DesiredAccess);
//     }
//     else
//     {
//         /* Fail */
//         Status = STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     /* Close the handle and return status */
//     ZwClose(KeyHandle);
//     return Status;
// }
//
// ULONG
// NTAPI
// PnpDetermineResourceListSize(IN PCM_RESOURCE_LIST ResourceList)
// {
//     ULONG FinalSize, PartialSize, EntrySize, i, j;
//     PCM_FULL_RESOURCE_DESCRIPTOR FullDescriptor;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;
//
//     /* If we don't have one, that's easy */
//     if (!ResourceList) return 0;
//
//     /* Start with the minimum size possible */
//     FinalSize = FIELD_OFFSET(CM_RESOURCE_LIST, List);
//
//     /* Loop each full descriptor */
//     FullDescriptor = ResourceList->List;
//     for (i = 0; i < ResourceList->Count; i++)
//     {
//         /* Start with the minimum size possible */
//         PartialSize = FIELD_OFFSET(CM_FULL_RESOURCE_DESCRIPTOR, PartialResourceList) +
//         FIELD_OFFSET(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors);
//
//         /* Loop each partial descriptor */
//         PartialDescriptor = FullDescriptor->PartialResourceList.PartialDescriptors;
//         for (j = 0; j < FullDescriptor->PartialResourceList.Count; j++)
//         {
//             /* Start with the minimum size possible */
//             EntrySize = sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR);
//
//             /* Check if there is extra data */
//             if (PartialDescriptor->Type == CmResourceTypeDeviceSpecific)
//             {
//                 /* Add that data */
//                 EntrySize += PartialDescriptor->u.DeviceSpecificData.DataSize;
//             }
//
//             /* The size of partial descriptors is bigger */
//             PartialSize += EntrySize;
//
//             /* Go to the next partial descriptor */
//             PartialDescriptor = (PVOID)((ULONG_PTR)PartialDescriptor + EntrySize);
//         }
//
//         /* The size of full descriptors is bigger */
//         FinalSize += PartialSize;
//
//         /* Go to the next full descriptor */
//         FullDescriptor = (PVOID)((ULONG_PTR)FullDescriptor + PartialSize);
//     }
//
//     /* Return the final size */
//     return FinalSize;
// }
//
// NTSTATUS
// NTAPI
// PiGetDeviceRegistryProperty(IN PDEVICE_OBJECT DeviceObject,
//                             IN ULONG ValueType,
//                             IN PWSTR ValueName,
//                             IN PWSTR KeyName,
//                             OUT PVOID Buffer,
//                             IN PULONG BufferLength)
// {
//     NTSTATUS Status;
//     HANDLE KeyHandle, SubHandle;
//     UNICODE_STRING KeyString;
//     PKEY_VALUE_FULL_INFORMATION KeyValueInfo = NULL;
//     ULONG Length;
//     PAGED_CODE();
//
//     /* Find the instance key */
//     Status = PnpDeviceObjectToDeviceInstance(DeviceObject, &KeyHandle, KEY_READ);
//     if (NT_SUCCESS(Status))
//     {
//         /* Check for name given by caller */
//         if (KeyName)
//         {
//             /* Open this key */
//             RtlInitUnicodeString(&KeyString, KeyName);
//             Status = IopOpenRegistryKeyEx(&SubHandle,
//                                           KeyHandle,
//                                           &KeyString,
//                                           KEY_READ);
//             if (NT_SUCCESS(Status))
//             {
//                 /* And use this handle instead */
//                 ZwClose(KeyHandle);
//                 KeyHandle = SubHandle;
//             }
//         }
//
//         /* Check if sub-key handle succeeded (or no-op if no key name given) */
//         if (NT_SUCCESS(Status))
//         {
//             /* Now get the size of the property */
//             Status = IopGetRegistryValue(KeyHandle,
//                                          ValueName,
//                                          &KeyValueInfo);
//         }
//
//         /* Close the key */
//         ZwClose(KeyHandle);
//     }
//
//     /* Fail if any of the registry operations failed */
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Check how much data we have to copy */
//     Length = KeyValueInfo->DataLength;
//     if (*BufferLength >= Length)
//     {
//         /* Check for a match in the value type */
//         if (KeyValueInfo->Type == ValueType)
//         {
//             /* Copy the data */
//             RtlCopyMemory(Buffer,
//                           (PVOID)((ULONG_PTR)KeyValueInfo +
//                           KeyValueInfo->DataOffset),
//                           Length);
//         }
//         else
//         {
//             /* Invalid registry property type, fail */
//            Status = STATUS_INVALID_PARAMETER_2;
//         }
//     }
//     else
//     {
//         /* Buffer is too small to hold data */
//         Status = STATUS_BUFFER_TOO_SMALL;
//     }
//
//     /* Return the required buffer length, free the buffer, and return status */
//     *BufferLength = Length;
//     ExFreePool(KeyValueInfo);
//     return Status;
// }
//
// #define PIP_RETURN_DATA(x, y)   {ReturnLength = x; Data = y; Status = STATUS_SUCCESS; break;}
// #define PIP_REGISTRY_DATA(x, y) {ValueName = x; ValueType = y; break;}
// #define PIP_UNIMPLEMENTED()     {UNIMPLEMENTED_DBGBREAK(); break;}
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoGetDeviceProperty(IN PDEVICE_OBJECT DeviceObject,
//                     IN DEVICE_REGISTRY_PROPERTY DeviceProperty,
//                     IN ULONG BufferLength,
//                     OUT PVOID PropertyBuffer,
//                     OUT PULONG ResultLength)
// {
//     PDEVICE_NODE DeviceNode = IopGetDeviceNode(DeviceObject);
//     DEVICE_CAPABILITIES DeviceCaps;
//     ULONG ReturnLength = 0, Length = 0, ValueType;
//     PWCHAR ValueName = NULL, EnumeratorNameEnd, DeviceInstanceName;
//     PVOID Data = NULL;
//     NTSTATUS Status = STATUS_BUFFER_TOO_SMALL;
//     GUID BusTypeGuid;
//     POBJECT_NAME_INFORMATION ObjectNameInfo = NULL;
//     BOOLEAN NullTerminate = FALSE;
//     DEVICE_REMOVAL_POLICY Policy;
//
//     DPRINT("IoGetDeviceProperty(0x%p %d)\n", DeviceObject, DeviceProperty);
//
//     /* Assume failure */
//     *ResultLength = 0;
//
//     /* Only PDOs can call this */
//     if (!DeviceNode) return STATUS_INVALID_DEVICE_REQUEST;
//
//     /* Handle all properties */
//     switch (DeviceProperty)
//     {
//         case DevicePropertyBusTypeGuid:
//
//             /* Get the GUID from the internal cache */
//             Status = PnpBusTypeGuidGet(DeviceNode->ChildBusTypeIndex, &BusTypeGuid);
//             if (!NT_SUCCESS(Status)) return Status;
//
//             /* This is the format of the returned data */
//             PIP_RETURN_DATA(sizeof(GUID), &BusTypeGuid);
//
//         case DevicePropertyLegacyBusType:
//
//             /* Validate correct interface type */
//             if (DeviceNode->ChildInterfaceType == InterfaceTypeUndefined)
//                 return STATUS_OBJECT_NAME_NOT_FOUND;
//
//             /* This is the format of the returned data */
//             PIP_RETURN_DATA(sizeof(INTERFACE_TYPE), &DeviceNode->ChildInterfaceType);
//
//         case DevicePropertyBusNumber:
//
//             /* Validate correct bus number */
//             if ((DeviceNode->ChildBusNumber & 0x80000000) == 0x80000000)
//                 return STATUS_OBJECT_NAME_NOT_FOUND;
//
//             /* This is the format of the returned data */
//             PIP_RETURN_DATA(sizeof(ULONG), &DeviceNode->ChildBusNumber);
//
//         case DevicePropertyEnumeratorName:
//
//             /* Get the instance path */
//             DeviceInstanceName = DeviceNode->InstancePath.Buffer;
//
//             /* Sanity checks */
//             ASSERT((BufferLength & 1) == 0);
//             ASSERT(DeviceInstanceName != NULL);
//
//             /* Get the name from the path */
//             EnumeratorNameEnd = wcschr(DeviceInstanceName, OBJ_NAME_PATH_SEPARATOR);
//             ASSERT(EnumeratorNameEnd);
//
//             /* This string needs to be NULL-terminated */
//             NullTerminate = TRUE;
//
//             /* This is the format of the returned data */
//             PIP_RETURN_DATA((ULONG)(EnumeratorNameEnd - DeviceInstanceName) * sizeof(WCHAR),
//                             DeviceInstanceName);
//
//         case DevicePropertyAddress:
//
//             /* Query the device caps */
//             Status = IopQueryDeviceCapabilities(DeviceNode, &DeviceCaps);
//             if (!NT_SUCCESS(Status) || (DeviceCaps.Address == MAXULONG))
//                 return STATUS_OBJECT_NAME_NOT_FOUND;
//
//             /* This is the format of the returned data */
//             PIP_RETURN_DATA(sizeof(ULONG), &DeviceCaps.Address);
//
//         case DevicePropertyBootConfigurationTranslated:
//
//             /* Validate we have resources */
//             if (!DeviceNode->BootResources)
// //            if (!DeviceNode->BootResourcesTranslated) // FIXFIX: Need this field
//             {
//                 /* No resources will still fake success, but with 0 bytes */
//                 *ResultLength = 0;
//                 return STATUS_SUCCESS;
//             }
//
//             /* This is the format of the returned data */
//             PIP_RETURN_DATA(PnpDetermineResourceListSize(DeviceNode->BootResources), // FIXFIX: Should use BootResourcesTranslated
//                             DeviceNode->BootResources); // FIXFIX: Should use BootResourcesTranslated
//
//         case DevicePropertyPhysicalDeviceObjectName:
//
//             /* Sanity check for Unicode-sized string */
//             ASSERT((BufferLength & 1) == 0);
//
//             /* Allocate name buffer */
//             Length = BufferLength + sizeof(OBJECT_NAME_INFORMATION);
//             ObjectNameInfo = ExAllocatePool(PagedPool, Length);
//             if (!ObjectNameInfo) return STATUS_INSUFFICIENT_RESOURCES;
//
//             /* Query the PDO name */
//             Status = ObQueryNameString(DeviceObject,
//                                        ObjectNameInfo,
//                                        Length,
//                                        ResultLength);
//             if (Status == STATUS_INFO_LENGTH_MISMATCH)
//             {
//                 /* It's up to the caller to try again */
//                 Status = STATUS_BUFFER_TOO_SMALL;
//             }
//
//             /* This string needs to be NULL-terminated */
//             NullTerminate = TRUE;
//
//             /* Return if successful */
//             if (NT_SUCCESS(Status)) PIP_RETURN_DATA(ObjectNameInfo->Name.Length,
//                                                     ObjectNameInfo->Name.Buffer);
//
//             /* Let the caller know how big the name is */
//             *ResultLength -= sizeof(OBJECT_NAME_INFORMATION);
//             break;
//
//         case DevicePropertyRemovalPolicy:
//
//             Policy = DeviceNode->RemovalPolicy;
//             PIP_RETURN_DATA(sizeof(Policy), &Policy);
//
//         /* Handle the registry-based properties */
//         case DevicePropertyUINumber:
//             PIP_REGISTRY_DATA(REGSTR_VAL_UI_NUMBER, REG_DWORD);
//         case DevicePropertyLocationInformation:
//             PIP_REGISTRY_DATA(REGSTR_VAL_LOCATION_INFORMATION, REG_SZ);
//         case DevicePropertyDeviceDescription:
//             PIP_REGISTRY_DATA(REGSTR_VAL_DEVDESC, REG_SZ);
//         case DevicePropertyHardwareID:
//             PIP_REGISTRY_DATA(REGSTR_VAL_HARDWAREID, REG_MULTI_SZ);
//         case DevicePropertyCompatibleIDs:
//             PIP_REGISTRY_DATA(REGSTR_VAL_COMPATIBLEIDS, REG_MULTI_SZ);
//         case DevicePropertyBootConfiguration:
//             PIP_REGISTRY_DATA(REGSTR_VAL_BOOTCONFIG, REG_RESOURCE_LIST);
//         case DevicePropertyClassName:
//             PIP_REGISTRY_DATA(REGSTR_VAL_CLASS, REG_SZ);
//         case DevicePropertyClassGuid:
//             PIP_REGISTRY_DATA(REGSTR_VAL_CLASSGUID, REG_SZ);
//         case DevicePropertyDriverKeyName:
//             PIP_REGISTRY_DATA(REGSTR_VAL_DRIVER, REG_SZ);
//         case DevicePropertyManufacturer:
//             PIP_REGISTRY_DATA(REGSTR_VAL_MFG, REG_SZ);
//         case DevicePropertyFriendlyName:
//             PIP_REGISTRY_DATA(REGSTR_VAL_FRIENDLYNAME, REG_SZ);
//         case DevicePropertyContainerID:
//             //PIP_REGISTRY_DATA(REGSTR_VAL_CONTAINERID, REG_SZ); // Win7
//             PIP_UNIMPLEMENTED();
//             break;
//         case DevicePropertyInstallState:
//             PIP_REGISTRY_DATA(REGSTR_VAL_CONFIGFLAGS, REG_DWORD);
//             break;
//         case DevicePropertyResourceRequirements:
//             PIP_UNIMPLEMENTED();
//         case DevicePropertyAllocatedResources:
//             PIP_UNIMPLEMENTED();
//         default:
//             return STATUS_INVALID_PARAMETER_2;
//     }
//
//     /* Having a registry value name implies registry data */
//     if (ValueName)
//     {
//         /* We know up-front how much data to expect */
//         *ResultLength = BufferLength;
//
//         /* Go get the data, use the LogConf subkey if necessary */
//         Status = PiGetDeviceRegistryProperty(DeviceObject,
//                                              ValueType,
//                                              ValueName,
//                                              (DeviceProperty ==
//                                               DevicePropertyBootConfiguration) ?
//                                              L"LogConf":  NULL,
//                                              PropertyBuffer,
//                                              ResultLength);
//     }
//     else if (NT_SUCCESS(Status))
//     {
//         /* We know up-front how much data to expect, check the caller's buffer */
//         *ResultLength = ReturnLength + (NullTerminate ? sizeof(UNICODE_NULL) : 0);
//         if (*ResultLength <= BufferLength)
//         {
//             /* Buffer is all good, copy the data */
//             RtlCopyMemory(PropertyBuffer, Data, ReturnLength);
//
//             /* Check if we need to NULL-terminate the string */
//             if (NullTerminate)
//             {
//                 /* Terminate the string */
//                 ((PWCHAR)PropertyBuffer)[ReturnLength / sizeof(WCHAR)] = UNICODE_NULL;
//             }
//
//             /* This is the success path */
//             Status = STATUS_SUCCESS;
//         }
//         else
//         {
//             /* Failure path */
//             Status = STATUS_BUFFER_TOO_SMALL;
//         }
//     }
//
//     /* Free any allocation we may have made, and return the status code */
//     if (ObjectNameInfo) ExFreePool(ObjectNameInfo);
//     return Status;
// }
//
// /**
//  * @name IoOpenDeviceRegistryKey
//  *
//  * Open a registry key unique for a specified driver or device instance.
//  *
//  * @param DeviceObject   Device to get the registry key for.
//  * @param DevInstKeyType Type of the key to return.
//  * @param DesiredAccess  Access mask (eg. KEY_READ | KEY_WRITE).
//  * @param DevInstRegKey  Handle to the opened registry key on
//  *                       successful return.
//  *
//  * @return Status.
//  *
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoOpenDeviceRegistryKey(IN PDEVICE_OBJECT DeviceObject,
//                         IN ULONG DevInstKeyType,
//                         IN ACCESS_MASK DesiredAccess,
//                         OUT PHANDLE DevInstRegKey)
// {
//     static WCHAR RootKeyName[] =
//         L"\\Registry\\Machine\\System\\CurrentControlSet\\";
//     static WCHAR ProfileKeyName[] =
//         L"Hardware Profiles\\Current\\System\\CurrentControlSet\\";
//     static WCHAR ClassKeyName[] = L"Control\\Class\\";
//     static WCHAR EnumKeyName[] = L"Enum\\";
//     static WCHAR DeviceParametersKeyName[] = L"Device Parameters";
//     ULONG KeyNameLength;
//     PWSTR KeyNameBuffer;
//     UNICODE_STRING KeyName;
//     ULONG DriverKeyLength;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PDEVICE_NODE DeviceNode = NULL;
//     NTSTATUS Status;
//
//     DPRINT("IoOpenDeviceRegistryKey() called\n");
//
//     if ((DevInstKeyType & (PLUGPLAY_REGKEY_DEVICE | PLUGPLAY_REGKEY_DRIVER)) == 0)
//     {
//         DPRINT1("IoOpenDeviceRegistryKey(): got wrong params, exiting...\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if (!IopIsValidPhysicalDeviceObject(DeviceObject))
//         return STATUS_INVALID_DEVICE_REQUEST;
//     DeviceNode = IopGetDeviceNode(DeviceObject);
//
//     /*
//      * Calculate the length of the base key name. This is the full
//      * name for driver key or the name excluding "Device Parameters"
//      * subkey for device key.
//      */
//
//     KeyNameLength = sizeof(RootKeyName);
//     if (DevInstKeyType & PLUGPLAY_REGKEY_CURRENT_HWPROFILE)
//         KeyNameLength += sizeof(ProfileKeyName) - sizeof(UNICODE_NULL);
//     if (DevInstKeyType & PLUGPLAY_REGKEY_DRIVER)
//     {
//         KeyNameLength += sizeof(ClassKeyName) - sizeof(UNICODE_NULL);
//         Status = IoGetDeviceProperty(DeviceObject, DevicePropertyDriverKeyName,
//                                      0, NULL, &DriverKeyLength);
//         if (Status != STATUS_BUFFER_TOO_SMALL)
//             return Status;
//         KeyNameLength += DriverKeyLength;
//     }
//     else
//     {
//         KeyNameLength += sizeof(EnumKeyName) - sizeof(UNICODE_NULL) +
//                          DeviceNode->InstancePath.Length;
//     }
//
//     /*
//      * Now allocate the buffer for the key name...
//      */
//
//     KeyNameBuffer = ExAllocatePool(PagedPool, KeyNameLength);
//     if (KeyNameBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     KeyName.Length = 0;
//     KeyName.MaximumLength = (USHORT)KeyNameLength;
//     KeyName.Buffer = KeyNameBuffer;
//
//     /*
//      * ...and build the key name.
//      */
//
//     KeyName.Length += sizeof(RootKeyName) - sizeof(UNICODE_NULL);
//     RtlCopyMemory(KeyNameBuffer, RootKeyName, KeyName.Length);
//
//     if (DevInstKeyType & PLUGPLAY_REGKEY_CURRENT_HWPROFILE)
//         RtlAppendUnicodeToString(&KeyName, ProfileKeyName);
//
//     if (DevInstKeyType & PLUGPLAY_REGKEY_DRIVER)
//     {
//         RtlAppendUnicodeToString(&KeyName, ClassKeyName);
//         Status = IoGetDeviceProperty(DeviceObject, DevicePropertyDriverKeyName,
//                                      DriverKeyLength, KeyNameBuffer +
//                                      (KeyName.Length / sizeof(WCHAR)),
//                                      &DriverKeyLength);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Call to IoGetDeviceProperty() failed with Status 0x%08lx\n", Status);
//             ExFreePool(KeyNameBuffer);
//             return Status;
//         }
//         KeyName.Length += (USHORT)DriverKeyLength - sizeof(UNICODE_NULL);
//     }
//     else
//     {
//         RtlAppendUnicodeToString(&KeyName, EnumKeyName);
//         Status = RtlAppendUnicodeStringToString(&KeyName, &DeviceNode->InstancePath);
//         if (DeviceNode->InstancePath.Length == 0)
//         {
//             ExFreePool(KeyNameBuffer);
//             return Status;
//         }
//     }
//
//     /*
//      * Open the base key.
//      */
//     Status = IopOpenRegistryKeyEx(DevInstRegKey, NULL, &KeyName, DesiredAccess);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IoOpenDeviceRegistryKey(%wZ): Base key doesn't exist, exiting... (Status 0x%08lx)\n", &KeyName, Status);
//         ExFreePool(KeyNameBuffer);
//         return Status;
//     }
//     ExFreePool(KeyNameBuffer);
//
//     /*
//      * For driver key we're done now.
//      */
//
//     if (DevInstKeyType & PLUGPLAY_REGKEY_DRIVER)
//         return Status;
//
//     /*
//      * Let's go further. For device key we must open "Device Parameters"
//      * subkey and create it if it doesn't exist yet.
//      */
//
//     RtlInitUnicodeString(&KeyName, DeviceParametersKeyName);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                *DevInstRegKey,
//                                NULL);
//     Status = ZwCreateKey(DevInstRegKey,
//                          DesiredAccess,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_NON_VOLATILE,
//                          NULL);
//     ZwClose(ObjectAttributes.RootDirectory);
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoInvalidateDeviceRelations(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN DEVICE_RELATION_TYPE Type)
// {
//     if (!IopIsValidPhysicalDeviceObject(DeviceObject))
//     {
//         KeBugCheckEx(PNP_DETECTED_FATAL_ERROR, 0x2, (ULONG_PTR)DeviceObject, 0, 0);
//     }
//
//     switch (Type)
//     {
//         case BusRelations:
//             /* Enumerate the device */
//             PiQueueDeviceAction(DeviceObject, PiActionEnumDeviceTree, NULL, NULL);
//             break;
//         default:
//             /* Everything else is not implemented */
//             break;
//     }
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoSynchronousInvalidateDeviceRelations(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN DEVICE_RELATION_TYPE Type)
// {
//     PAGED_CODE();
//
//     if (!IopIsValidPhysicalDeviceObject(DeviceObject))
//     {
//         KeBugCheckEx(PNP_DETECTED_FATAL_ERROR, 0x2, (ULONG_PTR)DeviceObject, 0, 0);
//     }
//
//     switch (Type)
//     {
//         case BusRelations:
//             /* Enumerate the device */
//             return PiPerformSyncDeviceAction(DeviceObject, PiActionEnumDeviceTree);
//         case PowerRelations:
//              /* Not handled yet */
//              return STATUS_NOT_IMPLEMENTED;
//         case TargetDeviceRelation:
//             /* Nothing to do */
//             return STATUS_SUCCESS;
//         default:
//             /* Ejection relations are not supported */
//             return STATUS_NOT_SUPPORTED;
//     }
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// IoTranslateBusAddress(IN INTERFACE_TYPE InterfaceType,
//                       IN ULONG BusNumber,
//                       IN PHYSICAL_ADDRESS BusAddress,
//                       IN OUT PULONG AddressSpace,
//                       OUT PPHYSICAL_ADDRESS TranslatedAddress)
// {
//     /* FIXME: Notify the resource arbiter */
//
//     return HalTranslateBusAddress(InterfaceType,
//                                   BusNumber,
//                                   BusAddress,
//                                   AddressSpace,
//                                   TranslatedAddress);
// }
//
// VOID
// NTAPI
// IoInvalidateDeviceState(
//     IN PDEVICE_OBJECT DeviceObject)
// {
//     if (!IopIsValidPhysicalDeviceObject(DeviceObject))
//     {
//         KeBugCheckEx(PNP_DETECTED_FATAL_ERROR, 0x2, (ULONG_PTR)DeviceObject, 0, 0);
//     }
//
//     PiQueueDeviceAction(DeviceObject, PiActionQueryState, NULL, NULL);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
