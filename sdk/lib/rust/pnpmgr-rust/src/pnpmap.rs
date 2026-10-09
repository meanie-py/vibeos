use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{


    CM_FULL_RESOURCE_DESCRIPTOR,
    ExAllocatePool,
    ExFreePool,
    IopOpenRegistryKeyEx,
    KEY_BASIC_INFORMATION,
    KEY_VALUE_PARTIAL_INFORMATION,
    _OBJECT_ATTRIBUTES,
    _UNICODE_STRING,
    PCM_FULL_RESOURCE_DESCRIPTOR,
    PCUNICODE_STRING,
    PKEY_BASIC_INFORMATION,
    PKEY_VALUE_PARTIAL_INFORMATION,
    PUNICODE_STRING,
    RtlCompareUnicodeString,
    RtlInitUnicodeString,
    ZwClose,
    ZwCreateKey,
    ZwDeleteKey,
    ZwEnumerateKey,
    ZwOpenKey,
    ZwQueryValueKey,
    ZwSetValueKey,
    SIZE_T,
};

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

const REG_OPTION_NON_VOLATILE: ULONG = 0x00000000;
const REG_OPTION_VOLATILE: ULONG = 0x00000001;
const REG_FULL_RESOURCE_DESCRIPTOR: ULONG = 9;
const REG_RESOURCE_LIST: ULONG = 8;

#[repr(C)]
struct PnpMapperDeviceId {
    TypeName: *const u16,
    PnPId: *mut u16,
}

#[repr(C)]
struct PnpDetectIdentifierMap {
    DetectId: *const u16,
    PnPId: *mut u16,
    PeripheralMap: *mut PnpMapperDeviceId,
    Counter: ULONG,
}

static mut IdentifierU: _UNICODE_STRING = _UNICODE_STRING {
    Length: 20,
    MaximumLength: 22,
    Buffer: w!("Identifier").as_ptr() as *mut u16,
};

static mut HardwareIDU: _UNICODE_STRING = _UNICODE_STRING {
    Length: 20,
    MaximumLength: 22,
    Buffer: w!("HardwareID").as_ptr() as *mut u16,
};

static mut ConfigurationDataU: _UNICODE_STRING = _UNICODE_STRING {
    Length: 34,
    MaximumLength: 36,
    Buffer: w!("Configuration Data").as_ptr() as *mut u16,
};

static mut BootConfigU: _UNICODE_STRING = _UNICODE_STRING {
    Length: 20,
    MaximumLength: 22,
    Buffer: w!("BootConfig").as_ptr() as *mut u16,
};

static mut LogConfU: _UNICODE_STRING = _UNICODE_STRING {
    Length: 14,
    MaximumLength: 16,
    Buffer: w!("LogConf").as_ptr() as *mut u16,
};

static mut KeyboardMap: [PnpMapperDeviceId; 16] = [
    PnpMapperDeviceId { TypeName: w!("XT_83KEY").as_ptr() as *const u16, PnPId: w!("*PNP0300").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("PCAT_86KEY").as_ptr() as *const u16, PnPId: w!("*PNP0301").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("PCXT_84KEY").as_ptr() as *const u16, PnPId: w!("*PNP0302").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("XT_84KEY").as_ptr() as *const u16, PnPId: w!("*PNP0302").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("101-KEY").as_ptr() as *const u16, PnPId: w!("*PNP0303").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("OLI_83KEY").as_ptr() as *const u16, PnPId: w!("*PNP0304").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("ATT_301").as_ptr() as *const u16, PnPId: w!("*PNP0304").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("OLI_102KEY").as_ptr() as *const u16, PnPId: w!("*PNP0305").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("OLI_86KEY").as_ptr() as *const u16, PnPId: w!("*PNP0306").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("OLI_A101_102KEY").as_ptr() as *const u16, PnPId: w!("*PNP0309").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("ATT_302").as_ptr() as *const u16, PnPId: w!("*PNP030a").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("PCAT_ENHANCED").as_ptr() as *const u16, PnPId: w!("*PNP030b").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("PC98_106KEY").as_ptr() as *const u16, PnPId: w!("*nEC1300").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("PC98_LaptopKEY").as_ptr() as *const u16, PnPId: w!("*nEC1300").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("PC98_N106KEY").as_ptr() as *const u16, PnPId: w!("*PNP0303").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: core::ptr::null(), PnPId: core::ptr::null_mut() },
];

static mut PointerMap: [PnpMapperDeviceId; 11] = [
    PnpMapperDeviceId { TypeName: w!("PS2 MOUSE").as_ptr() as *const u16, PnPId: w!("*PNP0F0E").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("SERIAL MOUSE").as_ptr() as *const u16, PnPId: w!("*PNP0F0C").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("MICROSOFT PS2 MOUSE").as_ptr() as *const u16, PnPId: w!("*PNP0F03").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("LOGITECH PS2 MOUSE").as_ptr() as *const u16, PnPId: w!("*PNP0F12").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("MICROSOFT INPORT MOUSE").as_ptr() as *const u16, PnPId: w!("*PNP0F02").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("MICROSOFT SERIAL MOUSE").as_ptr() as *const u16, PnPId: w!("*PNP0F01").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("MICROSOFT BALLPOINT SERIAL MOUSE").as_ptr() as *const u16, PnPId: w!("*PNP0F09").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("LOGITECH SERIAL MOUSE").as_ptr() as *const u16, PnPId: w!("*PNP0F08").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("MICROSOFT BUS MOUSE").as_ptr() as *const u16, PnPId: w!("*PNP0F00").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: w!("NEC PC-9800 BUS MOUSE").as_ptr() as *const u16, PnPId: w!("*nEC1F00").as_ptr() as *mut u16 },
    PnpMapperDeviceId { TypeName: core::ptr::null(), PnPId: core::ptr::null_mut() },
];

static mut PnPMap: [PnpDetectIdentifierMap; 8] = [
    PnpDetectIdentifierMap { DetectId: w!("SerialController").as_ptr() as *const u16, PnPId: w!("*PNP0501").as_ptr() as *mut u16, PeripheralMap: core::ptr::null_mut(), Counter: 0 },
    PnpDetectIdentifierMap { DetectId: w!("KeyboardPeripheral").as_ptr() as *const u16, PnPId: core::ptr::null_mut(), PeripheralMap: core::ptr::null_mut(), Counter: 0 },
    PnpDetectIdentifierMap { DetectId: w!("PointerPeripheral").as_ptr() as *const u16, PnPId: core::ptr::null_mut(), PeripheralMap: core::ptr::null_mut(), Counter: 0 },
    PnpDetectIdentifierMap { DetectId: w!("ParallelController").as_ptr() as *const u16, PnPId: w!("*PNP0400").as_ptr() as *mut u16, PeripheralMap: core::ptr::null_mut(), Counter: 0 },
    PnpDetectIdentifierMap { DetectId: w!("FloppyDiskPeripheral").as_ptr() as *const u16, PnPId: w!("*PNP0700").as_ptr() as *mut u16, PeripheralMap: core::ptr::null_mut(), Counter: 0 },
    PnpDetectIdentifierMap { DetectId: core::ptr::null(), PnPId: core::ptr::null_mut(), PeripheralMap: core::ptr::null_mut(), Counter: 0 },
    PnpDetectIdentifierMap { DetectId: core::ptr::null(), PnPId: core::ptr::null_mut(), PeripheralMap: core::ptr::null_mut(), Counter: 0 },
    PnpDetectIdentifierMap { DetectId: core::ptr::null(), PnPId: core::ptr::null_mut(), PeripheralMap: core::ptr::null_mut(), Counter: 0 },
];

unsafe fn IopMapPeripheralId(
    Value: PCUNICODE_STRING,
    DeviceList: *mut PnpMapperDeviceId,
) -> PWSTR {
    let mut i: ULONG = 0;
    let mut CmpId: _UNICODE_STRING = core::mem::zeroed();

    loop {
        if (*DeviceList.add(i as usize)).TypeName.is_null() {
            break;
        }

        RtlInitUnicodeString(&mut CmpId, (*DeviceList.add(i as usize)).TypeName as *mut u16);

        if RtlCompareUnicodeString(Value, &mut CmpId, 0u8) == 0 {
            break;
        }

        i += 1;
    }

    (*DeviceList.add(i as usize)).PnPId
}

unsafe fn IopMapDetectedDeviceId(
    DetectId: PUNICODE_STRING,
    Value: PUNICODE_STRING,
    DeviceIndex: *mut ULONG,
) -> PWSTR {
    let mut i: ULONG = 0;
    let mut CmpId: _UNICODE_STRING = core::mem::zeroed();

    if DetectId.is_null() {
        return core::ptr::null_mut();
    }

    loop {
        if PnPMap[i as usize].DetectId.is_null() {
            break;
        }

        RtlInitUnicodeString(&mut CmpId, PnPMap[i as usize].DetectId as *mut u16);

        if RtlCompareUnicodeString(DetectId, &mut CmpId, 0u8) == 0 {
            *DeviceIndex = PnPMap[i as usize].Counter;
            PnPMap[i as usize].Counter += 1;

            if !PnPMap[i as usize].PeripheralMap.is_null() {
                return IopMapPeripheralId(Value, PnPMap[i as usize].PeripheralMap);
            }
            break;
        }

        i += 1;
    }

    PnPMap[i as usize].PnPId
}

unsafe fn IopEnumerateDetectedDevices(
    hBaseKey: HANDLE,
    RelativePath: PUNICODE_STRING,
    hRootKey: HANDLE,
    EnumerateSubKeys: BOOLEAN,
    BootResources: PCM_FULL_RESOURCE_DESCRIPTOR,
    BootResourcesLength: ULONG,
    ParentBootResources: PCM_FULL_RESOURCE_DESCRIPTOR,
    ParentBootResourcesLength: ULONG,
) -> NTSTATUS {
    let mut hDevicesKey: HANDLE = core::ptr::null_mut();
    let mut KeyIndex: ULONG = 0;
    let mut pDeviceInformation: PKEY_BASIC_INFORMATION = core::ptr::null_mut();
    let mut DeviceInfoLength: ULONG = core::mem::size_of::<KEY_BASIC_INFORMATION>() as ULONG + 50 * core::mem::size_of::<u16>() as ULONG;
    let mut Status: NTSTATUS;

    if BootResources.is_null() && !RelativePath.is_null() {
        Status = IopOpenRegistryKeyEx(&mut hDevicesKey, hBaseKey, RelativePath, KEY_ENUMERATE_SUB_KEYS);

        if Status < 0 {
            return Status;
        }
    } else {
        hDevicesKey = hBaseKey;
    }

    pDeviceInformation = ExAllocatePool(PagedPool, DeviceInfoLength as SIZE_T) as PKEY_BASIC_INFORMATION;
    if pDeviceInformation.is_null() {
        Status = STATUS_NO_MEMORY;
        if !hDevicesKey.is_null() && hDevicesKey != hBaseKey {
            ZwClose(hDevicesKey);
        }
        return Status;
    }

    loop {
        let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
        let mut hDeviceKey: HANDLE = core::ptr::null_mut();
        let mut hLevel1Key: HANDLE = core::ptr::null_mut();
        let mut hLevel2Key: HANDLE = core::ptr::null_mut();
        let mut hLogConf: HANDLE = core::ptr::null_mut();
        let mut Level2NameU: _UNICODE_STRING = core::mem::zeroed();
        let mut Level2Name: [u16; 5] = [0; 5];
        let mut pValueInformation: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();
        let mut ValueInfoLength: ULONG = core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as ULONG + 50 * core::mem::size_of::<u16>() as ULONG;
        let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
        let mut RequiredSize: ULONG = 0;

        let mut pHardwareId: PWSTR;
        let mut DeviceIndex: ULONG = 0;
        let mut LocalBootResources: PCM_FULL_RESOURCE_DESCRIPTOR = BootResources;
        let mut LocalBootResourcesLength: ULONG = BootResourcesLength;

        Status = ZwEnumerateKey(
            hDevicesKey,
            KeyIndex,
            0,
            pDeviceInformation as PVOID,
            DeviceInfoLength,
            &mut RequiredSize,
        );

        if Status == STATUS_NO_MORE_ENTRIES {
            break;
        } else if Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL {
            ExFreePool(pDeviceInformation as PVOID);
            DeviceInfoLength = RequiredSize;
            pDeviceInformation = ExAllocatePool(PagedPool, DeviceInfoLength as SIZE_T) as PKEY_BASIC_INFORMATION;

            if pDeviceInformation.is_null() {
                Status = STATUS_NO_MEMORY;
                if !hDevicesKey.is_null() && hDevicesKey != hBaseKey {
                    ZwClose(hDevicesKey);
                }
                return Status;
            }

            Status = ZwEnumerateKey(
                hDevicesKey,
                KeyIndex,
                0,
                pDeviceInformation as PVOID,
                DeviceInfoLength,
                &mut RequiredSize,
            );
        }

        if Status < 0 {
            if !hDevicesKey.is_null() && hDevicesKey != hBaseKey {
                ZwClose(hDevicesKey);
            }
            ExFreePool(pDeviceInformation as PVOID);
            return Status;
        }
        KeyIndex += 1;

        DeviceName.Length = (*pDeviceInformation).NameLength as u16;
        DeviceName.MaximumLength = (*pDeviceInformation).NameLength as u16;
        DeviceName.Buffer = (*pDeviceInformation).Name.as_mut_ptr();

        if !BootResources.is_null() {
            Status = IopEnumerateDetectedDevices(
                hDevicesKey,
                &mut DeviceName,
                hRootKey,
                1u8,
                core::ptr::null_mut(),
                0,
                BootResources,
                BootResourcesLength,
            );

            if Status < 0 {
                if !hDevicesKey.is_null() && hDevicesKey != hBaseKey {
                    ZwClose(hDevicesKey);
                }
                ExFreePool(pDeviceInformation as PVOID);
                return Status;
            }

            continue;
        }

        pValueInformation = ExAllocatePool(PagedPool, ValueInfoLength as SIZE_T) as PKEY_VALUE_PARTIAL_INFORMATION;
        if pValueInformation.is_null() {
            Status = STATUS_NO_MEMORY;
            if !hDevicesKey.is_null() && hDevicesKey != hBaseKey {
                ZwClose(hDevicesKey);
            }
            ExFreePool(pDeviceInformation as PVOID);
            return Status;
        }

        Status = IopOpenRegistryKeyEx(
            &mut hDeviceKey,
            hDevicesKey,
            &mut DeviceName,
            KEY_QUERY_VALUE | if EnumerateSubKeys != 0 { KEY_ENUMERATE_SUB_KEYS } else { 0 },
        );

        if Status < 0 {
            ExFreePool(pValueInformation as PVOID);
            continue;
        }

        Status = ZwQueryValueKey(
            hDeviceKey,
            &raw mut ConfigurationDataU,
            ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as _,
            pValueInformation as PVOID,
            ValueInfoLength,
            &mut RequiredSize,
        );

        if Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL {
            ExFreePool(pValueInformation as PVOID);
            ValueInfoLength = RequiredSize;
            pValueInformation = ExAllocatePool(PagedPool, ValueInfoLength as SIZE_T) as PKEY_VALUE_PARTIAL_INFORMATION;

            if pValueInformation.is_null() {
                ZwDeleteKey(hLevel2Key);
                Status = STATUS_NO_MEMORY;
                ZwClose(hDeviceKey);
                continue;
            }

            Status = ZwQueryValueKey(
                hDeviceKey,
                &raw mut ConfigurationDataU,
                ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as _,
                pValueInformation as PVOID,
                ValueInfoLength,
                &mut RequiredSize,
            );
        }

        if Status == STATUS_OBJECT_NAME_NOT_FOUND {
            LocalBootResources = ParentBootResources;
            LocalBootResourcesLength = ParentBootResourcesLength;
        } else if Status < 0 {
            ExFreePool(pValueInformation as PVOID);
            ZwClose(hDeviceKey);
            continue;
        } else if (*pValueInformation).Type != REG_FULL_RESOURCE_DESCRIPTOR {
            ExFreePool(pValueInformation as PVOID);
            ZwClose(hDeviceKey);
            continue;
        } else {
            const HEADER: usize = core::mem::offset_of!(CM_FULL_RESOURCE_DESCRIPTOR, PartialResourceList);

            if ParentBootResourcesLength == 0 {
                LocalBootResourcesLength = (*pValueInformation).DataLength;
            } else {
                LocalBootResourcesLength = ParentBootResourcesLength + (*pValueInformation).DataLength - HEADER as ULONG;
            }

            LocalBootResources = ExAllocatePool(PagedPool, LocalBootResourcesLength as SIZE_T) as PCM_FULL_RESOURCE_DESCRIPTOR;
            if LocalBootResources.is_null() {
                ExFreePool(pValueInformation as PVOID);
                ZwClose(hDeviceKey);
                continue;
            }

            if (ParentBootResourcesLength as usize) < core::mem::size_of::<CM_FULL_RESOURCE_DESCRIPTOR>() {
                core::ptr::copy_nonoverlapping(
                    (*pValueInformation).Data.as_ptr(),
                    LocalBootResources as *mut u8,
                    (*pValueInformation).DataLength as usize,
                );
                // (Next arm.) PartialDescriptors is a C flexible array (`PartialDescriptors[1]`,
                // xdk/cmtypes.h:350) of Count descriptors. Indexing bindgen's [_; 1] at
                // Count - 1 panicked -- in the kernel, during device enumeration -- for any
                // parent whose boot resources hold two descriptors or more.
            } else if (*(&raw const (*ParentBootResources).PartialResourceList.PartialDescriptors[0])
                .add((*ParentBootResources).PartialResourceList.Count as usize - 1))
            .Type
                == 4
            {
                core::ptr::copy_nonoverlapping(
                    (*pValueInformation).Data.as_ptr(),
                    LocalBootResources as *mut u8,
                    (*pValueInformation).DataLength as usize,
                );
                core::ptr::copy_nonoverlapping(
                    (ParentBootResources as *const u8).add(HEADER), // nolint: magic-offset (offset_of!-derived const)
                    (LocalBootResources as *mut u8).add((*pValueInformation).DataLength as usize),
                    ParentBootResourcesLength as usize - HEADER,
                );
                (*LocalBootResources).PartialResourceList.Count += (*ParentBootResources).PartialResourceList.Count;
            } else {
                core::ptr::copy_nonoverlapping(
                    (*pValueInformation).Data.as_ptr(),
                    LocalBootResources as *mut u8,
                    HEADER,
                );
                core::ptr::copy_nonoverlapping(
                    (ParentBootResources as *const u8).add(HEADER), // nolint: magic-offset (offset_of!-derived const)
                    (LocalBootResources as *mut u8).add(HEADER), // nolint: magic-offset (offset_of!-derived const)
                    ParentBootResourcesLength as usize - HEADER,
                );
                core::ptr::copy_nonoverlapping(
                    (*pValueInformation).Data.as_ptr().add(HEADER), // nolint: magic-offset (offset_of!-derived const)
                    (LocalBootResources as *mut u8).add(ParentBootResourcesLength as usize),
                    (*pValueInformation).DataLength as usize - HEADER,
                );
                (*LocalBootResources).PartialResourceList.Count += (*ParentBootResources).PartialResourceList.Count;
            }
        }

        if EnumerateSubKeys != 0 {
            Status = IopEnumerateDetectedDevices(
                hDeviceKey,
                RelativePath,
                hRootKey,
                1u8,
                LocalBootResources,
                LocalBootResourcesLength,
                ParentBootResources,
                ParentBootResourcesLength,
            );

            if Status < 0 {
                if !LocalBootResources.is_null() && LocalBootResources != ParentBootResources {
                    ExFreePool(LocalBootResources as PVOID);
                }
                ExFreePool(pValueInformation as PVOID);
                ZwClose(hDeviceKey);
                if !hDevicesKey.is_null() && hDevicesKey != hBaseKey {
                    ZwClose(hDevicesKey);
                }
                ExFreePool(pDeviceInformation as PVOID);
                return Status;
            }
        }

        Status = ZwQueryValueKey(
            hDeviceKey,
            &raw mut IdentifierU,
            ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as _,
            pValueInformation as PVOID,
            ValueInfoLength,
            &mut RequiredSize,
        );

        if Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL {
            ExFreePool(pValueInformation as PVOID);
            ValueInfoLength = RequiredSize;
            pValueInformation = ExAllocatePool(PagedPool, ValueInfoLength as SIZE_T) as PKEY_VALUE_PARTIAL_INFORMATION;

            if pValueInformation.is_null() {
                Status = STATUS_NO_MEMORY;
                if !LocalBootResources.is_null() && LocalBootResources != ParentBootResources {
                    ExFreePool(LocalBootResources as PVOID);
                }
                ZwClose(hDeviceKey);
                if !hDevicesKey.is_null() && hDevicesKey != hBaseKey {
                    ZwClose(hDevicesKey);
                }
                ExFreePool(pDeviceInformation as PVOID);
                return Status;
            }

            Status = ZwQueryValueKey(
                hDeviceKey,
                &raw mut IdentifierU,
                ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as _,
                pValueInformation as PVOID,
                ValueInfoLength,
                &mut RequiredSize,
            );
        }

        let mut ValueName: _UNICODE_STRING = core::mem::zeroed();

        if Status < 0 {
            if Status != STATUS_OBJECT_NAME_NOT_FOUND {
                if !LocalBootResources.is_null() && LocalBootResources != ParentBootResources {
                    ExFreePool(LocalBootResources as PVOID);
                }
                ExFreePool(pValueInformation as PVOID);
                ZwClose(hDeviceKey);
                continue;
            }
            ValueName.Length = 0;
            ValueName.MaximumLength = 0;
        } else if (*pValueInformation).Type != REG_SZ {
            if !LocalBootResources.is_null() && LocalBootResources != ParentBootResources {
                ExFreePool(LocalBootResources as PVOID);
            }
            ExFreePool(pValueInformation as PVOID);
            ZwClose(hDeviceKey);
            continue;
        } else {
            ValueName.Length = (*pValueInformation).DataLength as u16;
            ValueName.MaximumLength = (*pValueInformation).DataLength as u16;
            ValueName.Buffer = (*pValueInformation).Data.as_mut_ptr() as *mut u16;
            if ValueName.Length >= core::mem::size_of::<u16>() as u16 && *ValueName.Buffer.add((ValueName.Length / core::mem::size_of::<u16>() as u16 - 1) as usize) == 0 {
                ValueName.Length -= core::mem::size_of::<u16>() as u16;
            }
        }

        pHardwareId = IopMapDetectedDeviceId(RelativePath, &mut ValueName, &mut DeviceIndex);
        if pHardwareId.is_null() {
            if !LocalBootResources.is_null() && LocalBootResources != ParentBootResources {
                ExFreePool(LocalBootResources as PVOID);
            }
            ExFreePool(pValueInformation as PVOID);
            ZwClose(hDeviceKey);
            continue;
        }

        let mut HardwareIdKey: _UNICODE_STRING = core::mem::zeroed();
        HardwareIdKey.Length = {
            let mut len = 0;
            while *pHardwareId.add(len) != 0 {
                len += 1;
            }
            (len * core::mem::size_of::<u16>()) as u16
        };
        HardwareIdKey.MaximumLength = HardwareIdKey.Length + (core::mem::size_of::<u16>() * 2) as u16;
        HardwareIdKey.Buffer = pHardwareId;

        InitializeObjectAttributes(&mut ObjectAttributes, &mut HardwareIdKey, OBJ_KERNEL_HANDLE, hRootKey, core::ptr::null_mut());

        Status = ZwCreateKey(
            &mut hLevel1Key,
            KEY_CREATE_SUB_KEY,
            &mut ObjectAttributes,
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            core::ptr::null_mut(),
        );

        if Status < 0 {
            if !LocalBootResources.is_null() && LocalBootResources != ParentBootResources {
                ExFreePool(LocalBootResources as PVOID);
            }
            ExFreePool(pValueInformation as PVOID);
            ZwClose(hDeviceKey);
            continue;
        }

        let wide_digits = [
            b'0' as u16, b'1' as u16, b'2' as u16, b'3' as u16, b'4' as u16,
            b'5' as u16, b'6' as u16, b'7' as u16, b'8' as u16, b'9' as u16,
        ];
        Level2Name[0] = wide_digits[((DeviceIndex / 1000) % 10) as usize];
        Level2Name[1] = wide_digits[((DeviceIndex / 100) % 10) as usize];
        Level2Name[2] = wide_digits[((DeviceIndex / 10) % 10) as usize];
        Level2Name[3] = wide_digits[(DeviceIndex % 10) as usize];
        Level2Name[4] = 0;

        RtlInitUnicodeString(&mut Level2NameU, Level2Name.as_mut_ptr());
        InitializeObjectAttributes(&mut ObjectAttributes, &mut Level2NameU, OBJ_KERNEL_HANDLE, hLevel1Key, core::ptr::null_mut());

        Status = ZwCreateKey(
            &mut hLevel2Key,
            KEY_SET_VALUE | KEY_CREATE_SUB_KEY,
            &mut ObjectAttributes,
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            core::ptr::null_mut(),
        );

        ZwClose(hLevel1Key);
        if Status < 0 {
            if !LocalBootResources.is_null() && LocalBootResources != ParentBootResources {
                ExFreePool(LocalBootResources as PVOID);
            }
            ExFreePool(pValueInformation as PVOID);
            ZwClose(hDeviceKey);
            continue;
        }

        Status = ZwSetValueKey(hLevel2Key, &raw mut HardwareIDU, 0, REG_MULTI_SZ, HardwareIdKey.Buffer as PVOID, HardwareIdKey.MaximumLength as ULONG);

        if Status < 0 {
            ZwDeleteKey(hLevel2Key);
            if !LocalBootResources.is_null() && LocalBootResources != ParentBootResources {
                ExFreePool(LocalBootResources as PVOID);
            }
            ZwClose(hLevel2Key);
            ExFreePool(pValueInformation as PVOID);
            ZwClose(hDeviceKey);
            continue;
        }

        InitializeObjectAttributes(&mut ObjectAttributes, &raw mut LogConfU, OBJ_KERNEL_HANDLE, hLevel2Key, core::ptr::null_mut());

        Status = ZwCreateKey(
            &mut hLogConf,
            KEY_SET_VALUE,
            &mut ObjectAttributes,
            0,
            core::ptr::null_mut(),
            REG_OPTION_VOLATILE,
            core::ptr::null_mut(),
        );

        if Status < 0 {
            ZwDeleteKey(hLevel2Key);
            if !LocalBootResources.is_null() && LocalBootResources != ParentBootResources {
                ExFreePool(LocalBootResources as PVOID);
            }
            ZwClose(hLevel2Key);
            ExFreePool(pValueInformation as PVOID);
            ZwClose(hDeviceKey);
            continue;
        }

        if LocalBootResourcesLength as usize >= core::mem::size_of::<CM_FULL_RESOURCE_DESCRIPTOR>() {
            let mut CmResourceList: *mut u8 = core::ptr::null_mut();
            let ListCount: ULONG = 1;

            CmResourceList = ExAllocatePool(PagedPool, (LocalBootResourcesLength + core::mem::size_of::<ULONG>() as ULONG) as SIZE_T) as *mut u8;
            if CmResourceList.is_null() {
                ZwClose(hLogConf);
                ZwDeleteKey(hLevel2Key);
                if !LocalBootResources.is_null() && LocalBootResources != ParentBootResources {
                    ExFreePool(LocalBootResources as PVOID);
                }
                ZwClose(hLevel2Key);
                ExFreePool(pValueInformation as PVOID);
                ZwClose(hDeviceKey);
                continue;
            }

            core::ptr::copy_nonoverlapping(
                &ListCount as *const ULONG as *const u8,
                CmResourceList,
                core::mem::size_of::<ULONG>(),
            );

            core::ptr::copy_nonoverlapping(
                LocalBootResources as *const u8,
                CmResourceList.add(core::mem::size_of::<ULONG>()),
                LocalBootResourcesLength as usize,
            );

            Status = ZwSetValueKey(
                hLogConf,
                &raw mut BootConfigU,
                0,
                REG_RESOURCE_LIST,
                CmResourceList as PVOID,
                LocalBootResourcesLength + core::mem::size_of::<ULONG>() as ULONG,
            );

            ExFreePool(CmResourceList as PVOID);

            if Status < 0 {
                ZwClose(hLogConf);
                ZwDeleteKey(hLevel2Key);
                if !LocalBootResources.is_null() && LocalBootResources != ParentBootResources {
                    ExFreePool(LocalBootResources as PVOID);
                }
                ZwClose(hLevel2Key);
                ExFreePool(pValueInformation as PVOID);
                ZwClose(hDeviceKey);
                continue;
            }
        }
        ZwClose(hLogConf);

        if !LocalBootResources.is_null() && LocalBootResources != ParentBootResources {
            ExFreePool(LocalBootResources as PVOID);
        }
        if !hLevel2Key.is_null() {
            ZwClose(hLevel2Key);
        }
        if !hDeviceKey.is_null() {
            ZwClose(hDeviceKey);
        }
        ExFreePool(pValueInformation as PVOID);
    }

    if !hDevicesKey.is_null() && hDevicesKey != hBaseKey {
        ZwClose(hDevicesKey);
    }
    ExFreePool(pDeviceInformation as PVOID);

    STATUS_SUCCESS
}

unsafe fn IopIsFirmwareMapperDisabled() -> BOOLEAN {
    let mut KeyPathU: _UNICODE_STRING = _UNICODE_STRING {
        Length: 96,
        MaximumLength: 98,
        Buffer: w!("\\Registry\\Machine\\SYSTEM\\CURRENTCONTROLSET\\Control\\Pnp").as_ptr() as *mut u16,
    };
    let mut KeyNameU: _UNICODE_STRING = _UNICODE_STRING {
        Length: 42,
        MaximumLength: 44,
        Buffer: w!("DisableFirmwareMapper").as_ptr() as *mut u16,
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut hPnpKey: HANDLE = core::ptr::null_mut();
    let mut KeyInformation: PKEY_VALUE_PARTIAL_INFORMATION;
    let mut DesiredLength: ULONG = 0;
    let mut Length: ULONG;
    let mut KeyValue: ULONG = 0;
    let mut Status: NTSTATUS;

    InitializeObjectAttributes(&mut ObjectAttributes, &mut KeyPathU, OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE, core::ptr::null_mut(), core::ptr::null_mut());
    Status = ZwOpenKey(&mut hPnpKey, KEY_QUERY_VALUE, &mut ObjectAttributes);
    if Status >= 0 {
        Status = ZwQueryValueKey(
            hPnpKey,
            &mut KeyNameU,
            ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as _,
            core::ptr::null_mut(),
            0,
            &mut DesiredLength,
        );
        if Status == STATUS_BUFFER_TOO_SMALL || Status == STATUS_BUFFER_OVERFLOW {
            Length = DesiredLength;
            KeyInformation = ExAllocatePool(PagedPool, Length as SIZE_T) as PKEY_VALUE_PARTIAL_INFORMATION;
            if !KeyInformation.is_null() {
                Status = ZwQueryValueKey(
                    hPnpKey,
                    &mut KeyNameU,
                    ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as _,
                    KeyInformation as PVOID,
                    Length,
                    &mut DesiredLength,
                );
                if Status >= 0 && (*KeyInformation).DataLength == core::mem::size_of::<ULONG>() as ULONG {
                    KeyValue = *((*KeyInformation).Data.as_ptr() as *const ULONG);
                }

                ExFreePool(KeyInformation as PVOID);
            }
        }

        ZwClose(hPnpKey);
    }

    if KeyValue != 0 { 1u8 } else { 0u8 }
}

#[no_mangle]
pub unsafe extern "system" fn IopUpdateRootKey() -> NTSTATUS {
    let mut EnumU: _UNICODE_STRING = _UNICODE_STRING {
        Length: 88,
        MaximumLength: 90,
        Buffer: w!("\\Registry\\Machine\\SYSTEM\\CurrentControlSet\\Enum").as_ptr() as *mut u16,
    };
    let mut RootPathU: _UNICODE_STRING = _UNICODE_STRING {
        Length: 8,
        MaximumLength: 10,
        Buffer: w!("Root").as_ptr() as *mut u16,
    };
    let mut MultiKeyPathU: _UNICODE_STRING = _UNICODE_STRING {
        Length: 130,
        MaximumLength: 132,
        Buffer: w!("\\Registry\\Machine\\HARDWARE\\DESCRIPTION\\System\\MultifunctionAdapter").as_ptr() as *mut u16,
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut hEnum: HANDLE = core::ptr::null_mut();
    let mut hRoot: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    InitializeObjectAttributes(&mut ObjectAttributes, &mut EnumU, OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE, core::ptr::null_mut(), core::ptr::null_mut());
    Status = ZwCreateKey(&mut hEnum, KEY_CREATE_SUB_KEY, &mut ObjectAttributes, 0, core::ptr::null_mut(), REG_OPTION_NON_VOLATILE, core::ptr::null_mut());
    if Status < 0 {
        return Status;
    }

    InitializeObjectAttributes(&mut ObjectAttributes, &mut RootPathU, OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE, hEnum, core::ptr::null_mut());
    Status = ZwCreateKey(&mut hRoot, KEY_CREATE_SUB_KEY, &mut ObjectAttributes, 0, core::ptr::null_mut(), REG_OPTION_NON_VOLATILE, core::ptr::null_mut());
    ZwClose(hEnum);
    if Status < 0 {
        return Status;
    }

    if IopIsFirmwareMapperDisabled() == 0 {
        Status = IopOpenRegistryKeyEx(&mut hEnum, core::ptr::null_mut(), &mut MultiKeyPathU, KEY_ENUMERATE_SUB_KEYS);
        if Status < 0 {
            ZwClose(hRoot);
            return STATUS_SUCCESS;
        }
        Status = IopEnumerateDetectedDevices(
            hEnum,
            core::ptr::null_mut(),
            hRoot,
            1u8,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
        );
        ZwClose(hEnum);
    } else {
        Status = STATUS_SUCCESS;
    }

    ZwClose(hRoot);

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     PnP manager Firmware Mapper functions
//  * COPYRIGHT:   Copyright 2006-2007 Hervé Poussineau <hpoussin@reactos.org>
//  *              Copyright 2008-2011 Cameron Gutman <cameron.gutman@reactos.org>
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* TYPES *********************************************************************/
//
// typedef struct _PNP_MAPPER_DEVICE_ID
// {
//     PCWSTR TypeName;
//     PWSTR PnPId;
// } PNP_MAPPER_DEVICE_ID, *PPNP_MAPPER_DEVICE_ID;
//
// typedef struct _PNP_DETECT_IDENTIFIER_MAP
// {
//     PCWSTR DetectId;
//     PWSTR PnPId;
//     PPNP_MAPPER_DEVICE_ID PeripheralMap;
//     ULONG Counter;
// } PNP_DETECT_IDENTIFIER_MAP;
//
// /* DATA **********************************************************************/
//
// static UNICODE_STRING IdentifierU = RTL_CONSTANT_STRING(L"Identifier");
// static UNICODE_STRING HardwareIDU = RTL_CONSTANT_STRING(L"HardwareID");
// static UNICODE_STRING ConfigurationDataU = RTL_CONSTANT_STRING(L"Configuration Data");
// static UNICODE_STRING BootConfigU = RTL_CONSTANT_STRING(L"BootConfig");
// static UNICODE_STRING LogConfU = RTL_CONSTANT_STRING(L"LogConf");
//
// /* FIXME: Trailing \0 in structures below are hacks, should be removed.
//  * Hardware identifiers also can be mapped using "LegacyXlate" sections
//  * of driver INF files. */
//
// DATA_SEG("INITDATA")
// static
// PNP_MAPPER_DEVICE_ID KeyboardMap[] =
// {
//     { L"XT_83KEY", L"*PNP0300\0" },
//     { L"PCAT_86KEY", L"*PNP0301\0" },
//     { L"PCXT_84KEY", L"*PNP0302\0" },
//     { L"XT_84KEY", L"*PNP0302\0" },
//     { L"101-KEY", L"*PNP0303\0" },
//     { L"OLI_83KEY", L"*PNP0304\0" },
//     { L"ATT_301", L"*PNP0304\0" },
//     { L"OLI_102KEY", L"*PNP0305\0" },
//     { L"OLI_86KEY", L"*PNP0306\0" },
//     { L"OLI_A101_102KEY", L"*PNP0309\0" },
//     { L"ATT_302", L"*PNP030a\0" },
//     { L"PCAT_ENHANCED", L"*PNP030b\0" },
//     { L"PC98_106KEY", L"*nEC1300\0" },
//     { L"PC98_LaptopKEY", L"*nEC1300\0" },
//     { L"PC98_N106KEY", L"*PNP0303\0" },
//     { NULL, NULL }
// };
//
// DATA_SEG("INITDATA")
// static
// PNP_MAPPER_DEVICE_ID PointerMap[] =
// {
//     { L"PS2 MOUSE", L"*PNP0F0E\0" },
//     { L"SERIAL MOUSE", L"*PNP0F0C\0" },
//     { L"MICROSOFT PS2 MOUSE", L"*PNP0F03\0" },
//     { L"LOGITECH PS2 MOUSE", L"*PNP0F12\0" },
//     { L"MICROSOFT INPORT MOUSE", L"*PNP0F02\0" },
//     { L"MICROSOFT SERIAL MOUSE", L"*PNP0F01\0" },
//     { L"MICROSOFT BALLPOINT SERIAL MOUSE", L"*PNP0F09\0" },
//     { L"LOGITECH SERIAL MOUSE", L"*PNP0F08\0" },
//     { L"MICROSOFT BUS MOUSE", L"*PNP0F00\0" },
//     { L"NEC PC-9800 BUS MOUSE", L"*nEC1F00\0" },
//     { NULL, NULL }
// };
//
// DATA_SEG("INITDATA")
// static
// PNP_DETECT_IDENTIFIER_MAP PnPMap[] =
// {
//     { L"SerialController", L"*PNP0501\0", NULL, 0 },
//     //{ L"KeyboardController", L"*PNP0303\0", NULL, 0 },
//     //{ L"PointerController", L"*PNP0F13\0", NULL, 0 },
//     { L"KeyboardPeripheral", NULL, KeyboardMap, 0 },
//     { L"PointerPeripheral", NULL, PointerMap, 0 },
//     { L"ParallelController", L"*PNP0400\0", NULL, 0 },
//     { L"FloppyDiskPeripheral", L"*PNP0700\0", NULL, 0 },
//     { NULL, NULL, NULL, 0 }
// };
//
// /* FUNCTIONS *****************************************************************/
//
// static
// CODE_SEG("INIT")
// PWSTR
// IopMapPeripheralId(
//     _In_ PCUNICODE_STRING Value,
//     _In_ PPNP_MAPPER_DEVICE_ID DeviceList)
// {
//     ULONG i;
//     UNICODE_STRING CmpId;
//
//     for (i = 0; DeviceList[i].TypeName; i++)
//     {
//         RtlInitUnicodeString(&CmpId, DeviceList[i].TypeName);
//
//         if (RtlCompareUnicodeString(Value, &CmpId, FALSE) == 0)
//             break;
//     }
//
//     return DeviceList[i].PnPId;
// }
//
// static
// CODE_SEG("INIT")
// PWSTR
// IopMapDetectedDeviceId(
//     _In_ PUNICODE_STRING DetectId,
//     _In_ PUNICODE_STRING Value,
//     _Out_ PULONG DeviceIndex)
// {
//     ULONG i;
//     UNICODE_STRING CmpId;
//
//     if (!DetectId)
//         return NULL;
//
//     for (i = 0; PnPMap[i].DetectId; i++)
//     {
//         RtlInitUnicodeString(&CmpId, PnPMap[i].DetectId);
//
//         if (RtlCompareUnicodeString(DetectId, &CmpId, FALSE) == 0)
//         {
//             *DeviceIndex = PnPMap[i].Counter++;
//
//             if (PnPMap[i].PeripheralMap)
//                 return IopMapPeripheralId(Value, PnPMap[i].PeripheralMap);
//             break;
//         }
//     }
//
//     return PnPMap[i].PnPId;
// }
//
// static
// CODE_SEG("INIT")
// NTSTATUS
// IopEnumerateDetectedDevices(
//     _In_ HANDLE hBaseKey,
//     _In_opt_ PUNICODE_STRING RelativePath,
//     _In_ HANDLE hRootKey,
//     _In_ BOOLEAN EnumerateSubKeys,
//     _In_opt_ PCM_FULL_RESOURCE_DESCRIPTOR BootResources,
//     _In_opt_ ULONG BootResourcesLength,
//     _In_ PCM_FULL_RESOURCE_DESCRIPTOR ParentBootResources,
//     _In_ ULONG ParentBootResourcesLength)
// {
//     HANDLE hDevicesKey = NULL;
//     ULONG KeyIndex = 0;
//     PKEY_BASIC_INFORMATION pDeviceInformation = NULL;
//     ULONG DeviceInfoLength = sizeof(KEY_BASIC_INFORMATION) + 50 * sizeof(WCHAR);
//     NTSTATUS Status;
//
//     if (!BootResources && RelativePath)
//     {
//         Status = IopOpenRegistryKeyEx(&hDevicesKey, hBaseKey, RelativePath, KEY_ENUMERATE_SUB_KEYS);
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT("ZwOpenKey() failed with status 0x%08lx\n", Status);
//             goto cleanup;
//         }
//     }
//     else
//         hDevicesKey = hBaseKey;
//
//     pDeviceInformation = ExAllocatePool(PagedPool, DeviceInfoLength as SIZE_T);
//     if (!pDeviceInformation)
//     {
//         DPRINT("ExAllocatePool() failed\n");
//         Status = STATUS_NO_MEMORY;
//         goto cleanup;
//     }
//
//     while (TRUE)
//     {
//         OBJECT_ATTRIBUTES ObjectAttributes;
//         HANDLE hDeviceKey = NULL;
//         HANDLE hLevel1Key, hLevel2Key = NULL, hLogConf;
//         UNICODE_STRING Level2NameU;
//         WCHAR Level2Name[5];
//         PKEY_VALUE_PARTIAL_INFORMATION pValueInformation = NULL;
//         ULONG ValueInfoLength = sizeof(KEY_VALUE_PARTIAL_INFORMATION) + 50 * sizeof(WCHAR);
//         UNICODE_STRING DeviceName, ValueName;
//         ULONG RequiredSize;
//
//         UNICODE_STRING HardwareIdKey;
//         PWSTR pHardwareId;
//         ULONG DeviceIndex = 0;
//
//         Status = ZwEnumerateKey(hDevicesKey,
//                                 KeyIndex,
//                                 KeyBasicInformation,
//                                 pDeviceInformation,
//                                 DeviceInfoLength,
//                                 &RequiredSize);
//
//         if (Status == STATUS_NO_MORE_ENTRIES)
//             break;
//         else if (Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL)
//         {
//             ExFreePool(pDeviceInformation);
//             DeviceInfoLength = RequiredSize;
//             pDeviceInformation = ExAllocatePool(PagedPool, DeviceInfoLength as SIZE_T);
//
//             if (!pDeviceInformation)
//             {
//                 DPRINT("ExAllocatePool() failed\n");
//                 Status = STATUS_NO_MEMORY;
//                 goto cleanup;
//             }
//
//             Status = ZwEnumerateKey(hDevicesKey,
//                                     KeyIndex,
//                                     KeyBasicInformation,
//                                     pDeviceInformation,
//                                     DeviceInfoLength,
//                                     &RequiredSize);
//         }
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT("ZwEnumerateKey() failed with status 0x%08lx\n", Status);
//             goto cleanup;
//         }
//         KeyIndex++;
//
//         /* Open device key */
//         DeviceName.Length = DeviceName.MaximumLength = (USHORT)pDeviceInformation->NameLength;
//         DeviceName.Buffer = pDeviceInformation->Name;
//
//         if (BootResources)
//         {
//             Status = IopEnumerateDetectedDevices(
//                 hDevicesKey,
//                 &DeviceName,
//                 hRootKey,
//                 TRUE,
//                 NULL,
//                 0,
//                 BootResources,
//                 BootResourcesLength);
//
//             if (!NT_SUCCESS(Status))
//                 goto cleanup;
//
//             continue;
//         }
//
//         pValueInformation = ExAllocatePool(PagedPool, ValueInfoLength as SIZE_T);
//         if (!pValueInformation)
//         {
//             DPRINT("ExAllocatePool() failed\n");
//             Status = STATUS_NO_MEMORY;
//             goto cleanup;
//         }
//
//         Status = IopOpenRegistryKeyEx(&hDeviceKey, hDevicesKey, &DeviceName,
//             KEY_QUERY_VALUE + (EnumerateSubKeys ? KEY_ENUMERATE_SUB_KEYS : 0));
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT("ZwOpenKey() failed with status 0x%08lx\n", Status);
//             goto cleanup;
//         }
//
//         /* Read boot resources, and add then to parent ones */
//         Status = ZwQueryValueKey(hDeviceKey,
//                                  &ConfigurationDataU,
//                                  KeyValuePartialInformation,
//                                  pValueInformation,
//                                  ValueInfoLength,
//                                  &RequiredSize);
//
//         if (Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL)
//         {
//             ExFreePool(pValueInformation);
//             ValueInfoLength = RequiredSize;
//             pValueInformation = ExAllocatePool(PagedPool, ValueInfoLength as SIZE_T);
//
//             if (!pValueInformation)
//             {
//                 DPRINT("ExAllocatePool() failed\n");
//                 ZwDeleteKey(hLevel2Key);
//                 Status = STATUS_NO_MEMORY;
//                 goto cleanup;
//             }
//
//             Status = ZwQueryValueKey(hDeviceKey,
//                                      &ConfigurationDataU,
//                                      KeyValuePartialInformation,
//                                      pValueInformation,
//                                      ValueInfoLength,
//                                      &RequiredSize);
//         }
//
//         if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//         {
//             BootResources = ParentBootResources;
//             BootResourcesLength = ParentBootResourcesLength;
//         }
//         else if (!NT_SUCCESS(Status))
//         {
//             DPRINT("ZwQueryValueKey() failed with status 0x%08lx\n", Status);
//             goto nextdevice;
//         }
//         else if (pValueInformation->Type != REG_FULL_RESOURCE_DESCRIPTOR)
//         {
//             DPRINT("Wrong registry type: got 0x%lx, expected 0x%lx\n", pValueInformation->Type, REG_FULL_RESOURCE_DESCRIPTOR);
//             goto nextdevice;
//         }
//         else
//         {
//             static const ULONG Header = FIELD_OFFSET(CM_FULL_RESOURCE_DESCRIPTOR, PartialResourceList.PartialDescriptors);
//
//             /* Concatenate current resources and parent ones */
//             if (ParentBootResourcesLength == 0)
//                 BootResourcesLength = pValueInformation->DataLength;
//             else
//                 BootResourcesLength = ParentBootResourcesLength
//                     + pValueInformation->DataLength
//                     - Header;
//
//             BootResources = ExAllocatePool(PagedPool, BootResourcesLength);
//             if (!BootResources)
//             {
//                 DPRINT("ExAllocatePool() failed\n");
//                 goto nextdevice;
//             }
//
//             if (ParentBootResourcesLength < sizeof(CM_FULL_RESOURCE_DESCRIPTOR))
//             {
//                 RtlCopyMemory(BootResources, pValueInformation->Data, pValueInformation->DataLength);
//             }
//             else if (ParentBootResources->PartialResourceList.PartialDescriptors[ParentBootResources->PartialResourceList.Count - 1].Type == CmResourceTypeDeviceSpecific)
//             {
//                 RtlCopyMemory(BootResources, pValueInformation->Data, pValueInformation->DataLength);
//                 RtlCopyMemory(
//                     (PVOID)((ULONG_PTR)BootResources + pValueInformation->DataLength),
//                     (PVOID)((ULONG_PTR)ParentBootResources + Header),
//                     ParentBootResourcesLength - Header);
//                 BootResources->PartialResourceList.Count += ParentBootResources->PartialResourceList.Count;
//             }
//             else
//             {
//                 RtlCopyMemory(BootResources, pValueInformation->Data, Header);
//                 RtlCopyMemory(
//                     (PVOID)((ULONG_PTR)BootResources + Header),
//                     (PVOID)((ULONG_PTR)ParentBootResources + Header),
//                     ParentBootResourcesLength - Header);
//                 RtlCopyMemory(
//                     (PVOID)((ULONG_PTR)BootResources + ParentBootResourcesLength),
//                     pValueInformation->Data + Header,
//                     pValueInformation->DataLength - Header);
//                 BootResources->PartialResourceList.Count += ParentBootResources->PartialResourceList.Count;
//             }
//         }
//
//         if (EnumerateSubKeys)
//         {
//             Status = IopEnumerateDetectedDevices(
//                 hDeviceKey,
//                 RelativePath,
//                 hRootKey,
//                 TRUE,
//                 BootResources,
//                 BootResourcesLength,
//                 ParentBootResources,
//                 ParentBootResourcesLength);
//
//             if (!NT_SUCCESS(Status))
//                 goto cleanup;
//         }
//
//         /* Read identifier */
//         Status = ZwQueryValueKey(hDeviceKey,
//                                  &IdentifierU,
//                                  KeyValuePartialInformation,
//                                  pValueInformation,
//                                  ValueInfoLength,
//                                  &RequiredSize);
//
//         if (Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL)
//         {
//             ExFreePool(pValueInformation);
//             ValueInfoLength = RequiredSize;
//             pValueInformation = ExAllocatePool(PagedPool, ValueInfoLength as SIZE_T);
//
//             if (!pValueInformation)
//             {
//                 DPRINT("ExAllocatePool() failed\n");
//                 Status = STATUS_NO_MEMORY;
//                 goto cleanup;
//             }
//
//             Status = ZwQueryValueKey(hDeviceKey,
//                                      &IdentifierU,
//                                      KeyValuePartialInformation,
//                                      pValueInformation,
//                                      ValueInfoLength,
//                                      &RequiredSize);
//         }
//
//         if (!NT_SUCCESS(Status))
//         {
//             if (Status != STATUS_OBJECT_NAME_NOT_FOUND)
//             {
//                 DPRINT("ZwQueryValueKey() failed with status 0x%08lx\n", Status);
//                 goto nextdevice;
//             }
//             ValueName.Length = ValueName.MaximumLength = 0;
//         }
//         else if (pValueInformation->Type != REG_SZ)
//         {
//             DPRINT("Wrong registry type: got 0x%lx, expected 0x%lx\n", pValueInformation->Type, REG_SZ);
//             goto nextdevice;
//         }
//         else
//         {
//             /* Assign hardware id to this device */
//             ValueName.Length = ValueName.MaximumLength = (USHORT)pValueInformation->DataLength;
//             ValueName.Buffer = (PWCHAR)pValueInformation->Data;
//             if (ValueName.Length >= sizeof(WCHAR) && ValueName.Buffer[ValueName.Length / sizeof(WCHAR) - 1] == UNICODE_NULL)
//                 ValueName.Length -= sizeof(WCHAR);
//         }
//
//         pHardwareId = IopMapDetectedDeviceId(RelativePath, &ValueName, &DeviceIndex);
//         if (!pHardwareId)
//         {
//             /* Unknown key path */
//             DPRINT("Unknown key path '%wZ' value '%wZ'\n", RelativePath, &ValueName);
//             goto nextdevice;
//         }
//
//         /* Prepare hardware id key (hardware id value without final \0) */
//         HardwareIdKey.Length = (USHORT)wcslen(pHardwareId) * sizeof(WCHAR);
//         HardwareIdKey.MaximumLength = HardwareIdKey.Length + sizeof(UNICODE_NULL) * 2;
//         HardwareIdKey.Buffer = pHardwareId;
//
//         /* Add the detected device to Root key */
//         InitializeObjectAttributes(&ObjectAttributes, &HardwareIdKey, OBJ_KERNEL_HANDLE, hRootKey, NULL);
//
//         Status = ZwCreateKey(
//             &hLevel1Key,
//             KEY_CREATE_SUB_KEY,
//             &ObjectAttributes,
//             0,
//             NULL,
//             REG_OPTION_NON_VOLATILE,
//             NULL);
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT("ZwCreateKey() failed with status 0x%08lx\n", Status);
//             goto nextdevice;
//         }
//
//         swprintf(Level2Name, L"%04lu", DeviceIndex);
//         RtlInitUnicodeString(&Level2NameU, Level2Name);
//         InitializeObjectAttributes(&ObjectAttributes, &Level2NameU, OBJ_KERNEL_HANDLE, hLevel1Key, NULL);
//
//         Status = ZwCreateKey(
//             &hLevel2Key,
//             KEY_SET_VALUE | KEY_CREATE_SUB_KEY,
//             &ObjectAttributes,
//             0,
//             NULL,
//             REG_OPTION_NON_VOLATILE,
//             NULL);
//
//         ZwClose(hLevel1Key);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT("ZwCreateKey() failed with status 0x%08lx\n", Status);
//             goto nextdevice;
//         }
//
//         DPRINT("Found %wZ #%lu (%wZ)\n", &ValueName, DeviceIndex, &HardwareIdKey);
//         Status = ZwSetValueKey(hLevel2Key, &HardwareIDU, 0, REG_MULTI_SZ, HardwareIdKey.Buffer, HardwareIdKey.MaximumLength);
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT("ZwSetValueKey() failed with status 0x%08lx\n", Status);
//             ZwDeleteKey(hLevel2Key);
//             goto nextdevice;
//         }
//
//         /* Create 'LogConf' subkey */
//         InitializeObjectAttributes(&ObjectAttributes, &LogConfU, OBJ_KERNEL_HANDLE, hLevel2Key, NULL);
//
//         Status = ZwCreateKey(
//             &hLogConf,
//             KEY_SET_VALUE,
//             &ObjectAttributes,
//             0,
//             NULL,
//             REG_OPTION_VOLATILE,
//             NULL);
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT("ZwCreateKey() failed with status 0x%08lx\n", Status);
//             ZwDeleteKey(hLevel2Key);
//             goto nextdevice;
//         }
//
//         if (BootResourcesLength >= sizeof(CM_FULL_RESOURCE_DESCRIPTOR))
//         {
//             PUCHAR CmResourceList;
//             ULONG ListCount;
//
//             CmResourceList = ExAllocatePool(PagedPool, BootResourcesLength + sizeof(ULONG));
//             if (!CmResourceList)
//             {
//                 ZwClose(hLogConf);
//                 ZwDeleteKey(hLevel2Key);
//                 goto nextdevice;
//             }
//
//             /* Add the list count (1st member of CM_RESOURCE_LIST) */
//             ListCount = 1;
//             RtlCopyMemory(CmResourceList,
//                           &ListCount,
//                           sizeof(ULONG));
//
//             /* Now add the actual list (2nd member of CM_RESOURCE_LIST) */
//             RtlCopyMemory(CmResourceList + sizeof(ULONG),
//                           BootResources,
//                           BootResourcesLength);
//
//             /* Save boot resources to 'LogConf\BootConfig' */
//             Status = ZwSetValueKey(hLogConf,
//                                    &BootConfigU,
//                                    0,
//                                    REG_RESOURCE_LIST,
//                                    CmResourceList,
//                                    BootResourcesLength + sizeof(ULONG));
//
//             ExFreePool(CmResourceList);
//
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT("ZwSetValueKey() failed with status 0x%08lx\n", Status);
//                 ZwClose(hLogConf);
//                 ZwDeleteKey(hLevel2Key);
//                 goto nextdevice;
//             }
//         }
//         ZwClose(hLogConf);
//
// nextdevice:
//         if (BootResources && BootResources != ParentBootResources)
//         {
//             ExFreePool(BootResources);
//             BootResources = NULL;
//         }
//         if (hLevel2Key)
//         {
//             ZwClose(hLevel2Key);
//             hLevel2Key = NULL;
//         }
//         if (hDeviceKey)
//         {
//             ZwClose(hDeviceKey);
//             hDeviceKey = NULL;
//         }
//         if (pValueInformation)
//             ExFreePool(pValueInformation);
//     }
//
//     Status = STATUS_SUCCESS;
//
// cleanup:
//     if (hDevicesKey && hDevicesKey != hBaseKey)
//         ZwClose(hDevicesKey);
//     if (pDeviceInformation)
//         ExFreePool(pDeviceInformation);
//
//     return Status;
// }
//
// static
// CODE_SEG("INIT")
// BOOLEAN
// IopIsFirmwareMapperDisabled(VOID)
// {
//     UNICODE_STRING KeyPathU = RTL_CONSTANT_STRING(L"\\Registry\\Machine\\SYSTEM\\CURRENTCONTROLSET\\Control\\Pnp");
//     UNICODE_STRING KeyNameU = RTL_CONSTANT_STRING(L"DisableFirmwareMapper");
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE hPnpKey;
//     PKEY_VALUE_PARTIAL_INFORMATION KeyInformation;
//     ULONG DesiredLength, Length;
//     ULONG KeyValue = 0;
//     NTSTATUS Status;
//
//     InitializeObjectAttributes(&ObjectAttributes, &KeyPathU, OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE, NULL, NULL);
//     Status = ZwOpenKey(&hPnpKey, KEY_QUERY_VALUE, &ObjectAttributes);
//     if (NT_SUCCESS(Status))
//     {
//         Status = ZwQueryValueKey(hPnpKey,
//                                  &KeyNameU,
//                                  KeyValuePartialInformation,
//                                  NULL,
//                                  0,
//                                  &DesiredLength);
//         if ((Status == STATUS_BUFFER_TOO_SMALL) ||
//             (Status == STATUS_BUFFER_OVERFLOW))
//         {
//             Length = DesiredLength;
//             KeyInformation = ExAllocatePool(PagedPool, Length);
//             if (KeyInformation)
//             {
//                 Status = ZwQueryValueKey(hPnpKey,
//                                          &KeyNameU,
//                                          KeyValuePartialInformation,
//                                          KeyInformation,
//                                          Length,
//                                          &DesiredLength);
//                 if (NT_SUCCESS(Status) && KeyInformation->DataLength == sizeof(ULONG))
//                 {
//                     KeyValue = (ULONG)(*KeyInformation->Data);
//                 }
//                 else
//                 {
//                     DPRINT1("ZwQueryValueKey(%wZ%wZ) failed\n", &KeyPathU, &KeyNameU);
//                 }
//
//                 ExFreePool(KeyInformation);
//             }
//             else
//             {
//                 DPRINT1("Failed to allocate memory for registry query\n");
//             }
//         }
//         else
//         {
//             DPRINT1("ZwQueryValueKey(%wZ%wZ) failed with status 0x%08lx\n", &KeyPathU, &KeyNameU, Status);
//         }
//
//         ZwClose(hPnpKey);
//     }
//     else
//     {
//         DPRINT1("ZwOpenKey(%wZ) failed with status 0x%08lx\n", &KeyPathU, Status);
//     }
//
//     DPRINT("Firmware mapper is %s\n", KeyValue != 0 ? "disabled" : "enabled");
//
//     return (KeyValue != 0) ? TRUE : FALSE;
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// IopUpdateRootKey(VOID)
// {
//     UNICODE_STRING EnumU = RTL_CONSTANT_STRING(L"\\Registry\\Machine\\SYSTEM\\CurrentControlSet\\Enum");
//     UNICODE_STRING RootPathU = RTL_CONSTANT_STRING(L"Root");
//     UNICODE_STRING MultiKeyPathU = RTL_CONSTANT_STRING(L"\\Registry\\Machine\\HARDWARE\\DESCRIPTION\\System\\MultifunctionAdapter");
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE hEnum, hRoot;
//     NTSTATUS Status;
//
//     InitializeObjectAttributes(&ObjectAttributes, &EnumU, OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE, NULL, NULL);
//     Status = ZwCreateKey(&hEnum, KEY_CREATE_SUB_KEY, &ObjectAttributes, 0, NULL, REG_OPTION_NON_VOLATILE, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwCreateKey() failed with status 0x%08lx\n", Status);
//         return Status;
//     }
//
//     InitializeObjectAttributes(&ObjectAttributes, &RootPathU, OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE, hEnum, NULL);
//     Status = ZwCreateKey(&hRoot, KEY_CREATE_SUB_KEY, &ObjectAttributes, 0, NULL, REG_OPTION_NON_VOLATILE, NULL);
//     ZwClose(hEnum);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwOpenKey() failed with status 0x%08lx\n", Status);
//         return Status;
//     }
//
//     if (!IopIsFirmwareMapperDisabled())
//     {
//         Status = IopOpenRegistryKeyEx(&hEnum, NULL, &MultiKeyPathU, KEY_ENUMERATE_SUB_KEYS);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Nothing to do, don't return with an error status */
//             DPRINT("ZwOpenKey() failed with status 0x%08lx\n", Status);
//             ZwClose(hRoot);
//             return STATUS_SUCCESS;
//         }
//         Status = IopEnumerateDetectedDevices(
//             hEnum,
//             NULL,
//             hRoot,
//             TRUE,
//             NULL,
//             0,
//             NULL,
//             0);
//         ZwClose(hEnum);
//     }
//     else
//     {
//         /* Enumeration is disabled */
//         Status = STATUS_SUCCESS;
//     }
//
//     ZwClose(hRoot);
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
