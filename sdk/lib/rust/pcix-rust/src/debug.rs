#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{PCI_SLOT_NUMBER, PCM_FULL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_RESOURCE_LIST, PDEVICE_CAPABILITIES, PIO_RESOURCE_DESCRIPTOR, PIO_RESOURCE_LIST, PIO_RESOURCE_REQUIREMENTS_LIST, PIO_STACK_LOCATION, PPCI_COMMON_HEADER};
use ntkrnl_sys::ddk::{
    _DEVICE_POWER_STATE_PowerDeviceMaximum as PowerDeviceMaximum,
    _SYSTEM_POWER_STATE_PowerSystemMaximum as PowerSystemMaximum,
    _SYSTEM_POWER_STATE_PowerSystemWorking as PowerSystemWorking,
};
// PCI_COMMON_HDR_LENGTH = FIELD_OFFSET(PCI_COMMON_CONFIG, DeviceSpecific) = 0x40
pub const PCI_COMMON_HDR_LENGTH: i32 = 0x40;
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{CM_FULL_RESOURCE_DESCRIPTOR, CM_PARTIAL_RESOURCE_DESCRIPTOR, PCSTR};

use crate::glue::{CmResourceTypeBusNumber, CmResourceTypeInterrupt, CmResourceTypePort, CmResourceTypeMemory};
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut PnpCodes: [PCHAR; 23] = [
    b"START_DEVICE\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"QUERY_REMOVE_DEVICE\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"REMOVE_DEVICE\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"CANCEL_REMOVE_DEVICE\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"STOP_DEVICE\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"QUERY_STOP_DEVICE\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"CANCEL_STOP_DEVICE\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"QUERY_DEVICE_RELATIONS\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"QUERY_INTERFACE\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"QUERY_CAPABILITIES\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"QUERY_RESOURCES\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"QUERY_RESOURCE_REQUIREMENTS\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"QUERY_DEVICE_TEXT\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"FILTER_RESOURCE_REQUIREMENTS\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"** UNKNOWN PNP IRP Minor Code **\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"READ_CONFIG\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"WRITE_CONFIG\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"EJECT\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"SET_LOCK\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"QUERY_ID\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"QUERY_PNP_DEVICE_STATE\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"QUERY_BUS_INFORMATION\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"DEVICE_USAGE_NOTIFICATION\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
];

pub static mut PoCodes: [PCHAR; 4] = [
    b"WAIT_WAKE\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"POWER_SEQUENCE\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"SET_POWER\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"QUERY_POWER\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
];

pub static mut SystemPowerStates: [PCHAR; 7] = [
    b"Unspecified\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"Working\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"Sleeping1\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"Sleeping2\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"Sleeping3\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"Hibernate\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"Shutdown\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
];

pub static mut DevicePowerStates: [PCHAR; 5] = [
    b"Unspecified\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"D0\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"D1\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"D2\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"D3\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
];

pub static mut PciBreakOnFdoPowerIrp: ULONG = 0;

pub static mut PciBreakOnPdoPowerIrp: ULONG = 0;

pub static mut PciBreakOnFdoPnpIrp: ULONG = 0;

pub static mut PciBreakOnPdoPnpIrp: ULONG = 0;
#[no_mangle]
pub unsafe extern "system" fn PciDebugPnpIrpTypeToText(MinorFunction: USHORT) -> PCHAR {
    if MinorFunction as i32 >= IRP_MN_SURPRISE_REMOVAL as i32 {
        b"** UNKNOWN PNP IRP Minor Code **\0".as_ptr() as PCHAR
    } else {
        PnpCodes[MinorFunction as usize]
    }
}

#[no_mangle]
pub unsafe extern "system" fn PciDebugPoIrpTypeToText(MinorFunction: USHORT) -> PCHAR {
    let Text: PCHAR;
    if MinorFunction as i32 >= IRP_MN_QUERY_POWER as i32 {
        Text = b"** UNKNOWN PO IRP Minor Code **\0".as_ptr() as *const i8 as PCHAR;
    } else {
        Text = PoCodes[MinorFunction as usize];
    }
    Text
}
#[no_mangle]
pub unsafe extern "system" fn PciDebugIrpDispatchDisplay(
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_FDO_EXTENSION,
    MaxMinor: USHORT,
) -> BOOLEAN {
    let mut PdoDeviceExtension: PPCI_PDO_EXTENSION = core::ptr::null_mut();
    let BreakMask: ULONG;
    let IrpString: PCHAR;

    match (*IoStackLocation).MajorFunction {
        IRP_MJ_POWER => {
            BreakMask = if (*DeviceExtension).ExtensionType == PciPdoExtensionType as u32 {
                PciBreakOnPdoPowerIrp
            } else {
                PciBreakOnFdoPowerIrp
            };
            IrpString = PciDebugPoIrpTypeToText((*IoStackLocation).MinorFunction as USHORT);
        }
        IRP_MJ_PNP => {
            BreakMask = if (*DeviceExtension).ExtensionType == PciFdoExtensionType as u32 {
                PciBreakOnPdoPnpIrp
            } else {
                PciBreakOnFdoPnpIrp
            };
            IrpString = PciDebugPnpIrpTypeToText((*IoStackLocation).MinorFunction as USHORT);
        }
        _ => {
            BreakMask = 0u32;
            IrpString = b"\0".as_ptr() as PCHAR;
        }
    }

    if (*DeviceExtension).ExtensionType == PciPdoExtensionType as u32 {
        PdoDeviceExtension = DeviceExtension as PPCI_PDO_EXTENSION;
        if DbgPrint(
            b"(%s:%d) PDO(b=0x%x, d=0x%x, f=0x%x)<-%s\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/debug.c\0".as_ptr()
                as *const i8,
            179i32,
            (*(*PdoDeviceExtension).ParentFdoExtension).BaseBus as i32,
            (*PdoDeviceExtension).Slot.u.bits.DeviceNumber() as i32,
            (*PdoDeviceExtension).Slot.u.bits.FunctionNumber() as i32,
            IrpString,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"drivers/bus/pcix/debug.c\0".as_ptr()
                    as *const i8,
                179i32,
            );
        }
    } else if (*DeviceExtension).ExtensionType == PciFdoExtensionType as u32 {
        if DbgPrint(
            b"(%s:%d) FDO(%p)<-%s\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/debug.c\0".as_ptr()
                as *const i8,
            194i32,
            DeviceExtension,
            IrpString,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"drivers/bus/pcix/debug.c\0".as_ptr()
                    as *const i8,
                194i32,
            );
        }
    }

    if (*IoStackLocation).MinorFunction as i32 > MaxMinor as i32 {
        if DbgPrint(
            b"(%s:%d) Unknown IRP, minor = 0x%x\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/debug.c\0".as_ptr()
                as *const i8,
            199i32,
            (*IoStackLocation).MinorFunction as i32,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"drivers/bus/pcix/debug.c\0".as_ptr()
                    as *const i8,
                199i32,
            );
        }
    }

    (((1u32 << (*IoStackLocation).MinorFunction as u32) & BreakMask) as BOOLEAN)
}
#[no_mangle]
pub unsafe extern "system" fn PciDebugDumpCommonConfig(PciData: PPCI_COMMON_HEADER) {
    let mut i: u16 = 0;
    while (i as i32) < PCI_COMMON_HDR_LENGTH {
        let value_ptr = (PciData as *const u8).add(i as usize) as *const u32;
        let value: u32 = *value_ptr;
        let status = DbgPrint(
            b"(%s:%d) %02x - %08x\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/debug.c\0".as_ptr() as *const i8,
            215i32,
            i as i32,
            value,
        );
        if status != 0 {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"drivers/bus/pcix/debug.c\0".as_ptr() as *const i8,
                215i32,
            );
        }
        i = (i as i32 + 4) as u16;
    }
}

#[no_mangle]
pub unsafe extern "system" fn PciDebugDumpQueryCapabilities(DeviceCaps: PDEVICE_CAPABILITIES) {
    let mut i: u32;
    if DbgPrint(
        b"(%s:%d) Capabilities\n  Lock:%u, Eject:%u, Remove:%u, Dock:%u, UniqueId:%u\n\0".as_ptr() as PCSTR,
        b"drivers/bus/pcix/debug.c\0".as_ptr() as *const i8,
        231i32,
        (*DeviceCaps).LockSupported() as i32,
        (*DeviceCaps).EjectSupported() as i32,
        (*DeviceCaps).Removable() as i32,
        (*DeviceCaps).DockDevice() as i32,
        (*DeviceCaps).UniqueID() as i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/debug.c\0".as_ptr() as *const i8,
            231i32,
        );
    }
    DbgPrint(
        b"  SilentInstall:%u, RawOk:%u, SurpriseOk:%u\n\0".as_ptr() as PCSTR,
        (*DeviceCaps).SilentInstall() as i32,
        (*DeviceCaps).RawDeviceOK() as i32,
        (*DeviceCaps).SurpriseRemovalOK() as i32,
    );
    DbgPrint(
        b"  Address %08x, UINumber %08x, Latencies D1 %u, D2 %u, D3 %u\n\0".as_ptr() as PCSTR,
        (*DeviceCaps).Address,
        (*DeviceCaps).UINumber,
        (*DeviceCaps).D1Latency,
        (*DeviceCaps).D2Latency,
        (*DeviceCaps).D3Latency,
    );
    DbgPrint(
        b"  System Wake: %s, Device Wake: %s\n  DeviceState[PowerState] [\0".as_ptr() as PCSTR,
        SystemPowerStates[if ((*DeviceCaps).SystemWake as u32) < (PowerSystemMaximum as u32) {
            (*DeviceCaps).SystemWake as usize
        } else {
            PowerSystemMaximum as usize
        }],
        DevicePowerStates[if ((*DeviceCaps).DeviceWake as u32) < (PowerDeviceMaximum as u32) {
            (*DeviceCaps).DeviceWake as usize
        } else {
            PowerDeviceMaximum as usize
        }],
    );
    i = PowerSystemWorking as u32;
    while i < PowerSystemMaximum as u32 {
        DbgPrint(
            b" %s\0".as_ptr() as PCSTR,
            DevicePowerStates[(*DeviceCaps).DeviceState[i as usize] as usize],
        );
        i = i.wrapping_add(1);
    }
    DbgPrint(b" ]\n\0".as_ptr() as PCSTR);
}

#[no_mangle]
pub unsafe extern "system" fn PciDebugCmResourceTypeToText(Type: u8) -> PCHAR {
    // CmResourceType* consts missing from crate::glue (xdk/cmtypes.h values).
    // Without these in scope, the match arms below would become silent bindings
    // (catch-alls) — a logic bug. Defined here so each arm is a real const pattern.
    const CmResourceTypeDeviceSpecific: u32 = 5;
    const CmResourceTypeDma: u32 = 4;
    const CmResourceTypeConfigData: u32 = 128;
    const CmResourceTypeDevicePrivate: u32 = 129;
    const CmResourceTypePcCardConfig: u32 = 130;
    match Type as u32 {
        CmResourceTypeDeviceSpecific => {
            b"CmResourceTypeDeviceSpecific\0".as_ptr() as *const i8 as PCHAR
        }
        CmResourceTypePort => b"CmResourceTypePort\0".as_ptr() as *const i8 as PCHAR,
        CmResourceTypeInterrupt => {
            b"CmResourceTypeInterrupt\0".as_ptr() as *const i8 as PCHAR
        }
        CmResourceTypeMemory => b"CmResourceTypeMemory\0".as_ptr() as *const i8 as PCHAR,
        CmResourceTypeDma => b"CmResourceTypeDma\0".as_ptr() as *const i8 as PCHAR,
        CmResourceTypeBusNumber => {
            b"CmResourceTypeBusNumber\0".as_ptr() as *const i8 as PCHAR
        }
        CmResourceTypeConfigData => {
            b"CmResourceTypeConfigData\0".as_ptr() as *const i8 as PCHAR
        }
        CmResourceTypeDevicePrivate => {
            b"CmResourceTypeDevicePrivate\0".as_ptr() as *const i8 as PCHAR
        }
        CmResourceTypePcCardConfig => {
            b"CmResourceTypePcCardConfig\0".as_ptr() as *const i8 as PCHAR
        }
        crate::glue::CmResourceTypeMemoryLarge => {
            b"CmResourceTypeMemoryLarge\0".as_ptr() as *const i8 as PCHAR
        }
        _ => b"*** INVALID RESOURCE TYPE ***\0".as_ptr() as *const i8 as PCHAR,
    }
}
#[no_mangle]
pub unsafe extern "system" fn PciDebugPrintIoResource(Descriptor: PIO_RESOURCE_DESCRIPTOR) {
    DbgPrint(
        b"(%s:%d)      IoResource Descriptor dump:  Descriptor @0x%p\n\0" as *const u8 as *const i8,
        b"drivers/bus/pcix/debug.c\0" as *const u8
            as *const ::core::ffi::c_char,
        285 as ::core::ffi::c_int,
        Descriptor,
    );
    DbgPrint(
        b"(%s:%d)         Option           = 0x%x\n\0" as *const u8 as *const i8,
        b"drivers/bus/pcix/debug.c\0" as *const u8
            as *const ::core::ffi::c_char,
        286 as ::core::ffi::c_int,
        (*Descriptor).Option as ::core::ffi::c_int,
    );
    DbgPrint(
        b"(%s:%d)         Type             = %u (%s)\n\0" as *const u8 as *const i8,
        b"drivers/bus/pcix/debug.c\0" as *const u8
            as *const ::core::ffi::c_char,
        287 as ::core::ffi::c_int,
        (*Descriptor).Type as ::core::ffi::c_int,
        PciDebugCmResourceTypeToText((*Descriptor).Type),
    );
    DbgPrint(
        b"(%s:%d)         ShareDisposition = %u\n\0" as *const u8 as *const i8,
        b"drivers/bus/pcix/debug.c\0" as *const u8
            as *const ::core::ffi::c_char,
        288 as ::core::ffi::c_int,
        (*Descriptor).ShareDisposition as ::core::ffi::c_int,
    );
    DbgPrint(
        b"(%s:%d)         Flags            = 0x%04X\n\0" as *const u8 as *const i8,
        b"drivers/bus/pcix/debug.c\0" as *const u8
            as *const ::core::ffi::c_char,
        289 as ::core::ffi::c_int,
        (*Descriptor).Flags as ::core::ffi::c_int,
    );

    let Data = core::ptr::addr_of_mut!((*Descriptor).u.DevicePrivate) as PULONG;
    let mut i: u32 = 0;
    while i < 6 {
        DbgPrint(
            b"(%s:%d)         Data[%u] = %08x  %08x  %08x\n\0" as *const u8 as *const i8,
            b"drivers/bus/pcix/debug.c\0" as *const u8
                as *const ::core::ffi::c_char,
            296 as ::core::ffi::c_int,
            i,
            *Data.offset(0),
            *Data.offset(1),
            *Data.offset(2),
        );
        i += 3;
    }
}
#[no_mangle]
pub unsafe extern "system" fn PciDebugPrintIoResReqList(
    Requirements: PIO_RESOURCE_REQUIREMENTS_LIST,
) {
    if Requirements.is_null() {
        return;
    }
    let mut AlternativeLists: u32 = (*Requirements).AlternativeLists;
    let mut List: PIO_RESOURCE_LIST =
        core::ptr::addr_of_mut!((*Requirements).List) as PIO_RESOURCE_LIST;
    let mut Count: u32;
    let mut Descriptor: PIO_RESOURCE_DESCRIPTOR;

    if DbgPrint(
        b"(%s:%d)   IO_RESOURCE_REQUIREMENTS_LIST (PCI Bus Driver)\n\0".as_ptr() as PCSTR,
        b"drivers/bus/pcix/debug.c\0".as_ptr()
            as *const i8,
        317i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/debug.c\0".as_ptr()
                as *const i8,
            317i32,
        );
    }
    if DbgPrint(
        b"(%s:%d)      InterfaceType        %d\n\0".as_ptr() as PCSTR,
        b"drivers/bus/pcix/debug.c\0".as_ptr()
            as *const i8,
        318i32,
        (*Requirements).InterfaceType as i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/debug.c\0".as_ptr()
                as *const i8,
            318i32,
        );
    }
    if DbgPrint(
        b"(%s:%d)      BusNumber            0x%x\n\0".as_ptr() as PCSTR,
        b"drivers/bus/pcix/debug.c\0".as_ptr()
            as *const i8,
        319i32,
        (*Requirements).BusNumber,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/debug.c\0".as_ptr()
                as *const i8,
            319i32,
        );
    }
    let slot_ptr: *const PCI_SLOT_NUMBER =
        core::ptr::addr_of!((*Requirements).SlotNumber) as *const PCI_SLOT_NUMBER;
    let slot = &*slot_ptr;
    if DbgPrint(
        b"(%s:%d)      SlotNumber           %d (0x%x), (d/f = 0x%x/0x%x)\n\0".as_ptr() as PCSTR,
        b"drivers/bus/pcix/debug.c\0".as_ptr()
            as *const i8,
        324i32,
        (*Requirements).SlotNumber,
        (*Requirements).SlotNumber,
        slot.u.bits.DeviceNumber() as i32,
        slot.u.bits.FunctionNumber() as i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/debug.c\0".as_ptr()
                as *const i8,
            324i32,
        );
    }
    if DbgPrint(
        b"(%s:%d)      AlternativeLists     %u\n\0".as_ptr() as PCSTR,
        b"drivers/bus/pcix/debug.c\0".as_ptr()
            as *const i8,
        325i32,
        AlternativeLists,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/debug.c\0".as_ptr()
                as *const i8,
            325i32,
        );
    }
    loop {
        let fresh0 = AlternativeLists;
        AlternativeLists = AlternativeLists.wrapping_sub(1);
        if fresh0 == 0 {
            break;
        }
        Descriptor = core::ptr::addr_of_mut!((*List).Descriptors) as PIO_RESOURCE_DESCRIPTOR;
        Count = (*List).Count;
        if DbgPrint(
            b"(%s:%d) \n     List[%u].Count = %u\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/debug.c\0".as_ptr()
                as *const i8,
            335i32,
            AlternativeLists,
            Count,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"drivers/bus/pcix/debug.c\0".as_ptr()
                    as *const i8,
                335i32,
            );
        }
        loop {
            let fresh1 = Count;
            Count = Count.wrapping_sub(1);
            if fresh1 == 0 {
                break;
            }
            let fresh2 = Descriptor;
            Descriptor = Descriptor.offset(1);
            PciDebugPrintIoResource(fresh2);
        }
        List = Descriptor as PIO_RESOURCE_LIST;
    }
    if DbgPrint(
        b"(%s:%d) \n\0".as_ptr() as PCSTR,
        b"drivers/bus/pcix/debug.c\0".as_ptr()
            as *const i8,
        343i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/debug.c\0".as_ptr()
                as *const i8,
            343i32,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn PciDebugPrintPartialResource(
    PartialResource: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) {
    if DbgPrint(
        b"(%s:%d)      Partial Resource Descriptor @0x%p\n\0".as_ptr() as PCSTR,
        b"drivers/bus/pcix/debug.c\0".as_ptr() as PCSTR,
        351i32,
        PartialResource,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/debug.c\0".as_ptr() as PCSTR,
            351i32,
        );
    }
    if DbgPrint(
        b"(%s:%d)         Type             = %u (%s)\n\0".as_ptr() as PCSTR,
        b"drivers/bus/pcix/debug.c\0".as_ptr() as PCSTR,
        352i32,
        (*PartialResource).Type as i32,
        PciDebugCmResourceTypeToText((*PartialResource).Type),
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/debug.c\0".as_ptr() as PCSTR,
            352i32,
        );
    }
    if DbgPrint(
        b"(%s:%d)         ShareDisposition = %u\n\0".as_ptr() as PCSTR,
        b"drivers/bus/pcix/debug.c\0".as_ptr() as PCSTR,
        353i32,
        (*PartialResource).ShareDisposition as i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/debug.c\0".as_ptr() as PCSTR,
            353i32,
        );
    }
    if DbgPrint(
        b"(%s:%d)         Flags            = 0x%04X\n\0".as_ptr() as PCSTR,
        b"drivers/bus/pcix/debug.c\0".as_ptr() as PCSTR,
        354i32,
        (*PartialResource).Flags as i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/debug.c\0".as_ptr() as PCSTR,
            354i32,
        );
    }
    if DbgPrint(
        b"(%s:%d)         Data[%d] = %08x  %08x  %08x\n\0".as_ptr() as PCSTR,
        b"drivers/bus/pcix/debug.c\0".as_ptr() as PCSTR,
        359i32,
        0i32,
        (*PartialResource).u.Generic.Start.__bindgen_anon_1.LowPart,
        (*PartialResource).u.Generic.Start.__bindgen_anon_1.HighPart,
        (*PartialResource).u.Generic.Length,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/debug.c\0".as_ptr() as PCSTR,
            359i32,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn PciDebugPrintCmResList(PartialList: PCM_RESOURCE_LIST) {
    if PartialList.is_null() {
        return;
    }
    let ListCount = (*PartialList).Count;
    let mut FullDescriptor: PCM_FULL_RESOURCE_DESCRIPTOR =
        &raw mut (*PartialList).List as *mut CM_FULL_RESOURCE_DESCRIPTOR
            as PCM_FULL_RESOURCE_DESCRIPTOR;
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;

    DbgPrint(
        b"  CM_RESOURCE_LIST (PCI Bus Driver) (List Count = %u)\n\0".as_ptr() as *const u8
            as PCSTR,
        ListCount,
    );

    let mut i: ULONG = 0;
    while i < ListCount {
        DbgPrint(
            b"     InterfaceType        %d\n\0".as_ptr() as *const u8 as PCSTR,
            (*FullDescriptor).InterfaceType as i32,
        );
        DbgPrint(
            b"     BusNumber            0x%x\n\0".as_ptr() as *const u8 as PCSTR,
            (*FullDescriptor).BusNumber,
        );

        let mut Count = (*FullDescriptor).PartialResourceList.Count;
        PartialDescriptor = &raw mut (*FullDescriptor).PartialResourceList.PartialDescriptors
            as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR
            as PCM_PARTIAL_RESOURCE_DESCRIPTOR;
        while Count != 0 {
            PciDebugPrintPartialResource(PartialDescriptor);
            Count -= 1;
            PartialDescriptor = CmiGetNextPartialDescriptor(PartialDescriptor);
        }

        FullDescriptor = PartialDescriptor as PCM_FULL_RESOURCE_DESCRIPTOR;

        i += 1;
    }

    DbgPrint(b"\n\0".as_ptr() as *const u8 as PCSTR);
}
