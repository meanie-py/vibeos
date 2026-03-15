use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _DRIVER_OBJECT, _DEVICE_OBJECT, _ARBITER_REQUEST_SOURCE,
    _ARBITER_REQUEST_SOURCE_ArbiterRequestLegacyAssigned,
    _IO_QUERY_DEVICE_DATA_FORMAT_IoQueryDeviceConfigurationData,
    _UNICODE_STRING,
    PDRIVER_OBJECT, PDEVICE_OBJECT, PCONFIGURATION_INFORMATION,
    CONFIGURATION_INFORMATION, PINTERFACE_TYPE, PCONFIGURATION_TYPE,
    PIO_QUERY_DEVICE_ROUTINE, PCM_RESOURCE_LIST, PIO_RESOURCE_REQUIREMENTS_LIST,
    PUNICODE_STRING, PKEY_BASIC_INFORMATION, PKEY_FULL_INFORMATION,
    PKEY_VALUE_FULL_INFORMATION, GUID, _OBJECT_ATTRIBUTES,
    ZwOpenKey, ZwClose, ZwCreateKey, ZwQueryKey, ZwEnumerateKey,
    ZwQueryValueKey, ZwSetValueKey, ZwOpenSymbolicLinkObject,
    ZwQuerySymbolicLinkObject, RtlInitUnicodeString, RtlAppendUnicodeToString,
    RtlAppendUnicodeStringToString, RtlInitEmptyUnicodeString,
    RtlIntegerToUnicodeString, ObCloseHandle, IopOpenRegistryKeyEx,
    IopCreateRegistryKeyEx, IopGetDeviceNode, IopDetectResourceConflict,
    IopFixupResourceListWithRequirements, IopFetchConfigurationInformation,
    IopStoreSystemPartitionInformation, IoGetDeviceInterfaces,
    PDEVICE_NODE, KEY_BASIC_INFORMATION,
};

use ntkrnl_sys::inline::common::InitializeObjectAttributes;

const ArbiterRequestLegacyAssigned: _ARBITER_REQUEST_SOURCE = _ARBITER_REQUEST_SOURCE_ArbiterRequestLegacyAssigned;
const IoQueryDeviceConfigurationData: usize = _IO_QUERY_DEVICE_DATA_FORMAT_IoQueryDeviceConfigurationData as usize;

const KEY_ALL_ACCESS: u32 = 0xF003F;
const REG_OPTION_VOLATILE: u32 = 1;
const REG_RESOURCE_LIST: u32 = 8;

const TAG_IO_RESOURCE: ULONG = 0x52734F49;
const MaximumType: usize = 41;
const IoQueryDeviceMaxData: usize = 3;

static mut _SystemConfigurationInformation: CONFIGURATION_INFORMATION = 
    CONFIGURATION_INFORMATION {
        DiskCount: 0,
        FloppyCount: 0,
        CdRomCount: 0,
        TapeCount: 0,
        ScsiPortCount: 0,
        SerialCount: 0,
        ParallelCount: 0,
        AtDiskPrimaryAddressClaimed: 0,
        AtDiskSecondaryAddressClaimed: 0,
        Version: 0,
        MediumChangerCount: 0,
    };

#[repr(C)]
struct IO_QUERY {
    BusType: PINTERFACE_TYPE,
    BusNumber: *mut ULONG,
    ControllerType: PCONFIGURATION_TYPE,
    ControllerNumber: *mut ULONG,
    PeripheralType: PCONFIGURATION_TYPE,
    PeripheralNumber: *mut ULONG,
    CalloutRoutine: PIO_QUERY_DEVICE_ROUTINE,
    Context: PVOID,
}

// These arrays contain read-only pointers to static byte string data.
// They are never mutated at runtime, but use `static mut` to avoid Sync bounds on raw pointers.
static mut ARC_TYPES: [PCWSTR; MaximumType + 1] = [
    b"S\0y\0s\0t\0e\0m\0\0\0".as_ptr() as PCWSTR,
    b"C\0e\0n\0t\0r\0a\0l\0P\0r\0o\0c\0e\0s\0s\0o\0r\0\0\0".as_ptr() as PCWSTR,
    b"F\0l\0o\0a\0t\0i\0n\0g\0P\0o\0i\0n\0t\0P\0r\0o\0c\0e\0s\0s\0o\0r\0\0\0".as_ptr() as PCWSTR,
    b"P\0r\0i\0m\0a\0r\0y\0I\0C\0a\0c\0h\0e\0\0\0".as_ptr() as PCWSTR,
    b"P\0r\0i\0m\0a\0r\0y\0D\0C\0a\0c\0h\0e\0\0\0".as_ptr() as PCWSTR,
    b"S\0e\0c\0o\0n\0d\0a\0r\0y\0I\0C\0a\0c\0h\0e\0\0\0".as_ptr() as PCWSTR,
    b"S\0e\0c\0o\0n\0d\0a\0r\0y\0D\0C\0a\0c\0h\0e\0\0\0".as_ptr() as PCWSTR,
    b"S\0e\0c\0o\0n\0d\0a\0r\0y\0C\0a\0c\0h\0e\0\0\0".as_ptr() as PCWSTR,
    b"E\0i\0s\0a\0A\0d\0a\0p\0t\0e\0r\0\0\0".as_ptr() as PCWSTR,
    b"T\0c\0A\0d\0a\0p\0t\0e\0r\0\0\0".as_ptr() as PCWSTR,
    b"S\0c\0s\0i\0A\0d\0a\0p\0t\0e\0r\0\0\0".as_ptr() as PCWSTR,
    b"D\0t\0i\0A\0d\0a\0p\0t\0e\0r\0\0\0".as_ptr() as PCWSTR,
    b"M\0u\0l\0t\0i\0f\0u\0n\0c\0t\0i\0o\0n\0A\0d\0a\0p\0t\0e\0r\0\0\0".as_ptr() as PCWSTR,
    b"D\0i\0s\0k\0C\0o\0n\0t\0r\0o\0l\0l\0e\0r\0\0\0".as_ptr() as PCWSTR,
    b"T\0a\0p\0e\0C\0o\0n\0t\0r\0o\0l\0l\0e\0r\0\0\0".as_ptr() as PCWSTR,
    b"C\0d\0R\0o\0m\0C\0o\0n\0t\0r\0o\0l\0l\0e\0r\0\0\0".as_ptr() as PCWSTR,
    b"W\0o\0r\0m\0C\0o\0n\0t\0r\0o\0l\0l\0e\0r\0\0\0".as_ptr() as PCWSTR,
    b"S\0e\0r\0i\0a\0l\0C\0o\0n\0t\0r\0o\0l\0l\0e\0r\0\0\0".as_ptr() as PCWSTR,
    b"N\0e\0t\0w\0o\0r\0k\0C\0o\0n\0t\0r\0o\0l\0l\0e\0r\0\0\0".as_ptr() as PCWSTR,
    b"D\0i\0s\0p\0l\0a\0y\0C\0o\0n\0t\0r\0o\0l\0l\0e\0r\0\0\0".as_ptr() as PCWSTR,
    b"P\0a\0r\0a\0l\0l\0e\0l\0C\0o\0n\0t\0r\0o\0l\0l\0e\0r\0\0\0".as_ptr() as PCWSTR,
    b"P\0o\0i\0n\0t\0e\0r\0C\0o\0n\0t\0r\0o\0l\0l\0e\0r\0\0\0".as_ptr() as PCWSTR,
    b"K\0e\0y\0b\0o\0a\0r\0d\0C\0o\0n\0t\0r\0o\0l\0l\0e\0r\0\0\0".as_ptr() as PCWSTR,
    b"A\0u\0d\0i\0o\0C\0o\0n\0t\0r\0o\0l\0l\0e\0r\0\0\0".as_ptr() as PCWSTR,
    b"O\0t\0h\0e\0r\0C\0o\0n\0t\0r\0o\0l\0l\0e\0r\0\0\0".as_ptr() as PCWSTR,
    b"D\0i\0s\0k\0P\0e\0r\0i\0p\0h\0e\0r\0a\0l\0\0\0".as_ptr() as PCWSTR,
    b"F\0l\0o\0p\0p\0y\0D\0i\0s\0k\0P\0e\0r\0i\0p\0h\0e\0r\0a\0l\0\0\0".as_ptr() as PCWSTR,
    b"T\0a\0p\0e\0P\0e\0r\0i\0p\0h\0e\0r\0a\0l\0\0\0".as_ptr() as PCWSTR,
    b"M\0o\0d\0e\0m\0P\0e\0r\0i\0p\0h\0e\0r\0a\0l\0\0\0".as_ptr() as PCWSTR,
    b"M\0o\0n\0i\0t\0o\0r\0P\0e\0r\0i\0p\0h\0e\0r\0a\0l\0\0\0".as_ptr() as PCWSTR,
    b"P\0r\0i\0n\0t\0e\0r\0P\0e\0r\0i\0p\0h\0e\0r\0a\0l\0\0\0".as_ptr() as PCWSTR,
    b"P\0o\0i\0n\0t\0e\0r\0P\0e\0r\0i\0p\0h\0e\0r\0a\0l\0\0\0".as_ptr() as PCWSTR,
    b"K\0e\0y\0b\0o\0a\0r\0d\0P\0e\0r\0i\0p\0h\0e\0r\0a\0l\0\0\0".as_ptr() as PCWSTR,
    b"T\0e\0r\0m\0i\0n\0a\0l\0P\0e\0r\0i\0p\0h\0e\0r\0a\0l\0\0\0".as_ptr() as PCWSTR,
    b"O\0t\0h\0e\0r\0P\0e\0r\0i\0p\0h\0e\0r\0a\0l\0\0\0".as_ptr() as PCWSTR,
    b"L\0i\0n\0e\0P\0e\0r\0i\0p\0h\0e\0r\0a\0l\0\0\0".as_ptr() as PCWSTR,
    b"N\0e\0t\0w\0o\0r\0k\0P\0e\0r\0i\0p\0h\0e\0r\0a\0l\0\0\0".as_ptr() as PCWSTR,
    b"S\0y\0s\0t\0e\0m\0M\0e\0m\0o\0r\0y\0\0\0".as_ptr() as PCWSTR,
    b"D\0o\0c\0k\0i\0n\0g\0I\0n\0f\0o\0r\0m\0a\0t\0i\0o\0n\0\0\0".as_ptr() as PCWSTR,
    b"R\0e\0a\0l\0M\0o\0d\0e\0I\0r\0q\0R\0o\0u\0t\0i\0n\0g\0T\0a\0b\0l\0e\0\0\0".as_ptr() as PCWSTR,
    b"R\0e\0a\0l\0M\0o\0d\0e\0P\0C\0I\0E\0n\0u\0m\0e\0r\0a\0t\0i\0o\0n\0\0\0".as_ptr() as PCWSTR,
    b"U\0n\0d\0e\0f\0i\0n\0e\0d\0\0\0".as_ptr() as PCWSTR,
];

static mut IO_DEVICE_INFO_NAMES: [PCWSTR; IoQueryDeviceMaxData] = [
    b"I\0d\0e\0n\0t\0i\0f\0i\0e\0r\0\0\0".as_ptr() as PCWSTR,
    b"C\0o\0n\0f\0i\0g\0u\0r\0a\0t\0i\0o\0n\0 \0D\0a\0t\0a\0\0\0".as_ptr() as PCWSTR,
    b"C\0o\0m\0p\0o\0n\0e\0n\0t\0 \0I\0n\0f\0o\0r\0m\0a\0t\0i\0o\0n\0\0\0".as_ptr() as PCWSTR,
];

unsafe fn IopQueryDeviceDescription(
    Query: *mut IO_QUERY,
    RootKey: _UNICODE_STRING,
    RootKeyHandle: HANDLE,
    Bus: ULONG,
    BusInformation: *mut PKEY_VALUE_FULL_INFORMATION,
) -> NTSTATUS {
    let mut Status = STATUS_SUCCESS;

    let mut ControllerString: _UNICODE_STRING = core::mem::zeroed();
    let mut ControllerRootRegName = RootKey;
    let ControllerRegName: _UNICODE_STRING;
    let mut ControllerKeyHandle: HANDLE = core::ptr::null_mut();
    let mut ControllerFullInformation: PKEY_FULL_INFORMATION = core::ptr::null_mut();
    let mut ControllerInformation: [PKEY_VALUE_FULL_INFORMATION; IoQueryDeviceMaxData] =
        [core::ptr::null_mut(); IoQueryDeviceMaxData];
    let ControllerNumber: ULONG;
    let mut ControllerLoop: ULONG;
    let MaximumControllerNumber: ULONG;

    let mut PeripheralString: _UNICODE_STRING = core::mem::zeroed();
    let mut PeripheralKeyHandle: HANDLE = core::ptr::null_mut();
    let mut PeripheralFullInformation: PKEY_FULL_INFORMATION = core::ptr::null_mut();
    let mut PeripheralInformation: [PKEY_VALUE_FULL_INFORMATION; IoQueryDeviceMaxData] =
        [core::ptr::null_mut(); IoQueryDeviceMaxData];
    let mut PeripheralNumber: ULONG;
    let mut PeripheralLoop: ULONG;
    let mut MaximumPeripheralNumber: ULONG;

    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut LenFullInformation: ULONG = 0;
    let mut LenKeyFullInformation: ULONG = 0;
    let mut TempString: _UNICODE_STRING = core::mem::zeroed();
    let mut TempBuffer: [u16; 14] = [0; 14];

    RtlInitEmptyUnicodeString(&mut TempString, TempBuffer.as_mut_ptr(), core::mem::size_of_val(&TempBuffer) as u16);

    RtlAppendUnicodeToString(&mut ControllerRootRegName, b"\\\0".as_ptr() as PCWSTR);
    RtlAppendUnicodeToString(&mut ControllerRootRegName, ARC_TYPES[*(*Query).ControllerType as usize]);

    if !(*Query).ControllerNumber.is_null() {
        ControllerNumber = *(*Query).ControllerNumber;
        MaximumControllerNumber = ControllerNumber + 1;
    } else {
        InitializeObjectAttributes(
            &mut ObjectAttributes,
            &mut ControllerRootRegName,
            OBJ_CASE_INSENSITIVE,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        Status = ZwOpenKey(&mut ControllerKeyHandle, KEY_READ, &mut ObjectAttributes);

        if Status >= 0 {
            ZwQueryKey(
                ControllerKeyHandle,
                1,
                core::ptr::null_mut(),
                0,
                &mut LenFullInformation,
            );

            ControllerFullInformation = ExAllocatePoolWithTag(PagedPool, LenFullInformation as usize, TAG_IO_RESOURCE) as PKEY_FULL_INFORMATION;
            if ControllerFullInformation.is_null() {
                ZwClose(ControllerKeyHandle);
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            Status = ZwQueryKey(
                ControllerKeyHandle,
                1,
                ControllerFullInformation as PVOID,
                LenFullInformation,
                &mut LenFullInformation,
            );
            ZwClose(ControllerKeyHandle);
            ControllerKeyHandle = core::ptr::null_mut();
        }

        if Status < 0 {
            if !ControllerFullInformation.is_null() {
                ExFreePoolWithTag(ControllerFullInformation as PVOID, TAG_IO_RESOURCE);
            }
            return Status;
        }

        ControllerNumber = 0;
        MaximumControllerNumber = (*ControllerFullInformation).SubKeys;

        ExFreePoolWithTag(ControllerFullInformation as PVOID, TAG_IO_RESOURCE);
        ControllerFullInformation = core::ptr::null_mut();
    }

    ControllerRegName = ControllerRootRegName;

    let mut ControllerNum = ControllerNumber;
    while ControllerNum < MaximumControllerNumber {
        ControllerRootRegName = ControllerRegName;

        Status = RtlIntegerToUnicodeString(ControllerNum, 10, &mut TempString);

        Status |= RtlAppendUnicodeToString(&mut ControllerRootRegName, b"\\\0".as_ptr() as PCWSTR);
        Status |= RtlAppendUnicodeStringToString(&mut ControllerRootRegName, &mut TempString);

        if Status < 0 {
            break;
        }

        InitializeObjectAttributes(
            &mut ObjectAttributes,
            &mut ControllerRootRegName,
            OBJ_CASE_INSENSITIVE,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        Status = ZwOpenKey(&mut ControllerKeyHandle, KEY_READ, &mut ObjectAttributes);

        if Status >= 0 {
            ControllerLoop = 0;
            while ControllerLoop < IoQueryDeviceMaxData as ULONG {
                RtlInitUnicodeString(&mut ControllerString, IO_DEVICE_INFO_NAMES[ControllerLoop as usize]);

                Status = ZwQueryValueKey(
                    ControllerKeyHandle,
                    &mut ControllerString,
                    2,
                    core::ptr::null_mut(),
                    0,
                    &mut LenKeyFullInformation,
                );

                if Status < 0 && Status != STATUS_BUFFER_TOO_SMALL && Status != STATUS_BUFFER_OVERFLOW {
                    ControllerInformation[ControllerLoop as usize] = core::ptr::null_mut();
                    ControllerLoop += 1;
                    continue;
                }

                ControllerInformation[ControllerLoop as usize] = ExAllocatePoolWithTag(
                    PagedPool,
                    LenKeyFullInformation as usize,
                    TAG_IO_RESOURCE,
                ) as PKEY_VALUE_FULL_INFORMATION;
                if ControllerInformation[ControllerLoop as usize].is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break;
                }

                Status = ZwQueryValueKey(
                    ControllerKeyHandle,
                    &mut ControllerString,
                    2,
                    ControllerInformation[ControllerLoop as usize] as PVOID,
                    LenKeyFullInformation,
                    &mut LenKeyFullInformation,
                );

                ControllerLoop += 1;
            }

            ZwClose(ControllerKeyHandle);
            ControllerKeyHandle = core::ptr::null_mut();
        }

        if Status < 0 {
            ControllerLoop = 0;
            while ControllerLoop < IoQueryDeviceMaxData as ULONG {
                if !ControllerInformation[ControllerLoop as usize].is_null() {
                    ExFreePoolWithTag(ControllerInformation[ControllerLoop as usize] as PVOID, TAG_IO_RESOURCE);
                    ControllerInformation[ControllerLoop as usize] = core::ptr::null_mut();
                }
                ControllerLoop += 1;
            }
            if Status < 0 {
                break;
            }
            ControllerNum += 1;
            continue;
        }

        if (*Query).PeripheralType.is_null() || *(*Query).PeripheralType == 0 {
            Status = ((*Query).CalloutRoutine.unwrap())(
                (*Query).Context,
                &mut ControllerRootRegName,
                *(*Query).BusType,
                Bus,
                BusInformation,
                *(*Query).ControllerType,
                ControllerNum,
                ControllerInformation.as_mut_ptr(),
                0,
                0,
                core::ptr::null_mut(),
            );
            ControllerLoop = 0;
            while ControllerLoop < IoQueryDeviceMaxData as ULONG {
                if !ControllerInformation[ControllerLoop as usize].is_null() {
                    ExFreePoolWithTag(ControllerInformation[ControllerLoop as usize] as PVOID, TAG_IO_RESOURCE);
                    ControllerInformation[ControllerLoop as usize] = core::ptr::null_mut();
                }
                ControllerLoop += 1;
            }
            if Status < 0 {
                break;
            }
            ControllerNum += 1;
            continue;
        }

        Status = RtlAppendUnicodeToString(&mut ControllerRootRegName, b"\\\0".as_ptr() as PCWSTR);
        Status |= RtlAppendUnicodeToString(&mut ControllerRootRegName, ARC_TYPES[*(*Query).PeripheralType as usize]);

        if Status < 0 {
            ControllerLoop = 0;
            while ControllerLoop < IoQueryDeviceMaxData as ULONG {
                if !ControllerInformation[ControllerLoop as usize].is_null() {
                    ExFreePoolWithTag(ControllerInformation[ControllerLoop as usize] as PVOID, TAG_IO_RESOURCE);
                    ControllerInformation[ControllerLoop as usize] = core::ptr::null_mut();
                }
                ControllerLoop += 1;
            }
            if Status < 0 {
                break;
            }
            ControllerNum += 1;
            continue;
        }

        if !(*Query).PeripheralNumber.is_null() {
            PeripheralNumber = *(*Query).PeripheralNumber;
            MaximumPeripheralNumber = PeripheralNumber + 1;
        } else {
            InitializeObjectAttributes(
                &mut ObjectAttributes,
                &mut ControllerRootRegName,
                OBJ_CASE_INSENSITIVE,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            Status = ZwOpenKey(&mut PeripheralKeyHandle, KEY_READ, &mut ObjectAttributes);

            if Status >= 0 {
                ZwQueryKey(
                    PeripheralKeyHandle,
                    1,
                    core::ptr::null_mut(),
                    0,
                    &mut LenFullInformation,
                );

                PeripheralFullInformation = ExAllocatePoolWithTag(PagedPool, LenFullInformation as usize, TAG_IO_RESOURCE) as PKEY_FULL_INFORMATION;
                if PeripheralFullInformation.is_null() {
                    ZwClose(PeripheralKeyHandle);
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    ControllerLoop = 0;
                    while ControllerLoop < IoQueryDeviceMaxData as ULONG {
                        if !ControllerInformation[ControllerLoop as usize].is_null() {
                            ExFreePoolWithTag(ControllerInformation[ControllerLoop as usize] as PVOID, TAG_IO_RESOURCE);
                            ControllerInformation[ControllerLoop as usize] = core::ptr::null_mut();
                        }
                        ControllerLoop += 1;
                    }
                    if Status < 0 {
                        break;
                    }
                    ControllerNum += 1;
                    continue;
                }

                Status = ZwQueryKey(
                    PeripheralKeyHandle,
                    1,
                    PeripheralFullInformation as PVOID,
                    LenFullInformation,
                    &mut LenFullInformation,
                );
                ZwClose(PeripheralKeyHandle);
                PeripheralKeyHandle = core::ptr::null_mut();
            }

            if Status < 0 {
                Status = STATUS_SUCCESS;
                ControllerLoop = 0;
                while ControllerLoop < IoQueryDeviceMaxData as ULONG {
                    if !ControllerInformation[ControllerLoop as usize].is_null() {
                        ExFreePoolWithTag(ControllerInformation[ControllerLoop as usize] as PVOID, TAG_IO_RESOURCE);
                        ControllerInformation[ControllerLoop as usize] = core::ptr::null_mut();
                    }
                    ControllerLoop += 1;
                }
                if Status < 0 {
                    break;
                }
                ControllerNum += 1;
                continue;
            }

            PeripheralNumber = 0;
            MaximumPeripheralNumber = (*PeripheralFullInformation).SubKeys;

            ExFreePoolWithTag(PeripheralFullInformation as PVOID, TAG_IO_RESOURCE);
            PeripheralFullInformation = core::ptr::null_mut();
        }

        let ControllerRegName2 = ControllerRootRegName;

        let mut PeripheralNum = PeripheralNumber;
        while PeripheralNum < MaximumPeripheralNumber {
            ControllerRootRegName = ControllerRegName2;

            Status = RtlIntegerToUnicodeString(PeripheralNum, 10, &mut TempString);

            Status |= RtlAppendUnicodeToString(&mut ControllerRootRegName, b"\\\0".as_ptr() as PCWSTR);
            Status |= RtlAppendUnicodeStringToString(&mut ControllerRootRegName, &mut TempString);

            if Status < 0 {
                break;
            }

            InitializeObjectAttributes(
                &mut ObjectAttributes,
                &mut ControllerRootRegName,
                OBJ_CASE_INSENSITIVE,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            Status = ZwOpenKey(&mut PeripheralKeyHandle, KEY_READ, &mut ObjectAttributes);

            if Status >= 0 {
                PeripheralLoop = 0;
                while PeripheralLoop < IoQueryDeviceMaxData as ULONG {
                    RtlInitUnicodeString(&mut PeripheralString, IO_DEVICE_INFO_NAMES[PeripheralLoop as usize]);

                    Status = ZwQueryValueKey(
                        PeripheralKeyHandle,
                        &mut PeripheralString,
                        2,
                        core::ptr::null_mut(),
                        0,
                        &mut LenKeyFullInformation,
                    );

                    if Status < 0 && Status != STATUS_BUFFER_TOO_SMALL && Status != STATUS_BUFFER_OVERFLOW {
                        PeripheralInformation[PeripheralLoop as usize] = core::ptr::null_mut();
                        PeripheralLoop += 1;
                        continue;
                    }

                    PeripheralInformation[PeripheralLoop as usize] = ExAllocatePoolWithTag(
                        PagedPool,
                        LenKeyFullInformation as usize,
                        TAG_IO_RESOURCE,
                    ) as PKEY_VALUE_FULL_INFORMATION;
                    if PeripheralInformation[PeripheralLoop as usize].is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        break;
                    }

                    Status = ZwQueryValueKey(
                        PeripheralKeyHandle,
                        &mut PeripheralString,
                        2,
                        PeripheralInformation[PeripheralLoop as usize] as PVOID,
                        LenKeyFullInformation,
                        &mut LenKeyFullInformation,
                    );

                    PeripheralLoop += 1;
                }

                ZwClose(PeripheralKeyHandle);
                PeripheralKeyHandle = core::ptr::null_mut();

                if Status >= 0 {
                    Status = ((*Query).CalloutRoutine.unwrap())(
                        (*Query).Context,
                        &mut ControllerRootRegName,
                        *(*Query).BusType,
                        Bus,
                        BusInformation,
                        *(*Query).ControllerType,
                        ControllerNum,
                        ControllerInformation.as_mut_ptr(),
                        *(*Query).PeripheralType,
                        PeripheralNum,
                        PeripheralInformation.as_mut_ptr(),
                    );
                }

                PeripheralLoop = 0;
                while PeripheralLoop < IoQueryDeviceMaxData as ULONG {
                    if !PeripheralInformation[PeripheralLoop as usize].is_null() {
                        ExFreePoolWithTag(PeripheralInformation[PeripheralLoop as usize] as PVOID, TAG_IO_RESOURCE);
                        PeripheralInformation[PeripheralLoop as usize] = core::ptr::null_mut();
                    }
                    PeripheralLoop += 1;
                }

                if Status < 0 {
                    break;
                }
            }

            PeripheralNum += 1;
        }

        ControllerLoop = 0;
        while ControllerLoop < IoQueryDeviceMaxData as ULONG {
            if !ControllerInformation[ControllerLoop as usize].is_null() {
                ExFreePoolWithTag(ControllerInformation[ControllerLoop as usize] as PVOID, TAG_IO_RESOURCE);
                ControllerInformation[ControllerLoop as usize] = core::ptr::null_mut();
            }
            ControllerLoop += 1;
        }

        if Status < 0 {
            break;
        }

        ControllerNum += 1;
    }

    Status
}

unsafe fn IopQueryBusDescription(
    Query: *mut IO_QUERY,
    RootKey: _UNICODE_STRING,
    RootKeyHandle: HANDLE,
    Bus: *mut ULONG,
    KeyIsRoot: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut BusLoop: ULONG;
    let mut SubRootRegName: _UNICODE_STRING;
    let mut BusString: _UNICODE_STRING = core::mem::zeroed();
    let mut SubBusString: _UNICODE_STRING = core::mem::zeroed();
    let mut LenBasicInformation: ULONG = 0;
    let mut LenFullInformation: ULONG = 0;
    let mut LenKeyFullInformation: ULONG = 0;
    let mut LenKey: ULONG = 0;
    let mut SubRootKeyHandle: HANDLE = core::ptr::null_mut();
    let mut FullInformation: PKEY_FULL_INFORMATION;
    let mut BasicInformation: PKEY_BASIC_INFORMATION = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut BusInformation: [PKEY_VALUE_FULL_INFORMATION; IoQueryDeviceMaxData] =
        [core::ptr::null_mut(); IoQueryDeviceMaxData];

    Status = ZwQueryKey(
        RootKeyHandle,
        1,
        core::ptr::null_mut(),
        0,
        &mut LenFullInformation,
    );

    if Status < 0 && Status != STATUS_BUFFER_TOO_SMALL && Status != STATUS_BUFFER_OVERFLOW {
        return Status;
    }

    FullInformation = ExAllocatePoolWithTag(PagedPool, LenFullInformation as usize, TAG_IO_RESOURCE) as PKEY_FULL_INFORMATION;
    if FullInformation.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = ZwQueryKey(
        RootKeyHandle,
        1,
        FullInformation as PVOID,
        LenFullInformation,
        &mut LenFullInformation,
    );
    if Status >= 0 {
        LenBasicInformation = (*FullInformation).MaxNameLen + core::mem::size_of::<KEY_BASIC_INFORMATION>() as ULONG;

        BasicInformation = ExAllocatePoolWithTag(PagedPool, LenBasicInformation as usize, TAG_IO_RESOURCE) as PKEY_BASIC_INFORMATION;
        if BasicInformation.is_null() {
            ExFreePoolWithTag(FullInformation as PVOID, TAG_IO_RESOURCE);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    ExFreePoolWithTag(FullInformation as PVOID, TAG_IO_RESOURCE);

    BusLoop = 0;
    while Status >= 0 {
        if !(*Query).BusNumber.is_null() && *(*Query).BusNumber == *Bus {
            break;
        }

        Status = ZwEnumerateKey(
            RootKeyHandle,
            BusLoop,
            0,
            BasicInformation as PVOID,
            LenBasicInformation,
            &mut LenKey,
        );

        if Status < 0 {
            break;
        }

        if KeyIsRoot != 0 {
            let name_ptr = (*BasicInformation).Name.as_ptr();
            let name_len = (*BasicInformation).NameLength / core::mem::size_of::<u16>() as ULONG;

            fn wcsncmp(s1: *const u16, s2: &[u16], n: ULONG) -> bool {
                let mut i = 0;
                while i < n as usize && i < s2.len() {
                    if unsafe { *s1.add(i) } != s2[i] {
                        return false;
                    }
                    i += 1;
                }
                i == n as usize
            }

            static MULTIFUNCTION: &[u16] = &[
                b'M' as u16, b'u' as u16, b'l' as u16, b't' as u16, b'i' as u16,
                b'f' as u16, b'u' as u16, b'n' as u16, b'c' as u16, b't' as u16,
                b'i' as u16, b'o' as u16, b'n' as u16, b'A' as u16, b'd' as u16,
                b'a' as u16, b'p' as u16, b't' as u16, b'e' as u16, b'r' as u16,
            ];
            static EISA: &[u16] = &[
                b'E' as u16, b'i' as u16, b's' as u16, b'a' as u16, b'A' as u16,
                b'd' as u16, b'a' as u16, b'p' as u16, b't' as u16, b'e' as u16,
                b'r' as u16,
            ];
            static TC: &[u16] = &[
                b'T' as u16, b'c' as u16, b'A' as u16, b'd' as u16, b'a' as u16,
                b'p' as u16, b't' as u16, b'e' as u16, b'r' as u16,
            ];

            if !wcsncmp(name_ptr, MULTIFUNCTION, name_len)
                && !wcsncmp(name_ptr, EISA, name_len)
                && !wcsncmp(name_ptr, TC, name_len)
            {
                BusLoop += 1;
                continue;
            }
        }

        BusString.Buffer = (*BasicInformation).Name.as_mut_ptr();
        BusString.Length = (*BasicInformation).NameLength as u16;
        BusString.MaximumLength = (*BasicInformation).NameLength as u16;

        InitializeObjectAttributes(
            &mut ObjectAttributes,
            &mut BusString,
            OBJ_CASE_INSENSITIVE,
            RootKeyHandle,
            core::ptr::null_mut(),
        );

        Status = ZwOpenKey(&mut SubRootKeyHandle, KEY_READ, &mut ObjectAttributes);

        if Status < 0 {
            BusLoop += 1;
            continue;
        }

        SubRootRegName = RootKey;
        RtlAppendUnicodeToString(&mut SubRootRegName, b"\\\0".as_ptr() as PCWSTR);
        RtlAppendUnicodeStringToString(&mut SubRootRegName, &mut BusString);

        if KeyIsRoot == 0 {
            let mut SubBusLoop: ULONG = 0;
            while SubBusLoop < IoQueryDeviceMaxData as ULONG {
                RtlInitUnicodeString(&mut SubBusString, IO_DEVICE_INFO_NAMES[SubBusLoop as usize]);

                ZwQueryValueKey(
                    SubRootKeyHandle,
                    &mut SubBusString,
                    2,
                    core::ptr::null_mut(),
                    0,
                    &mut LenKeyFullInformation,
                );

                BusInformation[SubBusLoop as usize] = ExAllocatePoolWithTag(
                    PagedPool,
                    LenKeyFullInformation as usize,
                    TAG_IO_RESOURCE,
                ) as PKEY_VALUE_FULL_INFORMATION;
                if BusInformation[SubBusLoop as usize].is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break;
                }

                Status = ZwQueryValueKey(
                    SubRootKeyHandle,
                    &mut SubBusString,
                    2,
                    BusInformation[SubBusLoop as usize] as PVOID,
                    LenKeyFullInformation,
                    &mut LenKeyFullInformation,
                );

                SubBusLoop += 1;
            }

            if Status >= 0 {
                let BusConfigData = BusInformation[IoQueryDeviceConfigurationData as usize];

                if !BusConfigData.is_null()
                    && (*BusConfigData).DataLength != 0
                    && (*(((BusConfigData as usize) + (*BusConfigData).DataOffset as usize) as *const u32) == *(*Query).BusType as u32)
                {
                    *Bus += 1;

                    if (*Query).BusNumber.is_null() || *(*Query).BusNumber == *Bus {
                        if (*Query).ControllerType.is_null() {
                            Status = ((*Query).CalloutRoutine.unwrap())(
                                (*Query).Context,
                                &mut SubRootRegName,
                                *(*Query).BusType,
                                *Bus,
                                BusInformation.as_mut_ptr(),
                                0,
                                0,
                                core::ptr::null_mut(),
                                0,
                                0,
                                core::ptr::null_mut(),
                            );
                        } else {
                            Status = IopQueryDeviceDescription(
                                Query,
                                SubRootRegName,
                                RootKeyHandle,
                                *Bus,
                                BusInformation.as_mut_ptr(),
                            );
                        }
                    }
                }
            }

            SubBusLoop = 0;
            while SubBusLoop < IoQueryDeviceMaxData as ULONG {
                if !BusInformation[SubBusLoop as usize].is_null() {
                    ExFreePoolWithTag(BusInformation[SubBusLoop as usize] as PVOID, TAG_IO_RESOURCE);
                    BusInformation[SubBusLoop as usize] = core::ptr::null_mut();
                }
                SubBusLoop += 1;
            }

            if !(*Query).BusNumber.is_null() && *(*Query).BusNumber == *Bus {
                ZwClose(SubRootKeyHandle);
                SubRootKeyHandle = core::ptr::null_mut();
                BusLoop += 1;
                continue;
            }
        }

        Status = IopQueryBusDescription(Query, SubRootRegName, SubRootKeyHandle, Bus, if KeyIsRoot != 0 { 0 } else { 1 });

        if Status == STATUS_NO_MORE_ENTRIES {
            Status = STATUS_SUCCESS;
        }

        ZwClose(SubRootKeyHandle);
        SubRootKeyHandle = core::ptr::null_mut();

        BusLoop += 1;
    }

    if !BasicInformation.is_null() {
        ExFreePoolWithTag(BasicInformation as PVOID, TAG_IO_RESOURCE);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoGetConfigurationInformation() -> PCONFIGURATION_INFORMATION {
    core::ptr::addr_of_mut!(_SystemConfigurationInformation)
}

#[no_mangle]
pub unsafe extern "system" fn IoReportResourceUsage(
    DriverClassName: PUNICODE_STRING,
    DriverObject: PDRIVER_OBJECT,
    DriverList: PCM_RESOURCE_LIST,
    DriverListSize: ULONG,
    DeviceObject: PDEVICE_OBJECT,
    DeviceList: PCM_RESOURCE_LIST,
    DeviceListSize: ULONG,
    OverrideConflict: BOOLEAN,
    ConflictDetected: PBOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let ResourceList: PCM_RESOURCE_LIST;

    if DriverList.is_null() && DeviceList.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if !DeviceList.is_null() {
        ResourceList = DeviceList;
    } else {
        ResourceList = DriverList;
    }

    Status = IopDetectResourceConflict(ResourceList, 0u8, core::ptr::null_mut());
    if Status == STATUS_CONFLICTING_ADDRESSES {
        *ConflictDetected = 1;

        if OverrideConflict == 0 {
            return STATUS_CONFLICTING_ADDRESSES;
        }
    } else if Status < 0 {
        return Status;
    }

    *ConflictDetected = 0;

    STATUS_SUCCESS
}

unsafe fn IopLegacyResourceAllocation(
    AllocationType: _ARBITER_REQUEST_SOURCE,
    DriverObject: PDRIVER_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
    ResourceRequirements: PIO_RESOURCE_REQUIREMENTS_LIST,
    AllocatedResources: *mut PCM_RESOURCE_LIST,
) -> NTSTATUS {
    let Status: NTSTATUS;

    if ResourceRequirements.is_null() {
        return STATUS_NOT_IMPLEMENTED;
    }

    Status = IopFixupResourceListWithRequirements(ResourceRequirements, AllocatedResources);
    if Status < 0 {
        if Status == STATUS_CONFLICTING_ADDRESSES {
        }

        return Status;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoAssignResources(
    RegistryPath: PUNICODE_STRING,
    DriverClassName: PUNICODE_STRING,
    DriverObject: PDRIVER_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
    RequestedResources: PIO_RESOURCE_REQUIREMENTS_LIST,
    AllocatedResources: *mut PCM_RESOURCE_LIST,
) -> NTSTATUS {
    let DeviceNode: PDEVICE_NODE;

    if !DeviceObject.is_null() {
        DeviceNode = IopGetDeviceNode(DeviceObject);
        if !DeviceNode.is_null() && ((*DeviceNode).Flags & 0x400) == 0 {
            KeBugCheckEx(
                0xCA,
                0x2,
                DeviceObject as usize,
                DriverObject as usize,
                0,
            );
        }
    }

    let mut RequestedResourcesFixed = RequestedResources;
    if !RequestedResources.is_null() {
        if (*RequestedResources).AlternativeLists == 0 || (*(*RequestedResources).List.as_ptr()).Count == 0 {
            RequestedResourcesFixed = core::ptr::null_mut();
        }
    }

    if !AllocatedResources.is_null() {
        *AllocatedResources = core::ptr::null_mut();
    }

    IopLegacyResourceAllocation(
        ArbiterRequestLegacyAssigned,
        DriverObject,
        DeviceObject,
        RequestedResourcesFixed,
        AllocatedResources,
    )
}

#[no_mangle]
pub unsafe extern "system" fn IoQueryDeviceDescription(
    BusType: PINTERFACE_TYPE,
    BusNumber: *mut ULONG,
    ControllerType: PCONFIGURATION_TYPE,
    ControllerNumber: *mut ULONG,
    PeripheralType: PCONFIGURATION_TYPE,
    PeripheralNumber: *mut ULONG,
    CalloutRoutine: PIO_QUERY_DEVICE_ROUTINE,
    Context: PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut BusLoopNumber: ULONG = 0xFFFFFFFF;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut RootRegKey: _UNICODE_STRING = core::mem::zeroed();
    let mut RootRegHandle: HANDLE = core::ptr::null_mut();
    let mut Query: IO_QUERY = core::mem::zeroed();

    if BusType.is_null() {
        return STATUS_NOT_IMPLEMENTED;
    }

    RootRegKey.Length = 0;
    RootRegKey.MaximumLength = 2048;
    RootRegKey.Buffer = ExAllocatePoolWithTag(PagedPool, RootRegKey.MaximumLength as usize, TAG_IO_RESOURCE) as *mut u16;
    if RootRegKey.Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlAppendUnicodeToString(
        &mut RootRegKey,
        b"\\\0R\0E\0G\0I\0S\0T\0R\0Y\0\\\0M\0A\0C\0H\0I\0N\0E\0\\\0H\0A\0R\0D\0W\0A\0R\0E\0\\\0D\0E\0S\0C\0R\0I\0P\0T\0I\0O\0N\0\\\0S\0Y\0S\0T\0E\0M\0\0\0".as_ptr() as PCWSTR,
    );

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut RootRegKey,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    Status = ZwOpenKey(&mut RootRegHandle, KEY_READ, &mut ObjectAttributes);

    if Status >= 0 {
        Query.BusType = BusType;
        Query.BusNumber = BusNumber;
        Query.ControllerType = ControllerType;
        Query.ControllerNumber = ControllerNumber;
        Query.PeripheralType = PeripheralType;
        Query.PeripheralNumber = PeripheralNumber;
        Query.CalloutRoutine = CalloutRoutine;
        Query.Context = Context;
        Status = IopQueryBusDescription(
            &mut Query,
            RootRegKey,
            RootRegHandle,
            &mut BusLoopNumber,
            1,
        );

        ZwClose(RootRegHandle);
    }

    ExFreePoolWithTag(RootRegKey.Buffer as PVOID, TAG_IO_RESOURCE);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoReportHalResourceUsage(
    HalName: PUNICODE_STRING,
    RawResourceList: PCM_RESOURCE_LIST,
    TranslatedResourceList: PCM_RESOURCE_LIST,
    ResourceListSize: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut Disposition: ULONG = 0;
    let mut ResourceMapKey: HANDLE = core::ptr::null_mut();
    let mut HalKey: HANDLE = core::ptr::null_mut();
    let mut DescriptionKey: HANDLE = core::ptr::null_mut();

    RtlInitUnicodeString(&mut Name, b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0H\0A\0R\0D\0W\0A\0R\0E\0\\\0R\0E\0S\0O\0U\0R\0C\0E\0M\0A\0P\0\0\0".as_ptr() as PCWSTR);
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut Name,
        OBJ_CASE_INSENSITIVE | OBJ_OPENIF,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = ZwCreateKey(
        &mut ResourceMapKey,
        KEY_ALL_ACCESS,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        REG_OPTION_VOLATILE,
        &mut Disposition,
    );
    if Status < 0 {
        return Status;
    }

    RtlInitUnicodeString(&mut Name, b"H\0a\0r\0d\0w\0a\0r\0e\0 \0A\0b\0s\0t\0r\0a\0c\0t\0i\0o\0n\0 \0L\0a\0y\0e\0r\0\0\0".as_ptr() as PCWSTR);
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut Name,
        OBJ_CASE_INSENSITIVE | OBJ_OPENIF,
        ResourceMapKey,
        core::ptr::null_mut(),
    );
    Status = ZwCreateKey(
        &mut HalKey,
        KEY_ALL_ACCESS,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        REG_OPTION_VOLATILE,
        &mut Disposition,
    );
    ZwClose(ResourceMapKey);
    if Status < 0 {
        return Status;
    }

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        HalName,
        OBJ_CASE_INSENSITIVE,
        HalKey,
        core::ptr::null_mut(),
    );
    Status = ZwCreateKey(
        &mut DescriptionKey,
        KEY_ALL_ACCESS,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        REG_OPTION_VOLATILE,
        &mut Disposition,
    );
    ZwClose(HalKey);
    if Status < 0 {
        return Status;
    }

    RtlInitUnicodeString(&mut Name, b".\0R\0a\0w\0\0\0".as_ptr() as PCWSTR);
    Status = ZwSetValueKey(
        DescriptionKey,
        &mut Name,
        0,
        REG_RESOURCE_LIST,
        RawResourceList as PVOID,
        ResourceListSize,
    );
    if Status < 0 {
        ZwClose(DescriptionKey);
        return Status;
    }

    RtlInitUnicodeString(&mut Name, b".\0T\0r\0a\0n\0s\0l\0a\0t\0e\0d\0\0\0".as_ptr() as PCWSTR);
    Status = ZwSetValueKey(
        DescriptionKey,
        &mut Name,
        0,
        REG_RESOURCE_LIST,
        TranslatedResourceList as PVOID,
        ResourceListSize,
    );
    ZwClose(DescriptionKey);

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Hardware resource management
//  * COPYRIGHT:   Copyright 1998 David Welch <welch@mcmail.com>
//  *              Copyright 2001 Eric Kohl <eric.kohl@reactos.org>
//  *              Copyright 2004-2013 Alex Ionescu <alex.ionescu@reactos.org>
//  *              Copyright 2010 Cameron Gutman <cameron.gutman@reactos.org>
//  *              Copyright 2010 Pierre Schweitzer <pierre@reactos.org>
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #ifndef NDEBUG
//     #define IORSRCTRACE(...)    DbgPrint(__VA_ARGS__)
// #else
//     #if defined(_MSC_VER)
//     #define IORSRCTRACE     __noop
//     #else
//     #define IORSRCTRACE(...)    do { if(0) { DbgPrint(__VA_ARGS__); } } while(0)
//     #endif
// #endif
//
// /* GLOBALS *******************************************************************/
//
// static CONFIGURATION_INFORMATION
// _SystemConfigurationInformation = { 0, 0, 0, 0, 0, 0, 0, FALSE, FALSE, 0, 0 };
//
// /* API parameters to pass to IopQueryBusDescription() */
// typedef struct _IO_QUERY
// {
//     PINTERFACE_TYPE BusType;
//     PULONG BusNumber;
//     PCONFIGURATION_TYPE ControllerType;
//     PULONG ControllerNumber;
//     PCONFIGURATION_TYPE PeripheralType;
//     PULONG PeripheralNumber;
//     PIO_QUERY_DEVICE_ROUTINE CalloutRoutine;
//     PVOID Context;
// } IO_QUERY, *PIO_QUERY;
//
// /* Strings corresponding to CONFIGURATION_TYPE */
// PCWSTR ArcTypes[MaximumType + 1] =
// {
//     L"System",
//     L"CentralProcessor",
//     L"FloatingPointProcessor",
//     L"PrimaryICache",
//     L"PrimaryDCache",
//     L"SecondaryICache",
//     L"SecondaryDCache",
//     L"SecondaryCache",
//     L"EisaAdapter",
//     L"TcAdapter",
//     L"ScsiAdapter",
//     L"DtiAdapter",
//     L"MultifunctionAdapter",
//     L"DiskController",
//     L"TapeController",
//     L"CdRomController",
//     L"WormController",
//     L"SerialController",
//     L"NetworkController",
//     L"DisplayController",
//     L"ParallelController",
//     L"PointerController",
//     L"KeyboardController",
//     L"AudioController",
//     L"OtherController",
//     L"DiskPeripheral",
//     L"FloppyDiskPeripheral",
//     L"TapePeripheral",
//     L"ModemPeripheral",
//     L"MonitorPeripheral",
//     L"PrinterPeripheral",
//     L"PointerPeripheral",
//     L"KeyboardPeripheral",
//     L"TerminalPeripheral",
//     L"OtherPeripheral",
//     L"LinePeripheral",
//     L"NetworkPeripheral",
//     L"SystemMemory",
//     L"DockingInformation",
//     L"RealModeIrqRoutingTable",
//     L"RealModePCIEnumeration",
//     L"Undefined"
// };
//
// /* Strings corresponding to IO_QUERY_DEVICE_DATA_FORMAT */
// PCWSTR IoDeviceInfoNames[IoQueryDeviceMaxData] =
// {
//     L"Identifier",
//     L"Configuration Data",
//     L"Component Information"
// };
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// /**
//  * @brief
//  * Reads and returns Hardware information from the appropriate hardware
//  * registry key. Helper stub of IopQueryBusDescription().
//  *
//  * @param[in]   Query
//  * What the parent function wants.
//  *
//  * @param[in]   RootKey
//  * Which key to look in.
//  *
//  * @param[in]   RootKeyHandle
//  * Handle to the key.
//  *
//  * @param[in]   Bus
//  * The bus number.
//  *
//  * @param[in]   BusInformation
//  * The configuration information being sent.
//  *
//  * @return  A status code.
//  **/
// static NTSTATUS
// IopQueryDeviceDescription(
//     _In_ PIO_QUERY Query,
//     _In_ UNICODE_STRING RootKey,
//     _In_ HANDLE RootKeyHandle,
//     _In_ ULONG Bus,
//     _In_ PKEY_VALUE_FULL_INFORMATION* BusInformation)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* Controller data */
//     UNICODE_STRING ControllerString;
//     UNICODE_STRING ControllerRootRegName = RootKey;
//     UNICODE_STRING ControllerRegName;
//     HANDLE ControllerKeyHandle;
//     PKEY_FULL_INFORMATION ControllerFullInformation = NULL;
//     PKEY_VALUE_FULL_INFORMATION ControllerInformation[IoQueryDeviceMaxData] =
//         {NULL, NULL, NULL};
//     ULONG ControllerNumber;
//     ULONG ControllerLoop;
//     ULONG MaximumControllerNumber;
//
//     /* Peripheral data */
//     UNICODE_STRING PeripheralString;
//     HANDLE PeripheralKeyHandle;
//     PKEY_FULL_INFORMATION PeripheralFullInformation;
//     PKEY_VALUE_FULL_INFORMATION PeripheralInformation[IoQueryDeviceMaxData] =
//         {NULL, NULL, NULL};
//     ULONG PeripheralNumber;
//     ULONG PeripheralLoop;
//     ULONG MaximumPeripheralNumber;
//
//     /* Global Registry data */
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     ULONG LenFullInformation;
//     ULONG LenKeyFullInformation;
//     UNICODE_STRING TempString;
//     WCHAR TempBuffer[14];
//
//     IORSRCTRACE("\nIopQueryDeviceDescription(Query: 0x%p)\n"
//                 "    RootKey: '%wZ'\n"
//                 "    RootKeyHandle: 0x%p\n"
//                 "    Bus: %lu\n",
//                 Query,
//                 &RootKey, RootKeyHandle,
//                 Bus);
//
//     /* Temporary string */
//     RtlInitEmptyUnicodeString(&TempString, TempBuffer, sizeof(TempBuffer));
//
//     /* Append controller name to string */
//     RtlAppendUnicodeToString(&ControllerRootRegName, L"\\");
//     RtlAppendUnicodeToString(&ControllerRootRegName, ArcTypes[*Query->ControllerType]);
//
//     /* Set the controller number if specified */
//     if (Query->ControllerNumber)
//     {
//         ControllerNumber = *(Query->ControllerNumber);
//         MaximumControllerNumber = ControllerNumber + 1;
//         IORSRCTRACE("    Getting controller #%lu\n", ControllerNumber);
//     }
//     else
//     {
//         IORSRCTRACE("    Enumerating controllers in '%wZ'...\n", &ControllerRootRegName);
//
//         /* Find out how many controllers there are */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &ControllerRootRegName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    NULL,
//                                    NULL);
//
//         Status = ZwOpenKey(&ControllerKeyHandle, KEY_READ, &ObjectAttributes);
//
//         if (NT_SUCCESS(Status))
//         {
//             /* Retrieve the necessary buffer space */
//             ZwQueryKey(ControllerKeyHandle,
//                        KeyFullInformation,
//                        NULL, 0,
//                        &LenFullInformation);
//
//             /* Allocate it */
//             ControllerFullInformation = ExAllocatePoolWithTag(PagedPool, LenFullInformation as usize, TAG_IO_RESOURCE);
//             if (!ControllerFullInformation)
//             {
//                 ZwClose(ControllerKeyHandle);
//                 return STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             /* Get the information */
//             Status = ZwQueryKey(ControllerKeyHandle,
//                                 KeyFullInformation,
//                                 ControllerFullInformation,
//                                 LenFullInformation,
//                                 &LenFullInformation);
//             ZwClose(ControllerKeyHandle);
//             ControllerKeyHandle = NULL;
//         }
//
//         /* No controller was found, bail out */
//         if (!NT_SUCCESS(Status))
//         {
//             if (ControllerFullInformation)
//                 ExFreePoolWithTag(ControllerFullInformation, TAG_IO_RESOURCE);
//             return Status;
//         }
//
//         /* Find out the controllers */
//         ControllerNumber = 0;
//         MaximumControllerNumber = ControllerFullInformation->SubKeys;
//
//         /* Cleanup */
//         ExFreePoolWithTag(ControllerFullInformation, TAG_IO_RESOURCE);
//         ControllerFullInformation = NULL;
//     }
//
//     /* Save string */
//     ControllerRegName = ControllerRootRegName;
//
//     /* Loop through controllers */
//     for (; ControllerNumber < MaximumControllerNumber; ControllerNumber++)
//     {
//         /* Load string */
//         ControllerRootRegName = ControllerRegName;
//
//         /* Convert controller number to registry string */
//         Status = RtlIntegerToUnicodeString(ControllerNumber, 10, &TempString);
//
//         /* Create string */
//         Status |= RtlAppendUnicodeToString(&ControllerRootRegName, L"\\");
//         Status |= RtlAppendUnicodeStringToString(&ControllerRootRegName, &TempString);
//
//         /* Something messed up */
//         if (!NT_SUCCESS(Status))
//             break;
//
//         IORSRCTRACE("    Retrieving controller '%wZ'\n", &ControllerRootRegName);
//
//         /* Open the registry key */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &ControllerRootRegName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    NULL,
//                                    NULL);
//
//         Status = ZwOpenKey(&ControllerKeyHandle, KEY_READ, &ObjectAttributes);
//
//         /* Read the configuration data */
//         if (NT_SUCCESS(Status))
//         {
//             for (ControllerLoop = 0; ControllerLoop < RTL_NUMBER_OF(IoDeviceInfoNames); ControllerLoop++)
//             {
//                 /* Identifier string first */
//                 RtlInitUnicodeString(&ControllerString, IoDeviceInfoNames[ControllerLoop]);
//
//                 /* Retrieve the necessary buffer space */
//                 Status = ZwQueryValueKey(ControllerKeyHandle,
//                                          &ControllerString,
//                                          KeyValueFullInformation,
//                                          NULL, 0,
//                                          &LenKeyFullInformation);
//
//                 if (!NT_SUCCESS(Status) &&
//                     (Status != STATUS_BUFFER_TOO_SMALL) &&
//                     (Status != STATUS_BUFFER_OVERFLOW))
//                 {
//                     ControllerInformation[ControllerLoop] = NULL;
//                     continue;
//                 }
//
//                 /* Allocate it */
//                 ControllerInformation[ControllerLoop] = ExAllocatePoolWithTag(PagedPool, LenKeyFullInformation, TAG_IO_RESOURCE);
//                 if (!ControllerInformation[ControllerLoop])
//                 {
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                     break;
//                 }
//
//                 /* Get the information */
//                 Status = ZwQueryValueKey(ControllerKeyHandle,
//                                          &ControllerString,
//                                          KeyValueFullInformation,
//                                          ControllerInformation[ControllerLoop],
//                                          LenKeyFullInformation,
//                                          &LenKeyFullInformation);
//             }
//
//             /* Cleanup */
//             ZwClose(ControllerKeyHandle);
//             ControllerKeyHandle = NULL;
//         }
//
//         /* Something messed up */
//         if (!NT_SUCCESS(Status))
//             goto EndLoop;
//
//         /* We now have bus *AND* controller information, is it enough? */
//         if (!Query->PeripheralType || !(*Query->PeripheralType))
//         {
//             IORSRCTRACE("    --> Bus #%lu Controller #%lu Callout: '%wZ'\n",
//                         Bus, ControllerNumber, &ControllerRootRegName);
//
//             Status = Query->CalloutRoutine(Query->Context,
//                                            &ControllerRootRegName,
//                                            *Query->BusType,
//                                            Bus,
//                                            BusInformation,
//                                            *Query->ControllerType,
//                                            ControllerNumber,
//                                            ControllerInformation,
//                                            0,
//                                            0,
//                                            NULL);
//             goto EndLoop;
//         }
//
//         /* Not enough: the caller also wants peripheral name */
//         Status = RtlAppendUnicodeToString(&ControllerRootRegName, L"\\");
//         Status |= RtlAppendUnicodeToString(&ControllerRootRegName, ArcTypes[*Query->PeripheralType]);
//
//         /* Something messed up */
//         if (!NT_SUCCESS(Status))
//             goto EndLoop;
//
//         /* Set the peripheral number if specified */
//         if (Query->PeripheralNumber)
//         {
//             PeripheralNumber = *(Query->PeripheralNumber);
//             MaximumPeripheralNumber = PeripheralNumber + 1;
//             IORSRCTRACE("    Getting peripheral #%lu\n", PeripheralNumber);
//         }
//         else
//         {
//             IORSRCTRACE("    Enumerating peripherals in '%wZ'...\n", &ControllerRootRegName);
//
//             /* Find out how many peripherals there are */
//             InitializeObjectAttributes(&ObjectAttributes,
//                                        &ControllerRootRegName,
//                                        OBJ_CASE_INSENSITIVE,
//                                        NULL,
//                                        NULL);
//
//             Status = ZwOpenKey(&PeripheralKeyHandle, KEY_READ, &ObjectAttributes);
//
//             if (NT_SUCCESS(Status))
//             {
//                 /* Retrieve the necessary buffer space */
//                 ZwQueryKey(PeripheralKeyHandle,
//                            KeyFullInformation,
//                            NULL, 0,
//                            &LenFullInformation);
//
//                 /* Allocate it */
//                 PeripheralFullInformation = ExAllocatePoolWithTag(PagedPool, LenFullInformation as usize, TAG_IO_RESOURCE);
//                 if (!PeripheralFullInformation)
//                 {
//                     ZwClose(PeripheralKeyHandle);
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                     goto EndLoop;
//                 }
//
//                 /* Get the information */
//                 Status = ZwQueryKey(PeripheralKeyHandle,
//                                     KeyFullInformation,
//                                     PeripheralFullInformation,
//                                     LenFullInformation,
//                                     &LenFullInformation);
//                 ZwClose(PeripheralKeyHandle);
//                 PeripheralKeyHandle = NULL;
//             }
//
//             /* No controller was found, cleanup and bail out */
//             if (!NT_SUCCESS(Status))
//             {
//                 Status = STATUS_SUCCESS;
//                 goto EndLoop;
//             }
//
//             /* Find out peripheral number */
//             PeripheralNumber = 0;
//             MaximumPeripheralNumber = PeripheralFullInformation->SubKeys;
//
//             /* Cleanup */
//             ExFreePoolWithTag(PeripheralFullInformation, TAG_IO_RESOURCE);
//             PeripheralFullInformation = NULL;
//         }
//
//         /* Save name */
//         ControllerRegName = ControllerRootRegName;
//
//         /* Loop through peripherals */
//         for (; PeripheralNumber < MaximumPeripheralNumber; PeripheralNumber++)
//         {
//             /* Restore name */
//             ControllerRootRegName = ControllerRegName;
//
//             /* Convert peripheral number to registry string */
//             Status = RtlIntegerToUnicodeString(PeripheralNumber, 10, &TempString);
//
//             /* Create string */
//             Status |= RtlAppendUnicodeToString(&ControllerRootRegName, L"\\");
//             Status |= RtlAppendUnicodeStringToString(&ControllerRootRegName, &TempString);
//
//             /* Something messed up */
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             IORSRCTRACE("    Retrieving peripheral '%wZ'\n", &ControllerRootRegName);
//
//             /* Open the registry key */
//             InitializeObjectAttributes(&ObjectAttributes,
//                                        &ControllerRootRegName,
//                                        OBJ_CASE_INSENSITIVE,
//                                        NULL,
//                                        NULL);
//
//             Status = ZwOpenKey(&PeripheralKeyHandle, KEY_READ, &ObjectAttributes);
//
//             if (NT_SUCCESS(Status))
//             {
//                 for (PeripheralLoop = 0; PeripheralLoop < RTL_NUMBER_OF(IoDeviceInfoNames); PeripheralLoop++)
//                 {
//                     /* Identifier string first */
//                     RtlInitUnicodeString(&PeripheralString, IoDeviceInfoNames[PeripheralLoop]);
//
//                     /* Retrieve the necessary buffer space */
//                     Status = ZwQueryValueKey(PeripheralKeyHandle,
//                                              &PeripheralString,
//                                              KeyValueFullInformation,
//                                              NULL, 0,
//                                              &LenKeyFullInformation);
//
//                     if (!NT_SUCCESS(Status) &&
//                         (Status != STATUS_BUFFER_TOO_SMALL) &&
//                         (Status != STATUS_BUFFER_OVERFLOW))
//                     {
//                         PeripheralInformation[PeripheralLoop] = NULL;
//                         continue;
//                     }
//
//                     /* Allocate it */
//                     PeripheralInformation[PeripheralLoop] = ExAllocatePoolWithTag(PagedPool, LenKeyFullInformation, TAG_IO_RESOURCE);
//                     if (!PeripheralInformation[PeripheralLoop])
//                     {
//                         Status = STATUS_INSUFFICIENT_RESOURCES;
//                         break;
//                     }
//
//                     /* Get the information */
//                     Status = ZwQueryValueKey(PeripheralKeyHandle,
//                                              &PeripheralString,
//                                              KeyValueFullInformation,
//                                              PeripheralInformation[PeripheralLoop],
//                                              LenKeyFullInformation,
//                                              &LenKeyFullInformation);
//                 }
//
//                 /* Cleanup */
//                 ZwClose(PeripheralKeyHandle);
//                 PeripheralKeyHandle = NULL;
//
//                 /* We now have everything the caller could possibly want */
//                 if (NT_SUCCESS(Status))
//                 {
//                     IORSRCTRACE("    --> Bus #%lu Controller #%lu Peripheral #%lu Callout: '%wZ'\n",
//                                 Bus, ControllerNumber, PeripheralNumber, &ControllerRootRegName);
//
//                     Status = Query->CalloutRoutine(Query->Context,
//                                                    &ControllerRootRegName,
//                                                    *Query->BusType,
//                                                    Bus,
//                                                    BusInformation,
//                                                    *Query->ControllerType,
//                                                    ControllerNumber,
//                                                    ControllerInformation,
//                                                    *Query->PeripheralType,
//                                                    PeripheralNumber,
//                                                    PeripheralInformation);
//                 }
//
//                 /* Free the allocated memory */
//                 for (PeripheralLoop = 0; PeripheralLoop < RTL_NUMBER_OF(IoDeviceInfoNames); PeripheralLoop++)
//                 {
//                     if (PeripheralInformation[PeripheralLoop])
//                     {
//                         ExFreePoolWithTag(PeripheralInformation[PeripheralLoop], TAG_IO_RESOURCE);
//                         PeripheralInformation[PeripheralLoop] = NULL;
//                     }
//                 }
//
//                 /* Something messed up */
//                 if (!NT_SUCCESS(Status))
//                     break;
//             }
//         }
//
// EndLoop:
//         /* Free the allocated memory */
//         for (ControllerLoop = 0; ControllerLoop < RTL_NUMBER_OF(IoDeviceInfoNames); ControllerLoop++)
//         {
//             if (ControllerInformation[ControllerLoop])
//             {
//                 ExFreePoolWithTag(ControllerInformation[ControllerLoop], TAG_IO_RESOURCE);
//                 ControllerInformation[ControllerLoop] = NULL;
//             }
//         }
//
//         /* Something messed up */
//         if (!NT_SUCCESS(Status))
//             break;
//     }
//
//     return Status;
// }
//
// /**
//  * @brief
//  * Reads and returns Hardware information from the appropriate hardware
//  * registry key. Helper stub of IoQueryDeviceDescription(). Has two modes
//  * of operation, either looking for root bus types or for sub-bus
//  * information.
//  *
//  * @param[in]   Query
//  * What the parent function wants.
//  *
//  * @param[in]   RootKey
//  * Which key to look in.
//  *
//  * @param[in]   RootKeyHandle
//  * Handle to the key.
//  *
//  * @param[in,out]   Bus
//  * Pointer to the current bus number.
//  *
//  * @param[in]   KeyIsRoot
//  * Whether we are looking for root bus types or information under them.
//  *
//  * @return  A status code.
//  **/
// static NTSTATUS
// IopQueryBusDescription(
//     _In_ PIO_QUERY Query,
//     _In_ UNICODE_STRING RootKey,
//     _In_ HANDLE RootKeyHandle,
//     _Inout_ PULONG Bus,
//     _In_ BOOLEAN KeyIsRoot)
// {
//     NTSTATUS Status;
//     ULONG BusLoop;
//     UNICODE_STRING SubRootRegName;
//     UNICODE_STRING BusString;
//     UNICODE_STRING SubBusString;
//     ULONG LenBasicInformation = 0;
//     ULONG LenFullInformation;
//     ULONG LenKeyFullInformation;
//     ULONG LenKey;
//     HANDLE SubRootKeyHandle;
//     PKEY_FULL_INFORMATION FullInformation;
//     PKEY_BASIC_INFORMATION BasicInformation = NULL;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PKEY_VALUE_FULL_INFORMATION BusInformation[IoQueryDeviceMaxData] =
//         {NULL, NULL, NULL};
//
//     IORSRCTRACE("\nIopQueryBusDescription(Query: 0x%p)\n"
//                 "    RootKey: '%wZ'\n"
//                 "    RootKeyHandle: 0x%p\n"
//                 "    KeyIsRoot: %s\n"
//                 "    Bus: 0x%p (%lu)\n",
//                 Query,
//                 &RootKey, RootKeyHandle,
//                 KeyIsRoot ? "TRUE" : "FALSE",
//                 Bus, Bus ? *Bus : -1);
//
//     /* Retrieve the necessary buffer space */
//     Status = ZwQueryKey(RootKeyHandle,
//                         KeyFullInformation,
//                         NULL, 0,
//                         &LenFullInformation);
//
//     if (!NT_SUCCESS(Status) &&
//         (Status != STATUS_BUFFER_TOO_SMALL) &&
//         (Status != STATUS_BUFFER_OVERFLOW))
//     {
//         return Status;
//     }
//
//     /* Allocate it */
//     FullInformation = ExAllocatePoolWithTag(PagedPool, LenFullInformation as usize, TAG_IO_RESOURCE);
//     if (!FullInformation)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Get the information */
//     Status = ZwQueryKey(RootKeyHandle,
//                         KeyFullInformation,
//                         FullInformation,
//                         LenFullInformation,
//                         &LenFullInformation);
//     if (NT_SUCCESS(Status))
//     {
//         /* Buffer needed for all the keys under this one */
//         LenBasicInformation = FullInformation->MaxNameLen + sizeof(KEY_BASIC_INFORMATION);
//
//         /* Allocate it */
//         BasicInformation = ExAllocatePoolWithTag(PagedPool, LenBasicInformation as usize, TAG_IO_RESOURCE);
//         if (!BasicInformation)
//         {
//             ExFreePoolWithTag(FullInformation, TAG_IO_RESOURCE);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//
//     /* Deallocate the old buffer */
//     ExFreePoolWithTag(FullInformation, TAG_IO_RESOURCE);
//
//     /* Try to find a bus */
//     for (BusLoop = 0; NT_SUCCESS(Status); BusLoop++)
//     {
//         /* Bus parameter was passed and number was matched */
//         if (Query->BusNumber && (*(Query->BusNumber) == *Bus))
//             break;
//
//         /* Enumerate the Key */
//         Status = ZwEnumerateKey(RootKeyHandle,
//                                 BusLoop,
//                                 KeyBasicInformation,
//                                 BasicInformation,
//                                 LenBasicInformation,
//                                 &LenKey);
//
//         /* Stop if everything was enumerated */
//         if (!NT_SUCCESS(Status))
//             break;
//
//         IORSRCTRACE("    Seen: '%.*ws'\n", BasicInformation->NameLength/sizeof(WCHAR), BasicInformation->Name);
//
//         /* What bus are we going to go down? (only check if this is a root key) */
//         if (KeyIsRoot)
//         {
//             if (wcsncmp(BasicInformation->Name, L"MultifunctionAdapter", BasicInformation->NameLength / sizeof(WCHAR)) &&
//                 wcsncmp(BasicInformation->Name, L"EisaAdapter", BasicInformation->NameLength / sizeof(WCHAR)) &&
//                 wcsncmp(BasicInformation->Name, L"TcAdapter", BasicInformation->NameLength / sizeof(WCHAR)))
//             {
//                 /* Nothing found, check next */
//                 continue;
//             }
//         }
//
//         /* Enumerate the bus */
//         BusString.Buffer = BasicInformation->Name;
//         BusString.Length = (USHORT)BasicInformation->NameLength;
//         BusString.MaximumLength = (USHORT)BasicInformation->NameLength;
//
//         /* Open a handle to the root registry key */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &BusString,
//                                    OBJ_CASE_INSENSITIVE,
//                                    RootKeyHandle,
//                                    NULL);
//
//         Status = ZwOpenKey(&SubRootKeyHandle, KEY_READ, &ObjectAttributes);
//
//         /* Go on if we failed */
//         if (!NT_SUCCESS(Status))
//             continue;
//
//         /* Key opened, create the path */
//         SubRootRegName = RootKey;
//         RtlAppendUnicodeToString(&SubRootRegName, L"\\");
//         RtlAppendUnicodeStringToString(&SubRootRegName, &BusString);
//
//         IORSRCTRACE("    SubRootRegName: '%wZ'\n", &SubRootRegName);
//
//         if (!KeyIsRoot)
//         {
//             /* Parsing a sub-bus key */
//             ULONG SubBusLoop;
//             for (SubBusLoop = 0; SubBusLoop < RTL_NUMBER_OF(IoDeviceInfoNames); SubBusLoop++)
//             {
//                 /* Identifier string first */
//                 RtlInitUnicodeString(&SubBusString, IoDeviceInfoNames[SubBusLoop]);
//
//                 IORSRCTRACE("    Getting bus value: '%wZ'\n", &SubBusString);
//
//                 /* Retrieve the necessary buffer space */
//                 ZwQueryValueKey(SubRootKeyHandle,
//                                 &SubBusString,
//                                 KeyValueFullInformation,
//                                 NULL, 0,
//                                 &LenKeyFullInformation);
//
//                 /* Allocate it */
//                 BusInformation[SubBusLoop] = ExAllocatePoolWithTag(PagedPool, LenKeyFullInformation, TAG_IO_RESOURCE);
//                 if (!BusInformation[SubBusLoop])
//                 {
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                     break;
//                 }
//
//                 /* Get the information */
//                 Status = ZwQueryValueKey(SubRootKeyHandle,
//                                          &SubBusString,
//                                          KeyValueFullInformation,
//                                          BusInformation[SubBusLoop],
//                                          LenKeyFullInformation,
//                                          &LenKeyFullInformation);
//             }
//
//             if (NT_SUCCESS(Status))
//             {
//                 PKEY_VALUE_FULL_INFORMATION BusConfigData =
//                     BusInformation[IoQueryDeviceConfigurationData];
//
//                 /* Do we have something? */
//                 if (BusConfigData != NULL &&
//                     BusConfigData->DataLength != 0 &&
//                     /* Does it match what we want? */
//                     (((PCM_FULL_RESOURCE_DESCRIPTOR)((ULONG_PTR)BusConfigData +
//                         BusConfigData->DataOffset))->InterfaceType == *(Query->BusType)))
//                 {
//                     /* Found a bus */
//                     (*Bus)++;
//
//                     /* Is it the bus we wanted? */
//                     if (Query->BusNumber == NULL || *(Query->BusNumber) == *Bus)
//                     {
//                         if (Query->ControllerType == NULL)
//                         {
//                             IORSRCTRACE("    --> Bus #%lu Callout: '%wZ'\n", *Bus, &SubRootRegName);
//
//                             /* We don't want controller information: call the callback */
//                             Status = Query->CalloutRoutine(Query->Context,
//                                                            &SubRootRegName,
//                                                            *(Query->BusType),
//                                                            *Bus,
//                                                            BusInformation,
//                                                            0,
//                                                            0,
//                                                            NULL,
//                                                            0,
//                                                            0,
//                                                            NULL);
//                         }
//                         else
//                         {
//                             IORSRCTRACE("    --> Getting device on Bus #%lu : '%wZ'\n", *Bus, &SubRootRegName);
//
//                             /* We want controller information: get it */
//                             Status = IopQueryDeviceDescription(Query,
//                                                                SubRootRegName,
//                                                                RootKeyHandle,
//                                                                *Bus,
//                                                                (PKEY_VALUE_FULL_INFORMATION*)BusInformation);
//                         }
//                     }
//                 }
//             }
//
//             /* Free the allocated memory */
//             for (SubBusLoop = 0; SubBusLoop < RTL_NUMBER_OF(IoDeviceInfoNames); SubBusLoop++)
//             {
//                 if (BusInformation[SubBusLoop])
//                 {
//                     ExFreePoolWithTag(BusInformation[SubBusLoop], TAG_IO_RESOURCE);
//                     BusInformation[SubBusLoop] = NULL;
//                 }
//             }
//
//             /* Exit the loop if we found the bus */
//             if (Query->BusNumber && (*(Query->BusNumber) == *Bus))
//             {
//                 ZwClose(SubRootKeyHandle);
//                 SubRootKeyHandle = NULL;
//                 continue;
//             }
//         }
//
//         /* Enumerate the buses below us recursively if we haven't found the bus yet */
//         Status = IopQueryBusDescription(Query, SubRootRegName, SubRootKeyHandle, Bus, !KeyIsRoot);
//
//         /* Everything enumerated */
//         if (Status == STATUS_NO_MORE_ENTRIES)
//             Status = STATUS_SUCCESS;
//
//         ZwClose(SubRootKeyHandle);
//         SubRootKeyHandle = NULL;
//     }
//
//     /* Free the last remaining allocated memory */
//     if (BasicInformation)
//         ExFreePoolWithTag(BasicInformation, TAG_IO_RESOURCE);
//
//     return Status;
// }
//
// NTSTATUS
// IopFetchConfigurationInformation(
//     _Out_ PWSTR* SymbolicLinkList,
//     _In_ GUID Guid,
//     _In_ ULONG ExpectedInterfaces,
//     _Out_ PULONG Interfaces)
// {
//     NTSTATUS Status;
//     ULONG interfaces = 0;
//     PWSTR symbolicLinkList;
//
//     /* Get the associated enabled interfaces with the given GUID */
//     Status = IoGetDeviceInterfaces(&Guid, NULL, 0, SymbolicLinkList);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Zero output and leave */
//         if (SymbolicLinkList)
//             *SymbolicLinkList = NULL;
//
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     symbolicLinkList = *SymbolicLinkList;
//
//     /* Count the number of enabled interfaces by counting the number of symbolic links */
//     while (*symbolicLinkList != UNICODE_NULL)
//     {
//         interfaces++;
//         symbolicLinkList += (wcslen(symbolicLinkList) + 1);
//     }
//
//     /* Matching result will define the result */
//     Status = (interfaces >= ExpectedInterfaces) ? STATUS_SUCCESS : STATUS_UNSUCCESSFUL;
//     /* Finally, give back to the caller the number of found interfaces */
//     *Interfaces = interfaces;
//
//     return Status;
// }
//
// VOID
// IopStoreSystemPartitionInformation(
//     _In_ PUNICODE_STRING NtSystemPartitionDeviceName,
//     _In_ PUNICODE_STRING OsLoaderPathName)
// {
//     NTSTATUS Status;
//     UNICODE_STRING LinkTarget, KeyName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE LinkHandle, RegistryHandle, KeyHandle;
//     WCHAR LinkTargetBuffer[256];
//     UNICODE_STRING CmRegistryMachineSystemName = RTL_CONSTANT_STRING(L"\\Registry\\Machine\\SYSTEM");
//
//     ASSERT(NtSystemPartitionDeviceName->MaximumLength >= NtSystemPartitionDeviceName->Length + sizeof(WCHAR));
//     ASSERT(NtSystemPartitionDeviceName->Buffer[NtSystemPartitionDeviceName->Length / sizeof(WCHAR)] == UNICODE_NULL);
//     ASSERT(OsLoaderPathName->MaximumLength >= OsLoaderPathName->Length + sizeof(WCHAR));
//     ASSERT(OsLoaderPathName->Buffer[OsLoaderPathName->Length / sizeof(WCHAR)] == UNICODE_NULL);
//
//     /* First define needed stuff to open NtSystemPartitionDeviceName symbolic link */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                NtSystemPartitionDeviceName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//
//     /* Open NtSystemPartitionDeviceName symbolic link */
//     Status = ZwOpenSymbolicLinkObject(&LinkHandle,
//                                       SYMBOLIC_LINK_QUERY,
//                                       &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("Failed to open symlink %wZ, Status=%lx\n", NtSystemPartitionDeviceName, Status);
//         return;
//     }
//
//     /* Prepare the string that will receive where symbolic link points to.
//      * We will zero the end of the string after having received it */
//     RtlInitEmptyUnicodeString(&LinkTarget, LinkTargetBuffer,
//                               sizeof(LinkTargetBuffer) - sizeof(UNICODE_NULL));
//
//     /* Query target */
//     Status = ZwQuerySymbolicLinkObject(LinkHandle, &LinkTarget, NULL);
//
//     /* We are done with symbolic link */
//     ObCloseHandle(LinkHandle, KernelMode);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("Failed querying symlink %wZ, Status=%lx\n", NtSystemPartitionDeviceName, Status);
//         return;
//     }
//
//     /* As promised, we zero the end */
//     LinkTarget.Buffer[LinkTarget.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     /* Open registry to save data (HKLM\SYSTEM) */
//     Status = IopOpenRegistryKeyEx(&RegistryHandle,
//                                   NULL,
//                                   &CmRegistryMachineSystemName,
//                                   KEY_ALL_ACCESS);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("Failed to open HKLM\\SYSTEM, Status=%lx\n", Status);
//         return;
//     }
//
//     /* Open or create the Setup subkey where we'll store in */
//     RtlInitUnicodeString(&KeyName, L"Setup");
//
//     Status = IopCreateRegistryKeyEx(&KeyHandle,
//                                     RegistryHandle,
//                                     &KeyName,
//                                     KEY_ALL_ACCESS,
//                                     REG_OPTION_NON_VOLATILE,
//                                     NULL);
//
//     /* We're done with HKLM\SYSTEM */
//     ObCloseHandle(RegistryHandle, KernelMode);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("Failed opening/creating Setup key, Status=%lx\n", Status);
//         return;
//     }
//
//     /* Prepare first data writing */
//     RtlInitUnicodeString(&KeyName, L"SystemPartition");
//
//     /* Write SystemPartition value which is the target of the symbolic link */
//     Status = ZwSetValueKey(KeyHandle,
//                            &KeyName,
//                            0,
//                            REG_SZ,
//                            LinkTarget.Buffer,
//                            LinkTarget.Length + sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("Failed writing SystemPartition value, Status=%lx\n", Status);
//     }
//
//     /* Prepare for second data writing */
//     RtlInitUnicodeString(&KeyName, L"OsLoaderPath");
//
//     /* Remove trailing slash if any (one slash only excepted) */
//     if (OsLoaderPathName->Length > sizeof(WCHAR) &&
//         OsLoaderPathName->Buffer[(OsLoaderPathName->Length / sizeof(WCHAR)) - 1] == OBJ_NAME_PATH_SEPARATOR)
//     {
//         OsLoaderPathName->Length -= sizeof(WCHAR);
//         OsLoaderPathName->Buffer[OsLoaderPathName->Length / sizeof(WCHAR)] = UNICODE_NULL;
//     }
//
//     /* Then, write down data */
//     Status = ZwSetValueKey(KeyHandle,
//                            &KeyName,
//                            0,
//                            REG_SZ,
//                            OsLoaderPathName->Buffer,
//                            OsLoaderPathName->Length + sizeof(UNICODE_NULL));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("Failed writing OsLoaderPath value, Status=%lx\n", Status);
//     }
//
//     /* We're finally done! */
//     ObCloseHandle(KeyHandle, KernelMode);
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /**
//  * @brief
//  * Returns a pointer to the I/O manager's global configuration
//  * information structure.
//  *
//  * This structure contains the current values for how many physical storage
//  * media, SCSI HBA, serial, and parallel devices have device objects created
//  * to represent them by drivers as they are loaded.
//  **/
// PCONFIGURATION_INFORMATION
// NTAPI
// IoGetConfigurationInformation(VOID)
// {
//     return &_SystemConfigurationInformation;
// }
//
// /**
//  * @halfplemented
//  *
//  * @brief
//  * Reports hardware resources in the \Registry\Machine\Hardware\ResourceMap
//  * tree, so that a subsequently loaded driver cannot attempt to use the
//  * same resources.
//  *
//  * @param[in]   DriverClassName
//  * The driver class under which the resource information should be stored.
//  *
//  * @param[in]   DriverObject
//  * The driver object that was provided to the DriverEntry routine.
//  *
//  * @param[in]   DriverList
//  * Resources claimed for the all the driver's devices, rather than per-device.
//  *
//  * @param[in]   DriverListSize
//  * Size in bytes of the DriverList.
//  *
//  * @param[in]   DeviceObject
//  * The device object for which resources should be claimed.
//  *
//  * @param[in]   DeviceList
//  * List of resources that should be claimed for the device.
//  *
//  * @param[in]   DeviceListSize
//  * Size of the per-device resource list in bytes.
//  *
//  * @param[in]   OverrideConflict
//  * TRUE if the resources should be claimed even if a conflict is found.
//  *
//  * @param[out]  ConflictDetected
//  * Points to a variable that receives TRUE if a conflict is detected
//  * with another driver.
//  **/
// NTSTATUS
// NTAPI
// IoReportResourceUsage(
//     _In_opt_ PUNICODE_STRING DriverClassName,
//     _In_ PDRIVER_OBJECT DriverObject,
//     _In_reads_bytes_opt_(DriverListSize) PCM_RESOURCE_LIST DriverList,
//     _In_opt_ ULONG DriverListSize,
//     _In_opt_ PDEVICE_OBJECT DeviceObject,
//     _In_reads_bytes_opt_(DeviceListSize) PCM_RESOURCE_LIST DeviceList,
//     _In_opt_ ULONG DeviceListSize,
//     _In_ BOOLEAN OverrideConflict,
//     _Out_ PBOOLEAN ConflictDetected)
// {
//     NTSTATUS Status;
//     PCM_RESOURCE_LIST ResourceList;
//
//     DPRINT1("IoReportResourceUsage is halfplemented!\n");
//
//     if (!DriverList && !DeviceList)
//         return STATUS_INVALID_PARAMETER;
//
//     if (DeviceList)
//         ResourceList = DeviceList;
//     else
//         ResourceList = DriverList;
//
//     Status = IopDetectResourceConflict(ResourceList, FALSE, NULL);
//     if (Status == STATUS_CONFLICTING_ADDRESSES)
//     {
//         *ConflictDetected = TRUE;
//
//         if (!OverrideConflict)
//         {
//             DPRINT1("Denying an attempt to claim resources currently in use by another device!\n");
//             return STATUS_CONFLICTING_ADDRESSES;
//         }
//         else
//         {
//             DPRINT1("Proceeding with conflicting resources\n");
//         }
//     }
//     else if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* TODO: Claim resources in registry */
//
//     *ConflictDetected = FALSE;
//
//     return STATUS_SUCCESS;
// }
//
// static NTSTATUS
// IopLegacyResourceAllocation(
//     _In_ ARBITER_REQUEST_SOURCE AllocationType,
//     _In_ PDRIVER_OBJECT DriverObject,
//     _In_opt_ PDEVICE_OBJECT DeviceObject,
//     _In_opt_ PIO_RESOURCE_REQUIREMENTS_LIST ResourceRequirements,
//     _Inout_ PCM_RESOURCE_LIST* AllocatedResources)
// {
//     NTSTATUS Status;
//
//     DPRINT1("IopLegacyResourceAllocation is halfplemented!\n");
//
//     if (!ResourceRequirements)
//     {
//         /* We can get there by calling IoAssignResources() with RequestedResources = NULL.
//          * TODO: not sure what we should do, but we shouldn't crash.
//          */
//         UNIMPLEMENTED;
//         return STATUS_NOT_IMPLEMENTED;
//     }
//
//     Status = IopFixupResourceListWithRequirements(ResourceRequirements,
//                                                   AllocatedResources);
//     if (!NT_SUCCESS(Status))
//     {
//         if (Status == STATUS_CONFLICTING_ADDRESSES)
//         {
//             DPRINT1("Denying an attempt to claim resources currently in use by another device!\n");
//         }
//
//         return Status;
//     }
//
//     /* TODO: Claim resources in registry */
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoAssignResources(
//     _In_ PUNICODE_STRING RegistryPath,
//     _In_opt_ PUNICODE_STRING DriverClassName,
//     _In_ PDRIVER_OBJECT DriverObject,
//     _In_opt_ PDEVICE_OBJECT DeviceObject,
//     _In_opt_ PIO_RESOURCE_REQUIREMENTS_LIST RequestedResources,
//     _Inout_ PCM_RESOURCE_LIST* AllocatedResources)
// {
//     PDEVICE_NODE DeviceNode;
//
//     UNREFERENCED_PARAMETER(RegistryPath);
//     UNREFERENCED_PARAMETER(DriverClassName);
//
//     /* Do we have a DO? */
//     if (DeviceObject)
//     {
//         /* Get its device node */
//         DeviceNode = IopGetDeviceNode(DeviceObject);
//         if (DeviceNode && !(DeviceNode->Flags & DNF_LEGACY_RESOURCE_DEVICENODE))
//         {
//             /* New drivers should not call this API */
//             KeBugCheckEx(PNP_DETECTED_FATAL_ERROR,
//                          0x2,
//                          (ULONG_PTR)DeviceObject,
//                          (ULONG_PTR)DriverObject,
//                          0);
//         }
//     }
//
//     /* Did the driver supply resources? */
//     if (RequestedResources)
//     {
//         /* Make sure there's actually something useful in them */
//         if (!(RequestedResources->AlternativeLists) || !(RequestedResources->List[0].Count))
//         {
//             /* Empty resources are no resources */
//             RequestedResources = NULL;
//         }
//     }
//
//     /* Initialize output if given */
//     if (AllocatedResources)
//         *AllocatedResources = NULL;
//
//     /* Call internal helper function */
//     return IopLegacyResourceAllocation(ArbiterRequestLegacyAssigned,
//                                        DriverObject,
//                                        DeviceObject,
//                                        RequestedResources,
//                                        AllocatedResources);
// }
//
// /**
//  * @brief
//  * Reads and returns Hardware information from the appropriate
//  * hardware registry key.
//  *
//  * @param[in]   BusType
//  * Specifies the bus type, for example: MCA, ISA, EISA, etc.
//  *
//  * @param[in]   BusNumber
//  * The number of the specified bus type to query.
//  *
//  * @param[in]   ControllerType
//  * Specifies the controller type
//  *
//  * @param[in]   ControllerNumber
//  * The number of the specified controller type to query.
//  *
//  * @param[in]   CalloutRoutine
//  * A user-provided callback function to call for each valid query.
//  *
//  * @param[in]   Context
//  * A callback-specific context value.
//  *
//  * @return  A status code.
//  **/
// NTSTATUS
// NTAPI
// IoQueryDeviceDescription(
//     _In_opt_ PINTERFACE_TYPE BusType,
//     _In_opt_ PULONG BusNumber,
//     _In_opt_ PCONFIGURATION_TYPE ControllerType,
//     _In_opt_ PULONG ControllerNumber,
//     _In_opt_ PCONFIGURATION_TYPE PeripheralType,
//     _In_opt_ PULONG PeripheralNumber,
//     _In_ PIO_QUERY_DEVICE_ROUTINE CalloutRoutine,
//     _In_opt_ PVOID Context)
// {
//     NTSTATUS Status;
//     ULONG BusLoopNumber = -1; /* Root Bus */
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING RootRegKey;
//     HANDLE RootRegHandle;
//     IO_QUERY Query;
//
//     IORSRCTRACE("\nIoQueryDeviceDescription()\n"
//                 "    BusType:          0x%p (%lu)\n"
//                 "    BusNumber:        0x%p (%lu)\n"
//                 "    ControllerType:   0x%p (%lu)\n"
//                 "    ControllerNumber: 0x%p (%lu)\n"
//                 "    PeripheralType:   0x%p (%lu)\n"
//                 "    PeripheralNumber: 0x%p (%lu)\n"
//                 "    CalloutRoutine:   0x%p\n"
//                 "    Context:          0x%p\n"
//                 "--> Query: 0x%p\n",
//                 BusType, BusType ? *BusType : -1,
//                 BusNumber, BusNumber ? *BusNumber : -1,
//                 ControllerType, ControllerType ? *ControllerType : -1,
//                 ControllerNumber, ControllerNumber ? *ControllerNumber : -1,
//                 PeripheralType, PeripheralType ? *PeripheralType : -1,
//                 PeripheralNumber, PeripheralNumber ? *PeripheralNumber : -1,
//                 CalloutRoutine, Context,
//                 &Query);
//
//     if (!BusType)
//         return STATUS_NOT_IMPLEMENTED;
//
//     /* Set up the string */
//     RootRegKey.Length = 0;
//     RootRegKey.MaximumLength = 2048;
//     RootRegKey.Buffer = ExAllocatePoolWithTag(PagedPool, RootRegKey.MaximumLength, TAG_IO_RESOURCE);
//     if (!RootRegKey.Buffer)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     RtlAppendUnicodeToString(&RootRegKey, L"\\REGISTRY\\MACHINE\\HARDWARE\\DESCRIPTION\\SYSTEM");
//
//     /* Open a handle to the Root Registry Key */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &RootRegKey,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     Status = ZwOpenKey(&RootRegHandle, KEY_READ, &ObjectAttributes);
//
//     if (NT_SUCCESS(Status))
//     {
//         /* Use a helper function to loop though this key and get the information */
//         Query.BusType = BusType;
//         Query.BusNumber = BusNumber;
//         Query.ControllerType = ControllerType;
//         Query.ControllerNumber = ControllerNumber;
//         Query.PeripheralType = PeripheralType;
//         Query.PeripheralNumber = PeripheralNumber;
//         Query.CalloutRoutine = CalloutRoutine;
//         Query.Context = Context;
//         Status = IopQueryBusDescription(&Query,
//                                         RootRegKey,
//                                         RootRegHandle,
//                                         &BusLoopNumber,
//                                         TRUE);
//
//         /* Close registry key */
//         ZwClose(RootRegHandle);
//     }
//
//     /* Cleanup */
//     ExFreePoolWithTag(RootRegKey.Buffer, TAG_IO_RESOURCE);
//
//     return Status;
// }
//
// /**
//  * @brief
//  * Reports hardware resources of the HAL in the
//  * \Registry\Machine\Hardware\ResourceMap tree.
//  *
//  * @param[in]   HalName
//  * Descriptive name of the HAL.
//  *
//  * @param[in]   RawResourceList
//  * List of raw (bus specific) resources which should be claimed
//  * for the HAL.
//  *
//  * @param[in]   TranslatedResourceList
//  * List of translated (system wide) resources which should be claimed
//  * for the HAL.
//  *
//  * @param[in]   ResourceListSize
//  * Size in bytes of the raw and translated resource lists.
//  * Both lists have the same size.
//  *
//  * @return  A status code.
//  **/
// NTSTATUS
// NTAPI
// IoReportHalResourceUsage(
//     _In_ PUNICODE_STRING HalName,
//     _In_ PCM_RESOURCE_LIST RawResourceList,
//     _In_ PCM_RESOURCE_LIST TranslatedResourceList,
//     _In_ ULONG ResourceListSize)
// {
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING Name;
//     ULONG Disposition;
//     HANDLE ResourceMapKey;
//     HANDLE HalKey;
//     HANDLE DescriptionKey;
//
//     /* Open/Create 'RESOURCEMAP' key */
//     RtlInitUnicodeString(&Name, L"\\Registry\\Machine\\HARDWARE\\RESOURCEMAP");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &Name,
//                                OBJ_CASE_INSENSITIVE | OBJ_OPENIF,
//                                0,
//                                NULL);
//     Status = ZwCreateKey(&ResourceMapKey,
//                          KEY_ALL_ACCESS,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_VOLATILE,
//                          &Disposition);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Open/Create 'Hardware Abstraction Layer' key */
//     RtlInitUnicodeString(&Name, L"Hardware Abstraction Layer");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &Name,
//                                OBJ_CASE_INSENSITIVE | OBJ_OPENIF,
//                                ResourceMapKey,
//                                NULL);
//     Status = ZwCreateKey(&HalKey,
//                          KEY_ALL_ACCESS,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_VOLATILE,
//                          &Disposition);
//     ZwClose(ResourceMapKey);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Create 'HalName' key */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                HalName,
//                                OBJ_CASE_INSENSITIVE,
//                                HalKey,
//                                NULL);
//     Status = ZwCreateKey(&DescriptionKey,
//                          KEY_ALL_ACCESS,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_VOLATILE,
//                          &Disposition);
//     ZwClose(HalKey);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Add '.Raw' value */
//     RtlInitUnicodeString(&Name, L".Raw");
//     Status = ZwSetValueKey(DescriptionKey,
//                            &Name,
//                            0,
//                            REG_RESOURCE_LIST,
//                            RawResourceList,
//                            ResourceListSize);
//     if (!NT_SUCCESS(Status))
//     {
//         ZwClose(DescriptionKey);
//         return Status;
//     }
//
//     /* Add '.Translated' value */
//     RtlInitUnicodeString(&Name, L".Translated");
//     Status = ZwSetValueKey(DescriptionKey,
//                            &Name,
//                            0,
//                            REG_RESOURCE_LIST,
//                            TranslatedResourceList,
//                            ResourceListSize);
//     ZwClose(DescriptionKey);
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
